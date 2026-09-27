#!/usr/bin/env bash
set -euo pipefail
export PATH=/nix/store/gyyyvd2n6gi6r4k7viincglr0bi5srzd-wine-wow64-11.0/bin:/nix/store/5p6sffnrasbn2dr6wlqsrfg3qzcqwpkk-xvfb-run-1+g87f6705/bin:/nix/store/lck8j0ci1qqbhqhqcmwh9w4nj4xans2n-xdotool-4.20260303.1/bin:/nix/store/i22x77kkpiss54gc4xrddx5kv7bk8inf-imagemagick-7.1.2-31/bin:$PATH
export SKETCHPAD_WINEPREFIX=/home/w/.cache/sketchpad_rs_wine
for state in "$@"; do
 input="/tmp/iteration_binding47_${state}_input.gsp"
 output="/tmp/iteration_binding47_${state}_saved.gsp"
 cp "$input" "$output"
 SKETCHPAD_WINE_CAPTURE_PREFIX="/tmp/iteration_binding47_${state}" dev/wine_roundtrip.sh "$output"
 cmp "$input" "$output"
 cp "$output" "tests/fixtures/iteration_binding47_${state}_native_verified.gsp"
 capture="/tmp/iteration_binding47_${state}_before_save.png"
 if [[ -n "${SKETCHPAD_WINE_SETUP_SCRIPT:-}" ]]; then capture="/tmp/iteration_binding47_${state}_after_setup.png"; fi
 cp "$capture" "tests/fixtures/iteration_binding47_${state}_native_verified.png"
 echo "DONE $state"
done
