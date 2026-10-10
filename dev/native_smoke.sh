#!/usr/bin/env bash
set -euo pipefail

# Capture the native window on an isolated X11 display with software rendering.
# Usage: dev/native_smoke.sh [--demo|--save-demo|sketch.gsp] [capture-prefix]
if [[ -z "${IN_NIX_SHELL:-}" ]] && command -v nix >/dev/null 2>&1; then
  exec nix develop -c bash "$0" "$@"
fi

source_arg="${1:---demo}"
capture_prefix="${2:-/tmp/sketchpad_native_smoke}"
case "$source_arg" in
  --demo)
    smoke_mode=demo
    sketch_path=--demo
    ;;
  --save-demo)
    smoke_mode=save-demo
    sketch_path="${capture_prefix}.gsp"
    ;;
  *)
    smoke_mode=sketch
    sketch_path="$(realpath "$source_arg")"
    ;;
esac

cargo build --quiet --bin sketchpad_rs
if [[ "$smoke_mode" == save-demo ]]; then
  cargo build --quiet --no-default-features --bin gsp_probe
  target/debug/gsp_probe demo "$sketch_path"
  source_hash="$(sha256sum "$sketch_path")"
  source_hash="${source_hash%% *}"
fi

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
  if [[ "$3" == demo || "$3" == save-demo ]]; then
    xdotool mousemove --window "$window_id" 497 507 \
      mousedown 1 \
      mousemove --window "$window_id" 557 465 \
      sleep 0.5 \
      mouseup 1
    changed_pixels=0
    for _ in $(seq 1 40); do
      import -window "$window_id" "${2}_dragged.png"
      changed_pixels="$(magick compare -metric AE "${2}.png" "${2}_dragged.png" null: 2>&1 || true)"
      changed_pixels="${changed_pixels%% *}"
      changed_pixels="${changed_pixels%%.*}"
      if [[ "$changed_pixels" =~ ^[0-9]+$ ]] && (( changed_pixels > 1000 )); then
        break
      fi
      sleep 0.25
    done
    [[ "$changed_pixels" =~ ^[0-9]+$ ]]
    (( changed_pixels > 1000 ))
    dirty_title="$(xdotool getwindowname "$window_id")"
    [[ "$dirty_title" == \**" — Sketchpad RS" ]]
    printf "Dragged capture: %s_dragged.png (%s changed pixels)\n" "$2" "$changed_pixels"
    if [[ "$3" == save-demo ]]; then
      xdotool windowfocus --sync "$window_id"
      xdotool key --clearmodifiers ctrl+s
      saved=false
      for _ in $(seq 1 80); do
        if [[ "$(xdotool getwindowname "$window_id")" != \** ]]; then
          saved=true
          break
        fi
        sleep 0.25
      done
      test "$saved" = true
      printf "Saved window: %s\n" "$(xdotool getwindowname "$window_id")"
    fi
  fi
' _ "$sketch_path" "$capture_prefix" "$smoke_mode"

if [[ "$smoke_mode" == save-demo ]]; then
  saved_hash="$(sha256sum "$sketch_path")"
  saved_hash="${saved_hash%% *}"
  test "$saved_hash" != "$source_hash"
  target/debug/gsp_probe verify "$sketch_path"
fi
