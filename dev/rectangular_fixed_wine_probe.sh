#!/usr/bin/env bash
set -euo pipefail

if (( $# != 1 )); then
  echo "Usage: dev/rectangular_fixed_wine_probe.sh SKETCH.gsp" >&2
  exit 2
fi
sketch_path="$(realpath "$1")"
exe_path="$(realpath GSP5Chs.exe)"
capture_prefix="${SKETCHPAD_RECT_FIXED_CAPTURE_PREFIX:-/tmp/sketchpad_rectangular_fixed_probe}"
exec 9>/tmp/sketchpad_rs_wine.lock
flock 9

xvfb-run -a bash -c '
  set -euo pipefail
  export WINEPREFIX="${SKETCHPAD_WINEPREFIX:-/tmp/sketchpad_rs_wine}"
  wine "$1" "$2" > "${3}_runtime.log" 2>&1 &
  wine_pid=$!
  trap '\''kill "$wine_pid" 2>/dev/null || true'\'' EXIT
  sleep 6
  window_id="$(xdotool search --onlyvisible --name "Sketchpad|几何" | tail -1)"
  xdotool windowfocus "$window_id"
  xdotool key --clearmodifiers ctrl+o
  sleep 1
  wine_path="$(winepath -w "$2" | tr -d "\r")"
  xdotool type --clearmodifiers --delay 1 "$wine_path"
  xdotool key Return
  sleep 3
  [[ "$(xdotool getwindowname "$window_id")" == *"$(basename "$2")"* ]]
  xdotool mousemove 159 279 click 1
  xdotool mousemove 220 40 click 1
  sleep 0.3
  xdotool mousemove 275 171 click 1
  sleep 0.8
  import -window root "${3}_initial.png"
  xdotool mousemove 196 71 click 1
  sleep 0.3
  import -window root "${3}_rectangular.png"
  xdotool mousemove 112 125 click 1
  xdotool mousemove 112 228 click 1
  sleep 0.3
  import -window root "${3}_fixed.png"
  if [[ -n "${SKETCHPAD_RECT_FIXED_X:-}" ]]; then
    xdotool mousemove 221 162 click 1
    xdotool key --clearmodifiers ctrl+a
    printf '%s' "$SKETCHPAD_RECT_FIXED_X" | xdotool type --clearmodifiers --delay 20 --file -
  fi
  if [[ -n "${SKETCHPAD_RECT_FIXED_Y:-}" ]]; then
    xdotool mousemove 221 264 click 1
    xdotool key --clearmodifiers ctrl+a
    printf '%s' "$SKETCHPAD_RECT_FIXED_Y" | xdotool type --clearmodifiers --delay 20 --file -
  fi
  import -window root "${3}_values.png"
  xdotool mousemove 350 321 click 1
  sleep 0.8
  import -window root "${3}_translated.png"
  xdotool key --clearmodifiers ctrl+s
  sleep 2
  echo "Saved copy: $2 ($(stat -c %s "$2") bytes)"
' _ "$exe_path" "$sketch_path" "$capture_prefix"
