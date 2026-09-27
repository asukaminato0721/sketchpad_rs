#!/usr/bin/env bash
set -euo pipefail
xdotool key --clearmodifiers ctrl+F10
sleep 1
for spec in length_exponent:94 area_exponent:67 angle_exponent:123 radius_angle_exponent:94; do
 IFS=: read -r name y <<< "$spec"
 xdotool mousemove 110 "$y" click 1
 if [[ "$name" == radius_angle* ]]; then
  xdotool keydown ctrl
  xdotool mousemove 110 123 click 1
  xdotool keyup ctrl
 fi
 xdotool mousemove 346 40 click 1
 sleep 0.2
 xdotool mousemove 355 87 click 1
 sleep 0.4
 if [[ "$name" == radius_angle* ]]; then
  xdotool mousemove 480 290 click 1
  sleep 0.2
  xdotool mousemove 440 316 click 1
 else
  xdotool type --clearmodifiers -- '2'
 fi
 xdotool mousemove 394 290 click 1
 xdotool mousemove 480 290 click 1
 sleep 0.2
 if [[ "$name" == radius_angle* ]]; then xdotool mousemove 440 335 click 1; else xdotool mousemove 440 316 click 1; fi
 sleep 0.3
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_input.png"
 xdotool key Return
 sleep 0.4
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_created.png"
 xdotool key --clearmodifiers ctrl+h
 sleep 0.2
 done
xdotool mousemove 110 94 click 1
xdotool mousemove 346 40 click 1
sleep 0.2
xdotool mousemove 355 87 click 1
sleep 0.4
xdotool mousemove 480 290 click 1
sleep 0.2
xdotool mousemove 440 316 click 1
xdotool mousemove 394 290 click 1
xdotool type --clearmodifiers -- '2'
xdotool mousemove 480 357 click 1
sleep 0.3
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_literal_units_menu.png"
xdotool key Escape
xdotool key Escape
