# Corpus decoding coverage

Checkpoint 38 snapshot on 2026-09-27, generated with `gsp_probe coverage ../gsp-rs`.

Documents: 1599 total, 89 with all object kinds decoded, 1510 with unsupported objects, 0 failed.

Byte-preserving round trips: 1599/1599; object slots: 285703.

Partial views: 1519/1599 (includes multi-page structural edit restrictions).

Drawing objects: 280347 slots; 90 documents with all drawing kinds decoded.

Embedded tool prototypes: 103 definitions in 65 documents; 5356 slots, 4167 decoded, 1189 undecoded.

Isolated tool views: 31 complete, 72 partial, 0 failed, with separate object and font namespaces.

Unsupported object slots including tool prototypes: 64477 (63288 drawing slots and 1189 prototype slots). Object decoding does not establish editing, interaction, or command parity. The release requirements remain in `COMMAND_PARITY.md`.

Embedded tool prototypes are decoded through isolated documents. Declared point placeholders receive neutral coordinates; declared path, finite-segment, and generic-numeric placeholders retain typed abstract inputs. Other unknown prototype constructions remain unsupported. The drawing and prototype histograms retain separate namespaces. Complete isolated decoding does not establish invocation or authoring support for every tool. This snapshot is not directly comparable to earlier drawing histograms that mixed tool prototypes with scene objects, or to snapshots that counted every prototype slot as undecoded.

The counts include missing variants and failures caused by unsupported parents.
Inspect complete groups and parent payloads before selecting a target.

| Native code | Unsupported objects | Documents | First example |
| --- | ---: | ---: | --- |
| 77 | 7405 | 797 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/【教程】用迭代构造平面直角坐标系.gsp` |
| 48 | 5982 | 481 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 62 | 5540 | 428 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/09年潍坊17题答案（蚂蚁制作）.gsp` |
| 16 | 3493 | 362 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 73 | 3298 | 1350 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/09年潍坊17题答案（蚂蚁制作）.gsp` |
| 75 | 2879 | 445 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/10福建宁德26题2(蚂蚁).gsp` |
| 90 | 2711 | 399 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 89 | 2538 | 772 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/【教程】用迭代构造平面直角坐标系.gsp` |
| 34 | 1928 | 139 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/化圆为方(蚂蚁制作).gsp` |
| 35 | 1904 | 402 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/09年潍坊17题答案（蚂蚁制作）.gsp` |

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
