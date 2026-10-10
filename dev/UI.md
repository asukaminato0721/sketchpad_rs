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

## Object dependency inspector

Open **Window → Object Information** or click the `i` button in the tool strip.
The inspector lists the current page's objects, including hidden objects.
Each row shows the original ID, type, and label. The filter accepts an ID,
a type, or label text.

Choose **Depends on** to expand source objects. Choose **Used by** to expand
objects that depend on the row's object. Shared dependencies can appear in
several branches. All occurrences refer to the same document object.

Click a row to select that object on the canvas. Ctrl-click or Shift-click adds
or removes it from the selection. Double-click centers supported geometry in the
canvas. Hidden objects stay hidden. Use **Properties** to change their visibility.
Canvas selections highlight the corresponding rows. **Show selected** clears the
filter and scrolls to the selected root. Selection and navigation do not create
undo steps. The existing text and label controls remain below the tree.

`editor/inspector.rs` uses a tagless-final display algebra, `DependencyView`.
The generic traversal calls `object` and `notice`. Each object operation receives
a branch continuation. The interpreter decides whether to run that continuation.
The traversal does not construct a tagged UI tree. `EguiDependencyView` renders
the controls and records selection intents. The test interpreter records paths
and can skip collapsed branches without starting egui.

The graph uses `Document::dependencies_of`, including native midpoint hosts.
Reverse edges supply the **Used by** view. A branch stops at a cycle, 24 levels,
or 256 visited entries. Deleted objects are omitted. Missing references produce
a notice. Select an object in a truncated branch, then choose **Show selected**
to inspect its root separately.
The graph is rebuilt from current document state when the inspector renders, so
construction, edits, deletion, undo, and redo appear without a separate cache.
Object type labels use the stored geometry class. They do not evaluate source
coordinates. This also preserves type labels when source geometry is undefined.
Evaluating geometry for each label previously blocked the inspector on large
documents, including the 38-page physics teaching sample with 7,752 objects.

## Action button input

Action buttons use compact caption widths. Their canvas, SVG, and hit rectangles
share the same dimensions. Buttons receive clicks before caption and image bounds.
Page tabs reserve space below the labels for their horizontal scroll bar.

Each animation track records its source button. Clicking that button again stops
its tracks. A different button replaces tracks for shared targets immediately.
This lets **Reset** replace **Start** without a second click.
Parameter buttons convert native lengths and angles to displayed units. Large
bidirectional steps stay within their range. Animation preserves cached caption
boxes, including narrow boxes on hidden parameters. Existing parameters can run
in multi-page documents with opaque images. Save still validates the full file.

Presentation buttons can execute hidden child buttons, including nested
presentations. Deleted children and children on another page remain unavailable.

To check local animation, visibility, movement, and presentation buttons on every
page of a sample, run:

```sh
SKETCHPAD_ACTION_SAMPLE=/path/to/sample.gsp cargo test -p sketchpad-editor external_sample_action_buttons --lib -- --ignored --nocapture
```

The check validates caption and center hits, activation, continued animation,
parameter ranges, and a parameter save round trip that preserves images.
It does not execute URL or sound buttons.

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

For a local large-file regression, run:

```sh
SKETCHPAD_INSPECTOR_SAMPLE=/path/to/sample.gsp cargo test -p sketchpad-editor external_sample_inspector_frames --lib -- --ignored --nocapture
xvfb-run -a -s '-screen 0 1280x800x24' python3 dev/inspector_smoke.py /path/to/sample.gsp
```

The first check measures dependency metadata and inspector frames on every page.
The window check opens Object Information, checks event-loop response time, and
selects the first object. Both checks use a two-second regression limit.
The window check saves `/tmp/sketchpad-inspector-sample.png`.
