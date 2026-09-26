# Point-to-straight-object distance

Native Sketchpad 5 creates code 38 for Measure → Distance with a point and
segment, ray, or line selected. Parents are `[point, straight object]` in 0x07d2.
The 114-byte object group has the same layout for all three hosts; the 0x0903
format payload begins at group offset 64, and saved precision is at payload
bytes 28–29. The template in `assets/point_line_distance_measurement_group.bin`
was extracted from the native segment fixture.

## Geometry evidence

The segment runs from (100,180) to (200,180), with the measured point at
(250,120), beyond its end. The ray has the same endpoints with the measured
point at (50,120), behind its start. The infinite-line case uses (250,120).
Native captions show **1.59 cm** in every case: 60 pixels / 37.79527559055118.
Clamping to an endpoint would give 2.07 cm, so the evaluator measures the
perpendicular distance to the infinite supporting line in all three cases.
The sibling `../gsp-rs` code-38 decoder independently uses its raw supporting
line evaluator. Degenerate straight geometry gives no numeric value; restoring
its endpoints restores downstream formulas.

## Port support

Measure → Distance accepts two points or a point and straight object, in either
selection order. Code 38 imports, adds, copies/remaps, removes with dependents,
and saves on supported pages. Saved precision 0–5 affects captions and SVG;
calculations receive the full value in centimeters. The value can feed generic
length formulas, measured-length arithmetic, and marked-distance transforms.
Calculated distances can drive point iteration. Transformed straight hosts use
the shared live line geometry.

`tests/replay_point_line_distance.txt` creates seven cross-open probes: the three
host families, a moved/precision-edited segment, calculated-distance iteration
and its live edit, and a second-page ray measurement. Native creation uses
`dev/point_line_distance_wine_setup.sh` with each base geometry and an appropriate
capture prefix containing `segment`, `ray`, or `line`.

All seven probes opened in native Sketchpad 5 and saved byte-identically.
Captured captions and iteration points agree with the port before and after the
live edit. The edited segment caption and SVG both read 2.3813 cm.
The full test suite passes 577 tests (478 library, 94 desktop, and five integration).
