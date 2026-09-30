# Sampled-curve direction arrow

Audited 2026-09-30 against the complete 1,599-file corpus and Native Sketchpad
5 under Xvfb/Wine.

## Scope

The last drawing code-121 record is object 35 in `三角形旋转(gjq).gsp`.
Unlike the 27 previously decoded congruence and direction marks, its sole host
is object 33, a decoded `SampledMapped` point curve rather than a Segment. Its
wire packet is otherwise the established direction-arrow form: normalized
parameter 1.0, kind `0x00010001`, and forward flag 1.

Import admits this extension only for an earlier, same-page, independently
supported `SampledMapped` curve. Tick marks remain Segment-only. Presentation
evaluates the curve at the stored parameter and estimates its local forward
tangent from bounded neighboring samples, using a one-sided difference at an
endpoint. The existing ten-unit arrow arms are then oriented by that tangent.
Undefined or locally degenerate curve geometry suppresses the arrow rather than
producing non-finite strokes.

The sampled-curve profile remains import-only. A clipboard experiment copied
the arrow with its complete 19-object ancestry and round-tripped inside the
port, but Native Sketchpad rejected that generated graph with its File Problem
dialog. The writers therefore retain their proved Segment-host boundary; an
unchanged imported group continues through the preserving serializer.

## Evidence

Native Sketchpad visibly renders the original arrow on the red sampled trace.
The port SVG places the same two black arrow arms at the evaluated endpoint and
matches the native direction. The untouched original opens and Ctrl-S saves
byte-for-byte in Native with SHA-256
`7230e871d3ee534ac8598e925ef352b66623b13973261651189660f07eba5968`.

The compiled full scan restores the one code-121 slot. Unsupported slots fall
**580→579**, complete documents rise **1,556→1,557**, drawing-complete
documents rise **1,596→1,597**, unsupported documents fall **43→42**, and
partial views fall **788→787**. All **1,599** files remain whole-file exact with
zero import failures. Embedded-tool coverage is unchanged at **4,779/5,356**.
No standalone test case was added.

Full scan: `/tmp/code121_coverage_all_tests.log`. Native capture:
`/tmp/code121_native_before_save.png`. Port render: `/tmp/code121_port.svg`.
