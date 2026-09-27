# Polar transform units and source chains

Native code21 translation contains an angle, angle convention, total pixel
distance, displayed distance magnitude and distance unit. Earlier import and
generation treated the pixel-distance cache at offset32 as a conversion scale.
That multiplies the magnitude twice: an original .3 cm translation with cache
11.338582677165356 became 3.4015748031496067 pixels. Paired Native HTML exports
independently give the correct displacement. See the exact original groups in
[the dependency audit](broad_measurement_dependency_analysis.md).

The shared transform model continues to store a numeric magnitude and scale.
Import now recovers scale from cache/magnitude, using the observed unit1
37.79527559055118 and unit2 96 for zero magnitudes and snapping normal scales
within relative 1e-12 to avoid introducing division rounding errors. Nonzero
magnitudes require a finite positive recovered scale. Generation writes
scale*magnitude into the pixel cache and selects unit2 for scale96. Fractional,
negative and signed-zero distances survive strong round-trip and undo/redo.
The generator already has the scale/magnitude representation; no additional
transform-by-shape Kind variants were added.

Angle word28=0 stores radians. Words1 and2 use degrees in the observed corpus.
The shared decoder checks finite fields and sin/cos cache errors below1e-6
in the declared convention before normalizing to model degrees. This covers
code21 fixed polar, code24 marked polar and code27 fixed rotation. Preserving
saves retain the original raw radians/degree flags and cache bytes. Unknown
angle conventions remain unsupported. Dynamic distance unit/source policies
are unchanged; Scalar and Length Parameter marked-distance gaps need separate
work, not guessed conversions.

Fixed-polar segments, circles and polygons accept supported transformed sources
using the shared predicates and recursive polygon vertex count. Native subtype,
payload length/opcode and literal validation remain. The two-step regression
checks matrix source/count, live source edits, downstream length/areas, save,
strong reparse validation and undo/redo.

## Evidence

- Five retained corpus originals: `polar_radian_fractional_distance_corpus.gsp`,
  `polar_fractional_distance_corpus.gsp`, `polar_inch_distance_corpus.gsp`,
  `rotation_radian_corpus.gsp`, and `marked_polar_radian_corpus.gsp`.
  Their paired source HTML and exact groups are documented in the dependency
  audit. Where original graph ancestors remain undefined, a regression
  evaluates the imported literal on a free source to isolate the conversion.
- The old port-generated `locus_polar_polygon_target_base.gsp` has one repaired
  8-byte distance cache (37.79527559055118 ->100.15748031496062 for2.65cm).
  Its retained Native locus counterpart already contained total pixel distance
  and remains untouched. Both now agree on locus geometry.
- Actual API writer: `/tmp/sketchpad_polar_cache_writer.rs` constructs two-step
  segment/circle/polygon translations by .3 and2.65cm at23 and-31degrees, plus
  downstream measurements. The unchanged `dev/wine_roundtrip.sh` helper ran
  with `/home/w/.cache/sketchpad_rs_wine`, exited0, and an independent `cmp`
  against `/tmp/sketchpad_polar_cache_writer_before.gsp` exited0. The output
  is retained as `polar_translation_cache_and_chains_writer_native_verified.gsp`
  and reconstructed byte-exactly by a library regression. Log:
  `/tmp/sketchpad_polar_cache_writer_native_gate.log`.
- `/tmp/sketchpad_polar_units_final_gate2.log`:606 library tests passed,
  110 copying/custom-tool name-filtered tests deferred.
- `/tmp/sketchpad_polar_units_editor_gate.log`: 106 editor tests passed,
  21 name-filtered tests deferred.
- `/tmp/sketchpad_corpus_checkpoint29.log`:1599 exact original round trips,
  86257 unsupported slots. This does not establish complete editing coverage.

Checkpoint28 recovers952 drawing/45 prototype slots; checkpoint29 adds5041
drawing/45 prototype slots, including1065 additional calculations. Combined
gain since27:5993 drawing slots,90 prototype slots and1423 calculations.
Original file integrity and semantic geometry evidence are separate gates.
