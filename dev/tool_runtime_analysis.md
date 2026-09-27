# Embedded tool runtime

## Typed path givens

Given descriptors now retain the native class/subtype separately from their
construction code. Contextual decoding recognizes point `(1,4)` and path
`(1,5)` givens. A path given is an abstract `ToolInput(Path)`, with no invented
geometry. Finite segment `(1,14)` and generic numeric `(11,33)` givens are also
recognized; their independent controls are documented in
[segment evidence](tool_segment_given_analysis.md) and
[numeric evidence](tool_numeric_inputs_analysis.md). Other categories stay
unsupported. Unplaced code15 outputs decode as
`ToolPointOnPath` only in an isolated tool canvas; saving an unbound prototype
as an ordinary drawing is rejected even when the prototype is unchanged.

Native-created segment and circle tools use the same path category. Independent
invocation and byte-identical reopens establish that it also accepts rays,
lines, arcs, and polygons. Polygon interior clicks produce boundary points.
The retained `embedded_tool_path_*_native_verified` fixtures record these
controls. The ray control has a parameter above one and the line control has
a negative parameter, so placement cannot be restricted to their endpoint
segment. Native can also construct a segment from two point clicks while
collecting a path given; a single point click is not evidence of rejection.

The runtime dispatches an unplaced output to the actual host's ordinary
constrained-point kind. It chooses placement once and retains it through
save/reparse, undo/redo, and subsequent host edits. Ray and line placement can
extend beyond the defining segment; polygon placement chooses an edge and a
fraction. This does not reproduce or claim Native's random-number algorithm.
Fresh path API writer Native verification is tracked separately from the
Native-created import controls below.

The initial point-only implementation is recorded below for chronology.

`Document::embedded_tool_document(index)` decodes a prototype in a separate
object and font namespace. It uses the normal drawing decoder in a synthetic
single-page document, retains the tool's local font tables and construction
groups, and replaces only declared free-point input placeholders with neutral
finite points. Native prototype `0899` input payloads are not drawing positions.
Nested/incomplete canvases, invalid input declarations, mismatched object counts,
and invalid canvas framing are rejected. Unsupported construction records stay
unsupported in the isolated document.

`Document::invoke_embedded_tool(index, inputs)` binds the ordered local `04b4`
inputs to available point objects on the destination's active page. The initial
supported input category is the native free-point placeholder. The remaining
known graph is materialized through `Kind::remap_checked`; unknown constructions,
unresolved dependencies, or undeclared free-point placeholders are rejected.
Geometry receives no clipboard offset. Input objects are retained unchanged.

Invocation preserves parsed visual styles, labels including their opaque bytes,
visibility, tracing, signed polygon order, midpoint hosts, and number/function
metadata. Native invocation uses the destination caption font, independently of
the prototype's local font table. The raw prototype remains byte-preserved.
The method returns all constructed IDs and selects visible constructed objects.
It validates saving and reparsing on a clone before committing one undo step.

The authentic registered Square has ordered inputs `[2,1]` and 13 local objects.
Binding destination points `(200,180)` and `(260,180)` creates 11 objects, with
corners `(200,180)`, `(260,180)`, `(260,120)`, `(200,120)`. Hidden helpers remain
hidden. Moving the second input to `(280,180)` updates the other corners to
`(280,100)` and `(200,100)`.

The independently Native-created simple tool has inputs `[2,1]` and four local
objects. Invocation creates a segment and its midpoint; the midpoint's explicit
segment host is remapped with its endpoint references. Regression coverage also
checks font namespace isolation, unchanged input styles, raw prototype bytes,
undo/redo, and atomic rejection of invalid inputs and unsupported prototypes.

The initial authored Square writer exposed an existing record-order issue.
Native rejected its drawing intersections when their labels preceded the
reference record. Changing only code9/code11 label placement to follow `07d2`
produced a 4452-byte file that Native opened and saved byte-identically with a
positive square capture and prototype captions C/D. The original type7 label
payloads and nonzero caches, hidden-helper flags, polygon style/rank, and raw
tool bytes remained unchanged. Those fields were not normalized to fix the
failure. Fresh API writer controls are checked separately after the shared
encoder correction; the isolated mutation is evidence for record order alone.

Both fresh API outputs now independently opened and saved in Native Sketchpad
with successful exits, exact byte comparisons, and inspected positive geometry
captures. The 4452-byte Square renders above the baseline with captions C/D.
The 3120-byte simple tool renders its new midpoint at `(330,210)`. Exact writer
goldens and matching captures are retained as
`embedded_tool_square_invoked_writer_native_verified.{gsp,png}` and
`embedded_tool_simple_invoked_writer_native_verified.{gsp,png}`. The existing
runtime regressions compare their actual API bytes to those Native writer
goldens; the earlier label-order mutants remain diagnostic controls.

## Native path givens and unplaced point recipes

Native-created segment and circle tools use the same given header:
code0, class1, subtype5. Their prototype groups differ only in the given caption
(`j` versus `c[1]`) and tool name. The output is code15, class1, subtype10,
with one host reference and no `07d3` placement constraint. The retained
`embedded_tool_path_segment_created_native_verified` and
`embedded_tool_path_circle_created_native_verified` GSP/PNG pairs each passed a
second Native open/save with identical bytes and positive geometry captures.

Crossbinding proves this given is a path category shared by segments, circles,
rays, lines, three-point arcs, and polygons. Segment-origin tools accept circles;
circle-origin tools accept the other tested hosts. Native invocation materializes
ordinary host-specific code15 constraints. A ray control produced parameter
1.100018921475875 and a line control produced -0.10516678365428633, preserving
their distinct extents. Arc invocation produced `PointOnArc`; polygon edge and
interior clicks both produced boundary points. The triangle constraints use
subtype52, edge indices1/2, and the observed cached tail `ff ff f0 bf`. Their
subtype still must match the host's vertex count, and edge/fraction bounds remain
checked. These controls are retained as `embedded_tool_path_segment_on_circle`
and `embedded_tool_path_circle_on_{segment,ray,line,arc,polygon}` with the
`_native_verified.{gsp,png}` suffix. Every accepted control passed a second Native
open/save byte-for-byte.

The source placement is omitted even when the host is constructed from point
givens. Selecting both free endpoints, their segment, and a point at fraction .5
produced a two-point-input tool whose code15 recipe references the constructed
segment but contains no `07d3`. The retained
`embedded_tool_constrained_point_from_endpoints_created_native_verified` control
passed Native creation, positive Script View capture, and exact reopen/save.
Native invocation assigns a placement once; the observed parameters do not
establish a random-number algorithm. A single free-point click produces no
immediate output, but two interactive point clicks can construct a segment host
and then its constrained point. This is host construction, rather than direct
binding of a free point as a path.

The fresh path writer initially triggered Native's `文件问题` (File Problem)
dialog. An isolated mutation moved only code15 label records after the host
references and placement constraint. Both the 3184-byte segment/circle output
and 3646-byte extended output then opened and saved identically, with inspected
positive captures. Their label payloads, styles, parameters, and raw tool bytes
were unchanged. The encoder therefore places labels for points on segments,
rays, lines, and circles after `07d3`, as already required for arcs and polygons.

Fresh API outputs were checked independently after that correction. Segment-origin
invocation (3184 bytes), circle-origin invocation (3194 bytes), extended
ray/line/arc/polygon invocation (3646 bytes), and constrained-point tool creation
(2724 bytes) each exited successfully in Native, saved byte-identically against
an untouched input snapshot, and produced inspected positive captures. Their
actual writer GSP/PNG pairs are retained as
`embedded_tool_path_{segment,circle,extended}_invoked_writer_native_verified`
and `embedded_tool_constrained_point_created_writer_native_verified`.

The Square runtime now preserves the prototype's independent presentation
suppression flag. The regenerated 4452-byte API writer differs from the earlier
writer in exactly four bytes: helpers5 (circle) and6/7/9 (perpendicular lines)
retain flags3 instead of being reduced to ordinary hidden flag1. These flags
match the independently Native-invoked 4472-byte Square control. Every other
byte in the earlier API writer remains unchanged. Native opened the regenerated
writer, saved it byte-for-byte, and displayed the same square above its baseline
with captions C/D and no helper geometry. Its distinct writer golden and capture
are retained as
`embedded_tool_square_invoked_suppression_writer_native_verified.{gsp,png}`.
Flag2-only helpers and their independently tested Show All Hidden behavior are
covered by the endpoint/midpoint controls in
[`tool_segment_given_analysis.md`](tool_segment_given_analysis.md).
