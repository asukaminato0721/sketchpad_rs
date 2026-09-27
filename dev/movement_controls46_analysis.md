# Checkpoint 46 movement Native controls

Production and tests are owned by other agents. These probes retain real original
record bytes and use the Native Movement group for conservative generated inputs.
`movement_controls46_generate.rs` creates the ordinary dependencies through existing
Document APIs. `movement_controls46_wire.py` adds the original button template,
changing only its ordered refs list and speed (Instant, except Slow control).
`movement_controls46_inspect.rs` reports actual saved construction fields.

## Original layout proof

Original corpus `个人专栏/况永胜作品/正方体的展开（3D效果）.gsp`
opened and saved in Native, terminal process exit 0, independent byte comparison
exact at 34196 bytes. It includes object42 three ordered movement pairs, paired
HTM `MoveButton(...)(21,13,2,12,15,8)`; object62 has no 07d5 label record.
Raw object42/object62 groups are retained in
`movement46_cube_three_pairs_group.bin` and `movement46_cube_unlabeled_group.bin`.

Original corpus `个人专栏/唐家军作品/课时6笔记.gsp` similarly opened and
saved Native exactly at 188102 bytes. Object367 on page5 has an additional empty
232e metadata record, retained in `movement46_notebook_empty_metadata_group.bin`.
This source has no paired HTM; this is wire acceptance proof, not click proof.
Both complete exact GSP files and before-save PNG captures are retained under
`tests/fixtures/movement46_{cube_missing_label,course_empty_metadata}_native_verified`.

## Click controls

The actual Native window in this environment offsets drawing coordinates by
(59,69), so the button at (10,5) occupies screen y75..94. The first attempted
click at y65 was inert. Those outputs are excluded as runtime evidence.
Corrected clicks use (95,84). Positive cases are recorded below only after their
specific process completes and source mutations are inspected.

Completed Instant/Follow controls:

- Ordered pairs A→B,B→C, initial (140,200),(240,240),(340,300), produce
  A=B=C=(340,300). Follow destination reevaluation or action processing matters;
  this does not justify assuming shared initial targets. Initial-destination
  and reverse-pair controls are required to distinguish mechanisms.
- Segment source retains `PointOnSegment(3,t)` and host, changing t=.25→.8;
  target (280,300) projects to source (280,200), host (120,200)–(320,200).
- Circle source retains host circle center(120,200),radius200, unit vector
  (0,1)→(.8479983040050957,-.5299989400031677), giving position
  (289.5996608010191,305.99978800063354), radial projection of (280,300).

These three corrected Native probes completed and saved before independent
source-field inspection. Both generated inputs and actual clicked outputs/PNG
are retained as `movement46_STATE_{input,clicked_native_verified}` fixtures.

Additional completed controls establish:

- Instant/Follow reversed pairs B→C,A→B gives A=(240,240),B=C=(340,300).
  Combined with forward pairs' all-C result this requires reverse pair
  processing for point destinations, using their live positions.
- Instant/initial-destination freezes shared initial positions: both forward
  and reversed pairs yield A=(240,240),B=C=(340,300).
- Slow/Follow forward pairs reach all-C after the 12-second observation;
  this proves endpoint behavior, not the exact native speed curve.
- Numeric Instant/Follow forward and reversed pairs a→b,b→constant7.125
  yield a=3.25,b=7.125 in both orders. Native samples numeric destinations
  before source mutation. Stored values retain exact fractions even though
  the display reads 3.3 and 7.13. Constant calculator literals normalize to
  native mode0 Parameter groups on save (separate unsupported decoder case).
- A Scalar→Length target7.125cm likewise stores Scalar7.125 in source b.
  Numeric movement uses destination magnitude across these units; it does
  not require equal unit tags in this completed raw control.
- Square-frame mixed horizontal/vertical sources trigger a Native modal and
  remain unchanged. This is a rejected source combination, excluded from
  positive setter evidence. Horizontal-only source is accepted: origin
  (200,300), target(275,200) changes horizontalstep37.795→75 and position
  (275,300), while verticalunit's persistedstep remains37.795.
- Ray and line source controls, same endpoints and target as segment, retain
  host/type and change their t=.25→.8, position(280,200).
- CenterArc source point changes .25→0 at arc start(320,200), host retained;
  target(280,300) is outside its directed sweep, so endpoint clamp is expected.
- Polygon source point changes edge0,t=.25→edge1,t=.36, projected(248,236).
  Native changes the final 07d3 reserved word0000→2039. The actual record is
  `0f0034000ad7a3703d0ad73f01000000ffff3920`; the old strict decoder rejects
  that cached metadata despite its established geometric fields.

Authored even/odd/empty button labels all opened and saved byte-exactly in
Native, using current action-record APIs' generated groups. Imported absent
label additions and fresh full Document API controls are recorded once their
specific live queues finish. No click with an unverified screen coordinate is
counted as evidence.

## Completed current public API controls

`movement_api46_generate.rs` calls `add_movement_button_for_pairs`, without
raw button injection or production guard bypass. Its three-Parameter forward
and reverse pairs both store a=3.25,b=7.125,c=7.125 after Native Instant/Follow
click. Their actual Native clicked outputs reopen through the current library
with `partial_view=false`, preserving their ordered pairs.

The two-path API button controls segment source4 and circle source6, both
targeting Free7=(280,300). Native changes segment parameter to.8 and circle
unitvector to(.8479983040050957,-.5299989400031677); hosts/types remain intact.
A separate read-only Native open/save of this exact full API output completed
with independent byte comparison exact at2428 bytes, retained as
`movement46_api_path_multi_native_verified.gsp/.png`. Actual clicked output is
2476 bytes and also reopens fully supported.

The rectangular-frame API button controls horizontalunit2→Free7 and
verticalunit4→Free8. Origin=(200,300), targets=(275,200)/(350,225), both initial
steps37.795. Native stores both steps75, positions(275,300)/(200,225), retaining
axes/frame refs. The 2672-byte clicked output reopens fully supported.

All six label probe queues and all movement control queues are terminal exit0.
Fresh input/actual clicked/PNG fixture names follow
`movement46_STATE_input.gsp`, `movement46_STATE_clicked_native_verified.gsp`,
`movement46_STATE_native_verified.png`. Label controls are read-only despite the
historical `clicked` queue output filename. The rejected square x/y source
combination uses `movement46_graph_rejected_native_verified.gsp`.

## Reproduction

Build the library, compile `movement_controls46_generate.rs`,
`movement_api46_generate.rs`, and `movement_controls46_inspect.rs` using rustc
with `-L dependency=target/debug/deps --extern
sketchpad_rs=target/debug/libsketchpad_rs.rlib`. The wire generator accepts
optional state arguments and never changes production sources. Compile
`movement_labels46_generate.rs` standalone for exact action-record label groups.

Use Wine WOW64 11.0, xvfb-run, xdotool, and ImageMagick on PATH, then run
`bash dev/movement_controls46_native.sh STATE click` or `STATE open`.
The helper sets the verified click(95,84), waits12 seconds for completion, and
uses the shared `wine_roundtrip.sh` flock with Wine prefix
`/home/w/.cache/sketchpad_rs_wine`. Open mode independently compares bytes.
`movement_controls46_collect.py` retains only controls with completed DONE
markers in logs, excluding pending files. No copy/custom-tool behavior was
implemented or tested by these controls.

## Edited graph writer proof

Fresh authored edited rectangular and square frames, generated through
`set_movement_value` before encode and `verify_roundtrip`, both opened/saved
Native byte-exact: rectangular2624 bytes, square2616 bytes. Retained as
`movement46_api_graph_{rect,x}_edited_native_verified.gsp/.png`.

A first imported-edit attempt used a stale library built before the code52
constraint_offset correction and failed `verify_roundtrip` before Native;
no Native proof is claimed for that failed attempt. The rebuilt current
library passes imported edit `verify_roundtrip`; dedicated Native imported
read-only controls are recorded after their queue completes. A separate square
frame transformed-Y point probe distinguishes persisted Y-unit metadata from
its actual evaluated location; no semantics are inferred from cached step.

All rebuilt imported edited writer controls completed Native byte-exact:
rectangular2624 bytes and square2612 bytes, retained as
`movement46_api_graph_{rect,x}_imported_edited_native_verified.gsp/.png`.

The independent square Y dependency probe is decisive. Source horizontalunit2
stores75, verticalunit4 still stores37.795. A visible vector-translated point
Y11 references unit4, origin1, target10=(300,300), so Y's x=300 and y directly
reveals unit4's evaluated height. Native places Y at(300,225), screenshot
(359,294), at the same y as target8=(350,225); stored-cache interpretation
would give(300,262.2047244), outside the visible lower boundary. Red pixel
components independently identify Free7 around(334,269), Free8 around(409,294),
and Y11 around(359,294). This requires square dependent Y geometry to consume
live horizontalstep75, while independent rectangular Y geometry uses its
storedstep. The full API-generated probe opens/saves byte-exact at2796 bytes,
retained as `movement46_api_graph_square_y_probe_edited_native_verified.gsp/.png`.
This is evaluator evidence; no extra enum variant is needed for the movement
source itself. Root is handling the shared geometry correction and regression.
