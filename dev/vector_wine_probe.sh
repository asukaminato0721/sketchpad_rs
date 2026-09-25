#!/usr/bin/env bash
set -euo pipefail

if (( $# != 1 )); then
  echo "Usage: dev/vector_wine_probe.sh SKETCH.gsp" >&2
  exit 2
fi
sketch_path="$(realpath "$1")"
exe_path="$(realpath GSP5Chs.exe)"
capture_prefix="${SKETCHPAD_VECTOR_CAPTURE_PREFIX:-/tmp/sketchpad_vector_probe}"
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
  import -window root "${3}_opened.png"
  xdotool mousemove 209 189 click 1
  xdotool keydown Shift
  xdotool mousemove 279 199 click 1
  xdotool keyup Shift
  xdotool mousemove 220 40 click 1
  sleep 0.3
  xdotool mousemove 310 129 click 1
  sleep 0.7
  import -window root "${3}_after_mark.png"
  xdotool key Escape
  xdotool mousemove "${SKETCHPAD_VECTOR_SOURCE_X:-159}" "${SKETCHPAD_VECTOR_SOURCE_Y:-269}" click 1
  xdotool mousemove 220 40 click 1
  sleep 0.3
  import -window root "${3}_transform_menu.png"
  xdotool mousemove 270 171 click 1
  sleep 0.8
  import -window root "${3}_translate_dialog.png"
  xdotool mousemove 351 320 click 1
  sleep 0.8
  import -window root "${3}_translated.png"
  xdotool key --clearmodifiers ctrl+s
  sleep 2
  echo "Saved copy: $2 ($(stat -c %s "$2") bytes)"
' _ "$exe_path" "$sketch_path" "$capture_prefix"
