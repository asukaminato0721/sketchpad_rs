# Generic iteration binding audit (checkpoint 47, before extension)

Read-only drawing audit compiled against the current `target/debug/libsketchpad_rs.rlib`.
Copying and custom-tool prototypes are excluded. Run:

```sh
rustc --edition=2024 dev/iteration_bindings47_audit.rs --extern sketchpad_rs=target/debug/libsketchpad_rs.rlib -L dependency=target/debug/deps -o /tmp/iteration_bindings47_audit
/tmp/iteration_bindings47_audit > /tmp/iteration_bindings47_audit.log
```

The full scan terminated successfully and found 6,212 currently unsupported code77 drawing bindings. All have the same class15/subtype65 header and `[source,parent]` references. Of these, **389 bindings in 102 files** already reference decoded `MappedPointIteration` parents, and their sources are explicitly in each parent's carried list:

| Output | Bindings | Details |
| --- | ---: | --- |
| Polygon | 316 | 275 ordinary +41 generic transformed polygons |
| Circle | 32 | 14 Circle +11 CircleBySegment +7 generic transformed circles |
| Arc | 35 | 34 CenterArc +1 ThreePointArc |
| Ray/Line | 6 | 4 Ray +2 Line |

These are immediate output-adapter candidates, not verified final runtime recovery. The remaining bindings mostly have unsupported parents:1,694 Segment,908 transformed sources,2,879 unsupported sources. Supporting typed numeric seeds alone therefore cannot recover the broad remainder without prerequisite work.

The common carried-DAG guard already admits circles, polygons, straight objects and arcs. The binding decoder currently narrows generic parents to point/segment sources. `mapped_iteration_images` already freezes external constructions, substitutes simultaneous seeds, reevaluates the shared DAG, hides undefined images independently, and implements all/final level selection. Add shape readers to that helper, preserving the source Kind and styles. No transform-by-shape or iteration-by-shape enum variants are needed.

Preserve strict same-page/backward references, class15/subtype65, two references, decoded mapped parent, explicitly carried source, and source type compatibility. Broader support should use `Kind::is_polygon/is_circle`, straight-mode classification and the known arc predicate. The current template selector routes generic numeric-depth polygon sources to a point header because depth logic runs before polygon output classification; classify output first. Original binding payloads retain `[77,0,..,15,0,65,0]`, with source appearance bytes10..18 copied into their own style. No arc-specific record is present in original generic bindings below.

## Original cases and source evidence

`iteration_bindings47_dump.rs` prints complete raw groups for supplied global IDs. Its output is diagnostic only. Original paired HTM establishes base constructions; it often omits iterations and does **not** independently prove iteration output semantics.

- `个人专栏/方小庆作品/勾股螺旋(inRm).gsp`: binding13 `[5,11]`; source5 `CircleBySegment(2,3)`; root11 pointseed2 ->8, carried `[6,5,7,8,9,10]`, Fixed5. Paired HTM defines Circle by radius(2,3), green/dashed. Full raw dump `/tmp/iteration_bindings47_circle.log`.
- `个人专栏/贺基旭作品/数轴（hjx4882）.gsp`: binding16 `[7,15]`; source7 Ray(1,2); root15 seed1 ->2, carried `[2,3,4,5,6,7,8,9,10,11,12,13,14]`, Value10. Paired HTM explicitly defines the hidden base Ray. Dump `/tmp/iteration_bindings47_ray.log`.
- `个人专栏/贺基旭作品/正方形迭代.gsp`: binding16 `[9,15]`; source9 CenterArc(3,2,7); root15 seed2 ->7, carried `[3,7,8,9,10,11,12,13,14]`, Value5. Paired HTM exists but does not export the arc iteration. Dump `/tmp/iteration_bindings47_arc.log`.
- `个人专栏/侯仰顺作品/杨老师问题-迭代（蚂蚁）.gsp`: binding22 `[20,21]`; transformed Polygon10, marked Cartesian values19/18; root21 Scalarseed15 ->16, carried `[16,17,18,19,20]`, Value14. No paired HTM; Native verification required. Dump `/tmp/iteration_bindings47_polygon.log`.
- `个人专栏/方益初作品/棋盘（方块＋颜色）.gsp`: transformed polygon binding36 `[33,35]`, paired HTM available.
- `个人专栏/向忠作品/黄金三角螺线.gsp`: arc bindings20 `[17,18]` final-only and25 `[21,23]` all-levels. Both roots substitute two points `[1,2]` -> `[2,5]`; first carries `[5,6,13,14,15,16,17]`, second `[5,6,13,14,15,21]`, depths10/22. No paired HTM; useful all/final Native distinction.
- `未分类档/两直线间光线反射 .gsp`: line binding27 `[11,26]`, root26 seeds `[5,6,7]` -> `[14,16,12]`, carried `[8,9,10,11,12,13,14,15,16,18,19,20,21,22,23,24,25]`. Second-page line binding58 `[43,56]` tests global/local reference remapping. No paired HTM.

For fresh Native controls, a derived point seed (Midpoint) mapped to a vector transform plus a carried CircleBySegment/Polygon/Ray/Line/CenterArc prevents canonicalization to existing specialized Point/Segment/Circle/Ray roots. Compare original/API bytes after Native save independently; verify generated images and style edits rather than inferring behavior from lossless import.

## Typed seed comparison

Recompiled the existing `single_seed_iteration_cohort_audit45.rs` against the same current library and ran with `ALL_SEEDS=1`; process terminated successfully. Results `/tmp/iteration_bindings47_roots.log`. One known carried-DAG root is blocked **solely** by typed seeds: `个人专栏/钮炳坤作品/系列正方形迭代作法（钮炳坤老师）.gsp`, root19, seeds `[8 Free,1 Length,2 Length]` -> `[14,2,3]`, depthValue4, carried `[3,12,11,13,14,16,18,17,10,15]`. A two-Degrees seed candidate additionally needs ParameterLocus support; other Length/Degrees seeds have unsupported predecessor constructions. Shape bindings are the wider immediate cohort. No candidate unsupported root using all currently supported seed/DAG categories remains in this audit.

## Focused regression coverage

`src/document/tests/iteration_bindings.rs` covers generic fixed/numeric-depth
circle, polygon, ray, line and arc outputs; derived midpoint seeds; live seed,
vector and numeric-depth edits; final-only levels; freezing omitted midpoint
hosts; source-kind stability; independent source/binding styles and exact undo;
noncarried-source and noniteration-parent rejection with raw-byte preservation.
Fresh Native API reconstruction compares nine baseline controls, three derived
seed controls and three transformed/frozen controls byte-for-byte. The retained
Native undefined-calculation control has two valid circle images (generations1
and3), independently confirming that an undefined middle output hides only that
image. Original Native families also check nonempty output and exact bytes.

The original `勾股螺旋(inRm)` exposed a shared circle prerequisite bug:
`CircleBySegment(2,3)` has a Length Parameter3, not a segment. Paired HTM declares
Circle by radius. The common radius reader now accepts defining segments or
known Length numeric values via the existing distance-to-pixels reader. A fresh
regression edits the typed Length parameter from1 to2 and checks its circle,
constrained point and all mapped circle images, preserves the original source
kind, strictly verifies serialization, and undoes to exact initial bytes.

An additional preexisting multipage validation defect was identified by keeping
the original strict verification gate: identity `remap_checked` on page1 object32
`ParameterCalculation(31, group)` rewrites its on-wire local source11 as global31.
The unchanged encoding is byte-exact, but expected Kind group bytes then disagree
with reparsed original bytes. Authoritative diagnostic:
`/tmp/iteration_bindings47_original_debug.log`, terminal test failure, first
identity difference object32/page1. The shared validator now remaps embedded
calculation/table references in page-local space while retaining global Kind IDs.
All 13 binding regressions pass, including every original family's strict
verification. The final scoped gates pass 859 checks and all 1,599 corpus
originals retain byte-exact round trips. Final scan recovers 390 slots (389 drawing
bindings and one prototype slot), leaving 54,072 unsupported.
