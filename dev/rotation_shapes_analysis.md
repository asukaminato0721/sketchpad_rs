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
