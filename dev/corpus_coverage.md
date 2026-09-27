# Corpus decoding coverage

Checkpoint 48 snapshot on 2026-09-27, generated with `gsp_probe coverage ../gsp-rs`.

Documents: 1599 total, 96 with all object kinds decoded, 1503 with unsupported objects, 0 failed.

Byte-preserving round trips: 1599/1599; object slots: 285703.

Partial views: 1512/1599 (includes multi-page structural edit restrictions).

Drawing objects: 280347 slots; 97 documents with all drawing kinds decoded.

Embedded tool prototypes: 103 definitions in 65 documents; 5356 slots, 4284 decoded, 1072 undecoded.

Isolated tool views: 33 complete, 70 partial, 0 failed, with separate object and font namespaces.

Unsupported object slots including tool prototypes: 50075 (49003 drawing slots and 1072 prototype slots). Object decoding does not establish editing, interaction, or command parity. The release requirements remain in `COMMAND_PARITY.md`.

Embedded tool prototypes are decoded through isolated documents. Declared point placeholders receive neutral coordinates; declared path, finite-segment, and generic-numeric placeholders retain typed abstract inputs. Other unknown prototype constructions remain unsupported. The drawing and prototype histograms retain separate namespaces. Complete isolated decoding does not establish invocation or authoring support for every tool. This snapshot is not directly comparable to earlier drawing histograms that mixed tool prototypes with scene objects, or to snapshots that counted every prototype slot as undecoded.

The counts include missing variants and failures caused by unsupported parents.
Inspect complete groups and parent payloads before selecting a target.

| Native code | Unsupported objects | Documents | First example |
| --- | ---: | ---: | --- |
| 77 | 5809 | 622 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/【教程】用迭代构造平面直角坐标系.gsp` |
| 48 | 5139 | 430 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 73 | 3298 | 1350 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/09年潍坊17题答案（蚂蚁制作）.gsp` |
| 75 | 2879 | 445 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/10福建宁德26题2(蚂蚁).gsp` |
| 90 | 2708 | 399 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 16 | 2674 | 315 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 89 | 1956 | 606 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/【教程】用迭代构造平面直角坐标系.gsp` |
| 35 | 1901 | 401 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/09年潍坊17题答案（蚂蚁制作）.gsp` |
| 15 | 1711 | 352 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 101 | 1441 | 80 | `../gsp-rs/tests/Samples/个人专栏/况永胜作品/正方体的展开（3D效果）.gsp` |

Run the command again for the current full report and all examples.

## Audited prerequisites

The current work addresses native numeric presentation records and optional
labels, broader scalar parameter/calculation layouts, independent point-value
projections, polygon boundary points, numeric coordinate points, and
parameter-driven point traces. See the corresponding analysis files in `dev`.
Native cross-open checks and save/reopen tests accompany each supported family.
The snapshot includes the verified numeric trace and multi-seed iteration
families, transformed-point labels, vector transform chains, and native URL
button layouts. It is a measurement of that build, not a final release claim.

Typed length and degree parameters, optional empty calculation attributes, polygon boundary value points, and further mapped point/segment iterations now have native proofs. Angle parameter curves now retain native radian ranges and expose degrees in the range editor. Fixed rotation/dilation/reflection chains, literal polygon path points, and transformed polygon boundaries also have focused native evidence. Scientific length/degree exponent-5 literals, π radian literals, optional parameter labels and page-generated names, and fixed ray/line rotations and dilations now also have edit/copy native proofs. Fixed ray/line transform chains preserve source strokes and undefined zero-scale ancestors. Negative code-95/123 positions on those ray hosts remain undefined while full lines extrapolate. Labeled code-123 path points retain typed label/font suffixes through edits and copying. Legacy free-point/polygon clipboard conversion preserves metadata and has structural undo. The next audited gaps include further calculation layouts, additional iteration definitions and their code-77 bindings, broader multi-page numeric record editing, and further vector-transform variants. Saved scalar parameter edits and supported table rows now have focused multi-page native proofs; this does not remove the document-wide partial-view flag. Original `cood-trace.gsp`,
`poly_point_with_val.gsp`, refraction and Ningde examples, and
`迭代方法2(蚂蚁).gsp` provide concrete native payloads and paired HTML definitions.
Retain native payload validation and regression/cross-open gates when extending
these families.

The next snapshot (checkpoint 12) retains the same drawing/tool scope and reduces
unsupported slots from 112506 to 108817. It includes source-free radian arithmetic,
centimeter calculation literals, numeric-ratio ray/line dilation, mixed fixed and
numeric straight-transform chains, and the proved arc plot hosts. The full
all-targets regression gate at this checkpoint passes 782 tests. Later constrained
arc semantics, trailing decimal parsing, and unified transformation-model work
are subsequent changes and require another gate.

Checkpoint 13 retains that same scope and reduces unsupported slots to 108783.
It includes the verified constrained linear arc semantics and trailing decimal
calculation literals. All 1599 documents remain byte-identical on round-trip.
The unified transform refactor does not itself count as additional decoding
coverage. Checkpoint 14 independently confirms the same counts after the shared consumer
migration. The combined all-targets gate passes 805 tests, including the
independently verified straight-geometry corrections and slope precision fixes.
Generic transform storage and the remaining command requirements are pending.

Checkpoint 16 verifies the generic transform storage cutover: all 1599 files
still round-trip byte-identically, and unsupported slots remain 108783. The
combined gate now passes 806 tests. Sixty product Kind variants have been
removed in favor of one generic transform variant; compatibility constructor
names remain. This representation change does not expand decoding coverage or
complete the pending command requirements.

Checkpoint 18 retains all 1599 byte-identical round trips and the same 280347
drawing slots and 285703 total slots. Drawing unsupported kinds remain at
103427. Coverage now uses the implemented isolated tool decoder rather than
classifying every prototype as unknown: 1822 of 5356 prototype slots decode,
leaving 3534 unknown and 106961 unknown slots overall. Eight isolated tool
definitions have complete views and 95 have partial views; none fails extraction.
Seven coverage regressions distinguish drawing preservation, unknown prototype
kinds, extraction failures, and known kinds with unsupported metadata.
The drawing partial-view count is 1522 after the proved labeled code-12
prototype support. Native Square and simple-tool invocation and creation controls are documented separately
in `tool_runtime_analysis.md` and `tool_creation_analysis.md`; these bounded proofs
do not establish corpus-wide command parity.
The settled combined all-targets gate passes 825 tests, including both saved
desktop invocation routes and the Native-verified creation writer.

Checkpoint 20 recognizes declared generic numeric and finite-segment tool inputs,
including their live dependent calculations and host-based midpoints. Prototype
known slots increased by 947; drawing known slots increased by 96. Every corpus
file still preserves its original bytes. Numeric tool authoring, caption font and
placement fidelity, other given categories, and the command gaps remain work in
progress. These counts do not prove that every decoded prototype executes.


Checkpoint 21 retains the checkpoint-20 counts and all 1599 exact round trips
following the ordered matrix evaluator and indexed scalar-table decoder work.
The new indexed table fixtures are separate native controls outside this corpus;
their support does not change this corpus histogram. Command behavior remains
subject to the separate native-editing proofs. Log:
`/tmp/sketchpad_corpus_checkpoint21.log` (exit 0).


Checkpoint 22 retains all checkpoint-21 counts and 1599/1599 exact round trips
after indexed-table font and UTF-8 negative-row support. Ordinary creation writer
fixtures are independent Native controls outside this corpus. Log:
`/tmp/sketchpad_corpus_checkpoint22.log` (exit 0). Copy and custom-tool implementation
remain deferred.


Checkpoint 23 retains the checkpoint-22 counts and all 1599 exact round trips
after generated t[1] table support. Legacy fixed-two-decimal table behavior is
still covered, while the independent counter-1 Native control and actual writer
are now supported. Log: `/tmp/sketchpad_corpus_checkpoint23.log` (exit 0).


Checkpoint 24 retains the checkpoint-23 counts and all 1599 exact round trips
after named table decoding preserves observed drawing font references. New table
creation now materializes other unnamed page parameters for default and plain-name
sources too, and resolves its default font against the drawing namespace. Those
writer controls are independent of this corpus. Log:
`/tmp/sketchpad_corpus_checkpoint24.log` (exit 0).


Checkpoint 25 retains the checkpoint-24 histogram and 1599 exact round trips
after integer/UTF-8 negative named table support and clean imported table
creation. Log: `/tmp/sketchpad_corpus_checkpoint25.log` (exit 0). Priority now
follows broad feature coverage and construction dependencies; matching original
rendering pixel-for-pixel is not required.


Checkpoint 26 broadens marked-distance translation to accept existing transformed
segments, circles and polygons through the shared transform source predicates.
This recovers 3171 drawing slots, including 3150 native-code-24 objects, while
retaining all 1599 exact byte round trips. The focused regression covers two
matrix steps, live distance changes, downstream length/area, save and undo/redo.

Checkpoint 27 accepts validated optional coordinate-measurement captions,
observed numeric text-cache fields and display modes 0/1/3, plus the observed
empty code-94 point-value attribute. Coordinate and point-value mathematics and
host validation remain unchanged. This recovers another 9233 drawing slots and
530 prototype slots through shared decoding, including 5567 drawing calculations.
Unsupported drawing calculations fall from 20270 to 14703. All 1599 original
files still round-trip byte-identically; document-wide partial views remain
1522 and fully decoded drawing documents remain 86. These figures do not imply
that the remaining unsupported or undefined graphs can be edited safely.

The three retained corpus regressions check native captions/formats, numeric
values and downstream calculations. A supported isolated graph combines the
original native caption with display modes 1/3 and nonzero cache bytes; its
coordinate and calculation values follow dragging and survive strong round-trip
validation and undo/redo. These are local file and semantic regressions, not
new Native application writer proofs. See
[broad prerequisite audit](broad_measurement_dependency_analysis.md).


Final scoped regression gates pass 602 library tests (110 filtered) and 106
editor tests (21 filtered), using the deferred copying/custom-tool name filters.
The additional isolated point-value attribute regression checks live values,
strong save/reparse validation, exact retained bytes and undo/redo. Evidence:
`/tmp/sketchpad_broad_measure_final_gate3.log`,
`/tmp/sketchpad_broad_measure_editor_gate.log`,
`/tmp/sketchpad_point_value_attribute_gate.log`, and
`/tmp/sketchpad_corpus_checkpoint27.log` (all exit 0).


Checkpoint 28 generalizes fixed-polar translation sources to transformed
segments/circles/polygons, recovering 952 drawing slots and 45 prototype slots,
including 358 calculations. Checkpoint 29 normalizes native radian transform
angles while validating their declared trigonometric caches and retaining both
observed degree flags. It also fixes the polar pixel-distance cache: native
records contain total displacement, while the shared model stores magnitude
and conversion scale. Import and generation now agree on these meanings.

Relative to checkpoint 27 these changes recover 5993 drawing slots and 90
prototype slots, including 1423 drawing calculations. Original round trips are
1599/1599 exact; unsupported slots total 86257. Drawing calculations fall to
13280, and fully decoded drawing documents rise to 87. Partial views remain
1521, so this is still partial implementation coverage.

Five retained original corpus files and paired HTML exports establish radian
rotation/translation and fractional-distance geometry. An actual API-generated
fractional-distance segment/circle/polygon chain file opened and saved
byte-identically in Sketchpad 5 (helper exit 0 and independent comparison).
The native result is retained as a reconstruction regression golden. The scoped
library gate passes 606 tests (110 filtered); see
[polar transform evidence](polar_translation_units_analysis.md).


Checkpoint 30 recognizes marked distance sources by their numeric Length unit,
including parameters and calculations, and evaluates their centimeter values
through the existing numeric evaluator. The finite/nonnegative pixel policy
remains unchanged. This recovers 1219 drawing slots (including 629 code24
objects and 32 calculations); prototype counts and partial-view totals are
unchanged. All 1599 original files retain byte-identical round trips. Unsupported
slots total 85038, including 83715 drawing objects and 1323 prototype objects.

The editor now accepts a Length Parameter through Mark Distance. Regressions
cover point/segment/circle/polygon polar translation, rectangular and mixed
point translation, typed calculations, live parameter updates, existing
negative/undefined behavior, save/reparse and undo/redo. The actual writer file
opened and saved byte-identically in Sketchpad 5, independently compared against
its untouched before-image. The scoped gates pass 608 library tests and 107
editor tests. See [typed distance evidence](typed_length_marked_transforms_analysis.md).


The additional numeric-import gate passes32 tests with10 copying/custom-tool
name-filtered cases deferred. Copy plural/participle aliases are filtered too;
the unfiltered plural-copy sourcefree-radian test fails at its pasted-point
lookup and is outside the current authorized implementation priorities. This
is not an all-targets completion claim.

## Checkpoint 31: source-free quantities

The shared calculator now accepts source-free centimeter lengths, area products,
inverse-angle functions, and scalar functions of radian literals. Expression edits
retain the numeric unit, record presentation and undo/redo behavior. Length results
feed the existing marked-distance transform model without new shape/transform enum
variants. Undefined expressions retain their unit and suppress dependent geometry.

The complete corpus retains 1599/1599 exact round trips and reduces unsupported
objects from 85038 to 84900: 138 drawing objects recovered, prototype counts
unchanged. Original Length and inverse-angle controls are retained; the corpus
contains no source-free Area control. An authored seven-object file including
Length, Area, scalar/radian functions, trailing decimal syntax and a marked-distance
translation was opened and saved by the unchanged native helper (exit 0), with an
independent byte comparison passing. This is writer evidence, not a claim of
matching every native display detail. See `source_free_quantity_calculations_analysis.md`.

## Checkpoint 32: many-input calculations

Source references use the continuous native `0x6000 + index` encoding. Corpus
profiles contain up to 51 references and tokens through `0x6032`; the old 16-input
limit rejected original formulas and their dependent constructions. The shared
parser and authoring path now accept up to 256 inputs/tokens, preserving source
ordering, aliases, units and exact record bytes. Input indexes must be within the
reference list. Source-free expressions reject source tokens across the new range.

The original 20-input `100位e.gsp` formula now yields 2.718281828. An authored
51-input sum with a length calculation and marked-distance image, edited at its
51st input, opened and saved byte-identically in native Sketchpad (13314 bytes).
The corpus recovers 156 drawing slots, including 80 calculations; prototype counts
remain unchanged, and all 1599 original files retain exact round trips. Large
calculation dependency-root counts overlap other missing families and should not
be confused with actual recovered-slot counts. See `many_input_calculations_analysis.md`.

## Checkpoint 33: scalar marked distances

Native marked-distance translations accept unitless values as centimeter
magnitudes, including negative values that reverse direction. The shared numeric
expression, transform evaluator, decoder, editor and iteration mapping now handle
these operands without shape/transform enum variants. Length arithmetic retains
its existing dimensional checks. Isolated native positive/negative Length controls also establish signed movement;
true undefined and nonfinite operands still suppress images.

Four native-open/save controls, an original scalar-calculation cube control and
an actual API writer containing polar point/segment/circle/polygon, Cartesian and
mixed translations provide evidence. The complete corpus recovers 6610 drawing
slots (760 calculations) and 108 prototype slots while preserving all 1599
original files exactly. No custom-tool authoring or copying support is added.
See `scalar_marked_distances_analysis.md`.


Checkpoint34 adds center arcs and their shared measurements/point paths. It
recovers4941 drawing objects, including361 calculations, and26 prototype slots.
The89 fully decoded documents and1599 exact source round trips are from the
completed final scan. Defined and degenerate actual API writer outputs both
open/save exactly in Native. See [center arc evidence](center_arc_analysis.md).
Scoped gates pass613 library tests,109 editor tests,34 numeric-import tests
and4 coverage tests, excluding the deferred copying/custom-tool tests.


Checkpoint 35 treats the coordinate measurement's final display word as
preserved presentation metadata, matching the existing ordinary measurement
reader. Native x/y controls and live API writer outputs prove the values and
exact saves. It recovers 2672 drawing objects, including 1232 calculations, with
70387 unsupported slots remaining and all 1599 exact original round trips. The
prototype and fully decoded document counts are unchanged. Unsupported groups'
raw numeric metadata is now retained during edited serialization as well.
See [coordinate presentation evidence](coordinate_presentation_tail_analysis.md).

Scoped checkpoint 35 regression gates pass 615 library, 109 editor,
34 numeric-import and 4 coverage tests. The complete final scan is recorded in
`/tmp/sketchpad_corpus_checkpoint35_final.log`.


Checkpoint 36 implements Native's unitless magnitude results for addition and
subtraction of unlike quantities. It recovers 5373 drawing objects, including
4351 calculations, while all 1599 source files preserve their bytes. Native
calculator-created controls cover variable, literal and compound quantities;
both actual 18-object API writer outputs save exactly in Native. The final
scan is `/tmp/sketchpad_corpus_checkpoint36_final.log`. Scoped gates pass 619
library, 109 editor, 34 numeric-import and 4 coverage tests. See
[mixed quantity arithmetic](mixed_quantity_arithmetic_analysis.md), including
its correction to the earlier cropped subtraction capture.


Checkpoint 37 supports typed quantity exponents and pixel calculation literals
through the shared AST. It recovers 398 drawing objects, including 392
calculations, with all 1599 exact source round trips and unchanged prototype
counts. Native creation controls establish units and pixel conversion; the
actual original/edited 17-object API writer outputs open and save byte-identically
in Native. The final scan is `/tmp/sketchpad_corpus_checkpoint37_final.log`.
See [quantity exponent evidence](quantity_exponents_analysis.md).
Scoped gates pass 623 library, 109 editor, 34 numeric-import and 4 coverage tests.


Checkpoint 38 adds referenced function calls to the shared calculator, including
constant calls, nested calls and derivative functions. Typed Degree arguments
follow the function's document angle mode. Numeric iteration labels use the
same callback and retain their function dependencies. It recovers 139 drawing
objects, including 75 calculations, with all 1599 exact source round trips.
The actual original/edited 28-object API writer outputs open and save exactly in
Native. The final scan is `/tmp/sketchpad_corpus_checkpoint38_final.log`.
See [function call evidence](function_applications_analysis.md).
Scoped gates pass 628 library, 109 editor, 34 numeric-import and four coverage tests.


Checkpoint 39 shares the calculator's built-in registry with primitive function
definitions: inverse trig, sign, rounding, truncation and π/e constants now
import, evaluate, plot and save. Signed caption positions are accepted without
the former artificial 4096 bound. It recovers 271 drawing objects, including
119 calculations, with all 1599 exact source round trips and unchanged
prototype counts. Native-created six-function controls and actual original /
edited 38-object API writer outputs establish native token semantics and exact
saves. The final scan is `/tmp/sketchpad_corpus_checkpoint39_final.log`.
See [primitive function evidence](function_primitives_analysis.md).

Scoped gates pass 631 library, 109 editor, 34 numeric-import and four coverage tests.


Checkpoint 40 establishes the referenced-expression parser and Native record
writer controls, without claiming new document coverage. Checkpoint 41 connects
those references to the real document DAG, authoring/editing, plotting,
calculator calls and numeric iteration. It restores 919 drawing objects,
including 229 function definitions and 439 calculations, plus three prototype
objects through the shared reader. All 1599 exact source round trips remain;
one additional drawing is fully decoded. Original/edited actual 38-object API
writer outputs save byte-identically in Native. The final scan is
`/tmp/sketchpad_corpus_checkpoint41_final.log`. See
[referenced function evidence](function_references_analysis.md).
Scoped gates pass 642 library, 112 editor, 34 numeric-import and four coverage tests.


Checkpoint 42 supports numeric-sized coordinate axes through shared numeric and
segment interfaces, and corrects literal Cartesian/Polar point dependencies:
07d2 stores a count followed by the frame ID, rather than two source IDs.
The shared reader restores 3841 drawing objects (282 calculations) and one
prototype object, reducing unsupported slots to 59442. All 1599 original files
remain byte-identical with zero import failures. Nine actual Document API writer
states save exactly in Native, covering live edits, Scalar/Degree/Area/Compound
units, negative/zero units and restoration. Meaningful regressions also cover
labels, removal of unrelated objects, malformed input and undo/redo.
See [coordinate units and point dependencies](coordinate_units_analysis.md).
The final scan is `/tmp/sketchpad_corpus_checkpoint42_final.log`.
Scoped gates pass 650 library, 115 editor, 34 numeric-import and four coverage
tests (803 total). Copying and custom tools remain deferred; these counts do not
establish complete command parity or unrestricted editing of partial documents.


Checkpoint43 extends simultaneous iteration through the shared dependency graph:
derived point seeds and Scalar seeds, including mixed mappings, substitute their
values before the next branch is evaluated. Generic point/segment transforms and
supported coordinate dependencies reuse ordinary geometry evaluation. The scan
restores269 drawing slots:12 fixed definitions,98 live-depth definitions and159
bindings, reducing unsupported slots to59173. Prototype counts are unchanged.
All1599 originals remain byte-identical with zero import failures. Native checks
prove the actual derived-segment and numeric/mixed API writers, live edits and
the original five-branch pentagon's25 final images. See
[derived seed evidence](mapped_derived_transform_analysis43.md) and
[scalar state evidence](numeric_mapped43_analysis.md).
The final scan is `/tmp/sketchpad_corpus_checkpoint43_final.log`.
Copying and custom tools remain deferred; general single-seed representation,
typed numeric states beyond Scalar and wider carried DAGs remain incomplete.
Scoped gates pass656 library,115 editor,34 numeric-import and four coverage tests
(809 total), using the same copying/custom-tool filters. Evidence:
`/tmp/sketchpad_checkpoint43_lib_final_gate2.log`,
`/tmp/sketchpad_checkpoint43_editor_gate.log`,
`/tmp/sketchpad_checkpoint43_numeric_gate.log`, and
`/tmp/sketchpad_checkpoint43_coverage_gate.log` (all exit0).


Checkpoint44 extends the existing mapped DAG predicate to ordinary geometry and
numeric measurements. It restores422 drawing slots across77 files:108 roots
(104 live-depth and4 fixed-depth) and314 point/segment bindings. Unsupported
slots fall to58751, including57566 drawing slots and1185 prototype slots.
All1599 originals remain byte-identical, with zero import failures. The final
scan is `/tmp/sketchpad_corpus_checkpoint44_final.log`.
Native full/abbreviated carried controls establish that only explicit carried
nodes and seeds update per state. External construction values and function
coefficient environments stay frozen; runtime caches preserve original Kind
metadata. Undefined carried images retain valid prefixes and other bindings.
Native measurements cover live angle/length inputs and a real center-arc graph;
path controls cover circle/segment positions and three intersection families.
The evidence also fixes oriented circle-circle branch choice and label ordering
for new transformed points and circle-circle intersections. Fixed/numeric
rotations and labeled API intersection files have exact Native writer proofs.
See [path/intersection evidence](mapped_path_intersection_analysis44.md) and
[measurement/arc evidence](measurement_mapped44_analysis.md).
Copying and custom tools remain deferred. General single-seed mapped storage,
binding types beyond points/segments and undefined branch-target behavior remain
outside this change. Decoding gains do not imply complete command parity.
Scoped regression gates pass664 library,115 editor,34 numeric-import and four
coverage tests (817 total), with the same copying/custom-tool filters.
Evidence: `/tmp/sketchpad_checkpoint44_lib_gate.log`,
`/tmp/sketchpad_checkpoint44_editor_gate.log`,
`/tmp/sketchpad_checkpoint44_numeric_gate.log`, and
`/tmp/sketchpad_checkpoint44_coverage_gate.log` (all exit0).

Checkpoint45 adds single-seed point/Scalar mappings through the same carried DAG,
recovering404 roots (377 live-depth and27 fixed-depth) plus720 point/segment
bindings. Generic authoring retains canonical existing point/segment/circle/ray
and branching-polygon wire adapters; shared binding templates and style handling
keep authoring, save/reopen and image-level edits consistent. Fresh carried
lists are stably ordered by their actual Native construction dependencies.
Midpoint dependencies include their segment host, and temporary seed values
preserve the original point kind. Original imported record order stays intact;
Native's historical unsorted-carried packet-order artifact is documented rather
than claimed equivalent to the port's graph evaluation.

The recursive polygon-source predicate additionally restores1314 drawing
reflection objects; the same parser changes restore5 prototype bindings/roots.
Total unsupported slots fall from58751 to56308:55128 drawing and1180 prototype
slots. All1599 originals are byte-identical and there are zero import failures;
93 documents now have all kinds decoded,94 all drawing kinds. The final scan is
`/tmp/sketchpad_corpus_checkpoint45_final.log`.

Native evidence includes11 fresh single-seed API controls, two original grid
constructions, an original compound-reflection graph and three fresh reflected
polygon API states including source and mirror edits. Source restoration,
final-level choice and structural undo/redo have meaningful regressions.
See [single-seed evidence](single_seed_iteration_analysis45.md) and
[recursive polygon evidence](transformed_curves45_analysis.md).
Scoped gates pass672 library,115 editor,34 numeric-import and4 coverage tests
(825 total), with the same copying/custom-tool filters. Evidence:
`/tmp/sketchpad_checkpoint45_lib_final.log`,
`/tmp/sketchpad_checkpoint45_editor_gate.log`,
`/tmp/sketchpad_checkpoint45_numeric_gate.log`, and
`/tmp/sketchpad_checkpoint45_coverage_gate.log` (all exit0).
Copying and custom tools remain deferred. Wider binding geometry, typed numeric
seeds beyond Scalar, general iteration dialogs and action variants remain open.


Checkpoint 46 adds shared lossless action record roles, optional labels and
empty metadata, plus ordered Movement pair lists. This restores 1,799 drawing
buttons and 47 prototype slots without adding copy or custom-tool implementation.
Unsupported slots fall 56,308 → 54,462. All 1,599 originals remain byte-identical,
zero imports fail, and 95 documents have every object kind decoded.
The final scan is `/tmp/sketchpad_corpus_checkpoint46_final.log`.

Movement controls share point/numeric values and preserve their original
construction: free and constrained points, supported coordinate units, and
editable Parameters use their existing mutation paths. Initial targets are
frozen together; actual Native overlap controls establish action order for
Follow. New buttons accept multiple selected source/destination pairs and an
entire activation has one undo checkpoint. Partial-document numeric editing
continues to use the existing Parameter safety policy. Runtime evidence and
limitations are recorded in [movement controls](movement_controls46_analysis.md)
and [action records](action_records46_analysis.md). Decoding counts include
buttons whose parents or runtime controls remain unavailable.


The shared horizontal-unit writer now emits edited positive steps for fresh
coordinate groups and patches saved code 52 constraints. Native accepted both
fresh square/rectangular edited controls; strict saved/reopen regressions cover
these paths. Stored Movement validation does not require an active page or a
currently defined value, while activation retains runtime safety guards.
Final scoped gates pass 687 library, 120 editor, 34 numeric-import and 4 coverage
checks (845 total), with the copying/custom-tool filters unchanged. Logs:
`/tmp/sketchpad_checkpoint46_lib_final.log`,
`/tmp/sketchpad_checkpoint46_editor_gate.log`,
`/tmp/sketchpad_checkpoint46_numeric_gate.log`, and
`/tmp/sketchpad_checkpoint46_coverage_gate.log`. All exited 0; `git diff --check`
is clean. Exact animation timing and newly observed polygon-point cache words
remain outside this checkpoint's verified parity.


Native's translated-Y control additionally proves that dependent square Y units
use the live horizontal step; their saved Y step is a cache. Shared geometry now
follows that dependency, while rectangular units retain independent steps. A
visible derived-point regression checks Native coordinates, horizontal edits,
unchanged cached metadata, strict save/reopen and whole-action undo/redo.


## Checkpoint 47: shared iteration output adapters

Generic carried-DAG bindings now expose circles, polygons, rays, lines and arcs
through the existing element/transform model. No shape-by-transform Kind products
were added. The corpus recovers 389 drawing bindings across 102 files plus one
prototype slot: 390 slots total, leaving 54,072 unsupported. All 1,599 originals
remain byte-exact, zero imports fail, and 95 documents have every kind decoded.
Final scan: `/tmp/sketchpad_corpus_checkpoint47_final.log`.

The Iterate command accepts a selected point mapping and supported geometry,
including derived point seeds and optional live Scalar depth. The adapters share
SVG/GPU drawing, picking, independent binding styles, final-level selection and
undefined-image isolation. Dependencies of derived seeds are carried; omitted
external hosts remain frozen. Circle-by-radius uses typed Length quantities as
well as defining segments. Original multipage calculation/table validation now
compares embedded references in page-local space while retaining global Kind IDs.

Scoped gates pass 700 library, 121 editor, 34 numeric-import and 4 coverage tests
(859 total); copying and custom-tool checks remain excluded. All gates exited 0.
Logs: `/tmp/sketchpad_checkpoint47_lib_final.log`,
`/tmp/sketchpad_checkpoint47_editor_gate.log`,
`/tmp/sketchpad_checkpoint47_numeric_gate.log`, and
`/tmp/sketchpad_checkpoint47_coverage_gate.log`.
See [binding audit and regressions](iteration_bindings47_analysis.md) and
[Native controls](iteration_bindings47_native_analysis.md). Decoding recovery does
not establish runtime completeness for every corpus iteration. Wider unsupported
parent roots and non-Scalar seed roles remain priorities; pixel-identical output,
copying and custom tools remain deferred.


## Checkpoint 48: analytic arcs and shared visibility actions

The common transform carrier now accepts analytic arcs across all fourteen wire
flavors, retaining source dependencies, directed parameterization, reflection,
signed scale, and undefined zero-scale ancestors. Linear three-point arcs retain
arc construction identity. Editing, picking, Point on Object, measurements and
mapped iteration consumers use the same geometry adapters. No shape-by-operation
Kind products were added. Actual coverage recovers 489 drawing slots and 14
prototype slots; the prior dependency audit's reach counts were upper bounds.
See [arc evidence](transformed_arcs48_native_analysis.md).

Hide/Show options 0–7 retain their native words. Fixed modes 0–3 assign one shared
visibility without changing the next action; alternating modes 4–7 change it
once per click. Caption format and visibility words are retained, custom format 7
is distinguished from automatically generated captions, and hidden Hide/Show
children can execute through Presentation. Full-group caption patches handle
changing byte lengths, optional label insertion and page-local references.
This recovers 3,448 drawing slots and 46 prototype slots.

Combined unsupported slots fall from 54,072 to 50,075: 3,997 recovered slots.
All 1,599 original files remain byte-exact, with 96 completely decoded documents
and zero failures. Final scan: `/tmp/sketchpad_corpus_checkpoint48_final.log`.
Copying and custom-tool support remain deferred. Exact raster matching is outside
the requested scope; this checkpoint does not establish full command parity.
