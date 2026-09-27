# Single-seed mapped iteration evidence (checkpoint 45)

The checkpoint 44 decoder accepted generic mapped roots with 2–16 point/Scalar
seeds. The same native parent schema also represents one seed. This audit concerns
drawing objects only; copying and custom-tool prototypes remain outside the task.

`single_seed_iteration_cohort_audit45.rs` is read-only. Compile against the library
and run normally for single seeds, or with `ALL_SEEDS=1` for other typed seed
blockers. Candidate is a prerequisite inventory, not a promise of Native semantics
or a final decoder recovery count. The checkpoint 44 snapshot is retained in
`/tmp/single_seed45_audit.log`; `/tmp/typed_seed45_audit.log` adds unit classifications.

Before production extension, the drawing inventory found 404 candidate roots in
178 files: 377 code 89 and27 code 76. They have720 point/segment bindings, a potential
1124 slots under the existing output types. There are1042 bindings of all types,
including231 Polygon,21 CenterArc,11 CircleBySegment,10 Circle,4 Ray and1
ThreePointArc sources, plus transformed sources whose output type is separately
checked. The 322 extra bindings need separate shape output support. Of 404 roots,
121 have paired HTM across 51 files. Seed types: 283 points and 121 Scalars (106
Parameter, 15 ParameterCalculation). Only one multi-seed candidate is blocked
solely by typed numeric units: Free +Length +Length in
`系列正方形迭代作法（钮炳坤老师）.gsp`, root 19. Other Length/Degrees candidates
also need unsupported predecessors or sampled loci.

## Native schema and original corpus

The parent header class remains `[14,0,64,0]`, with code76 for fixed depth or89
for a numeric depth. The24-byte090a payload has seed count1 atbytes6..8, branch
count at8..12, fixed/default depth at16..20, all/final image flag at4..6. Its
reference sequence is `[optional depth, seed, branch targets, carried nodes]`.
Binding code77 class `[15,0,65,0]` references `[source,parent]`. This requires no
additional shape×transform enum.

`/tmp/single_seed45_payload_and_htm.log` contains the complete raw groups, decoded
predecessors, and complete paired HTM for these original candidates:

- `tests/Samples/热研系列/迭代系列/三角网格.gsp`: root 15, depthValue11
  (`n−1=4`), scalar seed3 (`t=1`) →4 (`t+1`), carried `[4,10,12,13,14]`.
  Binding16/17/18 carries numeric-ratio dilation of triangle edges12/13/14.
  HTM explicitly defines triangle points259,278 /390,297, rotation60°, ratio
  `t/n`, and all three dilation prerequisites, while omitting the iteration.
  Native screenshot shows the subdivided triangle. Native save is independently
  byte-identical 3782 bytes. Retained as `single_seed45_triangle_corpus_native_verified`.
- `tests/Samples/个人专栏/贺基旭作品/简单表格制作（hjx4882）.gsp`: root 8,
  depthValue5 (`N=5`), point seed1 →GraphVerticalUnit3, carried `[2,3,6,7]`,
  segment binding9. Root12 analogously depthValue4 (`M=7`), seed1 →
  GraphHorizontalUnit2, carried `[2,3,10,11]`, segment binding13. HTM explicitly
  defines unit points, numeric dilation and the source edges, omitting iteration.
  Native screenshot shows a7×5 grid. Independent byte comparison succeeds 3710
  bytes. Retained as `single_seed45_table_corpus_native_verified`.

Native corpus captures have the usual content chrome offset(+59,+51). The table
also preserves its saved page pan(-414,-41), matching the HTM's adjusted origin
252,306 instead of the GSP's666,347. Do not compare its screenshot with unpanned
world coordinates.

## Fresh public API controls

`single_seed_iteration_generate45.rs` authors ordinary geometry through public
APIs and constructs the parent/bindings through
`add_mapped_point_iteration_with_bindings` with `VERIFY_API=1`. Raw insertion is
retained as a schema baseline. The initial API authored the caller’s carried order
verbatim; stable topological ordering was added after the live comparison below.
Native accepts and saves
all five initial API controls; each independently compares byte-identically with the
pre-Native API output. Fixtures have prefix `single_seed45_api_` and suffix
`_native_verified.gsp/.png`.

| Control | Bytes | Mapping and carried graph |
| --- | ---: | --- |
| derived |2528| Midpoint seed6 → vector translated point7; carried `[7,8]`, segment8 goes from seed to fixedB |
| geometric |2548| PointOnSegment seed6 at.25 → vector point7; carried `[7,8]` |
| branches |2624| Same seed → +40 or−40 vector points7/8; carried `[7,9,8]`, segment9 and point7 bindings |
| host_frozen |2544| Midpoint8 and image9 carried; Segment host7 external; image9 binding only, constant x242.5 |
| frozen |2608| PointOnSegment6 → translated midpoint9; midpoint8 omitted, carried `[9,7]` |
| live |2612| Same graph, midpoint8 carried, list `[8,9,7]`; Native packet-order behavior differs from topological DAG evaluation |
| live_ascending |2612| Same bytes except carried list `[7,8,9]`; Native gives point images281.25/300.625/310.3125 |
| live_free |2600| Unsorted live control with a Free seed instead of PointOnSegment; same Native discrepancy proves it is independent of seed type |

Native queues 60883 and 36645 both terminate0 and independently compare every API
file to its pre-Native copy. The unsorted-vs-sorted live difference changes only
carried reference order. Sorted captures show x340/360/370 (world+59), while
unsorted captures stop at~360 and their segment extent is shorter. The same
unsorted result persists after replacing the path seed with Free. This is evidence
that Native executes packet entries in an order-sensitive manner. The current
Rust shared value DAG evaluates dependencies topologically. These unsorted Native
controls are retained as historical packet-order evidence, not positive equivalence
proof. The public API now stably orders carried dependencies before their users,
so an input `[8,9,7]` authors `[7,8,9]`. Imported original bytes keep their original
packet order. Unordered nodes retain caller order; independent measurement
controls therefore keep their existing byte goldens.

The independent derived Native point images are world x270,310,350, y250.
Geometric images are x245,285,325, y250. These distinguish overriding the value
of a derived seed from reevaluating its original construction. Frozen images all
stay x242.5,y250. The branches produce the finite binary translation tree; x125
and x325 point images are both directly visible, while overlapping positions
make multiplicities unsuitable for screenshot-only assertions.

`single_seed_iteration_api45.rs` separately authors original parameter and
calculated-scalar seeds; branch is `a→a+b` with frozen external `b=1`, carried
`[sum,coordinateP,coordinateQ,segment]`. Native queue 19259 completes with independent
byte identity 3372 and 3338 bytes respectively. These fixtures are
`single_seed45_api_scalar_native_verified` and
`single_seed45_api_calculated_scalar_native_verified`. Coordinates imply scalar
states 2,3,4 without rewriting the original seed's Kind.

## Compatibility

Existing direct single-point/segment/circle/ray and branch-polygon layouts have
legacy adapters. Generic decoding must preserve those canonical forms; placing a
new one-seed arm ahead of them would change strict authored roundtrip kinds.
The generic public API must canonicalize those existing layouts. An early direct
point control found the binding's style/header class mismatch during strict
roundtrip before Native. After the shared binding style fix, fresh API direct-point, direct-segment
and branch-polygon controls all pass strict roundtrip and Native save checks.

Native queue 51744 terminates 0; direct-point 2204, direct-segment 2272 and
branch-polygon 2472 bytes each independently match the corresponding pre-Native
API output. Their fixture prefix is `single_seed45_api_direct_point`,
`single_seed45_api_direct_segment` and `single_seed45_api_branch_polygon`.
The branch-polygon control binds a carried point, proving that the common code77
point binding header is accepted with this legacy parent adapter.

Final fresh-library regeneration independently matches all ten positive API
goldens: derived, geometric, branches, frozen, live_ascending (the live API call
now canonicalizes its unsorted caller input), scalar, calculated_scalar,
direct_point, direct_segment and branch_polygon. The raw unsorted live and
Free controls remain historical evidence only. No final equivalence claim rests
on their unsorted geometry.

## Native midpoint hosts and strict construction checks

The initial geometry generator checked byte reencoding, which did not prove the
fresh document's semantic strict roundtrip. The root's stronger test exposed
`SegmentMidpoint(plain Segment)` reopening as `Midpoint(endpoints)` with an explicit
native `midpoint_host`. Fresh append now canonicalizes this alias and preserves
the host. The generator's `VERIFY_API` mode now calls `verify_roundtrip()` itself.
The native midpoint host is its actual construction parent, including when the
host remains external/frozen during iteration. Dependency ordering and freezing
must therefore use the object's native construction dependencies, not only the
endpoint-oriented Kind dependencies.

The additional `host_frozen` control carries midpoint8 and translated image9,
binds image9, and deliberately leaves its Segment host7 external. The mapped
seed is PointOnSegment6. A carried midpoint does not imply reevaluating its
omitted host: Native keeps image9 at x242.5,y250: its capture has no additional x281/300/310
images. Native queue 58334 terminates 0, independently comparing 2544 bytes to the
pre-Native API output. Retained as
`single_seed45_api_host_frozen_native_verified.gsp/.png`. The new evaluator yields
three copies of Point242.5,250 for binding11.

Against the normalized-host library, all six geometry API cases now pass the
actual fresh-document `verify_roundtrip()` check. All five previous positive
geometry goldens still match exactly, including the canonical sorted live golden.
The other five scalar/legacy API controls also pass actual strict roundtrip and
independent byte comparison. This supplies the stronger construction evidence
missing from the initial byte-only geometry check.
