# Polygon rotation and dilation

Native fixtures `tests/fixtures/fixed_rotation_polygon_original.gsp`,
`angle_rotation_polygon_original.gsp`, `fixed_dilation_polygon_original.gsp`,
and `ratio_dilation_polygon_original.gsp` establish the four polygon variants.
Their object codes are 27, 28, 30, and 32. Each header uses category 4,
subtype `49 + vertex count` (52 for a triangle), the source polygon's fill
color, interior drawing order 1 at offset 20, and marker bytes `51 5c` at offsets
18–19. References use the same order as the point variants: source and center
for fixed transforms, followed by three defining points for marked transforms.
Fixed rotation and dilation use the same 36-byte and 12-byte constraint payloads
as points; marked transforms have no constraint payload. A trailing `0x2328`
record identifies the selected transformed polygon.

The port evaluates transformed vertices from the source polygon and the live
center, angle, or ratio points. Rendering, hit testing, and SVG use that live
geometry. Imported native fixtures retain their fill style and re-encode
byte-identically. Editor and replay commands construct all four variants.

`tests/transform_fixed_rotation_polygon_replay.txt`,
`transform_marked_angle_polygon_replay.txt`,
`transform_fixed_dilation_polygon_replay.txt`, and
`transform_marked_ratio_polygon_replay.txt` generate polygons and then move
a source or defining point. Sketchpad 5 displayed both interiors in all four
moved files and saved each file byte-identically. The native fixtures and
editor dispatch have regression tests.
