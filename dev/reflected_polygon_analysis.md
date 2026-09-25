# Reflected polygon interiors

`tests/fixtures/reflection_polygon_original.gsp` is a native reflection of a
triangle across a vertical segment. The bundled `tests/fixtures/reflection.gsp`
contains a reflected quadrilateral. Both use code 34 with references
`[source polygon, mirror]`, category 4, and a polygon subtype equal to
`49 + vertex count` (52 for the triangle, 53 for the quadrilateral). Header
offset 20 contains a 32-bit transform flag of 1. The reflected header keeps
the source polygon's fill color.

The port evaluates each reflected vertex from the source polygon and the
mirror's live line, and uses the result for native rendering, SVG, and hit
testing. Native files import without partial view and re-encode identically.
The editor's Reflect command accepts a selected polygon interior. New
reflected polygons inherit the source fill style.

Native creation also writes a trailing `0x2328` record naming the selected
image. A Rust-written reflected polygon without that record hid its source
objects when reopened in Sketchpad 5. The writer now adds the record. When a
point is moved, it updates the record to that point, matching Sketchpad's
own drag save byte-for-byte in
`tests/fixtures/reflection_polygon_mirror_dragged_by_wine.gsp`.

`tests/transform_reflection_polygon_replay.txt` creates a reflected triangle
and moves a source vertex. Sketchpad 5 displayed both interiors and saved the
file byte-identically. A tilted mirror displays correctly during a native
drag, but Sketchpad 5 reopens even its own saved file with a blank canvas for
this probe; `tests/transform_reflection_polygon_native_moved_replay.txt`
reproduces that native limitation. The Rust document geometry remains live
for that case.
