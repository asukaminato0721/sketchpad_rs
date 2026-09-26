# Numeric-angle rotation of primitive rays and lines

Native rotation marked by a degree calculation creates code 29 with references
`[source, center, angle]`, without a constraint record. The target has the modern
28-byte class-2 header: subtype 13 for a ray, subtype 12 for an infinite line.
The source remains a primitive ray (64) or line (63), rather than a finite segment.

`numeric_angle_ray_original.gsp` and `numeric_angle_line_original.gsp` were created
in Sketchpad from a horizontal primitive, a free center, scalar `n = 4`, and the
calculation `360°/n`. Both contain eight objects. Native output inherits the
source's magenta, Thick, Dashed stroke. The visible ray begins at its rotated
first endpoint and extends upward; the line extends in both directions. The
reader preserves each original byte for byte.

The document and editor now support those two primitive source families with an
earlier live point center and an earlier live Degrees numeric value. Explicit
`CalculatedAngleRotatedRay` and `CalculatedAngleRotatedLine` kinds keep the native
mode through geometry, styling, rendering, serialization, clipboard remapping,
and dependent deletion. A scalar marked value is rejected as an angle. A saved
formula becoming undefined suppresses its image while retaining a savable graph;
restoring the input restores the image.

Rotations marked by three points remain a separate proof target. Numeric and fixed transforms of these images are now supported by the linked mixed-chain audit.
Numeric-ratio dilation of primitive rays/lines is now documented in
[numeric_ratio_straights_analysis.md](numeric_ratio_straights_analysis.md). The bounded transform source helper now includes the four proven numeric Ray/Line kinds.

An audit of sibling GSP4 scene records, excluding embedded tool scopes
`03ea..03eb`, found eleven code-29 rays sourced by primitive rays and three
code-29 lines sourced by primitive lines. For example,
`../gsp-rs/tests/Samples/未分类档/圆内点的弹性束缚 (3).gsp` contains primitive ray 8
and numeric-angle images 14 and 18; its paired HTM exports only the earlier base
construction, so it does not itself define these later images. The direct native
controls establish the missing records. The same audit found six numeric-ratio
ray images (33) and further numeric-angle line sources from parallel lines or
numeric rotations; those families are outside this slice.

The eight writer controls are retained as
`numeric_angle_{ray,line}_{generated,edited,negative,undefined}_native_verified.gsp`.
Each native process completed successfully and a separate comparison confirmed
identical saved bytes. Fresh and edited files are 2668 bytes; negative-input files
are 2670 bytes. Edited controls change `n` to 8 (45°), move the center to (140,160),
and move the source endpoints to (180,210) and (240,220). The images keep their
one-sided or two-sided extents. Negative controls use `n = -4` (-90°), producing
ordered endpoints (90,190) and (90,230). At `n = 0`, native displays an infinite
angle and draws only the unrotated primitive; the model suppresses the image and
preserves the graph. Regressions compare authored fresh, moved, negative, and
undefined outputs against these exact native-verified bytes, test restoration
before the first save and after reopening, and check SVG clipping and picking on
both sides of the ray origin.

Numeric and fixed transformed Ray/Line sources are subsequently supported by the bounded [mixed-transform chain proof](mixed_straight_transform_chains_analysis.md). Other straight-source families and plotted-point host guards remain separate.
