# Corpus decoding coverage

Latest snapshot, 2026-09-30: **1,585/1,599** documents have all object kinds
decoded; **1,599/1,599** have all drawing kinds decoded. All **1,599** round trips
are byte-exact, with zero failures. There are **105** unsupported slots, all in
embedded-tool prototypes, and 14 documents still contain unsupported
objects. Embedded tools contain 5,356 slots across 103 definitions: 5,251
decoded, 105 undecoded. Isolated decoding produces 71 complete and 32 partial
tool views. The current tool-prototype mapped-element and nested-mapping,
named referenced-caption, polymorphic-polygon tool, external-placeholder mapped-iteration,
embedded attached-picture, final
geometric-given, distance-Parameter,
polymorphic Preimage, legacy Curve, angle-Parameter, legacy generic-numeric and distance custom-tool-given,
function custom-tool-given, and function-source tool-calculation,
coordinate-system, polygon, and scalar-Parameter custom-tool-given
recoveries plus tool-picture-resource extraction, circular-segment, and endpoint
arc-length
measurement, transformed-point Animation, segment-marker, legacy-angle-rotation,
legacy polar-coordinate, circular-segment transform, and axis-rotation recoveries
plus literal locus-point, degree-coordinate, legacy coordinate-readout,
sampled-path point-value, anchored referenced-label, mapped ParameterLocus
collection, mapped action-capture, and mapped iteration-value-table recoveries
plus general native snapshot-table, labeled/traced custom polar-translation,
general Plot Value, duplicate-carried, nested-capture mapped-iteration,
transient explicit-path locus-driver, mapped FunctionPlot binding, high-rate
plot, scalar-angle custom-translation, high-depth mapped-iteration, and final
referenced-caption, point-angle fixed-polar, sampled-curve direction-arrow, and
final Animation-edge recoveries are summarized at the end; the older detailed
snapshots remain for comparison.

The polymorphic-polygon tool pass completes the 120-object rolling-polygon
prototype. Strict class-4/subtype-15 code-16 and code-29 transforms may treat a
raw subtype-15 polygon tool input or transformed polygon as their source; the
ordinary typed transform model remains unchanged. Exact 28-byte internal
iteration labels `[1]` through `[9]` on codes 76, 77, and 89 are treated as
metadata rather than editable labels. Invocation specializes the polygon
header and style, preserves region ordering, and uses the observed bold
16-point `黑体` caption metric. Carried point values are cached while replaying
deep mappings so repeated endpoint calculations share a state-local result.
The pass restores two prototype roots and makes three isolated tools complete:
decoded slots rise **5,110→5,112**, unsupported falls **246→244**, all-kind
complete documents rise **1,575→1,576**, and isolated complete tools rise
**49→52**. A real invocation creates 118 outputs, reopens non-partially, and
round-trips exactly; Sketchpad 5 under Wine renders it and Ctrl+S preserves the
file SHA-256. See `tool_polymorphic_polygon_analysis.md`.

The external-placeholder mapped-iteration pass distinguishes tool placeholders
used as invariant predecessors from placeholders that participate in the
substitution. A polygon tool input and a tool point on a path are admitted only
when neither is a seed, branch, nor carried output. Invocation first
materializes both placeholders, after which the existing mapped evaluator
freezes their values at the external boundary. This restores three code-76 and
three code-89 iteration descriptors plus 31 dependent prototype objects:
unsupported falls **283→246** and decoded prototype slots rise
**5,073→5,110**. Document and isolated-tool completeness are unchanged because
the affected prototypes retain unrelated unsupported objects. All 1,599 source
files remain byte-exact. See `tool_mapped_placeholder_analysis.md`.

The point-attached-picture pass extends strict code-99 import from zero rotation
to any finite native rotation with positive finite horizontal and vertical
scales. Its affine keeps the PNG center on the live anchor. The only additional
record admitted is the exact hidden internal `[1]` metadata payload observed in
two files; it is preserved but is not presented as an editable object label.
The compiled scan restores four picture roots and one downstream object:
unsupported falls 3,735→3,730, all-kind complete rises 1,167→1,170, and
drawing-complete rises 1,196→1,199. No unsupported class-16/code-99 picture
profile remains on a drawing page; twelve code-99 prototype slots remain among
the unchanged tool coverage. All 1,599 originals remain byte-exact. See
`image_analysis.md`.

The preceding transformed-picture pass represents image operations in the shared typed
transform model rather than as vector-only special cases. A class-16/subtype-78
output must reference an earlier decoded picture, pass the ordinary live
operand typing for its native flavor, use only the expected transform records,
and repeat the source PNG dimensions in its `090c` cache. The source affine
placement is composed with vector, marked rectangular/polar, point-angle or
numeric rotation, fixed dilation, numeric dilation, or reflection matrices for
canvas, SVG, picking, and iteration. The compiled scan restores 37 picture
transform roots and 22 downstream objects: unsupported falls 3,794→3,735,
all-kind complete rises 1,157→1,167, and drawing-complete rises 1,186→1,196.
The remaining ready-looking code-29 image headers target cropped images, whose
clip geometry and non-duplicated cache are a separate format. Prototype
coverage is unchanged. See `image_analysis.md`.

The preceding vector-translation pass extends the existing live transform path to two
strictly observed code-16 families. Generic class-4/subtype-15 polygons are
accepted only when the source has the same native header. Class-16/subtype-78
pictures require a code-99 or code-100 source picture, two live point
endpoints, the exact four-record layout, and a duplicated PNG width/height
cache. Translated pictures reuse the source PNG and compose its live placement
with the vector displacement for canvas, SVG, picking, and iteration. The
compiled scan restores 47 drawing code-16 slots and 26 downstream slots:
unsupported falls 3,867→3,794, all-kind complete rises 1,149→1,157, and
drawing-complete rises 1,178→1,186. The remaining drawing code-16 count is 87;
prototype coverage is unchanged. See `vector_polygon_translation_analysis.md`
and `image_analysis.md`.

The locus pass admits three independently observed native variations without
weakening dependency typing: capture lists may omit parallel ancestry branches
or retain an independent branch from the target's validated closure,
geometric sweeps retain either observed presentation word, and the filled-region
target family includes disks, sectors, and circular segments. A native locus may
also impose its explicit stored path on a base `Free` point only during sampling;
the path must be independent of that driver, preventing a cyclic construction.
Point-output `Locus`, `ParameterLocus`, and `SampledMapped` objects are admitted
as bounded curve-collection targets and render by replaying each inner curve in
the outer driver scope. These changes restore 107 drawing code-35 loci and 47
downstream objects: unsupported slots fall 4,021→3,867, all-kind-complete
documents rise 1,125→1,149, and drawing-complete documents rise
1,153→1,178. The 612
unsupported prototype slots are unchanged. See `locus_midpoint_analysis.md`.

The attached-picture pass strictly decodes every drawing code-85 two-point
rectangle and code-100 three-point parallelogram in the corpus: 136 picture
objects. All 39 remaining code-108 crop groups then resolve against their live
picture and polygon parents. Those objects and 115 downstream recoveries lower
unsupported slots 4,311→4,021. All-kind-complete documents rise
1,108→1,125 and drawing-complete documents rise 1,134→1,153. Placement is
resolved from live point parents for canvas, picking, SVG, and iteration;
resource-table mutation and fresh attachment authoring remain guarded. See
`image_analysis.md`.

The Presentation-action pass decodes the native Before Starting flags, simultaneous stop condition, and tenths-of-a-second elapsed/pause word. The separate native progress/state word is preserved without invented runtime semantics. The editor authors and runs the proved options. Against the immediate 4,479-slot baseline, the compiled scan restores 135 drawing code-62 buttons, 29 dependent drawing objects, and four prototype buttons. Drawing code 62 falls 199→64; all-kind-complete documents rise 1,089→1,108 and drawing-complete documents rise 1,114→1,134. The remaining drawing code-62 objects are other action families, not rejected Presentation records.

The animation-action audit admits corpus-observed low mode tags without assigning them invented semantics: point blocks use tags 0/1 and parameter blocks use 0/2/4. Editing direction, speed, Once Only, or parameter fields preserves the original tag. Runtime animation now follows `PointOnCurve` and the closed `PointOnSector` perimeter as well as the previously supported point paths. Strict target/settings matching retains prototype-only path points and rejects unsupported targets. Against the immediate 4,581-slot baseline, the compiled scan restores 94 drawing code-62 buttons, six code-73 descendants, and two prototype code-62 buttons. Drawing code 62 falls 293→199; all-kind-complete documents rise 1,043→1,089. These are measured recoveries, not claims about the ten remaining unrecognized settings layouts or other action families.

Checkpoint 95 snapshot on 2026-09-28, generated with `gsp_probe coverage ../gsp-rs/tests`.

Documents: 1599 total, 838 with all object kinds decoded, 761 with unsupported objects, 0 failed.

Byte-preserving round trips: 1599/1599; object slots: 285703.

Partial views: 1032/1599 (includes multi-page structural edit restrictions).

Drawing objects: 280347 slots; 853 documents with all drawing kinds decoded.

Embedded tool prototypes: 103 definitions in 65 documents; 5356 slots, 4548 decoded, 808 undecoded.

Isolated tool views: 34 complete, 69 partial, 0 failed, with separate object and font namespaces.

Unsupported object slots including tool prototypes: 9226 (8418 drawing slots and 808 prototype slots). Object decoding does not establish editing, interaction, or command parity. The release requirements remain in `COMMAND_PARITY.md`.

Embedded tool prototypes are decoded through isolated documents. Declared point placeholders receive neutral coordinates; declared path, finite-segment, and generic-numeric placeholders retain typed abstract inputs. Other unknown prototype constructions remain unsupported. The drawing and prototype histograms retain separate namespaces. Complete isolated decoding does not establish invocation or authoring support for every tool. This snapshot is not directly comparable to earlier drawing histograms that mixed tool prototypes with scene objects, or to snapshots that counted every prototype slot as undecoded.

The counts include missing variants and failures caused by unsupported parents.
Inspect complete groups and parent payloads before selecting a target.

| Native code | Unsupported objects | Documents | First example |
| --- | ---: | ---: | --- |
| 77 | 1371 | 191 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/化圆为方(蚂蚁制作).gsp` |
| 48 | 625 | 97 | `../gsp-rs/tests/Samples/个人专栏/况永胜作品/正方体的展开（3D效果）.gsp` |
| 0 | 545 | 177 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 73 | 413 | 176 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/09年潍坊17题答案（蚂蚁制作）.gsp` |
| 35 | 393 | 159 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/10福建宁德26题2(蚂蚁).gsp` |
| 16 | 323 | 94 | `../gsp-rs/tests/Samples/个人专栏/况永胜作品/正方体的展开（3D效果）.gsp` |
| 89 | 328 | 186 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/化圆为方(蚂蚁制作).gsp` |
| 90 | 303 | 104 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 27 | 297 | 68 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/关于分段函数的参数取值范围讨论一例.gsp` |
| 62 | 293 | 132 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/时钟(蚂蚁制作).gsp` |
| 29 | 235 | 63 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/n叶草系列迭代.gsp` |
| 102 | 231 | 53 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/六棱鼓.gsp` |
| 97 | 233 | 61 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/12T.gsp` |
| 26 | 143 | 33 | `../gsp-rs/tests/Samples/个人专栏/况永胜作品/正方体的展开（3D效果）.gsp` |
| 33 | 181 | 50 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/n叶草系列迭代.gsp` |
| 17 | 178 | 10 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/平面截圆柱面的展开图.gsp` |
| 75 | 137 | 51 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/三个参数控制颜色(蚂蚁).gsp` |
| 95 | 135 | 49 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/平面截圆柱的交线在侧面展开图中是什么曲线.gsp` |
| 24 | 132 | 18 | `../gsp-rs/tests/Samples/个人专栏/周维波作品/三角转子的滚动-雪山飞狐.gsp` |
| 98 | 130 | 46 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/关于分段函数的参数取值范围讨论一例.gsp` |
| 15 | 126 | 58 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/12T.gsp` |
| 34 | 114 | 33 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/时钟(蚂蚁制作).gsp` |
| 8 | 97 | 38 | `../gsp-rs/tests/Samples/个人专栏/况永胜作品/正方体的展开（3D效果）.gsp` |
| 100 | 106 | 28 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/计时器4.gsp` |
| 30 | 100 | 41 | `../gsp-rs/tests/Samples/个人专栏/况永胜作品/正方体的展开（3D效果）.gsp` |
| 28 | 81 | 32 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/关于分段函数的参数取值范围讨论一例.gsp` |
| 19 | 73 | 5 | `../gsp-rs/tests/Samples/个人专栏/庞坤生作品/动量守恒之摆球小车模型.gsp` |
| 88 | 70 | 37 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/圆环的迭代(5环旋转迭代).gsp` |
| 94 | 65 | 32 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/平面截圆柱的交线在侧面展开图中是什么曲线.gsp` |
| 21 | 49 | 15 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/平面截圆柱的交线在侧面展开图中是什么曲线.gsp` |
| 113 | 43 | 22 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/双曲线的判定实验.gsp` |
| 92 | 42 | 28 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/用迭代演示动态反比例函数作图（蚂蚁制作）.gsp` |
| 91 | 40 | 14 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/光的折射(蚂蚁制作).gsp` |
| 108 | 39 | 4 | `../gsp-rs/tests/Samples/个人专栏/周维波作品/2010年四川眉山中考数学试题最后一题.gsp` |
| 80 | 36 | 18 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/关于分段函数的参数取值范围讨论一例.gsp` |
| 37 | 37 | 20 | `../gsp-rs/tests/Samples/个人专栏/况永胜作品/正方体的展开（3D效果）.gsp` |
| 66 | 37 | 19 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/平面截圆柱的交线在侧面展开图中是什么曲线.gsp` |
| 71 | 36 | 20 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/六棱鼓.gsp` |
| 65 | 35 | 14 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/正n棱柱(圆柱)的表面展开.gsp` |
| 69 | 34 | 17 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/六棱鼓.gsp` |
| 101 | 27 | 9 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/六棱鼓.gsp` |
| 85 | 30 | 11 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/计时器3.gsp` |
| 121 | 28 | 8 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/椭圆、双曲线第一定义系列练习.gsp` |
| 72 | 27 | 20 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/关于分段函数的参数取值范围讨论一例.gsp` |
| 99 | 26 | 9 | `../gsp-rs/tests/Samples/个人专栏/周维波作品/不变面积（雪山飞狐）(津华园修改).gsp` |
| 32 | 23 | 13 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/圆环的迭代(5环旋转迭代).gsp` |
| 47 | 23 | 14 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/π日话π.gsp` |
| 52 | 22 | 7 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/平面截圆柱的交线在侧面展开图中是什么曲线.gsp` |
| 49 | 21 | 19 | `../gsp-rs/tests/Samples/个人专栏/周维波作品/2010年湖南衡阳数学中考.gsp` |
| 123 | 21 | 8 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/六棱鼓.gsp` |
| 23 | 20 | 7 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/双曲极限圆[三分线].gsp` |
| 12 | 18 | 8 | `../gsp-rs/tests/Samples/个人专栏/孟令岩作品/※圆锥，圆柱，圆台展开.gsp` |
| 82 | 18 | 7 | `../gsp-rs/tests/Samples/个人专栏/孙禄京作品/转盘.gsp` |
| 96 | 18 | 4 | `../gsp-rs/tests/Samples/个人专栏/唐家军作品/课时9笔记.gsp` |
| 76 | 17 | 12 | `../gsp-rs/tests/Samples/个人专栏/庞坤生作品/多普勒效应.gsp` |
| 120 | 17 | 12 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/n叶草系列迭代.gsp` |
| 41 | 16 | 11 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/n叶草系列迭代.gsp` |
| 44 | 16 | 2 | `../gsp-rs/tests/Samples/个人专栏/方小庆作品/圆的滚动全解(inRm).gsp` |
| 119 | 13 | 4 | `../gsp-rs/tests/Samples/个人专栏/唐家军作品/课后反思10.gsp` |
| 25 | 10 | 4 | `../gsp-rs/tests/Samples/个人专栏/李有贵作品/凸四边形的滚动（含七个）（yougui).gsp` |
| 55 | 10 | 1 | `../gsp-rs/tests/Samples/个人专栏/汪志平作品/100以内两位数加一位数小棒图.gsp` |
| 115 | 10 | 8 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/n叶草系列迭代.gsp` |
| 11 | 8 | 6 | `../gsp-rs/tests/Samples/个人专栏/孟令岩作品/※圆锥，圆柱，圆台展开.gsp` |
| 31 | 8 | 4 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/勾股定理证明及应用实验素材.gsp` |
| 93 | 8 | 6 | `../gsp-rs/tests/Samples/个人专栏/方小庆作品/(inRm)万花规.gsp` |
| 42 | 7 | 6 | `../gsp-rs/tests/Samples/个人专栏/孙禄京作品/2005年南京中考题.gsp` |
| 110 | 7 | 5 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/正弦函数、余弦函数的图象.gsp` |
| 46 | 6 | 5 | `../gsp-rs/tests/Samples/个人专栏/周维波作品/三角转子的滚动-雪山飞狐.gsp` |
| 70 | 6 | 3 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/圆环的迭代(5环旋转迭代).gsp` |
| 83 | 6 | 4 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/滑块(蚂蚁).gsp` |
| 87 | 5 | 4 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/线性规划作图一例.gsp` |
| 20 | 4 | 3 | `../gsp-rs/tests/Samples/个人专栏/方小庆作品/2010攀枝花中考题.gsp` |
| 79 | 4 | 4 | `../gsp-rs/tests/Samples/个人专栏/方小庆作品/割圆为方详解(inRm).gsp` |
| 109 | 4 | 2 | `../gsp-rs/tests/Samples/个人专栏/方小庆作品/2010攀枝花中考题.gsp` |
| 124 | 3 | 2 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/圆环的迭代(5环旋转迭代).gsp` |
| 10 | 2 | 1 | `../gsp-rs/tests/Samples/未分类档/时钟 (2).gsp` |
| 22 | 2 | 2 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/四圆极限集[31].gsp` |
| 58 | 2 | 1 | `../gsp-rs/tests/Samples/个人专栏/郑飞宇作品/广告片断.gsp` |
| 59 | 2 | 1 | `../gsp-rs/tests/Samples/个人专栏/方小庆作品/2010攀枝花中考题.gsp` |
| 61 | 2 | 2 | `../gsp-rs/tests/Samples/个人专栏/方小庆作品/2010攀枝花中考题.gsp` |
| 78 | 2 | 1 | `../gsp-rs/tests/Samples/未分类档/过笛卡尔心形线外一点的切线.gsp` |
| 112 | 2 | 2 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/圆环的迭代(5环旋转迭代).gsp` |
| 114 | 2 | 2 | `../gsp-rs/tests/Samples/个人专栏/方小庆作品/(inRm)摆线.gsp` |
| 125 | 2 | 1 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/圆环的迭代(5环旋转迭代).gsp` |
| 1 | 1 | 1 | `../gsp-rs/tests/Samples/未分类档/机械的连动.gsp` |
| 18 | 1 | 1 | `../gsp-rs/tests/Samples/个人专栏/庞坤生作品/机械波的形成和传播.gsp` |
| 43 | 1 | 1 | `../gsp-rs/tests/Samples/个人专栏/方益初作品/犹抱琵琶半遮面（温州小刀）.gsp` |
| 54 | 1 | 1 | `../gsp-rs/tests/Samples/个人专栏/郑飞宇作品/广告片断.gsp` |
| 86 | 1 | 1 | `../gsp-rs/tests/Samples/个人专栏/向忠作品/正方形总面积不变.gsp` |
| 116 | 1 | 1 | `../gsp-rs/tests/Samples/个人专栏/孙禄京作品/时钟.gsp` |
| 122 | 1 | 1 | `../gsp-rs/tests/fixtures/未实现的系统功能/绘图函数.gsp` |

## Checkpoint 95: nested polar point transforms

The strict code26 point-transform reader now accepts an already decoded `DualMarkedPolar` point as its source. Each object's own `[point, Degrees, Length]` references remain live in the DAG and evaluate through the existing polar translation; no new transform enum or rendering branch is needed. Ten Native states verify source movement, length cancellation and rebinding, and angle changes, with whole-file exact saves. Other source point flavors remain outside this narrow admission.

The compiled scan restores **105 drawing slots**, including **38** direct code26 roots in six files: unsupported **9,331→9,226**, all-kind complete **836→838**, drawing-complete **851→853**. All **1,599** originals remain byte exact with zero failed parses. The prior 119 sole-root count was only a structural upper bound. See [implementation](point26_dual95_implementation.md), [Native analysis](point26_dual94_native_analysis.md), and [frontier audit](frontier93_analysis.md). Corpus log: `/tmp/point26_dual95_corpus.log`.

## Checkpoint 94: point-anchored PNG images

Native code99/class16/subtype78 images with the strict six-record layout now reuse the existing PNG-backed image object. Their `07d2` parent is a live point and their positive `07d3` scale factors place the image around that point; the admitted corpus profile stores zero rotation. The PNG resource catalog validates signatures, dimensions, chunks and CRCs. The shared image renderer, SVG path and picking derive the current transform from the point at evaluation time. Nonzero rotation, labels and empty-marker layouts remain separate. Sixteen Native open/save controls across a free anchor, a constructed anchor, and scale changes are whole-file exact. Isolated image masks prove exact (+40,+20) movement for the free parent; the constructed parent moves the image with its changed polar distance. Pixel-perfect scaling is not claimed.

The compiled scan restores **97 code99 drawing slots in 44 files**, lowering unsupported **9,428→9,331**. All-kind complete rises **826→836**, drawing-complete **841→851**. All **1,599** originals remain byte-exact, and the scope-aware workspace gate passes **1,443** tests. The earlier 276 sole-root descendant count was a structural upper bound; downstream parsers still block those objects, so actual recovery is the 97 images. Native resource tables remain read-only for structural edits. See [implementation](anchored_image94_implementation.md), [Native analysis](anchored_image94_native_analysis.md), and [frontier audit](frontier93_analysis.md). Logs: `/tmp/anchored_image94_corpus_trial.log`, `/tmp/anchored_image94_workspace_final.log`.

## Checkpoint 93: native π suffix in referenced functions

The shared function parser now treats `000d,0100` as one π operand in native token streams. The evaluator and formula display accept this exact suffix; isolated, duplicate, and unrelated-operand suffixes remain invalid. Thirteen parent-ready code71 roots share this 54-word expression and strict four-source role profile. The existing object DAG and dependency evaluator decode them without a new function or transformation enum. The original 25,260-byte demonstration and five controlled variants open/save byte-exact in Native; the displayed formula is isolated by a hide control, and an upstream geometric change moves the ordinary scalar chain from 392 Hz to 440 Hz. Native did not display a direct `f97(x)` value, so numeric results remain engine predictions.

The compiled corpus scan restores **216 drawing code71 slots**, including the 13 audited roots and later functions that depend on them. Unsupported falls **9,644→9,428**; all-kind complete stays 826 and drawing-complete stays 841. All 1,599 originals round-trip byte-exactly with zero failures. The scope-aware workspace gate executes **1,437 passing tests**. This demonstration is still a partial document, and public expression editing there is refused by the existing structural guard. See [implementation](referenced_function93_implementation.md), [Native analysis](referenced_function93_native_analysis.md), and [cohort audit](frontier92_function71_audit.md). Logs: `/tmp/referenced_function93_corpus.log`, `/tmp/referenced_function93_workspace_final.log`.

## Checkpoint 92: R/G/L free-text run colors

The shared rich-run style now carries complete ordered native `R/G/L` channels with values1..256. SVG and canvas color individual runs, and the existing mixed-format content-edit guard remains in place. Native seven-state controls verify the date subspan changes from blue to green+1 or darker blue while preceding text and geometry remain fixed; an independently shifted text box moves all pixels +40. Zero-channel probes exhibit extra inheritance semantics and stay outside the admitted range.

The compiled scan restores 83 code0 text slots: 72 initially rejected at the R/G/L style attribute and 11 whose earlier diagnostic hid the same color syntax. Another 32 no-label color candidates still fail independent markup rules, not the color gate. Unsupported9,727→9,644; all-kind complete806→826 and drawing-complete821→841. All1,599 originals remain byte exact with zero failures; the scope-aware workspace gate passes1,435 test executions. See [implementation](rich_style92_implementation.md), [Native analysis](rich_style92_native_analysis.md), [cohort audit](frontier91_text_audit.md), and [recovery audit](frontier91_rgb_recovery.md). Logs: `/tmp/rich_style92_corpus_final.log`, `/tmp/rich_style92_workspace_final.log`.

## Checkpoint 91: multiple imported PNG objects

Native multi-picture documents now decode each independent code0/class16/sub78 object through a validated PNG resource catalog. The `1f44` index selects its own PNG; the image's saved affine transform and visibility remain separate. The strict importer covers five-record image groups and the otherwise identical six-record group with an empty `232e` marker. Two groups with an additional label record remain opaque. Multi-picture documents stay structurally read-only; this expands semantic viewing without assuming a safe resource-table writer. Native six-state controls isolate two images and their independent translations, all whole-file exact.

The compiled scan restores 146 code0 drawing slots, lowering unsupported9,873→9,727; all-kind complete797→806 and drawing-complete812→821. All1,599 originals remain byte exact with zero failures; the scope-aware workspace gate passes1,430 test executions. See [implementation](image_objects91_implementation.md), [Native analysis](image_objects91_native_analysis.md), and [frontier audit](frontier90_analysis.md). Logs: `/tmp/image_objects91_corpus_final.log`, `/tmp/image_objects91_workspace_final.log`.

## Checkpoint 90: opaque URL action suffixes

Strict URL ActionButton parsing now retains opaque nonzero bytes after the NUL-terminated URL in observed 44- and 56-byte `0906` payloads; labels and all other action fields keep their existing validation. Imported URLs with opaque suffixes are read and saved intact, and same-length caption edits preserve the suffix. Replacing such a URL is refused until its suffix mutation rule is known. Eight Native passive-open/save controls are whole-file exact, with unchanged canvas after clearing the suffix; a safe local-link click did not establish suffix activation meaning.

The compiled scan restores 408 code0 URL buttons and 241 code73 action-reference texts that decode through the already-supported Value role after their URL parent becomes semantic: 649 drawing slots total. Five more dependent texts in one file have a separate unsupported rich-text token. Unsupported10,522→9,873; all-kind complete678→797 and drawing-complete691→812. All1,599 originals remain byte exact, zero failures; the scope-aware workspace gate passes1,426 test executions. See [implementation](url_suffix90_implementation.md), [Native analysis](url_suffix90_native_analysis.md), and [cohort audit](frontier89_action_url_audit.md). Logs: `/tmp/url_suffix90_corpus_final.log`, `/tmp/url_suffix90_workspace_final.log`.

## Checkpoint 89: colorized polygon regions and mapped copies

The strict code75 class4/subtype15 packet now decodes a Polygon source with native Reflect Spectrum, Grayscale, RGB or HSV operands, including the independently verified Spectrum `0..2` range control. The generic Colorized node retains live polygon geometry and numeric color dependencies. Code89 mapped iteration evaluates each generation's vertices and color in the same scoped state; code77 renders each filled result with its own computed color. Source wrapper and mapped binding remain independently visible. Native direct-star isolation, color-range/point edits, and depth1–4 controls establish the behavior; all 11 successful Native controls save whole-file exact. The source12-hide control did not open and gives no visibility conclusion.

The compiled scan restores 66 drawing slots: 19 code75, 15 code89 and 32 code77. Unsupported10,588→10,522; all-kind complete671→678 and drawing-complete684→691. All1,599 originals remain byte exact, zero failures; the scope-aware workspace gate passes1,424 test executions. See [implementation](colorized_polygon89_implementation.md), [Native analysis](colorized_polygon89_native_analysis.md), and [cohort audit](frontier88_analysis.md). Logs: `/tmp/colorized_polygon89_corpus_final.log`, `/tmp/colorized_polygon89_workspace_final.log`.

## Checkpoint 88: mapped colorized disks

Code89 mapped-point iterations now carry an intrinsic `Colorized(Disk)` through the existing scoped DAG evaluator. Code77 disk bindings render analytic disks with colors reevaluated for each generation and use the stored binding opacity; circle bindings remain independent. The mapped read path accepts value depth through 4096 with the existing 20,000-per-level and 100,000-total generation guards. Authoring keeps its separate depth limit. The six Native visibility states save byte exact, and the prior same-document scalar/movement controls establish per-generation colors and live positions. Source16 hide inputs did not open under Native and provide no visibility conclusion.

The compiled scan restores 64 slots: 16 drawing code89, 43 drawing code77, one drawing code76, one prototype code89 and three prototype code77. Unsupported10,652→10,588; all-kind complete666→671 and drawing-complete679→684. All1,599 originals remain byte exact, zero failures. See [implementation](colorized_iteration88_implementation.md), [Native analysis](colorized_iteration88_native_analysis.md), and [cohort audit](frontier87_analysis.md). Corpus log `/tmp/colorized_iteration88_corpus.log`; scope-aware workspace gate `/tmp/colorized_iteration88_workspace_final2.log` passes 1,416 test executions.

## Checkpoint 87: live three-point angle markers and aliases

Strict code113 `AngleMarker` packets with one or two arc marks use three existing point objects. The marker remains one semantic object; sampled arc strokes are derived presentation geometry. A code120 alias reuses the shared numeric-alias node, reads a signed live angle, and formats degrees or radians according to page `0x273a`. Labeled marker text is retained independently of the live numeric value. Native controls cover hide/move, positive and negative values, and both unit-switch directions; all 23 complete saved files are byte exact. Unproved `090e` flags, code113 labels with extra marker records, and unrelated code120 parent types remain opaque.

The compiled corpus recovers 160 drawing slots: 125 code113 markers, eight code120 aliases, and 27 dependent objects. Unsupported slots fall 10,812→10,652; all-kind complete documents rise 659→666 and drawing-complete 672→679. All 1,599 originals retain byte-exact round trips with zero failures. Scope-aware workspace tests pass 1,411 executions. See [implementation](angle_marker87_implementation.md), [Native proof](angle_marker87_native_analysis.md), and [alias audit](frontier86_analysis.md). Logs: `/tmp/angle_marker87_corpus.log`, `/tmp/angle_marker87_workspace_final2.log`.

## Checkpoint 72: grouped unary referenced functions

The shared expression parser accepts a unary call whose grouped operand is
followed by an infix continuation before the call's final closing token. The
calendar's original `trunc((Y−1)/x)` definition and downstream calculation
decode through the existing DAG and live evaluator. The measured scan restores
623 slots: unsupported13,794→13,171, including code71 425→271, code72
132→27, and code48 1,199→1,135. All1,599 originals stay whole-byte exact;
all-kind complete rises626→633 and drawing-complete637→646. This is actual
recovery, separate from the prior109-root/585-reach grammar estimate.

The headless build succeeds. The scope-aware workspace gate passes1,373 tests
with seven copying/custom-tool tests explicitly excluded; the unrestricted
gate fails those seven deferred tests. Native original and synthetic controls
verify whole-file saves and the calendar's live numeric values. At `x=0`, Native
displays infinity while the finite-value engine API returns `None`. See
[implementation and gate details](referenced_function72_implementation.md).

## Checkpoint 73: standalone custom-transform points

The bounded code26 decoder admits only a three-reference Point with a Free
source, a Degrees value, a Length value, no extra group records and no direct
code102 trace association. It lowers to the existing generic polar transform
expression, so source and numeric-parent edits update downstream dependencies.
The imported native group remains byte-preserved; authoring this code is still
blocked pending a separate writer proof. Linked trace programs, Segment outputs,
Scalar operands and labelled extra-record groups remain unsupported.

The final compiled scan restores129 slots: unsupported13,171→13,042, code26
329→279 (50 directly recovered), code29 431→406, code77 1,808→1,791 and
code89 540→530. All1,599 originals remain whole-byte exact with zero scan
failures; all-kind complete rises633→636, drawing-complete646→649 and partial
views fall1,135→1,132. The scope-aware workspace gate passes1,376 executions,
with seven copying/custom-tool tests excluded under the user's deferred scope.
See [implementation and Native
evidence](code26_standalone73_implementation.md).

## Checkpoint 74: Vector-transformed source for standalone code26

The same typed code26 point operation now accepts a Vector-transformed Point
source, after a separate Native original and isolated marker/source-move
controls. The public edit of this partial original is blocked by an unrelated
interior-order guard, so saved move controls are explicitly synthetic.
All four Native states save whole-file exact. Code102-linked points and other
transformed-source flavors remain opaque.

The compiled scan restores96 slots: unsupported13,042→12,946, code26
279→253, code16 387→357, polygons8 191→163 and code29 406→395. All1,599
originals remain byte exact; complete-document and partial-view counts stay
unchanged. The headless build and1,377 scope-aware tests pass. See
[implementation and Native evidence](code26_transformed74_implementation.md).

## Checkpoint 75: bounded code71 high-word wrappers

The code71 expression count may carry upper word 1 or 2 while its lower word
still gives the exact token count. The parser retains its wrapper, source and
syntax checks; the writer preserves the high word on expression edits. Native
high1 and high2 controls demonstrate live source behavior, subject to the
partial-document and metadata-normalization limits in the [analysis](referenced_function75_native_analysis.md).

The compiled scan restores28 slots: unsupported12,946→12,918, code71
271→252, code48 1,135→1,130, code69 53→52 and code126 3→0. All1,599
originals remain byte exact; complete and partial-view counts stay unchanged.
The headless build and1,379 scope-aware tests pass. See [implementation](referenced_function75_implementation.md).

## Checkpoint 76: FixedPolar source for standalone code26

The existing code26 Point/Degrees/Length operation now follows a FixedPolar
transformed Point source after four whole-byte-exact Native controls prove the
marker and an upstream move. The partial original's saved move is explicitly
field-level synthetic. Exact record layout and no-code102 guards remain.

The compiled scan restores52 slots: unsupported12,918→12,866, code26
253→202 and code21 51→50. All1,599 originals remain byte exact; complete and
partial-view counts stay unchanged. The headless build and1,380 scope-aware
tests pass. See [implementation and Native evidence](code26_fixedpolar76_implementation.md).

## Checkpoint 77: MarkedPolar source and constrained segment values

Standalone code26 Points now follow a MarkedPolar-transformed Point source.
Fifteen Native controls preserve complete bytes and isolate source motion and
angle changes. The angle proof required a bounded PointValue correction for a
PointOnSegment measured against another Segment; independent Free-point
projection remains unbounded.

The compiled scan restores68 slots: unsupported12,866→12,798, including
code26 202→186, polygons8 163→144, code16 357→339 and code29 395→387.
All1,599 originals remain byte exact; all-kind complete rises636→637,
drawing-complete649→650 and partial views fall1,132→1,131. The headless
build and1,382 scope-aware tests pass. See [implementation and Native evidence](code26_markedpolar77_implementation.md).

## Checkpoint 78: empty scalar coordinate marker

Code65/66 now accept one empty `232e` after the exact label/style pair while
retaining the original bytes. A visible code65 Native file proves live x values
with two whole-file-exact saves. A paired hidden code65/66 original retains
both markers and refs through Native normalization, but its ordinate caption
is not independently readable.

The compiled scan restores1,017 slots: unsupported12,798→11,781,
code65 79→36, code66 79→37, code48 1,130→790, code77 1,791→1,455,
code89 530→362 and code90 388→304. All1,599 originals remain byte exact;
complete and partial-view counts stay unchanged. The headless build and1,385
scope-aware tests pass. See [implementation and Native evidence](coordinate_measurements78_implementation.md).

Run the command again for the current full report and all examples.

## Audited prerequisites

The current work addresses native numeric presentation records and optional
labels, broader scalar parameter/calculation layouts, independent point-value
projections, polygon boundary points, numeric coordinate points, and
parameter-driven point traces. See the corresponding analysis files in `dev`.
Native cross-open checks and save/reopen tests accompany each supported family.
The snapshot includes the verified numeric trace and multi-seed iteration
families, transformed-point labels, vector transform chains, and native URL
button layouts. It is a measurement of that build, not a final release claim.

Typed length and degree parameters, optional empty calculation attributes, polygon boundary value points, and further mapped point/segment iterations now have native proofs. Angle parameter curves now retain native radian ranges and expose degrees in the range editor. Fixed rotation/dilation/reflection chains, literal polygon path points, and transformed polygon boundaries also have focused native evidence. Scientific length/degree exponent-5 literals, π radian literals, optional parameter labels and page-generated names, and fixed ray/line rotations and dilations now also have edit/copy native proofs. Fixed ray/line transform chains preserve source strokes and undefined zero-scale ancestors. Negative code-95/123 positions on those ray hosts remain undefined while full lines extrapolate. Labeled code-123 path points retain typed label/font suffixes through edits and copying. Legacy free-point/polygon clipboard conversion preserves metadata and has structural undo. The next audited gaps include further calculation layouts, additional iteration definitions and their code-77 bindings, broader multi-page numeric record editing, and further vector-transform variants. Saved scalar parameter edits and supported table rows now have focused multi-page native proofs; this does not remove the document-wide partial-view flag. Original `cood-trace.gsp`,
`poly_point_with_val.gsp`, refraction and Ningde examples, and
`迭代方法2(蚂蚁).gsp` provide concrete native payloads and paired HTML definitions.
Retain native payload validation and regression/cross-open gates when extending
these families.

The next snapshot (checkpoint 12) retains the same drawing/tool scope and reduces
unsupported slots from 112506 to 108817. It includes source-free radian arithmetic,
centimeter calculation literals, numeric-ratio ray/line dilation, mixed fixed and
numeric straight-transform chains, and the proved arc plot hosts. The full
all-targets regression gate at this checkpoint passes 782 tests. Later constrained
arc semantics, trailing decimal parsing, and unified transformation-model work
are subsequent changes and require another gate.

Checkpoint 13 retains that same scope and reduces unsupported slots to 108783.
It includes the verified constrained linear arc semantics and trailing decimal
calculation literals. All 1599 documents remain byte-identical on round-trip.
The unified transform refactor does not itself count as additional decoding
coverage. Checkpoint 14 independently confirms the same counts after the shared consumer
migration. The combined all-targets gate passes 805 tests, including the
independently verified straight-geometry corrections and slope precision fixes.
Generic transform storage and the remaining command requirements are pending.

Checkpoint 16 verifies the generic transform storage cutover: all 1599 files
still round-trip byte-identically, and unsupported slots remain 108783. The
combined gate now passes 806 tests. Sixty product Kind variants have been
removed in favor of one generic transform variant; compatibility constructor
names remain. This representation change does not expand decoding coverage or
complete the pending command requirements.

Checkpoint 18 retains all 1599 byte-identical round trips and the same 280347
drawing slots and 285703 total slots. Drawing unsupported kinds remain at
103427. Coverage now uses the implemented isolated tool decoder rather than
classifying every prototype as unknown: 1822 of 5356 prototype slots decode,
leaving 3534 unknown and 106961 unknown slots overall. Eight isolated tool
definitions have complete views and 95 have partial views; none fails extraction.
Seven coverage regressions distinguish drawing preservation, unknown prototype
kinds, extraction failures, and known kinds with unsupported metadata.
The drawing partial-view count is 1522 after the proved labeled code-12
prototype support. Native Square and simple-tool invocation and creation controls are documented separately
in `tool_runtime_analysis.md` and `tool_creation_analysis.md`; these bounded proofs
do not establish corpus-wide command parity.
The settled combined all-targets gate passes 825 tests, including both saved
desktop invocation routes and the Native-verified creation writer.

Checkpoint 20 recognizes declared generic numeric and finite-segment tool inputs,
including their live dependent calculations and host-based midpoints. Prototype
known slots increased by 947; drawing known slots increased by 96. Every corpus
file still preserves its original bytes. Numeric tool authoring, caption font and
placement fidelity, other given categories, and the command gaps remain work in
progress. These counts do not prove that every decoded prototype executes.


Checkpoint 21 retains the checkpoint-20 counts and all 1599 exact round trips
following the ordered matrix evaluator and indexed scalar-table decoder work.
The new indexed table fixtures are separate native controls outside this corpus;
their support does not change this corpus histogram. Command behavior remains
subject to the separate native-editing proofs. Log:
`/tmp/sketchpad_corpus_checkpoint21.log` (exit 0).


Checkpoint 22 retains all checkpoint-21 counts and 1599/1599 exact round trips
after indexed-table font and UTF-8 negative-row support. Ordinary creation writer
fixtures are independent Native controls outside this corpus. Log:
`/tmp/sketchpad_corpus_checkpoint22.log` (exit 0). Copy and custom-tool implementation
remain deferred.


Checkpoint 23 retains the checkpoint-22 counts and all 1599 exact round trips
after generated t[1] table support. Legacy fixed-two-decimal table behavior is
still covered, while the independent counter-1 Native control and actual writer
are now supported. Log: `/tmp/sketchpad_corpus_checkpoint23.log` (exit 0).


Checkpoint 24 retains the checkpoint-23 counts and all 1599 exact round trips
after named table decoding preserves observed drawing font references. New table
creation now materializes other unnamed page parameters for default and plain-name
sources too, and resolves its default font against the drawing namespace. Those
writer controls are independent of this corpus. Log:
`/tmp/sketchpad_corpus_checkpoint24.log` (exit 0).


Checkpoint 25 retains the checkpoint-24 histogram and 1599 exact round trips
after integer/UTF-8 negative named table support and clean imported table
creation. Log: `/tmp/sketchpad_corpus_checkpoint25.log` (exit 0). Priority now
follows broad feature coverage and construction dependencies; matching original
rendering pixel-for-pixel is not required.


Checkpoint 26 broadens marked-distance translation to accept existing transformed
segments, circles and polygons through the shared transform source predicates.
This recovers 3171 drawing slots, including 3150 native-code-24 objects, while
retaining all 1599 exact byte round trips. The focused regression covers two
matrix steps, live distance changes, downstream length/area, save and undo/redo.

Checkpoint 27 accepts validated optional coordinate-measurement captions,
observed numeric text-cache fields and display modes 0/1/3, plus the observed
empty code-94 point-value attribute. Coordinate and point-value mathematics and
host validation remain unchanged. This recovers another 9233 drawing slots and
530 prototype slots through shared decoding, including 5567 drawing calculations.
Unsupported drawing calculations fall from 20270 to 14703. All 1599 original
files still round-trip byte-identically; document-wide partial views remain
1522 and fully decoded drawing documents remain 86. These figures do not imply
that the remaining unsupported or undefined graphs can be edited safely.

The three retained corpus regressions check native captions/formats, numeric
values and downstream calculations. A supported isolated graph combines the
original native caption with display modes 1/3 and nonzero cache bytes; its
coordinate and calculation values follow dragging and survive strong round-trip
validation and undo/redo. These are local file and semantic regressions, not
new Native application writer proofs. See
[broad prerequisite audit](broad_measurement_dependency_analysis.md).


Final scoped regression gates pass 602 library tests (110 filtered) and 106
editor tests (21 filtered), using the deferred copying/custom-tool name filters.
The additional isolated point-value attribute regression checks live values,
strong save/reparse validation, exact retained bytes and undo/redo. Evidence:
`/tmp/sketchpad_broad_measure_final_gate3.log`,
`/tmp/sketchpad_broad_measure_editor_gate.log`,
`/tmp/sketchpad_point_value_attribute_gate.log`, and
`/tmp/sketchpad_corpus_checkpoint27.log` (all exit 0).


Checkpoint 28 generalizes fixed-polar translation sources to transformed
segments/circles/polygons, recovering 952 drawing slots and 45 prototype slots,
including 358 calculations. Checkpoint 29 normalizes native radian transform
angles while validating their declared trigonometric caches and retaining both
observed degree flags. It also fixes the polar pixel-distance cache: native
records contain total displacement, while the shared model stores magnitude
and conversion scale. Import and generation now agree on these meanings.

Relative to checkpoint 27 these changes recover 5993 drawing slots and 90
prototype slots, including 1423 drawing calculations. Original round trips are
1599/1599 exact; unsupported slots total 86257. Drawing calculations fall to
13280, and fully decoded drawing documents rise to 87. Partial views remain
1521, so this is still partial implementation coverage.

Five retained original corpus files and paired HTML exports establish radian
rotation/translation and fractional-distance geometry. An actual API-generated
fractional-distance segment/circle/polygon chain file opened and saved
byte-identically in Sketchpad 5 (helper exit 0 and independent comparison).
The native result is retained as a reconstruction regression golden. The scoped
library gate passes 606 tests (110 filtered); see
[polar transform evidence](polar_translation_units_analysis.md).


Checkpoint 30 recognizes marked distance sources by their numeric Length unit,
including parameters and calculations, and evaluates their centimeter values
through the existing numeric evaluator. The finite/nonnegative pixel policy
remains unchanged. This recovers 1219 drawing slots (including 629 code24
objects and 32 calculations); prototype counts and partial-view totals are
unchanged. All 1599 original files retain byte-identical round trips. Unsupported
slots total 85038, including 83715 drawing objects and 1323 prototype objects.

The editor now accepts a Length Parameter through Mark Distance. Regressions
cover point/segment/circle/polygon polar translation, rectangular and mixed
point translation, typed calculations, live parameter updates, existing
negative/undefined behavior, save/reparse and undo/redo. The actual writer file
opened and saved byte-identically in Sketchpad 5, independently compared against
its untouched before-image. The scoped gates pass 608 library tests and 107
editor tests. See [typed distance evidence](typed_length_marked_transforms_analysis.md).


The additional numeric-import gate passes32 tests with10 copying/custom-tool
name-filtered cases deferred. Copy plural/participle aliases are filtered too;
the unfiltered plural-copy sourcefree-radian test fails at its pasted-point
lookup and is outside the current authorized implementation priorities. This
is not an all-targets completion claim.

## Checkpoint 31: source-free quantities

The shared calculator now accepts source-free centimeter lengths, area products,
inverse-angle functions, and scalar functions of radian literals. Expression edits
retain the numeric unit, record presentation and undo/redo behavior. Length results
feed the existing marked-distance transform model without new shape/transform enum
variants. Undefined expressions retain their unit and suppress dependent geometry.

The complete corpus retains 1599/1599 exact round trips and reduces unsupported
objects from 85038 to 84900: 138 drawing objects recovered, prototype counts
unchanged. Original Length and inverse-angle controls are retained; the corpus
contains no source-free Area control. An authored seven-object file including
Length, Area, scalar/radian functions, trailing decimal syntax and a marked-distance
translation was opened and saved by the unchanged native helper (exit 0), with an
independent byte comparison passing. This is writer evidence, not a claim of
matching every native display detail. See `source_free_quantity_calculations_analysis.md`.

## Checkpoint 32: many-input calculations

Source references use the continuous native `0x6000 + index` encoding. Corpus
profiles contain up to 51 references and tokens through `0x6032`; the old 16-input
limit rejected original formulas and their dependent constructions. The shared
parser and authoring path now accept up to 256 inputs/tokens, preserving source
ordering, aliases, units and exact record bytes. Input indexes must be within the
reference list. Source-free expressions reject source tokens across the new range.

The original 20-input `100位e.gsp` formula now yields 2.718281828. An authored
51-input sum with a length calculation and marked-distance image, edited at its
51st input, opened and saved byte-identically in native Sketchpad (13314 bytes).
The corpus recovers 156 drawing slots, including 80 calculations; prototype counts
remain unchanged, and all 1599 original files retain exact round trips. Large
calculation dependency-root counts overlap other missing families and should not
be confused with actual recovered-slot counts. See `many_input_calculations_analysis.md`.

## Checkpoint 33: scalar marked distances

Native marked-distance translations accept unitless values as centimeter
magnitudes, including negative values that reverse direction. The shared numeric
expression, transform evaluator, decoder, editor and iteration mapping now handle
these operands without shape/transform enum variants. Length arithmetic retains
its existing dimensional checks. Isolated native positive/negative Length controls also establish signed movement;
true undefined and nonfinite operands still suppress images.

Four native-open/save controls, an original scalar-calculation cube control and
an actual API writer containing polar point/segment/circle/polygon, Cartesian and
mixed translations provide evidence. The complete corpus recovers 6610 drawing
slots (760 calculations) and 108 prototype slots while preserving all 1599
original files exactly. No custom-tool authoring or copying support is added.
See `scalar_marked_distances_analysis.md`.


Checkpoint34 adds center arcs and their shared measurements/point paths. It
recovers4941 drawing objects, including361 calculations, and26 prototype slots.
The89 fully decoded documents and1599 exact source round trips are from the
completed final scan. Defined and degenerate actual API writer outputs both
open/save exactly in Native. See [center arc evidence](center_arc_analysis.md).
Scoped gates pass613 library tests,109 editor tests,34 numeric-import tests
and4 coverage tests, excluding the deferred copying/custom-tool tests.


Checkpoint 35 treats the coordinate measurement's final display word as
preserved presentation metadata, matching the existing ordinary measurement
reader. Native x/y controls and live API writer outputs prove the values and
exact saves. It recovers 2672 drawing objects, including 1232 calculations, with
70387 unsupported slots remaining and all 1599 exact original round trips. The
prototype and fully decoded document counts are unchanged. Unsupported groups'
raw numeric metadata is now retained during edited serialization as well.
See [coordinate presentation evidence](coordinate_presentation_tail_analysis.md).

Scoped checkpoint 35 regression gates pass 615 library, 109 editor,
34 numeric-import and 4 coverage tests. The complete final scan is recorded in
`/tmp/sketchpad_corpus_checkpoint35_final.log`.


Checkpoint 36 implements Native's unitless magnitude results for addition and
subtraction of unlike quantities. It recovers 5373 drawing objects, including
4351 calculations, while all 1599 source files preserve their bytes. Native
calculator-created controls cover variable, literal and compound quantities;
both actual 18-object API writer outputs save exactly in Native. The final
scan is `/tmp/sketchpad_corpus_checkpoint36_final.log`. Scoped gates pass 619
library, 109 editor, 34 numeric-import and 4 coverage tests. See
[mixed quantity arithmetic](mixed_quantity_arithmetic_analysis.md), including
its correction to the earlier cropped subtraction capture.


Checkpoint 37 supports typed quantity exponents and pixel calculation literals
through the shared AST. It recovers 398 drawing objects, including 392
calculations, with all 1599 exact source round trips and unchanged prototype
counts. Native creation controls establish units and pixel conversion; the
actual original/edited 17-object API writer outputs open and save byte-identically
in Native. The final scan is `/tmp/sketchpad_corpus_checkpoint37_final.log`.
See [quantity exponent evidence](quantity_exponents_analysis.md).
Scoped gates pass 623 library, 109 editor, 34 numeric-import and 4 coverage tests.


Checkpoint 38 adds referenced function calls to the shared calculator, including
constant calls, nested calls and derivative functions. Typed Degree arguments
follow the function's document angle mode. Numeric iteration labels use the
same callback and retain their function dependencies. It recovers 139 drawing
objects, including 75 calculations, with all 1599 exact source round trips.
The actual original/edited 28-object API writer outputs open and save exactly in
Native. The final scan is `/tmp/sketchpad_corpus_checkpoint38_final.log`.
See [function call evidence](function_applications_analysis.md).
Scoped gates pass 628 library, 109 editor, 34 numeric-import and four coverage tests.


Checkpoint 39 shares the calculator's built-in registry with primitive function
definitions: inverse trig, sign, rounding, truncation and π/e constants now
import, evaluate, plot and save. Signed caption positions are accepted without
the former artificial 4096 bound. It recovers 271 drawing objects, including
119 calculations, with all 1599 exact source round trips and unchanged
prototype counts. Native-created six-function controls and actual original /
edited 38-object API writer outputs establish native token semantics and exact
saves. The final scan is `/tmp/sketchpad_corpus_checkpoint39_final.log`.
See [primitive function evidence](function_primitives_analysis.md).

Scoped gates pass 631 library, 109 editor, 34 numeric-import and four coverage tests.


Checkpoint 40 establishes the referenced-expression parser and Native record
writer controls, without claiming new document coverage. Checkpoint 41 connects
those references to the real document DAG, authoring/editing, plotting,
calculator calls and numeric iteration. It restores 919 drawing objects,
including 229 function definitions and 439 calculations, plus three prototype
objects through the shared reader. All 1599 exact source round trips remain;
one additional drawing is fully decoded. Original/edited actual 38-object API
writer outputs save byte-identically in Native. The final scan is
`/tmp/sketchpad_corpus_checkpoint41_final.log`. See
[referenced function evidence](function_references_analysis.md).
Scoped gates pass 642 library, 112 editor, 34 numeric-import and four coverage tests.


Checkpoint 42 supports numeric-sized coordinate axes through shared numeric and
segment interfaces, and corrects literal Cartesian/Polar point dependencies:
07d2 stores a count followed by the frame ID, rather than two source IDs.
The shared reader restores 3841 drawing objects (282 calculations) and one
prototype object, reducing unsupported slots to 59442. All 1599 original files
remain byte-identical with zero import failures. Nine actual Document API writer
states save exactly in Native, covering live edits, Scalar/Degree/Area/Compound
units, negative/zero units and restoration. Meaningful regressions also cover
labels, removal of unrelated objects, malformed input and undo/redo.
See [coordinate units and point dependencies](coordinate_units_analysis.md).
The final scan is `/tmp/sketchpad_corpus_checkpoint42_final.log`.
Scoped gates pass 650 library, 115 editor, 34 numeric-import and four coverage
tests (803 total). Copying and custom tools remain deferred; these counts do not
establish complete command parity or unrestricted editing of partial documents.


Checkpoint43 extends simultaneous iteration through the shared dependency graph:
derived point seeds and Scalar seeds, including mixed mappings, substitute their
values before the next branch is evaluated. Generic point/segment transforms and
supported coordinate dependencies reuse ordinary geometry evaluation. The scan
restores269 drawing slots:12 fixed definitions,98 live-depth definitions and159
bindings, reducing unsupported slots to59173. Prototype counts are unchanged.
All1599 originals remain byte-identical with zero import failures. Native checks
prove the actual derived-segment and numeric/mixed API writers, live edits and
the original five-branch pentagon's25 final images. See
[derived seed evidence](mapped_derived_transform_analysis43.md) and
[scalar state evidence](numeric_mapped43_analysis.md).
The final scan is `/tmp/sketchpad_corpus_checkpoint43_final.log`.
Copying and custom tools remain deferred; general single-seed representation,
typed numeric states beyond Scalar and wider carried DAGs remain incomplete.
Scoped gates pass656 library,115 editor,34 numeric-import and four coverage tests
(809 total), using the same copying/custom-tool filters. Evidence:
`/tmp/sketchpad_checkpoint43_lib_final_gate2.log`,
`/tmp/sketchpad_checkpoint43_editor_gate.log`,
`/tmp/sketchpad_checkpoint43_numeric_gate.log`, and
`/tmp/sketchpad_checkpoint43_coverage_gate.log` (all exit0).


Checkpoint44 extends the existing mapped DAG predicate to ordinary geometry and
numeric measurements. It restores422 drawing slots across77 files:108 roots
(104 live-depth and4 fixed-depth) and314 point/segment bindings. Unsupported
slots fall to58751, including57566 drawing slots and1185 prototype slots.
All1599 originals remain byte-identical, with zero import failures. The final
scan is `/tmp/sketchpad_corpus_checkpoint44_final.log`.
Native full/abbreviated carried controls establish that only explicit carried
nodes and seeds update per state. External construction values and function
coefficient environments stay frozen; runtime caches preserve original Kind
metadata. Undefined carried images retain valid prefixes and other bindings.
Native measurements cover live angle/length inputs and a real center-arc graph;
path controls cover circle/segment positions and three intersection families.
The evidence also fixes oriented circle-circle branch choice and label ordering
for new transformed points and circle-circle intersections. Fixed/numeric
rotations and labeled API intersection files have exact Native writer proofs.
See [path/intersection evidence](mapped_path_intersection_analysis44.md) and
[measurement/arc evidence](measurement_mapped44_analysis.md).
Copying and custom tools remain deferred. General single-seed mapped storage,
binding types beyond points/segments and undefined branch-target behavior remain
outside this change. Decoding gains do not imply complete command parity.
Scoped regression gates pass664 library,115 editor,34 numeric-import and four
coverage tests (817 total), with the same copying/custom-tool filters.
Evidence: `/tmp/sketchpad_checkpoint44_lib_gate.log`,
`/tmp/sketchpad_checkpoint44_editor_gate.log`,
`/tmp/sketchpad_checkpoint44_numeric_gate.log`, and
`/tmp/sketchpad_checkpoint44_coverage_gate.log` (all exit0).

Checkpoint45 adds single-seed point/Scalar mappings through the same carried DAG,
recovering404 roots (377 live-depth and27 fixed-depth) plus720 point/segment
bindings. Generic authoring retains canonical existing point/segment/circle/ray
and branching-polygon wire adapters; shared binding templates and style handling
keep authoring, save/reopen and image-level edits consistent. Fresh carried
lists are stably ordered by their actual Native construction dependencies.
Midpoint dependencies include their segment host, and temporary seed values
preserve the original point kind. Original imported record order stays intact;
Native's historical unsorted-carried packet-order artifact is documented rather
than claimed equivalent to the port's graph evaluation.

The recursive polygon-source predicate additionally restores1314 drawing
reflection objects; the same parser changes restore5 prototype bindings/roots.
Total unsupported slots fall from58751 to56308:55128 drawing and1180 prototype
slots. All1599 originals are byte-identical and there are zero import failures;
93 documents now have all kinds decoded,94 all drawing kinds. The final scan is
`/tmp/sketchpad_corpus_checkpoint45_final.log`.

Native evidence includes11 fresh single-seed API controls, two original grid
constructions, an original compound-reflection graph and three fresh reflected
polygon API states including source and mirror edits. Source restoration,
final-level choice and structural undo/redo have meaningful regressions.
See [single-seed evidence](single_seed_iteration_analysis45.md) and
[recursive polygon evidence](transformed_curves45_analysis.md).
Scoped gates pass672 library,115 editor,34 numeric-import and4 coverage tests
(825 total), with the same copying/custom-tool filters. Evidence:
`/tmp/sketchpad_checkpoint45_lib_final.log`,
`/tmp/sketchpad_checkpoint45_editor_gate.log`,
`/tmp/sketchpad_checkpoint45_numeric_gate.log`, and
`/tmp/sketchpad_checkpoint45_coverage_gate.log` (all exit0).
Copying and custom tools remain deferred. Wider binding geometry, typed numeric
seeds beyond Scalar, general iteration dialogs and action variants remain open.


Checkpoint 46 adds shared lossless action record roles, optional labels and
empty metadata, plus ordered Movement pair lists. This restores 1,799 drawing
buttons and 47 prototype slots without adding copy or custom-tool implementation.
Unsupported slots fall 56,308 → 54,462. All 1,599 originals remain byte-identical,
zero imports fail, and 95 documents have every object kind decoded.
The final scan is `/tmp/sketchpad_corpus_checkpoint46_final.log`.

Movement controls share point/numeric values and preserve their original
construction: free and constrained points, supported coordinate units, and
editable Parameters use their existing mutation paths. Initial targets are
frozen together; actual Native overlap controls establish action order for
Follow. New buttons accept multiple selected source/destination pairs and an
entire activation has one undo checkpoint. Partial-document numeric editing
continues to use the existing Parameter safety policy. Runtime evidence and
limitations are recorded in [movement controls](movement_controls46_analysis.md)
and [action records](action_records46_analysis.md). Decoding counts include
buttons whose parents or runtime controls remain unavailable.


The shared horizontal-unit writer now emits edited positive steps for fresh
coordinate groups and patches saved code 52 constraints. Native accepted both
fresh square/rectangular edited controls; strict saved/reopen regressions cover
these paths. Stored Movement validation does not require an active page or a
currently defined value, while activation retains runtime safety guards.
Final scoped gates pass 687 library, 120 editor, 34 numeric-import and 4 coverage
checks (845 total), with the copying/custom-tool filters unchanged. Logs:
`/tmp/sketchpad_checkpoint46_lib_final.log`,
`/tmp/sketchpad_checkpoint46_editor_gate.log`,
`/tmp/sketchpad_checkpoint46_numeric_gate.log`, and
`/tmp/sketchpad_checkpoint46_coverage_gate.log`. All exited 0; `git diff --check`
is clean. Exact animation timing and newly observed polygon-point cache words
remain outside this checkpoint's verified parity.


Native's translated-Y control additionally proves that dependent square Y units
use the live horizontal step; their saved Y step is a cache. Shared geometry now
follows that dependency, while rectangular units retain independent steps. A
visible derived-point regression checks Native coordinates, horizontal edits,
unchanged cached metadata, strict save/reopen and whole-action undo/redo.


## Checkpoint 47: shared iteration output adapters

Generic carried-DAG bindings now expose circles, polygons, rays, lines and arcs
through the existing element/transform model. No shape-by-transform Kind products
were added. The corpus recovers 389 drawing bindings across 102 files plus one
prototype slot: 390 slots total, leaving 54,072 unsupported. All 1,599 originals
remain byte-exact, zero imports fail, and 95 documents have every kind decoded.
Final scan: `/tmp/sketchpad_corpus_checkpoint47_final.log`.

The Iterate command accepts a selected point mapping and supported geometry,
including derived point seeds and optional live Scalar depth. The adapters share
SVG/GPU drawing, picking, independent binding styles, final-level selection and
undefined-image isolation. Dependencies of derived seeds are carried; omitted
external hosts remain frozen. Circle-by-radius uses typed Length quantities as
well as defining segments. Original multipage calculation/table validation now
compares embedded references in page-local space while retaining global Kind IDs.

Scoped gates pass 700 library, 121 editor, 34 numeric-import and 4 coverage tests
(859 total); copying and custom-tool checks remain excluded. All gates exited 0.
Logs: `/tmp/sketchpad_checkpoint47_lib_final.log`,
`/tmp/sketchpad_checkpoint47_editor_gate.log`,
`/tmp/sketchpad_checkpoint47_numeric_gate.log`, and
`/tmp/sketchpad_checkpoint47_coverage_gate.log`.
See [binding audit and regressions](iteration_bindings47_analysis.md) and
[Native controls](iteration_bindings47_native_analysis.md). Decoding recovery does
not establish runtime completeness for every corpus iteration. Wider unsupported
parent roots and non-Scalar seed roles remain priorities; pixel-identical output,
copying and custom tools remain deferred.


## Checkpoint 48: analytic arcs and shared visibility actions

The common transform carrier now accepts analytic arcs across all fourteen wire
flavors, retaining source dependencies, directed parameterization, reflection,
signed scale, and undefined zero-scale ancestors. Linear three-point arcs retain
arc construction identity. Editing, picking, Point on Object, measurements and
mapped iteration consumers use the same geometry adapters. No shape-by-operation
Kind products were added. Actual coverage recovers 489 drawing slots and 14
prototype slots; the prior dependency audit's reach counts were upper bounds.
See [arc evidence](transformed_arcs48_native_analysis.md).

Hide/Show options 0–7 retain their native words. Fixed modes 0–3 assign one shared
visibility without changing the next action; alternating modes 4–7 change it
once per click. Caption format and visibility words are retained, custom format 7
is distinguished from automatically generated captions, and hidden Hide/Show
children can execute through Presentation. Full-group caption patches handle
changing byte lengths, optional label insertion and page-local references.
This recovers 3,448 drawing slots and 46 prototype slots. See [visibility evidence](visibility_options48_analysis.md), including Native frame/font normalization boundaries.

Combined unsupported slots fall from 54,072 to 50,075: 3,997 recovered slots.
All 1,599 original files remain byte-exact, with 96 completely decoded documents
and zero failures. Final scan: `/tmp/sketchpad_corpus_checkpoint48_final.log`.
Copying and custom-tool support remain deferred. Exact raster matching is outside
the requested scope; this checkpoint does not establish full command parity.


Final scoped gates exited 0: 720 library, 124 editor, 34 numeric-import and four
coverage tests (882 total). Copying and custom-tool checks remain excluded.
Logs: `/tmp/sketchpad_checkpoint48_lib_final.log`,
`/tmp/sketchpad_checkpoint48_editor_gate.log`,
`/tmp/sketchpad_checkpoint48_numeric_gate.log`, and
`/tmp/sketchpad_checkpoint48_coverage_gate.log`.
The 13 analytic-arc tests and seven visibility tests are included in the library
count; the editor count includes all fourteen arc commands and hidden-child /
fixed / alternating Presentation dispatch. `git diff --check` is clean.

## Checkpoint 49: shared dependency roots

Stored Point construction application (code101), coordinate-axis constraints
(code15/subtype38), and general point/polygon value measurements (code94) restore
4,321 slots compared with checkpoint48: 4,315 drawing slots and six prototype
slots. Root-code reductions are 865 code101, 285 code15 and 134 code94; shared
parents also unlock 1,405 calculations, 94 iteration bindings and other dependents.
These are actual full-scan changes, rather than overlapping reach estimates.
All 1,599 original documents save byte-identically, with zero scan failures.

The construction application is one generic definition with ordered dependency
references and scoped input substitution. It preserves live dependencies and
freezes omitted external nodes, including under an enclosing iteration context.
Affine transforms continue to use the common element/matrix-sequence evaluator.
Point seeds are Native-proved; numeric seeds and other output types remain held.
No custom-tool authoring or invocation is introduced.

Axis constraints use the independently measured fixed 200-pixel parameter span,
with creation, dragging and Value of Point sharing that normalization. Coordinate
equations and plots retain their coordinate-unit bases. Polygon point values use
live boundary projection and perimeter; the editor accepts constrained points or
a point/polygon pair in either selection order. Native controls and live edit,
undo/redo, deletion/remapping, malformed-packet and save/reopen tests accompany
these paths. See [mapped construction evidence](mapped_construction49_native_analysis.md),
[axis evidence](graph_axis_point49_native_analysis.md), and
[polygon value evidence](numeric_roots49_analysis.md).

Final scoped validation: 907 passing checks (743 library, 126 editor, 34 numeric
imports and four coverage tests). Copying/custom-tool cases are excluded. Scan
log: `/tmp/sketchpad_corpus_checkpoint49_final.log`.


## Checkpoint 52: shared path host identities and coordinate points

The final compiled corpus scan restores 6,893 drawing slots relative to
checkpoint51, with prototype support unchanged. Unsupported total decreases
37,584→30,691 (29,843 drawing +848 prototype). All1,599 originals remain
byte-exact, zero files fail, fully decoded documents increase340→355, and
fully decoded drawing documents347→362. Partial views decrease1,321→1,310;
that flag also retains multi-page structural editing restrictions. Final scan:
`/tmp/sketchpad_corpus_checkpoint52_final.log`.

Point-on-object constraints now recognize the proved native Parallel46 and
Perpendicular47 host subtypes while using their existing live parameter frames.
Polygon constraints validate the loweru16 edge index and preserve six opaque
bytes rather than misreading style/cache remnants as a u32 edge. Single-page
and multiple-page public dragging controls reopen in Native byte-exactly and
place points at independently checked coordinates. Generic polygon subtype15
and Bisector44 parameter frames remain deferred; Native bisector measurements
contradict the current unit-direction approximation.

Coordinate helpers109/110 use one CoordinatePointDefinition with shared
NumericExpr operands and live coordinate-frame evaluation: fixedx/livey or
livex/fixedy. Native original profiles and public authoring APIs prove nonzero
fixed coordinates, negative live values and unequal rectangular frame scales.
Source formatting, transforms and mapped iteration bindings reuse that same
point type. Stored literals/roles and opaque records survive edits and strict
save/reopen. The Scalar operand guard remains bounded; other units and rotated
frames require independent evidence.

Measured drawing reductions include calculations48:3,732→2,537 (1,195),
iteration bindings77:4,213→2,883 (1,330), value-depth roots89:1,338→850 (488),
helpers109:560→18 (542), helpers110:638→20 (618), and constrained points15:
1,420→911 (509). These are actual full-scan differences, not sums of overlapping
root-reach estimates. Final scoped gates all exited0:790 library,131 editor,34 numeric-import and
four coverage checks (959 total), excluding deferred copying/custom-tool cases.
Logs: `/tmp/sketchpad_checkpoint52_lib_final3.log`,
`/tmp/sketchpad_checkpoint52_editor_final.log`,
`/tmp/sketchpad_checkpoint52_numeric.log`, and
`/tmp/sketchpad_checkpoint52_coverage.log`. `git diff --check` is clean.

Evidence: [geometry prerequisites](geometry_roots52_analysis.md),
[host identity and public dragging](path_host_subtypes52_native_analysis.md),
[polygon constraint writer](polygon_constraint52_api_native_analysis.md),
[coordinate API](coordinate_points52_api_native_analysis.md), and
[coordinate model and dependency roots](mapped_iteration_roots52_analysis.md).
Copying and custom tools remain deferred; object decoding does not establish
complete command parity or pixel-identical output.


## Checkpoint 53: numeric locus dependencies and inherited transformed points

The compiled full corpus scan restores 466 drawing slots relative to checkpoint52:
unsupported total30,691→30,225 (29,377 drawing +848 prototype). Prototype decoding
is unchanged. All1,599 originals remain byte-exact, zero files fail, fully decoded
documents355→365, fully decoded drawing documents362→374, partial views1,310→1,301.
Log: `/tmp/sketchpad_corpus_checkpoint53_final.log`.

Ordinary code35 locus ancestry now includes typed numeric nodes, preserving exact
reference order and the duplicated target. PointValue→Calculation→numeric-dilation
point curves and primitive-segment sweeps recompute under each driver sample.
Independent Native controls verify full curves before/after source movement and
sweep geometry. Target and driver families are otherwise unchanged. Typed numeric
ancestry alone does not establish every code35/97 range or swept-element profile.

Vector-transformed points inheriting native class1/subtype89 reuse the existing
TransformDefinition and live evaluator. The decoder checks the source's actual
saved subtype, exact roles and packet shape; it does not accept arbitrary points
under that inherited header. Independent mapped-source and chained controls retain
own styles and preserve packets across editing and undo.

Bisector path geometry now uses the proved fixed coefficient15sqrt10 multiplying
the sum of unit arms. Both canonical PointOnRay13 and native host44 share that
parameter frame; dragging preserves each original subtype on single and multiple
pages. Zero arms and opposite arms leave the path undefined. Valid PointDistance
and straight-path PointValue observers retain their definitions when the geometry
becomes undefined; invalid parent roles still fail edited-save validation.
Native independent-marker calculations amplify geometric error by1e9 to distinguish
the coefficient from the rejected1.25cm approximation. Actual public API
Bisector single/multi-page dragging and undefined-source saves reopen in Native
byte-exactly (2,728/4,000/2,728bytes). Public construct_locus controls for point,
edited point and primitive-segment sweep also save exactly there
(2,976/2,984/3,188bytes). Logs: `/tmp/bisector53_api_native.log`,
`/tmp/bisector53_api_native_remaining.log`, `/tmp/locus_numeric53_api_native.log`.

Measured drawing changes: loci35:1,886→1,714 (172), fixed dilations30:665→551 (114),
vector translations16:1,840→1,769 (71), bindings77:2,883→2,835 (48), inherited roots89:
850→828 (22), constrained points15:911→900 (11), calculations48:2,537→2,528 (9).
These are full-scan differences, not sums of overlapping dependency estimates.
Final scoped gates pass972 checks:803 library,131 editor,34 numeric import and4
coverage, excluding copying/custom-tool work. Logs:
`/tmp/sketchpad_checkpoint53_lib_final.log`,
`/tmp/sketchpad_checkpoint53_editor.log`,
`/tmp/sketchpad_checkpoint53_numeric.log`,
`/tmp/sketchpad_checkpoint53_coverage.log`. `git diff --check` is clean.

Evidence: [numeric locus](locus_numeric53_native_analysis.md),
[inherited transformed points](inherited_point_transform53_native_analysis.md),
[bisector parameters](bisector53_native_analysis.md),
[remaining geometry prerequisites](geometry_roots53_analysis.md).
Copying and custom tools remain deferred. This is a coverage checkpoint, not a
complete command-parity or pixel-parity claim.


## Checkpoint 54: retained locus metadata and shared straight transforms

The final compiled full scan restores 1,120 slots relative to checkpoint53:
1,118 drawing objects and two prototype objects through shared decoding. Total
unsupported decreases 30,225→29,105 (28,259 drawing +846 prototype). All 1,599
originals remain byte-exact, zero files fail, complete documents increase 365→377,
and complete drawing documents 374→386. Partial views decrease 1,301→1,290.
Log: `/tmp/sketchpad_corpus_checkpoint54_final2.log`.

Ordinary point loci retain either proved word32 value (0 or1), with the final
word zero. Captured ancestor lists preserve their original body order. A shared
matcher in decoding and edited-save validation requires the exact target/path/
driver prefix, final target, list length and complete ancestor multiset. Missing,
duplicated, unrelated, forward and cross-page dependencies remain rejected.
Canonical new authoring keeps its previous sorted references and flag1. Five
field-only Native controls retain flags and order byte-exactly and positively
paint the live baseline/edited curves; three actual public-move API files also
cross-open byte-identically. These metadata guards restore exactly 636 loci
(1,714→1,078), rather than the overlapping 2,087-slot potential reach estimate.

Native ray/line vector translation, reflection and fixed-polar translation now
reuse the generic TransformDefinition. Arc and straight outputs share operation
packet parsing in `src/document/transform_decode.rs`, with strict output classes,
canonical subtypes, source extent, backward live same-page references, payload
lengths and literal/quantity guards. Primitive rays/lines, Parallel, Perpendicular,
Bisector and supported transformed chains share source geometry and matrix
sequences. Zero straight direction leaves the full chain undefined; valid typed
records survive source edits and can recover after undo. A zero segment can still
retain its finite segment geometry. Native point observers and separate support
samples prove both parameter frames and directions. GraphAxis remains excluded:
its eight-stage probe has no positive Native open/save result, so no geometry
conclusion or source activation is claimed.

Desktop fixed-polar/vector translation and reflection now retain ray/line extent
through the common append path. Existing fixed and numeric rotation/dilation
consumers accept the new supported straight chain sources. Other translation
flavors and point-angle straight rotation remain guarded. No shape-by-operation
Kind variants or constructor aliases were added.

Measured drawing reductions include loci35:636; reflection34:106; center arcs80:
64; calculations48:41; vector translation16:41; bindings77:37; intersections11/12:
33 each; numeric rotation29:25; constrained points15:18; polar translation21:17.
These are actual full-scan differences, not predicted descendant unions.
Final scoped gates all pass: 811 library, 133 editor, 34 numeric import and four
coverage checks (982 total), excluding copying/custom-tool work. Logs:
`/tmp/sketchpad_checkpoint54_lib_final2.log`,
`/tmp/sketchpad_checkpoint54_editor_final2.log`,
`/tmp/sketchpad_checkpoint54_numeric_final2.log`,
`/tmp/sketchpad_checkpoint54_coverage_final2.log`. `git diff --check` is clean.

Evidence: [locus frontier](locus_frontier54_analysis.md),
[Native retained metadata and public moves](locus_metadata54_native_analysis.md),
[straight source packets](straight_transform_sources54_analysis.md),
[Native straight chains](straight_transform54_native_analysis.md).
Copying/custom tools remain deferred, and pixel-identical output is outside the
current goal. This checkpoint does not establish full command parity.


Final API proof: all eight generic straight author/edit controls reopen and save
byte-identically in Native (4,178–4,198bytes). Independent checking passes28 point
samples and56 support samples; the public source-degeneration control retains all
records while suppressing paths, points and observer captions. Three locus metadata
public edits/restoration controls likewise save exactly, preserving captured order
and flags. Final strict schema rejection also prevents a malformed canonical
ray/line output from falling through to a legacy point/circle interpretation.
Native logs: `/tmp/straight_transform54_api_native.log`,
`/tmp/locus_metadata54_api_native.log`. Detailed evidence includes the bounded
GraphAxis failure and checker color/occlusion corrections, without pixel-parity claims.


## Checkpoint55: terminal mapped points and analytic arc intersections

The final rebuilt scan recovers2,560 drawing slots relative to checkpoint54,
with no prototype count change. All1,599 originals retain byte-exact round trips;
392 documents have all object kinds decoded (401 for drawing kinds), zero fail,
and1,278 retain a partial-view restriction. Unsupported drawing slots are25,699;
846 unsupported prototype slots remain separate. Final compiled log:
`/tmp/sketchpad_corpus_checkpoint55_final2.log`.

Actual drawing reductions include calculations48:458; vector transforms16:244;
dilation30:243; terminal point aliases88:192; iteration bindings77:167; marked
angle rotation32:109; reflection34:100; straight/arc intersections11:93 and12:47;
polygons8:74; straight transforms27:72; center arcs80:71; numeric dilation33:69;
numeric rotation29:68. These are final histogram deltas, not summed dependency
reach predictions. The point and arc features share predecessors, so attribution
by isolated family is not inferred from this combined scan.

An intrinsic IterationPointAlias refers to a supported single-branch point
binding. The common mapped executor retains undefined terminal values internally,
while ordinary binding rendering continues to filter each undefined image.
Numeric zero returns the original source output; negative depths are undefined.
Terminal evaluation retains caller substitutions and freezes external boundaries
in that scope. Styles, dependency remapping, main/multi-page saving and existing
point consumers reuse the point model. Fresh API creation uses valid metadata;
old cross-document wire controls retain their original invalid font selector
and are correctly partial, without weakening font checks.

Straight/arc intersections reuse LineCircleIntersection and the analytic circle
root kernel. Native evidence distinguishes circle fixed branches from arc
branches: arc sweep and straight extent filtering precede selection, signed
orientation and ordered straight parameters select two hits, and a sole hit
belongs to both codes. Linear three-point arc limits reuse bounded straight
intersection. No shape-by-operation variants or sampled curve approximations
were added. Desktop Intersection accepts straight/arc pairs in either order.
Already typed13/14 arc-parent objects remain a documented geometry gap awaiting
independent branch proof. General curve98 hints and polygon93 intersections
remain next broad candidates; copying and custom tools stay deferred.

Final scoped gates pass994 checks:821 library,135 editor,34 numeric import,
and four coverage tests. Logs are `/tmp/sketchpad_checkpoint55_lib_final.log`,
`/tmp/sketchpad_checkpoint55_editor_final.log`,
`/tmp/sketchpad_checkpoint55_numeric_final.log`, and
`/tmp/sketchpad_checkpoint55_coverage_final.log`. New focused tests cover Native
positions, strict malformed roles/payloads, live edits, undefined recovery,
undo/deletion, scoped evaluation and retained multi-page groups.

Evidence: [fresh dependency frontier](numeric_shared_roots55_analysis.md),
[terminal point controls](iteration_endpoint55_native_analysis.md),
and [analytic intersections with Native branch observers](ANALYSIS.md#round-arc-intersections).


All34 actual Native55 controls complete byte-exact saves:20 arc controls,
seven retained original-layout endpoints and seven direct public API endpoints.
The20 direct API arc documents match their independently Native-saved control
bytes exactly. Separate observer checks establish both coincident branches and
undefined endpoints. Seven API endpoint files use valid formatting and strict
editable core reopens; their five aliases and two valid prefix samples match
actual Native positions at0px. All Native and checker queues are complete.
`git diff --check` is clean.


## Checkpoint 56: directed polygon events and inherited scalar dilation

The refreshed prerequisite audit selected code93 boundary points and inherited
point-subtype89 Scalar dilation33. Their initial direct-ready cohorts reached
1,520 and993 distinct unsupported descendants respectively; those overlapping
unions are audit priorities, not additive recovery counts.

One intrinsic `BoundaryIntersection(straight, polygon)` construction preserves
the source Segment/Ray/Line identity. Native93 searches strictly forward along
the ordered carrier, ignores its ordinary finite/negative extent, skips the
origin, and chooses the nearest positive boundary event. Collinear spans
contribute their endpoint events. Concave, repeated, retraced, reversed-winding,
transformed and edited polygons use the shared straight intersection kernel.
Typed undefined results and observer segments retain their definitions through
save/reopen and undo; zero-ray geometry remains undefined. Unproved coordinate
axis and abstract tool hosts remain guarded. Polygon authoring permits repeated
vertex runs when at least three distinct IDs remain. The multi-page writer uses
page-local parents and rejects cross-page references.

Twenty-four actual Native controls save exactly, with20 defined red results
matching at0px,60 observer samples within0.708px, and four independently painted
undefined controls. Twenty-four independently authored public API documents
match those complete Native files byte-for-byte, including repeated/retraced
refs and zero geometry; no packet grafting is used in the API generator.
See [retained runtime and multi-page evidence](ANALYSIS.md#polygon-boundary-intersections).

The inherited Scalar dilation adapter reuses the existing generic transform
definition; no transform-product Kind or writer is added. Actual source
subtype89, same-page backward live operands, Scalar coefficient and absence
of07d3 are required. Canonical creation remains subtype4; imported subtype89
records preserve their identity and point style through edits. Six actual
Native controls prove canonical/inherited equivalence, edits, frozen inputs
and positive/negative ratios. A seventh one-byte visibility discriminator
shows the zero-ratio output as the red center point, independently of the
blue center that previously occluded it. Quantity dilation and inherited
rotation remain unproved. See [numeric frontier](numeric_shared_roots56_analysis.md)
and [Scalar Native evidence](inherited_numeric_transform56_native_analysis.md).

The fresh full scan recovers1,779 drawing slots and seven prototype slots
incidentally through shared decoding:26,545→24,759 total unsupported. Drawing
reductions include355 boundary points,233 vector transforms,194 calculations,
187 numeric dilations,154 polygons and65 iteration bindings. Complete documents
increase392→425 and complete drawing documents401→435. All1,599 originals
remain byte-identical with zero failures. Prototype decoding improvements
do not implement custom-tool invocation; copying and custom tools stay deferred.

Final scoped gates pass1,004 tests:829 core,137 editor (including its PDF check),
34 numeric imports and four coverage tests. Copying/custom-tool filters match
the prior checkpoint. A separate public API check verifies new second-page
boundary creation, local references and cross-page rejection. Logs are
`/tmp/sketchpad_corpus_checkpoint56_final.log`,
`/tmp/sketchpad_checkpoint56_{lib_final,editor_final2,numeric_final,coverage_final}.log`,
and `/tmp/boundary56_multipage.log`. The release command gate remains unmet;
code98 curve hints and the typed13/14 arc-parent geometry gap remain separate
frontiers for new Native proof.

## Checkpoint 57: point curves, round-pair arcs and long calculations

The finalized56 fresh audit selected code98 point-curve intersections as a shared
prerequisite:230 ready roots reached1,937 distinct unsupported slots, an ancestry
estimate rather than a promised recovery count. The intrinsic ranked crossing
construction now evaluates FunctionPlot and point-target geometric/parameter
traces against straight, circular and arc carriers. Refinement evaluates actual
source points, rejects pole bridges and tangencies, preserves undefined sources,
and applies the separately proved source endpoint/carrier ordering rules. Strict
native payload and page-local role guards remain in place. See
[curve packets and tests](curve_intersections57_analysis.md) and
[Native controls](curve_intersection57_native_analysis.md).

Existing13/14 constructions additionally evaluate circular and linear arc
parents through shared geometry kernels. No additional shape/operation variants
are introduced. See [round-pair adapter](round_intersections57_analysis.md).
The long calculation reader/writer resource bound is4,096 tokens; source count,
exact metadata, expression schema and units remain guarded. See
[numeric audit](numeric_shared_roots57_analysis.md) and
[Native long calculations](calculation_tokens57_native_analysis.md).

Native evidence covers65 curve saves (57 defined and eight undefined controls),
18 round-pair saves and six long-calculation saves, all byte-identical. Direct
public API reconstruction additionally matches61 canonical curve files and all
18 round-pair files. Optional empty curve metadata remains preserved on import;
three reversed-role curve negatives remain opaque, and two swept-Segment inputs
remain unsupported (one crashed the Native application).

The final scan reduces unsupported24,759→23,415, entirely within drawing objects:
1,344 recovered, including code48 calculations304, code98 roots239, code33
dilations220 and code16 coordinate points214. Drawing-complete documents435→444;
all-kinds-complete425→434. The prototype namespace is unchanged at839 unknown.
All1,599 originals still preserve their complete bytes. The scan log is
`/tmp/sketchpad_corpus_checkpoint57_final.log`; API checks are
`/tmp/curve_intersection57_api_generate.log` and
`/tmp/circle_arc57_api_generate.log`. Copying and custom tools remain skipped.

Final scoped checks pass1,019 tests:841 core,140 editor,34 numeric import and
four corpus checks. Filters retain the requested copying/custom-tool exclusions.
The newly authored second-page intersection test retains imported curve sources;
it does not expand the existing single-page restriction on new plot authoring.
Final logs are `/tmp/sketchpad_checkpoint57_{lib_final,editor_final,numeric_final,coverage_final}.log`;
all complete with exit0. The library/probe build also completes with exit0 at
`/tmp/sketchpad_checkpoint57_final_build.log`. These checks remain a progress
checkpoint, not completion of the release command requirements.


Checkpoint58 recovers 2,337 drawing slots through shared constrained and numeric
points on point-target loci, the proved numeric-path hidden alias, and inherited
nine-reference Polygon identities. The measured reductions include371 code95,
368 code15,361 numeric coordinate points,302 numeric dilations,118 calculations
and58 point values. All1,599 originals remain byte-identical with zero failures;
embedded prototype counts are unchanged. Complete decoded documents rise434→445
and complete drawing documents444→455. Scoped gates pass1,035 tests (856 core,
141 editor,34 numeric andfour coverage). Copying and custom tools remain deferred.
See [curve point implementation](path_curve_points58_analysis.md),
[Native curve semantics](path_curve_points58_native_analysis.md),
[header and polygon evidence](path_identity58_native_analysis.md), and
[inherited polygon implementation](polygon_identity58_analysis.md).


Checkpoint59 restores2,267 drawing slots and four prototype slots through shared geometry decoding, without custom-tool authoring changes. General ArcOnCircle endpoints, intrinsic SectorBoundary/Point68 and common straight path points account for the verified new prerequisites. Unsupported totals21,078→18,807; complete drawing documents455→475. All1,599 originals remain byte-identical with zero failures. Scoped gates pass1,052 tests (869 core,145 editor,34 numeric,four coverage), with copying and custom tools skipped. The final corpus log is `/tmp/sketchpad_corpus_checkpoint59_final.log`. Gate logs: `/tmp/sketchpad_checkpoint59_lib_final.log`, `/tmp/sketchpad_checkpoint59_editor.log`, `/tmp/sketchpad_checkpoint59_numeric.log`, `/tmp/sketchpad_checkpoint59_coverage.log`; the fresh public out-of-domain PointValue save regression and suffix guards also pass in `/tmp/sketchpad_checkpoint59_straight_final.log`.

Native retained controls:14 ArcOnCircle,8 SectorBoundary and38 straight paths, all whole-file exact with independent geometry/observer checks. Public API14 arcs and36 ordinary straight controls match these independently saved files entirely; all eight fresh Sector API files also independently open/save byte-exact in Native with positive geometry checks, establishing the actual writer despite its different ordinary measurement template. See [arc evidence](arc_sources59_native_analysis.md), [sector implementation](sector_boundary59_analysis.md), and [straight evidence](value_straight_paths59_native_analysis.md). Pixel-identical rendering remains outside the requested scope, and decoding does not imply full command parity.

The checkpoint has68 independently exact Native files (14 arcs,8 retained sectors,38 straight paths,8 fresh API sectors). Four actual HTM exports are retained as incomplete predecessor references: [export evidence](native_html59_export_analysis.md). A fresh read-only frontier audit is recorded in [next prerequisites](numeric_shared_roots60_analysis.md).


## Checkpoint 60: intrinsic Circle fill and function-path points

The final compiled scan restores516 drawing slots and five prototype slots. Unsupported totals18,807→18,286; drawing-complete documents475→484 and all-kind complete465→474. All1,599 originals retain entire file bytes, with zero failures. Main measured restorations: CircleInterior10=399, point15=48, calculation48=14, vector16=14, reflection34=9, numeric65=8. These are actual scan deltas, distinct from overlapping prerequisite reach counts.

One intrinsic `CircleInterior(source)` retains its own identity/style/visibility/order and follows a live analytic Circle even when the source is hidden. Modern28-byte records, including page-local writes and shared fills/picking/snapshots, are supported; the two16-byte legacy interiors remain opaque because Native never reached a window/save/capture for that control. Function15/58 points reuse `PointOnCurve`, evaluate finite normalized stored parameters outside the displayed range, return canonical x for PointValue, and clamp nearest-curve interactive projection to the displayed interval. A12-byte constraint guard separates function subtype58 from the existing20-byte nine-vertex polygon subtype58. Numeric95/123 function points and disk transforms remain separate unproved work.

Independent Native proof: Function13 static whole-file exact saves plus two interactive drags, Circle8 modern exact saves, and independent analytic paint/observer checks. Fresh public API generation matches all21 static goldens entirely. Scoped gates pass1,062 tests (877core,147editor,34numeric,fourcoverage), copying/custom tools skipped. Final logs: `/tmp/sketchpad_checkpoint60_lib_final2.log`, `/tmp/sketchpad_checkpoint60_editor_final3.log`, `/tmp/sketchpad_checkpoint60_numeric_final.log`, `/tmp/sketchpad_checkpoint60_coverage_final.log`, `/tmp/sketchpad_checkpoint60_final_build.log`, `/tmp/sketchpad_corpus_checkpoint60_final.log`; API proofs: `/tmp/function_points60_api_final.log`, `/tmp/circle_interior60_api_final.log`.

Evidence: [Function rules](function_points60_native_analysis.md), [Circle wire/paired references](circle_interior60_analysis.md), [Circle Native identity/order](circle_interior60_native_analysis.md). The sampled mapped102 and expanded locus35 controls/design are separate pending work; no production admission is justified by this checkpoint's counts.


## Checkpoint 61: shared sampled DAG and locus drivers

Intrinsic code102 uses one `SampledMappedDefinition(output,path,seed,carried,native_range)` and the existing scoped dependency evaluator. Segment, Circle, curved Arc and collinear ThreePointArc profiles preserve omitted external constructions as frozen boundaries while carried constructions follow the seed. Literal15/90 children reuse PointOnCurve: finite Segment fractions extrapolate, Circle fractions wrap, and curved/linear Arc fractions remain bounded. Scalar95 and literal123 instead consume normalized fractions strictly within [0,1] on every proved sampled path. PointValue94 is established only for literal15/90 children; no inverse measurement policy is inferred for95/123. Samples above4096 remain valid;4096 limits only presentation.

Code35 locus construction now accepts existing PointOnCurve, Polygon and Sector constrained drivers, using the same analytic/perimeter driver replay for rendering and path points. Coarse/dense off-grid controls rule out chord interpolation. The independently proved89 prefix word2=1 is admitted without widening other flags or iteration constraints. Copying and custom-tool authoring remain skipped.

The final compiled full scan recovers1,386 drawing objects:102=406,35=249,16=112,98=73,8=72,15=71,48=67,77=46,95=40. Unsupported totals18,286→16,900, with the830 prototype unknowns unchanged. Complete drawings484→529; all-kind complete474→519. All1,599 originals retain entire file bytes and no scan fails. This is observed recovery, not overlapping descendant predictions. Log: `/tmp/sketchpad_corpus_checkpoint61_final.log`.

Scoped gates pass1,077 tests (891 core,148 editor,34 numeric import,four coverage), excluding requested copying/custom-tool work. Final logs: `/tmp/sketchpad_checkpoint61_lib_final2.log`, `/tmp/sketchpad_checkpoint61_editor_final.log`, `/tmp/sketchpad_checkpoint61_numeric_final.log`, `/tmp/sketchpad_checkpoint61_coverage_final.log`, `/tmp/sketchpad_checkpoint61_final_build.log`.

Independent Native evidence comprises8 sampled DAG controls,12 driver controls,4 prefix controls and12 numeric/literal follow-ups, all36 whole-file exact with analytic/observer checks. Public API8 DAG and12 locus files match their independently saved goldens entirely. The additional12 public numeric/literal files match every geometry record; their fresh94 measurement format defaults differ explicitly from the Arc-based donor templates, so whole-file equality against those older controls is not claimed. Four fresh combined API controls have now independently cross-opened and saved whole-file exactly in Native. Their decoded RGB images also equal the four previously proved combined controls entirely, including measurement captions; the separate queue and checker exit0. The checkpoint therefore retains40 independently exact Native files. API final logs: `/tmp/sampled_mapped61_api_final.log`, `/tmp/locus_drivers61_api_final.log`, `/tmp/sampled_mapped61_numeric_api.log`; fresh Native log: `/tmp/sampled_mapped61_numeric_api_native.log`. See [sampled implementation](sampled_mapped61_analysis.md), [sampled Native evidence](sampled_mapped60_native_analysis.md), [child parameters](sampled_mapped61_numeric_native_analysis.md), [locus implementation](locus_drivers61_analysis.md), [locus Native proof](locus_drivers60_native_analysis.md), [prefix proof](mapped_prefix60_native_analysis.md). These checks do not establish remaining release command parity.


## Checkpoint 62: crate boundaries and shared point-curve drivers

The authorized workspace split separates lossless format framing, generic object/transform IR and derived dependencies, the scoped document engine, and desktop editing/rendering. A headless probe omits production GUI dependencies. The migration-only scan retained checkpoint61 coverage exactly; the subsequent FunctionPlot58/SampledMapped90 locus-driver extension restores99 drawing slots, including57 loci (44 directly ready roots plus13 further shared-path constructions).

Function source intervals and sampled carried/frozen graphs remain authoritative. Literal and numeric outer queries retain separately proved domains, including outer sampled-Circle numeric wrapping while direct95->102 remains bounded. Both open and closed Circle metadata preserve exact originals; authoring emits the independently proved closed profile. Ten Native controls are entire-file exact;120 independent query roles and identical open/closed RGB captures confirm evaluation. Nine public-API authored documents match Native saves entirely without record grafting.

Final complete scan: `/tmp/locus_point_sources62_corpus.log`, exit0, all1,599 byte exact,524 all-kind complete,534 drawing complete,16,801 unsupported slots (15,971 drawing+830 prototypes). Workspace gates: `/tmp/locus_point_sources62_workspace_tests.log`, exit0,1,143 passing executions, including30 duplicated renderer assertions; seven new driver regressions pass. Headless build and both API checkers exit0. Copying/custom tools remain skipped; remaining release command parity is not established. See [implementation](locus_point_sources62_analysis.md), [Native evidence](locus_point_sources62_native_analysis.md), and [crate separation](crate_layers62_analysis.md).


## Checkpoint 63: shared straight-family locus targets and hosts

Geometric-driver loci accept transformed and Colorized Segment/Ray/Line targets and generic finite Segment hosts through the shared transform/scoped dependency evaluator. Independent sample strokes carry their pen and per-sample color into SVG and canvas; picking preserves carrier extent and leaves fan gaps empty. Direct Colorized targets override locus color per sample, while ordinary transformed Colorized ancestors retain the locus color, matching the independent Native control. Unknown axes/tool carriers and unproved shape profiles remain excluded. Unsaved dependent deletion now avoids evaluating removed constructions before compaction; unsupported surviving plot/function remapping remains an explicit atomic refusal.

Actual compiled recovery is118 drawing slots, including94 loci. All1,599 originals remain whole-file exact, all-kind complete524→531, drawing complete534→542, zero failures, unsupported16,801→16,683 (15,853 drawing+830 prototypes). Static93-root/185-reach predictions are not recovery counts. Final scan: `/tmp/locus_generic_straights63_corpus.log`, exit0.

Workspace gates pass1,155 executions (32 duplicated renderer assertions), headless build passes, and all12 final-library public API outputs match their actual Native saves entirely. Native independently opens/saves30 exact documents and checks two174-witness geometry/color sets plus pen/extent fields. Hints3/4/5 all draw four Native intervals; no general sampling formula or raster identity is claimed. Copying/custom tools remain skipped; full release command parity remains unproven. See [implementation and gates](locus_generic_straights63_implementation.md), [Native evidence](locus_generic_straights63_native_analysis.md), and [independent review](locus_generic_straights63_analysis.md).


## Checkpoint64 live rich references and generated names

Actual compiled recovery is1,378 drawing slots:379 code73 and999 code90. All1,599 originals remain whole-file exact; all-kind complete531→562, drawing complete542→573, zero failures, unsupported16,683→15,305 (14,475 drawing+830 prototypes). Final scan `/tmp/referenced_shared64_corpus_final.log` exits0. Initial2,654 overlapping reach is not a restoration count.

The shared role-aware AST/evaluator supports selected live text/function/numeric A/B/E meanings, fraction content and generic point-anchored text. Native names use active demands in object/operand order and0967; runtime/history assignments survive hiding and dirty saves materialize07d5. All three final-library public text writer controls match Native saves entirely. Workspace1,172 executions and the headless build pass. Eight additional public Name controls pass strict physical-label/writer checks and independent Native whole-file exact saves with actual painted name/value witnesses. Copy/custom tools remain skipped and full release command parity stays open. See [implementation and gate evidence](referenced_shared64_implementation.md).

## Native rich-caption symbol and layout expansion

The caption-checkpoint scan reports 1,228/1,599 documents with every object decoded,
1,258 drawing-complete documents, 880 partial views, and 3,471 unsupported slots
(2,859 drawing plus 612 embedded-tool prototype slots). All 1,599 files still
round-trip byte-for-byte with no failures.

This is a measured reduction of 259 unsupported slots from the preceding 3,730
baseline. Direct code-73 captions fall 180→14; the other restored slots are
their already-supported descendants. The parser now covers the observed symbol
font, numeric signed-term role `D`, `O1`–`O5` geometry decorations, centered
vertical layout, trailing five-field frame metadata, and observed source-specific
names, definitions, expressions, and geometry values. A strict whole-markup GBK
fallback decodes the legacy Chinese clock literals without misreading valid UTF-8.
An exhaustive audit shows that all 14 remaining code-73 objects parse and satisfy
their source-role contracts, but depend on parent object kinds that remain
unsupported; cached display text is not promoted to live semantics.

## Parameter-driven geometric loci

The current full scan reports 1,243/1,599 documents with every object decoded,
1,273 drawing-complete documents, 877 partial views, and 3,363 unsupported slots
(2,753 drawing plus 610 embedded-tool prototype slots). All 1,599 files remain
byte-exact with no failures.

The shared parameter sampler now evaluates subtype-43 code-97 families whose
target is a validated straight, filled region, or point-output locus collection.
All 67 recovered records use decoded parents, an actual Parameter driver, exact
captured dependencies, and the already-supported native range/unit payload.
Runtime auditing exercises every recovered family: 7 straight sweeps, 13 filled
region sweeps, and 47 nested curve collections. Direct unsupported code-97
objects initially fall 122→55.

The calculated-driver pass admits the exact native descriptor profiles for
scalar and degree ParameterCalculation objects. It recovers 22 direct loci (20
drawing and two prototype records) plus 19 drawing descendants. All 22 enter
their runtime sampler: the eight-step probe produces strokes for 18 and retains
undefined target gaps for four whose current target geometry is also undefined.
Fresh scalar and degree controls cover subtype-42 points, subtype-43 segments,
save/reopen, and degree range editing. Unsupported drawing code 97 falls 55→35
and prototype code 97 falls 9→7; length drivers and unsupported-parent variants
remain closed.

## Arc and sector Radius measurements

The current full scan reports 1,248/1,599 documents with every object decoded,
1,278 drawing-complete documents, 875 partial views, and 3,266 unsupported slots
(2,656 drawing plus 610 embedded-tool prototype slots). All 1,599 files remain
byte-exact with no failures.

Code 46 now reuses the live round-geometry radius for supported arcs and sectors.
The corpus contains five CenterArc measurements and one SectorBoundary
measurement; all six have the established numeric format/style layout and return
finite positive values in the runtime audit. They restore 91 supported
descendants for 97 drawing slots total. The editor and multi-page writer can
author the same Radius measurement for supported circular arcs and sectors;
undefined circular constructions retain their measurement DAG just as existing
Arc Length and Arc Angle values do.

## Segment Ratio measurements

The current full scan reports 1,253/1,599 documents with every object decoded,
1,283 drawing-complete documents, 871 partial views, and 3,190 unsupported slots
(2,580 drawing plus 610 embedded-tool prototype slots). All 1,599 files remain
byte-exact with no failures.

Code 47 now accepts the corpus-observed two-reference form when both parents are
supported finite segments. It evaluates the first segment's live length divided
by the second's and retains the existing three-point signed-projection form.
All eight recovered roots return finite positive values in the runtime audit;
they restore 68 descendants for 76 drawing slots total. Existing tests cover
endpoint edits, single- and multi-page save/reopen, and editor authoring without
adding a new test function.

## Sector interior name metadata

The current full scan reports 1,256/1,599 documents with every object decoded,
1,286 drawing-complete documents, 871 partial views, and 3,111 unsupported slots
(2,513 drawing plus 598 embedded-tool prototype slots). All 1,599 files remain
byte-exact with no failures.

Code 82 now admits the observed optional `07d5`/`07d8` name and text-style pair
after validating both records. As with existing CircleInterior metadata, it is
preserved but not promoted to an editable geometric label. The pass restores 16
direct drawing sectors and 51 drawing descendants, plus 12 direct sectors in
tool-local prototypes: 79 slots total. The strict source, header, reference,
marker, page, and dependency guards are otherwise unchanged. An existing sector
test now also decodes, evaluates, and exactly saves a recovered corpus example;
no new test function was added.

## Circular-segment interiors and measurements

Code 83 now accepts both observed circular-segment presentation profiles: the
existing plain group and the validated optional name/style or empty-marker
records. The live region uses its source arc plus endpoint chord. Shape Area and
Perimeter evaluate analytically, and the editor and both writers can author the
same measurements. Six direct interiors, two Area measurements, and three
existing descendants restore 11 drawing slots. Unsupported falls 3,111→3,100;
all-kind-complete documents rise 1,256→1,259 and drawing-complete documents rise
1,286→1,289. All 1,599 originals remain byte-exact.

## Circle-and-endpoints Arc Length

Native code 44 with three references is represented separately from a constructed
arc: the first parent is a circle and the next two are point directions. Its live
value is the circle radius times the smaller unsigned angle between those
directions. The shared writer expands the existing code-44 template to three
references, and the editor accepts circle/start/end selection order on single-
and multi-page sketches. The corpus contains 12 roots in two documents; seven
currently evaluate to finite positive values and five retain structurally valid
but undefined upstream geometry. They unlock 61 descendants for 73 restored
drawing slots. Unsupported falls 3,100→3,027, all-kind-complete documents rise
1,259→1,260, drawing-complete documents rise 1,289→1,290, and partial views fall
871→870. Prototype coverage stays at 598 unsupported slots and all 1,599
originals remain byte-exact.

## Colorized sector measurements

The shared sector predicate now admits transformed and Colorized sectors for
native Area, Perimeter, and Arc Length records. The compiled scan restores six
drawing slots: one direct code-42 Area and five supported descendants. The
intermediate corpus checkpoint has 3,021 unsupported slots, 1,261 all-kind
complete documents, and 1,291 drawing-complete documents. All 1,599 originals
remain byte-exact.

## Transformed-point Animation buttons

Code-62 Animation buttons may target a supported transformed point with ordinary
point settings. The editor uses the existing scoped evaluator to place the target
temporarily, so dependent geometry follows during playback without rewriting the
transformation, dirtying the document, adding an undo entry, or changing saved
bytes. Stopping the track removes the override and restores its constructed
position.

Native GSP was exercised under Xvfb/Wine on `自由运动二（李章博）.gsp`: object 66
and its dependents moved while object 1 retained its exact free coordinates in a
save made during playback; Stop returned object 66 to its original constructed
position. The compiled scan restores 32 drawing code-62 buttons. Unsupported
falls 3,021→2,989 (2,391 drawing plus 598 prototype), complete documents rise
1,261→1,262, complete drawings rise 1,291→1,292, and partial views fall 870→869.
All 1,599 originals remain byte-exact with zero failures. Final scan:
`/tmp/sketchpad_transformed_animation_coverage.log`.

## Segment markers

Native code-121 ordinary marker groups now resolve a single earlier supported
Segment host, including transformed segments, and retain the stored finite
parameter and one-, two-, or three-tick class. Desktop and SVG rendering use the
native fixed fourteen-unit tick length and six-unit spacing verified visually in
Sketchpad 5 under Xvfb/Wine. Its flagged `0x00010001` profile draws a native
ten-unit arrow in the host direction or its reverse, selected by the final field;
both orientations were verified in native multi-page documents. The same stroke
style, visibility, picking, dependency remapping, and single-/multi-page writers
cover imported and copied objects.

The decoder deliberately excludes the one arrow attached to a sampled curve.
The compiled scan restores 27 drawing slots. Unsupported falls 2,989→2,962
(2,364 drawing plus 598 prototype), complete documents rise 1,262→1,268,
complete drawings rise 1,292→1,298, and partial views fall 869→867. All 1,599
originals remain byte-exact with zero failures. Final scan:
`/tmp/sketchpad_segment_markers_final_coverage.log`.

## Legacy angle-value rotations

Native code 119 is the GSP4 form of a rotation driven by a separate angle-value
object. Its references are `[source, center, angle value]`; the corpus uses the
same degree-valued `VertexAngleValue` semantics as modern code 29. Exact plain
groups, plus the one observed labeled-point profile, now enter the shared live
`NumericRotation` model for supported Point, Segment, and analytic Arc sources.
Untouched files preserve code 119 byte-for-byte; copied objects use the modern
code-29 writer.

Seven direct roots and three dependents recover. The six remaining code-119
objects depend on an unsupported legacy angle source, transformed interior, or
circular-segment boundary and remain unsupported. The compiled scan lowers the
total 2,962→2,952 (2,354 drawing plus 598 prototype), with document-completion
and partial-view counts unchanged. All 1,599 originals remain byte-exact. The
16-page `课时7笔记.gsp` opened and resaved byte-identically (387,588 bytes) in
Sketchpad 5 under Xvfb/Wine. Final scan:
`/tmp/sketchpad_legacy_rotation_coverage.log`.

## Legacy polar coordinate values

Native codes 125 and 124 are respectively the radius and polar angle of a
point relative to a coordinate system. Their meaning does not change with the
displayed grid form. Code 125 evaluates to `hypot(x, y)` as a scalar. Code 124
evaluates internally in degrees, then its caption follows the document's angle
unit: degrees with a `°` suffix or radians without one. The strict decoder
requires the exact scalar-measurement layout and references to an earlier
supported Point and CoordinateSystem.

The original `圆环的迭代(5环旋转迭代).gsp` displays `r_z = 5.71` and
`θ_z = 0.04` in its radian document. A port-authored code-125/code-124 pair
displayed `r_A = 1.36` and `θ_A = 5.60°` in Sketchpad 5 and resaved
byte-identically at 2,796 bytes. Two direct roots and six dependent code-48
calculations recover. Two code-124 roots and one code-125 root that depend on
unsupported code-88 aliases remain guarded. The compiled scan lowers the total
2,952→2,944 (2,346 drawing plus 598 prototype), with document-completion and
partial-view counts unchanged. All 1,599 originals remain byte-exact. Final
scan: `/tmp/sketchpad_legacy_polar_coordinates_final_coverage.log`.

## Inch-unit fixed rectangular translations

Native code-17 fixed-Cartesian payloads use length-unit tag 1 for centimeters
at 37.79527559055118 pixels per unit and tag 2 for inches at 96 pixels per
unit. The shared decoder now validates each axis against its own declared unit,
the transform model retains both scales, and the writer restores the matching
tags when an object is copied or authored.

The 18-page `初中数学积件.gsp` contains ten visible Segment translations with
inch tags on both axes. Each has magnitude `0.03937007983525594` and pixel cache
`3.7795276641845703`; all ten now evaluate live and remain byte-exact. A
port-authored `(0.5, -0.25)`-inch Segment translation displayed the expected
48-pixel right and 24-pixel down displacement in Sketchpad 5 and resaved
byte-identically at 2,088 bytes. The compiled scan lowers unsupported slots
2,944→2,934 (2,336 drawing plus 598 prototype), with completion and partial-view
counts unchanged. All 1,599 originals remain byte-exact.

## Mixed translated circular segments

Code 19 with class 13/subtype 69 is a circular-segment region translated by a
marked horizontal Length and a fixed vertical centimeter literal. The two corpus
files each contain one intrinsic circular-segment root followed by 21 live images;
once the first image is typed, the full recursive chain decodes through the shared
affine evaluator. Arc-plus-chord fill geometry, drawing order, style, picking,
area, and perimeter all use the transformed region rather than treating it as a
sector.

The first pass admitted this independently observed fixed-vertical flavor. A
port-authored source/image pair
opened in Sketchpad 5 under Xvfb/Wine and displayed the expected 150-pixel right,
37.79527559055118-pixel up offset. Its generated code-19 header is class
13/subtype 69, and the port save/reopen preserves the live marked-distance
dependency. The compiled scan restores all 44 drawing slots across the two
`方轮小汽车` files: unsupported falls 2,934→2,890 (2,292 drawing plus 598
prototype), complete documents rise 1,268→1,270, and complete drawings rise
1,298→1,300. Partial views remain 867, and all 1,599 originals remain byte-exact.
See `circular_segment_translation_analysis.md`.

## Fixed circular-segment rotation, dilation, and reflection

The same class-13/subtype-69 region appears with code 27 fixed rotation, code 30
fixed dilation, and code 34 reflection. Fourteen roots have already decoded
Point or straight-object operands; their ordinary constraint payloads match the
shared transform schemas. Admitting those three native flavors also resolves
recursive circular-segment chains while retaining the arc-plus-chord geometry.

An isolated 13-object writer places a source region beside its 180-degree
rotation, reflection, and half-scale dilation. Sketchpad 5 opened it under
Xvfb/Wine and rendered all four filled regions with the expected orientation,
placement, and radius. Existing editor commands now author exactly these verified
fixed forms; marked-angle and marked-ratio variants remain guarded.

The compiled scan restores 37 drawing slots: unsupported falls 2,890→2,853
(2,255 drawing plus 598 prototype). Completion and partial-view counts remain
1,270 all-kind documents, 1,300 drawing-complete documents, and 867 partial
views. All 1,599 originals remain byte-exact.

## Coordinate-axis rotations

Two native constructions use a class-2/subtype-12 Line image whose source is an
ordinary form-1 `GraphAxis`: one code-27 fixed rotation and one code-29 numeric
rotation. The numeric image then feeds another numeric rotation. The decoder,
evaluator, validator, document API, and editor now share this narrow rotation-only
source rule; translation, dilation, reflection, point-angle rotation, y-axis, and
other axis profiles remain closed.

A 10-object, 2,750-byte port writer created 30-degree fixed and negative-45-degree
calculation-driven images of an x-axis. Sketchpad 5 opened it under Xvfb/Wine and
rendered both as full infinite lines at the expected angles. Save/reopen in the
port preserves both live definitions.

The compiled scan restores 23 drawing slots: the two axis-source roots and 21
descendants. Unsupported falls 2,853→2,830 (2,232 drawing plus 598 prototype),
complete documents rise 1,270→1,272, complete drawings rise 1,300→1,302, and
partial views fall 867→865. All 1,599 originals remain byte-exact.

## Pixel-unit fixed rectangular translations

Code-17 fixed rectangular constraints use tag 0 for literal pixels, alongside
the previously supported tag-1 centimeters and tag-2 inches. Seven point images
in `初中数学积件.gsp` use tag 0 on both axes, with each pixel cache exactly equal
to its stored magnitude. Their page-local sources are supported points on
segments, circles, or an arc.

The decoder, save validator, and writer now preserve scale 1 and emit tag 0. A
1,956-byte port writer translated a point by `(120, −80)` pixels. Sketchpad 5
opened it under Xvfb/Wine and displayed the image exactly 120 screen pixels right
and 80 screen pixels down from its source.

The compiled scan restores all seven drawing slots. Unsupported falls
2,830→2,823 (2,225 drawing plus 598 prototype); completion and partial-view
counts remain 1,272 all-kind documents, 1,302 drawing-complete documents, and
865 partial views. All 1,599 originals remain byte-exact.

## Legacy short-header circle interiors

The legacy clock contains two code-10 class-5/subtype-16 circle interiors with
16-byte GSP4 headers. Their reference groups are otherwise the ordinary
one-Circle disk layout. The short header stores RGB but has no opacity or drawing
rank field, matching legacy polygons: Native renders the implicit fill at 75%
and no rank is invented while the original group is retained.

Both disks now keep live analytic circle geometry and `0xbfff` implicit opacity.
The legacy-upgrade path recognizes the same exact header if a future complete
legacy document needs conversion to modern records. Sketchpad 5 opened the
original clock under Xvfb/Wine and visibly rendered the overlapping filled disks
above the dial. The compiled scan restores two drawing slots, lowering
unsupported 2,823→2,821 (2,223 drawing plus 598 prototype). Completion and
partial-view counts are unchanged because that clock retains unrelated legacy
objects. All 1,599 originals remain byte-exact.

## Literal points on ordinary point loci

Four native code-123 class-1/subtype-56 groups reference already-supported
ordinary point-output `Locus` objects. They use the existing strict literal-path
record layout and store normalized fractions `1` or `2/3`. The decoder now uses
the shared point-curve evaluator for this exact host family; intrinsic
`SampledMapped` literals retain their previous support, while `ParameterLocus`
and FunctionPlot literal contracts remain guarded.

An existing locus regression now covers authoring, live evaluation, save, and
reopen without adding another test case. A 2,456-byte port-authored midpoint
locus with a literal point at `2/3` opened and rendered at the expected position
in Sketchpad 5 under Xvfb/Wine. Native Ctrl+S preserved the file exactly (SHA-256
`23c703216253a74ef1459ee9114190c027f4cd1ad507a931174a235d92f915eb`).

The four roots unlock 72 descendants, restoring 76 drawing slots. Unsupported
falls 2,821→2,745 (2,147 drawing plus 598 prototype); all-kind-complete documents
rise 1,272→1,274 and drawing-complete documents rise 1,302→1,304. Partial views
remain 865, all 1,599 originals remain byte-exact, and there are zero failures.

## Degree-valued fixed-coordinate points

Three remaining native code-110 fixed-y points use Degrees calculations as their
live x coordinate. One is a nonzero control: `(t[4] · 180°)` evaluates to `360°`.
The other two are duplicate sine/cosine sketches with degree-valued live sources.
This is consistent with Native numeric XY points, which use the displayed degree
magnitude as the coordinate rather than converting it to radians.

`CoordinatePointDefinition` now retains the live operand's unit and admits only
Scalar or Degrees sources whose declared unit matches the referenced object.
Evaluation uses the ordinary numeric magnitude. Length, Area, and higher units
remain rejected. A public writer control with `t=0.05`, `t·30°=1.5°`, and a
fixed-y point encodes and reopens as a complete 10-object document; the optional
Native launch was not counted because the local WoW64 wrapper became unavailable.

The three roots unlock 82 descendants, restoring 85 drawing slots. Unsupported
falls 2,745→2,660 (2,062 drawing plus 598 prototype); all-kind-complete documents
rise 1,274→1,276 and drawing-complete documents rise 1,304→1,306. Partial views
remain 865, all 1,599 originals remain byte-exact, and there are zero failures.
Full scan: `/tmp/sketchpad_degree_coordinate_full_coverage.log`.

## Legacy coordinate-pair rich-text tails

Twenty decoded-parent code-49 Cartesian coordinate-pair readouts use the same
class-0/subtype-34 group already supported by the shared measurement model. Their
`0x08fc` payloads consistently store an inner size two bytes shorter than the
record payload, leaving an opaque two-byte cache tail. Payload lengths range from
84 to 100 bytes and the tail varies, so it is preserved rather than interpreted.
The existing line/circle equation parser already recognizes exactly this Native
`0 | 2` size delta.

The coordinate parser now shares that bounded rule. All twenty ready sources are
supported points: numeric-coordinate, curve-intersection, iteration-alias, or
colorized point objects. The one remaining code-49 object retains an unsupported
code-69 point parent and remains opaque. The existing coordinate regression now
also covers the legacy tail, evaluated `(0, 0)` result, and exact whole-file
round trip without adding another test case.

Actual recovery is 21 drawing slots: unsupported falls 2,660→2,639 (2,041
drawing plus 598 prototype); all-kind-complete documents rise 1,276→1,277 and
drawing-complete documents rise 1,306→1,307. Partial views remain 865, all 1,599
originals remain byte-exact, and there are zero failures. Full scan:
`/tmp/sketchpad_legacy_coordinate_tail_full_coverage.log`.

## Value of Point on sampled paths and sectors

Twenty-two decoded-parent code-94 records measure points against ordinary loci,
parameter loci, function plots, sampled mappings, or a sector boundary. The
sampled path families now use the existing nearest-point evaluator and convert
the normalized result to each path's established logical domain. A matching
PointOnSector returns its exact stored physical-perimeter parameter. Structurally
valid measurements survive currently undefined curve geometry.

The common host predicate now drives import, validation, and editor selection.
The existing point-value regression covers a six-measurement locus file and a
sector file; no standalone test case was added. Twenty-two roots and 143
dependents restore 165 drawing slots. Unsupported falls 2,639→2,474 (1,876
drawing plus 598 prototype); all-kind-complete documents rise 1,277→1,285,
drawing-complete documents rise 1,307→1,315, and unsupported documents fall
322→314. Partial views remain 865, all 1,599 originals remain byte-exact, and
there are zero failures. Eight code-94 objects with unsupported point or axis
parents remain opaque. Full scan:
`/tmp/sketchpad_curve_point_values_full_coverage.log`.

The supplied Windows Sketchpad 5 executable opened a freshly authored
sector-boundary Value-of-Point file under Xvfb/Wine and saved it without changing
its 2,294 bytes or SHA-256
`5091d091477ae3f18f883be1b57823abf4b10506feccfbbb30a628fbf4efeb23`.
This is an open/save writer check, not a pixel-parity claim.

The next code-90 pass removes a redundant source whitelist from anchored rich
captions and relies on the same role validator used by ordinary referenced text.
Coordinate-pair captions use the ordinary two-decimal fallback when the scalar
precision field is absent, and nested point numeric/referenced labels resolve
through their live caption value. Exact `?3` and `?4` containers are accepted
only inside an outer reference's cached presentation tree; they do not create
new live operands, and other nested reference-cache modes remain rejected.
Thirty-two direct drawing slots are restored: unsupported falls 2,474→2,442
(1,844 drawing plus 598 prototype), complete documents/drawings rise
1,285/1,315→1,305/1,335, unsupported documents fall 314→294, and partial
views fall 865→864. All 1,599 originals remain byte-exact. Eighteen drawing
code-90 records remain blocked or outside the proved source/cache grammar.
Full scan: `/tmp/sketchpad_calculation_cache_labels_full_coverage.log`.

Mapped collection admission now treats point-output `ParameterLocus` objects the
same as ordinary point-output `Locus` objects. Both already use the same local
driver sampler and the same outer scoped evaluator; the decoder, binding guard,
and new-binding template now agree with those shared runtime paths. Ten code-89
roots, 25 code-77 bindings, and eight dependent objects restore 43 drawing
slots. Unsupported falls 2,442→2,399 (1,801 drawing plus 598 prototype),
complete documents/drawings rise 1,305/1,335→1,314/1,345, unsupported documents
fall 294→285, and partial views fall 864→860. All 1,599 originals remain
byte-exact with zero failures. Full scan:
`/tmp/sketchpad_parameter_locus_iteration_coverage.log`.

For the Native control, the direct `ParameterLocus` in
`双控旋梯（hjx4882）.gsp` was hidden while its mapped binding remained visible.
Sketchpad 5 still displayed the red five-level spiral; the port SVG displayed
the same bound spiral. Native Ctrl+S retained all 4,532 bytes with SHA-256
`53270fd8b9997ed80942028d8fdfa9d424097e1da0c2ccf51e66924242dfb14f`.
This isolates collection-binding visibility rather than relying only on an
untouched-source round trip.

## Capture-only actions in mapped iterations

Hide/show, animation, and movement buttons can be carried by a Native mapping
even though actions themselves are not repeated output values. The evaluator now
accepts these objects only as dependency-capture nodes: it traverses their target
references while building the scoped graph, but `mapped_binding_kind` continues
to reject action bindings. This keeps ordinary action activation semantics
unchanged while allowing the mapped geometry in the same carried graph to run.

Twenty-one code-89 roots, 147 code-77 geometry bindings, and eight dependent
objects restore 176 drawing slots. Unsupported falls 2,399→2,223 (1,625 drawing
plus 598 prototype), complete documents/drawings rise
1,314/1,345→1,326/1,358, unsupported documents fall 285→273, and partial views
fall 860→856. All 1,599 originals remain byte-exact with zero failures. Full
scan: `/tmp/sketchpad_iteration_action_capture_coverage.log`.

The Native control is derived from `未命名迭代钮炳坤老师.gsp`. Every ordinary
source object is hidden, leaving only its five iteration bindings visible.
Sketchpad 5 and the port both display the recovered red point sequence and green
arc sequence. Native Ctrl+S preserves all 5,780 bytes with SHA-256
`730248f8ed12e9c959e2811d7631174773438d360b83e3beb1bb86bf9a7276d2`.

## Mapped iteration value tables

Native code-92 tables now retain an ordered list of carried numeric/function
values instead of the former single-calculation special case. Import validates
that every displayed value belongs to its mapped iteration, preserves the
table anchor, optional caption, font, visibility, and presentation marker, and
renders the initial row plus generation-numbered live rows. Branching and
final-image-only mappings retain their actual generation indices. Copy/remap,
deletion, dirty save, anchor movement, and newly authored point/parameter
iteration companions use the same generalized representation.

All 42 decoded-parent code-92 drawing slots are restored. Unsupported falls
2,223→2,181 (1,583 drawing plus 598 prototype), complete documents/drawings rise
1,326/1,358→1,346/1,381, unsupported documents fall 273→253, and partial views
fall 856→843. All 1,599 originals remain byte-exact with zero failures. Full
scan: `/tmp/sketchpad_iteration_values_coverage.log`.

The supplied Sketchpad 5 displayed the `N!.gsp` table with columns `n`,
`t₁ + 1`, and `s·(t₁ + 1)`, ending at `5 | 7.00 | 5040.00`; its Ctrl+S save
remained 4,802 bytes. Port SVG evaluation produced the same six value rows.
An out-of-tree edit probe moved the table anchor, saved, reopened, and retained
both the anchor and all rows. No standalone regression test was added.

## General native snapshot tables

Native code-91 tables now retain two to seven ordered numeric or referenced-text
sources, rich headers, one to 21 stored rows, their anchor, optional caption,
font reference, visibility, and opaque alignment bytes. The shared rich-text
decoder presents subscripts, superscripts, units, formulas, Unicode headers,
and cached undefined values; save and dependency remapping keep the original
wire records rather than rebuilding their presentation.

All 40 decoded-parent code-91 drawing slots are restored. Unsupported falls
2,181→2,141 (1,543 drawing plus 598 prototype), complete documents/drawings rise
1,346/1,381→1,358/1,393, unsupported documents fall 253→241, and partial views
fall 843→840. All 1,599 originals remain byte-exact with zero failures. Full
scan: `/tmp/sketchpad_general_tables_coverage.log`.

Sketchpad 5 and the port both display the nine-row coordinate-history table in
`利用制表功能快速获取点的坐标.gsp`. A second Native check establishes the
zero-state `抛豆实验估算圆周率.gsp` table, including its cached undefined π
cell; Native Ctrl+S preserves both source files byte-for-byte. No standalone
regression test was added.

## General imported custom polar translations

Native code-26 objects with the strict `[source, Degrees, Length]` reference
shape now use the shared polar-translation evaluator for both Point and Segment
outputs. The decoder accepts the exact unlabelled group and the observed native
label/font group, so labels remain visible and byte-preserved. A code-26 point
may also be the endpoint of a later code-102 sampled mapping: the point keeps its
own polar construction while the mapping independently consumes it.

All 20 typed drawing code-26 records in this cohort are restored, together with
75 downstream slots. Unsupported falls **2,141→2,046** (1,448 drawing plus 598
prototype), complete documents/drawings rise **1,358/1,393→1,369/1,404**,
unsupported documents fall **241→230**, and partial views fall **840→839**. All
**1,599** originals remain byte-exact with zero failures. Nine drawing code-26
objects with different scalar operand profiles and five embedded-tool instances
remain opaque pending separate semantic evidence. Full scan:
`/tmp/sketchpad_code26_general_polar_coverage.log`.

Sketchpad 5 under Xvfb/Wine opened and saved representative labelled free-source,
nested-source, trace-linked, and Segment files byte-for-byte. The labelled point
control independently matches its 60.27891° and 0.6028 cm displacement in Native
and the port; the Segment control confirms Native accepts the same typed opcode.
No standalone regression test was added.

## General Plot Value sources and paths

Native code-95 Plot Value points now consume every decoded numeric magnitude,
not only dimensionless scalars. Length, Area, and Degrees sources retain their
units for display while their finite magnitude drives an axis, straight path,
circle, polygon, arc, or sampled path. Function-plot hosts interpret the source
as a live x-domain value. Sector-boundary hosts use the same normalized physical-
perimeter parameter as ordinary sector points; the symmetric code-123 literal
form shares that path evaluator.

The compiled corpus restores **22 code-95 roots**, **one code-123 sector root**,
and **67 dependents**. Unsupported falls **2,046→1,956** (1,358 drawing plus 598
prototype), complete documents/drawings rise **1,369/1,404→1,380/1,415**,
unsupported documents fall **230→219**, and partial views fall **839→834**. All
**1,599** originals remain byte-exact with zero failures. The remaining ten
drawing code-95 records depend on still-opaque path parents; 16 embedded-tool
instances remain separate. Full scan:
`/tmp/sketchpad_code95_general_sector_coverage.log`.

Native Xvfb/Wine controls show Area and Length magnitudes at their numeric axis
coordinates and a scalar at x=1.57 on its function plot. A freshly authored
Length-on-axis file and a fresh sector file containing both live and literal
points each reopen and save byte-for-byte in Sketchpad 5. The port reproduces
the same function point and sector-locus geometry. No standalone test case was
added.

## Duplicate carried references in mapped iterations

Some native code-89 packets repeat a prerequisite or branch image in the
carried-reference tail. The tail describes a construction DAG rather than
multiple output instances, so import now keeps the first occurrence of each ID
before applying the existing mapped-DAG validator. The original reference bytes
remain in the preserving source records. Public construction still rejects
duplicate carried inputs, and no evaluator, renderer, or binding rule was
broadened.

The compiled scan restores **16 code-89 definitions** and **59 code-77
bindings**. Unsupported falls **1,956→1,881** (1,283 drawing plus 598 prototype),
complete documents/drawings rise **1,380/1,415→1,391/1,426**, unsupported
documents fall **219→208**, and partial views fall **834→831**. All **1,599**
originals remain byte-exact with zero failures. Full scan:
`/tmp/code89_dedup_coverage.log`.

Sketchpad 5 under Xvfb/Wine opened and saved the 8,184-byte
`磁砖拼花（系列）.gsp` control byte-for-byte (SHA-256
`80bca845cf2a3a1a2243aedd6de1c5b01b796fc8dbb37c6a4e721a16f9dc0c1c`).
Its native view and the port SVG show the same repeated blue-square and green-arc
tiling. No standalone test case was added. See
[analysis](mapped_iteration_duplicate_carried_analysis.md).

## Nested mapped-iteration capture

Native mappings may carry an earlier mapped root and its code-77 bindings as
structural scope descriptors. They are not themselves outer output values. The
decoder now accepts that relationship only when the inner root and binding are
earlier, valid, on the same page, the bound source belongs to the inner carried
DAG, the root is explicitly present in the outer packet, and every inner node
that depends on an outer seed is completely captured. Other carried code-77
roles remain rejected.

The compiled scan restores **11 code-89 definitions** and **48 code-77
bindings**. Unsupported falls **1,881→1,822** (1,224 drawing plus 598 prototype),
complete documents/drawings rise **1,391/1,426→1,401/1,436**, unsupported
documents fall **208→198**, and partial views fall **831→826**. All **1,599**
originals remain byte-exact with zero failures. The ready code-89 frontier falls
from 46 roots/376 reachable slots to 35 roots/317 reachable slots. Full scan:
`/tmp/sketchpad_code89_nested_capture_coverage.log`; frontier:
`/tmp/sketchpad_frontier_after_code89_nested_capture.log`.

Sketchpad 5 under Xvfb/Wine displays the `堆砌钢管.gsp` progression as piles
labelled 3, 9, 18, 30, and 45. The port evaluates the four mapped labels as
9/18/30/45 and produces 102 distinct mapped circles plus the three source
circles. Native saves all 5,682 bytes unchanged (SHA-256
`06f33ecd8f6d40995f7d3cdc872199c9f251e02c2413ff2daeee2d018837f5dc`).
No standalone test case was added. See
[analysis](mapped_iteration_nested_capture_analysis.md).

## Shortened segment locus drivers

One native code-35 locus names a full source segment while its moving point is
constrained to a shortened collinear segment which omits a singular endpoint.
The decoder now admits only the proved shared-endpoint/fixed-dilation form and
samples the moving point on its live shortened host. Arbitrary transformed and
iterated locus drivers remain opaque.

The sandglass root and nine downstream drawing slots are restored. Unsupported
falls **853→843**, complete documents/drawings rise
**1,495/1,536→1,496/1,537**, and unsupported documents fall **104→103**. All
**1,599** originals remain byte-exact with zero failures; prototype coverage is
unchanged. Native and port views show the same rail and magenta trace, and the
Native control saves byte-for-byte with SHA-256
`71369f4f5895f4038c42bf9693cd0867be08c65d30aafa157f919b662f975d06`.
No standalone test case was added. See
[analysis](shortened_segment_locus_analysis.md). Full scan:
`/tmp/shortened_segment_locus_coverage.log`.

## Points on parametric curves

Native code-15 subtype-99 points use a normalized fraction on Cartesian
parametric plots. The shared point-curve evaluator now evaluates both component
functions in the saved frame; the same host admission also restores four
circle/parametric code-98 intersections through the existing ranked crossing
kernel. Plot/plot crossings and live/literal Plot Value admission are unchanged.

Five point roots, four curve intersections, and three dependents are restored:
unsupported falls **843→831**, while complete documents/drawings remain
**1,496/1,537** and partial views remain **802**. All **1,599** originals remain
byte-exact with zero failures; prototype coverage is unchanged. Native `Show
All Hidden` confirms the points on the plotted outline, and the original control
saves byte-for-byte with SHA-256
`6cea4c774f57b8dda6bd6623be6ce549071d5c9714f8fd28a9615c34dfd2035b`.
No standalone test case was added. See
[analysis](parametric_curve_points_analysis.md). Full scan:
`/tmp/parametric_curve_points_coverage.log`.

## Function-plot intersections

Native code-98 intersections between two same-frame `YFromX` plots now use a
dedicated proper-crossing kernel ordered by increasing x. It retains the existing
high-hint clamping policy, rejects tangencies, overlaps, and pole bridges, and
freezes invariant referenced-function inputs once per calculation. Other
curve-pair profiles remain opaque.

All ten ready code-98 roots and four dependents are restored. Unsupported falls
**831→817**, complete documents/drawings rise
**1,496/1,537→1,499/1,540**, unsupported documents fall **103→100**, and partial
views remain **802**. All **1,599** originals remain byte-exact with zero
failures; prototype coverage is unchanged. A fresh `y=x`/`y=x²` control opens
in Native with intersections at `(0,0)` and `(1,1)` and saves byte-for-byte with
SHA-256
`dd57fcda4bd6b5ee1ab7df7114a77cf3ca92d913f09bf771894cc1fe4badfc0f`.
No standalone test case was added. See
[analysis](function_plot_intersections_analysis.md). Full scan:
`/tmp/function_plot_intersections_coverage.log`.

## Movement and Scroll action state words

Native Movement and Scroll buttons may retain state word `1` in the final word
of their otherwise standard 24-byte action payload. The strict readers now
accept only the observed values `0` and `1`, preserve imported state through
edits, and continue authoring/copying with state zero.

The two action roots and one dependent caption are restored. Unsupported falls
**817→814**, code-62 drawing objects fall **31→29**, complete
documents/drawings rise **1,499/1,540→1,501/1,542**, unsupported documents fall
**100→98**, and partial views remain **802**. Embedded-tool coverage is
unchanged and all **1,599** originals remain byte-exact. Both source files also
open and save byte-for-byte in Sketchpad 5, with SHA-256 values
`6e88700d4461eb5e8f2663ebba77c4aff1271b9fbd28b2acbf60a45e78514c9c` and
`ce52fadb3378082b737a546fb9073a0679b1bda43d67df2b287b03d3a9eac928`.
No standalone test case was added. See
[analysis](action_state_words_analysis.md). Full scan:
`/tmp/action_state_coverage.log`.

## Parametric-plot loci

Native code-35 loci driven by a code-15 point on a Cartesian parametric plot use
the same normalized `0..1` open-curve profile as function-plot loci. The locus
classifier now admits that path kind through the already established shared
point-curve evaluator. Other path profiles and unsupported plot intersections
remain opaque.

Thirteen loci and eight dependent code-98 intersections are restored.
Unsupported falls **814→793**, complete documents/drawings rise
**1,501/1,542→1,503/1,544**, unsupported documents fall **98→96**, and partial
views remain **802**. Embedded-tool coverage remains **4,766/5,356**, and all
**1,599** originals remain byte-exact. Native `Show All Hidden` and the port
render expose the parametric construction, and the original Native control
saves byte-for-byte with SHA-256
`6cea4c774f57b8dda6bd6623be6ce549071d5c9714f8fd28a9615c34dfd2035b`.
No standalone test case was added. See
[analysis](parametric_plot_locus_analysis.md). Full scan:
`/tmp/parametric_locus_coverage.log`.

## Legacy circular-segment rotations

Four code-119 class-13/subtype-69 groups rotate supported circular segments
using the already validated legacy three-reference numeric-rotation layout.
The circular-segment transform guard now includes that opcode while retaining
the existing strict header, record, reference, ordering, page, and degree-unit
checks. Two code-119 point rotations with unsupported sources remain opaque.

Unsupported falls **793→789**, complete documents/drawings rise
**1,503/1,544→1,504/1,545**, unsupported documents fall **96→95**, and partial
views remain **802**. Embedded-tool coverage remains **4,766/5,356**, and all
**1,599** originals remain byte-exact. Native and port views show the same
overlapping circular-segment construction; an untouched Native open/save is
byte-identical with SHA-256
`e773a15dfe848aef87c4c644ab2ec85f0981fbcd4a0799cbcfbde23e0a5bdc12`.
No standalone test case was added. See
[analysis](legacy_circular_segment_rotation_analysis.md). Full scan:
`/tmp/circular_segment119_coverage.log`.

## Transformed cropped pictures

Eleven code-29 numeric rotations retain a live cropped-picture source, and one
downstream code-16 vector translation retains that transformed crop. The
dedicated model composes both the pixel transform and clipping polygon while
requiring the exact observed picture-cache layout. SVG export also follows the
shared Native picture/filled-region rank and applies crop polygons in screen
space.

Unsupported falls **789→777**, complete documents/drawings rise
**1,504/1,545→1,506/1,547**, unsupported documents fall **95→93**, and partial
views remain **802**. Embedded-tool coverage remains **4,766/5,356**, and all
**1,599** originals remain byte-exact. Native and port renders show the same
yellow/gray car composition; an untouched Native open/save is byte-identical
with SHA-256
`40d7b467ec4cfed98af88d36c4a4c258502c5134802e5214d9ee1124f32ffb73`.
No standalone test case was added. See
[analysis](transformed_cropped_pictures_analysis.md). Full scan:
`/tmp/transformed_crop_final_full_coverage.log`.

## Circular-segment boundary point

The observed code-15 class-1/subtype-10 point with constraint subtype 69 now
retains a live circular-segment boundary host. Its unit parameter traverses the
arc and then the closing chord by physical perimeter; exact arc/chord projection
supports dragging without depending on presentation sampling.

The single root is restored. Unsupported falls **777→776**, complete
documents/drawings rise **1,506/1,547→1,507/1,548**, unsupported documents fall
**93→92**, and partial views fall **802→801**. Embedded-tool coverage remains
**4,766/5,356**, and all **1,599** originals remain byte-exact. The port render
places the fixture's late-parameter point on the chord close to A, and the
untouched source round-trips with SHA-256
`73b2ae197c314e0b1d0aefb55e0f8c0841536766af63ad9e811bd39e2a273a6f`.
No standalone test case was added. See
[analysis](circular_segment_point_analysis.md). Full scan:
`/tmp/circular_segment_point_coverage.log`.

## Drawing-function expression cache

Code-122 functions defined from a drawing retain that drawing as a transparent
PNG and use the established alpha-top-edge evaluator. Their third opaque
expression cache field occupies bytes 60–63; the reader had accidentally
ignored the preceding fixed word and required this cache instead. Correcting
the four-byte window preserves all structural and evaluability checks.

The single drawing-function root is restored. Unsupported falls **776→775**,
complete documents/drawings rise **1,507/1,548→1,508/1,549**, unsupported
documents fall **92→91**, and partial views remain **801**. Embedded-tool
coverage remains **4,766/5,356**, and all **1,599** originals remain
byte-exact. The untouched fixture round-trips with SHA-256
`88555d49ece2998b3e28b40b640fd5a382eff82bab76a7c5085b68b4df168ebc`.
No standalone test case was added. See [analysis](data_function_analysis.md).
Full scan: `/tmp/drawing_function_cache_coverage.log`.

## Legacy signed-angle values

Two code-114 class-18/subtype-95 objects retain a live signed shortest angle
from three points. A real Sketchpad 5 Show All Hidden capture confirms the
negative turn used by the dependent legacy code-119 rotations; a separate
untouched Native open/save is byte-identical. The distinct semantic kind keeps
code 114 separate from ordinary angle markers and positive directed angles,
while supporting degree evaluation, marker presentation, remapping, and an
exact writer.

The two values, two rotations, and two downstream loci are restored.
Unsupported falls **775→769**, complete documents/drawings rise
**1,508/1,549→1,510/1,551**, unsupported documents fall **91→89**, and partial
views fall **801→800**. Embedded-tool coverage remains **4,766/5,356**, and all
**1,599** originals remain byte-exact. The smaller Native control saves with
SHA-256
`e4fc80205c12fd460f8ce849a09bd8ca55aa7e57e2c2166e3ba56c703ae5ab20`.
No standalone test case was added. See
[analysis](signed_angle114_analysis.md). Full scan:
`/tmp/signed_angle114_coverage.log`.

## Sampled custom-transformed pictures

Eighteen code-96 class-6/subtype-91 objects now retain a source picture and the
supported custom point-mapping DAG that warps its pixels. A bounded textured
mesh drives canvas rendering, picking, and triangulated SVG output in native
picture order. The reader validates the exact five-record layout and native
range flags while preserving its opaque cache bytes; the writer emits the same
layout.

Unsupported falls **769→751**, complete documents/drawings rise
**1,510/1,551→1,513/1,554**, unsupported documents fall **89→86**, and partial
views remain **800**. Embedded-tool coverage remains **4,766/5,356**, and all
**1,599** originals remain byte-exact. An Xvfb desktop run of the native fan
construction visibly renders its sampled character pictures. No standalone
test case was added. See
[analysis](sampled_transformed_pictures_analysis.md). Full scan:
`/tmp/sampled_picture_coverage.log`.

## Legacy and high-rate Animation settings

Ten additional code-62 Animation buttons now pass the same strict model as the
existing action runtime. Eight modern parameter-reset buttons use large finite
rates over tiny domains; the bounded parameter limit now covers the observed
range without changing the point limit. Two legacy clock buttons retain their
16-byte header and decode their code-page-936 labels without rewriting
unchanged label bytes.

Unsupported falls **751→741**, complete documents/drawings rise
**1,513/1,554→1,514/1,555**, unsupported documents fall **86→85**, and partial
views remain **800**. Embedded-tool coverage remains **4,766/5,356**, and all
**1,599** originals remain byte-exact. Two contradictory Animation packets
remain opaque rather than receiving invented semantics. The remaining 17
code-62 objects are Sound buttons. No standalone test case was added. See
[analysis](legacy_animation_settings_analysis.md). Full scan:
`/tmp/animation_legacy_coverage.log`.

## Sound action buttons

All 17 corpus Sound roots now use a strict code-62/action-code-8 model. Each
observed root targets one supported function; the model and editor also support
the documented two-function stereo form. The desktop runtime evaluates the
function argument as seconds at 44.1 kHz, clamps finite pressure samples, emits
silence for undefined samples, and toggles each button's playback independently.

Unsupported falls **741->723**, complete documents/drawings rise
**1,514/1,555->1,523/1,565**, unsupported documents fall **85->76**, and partial
views fall **800->795**. Embedded-tool coverage remains **4,766/5,356**, and all
**1,599** originals remain byte-exact. A newly authored Sound button opened and
saved unchanged in Sketchpad 5 with SHA-256
`f74e8f0b2fa7a5bec0cfb8592a25bf6d09fe0e3c425d4e865550699731c64a00`.
The port editor also toggled real playback under Xvfb with an ALSA null device.
Only the two contradictory Animation packets remain among drawing code 62. No
standalone test case was added. See [analysis](sound_buttons_analysis.md). Full
scan: `/tmp/sound_button_coverage.log`.

## Transient explicit-path locus drivers

Native code-35 loci can temporarily place an existing point on the locus
record's explicit path even when that point's saved construction uses a
different path or is transformed, calculated, aliased, or colorized. Import now
distinguishes that form from an ordinary locus whose driver is already stored
on its path. The transient path must remain independent of the driver, must be
supported by the existing path sampler, and is applied only inside the locus
evaluation scope.

Sixteen drawing loci and 28 drawing descendants are restored; 13 embedded-tool
loci also decode. Unsupported falls **723->666**, complete documents/drawings
rise **1,523/1,565->1,540/1,579**, unsupported documents fall **76->59**, and
partial views fall **795->792**. Embedded-tool coverage rises
**4,766/5,356->4,779/5,356**. All **1,599** originals remain byte-exact. Three
representative originals—including a nested locus and a colorized-circle
target—opened and saved unchanged in Sketchpad 5 under Xvfb/Wine. No standalone
test case was added. See [analysis](transient_locus_drivers_analysis.md). Full
scan: `/tmp/transient_locus_coverage_final.log`.

## Mapped and high-rate function plots

Three native code-77 bindings present an already-carried `FunctionPlot` as a
visible mapped result. They now use the same scoped iteration evaluator as
mapped locus and sampled-curve collections, while retaining the binding's own
stroke color, width, visibility, and selection behavior. The shared result
feeds canvas, SVG, and picking without a second mapping implementation.

Two code-72 plots retain native sample counts of 200,000 and 250,000. Stored
plot descriptors, validation, editing, copying, and writing now accept the full
`u32` count range above one; actual evaluation remains bounded by each caller
and never allocates from the stored count directly. These two roots restore five
code-98 intersections, four code-65 coordinate measurements, eight code-48
calculations, and four code-69 plotted points.

Together the two changes restore 26 drawing slots. Unsupported falls
**666->640**, complete documents/drawings rise
**1,540/1,579->1,545/1,584**, and unsupported documents fall **59->54**;
partial views remain **792**. Embedded-tool coverage remains
**4,779/5,356**, and all **1,599** originals remain byte-exact. Five
representative originals opened, rendered, and saved unchanged in Sketchpad 5
under Xvfb/Wine. No standalone test case was added. See
[analysis](function_plot_completion_analysis.md). Full scans:
`/tmp/mapped_function_binding_coverage.log` and
`/tmp/high_rate_function_coverage.log`.

## Contextual scalar angles in custom polar translations

Native code-26 custom polar translations use operation-context units: the
second reference supplies a degree magnitude and the third supplies a signed
centimeter magnitude. Six final drawing roots use calculations whose stored
result category is Scalar in those slots. The shared transform model and
evaluator already apply that contextual contract, but the importer still
required the angle source itself to be categorized as Degrees. It now accepts
the same supported numeric quantity family used by numeric rotation and
dilation while retaining the exact three-reference/no-payload record guard.
The recovered colorized involute also exposed a presentation loss in direct
`SampledMapped` curves. Their styled path now evaluates the carried color at
each sample instead of flattening the entire spectrum to the object's fallback
stroke color; selection and picking continue to use the same sampled geometry.

The six roots restore 30 dependent drawing objects: eight code-35 loci, six
code-16 transforms, six code-101 sampled transformations, two code-77 bindings,
two code-30 transforms, two code-98 intersections, and one each of code 75, 89,
90, and 102. Unsupported falls **640->604**, complete documents/drawings rise
**1,545/1,584->1,551/1,590**, and unsupported documents fall **54->48**;
partial views remain **792**. Embedded-tool coverage remains **4,779/5,356**,
and all **1,599** originals remain byte-exact. Three representative originals
visibly rendered and saved unchanged in Sketchpad 5 under Xvfb/Wine; a port SVG
reproduces the native green-to-blue-to-violet involute spectrum. No standalone
test case was added. See
[analysis](scalar_angle_polar_translation_analysis.md). Full scan:
`/tmp/scalar_angle_code26_coverage_final.log`.

## High-depth general mapped iterations

Two final code-76 drawing roots are ordinary general mapped iterations whose
only rejected field was their fixed depth: the protractor stores 179 and the
Collatz construction stores 800. Live mapped depths already accepted values
through 4,096 and evaluation already stopped any branch expansion above 20,000
states at one level or 100,000 generated states overall. Imported and preserved
fixed depths now share that 4,096 ceiling. Desktop construction and specialized
point, segment, circle, ray, polygon, and point/parameter iteration commands
retain their separate 1–128 authoring limit.

The two roots restore all ten remaining drawing code-77 bindings, two dependent
code-89 mappings, and one each of codes 47, 48, and 88: **17 drawing slots** in
total. The port renders all 179 protractor ticks and labels and the Collatz
sequence `1541, 4624, ... 4, 2, 1` in the native ten-column layout. Sketchpad 5
under Xvfb/Wine visibly rendered both untouched originals and saved them
byte-for-byte with SHA-256 values
`91bbd1a80bc1a983ce24ef75ecdb8bede9e617a3686c634725c89c0958339b4f`
and
`abce071097c876be745610e25ef59e7ce66265ffe6a1b56c874d4a6caba23c92`.

Unsupported falls **604->587**, complete documents/drawings rise
**1,551/1,590->1,553/1,592**, unsupported documents fall **48->46**, and
partial views fall **792->790**. Embedded-tool coverage remains
**4,779/5,356**, and all **1,599** originals remain byte-exact. No standalone
test case was added. See
[analysis](high_depth_mapped_iteration_analysis.md). Full scan:
`/tmp/high_depth_iteration_coverage_full.log`.

## Final drawing referenced captions

Three modern code-73 captions use independent or sampled pictures as Value
sources. Pictures have no separate editable label; Native stores their
localized generated name in the reference cache. The live evaluator now uses
that cache and has a page-local `Picture [n]` fallback for API-authored bare
references. No other cached source family is promoted to live semantics.

The final legacy caption uses the exact observed 16-byte class-0/subtype-59
header. Its markup parser already supported GBK, but one referenced free-text
literal (`CA B1`, `时`) is also valid UTF-8 and was therefore ambiguous in
isolation. Short-header free text now shares GBK mode when another member of
the same document unambiguously establishes the old Windows Chinese cohort.
The original bytes remain in the rich-text template for unchanged and dirty
saves.

These changes restore four drawing code-73 slots in two documents. Unsupported
falls **587->583**, complete documents/drawings rise
**1,553/1,592->1,555/1,594**, unsupported documents fall **46->44**, and
partial views fall **790->789**. Embedded-tool coverage remains
**4,779/5,356**, and all **1,599** originals remain byte-exact. Sketchpad 5
under Xvfb/Wine opened and saved both originals unchanged with SHA-256 values
`cd793b1118c29f81e811fcadfc83022cc9d40b9bd53aad92ca7d23669e19c0ac`
and
`c95adfc4606abb897b28b4cb2f6393f7318cd59005cbf0b1cb8a0ff4f7c4993d`.
No standalone test case was added. See
[analysis](referenced_caption_completion_analysis.md). Full scan:
`/tmp/referenced_caption_coverage_full.log`.

## Point-angle fixed polar translation

The two drawing code-22 Point transforms use four references—source, first,
vertex, and last—and an exact 28-byte literal-distance packet. Import now maps
that wire form to the shared polar translation evaluator with a live directed
angle and literal Length. Admission requires every referenced object to be an
earlier same-page Point, plus the observed packet code, length, unit tag,
finite values, positive scale, and zero tail. Fresh construction and clipboard
reconstruction remain guarded.

The two roots also restore one dependent code-16 vector point. Unsupported
falls **583→580**, complete documents/drawings rise
**1,555/1,594→1,556/1,596**, unsupported documents fall **44→43**, and partial
views fall **789→788**. Embedded-tool coverage remains **4,779/5,356**, and all
**1,599** originals remain byte-exact. Both untouched originals visibly open
and save unchanged in Sketchpad 5 under Xvfb/Wine. No standalone test case was
added. See [analysis](point_angle_fixed_polar_analysis.md). Full scan:
`/tmp/code22_coverage_all_tests.log`.

## Sampled-curve direction arrow

The last drawing code-121 record uses the already proved direction-arrow packet
with a `SampledMapped` point curve as its host. Import requires that exact
earlier same-page curve family; congruence ticks remain Segment-only. Rendering
evaluates the stored normalized parameter and derives a bounded local tangent,
including a one-sided endpoint difference, before applying the established
ten-unit arrow geometry. The writer stays Segment-only because Native rejected
a generated copied curve-arrow graph.

This restores one drawing slot. Unsupported falls **580→579**, complete
documents/drawings rise **1,556/1,596→1,557/1,597**, unsupported documents fall
**43→42**, and partial views fall **788→787**. Embedded-tool coverage remains
**4,779/5,356**, and all **1,599** originals remain byte-exact. The untouched
original visibly renders and saves unchanged in Sketchpad 5 under Xvfb/Wine.
No standalone test case was added. See
[analysis](sampled_curve_arrow_analysis.md). Full scan:
`/tmp/code121_coverage_all_tests.log`.

## Final drawing Animation edge settings

Native Xvfb runs establish semantics for the last two drawing code-62 records.
A bidirectional point ignores its otherwise inapplicable stored Once bit and
continues moving. In a three-target button, a zero-rate target stays stationary
while the positive-rate targets animate. Import and runtime now preserve those
behaviors; newly authored buttons still require positive rates and disallow
Once Only for bidirectional motion.

This restores the final two drawing slots. Unsupported falls **579→577**,
complete documents/drawings rise **1,557/1,597→1,559/1,599**, unsupported
documents fall **42→40**, and partial views fall **787→785**. The remaining
**577** slots are all embedded-tool prototypes. Embedded-tool coverage remains
**4,779/5,356**, and all **1,599** originals remain byte-exact. No standalone
test case was added. See
[analysis](legacy_animation_settings_analysis.md). Full scan:
`/tmp/final62_coverage_all.log`.

## Scalar-Parameter custom-tool given

The two class-11/subtype-79 givens in `系列迭代案例.gsp` are now represented as
an isolated `ScalarParameter` tool input rather than an unsupported code-zero
object. A Native Xvfb invocation of `平方差方格系列图` prompted to match parameter
`n`; selecting the visible `n=5` Parameter and a canvas point created the
construction and increased the saved file by 3,546 bytes. The subtype therefore
does not inherit the broader generic-numeric matching rule.

Both placeholders use the established inert 94-byte numeric-given body. Their
stored step/domain bytes decode as nonsensical finite cache values, so validation
checks the exact placeholder family rather than treating the group as a live
Parameter. Runtime and desktop binding accept only an actual scalar
`Kind::Parameter`; calculations and other scalar-valued objects do not match.
The two source prototypes remain partial because of unrelated unsupported
constructions, so this checkpoint does not claim successful port-side execution
of either complete tool. At this checkpoint, subtypes 80 and 81 remained
unclassified; later sections establish both categories.

The dependency-aware scan restores **87** prototype slots: embedded-tool
coverage rises **4,779→4,866/5,356**, and unsupported slots fall **577→490**.
The 1,599 drawing documents remain drawing-complete, all 1,599 original files
round-trip byte-exactly, partial views remain 785, and no standalone test case
was added. See [analysis](tool_scalar_parameter_given_analysis.md). Full scan:
`/tmp/subtype79_coverage_all.log`.

## Embedded-tool picture resources

Native stores PNG resources in the document-wide resource section even when a
picture construction belongs to an embedded tool prototype. The isolated-tool
decoder previously copied the prototype groups into a synthetic document but
omitted those global resources, leaving otherwise supported code-99 attached
pictures unresolved. Picture-bearing prototypes now receive the source resource
records in their original order, preserving the resource IDs referenced by
their local picture groups.

This restores four direct code-99 pictures and 43 dependent objects in
`100以内两位数加一位数小棒图.gsp`: prototype coverage rises
**4,866→4,913/5,356** and unsupported slots fall **490→443**. The recovered
descendants include six code-16 constructions, twelve code-24 constructions,
seven code-33 values, six code-52 objects, five code-55 objects, three code-30
objects, and four mapped/binding objects. Eight code-99 prototype slots remain
behind independent unsupported prerequisites. Isolated-view completeness stays
36 complete/67 partial because the affected 149-object tool still has other
gaps.

All 1,599 drawings remain drawing-complete, all original files remain byte-exact,
and no standalone test case was added. See
[analysis](tool_picture_resources_analysis.md). Full scan:
`/tmp/tool_picture_resources_coverage.log`.

## Polygon custom-tool given

The sole class-1/subtype-15 given is now an abstract `Polygon` tool input rather
than an unsupported code-zero object. Under Native Xvfb, selecting
`凸多边形在直线上的滚动（李章博）` displayed `1. 匹配多边形 P[1]`; clicking inside
the source polygon advanced to its point input, and clicking a vertex created
the magenta rolling construction. Saving grew the copied file from 35,946 to
51,334 bytes.

The abstract input satisfies polygon-family predicates but exposes no invented
vertices or fixed vertex count. Runtime and desktop matching accept supported
polygon objects, including an interior click, while the prototype's remaining
unsupported constructions keep invocation behind the normal partial-view gate.
This restores **23** prototype slots: one input, its perimeter, eight code-95
plots, seven code-48 values, three code-21 objects, two code-94 objects, and one
code-24 object. Prototype coverage rises **4,913→4,936/5,356** and unsupported
falls **443→420**.

All 1,599 drawings remain drawing-complete, all original files remain byte-exact,
and no standalone test case was added. See
[analysis](tool_polygon_given_analysis.md). Full scan:
`/tmp/tool_polygon_given_coverage.log`.

## Coordinate-system custom-tool given

The ten class-1/subtype-39 tool givens now decode as abstract coordinate-system
inputs. Native coordinate systems use the same subtype 39 under their ordinary
class-8/code-61 construction header, and every supported prototype consumer
uses the given in a coordinate-frame role. The placeholder satisfies only that
predicate; axes, origin, scale, and cached `0899` values remain unavailable.
Runtime binding accepts an actual active-page coordinate system, and desktop
click binding uses grid hit testing.

This restores **26** prototype slots: ten inputs, five code-65 and four code-66
coordinate measurements, six code-48 calculations, and one code-69 numeric
point. Prototype coverage rises **4,936→4,962/5,356**, unsupported falls
**420→394**, complete isolated views rise **36→40**, and all-kind-complete
documents rise **1,559→1,562**. The newly complete prototypes still hit the
separate guarded coordinate-measurement caption-placement rule during actual
materialization. All 1,599 drawings remain complete, all original files remain
byte-exact, and no standalone test case was added. See
[analysis](tool_coordinate_system_given_analysis.md). Full scan:
`/tmp/tool_coordinate_system_given_coverage.log`.

## Function sources in custom-tool calculations

Three 39-input prototypes contain an identical wide code-48 shape: the native
expression declares one referenced Function role followed by 34 Numeric roles.
The abstract-tool decoder previously rejected the function parent before reading
those roles. It now validates each exact reference against the parsed role, and
runtime specialization applies the same check after remapping. Numeric unit
specialization remains unchanged; a function slot cannot accept a numeric
object or placeholder.

This restores **42** prototype slots: nine code-48 calculations, three code-71
functions, twelve code-33 objects, nine code-16 objects, and nine code-29
objects. Prototype coverage rises **4,962→5,004/5,356** and unsupported falls
**394→352**. The three prototypes remain partial because of unrelated code-100
objects, so complete-view and all-kind-document counts do not change. All 1,599
drawings remain complete, every original remains byte-exact, and no standalone
test case was added. See [analysis](tool_function_calculation_analysis.md). Full
scan: `/tmp/tool_function_calculation_coverage.log`.

## Function custom-tool given

The eight class-11/subtype-57 givens now decode as abstract Function inputs.
Native independently identifies the category: choosing the representative
`新f(x)的图象工具` displays `1. 匹配函数 h`. Each placeholder retains the exact
six-record class-11 family, including a validated source-free function
expression, label, style, and text bounds; its stored expression is not used as
the destination function.

Referenced-function source validation now admits generic Numeric placeholders
only for expression-declared Numeric roles and the new Function placeholder only
for Function roles. Runtime binding requires a supported active-page function,
and desktop binding uses the function caption. A representative newly complete
tool invoked with an identity function, four Parameters, and a coordinate system
created a live referenced function and plot, evaluated finite samples, produced
18 plot segments, and encoded/reparsed exactly.

This restores **22** prototype slots: eight inputs, two code-48 calculations,
six code-71 referenced functions, and six code-72 plots. Prototype coverage
rises **5,004→5,026/5,356**, unsupported falls **352→330**, complete isolated
views rise **40→47**, and all-kind-complete documents rise **1,562→1,569**.
All 1,599 drawings remain complete, every original remains byte-exact, and no
standalone test case was added. See [analysis](tool_function_given_analysis.md).
Full scan: `/tmp/tool_function_given_coverage.log`.

## Legacy generic-numeric and distance custom-tool givens

Native gives class-11/subtype-51 and the already-supported subtype-33 the same
`匹配度量值` (match Measurement Value) prompt; only their requested labels differ.
Subtype 51 therefore shares the generic Numeric destination category while
retaining its older source-free Calculation group validator. Its stored literal
does not supply a prototype unit.

Class-11/subtype-48 is distinct: Native prompts `匹配距离度量值 t[1]` (match
Distance Measurement Value). The abstract placeholder has a known Length unit,
and runtime plus desktop binding require a finite live distance value. Its
source-free literal zero is inert and scalar, so validation temporarily
normalizes only the header subtype while parsing the body; the original bytes
and contextual Length category remain unchanged.

The six subtype-51 inputs restore themselves and one code-71 descendant. The
single subtype-48 input restores 24 slots: its root, two code-48 calculations,
four code-16 objects, two code-95 plots, five code-29 and five code-26 objects,
one code-24, two code-27, one code-35, and one code-8 object. Combined prototype
coverage rises **5,026→5,057/5,356**, unsupported falls **330→299**, and
all-kind-complete documents rise **1,569→1,570**. Complete isolated views remain
47 because the recovered distance tool retains an independent structural-safety
restriction. All drawings remain complete, all originals remain byte-exact,
and no standalone test case was added. See
[analysis](tool_legacy_numeric_givens_analysis.md). Full scan:
`/tmp/tool_legacy_numeric_distance_given_coverage.log`.

## Angle-Parameter custom-tool given

The sole class-11/subtype-81 given belongs to `工具#1` in
`凸四边形的滚动（含七个）（yougui).gsp`. A same-size temporary copy changed only
its complete `04b4` order from `[9,8,7,6,5,4,3,2,1]` to `[1,9,8,7,6,5,4,3,2]`.
Native then displayed `1. 匹配角度参数 t[1]` (match Angle Parameter `t[1]`). On
the drawing's third page, Native outlined the visible `t[1] = 1.00°` Parameter
caption as an eligible match.

Subtype 81 is therefore a degree-valued Parameter category rather than generic
Numeric or every degree-valued measurement. Its inert 94-byte placeholder uses
the same strict family as subtype-79 scalar-Parameter givens. Prototype/runtime
binding and the desktop picker require an actual live `Parameter` whose unit is
Degrees.

The given root and its direct code-29 numeric-rotation descendant restore two
slots. Prototype coverage rises **5,057→5,059/5,356** and unsupported falls
**299→297**. Complete documents and isolated views remain 1,570 and 47/56: the
83-object tool retains unrelated unsupported code-97 and code-15 constructions.
All drawings remain complete, all originals remain byte-exact, and no standalone
test case was added. See [analysis](tool_angle_parameter_given_analysis.md).
Full scan: `/tmp/tool_angle_parameter_given_coverage.log`.

## Legacy Curve custom-tool given

The sole class-1/subtype-8 given belongs to `直尺画线工具` in
`圆弧展平（修改）.gsp`. A same-size temporary copy changed only its two-input
`04b4` order from `[2,1]` to `[1,2]`. Native then displayed
`1. 匹配曲线 c[1]` (match Curve `c[1]`). Independent hover controls marked an
ordinary circle, arc, full line, and finite segment as eligible.

Subtype 8 therefore shares the port's polymorphic `Path` destination family
while retaining its distinct native tag and placeholder bytes. The typed root
makes one source-free code-15 point-on-path child reachable. Its later generic
curve intersection remains unsupported rather than being forced into the
port's fixed line-circle intersection variants.

Prototype coverage rises **5,059→5,061/5,356** and unsupported falls
**297→295**. Complete documents and isolated views remain 1,570 and 47/56
because the prototype retains that intersection gap. All drawings remain
complete, all originals remain byte-exact, and no standalone test case was
added. See [analysis](tool_curve_given_analysis.md). Full scan:
`/tmp/tool_curve_given_coverage.log`.

## Polymorphic Preimage custom-tool given

The sole class-1/subtype-92 given belongs to `换行迭代格式工具1` in
`换行迭代格式工具及其运用示例.gsp`. A same-size temporary copy reordered only
the first tool's inputs from `[4,2,3,1]` to `[3,1,4,2]`. Native displayed
`1. 匹配原象 C` (match Preimage `C`). Clicking either the page-4 finite segment
or its point-on-segment endpoint advanced to the second input; the source
polygon also highlighted as eligible, while scalar Parameters did not.

Subtype 92 is therefore retained as a distinct polymorphic `Preimage` root. It
is not collapsed to Point, Path, or an iteration binding. The prototype's
dependent code-16/code-33/code-16 records have ordinary three-object reference
lists `[3,4,10]`, `[11,10,7]`, and `[12,4,2]`; their subtype-92 headers carry
the polymorphic result type. They decode through a prototype-only Point
stand-in and specialize to the bound Point, finite Segment, or Polygon element
type during invocation. Native invocation proves that the two suppressed
auxiliary formulas reuse the next visible formula's anchor and that Polygon
specialization assigns fresh drawing orders while dropping point-only labels.
The port now completes and reparses all nine outputs for Point, Segment, and
Polygon bindings. A port-authored Polygon result opens and saves byte-identically
in Sketchpad 5 under Xvfb/Wine.

The three recovered descendants raise prototype coverage
**5,062→5,065/5,356** and lower unsupported slots **294→291**. Fully decoded
documents rise **1,570→1,571**, and complete isolated views rise from 47 to 48
of 103. All drawings remain complete, all 1,599 originals remain byte-exact,
and no standalone test case was added. See
[analysis](tool_preimage_given_analysis.md). Full scan:
`/tmp/subtype92_runtime_coverage.txt`.

## Distance-Parameter custom-tool given

The sole class-11/subtype-80 given is the prerequisite of `工具#1` in
`杨辉三角1（李章博）.gsp`. Native Script View identifies it as
`距离参数 行距` (Distance Parameter `行距`) and explicitly asks the user to click
a distance Parameter. This distinguishes subtype 80 from subtype 48, whose
Native prompt accepts a Distance Measurement Value.

The port retains subtype 80 as a length-valued `DistanceParameter` root while
requiring the destination itself to be a live Length `Parameter`. The strict
placeholder validator is shared with the established scalar- and
angle-Parameter families. Contextual decoding restores its two code-48
calculation descendants. Runtime invocation against the source `行距` parameter
creates those two calculations and two movement buttons, then encodes and
reparses all five objects.

The 2,716-byte port-authored result opens in Sketchpad 5 under Xvfb/Wine with
`行距 = 0.74 cm` and both buttons visible. Ctrl+S preserves SHA-256
`c1981ec51c4fddd460fb765392e4920d2c1ee35d6dad9a81afbf77771822fe19`;
clicking the first button changes the value to `0.75 cm` and saves successfully.

The root and two descendants raise prototype coverage
**5,065→5,068/5,356** and lower unsupported slots **291→288**. Fully decoded
documents rise **1,571→1,572**, and complete isolated views rise from 48 to 49
of 103. All drawings remain complete, all 1,599 originals remain byte-exact,
and no standalone test case was added. See
[analysis](tool_distance_parameter_given_analysis.md). Full scan:
`/tmp/subtype80_runtime_coverage.txt`.

## Final code-zero custom-tool given categories

The last two unknown code-zero roots are class-1/subtypes 6 and 60. Native
Script View for `7.多面体表面(逆时向)着色` in `颗粒坐标系(3D).gsp` labels both
subtype-6 `面P[1]` and the already-supported subtype-5 `j` as `路径对象` (Path
Object). Subtype 6 consequently shares the polymorphic `Path` category while
retaining its exact native header.

Native Script View for `三维空间中的点2` in `也做正n棱锥表面展开.gsp` labels
subtype-60 `r` as `距离` (Distance). After the preceding point input is matched,
clicking the displayed `OA` distance caption does not advance the tool, while
clicking its finite segment matches `r`, automatically matches the last numeric
input, and begins constructing the result. The prototype uses `r` solely as the
radius source of `CircleBySegment`.

Subtype 60 is therefore represented as `SegmentDistance`: it binds a finite
segment but exposes Length magnitude semantics to dependent constructions. It
remains distinct from subtype-14 Segment geometry and subtype-48 numeric
Distance Measurement Values. Runtime validation and the desktop picker require
a finite segment, and the abstract prototype root satisfies the existing
distance-source predicate without acquiring segment geometry.

The two roots raise prototype coverage **5,068→5,070/5,356** and lower
unsupported slots **288→286**. No dependent slot changes classification, so
complete-document and isolated-view counts remain **1,572/1,599** and 49 of
103. All drawings remain complete, all originals remain byte-exact, and no
standalone test case was added. See
[analysis](tool_final_geometric_givens_analysis.md). Full scan:
`/tmp/tool_final_given_coverage.txt`.

## Attached pictures in embedded-tool invocation

The final object in three otherwise decoded 69-object prototypes is the same
code-100 class-16/subtype-78 picture, attached to three constructed points and
backed by the prototype's document-wide PNG resource. The exact hidden `[1]`
metadata record is shared with the established code-99 attached-picture family.
The decoder now treats both forms consistently and distinguishes a structurally
understood read-only resource from an unsupported record.

Invocation may consequently materialize the immutable prototype. The bounded
writer emits the proved three-point parallelogram group and one PNG resource;
other attached placements and multiple writable pictures remain guarded.
Numeric outputs retain their prototype caption fonts, including the observed
normal Times New Roman size-16 21-pixel row step and flat `NumericAlias`
caption. A 39-input invocation creates all 30 outputs and reparses exactly.

The 8,596-byte port-authored result visibly opens and renders in Sketchpad 5
under Xvfb/Wine. Ctrl+S preserves SHA-256
`a811318060ca4b96b496b34845f02e3a7fda2d05d51d95fdb61e56ad4500e063`.
The three roots raise prototype coverage **5,070→5,073/5,356**, lower
unsupported slots **286→283**, and raise fully decoded documents
**1,572→1,575/1,599**. Complete isolated views remain 49 because the source
resources intentionally remain read-only. All drawings remain complete, all
originals remain byte-exact, and no standalone test case was added. See
[analysis](tool_attached_picture_analysis.md). Full scan:
`/tmp/tool_code100_coverage.txt`.

## Named referenced captions in embedded tools

Strict code-73/code-90 caption groups now retain an optional `07d5` object name
between their rich text and font records. The same bounded name is written for
newly materialized referenced, point-referenced, and point-numeric captions.
Strict code-35 Locus name/font metadata is admitted as inert structural data,
and attached point values no longer displace free-standing tool measurement
rows. The observed bold 16-point `宋体` profile advances those rows by 19
pixels; point and segment ratios retain their 64-pixel fraction row.

The pass restores 48 code-90 roots, 20 code-73 roots, and 27 descendants.
Prototype coverage rises **5,112→5,207/5,356**, unsupported prototype slots
fall **244→149**, and fully decoded documents rise **1,576→1,581/1,599**.
Strict Locus metadata raises complete isolated views **52→67/103** and lowers
partial source views **785→769**. All drawings remain complete and all 1,599
originals remain byte-exact.

A three-point invocation creates 76 outputs and reparses all 79 objects with
one named attached caption. Its 11,164-byte result opens and renders in Native
Sketchpad 5 under Xvfb/Wine; Ctrl+S preserves SHA-256
`26014846f70421cb6ed74646ab308ce86db50ab24c94f2dfca74c6bada089ad6`.
No standalone test case was added. See
[analysis](tool_named_referenced_caption_analysis.md). Full scan:
`/tmp/locus_named_caption_final_coverage.log`.

## Tool-prototype mapped elements and nested mappings

Strict tool-prototype decoding now recognizes code-101 mappings whose seed and
replacement are declared path-point placeholders. Declared numeric tool inputs
outside the substitution are frozen predecessors, and the same contextual rule
is propagated through nested captured code-89 mappings. Ordinary drawing
validation remains unchanged. Code-101 point names and fonts are retained, and
the writer emits them in Native's reference-before-label order. The observed
normal Times New Roman size-24 caption uses a 19-pixel row step.

The pass restores all seven code-101 roots and 37 dependent prototype slots.
Prototype coverage rises **5,207→5,251/5,356**, unsupported falls **149→105**,
fully decoded documents rise **1,581→1,585/1,599**, complete isolated tools
rise **67→71/103**, and partial source views fall **769→765**. All drawings
remain complete and all 1,599 originals remain byte-exact.

A representative invocation creates 35 outputs and 41 total objects, reopens
non-partially, and re-encodes exactly. Sketchpad 5 under Xvfb/Wine renders the
result; Ctrl+S preserves SHA-256
`09394130b5c28571480ac0e33663f1fb7fca2bfc759c186931bbebf149ecf562`.
No standalone test case was added. See
[analysis](tool_mapped_element_placeholders_analysis.md). Final scan:
`/tmp/tool_mapped_element_final_coverage.log`.
