#!/usr/bin/env bash
set -euo pipefail
xdotool key --clearmodifiers ctrl+F10
sleep 1
xdotool mousemove 420 300 click 1
if [[ "$SKETCHPAD_WINE_CAPTURE_PREFIX" == *parallel* ]]; then
 xdotool mousemove 199 209 click 1
else
 xdotool mousemove 139 289 click 1
fi
xdotool mousemove 286 40 click 1
sleep 0.4
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_measure_menu.png"
xdotool mousemove 330 342 click 1
sleep 0.6
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_created.png"
