# Constrained points and Value of Point on transformed paths

Native Sketchpad 5 creates ordinary code-15 constrained points on transformed
objects. A point halfway along a fixed 90° rotated segment has a single parent
(the rotated segment) and 0x07d3 payload `[15,14]` followed by fraction 0.5.
A point at the top of a numeric-ratio dilated circle has that circle as its sole
parent and payload `[15,28]` followed by its normalized direction, approximately
(0,1). The native files are `transformed_segment_point_native.gsp` and
`transformed_circle_point_native.gsp`; unedited import/save preserves their bytes.

The port now resolves supported straight host kinds to segment, ray, or infinite
line mode consistently when creating, importing, and saving constrained points.
Parallel/perpendicular lines and coordinate axes use infinite-line mode;
angle bisectors use ray mode. Supported transformed circles use the same circle constraint format as
original circles. Points continue to follow their host when its geometry changes.
The Point tool and Measure → Value of Point use these shared paths.

Value of Point evaluates the live fraction along a straight path or the stored
normalized turn around a circle. Circle turns run counterclockwise from the
rightmost point: top = 0.25 and left = 0.5. Display precision 0–5 affects the
caption without rounding the value supplied to calculations and iteration.
Infinite-line/ray measurements retain their verified zeroed native format fields;
segment and circle measurements retain the existing code-94 template.

Tests cover native-file preservation, eight constructed/transformed host families,
constraint creation, move/import/save, measured numeric values, calculated live
iteration depth, copy/remapping, and dependent deletion. Native-based tests move
the segment point from fraction 0.5 to 0.75 (depth 2 to 3), and move the circle
point from turn 0.25 to 0.5 (depth 3 to 5).

`tests/replay_transformed_point_values.txt` creates the two native creation bases
and six generated measurement/iteration probes with live edits. The circle edit
also changes its dilation ratio to −0.75. The third pair uses a constrained point
on a parallel line. To reproduce native constrained-point creation, run
`dev/wine_roundtrip.sh` on a copy of the corresponding
`transformed_{segment,circle}_point_base_port.gsp` with
`SKETCHPAD_WINE_SETUP_SCRIPT=dev/transformed_point_paths_wine_setup.sh` and a capture
prefix containing the matching `segment` or `circle` name.

All six generated files open and save byte-identically in native Sketchpad 5.
Captured captions and repeated point counts match the port before and after edits.
The negative circle dilation retains a counterclockwise turn of 0.5 at its leftmost
point and depth 5. All six fixtures verify without partial view. The full test
suite passes 582 tests (481 library, 96 desktop, and five integration).
