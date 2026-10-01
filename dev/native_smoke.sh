#!/usr/bin/env bash
set -euo pipefail

# Capture the native window on an isolated X11 display with software rendering.
# Usage: dev/native_smoke.sh [--demo|sketch.gsp] [capture-prefix]
if [[ -z "${IN_NIX_SHELL:-}" ]] && command -v nix >/dev/null 2>&1; then
  exec nix develop -c bash "$0" "$@"
fi

sketch_path="${1:---demo}"
capture_prefix="${2:-/tmp/sketchpad_native_smoke}"
if [[ "$sketch_path" != --demo ]]; then
  sketch_path="$(realpath "$sketch_path")"
fi

cargo build --quiet --bin sketchpad_rs

xvfb-run -a -s "-screen 0 1280x800x24" bash -c '
  set -euo pipefail
  unset WAYLAND_DISPLAY WAYLAND_SOCKET
  export XDG_SESSION_TYPE=x11
  export WINIT_UNIX_BACKEND=x11
  export WGPU_BACKEND=gl
  export LIBGL_ALWAYS_SOFTWARE=1
  target/debug/sketchpad_rs "$1" > "${2}_runtime.log" 2>&1 &
  app_pid=$!
  cleanup() {
    cleanup_status=$?
    trap - EXIT
    kill "$app_pid" 2>/dev/null || true
    wait "$app_pid" 2>/dev/null || true
    exit "$cleanup_status"
  }
  trap cleanup EXIT
  window_id="$(timeout 20 xdotool search --sync --onlyvisible --name "Sketchpad RS" | tail -1)"
  test -n "$window_id"
  window_title="Sketchpad RS"
  for _ in $(seq 1 80); do
    window_title="$(xdotool getwindowname "$window_id")"
    if [[ "$window_title" != "Sketchpad RS" ]]; then
      break
    fi
    sleep 0.25
  done
  test "$window_title" != "Sketchpad RS"
  rendered=false
  for _ in $(seq 1 40); do
    import -window "$window_id" "${2}.png"
    if [[ "$(magick "${2}.png" -format "%[fx:standard_deviation>0.001]" info:)" == 1 ]]; then
      rendered=true
      break
    fi
    sleep 0.5
  done
  test "$rendered" = true
  printf "Native window: %s\nCapture: %s.png\n" "$window_title" "$2"
' _ "$sketch_path" "$capture_prefix"
