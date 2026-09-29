# Nested mapped-iteration capture

After duplicate carried IDs were canonicalized, 18 ready code-89 attempts still
stopped at an unsupported carried object. Eleven of those objects were ordinary
code-77 bindings to an earlier decoded `MappedPointIteration`; the other seven
were unrelated referenced text, coordinate measurement, or opaque formats.

In all eleven cases, Native stores the earlier mapped root, its local carried
graph, and its bindings inside the later mapping's carried packet. The later
code-77 outputs select ordinary geometry or labels, not the inner binding. The
inner binding is therefore a structural scope descriptor: it records the
earlier mapping relationship while the later mapping re-evaluates dependent
objects in the outer state.

Admission is deliberately narrower than adding `IterationBinding` to the generic
mapped-value classifier. A captured binding must precede the outer root, refer
to an earlier valid mapped root on the same page, bind a source in that root's
carried DAG, and have the root explicitly present in the outer carried packet.
Every branch or carried node of the inner mapping that depends on an outer seed
must also be present. The existing nested scoped-evaluation and external-input
rules remain unchanged.

The compiled corpus restores 11 drawing code-89 definitions and 48 code-77
bindings. Unsupported slots fall 1,881→1,822 (1,224 drawing and 598 prototype),
all-kind complete documents rise 1,391→1,401, drawing-complete documents rise
1,426→1,436, unsupported documents fall 208→198, and partial views fall
831→826. All 1,599 originals remain byte-exact. The ready code-89 frontier falls
from 46 roots/376 reachable unsupported slots to 35 roots/317 reachable slots.
Corpus log: `/tmp/sketchpad_code89_nested_capture_coverage.log`; frontier log:
`/tmp/sketchpad_frontier_after_code89_nested_capture.log`.

The smallest recovered source, `堆砌钢管.gsp`, uses an inner four-branch mapping
of three circles and an outer mapping for its numeric count label. Sketchpad 5
under Xvfb/Wine displays piles labelled 3, 9, 18, 30, and 45. The port produces
the same four mapped labels and 102 distinct mapped circles plus the three
original circles; overlapping branch paths account for repeated presentation
instances. Native Ctrl+S preserves all 5,682 bytes with SHA-256
`06f33ecd8f6d40995f7d3cdc872199c9f251e02c2413ff2daeee2d018837f5dc`.
Captures are `/tmp/code89_nested_capture_native_before_save.png` and
`/tmp/code89_nested_capture_port.svg`. No standalone test case was added.
