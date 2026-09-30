# Mapped and high-rate function plots

## Visible mapped bindings

A mapped iteration's carried list is both a dependency closure and the source
catalog for its code-77 presentation bindings. Three remaining drawing bindings
already referenced decoded `MappedPointIteration` parents, and each source was
an explicitly carried `FunctionPlot`. Function plots were already valid inside
the mapped evaluator—downstream points and intersections depended on that—but
the visible binding allowlist and styled collection renderer omitted the plot
itself.

`FunctionPlot` is now a mapped binding kind. Its code-77 header uses the existing
strict stroke-style reader, and each iteration state calls the ordinary
discontinuity-aware `function_plot_segments` evaluator inside the established
mapped scope. The flattened result retains separate segments, so distinct
generations and gaps are never joined. The existing collection path supplies
canvas rendering, SVG, selection highlighting, and picking.

A public evaluator probe at 64 samples produced finite styled geometry for all
three bindings:

- `周期函数5法.gsp`: 378 purple medium-width segments
- `幂函数y=x^α图象图谱.gsp`: 2,264 red thin segments
- `玫瑰线系列.gsp`: 129 green hairline segments

Native Sketchpad visibly rendered the repeated red power graphs and green polar
roses. A fixed-viewport port SVG independently rendered the full green rose
sequence with its native hairline style. All three untouched files saved
byte-identically under Xvfb/Wine:

- `b7c776c1ed5dcf7a5afbdb02c9c9415d1d72fd718a9e7ca084e5c5a7107fb09c`
- `3f70a2942e4c79f6fe690d2e3cefcec82f95844e14c9cb4277c0aa789d488501`
- `c6236cfd1cd16ceb0f50147e27ec8dc03aad3725aaff32026becfe4bd8364808`

## High-rate descriptors

Two otherwise ordinary code-72 plots store 200,000 and 250,000 samples in their
40-byte `0902` descriptors. Their polar and Cartesian mode words, finite ranges,
function parents, coordinate frames, and group layouts already match supported
forms. The old 4,096 check was a port-authored limit rather than a native format
boundary.

Plot descriptors now accept every `u32` sample count of at least two across
decode, validation, editing, copying, and writing. This does not make rendering
unbounded: `function_plot_points`, `function_plot_segments`, `PointCurve`, and
their consumers independently clamp requested evaluation to at most 4,096
samples. The stored value is therefore preserved as native document state
without controlling allocation size.

At a requested 384 samples, the port produced 339 finite polar segments for
`莲花曲线.gsp` and 383 finite Cartesian segments for
`过笛卡尔心形线外一点的切线.gsp`. In the latter, all five ranked axis/plot
intersections evaluate; their four abscissas, eight dependent calculations, and
four numeric coordinate points are defined. Native Sketchpad visibly rendered
the orange polar curve and the red tangent construction, and saved both files
unchanged:

- `366adef8503dcd7579e7b34c8489479b15ca3bc3e365c736b05f9b14872a2d3f`
- `b78dc66f12ee475ddb876b69665e4ec60543f106f5022268d7d967a7285598e6`

## Corpus effect

The mapped cohort restores three code-77 roots. The high-rate cohort restores
two code-72 roots plus five code-98 intersections, four code-65 coordinate
measurements, eight code-48 calculations, and four code-69 plotted points: 26
drawing slots total. Unsupported falls **666->640**, fully decoded documents
rise **1,540->1,545**, drawing-complete documents rise **1,579->1,584**, and
unsupported documents fall **59->54**. Partial views remain **792** and embedded
tool decoding remains **4,779/5,356**. All **1,599** originals remain
byte-exact with zero parse failures. No standalone test case was added.
