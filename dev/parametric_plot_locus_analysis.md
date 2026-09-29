# Parametric-plot loci

## Scope and profile

Thirteen ready code-35 drawing loci use a code-126 Cartesian parametric plot as
their path and a code-15 `PointOnCurve` point as their driver. Native stores the
same normalized `0..1` range and open-curve profile already established for
function-plot loci. The shared point-curve evaluator already maps that fraction
through the parametric plot's saved domain and coordinate frame.

The locus classifier now admits `ParametricPlot` beside `FunctionPlot` for this
existing profile. It does not add another evaluator, relax reference ordering,
or change the closed-path, sampled-mapped, nested-locus, or shortened-segment
profiles. Plot pairs which do not satisfy an independently supported
intersection profile remain opaque.

## Native and corpus evidence

Sketchpad 5 under Xvfb/Wine shows the parametric construction and its hidden
geometry in `眼中的佛头.gsp` after `Show All Hidden`. The port renders the same
document with the newly decoded traces evaluated through the shared parametric
curve path. The original opens and saves byte-for-byte in Native with SHA-256
`6cea4c774f57b8dda6bd6623be6ce549071d5c9714f8fd28a9615c34dfd2035b`.

The compiled corpus scan restores **13 code-35 loci** and **eight code-98
intersections** which depend on them. Unsupported falls **814→793**; complete
documents/drawings rise **1,501/1,542→1,503/1,544**, unsupported documents fall
**98→96**, and partial views remain **802**. Embedded-tool coverage remains
**4,766/5,356**, and all **1,599** originals remain byte-exact with zero
failures. Full scan: `/tmp/parametric_locus_coverage.log`.

No new test case was added.
