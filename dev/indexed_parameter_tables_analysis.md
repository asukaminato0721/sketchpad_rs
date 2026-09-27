# Indexed scalar parameter tables

The native controls
`numeric_parameter_table_unnamed_scalar_native_created.gsp` and
`numeric_parameter_table_all_unnamed_native_created.gsp` store a scalar source
named t[4] or t[6] after Tabulate materializes generated parameter names. Their
code-91 table headers encode the base letter and numeric subscript in separate
text runs: `000<-<SS2<T1xt>><T1x4>>`, followed by length-prefixed row values.
The existing plain-name table grammar refused those runs.

The decoder now accepts the bounded indexed grammar and retains the original
header, source reference, anchor, row cache and observed font index. It requires the
caption name to match the actual scalar parameter name. Length and angle
sources, typed literal-definition captions, other table layouts, malformed
UTF-8/text lengths, invalid row counts and unsupported metadata remain guarded.
Existing t[1] default-table and plain-name table representations remain intact.

Imported tables render their subscript title and current final-row value.
Snapshot addition/removal and movement preserve the indexed grammar and font;
structural undo/redo returns the exact original bytes. Creating a numbered scalar
table now materializes unnamed parameters on the page, restores verified caption
bounds, advances the page counter and adds/reuses the Tahoma 14 table font as one
transaction. A failed operation preserves the document and its undo history.
Naming is bounded to Tahoma 18 Normal decimal captions; other fonts and
scientific captions remain guarded. Generated t[1] now uses the same indexed
grammar as other generated names. The complete legacy fixed-two-decimal table
layout retains its original API; a subscript value alone no longer excludes
new native t[1] tables. The indexed negative row uses
U+2013 EN DASH and UTF-8 byte lengths, matching the Native control.

The actual API writers change the source to 2.25, add/remove a snapshot, and move
the table to (280, 140). Each opened and saved byte-identically in native
Sketchpad through the unchanged central Wine helper, with an independent
comparison against an untouched snapshot. Positive captures visibly show t₄ or
t₆ above 2.25. The retained writer goldens are:

- [scalar writer](../tests/fixtures/numeric_parameter_table_indexed_scalar_edited_writer_native_verified.gsp), 3788 bytes; [native capture](../tests/fixtures/numeric_parameter_table_indexed_scalar_edited_writer_native_verified.png).
- [all-named writer](../tests/fixtures/numeric_parameter_table_indexed_all_named_edited_writer_native_verified.gsp), 3808 bytes; [native capture](../tests/fixtures/numeric_parameter_table_indexed_all_named_edited_writer_native_verified.png).

Earlier probes at (80, 220) also saved byte-identically, but clipped the table
value at the restored MDI window boundary. The retained goldens use the visible
placement above. No caches, fonts or opaque fields were normalized for the
comparison. Native proof log: `/tmp/sketchpad_indexed_table_native_visible_gate.log`
(exit 0).

Tests assert actual writer bytes, names, live values, row counts, movement,
undo/redo and save/reopen. Mutated source-name mismatches remain Unsupported(91)
and preserve their bytes. The literal-definition table still imports losslessly
as unsupported. Corpus checkpoint 23 retains 1599/1599 exact round trips and
105274 unsupported drawing/prototype slots; the independent native fixtures
are outside that corpus. Copy and custom-tool work remains deferred.

The bounded library gate passes 594 tests (110 clipboard/copy/paste/tool tests
excluded). Both focused native-import gates pass. Logs:
`/tmp/sketchpad_generated_table_lib_final.log`,
`/tmp/sketchpad_generated_table_reject_final.log` and
`/tmp/sketchpad_generated_table_import_final.log`. `git diff --check` passes.
These gates establish the indexed-table change, not overall command completion.


Four creation writers each opened and saved byte-identically through the central
Wine helper, with a positive capture showing the complete table. The untouched
snapshots were compared independently; no opaque fields or caches were normalized.
The proof command completed with exit 0 (`/tmp/sketchpad_generated_table_native_gate.log`).
Tests assert the exact retained writer bytes as well as naming, caption bounds,
live rows, counter advancement and undo/redo.

| Writer | Native capture |
| --- | --- |
| [scalar](../tests/fixtures/numeric_parameter_table_generated_scalar_created_writer_native_verified.gsp) | [capture](../tests/fixtures/numeric_parameter_table_generated_scalar_created_writer_native_verified.png) |
| [all_named](../tests/fixtures/numeric_parameter_table_generated_all_named_created_writer_native_verified.gsp) | [capture](../tests/fixtures/numeric_parameter_table_generated_all_named_created_writer_native_verified.png) |
| [counter12](../tests/fixtures/numeric_parameter_table_generated_counter12_created_writer_native_verified.gsp) | [capture](../tests/fixtures/numeric_parameter_table_generated_counter12_created_writer_native_verified.png) |
| [negative25](../tests/fixtures/numeric_parameter_table_generated_negative25_created_writer_native_verified.gsp) | [capture](../tests/fixtures/numeric_parameter_table_generated_negative25_created_writer_native_verified.png) |


The counter-1 Native action control materializes t[1], advances the counter to 2
and writes a zero-precision row (`0<H<T1x0>>`). Its independent second open/save
was byte-identical, and the full table was visible in the capture:
[counter-1 control](../tests/fixtures/numeric_parameter_table_unnamed_scalar_counter1_native_created.gsp),
[capture](../tests/fixtures/numeric_parameter_table_unnamed_scalar_counter1_native_created.png).
The corresponding actual API writer also completed the central Wine helper with
exit 0 and an independent exact comparison. The retained
[writer](../tests/fixtures/numeric_parameter_table_generated_counter1_created_writer_native_verified.gsp)
and [capture](../tests/fixtures/numeric_parameter_table_generated_counter1_created_writer_native_verified.png)
are asserted by the creation test. Counter-1 regression log:
`/tmp/sketchpad_generated_counter1_lib_gate.log` (594 passed, 110 deferred tests excluded).


Default t[1] and named ASCII creation now use the same name transaction as
numbered sources, and all new single-column table fonts resolve against the
drawing namespace. Additional Native writer goldens, font collision coverage
and the 596-test bounded gate are documented in
[single-column table transactions](default_parameter_table_transactions_analysis.md).
Corpus checkpoint 24 retains the original 1599 exact round trips.
