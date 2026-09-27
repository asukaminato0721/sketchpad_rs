# Shared analytic arc transforms (checkpoint48 audit)

Drawing-only audit `transformed_curves48_audit.rs` compiled against the current
`target/debug/libsketchpad_rs.rlib` before arc decoding was enabled. The complete
1599-file scan terminated0; `/tmp/transformed_curves48_audit.log` is the snapshot.
Embedded custom-tool prototypes and copying are excluded.

| Family | Unsupported roots | All direct prerequisites decode | Unsupported descendants reachable from ready roots |
| --- | ---: | ---: | ---: |
| Arc, class12/subtype67 |483|252|1144|
| Straight, class2 |613|141|1105|
| Polygon, class4 |387|76|389|
| Arc interior, class13 |356|0|0|
| Image, class16 |79|0|0|

Reach includes each ready root, deduplicates within a family and counts only
currently unsupported drawing slots. It is a dependency upper bound, not a
runtime/decode recovery estimate. Families overlap and cannot be added together.
The polygon family shrank after prior recursive polygon source fixes. Arc has
more immediately typed roots than straight and slightly greater dependency reach,
so it is the selected broad prerequisite. Ready arc source types are194CenterArc
and58ThreePointArc; none are sampled loci or function plots.

| Arc native code | Operation | Unsupported | Direct-ready | Ready-root reach |
| --- | --- | ---: | ---: | ---: |
|16|Vector translation|50|23|55|
|19|Marked horizontal/fixed vertical translation|48|3|57|
|21|Fixed polar translation|43|40|64|
|24|Marked distance/fixed angle translation|16|1|15|
|27|Fixed rotation|107|37|345|
|29|Numeric angle rotation|44|4|40|
|30|Fixed dilation|30|28|57|
|32|Point distance-ratio dilation|2|2|4|
|33|Numeric ratio dilation|38|30|220|
|34|Reflection|105|84|313|

Codes17,18,20,28 have no class12 arc instances in this corpus. Fresh Native API
controls now prove all14 flavors, including those four absent flavors, with exact
Native open/save bytes and independent geometry samples. Each operation keeps the existing shared transform schema;
there is no arc-specific transform product Kind.

Complete selected original groups are retained in
`transformed_curves48_original_groups.txt`; corresponding whole paired Java
construction sections are in `transformed_curves48_paired_htm.txt`.
`transformed_curves48_dump.rs` reproduces the raw groups from GSP global IDs.
The paired HTMs generally omit arc sources and their transforms. They establish
point, numeric, vector, mirror and center predecessors, not full arc semantics.
Native original open/save, preserved group bytes and independent sampling/captures
are therefore required for the arc behavior claims.

## Shared carrier contract

`ElementType::Arc` and analytic `TransformGeometry::Arc` preserve center, radius,
start, signed sweep and the source's y-axis convention. Reflection reverses signed
sweep; negative dilation rotates the start point by180° and retains positive
radius. Translation and rotation preserve radius/sweep. Zero-radius curved arcs
remain undefined. A collinear ThreePointArc has a linear analytic view represented
by existing Straight geometry with Segment mode, while its element remains Arc;
it must retain parameter values, measurements and live source type.

Reuse existing matrix operations, source chains, native flavors and unit readers.
Do not replace arc geometry with sampled paths: constrained points, projections,
measurements, intersections, loci and iteration outputs need a live analytic host.
Sampling is a rendering boundary. Existing arc readers expose clockwise_y=true for
CenterArc/ArcOnCircle and=false for ThreePointArc, so the transform must preserve
that convention rather than assuming a common angular sign.

The decoder helper `src/document/transform_arc_decode.rs` accepts only28-byte
class12/subtype67 headers, known exact reference counts and known operation
payload sizes. It checks backward/same-page/live IDs, a recursive arc source,
point/mirror roles, Degrees/Scalar numeric roles and marked-distance roles. Angle
cache validation and polar unit scale use the existing shared decoder readers.
Unknown layouts remain Unsupported and their bytes remain preserved.

## Remaining straight and polygon holes

Ready straight source types:72generic Transformed,32Ray,13Perpendicular,10Parallel,
9Segment,4Line,1GraphAxis. Some ordinary transform source predicates and
`fixed_transform_straight_mode` admit only literal/generated Ray/Line families,
while `line_points` already evaluates broader straight constructions. Relax only
with original schema plus Native source-role proof; class2/subtype12 denotes Line,
13Ray and14Segment, so substituting finite segments for infinite outputs is wrong.
Ready polygon source types:55generic Transformed and21literal Polygon. Fixed
Cartesian/mixed translation, point-angle rotation and fixed/point-ratio dilation
still use literal source predicates in selected decoder/validation arms. These
smaller shared source holes can be handled after arc prerequisites without adding
shape×operation Kind variants.


## Completed Native evidence and regression coverage

`transformed_arcs48_native_analysis.md` records 25 exact Native open/save controls:
four original documents and 21 fresh documents. Fresh controls cover all14
operations, the vector → reflection → negative dilation chain and a live endpoint
edit, numeric zero dilation with and without dependent leaves, and transformed
ThreePointArc, ArcOnCircle and collinear ThreePointArc sources. The normal Native
zero captures contain no output circles/arcs or constrained points. The collinear
capture retains the horizontal finite arc view and independent endpoint order.
All18 curved captures have independently predicted constrained points within
0.590px and sampled green output points within1.425px. This verifies geometry and
traversal; hidden Native captions do not provide displayed measurement values.

`src/document/tests/transformed_arcs.rs` compares public API-generated complete
object groups byte-for-byte against all17 baseline flavor/chain/zero Native files.
Additional retained Native tests cover zero with constraint/measurement records,
the three source variants and original rolling/rotation/reflection/polar/dilation
families. Independent formulas test samples at t=0,.25,.5,1, live source edits,
constraints, measurement readers, omitted output recovery and strict save/undo.
Malformed header/reference/unit/cache controls must remain Unsupported and preserve
original bytes. An analytic arc binding in the shared mapped iteration evaluator
checks live seed evaluation and final-only presentation without flattening its
source transform chain.

Two production issues surfaced during these tests and were fixed by the root:
arc stroke headers for fresh API wire groups, and strict roundtrip validation of
zero dilation while structurally valid arc/point/measurement records become
undefined. A degenerate transformed ThreePointArc remains an Arc semantic host;
`line_points` intentionally excludes it to avoid changing constraint dispatch.
