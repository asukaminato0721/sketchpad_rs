#!/usr/bin/env bash
set -euo pipefail
xdotool key --clearmodifiers ctrl+F10
sleep 1
for spec in angle_subtract:123:0 area_subtract:67:1 radius_subtract:94:1 compound_add:94:+ compound_subtract:94:-; do
 IFS=: read -r name y value <<< "$spec"
 xdotool mousemove 110 "$y" click 1
 if [[ "$name" == compound* ]]; then
  xdotool keydown ctrl
  xdotool mousemove 110 123 click 1
  xdotool keyup ctrl
 fi
 xdotool mousemove 346 40 click 1
 sleep 0.2
 xdotool mousemove 355 87 click 1
 sleep 0.4
 if [[ "$name" == compound* ]]; then xdotool type --clearmodifiers -- '('; fi
 xdotool mousemove 480 290 click 1
 sleep 0.2
 xdotool mousemove 440 316 click 1
 if [[ "$name" == compound* ]]; then
  xdotool type --clearmodifiers -- '*'
  xdotool mousemove 480 290 click 1
  sleep 0.2
  xdotool mousemove 440 335 click 1
  xdotool type --clearmodifiers -- ')'
  xdotool mousemove 394 290 click 1
  xdotool type --clearmodifiers -- "2${value}1"
 else
  xdotool type --clearmodifiers -- "-${value}"
 fi
 sleep 0.3
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_input.png"
 xdotool key Return
 sleep 0.4
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_created.png"
 xdotool key --clearmodifiers ctrl+h
 sleep 0.2
 done
