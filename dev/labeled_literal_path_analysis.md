# Native labeled literal points on paths

A read-only corpus scan found 140 labeled code-123 scene groups in 53 files. Their
hosts include 47 segment, 26 circle, 25 polygon, and 11 arc groups. This slice
adds the proved presentation suffix while retaining the existing supported
host whitelist; it does not infer arc or other missing path geometry.

The authentic source is
`../gsp-rs/tests/fixtures/未实现的系统功能/给定的数值在路径上绘制点.gsp`.
All six other objects already decode. Object 4 is a literal 0 on circle 3,
and object 8 is literal 0.5 on segment 7. Both were rejected solely because
the parser required exactly 80 bytes and the decoder omitted code 123 from its
point-label whitelist. Each actual group is 124 bytes:

1. 28-byte class-1 point header (`0x07d0`).
2. One path reference (`0x07d2`, payload 8 bytes).
3. Code 123 and finite `f64` literal (`0x07d3`, payload 12 bytes).
4. `LabelAux` (`0x07d5`, payload 26 bytes here).
5. Caption font index (`0x07d8`, payload 2 bytes).
6. Terminator (`0x07d7`, empty payload).

Native opened and saved the authentic source byte-identically at 2620 bytes.
Its original geometry lies outside the small default viewport. A separate
control changes only the four free-point coordinate payloads: circle center
(130,150) and radius point (170,150), segment endpoints (130,230), (290,230).
Native again saved all 2620 bytes unchanged. Its visible labeled C is at
(170,150), the circle's literal 0; F is at (210,230), the segment's literal 0.5.
This establishes native geometry and labeled presentation independently of
mere file acceptance. The native group extracts and both whole files are
retained as `labeled_literal_{circle,segment}_native.bin` and
`labeled_literal_path_{original,control}_native_verified.gsp`.

The parser permits only the optional typed label and mandatory two-byte font
in this position. Truncated labels, invalid name counts, missing/wrong-size
font records, extra suffix records, and reordered constraints remain rejected.
The document validates the font against its actual font table. Labels preserve
opaque prefix/tail bytes. Literal values, host guards, geometry, styles, and
unknown-record save restrictions are unchanged.

New literal-point labels are emitted after the constraint. Adding a label to a
saved bare literal point also inserts it before the font/terminator, rather
than between its header and references. Existing labeled imports use the
bounded label/font patcher. This placement is required by the native schema.

`tests/labeled_literal_path_replay.txt` exercises actual public APIs: source
coordinate edits, CircleValue/HalfValue renaming, a bold toggle, Arial family,
copying both literal points with their eight dependencies, labeling a saved
bare literal, and labeling a fresh appended literal. Each output successfully
opened and saved unchanged in native Sketchpad before byte comparison:

| State | Objects | Bytes |
| --- | ---: | ---: |
| Source and caption edits | 8 | 2714 |
| Copied dependency graph | 16 | 3556 |
| Saved-bare and newly appended labels | 18 | 3832 |

The three retained `labeled_literal_path_{edited,copied,new_label}_native_verified.gsp`
files are compared with public API output in a regression. Native captures at
`/tmp/labeled_literal_path_{edited,copied,new_label}_port_before_save.png` show
renamed captions, copied fonts and positions offset by (20,20), and both newly
labeled literals. Tests also cover exact unedited save, native positions,
caption/font import, proper record order for both writer paths, and undo/redo.
