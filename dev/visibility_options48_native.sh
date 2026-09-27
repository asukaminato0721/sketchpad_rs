#!/usr/bin/env bash
# Execute through shared Wine helper lock; retain per-step saves and screenshots.
set -euo pipefail
export SKETCHPAD_WINEPREFIX=/home/w/.cache/sketchpad_rs_wine
for prefix in /nix/store/*wine-wow64-11.0 /nix/store/*xvfb-run-* /nix/store/*xdotool-* /nix/store/*imagemagick-7*; do
  [[ ! -d "$prefix/bin" ]] || export PATH="$prefix/bin:$PATH"
done
for state in "$@"; do
 input="/tmp/visibility48_${state}_input.gsp"
 output="/tmp/visibility48_${state}_native_work.gsp"
 cp "$input" "$output"
 export SKETCHPAD_WINE_CAPTURE_PREFIX="/tmp/visibility48_${state}_native"
 export SKETCHPAD_WINE_SETUP_SCRIPT="/tmp/visibility48_${state}_setup.sh"
 dev/wine_roundtrip.sh "$output"
 cmp "$input" "/tmp/visibility48_${state}_opened.gsp"
 echo "DONE $state"
done
