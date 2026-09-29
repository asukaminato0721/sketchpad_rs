# Coordinate-axis rotations

Read-only corpus inspection found two independently ready Line-image roots whose
source is a native `GraphAxis(_, _, 1)` rather than a primitive or constructed
Line:

- code 27, class 2/subtype 12, references `[axis, center]`, with the ordinary
  36-byte fixed-angle constraint;
- code 29, class 2/subtype 12, references `[axis, center, numeric angle]`, with
  no constraint.

The code-29 example is object 27 in
`热研系列/滚动系列/正n边形的滚动.gsp`; object 28 rotates that result again.
The fixed example occurs in `第9题 正方形顶点绕圈.gsp`. Both sources are the
ordinary form-1 x-axis. No corpus evidence in this pass establishes axis
translation, dilation, reflection, point-angle rotation, y-axis rotation, or
the other axis families.

The shared transform decoder therefore treats the form-1 GraphAxis as a Line
source only for code 27 and code 29. Evaluation and save validation apply the
same fixed/numeric-rotation boundary. The editor and document API can author
both forms, while the generic straight-transform predicate remains unchanged
for every other operation.

An isolated port writer created a default coordinate system, then a 30-degree
fixed image and a negative-45-degree calculation-driven image of its x-axis.
The resulting 10-object file is 2,750 bytes and reopens as two live Line
transforms. Sketchpad 5 opened the file under Xvfb/Wine and displayed both full
infinite lines at the expected angles over the coordinate grid.

The compiled scan of all 1,599 files restores 23 drawing slots: the two direct
axis-source roots and 21 descendants. Unsupported slots fall 2,853→2,830
(2,232 drawing plus 598 embedded prototypes), all-kind-complete documents rise
1,270→1,272, drawing-complete documents rise 1,300→1,302, and partial views
fall 867→865. Every original remains byte-exact and no parse fails.
