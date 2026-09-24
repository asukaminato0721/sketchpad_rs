# File format and runtime analysis

The development tools follow the same pattern as the headless probes in `../siglus_rs`: a deterministic Rust probe for file and model behavior, plus targeted analysis of the original executable. Keep conclusions tied to a fixture, a runtime observation, or a specific instruction in the original binary.

## Headless probe

```sh
cargo run --bin gsp_probe -- inspect tests/fixtures/point.gsp
cargo run --bin gsp_probe -- diff tests/fixtures/point.gsp tests/fixtures/point_hidden.gsp
cargo run --bin gsp_probe -- verify tests/fixtures/*.gsp
cargo run --bin gsp_probe -- demo /tmp/sketchpad_demo.gsp 9
cargo run --bin gsp_probe -- replay tests/replay_basic.txt
cargo run --bin gsp_probe -- replay tests/replay_circle_intersections.txt
cargo run --bin gsp_probe -- replay tests/replay_single_circle_root.txt
cargo run --bin gsp_probe -- replay tests/replay_circle_first.txt
cargo run --bin gsp_probe -- replay tests/replay_bisector.txt
cargo run --bin gsp_probe -- replay tests/replay_three_point_arc.txt
cargo run --bin gsp_probe -- replay tests/replay_circle_segment.txt
cargo run --bin gsp_probe -- replay tests/replay_point_on_object.txt
cargo run --bin gsp_probe -- replay tests/replay_arc_on_circle.txt
cargo run --bin gsp_probe -- replay tests/replay_point_on_arc.txt
cargo run --bin gsp_probe -- replay tests/replay_translation.txt
cargo run --bin gsp_probe -- replay tests/replay_reflection.txt
cargo run --bin gsp_probe -- replay tests/replay_dilation.txt
cargo run --bin gsp_probe -- replay tests/replay_rotation.txt
cargo run --bin gsp_probe -- replay tests/replay_clear.txt
cargo run --bin gsp_probe -- replay tests/replay_clear_remap.txt
cargo run --bin gsp_probe -- replay tests/replay_copy_paste.txt
cargo run --bin gsp_probe -- replay tests/replay_cut_paste.txt
cargo run --bin gsp_probe -- replay tests/replay_length.txt
cargo run --bin gsp_probe -- replay tests/replay_distance.txt
cargo run --bin gsp_probe -- replay tests/replay_circumference.txt
cargo run --bin gsp_probe -- replay tests/replay_radius.txt
cargo run --bin gsp_probe -- replay tests/replay_angle.txt
cargo run --bin gsp_probe -- svg /tmp/sketchpad_arc_on_circle.gsp /tmp/sketchpad_arc_on_circle.svg
```

`inspect` prints each record's file offset, type, payload length, and the decoded objects. `diff` reports changed record slots and byte ranges. `verify` requires an exact byte round trip. `demo` creates one to nine objects in a repeatable order for reopening in Sketchpad 5. `replay` runs a bounded command script against the document model; it supports construction, point movement, visibility, undo/redo, cut/copy/paste, length, distance, circumference, radius, angle, area, arc angle, and arc length measurement, save/reopen checks, and optional `SKETCHPAD_TRACE=1` model dumps. The current bundled corpus has 54 files; all pass `verify`. All 105 sibling `.gsp` files also pass an exact byte round trip, although many are only partially decoded. The original hidden-point variant differs from the visible file at only payload byte `+2` of object header record `0x7d0`.

## Ghidra

The supplied `GSP5Chs.exe` is a 32-bit Windows PE with SHA-256 `5df6592cdd3475fec6c31998527f3bf78ef8bb72d476bbd1578f08d8e2c1a2c7`. `Gsp4Xrefs.java` finds the GSP4 signature; `DumpFunction.java` decompiles specified addresses. Example headless invocation after importing the executable into a Ghidra project:

```sh
XDG_CONFIG_HOME=/tmp/sketchpad-ghidra-config XDG_CACHE_HOME=/tmp/sketchpad-ghidra-cache \
  /opt/ghidra/support/analyzeHeadless /tmp/sketchpad-ghidra-project Sketchpad5 \
  -process GSP5Chs.exe -noanalysis -scriptPath "$PWD/dev/ghidra" \
  -postScript Gsp4Xrefs.java
```

The signature occurs as an immediate operand at `00416296` (`CMP ECX,0x34505347`), `00416642`, and `004f5b52`. The first is in a reader function entered at `00416230`; the second is in a header writer entered at `00416610`. The page writer at `004a8520` counts objects in its linked list before calling `004a80f0`, which emits the `1000` page preamble record. This supports the observed fixture behavior: a one-object file has value `1` at offset `+24` of the `0x3e8` preamble payload, and updating that value to `9` was required before the generated nine-object file opened in Sketchpad 5. The exact semantics of the other preamble fields remain unverified.

## Original application runtime check

Create a copy of the nine-object demo and run `SKETCHPAD_WINE_DRAG=1 ./dev/wine_roundtrip.sh COPY.gsp`. The script opens it in Sketchpad under Wine, drags the first point from `(220, 230)` to `(261, 181)`, and sends Save. The saved result is bundled as `tests/fixtures/wine_moved_demo.gsp`. The point coordinate record changed, and the original application added a `0x2328` selection record. The port reopens this file with all nine objects and evaluates the midpoint at `(370.5, 270.5)`. The original application also rewrote window and style metadata, so exact byte identity after an original-app save is not expected.

A second check reopened that original-app save in Rust, moved point 2, saved a new file, and opened the new file in Sketchpad 5. The nine-object sketch rendered there. This confirms a limited bidirectional edit/save/open cycle for these object kinds.

`tests/replay_intersection.txt` constructs two segments and their crossing, then saves it. Sketchpad 5 opened the Rust-generated file and rendered the crossing point. The source fixture uses object code `9` with references to the two straight objects; the port uses that record layout for writing.

The original line-circle and circle-circle fixtures use object codes `11` and `12` for the two line-circle roots and `13` and `14` for the two circle-circle roots. `tests/replay_circle_intersections.txt` creates all four from Rust. Sketchpad 5 opened the generated 13-object sketch, displayed both circle-circle roots and both line-circle roots, and saved it without changing any record payloads. The port reopens the saved copy with all 13 objects. These checks cover the displayed initial positions; they do not yet establish all topology changes after dragging.

`tests/replay_single_circle_root.txt` constructs a segment with only one of the two full-line circle crossings on it, then writes a code `11` intersection. Sketchpad 5 displays the remaining crossing and saves the file unchanged. The port selects the remaining valid crossing for either root variant in this case.

The line-circle reference order matters. Sketchpad 5 opened a generated file whose code `11` and `12` records listed the circle before the segment, but displayed neither intersection. `tests/replay_circle_first.txt` creates the circle first, then serializes each intersection with its later straight-object ID before the earlier circle ID. Sketchpad 5 displayed the remaining crossing and saved that file unchanged. The parser now treats the reversed order as unsupported, and the writer refuses it.

The original angle bisector fixture uses object code `7` and three point references in start, vertex, end order. `tests/replay_bisector.txt` writes this construction from Rust. Sketchpad 5 displayed the bisector ray and saved the generated file without changing any record payloads.

The original three-point arc fixture uses object code `81` and start, on-arc, end references. `tests/replay_three_point_arc.txt` writes this construction from Rust. Sketchpad 5 displayed the arc through all three points and saved the generated file without changing any record payloads.

The original center-and-radius-segment circle fixture uses object code `4`, with center point and segment references. `tests/replay_circle_segment.txt` writes this construction from Rust. Sketchpad 5 displayed the expected circle and saved the generated file without changing any record payloads.

Original files use object code `15` for points on objects. Segment points have one host reference and a `0x7d3` constraint payload with prefix `0f 00 0e 00` followed by a segment parameter. Circle points use prefix `0f 00 1c 00` followed by two unit-vector coordinates; the Y component is positive upward in this payload. `tests/replay_point_on_object.txt` writes both variants. Sketchpad 5 opened the six-object file, displayed both constrained points in the expected locations, and saved it without changing any record payloads. The first cross-open screenshot exposed the Y sign error, which was corrected and verified in a second cross-open check.

The original Arc on Circle fixture uses object code `79` with references to a circle and two points constrained to it. `tests/replay_arc_on_circle.txt` writes this construction, adds a point constrained to the arc, and hides the source circle. Sketchpad 5 displayed the expected quarter arc and its constrained point, then saved the file without changing any record payloads.

Points constrained to arcs also use object code `15`, with `0x7d3` prefix `0f 00 43 00` and a normalized arc parameter. `tests/replay_point_on_arc.txt` writes a point on a three-point arc. Sketchpad 5 displayed it at the expected position and saved the generated file without changing any record payloads.

The visible page can be exported as SVG from the menu or headless probe. The SVG generated from the seven-object Arc on Circle replay rendered correctly through `rsvg-convert`, and conversion to PDF succeeded. The Print menu sends that PDF through `lp`; a physical printer was not available for an end-to-end print check.

The original polar point translation fixture uses object code `21`, one source-point reference, and a 60-byte `0x7d3` payload. Its angle is the `f64` at offset 20, unit scale at offset 32, and distance at offset 40. The supplied fixture translates a point upward by one centimeter at 90 degrees. `tests/replay_translation.txt` writes the same record shape from Rust. Sketchpad 5 displayed the generated point at the expected offset and saved the file without changing any record payloads. Vector-based and measurement-driven translations remain unverified.

A second Wine run dragged the generated translation's source point from screen `(259, 289)` to `(300, 290)`. Sketchpad moved the translated point by the same offset and saved a document that the port reopened as source point `(241, 221)` plus the unchanged 90-degree translation. The original application also refreshed its document metadata, a selection record, and trailing bytes of the code-21 constraint payload; the semantic angle, scale, and distance fields stayed unchanged.

The original reflection fixture uses object code `34` with two references: source object, then mirror line. Its point variant has the same point-class header pattern as a free point, with no constraint payload. `tests/replay_reflection.txt` generates that record from Rust. Sketchpad 5 displayed the reflected point across the segment and saved all 86 records unchanged. In a second Wine run, dragging the source point from screen `(259, 179)` to `(300, 199)` moved the reflected point as expected. The saved file is `tests/fixtures/wine_moved_reflection.gsp`; the port reopens its source at `(241, 130)` and dependent at `(241, 210)`.

The same code `34` and reference order handles a reflected circle; its header uses the circle-class bytes from the original fixture. The expanded reflection replay generates both variants. Sketchpad 5 displayed the reflected circle and saved all 98 records unchanged. Dragging the source circle's center changed the source radius and moved and resized the reflected circle in the original app. The saved file `tests/fixtures/wine_moved_reflected_circle.gsp` reopens in the port with the matching center and radius. Line and polygon reflection variants remain unsupported.

The original scale fixture uses object code `30` with source and center references and a 12-byte `0x7d3` payload: prefix `1e 00 00 00`, then an `f64` ratio. The fixture includes point and circle dilation by one third. `tests/replay_dilation.txt` creates both variants at ratio one half. Sketchpad 5 displayed both at the expected positions and sizes, and saved all 94 records unchanged. Dragging the shared center updated both in the original app; its saved result is `tests/fixtures/wine_moved_dilation.gsp`, which the port reopens with both dependents at the expected positions.

The original rotation fixture uses object code `27` with source point and center point references. Its 36-byte `0x7d3` payload stores negative sine at offset 4, cosine at offset 12, and angle in degrees at offset 20. The first generated 90-degree file changed only the angle field; Sketchpad 5 still displayed the template's 60-degree rotation. Updating all three fields produced the expected 90-degree point. Sketchpad 5 saved that file's 81 records unchanged. Dragging the source point moved the rotated point in the original app; its saved result is `tests/fixtures/wine_moved_rotation.gsp`, which the port reopens with the expected position. Other rotated object classes remain unsupported.

For deletion, `tests/replay_delete_probe.txt` creates two free points, a segment, its midpoint, and an unrelated point. Selecting the first point and pressing Delete in Sketchpad 5 removed the point, segment, and midpoint. Its saved result `tests/fixtures/wine_deleted_source.gsp` has two point groups and an object count of two. `tests/replay_clear.txt` generates the same two-object result in Rust; the object records and file size match the original app's result, with differences only in document metadata the original rewrote. Sketchpad 5 reopened and saved the Rust file without changing any record payload. `tests/replay_clear_remap.txt` deletes an independent earlier point while retaining a segment and midpoint; Sketchpad 5 displayed both dependent shapes with the remapped references and saved all 83 records unchanged. The Rust editor permits undo and redo before saving the deletion and refuses deletion when unsupported or multiple-page content makes structural rewriting unsafe.

The same five-object probe was used for Copy and Paste. Copying the first free point in Sketchpad 5 and pasting it created a point at `(170, 140)`, an offset of `(20, 20)`. Selecting only the segment and pasting copied its two parent points and the segment, but not the dependent midpoint; the saved original-app file is `tests/fixtures/wine_pasted_segment.gsp`. `tests/replay_copy_paste.txt` generates the same eight object kinds and positions in Rust. Sketchpad 5 opened the Rust file, displayed the pasted endpoints, and saved all 95 records unchanged. `tests/replay_cut_paste.txt` checks deletion, pasted reference remapping, and reopening together; Sketchpad 5 saved its generated six-object result with all 89 records unchanged. The current clipboard is internal to the editor and supports only modeled object kinds.

Selecting only the segment and cutting it in Sketchpad 5 removed both the segment and its midpoint, while leaving the three free points. That original-app result is `tests/fixtures/wine_cut_segment.gsp`. The port now treats the segment as a required parent for midpoint copy and as a deletion dependency.

Selecting a segment and choosing Measure > Length in Sketchpad 5 created object code `36` with one segment reference, a 32-byte `0x0903` format record, and a 2-byte `0x07d8` record. The `0x0903` record was identical for 100- and 200-unit segments and in sketches with three and five prior objects. The original measured 100 raw units as `2.65 cm`, implying the same 96-dpi centimetre conversion used by the port. `tests/replay_length.txt` writes a four-object measured sketch in Rust; Sketchpad 5 displayed `2.65` and saved all 85 records unchanged. Dragging its endpoint in Sketchpad 5 from 100 to 200 raw units updated the display to `5.29`; that saved file is `tests/fixtures/wine_moved_length.gsp`. The original rewrote two presentation bytes in `0x0903` when saving the drag, so the parser accepts both observed variants. The original-created reference is `tests/fixtures/wine_segment_length.gsp`. Other measurement commands and label placement remain pending.

Selecting two points and choosing Measure > Distance created object code `37` with two point references, a 32-byte `0x0903` format record, and a 2-byte `0x07d8` record. The original-created reference is `tests/fixtures/wine_point_distance.gsp`. `tests/replay_distance.txt` writes a three-object distance sketch in Rust; Sketchpad 5 displayed `2.65` and saved all 82 records unchanged. Dragging the second point from 100 to 200 raw units updated the value to `5.29`; that saved file is `tests/fixtures/wine_moved_distance.gsp`. The original changed one byte in `0x0903` from `0x72` to `0x61` during the drag/save, and the parser accepts both observed values. Text labels and their position are still approximated in the port.

Selecting a circle and choosing Measure > Circumference created object code `40` with one circle reference and the same format/style record kinds. The original-created reference is `tests/fixtures/wine_circle_circumference.gsp`. `tests/replay_circumference.txt` writes a four-object measured circle; Sketchpad 5 displayed `16.62` for radius 100 and saved all 85 records unchanged. Dragging the radius point to radius 150 updated the original display to `24.94`; that saved file is `tests/fixtures/wine_moved_circumference.gsp`. The original replaced two apparent presentation bytes in `0x0903` with zeros, and the parser accepts both observed variants.

Selecting a circle and choosing Measure > Radius created object code `46` with one circle reference and the same format/style record kinds. The original-created reference is `tests/fixtures/wine_circle_radius.gsp`. `tests/replay_radius.txt` writes a four-object measured circle; Sketchpad 5 displayed `2.65` for radius 100 and saved all 85 records unchanged. Dragging the radius point to radius 150 updated the original display to `3.97`; that saved file is `tests/fixtures/wine_moved_radius.gsp`. The original changed a presentation byte in `0x0903` from zero to `0x61`; the parser accepts both observed values.

Selecting three points in start, vertex, end order and choosing Measure > Angle created object code `41` with three point references, a 32-byte `0x0903` format record, and a 2-byte `0x07d8` style record. The original-created reference is `tests/fixtures/wine_point_angle.gsp`. `tests/replay_angle.txt` writes a four-object measured sketch; Sketchpad 5 displayed `90.00°` and saved all 85 records unchanged. Moving the third point changed the original display to `135.00°`; both the original-created and Rust-generated edited files reopen as dynamic angle measurements in the port (`tests/fixtures/wine_moved_angle.gsp` and `tests/fixtures/wine_moved_generated_angle.gsp`). Comparing these and the other measurement fixtures shows that Sketchpad rewrites bytes 22, 30, and 31 of `0x0903` without changing measurement behavior. The parser compares the other 29 bytes and preserves all source bytes on a round trip.

Selecting a circle and choosing Measure > Area created object code `42` with one circle reference, the same 32-byte format record, and a 2-byte style record. The original-created reference is `tests/fixtures/wine_circle_area.gsp`. `tests/replay_area.txt` writes a four-object measured circle; Sketchpad 5 displayed `21.99 cm²` for radius 100 and saved all 85 records unchanged. Dragging the radius point to radius 150 updated the display to `49.48 cm²` in both the original-created and Rust-generated files. The edited files (`tests/fixtures/wine_moved_area.gsp` and `tests/fixtures/wine_moved_generated_area.gsp`) reopen with a dynamic area measurement in the port.

Selecting a three-point arc and choosing Measure > Arc Angle created object code `43` with one arc reference, a 32-byte format record, and a 2-byte style record. The original-created reference is `tests/fixtures/wine_arc_angle.gsp`. `tests/replay_arc_angle.txt` writes a five-object measured sketch; Sketchpad 5 displayed `159.22°` and saved all 88 generated records unchanged. Dragging the middle point from y=130 to y=90 changed the original display to `225.24°`; that edited generated file is `tests/fixtures/wine_moved_arc_angle.gsp` and reopens as a dynamic measurement in the port. A generated arc-on-circle angle displayed `90.00°` in Sketchpad 5.

Selecting a three-point arc and choosing Measure > Arc Length created object code `44` with one arc reference, a 32-byte format record, and a 2-byte style record. The original-created reference is `tests/fixtures/wine_arc_length.gsp`. `tests/replay_arc_length.txt` writes the same five-object geometry with a length measurement; Sketchpad 5 displayed `4.49 cm` and saved all 88 generated records unchanged. Dragging the middle point from y=130 to y=90 updated the original display to `6.76 cm`; that edited generated file is `tests/fixtures/wine_moved_arc_length.gsp` and reopens as a dynamic measurement in the port. A generated arc-on-circle length displayed `2.91 cm` in Sketchpad 5.

`tests/replay_arc_measurements_on_circle.txt` adds both arc measurements to one file. Sketchpad 5 opened it, displayed both values, and saved all 101 records unchanged; the saved result is `tests/fixtures/wine_arc_measurements_on_circle.gsp`. The first writer version placed both labels at the same location. Adding Radius to an original Area sketch showed that Sketchpad moves a second measurement label down by 28 units, stored at bytes 14–15 of its `0x0903` payload. The writer now follows that spacing, and the parser accepts label coordinates that differ from the template. The original two-measurement reference is `tests/fixtures/wine_circle_area_radius.gsp`.

Use a copy for this check because the original application edits the file. The current automated tests establish byte preservation and model behavior; they do not establish full command parity or cross-application editing for every command.
