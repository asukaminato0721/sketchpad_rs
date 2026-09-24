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

## Integration into the document model

1. Export the module with `pub mod graph;` in `src/lib.rs`.
2. While parsing each object group, gather `(record_kind, payload)` for the
   five graph codes and pass the references to `decode_graph_object`. Keep
   unrecognized payloads as unsupported, retaining their bytes for round trip.
3. Add model variants for the six frame objects and slope, with the dependency
   lists returned by `GraphObject::dependencies`. For the observed default
   frame, the X-unit point is origin plus one step in screen X, and the Y-unit
   point is origin minus one step in screen Y. More general constructed frames
   should use their evaluated axis-unit point positions in `CoordinateFrame`.
4. Append the template groups before the page trailer and update the page
   object count. Validate the line, frame, and all internal references before
   serialization. Keep page/grid style records when an imported file has them.
5. Add menu actions for Graph > Define Coordinate System, Show Grid, and
   Measure > Slope, then render and hit-test axis geometry through the common
   viewport transform.

These probes cover the default Cartesian frame and segment slope. They do not
establish the behavior of frames constructed from selected objects, grid form,
axis dragging, or the other graph commands.
