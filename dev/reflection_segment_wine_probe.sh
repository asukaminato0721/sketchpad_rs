#!/usr/bin/env bash
set -euo pipefail

if (( $# != 1 )); then
  echo "Usage: dev/reflection_segment_wine_probe.sh SKETCH.gsp" >&2
  exit 2
fi
sketch_path="$(realpath "$1")"
exe_path="$(realpath GSP5Chs.exe)"
capture_prefix="${SKETCHPAD_REFLECTION_CAPTURE_PREFIX:-/tmp/sketchpad_reflection_segment_probe}"
exec 9>/tmp/sketchpad_rs_wine.lock
flock 9

xvfb-run -a bash -c '
  set -euo pipefail
  export WINEPREFIX="${SKETCHPAD_WINEPREFIX:-/tmp/sketchpad_rs_wine}"
  wine "$1" "$2" > "${3}_runtime.log" 2>&1 &
  wine_pid=$!
  trap '\''kill "$wine_pid" 2>/dev/null || true'\'' EXIT
  window_id=""
  for _ in $(seq 1 20); do
    window_id="$(xdotool search --onlyvisible --name "Sketchpad|几何" 2>/dev/null | tail -1 || true)"
    [[ -n "$window_id" ]] && break
    sleep 1
  done
  test -n "$window_id"
  xdotool windowfocus "$window_id"
  xdotool key --clearmodifiers ctrl+o
  sleep 1
  wine_path="$(winepath -w "$2" | tr -d "\r")"
  xdotool type --clearmodifiers --delay 1 "$wine_path"
  xdotool key Return
  sleep 3
  [[ "$(xdotool getwindowname "$window_id")" == *"$(basename "$2")"* ]]
  import -window root "${3}_opened.png"
  xdotool mousemove "${SKETCHPAD_REFLECTION_MIRROR_X:-159}" "${SKETCHPAD_REFLECTION_MIRROR_Y:-229}" click 1
  xdotool mousemove 220 40 click 1
  sleep 0.3
  import -window root "${3}_mark_menu.png"
  xdotool mousemove 275 78 click 1
  sleep 0.5
  xdotool key Escape
  xdotool mousemove "${SKETCHPAD_REFLECTION_SOURCE_X:-239}" "${SKETCHPAD_REFLECTION_SOURCE_Y:-229}" click 1
  xdotool mousemove 220 40 click 1
  sleep 0.3
  import -window root "${3}_reflect_menu.png"
  xdotool mousemove 275 223 click 1
  sleep 0.8
  import -window root "${3}_reflected.png"
  xdotool key --clearmodifiers ctrl+s
  sleep 2
  echo "Saved copy: $2 ($(stat -c %s "$2") bytes)"
' _ "$exe_path" "$sketch_path" "$capture_prefix"
