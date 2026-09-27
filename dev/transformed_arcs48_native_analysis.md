# Analytic arc transforms, checkpoint 48

This evidence work owns dev tools and fixtures only. It follows the shared
`ElementType::Arc` approach: preserve center, radius, start angle, signed sweep
and orientation while reusing existing transform expressions. Copying and
custom tools are deferred; raster equality is not required.

## Original Native wire evidence

The probe `transformed_arcs48_native_probe.rs` reads the original bytes with
current `Document`, requires byte-exact Rust encoding, and retains complete
07d0..07d7 groups. Originals were opened and saved by actual Native Sketchpad
under the shared Wine lock, followed by an independent `cmp`. Native queue
31859 finished exit 0. The rolling original completed its save/cmp step before
its subsequent visibility-only probe failed startup detection.

| Fixture state | Original corpus file | Objects retained | Exact Native bytes |
| --- | --- | --- | ---: |
| rolling | 周维波作品/半圆在地面的上的滚动（烟台飞狐） | 21,33,66,83 | 16016 |
| rotation_reflection | 侯仰顺作品/化圆为方(蚂蚁制作) | 41,42,43 | 9676 |
| dilation | 方小庆作品/圆台展开(inRm) | 25,27 | 12732 |
| cartesian | 况永胜作品/正方体的展开（3D效果） | 33,40,65,66 | 34196 |

Retained files are `tests/fixtures/transformed_arc48_STATE_native_verified.gsp`
and `.png`; complete groups are `transformed_arc48_STATE_ID_group.bin`.
Logs are `/tmp/transformed_arc48_{rolling,other}_native.log` and each state's
runtime log. The original broad audit is in `transformed_curves45_analysis.md`;
its counts are historical and are not a refreshed recovery prediction.

All selected arcs use class12/subtype67. Original33 and83 are code16 Vector
Translation. Both have only header28, references16 and empty07d7, with no
operation-specific literal record. Source21/66 are literal CenterArc. Object83
belongs to page1: raw references [21,1,31] resolve to global[66,46,76].
Original42 is code27 fixed rotation, refs[41,16], auxiliary07d3 length36;
43 is code34 reflection, refs[41,26], no auxiliary. Object27 is code32
point-ratio dilation, refs[25,13,13,5,11], no auxiliary. The historical fixture state named cartesian actually contains objects65/66,
which are code21 FixedPolar, source CenterArc40 / ThreePointArc33, refs[source],
auxiliary07d3 length60; object65 additionally retains empty232e. These are
construction identities, not evidence that every arbitrary affine matrix
maps a circle to an analytic circle.

## Independent rolling arc geometry

The paired HTM exists but describes only the second page's predecessor points,
rotations, measures, calculations and straight geometry. It omits the arc and
its transformed image, so it establishes predecessors rather than transformed
arc semantics. Native complete original screenshots contain overlapping fills.

The probe reads source point coordinates and computes center/radius/start/sweep
independently of transformed arc evaluation. Coordinates below are normalized
Document coordinates, not assumed screenshot pixels:

| Object | Source center | Vector | Result center | Radius | Result start / midpoint / end |
| --- | --- | --- | --- | ---: | --- |
| 33 | (124,398) | 1→31 ≈(0,0) | (124,398) | 100 | (224,398) / (124,298) / (24,398) |
| 83 | (235,335) | 46→76=(0,-86) | (235,249) | 86 | (235,335) / (321,249) / (235,163) |

Source21 starts at angle≈0 and sweeps π counterclockwise in mathematical
coordinates (upper semicircle in downward screen y). Source66 starts at
−π/2 and sweeps π (right semicircle). Vector translation preserves both radius
and traversal direction. Analytic log: `/tmp/transformed_arc48_rolling_analytic.log`.

## Visibility controls and evidence limits

Visibility-only controls alter only native hidden flags; their decode and
Rust encode remain exact. First-page variants additionally change active_page
through the public Document field and require verify_roundtrip. Initial
isolated33 attempts failed before Native window detection (sessions6742/74344,
exit1). No image, Native save, or geometry acceptance is inferred from these
attempts. The isolated83/page0 retry also failed startup detection (session1266, exit1).
All failed probes stopped before save; their bytes pass Rust checks but are not
Native-verified fixtures. Fresh controls provide independent geometry evidence.

## Fresh public API controls

`transformed_arcs48_native_generate.rs` prepares all14 existing Native transform
flavors plus vector→reflection→negative-dilation chains. Source13 is CenterArc
(center1=(100,180),start2=(140,180),end3=(100,140)): radius40 and90° upper
quarter. Transform18 uses the shared TransformDefinition and ElementType::Arc;
no new per-operation Kind is introduced. Length expressions14/15 are1/2cm,
angle16 is30°, named Scalar Parameter q17 is−.5. Chain19 reflects image18 across vertical mirror9
(x=220), and20 dilates image19 by−.5 about4=(200,200). PointOnArc, ArcAngle
and ArcLength depend on the final analytic host. Source edit moves2 to(160,180),
changing source radius to60. All17 authored API states pass strict verify_roundtrip, decode after save, and
byte-exact Rust re-encoding after the common stroke-style guard was corrected.
Numeric zero omits constrained point/measures, saves/reopens the negative
baseline and sets q17=0 through the public setter; the resulting analytic arc
is undefined. Native queue21674 finished exit0, covering all14 flavors plus chain/edit and
zero without leaves. Extra queue83695 finished exit0 for zero with leaves and
three additional primitive-source controls. Every save passed independent cmp;
there are21 Native-verified fresh controls and four exact complete originals.
No Native queue remains live.

Independent expectations use `transformed_arcs48_native_expected.py`, which
constructs explicit translation, rotation, reflection and signed dilation
matrices without calling production arc/transform evaluators. Pixel checker
`transformed_arcs48_native_pixels.py` compares analytic samples and the visible
red PointOnArc at t=.25 against Native captures using offset(+59,+69).
First FixedCartesian constrained point is within0.270 pixels of its independent
expectation. Samples at t=0,.5,.75,1 are within1.425 pixels of green strokes.
The red point occludes the green stroke near t=.25; green distance there is
not used as a direction check. Curve bounds and red constrained point together
check analytic geometry and traversal direction rather than raster equality.

## Reflection, signed dilation and zero evidence

Native chain and source-edited chain each save exactly3900 bytes. Independent
chain20 center is(160,200), radius20, start(180,200), endpoint(160,220), and
PointOnArc(t=.25)=(178.47759065022575,207.6536686473018). Native red point is
within0.590px of that independently composed matrix result. Edited source
radius60 gives final radius30 and point(187.7163859753386,211.4805029709527),
with Native red-point distance0.558px. Endpoints and interior green samples
are within1px, excluding the sample occluded by the red point. This checks
traversal parameter direction through reflection followed by negative dilation,
which cannot be established from the green curve alone.

Numericzero without dependent leaves saves exactly3446 bytes. Its fully painted
Native UI shows no green arc or red constrained point. A second control retains
PointOnArc19, ArcAngle20 and ArcLength21 from the valid negative-ratio API
baseline. `transformed_arcs48_native_zero_leaves.rs` replaces only the existing
q17 typed ParameterGroup with its public group setter's zero encoding. It does
not change dependent definitions or transform records. This dev-only payload
control passes the current repaired strict validator, saves Native-exactly3746
bytes, and has a normal painted blank canvas. The point and arc disappear
without deleting their constructions. Numeric measurement definitions are
retained byte-exactly; their hidden captions are not a Native displayed-value
proof. Port evaluation returns undefined for all three dependent values.

Original isolated visibility probes remain unproven; their startup failures
are explicitly separate from the positive fresh API evidence.

## Other primitive arc sources and linear degeneration

Extra Native queue83695 finished exit0. Curved ThreePointArc13(2,1,3), with
points(140,180),(100,180),(100,140), has circumcenter(120,160), radius√800
and a180° sweep passing through the middle point. Its vector image18 has
center(180,180), start(200,200), midpoint(160,200), endpoint(160,160), and
PointOnArc(t=.25)=(180,208.2842712474619). Native saves exactly3752 bytes;
the visible red point is within0.285px of this independent three-point-circle
calculation. Four green samples outside the red-point occlusion are within1px.
This verifies the ThreePointArc source role independently of CenterArc.

ArcOnCircle uses a separate public API graph: center1=(100,180), radiuspoint2,
Circle3, PointOnCircle4(3,1,0) /5(3,0,1), ArcOnCircle6(3,4,5), vector endpoints
7/8, transformed image9, PointOnArc10, ArcAngle11 and ArcLength12. The
PointOnCircle coordinates are the existing complex/upward-y convention.
Free endpoint points do not meet the API ArcOnCircle source contract, so the
first extra-source generator attempt stopped before Native and was corrected
to constrained circle points. The final control saves Native-exactly2716 bytes
and reproduces the vector-translated90° quarter geometry.

The collinear control retains ThreePointArc13 and changes sourcepoint3 to
(60,180), producing a linear arc with image endpoints(200,200),(120,200).
It intentionally omits dependent point and measurements. Native saves
exactly3452 bytes and renders a finite horizontal green segment from
capture(179,269) to(258,269), matching the expected endpoints after view offset
within1px. Analytic linear degeneration therefore retains an Arc source's
construction identity while using straight geometry for its rendering adapter;
this does not classify the object as a general line/segment construction.


## Completed fresh Native fixture manifest

All entries are GSP/PNG fixture pairs with prefix `transformed_arc48_api_` and
suffix `_native_verified`. Complete Native logs are
`/tmp/transformed_arc48_api_native.log` and
`/tmp/transformed_arc48_extra_sources_native.log`.

| API state | Exact Native bytes |
| --- | ---: |
| chain_edited | 3900 |
| chain | 3900 |
| fixed_cartesian | 3804 |
| fixed_dilation | 3768 |
| fixed_horizontal | 3784 |
| fixed_polar | 3812 |
| fixed_rotation | 3792 |
| fixed_vertical | 3784 |
| marked_cartesian | 3752 |
| marked_polar | 3792 |
| numeric_dilation | 3752 |
| numeric_dilation_zero_leaves | 3746 |
| numeric_dilation_zero | 3446 |
| numeric_rotation | 3752 |
| point_angle_rotation | 3760 |
| point_ratio_dilation | 3760 |
| reflection | 3748 |
| vector | 3752 |
| vector_on_circle | 2716 |
| vector_threepoint_collinear | 3452 |
| vector_threepoint | 3752 |

The final independent pixel batch46413 finished exit0. All18 curved controls
(the14 flavors, two chains and two additional curved source kinds) have visible
red t=.25 points within0.590px of the independent analytic targets. Their green
samples at t=0,.5,.75,1 are within1.425px of the expected curve, including
reflection and signed dilation. The collinear control's two endpoints are
within1px. Both zero controls have zero green/red pixels on normal painted
canvases. Full results are `/tmp/transformed_arc48_all_pixels.log`. These bounds
are evidence tolerances for rendered samples, not pixel-identical requirements.

Shell/Python syntax checks and whitespace checks passed for the new dev tools.
No production code or test source was changed by this Native evidence work.
