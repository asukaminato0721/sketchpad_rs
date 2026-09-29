# Locus probes

`tests/fixtures/locus_midpoint_base.gsp` has a point constrained to segment 3
(object 4), a segment from that point to a free point (object 6), and its
midpoint (object 7). Selecting objects 4 and 7 in Sketchpad 5 and choosing
Construct > Locus produced `tests/fixtures/locus_midpoint_original.gsp`.

The native file adds object 8 with a code 35 `0x07d0` header, `0x07d2`
references `[7, 3, 4, 6, 7]`, and a 40-byte `0x0902` range record. The range
is `0.0..1.0` with 500 samples. The references identify the target midpoint,
driver path, constrained driver point, midpoint segment, and target again.
The trailing range fields in this case are `04 00 01 00`, eight zero bytes,
`01 00 00 00`, and four zero bytes.

`tests/locus_generated_replay.txt` constructs the same object in Rust and
produces `tests/fixtures/locus_midpoint_generated.gsp`. Its
object records are byte-for-byte identical to the native object records;
differences between whole files are in native session metadata inherited from
the base file. Sketchpad 5 opened the generated file, displayed the locus,
and resaved it byte-identically. `tests/locus_native_edit_replay.txt` moves the
left endpoint to `(100, 220)`, producing
`tests/fixtures/locus_midpoint_moved_generated.gsp`. Sketchpad 5 displayed the
expected sloping midpoint locus and resaved the edited file byte-identically.

`tests/fixtures/locus_translated_point_original.gsp` uses the same
segment-constrained driver with a directly translated target. Its code 35
references are `[5, 3, 4, 5]`. The range remains `0..1`, but the native
40-byte record carries a finite value at offset 24 that differs from the
zero value used by the midpoint fixture. Sketchpad 5 opened and resaved the
Rust-generated and parent-edited translated-point fixtures byte-identically.

`tests/fixtures/locus_circle_driver_original.gsp` uses a point constrained
to a circle with a directly translated target. Its code 35 references are
also `[5, 3, 4, 5]`. The range is `0..2π`, the range mode changes from
`04 00 01 00` to `04 00 00 00`, and offset 24 again contains a finite
native value. The native maximum differs from Rust's `TAU` by a few ULPs.
Sketchpad 5 opened and resaved the generated circle locus byte-identically,
including after the center was moved.

`tests/fixtures/locus_arc_driver_original.gsp` uses a point on a three-point
arc and a translated target. Its references are `[6, 4, 5, 6]`; its range
uses `0..1` and the segment-mode bytes `04 00 01 00`. The generated file and
an edit moving the arc's middle point opened and resaved byte-identically in
Sketchpad 5.

`tests/fixtures/locus_two_translations_original.gsp` traces a point after
two successive translations of a segment-constrained driver. Its references
are `[6, 3, 4, 5, 6]`, recording the intermediate translated point. The
writer follows dependencies through supported points and segments, including
the segment host used by a midpoint. Generated and
parent-edited two-step files opened and resaved byte-identically in Sketchpad
5.

`tests/fixtures/locus_branch_midpoint_original.gsp` traces a midpoint whose
two endpoints are separately translated from the same moving point. Its
references are `[8, 3, 4, 5, 6, 7, 8]`: both branches and the midpoint's
segment host occur once, in object order. The generated and parent-edited
files opened and resaved byte-identically in Sketchpad 5.

`tests/fixtures/locus_ray_target_original.gsp` traces a ray from the same
moving point toward a fixed point. Sketchpad 5 uses the same code 35 subtype
43, `66 66 b2 00` header style, 250 samples, and zero final range flag as
the segment and line families. Each sampled ray is clipped at the viewport
boundary, retaining its moving point as its origin. Generated and
parent-edited ray families opened and resaved byte-identically in Sketchpad 5.

This probe also found that the 60-byte fixed polar translation payload
contains cached `-sin(angle)` and `cos(angle)` values at offsets 4 and 12.
Writing a non-90° angle without updating those fields made Sketchpad draw
the point at the wrong position. The writer now updates both fields and the
parser treats inconsistent caches as unsupported. A generated 180°
translation with a branched locus cross-opened byte-identically.

`tests/fixtures/locus_segment_target_original.gsp` traces a segment from a
segment-constrained moving point to a fixed point. Native Sketchpad 5 draws
a fan of segment samples. Its code 35 header uses subtype 43 (rather than
the point-locus subtype 42), the same `[target, path, driver, target]`
reference pattern, 250 samples, and a zero final flag in the range record.
The generated and parent-edited segment families opened and resaved
byte-identically in Sketchpad 5.

`tests/fixtures/locus_circle_target_original.gsp` traces a circle whose
center follows a point constrained to a segment, with a fixed radius from
another segment. The native locus draws a family of circles. It also uses
code 35 subtype 43, `[target, path, driver, target]` references, 250 samples,
and a zero final range flag. Its header bytes 12–15 are `66 b2 66 00`,
where the segment family uses `66 66 b2 00`. The Rust renderer approximates
each sampled circle with 64 line segments. Generated and parent-edited files
opened and resaved byte-identically in Sketchpad 5.

`tests/fixtures/locus_line_target_original.gsp` traces an infinite line from
the same segment-constrained moving point to a fixed point. Its code 35 record
uses the segment-family subtype 43, `66 66 b2 00` header style, 250 samples,
and a zero final range flag. Each sampled line is clipped to the viewport
during rendering and hit testing. The generated and parent-edited line-family
files opened and resaved byte-identically in Sketchpad 5.

`tests/fixtures/locus_arc_target_original.gsp` traces a three-point arc whose
start follows a point constrained to a segment. Its code 35 record uses subtype
43, `[target, path, driver, target]` references, 250 samples, zero final range
flag, and the `66 b2 66 00` header style also used by circle families. The
Rust renderer approximates each sampled arc with 64 line segments. Generated
and parent-edited arc families opened and resaved byte-identically in
Sketchpad 5.

`tests/fixtures/locus_arc_on_circle_target_original.gsp` traces an Arc on
Circle whose start point drives around its parent circle. Native Sketchpad 5
uses subtype 43 and the same `66 b2 66 00` style as the three-point arc
family, with a `0..2π` driver range. The native, generated, and parent-edited
files render an arc family; the generated files opened and resaved
byte-identically in Sketchpad 5. SVG export contains the sampled family.

`tests/fixtures/locus_parallel_target_original.gsp` and
`tests/fixtures/locus_perpendicular_target_original.gsp` trace constructed
infinite lines through a segment-constrained moving point. Sketchpad 5 uses
code 35 subtype 43, 250 samples, and the `66 66 b2 00` straight-family style
for both. The native references are `[target, path, driver, target]`; the fixed
direction-defining line is not part of the dynamic dependency list. Generated
and parent-edited parallel and perpendicular families opened, displayed, and
resaved byte-identically in Sketchpad 5.

`tests/fixtures/locus_bisector_target_original.gsp` traces an angle bisector
whose vertex is the segment-constrained moving point. Sketchpad 5 uses code 35
subtype 43, 250 samples, and the same `66 66 b2 00` style and
`[target, path, driver, target]` references as direct ray families. Each
sample remains a ray from the moving vertex. Generated and parent-edited files
opened, displayed, and resaved byte-identically in Sketchpad 5.

`tests/fixtures/locus_polygon_target_original.gsp` traces a triangle interior
whose vertex follows a segment. Sketchpad 5 uses code 35 subtype 43 with
`ff b2 66 00` fill bytes, 125 samples, and a zero final range flag. Its
unselected display fills the union of sampled triangles; the generated and
parent-edited files opened, displayed, and resaved byte-identically. The
probe also exposed a writer bug: a three-vertex polygon used subtype 53, while
Sketchpad uses 52. The writer now derives that subtype from the vertex count.

`tests/fixtures/locus_vector_polygon_target_original.gsp` traces the same
triangle after a fixed vector translation. Its code 35 references are
`[target, path, driver, source polygon, target]`, with 125 samples and the
polygon fill style. The header's stage field at bytes 20–23 is 2 instead of
the base polygon's 1. Constructing this locus also changes the translated
polygon's header bytes 18–19 from `51 5c` to `39 20`; the writer now applies
that change when the target has an active locus. Generated and parent-edited
files opened, displayed, and resaved byte-identically in Sketchpad 5.

`tests/fixtures/locus_polar_polygon_target_original.gsp` traces the same
triangle after a fixed 0° polar translation of 2.65 cm. Its code 35 references
and 125 samples match the vector-translated polygon locus; the stage field is
2, while header bytes 18–19 are `f0 3f`. Native locus construction refreshes
the polar translation's total-pixel-distance cache and replaces the trailing
cache bytes. The earlier port-generated base stored a conversion scale there,
which was a writer error. Checkpoint 29 corrects that generated base fixture
and uses total pixel distance consistently for all polar translations;
untouched original native files retain their raw bytes. A Rust-generated locus opened, displayed,
and resaved byte-identically in Sketchpad 5; moving the path endpoint updates
the swept interior in the port.

`tests/fixtures/locus_rectangular_polygon_target_original.gsp` traces the
triangle after a fixed rectangular translation of 2 cm horizontally and
−0.5 cm vertically. It uses the same five references and 125 samples, with
header bytes 18–19 set to `00 40` and stage 2. Native creation clears the
last eight bytes of the translation constraint. The generated file opened,
displayed, and resaved byte-identically in Sketchpad 5; a moved path endpoint
also updates the locus in the port.

## Undefined target samples

`tests/locus_undefined_samples_replay.txt` builds a segment-constrained driver
and an intersection of its line through a fixed point with a vertical line.
At the driver's midpoint its line endpoints coincide, making the intersection
undefined. The 11-object result in
`tests/fixtures/locus_undefined_sample_verified.gsp` opened and saved
byte-identically in Sketchpad 5 through Wine on 2026-09-26. Its file retains
the standard native code-35 locus representation; no new record fields are
needed.

The Rust sampler now keeps missing sample slots. Point loci join only adjacent
defined samples, and object families omit undefined members. Polygon sweeps
join only adjacent defined polygons. Regressions cover point curves, segment,
circle, arc, and polygon families, empty loci, and save/reopen. The cross-open
proves acceptance of the generated construction; exact native gap rendering
and discontinuities between finite samples still need separate verification.

## Native sparse captures and explicit free drivers

The 2026-09-29 corpus audit found two broader code-35 conventions. First, the
stored capture tail is not necessarily the complete live dependency slice.
Native files omit parallel branches, while others retain an independent style
or geometry branch from the target. Import now requires target, path, driver,
proof that the target depends on the driver, a unique subset of the proved
driver ancestry or validated full target closure, and the final target.
Unrelated or duplicate captures remain invalid.

Second, some loci store a `Free` base point as the driver and a separately
constructed path in reference slot 2. The path is often later than the point,
which explains why the point's own record has no constraint. During locus
evaluation only, the engine substitutes the corresponding point-on-segment,
ray, line, circle, arc, polygon, sector, or point-curve kind. The original
point remains free. Admission requires same-page live objects, the exact native
range/closure profile, a supported path family, and proof that the path does
not depend on the driver; ordinary authored loci still require a constrained
point.

Native geometric sweeps also use both observed word-32 presentation values,
and code-43 filled targets include the already supported disk, sector, and
circular-segment region families. Their outlines and surfaces now use the
shared filled-region sampler.

Point-output `Locus`, `ParameterLocus`, and `SampledMapped` objects are curves
in their own right. Native code-43 records can sweep one of these curves along
an outer driver. The evaluator replays the earlier validated curve independently
at each outer sample, preserving undefined gaps and bounding both presentation
budgets. Geometric-output loci are deliberately not reclassified as curves, so
this does not create unbounded recursive target inference. An authored nested
point-locus control covers evaluation, serialization, and reopen inside the
existing locus-family test.

The compiled scan restores 107 drawing code-35 loci plus 47 downstream objects.
Unsupported slots fall 4,021→3,867; all-kind-complete documents rise
1,125→1,149 and drawing-complete documents rise 1,153→1,178. Partial views
fall to 895. All 1,599 source files still parse and round-trip byte-exactly;
embedded prototype coverage remains 4,744/5,356. The existing locus tests cover
the transient free-driver path, cyclic-path rejection, sparse capture
validation, sampling, save, and reopen without adding another test case.

## Parameter-locus mapped collections

The remaining decoded-parent iteration frontier showed that `ParameterLocus`
was already evaluated by the same point-locus sampler as `Locus`, including its
explicit numeric range, but was absent from the mapped construction and binding
classifiers. Adding it to those shared roles restores ten native code-89 roots,
25 code-77 bindings, and eight downstream objects without a new evaluator or
object variant. The existing locus regression now opens the compact
`双控旋梯（hjx4882）.gsp` example, verifies its exact bytes, evaluates 1,440
bound strokes, hides the direct source, and verifies the bound collection after
save/reopen; no new test function was added.

The same hidden-source state was opened under the supplied Windows Sketchpad 5
using Xvfb/Wine. Native still displayed the red five-level spiral from the
visible binding, matching the port SVG, and Ctrl+S preserved the 4,532-byte file
with SHA-256
`53270fd8b9997ed80942028d8fdfa9d424097e1da0c2ccf51e66924242dfb14f`.
The full corpus remains 1,599/1,599 byte-exact; unsupported slots fall
2,442→2,399 and complete documents/drawings rise 1,305/1,335→1,314/1,345.
