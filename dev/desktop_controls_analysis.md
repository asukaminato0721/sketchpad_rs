# Toolbox and mapped iteration controls

Display > Hide Toolbox hides the five tool buttons and the text-placement field.
Show Toolbox restores them. Page navigation and selected-object controls remain
available. The current tool stays active; this is an application setting, so it
does not mark the sketch dirty. Floating toolbox layout and preference persistence
remain pending.

Selecting a supported mapped iteration or one of its carried point/segment
bindings exposes “Show only final iteration” in the sidebar. The control uses
`set_mapped_iteration_final_only`, including its atomic save validation and undo
checkpoint. The native property is documented in
[mapped iteration image levels](mapped_iteration_image_levels_analysis.md).

The desktop was run under Xvfb with Mesa lavapipe using a disposable copy of
`mapped_midpoint_triangle_native_verified.gsp`. Mouse clicks hid the toolbox,
selected a carried point image, and enabled the final-level control. The canvas
immediately removed intermediate triangles and retained the last triangle.
Ctrl+S produced a 3,144-byte file differing from the original at exactly byte
2708: the native image-level flag changed from zero to one. Native Sketchpad 5
opened and saved that actual desktop output byte-identically; the result is
`tests/fixtures/mapped_midpoint_desktop_final_native_verified.gsp`.

The winit event loop now schedules egui's requested repaint delay alongside the
next animation frame. This handles immediate follow-up UI frames and delayed
repaints while preserving idle waiting when neither UI nor animation needs a
frame. The toolbox and iteration control clicks above were checked on this build.
