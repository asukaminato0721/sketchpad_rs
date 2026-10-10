#!/usr/bin/env python3
"""Check MCP against a real editor window on an isolated X11 display."""

import json
import math
import os
from pathlib import Path
import select
import subprocess
import sys
import tempfile
import time
import xml.etree.ElementTree as ET


class Client:
    def __init__(self, binary, document="--demo"):
        env = os.environ.copy()
        env.pop("WAYLAND_DISPLAY", None)
        env.pop("WAYLAND_SOCKET", None)
        env.update(
            XDG_SESSION_TYPE="x11",
            WINIT_UNIX_BACKEND="x11",
            WGPU_BACKEND="gl",
            LIBGL_ALWAYS_SOFTWARE="1",
        )
        self.log = tempfile.TemporaryFile()
        self.process = subprocess.Popen(
            [str(binary), "--mcp", str(document)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=self.log,
            env=env,
            bufsize=0,
        )
        self.serial = 0

    def send(self, message):
        self.process.stdin.write((json.dumps(message) + "\n").encode())
        self.process.stdin.flush()

    def request(self, method, params=None):
        self.serial += 1
        self.send(
            {"jsonrpc": "2.0", "id": self.serial, "method": method, "params": params or {}}
        )
        if not select.select([self.process.stdout], [], [], 30)[0]:
            raise AssertionError(f"No response to {method}")
        line = self.process.stdout.readline()
        assert line, f"Editor closed before response to {method}"
        response = json.loads(line)
        assert response["id"] == self.serial, response
        return response

    def initialize(self):
        result = self.request(
            "initialize",
            {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "sketchpad-mcp-smoke", "version": "1"},
            },
        )["result"]
        assert result["protocolVersion"] == "2025-11-25"
        self.send({"jsonrpc": "2.0", "method": "notifications/initialized"})

    def tool(self, name, arguments=None):
        response = self.request("tools/call", {"name": name, "arguments": arguments or {}})
        assert "error" not in response, response
        result = response["result"]
        assert not result["isError"], result
        assert json.loads(result["content"][0]["text"]) == result["structuredContent"]
        return result["structuredContent"]

    def close(self):
        if self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=5)
        self.process.stdin.close()
        self.process.stdout.close()
        self.log.close()

    def print_log(self):
        self.log.seek(0)
        sys.stderr.write(self.log.read().decode(errors="replace"))


def wait_for(probe, description):
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        value = probe()
        if value:
            return value
        time.sleep(0.05)
    raise AssertionError(f"Timed out: {description}")


def xdotool(*args):
    return subprocess.check_output(["xdotool", *map(str, args)], timeout=10).decode().strip()


def check_window(client):
    client.initialize()
    tools = client.request("tools/list")["result"]["tools"]
    assert {tool["name"] for tool in tools} == {
        "get_state", "list_objects", "inspect_object", "select_objects",
        "move_point", "set_parameter", "undo", "redo", "export_svg", "create_point", "construct",
    }
    wait_for(lambda: client.tool("get_state")["viewport"]["rect"], "first canvas frame")
    window = xdotool("search", "--sync", "--onlyvisible", "--pid", client.process.pid).splitlines()[-1]
    first = client.tool("list_objects", {"limit": 2})
    assert len(first["objects"]) == 2 and first["next_offset"] == 2
    a = client.tool("inspect_object", {"id": 1})
    b = client.tool("inspect_object", {"id": 2})
    before_svg = client.tool("export_svg")["svg"]

    moved = {"x": a["position"]["x"] + 40, "y": a["position"]["y"] + 20}
    result = client.tool("move_point", {"id": 1, **moved})
    assert result["position"] == moved
    midpoint = client.tool("inspect_object", {"id": 5})["position"]
    for axis in ("x", "y"):
        assert math.isclose(midpoint[axis], (moved[axis] + b["position"][axis]) / 2)
    wait_for(lambda: xdotool("getwindowname", window).startswith("*"), "dirty window title")
    svg = client.tool("export_svg")["svg"]
    assert svg != before_svg
    assert ET.fromstring(svg).tag == "{http://www.w3.org/2000/svg}svg"

    # Invalid edits must preserve both geometry and history.
    state = client.tool("get_state")
    rejected = client.request("tools/call", {"name": "move_point", "arguments": {"id": 0, "x": 1, "y": 2}})
    assert rejected["result"]["isError"]
    assert client.tool("get_state")["undo_steps"] == state["undo_steps"]
    assert client.tool("inspect_object", {"id": 1})["position"] == moved
    unknown = client.request("tools/call", {"name": "missing"})
    assert unknown["error"]["code"] == -32602

    # Drag the same point through the window. MCP must observe the GUI edit.
    viewport = state["viewport"]
    rect = viewport["rect"]
    x = round(rect["x"] + rect["width"] / 2 + (moved["x"] - viewport["center"]["x"]) * viewport["scale"])
    y = round(rect["y"] + rect["height"] / 2 + (moved["y"] - viewport["center"]["y"]) * viewport["scale"])
    xdotool("windowfocus", "--sync", window)
    xdotool("mousemove", "--window", window, x, y, "sleep", "0.1", "mousedown", "1", "sleep", "0.1",
            "mousemove", "--window", window, x + 35, y - 25, "sleep", "0.2", "mouseup", "1")
    wait_for(lambda: client.tool("inspect_object", {"id": 1})["position"] != moved, "GUI drag visible through MCP")
    wait_for(lambda: client.tool("get_state")["dragging"] is None, "GUI drag release")
    dragged = client.tool("inspect_object", {"id": 1})["position"]
    client.tool("undo")
    assert client.tool("inspect_object", {"id": 1})["position"] == moved
    client.tool("redo")
    assert client.tool("inspect_object", {"id": 1})["position"] == dragged
    client.tool("select_objects", {"ids": [2, 1]})
    assert client.tool("get_state")["selection_order"] == [2, 1]
    # A construction and its helper objects share one undo step.
    before = client.tool("get_state")
    point = client.tool("create_point", {"x": 220, "y": 180, "label": "新点"})["created_ids"][0]
    created = client.tool("construct", {"kind": "midpoint", "inputs": [1, point]})
    assert len(created["created_ids"]) == 2
    midpoint_id = created["selected_ids"][0]
    assert client.tool("get_state")["undo_steps"] == before["undo_steps"] + 2
    assert client.tool("inspect_object", {"id": midpoint_id})["position"]
    client.tool("undo")
    client.tool("undo")
    assert client.tool("get_state")["object_count"] == before["object_count"]
    assert client.tool("get_state")["selected"] == []
    client.tool("redo")
    client.tool("redo")
    assert client.tool("inspect_object", {"id": midpoint_id})["position"]
    client.process.stdin.close()
    time.sleep(0.2)
    assert client.process.poll() is None, "Dirty editor must show its save confirmation"


def main():
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/sketchpad_rs").resolve()
    client = Client(binary)
    try:
        check_window(client)
    except BaseException:
        client.print_log()
        raise
    finally:
        client.close()

    client = Client(binary)
    try:
        client.initialize()
        assert not client.tool("get_state")["dirty"]
        client.process.stdin.close()
        assert client.process.wait(timeout=10) == 0
    except BaseException:
        client.print_log()
        raise
    finally:
        client.close()
    print("MCP window smoke passed: handshake, live edits, GUI drag, undo/redo, SVG, errors, and shutdown")


if __name__ == "__main__":
    main()
