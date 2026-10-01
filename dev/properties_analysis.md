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

Labels on other geometry, picture and sampled-object opacity, Table, Iteration,
Axis, Function, Plot, Marker, and action-specific panels remain outside this
increment.
