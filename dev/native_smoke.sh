#!/usr/bin/env bash
set -euo pipefail

# Capture the native window on an isolated X11 display with software rendering.
# Usage: dev/native_smoke.sh [sketch.gsp] [capture-prefix]
sketch_path="${1:-tests/fixtures/point.gsp}"
capture_prefix="${2:-/tmp/sketchpad_native_smoke}"
sketch_path="$(realpath "$sketch_path")"

xvfb-run -a bash -c '
  set -euo pipefail
  unset WAYLAND_DISPLAY
  export XDG_SESSION_TYPE=x11
  export WGPU_BACKEND=gl
  export LIBGL_ALWAYS_SOFTWARE=1
  target/debug/sketchpad_rs "$1" > "${2}_runtime.log" 2>&1 &
  app_pid=$!
  trap '\''kill "$app_pid" 2>/dev/null || true'\'' EXIT
  sleep 6
  window_id="$(xdotool search --onlyvisible --name "Sketchpad" | tail -1)"
  test -n "$window_id"
  echo "Native window: $(xdotool getwindowname "$window_id")"
  import -window root "${2}.png"
' _ "$sketch_path" "$capture_prefix"
