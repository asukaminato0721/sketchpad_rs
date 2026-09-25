# Polygon translation by a marked vector

The native fixture `tests/fixtures/vector_translation_polygon_original.gsp`
uses code 16 with references `[source polygon, vector start, vector end]`.
Its header uses category 4, subtype `49 + vertex count` (52 for this
triangle), the source fill color, marker bytes `51 5c`, and transform flag 1
at offset 20. There is no constraint payload. Native creation also writes a
trailing `0x2328` selection record.

The port evaluates each translated vertex from the source polygon and the
live vector endpoints. Native import is editable and byte identical on
unchanged save. The editor and replay command construct the polygon variant.
`tests/transform_vector_polygon_replay.txt` moves the vector endpoint after
creation; Sketchpad 5 displayed both interiors and saved that generated file
byte-identically.
