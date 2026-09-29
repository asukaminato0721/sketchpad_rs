# Broad feature priorities

User direction: prioritize features with broad coverage; visual output does not
need to match the original application pixel-for-pixel. Copying and custom tools
remain deferred. File integrity, dependency semantics and live calculation stay
required.

The current point-attached-picture pass restores five measured drawing slots:
four code-99 picture roots and one downstream object. Point attachments now
retain finite native rotations as well as positive scales, with the PNG center
fixed to the live anchor. The importer also preserves the exact hidden internal
`[1]` metadata payload observed in two files without treating it as an editable
object label. Unsupported slots fall 3,735→3,730, complete documents rise
1,167→1,170, drawing-complete documents rise 1,196→1,199, and all 1,599
originals remain byte exact. No unsupported class-16/code-99 picture profile
remains on a drawing page; twelve prototype slots remain outside this pass. See
`image_analysis.md`.

The preceding transformed-picture pass restores 59 measured drawing slots: 37
class-16 transform roots and 22 downstream objects. Picture transforms now use
the shared typed affine model for vector and marked translations, point-angle
and numeric rotations, fixed and numeric dilations, and reflections. Admission
requires an earlier decoded picture source, valid live operand roles, bounded
native records, and a duplicated PNG dimension cache. Unsupported slots fall
3,794→3,735, complete documents rise 1,157→1,167, drawing-complete documents
rise 1,186→1,196, and all 1,599 originals remain byte exact. Transformed crops
remain separate because they carry clip geometry and a different size cache.
See `image_analysis.md`.

The preceding vector-translation pass restores 73 measured drawing slots: 47
code-16 objects and 26 downstream objects. It admits native generic
class-4/subtype-15 polygons only when their source has the same header, plus
class-16/subtype-78 pictures with a proved code-99/code-100 source, two live
point endpoints, exact record layout, and matching duplicated PNG dimensions.
Picture placement composes the source's live affine matrix with the vector.
Unsupported slots fall 3,867→3,794, complete documents rise 1,149→1,157,
drawing-complete documents rise 1,178→1,186, and all 1,599 originals remain
byte exact. See `vector_polygon_translation_analysis.md` and
`image_analysis.md`.

The preceding locus pass restores 154 measured drawing slots: 107 code-35 loci
and 47 downstream objects. Native capture lists may omit branches or include a
validated independent target branch, both observed presentation words are
retained, and filled regions share the existing locus surface sampler.
Point-output sampled objects can be swept as bounded curve collections. Native records
that keep their base driver `Free` use the explicit locus path transiently;
admission proves that the later path does not depend on the driver. Unsupported
slots fall 4,021→3,867, complete documents rise 1,125→1,149,
drawing-complete documents rise 1,153→1,178, and all 1,599 originals remain
byte exact. See `locus_midpoint_analysis.md`.

The current attached-picture pass restores 290 measured drawing slots: all 30
code-85 two-point rectangles, all 106 code-100 three-point parallelograms, all
39 code-108 crops, and 115 downstream objects. Unsupported slots fall
4,311→4,021, complete documents rise 1,108→1,125, drawing-complete documents
rise 1,134→1,153, and
all 1,599 originals remain byte exact. Placement follows live point parents in
the shared canvas, SVG, picking, and iteration paths. Resource-table mutation
and fresh attached-picture authoring remain guarded. See `image_analysis.md`.

The preceding Presentation-action pass restores 168 measured slots: 135 drawing
code-62 buttons, 29 downstream objects, and four embedded-prototype buttons.
Unsupported slots fall 4,479→4,311, complete documents rise 1,089→1,108,
drawing-complete documents rise 1,114→1,134, and all 1,599 originals remain
byte exact. Native property probes establish the Before Starting flags,
simultaneous stop policies, and tenths-of-a-second elapsed/pause word. The
separate native progress/state word is preserved without invented semantics.
The remaining 64 drawing code-62 objects belong to Animation, Movement, Scroll,
or action code 8; no corpus-observed Presentation record remains rejected. See
`presentation_button_analysis.md`.

The preceding animation-action pass restored 102 measured slots: 94 drawing
code-62 buttons, six downstream code-73 objects, and two embedded-prototype
buttons. Corpus-observed opaque mode tags are preserved through edits; strict
target validation admits only implemented animation targets, including point
curves, closed sector boundaries, and prototype path points. See
`animation_button_analysis.md`.

Checkpoint95 admits nested code26 polar point transforms whose source is an
already decoded `DualMarkedPolar` point. Ten Native controls independently
establish live source, distance, and angle behavior with byte-exact saves.
The corpus restores 105 drawing slots, lowering unsupported9,331→9,226; all
1,599 originals remain byte exact. Other point source flavors still need
separate evidence. See `point26_dual95_implementation.md` and
`point26_dual94_native_analysis.md`.

Checkpoint94 imports strict code99 pictures anchored to a live point, reusing
the PNG catalog and the shared image transform for canvas, SVG, and picking.
The 97 exact zero-rotation objects in44 files restore 97 drawing slots,
lowering unsupported9,428→9,331 while all1,599 originals remain byte exact.
Sixteen Native controls establish free/constructed anchor movement and scale
response; structural picture-resource editing remains guarded. The earlier276
sole-root reach was an upper bound, not additional observed recovery. See
`anchored_image94_implementation.md` and `frontier93_analysis.md`.

Checkpoint93 consumes native `0x0100` only as a suffix of π in the shared
function grammar. Thirteen matching code71 roots unlock 216 actual drawing
function slots; unsupported9,644→9,428, with all1,599 originals byte exact.
The six Native controls establish record preservation and a live upstream
scalar chain, while function-value parity remains unobserved. The imported
music example remains structurally guarded for edits. See
`referenced_function93_implementation.md` and `frontier92_function71_audit.md`.

Checkpoint92 adds complete ordered native R/G/L channel styles to the shared
rich-text run parser and renderer. Seven Native controls are whole-file exact
and isolate subspan color from position. The strict1..256 import restores83
code0 text slots, lowering unsupported9,727→9,644 with all1,599 originals
byte exact. Zero-channel inheritance, mixed-format content replacement, and
other markup remain separate. See `rich_style92_implementation.md` and
`frontier91_rgb_recovery.md`.

Checkpoint91 adds a strict read-only PNG resource catalog for independent
native image objects. Each object selects its own validated resource by index
and keeps its affine transform/visibility. Six Native controls are whole-file
exact. The corpus restores146 image slots, lowering unsupported9,873→9,727
while all1,599 originals remain byte exact. Two labeled variants and
multi-resource structural writing remain outside this admission. See
`image_objects91_implementation.md`.

Checkpoint90 restores 408 imported URL buttons whose strict native
action records contain opaque suffix bytes after the terminated URL. Existing
generic action-reference text decoding then recovers 241 more objects: 649
drawing slots total, unsupported10,522→9,873, with all1,599 originals byte
exact. Eight Native passive controls are whole-file exact; opaque-suffix URL
replacement remains guarded pending a mutation rule. See
`url_suffix90_implementation.md` and `url_suffix90_native_analysis.md`.

Checkpoint89 admits Native class4/subtype15 `Colorized(Polygon)` into the
existing generic colored-object DAG and mapped iteration evaluator. Eleven
Native whole-file-exact controls isolate direct and mapped visibility, live
geometry/color, and four distinct depth colors. The compiled corpus restores
66 drawing slots, lowering unsupported10,588→10,522; all1,599 originals
remain byte exact. The source-polygon-hide control has no valid Native result.
See `colorized_polygon89_implementation.md`.

Checkpoint88 extends the shared mapped DAG to carry `Colorized(Disk)` and
draw analytic filled disks with per-generation colors through a generic
binding. Six Native visibility states are whole-file exact; scalar and Free
controls establish live colors and positions. The compiled corpus restores
64 slots, lowering unsupported10,652→10,588, with all1,599 originals byte
exact. Fresh authoring still has the bounded depth writer, and source-interior
hiding remains unproved. See `colorized_iteration88_implementation.md`.

Checkpoint87 adds a bounded native three-point AngleMarker node with derived
one/two-arc presentation and signed numeric aliases using page angle units.
Native hide, motion, signed-value and unit-switch controls cover 23 whole-file
exact saves. The compiled corpus gains 160 drawing slots and seven complete
documents, lowering unsupported10,812→10,652 with all1,599 originals byte
exact; 1,411 scope-aware test executions pass. Other marker flags and alias
parent roles remain outside the proved boundary. See
`angle_marker87_implementation.md` and `angle_marker87_native_analysis.md`.

Checkpoint86 adds an intrinsic Disk element to the shared Colorized binding and
filled-region evaluator, with Disk transforms explicitly rejected. Native
Spectrum and Grayscale controls show independent wrapper fill, live color and
source motion, and all11 whole-file saves exact. The strict25-root import plus
two prototype slots lowers unsupported10,839→10,812 across all1,599 exact
originals; 1,404 scope-aware tests pass. Other disk-color operand profiles and fresh-object authoring remain
separate; see `colorized_disk86_implementation.md`.

Checkpoint85 imports 31 strictly bounded code120 named aliases of the two live
angle values through one generic numeric-alias DAG node. Native ten-state
controls establish default/explicit caption names and independently separate
the alias text from its parent angle marker. The compiled scan restores 302
drawing and 11 prototype slots, unsupported11,152→10,839, with all1,599
originals byte exact and 1,400 scoped test executions passing. Remaining
code120 source categories and new-object authoring are separate; see
`angle_alias85_implementation.md`. The next independent bounded candidate is
code75 coloring of CircleInterior by a scalar calculation, pending Native
geometry/style controls (`frontier84_analysis.md`).

Checkpoint84 adds two strictly bounded three-point angle values with a shared
geometry kernel but opposite directed conventions: code115 uses S mod360°,
code116 uses −S mod360°. Native one-Free edits and unchanged downstream labels
prove both degrees readouts; eight full-file saves are exact. The corpus restores
36 drawing slots plus five isolated prototype slots, lowering unsupported
11,193→11,152 while all1,599 originals stay byte exact. Alternate090e packets
and downstream labels remain separate. See `angle_values84_implementation.md`.

Checkpoint83 admits102 intrinsic CircularSegmentBoundary objects with a strict
class13/sub69 and exact no-marker group. The shared Arc-derived outline closes
with a chord, preserving live geometry without stored polygon vertices. A
seven-object Native original and endpoint edit isolate 3,570/8,413 fill pixels;
14 whole-file saves are exact. All1,599 corpus originals stay byte exact and
unsupported falls11,295→11,193. A dependent boundary point has an unproved
parameter mapping and remains opaque. See `circular_segment83_implementation.md`.

Checkpoint82 admits strict imported subtype-15 PointOnPolygon constraints on
ordered polygon walks of differing lengths. Native visible controls verify an
eight-slot walk through parameter and vertex edits, a twelve-slot walk with
repeated vertices, and a high-word-1 266-slot walk using stored edge70; all26
tested states save whole-byte exact. The compiled corpus
restores290 slots behind14 points, lowering unsupported11,585→11,295 while
all1,599 originals stay exact. Extra-marker layouts remain unproved. See
`polygon_walk82_implementation.md` and Native analysis.

Checkpoint81 admits a fixed-rotated Sector as the source of another fixed
rotation under the same exact wire layout and Free pivot. Native controls
isolate the second object and prove two independent 90° paths through ordered
matrices; all tested full-file saves are exact. The compiled corpus restores
160 code27 slots, unsupported11,745→11,585, while all1,599 originals remain
byte exact. See `sector_transform81_implementation.md` and Native analysis.

Checkpoint80 extends the analytic Sector element to a strict Segment reflection
and a chained fixed rotation of that reflected Sector. Seven whole-byte-exact
Native saves isolate each operation, while the compiled corpus gain is19 more
slots (18 code34, one code27), lowering unsupported11,764→11,745. Unproved
auxiliary-marker reflection layouts and non-Segment mirrors stay separate.
See `sector_transform80_implementation.md` and the Native analysis.

Checkpoint79 admitted the first analytic Sector element operation: fixed rotation
of an intrinsic sector about a Free point. Six byte-exact Native saves establish
source geometry, angle change and live upstream movement. The strict admission
restores17 code27 roots while all1,599 originals remain byte exact. A frontier
of194 unsupported descendants was not decoded by that step; checkpoint80 now
adds the reflected/chained examples, while mapping prerequisites still remain.
See `sector_transform79_implementation.md`,
`sector_transform79_native_analysis.md` and `frontier79_analysis.md`.

Checkpoint78 admits one empty `232e` record only in the strict scalar
coordinate-measurement 65/66 layout. A visible Native x control is whole-file
exact and live; a paired hidden x/y file preserves both marker records in
Native saves but normalizes unrelated bytes, and its captions cannot be read
independently. The compiled gain is1,017 slots across85 measurement roots,
340 calculations,336 bindings,168 mappings and84 anchored texts, while all
1,599 originals remain byte exact. See
`coordinate_measurements78_implementation.md` and
`coordinate_measurements78_frontier.md`.

Checkpoint77 extends the standalone code26 Point operation to a MarkedPolar
transformed Point source. Fifteen Native states save whole-file exact; isolated
markers prove upstream source and angle changes after a bounded correction to
PointValue of a PointOnSegment measured against another Segment. Free-point
projections remain unbounded. The scan restores68 slots, including16 code26
roots and downstream polygon/translation/rotation objects, with all1,599
originals byte exact. The next broad parent-ready cohort is coordinate
measurements 65/66 with a shared empty `232e` record (see
`coordinate_measurements78_frontier.md`). See
`code26_markedpolar77_implementation.md`.

Checkpoint76 extends standalone code26 points to a FixedPolar-transformed
Point source through the same live polar expression. Four whole-byte-exact
Native states isolate the marker and demonstrate an upstream `(40,20)` move;
the saved move is field-level synthetic because the public edit is blocked by
unrelated partial-document validation. The measured gain is52 slots, including
51 code26 roots, with all1,599 originals byte exact. MarkedPolar has16 clean
ready roots across four files and is the next independent source-flavor proof.
See `code26_fixedpolar76_implementation.md` and `code26_frontier76_analysis.md`.

Checkpoint75 admits only code71 high-word expression wrappers with upper word
1 or 2, exact low token count and all existing syntax/source checks. The writer
retains the high word on edits. Native high1 and high2 controls establish live
source behavior, with partial-document edits explicitly synthetic; high2 Native
save also normalizes unrelated metadata. The measured gain is28 slots, including
19 code71 roots, while all1,599 originals remain byte exact. The next function
frontier consists of distinct syntax and longer-token cohorts, not a blanket
high-word allowance. See `referenced_function75_implementation.md`.

Checkpoint74 extends standalone code26 points to Vector-transformed Point
sources through the same expression and dependency evaluator. Four Native
states isolate the point and show that a `(40,20)` upstream move propagates;
the saved moved state is clearly synthetic because the public edit is blocked
by an unrelated partial-view interior. The measured gain is96 slots, including
26 code26 points and downstream translations/polygons; all1,599 originals
remain byte exact. Other transformed source flavors and code102 programs still
need separate proof. See `code26_transformed74_implementation.md`.

Checkpoint73 admits only standalone code26 Point outputs with a Free source
and live Degrees/Length operands through the shared polar transform evaluator.
Native controls in two independent originals prove source, angle and distance
movement; synthetic zero/negative-distance controls retain the original code26
group. The measured recovery is129 slots, including50 code26 roots, with all
1,599 originals byte exact. Three ready Free-source roots still carry unproved
label/font records; transformed sources, code102-linked programs, Segment
outputs, and Scalar operands remain separate. See `code26_standalone73_implementation.md`
and `point_construction73_frontier.md`.

Checkpoint72 fixes a shared referenced-function grammar case: a grouped
unary operand may continue through an infix operator before the function's
closing token. The final scan restores623 object slots, including154 code71
definitions and105 code72 plots; all1,599 originals remain byte exact.
The original calendar and isolated Native controls establish live year and
function values, while the original remains structurally partial. Scope-aware
gates pass1,373 executions with seven requested-deferred copying/custom-tool
tests excluded. The next broad prerequisite should be selected from the new
ready-root audit, with code26 point construction and the remaining code71
wrapper/token cohorts kept distinct. See `referenced_function72_implementation.md`
and `plot_point_frontier72_analysis.md`.

Checkpoint70's scoped point-output Locus consumer restores 261 additional
object slots in the final corpus scan, including 50 mapped roots and 98
bindings, while preserving all 1,599 whole-file bytes. A Native original,
parameter edit and visibility edit distinguish direct source traces from
generated binding traces; public writer states also cross-open exactly. The
remaining collection frontier includes nested Locus paths and circle/arc
outputs. See `scoped_point_locus70_implementation.md`.

Checkpoint71's bounded rich-text operators restore 171 more slots through the
shared referenced-text AST, including 35 captions and downstream numeric and
iteration objects. Seven Native controls separate a π suffix from a live
numeric subscript and show that stale cache text does not replace current
parent values. All 1,599 originals remain byte exact; 1,224 workspace test
executions pass. The current ready-root audit should guide the next broad
prerequisite, not the raw remaining code counts. See
`referenced_token71_implementation.md` and `frontier71_analysis.md`.

Checkpoint67's scoped straight-Locus collection consumer recovers 38 drawing
objects across code77/89/76 while preserving all1,599 full-file bytes. Native
and public writer evidence is recorded in `scoped_collection67_implementation.md`.
The remaining mapped-root cohort represents procedural scalar reconstruction
feeding ordinary outputs, not directly drawable inner mapping objects; the
actual original role and failed direct-root probe are recorded in
`nested_collection67_failure_analysis.md`.

The corrected drawing-only numeric prerequisite audit covers exactly the 20,270
unsupported drawing code-48 objects from corpus checkpoint 25. It excludes
embedded prototype groups before matching raw groups to drawing object IDs.
Earlier raw-record audits could misalign those namespaces, so their larger
record-layout failure totals should not guide priorities.

| Calculation failure | Drawing objects |
| --- | ---: |
| Numeric parent unit unavailable | 19,472 |
| Unsupported expression token | 394 |
| Invalid source token | 180 |
| Incompatible calculation units | 168 |
| Invalid reference count | 46 |
| Unsupported header | 7 |
| Incompatible function units | 3 |

The reason counts sum to 20,270. Missing-parent reference counts can exceed the
object count because a calculation has multiple parents. The largest direct
missing parents are other calculations (35,179 references), code-65 and code-66
coordinate measurements (3,899 and 2,740), code-41 measurements (2,522), code-94
point values (2,113), code-37 distances (1,497), and code-47 ratios (617). These
counts identify dependency families, not independently implementable objects:
a measurement may itself depend on an unsupported transformed or iterated point.

Next implementation priorities are the shared prerequisites of coordinate and
distance measurements, then general calculation/function and iteration chains.
Trace unsupported parents to their root construction before choosing a change;
recovering one reusable root should unlock downstream measured values and
calculations. Further one-off table presentation profiles have lower priority.

Evidence: `/tmp/sketchpad_broad_numeric_import_audit.log` (exit 0), generated by
[the drawing-only audit](numeric_import_audit.rs), and
`/tmp/sketchpad_corpus_checkpoint25.log` (1599 exact round trips, exit 0).
The raw calculation header profile found all 38,459 drawing calculations use the
already accepted numeric prefix and display modes 0, 1 or 3, so loosening header
checks would not address the main prerequisite gap. Raw vector-transform counts
also predominantly describe point/segment chains, not isolated missing ray/line
variants. Those raw profiles include decoded objects and must not be mistaken
for unsupported-object counts.


Implemented broad prerequisites (checkpoint 27): marked-distance shape chains,
coordinate captions and supported display/cache fields, and empty point-value
attributes. Across drawing objects these recover 12404 slots relative to
checkpoint 25, including 5567 calculations; all 1599 source files retain exact
byte round trips. Shared decoding additionally recovers 530 prototype slots,
without adding custom-tool authoring or copying support. Root dependency audit:
[broad measurement analysis](broad_measurement_dependency_analysis.md).


Checkpoint 29 adds a further 5993 drawing slots (1423 calculations) by handling
fixed-polar source chains and native angle conventions. The shared polar cache
fix also corrects fractional-distance geometry of existing decoded objects;
that semantic gain is separate from the decoded-slot count. An actual writer
file with fractional-distance segment/circle/polygon chains opened and saved
byte-identically in Sketchpad 5. See [polar evidence](polar_translation_units_analysis.md).
The next marked-distance prerequisites include typed Length Parameters; Scalar
numeric sources still need a proved conversion policy. Imported graphs with
undefined coordinates also expose save-validation restrictions that should be
reviewed separately from parser coverage.


Checkpoint 30 implements typed Length Parameter distance sources through the
shared numeric-unit and value interfaces, including the editor's Mark Distance
command. It recovers another 1219 drawing slots with1599 exact source round
trips. Actual writer Native verification and live/edit/undo tests are recorded
in [typed distance evidence](typed_length_marked_transforms_analysis.md).
Scalar distance conventions and undefined imported graph save restrictions
remain separate work, as does source-free centimeter calculation authoring.


Checkpoint 31 completes the source-free quantity prerequisite using the existing
calculator AST. Original Length and inverse-angle calculations recover 138 more
drawing objects including downstream consumers; source-free area authoring has a
separate native writer control. All 1599 corpus round trips remain exact.
See [quantity calculation evidence](source_free_quantity_calculations_analysis.md).
Future prioritization should continue to use unsupported dependency roots rather
than isolated enum counts: calculator/code-48 source families and iteration
bindings/code-77 have substantial reach. Copying and custom tools remain deferred.


Checkpoint 32 extends the shared calculator beyond the old 16-input and
32-authored-token limits. Original 20-input arithmetic and a native-saved 51-input
writer control are verified; another 156 drawing objects (80 calculations) decode.
All original files still round trip exactly. Evidence:
[many-input calculations](many_input_calculations_analysis.md). Remaining root
counts overlap: 4749 root-dependent calculations in the scalar calculation
category are not 4749 recoveries from this change.


Checkpoint33 addresses the broad Scalar-source marked-distance root. The common
conversion is proved by native measured +3/-3/0 controls and actual API writer
cross-opening. It recovers 6610 drawing objects (760 calculations) plus108
prototype slots, with all1599 source files preserved. Copying and custom-tool
authoring stay deferred. See [scalar distance evidence](scalar_marked_distances_analysis.md).


Checkpoint34 targets the widely used center-arc dependency root. It recovers
4941 drawing slots (361 calculations) plus26 prototype slots and three additional
fully decoded documents. Native controls distinguish radial direction from an
on-circle endpoint; actual API outputs prove defined/undefined saves. Shared
arc interfaces carry the benefit to measurements and point plotting. Copying
and custom-tool authoring remain deferred. See [center arc evidence](center_arc_analysis.md).


Checkpoint35 removes the coordinate reader's extra zero-only presentation-tail
gate, after original-payload and Native semantic/API writer checks. This recovers
2672 drawing slots including1232 calculations, while preserving all1599 originals.
The direct-parent audit distinguished this reusable format gap from coordinate
measurements blocked by unknown geometry. See
[coordinate presentation evidence](coordinate_presentation_tail_analysis.md).


Checkpoint 36 addresses shared expression-unit inference after isolating 588
known-input formula failures. Native calculator controls prove that unlike
quantities add/subtract by magnitude and return Scalar. The change recovers
5373 drawing objects (4351 calculations), beyond the direct failure cohort,
with all 1599 original round trips exact. Copying and custom tools remain
deferred. See [mixed arithmetic evidence](mixed_quantity_arithmetic_analysis.md).


Checkpoint 37 resolves typed-exponent unit failures in the shared calculator.
The checkpoint 36 known-input audit identified 92 such failures; recovering
those roots also recovers calculation chains and geometry, for 398 additional
drawing slots including 392 calculations. Length, Area and Degree exponent
sources retain live dependencies, and pixel calculation literals use Native's
centimeter conversion. The remaining known-input gaps include unsupported
expression tokens and function argument units; missing numeric parents still
account for most unsupported calculations. Native controls, actual writer
proof and the original trigonometry sample are documented in
[quantity exponent evidence](quantity_exponents_analysis.md). Copying and
custom-tool support remain deferred.


Checkpoint 38 implements the common referenced-function call prerequisite in
calculations and numeric iteration labels. It recovers 139 drawing slots
including 75 calculations, while removing expression-token/source-token failures
for the common call encoding. Remaining function calls now expose their actual
unsupported function parents in the audit rather than hiding them as parser
errors. All 1599 source files retain their bytes. Native controls and actual
API writer proofs include nested calls, derivatives, typed arguments, angle
modes and numeric iteration; see [function call evidence](function_applications_analysis.md).

The role-aware audit leaves 5982 unsupported calculations: 5968 have missing
numeric/function parents, nine fail headers and five fail function-unit rules.
The refreshed dependency-root audit identifies code-59 roots affecting 1566
calculations in 93 files, code-101 roots affecting 1140 in 27 files, and code-71
referenced functions affecting 812 in 86 files. These overlapping counts guide
next work on function definitions/plot dependencies and iteration bindings;
they are not additive recovery predictions. Copying and custom tools remain
deferred. Evidence: `/tmp/numeric_import_audit38.log` and
`/tmp/broad_measure_roots_checkpoint38.log`.


Checkpoint 39 restores common primitive-function roots before tackling referenced
function definitions. Definitions now share all 13 calculator built-ins and π/e
constants; valid signed function-caption positions also decode. This restores
271 drawing slots, including 119 downstream calculations, reducing unsupported
slots from 64477 to 64206. All 1599 files round-trip exactly; embedded prototype
counts are unchanged. The calendar's `1-sgn(x-trunc(x))` predecessor and original
negative-position music function are retained as real corpus regressions.
See [native creation and actual writer evidence](function_primitives_analysis.md).
Referenced/parameter-dependent function definitions, plots and iteration roots
remain next priorities. Copying and custom-tool authoring remain deferred.


The referenced-function prerequisite now has a validated expression API with
numeric/function source roles and live callbacks. Native-created parameter and
nested-function controls plus actual contextual record-writer edits establish
code71 headers, source indexes and exact native saves. The drawing audit finds
700 such primitive definitions,694 within the token bound;245 already have
supported parents. Document/plot/editor integration is still required and no
additional corpus recovery is claimed yet. See
[referenced expression evidence](function_references_analysis.md).


Checkpoint 41 connects the referenced expression API to code71 documents,
source-role validation, live function/calculation dependencies, authoring/editing,
Cartesian/parametric plots and numeric iteration. Native evidence requires
Degree predecessors to convert to radians in radian mode and mapped coefficients
to advance inside each function call; both behaviors are implemented. The actual
38-object original/edited API outputs save exactly in Native and match numeric
and iterative results. New definitions use the destination drawing's font
namespace; imported presentation records remain untouched.
The final scan restores 919 drawing slots (229 code71 definitions,439 code48
calculations and251 other descendants), plus three prototype slots in the shared
reader. All1599 source files remain exact; unsupported slots fall to63284.
See [implementation and Native evidence](function_references_analysis.md).
Remaining priorities include wider function/plot and iteration roots, function
wrapper variants and result-unit propagation. Copying and custom-tool authoring
remain deferred.


Checkpoint 42 targets the code59 coordinate-unit root identified by the dependency
audit, together with literal code67/68 point references. Numeric calculations and
parameters now define stored axis scale through the common numeric interface;
finite segments use shared geometry. The point reader no longer treats the
reference count as object1. This restores 3841 drawing slots, including282
calculations, plus one prototype slot, with all1599 original bytes preserved.
Nine actual API writer states have exact Native-save proofs, including zero,
negative and restored units. See [coordinate evidence](coordinate_units_analysis.md).

The refreshed drawing-only iteration audit finds706 unsupported iteration roots
with ready direct parents (45 code76 and661 code89), and1766 code77 bindings whose
iteration root has ready direct parents. These are overlapping prerequisite
cohorts, not additive recovery predictions. General mapped-DAG iteration is the
next broad target: evaluate carried constructions under simultaneous seed
substitution instead of adding per-shape variants. Code62 action layouts also
have3062 ready-parent objects, but their behavior requires separate Native
controls. The reproducible audit is [iteration_root_audit.rs](iteration_root_audit.rs),
with final results in `/tmp/iteration_root_audit42_final.log`.
Function-call result-unit propagation and broader mapped-coordinate variants
remain prerequisites for some numeric descendants. Copying and custom-tool
implementation remain deferred; exact display matching is not required.


Checkpoint43 implements the common simultaneous state substitution prerequisite
for derived point and Scalar seeds, with generic point/segment transforms and
supported coordinate DAGs. It restores269 drawing objects, including110
iteration definitions and159 bindings, while preserving all1599 source bytes.
Actual API writer controls and the original five-branch pentagon establish
Native behavior. No new transform-by-shape kinds were added. See
[derived graph evidence](mapped_derived_transform_analysis43.md) and
[numeric/mixed state evidence](numeric_mapped43_analysis.md).
The remaining broad iteration cohorts need geometric numeric inputs, path and
intersection graphs, wider carried shapes, or consolidation of the single-seed
specialized representations. They remain ahead of visual-fidelity work. Copying
and custom-tool support are still deferred.

Checkpoint 44 extends the same mapped evaluator to ordinary point/path,
intersection, circle, polygon, arc and measurement prerequisites. The initial
strict cohort audit restores 108 definitions and 314 point/segment bindings in
77 drawings, without adding shape-specific iteration kinds. Explicit carried
nodes recompute per state; omitted dependencies retain their original values,
including function coefficient environments. Runtime substitution preserves the
original element kinds. Native proofs cover circle/segment path points, three
intersection families, live angle/length measurements and an original arc graph.
Undefined carried geometry retains valid earlier images and independent bindings.
Single-seed consolidation, wider binding types and action prerequisites remain
broad follow-up targets; copying and custom tools stay deferred.
See [path/intersection evidence](mapped_path_intersection_analysis44.md) and
[measurement/arc evidence](measurement_mapped44_analysis.md).

Checkpoint45 implements the single-seed point/Scalar cohort through the common
mapped DAG:404 roots and720 point/segment bindings. A recursive polygon source
fix for reflection adds1314 drawing objects. The final corpus scan recovers2443
slots including5 prototype slots, leaving56308 unsupported; all1599 originals
remain exact and scoped gates pass825 tests. No shape×transform Kind products
were introduced. Source/mirror edits, canonical legacy writer layouts, real
midpoint-host dependencies, stable carried authoring order and frozen external
hosts have Native evidence. See [single-seed evidence](single_seed_iteration_analysis45.md)
and [polygon source evidence](transformed_curves45_analysis.md).

The refreshed [numeric prerequisite audit](numeric_import_audit.rs), logged in
`/tmp/sketchpad_numeric_audit45.log`, finds5142 unsupported drawing calculations:
5128 unavailable typed parents,9 header layouts,5 incompatible function units.
The remaining calculator work still mainly follows missing construction roots.
The [action-root audit](action_roots45_analysis.md) finds3064 of5540 unsupported
actions have decoded direct parents. Original payload+paired HTM prove lists of
movement pairs and optional labels; most movement sources need path/control
setters rather than only Free points. Hide/Show action option words must retain
their distinct semantics, not be discarded. Shared action layout/pair support
and wider iteration/arc outputs remain broad next targets. Copying and custom
tools remain deferred; pixel-identical raster output is not a release goal.


Checkpoint 46 implements the shared action-record prerequisite identified by
[action audit45](action_roots45_analysis.md): optional Native labels/metadata
and multi-pair Movement restore 1,799 drawing objects, 47 prototype slots, 1,846
slots total. Runtime movement uses one typed value interface across point,
path and numeric constructions. Copying and custom tools remain deferred.
Next high-coverage candidates remain broader iteration bindings (code 77),
the root dependencies behind 5,142 calculations, and remaining action families
(code 62 now 3,741). Native normalizes scalar constant calculations into a
Parameter layout with mode 0; inspect this shared numeric root against actual
GSP/HTM before widening its parser. Counts are root-priority evidence, not
runtime parity claims.


Checkpoint 47 extends the shared iteration output readers to circles, polygons,
rays, lines and arcs, recovering 389 drawing bindings in 102 files plus one
prototype slot. Authoring, rendering, picking, styles and level selection use the
same mapped DAG and existing transform model. Typed-Length circle radii repair a
shared prerequisite exposed by an original spiral. All 1,599 corpus originals
remain byte-exact; 859 scoped checks pass. See [binding evidence](iteration_bindings47_analysis.md)
and [Native evidence](iteration_bindings47_native_analysis.md).

The remaining code 77 cohort has 5,823 drawing slots and mostly needs unsupported
iteration parents or source constructions. Numeric calculations still have 5,142
unsupported slots, chiefly missing typed parents. Prioritize those common roots
and remaining action families over isolated enum variants. Copying and custom
tools stay deferred; matching every rendered pixel is outside the requested scope.


Checkpoint 48 applies two shared roots: analytic Arc in the existing generic
transform model and the Hide/Show option/caption policy. The actual final scan
recovers 3,997 slots (3,937 drawings and 60 prototypes), leaving 50,075 unsupported.
All 1,599 originals remain exact. Arc adds 503 slots across live transform chains
and downstream consumers; visibility adds 3,494 slots without requiring a separate
button enum for each mode or target type. See [arc evidence](transformed_arcs48_native_analysis.md) and [visibility evidence](visibility_options48_analysis.md).

Next prioritize missing iteration/source prerequisites behind 5,809 code-77
bindings and typed construction roots behind 5,139 calculations. Remaining
code-62 actions have 293 drawing slots. Favor common readers, shared geometry and
value adapters over special-case enum products. Copying and custom tools remain
deferred, and exact raster matching remains outside scope.


The refreshed read-only audits are logged in
`/tmp/sketchpad_numeric_import_audit48.log` and
`/tmp/sketchpad_iteration_root_audit48.log`. Of 5,139 unsupported calculations,
5,125 have unavailable numeric/function parent types; nine have unsupported
headers and five incompatible function units. Parent references overlap: 4,937
point to another code-48 calculation, 660 to code-94 point values, and 522 to
code-47 measurements. This makes upstream construction support the main next
step rather than broadly weakening calculation parsing. The iteration audit
also separates ready direct parents from unproven semantic support.

Checkpoint 48 scoped validation passes 882 checks; copying and custom tools stay excluded.

## Checkpoint 49: construction substitution and shared host values

Three shared roots are now implemented and independently Native-proved:
Point code101 construction application, coordinate-axis subtype38 constraints,
and general point/polygon code94 values. The fresh 1,599-file scan restores
4,321 slots (4,315 drawing + six prototype slots), reducing unsupported slots
from 50,075 to 45,754. All originals save byte-identically and scan failures
remain zero. Complete drawings increase from97 to99 and complete full documents
from96 to98. These counts are actual scan changes, not overlapping audit reach.

The broadest effect is in dependent calculations: code48 decreases by1,405.
Direct root-code reductions are865 code101,285 code15 and134 code94; iteration
bindings decrease94 and mapped roots33. The implementation adds one generic
stored mapping definition, reuses scoped DAG evaluation and preserves parent
iteration substitutions. Native Point seed replacement and omitted-boundary
freezing are proved; numeric seed roles and other output classes remain deferred.
Existing affine element/matrix sequences continue to handle geometric transforms.

Axis constraint creation, dragging and measurement share the Native200-pixel
centered parameter span; coordinate-unit graph equations/plots retain their
existing bases. Polygon values reuse live boundary projection/perimeter and
are exposed through the existing editor measurement command. Validation covers
source edits, undo/redo, deletion/reference remapping, invalid records and save/
reopen. The final scoped gates pass907 checks (743library,126editor,34numeric,
fourcoverage), excluding copying/custom-tool tests.

Next candidates are shared code75 spectrum/HSV presentation adapters and code90
live numeric-label bindings, followed by rich dynamic code73 captions/hotspots.
Their audits cover many source classes and files; use shared typed adapters
rather than operation-by-element enum products. Re-audit unsupported dependent
frontiers against the new baseline before interpreting old reach estimates.

Evidence: [mapped definitions](mapped_elements49_analysis.md),
[Native substitution](mapped_construction49_native_analysis.md),
[axis normalization](graph_axis_point49_native_analysis.md), and
[polygon point values](numeric_roots49_analysis.md).

## Checkpoint 50: live color properties and shared numeric presentation

Code75 now uses one `ColorizedDefinition` over Point, Segment, Ray, Line, Circle,
Polygon and Arc geometry. Spectrum, Grayscale, RGB and HSV share typed numeric
operands, native operand order and Clamp/Cycle/Reflect ranges. Geometry queries,
constraints, dependencies, deletion, evaluation contexts and saved styles reuse
the source paths. The editor creates live colors for multiple selected sources
as one undoable command; SVG and canvas geometry read effective live colors.
Spectrum interpolates nine Native measured palette knots, within the user's
stated display tolerance. Undefined color produces no displayed ink while the
source geometry remains available. White versus hidden Native behavior remains
unresolved; see [Native color evidence](color_wrappers50_native_analysis.md).

Code90 numeric labels now validate bare and literal-cached reference macros,
use the live source value, precision and quantity suffix, and preserve their
independent indexed font and cached bytes. PointDistance, PointRatio, coordinate
measurements and other typed numeric sources share the formatter. The existing
Attach Selected Value to Point command also accepts measured quantities.
Unknown markup, mathematical cached subtrees and nonnumeric source roles retain
unsupported status; [label evidence](point_numeric_labels50_analysis.md) records
18 independent Native controls. Scalar iteration depth now uses the common
numeric-unit classifier instead of a parameter/calculation variant list; live
ratios and point values are covered by [depth evidence](scalar_iteration_depth50_analysis.md).

The fresh complete corpus scan restores 4,394 actual slots relative to
checkpoint49: 4,176 drawing and218 prototype slots. Unsupported slots decrease
from45,754 to41,360 (40,512 drawing +848 prototype). Drawing code75 decreases
from2,879 to505 and code90 from2,708 to1,915; supported shared parents also unlock
iteration and other dependent constructions. These are measured restorations,
not the larger potential dependency reach from the earlier audit. All1,599
original documents still round-trip byte-exactly, zero fail,102 documents have
all kinds decoded and103 have all drawing kinds decoded. Final scoped gates
pass926 checks:759 library,129 editor,34 numeric import and4 coverage checks.
Logs: `/tmp/sketchpad_corpus_checkpoint50.log` and
`/tmp/sketchpad_checkpoint50_{lib_final,editor_final,numeric,coverage}.log`.
Copying and custom-tool implementation remain deferred.

## Checkpoint 51: referenced captions and carried numeric labels

Native73 now uses a bounded rich AST with ordinary one-based parent slots,
retaining original caches, UTF8 byte lengths, typography and opaque alignment
bytes. Current displayed text comes from live point/object names, numeric values
or action captions; stale cache text is not substituted for semantic evaluation.
The shared text layout supplies SVG/canvas output, picking, movement, font edits
and serialization. Live Caption from Selection creates these bindings in the
editor. Referenced action text dispatches the existing action handlers, including
hidden source buttons, while modified selection clicks select the caption.
Independent Native controls prove single-click HideShow, world-coordinate pan,
source edits and actual API writer files. Encoded R/G/L style components are
retained in the AST; the current common font layout does not promise their exact
color rendering. Flagged name/expression references and mathematical subtrees
remain opaque pending their shared role/formatting implementation.

PointNumericLabel is now a valid carried DAG leaf and generic IterationBinding
output. Simultaneous Point/Scalar substitutions, frozen omitted numeric hosts,
undefined values, final-only output, deletion/remapping and old specialized
numeric iterations share the established evaluator. No shape/output product
enum is added. Label positions/text/fonts reach rendering and picking through
one adapter. Nine Native raw/profile/API controls prove live values and source
positions; six focused regressions preserve legacy canonicalization and packet
precedence. Nonzero90 allocation padding is retained rather than interpreted as
text; a fresh Native44-byte rich payload proves the accepted extra allocation
word without modifying markup, length or cached source spelling.

Actual complete-corpus recovery relative to checkpoint50 is 3,776 drawing slots:
2,177 code73,925 code77,379 code90,292 code89 and3 code76. Prototype recovery is
unchanged. Unsupported total falls41,360→37,584 (36,736 drawing +848 prototype).
All1,599 originals round-trip exactly with zero scan failures. Complete full
documents increase102→340, and complete drawing documents103→347. Final scoped
checks pass945:776 library,131 editor,34 numeric import and4 coverage checks.
Evidence: [referenced AST](referenced_rich51_analysis.md),
[Native referenced text](referenced_rich51_native_analysis.md),
[iteration frontier](iteration_outputs51_analysis.md),
[Native numeric-label iteration](mapped_numeric_labels51_native_analysis.md).
Logs: `/tmp/sketchpad_corpus_checkpoint51_final.log` and
`/tmp/sketchpad_checkpoint51_{lib_final,editor_final,numeric,coverage}.log`.
Copy and custom-tool implementation remain deferred. Remaining highest drawing
families include blocked iteration outputs, numeric-expression parents, circle/
line intersections and flagged/rich anchored labels; refresh parent-ready
frontiers before choosing another root rather than targeting raw code counts.


## Checkpoint 52: measured shared prerequisite recovery

Fresh compiled coverage restores6,893 drawing slots relative to checkpoint51:
unsupported total37,584→30,691, with848 undecoded prototype slots unchanged.
All1,599 source files remain byte-exact, zero fail, complete documents340→355,
and complete drawing documents347→362. The measured changes include code48
calculations3,732→2,537 (−1,195), code77 bindings4,213→2,883 (−1,330), code89
value-depth roots1,338→850 (−488), code109 helpers560→18 (−542), code110
helpers638→20 (−618), and code15 constrained points1,420→911 (−509).
Log: `/tmp/sketchpad_corpus_checkpoint52_final.log`. Final scoped gates all exited0:790 library,131 editor,34 numeric import and
four coverage checks (959 total), excluding deferred copying/custom-tool cases.
Logs: `/tmp/sketchpad_checkpoint52_lib_final3.log`,
`/tmp/sketchpad_checkpoint52_editor_final.log`,
`/tmp/sketchpad_checkpoint52_numeric.log`, and
`/tmp/sketchpad_checkpoint52_coverage.log`. `git diff --check` is clean.

The initial dependency audit identified native point-on-object host identities
as the broadest available prerequisite. Parallel46 and Perpendicular47 reuse
existing shared straight parameter geometry; they do not add shape/operation
variants. Polygon constraints use a proved u16 edge index followed by six
preserved opaque bytes. Native raw-profile and public single/multiple-page
editing controls verify geometry, observer measurements and exact saved records.
Bisector44 remains deferred because its measured parameter magnitude differs
from the current unit-direction model; generic polygon subtype15 also retains
its separate unproved identity boundary.

A single CoordinatePointDefinition then restores109 fixedx/livey and110
livex/fixedy, using existing NumericExpr and coordinate-frame evaluation. Public
Native authoring controls verify negative quantities, nonzero fixed literals
and independent rectangular frame scales. Common point consumers unlock
calculations, mapped outputs and depth-driven iteration roots without a new
output-by-operation enum product. The source role remains Scalar; other units remain unsupported. Rotated frames
have no independent Native proof in these controls.

The next largest drawing families are bindings77 (2,883), calculations48
(2,537), loci35 (1,886), vector translations16 (1,840), point-anchored labels90
(1,458), numeric dilations33 (1,283), and rich captions73 (1,105). Raw totals
include source-blocked descendants. Recompute direct-ready and complete
ancestor frontiers against this new model before prioritizing another family.
Remaining reusable candidates include generic transform source/native subtype
identity, intersection and locus roots, flagged name/expression reference roles,
and broader typed iteration outputs. Preserve strict unsupported-unit and
unknown-markup boundaries until ordinary Native controls prove the corresponding
shared semantics. Copying, custom tools and exact raster matching remain deferred.

Evidence: [drawing prerequisite audit](geometry_roots52_analysis.md),
[Native host identities](path_host_subtypes52_native_analysis.md),
[polygon public writer](polygon_constraint52_api_native_analysis.md),
[coordinate public writer](coordinate_points52_api_native_analysis.md), and
[mapped/coordinate roots](mapped_iteration_roots52_analysis.md).


## Checkpoint 53: shared numeric ancestry and transformed-point consumers

Prioritized shared dependency gaps: ordinary locus traversal includes numeric
intermediates; inherited point89 vector packets reuse TransformDefinition; native
Bisector44 uses the independently measured parameter frame shared with13. No new
shape-by-transform enum product was introduced. Undefined geometry retains typed
measurement definitions, with strict parent-role validation.

Fresh compiled scan recovers466 drawing slots (30,691→30,225 total unsupported;
848 prototype slots unchanged). Major actual reductions: loci172, fixed dilation114,
vector translation71, iteration bindings48, inherited roots22, constrained points11,
calculations9. All1,599 originals remain byte-exact with0failures. Complete documents
355→365 and complete drawing documents362→374. Final scoped gates pass972 checks:
803 library,131 editor,34 numeric imports,4 coverage, excluding copying/custom tools.
Logs: `/tmp/sketchpad_corpus_checkpoint53_final.log` and
`/tmp/sketchpad_checkpoint53_{lib_final,editor,numeric,coverage}.log`.

The initial727 ready numeric-ancestry locus estimate was potential reach, not a
certain recovery count: packet/range profiles and remaining ancestry roles still
bound decoding. Choose the next shared prerequisite from refreshed parent-ready
frontiers and original payload evidence. Large remaining families include iteration
bindings, calculations, generic straight/polygon transform sources, loci and arc
intersections. Copying and custom tools remain outside the current implementation
priority, and display output need not be pixel-identical.

Evidence: [locus controls](locus_numeric53_native_analysis.md),
[point transform controls](inherited_point_transform53_native_analysis.md),
[Bisector coefficient](bisector53_native_analysis.md).


## Checkpoint 54: actual broad recovery from shared metadata and geometry

Refreshed exact guards ranked retained point-locus metadata above category
expansion: 636 ready loci versus 22 for generic driver/target categories. Native
proved point word32=0 as well as1 and arbitrary captured intermediate ancestry
order within the exact prefix/final-target/length/multiset contract. Shared decode
and edited-save validation preserve those records; canonical authoring is unchanged.

Ray/line vector, reflection and fixed-polar outputs reuse the generic transform
model and shared arc/straight packet parser. Primitive and derived straight sources,
chained matrices, live edits, parameter domains, zero-direction suppression and
restoration have original payload evidence and independent Native controls.
Desktop commands retain the selected straight extent through one append path.
GraphAxis remains closed without a positive Native proof; other unproved operation
and unit combinations remain closed. No product enums were introduced.

Fresh full coverage restores 1,118 drawing slots and two prototype slots from
shared decoding: total unsupported 30,225→29,105. Drawing reductions include636
loci,106 reflections,64 center arcs,41 calculations,41 vector translations,
37 iteration bindings and33 of each circle/line intersection branch. All1,599
originals remain byte-exact with zero failures, complete documents365→377 and
complete drawing documents374→386. The 2,087 potential metadata descendant union
was an audit estimate, not the measured recovery. Final scoped gates pass982:
811 library,133 editor,34 numeric imports and4 coverage, excluding copying/custom
tools. Logs: `/tmp/sketchpad_corpus_checkpoint54_final2.log` and
`/tmp/sketchpad_checkpoint54_{lib_final2,editor_final2,numeric_final2,coverage_final2}.log`.

Evidence: [remaining locus guards](locus_frontier54_analysis.md),
[retained metadata](locus_metadata54_native_analysis.md),
[shared straight sources](straight_transform_sources54_analysis.md),
[Native straight geometry](straight_transform54_native_analysis.md).
The next implementation should refresh the remaining independent roots; iteration
bindings and calculations still mostly wait on upstream definitions. Copying and
custom tools remain skipped; display need not be pixel-identical.


Checkpoint55 prioritizes a bounded terminal point construction over the larger
but less proved sampled-curve hint frontier. Only187 initial ready code88 roots
reach1,815 unsupported descendants; counts overlap and are not summed. Native
controls establish final carried source evaluation, live numeric depth including
zero, independence from final-only rendering, and no fallback when the terminal
source is undefined. The implementation reuses mapped scope evaluation. Shared
straight/arc intersections additionally recover actual original11/12 packets
without introducing shape-by-operation construction variants. The first rebuilt
full scan recovers2,560 drawing slots, including458 calculations, and preserves
all1,599 originals byte-for-byte. Final gates are recorded in corpus_coverage.md.
See [fresh shared prerequisite frontier](numeric_shared_roots55_analysis.md),
[terminal point Native controls](iteration_endpoint55_native_analysis.md), and
[analytic arc intersections](straight_arc55_analysis.md). Copying and custom
tool work remain deferred. Code98 curve hints and93 polygon boundary
intersections need their own Native branch proofs; already typed13/14 arc-parent
objects remain a separate known geometry gap.


Checkpoint56 verifies the broad code93 polygon boundary prerequisite and the
Scalar inherited89 dilation33 adapter. Native24 boundary controls and their
independently authored public API files all save byte-identically. The directed
query is one intrinsic construction; straight source modes and generic transform
definitions retain their identities. Scalar subtype inheritance reuses the
existing generic transform evaluator, with seven bounded Native controls and
strict import guards. Quantity/rotation broadening remains deferred.

Actual full recovery is1,779 drawing slots, including194 calculations, plus
seven incidental prototype slots; total unsupported26,545→24,759. All1,599
originals preserve their bytes, and the scoped core/editor/numeric/coverage
gates pass1,004 tests. Fresh audits should now rebuild the remaining root
frontier rather than reuse the pre56 ready cohorts. Curve-hint98 still needs
independent branch/hint proof; typed13/14 arc-parent geometry is a separate
evaluation gap. Copying and custom tools remain skipped.

Evidence: [root audit](numeric_shared_roots56_analysis.md),
[boundary and Native event evidence](ANALYSIS.md#polygon-boundary-intersections),
[Scalar inheritance](inherited_numeric_transform56_native_analysis.md).

Checkpoint57 implements the shared point-curve intersection prerequisite and
raises the calculation token resource bound while preserving schema and unit
guards. Native65 curve controls establish ranked crossings, operand order,
source-specific endpoints, proper crossings, domain compaction and undefined
cases. The intrinsic construction uses the same generic transformed geometry;
it does not add a shape-by-operation enum product. Native18 round-pair controls
also close the already typed13/14 Arc-parent evaluation gap, including linear
ThreePointArc limits. Six Native long-calculation controls independently verify
both retained original packets and public API creation.

The measured final scan restores1,344 drawing slots:239 code98 roots,304
calculations,220 numeric dilations and214 numeric coordinate points, with other
shared descendants making up the remainder. All1,599 originals remain
byte-identical; total unsupported24,759→23,415. The prototype namespace remains
unchanged. These are actual combined counts, not the larger potential ancestry
reach of the preimplementation audit. Broad remaining gaps include code77
bindings and their upstream definitions; fresh ready-root audits should precede
the next implementation. Copying and custom tools remain skipped; exact raster
identity is unnecessary.

Evidence: [shared roots](numeric_shared_roots57_analysis.md),
[point-curve packets and tests](curve_intersections57_analysis.md),
[Native curve geometry](curve_intersection57_native_analysis.md),
[round-pair implementation](round_intersections57_analysis.md),
[long calculations](calculation_tokens57_native_analysis.md).

Final checkpoint57 scoped gates pass1,019 tests (841 core,140 editor,34 numeric,
four coverage). New page-local intersections retain imported curve sources;
new plot authoring keeps its existing single-page restriction.


Checkpoint58 shares actual dependent-point evaluation between curve intersections,
free curve points and scalar-located curve points. It supports point-target
geometric and parameter loci without separate transformed-shape enums. The free
point carries a host and a normalized parameter; the scalar-driven point keeps
its live numeric source. Native60 controls establish source-specific clamping,
wrapping, undefined bounds and raw parameter units, including degree drivers,
coarse/dense off-grid positions and exact circle seams. Direct point-value
measurements, projection/drag, undo and retained page-local sources use that same
geometry. Swept targets, FunctionPlot free points, code123 curve literals and
arbitrary inverse curve measurements remain outside this proved scope.

The numeric-path flag4 alias hides only the output while keeping its geometry
and observers, and unchanged original bytes retain the alias. Flags5/6/7 remain
unsupported because their Native runs failed before opening a window. Nine-ref
Polygon sources with the retained class4/sub15 identity now produce matching
edge constraints, including repeated walks; other inherited counts are deferred.

Final measured recovery is2,337 drawing slots, including371 numeric path points,
368 constrained points,361 numeric coordinates,302 numeric dilations and118
calculations. Total unsupported23,415→21,078; prototypes remain839. All1,599
originals are byte-identical and scoped core/editor/numeric/coverage gates pass
1,035 tests. These combined observed counts replace ancestry reach estimates.
The next implementation should re-audit the remaining roots against this build;
code77 bindings, code102 definitions and their numeric parents remain broad gaps.
Copying and custom tools remain skipped, and exact raster equality is unnecessary.

Evidence: [curve prerequisite audit](path_curve_points58_analysis.md),
[Native curve rules](path_curve_points58_native_analysis.md),
[Native identities](path_identity58_native_analysis.md),
[Polygon implementation](polygon_identity58_analysis.md).


Public API generation coversall60 controls with strict save/reopen. The30 native15
files match the retained Native goldens entirely. Numeric95 uses the genuine
fresh writer template: only its two opaque header bytes differ from the older
placeholder-based controls. Four representative fresh API files independently
opened and saved byte-identically in Native, proving geometric and Scalar
parameter drivers, degree-driver radian inputs and undefined outside positions.
Their separate observer/paint checks pass (maximum .681px). Together with the
60 retained curve controls, two flag controls andfour polygon controls, the
checkpoint has70 independently exact Native open/save files.


Checkpoint59 shares common straight-host evaluation and extent across code95/123, general circle-arc endpoints, and intrinsic analytic SectorBoundary/Point68. The model keeps live elements and transform sequences rather than adding operation-specific path point kinds. Native60 retained controls establish both defined and undefined geometry; API14 arcs and36 ordinary straight files match those independently saved goldens entirely. All eight fresh Sector API files independently cross-open/save byte-exact and pass geometry checks; that proof is separately recorded because ordinary measurement templates differ.

The finalized compiled scan restores2,267 drawing slots and four prototype slots. Total unsupported21,078→18,807; complete documents445→465, drawing-complete455→475. All1,599 originals remain byte-identical, zero failures. Scoped gates pass1,052 tests (869 core,145 editor,34 numeric,four coverage). Copying/custom tools remain skipped and pixel identity is unnecessary. These measured counts replace the larger overlapping ready-root ancestry predictions.

Main actual drawing restorations are code33: 250, code82: 199, code79: 199, code15: 184, code101: 170, code95: 166, code27: 143, code16: 137, code24: 122, code48: 93. Remaining code77 bindings (2,549), code102 definitions (669), loci35 (823), numeric layouts and their shared source prerequisites need a fresh audit against this build. Generic sector transforms/aliases and Radius46 have no new proof. Evidence: [arc](arc_sources59_native_analysis.md), [sector](sector_boundary59_analysis.md), [straight](value_straight_paths59_analysis.md).


Checkpoint60 adds intrinsic CircleInterior(source) and reuses PointOnCurve for FunctionPlot subtype58 rather than multiplying operation-specific enums. Actual corpus recovery is516 drawing slots plus five prototype slots; unsupported18,807→18,286, drawing-complete475→484, all-kind complete465→474. All1,599 original files remain byte-identical, zero failures. Gates pass1,062 tests (877core,147editor,34numeric,fourcoverage);21 public API files exactly match independent Native saves. Copying/custom tools stay deferred and pixel equality is unnecessary.

Circle identity, hidden source, independent styles/order and zero/edit recovery are bounded by eight modern Native controls. Two short legacy interiors remain opaque. Function stored fractions extrapolate, canonical-x measurements remain live, and interactive projection is bounded, with13 static plus two drag controls. Code102 sampled DAG definitions and remaining35 driver categories still offer broad potential; their Native/control work is underway separately and not counted as supported. Evidence: [Function](function_points60_native_analysis.md), [Circle](circle_interior60_native_analysis.md), [root audit](numeric_shared_roots60_analysis.md).


Checkpoint61 finishes the proved shared code102 sampled dependency evaluator and expands code35 drivers to PointOnCurve, Polygon and Sector. Literal and Scalar-driven sampled children share source geometry with separately evidenced extent policies; no shape × transform enum product or custom-tool invocation is introduced. The admitted89 prefix value is limited to its two exactly proved profiles.

Measured restoration is1,386 drawing objects, led by102:406,35:249,16:112,98:73,8:72,15:71. Unsupported18,286→16,900; prototypes remain830. All1,599 originals remain whole-file exact, complete drawings484→529, and1,077 scoped tests pass. The36 retained Native controls andfour fresh public API controls are independently whole-file exact, with geometry/observer checks. Fresh numeric API measurement defaults differ from old Arc donor formats; their separate Native proof establishes the actual writer. Copying/custom tools stay skipped and raster equality is unnecessary.

Remaining gaps must be ranked from the rebuilt61 graph. Largest drawing counts:77=2,503,90=1,416,48=1,239,73=1,074,89=753,35=573;102 falls669→263. These are unsupported totals, not independently ready roots or recovery forecasts. Inspect live parents and actual record/Native profiles before broadening admission. Unbounded sampled paths, new closure flags, quantity seeds and arbitrary inverse measurements remain unproved.

See [post61 iteration frontier](iteration_frontier61_analysis.md) for the read-only grouping of remaining decoded-parent roots. Descendant cohort counts overlap and are not recovery forecasts.


Checkpoint62 first implements the authorized four-crate layering, then closes the shared FunctionPlot/SampledMapped locus-driver gap within the new engine/IR boundaries. Actual recovery99 drawing slots includes57 loci and16 constrained points; all1,599 originals remain exact, and complete drawings529→534. Ten Native controls establish literal/numeric extent distinctions and open/closed Circle metadata; nine fully API-authored files match independent saves. Workspace gates pass1,143 executions, with seven new driver regressions. Copying/custom tools remain skipped.

Remaining broad target frontier is generic straight/category support, not a demonstrated transformed-polygon recovery cohort: the read-only pre-repair audit measures88 roots/175 reach, or93/185 with generic Segment driver hosts, and finds no captured transformed Polygon in that cohort. These predictions must be rebuilt after the99-slot recovery. Paired HTML corroborates predecessor straight geometry but omits the sweep targets; independently prove the Native sweep/frozen-colour contracts before extending classifiers. See [driver implementation](locus_point_sources62_analysis.md), [Native proof](locus_point_sources62_native_analysis.md), and [generic-target audit](locus_generic_targets62_analysis.md).


Checkpoint63 closes the bounded generic straight locus target/host gap through shared transforms and scoped evaluation. Measured recovery is118 drawing objects, led by94 loci; unsupported16,801→16,683, complete drawings534→542, all1,599 originals exact. This observed recovery replaces the overlapping93-root/185-reach prediction. No transformed-Polygon cohort, axes or tool carrier support was inferred.

Independent Native proof covers30 exact files, including12 public API writer controls and two174-witness geometry/color checks. Direct Colorized targets use per-sample color while Transform(Colorized) uses locus own color. Stroke codec selection uses resolved target context; rendering/picking preserve independent straight families. Workspace1,155 executions and the headless build pass. Copying/custom tools stay skipped; pixel identity remains unnecessary.

The final rebuilt locus frontier retains422 unknown loci and213 decoded-parent candidates,185 containing captured numeric operands. Its simulated576-slot reach is potential rather than current support. Remaining shared contracts include captured ancestry/range profiles, omitted or extra captured references, unbounded Ray/Line driver intervals, and iteration-rich parents. Rank these using current decoded dependencies and independently prove their source/range/frozen-scope rules before extending admission. See [checkpoint implementation](locus_generic_straights63_implementation.md), [Native rules](locus_generic_straights63_native_analysis.md), and the next read-only shared frontier review.


## Checkpoint64 shared rich reference roles and automatic names

The shared engine now evaluates supported73 Value/A/B/E references and90 point-anchored text/function/numeric contents with live/frozen source semantics, strict dependency guards, font editing, deletion/undo and save/reopen. Anonymous Calculation/Ratio/Length Name requests use verified active-closure/object/operand order and0967; assigned names survive hiding and dirty saves persist07d5 rather than recomputing names from remaining visibility.

Actual scan recovers1,378 drawing slots (379 code73,999 code90); unsupported16,683→15,305, drawing-complete542→573, all1,599 originals whole-byte exact. Workspace1,172 test executions and headless build pass; three initial entirely API-authored Native controls are final-library exact, and eight Name writer controls pass strict physical-label checks plus independent Native exact saves and painted witnesses. Rich math/font layout, additional source roles and nested annotation iterations remain bounded gaps. Copy/custom tools stay deferred; this does not satisfy the full release command gate. See [implementation](referenced_shared64_implementation.md).

## Checkpoint65 shared filled-disk iteration

CircleInterior joins the common mapped DAG evaluator and filled binding presentation. Actual recovery is192 drawing slots, including35 mapped definitions and142 bindings; unsupported15,305→15,113, drawing-complete573→584. All1,599 originals remain whole-byte exact, and1,176 workspace test executions pass. Seven independent Native controls establish live/frozen geometry, final-only and undefined/recovered members. Nine fresh public API files pass strict engine preflight and whole-file exact Native saves, with8,156 independent grid witnesses; half-opacity controls establish three coincident frozen samples composite separately. See [implementation](circle_interior_iteration65_implementation.md).

Nested scope evidence now distinguishes the inner descriptor from its display binder: carried descriptors can inherit outer substitutions while omitted descriptors retain their entry environment. Partial factor/image capture remains unproved. Eight mapped rich-label Native controls independently confirm live and frozen contents; production admission and integrated regression tests remain the next shared extension. The full release goal remains active; copying and custom tools stay deferred.

## Checkpoint66 shared anchored text and nested terminal scopes

Value-role anchored rich text and complete-capture nested point terminals now share construction validation, entry environment selection, live/frozen evaluation and editor presentation. Actual recovery is338 drawing slots; unsupported15,113→14,775, drawing-complete584→604, all1,599 originals exact. Workspace1,190 executions and headless build pass. Six fresh public rich writers save exact in Native; eight fresh public nested writers pass strict preflight, Native exact saves and independent geometry checks. See [implementation](shared_scopes66_implementation.md).

The next shared gap is evaluation phase for partially captured inner ancestors, rather than another shape-specific constructor. Independent precision readouts prove1.21000/1.58400/2.02800 terminal ratios and phase-dependent source readouts; they do not establish a universal lag formula. General nested collection outputs and captured Locus prerequisites remain substantial frontiers. Rank the freshly rebuilt dependency frontier separately from measured recovery. Copying/custom tools stay deferred and the full release goal remains active.

The final independent [frontier audit](iteration_frontier66_analysis.md) is terminal0:263 decoded-parent89/102 candidates reach1,465 different unknown objects. Sole collection roles include52 Locus roots→302 and17 nested descriptors→225. Original complete groups distinguish collection outputs from code88 terminal aliases; neither reach count is promised recovery. This supports a shared scoped collection evaluator as the next broad extension.

Checkpoint67 admits straight-output Locus collections, recovering38 drawing objects. Checkpoint68 extends the same scoped collection evaluator to sampled code102 point curves as outer code89/76 binding sources. It recovers another352 drawing objects, including76 code77 bindings and265 downstream code101 records; all1,599 original documents remain byte exact. Native visibility controls isolate the actual green binder geometry and a public move verifies its live scope. Workspace1,206 executions pass. The remaining unsupported totals (code77=2,055, code48=1,222, code0=1,194) are not independent feature estimates: inspect ready roots and parent profiles before choosing the next broad prerequisite. Copying/custom tools remain deferred. See [sampled implementation](sampled_collection68_implementation.md).

The [post-68 read-only mapped frontier](iteration_frontier68_analysis.md) finds no directly ready code77 binding among the2,055 remaining. Ready upstream89 roots include27 FunctionPlot-only closures (178 distinct unsupported descendants) and49 Locus-only closures (266), with overlapping reach. These are candidates for source/capture proof, not supported features or recovery forecasts.

Checkpoint69 closes the FunctionPlot prerequisite through shared scoped input resolution. Actual restoration is159 object slots (27 code89 roots and95 code77 binding descendants), with all1,599 originals exact and1,211 workspace executions passing. The Native triangle example exposed and corrected whole-plot versus input-level freezing; see [implementation](mapped_function_plot69_implementation.md). The remaining Locus diagnostic identifies41 ordinary-path point-trace roots/229 overlapping reach as the next common collection candidate, pending its own Native proof; see [frontier](locus_scope69_frontier.md).

Checkpoint70 implements the point-output Locus collection prerequisite. Final measured restoration is261 object slots, including50 code89 mapped roots and98 code77 bindings; all1,599 originals remain exact, and1,216 workspace executions pass. Native visibility controls distinguish three bound traces from one directly visible source trace after a parameter edit. Remaining collection profiles include nested Locus paths and circle/arc outputs. Refresh the ready-root audit before choosing the next broad shared dependency rather than treating the raw code48/77 counts as independent opportunities. See [implementation](scoped_point_locus70_implementation.md).

Checkpoint71 admits a Native π suffix and live numeric subscript as two typed referenced-text AST operators. The measured gain is171 object slots, with all1,599 originals exact and1,224 workspace executions passing. The two wire signatures had342 overlapping descendant reach before implementation; the actual recovery is smaller because other source/structure guards remain. Code77 still has no independently ready roots. See [implementation](referenced_token71_implementation.md), [Native evidence](referenced_token71_native_analysis.md) and the [post70 frontier](frontier71_analysis.md).

The latest referenced-caption pass closes the bounded markup and source-semantic
frontier without weakening source validation. Observed symbol-font codes, signed
numeric terms, five geometry decorations, centered vertical groups, outer frame
metrics, and source-specific name/definition/expression/value roles restore 259
slots and reduce direct unsupported code-73 captions 180→14. Complete documents
rise 1,170→1,228 and complete drawings 1,199→1,258, with all 1,599 originals
exact. A strict whole-markup GBK fallback covers the legacy Chinese literals.
Every remaining caption parses and has a recognized role; progress now depends
on implementing its unsupported parent object rather than cached caption text.

The following parameter-locus pass reuses the same scoped evaluator for native
code-97 subtype-43 outputs. Seven segment families, 13 filled-region families,
and 47 nested point-curve collections all execute through their runtime samplers;
calculation-driven and unsupported-parent profiles remain guarded. The measured
67-slot recovery lowers unsupported slots 3,471→3,404, raises complete documents
1,228→1,239 and complete drawings 1,258→1,269, and preserves all 1,599 originals
byte-for-byte. The next broad target should again be selected from decoded-parent
roots rather than raw downstream code-77 or code-48 totals.
