# Native straight parameters and line-circle branches

The registered Square tool's native drawing exposed a solver error: code 11 intersection object 8 is (200,120), and its dependent object 10 is (260,120). Rust previously put them at y=240. The perpendicular direction was not the cause.

Six independent 47-object controls are retained as `tests/fixtures/straight_direction_{probe,quarter,three_quarter,offbase,reversed,clipped}_native_verified.gsp`. Each was opened and saved in Sketchpad with successful exit and byte-identical output (4936 bytes). Each contains Segment, Line, Ray, Parallel and Perpendicular controls, with native code 12 colored red, code 11 blue, and code 15 at a specified path fraction colored green. Native captures are retained beside the documents as `tests/fixtures/straight_direction_*_native_verified.png`.

For a horizontal source ordered left to right, code 12 (model variant 0) is the right circle crossing, and code 11 (variant 1) the left crossing. Reversing source endpoints swaps these positions for every mode. Therefore variant 0 selects the larger line parameter and variant 1 the smaller. The chosen root is tested against the host extent afterward. A segment/ray starting at the circle center shows the positive red crossing and suppresses the negative blue crossing; the blue branch must not fall back to the remaining red root.

Parallel and Perpendicular path parameters have anchors centered on their construction point, with half the source vector on each side. On a 200-pixel source, fraction .5 is the construction point, .25 is 50 pixels backward and .75 50 pixels forward. A separate control moves the construction point 40 pixels right and 30 down without moving the defining line; .5 follows that moved point. These results establish the anchor location and magnitude independently.

Perpendicular retains the existing screen-clockwise direction `(-dy,+dx)`. A horizontal 200-pixel source gives green fractions .25/.75 at 50 pixels above/below the perpendicular construction point. The intersection branch correction produces the native Square orientation without reversing the perpendicular direction.

Primitive straight modes, circle-circle conventions, native record layouts and transform capability guards remain unchanged. The regression checks the six native controls, source/center edits, exact saved bytes, and the registered Square's independent native coordinates.

The centered anchors also exposed cancellation in slope measurements when two separately converted endpoints were subtracted. Slope now transforms the screen direction vector through the same inverse coordinate basis, preserving skew and unequal axis scales, and normalizes horizontal zero for numeric captions. The live calculated-depth regression retains its expected two iterations.
