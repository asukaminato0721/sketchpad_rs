# Named referenced captions in embedded tools

Snapshot: 2026-09-30.

## Native profile

Strict class-0/subtype-59 code-73 and code-90 captions may place a `07d5`
object-name record between their `08fc` rich text and `07d8` font reference.
The name uses the ordinary bounded `LabelAux` grammar; the surrounding record
order, counted references, rich payload, font index, and optional empty `232e`
marker remain unchanged.

The shared referenced-caption and attached-point numeric-caption decoders now
accept exactly that optional position. The document decoder retains the name on
`ReferencedText`, `PointReferencedText`, and `PointNumericLabel`, while the
writers put it back in the same position for newly materialized objects.
Validation uses the same three-kind admission.

Code-35 Locus groups in the affected tools also carry a valid `07d5` name and
indexed `07d8` style. As with polygon and iteration names, this is internal
metadata rather than an editable geometry label. Both records are strictly
validated and admitted without exposing a label on the locus object. This
removes an independent structural-safety restriction from 15 isolated tools.

## Invocation placement

An attached point value is spatially anchored to its point and does not occupy
a free-standing numeric-caption row. Tool placement now skips it when locating
the previous measurement row. Point and segment ratios retain their observed
64-pixel fraction row, and the adjacent native anchors in this prototype prove
a 19-pixel row step for bold 16-point `宋体` captions.

Tool 1 in
`圆在正多边形外的滚动完整版（雪山飞狐11.01.07）.gsp` consequently invokes
against three points. It creates 76 outputs and 79 total objects, retains one
named attached caption, reopens without a partial view, and re-encodes all
11,164 bytes exactly.

The generated file opened and rendered in the supplied Chinese Sketchpad 5
under Wine and Xvfb, including its attached numeric caption, triangle, circle,
baseline, and dependent construction. Native Ctrl+S preserved SHA-256
`26014846f70421cb6ed74646ab308ce86db50ab24c94f2dfca74c6bada089ad6`.

## Corpus result

The full 1,599-file scan reports:

- 1,599/1,599 byte-exact source round trips and zero failures;
- 1,581/1,599 documents with every object kind decoded;
- 1,599/1,599 documents with every drawing kind decoded;
- 5,207/5,356 decoded prototype slots, leaving 149 unsupported;
- 67 complete and 36 partial isolated tool views;
- 769 partial source views, down from 785.

Relative to the previous checkpoint, 48 code-90 roots, 20 code-73 roots, and
27 descendants restore 95 prototype slots. Strict Locus name metadata raises
complete isolated tools from 52 to 67. No standalone test case was added; the
existing workspace suite, strict Clippy, corpus scan, invocation round trip,
and Native open/save control are the regression gates.
