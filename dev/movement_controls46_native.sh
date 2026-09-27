#!/usr/bin/env bash
# Reproduce one completed Native probe using generated /tmp movement46_STATE_input.gsp.
set -euo pipefail
state="$1"
mode="${2:-click}"
export SKETCHPAD_WINEPREFIX="${SKETCHPAD_WINEPREFIX:-/home/w/.cache/sketchpad_rs_wine}"
input="/tmp/movement46_${state}_input.gsp"
output="/tmp/movement46_${state}_${mode}.gsp"
cp "$input" "$output"
export SKETCHPAD_WINE_CAPTURE_PREFIX="/tmp/movement46_${state}_${mode}"
if [[ "$mode" == "click" ]]; then
  export SKETCHPAD_WINE_ACTION_CLICK_ONLY=1
  export SKETCHPAD_WINE_ACTION_CLICK_X=95
  export SKETCHPAD_WINE_ACTION_CLICK_Y=84
  export SKETCHPAD_WINE_ACTION_WAIT_SECONDS=12
fi
dev/wine_roundtrip.sh "$output"
if [[ "$mode" == "open" ]]; then
  cmp "$input" "$output"
fi
