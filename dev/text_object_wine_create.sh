#!/usr/bin/env bash
set -euo pipefail

if (( $# != 2 )); then
  echo "Usage: dev/text_object_wine_create.sh COPY_OF_BLANK_FILE.gsp TEXT" >&2
  exit 2
fi

exec 9>/tmp/sketchpad_rs_wine.lock
flock 9

sketch_path="$(realpath "$1")"
exe_path="$(realpath GSP5Chs.exe)"
text_to_type="$2"
test -f "$sketch_path"
test -f "$exe_path"

xvfb-run -a bash -c '
  set -euo pipefail
  # Only the outer probe owns the queue lock; Wine services must not inherit it.
  exec 9>&-
  export WINEPREFIX=/tmp/sketchpad_rs_wine
  wine "$1" "$2" > /tmp/sketchpad_text_object_wine_runtime.log 2>&1 &
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
  import -window root /tmp/sketchpad_text_object_before.png
  xdotool mousemove 29 329 click 1
  sleep 0.4
  import -window root /tmp/sketchpad_text_object_tool.png
  xdotool mousemove 150 150 click --repeat 2 --delay 100 1
  sleep 0.4
  import -window root /tmp/sketchpad_text_object_started.png
  if [[ "${SKETCHPAD_TEXT_OBJECT_MULTILINE:-}" == "1" ]]; then
    xdotool type --clearmodifiers --delay 80 A
    xdotool key Return
    xdotool type --clearmodifiers --delay 80 B
  else
    xdotool type --clearmodifiers --delay 80 "$3"
  fi
  sleep 0.4
  import -window root /tmp/sketchpad_text_object_typed.png
  xdotool mousemove 30 95 click 1
  sleep 0.4
  xdotool key --clearmodifiers ctrl+s
  sleep 2
  import -window root /tmp/sketchpad_text_object_saved.png
  echo "Saved $2 ($(stat -c %s "$2") bytes)"
' _ "$exe_path" "$sketch_path" "$text_to_type"
