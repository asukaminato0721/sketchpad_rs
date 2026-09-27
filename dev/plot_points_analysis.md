# Graph > Plot Points

The original one-column t₁ table and default Cartesian frame produce one code 67 point for Graph > Plot as (x, y), or one code 68 point for Plot as (r, θ). The point group has records `0x07d0`, `0x07d2`, `0x07d3`, `0x07d7`. Its reference packet is `[count=1, frame=8]` in the fixtures; the parameter is not a dependency. Code 67 stores `(1, 1)`; code 68 stores radius `1` and angle `π/180` radians. Both point headers carry a medium red point style.

The document model now decodes those points, evaluates them against the live coordinate frame, renders and hit tests them as points, and writes a new point from the original group template. The Graph menu offers both modes when the document has the observed eight-object t₁ table and frame. `tests/replay_plot_points.txt` creates and reopens both variants. Rust tests cover original decode, frame dragging, undo/redo, generated save/reopen, and point style. The generated last object groups match the corresponding original groups byte for byte. SVG export shows the two distinct positions.

The writer supports repeated plotted points in supported one-page sketches with an active coordinate frame. The original two-row table has a `t` byte in a padding slot; the parser accepts and preserves that exact variant. Multi-page plotting and broader source layouts need separate record and behavior probes.

The integrated Cartesian and polar replay files opened in the bundled Windows Sketchpad 5 under Wine. Both displayed the expected point, and saving left each file byte-identical. Saved copies are `tests/fixtures/plot_points_integrated_cartesian_saved_by_wine.gsp` and `plot_points_integrated_polar_saved_by_wine.gsp`. Captures are `/tmp/sketchpad_plot_points_cartesian_roundtrip_before_save.png` and `/tmp/sketchpad_plot_points_polar_roundtrip_before_save.png`. The native window capture `/tmp/sketchpad_plot_points_desktop.png` shows the Cartesian red point in the winit/egui/wgpu editor.

Selecting a one-column named `speed = 2.5` table and choosing Graph > Plot Points opens the original coordinate dialog with default fields `(1, 1)`. Confirming it creates code 67 with the counted frame packet `[1, 8]` and stored coordinates `[1, 1]`; the table value is not copied into those fields. `tests/fixtures/plot_points_named_speed_original.gsp` is the original-app save. `tests/replay_plot_named_parameter.txt` generates the same point object group, and the file opened and saved byte-identically in Sketchpad 5.

Entering Cartesian `(2.5, -1.25)` and polar `(2.5, 30°)` in the original dialog produced `plot_points_named_speed_custom_cartesian_original.gsp` and `plot_points_named_speed_custom_polar_original.gsp`. Both object groups match port output byte-for-byte. Polar records store angle in radians; the observed degree factor is `0.017453292519943334`. The port dialog accepts both modes and repeated points. `tests/replay_plot_named_coordinates.txt` generates three mixed Cartesian and polar points; Sketchpad 5 opened and saved the generated file byte-identically.

Plot Points also works with a coordinate frame alone. The original `plot_points_frame_only_original.gsp` has code 67 packet `[count=1, frame=6]`: only the coordinate frame is a dependency. The parser now treats this as a plotted point, and the generated group matches the original. `tests/replay_plot_frame_only.txt` creates Cartesian and polar points in that layout; the generated file opened and saved byte-identically in Sketchpad 5.

The first page object is unrelated to the literal point; the constant first packet word is the reference count. `plot_points_prepoint_original.gsp` places a free point before the frame and writes `[count=1, frame=7]`; `plot_points_pretext_original.gsp` does the same with a plain text object first. Both now decode and round-trip, and their generated last object groups match. `tests/replay_plot_mixed_layout.txt` puts text and an extra point before the frame, then plots a custom coordinate; Sketchpad 5 opened and saved it byte-identically. The table selection is not required for Plot Points in these observed cases.

`tests/replay_plot_parameter_no_table.txt` creates a named parameter and frame without tabulating the parameter, then plots `(2.5, -1.25)`. Sketchpad 5 opened and saved that generated file byte-identically.


Checkpoint42 corrects the earlier misinterpretation of the packet count as a
source ID. `Kind::PlottedPoint` now stores just its frame, mode and literal
coordinates. The 790 audited original groups (789 Cartesian, one Polar) all
have one counted frame reference. Header high words control visibility and
presentation; they are independent of reference grammar. Labels, fonts and
empty232e suffixes are supported with bounded validation. Unrelated first-object
deletion no longer cascades to the point, and plotting does not require that
object. Existing Native-verified output bytes stay unchanged. See
[coordinate dependency evidence](coordinate_units_analysis.md).
