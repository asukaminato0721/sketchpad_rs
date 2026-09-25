# Marked polar distance translation of segments and circles

`tests/fixtures/marked_distance_segment_translation_original.gsp` and
`tests/fixtures/marked_distance_circle_translation_original.gsp` were created
with Sketchpad 5's Mark Distance and polar Translate commands. Both translate a
source shape 100 sketch units upward at a fixed 90° angle. Their last object is
code 24 with `0x07d2` references `[source shape, distance measurement]` and the
same 36-byte `0x07d3` angle constraint used for a marked-distance translated
point. The segment header has subtype 14, the circle header subtype 28, and
both use transformed-object marker bytes `51 5c`.

The port imports, evaluates, and writes these objects. A translated segment
offsets both live endpoints; a translated circle offsets its live center and
retains its radius. The editor's marked-distance polar Translate action accepts
selected segments and circles as well as points.

`tests/transform_marked_distance_segment_replay.txt` and
`tests/transform_marked_distance_circle_replay.txt` create the port's versions,
then move the measurement endpoint so the distance changes from 100 to 150
sketch units. Sketchpad 5 opened and saved both moved files byte-identically.
The importer reports neither native fixture as a partial view. Core and editor
tests cover the shape variants and their live measurement dependency.
