# Display > Text font, style, and size records

The bundled Reference Center pages `menus_display_text.htm` and `menus_display_textpalette.htm` describe the Display > Text commands: Bold, Italic, Underline, Increase Size, Decrease Size, and font family selection. The Text Palette applies the same categories. The documented Windows shortcuts are Ctrl+B, Ctrl+I, Ctrl+U, Alt+>, and Alt+<. The guide says Shift while choosing a command prevents updating the default style. This probe used the ordinary commands, so default-font records may have changed along with the selected object.

## Original application probes

The source `tests/fixtures/text_object_generated_Hello.gsp` has one free text object, `Hello`. It carries a checked Bold flag in the Text submenu when opened. I selected the text in Sketchpad 5 under Wine, invoked Ctrl+B, Ctrl+I, Ctrl+U, and Alt+> separately, and saved `text_style_bold_off.gsp`, `text_style_italic_on.gsp`, `text_style_underline_on.gsp`, and `text_style_larger.gsp`. I also chose `@Noto Sans CJK JP` from Display > Text and saved `text_style_font_cjk.gsp`. These variants were made from independent copies of the same source. The shared Wine prefix was serialized by `dev/wine_roundtrip.sh` and `/tmp/sketchpad_rs_wine.lock`.

For a selected point with label `A`, Ctrl+B produced `text_style_label_bold_off.gsp`. The label's `0x07d5` payload remained byte-for-byte unchanged. This shows that its font state is outside the name/offset record; it does not prove whether changing one point label also changes other labels in a multi-object sketch.

## Observed fields

| Record | Observation |
| --- | --- |
| `0x04b1` | Active font family starts at payload byte 2 and is NUL terminated. The font-menu choice changed `Tahoma` to `@Noto Sans CJK JP` and grew this record from 12 to 22 bytes. Bytes following the NUL were not interpreted. |
| `0x04b3` | A 20-byte active text-style payload. The `u16` at byte 2 changed from 24 to 36 for Increase Size. `u16` flags at bytes 4, 6, and 8 correspond to Bold, Italic, and Underline; observed values 1 and 2 mean off and on. The source has Bold on. |
| `0x273c` | Font-table records. Free text variants changed the row with ID 1; the point-label probe changed the row with ID 0. Font-family selection expanded the ID-1 row and changed its font index. Their remaining bytes and cross-object scope are unresolved. |
| `0x08fc` | Every free-text style/size/font command changed `<VL<T5xHello>>` into `<T5xHello>`, reduced the inner size 34→30, and reduced the payload length 36→32. This is a structural change to the rich-text run. |
| `0x0898` | The right and bottom edges changed with the selected font and size. For example, the source rectangle `(91,81,149,104)` became `(91,81,193,128)` for Increase Size. The text anchor stayed `(91,81)`. |

The original application also changed preference bytes and added a selected-object record (`0x2328`) during its saves. Some trailing font-table bytes varied between runs; their role is unknown.

## Bounded implementation and evidence

`src/text_style_records.rs` decodes the known `0x04b1`/`0x04b3` fields with bounds checks and preserves style variants from original-app records. Its writer accepts **only the exact one-object source fixture**. It replaces the observed font/style/rich-text/rectangle records with an original-app variant and preserves all other source records. It rejects any other source file, including one with additional text or labels; this prevents applying a document-level font change whose scope is not yet understood. Three standalone tests pass:

```sh
rustc --edition=2024 --test src/text_style_records.rs -o /tmp/sketchpad_text_style_tests
/tmp/sketchpad_text_style_tests
rustc --edition=2024 -Awarnings dev/text_style_probe.rs -o /tmp/sketchpad_text_style_probe
/tmp/sketchpad_text_style_probe tests/fixtures/text_object_generated_Hello.gsp /tmp/styled.gsp larger
```

The Rust-generated CJK-font and larger-size files both opened visibly and saved in the supplied Sketchpad 5. Screenshots are `/tmp/sketchpad_text_style_rust_cjk_before_save.png` and `/tmp/sketchpad_text_style_rust_larger_before_save.png`. Their original-app saves are `text_style_rust_cjk_saved_by_wine.gsp` and `text_style_rust_larger_saved_by_wine.gsp`. `gsp_probe verify` reopened both byte-exactly with one object and `partial=true`. Partial view is expected: the shared document parser intentionally rejects styled `0x08fc` markup and therefore blocks structural edits of those files. The independent module does not weaken that guard.

The selected free-text screenshot for the CJK family still displayed Latin `Hello`, so visual glyph differences alone do not establish every font-table field. A mixed-font, multi-object document and a Shift-modified menu probe are needed before general text formatting can safely enter the editor model.
