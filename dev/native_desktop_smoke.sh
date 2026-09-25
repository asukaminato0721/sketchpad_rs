#!/usr/bin/env bash
set -euo pipefail

# Run against the current Wayland desktop when an Xvfb surface has no wgpu adapter.
# The binary needs the runtime libraries declared by the development shell.
if [[ -z "${IN_NIX_SHELL:-}" ]] && command -v nix >/dev/null 2>&1; then
  exec nix develop -c bash "$0" "$@"
fi

sketch_path="$(realpath "${1:-tests/fixtures/point.gsp}")"
capture_prefix="${2:-/tmp/sketchpad_native_desktop}"
target/debug/sketchpad_rs "$sketch_path" > "${capture_prefix}_runtime.log" 2>&1 &
app_pid=$!
trap 'kill "$app_pid" 2>/dev/null || true' EXIT
sleep 6
kill -0 "$app_pid"
if [[ "${XDG_CURRENT_DESKTOP:-}" == *KDE* ]] && command -v spectacle >/dev/null; then
  spectacle --background --nonotify --activewindow --output "${capture_prefix}.png"
else
  grim "${capture_prefix}.png"
fi
