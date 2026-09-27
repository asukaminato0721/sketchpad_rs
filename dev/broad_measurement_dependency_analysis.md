# Broad measurement dependency audit

Read-only corpus analysis, 2026-09-27. No production changes, Native sessions, copying, or custom-tool work were performed. This is the baseline matching `/tmp/sketchpad_broad_numeric_import_audit.log`, before the root agent's new code24 and measurement decoder changes.

## Scope and method

The existing audit finds 20,270 unsupported drawing calculations (code48), of which 19,472 cannot resolve all parent numeric units. Its immediate missing numeric references include code65 (3,899), code66 (2,740), code41 (2,522), code94 (2,113), and code37 (1,497), plus many unsupported calculations feeding other calculations. Immediate reference counts alone substantially understate the reach of a measurement used by a long calculation chain.

`/tmp/broad_measure_roots.rs` reads the current library's decoded scene objects and original object groups. It skips embedded-tool namespaces between03ea/03eb, maps native page-local07d2 references to global scene IDs, and walks unsupported ancestors of every unsupported48. A terminal blocker is an unsupported object with no unsupported direct parent. Known direct parents may still have the wrong semantic type, so this classification is not permission to accept every such group. Each calculation counts once per terminal code/class/subtype/header family; different families can overlap. The final results are `/tmp/broad_measure_roots4.log`, and raw examples are `/tmp/broad_measure_root_examples.txt`. Dependencies come from07d2, not constraint07d3, whose leading word is often an opcode rather than a reference count.

## Quantitative result

| Terminal unsupported family | Affected unsupported48 objects | Documents |
| --- | ---: | ---: |
| 65 / class7 / subtype51, horizontal coordinate | 8,494 | 104 |
| 66 / class7 / subtype51, vertical coordinate | 6,302 | 86 |
| 48 / class11 / subtype51, calculation itself | 4,224 | 65 |
| 94 / class7 / subtype51, point value | 2,505 | 106 |
| 101 / class1 / subtype89 | 1,096 | 19 |
| 21 / class2 / subtype14, polar translated segment | 1,013 | 27 |
| 21 / class1 / subtype4, polar translated point | 963 | 28 |
| 24 / class1 / subtype4, marked-distance translation | 889 | 30 |
| 71 / class11 / subtype57 | 659 | 68 |
| 27 / class1 / subtype4, rotated point | 559 | 33 |
| 80 / class12 / subtype67, centered arc family | 556 | 121 |
| 15 / class1 / subtype10, constrained point | 526 | 95 |

Coordinate65/66 combined affect **8,753 distinct calculations**. Including94 raises the union to **11,246**, or **55.5% of the20,270 unsupported calculations**. In6,486 calculations the terminal blockers are exclusively65/66; in8,296 they are exclusively65/66/94. These are opportunities, not a promise that every downstream formula will parse after the measurement fix: a calculation can also have its own unsupported expression or unit semantics.

The measurement objects themselves separate as follows:

| Unsupported measurement | Total objects | Has unsupported direct parent | All direct parents decoded |
| --- | ---: | ---: | ---: |
| 65 | 731 | 383 | 348 |
| 66 | 588 | 321 | 267 |
| 94 | 1,452 | 858 | 594 |
| 41 | 2,192 | 2,192 | 0 |
| 37 | 1,255 | 1,255 | 0 |

The largest reusable gap is therefore **measurement record parsing**, rather than new geometric math. Among the615 known-parent coordinate objects,259 fail because the native group includes07d5 before07d8, while356 fail the0903 formatting check. `coordinate_measure_records::decode_group` currently requires an exact five-record sequence and compares0903 bytes18..28 wholesale. This includes varying22/23 and display-mode26/27. The ordinary scalar-measurement decoder already permits22/23 variation and display modes0/1/3. The coordinate evaluator already handles the required Cartesian affine basis.

Among594 known-parent94 objects,527 fail solely at the observed group-layout gate:395 have an empty232e after07d8, and132 also have07d5 before07d8. The remaining67 pass `value_equation_records::decode_group` and fail the document's semantic host gate; they must remain a separate audit. Existing point-value math and supported-host restrictions should be preserved while permitting only the observed empty attachment. Do not accept nonempty232e or arbitrary hosts on this evidence.

For41 and37, every unsupported measurement has an unsupported geometric ancestor. They are consequences rather than an independent measurement-layout bottleneck in this snapshot. The next reusable geometry audit is existing polar translation21, especially recursive source-family recognition: its segment and point families affect1,013 and963 downstream calculations, respectively, and already have a transform-model representation. Its60-byte native payload must still pass all existing literal checks. Code24 is being audited/fixed independently by the root agent. Code101 and centered arc80 reach substantial numbers too, but require independent schema/geometry work; their reach does not justify inventing semantics.

## Three exact original samples

All three following sample IDs are first-page native object IDs. They demonstrate genuine drawing groups with already-decoded parents.

1. `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/用迭代演示动态反比例函数作图（蚂蚁制作）.gsp`, object15, horizontal coordinate. Parents13=`PointOnLine(12,3.8629166666666626)`,6=`CoordinateSystem(3,5)` are decoded. Object17 is an unsupported calculation with sources15/16. Layout is07d0,07d2,0903,07d5,07d8,07d7.

   - 07d0: `41000100000007003300210100000000ffff0000c1bdf0ff00000000`
   - 07d2: `020000000d00000006000000` (count2, point13, frame6)
   - 0903: `fc08000016000000040000000a003d0003003c303e0000800600000002000000`
   - 07d5: `0600000000000800050000000000986e12c1515c614104006d5b315d0000` (caption`m[1]`)
   - 07d8: `0300`

2. `../gsp-rs/tests/Samples/个人专栏/向忠作品/2-3-1(3).gsp`, object18, vertical coordinate. Parents8=`Free(541,207)`,7=`CoordinateSystem(6,5)` are decoded. Calculation19 depends on18 and several calculations descended from coordinate9. Layout again includes07d5.0903 has varying22/23=`4d52` and display mode3.

   - 07d0: `42000300000007003300210100000000ffff515cc1bdf0ff00000000`
   - 07d2: `020000000800000007000000`
   - 0903: `fc08000016000000040000000a007d0003003c303e004d520600030002000000`
   - 07d5: `0700000001000800050000000000986e12c1515c61410600cf865b4f765d0000`
   - 07d8: `0100`

3. `../gsp-rs/tests/Samples/个人专栏/况永胜作品/正方体的展开（3D效果）.gsp`, object13, point value. Parents12=`PointOnCircle(7,0.9774097060659551,0.21135341607852773)`,7=`Circle(1,2)` are decoded. Calculation14 references13. Layout is07d0,07d2,0903,07d8,**empty232e**,07d7.

   - 07d0: `5e000300000007003300210100000000ffffcb3fc1bdf0ff00000000`
   - 07d2: `020000000c00000007000000`
   - 0903: `fc08000016000000040000000a00050003003c303e0033060600010002000000`
   - 07d8: `0100`
   - 232e: length0, no payload

An additional pure-format control is the second-page coordinate in`2-3-1(3).gsp`, global object488 (native local178, refs177/7): no07d5,0903=`fc08000016000000040000001601840003003c303e004d520600000002000000`. Parents global487=`Free(296,455)` and317=`CoordinateSystem(316,315)` are decoded. This isolates22/23 variation from optional-caption support.

## Recommended order

1. Preserve existing measurement mathematics and semantic parent gates; support ordered optional coordinate captions and observed0903 presentation variation. Re-run the full corpus to measure actual newly decoded calculations.
2. Permit and preserve only the observed empty232e point-value attachment, keeping the67 parser-success/host-failure cases separate.
3. After measuring that gain, audit existing21/24 transform source chains and the residual15 roots by raw subtype/payload. Avoid treating all41/37 failures as fresh measurement families or prioritizing small ray/line additions over these shared blockers.

## Follow-up: fixed polar21, marked polar24, and fixed rotation27

Read-only classification using the checkpoint27-era library, before the root's recursive21 source fix. Scripts/output: `/tmp/polar_point_audit.rs`, `/tmp/polar_point_audit.log`, `/tmp/polar_point_27_audit.rs`, `/tmp/polar_point_27_audit.log`. These counts are **class1/subtype4 drawing points only**; segment/circle/polygon counts from the earlier table are separate. No production edits or Native probes were performed.

Every known-parent point failure has the expected constraint length/opcode and finite angle/cache fields. There were no additional length, opcode, nonfinite, or unexplained trig-cache failure classes in this cohort.

| Point family | Direct parents decoded, still unsupported | Specific failure |
| --- | ---: | --- |
| 21 fixed polar | 275 | word28=0; cache agrees with radians, not degrees |
| 27 fixed rotation | 175 | word28=0; cache agrees with radians, not degrees |
| 24 marked polar | 99 | word28=1, degree cache valid; source is a Length Parameter excluded by `is_distance_value` |
| 24 marked polar | 79 | word28=1, degree cache valid; numeric source is Scalar |
| 24 marked polar | 24 | word28=0, zero angle, valid currently accepted distance-value family |
| 24 marked polar | 1 | word28=0, nonzero radian angle, valid currently accepted distance-value family |
| 24 marked polar | 120 | word28=0, nonzero radian angle, Scalar source |
| 24 marked polar | 16 | word28=0, zero angle, Scalar source |
| 24 marked polar | 1 | word28=0, nonzero radian angle, Length Parameter |
| 24 marked polar | 15 | word28=2, degree cache valid, currently accepted distance-value family |
| 24 marked polar | 1 | word28=2, degree cache valid, Length Parameter |

Thus24 has356 known-parent failures. Its word28 values are1 in178,0 in162, and2 in16. Across **all24 point records**, word0 appears691 times, of which529 have an unsupported direct parent; it is not an isolated malformed record. For21,774 unsupported points have word0 (275 known-parent,499 unknown-parent), and112 zero-angle word0 points already decode because zero passes either trig convention. For27,807 unsupported points have word0 (175 known-parent,632 unknown-parent).

Word1 examples use degree magnitudes. Word0 nonzero examples use radian magnitudes, established both by their trig caches and paired HTML. Word2 must remain separately documented: existing supported27 records with nonzero angles use **degree** magnitudes despite this flag. A new shared angle decoder must not reject those existing supported cases. This audit establishes numerical conventions in the observed groups, not the general UI name/meaning of every unit flag.

### Exact radian controls with paired HTML

- `向忠作品/y=Asin(wx+v).gsp`, global45/page1/local22, code21, source global39/local16. Header `150001000000010004002101ff000000ffffefbcc1bdf0ff00000000`;07d2=`0100000010000000`;07d3=`15000000000000000000f0bf00000000d072efbc282d4454fb21f93f000000008fc7e3f1783c0e409a9999999999b93f01000000e698afe4b880e4b8`. Angle20=1.5707963267949, word28=0, cached sine=-1/cosine approximately0. The paired HTML line100 identifies local22 as `Translation(16,-1.31962e-014,3.77953)`.
- `向忠作品/π日话π.gsp`, global20/page0/local20, code24,07d2=`020000000100000013000000`, source1 Free and source19 Length calculation already decode. Header `180001000000010004002101ff000000fffff03fc1bdf0ff00000000`;07d3=`18000000000000000000f0bf205d67cece72efbc282d4454fb21f93f00000000b95c2e97`. Angle20=1.5707963267949, word28=0. Paired HTML line92 reads `Translation/FixedAngle/MarkedDistance(1,13,1.5708)`. HTML can reorder calculations, so semantic source relationships should be checked rather than assuming every exported numeric ID equals the original GSP ID.
- `向忠作品/12T.gsp`, global9/page0/local9, code27,07d2=`020000000200000001000000`, source2=`GraphHorizontalUnit(1,445.7952755905512)`, center1=`Free(191,491)`. Header `1b0003000000010004002100ff000000ffff515cc1bdf0ff00000000`;07d3=`1b000000000000000000f0bf205d67cece72efbc282d4454fb21f93f0000000000000000`. Paired HTML line87 is exactly `Rotation(2,1,1.5708)`.

A supported word2 comparison is `孟令岩作品/圆钉上的橡皮筋.gsp`, object5: code27, magnitude89.9999999999998, word28=2, cached sine=-1/cosine approximately0, refs4/1. Its paired HTML object5 reads `Rotation(4,1,1.5708)`. Three currently supported27 points and15 currently supported21 zero-angle points use word2. It should not be silently narrowed away by the refactor.

### Separate broad geometry issue:21 offset32 is a pixel-distance cache

The existing21 decoder passes offset32 as the transform model's units-to-pixels multiplier and offset40 as its magnitude. The evaluator multiplies them. Original paired HTML exposes a repeated-magnitude error:

| Original21 point | Offset32 | Offset40 | Native HTML displacement | Current model displacement magnitude |
| --- | ---: | ---: | ---: | ---: |
| `y=Asin(wx+v)`, global45/local22 | 3.779527559055118 | .1 | 3.77953 | .3779527559055118 |
| `100位e`, global35 | 11.338582677165356 | .3 | 11.3386 | 3.4015748031496067 |

Both use distance word48=1. Their offset32 equals37.79527559055118×offset40, not the constant conversion scale. In these native groups it is **total pixel distance**. The paired HTML is independent exported geometry evidence; mere byte-preserving acceptance would not catch the erroneous displacement.

A distance-word2 record also already decodes: `唐家军作品/课程3笔记.gsp`, global17/page1/local9, offset32=19.2, offset40=.2, giving96 pixels per stored magnitude. This audit has not assigned a UI unit name to that flag. Distances must be decoded using the observed unit/cache relationship, with explicit zero-magnitude handling. Do not blindly divide by zero or infer arbitrary flags. The ordinary writer's21 payload branch currently writes the model multiplier directly to offset32, so any normalization must also audit writer cache generation while preserving untouched source payloads.

Length Parameter support is another existing-architecture gap:99 degree24 points have valid payloads and explicitly decoded native subtype80 parameters, but `Kind::is_distance_value` and `Document::distance_value_pixels` omit `Kind::Parameter`. `圆内蜂窝（轨迹迭代修正版）.gsp` object15 is an exact control: refs14/12, source14 known transformed point, source12 native Length Parameter, angle0/word1; paired HTML object15 is `Translation/FixedAngle/MarkedDistance(14,12,0)`. This does **not** prove the meaning of the215 known Scalar-source24 cases, nor authorize changing signed/undefined distance policies. Those remain separate prerequisites.

## Follow-up: source-free quantity calculations

Read-only audit linked against the checkpoint30-era library: `/tmp/sourcefree_audit.rs`, `/tmp/sourcefree_audit.log`, and `/tmp/sourcefree_expr_examples.txt`. Embedded tool namespaces are excluded. Original07d2 dependencies are mapped per page, and downstream unsupported48 ancestry is counted once per root object per calculation. Counts accumulated over different roots can overlap. No production edits, Native, copying, or tool work were performed.

| Native source-free drawing family | Objects | Unsupported objects | Unsupported files | Downstream unsupported48 root-reference count |
| --- | ---: | ---: | ---: | ---: |
| code0/class11/sub48 Length | 6 | 6 | 5 | 1 |
| code0/class11/sub49 Degrees | 15 | 12 | 7 | 54 for the12 failures |
| code0/class11/sub50 Area | 0 | 0 | 0 | 0 |
| code0/class11/sub51 Scalar | 99 | 0 | 0 | 0 from unsupported roots |

Total120 source-free objects,18 unsupported. No drawing code48/class11 group with empty dependencies was found. Thus this is useful completeness of the existing expression API, but its55 accumulated downstream references are much smaller than the broad coordinate/value-measurement bottleneck. Do not present source-free quantities as the dominant remaining corpus blocker. The three already-supported degree literals and99 already-supported Scalars may feed other unsupported calculations, but they are not missing roots.

All six Length groups fail the current49/51-only subtype gate. Three share `3cm * sqrt(3)`; the remaining real expressions are `4cm / 24`, `pi * 2cm`, and `-3 * sqrt(3) * 1cm`. The12 unsupported Degrees groups fail the artificial “source-free radian angle layout” restriction. Examples include `atan(6)`, `atan(3/4)`, `atan(3/2)`, `atan(4/3)`, `360° / 12`, and degree-suffixed expressions involving pi. They are native subtype49 even when no literal radian suffix0100 appears. The cached/presentation prefix is the same observed0907 structure; the typed AST should determine result units and be checked against the native header rather than forcing every49 formula to contain0100.

### Real zero-reference controls

1. **Length:** `../gsp-rs/tests/Samples/个人专栏/方小庆作品/(inRm)重叠面积8例.gsp`, object3, subtype48. No07d2 exists. Layout07d0,0907,07d8,08a3,07d7.

   - 07d0: `0000000000000b003000210100000000ffff0000c1bdf0ff00000000`
   - 0907: `fc08000016000000040000000900efff03003c303e00000006000000020000000709000048000000300000000600040034f903009a0b1403120100003a296100ffffffffd31c5b00194f5800e004a90084f912004e08d377aa0a5401030001020210072003000c00`
   - Tokens: `0003 0201 1002 2007 0003 000c` =3cm × sqrt(3).
   - 07d8: `0100`;08a3: `0900efff83000500`.

2. **Inverse angle function:** `../gsp-rs/tests/Samples/个人专栏/唐家军作品/课程4笔记.gsp`, global101/page2/local31, subtype49. No07d2 exists. Layout includes optional07d5 before07d8.

   - 07d0: `0000000000000b0031002101ff000000ffff515cc1bdf0ff00000000`
   - 0907: `fc08000016000000040000002d02b60103003c303e0000000600000002000000070900004200000030000000030004000b0f0300000001000c00a8004c08040140f919001400000090f8190084000000280804010b0f317250f91900052006000c00`
   - Tokens: `2005 0006 000c` =atan(6), whose AST result is Degrees. The display mode is0; there is no0100 literal suffix.
   - 07d5: `0600000000000800050000000000986e12c1515c614104006d5b345d00be`;07d8: `0700`;08a3: `2d02b601b802d201`.

3. **Already-supported Scalar angle-function comparison:** `../gsp-rs/tests/Samples/热研系列/迭代系列/多层圆的展示.gsp`, object9, subtype51. No07d2 exists.

   - 07d0: `0000000000000b003300210100000000ffff515cc1bdf0ff00000000`
   - Tokens: `2000 0001 0005 0101 000c` =sin(15°), Scalar result. Both this group and the subtype49 inverse-function group have the same fixed0907 numeric prefix, with varying anchor/cache/memory bytes preserved.
   - 07d5 caption is`m[2]`;07d8=`0100`;08a3=`ea01520050026600`.

No native source-free Area control, or source-free unsupported Scalar angle-function control, was found in this corpus. Area generation should be clearly distinguished from these retained native import controls. Trailing-decimal generation also has no failing source-free import sample in this snapshot; supported literal Scalars include decimals such as `(100 / 99.899999)` and `(6.5 / 2.5)`, but they do not demonstrate a decimal point with no following digit.
