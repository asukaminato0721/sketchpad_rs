# Display animation

The editor animates selected parameters, free points, and points constrained to
segments, rays, lines, circles, and arcs. Parameters move over their stored
domain and update dependent tables. Segment and arc parameters reverse at their ends; circle
angles wrap. Free points wander within the visible canvas with changes of
direction at random intervals. The editor advances existing point kinds, so
dependent constructions and traces update through the normal document
evaluator. One undo checkpoint covers a motion session. The floating Motion
Controller pauses, resumes, stops, and adjusts speed; the Display menu also
has speed and stop commands.

## Indirect construction animation

Native Sketchpad 5's Display > Animate Objects moves both source points when a segment or circle is selected. The saved `animation_indirect_segment_native.gsp` and `animation_indirect_circle_native.gsp` contain changed positions for both free points. It also moves a segment whose endpoints are hidden: `animation_indirect_segment_hidden_native.gsp` preserves their hidden state and changed coordinates. All three import without partial view and round-trip exactly. The port resolves selected constructions to movable source points and keeps hidden sources hidden while they animate. Directly selected points on paths still animate on their paths.

Selecting a length measurement or parameter table in native Sketchpad leaves Animate Objects unavailable; both menu probes left their files unchanged. The port excludes measurements and tables from this indirect source search.

An Xvfb desktop probe opened
`tests/fixtures/locus_polar_polygon_target_original.gsp`, selected its point
on a segment, and used Display > Animate Objects. Captures at
`/tmp/sketchpad_animation_gui_run0.png` and
`/tmp/sketchpad_animation_gui_run1.png` show the point and dependent polygon
interior moving. Captures under `/tmp/sketchpad_animation_controls_*.png` show
pause holding the point still, resume moving it, and stop keeping its position.
The controller initially covered the Tools panel; its default position is now
inside the canvas.

`tests/fixtures/animation_segment_locus_saved_by_port.gsp` was saved from the
desktop with Ctrl+S after animation stopped. It has a changed segment
parameter, imports without partial view, and round-trips exactly in Rust.
Sketchpad 5 opened this file with the updated point and dependent polygon,
then saved it byte-identically through Wine. The keyboard shortcut originally
failed when keyboard focus was in the controller: reading the final modifier
state missed a key press whose modifier had already been released. Save,
Save As, and the existing Copy/Cut/Paste shortcuts now consume key events with
their own modifiers.

`tests/fixtures/animation_free_point_base.gsp` has one free point at
(200, 150). Selecting it in Sketchpad 5 and choosing Display > Animate Objects
opens the native Motion Controller and moves the point. A save during motion is
`tests/fixtures/animation_free_point_native_saved.gsp`; its point is at
(172.759, 111.861), and it imports without partial view. The capture at
`/tmp/sketchpad_animation_free_point_animate_after_menu.png` shows the native
controller. `dev/wine_free_animation_probe.sh` captured eight successive
positions of one native run at roughly 0.4-second intervals: screen centers
(280, 210), (258, 181), (232, 130), (272, 112), (268, 106), (258, 159),
(278, 207), and (306, 249). Those samples show changing directions, so the
port uses bounded wandering for its free-point mode. The exact native random
trajectory, speed distribution, and animation-button records remain open.

`tests/fixtures/animation_free_point_saved_by_port.gsp` was saved from the
desktop after stopping a free-point animation. Its point moved to
(169.694, 146.605); Rust reopens it without partial view, and Sketchpad 5
opened and resaved it byte-identically through Wine. The Xvfb captures at
`/tmp/sketchpad_free_port_animated_t0.png` and
`/tmp/sketchpad_free_port_animated_t1.png` show the motion.

## Ray and line points

Sketchpad 5's Display > Animate Objects starts motion for a selected point on a ray or line and opens its Motion Controller. Native saves after animation changed the ray parameter from 0.3 to 0.02944863557118989 and the line parameter from 0.3 to -0.029448635571189867. The captures `/tmp/sketchpad_ray_animate_native_after_menu.png` and `/tmp/sketchpad_line_animate_native_after_menu.png` show the controller. The port animates these points along the visible portion of the straight path, reverses at its ends, and updates dependents. The exact native speed and reversal position remain unmeasured.

## Parameters

Sketchpad 5's Display > Animate Objects opens its Motion Controller for a selected parameter. `parameter_animated_native_saved.gsp` contains a value of 1.957 after motion and imports without partial view. Its decimal record has three fractional digits, which the parser accepts. Native saves while edited Animation buttons run can contain up to 15 fractional digits; those also import and round-trip.

Native's Parameter Properties panel establishes five independent controls: continuous/discrete change, a rate numerator, a rate interval in seconds, animation-domain bounds, and a keyboard `+`/`−` adjustment. A controlled Native edit from the default parameter to discrete, 2 units per 4 seconds, domain −3 to 7, and keyboard adjustment 0.25 changed the `0907` payload's mode word by `0x04` and stored the five floating-point values at offsets 52, 60, 68, 76, and 84. Unit-valued fields use the same native unit scale as the parameter; seconds do not. The port preserves unrelated mode bits, validates finite positive rates and adjustment plus a strictly increasing domain, and writes all settings atomically with the value. `/tmp/parameter_port.gsp`, produced through the public Rust document API with those settings, opened and saved byte-identically in Sketchpad 5 (SHA-256 `da204ae47b74fc38ebd7ab032025341aff9c6bc6feb41610525a3e519a9db0dc`). Direct Display animation now uses the stored continuous or discrete timing, and keyboard `+`/`−` uses the stored adjustment without enforcing the animation domain.

The adjacent Value Properties precision choice is the `0907` payload word at offset 28. The writer synchronizes the keyboard adjustment when it still has the old precision's Native default and updates the saved caption extent. A public-API file combining thousandths precision with the custom animation settings opened and saved byte-identically in Sketchpad 5 (SHA-256 `285b9fd425bba7d5f0b128051a97e11e2e3d86ba94d1af862d2c8d9b38d9835e`).

The same panel's display word at offset 26 uses mode 1 for value only and mode 3 for the current parameter name. Changing only this established word is sufficient; Native accepts the existing caption extent. Rust rendering now follows the mode without changing the parameter's formula identity. `/tmp/parameter_value_only_port.gsp` combined value-only display, thousandths precision, and the custom animation settings; Native displayed `1.000` and saved the file byte-identically (SHA-256 `6d05779fc908f01c99962d35313cb55abe745e0e953ae2c257417b00ab7dcfc7`). The inline-edit-box presentation flag remains separate work because Native recomputes more of its caption layout when toggling it.
