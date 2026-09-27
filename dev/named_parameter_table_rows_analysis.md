# Named scalar table integer and negative rows

Ordinary named tables now share the numbered-table decimal row parser. Native
Tabulate controls establish zero-precision integer 3, negative –2.5 and the
bounded negative –99.99 row. Their rich-text lengths count UTF-8 bytes: 1, 6 and
8 respectively. The negative sign is U+2013 EN DASH. Each control completed the
central Wine helper and independently reopened/saved byte-identically.

New table creation and Add Table Data use the same formatter as live rendering.
Historical ASCII-minus named rows remain readable and retain their bytes. UTF-8
boundaries are checked before slicing; malformed lengths and numeric spelling
remain rejected. Row snapshots, removal, structural undo/redo and save/reopen
are covered for all three controls.

A clean imported parameter exposed a second issue: candidate table creation
appended an object without setting dirty when there were no unnamed parameters.
Serialization then returned the untouched original document and failed its
object-count check. Creation now marks the candidate dirty before verification;
undo still restores the untouched imported bytes.

Controls:

- [integer](../tests/fixtures/numeric_named_table_integer_native_created.gsp)
- [negative](../tests/fixtures/numeric_named_table_negative_native_created.gsp)
- [negative wide](../tests/fixtures/numeric_named_table_negative_wide_native_created.gsp)

Proof log: `/tmp/sketchpad_named_table_rows_native_controls.log` (exit 0).
The bounded library gate passes 597 tests (110 clipboard/copy/paste/tool tests
excluded), and corpus checkpoint 25 retains 1599/1599 exact round trips and
105274 unsupported drawing/prototype slots. Logs:
`/tmp/sketchpad_named_table_rows_lib_final.log` and
`/tmp/sketchpad_corpus_checkpoint25.log` (both exit 0).

Display does not need to reproduce the original application's pixels exactly.
Priority now goes to widely used features and their dependencies, following the
user's direction. Copying and custom tools remain deferred.
