# Embedded custom-tool given types: read-only corpus audit

Audited on 2026-09-26 after checkpoint 18. The recursive `../gsp-rs` corpus contains 1599 documents, 65 with embedded tools, 103 tool definitions, 5356 prototype objects, and 515 ordered given slots. All 103 `04b4` declarations are well formed. The checkpoint-18 isolated decoder reported 8 complete views and 95 partial views, with no extraction failures. These are the baseline counts before the contextual placeholder correction described below; they establish no new Native invocation semantics.

## Exact input headers and record families

Every given has a 28-byte `07d0` header with construction code 0. The table lists the native class at header offset 6 and subtype at offset 8. Counts are given slots, so a repeated category within a tool is counted repeatedly. Tool indices and local IDs are zero based and one based respectively.

| Class | Subtype | Slots | Tools | Record family | Representative tool and local given |
| ---: | ---: | ---: | ---: | --- | --- |
| 1 | 4 | 227 | 80 | P | `Samples/未分类档/圆周展开(changxde).gsp`, tool 0, given 1 |
| 1 | 5 | 6 | 3 | P | `Samples/个人专栏/李有贵作品/打台球轨迹(yougui).gsp`, tool 0, given 3 |
| 1 | 6 | 1 | 1 | P | `Samples/个人专栏/钟科作品/颗粒坐标系(3D).gsp`, tool 6, given 2 |
| 1 | 8 | 1 | 1 | P | `Samples/未分类档/圆弧展平（修改）.gsp`, tool 0, given 1 |
| 1 | 15 | 1 | 1 | P | `Samples/个人专栏/李章博作品/任意边数凸多边形在直线上滚动工具（李章博）.gsp`, tool 0, given 2 |
| 1 | 39 | 10 | 10 | P | `Samples/个人专栏/汪志平作品/圆柱中削最大圆锥2025.gsp`, tool 0, given 3 |
| 1 | 60 | 1 | 1 | P | `Samples/热研系列/3D系列/也做正n棱锥表面展开.gsp`, tool 1, given 2 |
| 1 | 92 | 1 | 1 | P | `Samples/热研系列/迭代系列/换行迭代格式工具及其运用示例.gsp`, tool 0, given 3 |
| 11 | 33 | 248 | 49 | A | `Samples/个人专栏/向忠作品/网球线.gsp`, tool 0, given 1 |
| 11 | 48 | 1 | 1 | A | `Samples/热研系列/滚动系列/三角形在正方形内的滚动.gsp`, tool 0, given 10 |
| 11 | 51 | 6 | 4 | A | `Samples/个人专栏/常新德作品/gsp音乐_两只老虎.gsp`, tool 0, given 1 |
| 11 | 57 | 8 | 8 | A | `Samples/个人专栏/向忠作品/悬垂曲面(表达式的极小曲面).gsp`, tool 0, given 1 |
| 11 | 79 | 2 | 2 | A | `Samples/个人专栏/汪志平作品/系列迭代案例.gsp`, tool 1, given 1 |
| 11 | 80 | 1 | 1 | A | `Samples/个人专栏/李章博作品/杨辉三角1（李章博）.gsp`, tool 0, given 1 |
| 11 | 81 | 1 | 1 | A | `Samples/个人专栏/李有贵作品/凸四边形的滚动（含七个）（yougui).gsp`, tool 0, given 1 |

Class 1 totals 248 slots; class 11 totals 267, for 515 altogether. There are no legacy 16-byte given headers and no given headers of another class. These are header classes, **not established destination geometry categories**.

Family P is exactly `07d0:28, 07d5:N, 07d8:2, 0899:16, 07d7:0`. `N` is 26, 28, 30, 32, 34, or 66 bytes. All 248 class-1 givens have this family; none has `07d1`. The 16 bytes in `0899` can be zero, tiny non-coordinate values resembling cache words, or finite coordinates. For example the first two `12T.gsp` point placeholders include zero bytes and `[0x550,0,0x553,0]`; the subtype-60 given in the second `也做正n棱锥表面展开.gsp` tool stores `(637,433)`. Treating every such payload as a real Free point is unsafe. Only declared class-1/subtype-4 placeholders currently receive neutral coordinates.

Family A is exactly `07d0:28, 0907:94, 07d5:N, 07d8:2, 08a3:8, 07d7:0`. `N` is 26, 28, 30, 32, or 44 bytes. All 267 class-11 givens have this family. No given has a `090d` parameter-animation record. Every arithmetic descriptor begins with the observed `<0>` placeholder machinery; cache bytes and subtype variations are not independent evidence for scalar, length, angle, or function semantics.

## Ordered givens and graph dependencies

`04b4` is a little-endian u32 input count followed by exactly that many local u32 object IDs. IDs are unique and lie within the enclosing tool canvas. Native given order must be retained; it is not ascending object order. For example Square and simple controls use `[2,1]`, while the mixed-number candidate below uses `[3,2,4,1,5,6,7]`.

**All 515 given groups have no `07d2` record.** They are dependency roots whose category is carried by placeholder metadata. Dependent output groups reference local given IDs in their own `07d2`/`07d3` records. A `07d2` payload starts with its reference count; that first u32 is not an object ID. Neither output references nor placeholder labels establish exact input compatibility on their own. No references belong to the drawing namespace.

## Category evidence and bounded next controls

The class-1/subtype-4 Native controls already prove ordinary point binding. The baseline ordinary construction-code-0 branch decoded other class-1 headers as Free, although subtype-specific point/style recognition was absent. The complete-view flag therefore must not be used as input-category validation. Contextual decoding now overrides declared givens before dependent construction guards; unsupported categories remain `Unsupported(0)`. Ordinary drawing decoding remains unchanged.

Representative non-4 labels are subtype 5 `j`, subtype 6 `面P[1]`, subtype 8 `c[1]`, subtype 15 `P[1]`, subtype 39 `S[1]`, subtype 60 `r`, and subtype 92 `C`. Downstream references give useful hypotheses: subtype 5 is a host of code-15 constrained points and code-16 reflections; subtype 8 is a host of a code-15 point and code-11 intersection; subtype 15 feeds code-39 perimeter and path-value calculations; subtype 39 appears as the frame reference of code-72 plots; subtype 60 is used as a point endpoint in straight constructions. These suggest several distinct categories and directly contradict blanket Free binding. They do not prove Native category acceptance or exclude alternate meanings.

The separate Native controls for Segment + Point on Segment and Circle + Point on Circle both produce the **same code-0/class-1/subtype-5 given header**. The segment control is `/tmp/native_segment_given_tool_default_native.gsp` (2664 bytes). Both tools declare `[1]` and have a code-15/subtype-10 output referring to that input, with no `07d3` position record. Subtype 5 therefore cannot be classified as a segment or circle from the selected source used to create the tool. It is modeled as an abstract Path placeholder; concrete path matching and invocation placement need the separate Native binding controls. The six corpus subtype-5 entries must not be counted as six proved segment inputs. Other tags have no independently established ray, line, polygon, or coordinate-system meaning. A search of the 518 sibling `.htm`/`.html`/`.txt` files found no exported given declarations or custom-tool Script View descriptions. Paired HTML exports the main drawing; it does not establish the prototype input categories.

Class-11/subtype-51 matches the ordinary calculation subtype; subtype 57 matches ordinary code-71 function headers. Subtypes 79–81 resemble scalar/length/angle parameter subtypes, but these placeholders lack a usable parameter animation layout. A later Native invocation independently establishes subtype 79 as a scalar-Parameter input; subtypes 80 and 81 remain unclassified. Subtype 33, which accounts for 248 slots, is especially important: `12T.gsp` uses such a given as the source of many code-48 calculations. Its apparent `<0>` words and cache fields must not be interpreted as an authored numeric literal.

## Smallest complete-view candidates

The smallest complete point-only prototype is `Samples/未分类档/圆周展开(changxde).gsp`, tool 0, 2 objects, inputs `[1]`; tool 1 has 3 objects. `Samples/热研系列/迭代系列/m×n网格的交错填充.gsp`, tool 1, also has 2 objects and input `[1]`.

The smallest complete mixed-number candidate is `Samples/个人专栏/钟科作品/颗粒坐标系(3D).gsp`, tool 1, `2.实点(x,y,z)`: 15 objects, inputs `[3,2,4,1,5,6,7]`. IDs 3,2,4,1 are class-1/subtype-4 placeholders; IDs 5,6,7 are class-11/subtype-51 placeholders labeled x,y,z. This is the best bounded next scalar binding probe. Its baseline complete view proved ordinary isolated decoding only; contextual decoding now correctly leaves these unknown number-input categories unsupported.

The other complete mixed candidate is `Samples/热研系列/3D系列/也做正n棱锥表面展开.gsp`, tool 1, `三维空间中的点2`: 28 objects, inputs `[8,7,6,5,4,2,1,3]`. It contains subtype-60 given 2 and subtype-51 given 1. It should follow the simpler 15-object probe rather than establish the subtype-60 category by inference.

No complete prototype containing a subtype-5, 6, 8, 15, or 39 given was found. Their smallest raw candidates are the table examples and remain partial. The smallest subtype-33 numeric candidate is `Samples/个人专栏/向忠作品/网球线.gsp`, tool 0, 3 objects, input `[1]`; subtype-80 candidate `杨辉三角1（李章博）.gsp` has 5 objects. Function subtype-57 candidate `悬垂曲面(表达式的极小曲面).gsp` has 5 objects and inputs `[3,2,1]`. These are useful isolated category controls, not complete runtime goldens.

## Contextual representation boundary

The `EmbeddedToolInput` descriptor retains raw local ID/class/subtype. Current independently established categories are Point `(1,4)`, Path `(1,5)`, finite Segment `(1,14)`, generic Numeric `(11,33)`, and scalar Parameter `(11,79)`. A private contextual decoder receives the declared-input map while ordinary `Document::from_bytes` has no such context. Point givens remain neutral Free points for compatibility. Other proved categories become typed `Kind::ToolInput` roots without invented cached values or geometry; unknown categories become unsupported before fallback decoding. Known input groups retain their strictly bounded native placeholder record family and contextual labels. A prototype-only `Kind::ToolPointOnPath(host)` represents the parameter-free native operation without inventing a saved fraction. It has a real host dependency and cannot be encoded as an ordinary drawing construction. The root owns destination matching and concrete invocation materialization; function and other subtype meanings remain unimplemented pending independent proofs.

The reproducible scratch audit is `/tmp/tool_given_audit.rs` linked to the settled library; `/tmp/tool_given_audit.tsv` records all 103 tools and 515 headers, `/tmp/tool_given_audit_full.tsv` includes presentation/descriptor bytes, and `/tmp/tool_given_consumers.tsv` records raw dependent reference payloads. These scratch files are not retained fixtures or Native writer proofs.


## Numeric given follow-up

Independent Native creation and typed binding now establish class11/subtype33
as a generic numeric given, with a strict zero-token 94-byte placeholder.
Scalar, Length, Degrees, and Area controls and the independent ordinary
calculator unit comparison are retained and documented in
[tool_numeric_inputs_analysis.md](tool_numeric_inputs_analysis.md). Other
class11 tags remain separate prerequisites. The contextual Numeric placeholder
has no units until invocation binds its actual source.

## Scalar-Parameter follow-up

A Native invocation of the second `系列迭代案例.gsp` tool prompts specifically
to match Parameter `n`; selecting its visible scalar Parameter completes that
input and allows the construction to be created. Class11/subtype79 is therefore
modeled separately from generic Numeric. Its exact placeholder body shares the
inert numeric-given layout, including unusable cache-like step/domain bytes, but
destination matching accepts only a live scalar Parameter. This recovers 87
prototype slots while both source tools remain partial. See
[the focused analysis](tool_scalar_parameter_given_analysis.md).
