# Interface and localization

The desktop uses egui 0.36 widgets with the existing winit and wgpu runtime.
`editor/chrome.rs` defines the light theme, tool strip, page tabs, and Window and
Help menus. The geometric renderer keeps its current render pass. Moving the
runtime to eframe is a separate integration task because eframe owns the window,
surface, and render loop.

The layout uses the Sketchpad 5 reference in
`dev/custom_tools_desktop_selected_native.png`. The tool strip exposes supported
tools. Object Information is a floating panel. It retains text editing, label
editing, text-range formatting, and iteration controls from the former sidebar.

## Translations

`editor/i18n.rs` selects English or Simplified Chinese on the UI thread.
`editor/i18n/zh-CN.json` maps English interface text to Chinese. Use `tr(...)`
when presenting interface text. Do not translate document captions, page names,
formulas, file paths, or protocol identifiers. Windows use stable IDs so that
language changes do not change their identity.

The catalog also supports formatted status text. Placeholder names and counts
must agree between the English and Chinese strings. Named placeholders can be
reordered. Format specifiers refer to values that Rust has already formatted.
More specific templates take priority. Avoid adjacent placeholders and templates
that consist of a single placeholder.

Missing entries use the English text. Some engine diagnostics still use this
fallback. Native file dialogs use the desktop's language settings. Chinese text
uses the existing system CJK font fallback.

Language changes take effect in the open window. They do not change document
content or undo history. The setting is stored separately from the document in
`$XDG_CONFIG_HOME/sketchpad_rs/language.json`, with `~/.config` as the fallback
configuration directory.

## Checks

```sh
cargo test -p sketchpad-editor --lib
cargo clippy -p sketchpad-editor --all-targets -- -D warnings
cargo build --bin sketchpad_rs
xvfb-run -a -s '-screen 0 1280x800x24' python3 dev/mcp_smoke.py
xvfb-run -a -s '-screen 0 1280x800x24' python3 dev/ui_smoke.py
```

The UI smoke test needs Xvfb, xdotool, and ImageMagick. It uses an isolated
configuration directory. It checks Chinese startup, a menu language change,
configuration persistence after restart, and the point tool. It writes captures
to `/tmp/sketchpad-ui-*.png`. The menu coordinates assume the default window size
and scale on the virtual display.
