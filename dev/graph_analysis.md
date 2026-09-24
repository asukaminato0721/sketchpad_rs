# Graph coordinate system and slope observations

## Original application probes

The source fixture `tests/fixtures/wine_segment_slope.gsp` has a horizontal
segment (objects 1–3). Measure > Slope created a Cartesian grid and objects
4–10. Graph > Define Coordinate System on the same port-generated segment
created objects 4–9 without a slope; the result is
`tests/fixtures/graph_defined_coordinate_system.gsp`. The menus were driven
through Wine/Xvfb with `dev/wine_roundtrip.sh`.

| ID | Code | Auxiliary records | References | Observed role |
| --- | --- | --- | --- | --- |
| 4 | 0 | `0x0899` | none | Free grid origin `(199, 125)` |
| 5 | 52 | `0x07d3` | 4 | X-unit point, step `37.79527559055118` screen units |
| 6 | 58 | `0x0905 = 1`, label | 4, 5 | X axis |
| 7 | 54 | `0x07d3` | 5 | Y-unit point, same step |
| 8 | 58 | `0x0905 = 0`, label | 4, 7 | Y axis |
| 9 | 61 | `0x0904 = (1, 0)` | 6, 8 | Coordinate system |
| 10 | 87 | `0x0903`, `0x07d8` | 3, 9 | Segment slope |

The screen Y axis points downward. The graph Y axis points upward. The
original showed `0.00` for the horizontal segment. Dragging its endpoint
from `(250, 120)` to `(300, 170)` showed `-0.33`; moving its other endpoint
from `(150, 120)` to `(150, 170)` showed `0.50`. Those results agree with
`CoordinateFrame::slope` in `src/graph.rs`. The two edited original files are
`graph_moved_slope.gsp` and `graph_moved_slope_source.gsp`.

Selecting the segment's two endpoints before Graph > Define Coordinate
System produced the same six graph groups as defining it with no selection.
Only an unrelated `0x04b1` presentation record differed in that probe.

Graph > Show Grid toggled page record `0x03e8` payload byte 10 from zero to
one. Its saved file is `graph_hidden_grid.gsp`. A page without any coordinate
system also stores zero there, so the flag only has meaning with a frame.

## Writer proof

`assets/graph_coordinate_system_groups.bin` contains the six complete groups
from Graph > Define Coordinate System. `assets/graph_slope_measurement_group.bin`
contains the code 87 group. `graph::coordinate_system_groups` remaps all
`0x07d2` references and patches the origin; `graph::slope_measurement_group`
patches the line and frame references.

I appended these two outputs to the port's three-object segment file and
updated the page object count from 3 to 10. The resulting 2,630-byte file
opened in Sketchpad 5, displayed a grid and `0.00` slope, and saved with **zero
changed record slots**. Its saved copy is
`tests/fixtures/graph_generated_slope_wine.gsp`. Dragging its endpoint to
`(300, 170)` displayed `-0.33`; the saved file reopens as ten objects and is
`tests/fixtures/graph_generated_moved_slope_wine.gsp`. The original added
point-label records on that edit and changed the slope `0x07d8` style from
`02 00` to `01 00`, so the decoder accepts both observed styles.

The generated file did not need the original's extra `0x04b3`, `0x232d`, or
`0x0961 = 2` records to display and update the slope. The writer still needs
to retain those records when opening an existing original file.

## Integrated behavior and verification

The document model decodes the six frame objects and slope, evaluates their
dependencies, and writes the observed default frame template. The native UI
has Define Coordinate System, Show Grid, and Slope commands. The renderer and
SVG exporter draw the grid using the same viewport transform as the sketch.
The replay harness has `coordinate_system`, `show_grid`, and `slope` commands.

Original files include a trailing `0x232d` page-link record whose four-byte
payload is the coordinate-system object ID. The first writer omitted it; the
original app still displayed the initial grid and slope, but its grid toggle
was inconsistent. The writer now emits and validates the page link, including
ID remapping when objects are deleted. In the code 54 Y-unit object header,
byte 10 is `0x04`; treating its visual invisibility as the generic hide bit
produced a file that the original refused. The model therefore retains that
header byte and the native renderer skips its point glyph.

`cargo test --offline` passed 74 library tests and one binary test on
2026-09-24. Replay generated a 10-object slope sketch; Sketchpad 5 opened it,
displayed `0.00`, and saved with zero changed record slots. Dragging an
endpoint in Sketchpad 5 displayed `-0.33`; Rust reopened and verified all ten
objects. A blank-document replay generated a six-object coordinate system
with IDs 1–6; Sketchpad 5 opened and saved it with zero changed record slots.
The hidden-grid replay generated the same page flag and `0x232d` page link as
the corresponding original file, and Sketchpad 5 saved it byte-for-byte.
However, Wine screenshots of the same hidden-grid bytes differed between
runs: one showed the grid and one did not. The visual persistence of Show Grid
therefore remains unverified in the original app despite the matching records.

These probes cover the default Cartesian frame and segment slope. They do not
establish selected-object frames, grid form, axis dragging, or the other Graph
commands.
