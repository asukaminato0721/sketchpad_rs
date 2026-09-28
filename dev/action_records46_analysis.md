# Checkpoint 46 shared action records

`src/action_records.rs` uses ordered `Vec<Record>` storage and named role
accessors for Movement, Hide/Show, Animation, Scroll, and Presentation groups.
Required records remain unique and ordered. The common layout allows an absent
07d5 label and one empty 232e after 07d8 and before 07d7. Unknown records,
duplicates, reordered roles, nonempty metadata, truncated labels, and truncated
payloads are rejected. Link Page and Link URL retain their specialized layouts.
Hide/Show action option validation is unchanged; optional records do not imply
new action semantics.

Movement references are a nonempty ordered list of nonzero source/destination
pairs. `target_pairs` / `set_target_pairs` and
`from_original_template_pairs` expose that list. `target_ids` returns the
flattened list. The existing constructor and `set_targets` remain single-pair
adapters; `set_targets` replaces the list with one pair.

Missing labels read as an empty string. Setting the missing label to an empty
string preserves absence. Imported labels preserve their complete metadata,
opaque suffix, and record padding. An unchanged caption is a byte-preserving
no-op. Editing an existing caption retains the former parity restriction;
Animation retains its previously proven zero-suffix parity conversion. Adding
a caption to a missing label inserts the family's Native template record before
07d8, with a one-byte zero suffix for odd text length and two-byte zero suffix
for even text length. Newly authored empty Movement captions omit 07d5.

Original wire proof is retained in `dev/movement_controls46_analysis.md`:
Native opened and saved the full cube and notebook sources exactly, proving the
three-pair list, absent label, and empty metadata layouts. Three extracted raw
groups are covered by exact roundtrip tests:

- `movement46_cube_three_pairs_group.bin`: [(8,28),(12,2),(22,41)].
- `movement46_cube_unlabeled_group.bin`: [(58,61)], absent 07d5.
- `movement46_notebook_empty_metadata_group.bin`: additional empty 232e.

Six added action-record tests cover those raw Native groups, invalid pair
counts and zero references, optional layout preservation, label insertion,
newly authored empty captions, opaque suffix and nonzero alignment padding preservation, and malformed record rejection. All 23 standalone
record tests pass in `/tmp/action_records46_standalone_gate.log`. This gate
isolates action records while the root implements the new document pair model.
The shared parser's other families use the same structural convention without
relaxing their settings/action payload validators.

Native validation of all six authored/imported even, odd, and empty captions
completed successfully: each full input opened and saved exactly. Generated
groups came directly from the public record API; the reproducible generator is
`dev/movement_labels46_generate.rs`. Inputs, independently compared Native
saves, and screenshots are retained as `tests/fixtures/movement46_*_label_*`.
