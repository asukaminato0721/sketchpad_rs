# Numeric caption records

Native measurements, coordinate measurements, slope, and point-value/equation
captions retain their signed `0x0903` coordinates at payload offsets 12 and 14.
The canvas, caption picking, drag handling, and SVG export use those positions.
The writer patches only these coordinates and the requested precision while
retaining native format fields and caches.

`tests/fixtures/measurement_copy_native.gsp` was created by opening the native
segment-length fixture in Sketchpad 5, selecting all, copying, pasting, and
saving. The source caption is at `(10,5)` and its copy at `(30,25)`. Native
geometry and caption selection were visible in the capture. Clipboard captions
in the port retain their source placement and add `(20,20)`; the old multi-page
perimeter paste fixture was updated from the template position to the source
position plus that offset.

`tests/fixtures/measurement_font_native_verified.gsp` changes the native
segment-length caption's `0x07d8` reference from style 1 to existing style 0.
Sketchpad displayed a 24-unit bold caption and saved byte-identically. The
original style 1 displays at size 18. The port reads the actual indexed font
rather than a numeric-kind-specific default and carries the style through
clipboard copying. Numeric measurement captions can use the existing text
format commands. Font edits preserve coordinates, geometry, and values in
save/reopen regressions. `tests/measurement_caption_replay.txt` formats a native
length caption to Arial, bold, size24; the generated file displayed those
settings in Sketchpad and saved byte-identically as
`measurement_caption_formatted_verified.gsp`. This does not establish exact
native glyph metrics.

Native files also use optional `0x07d5` labels and known `0x0903` representation
variants. Labels are retained independently from measurement expressions and
units. Some older port-generated files omit a numeric default style-table
entry; Sketchpad accepts their established implicit reference. The decoder
retains that compatibility while rejecting unrelated out-of-range references.

An unrelated opaque axis style no longer prevents appending a supported font:
the writer preserves every existing table slot and chooses a decodable template
for additions. It never substitutes a default for an unsupported selected font.

Regression coverage is in `document/tests/text_and_style.rs`,
`editor/tests/precision_and_clipboard.rs`, and the native numeric import tests.
Native measurement names and symbol typography are still separate work from
retaining label bytes, position, and indexed caption fonts.

## Moving existing captions on multiple pages

An imported multi-page document carries the decoder's partial-view flag even
when all its objects are known. Moving a saved measurement caption now accepts
that bounded case, requiring the original object count, no deleted or unknown
objects, and no image read-only state. A candidate document must serialize and
reparse successfully before either live state or history changes. Active-page
checks remain in place; save validation retains every other source object.

`multi_page_measurement_caption_moves_atomically_and_preserves_other_page`
checks a signed position (-20,150), values and fonts, unchanged other objects,
save/reopen, undo/redo, and rejected off-page/unknown-object attempts. To generate
the actual writer output for a native probe, run
`SKETCHPAD_WRITE_CAPTION_FIXTURE=1 cargo test multi_page_measurement_caption_moves --lib`.
The resulting 4,874-byte file opened and saved byte-identically in Sketchpad 5;
the script succeeded before the byte comparison. The durable native-verified
result is `tests/fixtures/two_page_measurement_caption_moved_native_verified.gsp`,
and the capture prefix is `/tmp/multi_page_measurement_caption_moved_native`.

## Calculation display modes

The shared calculation-caption renderer distinguishes the native display modes
verified by controlled probes. Mode 1 shows a value, optionally prefixed by its
saved numeric name. A named mode-3 value uses its saved name (for example,
`Color = 0.33`) rather than its full expression. Unnamed mode-3 and other modes
retain the expression display until independently probed. Header flag 2 hides
the observed mode-1 caption without hiding the calculation's dependent geometry;
visibility flag 1 is separate. The native empty `0x232e` attribute does not itself
hide the caption. The same utility supplies desktop/SVG text and picking bounds,
so a suppressed caption leaves no numeric hit rectangle. The numeric integration
regression checks the native named value and the controlled visible/suppressed
cases; detailed files and probes are in `numeric_native_import_analysis.md`.
