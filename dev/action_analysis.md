# Hide/Show Action Button record analysis

## Original application probe

The tested command is **Edit > Action Buttons > Hide/Show** in the supplied
Chinese Sketchpad 5 build. It is the first item in the Action Buttons submenu.
The earlier task description placed it under Display; the live menu shows it
under Edit.

`tests/replay_action_baseline.txt` creates two points and their segment. The
Wine probe `dev/wine_action_probe.sh create` selected the segment and chose the
command. It saved `tests/fixtures/action_hide_show.gsp`, with one new object
(code `62`) and a visible button at raw screen anchor `(10, 5)`. The button's
UTF-8 label is `隐藏线段` (Hide Segment). Its only target is segment object `3`.

The new object group starts at file offset `0x7b4` in the original-created
fixture and occupies 148 bytes. Its records are:

| Record | Payload | Observed role |
| --- | ---: | --- |
| `0x07d0` | 28 bytes | Object code `62`; header byte `+2` controls button visibility. |
| `0x07d2` | 8 bytes | Count `1`, then target object ordinal `3`. |
| `0x0906` | 24 bytes | Action payload; little-endian `(u16,u16)` at offsets `12,14` is `(0,7)` for Hide and `(1,7)` for Show. Signed 16-bit screen anchor at offsets `20,22`. |
| `0x07d5` | 38 bytes | Button label; name byte length is a `u16` at offset `22`, followed by UTF-8 bytes and an opaque suffix. |
| `0x07d8` | 2 bytes | Auxiliary payload, preserved unchanged. |
| `0x07d7` | 0 bytes | Object-group terminator. |

`assets/action_hide_show_group.bin` is the exact original 148-byte group.
`src/action_records.rs` parses and encodes this group without changing unknown
bytes, and exposes the known target, action, anchor, label, and button-hidden
fields. A label edit preserves the opaque suffix and refuses a change in byte
parity. The group writer is standalone: a document writer must also update the
page object count and insert the group before the page trailer.

## Behavior after click and save

`dev/wine_action_probe.sh click` reopened the original-created fixture and
clicked the button at screen `(105,85)`. The segment disappeared while both
endpoints remained visible. The saved file is
`tests/fixtures/action_hide_show_clicked.gsp`: segment header byte `+2`
changed `0` to `1`; action payload byte `+12` changed `0` to `1`; label text
changed from `隐藏线段` to `显示线段` (Show Segment). The original app also rewrote
document metadata and removed a `0x2328` record, so a click is not an isolated
three-field file patch.

Opening that saved file and clicking again restored the segment and Hide label.
`tests/fixtures/action_hide_show_reclicked.gsp` contains the restored object
and action states. The three action groups round-trip byte-for-byte through
`HideShowButtonGroup`.

`dev/action_record_probe.rs` appended a Rust-generated English button to the
same three-object baseline. Sketchpad 5 opened
`tests/fixtures/action_generated_english.gsp`, displayed `Hide Segment`, and
saved all 86 records without changing their payloads. Clicking the button
hid the segment; the saved result is
`tests/fixtures/action_generated_english_clicked.gsp`. The app changed the
button state to Show and replaced the English label with its Chinese localized
Show label. This means the observed original UI does not preserve an English
button label through a toggle, even though the generated English label is
accepted when opened.

## Scope and remaining work

The parser intentionally recognizes only code-62 Hide/Show groups with the
observed six-record layout and action subtype `(0 or 1, 7)`. Other Action
Button commands, multiple targets, alternate label payloads, and insertion
into arbitrary pages need their own probes. The shared document model and UI
have not yet been connected to `src/action_records.rs`.

Standalone tests:

```sh
rustc --edition=2024 --test src/action_records.rs -o /tmp/sketchpad_action_records_tests
/tmp/sketchpad_action_records_tests
```
