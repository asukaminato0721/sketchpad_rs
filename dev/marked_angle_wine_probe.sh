#!/usr/bin/env bash
set -euo pipefail

if (( $# != 1 )); then
  echo "Usage: dev/marked_angle_wine_probe.sh SKETCH.gsp" >&2
  exit 2
fi
sketch_path="$(realpath "$1")"
exe_path="$(realpath GSP5Chs.exe)"
capture_prefix="${SKETCHPAD_ANGLE_CAPTURE_PREFIX:-/tmp/sketchpad_marked_angle_probe}"
exec 9>/tmp/sketchpad_rs_wine.lock
flock 9

xvfb-run -a bash -c '
  set -euo pipefail
  # Only the outer probe owns the queue lock; Wine services must not inherit it.
  exec 9>&-
  export WINEPREFIX="${SKETCHPAD_WINEPREFIX:-/tmp/sketchpad_rs_wine}"
  wine "$1" "$2" > "${3}_runtime.log" 2>&1 &
  wine_pid=$!
  trap '\''kill "$wine_pid" 2>/dev/null || true; wineserver -k 2>/dev/null || true; wineserver -w 2>/dev/null || true'\'' EXIT
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
  xdotool mousemove 189 219 click 1
  xdotool mousemove 220 40 click 1
  sleep 0.3
  xdotool mousemove 275 61 click 1
  sleep 0.5
  xdotool mousemove 289 229 click 1
  xdotool keydown Shift
  xdotool mousemove 289 289 click 1
  xdotool mousemove 349 289 click 1
  xdotool keyup Shift
  import -window root "${3}_angle_selected.png"
  xdotool mousemove 220 40 click 1
  sleep 0.3
  import -window root "${3}_angle_menu.png"
  xdotool mousemove 275 95 click 1
  sleep 0.6
  import -window root "${3}_angle_marked.png"
  xdotool key --clearmodifiers ctrl+s
  sleep 1
  cp "$2" "${3}_marked_only.gsp"
  xdotool key Escape
  xdotool mousemove "${SKETCHPAD_ANGLE_SOURCE_X:-229}" "${SKETCHPAD_ANGLE_SOURCE_Y:-219}" click 1
  xdotool mousemove 220 40 click 1
  sleep 0.3
  import -window root "${3}_rotation_menu.png"
  xdotool mousemove 275 188 click 1
  sleep 0.8
  import -window root "${3}_rotation_dialog.png"
  xdotool mousemove 324 248 click 1
  sleep 0.8
  import -window root "${3}_rotated.png"
  xdotool key --clearmodifiers ctrl+s
  sleep 2
  echo "Saved copy: $2 ($(stat -c %s "$2") bytes)"
' _ "$exe_path" "$sketch_path" "$capture_prefix"
