# Native parameter name materialization and caption caches

These independent controls use the original unlabeled Scalar parameter document.
They modify no production code. The unchanged central Wine helper opens and
saves each native output under its shared lock. Each retained control has a
positive capture and an independent successful reopen/save compared exactly
with the first output.

The current maximized MDI canvas starts at screen y=51. The earlier table script
assumed y=68: its click at80,122 selects source3 in the current window. The target
controls instead clear selection at450,150, select source2 at80,104 and use
Number346,40 then Tabulate355,113. The output table explicitly shows the intended
source name and value. This coordinate correction is not a format rejection.

| Control | Source2 name/value | Source2 bbox size | Cursor | Objects |
| --- | --- | --- | --- | --- |
| Original | absent /0 |8×23 |4 |14 |
| Tabulate source2 after setting2.5 |t[4] /2.5 |84×32 |5 |15 |
| Change only counter4→12, then Tabulate source2 |t[12] /0 |75×32 |13 |15 |
| Edit source3 in Native1.00→2.50 and ordinary save |t[4] /0 |68×32 |5 |14 |
| Enter source3 value editor, Escape, ordinary save |t[4] /0 |68×32 |5 |14 |
| Tabulate source2 after setting−2.5 |t[4] /−2.5 |94×32 |5 |15 |

All source2 rectangles keep their origin(10,38). Original source1 and source3
rectangles remain(10,5)-(113,37) and(10,71)-(104,103) respectively. The source3
actual edit keeps its original precision2 and94×32 rectangle while changing
its value to2.50. Setting source2 to2.5 through the existing value setter changes
precision0→1 and its still-unnamed input cache to18×23; the native table operation
then restores the named84×32 rectangle. These are distinct operations.

The second subscript digit adds7 pixels: t[12] Scalar0 is75 pixels wide, compared
with the retained t[4] Scalar0 width68. The minus glyph adds10 pixels: named
−2.5 is94 pixels wide versus84 for named2.5. Together with the earlier controls,
this supports the bounded Tahoma18Normal cache rule: base58, plus7 for each
additional subscript digit,10 per value digit,6 for the decimal point and10 for
the minus glyph. The prior Degree suffix adds9 and centimeter suffix42. This is
not a claim for arbitrary fonts, names or mathematical token layouts.

Label materialization is not exclusive to Tabulate. The source3 value edit and
even entering then cancelling its value editor cause ordinary serialization to
write the previously absent source2 label and advance cursor4→5 without a table.
A clean save after clicking empty canvas preserves the missing label and cursor4.
These controls establish ordinary parameter-editor/save behavior; they do not
assert that every possible document mutation materializes names.

The source2 `07d0` **payload** word at offset18 (whole group offset26) changes
0x2039→0x5c51 when its name is materialized. That is a different location from
whole-group offset18. Existing source1 retains0x2039 and source3 retains0x5c51.
The `0907` payload word at offset22 is not a fixed naming value: it remains0
in the positive2.5 and counter12 tables, becomes34 across all three parameters
in the actual source3 edit, and becomes43 across all three in the negative table.
Earlier native tables contain34 or103. No semantic interpretation or normalization
of this opaque field is inferred from the naming operation.

The negative table row encodes U+2013 EN DASH in UTF-8: `0<H<T6x–2.5>>`.
The positive row is `0<H<T3x2.5>>`. The rich-text run length counts UTF-8 bytes;
the negative `090d` is68 bytes, versus64 for the positive control. ASCII hyphen
and U+2212 are not byte-equivalent to this native table representation.

Retained first outputs and positive captures:

- [Scalar2.5 table](../tests/fixtures/numeric_parameter_table_unnamed_scalar25_native_created.gsp),3786B; [capture](../tests/fixtures/numeric_parameter_table_unnamed_scalar25_native_created.png); [log](../tests/fixtures/numeric_parameter_table_unnamed_scalar25_native_created.log).
- [Counter12 table](../tests/fixtures/numeric_parameter_table_unnamed_scalar_counter12_native_created.gsp),3782B; [capture](../tests/fixtures/numeric_parameter_table_unnamed_scalar_counter12_native_created.png); [log](../tests/fixtures/numeric_parameter_table_unnamed_scalar_counter12_native_created.log).
- [Source3 dirty save](../tests/fixtures/numeric_parameter_unnamed_source3_dirty_saved_native_verified.gsp),3604B; [capture](../tests/fixtures/numeric_parameter_unnamed_source3_dirty_saved_native_verified.png); [log](../tests/fixtures/numeric_parameter_unnamed_source3_dirty_saved_native_verified.log).
- [Source3 editor cancel/save](../tests/fixtures/numeric_parameter_unnamed_source3_editor_cancel_saved_native_verified.gsp),3612B; [capture](../tests/fixtures/numeric_parameter_unnamed_source3_editor_cancel_saved_native_verified.png); [log](../tests/fixtures/numeric_parameter_unnamed_source3_editor_cancel_saved_native_verified.log).

- [Scalar−2.5 table](../tests/fixtures/numeric_parameter_table_unnamed_scalar_negative25_native_created.gsp),3792B; [capture](../tests/fixtures/numeric_parameter_table_unnamed_scalar_negative25_native_created.png); [log](../tests/fixtures/numeric_parameter_table_unnamed_scalar_negative25_native_created.log).

All five controls finished with native helper exit0 and independent exact
reopen/save comparisons. No native job remains active from this audit.
