# Custom tool Path input editor workflow

Native Path Given inputs accept straight paths, circles, arc strokes, and polygon
boundaries or interiors. The retained `embedded_tool_path_circle_on_arc_native_verified.gsp`
and `embedded_tool_path_circle_on_polygon_native_verified.gsp` controls establish
those host families. Polygon interior clicks produce boundary-constrained points,
not free points inside the polygon. The tool runtime owns output placement.

The editor now searches available displayed paths independently of point picking,
so a point over a path cannot accidentally bind a Point to a Path slot. Arc
picking measures distance to the sampled stroke; polygon picking includes the
boundary and its interior. No new host kind or runtime capability is introduced.

The independent Native two-click control in the Given audit shows that a first
blank click creates only a pending marker. The second point click creates a Free
point at the first location, a Segment using the ordered endpoints, and the
PointOnSegment tool output. The port likewise defers the first endpoint without
changing drawing bytes or undo history. A second click reuses an existing point
or creates a constrained/Free endpoint, then validates the completed segment on
a clone before committing it. Endpoint and segment additions share one undo
step; invoking the prototype uses the existing separate runtime transaction.
Duplicate endpoints fail without changing the drawing or its history.

Successful invocation clears the deferred endpoint and keeps the tool active for
another use. Escape drops the deferred marker. It preserves previously completed
input geometry and outputs. Undo/redo exits custom-tool binding through the
existing editor history reset. Native's broader undo grouping has not been
independently measured; the stated grouping is the port's explicit policy.

Focused editor tests cover live edits to an arc and polygon host, boundary and
interior clicks under point overlays, straight-path selection under a point,
selection-based invocation, blank/existing and blank/blank endpoint construction,
ordered Segment references, undo/redo, repeated use, cancellation, and atomic
same-endpoint failure. Existing Point Given tests remain unchanged.

The actual editor two-click writer was regenerated after the independent Native
code15 label-order correction. The locked Native helper exited zero, opened the
named input and saved it at 3,322 bytes; an independent comparison was exact.
`tests/fixtures/embedded_tool_path_two_click_editor_native_verified.gsp` retains
that result. The positive capture `dev/custom_tools_path_two_click_native.png`
shows both constructed segments and their constrained outputs. The first ordered
host is Segment(6,5), including the previously edited second endpoint; repeated
blank/blank construction is Segment(9,10). The document has twelve scene objects
and retains the one source tool definition. A focused retained-fixture regression
checks ordered references, fractions, exact roundtrip, and live endpoint edits.
The earlier pre-fix run without named-window/save captures is not acceptance
proof and was superseded by this positive result.
