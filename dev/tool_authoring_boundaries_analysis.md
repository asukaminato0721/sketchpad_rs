# Native tool authoring selection boundaries

The independent Native selection controls establish these distinct cases:

- Segment and PointOnSegment selected, with neither or only one endpoint selected:
  a generic Path given (code0/class1/subtype5) and an unplaced code15 output. The
  unused selected endpoint is pruned.
- Both endpoints, Segment and its constrained point selected: two Point givens;
  the Segment is constructed from them and the point has no captured placement.
- Segment and Midpoint selected: finite Segment given (code0/class1/subtype14),
  followed by a code1 midpoint recipe referring to that abstract given.
- Both endpoints and Midpoint selected, without selecting the needed Segment:
  two Point givens and an included Segment helper suppressed by Native flag2.
- A singleton Segment or singleton constrained point is rejected, as is one
  endpoint plus a Segment with its other independent endpoint omitted.

Tool creation now checks these supported boundaries. The existing point-only
exact Native writer goldens stay unchanged. Point-given definitions require all
independent Free roots to be selected; unselected constructed prerequisites are
included as suppressed helpers. A selected path consumed by a selected result,
whose geometric ancestry cannot be constructed from selected independent
Free roots, becomes an abstract input. Constructible unselected intermediate
geometry remains a helper rather than incorrectly replacing a selected derived
path with an abstract given.
Current filtered templates support one Path boundary with code15 outputs, or one
finite Segment boundary with midpoint outputs. Multiple boundaries, mixed
Path/Segment outputs, and other output auxiliary schemas fail atomically rather
than rewriting opaque numeric identifiers. Further categories need their own
semantic template handling and Native controls.

The Path/Segment branch first serializes the full ordinary clipboard graph.
It then emits a dense isolated namespace: canonical abstract given, neutral
16-byte 0899 payload, no given dependencies, followed by selected output groups
with their semantic host reference remapped to local1. Constrained output 07d3
placement is omitted. Numeric auxiliary groups are never remapped by scanning
arbitrary words. A finite Segment given is represented by ToolInput(Segment),
and its output by SegmentMidpoint; no invented Free endpoint geometry is used.

The source drawing is encoded independently and preserved while the new tool
canvas is appended. Tool creation commits one structural history checkpoint,
keeps the selection, and verifies save/reparse before committing. Runtime binding
uses existing available destination objects, chooses code15 placement once,
retains ordered references, and follows later source edits.

Four actual fresh Path API files opened and saved in Native with helper exit0,
then independent byte comparisons succeeded: Segment created2,582/invoked2,662
bytes, Circle created2,582/invoked2,726 bytes. Two finite Segment API files also
passed: created2,568/invoked2,692 bytes. These are retained as
`tests/fixtures/embedded_tool_path_authored_*_writer_native_verified.gsp` and
`embedded_tool_segment_authored_*_writer_native_verified.gsp`. Positive captures
are `dev/tool_authoring_path_native.png` and `dev/tool_authoring_segment_native.png`.
Created-file exact golden checks protect canonical group layout. Invocation
fractions are chosen dynamically, so runtime tests check semantic placement and
live references rather than requiring a newly generated random fraction to equal
a retained file.

The endpoints-plus-midpoint API writer also passed Native at 2,704 bytes; its
invocation using a separate endpoint pair passed at 2,964 bytes. The final
runtime helper retains flag2 through immutable `presentation_suppressed` metadata,
and stays absent from output selection and rendering while the ordinary
visibility bit remains separate. `dev/tool_authoring_hidden_helper_native.png`
shows the separate lower endpoint pair and midpoint without a connecting line.
The final fixtures are `embedded_tool_hidden_helper_authored*_writer_native_verified.gsp`.
Native Show All Hidden does not remove this helper suppression, established by
the independent presentation control.

The editor accepts finite Segment givens through existing finite segment picking;
Ray and Line clicks fail without creating endpoints. Two-click synthesis remains
bounded to independently proved Path givens. Numeric givens bind existing finite
numeric captions, including parameters, calculations, degree parameters, and
length measurements. Both numeric caption boxes and the actual measurement
overlay bounds are used; blank numeric clicks do not create points. Focused UI
tests cover those source families, selected-input invocation, atomic failures,
and live parameter edits. Numeric definition authoring is a separate integration.

The port-authored definitions were also invoked through Native's actual toolbox
flyout. A click on existing Segment3 added a PointOnSegment as scene object5;
Native saved 2,762 bytes. The finite Segment definition bound a separate new
Segment7 with no midpoint, added its midpoint as scene object8 at (450,350), and
saved 2,928 bytes. Both files passed a second Native reopen/save with exact byte
comparison. Retained actual execution files are
`embedded_tool_path_authored_native_invocation_verified.gsp` and
`embedded_tool_segment_authored_native_invocation_verified.gsp`; corresponding
positive captures are `dev/tool_authoring_{path,segment}_native_invocation.png`.
A regression verifies their input categories, live host references, roundtrip,
and subsequent host edits.

The first finite Segment invocation on the original host selected its already
existing midpoint without increasing the scene count. The separate new-host
control distinguishes that behavior from a failed definition. This suggests
Native construction reuse for identical midpoint dependencies; broader reuse,
style handling, and undo policy require a separate runtime audit and have not
been inferred here.

A further actual API file builds CircleBySegment from selected Free givens through
an unselected suppressed Segment helper and invokes it successfully. Native
opened/saved that file at 3,080 bytes, with exact comparison. The retained
`embedded_tool_transitive_geometry_authored_invoked_writer_native_verified.gsp`
and `dev/tool_authoring_transitive_geometry_native.png` protect the supported
transitive construction case.
