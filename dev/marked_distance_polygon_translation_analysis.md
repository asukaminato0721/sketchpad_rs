# Marked distance polygon translation

The native fixture `tests/fixtures/marked_distance_polygon_translation_original.gsp`
uses code 24 with references `[source polygon, distance measurement]`.
Its header has category 4, subtype `49 + vertex count`, source fill color,
marker bytes `51 5c`, and interior drawing order 1 at offset 20. The 36-byte angle
constraint matches marked-distance point, segment, and circle translation.
Native creation adds a trailing `0x2328` selection record.

The port translates each live source vertex by the current distance value
at the fixed angle. Native import is editable and re-encodes byte-identically.
The editor and replay command construct this variant. In
`tests/transform_marked_distance_polygon_replay.txt`, moving the measurement
endpoint changes the translated polygon's position. Sketchpad 5 displayed
both interiors and saved the moved generated file byte-identically.
