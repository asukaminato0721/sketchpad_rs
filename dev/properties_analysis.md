# Common Properties panels

Sketchpad's [Properties reference](https://keyonline.keypress.com/gsp-reference-center/menus_edit_properties.htm)
lists a common Object panel, Label panels for labelable objects, and Opacity
panels for translucent objects. The current port keeps the common identity,
visibility, parent, and child controls and now adds two bounded editing panels.

## Arrow-tool selection

The common Object panel exposes Native's Arrow Selectable checkbox for every
object. Native control ID 1159 stores the disabled state as an empty `232e`
record immediately before the object's `07d7` terminator; the absence of that
record means selectable. The decoder removes this common presentation record
before handing an object group to its type-specific parser, and the writer adds
or removes it transactionally without changing the object's semantic records.

The Arrow tool omits disabled objects from click selection, drag initiation,
and canvas double-click editing. Other tools and programmatic parent/child
selection remain independent, and action buttons remain clickable. A
port-authored disabled point opened with Native's checkbox clear and saved all
1,844 bytes unchanged in Sketchpad 5, retaining SHA-256
`dbb975633e2bb8be46d14e8b0122fbab54987e1961f92bdd1f75f52de7160056`.

## Geometric labels

For a point, straight object, circle, base arc, or transformed straight, circle,
or arc, Properties exposes its saved label name and visibility. Apply uses the
native `07d5` label transaction,
preserving an original record's opaque suffix and rejecting unsafe byte-parity
changes. The label record's native geometric parameter supplies its live anchor:
the observed segment uses 0.25 along its defining direction and the observed circle uses
315° around its center. Canvas and SVG rendering share this evaluator.

A generated Native control used Ctrl+K on a segment and circle. It produced
visible `j` and `c[1]` labels with parameters 0.25 and 7π/4; the port now decodes
the document without partial view, renders the saved `c[1]` spelling as `c₁`,
and preserves it byte-for-byte. In the other
direction, the port generated a labeled segment and circle. Sketchpad 5 displayed
both and saved all 2,256 bytes unchanged, retaining SHA-256
`6cc98c0bc5fdbdc433a0795561c9d98878516a7850ade8d5cbb5d209a36586c2`.

Native line and ray groups use the same label payload with presentation classes
2 and subtype codes 63/64. The existing hidden-ray fixture now decodes without
partial view, preserves its `k` label and geometric parameter, and can be shown
or hidden together with other selected labels in one undo transaction. A fresh
port-authored ray labeled `r[1]` displayed as `r₁` in Sketchpad 5 and saved all
2,016 bytes unchanged. Its before/after SHA-256 was
`7bb9f2a4c0e504e26a913e23ea419a7d653655439d6f58bc05786da2e2c9e837`.

Native Ctrl+K controls for an arc on a circle, a center-defined arc, and a
three-point arc all store the label after references with normalized parameter
0.25. The first two use offset `(6, -23)`; the three-point arc uses
`(-26, -30)`. Native-created controls now import as full views. In the other
direction, clean port-authored `a[1]` circle-host, center, and three-point arc
writers displayed as `a₁` and saved unchanged in Sketchpad 5. Their SHA-256
values are respectively
`5c5c870e8d06f0665f118e2fd4bccf23d1f02a2decbd1adf5dbf94f1d24cec3b`,
`2a22199e6cf33ae8b43ebc90ddada37a77742c28149ed6ae4d6eab1da57482f0`, and
`853396e46f7289fc22d09464b1b2e3fc8161836b2bc8ee0342175ecaf9cf5e22`.

A numeric 45° rotation of a center-defined arc retains the arc presentation
class `12/67` under transform code 29. Native stores its visible label after
the four transform references with parameter 0.25 and offset `(-8, -33)`; it
also assigns the source arc a hidden naming label. The decoder now imports both
without partial view, while the port can name the transformed arc directly.
A clean port-authored transformed arc labeled `b[1]` displayed in Sketchpad 5
and saved all 2,368 bytes unchanged, retaining SHA-256
`5f8bd5a4cf8ec06fe9a55ace2370dbbae5937feb6aff4ab76a39f04d9a78e424`.

Translated-segment and translated-circle Native controls use the same label
payload after their references and literal transform record. Their observed
automatic offsets were `(-10, 5)` and `(1, -1)` respectively. Fixed-rotation
ray and line controls confirmed the same ordering; their saved parameters and
offsets followed the clicked path position and direction. All four controls
decode without partial view. The port uses deterministic quarter-path straight
placement and the established circle angle, and can create labels for every
transformed segment/ray/line/circle/arc flavor. A combined translated segment
and circle file saved all 2,504 bytes unchanged in Sketchpad 5 with SHA-256
`a53ce5dc97d156ecd93a132c64bc03addd5af9bb6c7f53ee0f36858be5bd254a`.
A combined rotated ray and line file saved all 2,480 bytes unchanged with
SHA-256
`c3dc021329dab61d9604dc6653b3c2f01bf2f8dd997c3881b0e9243aafba353a`.

Live colorized geometry keeps native code 75 while retaining the source
element's presentation subtype. Native colorized segment and circle labels
already matched the shared straight/circle decoder. A colorized arc exposed the
remaining `12/67` case: its label follows the color references and payload with
offset `(6, -23)` and parameter 0.25. Native controls for all three import as
full views. A port-authored colorized segment, circle, and arc with subscripted
labels displayed in Sketchpad 5 and saved all 2,948 bytes unchanged, retaining
SHA-256
`f71303c85e63c799acf99de69808e832d27ea92a11132f791d416b421336d350`.
Port-authored colorized ray and line labels likewise saved all 2,668 bytes
unchanged with SHA-256
`e20ae7646845ab37c05298ae8a1a9f9edea8f2a6db0190c0e199e55265577288`.

## Polygon opacity

Objects with a `VisualStyle::Polygon` expose a 0–100% slider backed by the full
native `u16` opacity field. Apply is a verified document transaction with one
undo checkpoint, structural legacy-header upgrade when allowed, and save/reparse
validation before commit.

The port generated a three-point polygon at native opacity 49,151 (75%).
Sketchpad 5 opened it with the expected translucent fill and saved all 2,040
bytes unchanged. The SHA-256 before and after Native save was
`cf630a541350eb6c16920274ae3b6a756ddabf0108d8caf01e43494ddbe229ad`.

## Parameter values

The common Properties panel links parameters to the same atomic editor as Edit
Selected Parameter. Alongside value, precision, display mode, animation timing,
domain, and keyboard adjustment, it exposes Native's Show Edit Box flag. That
flag is bit `0x0002_0000` of the `0907` payload mode word at offset 48. Enabled
parameters retain their canvas border and open the editor on double-click;
disabled parameters render without the border and require the menu or common
Properties link. A public-API value-only control with the flag cleared displayed
`1.000` without a border and saved byte-identically in Sketchpad 5, retaining
SHA-256
`956fe4f46c9e8004a33153b9603b11377a16cc01556172a92a39e8ea82de22fc`.

## Hide/Show buttons

Hide/Show action buttons expose their Native Label and Hide/Show settings in
the common Properties window. The port can edit a custom caption, restore the
generated caption, choose Always Show, Always Hide, or alternating Hide/Show,
and control fading and post-show target selection. The action and option edit
is one verified undo transaction; a generated caption follows a fixed-action
change, while a custom or explicitly empty caption remains independent.

Native property probes identify the three action controls as IDs 1266–1268,
the fade checkbox as 1159, and post-show selection as 1129. The `0906` option
word uses bit 0 for post-show selection, bit 1 for fading, and bit 2 for
alternation; its action word selects the fixed or next action. A port-authored
fixed-Show button with both options disabled and custom caption `Reveal trio`
displayed those exact settings in Sketchpad 5 and saved byte-identically,
retaining SHA-256
`d78a10eb91d5f0967ea4e52c0a9b77fcda1e28414e6f45880b1cdb110ce0967e`.

Native probes confirmed that ordinary pictures, tables, angle markers, and
angle values have only the common Object panel; they do not represent missing
specialized Properties panels. The earlier function-plot conclusion was caused
by a probe click clearing the fixture's saved selection and is corrected below.

## Animation buttons

Animation action buttons expose Native Label and Animation panels in addition
to the common Object panel. The port edits the caption and every target's
direction, Once Only state, speed, and parameter continuous/discrete rate,
step, and domain in one verified undo transaction. The creation and Properties
flows share the same controls, while imported opaque low mode tags survive the
edit.

A three-point port file was edited through the public API to use Backward/Slow,
Forward/Fast/Once Only, and Random/Medium settings and the longer caption
`Animation Verified`. Native Properties displayed the first target as Backward
and Slow, and Sketchpad 5 saved all 2,478 bytes unchanged. SHA-256 before and
after the Native save was
`c2609cee6d2d8a2ed628f35222c196d4636540b7211a37e22c415489f970f26d`.

## Movement buttons

Movement action buttons expose Native Label and Movement panels. The port uses
the same speed and destination-policy controls for creation and Properties,
and applies those settings with the caption in one verified undo transaction.
The shared post-order action writer remaps page-local pairs and safely splices
variable-size labels. For the proved zero-suffix layout, it also rebuilds the
Native terminator when a caption changes byte parity.

A public-API edit changed a port button from Medium/Follow to Instant/initial
destination and changed the odd-length caption to the longer even-length
`Move Once Verified`. Native Properties showed combo index 3, the same as its
Instant fixture, and selected control 1267 for initial-destination movement.
Sketchpad 5 saved all 2,062 bytes unchanged, retaining SHA-256
`31928aeb5e89063ee470c1f1d140aa9f9196912aa620398b80afc7d6fb2a85d1`.

## Scroll buttons

Scroll action buttons expose Native Label and Scroll panels. Creation and
Properties share the Upper Left and Center controls, while one verified public
document command applies the placement and caption atomically. Scroll groups
now use the shared post-order action writer, including page-local target
remapping and safe same-parity variable-length label splicing.

A public-API edit changed a port button from Upper Left to Center and expanded
its caption from `Scroll` to `Scroll to Center`. Native Properties displayed
the new caption and selected Center control 1146. Sketchpad 5 saved all 2,120
bytes unchanged, retaining SHA-256
`b231e3af819a4ea1084080b6a78d004a671ebccbd46979720df53b759a0f2d9f`.

## Presentation buttons

Presentation action buttons expose Native Label and Presentation panels. The
creation and Properties paths share simultaneous/sequential mode, Deselect
Objects, Stop Animations, Erase Traces, simultaneous stop policy, and elapsed
or sequential pause time controls. One public document command applies those
fields and the caption atomically while retaining the opaque native progress
word. Presentation groups now use the shared post-order action writer for
page-local target remapping and safe same-parity variable-length label splicing.

A public-API edit changed a port button from simultaneous defaults to
sequential mode with all three pre-actions, a 2.5-second pause, and caption
`Start Presentation`. Native Properties displayed the expanded caption,
sequential-only pause controls, and all three selected pre-actions. Sketchpad 5
saved all 2,358 bytes unchanged, retaining SHA-256
`7f410fea0545a270718043e201d7ace7eee101dabdb0dc337757ce1a478a9432`.

## Link buttons

Link action buttons expose Native Label and Link panels. Creation and
Properties share URL/Page mode, URL text, destination page, and optional
destination Hide/Show controls. Public document commands apply either complete
target form and the caption atomically, including conversion between the URL
and Page wire groups. The shared post-order action writer preserves same-kind
native padding, reconstructs a group for cross-kind conversion, maintains page
object indexes for conversions, and recalculates destination markers only when
Page Link topology changes. Existing multi-page
action buttons can now use all supported Properties editors without changing
their target structure or anchor.

A same-kind URL edit expanded both text fields to
`https://example.net/verified` and `Open Verified Link`; Native displayed the
new caption in URL mode and saved all 2,022 bytes unchanged with SHA-256
`09b5d5cbfb8618dd811787917255de3dc9f4c047710fffcf59e69737eeb45696`.
A separate multi-page URL-to-Page conversion displayed `Go to Verified Page`,
Page 1, and no destination action, then saved all 3,262 bytes unchanged with
SHA-256
`05ce8353a6e48152e6c45d4cf40f843cb99683f0b5112554900f4bcbe665da6c`.
The reverse conversion from a Page Link with a Hide/Show destination to URL
also cleared the destination marker and saved all 3,606 bytes unchanged with
SHA-256
`93b5426e9f3535f787b50f3e753a05aa2affa2180d55e61202ec5d548d9f553d`.

## Sound buttons

Native Sound buttons expose only Object and Label panels, not a separate Sound
settings panel. The port therefore edits the caption while retaining the fixed
one-function mono or two-function stereo sources. A transactional public
command and the shared page-local action writer support variable-length labels
on both one-page and supported existing multi-page documents.

The 17 corpus Sound groups show that the zero label terminator may be followed
by one opaque alignment byte. Native preserves it until a parity-changing label
edit, then drops it and rebuilds the terminator. The bounded Sound writer follows
that rule without weakening suffix handling for other action families.

A public-API edit changed `听到函数 f` to `Play Verified Sound`. Native
Properties displayed the new caption and the unchanged function `f`, then
saved all 2,898 bytes unchanged with SHA-256
`d05070bc3fb7d2053832193c860ad18bec1fa5d4ba539a39763b959e82655488`.

## Iteration images

Native exposes an Iteration panel on a selected iteration image rather than on
the hidden iteration controller. The image forwards edits to its controller.
Control 1281 holds the iteration count, while radio controls 1266 and 1267
select all images or only the final image. In the `090a` definition, the display
mode is the `u16` at offset 4 and a fixed count is the `u32` at offset 16.

The port applies those fields in one verified transaction for every supported
iteration family. Fixed counts retain their family-specific 1–128 or mapped
1–4096 bound. A parameter-driven count keeps its existing dependency and is
edited through that source value; its all/final display mode remains editable.
Final-only mode also reaches point, carried straight/circle, numeric-label,
branching-polygon, and general mapped rendering rather than being a save-only
flag.

A public-API edit changed the native point-translation fixture from three/all
to five/final. Only `090a` offsets 4 and 16 changed. Native Properties selected
control 1267, and Sketchpad 5 saved all 2,184 bytes unchanged with SHA-256
`4f32a95725c8fd0f8813452ca3d5a3c634cfb70f74fbbe470691419cbc1124ac`.

## Coordinate axes

Native exposes an Axis panel on x and y axes with Decimal and fractions or
multiples of π controls (IDs 1380 and 1381). The choice is per-axis: in the
existing trigonometric fixture, x selects π while y remains Decimal. The low
bit of the `0905` value retains direction and bit 16 selects π labels, yielding
`0`, `1`, `0x00010000`, or `0x00010001`.

The port exposes the same choice in Properties for ordinary, circle-defined,
and distance-defined axes. Decode, geometry, constraints, clipboard, source
patching, generated templates, canvas, and SVG all retain the format. Three
public-API outputs exercised an ordinary y axis, both circle axes, and a
distance x axis with its ordinary y partner. Sketchpad 5 displayed the expected
π labels and saved every file byte-identically. Their SHA-256 values were
`219175d991cd0358bce5809d9a7894569d4b35709adb301042bb70b3641adb64`,
`fe16de9ca77d60f34f2e5f90fea1d5937146ed2b32de711ca33f1afca9946e44`, and
`6f4b1ab31f87be02e8371f1c047c534cb9a26856d784e6631a8f8c2dc6a9d06e`.

## Sampled transformed pictures

Code-96 transformed pictures expose a second Native tab named Drawing. Control
1075 is the sample count; Native's validation dialog requires 5–250000, and the
stored `u32` at bytes 16–19 of `0902` changes directly. Combo 1437 has four allowed-distortion
choices. Their `07d3` `f64` values are None=`0.001`, Some=`0.25`, Most=`0.6`,
and All=`0.0`; the zero value is a valid sentinel rather than malformed data.

The port exposes both fields as a verified, undoable Properties transaction,
including inside the fan fixture's otherwise partial view. Its in-place writer
changes only the selected group's resolution and sample-count bytes and retains
the opaque remainder of `0902`. A public edit of sampled picture 191 to 64/All
displayed `64` and combo index 3 in Native Properties. Sketchpad 5 then saved
all 73,316 bytes unchanged with SHA-256
`dcc8251373174cb0bfff024dbe8d5b3195368ba5ea632f7d39a1119a508f4677`.

## Function and parametric plots

Both code-72 function plots and code-126/127 parametric plots expose Object,
Label, and Plot tabs. Plot control 1075 is the sample count; Native validates
the editable range as 20–250000. Controls 1073/1074 choose continuous or
discrete drawing, 1159 toggles arrows and endpoints, and edits 1126/1127 hold
the numeric domain.

The 40-byte `0902` descriptor retains the domain at offsets 0 and 8, the sample
count at 16, and the coordinate mode at 20. The low byte at offset 24 is zero
for continuous and one for discrete drawing. The low byte at offset 32 is one
when arrows/endpoints are shown and zero when hidden. Their upper bytes plus
offsets 28 and 36 remain opaque and are preserved. The port decodes older sample counts leniently but
uses Native's editable range for public changes.

The shared Plot editor updates all five settings atomically. Discrete plots use
individual sample points for canvas, SVG, and hit testing; continuous plots
render endpoint arrowheads when enabled. A public function-plot edit to domain
−4…4, 20 samples, Discrete, and hidden arrows/endpoints displayed those exact
values in Native Properties. Sketchpad 5 then saved all 2,738 bytes unchanged
with SHA-256
`d2102c999e52e14b4b47378e5da7d8607006d4e9589110ee888b2ae056333d1b`.
The same edit on a legacy plot with nonzero opaque upper bytes changed only the
two low flag bytes; Native saved the 4,440-byte result unchanged with SHA-256
`c6538c79e03b361d116f496fb5a47528d160ac436ab95b715b07e976889a29f6`.

Labels on other geometry remain outside this increment.
