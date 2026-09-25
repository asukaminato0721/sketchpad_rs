#!/usr/bin/env bash
set -euo pipefail

# Capture one continuous free-point animation in Sketchpad 5. Run in an
# environment with Wine, Xvfb, and xdotool on PATH.
exec 9>/tmp/sketchpad_rs_wine.lock
flock 9

source_file="$(realpath tests/fixtures/animation_free_point_base.gsp)"
working_file=/tmp/sketchpad_free_animation_series.gsp
exe_file="$(realpath GSP5Chs.exe)"
cp "$source_file" "$working_file"

xvfb-run -a bash -c '
  set -euo pipefail
  export WINEPREFIX="${SKETCHPAD_WINEPREFIX:-/tmp/sketchpad_rs_wine}"
  wine "$1" "$2" >/tmp/sketchpad_free_animation_series.log 2>&1 &
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
  xdotool mousemove 259 219 click 1
  xdotool mousemove 144 40 click 1
  sleep 0.3
  xdotool mousemove 180 267 click 1
  sleep 0.3
  for step in $(seq 0 7); do
    import -window root "/tmp/sketchpad_free_animation_${step}.png"
    sleep 0.35
  done
  xdotool key --clearmodifiers ctrl+s
  sleep 0.5
' _ "$exe_file" "$working_file"
