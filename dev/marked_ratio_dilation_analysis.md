# Three-point marked ratio and dilation

`tests/transform_marked_ratio_replay.txt` creates a center `(100, 120)`,
ratio points `(180, 100)`, `(230, 100)`, `(205, 100)`, and a source point
`(140, 120)`. Transform > Mark Ratio uses signed `AC / AB`, so the initial
factor is ½ and the image point is `(120, 120)`. Moving the ratio target to
`(255, 100)` changes the factor to 1½ and the image to `(160, 120)`.

`dev/marked_ratio_wine_probe.sh /tmp/sketchpad_marked_ratio_base.gsp`
reproduces the native point case; setting `SKETCHPAD_RATIO_CIRCLE=1` and
using the base from `tests/transform_marked_ratio_circle_replay.txt` probes
a circle instead. Native results are
`tests/fixtures/ratio_dilation_original.gsp` and
`tests/fixtures/ratio_dilation_circle_original.gsp`. In both probes, saving
after marking the center and ratio but before dilation is byte-identical to
the unmarked base. The marks are session state.

Both native images use object code 32 and a `0x07d2` reference list
`[source, center, ratio origin, ratio base, ratio target]`. The point image
has a point header; the circle image has a circle header. Neither has a
constraint payload. The port reads, evaluates, and writes both, and its
editor offers Mark Ratio and a marked-ratio option in Dilate. A negative
ratio moves a point through the center and uses its absolute value for a
circle's radius.

The port-generated point and circle files, before and after moving the ratio
target, are `tests/fixtures/ratio_dilation_generated.gsp`,
`tests/fixtures/ratio_dilation_moved_generated.gsp`,
`tests/fixtures/ratio_dilation_circle_generated.gsp`, and
`tests/fixtures/ratio_dilation_circle_moved_generated.gsp`. Sketchpad 5
opened and re-saved all four byte-identically. Segment dilation is documented
in `dev/segment_dilation_reflection_analysis.md`. Scale-factor values and
other dilatable object types remain outside this implementation.
