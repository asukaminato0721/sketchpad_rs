#!/usr/bin/env python3
"""Check language switching, persistence, and toolbar input in a real window."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

from mcp_smoke import Client, wait_for, xdotool


def capture(window, language):
    subprocess.run(["import", "-window", window, f"/tmp/sketchpad-ui-{language}.png"], check=True)


def main():
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/sketchpad_rs").resolve()
    with tempfile.TemporaryDirectory(prefix="sketchpad-ui-") as config:
        os.environ["XDG_CONFIG_HOME"] = config
        folder = Path(config) / "sketchpad_rs"
        folder.mkdir()
        preference = folder / "language.json"
        preference.write_text('"zh-CN"')
        client = Client(binary)
        try:
            client.initialize()
            wait_for(lambda: client.tool("get_state")["viewport"]["rect"], "canvas")
            assert client.tool("get_state")["ui_language"] == "zh-CN"
            window = xdotool("search", "--sync", "--onlyvisible", "--pid", client.process.pid).splitlines()[-1]
            time.sleep(0.5)
            capture(window, "zh")
            # Chinese Window > Language > English, at the default UI scale.
            xdotool("mousemove", "--window", window, 310, 12, "click", 1)
            xdotool("mousemove", "--window", window, 340, 94, "sleep", "0.5")
            xdotool("mousemove", "--window", window, 414, 94, "click", 1)
            wait_for(lambda: client.tool("get_state")["ui_language"] == "en", "language change")
            assert json.loads(preference.read_text()) == "en"
            assert not client.tool("get_state")["dirty"]
            time.sleep(0.3)
            capture(window, "en")
            # The point icon must select the real drawing tool.
            before = client.tool("get_state")["object_count"]
            xdotool("mousemove", "--window", window, 22, 78, "click", 1)
            wait_for(lambda: client.tool("get_state")["tool"] == "point", "point tool")
            xdotool("mousemove", "--window", window, 800, 200, "click", 1)
            wait_for(lambda: client.tool("get_state")["object_count"] == before + 1, "new point")
            client.tool("undo")
            assert client.tool("get_state")["object_count"] == before
        except BaseException:
            client.print_log()
            raise
        finally:
            client.close()
        client = Client(binary)
        try:
            client.initialize()
            assert client.tool("get_state")["ui_language"] == "en"
            assert not client.tool("get_state")["dirty"]
        except BaseException:
            client.print_log()
            raise
        finally:
            client.close()
    print("UI smoke passed: Chinese, menu switch, saved preference, restart, point tool, and undo")


if __name__ == "__main__":
    main()
