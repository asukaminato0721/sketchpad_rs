# Segment dilation and reflection

Native Sketchpad 5 files in `tests/fixtures/fixed_dilation_segment_original.gsp`,
`ratio_dilation_segment_original.gsp`, and `reflection_segment_original.gsp`
establish three segment transforms. Fixed dilation uses code 30, references
`[source segment, center point]`, and has the same 12-byte factor payload as
point and circle dilation. Marked-ratio dilation uses code 32 and references
`[source segment, center, ratio origin, ratio base, ratio target]` with no
constraint payload. Both dilation headers have subtype 14 and marker bytes
`51 5c`. Reflection uses code 34, references `[source segment, mirror]`, and
has subtype 14 with the ordinary segment marker bytes `39 20`.

The port transforms both live endpoints and keeps the result a segment. A
negative dilation ratio moves endpoints through the center. Reflections use
the mirror's live line, including when its endpoints move. The editor and
headless replay commands accept segments for these three actions.

`tests/transform_fixed_dilation_segment_replay.txt`,
`tests/transform_marked_ratio_segment_replay.txt`, and
`tests/transform_reflection_segment_replay.txt` create Rust files and then
move the center, ratio target, or mirror endpoint. Sketchpad 5 opened and
saved all three moved files byte-identically. Native fixtures import without
partial view and re-encode identically; core and editor tests cover the live
dependencies.
