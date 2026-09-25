# Graph coordinate system and slope observations

## Multiple coordinate systems and marking

Sketchpad's [Define Coordinate System reference](https://referencecenter.dynamicgeometry.com/gsp-reference-center/menus_graph_definecoordsys.htm) says that selecting one point defines a frame at that point even when a frame already exists. Its [Mark Coordinate System reference](https://referencecenter.dynamicgeometry.com/gsp-reference-center/menus_graph_markcoordsystem.htm) says selecting a frame's origin, unit point, axis, or grid makes it active for future plots and measurements.

In Wine, selecting point 1 of `graph_defined_coordinate_system.gsp` and using Graph > Define Origin created objects 10–14. They are a horizontal unit constrained to point 1, X axis, vertical unit, Y axis, and frame; there is no duplicate origin object. The native result is `tests/fixtures/graph_two_coordinate_systems_original.gsp`. Its page-link record `0x232d` points to frame 14. Selecting frame 9's origin and using Graph > Mark Coordinate System, then moving an unrelated point to make the document dirty, changed that link from 14 to 9. The native result is `graph_mark_coordinate_system_original.gsp`. Marking alone did not rewrite the saved file in the probe.

The Rust writer now emits the five-object shared-origin construction and stores the marked frame from `0x232d`. `tests/graph_second_coordinate_system_replay.txt` creates frame 14; `tests/graph_mark_coordinate_system_replay.txt` marks frame 9 and plots a point at `(1, 2)` on that frame. Both generated files opened and saved byte-identically in Sketchpad 5. The native desktop capture shows both square grids and the new point relative to frame 9. Grid form and dotted state are retained per frame.

`tests/graph_two_frames_mixed_grid_replay.txt` changes frame 14 to dotted Polar, then marks square frame 9. Sketchpad 5 opened the generated 14-object file with frame 9's straight grid and frame 14's polar dots, then saved it byte-identically. The native renderer and SVG exporter now collect geometry per visible frame, so both sets appear together and hiding frame 9 leaves frame 14's dots visible. The resulting file is `tests/fixtures/graph_two_frames_mixed_grid_generated.gsp`.

Clicking the first frame's grid intersection at `(334, 270)` in the native two-frame sketch selected that grid. Graph > Mark Coordinate System then changed the page link from 14 to 9 after an unrelated edit forced a save. The Rust editor hit test now chooses a visible frame at nearby square or Polar grid intersections and highlights the selected grid. The original `graph_snap_points_original.gsp` also visibly retains its grid with Snap Points enabled; the Rust native canvas and SVG now do likewise.

Selecting circle 3 in `circle.gsp` and choosing Graph > Define Unit Circle
created two code 60 axes and a code 61 frame. Both axes refer directly to the
circle; their `0x0905` payloads distinguish X (`1`) from Y (`0`). The frame
refers to the axes and uses the usual square-grid `0x0904` record. The native
file is `tests/fixtures/graph_unit_circle_original.gsp`; its circle radius
sets both graph units, so dragging the radius point updates the frame. The
port generates those three original groups from
`assets/graph_unit_circle_groups.bin`. Sketchpad also writes a second page
`0x04b3` style record with style ID 14. A generated file without that record
opened with axes but no grid lines; adding the style made its native display
match the original. `tests/graph_unit_circle_replay.txt` produces a six-object
file that opened with grid lines and saved byte-identically in Sketchpad 5.
The Rust canvas and SVG export draw the circle axes' radius-scaled ticks and
numeric labels; dragging the radius point changes their spacing.

The [native command reference](https://referencecenter.dynamicgeometry.com/gsp-reference-center/menus_graph_definecoordsys.htm) also permits one or two defining distances (a segment or distance value), with an optional origin point. Native one-distance files add a free origin when needed, then code 59 X axis (origin and distance references), code 56 Y-unit point, code 58 Y axis, and code 61 square frame. Two distances use a code 59 Y axis referencing the second distance and a rectangular code 61 frame; no code 56 point is needed. Selected-origin forms omit the new free point. The five native files `graph_unit_segment_original.gsp`, `graph_unit_segment_point_original.gsp`, `graph_two_segments_original.gsp`, `graph_two_segments_point_original.gsp`, and `graph_unit_distance_value_original.gsp` decode as editable frames. The source segment or measurement controls the unit length dynamically; `graph_unit_segment_dragged_by_wine.gsp` confirms that Sketchpad updates both scales after dragging the segment endpoint.

The port writes the captured groups from `assets/graph_distance_square_groups.bin` and `assets/graph_distance_rectangular_groups.bin`, remapping origin, distance, and axis references. The four segment-based generated group sequences match their native counterparts byte for byte. Generated one- and two-distance files, both selected-origin forms, a point-distance value, and a segment-length value opened with their grids and saved byte-identically in Sketchpad 5. A one-distance Polar variant also opened with rings and saved unchanged. `tests/graph_defining_distances_replay.txt` reproduces these files. The native canvas and SVG draw tick marks at each axis's own scale.

Grid Form > Rectangular on a one-distance frame changes the Y-unit object from code 56 to 57, clears its header's `0x00040000` flag, stores the current segment length as the independent vertical step in `0x07d3`, and sets frame `0x0904` to `2`. The native result is `graph_unit_segment_rectangular_original.gsp`. Switching back to Square changes code 57 back to 56 while retaining the stored step; the displayed Square Y unit again follows the defining segment's length. Switching from Rectangular to Polar preserves code 57 and the independent Y step, whereas switching from Square to Polar preserves code 56 and the dependent Y unit. The Rust writer made the same five frame groups as the native Rectangular conversion. A port edit changed the independent Y step to 80 pixels; Sketchpad displayed horizontal grid lines 80 pixels apart, then elliptical Polar rings after the form changed, and saved both files byte-identically. `tests/graph_distance_grid_forms_replay.txt` reproduces the conversions and edit.

A two-distance frame can switch from Rectangular to Polar while retaining separate X and Y unit lengths, so its radial rings become ellipses. Native and generated four-group sequences match, and the generated file cross-opened unchanged. Selecting Square on a two-distance frame crashed Sketchpad under Wine on both native-created and generated files. A file with only the square flag changed opened but retained unequal axis spacing. The port therefore leaves that conversion unavailable until its native semantics can be established.

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

Further menu and drag probes corrected the earlier identification of page
record `0x03e8` payload byte 10. Graph > Show Grid changes the coordinate
system object's visibility bit in its `0x07d0` header. It hides the frame,
axes, origin, and unit points on reopen while leaving unrelated free points
visible. The original example is `graph_hidden_by_menu_original.gsp`.
The generated `tests/graph_hidden_replay.txt` frame opened with no graph
components in Sketchpad 5 and saved byte-identically.

Graph > Dotted Grid changes the coordinate-system object's `0x0904` payload
from `01 00 00 00 00 00 00 00` to `01 00 01 00 00 00 00 00`. The original
saved example is `graph_dotted_grid_original.gsp`. The port decodes that byte,
renders grid intersections as dots, and writes it for both existing and new
frames. A port-generated six-object dotted frame opened with the expected
dots in Sketchpad 5 and saved byte-identically. The generated and original
fixtures are in `tests/fixtures`, and `tests/graph_dotted_grid_replay.txt`
reproduces the generated file.

Grid Form's first three submenu entries are Polar, Square (the default),
and Rectangular. The coordinate-system `0x0904` first byte is respectively
`0`, `1`, or `2`. Polar retains the default unit objects and displays
concentric unit circles and 24 radial lines. Rectangular changes the vertical
unit object's code and `0x07d3` constraint code from decimal `54` to `55`;
its initial step is one `f64` bit below the default `37.79527559055118`.
Dragging that unit point upward by 18 screen units changed the step to
`55.79527559055117` while leaving horizontal spacing unchanged. The fourth
submenu entry opened a confirmation dialog. Accepting it labeled the
horizontal axis at `−π` and `π`; its X-axis `0x0905` payload changed from
`01 00 00 00` to `01 00 01 00`, while the coordinate-system form remained
Square. The port stores that X-axis flag, exposes the fourth Grid Form
command, and renders labels at multiples of π in the native canvas and SVG.
Original Polar and Rectangular examples, plus the dragged Rectangular unit,
are in `tests/fixtures`. Generated Polar, Rectangular, and dotted Polar files
opened with the expected grids in Sketchpad 5 and saved byte-identically.
An original Rectangular file edited by the port to use a 65-unit vertical
step reopened in Sketchpad 5 with independent vertical spacing. The original
trigonometric-axis file round-trips byte-identically. A generated file with
the same flag opened with `−π` and `π` labels in Sketchpad 5 and saved
byte-identically. Its fixture and replay script are
`graph_trigonometric_axis_generated.gsp` and
`graph_trigonometric_axis_replay.txt`.

Graph > Snap Points changes page record `0x03e8` payload byte 10 from `0` to
`1`. With the option on, dragging a free point toward `(250, 170)` in the
default frame saved it at `(236.79527559055117, 162.79527559055117)`, the
nearest grid intersection. With the option off, the same drag saved it at
`(250, 170)`. Reopening and dragging again still snapped. The original
example is `graph_snap_points_original.gsp`; a port-generated example from
`tests/graph_snap_points_replay.txt` opened, snapped again, and saved in
Sketchpad 5. The `0x2328` record seen after these probes is the selected
object marker, not the snap flag: it held the free point's ID after dragging.
Clicking Snap Points again restored byte 10 to `0`; the next drag moved the
point by the pointer delta without rounding it to the grid.
In a Polar frame, dragging a free point toward `(250, 170)` snapped it to
`(252.45059133378615, 178.45059133378635)`, the radius-2 ring at −45°.
The port's polar snap calculation matched within floating-point rounding;
`graph_polar_snap_original.gsp` retains the original result.

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
dependencies, and writes the observed frame templates. The native UI has
Define Coordinate System, all four Grid Form choices, Show Grid, Dotted Grid,
Snap Points, and Slope commands. The renderer and SVG exporter use the same viewport
transform as the sketch. The replay harness includes `coordinate_system`,
`grid_form`, `trigonometric_axis`, `show_grid`, `dotted_grid`, `snap_points`,
and `slope` commands.

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
The earlier `graph_hidden_grid.gsp` fixture was misnamed: its page flag enables
Snap Points. A later drag comparison established that distinction. The
corrected Show Grid replay now changes the frame header visibility bit, and
its file opened with no frame components in Sketchpad 5 and saved
byte-identically. The Snap Points replay rounded a free point to the same
intersection as the original app; Sketchpad 5 opened it and snapped a second
drag to the next intersection.

These probes cover the default frame, free-point snapping, all four Grid Form
choices, grid visibility and dotted form, and segment slope. They do not
establish selected-object frames, horizontal unit dragging, or the remaining
Graph commands.
