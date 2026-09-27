# Native clipboard boundaries

Copying a selected point does not necessarily copy its whole construction.
The controls below use direct Native selection, CtrlC/CtrlV, positive selection
and result captures, and independent reopen/save checks. API cross-open alone
establishes file acceptance; it does not establish Native selection semantics.

For cube84, selecting NumericCoordinatePoint14 alone produces Free15 at the
current point position plus20px in each coordinate. The new header retains
ordinary visibility, Medium point size, and red RGB; it changes code69/subtype56
to code0/subtype4. No label is added. Selecting numeric points13 and14 together
produces only two free points15/16 with the same displacement and styles.

The direct Native controls are
`numeric_calculation_cube84_point_clipboard_native_verified.gsp/png` and
`numeric_coordinate_two_points_clipboard_native_verified.gsp/png`. The latter
reopens and saves byte-identically at3,756 bytes. The bounded single-point API
implementation has its own independently verified exact3,570-byte writer:
`numeric_coordinate_point_clipboard_writer_native_verified.gsp/png`.
A regression checks its exact bytes, Native point position/style, source-edit
independence, save/reopen, and undo/redo. Ordinary clipboard copying still has
unproved families and partial-selection boundaries.

CtrlA of the complete cube84 graph preserves live numeric point construction,
with28 objects after paste. It converts suppressed calculation5 flag2 to
ordinary hidden flag1 on copied20. It also changes original vertical-unit9
flag4 to copied23 flag1. The latter is a separate frame visibility case; the
current live-graph test does not establish full frame presentation parity.
See `numeric_calculation_cube84_full_graph_clipboard_native_verified.gsp/png`
and the corresponding section of `numeric_native_import_analysis.md`.

Two attempts to Shift-select a numeric value box and geometric point did not
produce a valid mixed selection. They do not establish partial numeric graph
copy behavior or rejection. Numeric caption/value-box editing must be separated
from the intended selection gesture before interpreting such a control.

## Midpoint relationships

The source has Free1/2, Segment3, Midpoint4, and PointOnSegment5. Selecting
Midpoint4 alone copies only Free6 at(270,200), with the original red Medium
point style. Selecting endpoint1 and Midpoint4 preserves the construction:
Native allocates Free6/7, hidden Segment8, and live Midpoint9. Both endpoints
plus Midpoint produce the same construction and hidden helper. Selecting
Segment3 and Midpoint4 preserves Segment8 as visible. The unselected
PointOnSegment child5 is excluded in every case.

Thus a selected ancestor preserves a midpoint relationship even when the full
ancestor graph was not explicitly selected. The segment included solely as a
midpoint helper becomes ordinary hidden flag1; its original remains visible.
The direct Native controls and positive independent reopen/save captures are
`midpoint_{alone,one_endpoint,both_endpoints,segment}_clipboard_native_verified.gsp/png`,
with exact sizes2,188/2,388/2,388/2,392 bytes. Allocation order differs between
Native and the API, so tests compare copied positions, graph roles, visibility,
and independent live edits rather than claiming identical object IDs.

The implemented root boundary applies only to NumericCoordinatePoint,
Midpoint, and SegmentMidpoint. These roots freeze when no selected ancestor
exists; selecting an ancestor preserves the live construction. Traversal rejects
invalid, future, deleted, unsupported, and other-page references before taking
a snapshot. Frozen midpoints drop their old host. Unselected midpoint hosts
included in a retained construction become ordinary hidden; selected hosts
retain their ordinary visibility. Other point construction families,
unselected points used as prerequisites, locus seeds, and iteration bindings
still need independent Native controls before changing their copy semantics.

The four actual API writer outputs separately open and save unchanged in Native,
with positive free-point/live-point geometry and the expected helper visibility.
They are retained as
`midpoint_{alone,one_endpoint,both_endpoints,segment}_clipboard_writer_native_verified.gsp/png`
at2,176/2,368/2,368/2,368 bytes. These establish the writer independently of the
Native UI-created graph ordering. The explicit live-graph clipboard tests
select the prerequisites they intend to retain; their graph assertions remain
unchanged.
