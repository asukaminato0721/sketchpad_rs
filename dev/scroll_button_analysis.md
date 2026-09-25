# Scroll Action Button

The [Sketchpad 5 Reference Center](https://referencecenter.dynamicgeometry.com/gsp-reference-center/objects_btns_scrollprops.htm) says a Scroll button positions its selected point either at the sketch window's upper-left corner or at its center.

## Native records and behavior

Selecting point 1 in a two-point segment baseline and choosing Edit > Action Buttons > Scroll opens a two-choice dialog. The first choice, Upper Left, is the default. `action_scroll_top_left_original.gsp` and `action_scroll_center_original.gsp` preserve the two native results. The code-62 group has six records: `0x07d0`, `0x07d2`, `0x0906`, `0x07d5`, `0x07d8`, and `0x07d7`. The reference record stores one point ID. The `0x0906` payload has action code 4 at offset 12, placement code 0 Upper Left or 1 Center at offset 14, and button anchor `(10,5)` at offsets 20–23. The original label is `滚动` (Scroll). `assets/action_scroll_group.bin` preserves the original 142-byte group.

In a 400-by-250-pixel native canvas, clicking Upper Left moved point 1 from screen `(209,189)` to `(59,69)`, the canvas corner. Clicking Center moved it to `(259,194)`, the canvas center. The segment moved with the viewport. Neither click changed saved records; scrolling is a view action. Both native files import as editable with `partial_view=false` and round-trip byte-identically.

## Port behavior

`ScrollButtonGroup` reads and writes the observed one-point layout while preserving unrecognized bytes. The document model creates an editable Scroll button with either placement, and the editor changes its viewport center when the button is clicked. The SVG and desktop renderers include the button. `tests/replay_scroll_button.txt` generates an Upper Left button and a second Center button. One- and two-button generated files import as editable and round-trip locally. Both opened and saved byte-identically in Sketchpad 5. Clicking the port-generated Upper Left button there placed its point at the canvas corner; clicking the generated Center button placed its point at `(259,194)`, the native canvas center. Neither click changed saved bytes. The Rust desktop click likewise moved the target to its canvas corner and panned the button out of view, matching the native behavior.

The editor's SVG/PDF export carries the same button offset as its scrolled canvas. Broader point and document layouts remain to be verified.
