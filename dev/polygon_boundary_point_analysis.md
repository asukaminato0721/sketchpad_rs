# Polygon boundary point constraints

The sibling fixture `../gsp-rs/tests/fixtures/gsp/static/poly_point_with_val.gsp` and its paired HTM provide a four-vertex polygon and a boundary point. The native code-15 constraint is 20 bytes: u16 code 15, u16 polygon subtype (49 plus vertex count), f64 local edge fraction, u32 zero-based edge index, then bytes `ff ff 00 00`.

The fixture stores fraction 0.4450450665338869 on edge 2 (C to D). Native Sketchpad displays the point at approximately (607.23,355.28), matching interpolation on that edge. Its linked Value of Point is 0.58, the fraction of total perimeter traveled from A; the raw local fraction is not the displayed perimeter value. The HTM reports the raw 0.445045 fraction and is therefore insufficient by itself to derive perimeter behavior.

A native mouse drag from edge 2 onto A to B changes the saved edge to 0 and local fraction to 0.5116564522216039, retaining the four trailer bytes. This result is `tests/fixtures/polygon_boundary_dragged_native.gsp`; the original is `tests/fixtures/polygon_boundary_original.gsp`. Native probes used the 64-bit Wine prefix `/home/w/.cache/sketchpad_rs_wine` with the wow64 Wine binary and a 1280×1024 virtual display.

The Rust model retains the edge index and local fraction, so moving a defining vertex keeps the same relative edge location. Dragging projects to the nearest noncollapsed edge. Animation traverses normalized perimeter distance and wraps at the first vertex. The point has ordinary point labels/styles and participates in dependency copying, deletion, and point measurements. Native encoding stores the local fraction and edge, while linked point values compute perimeter distance from the current vertices.

`tests/polygon_boundary_replay.txt` creates a fresh rectangular polygon, projects a boundary point, adds its linked value, then drags it onto another edge. Native Sketchpad displayed the point on the lower edge and its perimeter value as 0.72. `dev/wine_roundtrip.sh` succeeded, and a separate `cmp` confirmed byte identity of the 2314-byte result. The saved result is `tests/fixtures/polygon_boundary_generated_verified.gsp`.
