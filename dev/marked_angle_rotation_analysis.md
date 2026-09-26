# Three-point marked angle and rotation

`tests/transform_marked_angle_replay.txt` creates a rotation center `(130, 150)`,
three angle points `(230, 160)`, `(230, 220)`, `(290, 220)`, and a source
point `(170, 150)`. Sketchpad's Transform > Mark Angle treats the first point
as the initial side, the second as the vertex, and the third as the terminal
side. The marked angle in this sketch is −90°; rotating the source about the
center creates `(130, 190)`.

`dev/marked_angle_wine_probe.sh /tmp/sketchpad_marked_angle_base.gsp`
reproduces the native command sequence. Its result is
`tests/fixtures/angle_rotation_original.gsp`. Saving after marking the center
and angle, before rotating, produces a file byte-identical to the unmarked
base. The marks are session state.

The native rotated point uses header code 28 and a `0x07d2` reference list
`[source, center, angle start, vertex, angle end]`; there is no constraint
payload. The port reads and writes this object and computes its angle from
the three point positions on each evaluation. The editor offers Mark Angle
and a marked-angle option in Rotate.

The port-generated files are
`tests/fixtures/angle_rotation_generated.gsp` and
`tests/fixtures/angle_rotation_moved_generated.gsp`. Sketchpad 5 opened and
re-saved both byte-identically. The moved file changes the terminal angle
point to `(290, 190)`, and Sketchpad renders the rotated point at its new
position. This implementation covers angles marked from three points and
rotation of points; segment and circle extensions are documented in
`dev/rotation_shapes_analysis.md`. Other angle sources and object types remain.

## Direct measured angle values

Transform > Mark Angle now accepts a selected three-point angle or arc-angle measurement directly, in addition to a degree-valued calculation. The numeric angle uses code 29 with references `[source point, center point, numeric angle]`, distinct from the five-reference code-28 three-point mapping. Import, creation, save validation, and the iteration mapping predicate accept any supported degree-valued numeric kind. The numeric measurement remains an external input during iteration, including an arc whose endpoint is the seed.

`tests/replay_measured_angle_rotation.txt` creates a 90° measured-angle segment iteration, edits its angle to 135°, and creates a circle iteration driven by a measured arc angle before and after changing the arc endpoint. The circle has live depth n=3.5, truncated to three steps. All four generated original/edited files open, display their corresponding orbits, and save byte-identically in Sketchpad 5.

`dev/measured_angle_rotation_wine_setup.sh`, used as `SKETCHPAD_WINE_SETUP_SCRIPT` with `dev/wine_roundtrip.sh`, selects the measurement in `measured_angle_rotation_base_port.gsp`, marks it and the center, and creates a native rotation. Its durable result is `tests/fixtures/measured_angle_rotation_native.gsp`. That native rotation's complete 68-byte object group is identical to the port-generated object. Regression tests compare these groups, preserve the native file, check source edits, undefined measurement recovery, dependency deletion, copy/paste, saved fractional depth, and desktop Mark Angle → Rotate → Iterate selection.
