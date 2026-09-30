# Polymorphic polygon transforms in embedded tools

Snapshot: 2026-09-30.

## Native profile

Tool 0 in `任意边数凸多边形在直线上滚动工具（李章博）.gsp` has 120
prototype objects. Local 2 is a polygon input stored with the generic
class-1/subtype-15 header. Two later transforms retain subtype 15 even though
their runtime result is a polygon:

- local 91, code 16, translates the input polygon by a live vector;
- local 120, code 29, rotates that translated polygon by a live numeric angle.

Both groups use a 28-byte header, have exactly three references, have no
constraint record, and name either the polygon input or another strict
subtype-15 polygon transform as their source. The admission is limited to this
tool-prototype context and profile. Other remaining code-16 prototype variants
stay unsupported.

The same prototype stores exact 28-byte `[1]` metadata records on its code-89
iteration descriptor and code-77 binding. A corpus audit of codes 76, 77, and
89 found only `[1]` through `[8]` in this layout. The decoder accepts the
single-digit `[1]` through `[9]` grammar as internal iteration metadata; it does
not expose these records as object labels.

## Invocation and writing

Invocation rewrites the polymorphic header to class 4 before deriving polygon
presentation, assigns the next interior drawing order, and drops the
point-family label. The shared polygon-style recognizer now accepts
class-4/subtype-15 transform headers, and transform authoring writes an
object's explicit visual style rather than silently reverting to the default.
This makes the transactional encode/reparse comparison exact.

Every numeric prototype caption uses bold 16-point `黑体`. Its observed row
metric is 21 pixels, which is now handled by the existing caption-placement
table. Parameter calculations retain their established Tahoma default.

Deep final-only mapped constructions repeatedly consume terminal carried
points. State-local evaluation now caches point values as well as numbers in
topological carried order, avoiding redundant endpoint replay without changing
serialized state or validation rules.

## Verification

A public invocation bound a three-vertex polygon and a free point, created 118
outputs, produced 123 total objects including two transformed polygons, and
reopened with `partial_view == false`. The 18,510-byte result re-encoded
byte-for-byte.

The generated file was then opened in the Chinese Sketchpad 5 executable under
Wine and Xvfb. Native rendered the input triangle, baseline, mapped
construction, and transformed polygon. Ctrl+S on a temporary copy preserved
SHA-256
`c5be068bd13750036347ee42fe8139d47ad657686445f780f959d5ce41676801`.

The final corpus scan reports:

- 1,599/1,599 byte-exact source round trips and zero failures;
- 1,576/1,599 documents with every object kind decoded;
- 1,599/1,599 documents with every drawing kind decoded;
- 5,112/5,356 decoded prototype slots, leaving 244 unsupported;
- 52 complete and 51 partial isolated tool views.

No standalone test case was added; the existing workspace suite, strict
Clippy, corpus scan, invocation round trip, and Native open/save control are the
regression gates.
