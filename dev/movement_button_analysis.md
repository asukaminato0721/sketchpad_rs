# Movement Action Button

The [Sketchpad 5 Reference Center](https://referencecenter.dynamicgeometry.com/gsp-reference-center/objects_btns_moveprops.htm) describes a Movement button as moving the first object in each selected point or value pair toward the second. Its properties choose Slow, Medium, Fast, or Instant speed, and either follow a moving destination or retain the destination's position at the initial click.

## Native record probes

A two-point baseline with a segment was opened in the supplied Chinese Sketchpad 5 build. Selecting point 1 and then point 2, then choosing Edit > Action Buttons > Movement produced `action_movement_default_original.gsp`. The code-62 button group contains six records: `0x07d0`, `0x07d2`, `0x0906`, `0x07d5`, `0x07d8`, and `0x07d7`. The references are ordered `[1, 2]` and mean source then destination. The 24-byte `0x0906` payload has nested action code 3 at offset 12, a speed code at offset 14, the destination mode at offset 16, and the screen anchor `(10,5)` at offsets 20–23. The original label is `移动点` (Move Point). `assets/action_movement_group.bin` is the original 148-byte group.

The four speed settings correspond to codes 0 Slow, 1 Medium, 2 Fast, and 3 Instant. Follow moving destination stores 0 at offset 16; Move toward initial destination stores 1. The native fixtures `action_movement_{default,slow,fast,instant,initial}_original.gsp` each change only the corresponding setting from the default group. All five import with `partial_view=false` and round-trip byte-identically.

## Port behavior and compatibility

`MovementButtonGroup` decodes and writes the observed one-pair layout while preserving unknown record bytes. The document model imports it as an editable button. A new Movement command accepts an ordered pair whose source is a free point and whose destination is a point, then opens a settings dialog. A click starts a movement; clicking the active button again stops it. Instant moves directly to the destination. The other speeds interpolate and honor the destination mode. Current speed rates are approximate; native timing has not yet been measured.

`tests/replay_movement_button.txt` generates a default button and a second Instant/initial-destination button. Both generated files import as editable and round-trip. The default file opened and saved byte-identically in Sketchpad 5. Clicking that generated button in Sketchpad 5 moved point 1 from `(150,120)` onto point 2 at `(270,120)`; `action_movement_port_clicked_by_wine.gsp` preserves that result and reopens without a partial view. The two-button file also cross-opened byte-identically.

Opening the generated default in the Rust desktop and clicking its button likewise moved point 1 to `(270,120)`. The saved `action_movement_port_desktop_clicked.gsp` then opened and saved byte-identically in Sketchpad 5.

## Properties editing

The common Properties window exposes the Native speed, follow-moving versus
initial-destination policy, and Label controls for an existing Movement button.
One public document command applies all three fields transactionally. Movement
now shares the post-order variable-size action writer with Hide/Show and
Animation, including guarded rebuilding of the known zero label suffix when a
caption changes byte parity.

A public-API edit changed the port default to Instant/initial destination and
renamed it from odd-length `Move to Point` to even-length
`Move Once Verified`. Native Properties showed combo index 3, matching the
Native Instant fixture, with initial-destination control 1267 selected. Native
saved all 2,062 bytes unchanged with SHA-256
`31928aeb5e89063ee470c1f1d140aa9f9196912aa620398b80afc7d6fb2a85d1`.

## Multi-page insertion

Sketchpad 5 created `two_page_movement_native.gsp` on page 2 with global source and destination IDs `[2, 3]`, encoded as local references `[1, 2]`. `tests/replay_multi_page_movement.txt` generates the corresponding English button and a second document with a Medium/Follow button on page 2 and an Instant/initial-destination button on page 1. Both generated files opened and saved byte-identically in Sketchpad 5. Clicking the generated page-2 button there moved its source onto the destination and left page 1 unchanged; `two_page_movement_port_clicked_by_wine.gsp` imports and round-trips. The Rust editor also runs and saves that multi-page movement.

Parameter/value pairs, multiple point pairs, detailed native movement timing, and broader document layouts remain open.
