# Embedded tool runtime

## Typed path givens

Given descriptors now retain the native class/subtype separately from their
construction code. Contextual decoding recognizes point `(1,4)` and path
`(1,5)` givens. A path given is an abstract `ToolInput(Path)`, with no invented
geometry. Finite segment `(1,14)` and generic numeric `(11,33)` givens are also
recognized; their independent controls are documented in
[segment evidence](tool_segment_given_analysis.md) and
[numeric evidence](tool_numeric_inputs_analysis.md). Scalar Parameter `(11,79)`
is recognized separately and matches only an actual scalar Parameter; its two
corpus prototypes remain partial, so runtime materialization is not claimed.
See [scalar-Parameter evidence](tool_scalar_parameter_given_analysis.md). Other
categories stay unsupported. Unplaced code15 outputs decode as
`ToolPointOnPath` only in an isolated tool canvas; saving an unbound prototype
as an ordinary drawing is rejected even when the prototype is unchanged.

Polygon `(1,15)` is a separate abstract category established by a Native match
prompt and successful interior-click invocation. It deliberately has no vertex
count until bound to a supported destination polygon. Its only corpus prototype
remains partial, so port-side materialization is not claimed. See
[polygon-given evidence](tool_polygon_given_analysis.md).

Picture resources remain document-global even when their construction groups
belong to a tool prototype. Isolated decoding copies an enclosing document's
ordered resource payloads into its synthetic canvas whenever the prototype has
a picture header, allowing existing bounded picture decoding to retain resource
IDs without inventing a separate namespace. See
[picture-resource analysis](tool_picture_resources_analysis.md).

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

## Numeric layout and current validation gate

The runtime maps numeric output fonts into the drawing font namespace before
canonical calculation decoding. Tool-local tables remain byte-preserved. Native
controls independently changed the destination caption to Tahoma24Bold, the
script output font to24Normal, and the script given font to26Bold; new results
still used the current application calculation default, Tahoma18Normal. These
controls establish independence from stored caption/script fonts, not a universal
font preference across historical files. Preferences remain incomplete.

Numeric results follow the last numeric object by ID on the active page, using
its current anchor and actual supported caption height. Native retained controls
establish parameter bounding-box height plus one, flat Tahoma18Normal calculation
step27, flat Tahoma24Bold step34, default measured-length step21, and measured-area
step23. Unsupported fraction/font layouts still require implementation and
independent native controls. Fresh area4204-byte and product3974-byte API writers
opened and saved byte-identically in Native with positive separated captions;
the product remains live through measured-source movement. See the retained
`embedded_tool_numeric_{area,product}_invoked_font_flow_writer_native_verified`
fixture pairs.

The complete `cargo test --all-targets` gate45 exited0 on2026-09-27:863 tests
(682 library,127 editor,42 numeric import,7 coverage,2 source preservation,
3 value-equation integration). Corpus checkpoint20 separately preserves all
1599 files exactly, with3413 of5356 prototype slots decoded. Neither gate proves
completion of the broader port; `COMMAND_PARITY.md` retains the command gaps.

All six fresh numeric font/flow writers subsequently passed Native open/save,
independent exact comparison, and positive caption inspection: scalar3338,
length3760, degree3762, measurement3972, area4204, product3974 bytes. Their
`*_invoked_font_flow_writer_native_verified` fixture pairs preserve the older
writer checkpoints. Generic numeric authoring now also covers selected scalar
and degree parameters or segment-length measurements with a direct supported
calculation, using Native script fonts18Normal for the given and16Normal for
the step. Broader authoring graphs remain incomplete.

The subsequent full gate47 exited0 with867 tests (686 library,127 editor,
42 numeric import,7 coverage,2 source preservation,3 value-equation integration).
It includes the actual Native execution regression for the port-authored numeric
tool, the independent single numeric-point clipboard boundary, and expression-tree
fraction placement. Single, nested, and24Bold fraction API writers each passed
Native0, exact independent comparison, and positive caption inspection. Fraction
source expansion with generated/named non-formula captions, broader measurement
layouts, broader authoring graphs, and other clipboard boundaries remain active
work. Corpus checkpoint20 retains1599 exact original-file roundtrips.

## Function operands in abstract calculations

The three 39-input PI/root prototypes retain one referenced Function operand
and 34 generic Numeric operands in a 35-source code-48 expression. Contextual
decoding and runtime specialization now use the expression tree's exact
Numeric/Function role vector instead of rejecting every nonnumeric parent before
role parsing. Function roles require a supported function kind; numeric roles
retain their existing unit and placeholder checks. The source order, count, and
native expression bytes remain exact. These tools still contain an unrelated
unsupported final object, so runtime execution is not claimed. See
[the focused analysis](tool_function_calculation_analysis.md).

## Function givens

Native explicitly prompts `1. 匹配函数 h` for the class11/subtype57 given in
`新f(x)的图象工具`. The contextual placeholder now satisfies function source
roles without supplying an invented expression or value. Invocation requires a
real active-page function, and the desktop picker restricts the click to its
visible function caption. Generic Numeric placeholders remain admissible only
for the referenced expression's Numeric roles until invocation supplies their
concrete units and values.

A headless invocation bound an identity function, four live scalar Parameters,
and a coordinate system to the newly complete logarithmic-graph tool. It
created a referenced function and FunctionPlot, returned finite identity samples
inside the chosen bounds, produced 18 plot segments, and survived exact
encode/reparse. See [the focused analysis](tool_function_given_analysis.md).

## Legacy generic values and distance givens

Native reordered-input controls distinguish two older numeric encodings.
Subtype51 and subtype33 both request a generic `匹配度量值`; subtype48 requests
`匹配距离度量值`. Subtype51 therefore reuses the unit-specializing Numeric
runtime category with a separate source-free Calculation validator. Subtype48
exposes Length during prototype decoding and accepts only finite live distance
values at runtime. The desktop picker applies the same unit gate to visible
numeric captions. See
[the focused analysis](tool_legacy_numeric_givens_analysis.md).

## Angle-Parameter given

A same-size Native input-order control moves the sole subtype81 given ahead of
eight point givens. Native then requests `匹配角度参数 t[1]` and recognizes the
visible `t[1] = 1.00°` Parameter caption. The port exposes Degrees on the
prototype root, validates the exact inert Parameter-placeholder family, and
accepts only a live degree-valued `Parameter` in both engine and desktop
binding. The source tool remains structurally partial for unrelated reasons.
See [the focused analysis](tool_angle_parameter_given_analysis.md).
