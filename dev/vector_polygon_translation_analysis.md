# Polygon translation by a marked vector

The native fixture `tests/fixtures/vector_translation_polygon_original.gsp`
uses code 16 with references `[source polygon, vector start, vector end]`.
Its header uses category 4, subtype `49 + vertex count` (52 for this
triangle), the source fill color, marker bytes `51 5c`, and interior drawing order 1
at offset 20. There is no constraint payload. Native creation also writes a
trailing `0x2328` selection record.

The port evaluates each translated vertex from the source polygon and the
live vector endpoints. Native import is editable and byte identical on
unchanged save. The editor and replay command construct the polygon variant.
`tests/transform_vector_polygon_replay.txt` moves the vector endpoint after
creation; Sketchpad 5 displayed both interiors and saved that generated file
byte-identically.

## Generic polygon profile

The 2026-09-29 frontier audit found 13 reachable translations whose source and
target both use the native generic polygon header `[4, 0, 15, 0]`. They retain
the same code-16 reference schema and have no constraint payload. The shared
decoder now admits subtype 15 only when the source's original header is also
class 4/subtype 15; changing only the target header remains rejected. Those 13
roots unlock another 18 code-16 descendants and 21 objects of other codes, for
52 recovered drawing slots. All 1,599 corpus files remain byte-exact.
