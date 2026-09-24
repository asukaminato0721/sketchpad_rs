# Value of Point and Equation in original Sketchpad 5

## Original commands and object groups

The original Measure menu places **Value of Point** at Y 231 and **Equation**
at Y 342 in the Wine/Xvfb window used by `dev/wine_roundtrip.sh`.

| Command and selected object | Code | References (`0x07d2`) | Other records | Original display |
| --- | ---: | --- | --- | --- |
| Value of Point, point on segment | 94 | constrained point 4, segment 3 | `0x0903` 32-byte format, `0x07d8` style | `0.50` |
| Value of Point, point on circle | 94 | constrained point 4, circle 3 | same record kinds | `0.38` for a 2.418-radian screen-angle position |
| Equation, line | 50 | line 3, Cartesian frame 9 | `0x08fc` rich text, `0x07d8` style | `y = −0.50x − 0.52` |
| Equation, vertical line | 50 | line 3, Cartesian frame 9 | rich-text variant | `x = −1.30` |
| Equation, circle | 51 | circle 3, Cartesian frame 9 | longer rich-text variant | `(x + 1.30)² + (y − 0.13)² = 1.85²` |

Value of Point for a free point is disabled. Equation for a selected segment
is disabled even with a visible Cartesian frame; its entire segment was
selected in the probe. A sloped line and a circle enabled Equation. The first
line probe hit an overlapping graph point and left Equation disabled; selecting
the line farther from that point produced code 50. These are observed menu
rules, not inferred file limitations.

The minimal source files are `value_equation_point_value_original.gsp`,
`value_equation_line_original.gsp`, `value_equation_vertical_original.gsp`, and
`value_equation_circle_original.gsp`. A copy of the sibling project's original
circle Value of Point sample is
`value_equation_circle_point_value_original.gsp`. Original object groups use
the same five-record shape: `0x07d0`, `0x07d2`, `0x0903` or `0x08fc`,
`0x07d8`, `0x07d7`. The command appends `0x2328` holding its new object ID.
Equation documents retain the `0x232d` link to their coordinate system.

## Dynamic evaluation

Dragging the segment's constrained point from `(200,120)` to `(220,120)`
changed Value of Point from `0.50` to `0.70`. For a circle, the observed
constraint unit vector has screen angle about `2.418` radians and Value of
Point displays `2.418/(2π) = 0.38`. The value is a normalized turn, not a
degree or radian readout.

The default Cartesian frame has origin `(199,125)` and unit length
`37.79527559` screen pixels. For line endpoints `(150,120)` and `(250,170)`,
the original showed `y = −0.50x − 0.52`. Dragging the second endpoint to
`(270,150)` changed it to `y = −0.25x − 0.19`. A vertical line displayed
`x = −1.30`. The circle centered at `(150,120)` with a radius point at
`(220,120)` showed radius `1.85`; moving that point to `(240,120)` changed
the equation to radius `2.38`. Edited original files are
`value_equation_point_value_moved.gsp`, `value_equation_line_moved.gsp`, and
`value_equation_circle_moved.gsp`.

`src/value_equation_records.rs` evaluates the line in full axis coordinates
and handles vertical lines. Its circle evaluator requires perpendicular axes
with equal unit lengths, the observed default frame. It rejects skewed or
unequally scaled frames because a screen circle then becomes a conic in graph
coordinates. The rich-text records contain cached formula text; numerical
evaluation uses live geometry, not that cache.

## Independent writer proof

`encode_group` copies each original group template and changes only its two
dependencies. `append_to_base` validates object kinds, appends one group,
updates the page count, and inserts the `0x2328` marker. It deliberately
rejects Equation on a segment.

`dev/value_equation_generate.rs` built three files with a new free point
before the measured objects. This shifted Value of Point references from
`[4,3]` to `[5,4]` and equation references from `[3,9]` to `[4,10]`.
`value_equation_generated_point.gsp`, `value_equation_generated_line.gsp`,
and `value_equation_generated_circle.gsp` all opened, displayed the expected
value/formula, and saved in original Sketchpad 5 under Wine. Their saved
copies are in `/tmp/value_equation_generated_*_wine.gsp`.

Six standalone tests verify original decoding, remapped encoding, edited
original files, dependent evaluation, vertical lines, degenerate geometry,
and refusal of unsupported hosts. Run with:

```sh
rustc --edition 2024 --test src/value_equation_records.rs -o /tmp/value_equation_records_tests
/tmp/value_equation_records_tests
rustc --edition 2024 dev/value_equation_generate.rs -o /tmp/value_equation_generate
/tmp/value_equation_generate line tests/fixtures/value_equation_generated_line_base.gsp 4 10 /tmp/line_equation.gsp
```

## Native integration and cross-open

The document parser and writer now recognize codes 94, 50, and 51 for the
verified hosts. The native Measure menu and `gsp_probe replay` create these
objects; native and SVG readouts evaluate them from live geometry. Two
integration tests cover parsing, writing, dragging, and save/reopen. The
scripts `tests/replay_value_of_point.txt`, `tests/replay_line_equation.txt`,
and `tests/replay_circle_equation.txt` generated six port files, including
edited copies. All reopen with `partial_view=false`.

The three unedited port files opened in the supplied Sketchpad 5 under Wine.
It displayed `0.50`, `y = −0.50x − 0.52`, and
`(x + 1.30)² + (y − 0.13)² = 1.85²`, then saved each file byte-for-byte
unchanged. Captures are `/tmp/value_equation_port_{point,line,circle}_wine_before_save.png`.
The native KDE screenshot `/tmp/sketchpad_native_equation.png` shows the live
line equation above the rendered line and grid without a runtime error.
The original point-value source file contains an additional document-level
font record and remains a partial view; its generated counterpart is a full
view. Arc point values, custom equation labels, skewed frames, and
conic equations remain unprobed.
