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

## Native integration and cross-open verification

The document model now decodes code-62 buttons as a target dependency, Hide or
Show action, label, and screen anchor. It preserves the original six-record
group, updates source groups in place, and writes new groups from the original
template. Edit > Action Buttons > Hide/Show creates a button for the selected
objects. The same viewport rectangle is used for drawing and click hit testing;
clicking toggles target visibility and the button's next action. SVG export
also includes the button. `tests/replay_action.txt` covers creation, activation,
save/reopen, and undo.

`tests/fixtures/action_port_generated.gsp` is the native writer's new English
Hide Segment button. The original Sketchpad 5 opened it and a click at
`(105,85)` hid segment object 3. Its saved file,
`tests/fixtures/action_port_wine_clicked.gsp`, has 88 records, a hidden segment,
and a Show button with the original application's Chinese label. The port
reopens it with `partial_view=false`; all action fixtures reencode byte for
byte. The native window was launched in an isolated Xvfb display, but that
environment did not expose a visible window for manual screenshot inspection.

## Two-target Hide/Show buttons

Sketchpad 5 created `tests/fixtures/action_hide_show_two_points_original.gsp` from two selected endpoints. Its `0x07d2` record contains count `2` followed by target IDs `1` and `2`. Clicking the button hid both points while their segment remained visible; `action_hide_show_two_points_clicked.gsp` contains the resulting Show action. Both native files import as editable and round-trip exactly.

The Rust editor now uses all selected supported objects when creating a Hide/Show button. The desktop two-point selection and File > Save flow produced the same bytes as `tests/replay_hide_show_multi.txt`, which generates and activates a two-point button. The generated default and clicked files opened and saved byte-identically in Sketchpad 5. Clicking the generated default in Sketchpad 5 hid both points; the saved `action_hide_show_two_points_port_clicked_by_wine.gsp` imports with the native `显示点` label. Copying a native two-point button with its targets also generates a valid file that Sketchpad 5 resaves byte-identically. Odd-length native point labels use `assets/action_hide_show_two_points_group.bin` as their template.

## Scope and remaining work

The parser intentionally recognizes only code-62 Hide/Show groups with the
observed six-record layout and action subtype `(0 or 1, 7)`. Other Action
Button commands, broader target combinations, alternate label payloads, and insertion
into arbitrary pages need their own probes. The writer currently refuses
button insertion in a multipage document. A saved button's label may change
only when its encoded byte length stays fixed, so opaque trailing bytes and
following object offsets remain valid.

Standalone tests:

```sh
rustc --edition=2024 --test src/action_records.rs -o /tmp/sketchpad_action_records_tests
/tmp/sketchpad_action_records_tests
```
