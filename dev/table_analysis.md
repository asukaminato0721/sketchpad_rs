# Number, calculation, and table records

## Scope and source

These layouts came from Sketchpad 5 run under Wine with a shared-prefix lock. `dev/table_wine_probe.sh` creates original application fixtures; `dev/table_record_probe.rs` writes bounded variants from Rust. The original files are in `tests/fixtures/table_*`. No layout was inferred solely from the sibling project. Its decimal-tail decoder in `../gsp-rs/src/runtime/extract/decode.rs` was used as a second check.

The port currently writes the verified default `t[1] = 1.00` parameter, the literal `2 + 3 = 5.00` calculation, and a one-column snapshot table for that parameter. New Parameter and Calculate are exposed only on blank one-page sketches; Tabulate requires that exact parameter as object 1. Arbitrary parameter values, expressions, formulas, column combinations, and dynamic table content remain unsupported for structural edits. The strict parsers leave other records in partial view and refuse saves that could lose content.

## Record groups

| Object | Header code | Record sequence | Verified fields |
| --- | ---: | --- | --- |
| Default parameter | 0 | `07d0 0907 07d5 07d8 08a3 07d7` | `0907` anchor at payload bytes 12–15; `07d5` label `t[1]`; `08a3` rectangle `(x,y,x+94,y+32)` |
| Literal calculation | 0 | `07d0 0907 07d8 08a3 07d7` | `0907` anchor at bytes 12–15; `08a3` rectangle `(x,y,x+114,y+26)` |
| One-column table | 91 | `07d0 07d2 090d 07d8 07d7` | `07d2` references parameter 1; `090d` anchor at bytes 12–15 and row count plus two at byte 16 |

The original `090d` table layout begins with its own `090d` tag and a 32-bit inner length at byte 4. A one-row table has 64 payload bytes, two rows 80 bytes. Each additional cached `1.00` row adds `0<H<T4x1.00>>` followed by alignment padding. The first row's fixed header is retained from the original fixture. `src/table_records.rs` checks the full canonical layout and caps rows at 256.

The original application changes opaque bytes 22–23 and 30–31 of the parameter `0907` payload during some saves; these bytes are carried through unchanged. Dragging the parameter shifts its `0907` anchor and all four `08a3` coordinates by the same delta. Dragging a table changes only its `090d` anchor. The Rust writers reject coordinate overflow before mutation.

The original UI's Add Table Data and Remove Table Data operations visibly changed rows but Ctrl+S alone did not update the file in these probes. Dragging the table then saving persisted the changed row count. The fixtures `table_parameter_two_rows.gsp` and `table_parameter_row_removed.gsp` capture the persisted layouts. The port marks each row edit dirty immediately.

## Verification

- `rustc --edition=2024 --test src/table_records.rs -o /tmp/sketchpad_table_records_tests && /tmp/sketchpad_table_records_tests` checks bounded encoding, original drag positions, row addition, and rejection of unsupported parameter values.
- `cargo test -q` passed 123 tests after integration. `cargo run --quiet --bin gsp_probe -- replay tests/replay_table.txt` and `... tests/replay_calculation.txt` passed save/reopen checks.
- Rust-generated standalone parameter, calculation, and table files opened visibly in the original Sketchpad 5 and saved. The integrated replay files also opened and saved there. Captures: `/tmp/sketchpad_number_integrated_before_save.png` and `/tmp/sketchpad_calculation_integrated_before_save.png`. Their original-saved copies are `table_integrated_saved_by_wine.gsp` and `table_calculation_integrated_saved_by_wine.gsp`.
- `gsp_probe verify` reopened those original-saved integrated files as two and one objects respectively, both with `partial=false`. The original application's row-removed fixture also reopened with `partial=false` after accepting its observed opaque padding changes.

The native UI draws and hit-tests the bounded number objects using page anchors, and the SVG path renders the same text and rows. Original-app cross-open is the compatibility proof for generated GSP files; native canvas interaction was checked through code and replay, not by an automated GUI assertion.
