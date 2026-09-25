# Polygon interior area

Sketchpad 5 created `tests/fixtures/polygon_area_original.gsp` by measuring
a triangular interior. It uses object code 42 with one reference to the
source polygon. Its 110-byte measurement group matches the circle Area
template exactly apart from the source object ID. The displayed area is
0.56 cm² for a triangle with base and height of 40 sketch units.

The port calculates live polygon area with the shoelace formula, imports the
native measurement as editable, and uses the same group for writing. Area
appears in the editor overlay and SVG export. In
`tests/polygon_area_replay.txt`, moving a vertex changes the area to
0.84 cm². Sketchpad 5 displayed that value and resaved the generated file
byte-identically.
