# Corpus decoding coverage

Snapshot on 2026-09-26, generated with `gsp_probe coverage ../gsp-rs`.

Documents: 1599 total, 76 with all object kinds decoded, 1523 with unsupported objects, 0 failed.

Byte-preserving round trips: 1599/1599; object slots: 285703.

Partial views: 1532/1599 (includes multi-page structural edit restrictions).

Unsupported object slots: 121697. Object decoding does not
establish editing, interaction, or command parity. The release requirements
remain in `COMMAND_PARITY.md`.

The counts include missing variants and failures caused by unsupported parents.
Inspect complete groups and parent payloads before selecting a target.

| Native code | Unsupported objects | Documents | First example |
| --- | ---: | ---: | --- |
| 48 | 24812 | 802 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 77 | 7508 | 801 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/【教程】用迭代构造平面直角坐标系.gsp` |
| 16 | 6836 | 615 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 24 | 6355 | 280 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/化圆为方(蚂蚁制作).gsp` |
| 62 | 5640 | 431 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/09年潍坊17题答案（蚂蚁制作）.gsp` |
| 33 | 5201 | 481 | `../gsp-rs/tests/Samples/个人专栏/况永胜作品/2024年年历表.gsp` |
| 27 | 5066 | 607 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 8 | 3353 | 561 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/10福建宁德26题(蚂蚁).gsp` |
| 73 | 3320 | 1351 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/09年潍坊17题答案（蚂蚁制作）.gsp` |
| 75 | 3106 | 447 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/10福建宁德26题2(蚂蚁).gsp` |
| 15 | 3008 | 588 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 21 | 2879 | 309 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 90 | 2764 | 405 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 34 | 2762 | 199 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/化圆为方(蚂蚁制作).gsp` |
| 30 | 2622 | 411 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 89 | 2577 | 775 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/【教程】用迭代构造平面直角坐标系.gsp` |
| 41 | 2263 | 115 | `../gsp-rs/tests/Samples/个人专栏/况永胜作品/正方体的展开（3D效果）.gsp` |
| 95 | 2262 | 529 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/10福建宁德26题(蚂蚁).gsp` |
| 0 | 2016 | 653 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/09年潍坊17题答案（蚂蚁制作）.gsp` |
| 35 | 2005 | 416 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/09年潍坊17题答案（蚂蚁制作）.gsp` |

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

The next audited gaps include typed length/degree parameter records, further
calculation layouts, additional iteration definitions and their code-77
bindings, broader multi-page numeric record editing, and further vector-transform variants. Saved scalar parameter edits and supported table rows now have focused multi-page native proofs; this does not remove the document-wide partial-view flag. Original `cood-trace.gsp`,
`poly_point_with_val.gsp`, refraction and Ningde examples, and
`迭代方法2(蚂蚁).gsp` provide concrete native payloads and paired HTML definitions.
Retain native payload validation and regression/cross-open gates when extending
these families.
