# Mark Distance and polar translation

`tests/transform_marked_distance_replay.txt` creates points `(150, 120)` and
`(250, 120)`, measures their distance, and translates a source point
`(100, 220)` by that marked distance at a fixed 90° angle. The measured
distance is 100 sketch units, so the image point is `(100, 120)`. Moving the
second measured point to `(300, 120)` moves the image to about `(100, 70)`.

`dev/marked_distance_wine_probe.sh /tmp/sketchpad_marked_distance_base.gsp`
reproduces Transform > Mark Distance and Translate in Sketchpad 5. The native
result is `tests/fixtures/marked_distance_translation_original.gsp`.
Saving after Mark Distance but before Translate is byte-identical to the
unmarked base: the mark is session state.

The native translated point has object header code 24. Its `0x07d2`
references are `[source point, distance value]`; a 36-byte `0x07d3`
constraint stores the fixed angle and its sine/cosine terms. The port reads,
evaluates, and writes this construction. The editor offers Mark Distance and
a polar Translate option that uses the marked distance with a chosen fixed
angle. It accepts distance measurements already supported by the document
model; the native probe uses a two-point distance measurement.

The port-generated files are
`tests/fixtures/marked_distance_translation_generated.gsp` and
`tests/fixtures/marked_distance_translation_moved_generated.gsp`. A third
file, `tests/fixtures/marked_distance_translation_angle30_generated.gsp`,
uses a 30° fixed angle. Sketchpad 5 opened and re-saved all three
byte-identically and displayed both translated positions as expected.
Marking two values for rectangular translation,
distance parameters and calculations, and combining marked angle with
marked distance remain outside this implementation.
