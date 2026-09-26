# Fixed rotation and dilation chains for rays and lines

The native-proven fixed-transform source family is exactly primitive `Ray`/`Line`, `RotatedRay`/`RotatedLine`, and `ScaledRay`/`ScaledLine`. `Kind::fixed_transform_straight_mode()` preserves the distinction between ray and infinite line. None of these new variants is classified as a finite segment. Bisectors, parallels, perpendiculars and axes are excluded from this fixed-transform source guard. Marked-angle, marked-ratio and reflected ray/line constructions remain separate unimplemented families. Primitive numeric-angle and numeric-ratio images are documented in [numeric_angle_straights_analysis.md](numeric_angle_straights_analysis.md) and [numeric_ratio_straights_analysis.md](numeric_ratio_straights_analysis.md); these numeric images remain excluded as sources for fixed chains.

## Authentic native construction

The four `tests/fixtures/fixed_straight_chain_{rotation,dilation}_{ray,line}_original.gsp` documents begin with center 1 = (130,150), endpoints 2 = (170,190), 3 = (210,190), primitive source 4 and transformed source 5. Source 5 is a 45-degree rotation or ratio 2 dilation. Its stroke was changed in the writer to magenta, Thick, Dashed, independently of native drawing defaults.

In original Sketchpad, the center was marked and source 5 selected. The fixed Rotate dialog created image 6 at 90 degrees; reselecting the same source 5 and using the fixed Dilate dialog created image 7 with numerator -2 and denominator 1. This independently proves all eight source/target combinations, rather than relying only on existing recursive geometry.

Every image retains class 2 / subtype 13 for rays or class 2 / subtype 12 for lines. Both reference payloads are `020000000500000001000000`: source 5 then center 1. The rotation constraint remains the 36-byte code 27 payload with sine/cosine caches and angle 90. The dilation constraint remains the 12-byte code 30 payload, here `1e00000000000000000000c0` for ratio -2. Native header bytes 18..20 vary among generated objects; imported objects retain these opaque bytes unchanged.

The native ray screenshots show only the direction starting at the first transformed control point. The line screenshots span both viewport directions. The two rotation-source originals contain 2380 bytes each; the two dilation-source originals contain 2356 bytes each.

## Stroke inheritance

Native copies source color, width and pattern at creation. Source 5 and both new images have header style/color bytes 10..15=`1301ff00ff00`, representing Thick/Dashed magenta. The new images do not use the default navy Medium/Solid style. The two `fixed_straight_chain_primitive_{ray,line}_style_native_verified.gsp` controls independently verify the same behavior for primitive source 4: both targets 6 and 7 inherit that source's magenta Thick/Dashed stroke, while the other existing image 5 retains its own style.

`Document::append_object` now snapshots the source visual style for the four explicit fixed rotation/dilation ray/line target kinds, with the normal default style as fallback. This change does not extend style inheritance to other transformation families.

## Negative and zero parent geometry

The four `fixed_straight_chain_dilation_{ray,line}_{negative,zero}_control_native_verified.gsp` controls vary only source 5's existing factor and visibility, while preserving both child definitions. Source 5 factor -2 yields child 6 rotation endpoints (50,230),(50,310), and child 7 dilation endpoints (290,310),(450,310). Their visible extents demonstrate the negative parent's ordered direction before applying the child transformation.

Source 5 factor 0 makes both descendants undefined. Native saved all four 2356-byte controls unchanged. In the negative controls only child 6 and child 7 were visible and both strokes appeared; with factor 0 the corresponding screenshots were blank. The bounded zero-ancestor helper allows these valid undefined chains to save and copy without turning them into collapsed segments. It follows only the four fixed ray/line variants, detects exact factor 0, and guards against cycles. Other causes of undefined authored geometry retain the existing validation boundary.

## Regression and writer checks

`fixed_straight_transform_chains_preserve_native_modes_styles_and_zero_ancestors` verifies native preservation, all eight kinds, inherited styles, independent affine geometry before and after source/center edits, clipboard offsets, deletion dependencies, zero-chain clipboard/save, negative controls and Bisector rejection. `fixed_ray_and_line_chain_commands_inherit_source_strokes_and_keep_marked_exclusions` verifies both editor commands on all four source kinds, source style copying, and the marked-transform boundary.

Fresh writer and moved-source native results are retained alongside the original controls and compared in the same core regression after successful native open/save and exact byte comparison. The moved inputs are center 1 = (150,170), source 2 = (180,220), source 3 = (260,200).

All eight writer checks succeeded in original Sketchpad with exact byte preservation and visible geometry inspection. The two fresh rotation-source files contain 2336 bytes each and their moved versions contain 2380 bytes each. The two fresh dilation-source files contain 2312 bytes each and their moved versions contain 2356 bytes each. The fixtures use `fixed_straight_chain_{rotation,dilation}_{ray,line}_{generated,edited}_native_verified.gsp`. The core regression compares the freshly authored and edited document bytes directly against every retained golden. `tests/transform_fixed_straight_chains_replay.txt` reproduces the four fresh constructions, including their inherited source strokes.
