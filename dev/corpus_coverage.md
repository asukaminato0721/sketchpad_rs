# Corpus decoding coverage

Checkpoint 20 snapshot on 2026-09-27, generated with `gsp_probe coverage ../gsp-rs`.

Documents: 1599 total, 85 with all object kinds decoded, 1514 with unsupported objects, 0 failed.

Byte-preserving round trips: 1599/1599; object slots: 285703.

Partial views: 1522/1599 (includes multi-page structural edit restrictions).

Drawing objects: 280347 slots; 86 documents with all drawing kinds decoded.

Embedded tool prototypes: 103 definitions in 65 documents; 5356 slots, 3413 decoded, 1943 undecoded.

Isolated tool views: 31 complete, 72 partial, 0 failed, with separate object and font namespaces.

Unsupported object slots including tool prototypes: 105274 (103331 drawing slots and 1943 prototype slots). Object decoding does not establish editing, interaction, or command parity. The release requirements remain in `COMMAND_PARITY.md`.

Embedded tool prototypes are decoded through isolated documents. Declared point placeholders receive neutral coordinates; declared path, finite-segment, and generic-numeric placeholders retain typed abstract inputs. Other unknown prototype constructions remain unsupported. The drawing and prototype histograms retain separate namespaces. Complete isolated decoding does not establish invocation or authoring support for every tool. This snapshot is not directly comparable to earlier drawing histograms that mixed tool prototypes with scene objects, or to snapshots that counted every prototype slot as undecoded.

The counts include missing variants and failures caused by unsupported parents.
Inspect complete groups and parent payloads before selecting a target.

| Native code | Unsupported objects | Documents | First example |
| --- | ---: | ---: | --- |
| 48 | 20270 | 626 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 77 | 7405 | 797 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/【教程】用迭代构造平面直角坐标系.gsp` |
| 62 | 5540 | 428 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/09年潍坊17题答案（蚂蚁制作）.gsp` |
| 24 | 5260 | 221 | `../gsp-rs/tests/Samples/个人专栏/况永胜作品/2024年年历表.gsp` |
| 16 | 5027 | 500 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 33 | 4203 | 418 | `../gsp-rs/tests/Samples/个人专栏/况永胜作品/2024年年历表.gsp` |
| 73 | 3298 | 1350 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/09年潍坊17题答案（蚂蚁制作）.gsp` |
| 27 | 3126 | 471 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 8 | 2899 | 474 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/2010湖南衡阳(蚂蚁制作).gsp` |
| 75 | 2879 | 445 | `../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/10福建宁德26题2(蚂蚁).gsp` |

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
