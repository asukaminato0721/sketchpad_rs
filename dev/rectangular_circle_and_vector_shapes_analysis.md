# Rectangular circle and vector translated shapes

Native probes translated a circle by fixed rectangular distances and translated a segment and circle by a marked vector. The files are `tests/fixtures/rectangular_circle_translation_original.gsp`, `tests/fixtures/vector_segment_translation_original.gsp`, and `tests/fixtures/vector_circle_translation_original.gsp`.

The rectangular circle is code 17 with circle header subtype 28, one reference to the source circle, and the same 52-byte fixed rectangular payload used by translated points and segments. Both vector translated shapes are code 16 with references `[source, vector_start, vector_end]` and no constraint payload. The segment has header subtype 14; the circle has subtype 28. Both vector headers use transformed-object marker bytes `51 5c`.

The port retains each source shape's live radius or endpoints, then offsets it by the fixed rectangular values or the displacement between the vector points. The importer recognizes these native files without partial view. The writer produces the matching headers, while `circle_points` and `line_points` feed SVG, hit testing, and the desktop renderer. The desktop circle path now includes all supported translated circle kinds.

`tests/transform_circle_rectangular_replay.txt`, `tests/transform_segment_vector_replay.txt`, and `tests/transform_circle_vector_replay.txt` create Rust objects and move their defining points. Sketchpad 5 opened and saved all three edited files byte-identically. Core and editor tests verify import identity and live dependencies.
