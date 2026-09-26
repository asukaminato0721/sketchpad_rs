# Slope of constructed and transformed straight objects

A slope measurement uses code 65 and two parents: the straight object and its
coordinate system. The original native format and group writer are unchanged.
The port previously restricted creation, import, and save validation to bare
segments, rays, and lines, although `slope_value` already resolved its host using
the shared straight-object evaluator. Those checks now accept supported straight
objects, including parallel/perpendicular/bisector constructions, transformed
segments, and coordinate axes. Slope uses the coordinate frame's horizontal and
vertical unit scales. Vertical or degenerate geometry gives no finite value;
restoring its direction restores downstream formulas and iteration.

Regression tests cover parallel/perpendicular lines, fixed and calculated-angle
rotations, signed dilation, reflection, and polar translation. Tests also cover
precision, import/save, copy/remapping, dependent deletion, and calculated live
iteration depth before and after a source edit or a vertical-line transition.
The desktop command accepts constructed and transformed straight hosts while
rejecting a free point.

`tests/replay_slope_hosts.txt` generates four native cross-open probes:

- Parallel and perpendicular slopes are 0.50 and −2.00. `abs(slope)*4` supplies
  depth 2; moving the source endpoint changes them to 1.00 and −1.00 and depth 4.
- A numeric-angle rotated segment has slope 1.5146 at 30°. Its calculated
  depth `abs(slope)*2` truncates to three steps. Changing the angle to −60° gives
  slope −0.6603 and one step.

The line geometry and transformation records come from previously verified
native groups. All measurements retain the code-65 parent format and saved
precision; calculation inputs use full numeric values.

All four generated files opened and saved byte-identically in native Sketchpad 5.
Native screenshots show the expected slopes, calculated depths, and two/four or
three/one repeated points. Rotated-segment probe captions are spaced to show the
slope, degree source, and calculation together. All four port fixtures verify
without partial view, and the complete test suite passes 579 tests.
