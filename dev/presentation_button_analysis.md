# Presentation Action Button

The [Sketchpad 5 Reference Center](https://referencecenter.dynamicgeometry.com/gsp-reference-center/objects_btns_presentprops.htm) describes Presentation as an action button that activates selected action buttons simultaneously or sequentially. Sequential mode waits for an action to finish before continuing, and a further click can advance it.

## Native records

`action_presentation_default_original.gsp` has two Hide/Show parents and a native default simultaneous Presentation. `action_presentation_sequential_scroll_original.gsp` has two Scroll parents and a native sequential Presentation. The object code is 62. Its six records are `0x07d0`, `0x07d2`, `0x0906`, `0x07d5`, `0x07d8`, and `0x07d7`. The reference record stores a count followed by the selected action button IDs in order. The `0x0906` payload has action code 7 at offset 12, mode 0 for simultaneous or 1 for sequential at offset 14, a 32-bit value 600 or 0 respectively at offset 16, and the button anchor at offsets 20–23. The native labels in these fixtures are `系列2个操作` and `顺序2个操作`. `assets/action_presentation_group.bin` preserves the default 156-byte group.

The native original and clicked fixtures import as editable with `partial_view=false` and preserve their bytes on unedited save. Reopening the native-created default and saving without a click produced `action_presentation_default_opened_by_wine.gsp`, where both points are hidden and both parents have Show actions. Clicking its Presentation and saving produced `action_presentation_default_clicked.gsp`, where both points are visible again.

## Port behavior and cross-open checks

`PresentationButtonGroup` reads and writes the observed group while retaining unknown record bytes. The document model validates selected parent action buttons, keeps their order, and writes either mode. The editor adds a mode dialog, draws the button, and activates Hide/Show, Animation, Movement, Scroll, and nested Presentation parents. Sequential execution waits for active motion. A second click advances a sequential presentation or stops simultaneous running actions.

`tests/replay_presentation_button.txt` generates simultaneous and sequential files from two Hide/Show parents. Both opened and saved byte-identically in native Sketchpad 5. Clicking either port-generated Presentation there hid both points and changed both parents to Show. `action_presentation_port_simultaneous_clicked_by_wine.gsp` and `action_presentation_port_sequential_clicked_by_wine.gsp` preserve these results. A click in the Rust desktop editor produced the same object state in `action_presentation_port_desktop_clicked.gsp`; that saved file then opened and saved byte-identically in native Sketchpad 5. Other Presentation settings and child action combinations remain to be checked.

`tests/replay_multi_page_presentation.txt` adds a simultaneous Presentation button on page 2 of a two-page document with two Hide/Show buttons. The port stores their page-local ordinals `[5, 6]` after the reference count, corresponding to global object IDs `[6, 7]`. The generated file opened and saved byte-identically in native Sketchpad 5. Clicking the Presentation button there hid all three parent targets, changed both Hide/Show actions to Show, and kept page 1 visible. The native-clicked file imports and round-trips in Rust.
