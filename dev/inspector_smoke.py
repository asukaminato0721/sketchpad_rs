#!/usr/bin/env python3
"""Check inspector responsiveness and selection with a supplied GSP file."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

from mcp_smoke import Client, wait_for, xdotool


def main():
    if len(sys.argv) < 2:
        raise SystemExit("Usage: inspector_smoke.py FILE.gsp [EDITOR_BINARY]")
    document = Path(sys.argv[1]).resolve()
    binary = Path(sys.argv[2] if len(sys.argv) > 2 else "target/debug/sketchpad_rs").resolve()
    with tempfile.TemporaryDirectory(prefix="sketchpad-inspector-") as config:
        os.environ["XDG_CONFIG_HOME"] = config
        folder = Path(config) / "sketchpad_rs"
        folder.mkdir()
        (folder / "language.json").write_text('"zh-CN"')
        client = Client(binary, document)
        try:
            client.initialize()
            wait_for(lambda: client.tool("get_state")["viewport"]["rect"], "canvas")
            window = xdotool("search", "--sync", "--onlyvisible", "--pid", client.process.pid).splitlines()[-1]
            client.tool("select_objects", {"ids": []})
            before = client.tool("get_state")
            first = client.tool("list_objects", {"limit": 1})["objects"][0]["id"]
            time.sleep(0.3)
            # Open Object Information through the tool strip.
            xdotool("mousemove", "--window", window, 15, 258, "click", 1)
            for index in range(3):
                time.sleep(0.2)
                start = time.monotonic()
                client.tool("get_state")
                elapsed = time.monotonic() - start
                print(f"Inspector response {index}: {elapsed:.3f}s", flush=True)
                assert elapsed < 2, "Inspector blocks the editor event loop"
            # Select the first row, then check the document's original object ID.
            xdotool("mousemove", "--window", window, 110, 175, "click", 1)
            after = wait_for(
                lambda: (state if (state := client.tool("get_state"))["selected"] == [first] else None),
                "tree selection",
            )
            assert after["undo_steps"] == before["undo_steps"]
            assert after["dirty"] == before["dirty"]
            subprocess.run(["import", "-window", window, "/tmp/sketchpad-inspector-sample.png"], check=True)
        except BaseException:
            client.print_log()
            raise
        finally:
            client.close()
    print("Inspector smoke passed: responsive window, original object selection, unchanged undo history")


if __name__ == "__main__":
    main()
