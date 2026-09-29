# Wider mapped-iteration seed tables

`指数计算器（李章博）.gsp` contains one valid Native code-89 mapping with 102
simultaneous numeric seeds, one branch, final-only output, and a live depth of
200. Its 101 code-77 bindings display the successive decimal pieces of 2^200.
The former 16-seed admission bound rejected the root even though the packet and
evaluation model can represent the mapping directly.

Decoder and validator admission now use a conservative 128-seed bound while
retaining the 16-branch bound. Existing mapped-evaluation limits still reject
more than 20,000 states at one level or 100,000 generated states overall. This
control has only one state at each step.

Rendering every attached label independently originally replayed the same
200-step mapping 101 times. Visible labels belonging to one mapped root are now
evaluated together. Within each substituted state, the carried numeric DAG is
also materialized once in topological object order, including undefined
results, and discarded before the next substitution. Frozen external inputs
and current seed values keep their existing lifetime. The complete release SVG
export takes about 0.84 seconds; the former repeated evaluation did not finish
within 60 seconds in a debug build.

Sketchpad 5 under Xvfb/Wine and the port SVG display the same value:
`1606938044258990275541962092341162602522202993782792835301376`. The
Native control opens and saves all 154,840 bytes unchanged with SHA-256
`5ec35ca7bb66b32199a6f4bbe558ac7033d472e52fe46740c4357a1133414544`.
Captures are `/tmp/code89_wide_seed_exponent_native_large.png` and
`/tmp/code89_wide_seed_exponent_release.svg`. The earlier mixed Name/Value
control remains byte-identical at the SVG level after batching.

The compiled corpus restores one drawing code-89 root and 101 drawing code-77
bindings. Unsupported slots fall 1,734→1,632 (1,034 drawing and 598 prototype),
all-kind complete documents rise 1,409→1,410, drawing-complete documents rise
1,445→1,446, and unsupported documents fall 190→189. Partial views remain 824,
and all 1,599 originals remain byte-exact. The ready code-89 frontier falls
from 10 roots/229 reachable unsupported slots to 9 roots/127 reachable slots.
No standalone test case was added.
