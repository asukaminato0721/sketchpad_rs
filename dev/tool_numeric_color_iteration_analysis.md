# Numeric tool placeholders, colorized objects, and mapped pictures

Audited on 2026-09-30. This pass follows the code-101 and nested-mapping
checkpoint and covers three related embedded-tool boundaries. Ordinary drawing
decoding and authoring rules are unchanged.

## Prototype-only numeric operands

Native tool scripts can feed a declared generic Numeric input directly into a
code-75 color binding or use it as the depth value of a code-89 mapped
iteration. The isolated prototype represents that declared given as
`ToolInput(Numeric)` until invocation binds it to an actual numeric object.

The decoder now admits that placeholder only while decoding an isolated tool
canvas. Runtime invocation still requires a finite numeric destination object,
and remapping replaces the placeholder before ordinary document validation.
This restores 15 code-75 objects, three code-89 roots, and five code-77
bindings. Tool path-point placeholders are likewise valid seeds, branch images,
or carried nodes because invocation materializes each one as an ordinary
constrained point before validating the destination graph.

Two representative colorized tools were invoked into new documents. The
`n×n立方体` invocation created 99 outputs including nine colorized objects; the
`堆砌的正方体` invocation created 106 outputs including six colorized objects.
Both results reopen non-partially and re-encode exactly. Sketchpad 5 under
Xvfb/Wine opened both files, and Ctrl+S preserved SHA-256 respectively:

- `44f33f7ed1a2e68eb6c8f7c8d5cdedec3d2091c1776c6179773100a614fd1a3c`;
- `dbecfdedb411b80e241c0a8a5df882b4b9f6be3115edb16bbb929300c2793063`.

The second control also establishes that normal and bold Times New Roman
size-16 numeric captions share Native's 21-pixel row step.

## Indexed hidden picture labels

The eight remaining code-99 prototype pictures use the already-supported
point-attached PNG record layout. Their only rejected record was the hidden
`07d5` label: the importer required the exact `[1]` payload, while Native uses
the same 28-byte metadata template with `[2]` through `[7]` as well.

The full corpus contains 17 code-99/code-100 hidden picture labels. Every byte
outside the one ASCII digit is identical, and the observed digit range is
exactly 1 through 7. The parser now accepts only that bounded family. Resource
indices, PNG dimensions, point references, finite positive scales, and the
terminating group structure retain their existing checks.

This restores all eight code-99 roots. Four recovered pictures are carried by
mapped iterations, which also restores four code-89 roots and four code-77
bindings. Original resources remain read-only; this change does not claim
general multi-picture authoring. A diagnostic invocation of the newly decoded
picture prototype reaches the existing unverified Tahoma size-36 caption-layout
guard, so that broader runtime remains intentionally pending rather than using
an invented metric.

## Measured result

Across the complete corpus, decoded prototype slots rise
**5,251→5,290/5,356** and unsupported slots fall **105→66**. Fully decoded
documents rise **1,585→1,591/1,599**, and complete isolated tool views rise
**71→75/103**. All **1,599** drawing canvases remain complete, all **1,599**
source files round-trip byte-exactly, and there are zero decode failures.
Partial source views remain 765 because the recovered PNG resources are still
structurally read-only. No standalone test case was added. Full scan:
`/tmp/tool_colorized_picture_iteration_coverage.log`.
