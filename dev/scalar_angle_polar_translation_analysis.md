# Contextual scalar angles in custom polar translations

## Native profile

Six remaining drawing code-26 objects use the already supported custom polar
translation packet: a Point header, exactly three references, no constraint or
auxiliary payload, and ordered operands `[source, angle, distance]`. Their only
difference from the earlier cohort is that the angle calculation reports a
Scalar result category rather than Degrees. Five distance calculations report
Length; the sixth reports Scalar.

This is an operation-context convention rather than a new transform family.
The native formulas include `((if[1] - if[2]) * alpha)` and
`(((3 * 180 degrees) * r) + 2)` in the angle slot. The square-root spiral's
distance slot is `(1 cm + r)`. At its saved state, Native therefore consumes
the displayed magnitudes as 2,486 degrees and 5.6 centimeters. The shared
`NumericExpr::ObjectValue` model already documents and implements this rule:
angle and ratio slots consume numeric magnitudes, while marked-distance slots
interpret Length or Scalar values as signed centimeters.

The decoder now applies the same general numeric-quantity predicate used by
numeric rotation and dilation to the code-26 angle reference. Existing guards
remain unchanged: the output must be a supported Point or Segment, references
must be earlier live objects on the same page, the distance must be Length or
Scalar, the group must use one of the two exact observed record layouts, and no
constraint payload is accepted. Imported `DualMarkedPolar` objects remain
read-only as newly authored records until separate writer evidence exists.

## Geometry and Native evidence

All six imported roots evaluate to finite points. Two nonzero saved states give
independent numeric checks:

- `任意凸四边形的渐开线（田野风）.gsp`: angle
  `441.37660547922985`, distance `3.4477229075020777 cm`, displacement
  `(19.538203560178, -128.834541094981)` pixels.
- `100位根号2.gsp`: angle `2486`, distance `5.6 cm`, displacement
  `(175.468739765980, 118.355159411761)` pixels.

The other four roots are at their zero-angle/zero-distance initial state, then
become nonzero inside their locus evaluation scopes. The public locus evaluator
produced finite sampled geometry for their restored dependent loci, including
128-segment circular-unrolling traces and the 100/100/46-segment cylinder
families.

The restored convex-involute root feeds a `Colorized` point into a direct
`SampledMapped` curve. Native colors the trace from green through blue to
violet, but the port's unstyled sampled path previously flattened it to the
sampled object's fallback navy. Direct sampled mappings now return styled
segments when their output is colorized: every scope evaluates the carried
spectrum color, while the sampled object's saved width and pattern remain in
force. Canvas and SVG share those segments, selection still overrides their
color, and picking retains the existing geometry-only path. A port SVG visibly
matches Native's green-to-blue-to-violet sequence.

Sketchpad 5 under Xvfb/Wine visibly rendered the multicolor convex-quadrilateral
involute, the square-root digit spiral, and the cylinder surface. Saving each
untouched file retained its exact SHA-256:

- `f97f504a2f243a41e095d330ca35375891bb808b949fd6c37988f8e3c045966a`
- `d7a648e831735adc803d5b51a181a0deb9371d8f130f2fbf77d4cbde5e7b77bd`
- `e03a217980efd050e2779363a8f09bbc18aefeea75105bbcedde7f6e4b9074ae`

## Corpus effect

The six code-26 roots restore 30 dependent drawing objects: eight code-35
loci, six code-16 transforms, six code-101 sampled transformations, two code-77
bindings, two code-30 transforms, two code-98 intersections, and one each of
code 75, 89, 90, and 102. Unsupported falls **640->604**, fully decoded
documents rise **1,545->1,551**, drawing-complete documents rise
**1,584->1,590**, and unsupported documents fall **54->48**. Partial views
remain **792**, embedded-tool decoding remains **4,779/5,356**, and all
**1,599** originals remain byte-exact with zero parse failures. No standalone
test case was added.
