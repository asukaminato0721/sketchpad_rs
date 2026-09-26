#!/usr/bin/env bash
# Create native quantity powers; see source_exponents_analysis.md.
set -euo pipefail
xdotool key --clearmodifiers ctrl+F10
sleep 1
for spec in area:85 radius:113 angle:141; do
 IFS=: read -r name y <<< "$spec"
xdotool mousemove 110 "$y" click 1
xdotool keydown ctrl
xdotool mousemove 330 85 click 1
xdotool keyup ctrl
xdotool mousemove 346 40 click 1
sleep 0.3
xdotool mousemove 355 87 click 1
sleep 0.4
xdotool mousemove 341 232 click 1
sleep 0.3
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_values.png"
xdotool mousemove 305 258 click 1
xdotool mousemove 250 232 click 1
xdotool mousemove 341 232 click 1
sleep 0.2
xdotool mousemove 305 276 click 1
sleep 0.5
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_input.png"
xdotool key Return
sleep 0.5
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_created.png"
xdotool key --clearmodifiers ctrl+h
sleep 0.3
done
xdotool mousemove 110 85 click 1
xdotool mousemove 346 40 click 1
sleep 0.3
xdotool mousemove 355 87 click 1
sleep 0.4
xdotool mousemove 341 232 click 1
sleep 0.2
xdotool mousemove 305 258 click 1
xdotool mousemove 250 232 click 1
xdotool mousemove 250 265 click 1
xdotool type --clearmodifiers --delay 50 '1+1'
xdotool mousemove 250 299 click 1
sleep 0.4
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_constant_input.png"
xdotool key Return
sleep 0.5
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_constant_created.png"
