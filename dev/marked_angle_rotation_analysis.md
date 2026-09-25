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
