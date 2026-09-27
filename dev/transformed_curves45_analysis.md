# Transform source families, checkpoint 45

This read-only drawing-only audit uses current library decoding, native object
header offsets and page-local reference lists. Embedded prototypes are excluded.
The independent audit [transformed_curves45_audit.rs](transformed_curves45_audit.rs)
ran to exit 0 against `libsketchpad_rs-e6192eec59300771.rlib`, producing
`/tmp/transformed_curves45_audit.log`. It does not change production decoding.
Direct-ready means every direct reference already decodes; it does not assert
that source types, layout or finite geometry meet all evaluator/writer guards.
Descendant counts follow original raw reference lists, include the root, deduplicate
within each cohort and only count currently unsupported drawing slots. They are
upper bounds on dependency reach, not independently recoverable slot predictions.

| Native transform cohort | Unsupported roots | Direct-ready roots | Distinct unsupported slots reachable from ready roots |
| --- | ---: | ---: | ---: |
| Code 16, class 12 / subtype 67 | 50 | 23 | 55 |
| Code 16, class 13 / subtype 68 | 18 | 0 | 0 |
| Code 16, class 16 / subtype 78 | 27 | 0 | 0 |
| Code 34, class 4 / subtype 52 | 1340 | 218 | 1319 |
| All arc transform flavors, class 12 | 483 | 252 | 1145 |
| All polygon transform flavors, class 4 | 1701 | 301 | 1703 |
| All straight transform flavors, class 2 | 613 | 141 | 1369 |

Families overlap through shared downstream constructions. None of the last
three rows should be added together as an implementation estimate.

Class 12/subtype 67 is an **arc**, not a locus: current CenterArc, ThreePointArc
and ArcOnCircle native headers use this identity. For example, original
`半圆在地面的上的滚动（烟台飞狐）.gsp` object 33, code 16, references
`[21,1,31]`; source 21 decodes as `CenterArc(1,19,20)` and vector endpoints 1
and 31 decode. Object 83 has equivalent references `[66,46,76]`. Their groups
are `07d0:28,07d2:16,07d7:0`, with no operation-specific literal record.
The paired HTML is retained in `/tmp/transformed_curves45_paired_htm.log`;
it omits the arcs, so it establishes predecessors rather than full arc transform
semantics. No Native arc proof is claimed in this audit.

Class 13/subtype 68 examples reference code 82 sector interiors, which currently
remain unsupported. They need an arc-interior element prerequisite, not a
function plot or locus adapter. Class 16/subtype 78 examples include code 99
sources plus `090c:16` image-size records; they belong to picture handling.
All 27 raw code-16 examples lack a matching HTML file. Neither family has a
direct-ready root; neither should be enabled based only on a class-number guess.

## Shared polygon source restriction

Code 34 is **Reflection**; code 16 is **VectorTranslation**. Polygon subtype
is `49 + vertex_count`, so subtype 52 is a triangle and 53 is a quadrilateral.
`decode.rs` currently admits a reflected polygon only when its source is the
literal `Kind::Polygon`. The model and evaluator already represent transformed
polygons through `Kind::Transformed` / `ElementType::Polygon`, and
`kind.is_polygon()` and `polygon_vertex_count` traverse those chains. This is a
source predicate restriction, not a missing transform operation.

Original `万花筒（学方老师）.gsp` supplies explicit evidence:

- Object 54 already decodes as `Transformed(Polygon, Reflect{mirror:53})`,
  with source 29, a literal triangle.
- Object 61 is native code 34, class 4/subtype 52, raw refs `[54,5]`, group
  `07d0:28,07d2:12,07d7:0`; mirror 5 already decodes as a rotated segment.
- Paired HTML explicitly states `{54} Reflection(29,53)` and
  `{61} Reflection(54,5)`. It also states `{55} Reflection(33,53)` and
  `{62} Reflection(55,5)` for the green triangle used in the isolated capture.
- The exact 26410-byte original was opened and saved in Native, session 52243
  exit 0, followed by an independent `cmp` against the source. Fixture and
  capture: `tests/fixtures/reflected_polygon_chain45_corpus_native_verified.gsp`
  and `.png`. Log: `/tmp/reflected_polygon_chain45_corpus_native.log`.

[reflected_polygon_chain45_probe.rs](reflected_polygon_chain45_probe.rs) evaluates
existing source polygons and computes reflection by independent projection onto
mirror 5, rather than calling the production reflection helper. The original
full capture contains overlapping fills, so an additional control changes only
visibility flags and shows object 62 alone. All references and constructions
remain those of the original. This control is used to inspect all three vertices.
Native verification details are appended below after that queue completes.

The reusable implementation is to accept a polygon source through the existing
shared polygon predicate and recursive vertex-count contract. It needs no new
Kind variants. Apply the same evidence-first approach to other operation source
predicates: fixed Cartesian segment and fixed rotation/dilation polygon guards
also admit literal sources more narrowly than the generic evaluator.

For arcs, extend the shared element carrier and geometry adapter, preserving
analytic center/radius/start/sweep/orientation so path points and measurements
remain live. Every existing transform flavor can reuse its affine matrix.
Sampled paths are appropriate for plots/locus rendering, but should not replace
analytic arc geometry needed by constraints and measurements. Sector interiors
and pictures require their own reusable element geometry before transforms can
consume them. Copying and custom tools remain deferred.

The isolated control completed session 33366, exit 0, independent `cmp` exact
26410 bytes. The anchor control completed session 10944, exit 0, independent
`cmp` exact 26410 bytes. Fixture/capture pairs are
`reflected_polygon_chain45_isolated_native_verified` and
`reflected_polygon_chain45_anchors_native_verified` in `tests/fixtures`.
The first shows only original object62; the second additionally shows original
fixed points1/2, establishing the view offset independently of the transformed
triangle. Native point1 appears at `(514,377)` for normalized document
`(455,222)`, giving capture offset `(+59,+155)`. Point2 appears at `(730,375)`
for normalized `(671.2294734194975,219.6410161513777)` within pixel rounding.
The original paired HTML places point1 at `(455,326)`; the stored viewport has
an additional104-pixel vertical shift relative to normalized Rust coordinates.

| Original object62 vertex, independent reflection | Expected capture pixel | Native green hull tip | Distance |
| --- | --- | --- | ---: |
| `(410.7737675209622,302.02911684736046)` | `(469.7738,457.0291)` | `(471,456)` | 1.61px |
| `(437.62625115332287,281.7401993879789)` | `(496.6263,436.7402)` | `(498,435)` | 2.22px |
| `(461.53732413032475,235.36030000888354)` | `(520.5373,390.3603)` | `(519,393)` | 3.06px |

The narrow triangle's acute third tip is clipped by filled rasterization. These
are pixel evidence tolerances; numeric regression expectations should use the
analytic coordinates, not raster tip locations. The isolated green mask has
bounds `x=471..519,y=393..456` and convex hull
`[(471,456),(486,436),(518,394),(519,393),(498,435),(494,439),(474,454)]`.
Native logs are `/tmp/reflected_polygon_chain45_{isolated,anchors}_native.log`;
analytic values are `/tmp/reflected_polygon_chain45_analytic.log`.

Imported nonnumeric transforms already preserve their raw save records.
Authored compound reflections also need `transform_validation.rs`'s polygon
source predicate to recognize `F::Reflection`, since that currently falls through
to literal `Kind::Polygon`. This is separate from reader recovery; the original control proves imported dependency semantics. Fresh authored
controls below additionally verify the actual API record/cache profiles.

## Fresh public API authoring and edits

After the shared polygon source predicate was widened for reflection, the
[public API generator](reflected_polygon_chain45_generate.rs) creates seven
fixed points, two independent mirror segments8/9, literal triangle10,
reflection11 of10 across8, and reflection12 of11 across9. It uses public
`Document::add`, `Kind::ReflectedPolygon` (the shared generic transform
constructor), and `move_point`; no raw record patches are used. Only the two
mirror segments and final polygon are visible.

Three states pass `verify_roundtrip`, full decode after reopening, byte-identical
Rust re-encoding, and independent projection-based reflection calculations
within1e-9. Native queue79641 completed exit0 and independent `cmp` for all
three actual API files: each Native save remains exactly2580 bytes.
Fixture/capture pairs in `tests/fixtures`:
`reflected_polygon_chain45_api_{original,source_edited,mirror_edited}_native_verified`.
Logs: `/tmp/reflected_polygon_chain45_api_generate.log` and
`/tmp/reflected_polygon_chain45_api_{original,source_edited,mirror_edited}_native.log`.

| State | Public edit | Independent final vertices |
| --- | --- | --- |
| Original | None | `(380,480),(300,500),(340,420)` |
| Source edited | `move_point(1,(110,190))` | `(370,450),(300,500),(340,420)` |
| Mirror edited | `move_point(7,(460,360))` | `(340.1095890410959,538.9589041095891),(257.69863013698625,541.8630136986301),(313.4794520547946,471.94520547945206)` |

Native screenshots establish the geometry changes as well as acceptance and
exact saves. The fresh controls use capture offset `(+59,+51)`; unlike the
original corpus file they have no additional104-pixel viewport shift. This
completes the authored compound-reflection evidence as well as the imported
chain evidence, without adding a new transform/shape Kind variant.
