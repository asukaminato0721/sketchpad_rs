# Corpus decoding coverage

Snapshot on 2026-09-26, generated with `gsp_probe coverage ../gsp-rs`.

Documents: 1599 total, 85 with all object kinds decoded, 1514 with unsupported objects, 0 failed.

Byte-preserving round trips: 1599/1599; object slots: 285703.

Partial views: 1527/1599 (includes multi-page structural edit restrictions).

Drawing objects: 280347 slots; 86 documents with all drawing kinds decoded.

Embedded tool prototypes: 103 definitions in 65 documents; 5356 undecoded slots.

Unsupported object slots including tool prototypes: 112506. Object decoding does not establish editing, interaction, or command parity. The release requirements remain in `COMMAND_PARITY.md`.

Embedded tool prototypes now have a separate namespace. They remain counted as undecoded slots; their exclusion from the drawing histogram does not establish tool support. This snapshot is not directly comparable to earlier drawing histograms that mixed tool prototypes with scene objects.

The counts include missing variants and failures caused by unsupported parents.
Inspect complete groups and parent payloads before selecting a target.

| Native code | Unsupported objects | Documents | First example |
| --- | ---: | ---: | --- |
| 48 | 21632 | 690 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 77 | 7405 | 797 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/【教程】用迭代构造平面直角坐标系.gsp` |
| 24 | 6143 | 266 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/橡皮筋大战钉子(蚂蚁制作).gsp` |
| 62 | 5540 | 428 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/09年潍坊17题答案（蚂蚁制作）.gsp` |
| 16 | 5378 | 519 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 33 | 4343 | 428 | `../gsp-rs/tests/Samples/个人专栏/况永胜作品/2024年年历表.gsp` |
| 73 | 3298 | 1350 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/09年潍坊17题答案（蚂蚁制作）.gsp` |
| 27 | 3188 | 478 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 8 | 3030 | 496 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 75 | 2879 | 445 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/10福建宁德26题2(蚂蚁).gsp` |
| 90 | 2712 | 399 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 89 | 2538 | 772 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/【教程】用迭代构造平面直角坐标系.gsp` |
| 34 | 2341 | 191 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/化圆为方(蚂蚁制作).gsp` |
| 21 | 2328 | 237 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 15 | 2309 | 520 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 30 | 2281 | 371 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 41 | 2190 | 107 | `../gsp-rs/tests/Samples/个人专栏/况永胜作品/正方体的展开（3D效果）.gsp` |
| 35 | 1978 | 409 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/09年潍坊17题答案（蚂蚁制作）.gsp` |
| 69 | 1782 | 251 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 80 | 1594 | 406 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/化圆为方(蚂蚁制作).gsp` |

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
