# Points on parametric curves

## Native layout and domain

Five drawing code-15 points in `直线与曲线.gsp` and `眼中的佛头.gsp`
reference Cartesian code-126 parametric plots. Their strict constraint payload
is tag `15`, subtype `99`, followed by one finite `f64`. The observed values are
normalized curve fractions (`0.25302896206549275` and
`0.37742897295290717`), not raw plot-domain parameters.

The shared point-curve evaluator now treats a parametric plot as two functions
evaluated at `min + (max - min) * fraction`, transformed through the saved
coordinate frame. Cartesian plots use the two values directly; polar plots use
the first as radius and the second as radians, matching the existing renderer.
Only free code-15 points gain this admission. Code-95 live values and code-123
literals retain their independently proved host sets.

Adding parametric plots to the existing curve-intersection target set also
restores four circle/parametric code-98 intersections. Plot/plot intersections
remain opaque and require a separate two-curve ordering contract.

## Native and corpus evidence

Sketchpad 5 under Xvfb/Wine shows the points and circle intersections on the
parametric outline after `Show All Hidden`. The original `眼中的佛头.gsp`
opens with the expected document title and saves byte-for-byte with SHA-256
`6cea4c774f57b8dda6bd6623be6ce549071d5c9714f8fd28a9615c34dfd2035b`.
The port evaluates the two labeled source points at `(428.08028443516895,
369.5760565386558)` and `(542.2252820756023, 469.76682636958674)` in saved
canvas coordinates.

The compiled scan restores **12 drawing slots**: five code-15 roots, four
code-98 intersections, and three ordinary dependents. Unsupported falls
**843→831**; complete document/drawing and partial-view counts remain
**1,496/1,537** and **802**. All **1,599** originals remain byte-exact with zero
failures, and embedded-tool coverage is unchanged. Full scan:
`/tmp/parametric_curve_points_coverage.log`.

No new test case was added.
