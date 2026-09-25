# Trace Objects and Erase Traces

Sketchpad 5 changes bit 0 of byte 4 in an object's `0x07d0` header payload when Trace Objects is enabled. This was observed for a free point and a segment. Its own save also changes document presentation records and adds a `0x2328` marker. A Rust-generated file with only the header bit changed opened in Sketchpad 5, retained the bit on save, and produced a visible red trail when its point was dragged. Erase Traces removed the painted trail while leaving tracing enabled.

The document model reads and writes that bit for every object header, preserving the other flag bits. The editor exposes Trace Objects / Stop Tracing for selected points and segments. During a drag it records point locations and segment endpoints, then draws their traces over the native canvas. Erase Traces clears only that in-memory history. Traces are bounded to 8,192 samples per session. Undo and redo cover the persistent trace flag.

`tests/replay_trace.txt` generates a traced point and segment. Both port-generated files opened in Sketchpad 5 and saved byte-identically; copies are `trace_port_point_saved_by_wine.gsp` and `trace_port_segment_saved_by_wine.gsp`. Dragging the port-generated point left a trail, and Erase Traces cleared it. The original-saved result is `trace_port_point_dragged_by_wine.gsp`; captures are `/tmp/sketchpad_trace_port_point_drag.trail.png` and `/tmp/sketchpad_trace_port_point_drag.erased.png`. Native trail sampling and clearing are covered by a binary test; a native GUI drag capture remains to be done.

Trace support is bounded to points and segments. Traced circles, rays, lines, arcs, polygons, and dependent motion through actions or animations still need behavior checks and drawing support.
