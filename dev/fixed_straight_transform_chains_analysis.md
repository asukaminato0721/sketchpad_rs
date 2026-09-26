# Fixed rotation and dilation chains for rays and lines

The native-proven fixed-transform source family is exactly primitive `Ray`/`Line`, `RotatedRay`/`RotatedLine`, and `ScaledRay`/`ScaledLine`. `Kind::fixed_transform_straight_mode()` preserves the distinction between ray and infinite line. None of these new variants is classified as a finite segment. Bisectors, parallels, perpendiculars and axes are excluded from this fixed-transform source guard. Marked-angle, calculated-angle, marked-ratio, calculated-ratio and reflected ray/line source constructions remain separate unimplemented families.

## Authentic native construction

The four `tests/fixtures/fixed_straight_chain_{rotation,dilation}_{ray,line}_original.gsp` documents begin with center1=(130,150), endpoints2=(170,190),3=(210,190), primitive source4 and transformed source5. Source5 is a45-degree rotation or ratio2 dilation. Its stroke was changed in the writer to magenta, Thick, Dashed, independently of native drawing defaults.

In original Sketchpad, the center was marked and source5 selected. The fixed Rotate dialog created image6 at90 degrees; reselecting the same source5 and using the fixed Dilate dialog created image7 with numerator-2 and denominator1. This independently proves all eight source/target combinations, rather than relying only on existing recursive geometry.

Every image retains class2/subtype13 for rays or class2/subtype12 for lines. Both reference payloads are `020000000500000001000000`: source5 then center1. The rotation constraint remains the36-byte code27 payload with sine/cosine caches and angle90. The dilation constraint remains the12-byte code30 payload, here `1e00000000000000000000c0` for ratio-2. Native header bytes18..20 vary among generated objects; imported objects retain these opaque bytes unchanged.

The native ray screenshots show only the direction starting at the first transformed control point. The line screenshots span both viewport directions. The two rotation-source originals contain2380 bytes each; the two dilation-source originals contain2356 bytes each.

## Stroke inheritance

Native copies source color, width and pattern at creation. Source5 and both new images have header style/color bytes10..15=`1301ff00ff00`, representing Thick/Dashed magenta. The new images do not use the default navy Medium/Solid style. The two `fixed_straight_chain_primitive_{ray,line}_style_native_verified.gsp` controls independently verify the same behavior for primitive source4: both targets6 and7 inherit that source's magenta Thick/Dashed stroke, while the other existing image5 retains its own style.

`Document::append_object` now snapshots the source visual style for the four explicit fixed rotation/dilation ray/line target kinds, with the normal default style as fallback. This change does not extend style inheritance to other transformation families.

## Negative and zero parent geometry

The four `fixed_straight_chain_dilation_{ray,line}_{negative,zero}_control_native_verified.gsp` controls vary only source5's existing factor and visibility, while preserving both child definitions. Source5 factor-2 yields child6 rotation endpoints(50,230),(50,310), and child7 dilation endpoints(290,310),(450,310). Their visible extents demonstrate the negative parent's ordered direction before applying the child transformation.

Source5 factor0 makes both descendants undefined. Native saved all four2356-byte controls unchanged. In the negative controls only child6 and child7 were visible and both strokes appeared; with factor0 the corresponding screenshots were blank. The bounded zero-ancestor helper allows these valid undefined chains to save and copy without turning them into collapsed segments. It follows only the four fixed ray/line variants, detects exact factor0, and guards against cycles. Other causes of undefined authored geometry retain the existing validation boundary.

## Regression and writer checks

`fixed_straight_transform_chains_preserve_native_modes_styles_and_zero_ancestors` verifies native preservation, all eight kinds, inherited styles, independent affine geometry before and after source/center edits, clipboard offsets, deletion dependencies, zero-chain clipboard/save, negative controls and Bisector rejection. `fixed_ray_and_line_chain_commands_inherit_source_strokes_and_keep_marked_exclusions` verifies both editor commands on all four source kinds, source style copying, and the marked-transform boundary.

Fresh writer and moved-source native results are retained alongside the original controls and compared in the same core regression after successful native open/save and exact byte comparison. The moved inputs are center1=(150,170), source2=(180,220), source3=(260,200).
