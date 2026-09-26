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
subtype 79. Native subtype 80 length and subtype 81 angle parameters now
retain their proven typed value, display, animation, and calculation semantics;
the native probes and remaining unit-specific limits are documented below.

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
dependent point; scalar defaults remain 0..1 with 500 samples, while angle
parameters use their saved animation domain in degrees.

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
The original path15 is a polygon. Its normalized perimeter semantics, wrapped negative/oversized values, and
live scalar calculation sources are now native-verified and implemented;
see `dev/numeric_polygon_path_analysis.md`.

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

Updated corpus audit after the function-argument and length-threshold fixes:
18,905 unsupported calculations lack numeric parent units;3,313 have unsupported
record layouts;888 unexpected tokens;732 unsupported headers;394 unsupported
tokens;174 unclosed ordinary groups;156 invalid source tokens;117 incompatible
units;79 unclosed function arguments;49 invalid reference counts. Function
argument failures fell from825 to79. The standalone audit output is
`/tmp/numeric_import_audit_updated.txt`.

The raw sibling corpus contains 313 native subtype 80 Length parameters and
123 subtype 81 angle parameters, alongside 3,435 subtype 79 scalar parameters.
`fixtures/未实现的系统功能/parameter.gsp` opens/saves unchanged natively and
displays t1=1.00°, t2=1.00cm, t3=1.00. Its numeric words are respectively
[1,0x0101], [1,0x0201], [1]. Degree animation step/domain use native radians
(step 0.08726646259971667 = 5°, domain 0..6.2831853071796); Length step/domain
use native pixels (step 0.3779527559055118 = 0.01cm, domain 0..3779.5275590551178).
The degree persisted cache is 0.78539816339745 despite its displayed 1.00° value:
cache fields must remain opaque and cannot supply current parameter values.
Bounded typed support now separates value words, display units, animation
conversion, header validation, and unit inference. Actual setters preserve the
opaque cache fields. A 13-object edited document and its 22-object clipboard
result open/save byte-identically in native Sketchpad (3,476 and 4,526 bytes).
Native numeric XY points use the displayed centimeter/degree magnitudes:
t1=0.50° plots y=0.5, rather than the internally stored radian magnitude.
Typed parameters are rejected from scalar-only path plotting, iteration depth,
length parameter traces, and current scalar table schemas. Angle parameter
traces have separate native unit descriptors and are supported below. The corresponding
regressions cover live calculations, units, precision, domains, and copies.

Separate native probes establish that suffix 0x0100 stores radians while
displaying degrees: input words [1,0x0100] give 57.30°, sin=.84, and
t1×2=114.59°. The public value therefore converts radians to degrees, and
setters apply the inverse conversion while preserving suffix 0x0100.
Editing this parameter through the public API to 3.5 gives native captions
3.50°, sin=.06, and t1×2=7.00°. Its dependent numeric XY point plots y=3.5.
The edited file opens/saves byte-identically (3,508 bytes).
Suffix 0x0102 is another decimal-degree input representation, not squared
degrees: 4.5 displays 4.50°, sin=.08, sqrt=2.12, and its numeric XY point has
y=4.5. The setter retains 0x0102; editing to 3.5 opens/saves byte-identically
(3,480 bytes). Both variants retain the actual native radian animation
step/domain and opaque cached values. These meanings come from live native
calculation/display probes, rather than from suffix bits or persisted caches.

The raw corpus also contains 82 code 97 curves driven by subtype 81 angles,
with mode 0x30004 and range endpoints such as 2π and π/2 stored in radians.
No subtype 80 Length driver occurs in that corpus sample. Four other curves
use code 0 subtype 51 from `向忠作品/12T.gsp` object 6: this is a named √3
expression with a different prefix and uninterpretable animation fields,
rather than a decimal parameter or a newly inferred unit. It remains unsupported.
Native angle curve descriptors use type tag 0x00010000 at offset 32, whereas
scalar curves use tag 1. A source-compatible rotation target and raw range
0..π/2 produce a visible quarter-circle with the angle tag; the scalar tag
does not. The retained 17-object native fixture opens/saves byte-identically
(3,828 bytes). Angle parameter curves now keep their exact native radian
endpoints in Kind and descriptors, while plot_range, set_plot_range, construction,
and the range editor expose degrees. Sampling converts the native endpoint
values to the driver's public degree values without mutating its stored words.
No-op range confirmation preserves the exact imported bytes; range edits
preserve the descriptor's mode, cache bytes, angle tag, and zero tail.
Native tags that disagree with driver units remain unsupported. Length curves
retain their explicit unsupported guard until their actual native descriptor
and sampling rules are established. The public API's edited range, new authored
angle curve, and clipboard copy also open/save byte-identically as a 23-object
native document (4,512 bytes). Native captures show the 270° edited curve,
90° authored curve, and shifted copy. The retained fixture is
`numeric_angle_parameter_locus_edited_copied_native_verified.gsp`.

Some native captures initially show an unpainted MDI child after the file title
changes. Cycling Ctrl+F6 twice repaints the loaded document; the positive edited
curve capture is `/tmp/numeric_angle_curve_ctrl_f6_again.png`. An unchanged
saved file alone was not used to establish curve geometry.

The 3,313 remaining calculation record-layout failures are precisely native
empty0x232e records immediately after the font and before the terminator
(2,265 unnamed,1,048 named). Bounded parsing now accepts that sequence only;
nonempty, duplicated, or misplaced attributes remain unsupported and preserve
the input bytes. Actual cube3D object14 is sin(source×360°). Rebinding its
source to cood's t1 proves live math with a dependent native numeric XY point:
t1=1 gives x≈0,y2; editing t1=1.25 gives x1,y2.25. Both files open/save
byte-identically. Native snapshots are retained under
`numeric_empty_calculation_attribute_xy*_native_verified.gsp`.

Caption control probes separate the empty attribute from presentation:
mode1/flags0/font0 renders value-only0.00 at(10,150), retaining0x232e;
mode1/flags2/font0 renders no caption at the same anchor after removing0x232e.
Headerflag2 therefore suppresses the observed mode1 caption independently
of the attribute and font. This behavior is represented separately from
visibility's flag1 bit, preserving the imported flag bytes. Other mode/flag
combinations are not generalized from this probe.

The five apparent subtype 80/81 values without a trailing unit suffix are not
five additional plain typed decimals. Three store scientific expressions:
`学做任意凸四边形的渐开线.gsp` object 15 and the Meishan exam object 60
store 1cm×10^-5; `物理教学积件.gsp` object 6137 stores
4.46583259°×10^-5. Their unit token precedes multiplication rather than
ending the group. Bounded support now accepts precisely the observed typed
literal followed by ×10^-5, with centimeter 0x0201 or degree 0x0101 units.
Other exponents, operators, unit/header conflicts, and radian/alternate-degree
scientific variants remain unsupported rather than guessed.

Native probes precede this parser support. The Length value multiplied by
100000 gives 1.00cm and a numeric XY point at x=1; the degree value gives 4.47°
and a numeric XY point at y=4.46583259. Their ordinary two-decimal captions
round the tiny original values to 0.00cm and 0.00°. Setters preserve the unit,
multiplication, power, negative exponent, and opaque caches; public values stay
centimeters/degrees. Decimal digit shifting encodes public 0.000025 as mantissa
2.5 rather than introducing a float-division artifact. Changed value precision
and standard animation steps follow the established setter policy, while
custom steps and domains remain untouched. Unchanged-value edits preserve the
original literal bytes; values that cannot fit the retained layout fail atomically.

Editing the Length/degree mantissas to 2.5/5.5 gives native five-decimal captions
0.00003cm/0.00006°, derived values 2.50cm/5.50°, and plotted x=2.5/y=5.5.
The API-generated 24-object clipboard documents open/save byte-identically
(4,810 and 4,814 bytes), including the shifted copied coordinate systems and
copied live calculations. The retained fixtures are
`numeric_scientific_{length,degree}_parameter_edited_copied_native_verified.gsp`.
The integration regression also exercises all three original groups, exact
untouched/edited/copy bytes, public unit guards, cached-field preservation,
live math/positions, no-op edits, malformed layouts, and atomic oversized edits.

The two zero-word groups (Yanghui triangle object 123 and rolling quadrilateral
object 788) have garbage or invalid animation metadata and need separate
structural evidence. They remain unsupported, with original raw fixtures and
explicit rejection regressions. No value or unit is inferred from their caches.

A complete typed-parameter re-audit after the scientific slice scans all 1,599
sibling `.gsp` files rather than only successfully imported objects. There are
313 subtype 80 groups and 123 subtype 81 groups. The current parser imports
307 Length and 118 Degrees groups, with no parser-to-decoder gaps. The only
scientific literals are the three already verified above; this corpus contains
no additional scientific exponent or scientific unit-token variants.
`dev/typed_parameter_audit.rs` reproduces both the literal-layout counts and
separate parser/import outcomes against the current library.

The eleven remaining typed groups comprise six without a label record, one
with an empty 0x232e after its bounding box, one with caption mode 0, one with
words `[0x000d, 0x0100]`, and the two invalid zero groups. The unlabeled and
mode/attribute cases are presentation prerequisites rather than new exponent
semantics. The 0x000d radian literal received a separate native constant-token probe,
described below. The audit itself did not broaden any production guard or
reinterpret the invalid zero metadata.

The exact native words `[0x000d, 0x0100]` in `12 函数工具⑵-分段函数.gsp`
object 125 encode π radians. An isolated 13-object document visibly displays
180.00°, sin(parameter)=0.00, and parameter×2=360.00°. Its dependent numeric
XY point lies at x=2.51, y≈0. The native program saves the isolated file
byte-identically (3,472 bytes); the positive capture is
`/tmp/numeric_pi_after_setup.png`. Bounded parsing accepts this constant only
with the radian suffix. It uses the already established native calculator π
constant, exposes degrees, and leaves the unrelated cached value opaque.

A no-op setter retains the exact π token. Changing the public value to 3.5°
uses the existing verified decimal-radian encoding, retaining the radian unit,
animation domain, custom step, and caches. Clipboard copying the original
sine point preserves its π parameter and live calculation independently of
the subsequent original edit. A second copy of the edited sine point retains
3.5°, computes sin≈0.06105, and plots against its copied frame. This 33-object
API-generated document opens/saves byte-identically (5,996 bytes). Both
original and copied native literal forms are retained in
`numeric_pi_radian_parameter_edited_copied_native_verified.gsp`; the raw
source and first isolated document are retained separately.
A separate native viewport-scroll probe makes the copied coordinate origins
and both copied sine points visible below the original horizontal axis. Its
positive capture is `/tmp/numeric_pi_geometry_after_setup.png`; the unscrolled
capture `/tmp/numeric_pi_api_after_setup.png` confirms the edited caption and
math. The viewport probe uses a separate temporary copy, preserving the exact
unscrolled fixture used for the API byte comparison. After this bounded π
support, the same corpus audit imports 426/436 typed groups (307 Length,
119 Degrees), leaving the ten presentation/invalid-zero groups above.

The subsequent presentation probes establish the remaining supported typed
forms independently of their numeric words. Native groups without 0x07d5
use `[07d0,0907,07d8,08a3,07d7]`; labeled groups can carry one exact-empty
0x232e immediately after 0x08a3 and before 0x07d7. The parser accepts precisely
these observed sequences and preserves label absence and the empty attribute
through numeric edits and anchor changes. Nonempty, misplaced, duplicated,
and unobserved optional-record combinations remain guarded. Bbox/font access follows record
kinds instead of assuming a label is always present. All six original typed
unlabeled groups, the actual Length attribute group and mode-zero group,
and authentic scalar controls have retained raw fixtures and regressions.

An absent label is not a value-only caption. Native display names come from
the page's 6-byte 0x0960 record: leading u16=0, then the next u32 parameter
index. The baseline cursor 4 generates t₄ for one unnamed parameter; making
all three unnamed produces t₄,t₅,t₆ in object order. Renaming the explicit
t₃ to a custom name or removing that unused parameter does not change the
unnamed parameter's t₄. Changing only the cursor to 7 produces t₇, including
in dependent formulas. The retained counter controls are
`numeric_parameter_autoname_{counter_seven,all_unnamed,two_parameters}_native_verified.gsp`.
No generated label is inserted into the source group. Document-level name
resolution applies the page cursor and preceding live unnamed groups;
parameter math continues to use references rather than names.

All 29 actual unnamed subtype79–81 groups have valid leading-zero counters:
21 use cursor 1, 4 use cursor 2, 2 use cursor 5, and 2 use cursor 11. A missing, malformed,
duplicated, zero, or overflowing naming counter does not receive a guessed
name: those unnamed constructions remain unsupported/partial. Explicitly
named parameters remain independent of this unrelated metadata. Zero-counter
naming semantics have not been established and are not generalized.

The one actual mode-zero parameter (timer object 15) is an ordinary0.5cm
literal with stored name t₂. Native mode 0 uses its unrounded numeric definition
as the caption name: `0.5cm = 0.50cm`, with dependent formula `(0.5cm)*1`.
Bounded support applies only to labeled, ordinary centimeter literals in the
standard sequence. Its stored name, mode, caches, domain, and step remain
unchanged; other unit, scientific, and optional-record mode-zero combinations
remain unsupported. The renderer and authored calculation input use the
resolved display name, while dependent formula formatting brackets the
literal definition separately from expression evaluation.

The independent native isolates demonstrate Length0.0001cm×10000=1cm,
Degrees35°→sin≈0.57, Scalar0+1=1, Length3.2cm×1=3.2cm,
mode-zero Length0.5cm×1=0.5cm, and attributed Scalar27/10=2.7.
Each includes a positive numeric XY point and an exact-byte native open/save
comparison. The empty attribute does not suppress these captions; its box or
other presentation meaning is not inferred (the scalar control still boxes
its value). Original caption modes, style fields, fonts, and cached bounds
are preserved independently from these established numeric semantics.

All six actual API edit/copy probes retain 24 objects and reopen/save with
identical bytes: unlabeled Length 4,714 bytes, unlabeled Degrees 4,710 bytes,
unlabeled Scalar 4,678 bytes, attributed Length 4,770 bytes, mode-zero Length
4,762 bytes, and attributed Scalar 4,770 bytes. Edited values respectively
produce plotted x=2.5, y=sin(45°), x=3.5, x=2.75, x=2.25, and x=3.25.
Each native capture shows the shifted copied point and its live calculation;
unnamed originals/copies display t₄/t₅, while mode-zero copies retain the
numeric definition caption. The retained files use
`numeric_parameter_presentation_*_edited_copied_native_verified.gsp`.
Positive captures are `/tmp/presentation_*_api_after_setup.png`.

The integration regression compares complete API output with every retained
native fixture, tests live updates and authored squares using the resolved
names (including `(0.5 cm)^2` followed by a source edit), and verifies reopening
preserves copied geometry/name resolution. The complete typed audit now imports
434 of 436 groups: 312 Length and 122 Degrees, with no parser/decoder gaps.
Only the two original zero-word groups with invalid/unestablished metadata
remain rejected. Explicit-name collision and deleted-unnamed ordinal controls
are separate naming checks rather than evidence for interpreting those zero
values or their animation bytes.

The final naming controls establish two edge cases. With explicit stored
`t[4]` and page cursor 4, an unnamed parameter also displays t₄; native does
not skip occupied explicit names. Their values and referenced calculations
remain distinct (Scalar 1 versus Length 0.0001cm), confirming that formulas
bind object IDs. Removing the first unnamed parameter through the API also
removes its dependent calculation and three coordinate points. Native reopening
assigns t₄/t₅ to the two remaining unnamed parameters in their new order,
matching resolution over live groups. The remaining Length calculation still
gives 1cm. Both controls compare byte-identically (3,518/2,868 bytes) and have
positive captures `/tmp/presentation_autoname_{collision,deleted}_after_setup.png`.
Retained fixtures and regressions cover the name collision, dependency deletion,
reference remapping, unchanged cursor, new display names, and live math.
