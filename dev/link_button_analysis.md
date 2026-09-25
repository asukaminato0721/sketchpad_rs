# Link Action Button: URL variant

The [Sketchpad 5 Reference Center](https://referencecenter.dynamicgeometry.com/gsp-reference-center/objects_btns_linkprops.htm) describes Link buttons for pages and URLs, including `sketchdoc://` documents next to the current sketch. This pass covers the URL variant. Page destinations and activation on the linked page remain to be analyzed.

## Native records

The default, custom, and even-length URL fixtures were created in native Sketchpad 5 from `tests/fixtures/point.gsp`. A URL Link uses object code 0 and six records: `0x07d0`, `0x0906`, `0x07d5`, `0x07d8`, `0x08a2`, and `0x07d7`. There is no `0x07d2` reference record. The action record has code 6 at offset 12, anchor `(10,5)` at offsets 20–23, a 16-bit URL byte count at offset 24, and the URL starting at offset 26. The initial button label in `0x07d5` repeats the URL, with a 16-bit count at offset 22 and text at offset 24. Both strings are terminated with a zero byte and padded inside their record payload to an even length. `assets/action_link_url_group.bin` preserves the native default 180-byte group.

The three native URL fixtures import as editable with `partial_view=false` and preserve their bytes on unedited save. `LinkUrlButtonGroup` validates and rewrites the observed layout, including the nested action record length and string padding.

## Port behavior and cross-open checks

`tests/replay_link_url_button.txt` creates a web URL button and a `sketchdoc://point.gsp` button. The web URL file opened and saved byte-identically in native Sketchpad 5. The Rust editor exposes URL Link creation from Edit > Action Buttons, draws the button in desktop and SVG output, and uses the system opener for HTTP(S) and file URLs. A `sketchdoc://` link opens a sibling sketch in the editor; a runtime test clicks that generated button and verifies the target document is loaded.

The native `sketchdoc://` click under Wine changed the visible sketch, but the capture did not identify the destination clearly enough to treat it as verified behavior. Page Links, other special URL schemes, and direct native click comparison remain pending.
