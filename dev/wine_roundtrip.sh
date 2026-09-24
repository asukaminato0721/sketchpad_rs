#!/usr/bin/env bash
set -euo pipefail

# Probes share a Wine prefix. Set SKETCHPAD_WINE_CAPTURE_PREFIX to keep captures.
exec 9>/tmp/sketchpad_rs_wine.lock
flock 9

if (( $# != 1 )); then
  echo "Usage: dev/wine_roundtrip.sh COPY_OF_GENERATED_FILE.gsp" >&2
  exit 2
fi

sketch_path="$(realpath "$1")"
exe_path="$(realpath GSP5Chs.exe)"
capture_prefix="${SKETCHPAD_WINE_CAPTURE_PREFIX:-/tmp/sketchpad_wine}"
test -f "$sketch_path"
test -f "$exe_path"

xvfb-run -a bash -c '
  set -euo pipefail
  export WINEPREFIX=/tmp/sketchpad_rs_wine
  capture_prefix="$3"
  wine "$1" "$2" > "${capture_prefix}_runtime.log" 2>&1 &
  wine_pid=$!
  trap '\''kill "$wine_pid" 2>/dev/null || true'\'' EXIT
  sleep 6
  window_id="$(xdotool search --onlyvisible --name "Sketchpad|几何" | tail -1)"
  test -n "$window_id"
  echo "Selected window: $(xdotool getwindowname "$window_id")"
  xdotool windowfocus "$window_id"
  xdotool key --clearmodifiers ctrl+o
  sleep 1
  wine_path="$(winepath -w "$2" | tr -d "\r")"
  echo "Opening $wine_path"
  xdotool type --clearmodifiers --delay 1 "$wine_path"
  xdotool key Return
  sleep 3
  opened_name="$(xdotool getwindowname "$window_id")"
  if [[ "$opened_name" != *"$(basename "$2")"* ]]; then
    echo "Sketchpad did not open $2: $opened_name" >&2
    exit 1
  fi
  import -window root "${capture_prefix}_before_save.png"
  if [[ "${SKETCHPAD_WINE_DRAG:-}" == "1" ]]; then
    xdotool mousemove "${SKETCHPAD_WINE_DRAG_FROM_X:-279}" "${SKETCHPAD_WINE_DRAG_FROM_Y:-299}" mousedown 1
    sleep 0.3
    xdotool mousemove "${SKETCHPAD_WINE_DRAG_TO_X:-320}" "${SKETCHPAD_WINE_DRAG_TO_Y:-250}"
    sleep 0.3
    xdotool mouseup 1
    sleep 0.5
    import -window root "${capture_prefix}_after_drag.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_ACTION_KEY:-}" ]]; then
    xdotool mousemove "${SKETCHPAD_WINE_ACTION_CLICK_X:-259}" "${SKETCHPAD_WINE_ACTION_CLICK_Y:-249}" click 1
    sleep 0.3
    xdotool key --clearmodifiers "$SKETCHPAD_WINE_ACTION_KEY"
    sleep 0.6
    import -window root "${capture_prefix}_after_action.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_ACTION_KEYS:-}" ]]; then
    xdotool mousemove "${SKETCHPAD_WINE_ACTION_CLICK_X:-259}" "${SKETCHPAD_WINE_ACTION_CLICK_Y:-249}" click 1
    sleep 0.3
    read -r -a action_keys <<< "$SKETCHPAD_WINE_ACTION_KEYS"
    for action_key in "${action_keys[@]}"; do
      xdotool key --clearmodifiers "$action_key"
      sleep 0.5
    done
    import -window root "${capture_prefix}_after_action.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_MENU_X:-}" ]]; then
    if [[ "${SKETCHPAD_WINE_CLEAR_SELECTION:-}" == "1" ]]; then
      xdotool mousemove "${SKETCHPAD_WINE_CLEAR_X:-450}" "${SKETCHPAD_WINE_CLEAR_Y:-280}" click 1
      sleep 0.2
    fi
    xdotool mousemove "${SKETCHPAD_WINE_ACTION_CLICK_X:-259}" "${SKETCHPAD_WINE_ACTION_CLICK_Y:-249}" click 1
    sleep 0.3
    import -window root "${capture_prefix}_after_select.png"
    if [[ -n "${SKETCHPAD_WINE_ACTION_SHIFT_CLICK_X:-}" ]]; then
      xdotool keydown Shift
      xdotool mousemove "$SKETCHPAD_WINE_ACTION_SHIFT_CLICK_X" "${SKETCHPAD_WINE_ACTION_SHIFT_CLICK_Y:-249}" click 1
      xdotool keyup Shift
      sleep 0.3
    fi
    if [[ -n "${SKETCHPAD_WINE_ACTION_SHIFT_CLICK_2_X:-}" ]]; then
      xdotool keydown Shift
      xdotool mousemove "$SKETCHPAD_WINE_ACTION_SHIFT_CLICK_2_X" "${SKETCHPAD_WINE_ACTION_SHIFT_CLICK_2_Y:-249}" click 1
      xdotool keyup Shift
      sleep 0.3
    fi
    xdotool mousemove "$SKETCHPAD_WINE_MENU_X" "${SKETCHPAD_WINE_MENU_Y:-40}" click 1
    sleep 0.5
    import -window root "${capture_prefix}_menu.png"
    if [[ -n "${SKETCHPAD_WINE_SUBMENU_X:-}" ]]; then
      test -n "${SKETCHPAD_WINE_MENU_ITEM_Y:-}"
      test -n "${SKETCHPAD_WINE_SUBMENU_Y:-}"
      xdotool mousemove "${SKETCHPAD_WINE_MENU_ITEM_X:-305}" "$SKETCHPAD_WINE_MENU_ITEM_Y"
      sleep 0.5
      import -window root "${capture_prefix}_submenu.png"
      xdotool mousemove "$SKETCHPAD_WINE_SUBMENU_X" "$SKETCHPAD_WINE_SUBMENU_Y" click 1
      sleep 0.7
      import -window root "${capture_prefix}_after_menu.png"
    elif [[ -n "${SKETCHPAD_WINE_MENU_ITEM_Y:-}" ]]; then
      xdotool mousemove "${SKETCHPAD_WINE_MENU_ITEM_X:-305}" "$SKETCHPAD_WINE_MENU_ITEM_Y" click 1
      sleep 0.7
      import -window root "${capture_prefix}_after_menu.png"
    else
      xdotool key Escape
    fi
  fi
  xdotool key --clearmodifiers ctrl+s
  sleep 3
  test -e "$2"
  echo "Saved copy: $2 ($(stat -c %s "$2") bytes)"
' _ "$exe_path" "$sketch_path" "$capture_prefix"
