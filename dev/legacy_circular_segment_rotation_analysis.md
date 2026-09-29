# Legacy circular-segment rotations

## Wire scope

Six drawing code-119 groups use the already recognized legacy numeric-rotation
layout: a 28-byte header, one three-reference record, and a terminator. Four of
them have class 13/subtype 69 and references `[circular segment, center point,
degree value]`. They rotate supported circular segments in two pages of
`课后反思10.gsp`. The shared transform decoder already validates this exact
code-119 layout and evaluates its `NumericRotation`; only the circular-segment
family guard omitted the legacy opcode.

The guard now admits code 119 beside the independently supported circular-
segment transform opcodes. This does not relax the header, record layout,
reference roles, ordering, page, or degree-unit checks. The other two drawing
code-119 groups are point rotations whose code-114 sources remain unsupported,
so they stay opaque.

## Native and corpus evidence

Sketchpad 5 under Xvfb/Wine displays the affected page with both rotated filled
segments in the expected overlapping-lens construction. The port renders the
same A–B–C–D geometry and filled regions through the existing analytic
circular-segment transform path. An untouched Native open/save preserves all
107,434 bytes with SHA-256
`e773a15dfe848aef87c4c644ab2ec85f0981fbcd4a0799cbcfbde23e0a5bdc12`.

The compiled corpus scan restores **four code-119 objects**. Unsupported falls
**793→789**, complete documents/drawings rise **1,503/1,544→1,504/1,545**,
unsupported documents fall **96→95**, and partial views remain **802**.
Embedded-tool coverage remains **4,766/5,356**, and all **1,599** originals
remain byte-exact with zero failures. Full scan:
`/tmp/circular_segment119_coverage.log`.

No new test case was added.
