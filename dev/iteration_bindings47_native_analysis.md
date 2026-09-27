# Checkpoint 47 Native generic geometry image bindings

These probes edit only dev artifacts and fixtures. Production and regression
work belong to root and the audit agent. Copy and custom-tool support remain
outside scope. Ordinary geometry is compared numerically; raster identity is
not a requirement.

## Original source evidence

Eight actual corpus files opened and saved in Native with independent exact byte
comparison. Their complete GSP/PNG fixtures have the **original_** prefix:

| Original state | Bytes | Evidence |
| --- | ---: | --- |
| circle_rolling | 5404 | Circle source5, generic root6, binding9 |
| circle_segment | 8974 | CircleBySegment source5, root11, binding13; paired HTM Circle by radius(2,3) |
| polygon_scalar | 6218 | transformed Polygon20, scalar root21, binding22 |
| arc_spiral | 5326 | CenterArc17/21, roots18/23, bindings20/25; final/all levels |
| ray_axis | 17040 | Ray7, root15, binding16; paired HTM predecessors |
| line_reflection | 9798 | Line11, root26, binding27 |
| arc_square | 4134 | CenterArc9, root15, binding16; paired HTM predecessors |
| polygon_board | 12948 | transformed Polygon33, root35, binding36; paired HTM predecessors |

Fixtures are `iteration_binding47_original_STATE_native_verified.gsp/.png`.
The circle_segment original initially shared its filename with a fresh control;
the collision was corrected and the original rerun byte-exact at8974 bytes.
Its distinct original fixture and capture now retain the actual source.

Paired HTMs prove predecessor construction where available, not iteration
image output: the HTML export omits iteration/bindings. The circle_segment
HTML explicitly names Circle by radius(2,3), with source3 a typed Length
parameter. Arc_square HTML establishes midpoint dilation, angle formula360/t,
rotation/vector translation and square edges. Polygon_board HTML establishes
its parameter progression, ray/perpendicular and quadrilateral geometry.
Auditor dumps are in `/tmp/iteration_bindings47_{circle,ray,arc,polygon}.log`.
Actual raw binding groups are separately retained as
`iteration_binding47_STATE_ID_group.bin`; all use native code77 class15/sub65.
Arc-spinal binding25 includes an empty232e record, preserved in the original.

## Fresh public API controls

`iteration_bindings47_native_generate.rs` uses
`add_mapped_point_iteration_with_bindings`, without raw packet injection or
production guard bypass. Each generated input passes strict `verify_roundtrip`
before its Native open/save; each retained output equals its input bytes.
An initial Arc style-loss failure was fixed by root's shared binding stroke
classification before Arc and multi controls were verified.

Baseline source IDs:

- Seeds1/2/3 are A=(100,180), B=(160,180), C=(100,240).
- Fixed origin4=(80,80), vector target5=(200,100).
- Each seed is dilated by.75 about origin, then translated by target−origin;
  target images7/9/11, with carried intermediate dilations6/8/10.
- Segment12; Circle13(A,B); CircleBySegment14(A,12); Polygon15(A,B,C);
  Ray16(A,B); Line17(A,C); CenterArc18(A,B,C).
- Carried4..18, parent19, single binding20 or six bindings20..25.
  Original objects are hidden; depth3; generated bindings remain visible.

Independent expected generated seed positions are:

| Level | A | B | C |
| --- | --- | --- | --- |
| 1 | (215,175) | (260,175) | (215,220) |
| 2 | (301.25,171.25) | (335,171.25) | (301.25,205) |
| 3 | (365.9375,168.4375) | (391.25,168.4375) | (365.9375,193.75) |

All six single-type controls (`circle`, `circle_segment`, `polygon`, `ray`,
`line`, `arc`) and the six-binding `multi` document completed Native exact
open/save. The captures translate drawing coordinates by(+59,+69).

Circle and CircleBySegment produce the same three centers/radii:
(215,175)/45, (301.25,171.25)/33.75, (365.9375,168.4375)/25.3125.
Arc captures have those same bounds and the expected270-degree sweep, with
start at B and endpoint at C. Polygon captures show three shrinking right
triangles. Rays start at A and continue right; Lines through A/C remain
unbounded vertical lines at the three A.x positions.

Independent ImageMagick color-component extraction uses only Native PNGs;
`iteration_bindings47_native_pixels.py` produced
`/tmp/iteration_binding47_pixels.log`. Observed bounds include stroke width:

| State | Capture bounds (x min..max, y min..max) |
| --- | --- |
| Circle / CircleBySegment | (228..320,197..290), (326..395,206..275), (399..451,210..264) |
| Polygon | (274..318,244..288), (360..393,240..273), (425..449,237..262) |
| Arc | (228..320,197..289), (326..395,206..274), (399..451,210..263) |

These are geometric extent/ordering evidence within stroke tolerance, not
pixel-identical renderer assertions. The second/third transformed circles
have touching stroke components, so component count alone does not establish
circle count for that particular capture.

## Source edits, derived seeds, transforms and carried mask

`circle_edited` changes B to(180,200). Its centers follow the same A sequence;
radii become sqrt(6800)*.75^level. Native exact controls `circle_final` and
`arc_final` display only the last geometry; their one component exactly matches
the final component of each corresponding all-level capture.

`transformed_circle` and `transformed_polygon` add source19 as a.5 dilation
of Circle13/Polygon15 about origin4, parent20, binding21. Both opened/saved
Native exactly at3356 bytes. Their shapes shrink and move per the shared
transformation, proving compound geometry bindings consume the live source.

`frozen_circle` adds external Midpoint19 of Segment12, omits it from carried,
and carries Circle20(center19,B2), parent21, binding22. Native retains center
(130,180) while B moves. Its three radii are distances from that fixed center
to the B values above. This is direct evidence for the existing external-value
freeze rule; reevaluating every reachable ancestor would disagree.

Derived controls keep the same geometric initial A but represent it as
SegmentMidpoint6 of FreeL1=(80,180), FreeR2=(120,180), Segment5. Other seeds
are B3/C4, origin7, vector target8, targets10/12/14. Segment15, Circle16,
CircleBySegment17, Polygon18, Ray19, Line20, Arc21; parent22; carried7..21;
seeds[6,3,4]. `derived_circle` reproduces the baseline Native rings;
`derived_circle_edited` changes FreeL1 to(100,180), so A=(110,180), and changes
circle centers/radii accordingly. `derived_multi` carries all six image types.
All three controls completed exact Native open/save; the edited source retains
its midpoint construction rather than being replaced by a Free point.

## Undefined image and numeric depth

`undefined_circle` adds Scalar t19=0, next20=t+1, ratio21=
1/((t−2)*(t−2)), transformed Circle22 from Circle13 about origin4 with that
ratio. Seeds[1,2,3,19], targets[7,9,11,20], parent23, binding24. Ratio is1 at
levels1/3 and undefined at level2. Native exact3926-byte output shows only the
first and third circle components. A missing carried image therefore preserves
its valid prefix and subsequent valid image; it does not abort the whole root.

`polygon_numeric` has typed Scalar depth19=3, parent20, binding21; Native
exact3466-byte file shows three triangles. `polygon_numeric_edited` uses
n=2 and also changes B to(180,200), following the generator's explicit edited
state, and shows two changed triangles. Both use the generic numeric-depth
Polygon binding packet, rather than a Point binding template.

## Typed numeric radius: evidence status

The circle_segment original's source5 is native Circle by radius(2,3), where
source3 is a typed Length parameter. Root's shared CircleBySegment radius
adapter now accepts Segment geometry or typed Length, without a new product
Kind. Paired HTM and original exact Native opening establish the construction
roles. Numeric radius visual-policy checks use
`iteration_bindings47_native_numeric_radius.rs`.

The initial minimal mapped controls used a legacy PolarTranslatedPoint branch.
Their1/2/-1/0cm files opened/saved exactly, but captures showed no generated
geometry; the positive case remains unexplained. Those files prove wire
acceptance only, and are **not** positive radius or nonpositive policy evidence.
Explicit idle/paint waits did not change that observation. The analysis does
not assume an offscreen target or infer negative/zero policy from blank images.

The visible-source controls and vector-mapped positive control completed their
save/cmp steps (`/tmp/iteration_binding47_radius_base_native.log`). Root also
independently compared all five input/saved pairs: base1/2cm are2196bytes,
negative2110bytes, zero2108bytes, and vector-mapped1cm2568bytes; all are exact.
Painted captures show source1cm radius about38pixels, source2cm about76pixels,
and three vector-mapped1cm rings with centers20pixels apart. The negative
source capture has normal UI and its center but no circle. The zero after_setup
capture is entirely black and supplies no rendering-policy evidence. Attempting to
hide every object in the original multi-page file still failed strict state
validation before Native; no isolated-original proof is claimed from that
failed attempt. The repaired unedited original remains byte-exact.

## Reproduction

Build the current library and compile the Native generator with rustc,
`-L dependency=target/debug/deps --extern
sketchpad_rs=target/debug/libsketchpad_rs.rlib`. The shell runner
`iteration_bindings47_native_controls.sh STATE...` uses the shared Wine flock,
WOW64 Wine11, Xvfb, xdotool and ImageMagick, prefix
`/home/w/.cache/sketchpad_rs_wine`. It performs an independent byte comparison
before retaining GSP/PNG fixtures. Original-source extraction and its dedicated
runner use distinct original_ names. No tool or clipboard protocol was added.

Final process status: base/vector queue81556 and legacy repaint queue10882 both
terminated with exit0. No Native queue from this checkpoint remains running.
