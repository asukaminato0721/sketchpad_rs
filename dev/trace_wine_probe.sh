#!/usr/bin/env bash
set -euo pipefail

# Run against a disposable copy; shared Wine state and screenshots are serialized.
exec 9>/tmp/sketchpad_rs_wine.lock
flock 9
if (( $# != 2 )); then
  echo "Usage: $0 COPY_OF_TRACED_POINT.gsp CAPTURE_PREFIX" >&2
  exit 2
fi
sketch_path="$(realpath "$1")"
capture_prefix="$2"
exe_path="$(realpath GSP5Chs.exe)"
xvfb-run -a bash -c '
  set -euo pipefail
  export WINEPREFIX="${SKETCHPAD_WINEPREFIX:-/tmp/sketchpad_rs_wine}"
  wine "$1" "$2" > "$3.runtime.log" 2>&1 &
  wine_pid=$!
  trap '\''kill "$wine_pid" 2>/dev/null || true'\'' EXIT
  window_id=""
  for _ in $(seq 1 20); do
    window_id="$(xdotool search --onlyvisible --name "Sketchpad|几何" 2>/dev/null | tail -1 || true)"
    [[ -n "$window_id" ]] && break
    sleep 1
  done
  test -n "$window_id"
  test -n "$window_id"
  xdotool windowfocus "$window_id"
  xdotool key --clearmodifiers ctrl+o
  sleep 1
  wine_path="$(winepath -w "$2" | tr -d "\r")"
  xdotool type --clearmodifiers --delay 1 "$wine_path"
  xdotool key Return
  sleep 3
  [[ "$(xdotool getwindowname "$window_id")" == *"$(basename "$2")"* ]]
  # The original free point is at screen (260,220). Keep the button down and
  # move through many positions so the application paints a visible trace.
  xdotool mousemove 260 220 mousedown 1
  for step in $(seq 1 15); do
    xdotool mousemove "$((260 + step * 6))" "$((220 + step * 3))"
    sleep 0.08
  done
  xdotool mouseup 1
  import -window root "$3.trail.png"
  xdotool mousemove 144 40 click 1
  sleep 0.2
  import -window root "$3.menu.png"
  xdotool mousemove 180 240 click 1
  sleep 0.2
  import -window root "$3.erased.png"
  xdotool key --clearmodifiers ctrl+s
  sleep 2
  echo "Saved: $2 ($(stat -c %s "$2") bytes)"
' _ "$exe_path" "$sketch_path" "$capture_prefix"
