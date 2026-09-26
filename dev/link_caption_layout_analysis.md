# Native URL padding and link-caption rectangles

The two-page ant source (`tests/fixtures/mapped_point_iteration_ant_native.gsp`)
contains URL link buttons 37 and 98. Their object code is 0, action type is 6,
URL and label are `http://exjh.com`, and their caption rectangle `08a2` is
`[33,575,138,595]`. Both action payloads have 44 bytes: URL length 15 at offset
24, string at 26, terminator at 41, then two additional zero bytes to align the
payload to four bytes. The previous validator required the two-byte aligned
length 42. Button 37 also has word `0x3264` at action offset 14, while button 98
has zero there. Native Sketchpad opens and saves the original file without
changing either word or the padding: 11254 bytes, SHA-256
`13450461cdc37ecedf14d7868c2572810164a7c356c8e04d684694271848b409`.
Neither link was activated.

The word's meaning is unknown. The URL parser retains it as opaque native data;
action type 6 and the surrounding checked fields still identify a URL action.
URL strings and labels accept only the observed exact two- or four-byte aligned
lengths, with a NUL terminator and all-zero padding. Unchanged URL/label setters
preserve the original payload exactly; edits retain the detected four-byte
alignment. Other action types retain their previous string validation.

A separate native probe changed the first URL and label to `http://exjk.com`
and action coordinates to `(90,575)`. Keeping `08a2` unchanged left the caption
visibly at `(33,575)` in native Sketchpad. Moving `08a2` to
`[90,575,195,595]` shifted the displayed caption 57 pixels right, as requested.
The repaired file opened and saved byte-identically, and retained `0x3264` and
44-byte padding. Its fixture is
`tests/fixtures/mapped_point_iteration_ant_url_edited_verified.gsp`, SHA-256
`bc458bcbbe06f55235a0f226f2bfb8fee80fcdd3ba9aac936deb65b7ace3aaef`.
The probe selected the first page explicitly; no URL was clicked.

Consequently, `LinkUrlButtonGroup::screen_anchor` reads the `08a2` rectangle.
Its setter translates all four rectangle coordinates while updating the
redundant action coordinates. Checked conversion prevents coordinate overflow;
an invalid translation returns an error before changing either record. Width
and height survive movement.

Page-link buttons have the same `08a2` caption layout, separate from their
24-byte action. The native fixture `action_link_page_two_original.gsp` has
anchor `(10,5)` and rectangle `[10,5,55,27]`. Movement to `(90,575)` changes that
to `[90,575,135,597]`, preserving its 45-by-22 dimensions and destination page
index 1. `LinkPageButtonGroup` now uses the same checked rectangle translation.
Its offset-14 word remains the semantic destination, unlike the opaque URL
word. Native opens and saves the moved page-link file byte-identically: fixture
`action_link_page_caption_moved_verified.gsp`, 3290 bytes, SHA-256
`ba28cbcbed0dddfa8cac071f8efd8ea6b747d00bc382c1407527f9690baf4d43`.
That capture selected the empty second page, so the page-link evidence establishes
record preservation and the shared rectangle layout without a separate visible
caption-position assertion.

Tests cover native URL import, untouched bytes, same-length URL and label edits,
padding rejection, opaque-word preservation, mismatched redundant action
coordinates, both caption translations, and atomic coordinate-range failures.
All 98 objects of the ant document now decode, including both links and both
mapped iteration definitions. The existing multi-page view flag and single-page
parameter-edit gate still apply; seed movement in the original ant source is
supported and separately native-verified.
