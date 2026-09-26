#!/usr/bin/env bash
# Compare native literal and grouped exponent syntax.
set -euo pipefail
xdotool key --clearmodifiers ctrl+F10
sleep 1
for expression in '-2' '+2' '(2)' '2.0' '2/1'; do
 case "$expression" in
  '-2') name=negative ;;
  '+2') name=literal ;;
  '(2)') name=parenthesized ;;
  '2.0') name=decimal ;;
  '2/1') name=quotient ;;
 esac
 xdotool mousemove 110 85 click 1
 xdotool mousemove 346 40 click 1
 sleep 0.3
 xdotool mousemove 355 87 click 1
 sleep 0.4
 xdotool mousemove 341 232 click 1
 sleep 0.2
 xdotool mousemove 305 258 click 1
 xdotool mousemove 250 232 click 1
 if [[ "$expression" == '(2)' ]]; then
  xdotool mousemove 250 265 click 1
  xdotool type --clearmodifiers --delay 50 '2'
  xdotool mousemove 250 299 click 1
 else
  xdotool type --clearmodifiers --delay 50 -- "$expression"
 fi
 sleep 0.4
 import -window root "${SKETCHPAD_WINE_CAPTURE_PREFIX}_${name}_input.png"
 xdotool key Return
 sleep 0.5
 xdotool key --clearmodifiers ctrl+h
 sleep 0.3
done
