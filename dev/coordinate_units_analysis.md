# Numeric coordinate units and literal point dependencies

Checkpoint42 implements two shared coordinate prerequisites identified by the
current drawing-only dependency audit: numeric-sized code59 axes and literal
code67/68 points. The final scan recovers3841 drawing slots, including282
calculations, and one prototype slot through the shared reader. No transform by
shape enum variants were added; coordinate evaluation uses the common numeric,
segment and frame interfaces.

## Actual corpus roots

The checkpoint41 numeric audit leaves5424 unsupported drawing calculations;
5410 lack supported parents, nine fail headers and five fail unit rules.
Terminal root families overlap: code59 affects1566 calculations in93 files,
code101 affects1140 in27, and code15 affects654 in101. These are dependency
counts, not projected independent recoveries. Logs are
`/tmp/numeric_import_audit42.log` and `/tmp/broad_measure_roots_checkpoint42.log`.

All509 drawing code59 axes use the observed direction values0/1 in0905.
Previously310 were unsupported:294 had ready Length sources,14 ready Scalar
sources, and two unsupported origins. All333 numeric sources in the original
corpus have positive finite magnitudes. The old distance predicate only listed
Segment, SegmentLength and PointDistance, excluding valid numeric calculations.

The retained [calendar](../tests/fixtures/coordinate_axes_calendar_corpus.gsp)
establishes exact dependencies: axes18/19 refer to origin11 and Length
calculations16/17; frame20 refers to those axes. Literal point21 is the origin
of that frame. Its recovered point-ratio33 and calculation35 produce13.758125
and13 respectively. The paired HTM exports the original unit points, distances
and calculations but omits the internal axes and renumbers later objects. It
cannot prove same-ID axis semantics. Raw evidence is
`/tmp/calendar_objects42.log`; unit/profile audits are `/tmp/axis_units59.log`
and `/tmp/axis_sign59.log`.

## Coordinate unit semantics established in Native

Stored numeric axes use their source's magnitude as centimeters, multiplied by
37.79527559055118 pixels/cm. Native controls independently prove Length, Scalar,
Degree, Area and Compound sources. Angular and dimensional units do not rescale
the magnitude. Finite segment sources use their geometric length. The editor's
Define Coordinate System selection accepts segments and Length quantities;
Native selection of a Scalar alone creates a default frame, ignoring that
selection. The editor retains that behavior.

A nonpositive numeric unit makes the frame and dependent points/measurements
undefined. Native negative/zero controls show no grid or plotted point. These
stored graphs now save and reopen; returning to a positive value restores their
geometry. Validation still rejects missing/forward/deleted/cross-page references,
wrong source kinds, invalid axis modes and invalid fixed steps. Undefined-state
exceptions are restricted to structurally valid distance-axis families.

## Counted literal point references

All790 audited literal groups (789 Cartesian, one Polar) store07d2 as
`[count=1, frame]`. The first word is not an object ID. The previous parser
invented a dependency on object1; existing writers produced valid bytes because
they used that same value1 by coincidence. `Kind::PlottedPoint` now stores its
real frame, mode and coordinates. Existing golden output bytes remain unchanged.

The parser accepts bounded class1/subtype56 headers, native visibility flags,
optional07d5/07d8 labels/fonts and empty232e attributes. Other layouts remain
unsupported. Meaningful regressions retain the original calendar and
[Polar corpus file](../tests/fixtures/literal_polar_point_corpus.gsp), check the
single dependency, preserve exact bytes, and remove an unrelated first point
without deleting the plot. The label/name writer supports imported and new
literal points. Source/source-role checks are performed on the actual frame.

## Actual Document writer proof

[The reproducible generator](coordinate_units_generate.rs) uses public Document
APIs and strict `verify_roundtrip` to reconstruct all nine Native-verified files.
The original writer creates a12-object graph with a Length parameter, Scalar
coefficient, computed horizontal unit, literal vertical unit, plotted point and
coordinate measurements. Edited and restored outputs retain the live graph.
All files opened and saved byte-identically in Sketchpad5, with helper exit0 and
independent comparisons. Numeric-unit controls initially used the actual record
API; the final Document generator reconstructs those exact Native-saved bytes.

| State | Bytes | Native behavior | Retained file/capture |
| --- | ---: | --- | --- |
| Original | 3512 | Units2cm/3cm; plotted(1,1), both coordinate measures1 | [GSP](../tests/fixtures/coordinate_units_writer_original_native_verified.gsp), [PNG](../tests/fixtures/coordinate_units_writer_original_native_verified.png) |
| Edited | 3520 | Units3cm/2.5cm after parameter/formula edits | [GSP](../tests/fixtures/coordinate_units_writer_edited_native_verified.gsp), [PNG](../tests/fixtures/coordinate_units_writer_edited_native_verified.png) |
| Negative | 3522 | Vertical-2.5cm; grid and dependent geometry undefined | [GSP](../tests/fixtures/coordinate_units_writer_negative_native_verified.gsp), [PNG](../tests/fixtures/coordinate_units_writer_negative_native_verified.png) |
| Zero | 3516 | Vertical0cm; grid and dependent geometry undefined | [GSP](../tests/fixtures/coordinate_units_writer_zero_native_verified.gsp), [PNG](../tests/fixtures/coordinate_units_writer_zero_native_verified.png) |
| Restored | 3564 | Vertical2.5cm; geometry restored and point namedP | [GSP](../tests/fixtures/coordinate_units_writer_restored_native_verified.gsp), [PNG](../tests/fixtures/coordinate_units_writer_restored_native_verified.png) |
| Scalar | 3520 | Source2 sets horizontal75.59px | [GSP](../tests/fixtures/coordinate_units_scalar_native_verified.gsp), [PNG](../tests/fixtures/coordinate_units_scalar_native_verified.png) |
| Degree | 3520 | Source1° sets horizontal37.80px | [GSP](../tests/fixtures/coordinate_units_degree_native_verified.gsp), [PNG](../tests/fixtures/coordinate_units_degree_native_verified.png) |
| Area | 3524 | Source4cm² sets horizontal151.18px | [GSP](../tests/fixtures/coordinate_units_area_native_verified.gsp), [PNG](../tests/fixtures/coordinate_units_area_native_verified.png) |
| Compound | 3536 | Source4cm⁴ sets horizontal151.18px | [GSP](../tests/fixtures/coordinate_units_higher_native_verified.gsp), [PNG](../tests/fixtures/coordinate_units_higher_native_verified.png) |

Final negative/zero/restored Native checks are logged in
`/tmp/coordinate_units42_final_native.log`; original/edited logs are
`/tmp/coordinate_length_writer42_{original,edited}_native.log`. Numeric-source
logs use `/tmp/axis_{scalar,degree}_remap42_actual_native.log` and
`/tmp/axis_{area,higher}_four_actual42_native.log`. Earlier Area candidates that
encoded unchanged originals and default-prefix kernel32 failures are not
verification evidence. Final files were decoded before Native checks and then
reconstructed exactly through the Document generator.

## Gates and remaining work

The scoped gates pass650 library,115 editor,34 numeric-import and four coverage
tests (803 total). Five new document, three record and three editor regressions
cover live edits, units, undefined/recovered states, exact authoring, undo/redo,
real dependencies and malformed-input rejection. All1599 corpus files retain
exact original bytes; imports have zero failures. Unsupported slots are59442:
58257 drawing slots and1185 prototype slots. Shared reader recovery adds no
custom-tool invocation or authoring capability.

General iteration/mapped-DAG roots, their bindings and several action layouts
remain broader follow-up work. Function-call result-unit propagation remains
partial; accepting a stored axis's numeric unit does not repair that upstream
limitation. General multi-page structure editing, copying and custom tools stay
outside this checkpoint; copying/custom tools remain deferred by user request.
Pixel-identical display is not required.
