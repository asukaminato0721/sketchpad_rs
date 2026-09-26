#!/usr/bin/env bash
# Create a point on the transformed segment/circle in the matching base probe.
set -euo pipefail
xdotool key --clearmodifiers ctrl+F10
sleep 1
xdotool mousemove 29 145 click 1
if [[ "$SKETCHPAD_WINE_CAPTURE_PREFIX" == *segment* ]]; then
 xdotool mousemove 179 209 click 1
else
 xdotool mousemove 219 219 click 1
fi
sleep 0.5
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_created.png"
