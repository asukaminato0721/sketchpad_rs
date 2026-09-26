#!/usr/bin/env bash
# Native calculator probes; see dimensional_roots_analysis.md for matching bases.
set -euo pipefail
xdotool key --clearmodifiers ctrl+F10
sleep 1
case "$SKETCHPAD_WINE_CAPTURE_PREFIX" in
 *units*) specs=(area_one:85:1 radius_two:113:2 radius_sqrt:113:sqrt angle_sqrt:141:sqrt angle_two:141:2) ;;
 *fractional*) specs=(area_fractional:85:0.5) ;;
 *) specs=(area_sqrt:85:sqrt) ;;
esac
for spec in "${specs[@]}"; do
 IFS=: read -r name y operation <<< "$spec"
 xdotool mousemove 110 "$y" click 1
 xdotool mousemove 346 40 click 1
 sleep 0.3
 xdotool mousemove 355 87 click 1
 sleep 0.4
 if [[ "$operation" == sqrt ]]; then
  xdotool mousemove 341 265 click 1
  sleep 0.2
  xdotool mousemove 300 371 click 1
 fi
 xdotool mousemove 341 232 click 1
 sleep 0.2
 xdotool mousemove 305 258 click 1
 if [[ "$operation" == sqrt ]]; then
  xdotool mousemove 250 299 click 1
 else
  xdotool mousemove 250 232 click 1
  xdotool type --clearmodifiers --delay 50 "$operation"
 fi
 sleep 0.4
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_input.png"
 xdotool key Return
 sleep 0.5
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_created.png"
 xdotool key --clearmodifiers ctrl+h
 sleep 0.3
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_hidden.png"
done
