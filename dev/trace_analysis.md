# Trace Objects and Erase Traces

Sketchpad 5 changes bit 0 of byte 4 in an object's `0x07d0` header payload when Trace Objects is enabled. This was observed for a free point, segment, ray, line, circle, and polygon interior. Its own save also changes document presentation records and adds a `0x2328` marker. A Rust-generated file with only the header bit changed opened in Sketchpad 5, retained the bit on save, and produced a visible red trail when its point was dragged. Erase Traces removed the painted trail while leaving tracing enabled.

The document model reads and writes that bit for every object header, preserving the other flag bits. The editor exposes Trace Objects / Stop Tracing for selected points, straight objects, circles, arcs, and polygon interiors. During a drag or animation it records their geometry and draws sampled point marks, clipped straight strokes, circle and arc outlines, and polygon fills below the current geometry. Polygon samples use the same concave polygon triangulation as live geometry. Erase Traces clears only that in-memory history. Traces are bounded to 8,192 samples per session. Undo and redo cover the persistent trace flag.

`tests/replay_trace.txt` generates a traced point and segment. Both port-generated files opened in Sketchpad 5 and saved byte-identically; copies are `trace_port_point_saved_by_wine.gsp` and `trace_port_segment_saved_by_wine.gsp`. Dragging the port-generated point left a trail, and Erase Traces cleared it. The original-saved result is `trace_port_point_dragged_by_wine.gsp`; captures are `/tmp/sketchpad_trace_port_point_drag.trail.png` and `/tmp/sketchpad_trace_port_point_drag.erased.png`. Native trail sampling and clearing are covered by a binary test; a native GUI drag capture remains to be done.

`trace_circle_original.gsp`, `trace_arc_original.gsp`, and `trace_polygon_original.gsp` are native files with the respective trace flags set. A twelve-step drag in Sketchpad 5 showed a pale filled polygon trail before mouse release; the capture is `/tmp/sketchpad_trace_polygon_multidrag_during_drag.png`. All three import, retain their trace flags on encode, and support undo and redo. `tests/replay_trace_arc.txt` enables tracing on a native three-point arc; Sketchpad 5 opened and saved the port-traced file byte-identically (2,040 bytes).

`tests/replay_trace_shapes.txt` generates a traced circle and polygon together. Sketchpad 5 opened and saved that port-generated file byte-identically (2,104 bytes, 86 records).
The desktop drag capture `/tmp/sketchpad_trace_shapes_native_during_drag_fixed.png` confirms that the port draws a pale polygon history below the current interior.

`tests/replay_trace_straight_base.txt` generates traced rays and lines. Native Trace Objects set the same header bit for each, and the native line drag capture `/tmp/sketchpad_trace_line_drag_probe_during_drag.png` shows viewport-length historical strokes. Both port-generated files opened and saved byte-identically in Sketchpad 5 (1,968 bytes each). The port stores the ray or line mode with each sampled pair of endpoints and clips the historical stroke to the viewport at draw time. `/tmp/sketchpad_trace_line_port_during_drag_fixed.png` confirms the port's desktop trail across the full visible line.

Sampled custom-transformation curves now retain traces during animation and dragging.
The `弦图【扫描版】.gsp` sample enables tracing on object 56, a sampled colorized curve.
Its Scan button moves hidden point 41. Previously, trace capture omitted sampled curves,
so the scan moved without retaining any history.

Trace capture now uses the same 384-subdivision budget as the live curve renderer.
Each snapshot retains separate strokes, their colors, and their line styles.
The sampler omits undefined spans. Each complete curve counts as one history sample,
so the 8,192-sample limit does not discard old scan rows after a few frames.
Rendering uses the stored styles and applies the existing trace fade setting.
Trace Objects and Stop Tracing also accept sampled curves.

`tests/fixtures/chord_scan_original.gsp` is an unchanged copy of the reported sample.
Editor tests activate its Scan button and check 24 retained snapshots, fixed historical
colors and positions, stopping, clearing, visibility, and the active-page filter.
A separate test covers curves without color mapping. The engine trace test checks
byte-exact import, Stop Tracing, undo/redo, and save/reopen for the reported sample.
An Xvfb desktop check confirms that the Scan button leaves a colored band after
stopping and that Display > Erase Traces removes that band.
The captures are `/tmp/chord-scan-before.png`, `/tmp/chord-scan-after.png`, and
`/tmp/chord-scan-erased.png`.
