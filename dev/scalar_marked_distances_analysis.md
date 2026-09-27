# Scalar values as marked translation distances

2026-09-27, checkpoint 33. Copying and custom tools remain deferred.

## Original dependency root

The checkpoint31 dependency audit counts 2117 unsupported calculations across
73 files with a terminal code24/class1/subtype4 root, including overlapping
blockers. Original `n×n立方体.gsp` object23 translates point14 using calculation17,
whose result is Scalar 14. Its paired original HTML explicitly declares
`Translation/FixedAngle/MarkedDistance(14,17,0)`. The retained
`scalar_marked_distance_cube_corpus.gsp` regression verifies that calculation,
the 14-centimeter translation, the subsequent rotation, a later calculation and
Cartesian translation, and exact preservation of the original file. HTML object
ordinals later diverge from GSP ordinals; regression IDs come from actual records.

## Native semantic controls

The local probe builds a free point, a code24 marked-polar image, their segment
and a length measurement. It retains a real Scalar Parameter schema and the
existing native transform record schema; the probe replaces a fixed-polar group
with the two-reference marked group. These hand-assembled controls establish
semantics, separately from production serializer proof.

The unchanged native helper opens and saves Scalar +3, -3 and 0 files. Native
captures show a measured segment of 3.00 with the image right of its source for
+3, a measured 3.00 with the image left for -3, and measured 0.00 with coincident
points for 0. Thus Scalar magnitudes use centimeters and retain sign. Each helper
exits 0 and an independent comparison to an untouched snapshot passes. Captures
and logs are `/tmp/scalar_distance_{positive,negative,zero}_native_*`.
An initial imported typed-negative control had a blank Native canvas, so it was
not sufficient evidence for its domain. Further isolated typed-Length +3/-3
controls use the same visible point/segment/measurement setup. Native displays
3.00 in both, with the negative image clearly left of its source. Both controls
save exactly (2330/2332 bytes). These direct controls establish that signed Length
operands reverse direction too; the old nonnegative marked policy was incorrect.
The retained typed-isolated fixtures and captures provide this evidence.

## Implementation

`Kind::is_marked_distance_value` accepts Length and Scalar. The separate
`marked_distance_value_pixels` evaluator converts numeric magnitude to centimeters
in pixels, permits signed Scalar and Length values. `is_distance_value` and `distance_value_pixels` remain typed Length
interfaces used by arithmetic. This prevents a geometry interpretation from
silently weakening dimensional checks in length calculations.

Existing `NumericExpr::ObjectValue { unit: Length }` operands describe the
translation's operation unit; they accept Scalar with this explicit conversion.
Other operation units still require exact numeric units. Polar, Cartesian and
mixed transform decoders, parent validation, matrix evaluation and iteration
mapping reuse the same marked-distance predicate/value evaluator. The editor
accepts one or two marked numeric distances and identifies the centimeter
interpretation for a unitless marked value. No new transform enum variants exist.

## Actual writer proof and regression

The public Document APIs build a Scalar Parameter and a Scalar constant, three
free points, segment/circle/polygon sources, four polar images, a Cartesian point
image and a mixed point image. The Parameter is edited to -3 before saving.
`dev/wine_roundtrip.sh` opens and saves this actual 3172-byte serializer output,
exits 0, and an independent byte comparison passes. Its retained
`scalar_distance_marked_transforms_writer_native_verified.gsp` is reconstructed
byte for byte by a regression; this proof is independent of the schema probes.

Tests verify Scalar +3/-3/0, all four element geometries, Cartesian and mixed
components, saved edits, undo/redo, undefined calculated operands, editor marking,
and iteration steps with pinned numeric values. Typed-Length negative
cases now verify reverse translations; true undefined expressions suppress images. The original cube source imports without rewriting bytes.

Corpus checkpoint33 retains 1599/1599 exact source round trips and recovers
6610 drawing objects including760 calculations, plus108 embedded prototype slots.
Prototype decoding does not establish custom-tool authoring or invocation support.
Unsupported slots total78026 (76811 drawing +1215 prototype).

Scoped final validation: 609 library tests, 34 numeric-import tests and 108 editor
tests pass, all with zero failures. The additional isolated typed positive/negative
control is also checked after adding its fixture. Logs:
`/tmp/sketchpad_scalar_signed_distance_final_lib_gate.log`,
`/tmp/sketchpad_scalar_signed_distance_final_import_gate.log`,
`/tmp/sketchpad_scalar_signed_distance_final_editor_gate.log`, and
`/tmp/sketchpad_scalar_signed_length_controls_gate.log`.
Native semantic captures for +3/-3/0 Scalars and +3/-3 Length operands are retained
next to their `.gsp` fixtures. Diff whitespace checks pass.
