# Literal captions on multiple pages

Multi-page captions use the existing verified code-0 text groups: `0x08fc`
contains the rich caption, `0x0898` its bounding rectangle, and `0x07d8`
references the document font table. Caption groups have no construction parents.
The native single-page caption fixtures remain the record-format reference;
this change inserts those same groups into the selected page.

The multi-page writer now accepts literal text additions and source caption
edits. It applies variable-length caption replacements after fixed-offset
source-state edits and page additions. Groups are matched in serialization
order (page, then original objects, then additions), so adding a caption on
page 1 after adding one on page 2 cannot apply text or fonts to the wrong group.
Whole-caption font overrides use this same order; existing font-family and
style table encoders preserve mixed-range formatting when copying captions.

`cargo test --lib multi_page_literal_text` covers source edits, movement,
whole-caption bold, substring italic, cross-page copy/paste, later additions on
an earlier page, line endings, Unicode, new caption and range font families,
undo/redo, and clearing an edited original caption while preserving the other
page. `tests/replay_multi_page_text.txt` exercises the creation and clipboard
flow through the command probe and verifies its saved round trip.

The four-caption two-page replay output opened in native Sketchpad 5 under
Wine and saved byte-identically (3,648 bytes). The first page displayed the
edited two-line caption with its italic substring. The saved native file is
`tests/fixtures/two_page_text_port_saved_by_wine.gsp`; it verifies import and
further caption edits without requiring Wine for the automated regression.
The native probe used `dev/wine_roundtrip.sh` with the known-good Wine prefix
at `/home/w/.cache/sketchpad_rs_wine`.

Multi-page imports retain `partial_view = true` because the application still
limits some editing commands on these documents. The canvas Text tool now
passes that generic multi-page gate and delegates its safety checks to
`Document::add_text`. An editor regression clicks the Text tool on the
native-saved two-page fixture, checks the new caption and selection, then
edits the caption and verifies the saved round trip. The sidebar text editor
and Display text-format menu already delegate to the document APIs.
