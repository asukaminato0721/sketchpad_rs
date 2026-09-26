# Numeric-ratio dilation of primitive rays and lines

Native Sketchpad dilation marked by a scalar calculation creates code 33 with
references `[source, center, ratio]`, without a constraint record. The target
uses the modern 28-byte class-2 header: subtype 13 for a ray, subtype 12 for an
infinite line. This is distinct from fixed dilation (30) and finite segments.

`numeric_ratio_ray_original.gsp` and `numeric_ratio_line_original.gsp` were
created in Sketchpad from primitive source 4, free center 1, scalar parameter
`n = 4` (5), and the scalar calculation `n/2` (7). Each has eight objects and is
2712 bytes. Native images inherit the source's magenta, Thick, Dashed stroke.
The reader preserves both originals exactly.

Independent byte candidates used the existing straight writer's native layout,
changing its operation code and third reference to the exact native code-33
record. Before expanding the decoder or authoring API, native Sketchpad opened
and saved all ten candidates byte-identically. The retained controls are
`numeric_ratio_{ray,line}_{generated,edited,negative,zero,undefined}_native_verified.gsp`.
Fresh, edited, zero, and undefined files are 2668 bytes; negative files are 2670.
The authored writer is regression-tested against those same bytes.

At ratio 2, the ordered endpoints are (210,230) and (290,230), with a ray extending
rightward and a line in both directions. Edited controls change `n` to 8, center
to (140,160), and source endpoints to (180,210) and (240,220); the images become
(300,360) and (540,400). At `n = -4`, ratio -2 produces (50,70) and (-30,70): the
ray extends leftward and the line remains infinite. At ratio zero, neither mode
draws an image. Separate reciprocal `2/n` controls at `n = 0` show native infinity
and likewise suppress the image. All of these records remain savable.

Explicit `CalculatedRatioScaledRay` and `CalculatedRatioScaledLine` kinds retain
the mode through geometry, stroke editing, SVG/canvas rendering, picking,
serialization, clipboard remapping, and dependent deletion. Source guards allow
primitive Ray/Line and the fixed/numeric transform images proven in the linked mixed-chain audit, with an earlier live point center and an earlier live
Scalar numeric source. Degrees are rejected as ratios. An initially undefined
ratio is rejected by the creation API; a previously created image remains savable
when its ratio becomes zero or undefined, including before the first save.
Restoring the input restores geometry. A defined zero ratio may author an image
that becomes visible when its numeric input changes.

Point-marked ratios (32) and reflected ray/line source families remain separate proof targets. The plotting/iteration host whitelists are not broadened by the mixed-transform proof. The sibling scene audit, excluding embedded tool
namespaces, found six primitive-ray code-33 images, including
`../gsp-rs/tests/Samples/个人专栏/常新德作品/多边形外角和.gsp`; the clean native line
control provides the corresponding line record evidence independently.

Numeric and fixed transformed Ray/Line sources are subsequently supported by the bounded [mixed-transform chain proof](mixed_straight_transform_chains_analysis.md). Other straight-source families and plotted-point host guards remain separate.
