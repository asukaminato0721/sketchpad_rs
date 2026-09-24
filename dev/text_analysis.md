# Sketchpad text and label records

## Evidence

I inspected 397 `0x07d5` label payloads and 49 `0x08fc` rich-text payloads across the sibling fixture corpus and this repository's fixtures. Every label payload was at least 24 bytes and its byte-counted name fit inside the record. Every rich-text payload had a valid 18-byte prefix and a markup byte count that fit inside the record.

`text_point_label.gsp`, `text_ray_label_hide.gsp`, and `text_on_point.gsp` are copies of the sibling project's original Sketchpad fixtures. `text_point_label_dragged.gsp` was edited and saved in the supplied Sketchpad 5 through Wine. Dragging the visible `A` label above its point changed only the label's offset among its object records, from `(8, 5)` to `(8, -34)`. The label remained attached to its point. The application also updated unrelated document preference bytes, which this module preserves untouched.

I generated a one-point document with `gsp_probe demo`, inserted the module's canonical point-label records with `dev/text_label_probe.rs`, and opened it through the Wine harness. The harness verified the Sketchpad window title and saved an 1880-byte file. The shared screenshot path was overwritten by another concurrent probe, so that run alone is not visual confirmation of the generated label.

## Observed layouts

| Record | Payload fields | Notes |
|---|---|---|
| `0x07d5` | `u16` at 0; visibility `u16` at 2; offset `i16 x/y` at 6/8; byte count `u16` at 22; name bytes at 24 | Object-local label. Bytes 4..5 and 10..21 vary by object; preserve them. Bytes after the name can be nonzero; preserve them as well. No object ordinal occurs in the point-label variant. |
| `0x07d8` | Two-byte auxiliary payload | For the labeled point fixture it is `00 00`, placed after `0x07d5` and before `0x0899` coordinates. Other object types use different values. |
| `0x08fc` | Magic `0x08fc` at 0; anchor `i16 x/y` at 12/14; markup byte count at 16; markup bytes at 18 | Used by text objects and other rich labels. Markup may contain dynamic references; preserve raw bytes until those references are understood. |
| `0x0898` | Four `i16` coordinates | Bounding rectangle corners of the plain text fixture. Move both corners when moving its anchor. |

The `0x07d5` name count is a **byte count**, not a character count. The corpus includes UTF-8 Chinese text and names such as `t[1]`. The source fixture's `A` name has count 1, one `A` byte, and one trailing zero byte. Several Example Tools labels have a nonzero second trailing byte. `LabelAux` carries that suffix forward and refuses a rename that would have to discard a nonzero byte when name parity changes.

## Integration

Add `pub mod text_records;` in `src/lib.rs`. The module has no dependencies and its six focused tests run with:

```sh
rustc --edition=2024 --test src/text_records.rs -o /tmp/sketchpad_text_records_tests
/tmp/sketchpad_text_records_tests
```

For an existing object, parse its `0x07d5` payload with `LabelAux::parse`, expose its name, visibility, and offset in the model, and preserve its full raw payload unless the user changes one of those fields. A generated point can use `LabelAux::new_point` plus `encode_point_label_records` inserted immediately after its `0x07d0` header. A labeled segment has a different record order: its reference record precedes `0x07d5`. Keep `0x07d8` values from source groups unless creating the canonical point-label variant.

For a plain text object, `RichTextPayload` exposes the anchor and raw markup, and `TextBox` exposes its screen rectangle. A text object in `text_on_point.gsp` has object code 0 but no `0x0899` coordinate record; it contains `0x08fc`, `0x07d8`, and `0x0898`. The core parser should distinguish it from a free point. Dynamic rich-text markup can contain group references, so structural edits should remain blocked for those objects until reference decoding and remapping are implemented.

To repeat the generated label probe:

```sh
cargo run --offline --bin gsp_probe -- replay tests/text_label_replay.txt
rustc --edition=2024 -Awarnings dev/text_label_probe.rs -o /tmp/sketchpad_text_label_probe
/tmp/sketchpad_text_label_probe /tmp/sketchpad_text_generated_unlabeled.gsp /tmp/sketchpad_text_generated_labeled.gsp P
./dev/wine_roundtrip.sh /tmp/sketchpad_text_generated_labeled.gsp
```
