#!/usr/bin/env bash
set -euo pipefail
xdotool key --clearmodifiers ctrl+F10
sleep 1
for spec in radius_angle_add:94:110:123:+ radius_angle_subtract:94:110:123:- radius_scalar_add:94:330:75:+ radius_scalar_subtract:94:330:75:- area_angle_add:67:110:123:+ area_radius_add:67:110:94:+; do
 IFS=: read -r name first_y second_x second_y op <<< "$spec"
 xdotool mousemove 110 "$first_y" click 1
 xdotool keydown ctrl
 xdotool mousemove "$second_x" "$second_y" click 1
 xdotool keyup ctrl
 xdotool mousemove 346 40 click 1
 sleep 0.2
 xdotool mousemove 355 87 click 1
 sleep 0.4
 xdotool mousemove 480 290 click 1
 sleep 0.2
 xdotool mousemove 440 316 click 1
 xdotool type --clearmodifiers --delay 50 "$op"
 xdotool mousemove 480 290 click 1
 sleep 0.2
 xdotool mousemove 440 335 click 1
 sleep 0.3
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_input.png"
 xdotool key Return
 sleep 0.4
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_created.png"
 xdotool key --clearmodifiers ctrl+h
 sleep 0.2
 done
