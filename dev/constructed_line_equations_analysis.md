# Equations of constructed infinite lines and coordinate axes

Native Measure → Equation accepts parallel and perpendicular lines and both
coordinate axes. Each created measurement uses code 50 with parents
`[infinite line, coordinate system]`, the existing 0x08fc rich equation text, and
its text-style reference. Native creation fixtures are
`parallel_line_equation_native.gsp`, `perpendicular_line_equation_native.gsp`,
`x_axis_line_equation_native.gsp`, and `y_axis_line_equation_native.gsp`.

The port command, importer, replay, and save validation now use a shared
infinite-line classification. Ordinary segments and rays remain ineligible.
Angle bisectors are rays in the native geometry, so they are also ineligible;
the shared constrained-point classifier was corrected to preserve their ray
constraint subtype. A regression test creates and reopens a point on a bisector.

Native equation creation adds optional 0x07d5/0x07d8 label/style records to
constructed straight objects. Their payload and style-index validation now
matches the existing bounded circle/polygon label validation, and original
source groups preserve these records. General curve-label editing remains
outside this host extension.

## Numeric evidence

The reference line runs from (100,180) to (180,140), and each constructed line
passes through (60,180). With origin (199,125) and unit 37.79527559055118 pixels:

- The parallel line has slope 0.5 and intercept 0.3836458333333333.
- The perpendicular line has slope −2 and intercept −8.810625.
- Moving the reference endpoint to (100,100) makes the parallel line vertical.
- Moving it to (180,180) makes the perpendicular line vertical.
- Both vertical equations have x = −3.6777083333333333, displayed as −3.68.
- The coordinate axes produce y = 0.00 and x = 0.00. Their equations remain zero
  after moving the shared origin.

Horizontal-line captions omit the zero coefficient of x, and exact negative zero
is displayed as positive zero, matching the native axis captions. Tests cover
native-file preservation, sloped/vertical updates, frame-relative axis equations,
clipboard dependency remapping, dependent deletion, desktop eligibility, and ray
bisector constraints.

`tests/replay_constructed_line_equations.txt` generates the two creation bases and
six cross-open probes (parallel/perpendicular before and after vertical edits,
plus both axes). To reproduce native creation, use the corresponding base with
`dev/wine_roundtrip.sh` and `dev/constructed_line_equations_wine_setup.sh`, with a
capture prefix containing `parallel` or `perpendicular`. For axes, create a bare
frame at (199,125) and use `dev/axis_line_equations_wine_setup.sh` with a capture
prefix containing `x_axis` or `y_axis`.

All six generated files opened and saved byte-identically in native Sketchpad 5.
Native captions show slopes/intercepts 0.50/+0.38 and −2.00/−8.81, both vertical
results x=−3.68, and axis equations y=0.00/x=0.00. SVG captions agree with the
vertical and horizontal-axis results. Four native creation fixtures preserve
their bytes and import fully. The complete suite passes 589 tests (486 library,
98 desktop, and five integration).
