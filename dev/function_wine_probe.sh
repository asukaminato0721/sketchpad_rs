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
exec 9>/tmp/sketchpad_rs_wine.lock
flock 9

xvfb-run -a bash -c '
  set -euo pipefail
  export WINEPREFIX=/tmp/sketchpad_rs_wine
  wine "$1" "$2" > "${5}_runtime.log" 2>&1 &
  wine_pid=$!
  trap '\''kill "$wine_pid" 2>/dev/null || true'\'' EXIT
  sleep 6
  window_id="$(xdotool search --onlyvisible --name "Sketchpad|几何" | tail -1)"
  test -n "$window_id"
  xdotool windowfocus "$window_id"
  xdotool key --clearmodifiers ctrl+o
  sleep 1
  wine_path="$(winepath -w "$2" | tr -d "\r")"
  xdotool type --clearmodifiers --delay 1 "$wine_path"
  xdotool key Return
  sleep 3
  opened_name="$(xdotool getwindowname "$window_id")"
  [[ "$opened_name" == *"$(basename "$2")"* ]]
  import -window root "${5}_opened.png"
  if [[ "$3" == plot ]]; then
    xdotool mousemove 400 40 click 1
    sleep 0.3
    xdotool mousemove 410 242 click 1
  elif [[ "$3" == define ]]; then
    xdotool mousemove 346 40 click 1
    sleep 0.4
    import -window root "${5}_number_menu.png"
    xdotool mousemove 365 172 click 1
  else
    exit 2
  fi
  sleep 0.5
  import -window root "${5}_dialog.png"
  xdotool mousemove 120 180 click 1
  xdotool type --clearmodifiers --delay 70 "$4"
  sleep 0.5
  import -window root "${5}_typed.png"
  xdotool key Return
  sleep 1
  import -window root "${5}_result.png"
  xdotool key --clearmodifiers ctrl+s
  sleep 2
  echo "Saved copy: $2 ($(stat -c %s "$2") bytes)"
' _ "$exe_path" "$sketch_path" "$mode" "$expression" "$capture_prefix"
