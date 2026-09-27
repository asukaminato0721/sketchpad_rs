#!/usr/bin/env bash
set -euo pipefail
xdotool key --clearmodifiers ctrl+F10
sleep 1
xdotool mousemove 105 323 click 1
xdotool mousemove 346 40 click 1
sleep 0.2
xdotool mousemove 355 87 click 1
sleep 0.4
xdotool mousemove 480 323 click 1
sleep 0.2
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_values_menu.png"
xdotool mousemove 440 368 click 1
sleep 0.3
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_function_input.png"
xdotool type --clearmodifiers -- '2'
xdotool mousemove 480 357 click 1
sleep 0.2
xdotool mousemove 447 466 click 1
xdotool type --clearmodifiers -- ')' 
sleep 0.3
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_literal_input.png"
xdotool key Return
sleep 0.5
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_literal_created.png"
