#!/usr/bin/env bash
set -euo pipefail

if (( $# != 3 )); then
  echo "Usage: dev/function_wine_probe.sh plot|define EXPRESSION SKETCH.gsp" >&2
  exit 2
fi
mode="$1"
expression="$2"
sketch_path="$(realpath "$3")"
exe_path="$(realpath GSP5Chs.exe)"
capture_prefix="${SKETCHPAD_FUNCTION_CAPTURE_PREFIX:-/tmp/sketchpad_function_probe}"
wine_prefix="${SKETCHPAD_WINEPREFIX:-${HOME}/.cache/sketchpad_rs_wine}"
image_capture="$(command -v import)"
confirm_angle="${SKETCHPAD_FUNCTION_CONFIRM_ANGLE:-0}"
exec 9>/tmp/sketchpad_rs_wine.lock
flock 9

xvfb-run -a bash -c '
  set -euo pipefail
  # Only the outer probe owns the queue lock; Wine services must not inherit it.
  exec 9>&-
  export WINEPREFIX="$6"
  wine "$1" "$2" > "${5}_runtime.log" 2>&1 &
  wine_pid=$!
  trap '\''kill "$wine_pid" 2>/dev/null || true; wineserver -k 2>/dev/null || true; wineserver -w 2>/dev/null || true'\'' EXIT
  window_id=""
  for _ in $(seq 1 20); do
    window_id="$(xdotool search --onlyvisible --name "Sketchpad|几何" 2>/dev/null | tail -1 || true)"
    [[ -n "$window_id" ]] && break
    sleep 1
  done
  test -n "$window_id"
  sleep 3
  xdotool windowfocus "$window_id"
  xdotool key --clearmodifiers ctrl+o
  sleep 1
  wine_path="$(winepath -w "$2" | tr -d "\r")"
  xdotool type --clearmodifiers --delay 1 "$wine_path"
  xdotool key Return
  sleep 3
  opened_name="$(xdotool getwindowname "$window_id")"
  [[ "$opened_name" == *"$(basename "$2")"* ]]
  "$7" -window root "${5}_opened.png"
  if [[ "$3" == plot ]]; then
    xdotool mousemove 400 40 click 1
    sleep 0.3
    xdotool mousemove 410 242 click 1
  elif [[ "$3" == define ]]; then
    xdotool mousemove 346 40 click 1
    sleep 0.4
    "$7" -window root "${5}_number_menu.png"
    xdotool mousemove 365 172 click 1
  else
    exit 2
  fi
  sleep 0.5
  "$7" -window root "${5}_dialog.png"
  xdotool mousemove 120 180 click 1
  printf %s "$4" | xdotool type --clearmodifiers --delay 70 --file -
  sleep 0.5
  "$7" -window root "${5}_typed.png"
  xdotool key Return
  sleep 1
  if [[ "$8" == 1 ]]; then
    "$7" -window root "${5}_angle_prompt.png"
    xdotool key Return
    sleep 1
  elif [[ "$8" == no ]]; then
    "$7" -window root "${5}_angle_prompt.png"
    xdotool mousemove 355 295 click 1
    sleep 1
  fi
  "$7" -window root "${5}_result.png"
  xdotool key --clearmodifiers ctrl+s
  sleep 2
  echo "Saved copy: $2 ($(stat -c %s "$2") bytes)"
' _ "$exe_path" "$sketch_path" "$mode" "$expression" "$capture_prefix" "$wine_prefix" "$image_capture" "$confirm_angle"
