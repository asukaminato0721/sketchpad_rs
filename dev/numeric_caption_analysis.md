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
