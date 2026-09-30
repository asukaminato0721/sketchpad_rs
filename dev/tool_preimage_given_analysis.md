# Polymorphic Preimage custom-tool given

The sole class-1/subtype-92 given is local object 3 of tool 0 in
`换行迭代格式工具及其运用示例.gsp`. Its exact group is
`07d0:28, 07d5:26, 07d8:2, 0899:16, 07d7:0`; the `0899` payload is all zeroes.
The source is 40,478 bytes with SHA-256
`1848fea89f9c3856e11ba857cde7ddfde60783d9f8a238016270459d4bb6f806`.

A same-size temporary control changed only the first tool's `04b4` order from
`[4,2,3,1]` to `[3,1,4,2]`. Native Sketchpad then displayed
`1. 匹配原象 C` (match Preimage `C`). Clicking either the ordinary finite
segment on page 4 or its point-on-segment endpoint advanced to input 2. Hovering
the source polygon outlined it as eligible, while the visible scalar Parameters
did not highlight. This is a polymorphic geometric source category, not an
ordinary Point, Path, iteration binding, or numeric value.

The three dependent transforms are code 16, code 33, and code 16. Their `07d2`
reference lists are `[3,4,10]`, `[11,10,7]`, and `[12,4,2]`; the leading count
word is not a reference. Each descendant retains class 1/subtype 92 in its
header, so that header, rather than an extra reference, carries the polymorphic
result type. The concrete element type is chosen from the bound preimage only
at invocation.

The isolated decoder admits exactly that three-reference code-16/code-33
family. It reuses the fixed Point transform representation as an internal,
unbound stand-in while retaining the original header bytes. Invocation accepts
only the independently observed Point, finite Segment, and Polygon source
families, then replaces the stand-in with the bound source's concrete element
type before insertion. Ordinary drawing decoding is unchanged.

All 13 objects in the affected prototype now decode. Native invocation on page
4 matched an existing point, created a point on the finite segment for the
second point input, matched the polygon and `n`, and emitted nine outputs. The
41,750-byte result has SHA-256
`f5d262480440b1e29b809aed41cd25619fea53d27d2315de41c0f89cc514078d`.
Its two auxiliary formulas carry caption-suppression flag 2 and share anchor
`(4,183)` with the following visible formula; Native does not lay out their
hidden `trunc(t/m)` caption. The three polygon results receive drawing orders 2,
3, and 4, retain the prototype color/opacity, and drop the point-only labels.

The port now applies those same materialization rules. Direct Point, finite
Segment, and Polygon invocation probes each create all nine outputs, serialize,
and reparse with the expected concrete transform element. A 3,664-byte
port-authored Polygon result opened and saved byte-identically in Sketchpad 5
under Xvfb/Wine (SHA-256
`fd9b22793493c0479842ad5b50a0d8737731bc1dade118d44f439123c346f942`).

The full corpus remains 1,599/1,599 byte-exact and drawing-complete. Embedded
tool coverage rises from 5,062 to 5,065 of 5,356 slots; unsupported prototype
slots fall from 294 to 291. Fully decoded documents rise from 1,570 to 1,571,
and complete isolated views rise from 47 to 48 of 103.
