# Link Action Button

The [Sketchpad 5 Reference Center](https://referencecenter.dynamicgeometry.com/gsp-reference-center/objects_btns_linkprops.htm) describes Link buttons for pages and URLs, including `sketchdoc://` documents next to the current sketch and optional button activation on a destination page.

## Native records

The default, custom, and even-length URL fixtures were created in native Sketchpad 5 from `tests/fixtures/point.gsp`. A URL Link uses object code 0 and six records: `0x07d0`, `0x0906`, `0x07d5`, `0x07d8`, `0x08a2`, and `0x07d7`. There is no `0x07d2` reference record. The action record has code 6 at offset 12, anchor `(10,5)` at offsets 20–23, a 16-bit URL byte count at offset 24, and the URL starting at offset 26. The initial button label in `0x07d5` repeats the URL, with a 16-bit count at offset 22 and text at offset 24. Both strings are terminated with a zero byte and padded inside their record payload to an even length. `assets/action_link_url_group.bin` preserves the native default 180-byte group.

The three native URL fixtures import as editable with `partial_view=false` and preserve their bytes on unedited save. `LinkUrlButtonGroup` validates and rewrites the observed layout, including the nested action record length and string padding.

## Port behavior and cross-open checks

`tests/replay_link_url_button.txt` creates a web URL button and a `sketchdoc://point.gsp` button. The web URL file opened and saved byte-identically in native Sketchpad 5. The Rust editor exposes URL Link creation from Edit > Action Buttons, draws the button in desktop and SVG output, and uses the system opener for HTTP(S) and file URLs. A `sketchdoc://` link opens a sibling sketch in the editor; a runtime test clicks that generated button and verifies the target document is loaded.

The native `sketchdoc://` click under Wine changed the visible sketch, but the capture did not identify the destination clearly enough to treat it as verified behavior. Other special URL schemes and direct native URL click comparison remain pending.

## Properties editing

The common Properties window exposes Native's Label and Link panels for URL
and Page buttons. Creation and Properties reuse URL/Page mode, URL text,
destination page, and optional destination Hide/Show controls. Two public
document commands apply the complete target form and caption transactionally,
including conversion between wire formats. The shared post-order writer retains
same-kind padding and opaque fields, reconstructs cross-kind groups, maintains
the page object index for conversions, and recalculates destination markers
only when Page Link topology changes. It also enables the
already-supported action Properties editors on existing multi-page documents
when their structural targets and anchors stay fixed.

A same-kind URL edit expanded the URL to `https://example.net/verified` and the
caption to `Open Verified Link`. Native displayed URL mode and the new caption,
then saved all 2,022 bytes unchanged with SHA-256
`09b5d5cbfb8618dd811787917255de3dc9f4c047710fffcf59e69737eeb45696`.
A multi-page URL-to-Page conversion displayed `Go to Verified Page`, Page 1,
and no destination action, then saved all 3,262 bytes unchanged with SHA-256
`05ce8353a6e48152e6c45d4cf40f843cb99683f0b5112554900f4bcbe665da6c`.
The reverse conversion started from a Page Link targeting Hide/Show, cleared
that destination marker, and saved all 3,606 bytes unchanged with SHA-256
`93b5426e9f3535f787b50f3e753a05aa2affa2180d55e61202ec5d548d9f553d`.

## Page destinations

`two_page_blank_native.gsp` has a free point on page 1 and an empty page 2. Native Sketchpad created `action_link_page_default_original.gsp` on page 2, targeting page 1, and `action_link_page_two_original.gsp` on page 1, targeting page 2. Both use object code 0 and the same six record kinds as URL Links. Their `0x0906` payload is 24 bytes with action code 5 at offset 12, a zero-based destination page index at offset 14, and zero at offset 16 when no button on the destination page is chosen. The native label is `链接` (Link). `assets/action_link_page_group.bin` preserves the 142-byte native group. The two files import losslessly, and clicking either in native Sketchpad navigated to the page indicated by the record.

The Rust editor renders and activates imported Page Links. `tests/replay_link_page_button.txt` appends a Page Link or a URL Link to the last page of the native two-page base. Both port-generated files opened and saved byte-identically in native Sketchpad, and clicking the generated Page Link there navigated to page 1. The writer updates each edited page's object count, inserts Link groups before its trailer, and validates reordered global object IDs after save. `two_page_two_points_native.gsp` exercises an earlier-page insertion that shifts a later-page point ID. `replay_link_earlier_page.txt` generated first-page Page and URL Links; native Sketchpad opened and saved both byte-identically. Clicking the generated first-page Page Link there navigated to page 2.

`two_page_hide_show_first_native.gsp` places a Hide/Show button on page 1. `action_link_page_activate_hide_show_native_created.gsp` was made through native Sketchpad's Link dialog on page 2. The Link action payload stores a one-based destination button ordinal at bytes 16–19; the destination Hide/Show action stores `1` at bytes 18–19. The Link's `0x07d8` payload is the source page's zero-based object index. Changing only the Link target field and `0x07d8` did not activate the destination action in native Sketchpad. Setting the destination Hide/Show action field did. Two native Links targeting the same button left that field at `1`. `replay_link_destination_button.txt` generates the equivalent document through Rust; native Sketchpad opened and saved it byte-identically, and clicking the Link hid the point on page 1. The Rust editor imported and activated both native and generated variants. Creation with Animation, Movement, Scroll, Presentation, or other destination actions and general multi-page structural editing remain pending.

`replay_link_destination_button_clicked.txt` saves the result after the destination Hide/Show button runs in Rust. The writer patches the target point's visibility and the button's action and label in place, then verifies a reparse. The saved file leaves page 2 bytes unchanged, opened and saved byte-identically in native Sketchpad, and displayed the point as hidden. A desktop click followed by Save also reopened with the hidden point and Show button. Other multi-page state changes remain guarded.

Creating a Page Link and changing its destination Hide/Show state before the first save produces the same bytes as saving the Link and the state in two steps.

`tests/replay_multi_page_hide_show.txt` also creates a Hide/Show button on page 1 and a Page Link to that new button on page 2 before either is saved. The writer sets the destination marker in the new Hide/Show action record while generating its group. `two_page_new_hide_show_link_port.gsp` opened and saved byte-identically in Sketchpad 5. Clicking its Page Link there hid the page-1 point and switched the new button to Show in `two_page_new_hide_show_link_port_clicked_by_wine.gsp`. The Rust editor also activates and saves this link.
