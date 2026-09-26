# Native numeric presentation, Cartesian points, and parameter traces

`../gsp-rs/tests/fixtures/gsp/static/cood-trace.gsp` and its paired `.htm`
provide the scalar parameter t[1]=1, code-48 calculation t[1]+1, a Cartesian
coordinate point, and a code-97 parameter trace. The copied source fixture is
`tests/fixtures/numeric_cood_trace_native.gsp`.

Parameter 1 differs from the established default template in its label flags
(low label word 7 instead of 6), as well as already-variable persisted cache
fields. Rejecting this presentation prevented calculation 2 from importing.
The numeric parser now validates the numeric representation, token count,
precision, positive finite animation step, and finite ordered animation domain
separately from header color/visibility, LabelAux contents, and font index.
Imported custom animation steps survive value edits. Scalar parameters retain
subtype 79. Unit-bearing native parameters (subtypes 80 length and 81 degrees)
remain a separate prerequisite.

`../gsp-rs/tests/fixtures/圆的形成.gsp` supplies parameter 5 (t[2]=5, precision 0)
and named code-48 calculation 6 (reference 5, source token 0x6000, label t[1],
font index 1). These are directly exercised by `tests/numeric_native_import.rs`.
Optional numeric labels retain their opaque suffix and presentation fields.
Calculations validate their 0907 prefix/token representation independently of
labels/fonts and infer/check units from their sources. Imported expressions
can contain up to 256 tokens; newly authored expressions retain the existing
32-token bound. Unknown descriptor modes and unknown expression tokens remain
unsupported. Source-free scalar calculations likewise preserve optional labels,
font indices, colors, and native caption boxes.

Native coordinate point 9 is code 69, point subtype 56, and has references
[1,2,8] (x value, y value, frame), with no scalar literal payload. This is distinct
from the existing stored-coordinate Plot Points objects. NumericCoordinatePoint
reads both live scalar sources and maps them through the graph frame. Its
reference order is also documented in sibling `htm_plot_xy_args`: JavaSketchpad
exports PlotXY(y,frame,x). Geometry, dependency remapping, clipboard insertion,
source preservation, and save/reopen cover the new point.

Native trace 10 is code 97, stroke subtype 42, with references [9,1,2,9]. These
are target, parameter driver, dependent intermediate calculations, target. Its
40-byte 0902 descriptor has min 0, max 1, sample count 500, mode 0x00030004,
category word 1 at offset 32, and zero tail at offset 36. This descriptor is
separate from the code-35 geometric locus modes and function plot modes.
ParameterLocus verifies its references against the dependent ancestry and
samples the target over the saved range. The current driver value does not
change that range. Sampling uses an internal floating-point value on a cloned
parameter, preserving the actual driver precision instead of rewriting or
rounding the stored decimal record. Undefined samples break the curve; adjacent
valid samples alone are joined. Construct Locus also accepts a parameter with a
dependent point; the default range is 0..1 with 500 samples.

Tests prove full cood-trace import without partial-view restrictions, live
parameter/calculation/Cartesian point updates, parameter trace endpoints and
fixed range, singularity gaps, numeric point clipboard remapping, native style
preservation, malformed token/source/unit/label rejection, and save/reopen.
The original full sketch was edited to t[1]=3.5: Sketchpad 5 displayed 3.50 and
4.50, opened/saved byte-identically. A newly added parameter locus also opened
and saved byte-identically. The native-saved copies are
`numeric_cood_trace_edited_native_verified.gsp` and
`numeric_cood_trace_added_locus_native_verified.gsp` in `tests/fixtures`.
`tests/replay_numeric_native_import.txt` replays source edits, locus creation,
frame movement, and clipboard copying for further native checks.

Native cross-open commands source `/tmp/sketchpad_verify_env.sh`, use
`SKETCHPAD_WINEPREFIX=/home/w/.cache/sketchpad_rs_wine`, and run
`dev/wine_roundtrip.sh` on disposable copies. This setup supplies the matching
Wine wow64 binary, Xvfb, xdotool, and ImageMagick.

## Audited next numerical prerequisites

The standalone `dev/numeric_import_audit.rs`, linked to the current library,
classifies unsupported code-48 groups independently of their parents. Its
2026-09-26 audit found 19,443 with unavailable numeric parent units, 3,313 with
unsupported record layouts, 886 with unexpected tokens, 825 with unclosed
function arguments, 732 with unsupported headers, 392 with unsupported tokens,
and 157 with incompatible inferred units. This is a prerequisite report, not a
claim that these groups now work. The first long function-argument counterexample
is Ningde2 object 29: `sgn((x-2)*(3-x))`. A function already has an implicit open;
its following token 11 can start a genuine grouped subexpression. Parsing it as
a special function opener truncates the argument before the multiplication.
Other concrete native tokens are radian suffix 0x0100 (圆柱侧面展开图 object28),
and referenced function call token 0x7000 (湖南衡阳 object23).

Native `10福建宁德26题2(蚂蚁).gsp` object 28 is code69, header class0x00040045,
refs[21,20,27]. Its x source is a point distance and y source is a polygon area.
A native screenshot of the actual document proves x=2.51cm and y=2.22cm² are
plotted at coordinate (2.51,2.22), and the Color calculation displays0.33. Numeric
coordinate quantities therefore use measured numeric magnitudes; scalar-only
source validation would reject valid native points. The original has no paired
`.htm`. Other prerequisites for this document remain calculation-driven polygon
path points17/18 and trace/link object families. The long Color syntax and
length-valued literal thresholds now work in an isolated native-verified sketch.

Native full cood clipboard copying (21 objects) and imported range editing
(min−1,max2,samples17) also open/save byte-identically. Their retained native
copies are `numeric_cood_trace_copied_native_verified.gsp` and
`numeric_cood_trace_range_native_verified.gsp`. The range editor preserves the
original native descriptor's mode and cache suffix. Safe deletion cascades
remove a driver and its dependent calculation/point/trace; deleting before a
surviving independent plot retains the existing atomic deletion guard.

Native numeric coordinate point labels follow the reference record, as shown by
`Samples/未分类档/涡旋星系.gsp` (code69 refs[23,24,6], label B). Inserting
a label before the references causes native Sketchpad to reject the file.
Both imported label insertion and appended clipboard point encoding now use
the native order. A 19-object cood sketch with a labeled, large blue numeric
point and its copied dependencies opens and saves byte-identically; its native
snapshot is `numeric_labeled_coordinate_copied_native_verified.gsp`. The label,
point style, color, remapped references, and live positions have regression
coverage. Header flag4 remains an imported presentation property; authored
numeric points with length/area parents were verified using flag0.

Native Ningde2 object17 (code95) carries a scalar calculation source16, a
path15, label N, and caption font0. The isolated path regression rebinds the
actual native group to cood's scalar calculation and X axis, verifying live
updates, imported presentation preservation, authored creation, and copying.
The original path15 is a polygon. Its numeric perimeter parameter semantics
require a separate native probe before extending the supported path families.

A focused length threshold probe preserves the source quantity and its native
scale: AB=2.51cm and the actual Color expression produces0.33. Native Sketchpad
opens/saves its 14-object file byte-identically, retained as
`numeric_measured_literal_threshold_native_verified.gsp`. Bare decimal literals
can adopt Length units in addition/subtraction; scalar source variables and
computed scalar expressions remain separate quantities. This is deliberately
limited to the proven centimeter case: degree and area literal promotion await
native evidence. The original source-free `30°+2` rejection remains intact.

Table value setters also preserve imported visibility and accepted opaque
padding. The observed two-row default table retains its native `t` padding
marker. Updated groups are parsed before mutation; if changed text leaves too
little padding for a nonzero imported marker the edit is rejected atomically.
