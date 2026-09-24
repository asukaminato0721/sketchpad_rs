# Point-style record evidence

The bundled Reference Center describes Point Style as Dot, Small, Medium, and **Thick**; the byte-3 variant called `Large` in the current Rust enum corresponds to the menu's Thick choice. It also says a style change updates the default for future similar objects unless Shift is held (or the related preference is disabled). Line Style has independent Hairline/Thin/Medium/Thick widths and Solid/Dashed/Dotted patterns. Polygon frames allow width changes but no pattern changes. Color changes similarly update the default for new objects of the same kind. The first native integration should implement selected-object styles and retain the original default fields until those preference records are understood; this is a remaining parity gap, not verified behavior.

Comparing the original red, orange, and blue point saves shows a six-byte `0x2724` record with index `0` changing from `00 00 ff 00 00 00` to `00 00 ff 80 00 00` or `00 00 00 00 ff 00`. The same RGB change appears in the point header. This record likely holds a configurable palette slot or a current color default, but its role needs a new-object probe before the writer changes it. The style-only saves did not reveal an equally clear document default field.

I opened a port-generated one-point document in the supplied Sketchpad 5 application under Wine, selected its point, and used Display > Point Style to choose Dot, Small, and Large. The default was Medium. The three edited files are `tests/fixtures/style_dot_point.gsp`, `style_small_point.gsp`, and `style_large_point.gsp`.

For all four variants, the point's `0x07d0` object-header payload byte 11 held the style: Dot `0`, Small `1`, Medium `2`, Large `3`. The document also changed unrelated preference and selection records on save. `src/style_records.rs` changes only that object-header byte and preserves the rest. Its point helpers recognize known point code/subtype pairs for free, midpoint, constrained, transformed, and intersection points; text objects also use code `0` but have subtype `19`, so they are rejected. The derived-point headers share the same default style and RGB bytes; editing them in the original menu has not yet been probed. Run the standalone tests with `rustc --edition=2024 --test src/style_records.rs`.

`tests/replay_style_midpoint.txt` creates a visible midpoint. `dev/style_record_probe.rs point-code-style` changed only the midpoint's code-1 header to Large and blue. Sketchpad 5 displayed the enlarged blue midpoint between the original red endpoints, saved it as `tests/fixtures/style_generated_large_blue_midpoint.gsp`, and `gsp_probe verify` reopened the original-app save with `partial=false`.

The native document model and UI do not yet expose this style. Integration should read the byte only for point object headers, keep unknown values untouched, and update the byte on save when the user changes a point style. Original Sketchpad also updates its default style for later objects; that document preference field has not been decoded.

## Segment line style

For a port-generated segment, I used Display > Line Style in the original application and saved each variant in `tests/fixtures/style_*_line.gsp`. The `0x07d0` segment-header payload byte 10 changed as follows:

| Variant | Byte 10 |
| --- | --- |
| Hairline, Solid | `0x20` |
| Thin, Solid | `0x21` |
| Medium, Solid | `0x22` |
| Thick, Solid | `0x23` |
| Medium, Dashed | `0x12` |
| Medium, Dotted | `0x02` |

The low nibble is the width (0–3) and the high nibble is the pattern (0 dotted, 1 dashed, 2 solid) for these observed variants. `src/style_records.rs` reads and updates the byte for code-2 segment headers, preserving every other header byte. Circle, line, ray, and arc variants are described below. The native model, drawing code, and menus still need to expose line styles.

## Point color

Changing the same original-app point from red to orange and blue changed only its `0x07d0` payload bytes 12–14 among the object records. They are direct RGB bytes: red `(255, 0, 0)`, orange `(255, 128, 0)`, blue `(0, 0, 255)`. `tests/fixtures/style_orange_point.gsp` and `style_blue_point.gsp` preserve the original saves. `RgbColor` and the point helpers in `src/style_records.rs` read and update these three bytes for point headers.

`dev/style_record_probe.rs` applies the observed fields to a GSP copy. A Rust-patched blue point opened, displayed blue, and saved in Sketchpad 5. A Rust-patched thick dashed segment likewise opened, displayed the expected line, and saved. Both saved files passed the port's byte-exact `gsp_probe verify` check.

The same RGB field occurs in a code-2 segment header: the port's default segment was navy `(0, 0, 128)`, and Display > Color > Blue changed its object-header bytes 12–14 to `(0, 0, 255)`. `tests/fixtures/style_blue_line.gsp` is the original-app save. The `segment_color` helpers are restricted to code-2 headers; other object codes need their own verification.

A code-3 circle also uses payload bytes 12–14: the generated default circle was green `(0, 128, 0)`, and the original Display > Color > Blue saved `(0, 0, 255)` in `tests/fixtures/style_blue_circle.gsp`. The Rust `circle_color` helper then produced a blue-circle file that opened, displayed blue, and saved in Sketchpad 5. The helper rejects other object codes.

## Circle line style

I selected a circle boundary in the original application and used Display > Line Style. Choosing Thick changed its code-3 `0x07d0` payload byte 10 from `0x22` to `0x23`; choosing Dashed changed it from `0x22` to `0x12`. Original saves are `tests/fixtures/style_thick_circle.gsp` and `style_dashed_circle.gsp`. This agrees with the segment width and pattern nibbles. `circle_line_style` and `set_circle_line_style` decode and edit this field only for code-3 headers. Native editing and rendering remain to be integrated.

`dev/style_record_probe.rs circle-line` generated a Thick, Solid circle from the green-circle fixture. Sketchpad 5 opened it, displayed a thick green circle, saved it, and `gsp_probe verify` reopened the save with three supported objects and `partial=false`.

## Lines, rays, and arcs

The original line, ray, three-point arc, and circle-based arc fixtures all start with `0x22` at the same object-header byte 10. `straight_or_arc_line_style` handles codes 63, 64, 79, and 81, leaving other object codes untouched. Rust-patched Thick, Dashed files for a line and a three-point arc opened and displayed as thick dashed strokes in Sketchpad 5; the original application saved both, and `gsp_probe verify` reopened them as supported files. The ray and circle-based arc share the header pattern, but their styled variants have not yet been cross-opened.

The same four headers contain RGB bytes 12–14. The source line and ray are navy `(0, 0, 128)`, while both source arcs are green `(0, 128, 0)`. `straight_or_arc_color` reads and writes the bytes only for those four codes. A Rust-patched blue three-point arc opened, displayed blue, and saved in Sketchpad 5; `gsp_probe verify` reopened its original-app save with `partial=false`.

The supplied and sibling fixtures for center-and-segment circles (code `4`), scaled circles (`30` with subtype `28`), reflected circles (`34` with subtype `28`), perpendicular lines (`5`), parallel lines (`6`), and angle bisectors (`7`) show the same `0x22` line-style and RGB fields. `visual_style` now recognizes those headers; a fixture test changes only byte 10 and bytes 12–14, then restores the exact original header. The perpendicular example was copied from `../gsp-rs/tests/fixtures/gsp/static/perpendicular.gsp` into the local corpus.

## Polygon interior color

The code-8 polygon interior header has RGB `(255, 128, 0)` in payload bytes 12–14. `polygon_color` edits only those three bytes for code-8 headers. A Rust-patched blue polygon opened as a blue filled triangle in Sketchpad 5, saved, and reopened in the port with five supported objects and `partial=false`. The source polygon's label and perimeter measurement remained intact.

## Native integration and cross-open

`Object.visual_style` now holds decoded point, stroke, or polygon style. Source and newly constructed objects keep that state through undo/redo, copy/paste, and save/reopen. The writer patches only the recognized object-header fields after other object-group edits, validates the reparse before atomic replacement, and refuses a style/type mismatch. Display provides Point Style, Line Width, Line Pattern, and Color controls. WGSL geometry rendering and SVG export use those same styles. The generated graph origin has subtype `41` rather than the free-point subtype `4`; it is now recognized as a styled point, which keeps graph save/reopen intact.

`tests/replay_styles.txt` builds a six-object sketch: thick blue point, thick green dashed segment, thin blue dotted circle, and purple polygon interior. `tests/fixtures/style_integrated_port.gsp` is the port output. The supplied Sketchpad 5 opened it in Wine, displayed all four styles, and saved it. The saved file is `style_integrated_saved_by_wine.gsp`; `cmp` confirmed that it is byte-identical to the port output. The port reopened it with six supported objects and `partial=false`. `/tmp/sketchpad_style_integrated_before_save.png` records the original UI, and `/tmp/sketchpad_native_style.png` records the native KDE window. SVG output contains the expected RGB, width, dash, and point-radius attributes.

Remaining Display parity includes defaults for future objects, text style, tracing, animation, and exact polygon frame behavior. The native drawing uses an approximate pixel size for the four point and line widths; the record values and save format are verified against the original.
