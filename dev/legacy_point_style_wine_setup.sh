#!/usr/bin/env bash
set -euo pipefail
xdotool mousemove 159 169 click 1
xdotool key --clearmodifiers alt+d
sleep 0.3
xdotool key p
sleep 0.3
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_point_size_submenu.png"
xdotool key m
sleep 0.3
xdotool key Escape
xdotool mousemove 420 280 click 1
