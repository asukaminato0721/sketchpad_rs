#!/usr/bin/env bash
set -euo pipefail

# Original Sketchpad Graph-menu probe. The shared Wine prefix is serialized.
if (( $# != 3 )); then
  echo "Usage: dev/plot_points_wine_probe.sh FILE.gsp MENU_Y CAPTURE_PREFIX" >&2
  exit 2
fi
sketch_path="$(realpath "$1")"
menu_y="$2"
capture_prefix="$3"
exe_path="$(realpath GSP5Chs.exe)"
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
  import -window root "${4}_opened.png"
  if [[ "${SKETCHPAD_PLOT_SELECT_TABLE:-0}" == 1 ]]; then
    xdotool mousemove 91 132 click 1
    sleep 0.3
  fi
  xdotool mousemove 388 40 click 1
  sleep 0.5
  import -window root "${4}_menu.png"
  xdotool mousemove 405 "$3" click 1
  sleep 0.8
  import -window root "${4}_after_menu.png"
  if [[ "${SKETCHPAD_PLOT_POLAR:-0}" == 1 ]]; then
    xdotool mousemove 290 229 click 1
    sleep 0.3
    import -window root "${4}_polar_dialog.png"
  fi
  if [[ "${SKETCHPAD_PLOT_ACCEPT_PROMPT:-0}" == 1 ]]; then
    xdotool key Return
    sleep 0.8
    import -window root "${4}_after_prompt.png"
  fi
  if [[ "${SKETCHPAD_PLOT_ACCEPT_DIALOG:-0}" == 1 ]]; then
    xdotool key Return
    sleep 0.8
    import -window root "${4}_after_dialog.png"
  fi
  if [[ "${SKETCHPAD_PLOT_CANCEL_DIALOG:-0}" == 1 ]]; then
    xdotool key Escape
    sleep 0.8
    import -window root "${4}_after_cancel.png"
  fi
  xdotool key --clearmodifiers ctrl+s
  sleep 2
  import -window root "${4}_saved.png"
  echo "Saved copy: $2 ($(stat -c %s "$2") bytes)"
' _ "$exe_path" "$sketch_path" "$menu_y" "$capture_prefix"
