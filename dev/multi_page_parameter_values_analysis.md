# Existing parameter values in multi-page documents

The original Ant mapped-iteration sketch has 98 known objects over two pages.
Its first-page depth parameter is object 25 (`n = 3`); its independent second-page
parameter is object 73 (`n = 4`). The decoder intentionally marks all imported
multi-page sketches partial: its single-preamble/global-count policy rejects
multiple page preambles. This flag therefore does not imply unknown objects in
this document. The decoder policy remains unchanged.

The existing scalar-parameter value setter now has a bounded multi-page path.
It requires a saved parameter on the active page, no image read-only state, no
unsupported/deleted objects, and exactly the original object count. It constructs
a candidate document, serializes it through the multi-page source-state writer,
and reparses/compares it before recording history or changing the live document.
Single-page parameter edits retain the existing partial/safe-structure guards.
Parameter additions and other numeric families are outside this change.

The writer compares every source object against its original object, retaining
its existing structural checks. A changed ParameterGroup must exactly match the
result of the typed value setter. When a prior fractional edit permanently
raised display precision, a precision witness followed by the final value must
reproduce the entire group exactly. Arbitrary caption, name, range, reference,
style, and opaque-field mutations cannot pass that comparison. The replacement
keeps the already-patched original 28-byte group header and inserts only the
checked typed group; variable-length replacements are applied from the end.
All unmodified records stay byte-for-byte in place.

For supported default parameter tables, only the last row may change. Their
sources remain page-local in the serialized group and global in the decoded
object graph. A paired table may update either or both parameter columns. The
writer reconstructs these changes from the changed same-page parameters and
requires exact equality with the proposed table group. Earlier rows cannot
change. The table setters preserve imported hidden-header state and observed
padding bytes when text alignment stays the same; a changed alignment width uses
canonical padding and reparses before mutation. Their
existing numeric representation supports two decimals and values within ±99.99;
this change does not broaden that table schema.

`multi_page_ant_parameter_depth_edit_is_local_atomic_and_undoable` verifies the
first-page depth edit, lower iteration count, unchanged second-page objects and
images, save/reopen, undo/redo, rejected off-page/nonfinite edits, and subsequent
second-page editing. `multi_page_parameter_tables_keep_local_refs_rows_and_other_page_bytes`
verifies both table kinds, previous rows, page-local references, variable text
length, byte-identical second-page storage, retained precision after a later
integer edit, and failed edits without history changes. It also rejects an
unrelated caption mutation and an unsupported object.

The two-page table fixture starts with a Rust-authored single-page parameter/
paired-table/default-table document and duplicates its page block using the
existing native page metadata schema. The separate `copy_page` API initially
rejected this source with “Copied page changes a source object”; that validation
gap is separate from editing saved values. The fixture itself cross-opens and
saves byte-identically in native Sketchpad (4,736 bytes).

`tests/multi_page_parameter_value_replay.txt` exercises the actual writer. The
Ant `n = 2` output cross-opens and saves byte-identically (11,254 bytes), and the
native capture `/tmp/mapped_point_iteration_ant_parameter_edited_port_native_before_save.png`
shows the edited depth. Native verification copies each file before opening,
requires the round-trip script to succeed, and then compares the saved bytes.
The decimal edit `t[1] = -12.34`, `t[2] = 7.50` and its later integer edit
`t[1] = 8.00` likewise cross-open and save byte-identically (4,758 and 4,740
bytes). Native before-save captures are
`/tmp/two_page_parameter_tables_edited_port_native_before_save.png` and
`/tmp/two_page_parameter_tables_integer_port_native_before_save.png`; they show
both live parameters and the paired table's updated final row, with the earlier
row intact. All four round-trip commands completed successfully before byte
comparison.

## Copying and reordering table pages

Page blocks retain their serialized local source IDs when copied or reordered.
The page-operation validation previously remapped those embedded IDs as though
all numeric records used document-wide IDs. This rejected valid table copies.
It now remaps only the model references while preserving each numeric group's
bytes. The regression copies page 2 of the two-page parameter/table fixture,
reorders its copied page to the front, and checks save/reopen and undo/redo.
All 12 page-operation tests pass.

`tests/parameter_table_pages_replay.txt` generates both actual writer outputs.
Native Sketchpad opens and saves each byte-identically (6,880 bytes). The native
script succeeded before comparing each file; captures use the prefixes
`/tmp/parameter_table_pages_{copied,reordered}_native`. The source tables retain
their rows and page-local parents across both operations.
