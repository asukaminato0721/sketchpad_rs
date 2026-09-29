# Function-plot intersections

## Scope and ordering

Ten ready drawing code-98 records pair two `YFromX` function plots in the same
coordinate frame. Nine payloads use ranks `0..4`; one older packet stores the
high value `102`. The existing native-hint policy clamps a rank beyond the live
crossing count to the final crossing, so this packet remains meaningful when
its referenced functions produce distinct curves and remains undefined while
they overlap.

The shared curve-intersection evaluator now supports this exact pair profile.
It evaluates both functions over their overlapping x-domain, orders proper
crossings by increasing x, refines sign changes by bisection, rejects tangencies
and overlapping spans, and applies the function renderer's pole-gap guard.
Different frames, `XFromY`, polar, parametric, sampled, and locus pairs remain
outside this admission.

Referenced functions can have large invariant numeric dependency trees.
Recomputing those values per root sample made the initial implementation take
tens of seconds. The final evaluator freezes those numeric inputs once through
the existing scoped evaluation context, reducing all eight crossings in the
largest corpus control to under one second in an unoptimized build.

## Native and corpus evidence

A fresh port-authored control plots `y=x` and `y=x²`, then adds both ranked
intersections. The port evaluates graph coordinates `(0,0)` and `(1,1)`.
Sketchpad 5 under Xvfb/Wine displays both red intersection points in that order,
opens the generated file with the expected title, and saves it byte-for-byte
with SHA-256
`dd57fcda4bd6b5ee1ab7df7114a77cf3ca92d913f09bf771894cc1fe4badfc0f`.

The compiled corpus scan restores all **10** ready code-98 roots and **four**
dependents. Unsupported falls **831→817**; complete documents/drawings rise
**1,496/1,537→1,499/1,540**, and unsupported documents fall **103→100**.
Partial views remain **802**, all **1,599** originals remain byte-exact with zero
failures, and embedded-tool coverage is unchanged. Full scan:
`/tmp/function_plot_intersections_coverage.log`.

No new test case was added.
