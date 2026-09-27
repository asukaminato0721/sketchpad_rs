# Single-column parameter table transactions and drawing fonts

Creating a table from an already named t[1] used to bypass name materialization
for the other parameters on the page. The actual Native control tabulates a
named Scalar t[1] = 2.50 with an unnamed Scalar on the same page. Native writes
that other parameter as t[4], restores its caption rectangle and advances the
page counter from 4 to 5. The selected source remains t[1], and the table has a
2.50 row. The complete table is visible in the retained positive capture.

- [Native action control](../tests/fixtures/numeric_parameter_table_default_with_unnamed_native_created.gsp), [capture](../tests/fixtures/numeric_parameter_table_default_with_unnamed_native_created.png).
- [Actual API writer](../tests/fixtures/numeric_parameter_table_default_with_unnamed_created_writer_native_verified.gsp), [capture](../tests/fixtures/numeric_parameter_table_default_with_unnamed_created_writer_native_verified.png).

The Native action control independently reopened and saved byte-identically.
The API writer completed the unchanged central Wine helper with exit 0 and an
independent comparison against an untouched snapshot. No opaque fields, naming
caches or font metadata were normalized.

Single-column parameter creation now validates the source, prepares names,
counter, table and font reference in a candidate document, verifies save/reopen,
and commits one undo checkpoint. The same transaction covers default t[1],
automatically numbered sources and named ASCII sources. An out-of-range table
value preserves the other unnamed parameter, counter and prior value-edit undo
history. Successful structural undo restores the complete pre-creation bytes;
redo restores the exact writer bytes.

The old generated t[1] writers referenced font 1 but contained only font 0
(Tahoma 24 Bold). Table rendering therefore lacked the requested font reference.
Creation now adds/reuses Tahoma 14 Normal in the drawing namespace. The raw table
and its pending style override use the same resolved index. A pending Tahoma 18
text caption occupies index 1 in the collision test; the table correctly uses
index 2. Ordinary plain-name table parsing and snapshot edits retain the observed
font reference instead of forcing index 1. Other table layouts remain guarded.

The four affected legacy assertions now use distinct Native-verified writer
fixtures. Historical generated fixtures remain unchanged. All four actual
writers completed the central helper with exit 0, saved byte-identically against
untouched snapshots and visibly displayed the complete table:

| Writer | Native capture |
| --- | --- |
| [Axis value plot](../tests/fixtures/graph_plot_value_axis_font14_writer_native_verified.gsp) | [capture](../tests/fixtures/graph_plot_value_axis_font14_writer_native_verified.png) |
| [Fractional snapshots](../tests/fixtures/table_t1_snapshots_font14_writer_native_verified.gsp) | [capture](../tests/fixtures/table_t1_snapshots_font14_writer_native_verified.png) |
| [Later source ID](../tests/fixtures/table_t1_late_source_snapshots_font14_writer_native_verified.gsp) | [capture](../tests/fixtures/table_t1_late_source_snapshots_font14_writer_native_verified.png) |
| [Mixed-width negative snapshots](../tests/fixtures/table_t1_mixed_width_snapshots_font14_writer_native_verified.gsp) | [capture](../tests/fixtures/table_t1_mixed_width_snapshots_font14_writer_native_verified.png) |

Native proof log: `/tmp/sketchpad_default_table_font14_native_gate.log` (exit 0).

The named speed writer contains historical/live rows 2.5, 3.25 and 3.25. It also
completed the central helper with exit 0 and an independent exact comparison:
[writer](../tests/fixtures/numeric_parameter_table_named_font14_writer_native_verified.gsp),
[capture](../tests/fixtures/numeric_parameter_table_named_font14_writer_native_verified.png).
Tests assert its exact bytes, saved font style, row history and structural undo.

Corpus checkpoint 24 retains 1599/1599 exact round trips and the prior histogram.
These controls establish ordinary single-column table creation; the complete
application remains in progress. Copying and custom tools remain deferred.


The pending-font writer also completed the central Native helper with exit 0
and an independent exact comparison. The capture shows the Tahoma 18 text
caption and the Tahoma 14 named table together:
[writer](../tests/fixtures/numeric_parameter_table_named_pending_font_writer_native_verified.gsp),
[capture](../tests/fixtures/numeric_parameter_table_named_pending_font_writer_native_verified.png).
The test asserts exact writer bytes with all three drawing font records.

The final bounded library gate passes 596 tests, with 110 deferred
clipboard/copy/paste/tool tests excluded. The editor table/animation-source
check and both focused numeric-native-import checks pass. Logs:
`/tmp/sketchpad_default_table_transaction_lib_final.log`,
`/tmp/sketchpad_default_table_editor_gate.log`,
`/tmp/sketchpad_default_table_transaction_reject_gate.log` and
`/tmp/sketchpad_default_table_transaction_import_gate.log`. `git diff --check` passes.
