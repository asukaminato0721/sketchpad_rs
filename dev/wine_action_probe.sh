#!/usr/bin/env bash
set -euo pipefail

if (( $# != 2 )); then
  echo "Usage: dev/wine_action_probe.sh create|click SKETCH.gsp" >&2
  exit 2
fi
mode="$1"
sketch_path="$(realpath "$2")"
exe_path="$(realpath GSP5Chs.exe)"
capture_prefix="${SKETCHPAD_ACTION_CAPTURE_PREFIX:-/tmp/sketchpad_action_probe}"
exec 9>/tmp/sketchpad_rs_wine.lock
flock 9

xvfb-run -a bash -c '
  set -euo pipefail
  # Only the outer probe owns the queue lock; Wine services must not inherit it.
  exec 9>&-
  export WINEPREFIX=/tmp/sketchpad_rs_wine
  wine "$1" "$2" > "${4}_runtime.log" 2>&1 &
  wine_pid=$!
  trap '\''kill "$wine_pid" 2>/dev/null || true; wineserver -k 2>/dev/null || true; wineserver -w 2>/dev/null || true'\'' EXIT
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
  import -window root "${4}_opened.png"
  if [[ "$3" == create ]]; then
    xdotool mousemove 269 189 click 1
    xdotool mousemove 90 40 click 1
    xdotool mousemove 130 181
    sleep 0.5
    import -window root "${4}_submenu.png"
    xdotool mousemove 280 181 click 1
    sleep 0.7
    import -window root "${4}_after_command.png"
    if xdotool search --onlyvisible --name "Hide|Show|隐藏|显示|Action" > /dev/null; then
      xdotool key Return
      sleep 0.5
    fi
  elif [[ "$3" == click ]]; then
    xdotool mousemove "${SKETCHPAD_ACTION_CLICK_X:-120}" "${SKETCHPAD_ACTION_CLICK_Y:-100}" click 1
    sleep 0.7
  else
    echo "Expected create or click mode" >&2
    exit 2
  fi
  import -window root "${4}_after_action.png"
  xdotool key --clearmodifiers ctrl+s
  sleep 2
  echo "Saved copy: $2 ($(stat -c %s "$2") bytes)"
' _ "$exe_path" "$sketch_path" "$mode" "$capture_prefix"
