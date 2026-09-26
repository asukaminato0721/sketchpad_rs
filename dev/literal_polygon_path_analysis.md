# Literal polygon path points (native code 123)

Code 123 has its own native proof; support is not inferred from code 95. The
existing native 80-byte literal group was rebound to a known polygon using only
its one host reference and its finite `f64` literal. The group retains its
28-byte point header, one-reference `0x07d2` record, 12-byte `0x07d3` payload
`123 + f64`, and terminator. No new header, subtype, or constraint schema was
invented for polygon hosts.

The controlled rectangle is `(100,100),(220,100),(220,160),(100,160)`, perimeter
360. Native marker segments expose the plotted point position:

| Literal | Native point | Interpretation |
| --- | --- | --- |
| 0.25 | (190,100) | 90 units along the perimeter |
| -0.25 | (130,160) | Same location as 0.75 |
| 1.25 | (190,100) | Same location as 0.25 |

All three native files opened successfully and saved byte-identically (2256
bytes). The corresponding before-save captures are
`/tmp/literal_polygon_{quarter,negative,over}_native_before_save.png`. Visible
marker endpoints establish both position and definedness; file acceptance alone
is not used to establish the domain. Verified source files are retained as
`tests/fixtures/literal_polygon_path_{quarter,negative,over}_native_verified.gsp`.

The literal parser remains the same strict 80-byte schema. Code 123's independent
host predicates now admit known polygon kinds with finite literals. Geometry
uses the live normalized polygon perimeter with modulo-one wrapping. Host vertex
movement changes edge lengths and the plotted position. The API requires a
single-page editable source, a live polygon on the active page, a finite literal,
and a defined perimeter before it checkpoints or creates an object. Undefined
perimeters yield no position. Import and roundtrip validation require a live,
earlier, same-page host and a defined point. Unknown record save guards remain
unchanged.

Regression tests cover the three actual native files, source movement,
save/reopen, clipboard remapping and standard paste offset, dependent deletion,
creation undo/redo, invalid references/nonfinite values, and atomic failed
creation on a collapsed polygon. `tests/literal_polygon_path_replay.txt` authors
all three points and marker segments through the actual document API, then moves
a source vertex and saves a second output.
Both actual API outputs subsequently opened and saved byte-identically in
Sketchpad (2544 bytes each). The retained
`literal_polygon_path_generated_native_verified.gsp` and
`literal_polygon_path_edited_native_verified.gsp` fixtures check all three
literal kinds, live geometry, wrapping, and unchanged save. Native before-save
captures `/tmp/literal_polygon_path_{generated,edited}_port_before_save.png`
show the marker endpoints before and after the source vertex movement. Each
byte comparison followed a successful native script exit.
