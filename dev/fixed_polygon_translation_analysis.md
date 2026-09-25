# Fixed polygon translations

Native fixtures `tests/fixtures/polar_translation_polygon_original.gsp`
and `rectangular_translation_polygon_original.gsp` use codes 21 and 17.
Both reference the source polygon alone. Their headers use category 4,
subtype `49 + vertex count`, the source fill color, transform flag 1 at
offset 20, and ordinary marker bytes `39 20`. The fixed polar variant uses
the same 60-byte constraint payload as a point translation. The rectangular
variant uses the same 52-byte payload. Native creation adds a trailing
`0x2328` selection record.

The port evaluates each translated vertex from live source polygon points.
Native fixtures import as editable documents and re-encode byte-identically.
The editor and replay commands construct both variants. Generated files with
one moved source vertex displayed both interiors in Sketchpad 5 and resaved
byte-identically. The replays are
`tests/transform_polar_polygon_replay.txt` and
`tests/transform_rectangular_polygon_replay.txt`.
