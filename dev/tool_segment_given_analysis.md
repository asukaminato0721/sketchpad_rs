# Native finite-segment tool givens

These controls use a drawing with two free endpoints, their segment, its
midpoint, and a point constrained to the segment at fraction .3. The source
is retained as `embedded_tool_given_selection_source_native_verified.gsp`
(2108 bytes). Every accepted Native-created tool below also passed a second
Native open/save with identical bytes and an inspected Script View capture.

| Selected objects | Native result |
| --- | --- |
| Segment only | Creation rejected |
| Constrained point only | Creation rejected |
| One endpoint and segment | Creation rejected |
| Segment and midpoint | One finite-segment given; midpoint recipe references it |
| One endpoint, segment, constrained point | One generic Path given; unused selected endpoint omitted |
| Both endpoints and midpoint, without selecting the segment | Two point givens; required segment included as a suppressed helper |

The rejected selections show the Native dialog text
`您选择的对象无法定义一个工具．请选择工具的最初对象和由它们构造的对象．`
(the selected objects cannot define a tool; select initial objects and objects
constructed from them). The rejection is more specific than a minimum selection
count: the two-object endpoint/segment selection fails, while segment/midpoint
succeeds. Rejection captures are retained as
`embedded_tool_given_selection_{segment,point,endpoint_segment}_rejected_native.png`.

## Specialized segment input

Selecting a segment and its midpoint produces a code-zero given with class1,
subtype14. Its exact 28-byte header is
`00000000000001000e00210100000000ffff0000c1bdf0ff00000000`.
The midpoint recipe is code1, class1, subtype11, with one `07d2` reference to
that given. `04b4` declares the single input `[1]`. This category is distinct
from the class1/subtype5 generic Path given used by unplaced code15 recipes.

Native invocation binds the subtype14 input to a primitive segment and to a
vector-translated segment. The resulting code1 groups reference the actual
segment host directly, including the transformed host. Clicking ray and line
strokes does not produce a bound midpoint; pending interactive clicks were
cancelled separately before the next attempt. The retained
`embedded_tool_segment_given_midpoint_invoked_native_verified` GSP/PNG pair is
3650 bytes and passed a second Native open/save unchanged. This proves the
tested finite-segment input behavior; it does not establish acceptance of other
given subtypes or infinite straight modes.

The selection tools are retained as
`embedded_tool_given_selection_midpoint_native_verified` (2726 bytes),
`embedded_tool_given_selection_one_endpoint_native_verified` (2730 bytes), and
`embedded_tool_given_selection_endpoints_midpoint_native_verified` (2916 bytes),
each with its matching PNG. The one-endpoint control still emits only a Path
given and its constrained-point recipe; the endpoint is not an additional given.

## Suppressed construction helper

Selecting both endpoints and the midpoint includes the unselected segment
because the midpoint needs it. The segment's prototype header has flag2,
and its local references are `[2,1]`; the ordered point givens are `[2,1]`.
Invoking this Native-created tool produces an ordinary drawing segment with the
same flag2. Its geometry remains live, but Native displays only the two inputs
and the midpoint. Pixels on both halves of the segment are white. The resulting
3188-byte drawing and positive capture are retained as
`embedded_tool_midpoint_hidden_helper_invoked_native_verified.{gsp,png}` and
passed a second Native open/save exactly.

Native Show All Hidden does not reveal that helper or clear flag2. An ordinary
flag1 segment in the same drawing serves as a positive control: Show All clears
its flag to0 and displays it, while the helper remains flag2 and undrawn. Native
also updates UI/font caches during the command; these changes are not described
as a byte-identical comparison against the pre-command drawing. The resulting
3188-byte output does pass an exact second reopen/save. It is retained as
`embedded_tool_hidden_helper_show_all_native_verified.{gsp,png}`, with a separate
before-command PNG. Flag2 is therefore presentation suppression, independent
of the editable ordinary visibility bit. An optional parent-selection probe did
not establish explicit selection/showing of the suppressed helper, so no such
behavior is claimed.

## Fresh API writer control

The actual subtype14 invocation API writer was checked separately after the
shared midpoint-host implementation. Its 3394-byte drawing includes a midpoint
on an ordinary segment at `(350,300)` and a midpoint on a vector-translated
segment at `(450,340)`. Native opened and saved it successfully; comparison
against an untouched input snapshot was exact, and the positive geometry
capture was inspected. The actual writer bytes and PNG are retained as
`embedded_tool_segment_midpoint_invoked_writer_native_verified.{gsp,png}`.
This is independent of the earlier tool created and invoked through Native UI.
