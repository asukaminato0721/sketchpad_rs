# Coordinate-system custom-tool given

Audited on 2026-09-30. The corpus contains ten declared
code-0/class-1/subtype-39 givens in ten tools. This checkpoint classifies that
family as an abstract coordinate-system input; it does not treat the
placeholder's `0899` cache as axes, an origin, or a grid scale.

## Category evidence

Native coordinate systems are code 61, class 8, subtype 39. Registered-tool
givens replace the construction class with the standard class-1 placeholder
class while retaining subtype 39. This is the same class/subtype split already
observed for other typed tool givens. All ten subtype-39 placeholders are
dependency-free and use the strict class-1 placeholder record family.

The prototype graph supplies an independent role check. Every supported direct
consumer uses the placeholder in the coordinate-frame position: code 65/66
coordinate measurements, code 69 Cartesian numeric points, code 48 expressions
fed by those measurements, or code 72 function plots. No occurrence is used as
a point, path, scalar, or drawable axis.

The representative Native tool `可见性索引` first asks to match point `A`.
Its normal interactive matching did not expose the frame as a separate prompt
under automation. Reordering or shortening the declaration was retained only as
a scratch diagnostic and is not used as category evidence: Native validates the
declaration structure and its matching order is not a raw `04b4` traversal. The
classification rests on the exact native subtype identity and the exhaustive
consumer roles, not on that ambiguous prompt.

## Representation and binding

Contextual decoding assigns `ToolInput(CoordinateSystem)`. The shared
coordinate-system predicate admits that abstract root only while decoding an
isolated prototype; it does not expose axes or frame geometry. Ordinary drawing
records still require a real `CoordinateSystem(x_axis, y_axis)`.

Engine invocation accepts only a live coordinate-system object on the active
page. The desktop picker uses grid hit testing and returns the actual frame ID.
The placeholder can therefore unlock frame-dependent prototype objects without
inventing any saved geometry, while materialization remaps their frame reference
to the selected destination system.

Four isolated prototypes become structurally complete. Their current runtime
still stops later at the independently guarded coordinate-measurement numeric
caption placement rule; this checkpoint does not widen that layout without a
separate proof.

## Measured recovery

The full 1,599-file scan restores 26 prototype slots: ten subtype-39 roots, five
code-65 measurements, four code-66 measurements, six code-48 calculations, and
one code-69 numeric point. Decoded prototype coverage rises from 4,936 to 4,962
of 5,356 and unsupported slots fall from 420 to 394. Complete isolated views
rise from 36 to 40, and documents with every drawing and prototype kind decoded
rise from 1,559 to 1,562. All 1,599 drawings remain complete, every source file
round-trips byte-exactly, and no standalone test case was added. Full scan:
`/tmp/tool_coordinate_system_given_coverage.log`.
