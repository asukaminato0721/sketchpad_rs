#!/usr/bin/env bash
# Run through dev/wine_roundtrip.sh with numeric_ratio_*_base_port.gsp.
# Set SKETCHPAD_RATIO_SHAPE_X/Y: point 279/249, segment 279/269, circle 319/249, polygon 299/279.
# Set SKETCHPAD_WINE_SETUP_SCRIPT to this script and SKETCHPAD_WINE_CAPTURE_PREFIX.
set -euo pipefail
xdotool key --clearmodifiers ctrl+F10
sleep 1
xdotool mousemove 159 249 click 1
xdotool mousemove 220 40 click 1
sleep 0.3
xdotool mousemove 275 61 click 1
xdotool key Escape
xdotool mousemove 110 85 click 1
xdotool mousemove 220 40 click 1
sleep 0.3
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_mark_menu.png"
xdotool mousemove 275 112 click 1
xdotool key Escape
if [[ "$SKETCHPAD_WINE_CAPTURE_PREFIX" == *polygon* ]]; then SKETCHPAD_RATIO_SHAPE_X=299; SKETCHPAD_RATIO_SHAPE_Y=279; fi
xdotool mousemove "$SKETCHPAD_RATIO_SHAPE_X" "$SKETCHPAD_RATIO_SHAPE_Y" click 1
xdotool mousemove 220 40 click 1
sleep 0.3
xdotool mousemove 275 205 click 1
sleep 0.7
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_dilation_dialog.png"
xdotool key Return
sleep 0.7
import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_created.png"
