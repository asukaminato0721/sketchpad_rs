#!/usr/bin/env bash
set -euo pipefail
xdotool key --clearmodifiers ctrl+F10
sleep 1
# Units menu: y=466 is degrees; y=381 is pixels (native suffix 0x0200).
for spec in radius_degree:110:94:466 radius_pixels:110:94:381 scalar_degree:330:75:466 scalar_pixels:330:75:381; do
 IFS=: read -r name x y unit_y <<< "$spec"
 xdotool mousemove "$x" "$y" click 1
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
 sleep 0.2
 xdotool mousemove 447 "$unit_y" click 1
 sleep 0.3
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_input.png"
 xdotool key Return
 sleep 0.4
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_created.png"
 xdotool key --clearmodifiers ctrl+h
 sleep 0.2
 done
