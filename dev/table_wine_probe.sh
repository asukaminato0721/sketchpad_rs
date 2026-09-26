#!/usr/bin/env bash
set -euo pipefail

if (( $# != 2 )); then
  echo "Usage: dev/table_wine_probe.sh parameter|calculate|tabulate|add_data|remove_data COPY_OF_SKETCH.gsp" >&2
  exit 2
fi
mode="$1"
sketch_path="$(realpath "$2")"
exe_path="$(realpath GSP5Chs.exe)"
capture_prefix="${SKETCHPAD_TABLE_CAPTURE_PREFIX:-/tmp/sketchpad_table_probe}"
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
  if [[ "$3" == parameter ]]; then
    xdotool mousemove 346 40 click 1
    sleep 0.3
    xdotool mousemove 355 61 click 1
  elif [[ "$3" == calculate ]]; then
    xdotool mousemove 346 40 click 1
    sleep 0.3
    xdotool mousemove 355 87 click 1
  elif [[ "$3" == tabulate ]]; then
    xdotool mousemove 346 40 click 1
    sleep 0.3
    xdotool mousemove 355 113 click 1
  elif [[ "$3" == add_data ]]; then
    xdotool mousemove 346 40 click 1
    sleep 0.3
    xdotool mousemove 355 130 click 1
  elif [[ "$3" == remove_data ]]; then
    xdotool mousemove 346 40 click 1
    sleep 0.3
    xdotool mousemove 355 147 click 1
  else
    exit 2
  fi
  sleep 0.6
  import -window root "${4}_dialog.png"
  if [[ "$3" == parameter ]]; then
    if [[ -n "${SKETCHPAD_TABLE_NAME:-}" ]]; then
      xdotool mousemove 165 129 click 1 key --clearmodifiers ctrl+a
      xdotool type --clearmodifiers --delay 80 "$SKETCHPAD_TABLE_NAME"
    fi
    if [[ -n "${SKETCHPAD_TABLE_VALUE:-}" ]]; then
      xdotool mousemove 258 129 click 1 key --clearmodifiers ctrl+a
      xdotool type --clearmodifiers --delay 80 "$SKETCHPAD_TABLE_VALUE"
    fi
  elif [[ "$3" == calculate && -n "${SKETCHPAD_TABLE_EXPR:-}" ]]; then
    xdotool mousemove 160 180 click 1
    xdotool type --clearmodifiers --delay 80 "$SKETCHPAD_TABLE_EXPR"
  fi
  sleep 1
  import -window root "${4}_before_save.png"
  xdotool key Return
  sleep 1
  import -window root "${4}_result.png"
  if [[ "${SKETCHPAD_TABLE_POST_DRAG:-}" == "1" ]]; then
    xdotool mousemove "${SKETCHPAD_TABLE_DRAG_FROM_X:-90}" "${SKETCHPAD_TABLE_DRAG_FROM_Y:-122}" mousedown 1
    sleep 0.3
    xdotool mousemove "${SKETCHPAD_TABLE_DRAG_TO_X:-180}" "${SKETCHPAD_TABLE_DRAG_TO_Y:-200}"
    sleep 0.3
    xdotool mouseup 1
    sleep 0.5
    import -window root "${4}_after_drag.png"
  fi
  xdotool key --clearmodifiers ctrl+s
  sleep 2
  echo "Saved copy: $2 ($(stat -c %s "$2") bytes)"
' _ "$exe_path" "$sketch_path" "$mode" "$capture_prefix"
