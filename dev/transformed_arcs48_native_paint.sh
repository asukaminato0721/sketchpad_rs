#!/usr/bin/env bash
set -euo pipefail
window_id="$(xdotool search --onlyvisible --name 'Sketchpad|几何' | tail -1)"
xdotool windowsize "$window_id" 900 650
xdotool key Escape
sleep 2
