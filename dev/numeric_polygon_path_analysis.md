# Calculation-driven polygon paths (native code 95)

The sibling `10福建宁德26题2(蚂蚁).gsp` has a native code 95 point N at
object 17, with scalar calculation 16 and polygon 15 as its parents. Its actual
polygon vertices are `[11,10,9,10]`: the repeated vertex is deliberate and the
path traverses each listed edge, including the final edge back to the first
vertex. The original source is retained as
`tests/fixtures/numeric_polygon_path_ningde_original.gsp`; its independent
unsupported object families remain unsupported. The new regression verifies
these exact dependencies, label, live scalar-derived position, and byte-identical
untouched save rather than substituting an axis for the original polygon.

The existing code 95 label/reference schema was already verified separately.
These probes rebind its exact native 108-byte group to an independent scalar
Parameter and a rectangle `(100,100),(300,100),(300,200),(100,200)` whose perimeter
is 600. A separate native segment marks the point location. No code 95 presentation
or constraint fields are invented or changed for the probe.

Native positions establish the parameter semantics:

| Input | Native point | Meaning |
| --- | --- | --- |
| 0 | (100,100) | First vertex |
| 0.25 | (250,100) | 150 units along the 600-unit perimeter |
| -0.25 | (150,200) | Same location as 0.75 |
| 1 | (100,100) | Wraps to the first vertex |
| 1.25 | (250,100) | Same location as 0.25 |

For the repeated-vertex polygon `[1,2,3,2]` on the same vertices, the perimeter
remains 600 but its third/fourth edges retrace earlier edges. Native `-0.25`
places the point at `(250,100)`, confirming that the full listed path contributes
to perimeter distance. This differs from the simple rectangle's `(150,200)`.
The smaller diagnostic files and the original larger rectangle probes all
cross-open and save byte-identically. Every comparison occurs after a successful
`dev/wine_roundtrip.sh` exit. Visible marker captures use
`/tmp/polygon_path_{quarter,negative,over}_diagnostic_before_save.png` and
`/tmp/polygon_path_{zero,one,repeated}_domain_before_save.png`; file acceptance
alone is not used as evidence of a defined point.

Code95 now evaluates supported polygon hosts with the existing normalized
perimeter helper, using the finite scalar source's value modulo 1. The existing
live polygon evaluator supplies vertices, including transformed polygon hosts,
so source geometry edits recalculate both edge lengths and point position.
An undefined source or perimeter gives no point. The scalar unit and same-page
reference checks remain enforced. The API validates the active page, same-page
host, finite value, and defined perimeter before adding its ordinary typed
PathValuePoint. New predicates `is_value_plot_path_kind` and
`value_plot_path_accepts` apply specifically to code 95; code 123 literal-path
points keep their original supported hosts and domain checks.

`native_calculated_polygon_path_uses_live_wrapped_perimeter_values` exercises the
actual imported native group, out-of-range wrapping, save/reopen, label retention,
a deformed live host, scalar calculation sources, authored API construction,
clipboard dependency remapping with the standard 20-pixel offset, and dependent
deletion. It also confirms literal polygon-path creation remains unavailable.
`original_ningde_calculated_point_retains_polygon_and_scalar_dependencies` retains
the complete named native source and verifies its original code 95 parents.

`tests/numeric_polygon_path_replay.txt` creates a fresh polygon, named parameter,
scalar calculation `p/2`, plotted point, and marker segment through the actual
Rust APIs. It also edits the source to produce a negative wrapped value.
Both actual API outputs cross-open and save byte-identically in native Sketchpad:
the initial `p=0.50`, calculation 0.25 file is 2,626 bytes, and the edited `p=-0.50`,
calculation -0.25 file is 2,628 bytes. Before-save captures
`/tmp/numeric_polygon_path_generated_port_before_save.png` and
`/tmp/numeric_polygon_path_negative_port_before_save.png` show the live calculation
and corresponding marker endpoints. Both commands exited successfully before
byte comparison.
