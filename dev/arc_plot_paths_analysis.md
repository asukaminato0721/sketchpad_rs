# Native value and literal points on known arcs

This slice covers code 95 scalar-value points and code 123 literal-value points
on the existing code-79 `ArcOnCircle` and code-81 `ThreePointArc` families.
The separate centered-FreePoint code-80 arc schema remains unsupported. A
scene-only corpus scan, excluding embedded tool canvases, found 35/16 code-95
groups and 1/1 code-123 groups referencing code 79/81 respectively.

Native controls use a quarter circle centered at (200,180), radius 60, from
(260,180) to (200,120). Value .25 gives the 22.5-degree position and literal
.5 gives the 45-degree position. Changing the three-point arc's middle vertex
from 45 degrees to 30 degrees leaves both plotted positions unchanged. The
parameter therefore measures normalized directed sweep, rather than assigning
.5 to the middle vertex. Both endpoint values 0 and 1 are defined. Finite
negative and greater-than-one values remain serialized, but their point glyphs
and attached marker segments are absent. They are undefined, not clamped or
wrapped. Native preserves all these source files byte-for-byte.

Degenerate controls distinguish geometric states rather than accepting every
failed circular calculation. Equal endpoints of a circle arc are undefined.
For a three-point arc, repeated references, repeated first/middle coordinates,
equal first/last coordinates, and an exactly collinear middle vertex outside
the endpoint span are undefined. A distinct exactly collinear middle vertex
strictly inside the span instead produces a finite straight arc. For endpoints
(260,180) and (200,180), .25 gives (245,180) and .5 gives (230,180). Moving the
middle vertex from (230,180) to (240,180) does not change these positions:
the fallback uses the affine first-to-last fraction. Native shows the host and
in-domain point glyphs and marker segments in both controls. Negative and
greater-than-one parameters remain undefined in the linear state.

The retained `arc_*_plot_native_verified.gsp` controls are 21-object documents.
Each Native roundtrip exited successfully before byte comparison, and all
retained control bytes matched exactly. The file named
`arc_three_point_collapsed_plot_native_verified.gsp` means collapsed circular
curvature: it is the defined linear control, not an undefined host. The uneven
middle and outside-middle fixtures make that distinction explicit.

The public APIs now support both known arc families using the same eligibility
and geometry helpers as Graph menu routing. The menu accepts supported scalar
numeric sources, including calculations, and supported polygons and fixed
straight paths already proved in earlier slices. It excludes code 80 and
unproved transformed path families. An undefined scalar source still reports
the API's undefined-value error; it does not silently open the literal dialog.

Arc plotting retains finite scalar/literal values, earlier live same-page
direct parents, and available finite geometry. The dependency walk checks
cycles and deleted, unknown, or cross-page descendants before evaluating
positions. It preserves existing permitted forward default-table references
inside numeric dependencies. Only the exact native-proved arc degeneracies
and finite outside-domain values permit an undefined plotted point. The
existing Segment-only exception permits marker segments attached to such a
point; other shape dependency guards remain unchanged. Nearly collinear
unsupported circular calculations are rejected atomically, rather than being
classified as exact linear or undefined states.

`linear_three_point_arc` supplies the independently proved straight fallback.
`arc_points` uses its two endpoints, so SVG, GPU rendering, and picking agree
on the displayed host. Circular `arc_geometry` remains unchanged. The separately
controlled constrained code-15 and arc-measurement behavior is described below;
point-value measurement on the linear state still has no independent control
in this slice.

A separate 37-object graph is authored through the actual public APIs. It
contains a 270-degree circle arc, a three-point arc, three scalar parameters,
value and literal plots, labels, and marker segments. Native accepted all four
actual writer outputs and saved them byte-identically:

| Stage | Objects | Bytes | Native observation |
| --- | ---: | ---: | --- |
| Authored | 37 | 5484 | In-domain plots and markers display; outside values are absent |
| Source edited | 37 | 5482 | Radius, middle vertex, and scalar edits update dependent plots |
| Copied | 74 | 9196 | Dependencies remap; defined geometry offsets by (20,20) |
| Degenerate hosts | 37 | 5482 | Zero circle arc is absent; collinear three-point arc and in-domain plots remain defined |

The four `arc_plots_authored_*_native_verified.gsp` files are exact regression
goldens. Tests also cover all control positions, source save/reopen, scalar
recovery, undo/redo, copy, cyclic/deleted dependency rejection, atomic invalid
creation, Graph eligibility, SVG host output, and picking on the finite linear
span. Six focused document tests and the editor picking test pass.

## Independent constrained-point and measurement controls

A separate nine-object circular fixture contains a code-15 `PointOnArc` with
stored parameter .25, a visible marker segment, an arc-length measurement
(native 44), and an arc-angle measurement (native 43). Native displays the
quarter-sweep point, length 2.49 cm and angle 90 degrees. Changing only the
source coordinates to endpoints (260,180)/(200,180) and middle (240,180)
produces a displayed constrained point at (245,180), length 1.59 cm (60 pixels
divided by the native centimeter scale), and angle 0.00 degrees. This proves
constrained-point and measurement semantics independently of code 95/123.
Moving the middle outside to either (290,180) or (170,180), repeating its
coordinates at either endpoint, equal endpoint coordinates, and repeated
first/middle references all remove the host, point, marker, and both
measurement captions. All eight labeled controls save byte-identically at
2,534 bytes.

An actual native drag from the linear constrained point to (215,180) preserves
the source vertices and rewrites its stored code-15 parameter to .75. The
native-edited 2,726-byte file is retained separately; native UI editing also
adds default labels, styles, and window metadata, so it is evidence for the
drag result rather than an expected byte match to the port's 2,534-byte writer.
Two further actual native drags beyond the last and first endpoints store
parameters 1 and 0 respectively, and display the point at the corresponding
endpoint. Those 2,706-byte native-edited files independently establish the
projection clamp used for creation and dragging.

The evaluator now uses the finite endpoint span for linear `point_on_arc`,
projected/clamped `arc_parameter`, endpoint distance for arc length, and zero
for arc angle. Only native-proved exact undefined three-point states permit
missing code-15 geometry, marker geometry, or these measurement values during
save/copy. Deleted/cross-page/unknown/cyclic dependencies and unsupported
nearly-collinear calculations remain guarded.

This control also exposed a concrete writer-order defect: a new labeled
`PointOnArc` written as header, label/font, references, constraint failed to
open twice. Its unlabeled counterpart opens and saves exactly (2,472 bytes).
Reordering the identical labeled payload as header, references, constraint,
label/font opens and saves exactly (2,534 bytes). Both new-object writing and
adding a label to an imported unlabeled arc point now use the accepted order.
Other point families are not broadened by this observation. Regression tests
compare both actual public-API paths with the retained positive native file.

Actual public-API writer outputs for a linear constrained drag (2,534 bytes),
copying its defined nine-object graph (18 objects, 3,300 bytes), and copying
the undefined outside-middle graph (18 objects, 3,300 bytes) each opened and
saved byte-identically in Native. The undefined copy preserves absent hosts,
points, markers, and measurement captions on both original and copied sides.
All three outputs are retained as exact regression goldens. Two additional
document tests cover the complete independent control matrix, native drag and
clamp results, both label-writing paths, source edits, recovery, undo/redo,
defined/undefined copy and save/reopen.


## Checkpoint34 center-arc follow-up

Native code80 is now a supported basic center/start/end-direction construction.
Shared sampling and point/value/literal plotting include it, with Native proof
for radial endpoints, major sweeps and undefined saves. See
[center arc evidence](center_arc_analysis.md). Earlier unsupported-family notes
in this file describe their historical checkpoints.
