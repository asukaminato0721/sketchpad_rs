# Embedded tool creation

Native's all-selected endpoints/segment/constrained-point control omits the
point's `07d3` placement parameter from the prototype, even though the host is
constructed from point givens. The retained
`embedded_tool_constrained_point_from_endpoints_created_native_verified`
fixture independently reopened byte-identically. Creation now omits placement
for supported constrained-point outputs; the source drawing retains its original
constraint and invocation generates a new persistent placement. Regression
coverage checks the isolated recipe, source preservation, and undo/redo.
The fresh 2724-byte API writer independently opened and saved byte-identically
in Native with a positive capture, retained as
`embedded_tool_constrained_point_created_writer_native_verified.{gsp,png}`.
The regression compares the actual creation API bytes to this verified golden.
Selected path boundaries becoming path givens are not yet authored by this API.

`Document::create_embedded_tool(name)` copies the selected construction and its
prerequisites into a separate prototype namespace. Independent free points
become the given inputs. At least one input and one constructed result are
required. The current creation scope is a supported single-page drawing;
other given categories and tool-definition editing remain pending.

Given points precede the remaining construction, in reverse source order.
The `04b4` declaration independently records their binding order. References,
midpoint hosts, and clipboard visibility metadata are remapped to dense local
IDs. Given coordinate records become neutral `0899` placeholders. Existing
styles and labels are retained; tool invocation supplies the actual points.
The original drawing and its selection are unchanged. Creation validates a
saved/reparsed clone before committing one structural undo step.

The native container requires a `0386` compatibility block, a 36-byte `03ea`
preamble with its local object count, local font tables and construction
groups, an input declaration, and `03eb` followed by the independent `0387`
section terminator. Document header counts include the new tool and canvas.
Tool names are NUL terminated with even-byte padding included in the declared
`044c` payload length. All 103 corpus definitions satisfy both framing rules.

The initial writer omitted `0387` and declared a 21-byte tool name with its
padding outside the payload. Adding only the terminator or changing only the
name length still failed Native opening. Fixing both, with no point caption,
style, or opaque-cache changes, produced a 2696-byte control that opened and
saved byte-identically in Native and appeared in its custom-tool menu as
`Segment and midpoint`. Earlier caption experiments also lacked one of these
framing fixes and do not establish any caption requirement.

As an independent format control, Native Create New Tool was also run on the
same authored four-object drawing. It produced a registered four-object tool
with inputs `[2,1]`; Native added prototype names and changed its given-point
styles. These changes were unnecessary for acceptance of the minimal writer
control and are not applied by the Rust creation API.

The fresh API-generated 2696-byte file independently opened and saved in Native
with a successful exit and exact byte comparison. Its menu listed
`Segment and midpoint`; the file, geometry capture, and menu capture are retained
as `embedded_tool_simple_created_writer_native_verified` fixtures. The creation
regression compares actual API bytes to this Native-verified golden.

Native invocation of that tool created two new inputs at `(300,280)` and
`(360,300)`, segment 7, and midpoint 8 at `(330,290)`, alongside the four unchanged
source objects. The resulting 3104-byte file independently reopened and saved
byte-identically in Native. This establishes execution of the newly authored
registered definition, separately from execution of imported tools.
The resulting file and a positive Native capture showing the second segment and
its midpoint are retained as
`embedded_tool_simple_created_writer_invoked_native_verified.{gsp,png}`.

The combined all-targets gate passes 837 tests. Corpus checkpoint 19 preserves
all 1599 source files byte-identically; neither this bounded creation support nor
prototype decoding establishes completion of the remaining custom-tool commands.
