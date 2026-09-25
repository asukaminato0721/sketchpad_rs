# Display > Text font, style, and size records

The bundled Reference Center pages `menus_display_text.htm` and `menus_display_textpalette.htm` describe the Display > Text commands: Bold, Italic, Underline, Increase Size, Decrease Size, and font family selection. The Text Palette applies the same categories. The documented Windows shortcuts are Ctrl+B, Ctrl+I, Ctrl+U, Alt+>, and Alt+<. The guide says Shift while choosing a command prevents updating the default style. This probe used the ordinary commands, so default-font records may have changed along with the selected object.

## Original application probes

The source `tests/fixtures/text_object_generated_Hello.gsp` has one free text object, `Hello`. It carries a checked Bold flag in the Text submenu when opened. I selected the text in Sketchpad 5 under Wine, invoked Ctrl+B, Ctrl+I, Ctrl+U, and Alt+> separately, and saved `text_style_bold_off.gsp`, `text_style_italic_on.gsp`, `text_style_underline_on.gsp`, and `text_style_larger.gsp`. I also chose `@Noto Sans CJK JP` from Display > Text and saved `text_style_font_cjk.gsp`. These variants were made from independent copies of the same source. The shared Wine prefix was serialized by `dev/wine_roundtrip.sh` and `/tmp/sketchpad_rs_wine.lock`.

For a selected point with label `A`, Ctrl+B produced `text_style_label_bold_off.gsp`. The label's `0x07d5` payload remained byte-for-byte unchanged. A later two-label probe made only `A` nonbold while `B` stayed bold. The original app added a second `0x04b3` style and set `B`'s `0x07d8` reference to index 1. A port file that instead appended a nonbold style and set only `A`'s reference to index 1 rendered the same two styles and saved byte-identically in Sketchpad 5.

## Observed fields

| Record | Observation |
| --- | --- |
| `0x04b1` | Font-family records are indexed from zero. The family name starts at payload byte 2 and is NUL terminated. A mixed-font document had `@Noto Sans CJK JP` and `Tahoma` in separate records. The initial `u16` can be nonzero for CJK. Bytes following the NUL were not interpreted. |
| `0x04b3` | A 20-byte text-style payload. The initial `u16` indexes `0x04b1`; the `u16` at byte 2 changed from 24 to 36 for Increase Size. `u16` flags at bytes 4, 6, and 8 correspond to Bold, Italic, and Underline; observed values 1 and 2 mean off and on. The source has Bold on. |
| `0x273c` | Font-table records. Free text variants changed the row with ID 1; the point-label probe changed the row with ID 0. Font-family selection expanded the ID-1 row and changed its font index. Their remaining bytes and cross-object scope are unresolved. |
| `0x08fc` | Every free-text style/size/font command changed `<VL<T5xHello>>` into `<T5xHello>`, reduced the inner size 34→30, and reduced the payload length 36→32. This is a structural change to the rich-text run. |
| `0x0898` | The right and bottom edges changed with the selected font and size. For example, the source rectangle `(91,81,149,104)` became `(91,81,193,128)` for Increase Size. The text anchor stayed `(91,81)`. |

The original application also changed preference bytes and added a selected-object record (`0x2328`) during its saves. Some trailing font-table bytes varied between runs; their role is unknown.

## Bounded implementation and evidence

`src/text_style_records.rs` decodes the known `0x04b1`/`0x04b3` fields with bounds checks and preserves style variants from original-app records. Its older standalone fixture writer accepts **only the exact one-object source fixture**. It replaces the observed font/style/rich-text/rectangle records with an original-app variant and preserves all other source records. It rejects any other source file, including one with additional text or labels; this prevents applying a document-level font change whose scope is not yet understood. Three standalone tests pass:

```sh
rustc --edition=2024 --test src/text_style_records.rs -o /tmp/sketchpad_text_style_tests
/tmp/sketchpad_text_style_tests
rustc --edition=2024 -Awarnings dev/text_style_probe.rs -o /tmp/sketchpad_text_style_probe
/tmp/sketchpad_text_style_probe tests/fixtures/text_object_generated_Hello.gsp /tmp/styled.gsp larger
```

The Rust-generated CJK-font and larger-size files both opened visibly and saved in the supplied Sketchpad 5. Screenshots are `/tmp/sketchpad_text_style_rust_cjk_before_save.png` and `/tmp/sketchpad_text_style_rust_larger_before_save.png`. Their original-app saves are `text_style_rust_cjk_saved_by_wine.gsp` and `text_style_rust_larger_saved_by_wine.gsp`. The shared document parser now accepts literal styled `0x08fc` runs and validates their `0x07d8` style index. Native two-caption probes established that each caption can reference a different `0x04b3` record. The port preserves those references while editing or moving text; two generated edits opened and saved byte-identically in Sketchpad 5. A byte probe also showed that changing the indexed style reference alone changes the caption's appearance. The shared writer now adds or reuses `0x04b3` records for Bold, Italic, Underline, Increase Size, and Decrease Size on selected free text and point labels. Native menu probes observed 24→36→48→64 when increasing and 24→18→16→14 when decreasing. Generated larger and smaller captions, plus styled and enlarged point labels, opened visibly and saved byte-identically in Sketchpad 5. SVG uses each object's saved font flags and size. The desktop renderer uses the saved size and approximates bold and underline. Other sizes and mixed-format text still need general writers.

The selected free-text screenshot for the CJK family still displayed Latin `Hello`, so visual glyph differences alone do not establish every font-table field. A Shift-modified menu probe would clarify how the original application updates its text defaults.

## Mixed-font probe

I selected `Hello` in the two-caption file and chose `@Noto Sans CJK JP`. Sketchpad wrote two `0x04b1` family records and two `0x04b3` styles, with each style's initial word pointing to its family. `Hello` and `World` then had distinct `0x07d8` style references. The ID-1 `0x273c` font-table row changed to the CJK family while the other rows kept Tahoma. The app also recomputed both caption bounds. A variant in `tests/fixtures/text_two_font_cjk_first.gsp` instead appended the CJK family and style, changed only `Hello`'s reference and font-table row, and opened and saved byte-identically in Sketchpad. This establishes that the indexed family reference is stable without reproducing Sketchpad's record ordering.

The shared decoder and writer now preserve independent families when styling, copying, and saving existing mixed-font documents. A port file that added Italic to the CJK caption and removed Bold from the Tahoma caption opened and saved byte-identically in Sketchpad.

A second native probe selected only `World` for the CJK font. The same ID-1 `0x273c` row changed, which indicates that row is a text default rather than a per-caption row. A variant with the CJK family and style records, but unchanged `0x273c` rows, opened and saved byte-identically. Zeroing the CJK family's initial word and trailing padding was accepted too. A constructed `Arial` family record also opened and saved byte-identically. The port now appends `0x04b1` family and `0x04b3` style records for printable ASCII family names, changes selected text or label `0x07d8` references, and exposes a Font Family dialog. A two-caption port file using Arial and `@Noto Sans CJK JP`, plus an Arial point label, opened and saved byte-identically in Sketchpad. The two-caption file is `tests/fixtures/text_two_new_families_generated.gsp`. SVG exports use the selected family. The desktop canvas now resolves the family through fontconfig and loads its file into egui, falling back to egui's proportional font if resolution fails. `/tmp/sketchpad_font_preview.png` shows the mixed-font fixture on the native desktop. Installed font availability and non-ASCII family names remain unverified.
