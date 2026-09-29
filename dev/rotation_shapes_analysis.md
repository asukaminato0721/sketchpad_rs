# Fixed and marked-angle rotation of segments and circles

Sketchpad 5 native files in `tests/fixtures/fixed_rotation_segment_original.gsp`,
`fixed_rotation_circle_original.gsp`, `angle_rotation_segment_original.gsp`,
and `angle_rotation_circle_original.gsp` establish the shape records. Fixed
rotation uses code 27, references `[source shape, center point]`, and the same
36-byte angle payload as a fixed rotated point. Marked-angle rotation uses
code 28, references `[source shape, center, angle start, vertex, angle end]`,
and has no constraint payload. Both codes use subtype 14 for segments and 28
for circles, with transformed-object marker bytes `51 5c`.

The port rotates both live segment endpoints or a circle's live center around
the selected or marked center. A circle retains its radius. The editor and
headless replay commands accept either shape for fixed or three-point marked
angles. Native fixtures import without partial view and re-encode identically.

`tests/transform_fixed_rotation_segment_replay.txt`,
`tests/transform_fixed_rotation_circle_replay.txt`,
`tests/transform_marked_angle_segment_replay.txt`, and
`tests/transform_marked_angle_circle_replay.txt` generate port files and move
the center or an angle point. Sketchpad 5 opened and saved all four moved
files byte-identically. Core and editor tests cover both shape variants and
their live dependencies.

## Numeric measured or calculated angles

Code 29 supports numeric-angle rotations of segments, circles, and polygon interiors as well as points. Its references are `[source shape, center point, degree-valued numeric object]`. Native subtype 14 uses family 2 for segments, subtype 28 uses family 3 for circles, and the polygon subtype follows vertex count with family 4 and transformed-interior flag 1. All three shapes use the transformed-object marker `51 5c`; there is no constraint payload. Their complete 68-byte native object groups equal the port-generated groups.

The new typed shape variants evaluate live angle measurements, arc-angle measurements, or degree-valued calculations. Segments rotate both endpoints; circles rotate their center and keep their radius; polygon vertices rotate while retaining fill style. The desktop Mark Angle → Rotate flow, object picking, GPU family dispatch, SVG geometry/colors, styling, copy/paste dependency remapping, deletion, and save/reopen cover these variants. Scalar angle inputs are rejected. Undefined numeric sources suppress geometry and recover when restored.

`tests/replay_numeric_angle_shapes.txt` creates three bases, measured-angle rotations before and after a 90°→135° edit and color change, and a calculated-angle segment with its length before and after editing n=4→8 and the source endpoint. Native captures show 360°/n changing from 90° to 45° and the transformed length changing from 1.06 cm to 1.59 cm. All eight generated original/edited results open and save byte-identically in native Sketchpad 5. The port also reopens segment-length measurements whose hosts are supported transformed segments, enabling their later calculations.

The native creation fixtures are `numeric_angle_segment_native.gsp`, `numeric_angle_circle_native.gsp`, and `numeric_angle_polygon_native.gsp`. To reproduce creation, use the corresponding `numeric_angle_*_base_port.gsp` with `dev/wine_roundtrip.sh` and `dev/numeric_angle_shapes_wine_setup.sh` as `SKETCHPAD_WINE_SETUP_SCRIPT`. Select the shape through `SKETCHPAD_ANGLE_SHAPE_X/Y`: segment 279/269, circle 319/249, polygon 299/279. The script marks the center and the angle measurement, then invokes Rotate. Native and generated full-file preservation and object-group equality are checked in regression tests.

## Legacy angle-value rotation

GSP4 code 119 stores `[source, center, angle value]` with no constraint payload.
The two corpus files with supported roots use code-116 `VertexAngleValue`
parents, whose reversed screen-coordinate turn is the value native Sketchpad
applies. Exact plain groups and the single observed point-label group decode as
modern `NumericRotation` semantics while retaining their original code and
records on an unchanged save. Point, Segment, and analytic Arc headers account
for seven direct roots; tests compare every resulting point or sampled endpoint
against an independent rotation of its live source.

Follow-up format work established circular-segment rotations and the two legacy
code-114 signed-angle sources. Their dependent point rotations now use the same
typed numeric-rotation evaluator, so all observed code-119 objects decode. The
original `课时7笔记.gsp` opened and resaved byte-identically in native Sketchpad
5 under Xvfb/Wine. See [signed angle analysis](signed_angle114_analysis.md).
