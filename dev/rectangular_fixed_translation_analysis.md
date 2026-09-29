# Fixed rectangular translation

The native probe uses `dev/rectangular_fixed_wine_probe.sh` and the five-point, two-measurement base sketch from `tests/transform_rectangular_distances_replay.txt`. In the native Translate dialog, rectangular mode offers independent fixed horizontal and vertical distances. The default is 1.0 cm for each. A second probe entered horizontal 2 cm and vertical −0.5 cm; its saved file is `tests/fixtures/rectangular_fixed_translation_original.gsp`.

The translated point is object code 17. Its point header has the transformed-point marker bytes `51 5c`, and `0x07d2` references the source point alone. The 52-byte `0x07d3` payload starts with code 17. At offsets 4 and 24 it stores horizontal and vertical signed pixel offsets, at 12 and 32 it stores the corresponding displayed magnitudes, and 32-bit values at 20 and 40 select the length unit. Tag 0 is literal pixels at one pixel per unit, tag 1 is centimeters at 37.79527559055118 pixels per unit, and tag 2 is inches at 96 pixels per unit. The trailing eight bytes varied between native saves, so the importer retains them in the source file and the writer uses the captured native default payload for a newly created point.

The port evaluates the new point as `(source.x + horizontal_px, source.y - vertical_px)`. Thus a negative vertical distance moves the point down in the document's screen coordinates. `tests/transform_rectangular_fixed_replay.txt` creates a 2 cm, −0.5 cm translation, moves its source point, and saves it. Sketchpad 5 opened and saved that generated file byte-identically after the move.

The 18-page native `初中数学积件.gsp` contains ten visible code-17 Segment
translations whose two unit tags are 2. Each stores magnitude
`0.03937007983525594` and pixel cache `3.7795276641845703`, exactly the
96-pixel/inch conversion. All ten now use the shared fixed-Cartesian transform
model and retain their labels and original bytes. A separate four-object file
authored by the port translates a Segment by `(0.5, -0.25)` inches. Sketchpad 5
displayed the expected `(48, 24)` screen-pixel displacement under Xvfb/Wine and
resaved all 2,088 bytes unchanged.

Seven path-point images in the same multi-page teaching file use tag 0 on both
axes. Each stores magnitude `0.001` and an identical `0.001` pixel cache. Their
page-local sources are supported points on segments, circles, or an arc, and all
seven now decode through the shared fixed-Cartesian model. A separate 1,956-byte
port writer used `(120, -80)` pixel magnitudes. Sketchpad 5 opened it under
Xvfb/Wine and placed the image exactly 120 pixels right and 80 pixels down from
the source. The full corpus remains byte-exact; unsupported slots fall
2,830→2,823.
