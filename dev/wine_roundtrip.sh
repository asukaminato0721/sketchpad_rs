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
image_capture="$(command -v import)"
test -f "$sketch_path"
test -f "$exe_path"

xvfb-run -a bash -c '
  set -euo pipefail
  export WINEPREFIX="${SKETCHPAD_WINEPREFIX:-/tmp/sketchpad_rs_wine}"
  capture_prefix="$3"
  wine "$1" "$2" > "${capture_prefix}_runtime.log" 2>&1 &
  wine_pid=$!
  trap '\''kill "$wine_pid" 2>/dev/null || true'\'' EXIT
  window_id=""
  for _ in $(seq 1 20); do
    window_id="$(xdotool search --onlyvisible --name "Sketchpad|几何" 2>/dev/null | tail -1 || true)"
    [[ -n "$window_id" ]] && break
    sleep 1
  done
  test -n "$window_id"
  test -n "$window_id"
  echo "Selected window: $(xdotool getwindowname "$window_id")"
  xdotool windowfocus "$window_id"
  sleep 3
  xdotool key --clearmodifiers ctrl+o
  sleep 2
  wine_path="$(winepath -w "$2" | tr -d "\r")"
  echo "Opening $wine_path"
  xdotool type --clearmodifiers --delay 1 "$wine_path"
  xdotool key Return
  opened_name=""
  for _ in $(seq 1 10); do
    sleep 1
    opened_name="$(xdotool getwindowname "$window_id")"
    [[ "$opened_name" == *"$(basename "$2")"* ]] && break
  done
  if [[ "$opened_name" != *"$(basename "$2")"* ]]; then
    "$4" -window root "${capture_prefix}_open_failed.png"
    echo "Sketchpad did not open $2: $opened_name" >&2
    exit 1
  fi
  "$4" -window root "${capture_prefix}_before_save.png"
  if [[ -n "${SKETCHPAD_WINE_SETUP_SCRIPT:-}" ]]; then
    bash "$SKETCHPAD_WINE_SETUP_SCRIPT"
    sleep 0.5
    "$4" -window root "${capture_prefix}_after_setup.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_TOOL_Y:-}" ]]; then
    xdotool mousemove "${SKETCHPAD_WINE_TOOL_X:-29}" "$SKETCHPAD_WINE_TOOL_Y" click 1
    sleep 0.3
    "$4" -window root "${capture_prefix}_after_tool.png"
  fi
  if [[ "${SKETCHPAD_WINE_ACTION_DOUBLE_CLICK:-}" == "1" ]]; then
    xdotool mousemove "${SKETCHPAD_WINE_ACTION_CLICK_X:-259}" "${SKETCHPAD_WINE_ACTION_CLICK_Y:-249}" click --repeat 2 --delay 100 1
    sleep 0.4
    "$4" -window root "${capture_prefix}_after_double_click.png"
    if [[ -n "${SKETCHPAD_WINE_ACTION_TEXT:-}" ]]; then
      xdotool key --clearmodifiers ctrl+a
      printf "%s" "$SKETCHPAD_WINE_ACTION_TEXT" | xdotool type --clearmodifiers --delay 20 --file -
      xdotool key Return
      sleep 0.5
      "$4" -window root "${capture_prefix}_after_action_text.png"
    fi
    if [[ -n "${SKETCHPAD_WINE_EDIT_KEYS:-}" ]]; then
      read -r -a edit_keys <<< "$SKETCHPAD_WINE_EDIT_KEYS"
      for edit_key in "${edit_keys[@]}"; do
        xdotool key --clearmodifiers "$edit_key"
        sleep 0.3
      done
      "$4" -window root "${capture_prefix}_after_edit_keys.png"
    fi
    if [[ -n "${SKETCHPAD_WINE_EDIT_SIZE:-}" ]]; then
      xdotool mousemove "${SKETCHPAD_WINE_EDIT_SIZE_X:-218}" "${SKETCHPAD_WINE_EDIT_SIZE_Y:-316}" click 1
      xdotool key --clearmodifiers Home Delete Delete
      xdotool type --clearmodifiers --delay 30 "$SKETCHPAD_WINE_EDIT_SIZE"
      xdotool key Return
      sleep 0.5
      "$4" -window root "${capture_prefix}_after_edit_size.png"
    fi
    if [[ -n "${SKETCHPAD_WINE_EDIT_FONT_ROW_Y:-}" ]]; then
      xdotool mousemove "${SKETCHPAD_WINE_EDIT_FONT_X:-185}" "${SKETCHPAD_WINE_EDIT_FONT_Y:-316}" click 1
      sleep 0.3
      "$4" -window root "${capture_prefix}_font_menu.png"
      xdotool mousemove "${SKETCHPAD_WINE_EDIT_FONT_ROW_X:-75}" "$SKETCHPAD_WINE_EDIT_FONT_ROW_Y" click 1
      sleep 0.5
      "$4" -window root "${capture_prefix}_after_edit_font.png"
    fi
  fi
  if [[ "${SKETCHPAD_WINE_ACTION_RIGHT_CLICK:-}" == "1" ]]; then
    if [[ -n "${SKETCHPAD_WINE_PRE_CONTEXT_KEY:-}" ]]; then
      xdotool key --clearmodifiers "$SKETCHPAD_WINE_PRE_CONTEXT_KEY"
      sleep 0.5
      "$4" -window root "${capture_prefix}_before_right_click.png"
    fi
    xdotool mousemove "${SKETCHPAD_WINE_ACTION_CLICK_X:-105}" "${SKETCHPAD_WINE_ACTION_CLICK_Y:-85}" click 3
    sleep 0.5
    "$4" -window root "${capture_prefix}_after_right_click.png"
    if [[ -n "${SKETCHPAD_WINE_RIGHT_MENU_Y:-}" ]]; then
      xdotool mousemove "${SKETCHPAD_WINE_RIGHT_MENU_X:-180}" "$SKETCHPAD_WINE_RIGHT_MENU_Y" click 1
      sleep 0.5
      "$4" -window root "${capture_prefix}_after_right_menu.png"
    fi
  fi
  if [[ "${SKETCHPAD_WINE_DRAG:-}" == "1" ]]; then
    xdotool mousemove "${SKETCHPAD_WINE_DRAG_FROM_X:-279}" "${SKETCHPAD_WINE_DRAG_FROM_Y:-299}" mousedown 1
    sleep 0.3
    drag_from_x="${SKETCHPAD_WINE_DRAG_FROM_X:-279}"
    drag_from_y="${SKETCHPAD_WINE_DRAG_FROM_Y:-299}"
    drag_to_x="${SKETCHPAD_WINE_DRAG_TO_X:-320}"
    drag_to_y="${SKETCHPAD_WINE_DRAG_TO_Y:-250}"
    drag_steps="${SKETCHPAD_WINE_DRAG_STEPS:-1}"
    for step in $(seq 1 "$drag_steps"); do
      xdotool mousemove "$((drag_from_x + (drag_to_x - drag_from_x) * step / drag_steps))" "$((drag_from_y + (drag_to_y - drag_from_y) * step / drag_steps))"
      sleep 0.08
    done
    sleep 0.3
    "$4" -window root "${capture_prefix}_during_drag.png"
    xdotool mouseup 1
    sleep 0.5
    "$4" -window root "${capture_prefix}_after_drag.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_ACTION_KEY:-}" ]]; then
    xdotool mousemove "${SKETCHPAD_WINE_ACTION_CLICK_X:-259}" "${SKETCHPAD_WINE_ACTION_CLICK_Y:-249}" click 1
    sleep 0.3
    xdotool key --clearmodifiers "$SKETCHPAD_WINE_ACTION_KEY"
    sleep 0.6
    "$4" -window root "${capture_prefix}_after_action.png"
  fi
  if [[ "${SKETCHPAD_WINE_ACTION_CLICK_ONLY:-}" == "1" ]]; then
    if [[ -n "${SKETCHPAD_WINE_PRE_ACTION_KEY:-}" ]]; then
      sleep "${SKETCHPAD_WINE_PRE_ACTION_DELAY_SECONDS:-1}"
      xdotool key --clearmodifiers "$SKETCHPAD_WINE_PRE_ACTION_KEY"
      sleep "${SKETCHPAD_WINE_PRE_ACTION_WAIT_SECONDS:-1}"
      "$4" -window root "${capture_prefix}_before_action_click.png"
    fi
    xdotool mousemove "${SKETCHPAD_WINE_ACTION_CLICK_X:-105}" "${SKETCHPAD_WINE_ACTION_CLICK_Y:-85}" click 1
    sleep "${SKETCHPAD_WINE_ACTION_WAIT_SECONDS:-1}"
    "$4" -window root "${capture_prefix}_after_action_click.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_ACTION_KEYS:-}" ]]; then
    xdotool mousemove "${SKETCHPAD_WINE_ACTION_CLICK_X:-259}" "${SKETCHPAD_WINE_ACTION_CLICK_Y:-249}" click 1
    sleep 0.3
    read -r -a action_keys <<< "$SKETCHPAD_WINE_ACTION_KEYS"
    for action_key in "${action_keys[@]}"; do
      xdotool key --clearmodifiers "$action_key"
      sleep 0.5
    done
    "$4" -window root "${capture_prefix}_after_action.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_MENU_X:-}" ]]; then
    if [[ -n "${SKETCHPAD_WINE_PRE_MENU_KEY:-}" ]]; then
      xdotool key --clearmodifiers "$SKETCHPAD_WINE_PRE_MENU_KEY"
      sleep 0.5
      "$4" -window root "${capture_prefix}_before_menu_selection.png"
    fi
    if [[ "${SKETCHPAD_WINE_CLEAR_SELECTION:-}" == "1" ]]; then
      xdotool mousemove "${SKETCHPAD_WINE_CLEAR_X:-450}" "${SKETCHPAD_WINE_CLEAR_Y:-280}" click 1
      sleep 0.2
    fi
    if [[ "${SKETCHPAD_WINE_KEEP_SELECTION:-}" != "1" ]]; then
      xdotool mousemove "${SKETCHPAD_WINE_ACTION_CLICK_X:-259}" "${SKETCHPAD_WINE_ACTION_CLICK_Y:-249}" click 1
      sleep 0.3
    fi
    "$4" -window root "${capture_prefix}_after_select.png"
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
    if [[ -n "${SKETCHPAD_WINE_ACTION_SHIFT_CLICK_X:-}" ]]; then
      "$4" -window root "${capture_prefix}_after_shift_select.png"
    fi
    xdotool mousemove "$SKETCHPAD_WINE_MENU_X" "${SKETCHPAD_WINE_MENU_Y:-40}" click 1
    sleep 0.5
    "$4" -window root "${capture_prefix}_menu.png"
    if [[ -n "${SKETCHPAD_WINE_MENU_KEYS:-}" ]]; then
      read -r -a menu_keys <<< "$SKETCHPAD_WINE_MENU_KEYS"
      for menu_key in "${menu_keys[@]}"; do
        xdotool key --clearmodifiers "$menu_key"
        sleep 0.3
      done
      "$4" -window root "${capture_prefix}_after_menu.png"
    elif [[ -n "${SKETCHPAD_WINE_SUBMENU_X:-}" ]]; then
      test -n "${SKETCHPAD_WINE_MENU_ITEM_Y:-}"
      test -n "${SKETCHPAD_WINE_SUBMENU_Y:-}"
      xdotool mousemove "${SKETCHPAD_WINE_MENU_ITEM_X:-305}" "$SKETCHPAD_WINE_MENU_ITEM_Y"
      sleep 0.5
      "$4" -window root "${capture_prefix}_submenu.png"
      xdotool mousemove "$SKETCHPAD_WINE_SUBMENU_X" "$SKETCHPAD_WINE_SUBMENU_Y" click 1
      sleep 0.7
      "$4" -window root "${capture_prefix}_after_menu.png"
    elif [[ -n "${SKETCHPAD_WINE_MENU_ITEM_Y:-}" ]]; then
      xdotool mousemove "${SKETCHPAD_WINE_MENU_ITEM_X:-305}" "$SKETCHPAD_WINE_MENU_ITEM_Y" click 1
      sleep 0.7
      "$4" -window root "${capture_prefix}_after_menu.png"
    else
      xdotool key Escape
    fi
  fi
  if [[ -n "${SKETCHPAD_WINE_CONFIRM_KEY:-}" ]]; then
    xdotool key --clearmodifiers "$SKETCHPAD_WINE_CONFIRM_KEY"
    sleep 0.5
    "$4" -window root "${capture_prefix}_after_confirm.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_CONFIRM_SECOND_KEY:-}" ]]; then
    xdotool key --clearmodifiers "$SKETCHPAD_WINE_CONFIRM_SECOND_KEY"
    sleep 0.5
    "$4" -window root "${capture_prefix}_after_second_confirm.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_DIALOG_MODE_X:-}" ]]; then
    xdotool mousemove "$SKETCHPAD_WINE_DIALOG_MODE_X" "${SKETCHPAD_WINE_DIALOG_MODE_Y:-129}" click 1
    sleep 0.3
    "$4" -window root "${capture_prefix}_dialog_mode.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_DIALOG_TEXT:-}" ]]; then
    xdotool mousemove "${SKETCHPAD_WINE_DIALOG_INPUT_X:-200}" "${SKETCHPAD_WINE_DIALOG_INPUT_Y:-128}" click 1
    xdotool key --clearmodifiers ctrl+a
    printf "%s" "$SKETCHPAD_WINE_DIALOG_TEXT" | xdotool type --clearmodifiers --delay 20 --file -
    sleep 0.3
    "$4" -window root "${capture_prefix}_dialog_text.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_DIALOG_TEXT2:-}" ]]; then
    xdotool mousemove "${SKETCHPAD_WINE_DIALOG_INPUT2_X:-258}" "${SKETCHPAD_WINE_DIALOG_INPUT2_Y:-129}" click 1
    xdotool key --clearmodifiers ctrl+a
    printf "%s" "$SKETCHPAD_WINE_DIALOG_TEXT2" | xdotool type --clearmodifiers --delay 20 --file -
    sleep 0.3
    "$4" -window root "${capture_prefix}_dialog_text2.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_DIALOG_CONFIRM_KEY:-}" ]]; then
    xdotool key --clearmodifiers "$SKETCHPAD_WINE_DIALOG_CONFIRM_KEY"
    sleep 0.5
    "$4" -window root "${capture_prefix}_after_dialog_confirm.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_DIALOG_CLOSE_KEY:-}" ]]; then
    xdotool key --clearmodifiers "$SKETCHPAD_WINE_DIALOG_CLOSE_KEY"
    sleep 0.5
    "$4" -window root "${capture_prefix}_after_dialog_close_key.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_DIALOG_PATH_X:-}" ]]; then
    xdotool mousemove "$SKETCHPAD_WINE_DIALOG_PATH_X" "${SKETCHPAD_WINE_DIALOG_PATH_Y:-230}" click 1
    sleep 0.3
    "$4" -window root "${capture_prefix}_dialog_path.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_DIALOG_CLICK_X:-}" ]]; then
    xdotool mousemove "$SKETCHPAD_WINE_DIALOG_CLICK_X" "${SKETCHPAD_WINE_DIALOG_CLICK_Y:-174}" click 1
    sleep 0.5
    "$4" -window root "${capture_prefix}_after_dialog_click.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_DIALOG_CLOSE_X:-}" ]]; then
    xdotool mousemove "$SKETCHPAD_WINE_DIALOG_CLOSE_X" "${SKETCHPAD_WINE_DIALOG_CLOSE_Y:-174}" click 1
    sleep 0.5
    "$4" -window root "${capture_prefix}_after_dialog_close.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_DIALOG_FINAL_X:-}" ]]; then
    xdotool mousemove "$SKETCHPAD_WINE_DIALOG_FINAL_X" "${SKETCHPAD_WINE_DIALOG_FINAL_Y:-348}" click 1
    sleep 0.5
    "$4" -window root "${capture_prefix}_after_dialog_final.png"
  fi
  if [[ -n "${SKETCHPAD_WINE_DIALOG_FINAL_CONFIRM_KEY:-}" ]]; then
    xdotool key --clearmodifiers "$SKETCHPAD_WINE_DIALOG_FINAL_CONFIRM_KEY"
    sleep 0.5
    "$4" -window root "${capture_prefix}_after_dialog_final_confirm.png"
  fi
  if [[ "${SKETCHPAD_WINE_DRAG_AFTER_MENU:-}" == "1" ]]; then
    xdotool mousemove "${SKETCHPAD_WINE_DRAG_FROM_X:-279}" "${SKETCHPAD_WINE_DRAG_FROM_Y:-299}" mousedown 1
    sleep 0.3
    xdotool mousemove "${SKETCHPAD_WINE_DRAG_TO_X:-320}" "${SKETCHPAD_WINE_DRAG_TO_Y:-250}"
    sleep 0.3
    xdotool mouseup 1
    sleep 0.5
    "$4" -window root "${capture_prefix}_after_menu_drag.png"
  fi
  xdotool key --clearmodifiers ctrl+s
  sleep 3
  test -e "$2"
  echo "Saved copy: $2 ($(stat -c %s "$2") bytes)"
' _ "$exe_path" "$sketch_path" "$capture_prefix" "$image_capture"
