# Typed length values as marked transform distances

The native corpus already includes code24 transformations whose distance source
is a subtype80 Length Parameter. The old distance predicate listed measurement
and selected calculation variants, omitting parameters even though their numeric
unit and value were decoded. This blocked the transform and its descendants.
An exact original control is honeycomb object15/source12, documented in
[the dependency audit](broad_measurement_dependency_analysis.md).

`Kind::is_distance_value` now asks whether the numeric unit is Length. The pixel
evaluator uses the common `numeric_value` evaluator and centimeter conversion,
removing a second list of concrete value variants. Existing finite/nonnegative
rules remain: zero is valid, negative or undefined values suppress the image,
and saving the supported stored graph is allowed. Scalar, angle and area values
are not accepted as distances. This unifies typed sources for existing polar,
Cartesian and mixed transforms, calculations and iteration prerequisites.
No transform-by-element enum variants were introduced.

The editor uses the same pixel evaluator for Mark Distance, so selecting a
Length Parameter now works through the normal command. Its invalid-selection
message asks for length values. The editor regression checks marking without
modifying saved bytes, command-created translation, parameter changes, strong
save/reparse validation and undo/redo.

## Verification

- Library live regression: a Native verified three-parameter document supplies
  the Length Parameter. Point/segment/circle/polygon polar translations use it;
  Cartesian and mixed translations additionally use typed calculation values.
  Parameter changes2/0/-1/1.5 preserve the existing zero/undefined policies and
  survive save, strong validation and undo/redo.
- Actual writer: `/tmp/sketchpad_typed_length_writer.rs` builds that supported
  graph, edits the length to2cm, and saves it. The unchanged
  `dev/wine_roundtrip.sh` opens and saves it in Sketchpad5; helper exit0 and an
  independent comparison against `/tmp/sketchpad_typed_length_writer_before.gsp`
  establish byte-identical output. The retained golden is
  `tests/fixtures/typed_length_marked_transforms_writer_native_verified.gsp`.
  The import regression checks original geometry, edits to1.5cm, dependent
  Cartesian/mixed results, save and undo/redo. The Native check proves this
  actual writer output, not Native UI behavior for negative length values.
- Native log: `/tmp/sketchpad_typed_length_writer_native_gate.log`.
- Library gate: `/tmp/sketchpad_typed_distance_final_gate.log`,608 passed,
  110 copying/custom-tool name-filtered tests deferred.
- Editor gate: `/tmp/sketchpad_typed_distance_editor_gate.log`,107 passed,
  21 name-filtered tests deferred.
- Numeric import integration gate:
  `/tmp/sketchpad_typed_distance_numeric_import_scoped_gate.log`,32 passed,
  10 filtered with clipboard/copy/copies/copied/paste/tool naming filters.
  The initial narrower name filter left the `sourcefree_radian...copies` test
  enabled; it failed while locating a pasted NumericCoordinatePoint. That
  copying path remains deferred by the user's scope, and was not changed here.
- Corpus checkpoint30:1599 exact original round trips,1219 recovered drawing
  objects, including629 code24 and32 code48 objects. Prototype coverage is
  unchanged. Unsupported counts are now83715 drawing +1323 prototype =85038.

Remaining prerequisites include Scalar-source marked distances with unproved
unit conventions, further transformation source layouts, and save restrictions
for otherwise valid imported graphs with undefined coordinates. Source-free
centimeter-expression authoring still has its own parser restriction; this
change does not claim it is implemented.


Checkpoint33 follow-up: Scalar marked distances now have native semantic and
actual-writer controls, documented in `scalar_marked_distances_analysis.md`.
They use signed centimeters. Isolated native controls also prove signed Length
translation, correcting the earlier unproved negative-Length suppression rule.
Typed-Length arithmetic retains its unit checks. Source-free
quantity authoring was completed at checkpoint31, with separate evidence.
