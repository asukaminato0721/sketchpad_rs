# Sketchpad RS

Experimental Linux editor for Sketchpad 5 `.gsp` documents. It uses winit and egui for the window and controls, and wgpu with a WGSL shader for sketch geometry.

## Run

```sh
cargo run --bin sketchpad_rs -- path/to/sketch.gsp
```

The path is optional. Use the File menu to create, open, save, close, and quit. New, Open, Close, and Quit prompt about unsaved changes. The File menu imports a single PNG picture, exports the visible page as SVG or PDF, and sends it to the Linux default printer. PDF output requires `rsvg-convert`; printing also requires `lp`. The toolbar creates points, segments, circles, and literal text. Clicking a segment, circle, or arc with the Point tool creates a point constrained to that object. Select the required objects for the Construct commands. The Measure menu adds live measurements, including length, distance, area, perimeter, signed point ratio, and slope in a Cartesian coordinate system. The Number menu supports a default parameter, a literal calculation, and a one-column parameter table with row controls. Display changes supported point styles, line widths and patterns, and colors. Edit > Cut, Copy, and Paste use an internal sketch clipboard for supported objects; Edit > Clear or Delete removes selected objects and their dependents in supported single-page files. The Transform menu can translate or rotate a point, reflect a point or circle across a selected line, or dilate a point or circle around a selected center. Drag a point to update its dependents or its position on a segment, circle, or arc. Scroll over the canvas to zoom. Multi-page files have page controls in the left panel.

For a small interoperability sample, run `cargo run --bin gsp_probe -- demo /tmp/sketch.gsp`.

## Current scope

The writer supports the constructions and measurements listed in [COMMAND_PARITY.md](COMMAND_PARITY.md), including polygons, signed point ratio, slope, Cartesian `x`/`x²` plots, Hide/Show action buttons, selected-object styling, and bounded parameter/table workflows. It can update coordinates, supported point constraints, visibility, and styles in place in existing documents, and remove selected objects with their dependents in supported single-page documents. Copy includes a selected object's construction parents; Paste duplicates them with a 20-unit offset. It preserves unsupported source records byte for byte. Structural edits to a document with unsupported content are refused when saving. Saves write through a temporary file in the destination directory and rename it into place.

Other Sketchpad features are incomplete. In particular, many transformations, graphing commands, advanced image and table workflows, custom tools, action button variants, other import formats, page setup, print preview, and full multi-page editing are still pending. See [COMMAND_PARITY.md](COMMAND_PARITY.md) for the command inventory. This build is not ready as a Sketchpad 5 replacement.

## Verification

Run `cargo test`. Tests cover record parsing and byte preservation, dependency updates, geometry, undo/redo, and save/reopen. All 1,599 `.gsp` files in the sibling project and the bundled 523-object Example Tools document pass unedited byte-preservation verification; many remain partial views and are guarded against unsafe edits. Generated constructions, measurements, text, action buttons, styles, bounded number/table files, and `x`/`x²` plots have been opened and saved in the supplied Windows Sketchpad 5 through Wine. Native KDE screenshots confirm geometry and style rendering; full command and interaction verification remains open.

`gsp_probe` provides headless `inspect`, `diff`, `verify`, `demo`, `replay`, and `svg` commands for comparing original and generated files and exercising document commands. The Ghidra scripts and current binary analysis are in [dev/ANALYSIS.md](dev/ANALYSIS.md).
