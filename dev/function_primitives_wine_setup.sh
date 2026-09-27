#!/usr/bin/env bash
set -euo pipefail
xdotool key --clearmodifiers ctrl+F10
sleep 1
for spec in trunc:586 round:566 sgn:546 asin:406 acos:426 atan:446; do
 IFS=: read -r name function_y <<< "$spec"
 xdotool mousemove 700 450 click 1
 xdotool mousemove 400 40 click 1
 sleep 0.2
 xdotool mousemove 410 242 click 1
 sleep 0.4
 xdotool mousemove 517 323 click 1
 sleep 0.2
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_menu.png"
 xdotool mousemove 440 "$function_y" click 1
 xdotool type --clearmodifiers -- 'x)'
 sleep 0.3
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_input.png"
 xdotool key Return
 sleep 0.5
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_created.png"
 # Creating a plot on the initial frame may offer to switch to radians.
 if [[ "$name" == trunc || "$name" == asin ]]; then xdotool key Return; sleep 1; fi
 done
