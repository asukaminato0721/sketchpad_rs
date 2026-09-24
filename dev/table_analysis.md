# Number, calculation, and table records

## Scope and source

These layouts came from Sketchpad 5 run under Wine with a shared-prefix lock. `dev/table_wine_probe.sh` creates original application fixtures; `dev/table_record_probe.rs` writes bounded variants from Rust. The original files are in `tests/fixtures/table_*`. No layout was inferred solely from the sibling project. Its decimal-tail decoder in `../gsp-rs/src/runtime/extract/decode.rs` was used as a second check.

The port writes default `t[1]` and `t[2]` parameters and multiple named ASCII parameters in supported single-page sketches, with editable values from −99.99 to 99.99 at up to two-decimal precision. Named parameters increase their display precision when a value needs a second decimal place. It also writes the literal `2 + 3 = 5.00` calculation, the original one-column t₁ table, named parameter tables with snapshot rows, and a verified two-column t₂/t₁ table with one or two rows. The two-column table currently requires t₁ = 1.00 and t₂ from 0.00 to 9.99. Unicode names, values beyond this range, formulas, and other column combinations remain unsupported for structural edits. The strict parsers leave other records in partial view and refuse saves that could lose content.

## Record groups

| Object | Header code | Record sequence | Verified fields |
| --- | ---: | --- | --- |
| Default parameter | 0 | `07d0 0907 07d5 07d8 08a3 07d7` | `0907` anchor at payload bytes 12–15; `07d5` label `t[1]`; `08a3` rectangle `(x,y,x+94,y+32)` |
| Literal calculation | 0 | `07d0 0907 07d8 08a3 07d7` | `0907` anchor at bytes 12–15; `08a3` rectangle `(x,y,x+114,y+26)` |
| One-column table | 91 | `07d0 07d2 090d 07d8 07d7` | `07d2` references parameter 1; `090d` anchor at bytes 12–15 and row count plus two at byte 16 |

The original `090d` table layout begins with its own `090d` tag and a 32-bit inner length at byte 4. A one-row table has 64 payload bytes, two rows 80 bytes. Each additional cached `1.00` row adds `0<H<T4x1.00>>` followed by alignment padding. The first row's fixed header is retained from the original fixture. `src/table_records.rs` checks the full canonical layout and caps rows at 256.

The default parameter's `0907` payload ends with two-byte decimal words. Single-digit values use one word; `3.25` uses `[3, 10, 2, 5]`, where 10 marks the decimal point; `−2.5` uses `[4097, 2, 10, 5]`, where 4097 marks the sign. `12.34` uses `[1, 2, 10, 3, 4]`. The inner length and digit count grow with these words. The port writes values within ±99.99 with up to two decimal places, adjusts the parameter box for an extra sign or integer digit, and retains source-file opaque bytes. Original fixtures for fractional, negative, and two-digit values parse without partial view; generated examples opened and saved byte-identically in Sketchpad 5.

The original `speed = 2.5`, `speed = 3.25`, and `test = 2.5` fixtures show that a named parameter uses a type-7 `07d5` label: 22 fixed bytes, a two-byte ASCII name length, name bytes, a null terminator, and zero padding to an even record length. Its `0907` precision is 1 with a 0.1 step for `2.5`; two decimals use precision 2 and a 0.01 step. The parameter box grows by 10 units for that extra decimal. Generated `speed` files with both precisions and an even-length `test` name opened and saved byte-identically in Sketchpad 5. A renamed default `t[1]` with label `a` uses the default parameter's numeric header instead and also parses without partial view.

A one-row table for `speed = 2.5` has a 60-byte `090d` layout. The text after its 28-byte header is `000<SS2<T5xspeed>>0<H<T3x2.5>>` plus zero padding to a four-byte boundary. The `07d2` record carries the source object ID; the original `test = 0.25` table points to object 5. The original app's Add Table Data command appends another `0<H<T3x2.5>>`, increases the `090d` length by 12, and increments the stored row count. The final row displays the live parameter value even when its cached text differs. When adding a row, Sketchpad replaces that last cache with the live value before appending the next live row. The original `2.5` cache with a live `3.25` parameter thus became two `3.25` rows after Add Table Data. Rust reconstructs these original groups byte for byte and preserves the original full files; its generated two- and three-row files cross-opened in Sketchpad 5.

The two-row table in `plot_points_table_two_rows_original.gsp` differs from the canonical table only at the penultimate `090d` padding byte, which contains `t` instead of zero. The parser accepts this exact observed variant and retains its original bytes on save.

The original application changes opaque bytes 22–23 and 30–31 of the parameter `0907` payload during some saves; these bytes are carried through unchanged. Dragging the parameter shifts its `0907` anchor and all four `08a3` coordinates by the same delta. Dragging a table changes only its `090d` anchor. The Rust writers reject coordinate overflow before mutation.

The original UI's Add Table Data and Remove Table Data operations visibly changed rows but Ctrl+S alone did not update the file in these probes. Dragging the table then saving persisted the changed row count. The fixtures `table_parameter_two_rows.gsp` and `table_parameter_row_removed.gsp` capture the persisted layouts. The port marks each row edit dirty immediately.

## Verification

- `rustc --edition=2024 --test src/table_records.rs -o /tmp/sketchpad_table_records_tests && /tmp/sketchpad_table_records_tests` checks bounded encoding, original drag positions, row addition, and rejection of unsupported parameter values.
- `cargo test -q` passed 123 tests after integration. `cargo run --quiet --bin gsp_probe -- replay tests/replay_table.txt` and `... tests/replay_calculation.txt` passed save/reopen checks.
- Rust-generated standalone parameter, calculation, and table files opened visibly in the original Sketchpad 5 and saved. The integrated replay files also opened and saved there. Captures: `/tmp/sketchpad_number_integrated_before_save.png` and `/tmp/sketchpad_calculation_integrated_before_save.png`. Their original-saved copies are `table_integrated_saved_by_wine.gsp` and `table_calculation_integrated_saved_by_wine.gsp`.
- `gsp_probe verify` reopened those original-saved integrated files as two and one objects respectively, both with `partial=false`. The original application's row-removed fixture also reopened with `partial=false` after accepting its observed opaque padding changes.

The native UI draws and hit-tests the bounded number objects using page anchors, and the SVG path renders the same text and rows. Original-app cross-open is the compatibility proof for generated GSP files; native canvas interaction was checked through code and replay, not by an automated GUI assertion.
