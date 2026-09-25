#!/usr/bin/env bash
set -euo pipefail

if (( $# != 2 )) || [[ "$2" != horizontal && "$2" != vertical ]]; then
  echo "Usage: dev/rectangular_mixed_wine_probe.sh SKETCH.gsp horizontal|vertical" >&2
  exit 2
fi
sketch_path="$(realpath "$1")"
exe_path="$(realpath GSP5Chs.exe)"
capture_prefix="${SKETCHPAD_RECT_MIXED_CAPTURE_PREFIX:-/tmp/sketchpad_rectangular_mixed_probe}"
axis="$2"
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
  xdotool mousemove 114 85 click 1
  xdotool keydown Shift
  xdotool mousemove 114 112 click 1
  xdotool keyup Shift
  xdotool mousemove 220 40 click 1
  sleep 0.3
  xdotool mousemove 275 146 click 1
  sleep 0.5
  xdotool key Escape
  xdotool mousemove "${SKETCHPAD_RECT_MIXED_SOURCE_X:-159}" "${SKETCHPAD_RECT_MIXED_SOURCE_Y:-279}" click 1
  xdotool mousemove 220 40 click 1
  sleep 0.3
  xdotool mousemove 275 171 click 1
  sleep 0.8
  import -window root "${3}_initial.png"
  if [[ "$4" == horizontal ]]; then
    xdotool mousemove 112 125 click 1
    xdotool mousemove 221 162 click 1
  else
    xdotool mousemove 112 228 click 1
    xdotool mousemove 221 264 click 1
  fi
  xdotool key --clearmodifiers ctrl+a
  printf %s "${SKETCHPAD_RECT_MIXED_FIXED_VALUE:-2}" | xdotool type --clearmodifiers --delay 20 --file -
  sleep 0.3
  import -window root "${3}_mixed.png"
  xdotool mousemove 350 321 click 1
  sleep 0.8
  import -window root "${3}_translated.png"
  xdotool key --clearmodifiers ctrl+s
  sleep 2
  echo "Saved copy: $2 ($(stat -c %s "$2") bytes)"
' _ "$exe_path" "$sketch_path" "$capture_prefix" "$axis"
