#!/usr/bin/env bash
# Create Distance for point 3 and straight object 4 in the three base probes.
set -euo pipefail
xdotool key --clearmodifiers ctrl+F10
sleep 1
point_x=309
if [[ "$SKETCHPAD_WINE_CAPTURE_PREFIX" == *ray* ]]; then point_x=109; fi
xdotool mousemove "$point_x" 189 click 1
xdotool keydown Shift
xdotool mousemove 209 249 click 1
xdotool keyup Shift
xdotool mousemove 286 40 click 1
sleep 0.4
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_measure_menu.png"
xdotool mousemove 330 78 click 1
sleep 0.6
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_created.png"
