# Rectangular polygon translations with measurements

Native fixture `tests/fixtures/rectangular_marked_polygon_original.gsp`
uses code 20 with references `[source polygon, horizontal measurement,
vertical measurement]`. The two mixed native fixtures use code 18 for fixed
horizontal and code 19 for fixed vertical, with references to the source
polygon and the remaining marked measurement. Both mixed variants have the
same 28-byte constraint payload structure as the corresponding point modes.
All three headers use category 4, subtype `49 + vertex count`, source fill
color, marker bytes `51 5c`, and transform flag 1 at offset 20. Native
creation also adds a trailing `0x2328` selection record.

The port evaluates the source vertices with the current measurements. Native
fixtures import as editable documents and re-encode byte-identically. The
editor and replay commands create each variant. Generated files after moving
both measurement endpoints displayed the translated interiors in Sketchpad 5
and resaved byte-identically. The replays are
`tests/transform_rectangular_marked_polygon_replay.txt` and
`tests/transform_rectangular_mixed_polygon_replay.txt`.
