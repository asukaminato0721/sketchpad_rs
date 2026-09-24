# Point-style record evidence

I opened a port-generated one-point document in the supplied Sketchpad 5 application under Wine, selected its point, and used Display > Point Style to choose Dot, Small, and Large. The default was Medium. The three edited files are `tests/fixtures/style_dot_point.gsp`, `style_small_point.gsp`, and `style_large_point.gsp`.

For all four variants, the point's `0x07d0` object-header payload byte 11 held the style: Dot `0`, Small `1`, Medium `2`, Large `3`. The document also changed unrelated preference and selection records on save. `src/style_records.rs` changes only that object-header byte and preserves the rest. Its three standalone tests pass with `rustc --edition=2024 --test src/style_records.rs`.

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
