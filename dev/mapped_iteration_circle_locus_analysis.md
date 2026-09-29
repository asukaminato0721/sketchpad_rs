# Mapped circle-locus families

After nested iteration capture was decoded, 23 ready code-89 mappings stopped at
a carried geometric locus. Eighteen are the same bounded family: a line-driven
locus whose sampled output is a circle, usually wrapped in a live HSV color.
Two output arcs, two output points over an existing locus, and one outputs a
filled polygon.

Circle families now participate in the mapped carried DAG and code-77 binding
decoder. Each inner locus retains its own sampling domain. Every sampled circle
is tessellated independently, and a colorized target is evaluated in that
sample's scope, so adjacent circles keep their native color progression. The
binding still owns line width and pattern. Ordinary visible circle and arc
loci use the same styled-stroke path instead of the previous fallback color.

Admission remains narrower than the locus evaluator. Mapped straight and
circle families are accepted; mapped arcs remain guarded because the two corpus
examples repeatedly evaluate deep intersection/transform chains and take 37 to
more than 60 seconds per debug SVG export. Point-on-curve and filled-polygon
families also retain their separate unsupported semantics.

The compiled corpus restores 18 drawing code-89 definitions and 38 code-77
bindings. Unsupported slots fall 1,822→1,766 (1,168 drawing and 598 prototype),
all-kind complete documents rise 1,401→1,405, drawing-complete documents rise
1,436→1,441, unsupported documents fall 198→194, and partial views fall
826→825. All 1,599 originals remain byte-exact. The ready code-89 frontier
falls from 35 roots/317 reachable unsupported slots to 17 roots/261 reachable
slots. Corpus log: `/tmp/sketchpad_code89_circle_locus_coverage.log`; frontier
log: `/tmp/sketchpad_frontier_after_code89_circle_locus.log`.

The representative `五圆极限集[41].gsp` contains five independently colored
circle families. Sketchpad 5 under Xvfb/Wine and the port both show the same
red, blue, cyan, green, and yellow concentric-circle groups. Native Ctrl+S
preserves all 28,150 bytes with SHA-256
`eb6f29c4821fdbcedf17e018a7b05faffeed15a1ce219322cf9af144d45a9af8`.
Captures are `/tmp/code89_circle_locus_native_before_save.png` and
`/tmp/code89_circle_locus_port.svg`. No standalone test case was added.
