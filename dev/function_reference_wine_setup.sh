#!/usr/bin/env bash
set -euo pipefail
xdotool key --clearmodifiers ctrl+F10
sleep 1
xdotool mousemove 330 75 click 1
xdotool mousemove 400 40 click 1
sleep 0.2
xdotool mousemove 410 242 click 1
sleep 0.4
xdotool mousemove 517 290 click 1
sleep 0.2
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_values_menu.png"
xdotool mousemove 440 343 click 1
xdotool type --clearmodifiers -- '*x+1'
sleep 0.3
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_parameter_input.png"
xdotool key Return
sleep 0.5
xdotool key Return
sleep 0.5
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_parameter_created.png"
xdotool mousemove 700 450 click 1
xdotool mousemove 100 147 click 1
xdotool mousemove 346 40 click 1
sleep 0.2
xdotool mousemove 355 172 click 1
sleep 0.4
xdotool mousemove 517 323 click 1
sleep 0.2
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_functions_menu.png"
xdotool mousemove 440 349 click 1
xdotool type --clearmodifiers -- 'x+1)'
sleep 0.3
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_function_input.png"
xdotool key Return
sleep 0.5
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_function_created.png"
