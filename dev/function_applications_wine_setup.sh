#!/usr/bin/env bash
set -euo pipefail
xdotool key --clearmodifiers ctrl+F10
sleep 1
for spec in f_parameter:323 g_parameter:363 derivative_parameter:403 f_length:323; do
 IFS=: read -r name function_y <<< "$spec"
 xdotool mousemove 700 450 click 1
 xdotool mousemove 105 "$function_y" click 1
 xdotool keydown ctrl
 if [[ "$name" == f_length ]]; then xdotool mousemove 110 94 click 1; else xdotool mousemove 330 75 click 1; fi
 xdotool keyup ctrl
 xdotool mousemove 346 40 click 1
 sleep 0.2
 xdotool mousemove 355 87 click 1
 sleep 0.4
 xdotool mousemove 480 323 click 1
 sleep 0.2
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_functions_menu.png"
 xdotool mousemove 440 348 click 1
 xdotool mousemove 480 290 click 1
 sleep 0.2
 xdotool mousemove 440 316 click 1
 xdotool type --clearmodifiers -- ')'
 sleep 0.3
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_input.png"
 xdotool key Return
 sleep 0.5
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_created.png"
 xdotool key --clearmodifiers ctrl+h
 sleep 0.2
 done
