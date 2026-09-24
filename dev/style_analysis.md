# Point-style record evidence

I opened a port-generated one-point document in the supplied Sketchpad 5 application under Wine, selected its point, and used Display > Point Style to choose Dot, Small, and Large. The default was Medium. The three edited files are `tests/fixtures/style_dot_point.gsp`, `style_small_point.gsp`, and `style_large_point.gsp`.

For all four variants, the point's `0x07d0` object-header payload byte 11 held the style: Dot `0`, Small `1`, Medium `2`, Large `3`. The document also changed unrelated preference and selection records on save. `src/style_records.rs` changes only that object-header byte and preserves the rest. Its point helpers require object code `0` and subtype `4`; text objects also use code `0` but have subtype `19`, so they are rejected. Run its standalone tests with `rustc --edition=2024 --test src/style_records.rs`.

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

The low nibble is the width (0–3) and the high nibble is the pattern (0 dotted, 1 dashed, 2 solid) for these observed variants. `src/style_records.rs` reads and updates the byte for code-2 segment headers only, preserving every other header byte. The five standalone style tests and all six line fixtures pass. The native model, drawing code, and menus still need to expose line styles; line, ray, circle, and arc codes have not been probed for this field.

## Point color

Changing the same original-app point from red to orange and blue changed only its `0x07d0` payload bytes 12–14 among the object records. They are direct RGB bytes: red `(255, 0, 0)`, orange `(255, 128, 0)`, blue `(0, 0, 255)`. `tests/fixtures/style_orange_point.gsp` and `style_blue_point.gsp` preserve the original saves. `PointColor` in `src/style_records.rs` reads and updates these three bytes for point headers; the sixth standalone test checks both original variants and lossless editing. Other object color fields have not been probed.

`dev/style_record_probe.rs` applies the observed fields to a GSP copy. A Rust-patched blue point opened, displayed blue, and saved in Sketchpad 5. A Rust-patched thick dashed segment likewise opened, displayed the expected line, and saved. Both saved files passed the port's byte-exact `gsp_probe verify` check.

The same RGB field occurs in a code-2 segment header: the port's default segment was navy `(0, 0, 128)`, and Display > Color > Blue changed its object-header bytes 12–14 to `(0, 0, 255)`. `tests/fixtures/style_blue_line.gsp` is the original-app save. The `segment_color` helpers are restricted to code-2 headers; other object codes need their own verification.

A code-3 circle also uses payload bytes 12–14: the generated default circle was green `(0, 128, 0)`, and the original Display > Color > Blue saved `(0, 0, 255)` in `tests/fixtures/style_blue_circle.gsp`. The Rust `circle_color` helper then produced a blue-circle file that opened, displayed blue, and saved in Sketchpad 5. The helper rejects other object codes.

## Circle line style

I selected a circle boundary in the original application and used Display > Line Style. Choosing Thick changed its code-3 `0x07d0` payload byte 10 from `0x22` to `0x23`; choosing Dashed changed it from `0x22` to `0x12`. Original saves are `tests/fixtures/style_thick_circle.gsp` and `style_dashed_circle.gsp`. This agrees with the segment width and pattern nibbles. `circle_line_style` and `set_circle_line_style` decode and edit this field only for code-3 headers. Native editing and rendering remain to be integrated.

`dev/style_record_probe.rs circle-line` generated a Thick, Solid circle from the green-circle fixture. Sketchpad 5 opened it, displayed a thick green circle, saved it, and `gsp_probe verify` reopened the save with three supported objects and `partial=false`.

## Lines, rays, and arcs

The original line, ray, three-point arc, and circle-based arc fixtures all start with `0x22` at the same object-header byte 10. `straight_or_arc_line_style` handles codes 63, 64, 79, and 81, leaving other object codes untouched. Rust-patched Thick, Dashed files for a line and a three-point arc opened and displayed as thick dashed strokes in Sketchpad 5; the original application saved both, and `gsp_probe verify` reopened them as supported files. The ray and circle-based arc share the header pattern, but their styled variants have not yet been cross-opened.

The same four headers contain RGB bytes 12–14. The source line and ray are navy `(0, 0, 128)`, while both source arcs are green `(0, 128, 0)`. `straight_or_arc_color` reads and writes the bytes only for those four codes. A Rust-patched blue three-point arc opened, displayed blue, and saved in Sketchpad 5; `gsp_probe verify` reopened its original-app save with `partial=false`.

## Polygon interior color

The code-8 polygon interior header has RGB `(255, 128, 0)` in payload bytes 12–14. `polygon_color` edits only those three bytes for code-8 headers. A Rust-patched blue polygon opened as a blue filled triangle in Sketchpad 5, saved, and reopened in the port with five supported objects and `partial=false`. The source polygon's label and perimeter measurement remained intact.
