# Derived point seeds with carried transformed segments

`dev/mapped_derived_transform_probe43.rs` creates controls through ordinary
Document geometry and the existing mapped definition writer, then patches the
counted references and seed count to bypass the current mapped graph guard.
The carried binding keeps the already verified native segment header and changes
only its source reference from the base segment to its generic transformed
segment. This is evidence preparation, not a production decoder exception.

The common graph is:

| ID | Construction |
| --- | --- |
| 1 | A = (80,160) |
| 2 | B = (160,160); edited control uses (200,160) |
| 3 | segment A–B, the native midpoint host |
| 4 | midpoint M of A–B |
| 5 | Q = (120,200) |
| 6 | N = M + (B−A) |
| 7 | R = Q + (B−A), carried prerequisite with no visible binding |
| 8 | segment M–Q |
| 9 | rotation of 8 by 90° about M, or dilation by 0.5 about M |
| 10 | fixed-depth 3 mapped definition |
| 11 | carried segment binding [9,10] |

The two-seed mapping is `[M,Q] -> [N,Q]`, carrying `[8,9,6,7]`.
Definition references are `[4,5,6,5,8,9,6,7]`; its 24-byte `090a` has seed
count 2, branch count 1, all-level flag 0, and cached depth 3. The single-seed
control uses `[M] -> [N]` and references `[4,6,8,9,6,7]`, with seed count 1.
Both forms leave A and B unchanged during substitution. The midpoint M is
substituted as a point value; its upstream A/B construction is not reevaluated
to overwrite that substitution.

For both forms, the rotation's carried images have world endpoints:

| Generation | Start | End |
| --- | --- | --- |
| 1 | (200,160) | (240,240) |
| 2 | (280,160) | (320,320) |
| 3 | (360,160) | (400,400) |

The corresponding dilation endpoints are `(200,160)–(160,180)`,
`(280,160)–(200,180)`, and `(360,160)–(240,180)`. These are reevaluated
transformed constructions, rather than uniform translations of the original
transformed segment: Q stays fixed while M advances.

With B changed to (200,160), initial M becomes (140,160), and B−A is (120,0).
The rotated images become `(260,160)–(300,300)`, `(380,160)–(420,420)`, and
`(500,160)–(540,540)`. This verifies the initial derived seed and live upstream
vector both change, while the sequence still overrides the midpoint value.

The two-seed controls were additionally opened and saved on a 1280×1024 X
screen, with `alt+space x` maximizing the Native document. Their retained captures
show all three transformed segments with complete endpoints. World-to-capture
offset is (59,51), so the edited third segment is visibly
(559,211)–(599,591), exactly (500,160)–(540,540) in document coordinates.
The initial single-seed captures retain the ordinary window and show the first
rotation endpoint and subsequent clipped slopes; its dilation segments are all
visible. Their offset is (59,69).

All five controls opened and saved successfully in native Sketchpad 5, and each
saved file compared byte-identically with its independent writer input. The
retained `.png` beside each fixture is the actual pre-save Native capture.

| Retained fixture (`tests/fixtures/`) | Bytes | SHA-256 |
| --- | ---: | --- |
| `mapped_derived43_single_rotation_native_verified.gsp` | 2580 | `9ee91180176534e469a059607a37b2766479e24e7d772f393c31f2b5bd7c8957` |
| `mapped_derived43_single_dilation_native_verified.gsp` | 2556 | `b085a6804b9dcfa4cfe80150c470d9b5c664563fc0e61f7bd539b3536e5871c5` |
| `mapped_derived43_two_rotation_native_verified.gsp` | 2588 | `4c6d828292ef63c84ad0dcee1775ddc94046d76c72a5ce0bd34d92f172b87ef7` |
| `mapped_derived43_two_dilation_native_verified.gsp` | 2564 | `31566bfdb71e59ddb3c454372a95283fd51e9992ddf39df5191b982632bd9677` |
| `mapped_derived43_two_rotation_edited_native_verified.gsp` | 2588 | `90b39efc4cb999affdf81445d545335a84294373ef6d57cc85cb0fc8c5347313` |

Logs are `/tmp/mapped_derived43_{STATE}_native.log`; the queue script completed
with exit 0 after all four remaining Native processes and their `cmp` checks.
The first single-rotation process also completed with exit 0 and its subsequent
independent `cmp` passed.

These controls prove midpoint seed substitution, one- and two-seed headers,
fixed rotation and dilation of carried finite segments, and reevaluation of a
shared derived center in a sequence. They do not by themselves prove arbitrary
numeric seeds, geometric numeric measurements inside a sequence, circles,
polygons, or every transform flavor. The single-seed header overlaps the older
specialized iteration layouts; consolidating those representations must retain
strict roundtrip behavior for their existing authoring APIs.

The wide captures ran under `/tmp/mapped_derived43_wide_native.sh`; all three
Native processes completed successfully and their independent `cmp` checks
passed. `/tmp/mapped_derived43_{STATE}_wide_native.log` records the exact saved
size. After the production graph guard extension, the generator independently
constructs the same two-seed documents via
`add_mapped_point_iteration_with_bindings`, runs `verify_roundtrip`, and asserts
byte identity against the raw writer controls. All three public API constructions
(two rotation, two dilation, edited rotation) reproduce their retained Native
fixtures exactly. Single-seed records remain evidence for representation
consolidation; the public mapped API still requires at least two seeds.

## Original five-branch golden-ratio composition

The original corpus source
`../gsp-rs/tests/Samples/个人专栏/贺基旭作品/五边形迭代（hjx4882）.gsp`
and its paired HTM provide a larger composition control. The HTM exports only
the fifteen original constructions; its rotations use 1.88496 radians (108°)
and dilations use 0.618034, the inverse golden ratio. Its A/B coordinates
(251,503), (363.073,501.853) differ from the precise native document coordinates
(291,486), (403.0729490168751,484.8531695488855) by a viewport translation of
(-40,+17); this is not a geometric discrepancy.

The raw native graph uses three consecutive 108° point rotations to construct
the remaining pentagon vertices, followed by five dilations with precise factor
0.6180339887498949 and centers among those derived rotated vertices. Definition
16 uses free seeds [1,2] and five simultaneous branches:
`[10,6]`, `[6,8]`, `[8,9]`, `[9,7]`, `[7,10]`.
Its carried list is every construction from 3 through 15. The definition bytes
are `000000000100020005000000d53d00180200000000000000`: final-only flag 1,
seed count 2, branch count 5, cached depth 2. Bindings 17–21 carry the five base
sides 11–15, respectively, with reference pairs `[11,16]` through `[15,16]`.
Thus two substitutions produce 25 final pentagon states and five carried side
images per state, without drawing the intermediate generated pentagons.

An untouched disposable copy opened and saved in actual Sketchpad 5, with all
4,332 bytes identical to the original. The maximized pre-save screenshot shows
the 25 small blue pentagons grouped in five outer clusters, the original labeled
vertices A/B and 1/2/3, and intermediate construction point labels 4–8. This
provides native composition evidence for all five branches and repeated
rotation/dilation DAG evaluation, beyond the isolated writer controls.
The original unsupported conditional action remains in the file; it was not
activated by the proof. The paired HTM omits the iteration and cannot establish
its branch layout or final-only mode by itself.

Retained proof:
`tests/fixtures/mapped_pentagon43_corpus_native_verified.gsp` and its `.png`;
SHA-256 `b46d080eef1711f4f308baab09a2a5912a87628b25d1a6c983571e29570fe633`.
Native process session 33114 completed with exit 0; log
`/tmp/mapped_pentagon43_corpus_native.log` records the selected filename and
4,332-byte saved copy. A subsequent independent whole-file byte comparison
against the original corpus source passed before retaining the fixture.

The retained pentagon regression checks25 final generations for each of its five
side bindings, adjacency, and native side lengths: each branch scales the seed
side by φ⁻², so two substitutions give φ⁻⁴. It preserves the original4332 bytes
and its structural edit restriction. The source graph's recovery does not imply
that a partial imported document has become freely editable.
