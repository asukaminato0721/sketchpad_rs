# Embedded custom-tool namespaces

Current implementation: isolated prototype decoding and point/path-input tool
execution are available, with Native-verified Square, segment/midpoint, and
polymorphic constrained-point API outputs. Complete registered tool sections also permit structural drawing edits
while their raw local records remain unchanged. Point-input tool creation also
has Native cross-open, menu registration, and execution evidence. The sections below
record the earlier namespace investigation and independent Native controls;
their pending statements describe those earlier stages. See
[runtime evidence](tool_runtime_analysis.md) and
[creation evidence](tool_creation_analysis.md) for the current scope.

Sketchpad stores each custom-tool prototype in a separate canvas (`03ea` through
`03eb`) with local object IDs, font tables, and input definitions. These records
must not be decoded as objects on the last drawing page.

The corpus `向忠作品/12T.gsp` has 145 drawing objects followed by one named tool,
`工具#1`, with 44 prototype objects. The paired HTML export also lists 145 drawing
objects. Previously the importer exposed 189 objects and resolved prototype
references against the drawing. Prototype calculation 151 therefore appeared to
reference drawing point 1; that was a namespace error, not evidence for another
calculation layout.

The importer now skips tool canvases when decoding drawing objects and reading
page preferences, parameter-name counters, and font counts. Ordinal-based save
patches use the same drawing record scope. Tool summaries expose their names,
prototype codes, and complete boundaries; raw records remain preserved. Tool
creation, execution, prototype decoding, structural edits, and font changes in
documents with tool font tables remain unsupported.

The original 30030-byte corpus file round-trips unchanged. A simpler control uses
the known point template with the authentic 7340-byte tool definition appended
before the document terminator. Native Sketchpad opened and saved this 9176-byte
control byte-identically and displayed the drawing point, without prototype
objects on the page. The retained fixture is
`embedded_tool_point_scene_original_native_verified.gsp`.

Regression coverage checks the original scene count, separate prototype summary,
unchanged tool bytes during supported point edits, save/reparse, and undo/redo.
Corpus reporting retains total object slots and reports undecoded tool prototypes
separately; excluding them from drawing objects is not counted as tool support.

The actual Rust-authored move of the drawing point to (130, 150) also opened
and saved byte-identically in native Sketchpad (9176 bytes). Its screenshot
shows the moved point; all prototype records remain unchanged. The native
edited file is retained as an exact regression golden.

The refreshed corpus contains 103 embedded tool definitions in 65 documents,
with 5356 prototype slots. All 1599 source files preserve their bytes. The
285703 total slots comprise 280347 drawing slots and those prototypes. The
112506 unsupported-slot count includes every prototype; 85 documents have
all object kinds decoded without tool prototypes, and 86 have all drawing
kinds decoded. Changes to the histogram also reflect scope correction, not
just the newly supported calculation and point families.

Drawing font-family lists and indexed style resolution also use the drawing
namespace. In the native-verified simple control the drawing has Tahoma and
the tool has Times New Roman. The drawing picker exposes only Tahoma; an
out-of-range drawing style index cannot resolve into the tool table. Existing
font payload validation is shared between scoped and standalone readers.

Tool summaries now expose ordered local input IDs from `04b4`. A declaration
must have an exact count/length match, unique IDs, and IDs within the prototype
object count. Missing, duplicate, or malformed declarations produce no input
list and remain byte-preserved. A drawing-scoped corpus audit found all 103
authentic definitions structurally valid, with 515 declared inputs. In 12T
the input order is [5, 2, 3, 4, 1]; this ordering is retained for future tool
invocation and does not imply that tool execution is implemented.

A separate negative control copied the authentic Square prototype groups into
a drawing canvas and replaced its point-input coordinates. Rust could decode
all 13 object kinds, but native Sketchpad displayed a `文件问题` (File Problem)
dialog. A known point control opened in the same locked diagnostic. This
conversion is not accepted as a tool implementation or native writer proof:
prototype records require an independently verified materialization step.
The negative control remains diagnostic only, with no schema widening.

Keeping the Square tool unchanged inside the known point document succeeds:
native Sketchpad opened and saved the 3516-byte control byte-identically and
displayed the drawing point. The retained
`embedded_tool_square_point_scene_original_native_verified.gsp` has one drawing
object and 13 separate prototype objects, with input order [2, 1]. This is
import/preservation evidence, and is distinct from invoking the tool.

The earlier appended-tool controls keep the point template's document header
(tool count 0, section count 2). They establish scene import and raw preservation;
they do not establish that Native registers those tools. A separately registered
Square control sets the header's tool count to 1 and total section count to 3.
Native now lists `Square Given Two Vertices` in its custom-tool menu and saves
all 3516 bytes identically. This retained control is
`embedded_tool_square_registered_native_verified.gsp`.

An audit of all 65 corpus documents containing tools found exact header counts:
the declared tool count equals the number of tool canvases, and the total section
count equals drawing pages plus tools plus one. No mismatches were found.

Actual Native invocation of that registered Square tool takes two point clicks,
creates a square with 11 dependent objects, and leaves the unrelated original
point intact. The result has 14 drawing objects and the same 13-object tool
prototype graph. Native normalizes its local Arial/Times fonts to Tahoma,
including the first local style’s font index; all other tool records are equal. Its corners are (200,180),(260,180),(260,120),(200,120); helpers 5, 6, 7, 9
are hidden. Native opened and saved the 4472-byte result again byte-identically,
with a positive square screenshot. The retained
`embedded_tool_square_invoked_native_verified.gsp` regression checks geometry,
visibility, separate namespaces, identical prototype records excluding local
font tables, and exact saving.
This establishes Native materialization behavior; Rust tool execution remains
pending.

Native UI probes may now set `SKETCHPAD_WINE_SCREEN=800x600` on the central
round-trip helper. The larger display exposes the ninth toolbox icon and full
tool menus; the default probe display remains unchanged. The queue lock and Wine
cleanup behavior are unchanged.

The Square assertion exposed two pre-existing geometry discrepancies. Six
independent Native controls establish that code 12 selects the larger line
parameter and code 11 the smaller before testing segment/ray extents; an
out-of-range selected branch remains undefined. Parallel and perpendicular
lines have parameter anchors centered at the construction point, separated by
the full source vector. Perpendicular direction remains clockwise in screen
coordinates. Reversed endpoints swap the branches, and parameter values
0.25/0.5/0.75 establish direction and anchor placement independently of Square.
All six controls were opened and saved byte-identically with positive geometry
captures. The corrected Square corners now match Native.

An independent Native Create Tool control selects two free points, their
segment, and its midpoint, then creates the default named tool through the
custom-tool menu. Native saves a 2816-byte document with four drawing objects,
a four-object prototype, input IDs [2,1], and registered header counts 1/3.
Reopening and saving a separate copy exits successfully and preserves all
2816 bytes; the capture visibly shows the two endpoints, midpoint and segment.
The retained control is `embedded_tool_simple_created_native_verified.gsp`.
This proves the native creation layout; Rust Create Tool remains pending.
