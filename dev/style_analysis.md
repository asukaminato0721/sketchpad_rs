# Point-style record evidence

I opened a port-generated one-point document in the supplied Sketchpad 5 application under Wine, selected its point, and used Display > Point Style to choose Dot, Small, and Large. The default was Medium. The three edited files are `tests/fixtures/style_dot_point.gsp`, `style_small_point.gsp`, and `style_large_point.gsp`.

For all four variants, the point's `0x07d0` object-header payload byte 11 held the style: Dot `0`, Small `1`, Medium `2`, Large `3`. The document also changed unrelated preference and selection records on save. `src/style_records.rs` changes only that object-header byte and preserves the rest. Its three standalone tests pass with `rustc --edition=2024 --test src/style_records.rs`.

The native document model and UI do not yet expose this style. Integration should read the byte only for point object headers, keep unknown values untouched, and update the byte on save when the user changes a point style. Original Sketchpad also updates its default style for later objects; that document preference field has not been decoded.
