#!/usr/bin/env bash
# Run through dev/wine_roundtrip.sh with measured_angle_rotation_base_port.gsp.
# Set SKETCHPAD_WINE_SETUP_SCRIPT to this script and SKETCHPAD_WINE_CAPTURE_PREFIX.
set -euo pipefail
xdotool key --clearmodifiers ctrl+F10
sleep 1
xdotool mousemove 219 249 click 1
xdotool mousemove 220 40 click 1
sleep 0.3
xdotool mousemove 275 61 click 1
xdotool key Escape
xdotool mousemove 110 85 click 1
xdotool mousemove 220 40 click 1
sleep 0.3
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_mark_menu.png"
xdotool mousemove 275 95 click 1
xdotool key Escape
xdotool mousemove 279 249 click 1
xdotool mousemove 220 40 click 1
sleep 0.3
xdotool mousemove 275 188 click 1
sleep 0.7
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_rotation_dialog.png"
xdotool key Return
sleep 0.7
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_created.png"
