# Checkpoint 44 Native path and intersection mapped controls

`dev/mapped_path_intersection_probe44.rs` constructs six explicit two-seed
mapped iteration packets, using the validated code76 definition / code77
segment binding layout. The positive controls independently reconstruct the
same bytes through `add_mapped_point_iteration_with_bindings` and perform strict
`verify_roundtrip`. These controls use real Sketchpad 5 opening and saving;
reader acceptance alone is not treated as geometric proof.

Generate with the current built library:

```
rustc --edition=2024 -L dependency=target/debug/deps --extern sketchpad_rs=target/debug/libsketchpad_rs.rlib dev/mapped_path_intersection_probe44.rs -o /tmp/mapped_path_intersection_probe44
SKIP_LABELS=1 /tmp/mapped_path_intersection_probe44 /tmp/mapped_path44_full
```

## Packet and construction

Seeds are A=(140,250), B=(240,250). Fixed anchors are U=(80,80),
V=(120,80), W=(80,120). Segment S=A–B, midpoint M=(A+B)/2, and
R=B+(V−U). All constructed nodes, including fixed anchors and intermediate
paths, are explicitly carried: object IDs3 through N inclusive. The target
branch is [N,R], depth3, all levels, and S has a segment image binding.
All auxiliary originals are hidden; A,B,S,N,R remain visible.

| Control | N | Definition/binding IDs | Native bytes |
| --- | --- | --- | --- |
| path_segment | point at fraction0.25 on S | 10/11 | 2576 |
| path_circle | normalized direction(0,1) on circle(M,A) | 11/12 | 2652 |
| line_line | horizontal line through A intersect vertical line through M | 14/15 | 2840 |
| line_circle | vertical line through M intersect circle(M,A), code12 | 13/14 | 2768 |
| circle_circle | circle(A,B) intersect circle(B,A), code14 | 12/13 | 2696 |
| line_circle_undefined | circle(A,B) intersect fixed horizontal line y=280 | 14/15,16 | 2904 |

For all positive controls B advances (280,250),(320,250),(360,250).
The expected native generated A coordinates are:

| Control | Generation1 | Generation2 | Generation3 |
| --- | --- | --- | --- |
| path_segment | (165,250) | (193.75,250) | (225.3125,250) |
| path_circle | (190,200) | (235,173.521849295) | (277.5,154.590330075) |
| line_line | (190,250) | (235,250) | (277.5,250) |
| line_circle | (190,300) | (235,326.478150705) | (277.5,345.409669925) |
| circle_circle | (190,336.602540378) | (310,371.243556530) | (420,319.282032303) |

The captures have document-to-screen translation(+59,+51). Noncollinear
controls visibly separate the three carried segment images. Circle-circle
revealed an existing geometry error: the pre-fix Rust variant0 chose
negative screen-y, whereas native code14 chooses positive screen-y. The
original N already exposes this discrepancy without iteration.

## The carried list is semantically significant

Earlier abbreviated controls carried only [S,N,R]. These are accepted and
saved unchanged by Native, but they **freeze unlisted prerequisites**. For
circle-path N the midpoint/circle stay at original geometry, so all three
segments fan out from (190,200), while B still advances. With the full carried
list all intermediate constructions update, and the three starts follow the
coordinates above. The analogous line-circle abbreviated control freezes its
center/radius and fans from (190,300). The generator can reproduce omitted-prerequisite packets with
`SKIP_LABELS=1 OMIT_PREREQUISITES=1`. This is direct evidence that unrestricted
reevaluation of every reachable ancestor disagrees with Native when a dependent
ancestor was omitted from the packet's carried list.

Retained freeze controls:
`tests/fixtures/mapped_path44_path_circle_omitted_prerequisites_native_verified.gsp`
and
`tests/fixtures/mapped_path44_line_circle_undefined_omitted_prerequisites_native_verified.gsp`,
each with its `.png`.

## Undefined geometry preserves a valid prefix

The last full control instead maps [A,B]→[M,B]. A progresses190,215,227.5 at
y250, B stays240,250, so radius shrinks50,25,12.5. The carried intersection N
with y280 exists at generation1: (230,280), then is undefined at generations2/3.
It has a point image binding in addition to the independent segment binding.
The native screenshot shows the first generated blue point at screen(289,331),
just left of original red N=(235.393920142,280), and no later point images.
The segment image remains visible. An evaluator that returns `None` for the
entire sequence when one later source is undefined loses valid Native output.
This packet does not make a branch target itself undefined; that is a separate
case requiring evidence.

## Retained Native results

All six final full controls opened and saved successfully through
`dev/wine_roundtrip.sh`, correct `/home/w/.cache/sketchpad_rs_wine` prefix,
serialized helper lock, wide screen and the standard maximize setup. Process
session8124 ended with exit0 after six independent whole-file `cmp` checks.
Logs are `/tmp/mapped_path44_full_{CONTROL}_native.log`. Retained artifacts are
`tests/fixtures/mapped_path44_{CONTROL}_native_verified.gsp/.png` using the six
control names from the table. These are writer controls, not original corpus
files; no paired HTM evidence is claimed for them.

## Independent transformed-point label defect

An initial labeled baseline failed before the main window, even without any
iteration. Same-environment known-good43 Native fixture opened and saved
unchanged, ruling out startup/prefix failure. A path-point N-only label baseline
opened/saved2424 bytes exactly. A vector-translated R-only label baseline failed.
Its group was header, label07d5, font07d8, refs07d2, terminator. Moving the label
and font after refs made the exact same base geometry open, show R and save
2424 bytes unchanged. Process86080 ended successfully and independent `cmp`
passed. Retained `tests/fixtures/vector_point_label44_native_verified.gsp/.png`
proves ordering header, refs07d2, label07d5, font07d8, terminator for a vector
point transform. This does not independently establish all other transform
flavors' label ordering.

## Additional literal and numeric rotation label controls

`dev/transform_point_labels_probe44.rs` starts from the original typed-parameter
fixture, sets degree parameter1 to60°, adds source(240,250) and center(140,250),
then adds either a fixed90° point rotation or a numeric60° rotation. Both result
points receive label T and pass strict authoring roundtrip. Native process27924
completed with exit0 and both independent whole-file comparisons passed.
Captures show T at(140,150) for fixed rotation and(190,163.397459622) for numeric
rotation (plus the same screen offset). Retained artifacts:

- `tests/fixtures/rotation_point_label44_fixed_native_verified.gsp/.png`,2692 bytes.
- `tests/fixtures/rotation_point_label44_numeric_native_verified.gsp/.png`,2652 bytes.

Together with the vector-label raw reorder this proves point labels after
references and any literal constraint for vector, fixed rotation and numeric
rotation. It does not by itself prove every remaining transform family.


## Labeled full controls after writer correction

All six full controls also opened and saved unchanged with A/B/N/R labels.
The first four completed in session37749. Circle-circle initially failed because
its generic code13/14 writer placed N's label before references; raw-moving that
label and font after references corrected it. Session99151 completed exit0 after
both the corrected circle-circle and the labeled undefined-prefix control, each
with an independent whole-file byte comparison. These label defects were
isolated from the mapped DAG semantics by baseline-only and known-good controls.

Retained artifacts use
`tests/fixtures/mapped_path44_{CONTROL}_labeled_native_verified.gsp/.png` for all
six control names. Original labeled circle-circle packet records are
header07d0,refs07d2,label07d5,font07d8,terminator07d7. Vector-point labels share
that ordering; literal rotations insert their literal constraint before labels.

Production completion: the shared ephemeral evaluation context freezes external
construction values without replacing original Kind metadata. Full and omitted
carried graphs now match the Native generation coordinates; individual undefined
carried images preserve valid prefixes and independent bindings. Circle-circle
intersection keeps the oriented Native result order. Point transform labels and
circle-circle labels follow their references. The public API recreates the
labeled circle-circle golden exactly; fixed/numeric point rotation goldens are
also reconstructed exactly. Eight focused regressions cover these semantics,
measurement/arc controls and frozen numeric/function environments. The scoped
library/editor/numeric/coverage gates pass817 tests; the final1599-file corpus
scan retains all bytes and restores422 drawing slots. Copying and custom tools
remain outside this work; undefined branch targets remain an explicit limitation.
