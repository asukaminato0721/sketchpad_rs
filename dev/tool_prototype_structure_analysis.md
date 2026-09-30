# Embedded-tool structural markers and locus labels

Audited 2026-09-30 against all 1,599 corpus documents and Native Sketchpad 5
under Xvfb/Wine.

## Boundary

After every prototype object decoded, 17 of 103 isolated tool views were still
structurally partial. A trace of every strict rejection separated them into
three groups:

- otherwise typed prototype objects with one empty `0x232e` metadata record;
- code-97/class-6/subtype-42 parameter loci with a valid internal `0x07d5`
  label and indexed `0x07d8` style;
- eight prototypes containing image resource tables that are not structurally
  writable yet.

The first two records are admitted only when the decoder has an embedded-tool
input namespace. This does not broaden ordinary drawing admission. The empty
marker has no payload and is not copied into materialized output; the
parameter-locus records use the existing strict `LabelAux` and font-index
validation already used for geometric loci.

## Result

Nine tools become structurally complete: the wall tool, all three compass
tools, the quadrilateral-rolling tool, both previously partial 3D-coordinate
helpers, the second caliper tool, and the perspective 3D-coordinate tool.
Complete isolated views rise from **86 to 95**, leaving **8** partial image
tools. All 5,356 prototype slots and all 285,703 corpus object slots remain
decoded, and every one of the 1,599 source files remains byte-exact.

The recovered `6.多面体棱虚实线` tool was invoked with four real points. It
created ten outputs in a 14-object, 3,420-byte document. The result reopens
without a partial view and re-encodes exactly. Native Sketchpad visibly renders
the points and constructed edge, and Ctrl+S preserves SHA-256
`db86b5026f00c78ab4987ba9a925cd4ad0f2d6745f289bcc273340596081d5c9`.

Follow-up Native inspection showed that the rolling and wall failures were not
missing font metrics: their preceding numeric objects have presentation flag 2
and occupy no caption row. Reusing those suppressed anchors lets the wall tool
create 34 outputs and the rolling tool create 74 outputs; both save, reopen,
and re-encode exactly. The same rule removes the false caption boundary from
the surface-color and picture-iteration tools.

Polymorphic colorized paths now receive the polygon drawing order and
type-specific default style that Native writes after specialization. The
surface-color tool creates 12 outputs and round-trips exactly. The image tool
creates 59 outputs, including two point-attached pictures backed by distinct,
sequential resources. Native Sketchpad visibly renders the resulting pattern
and Ctrl+S preserves SHA-256
`e8222056418d906715948403ae19631c31cb0eb161aaf779ca207a3b3620efc3`.
Arbitrary imported multi-picture editing remains read-only; the new writer is
limited to newly materialized point attachments with strict affine validation.
No standalone test case was added or removed. Full corpus scan:
`/tmp/tool_structure_markers_coverage.log`.
