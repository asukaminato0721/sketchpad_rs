# Constrained points on fixed rotated rays and lines

Adding the explicit rotated ray/line kinds makes the existing straight-path
point API recognize them through their preserved LineMode. Independent native
checks cover the resulting code-15 dependent objects, rather than inferring
support solely from the transformed parent's geometry.

Both probes rotate a primitive path from (170,190) toward (210,190) around
(130,150) by 45 degrees. The ray probe places a point at t=0.5, approximately
(200.710678119,135.857864376). The line probe deliberately uses t=-0.5,
approximately (172.426406871,164.142135624), behind the defining origin. The
constraint payloads retain subtype 13 for the ray and 12 for the line; the
parent reference targets the code-27 rotated object.

Both ordinary Rust writer files opened and saved byte-identically in native
Sketchpad at 2,224 bytes. Separate screenshots show the point on the correct
one-sided ray or two-sided line, including the line point behind its origin.
Fixtures are `rotated_ray_boundary_point_native_verified.gsp` and
`rotated_line_boundary_point_native_verified.gsp`. Capture prefixes are
`/tmp/rotated_{ray,line}_boundary_point`; generation is reproducible with
`gsp_probe replay tests/rotated_straight_dependents_replay.txt`.

No decoder or geometry broadening was needed for these dependents: the existing
point-on-path branches use straight_kind_mode. The regression asserts the exact
native writer files and fractions, moves the source endpoint to check live
interpolation, verifies save/reopen and undo, copies the dependency closure with
a (20,20) displacement, and deletes the original host to check cascading deletion.
Those subsequent API cases are save/reopen regressions; the native proofs in
this slice cover the two initial independently generated files.
