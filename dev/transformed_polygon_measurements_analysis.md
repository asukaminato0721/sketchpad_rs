# Measurements of transformed polygon interiors

Native files `tests/fixtures/transformed_polygon_perimeter_original.gsp` and
`transformed_polygon_area_original.gsp` measure a fixed-angle rotated
triangle. They use the same code 39 and 42 measurement groups as base
polygon perimeter and area, with their single reference pointing to the
transformed polygon. The native command also adds a 0x07d5 object label and
0x07d8 style reference to that polygon. The importer validates these records
and preserves their bytes without treating the document as a partial view.

The port accepts each supported live polygon variant as a perimeter or area
source. `tests/transformed_polygon_measurements_replay.txt` adds both values
to a rotated triangle and moves a source vertex. Sketchpad 5 displayed the
updated perimeter and area and saved the generated file byte-identically.
