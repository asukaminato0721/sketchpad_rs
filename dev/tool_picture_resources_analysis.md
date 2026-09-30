# Embedded-tool picture resources

Audited on 2026-09-30. The largest directly ready prototype root after the
scalar-Parameter checkpoint was local object 4 of
`Samples/个人专栏/汪志平作品/100以内两位数加一位数小棒图.gsp`, tool 0. It is a
code-99/class-16/subtype-78 picture attached to an already decoded free point.
Its unsupported dependency reach was 66 slots before accounting for independent
roots later in the graph.

## Resource ownership

The source file places four `0514` PNG resources between the document-global
`03ee`/`03ef` resource markers, before the drawing and embedded-tool canvases.
The tool's picture groups contain two-byte `1f44` resource references. Thus the
prototype owns picture constructions but not private copies of their image
bytes; resource IDs refer to the enclosing document's ordered resource table.

`Document::embedded_tool_document` previously copied tool groups and local font
tables into a blank synthetic document. That accurately isolated object and font
namespaces but left its resource table empty, so the ordinary bounded attached-
picture decoder could not resolve a valid resource ID. The extraction now detects
a class-16/subtype-78 picture header and copies the document-level `0514`
payloads, in source order, immediately before the synthetic document's `03ef`
marker. It does not reinterpret or renumber any picture group.

## Measured result and boundary

The 149-object prototype still remains partial, but local object 4 now decodes
as the already-supported point-attached `Image`. Four direct code-99 objects and
43 downstream objects recover across the corpus. Decoded prototype slots rise
from 4,866 to 4,913 of 5,356, leaving 443 unsupported. The eight remaining
code-99 prototype slots have separate unsupported prerequisites; this checkpoint
does not loosen their construction guards.

Drawing coverage remains 1,599/1,599, all 1,599 source files round-trip exactly,
and isolated tool extraction has zero failures. No standalone test case was
added. The final scan is `/tmp/tool_picture_resources_coverage.log`.
