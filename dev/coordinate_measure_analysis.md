# Cartesian coordinate measurements in original Sketchpad 5

## Original menu and records

The base sketch is `tests/fixtures/coordinate_measure_base.gsp`, derived from
the original Graph > Define Coordinate System result. It has point 1 at
`(150,120)`, point 2 at `(250,120)`, frame origin 4 at `(199,125)`, and
coordinate-system object 9. The axis unit is `37.79527559055118` screen
units. The Wine/Xvfb menu probe selected point 1 and clicked Measure at
`x=287`, then the following menu positions. For Coordinate Distance, it also
selected origin point 4 with Shift.

| Command | Menu Y | Object code | Dependencies (`0x07d2`) | Other records | Original result |
| --- | ---: | ---: | --- | --- | --- |
| Coordinates | 257 | 49 | point 1, frame 9 | `0x08fc` rich text, `0x07d8` style | `A: (−1.30, 0.13)` |
| Abscissa (x) | 274 | 65 | point 1, frame 9 | `0x0903` format, `0x07d8` style | `x_A = −1.30` |
| Ordinate (y) | 291 | 66 | point 1, frame 9 | `0x0903` format, `0x07d8` style | `y_A = 0.13` |
| Coordinate Distance | 308 | 86 | point 1, point 4, frame 9 | `0x0903` format, `0x07d8` style | `AB = 1.30` |

The corresponding original files are `coordinate_measure_menu257.gsp`,
`coordinate_measure_menu274.gsp`, `coordinate_measure_menu291.gsp`, and
`coordinate_measure_menu308.gsp`. Each added one object and a page-level
`0x2328` record holding object ID 10; the existing `0x232d` still holds
frame ID 9. An older sibling fixture, `../gsp-rs/tests/fixtures/gsp/xy_cood.gsp`,
independently uses codes 49, 65, and 66. The sibling `Γ函数.gsp` sample uses
code 86 with three dependencies.

The frame Y axis points upward in graph coordinates. For point 1, the
measured coordinates are `(150−199)/37.79527559 = −1.2965` and
`(125−120)/37.79527559 = 0.1323`. The point-to-origin distance is
`hypot(−1.2965, 0.1323) = 1.3032`. The original rounds all three to two
decimals.

## Dynamic behavior

Dragging point 1 from `(150,120)` to `(170,100)` in the original changed the
Coordinates readout to `A: (−0.77, 0.66)` and Coordinate Distance to
`AB = 1.01`. The edited original files are
`coordinate_measure_coordinates_moved.gsp` and
`coordinate_measure_distance_moved.gsp`. The measurement dependencies remain
the same. `coordinate_measure_records::evaluate` inverts the full two-axis
frame and calculates values from the current point positions. It rejects a
degenerate frame.

The `0x08fc` rich text record in the Coordinates group contains a stale
formatted value and a literal point name. After a Rust-generated file
remapped its dependency from point 1 to point 2, the original displayed
`A: (1.35, 0.13)` while the file still contained `−1.30, 0.13`. The original
recalculates the displayed numbers from dependencies on load. This cached
text must not be used for geometry evaluation. A port writer should update
the literal name when it assigns a different point label.

## Independent implementation and writer proof

`src/coordinate_measure_records.rs` decodes the four observed object groups,
validates record shape and references, evaluates each value, and encodes an
original-template group while changing its dependencies. Unknown layouts are
rejected. It is intentionally independent of the shared document model while
image work is integrated. `dev/coordinate_measure_generate.rs` creates a
ten-object file by appending one encoded group to the original nine-object
base, updating the page object count and `0x2328` marker.

The four Rust-generated files in `tests/fixtures/coordinate_measure_generated_*.gsp`
remap the point dependency to point 2. All four opened and saved in the
original application. The original displayed `1.35` for Abscissa, `0.13` for
Ordinate, `(1.35, 0.13)` for Coordinates, and `1.36` for distance to origin.
The saved copies are in `/tmp/coordinate_measure_generated_*_wine.gsp`.

Run focused tests and regenerate a sample with:

```sh
rustc --edition 2024 --test src/coordinate_measure_records.rs -o /tmp/coordinate_measure_records_tests
/tmp/coordinate_measure_records_tests
rustc --edition 2024 dev/coordinate_measure_generate.rs -o /tmp/coordinate_measure_generate
/tmp/coordinate_measure_generate abscissa 2 /tmp/coordinate_abscissa.gsp
```

The shared document now recognizes codes 49/65/66/86 and evaluates their
dependencies against `Document::graph_frame`. The native Measure menu, replay
commands (`coordinates`, `abscissa`, `ordinate`, `coordinate_distance`), SVG,
and native readouts use two-decimal live values. The writer uses the original
record templates, keeps unmodified source groups byte-exact, and maintains the
page-level `0x2328` marker. The replay in
`tests/replay_coordinate_measurements.txt` generated a 13-object file with
all four commands, then moved point 1 and saved a second file. Both reopen in
the port with `partial=false`.

`tests/fixtures/coordinate_measure_port_generated.gsp` opened in the original
Sketchpad 5 through Wine. Its screenshot at
`/tmp/coordinate_measure_port_crossopen_before_save.png` shows the four live
readouts: `A: (−1.30, 0.13)`, `x_C = 1.35`, `y_C = 0.13`, and `AB = 1.30`.
The original saved the file unchanged byte-for-byte (3,088 bytes), and the
port reopened that saved copy with all 13 objects and `partial=false`.
The original resolves implicit point names independently for scalar readouts;
the native UI currently assigns an ordinal fallback when a point has no
explicit label, so its fallback subscript can differ from Sketchpad's.

This bounded integration does not cover nondefault rich-text names, custom
display formats, non-Cartesian systems, or measurement tables. A new
Coordinates readout with a multi-character point name refuses serialization
because the observed rich-text template carries one ASCII letter.

## Legacy polar coordinate values

The corpus also contains scalar coordinate groups with the same two references
and measurement structure but legacy codes 125 and 124. Native inspection
establishes that these are not alternate Cartesian x/y records: code 125 is the
radius `hypot(x, y)`, and code 124 is the polar angle `atan2(y, x)`. Their
meaning is independent of whether the coordinate system currently displays a
square, rectangular, or polar grid. The angle is evaluated internally in
degrees and formatted according to the document angle unit.

In the original `圆环的迭代(5环旋转迭代).gsp`, Sketchpad 5 displays
`r_z = 5.71` and `θ_z = 0.04`; that document uses radians. A pair written by
the port from `coordinate_measure_base.gsp` displayed `r_A = 1.36` and
`θ_A = 5.60°` in the degree-mode base document. Sketchpad 5 resaved the
2,796-byte authored file without changing a byte. Moving the source point in
the port updates both values and their six dependent calculations.

Admission remains narrow: an exact scalar-measurement layout, one supported
Point reference, and one supported CoordinateSystem reference. Three other
roots depend on unsupported code-88 aliases and remain opaque. The final corpus
scan restores two roots and six descendants while preserving all 1,599 source
files byte-for-byte; see
`/tmp/sketchpad_legacy_polar_coordinates_final_coverage.log`.

## Legacy pair-readout cache tail

Twenty later-ready Cartesian pair readouts use the established code-49,
class-0/subtype-34 layout with supported constructed points and frames. Their
`0x08fc` inner size excludes exactly two trailing opaque bytes. This is the same
bounded size convention already accepted for line/circle equation text. The
tail varies independently of the displayed pair and remains byte-preserved.

The shared parser now accepts only a zero- or two-byte difference between the
record payload and its inner size. Numeric-coordinate, curve-intersection,
iteration-alias, and colorized point sources then use the existing live frame
evaluation. The remaining code-49 object still references an unsupported
code-69 point. The compiled corpus restores 21 drawing slots, reaches 2,639
unsupported slots and 1,277/1,307 complete documents/drawings, and preserves all
1,599 inputs exactly. Log: `/tmp/sketchpad_legacy_coordinate_tail_full_coverage.log`.
