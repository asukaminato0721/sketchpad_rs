# Sketchpad 5 parametric plots

The [Sketchpad Graph command reference](https://keyonline.keypress.com/gsp-reference-center/menus_graph_plotcurve.htm)
describes a plot from two function definitions, with rectangular or polar
coordinates and a numeric domain. The port presents these choices in Graph >
Plot Parametric Curve.

Native Wine probes selected `f(t)=t` and `f(t)=t²` and saved both modes. The
rectangular result is `tests/fixtures/parametric_rectangular_original.gsp`;
the polar result is `tests/fixtures/parametric_polar_original.gsp`. The
two-function source is
`tests/fixtures/parametric_two_functions_moved_original.gsp`. Its labels were
separated by dragging one formula in Sketchpad, which changed a presentation
word in its code-0 `0x0907` payload from `5` to `49`. Both observed values
decode with the same expression grammar.

The final object is code `126` for rectangular and `127` for polar. Its
`0x07d2` dependencies are `[x-function ID, y-function ID, coordinate-system ID]`.
The 40-byte `0x0902` payload stores two little-endian `f64` domain endpoints
at offsets `0` and `8`, sample count `750` at offset `16`, and a mode word at
offset `20`: `0x30000` rectangular or `0x30002` polar. The remaining bytes
are retained from the native template. The polar example's endpoint agrees
with `r=1, θ=1` radian; polar function output is therefore interpreted in
radians by the current renderer.

The port samples both defining functions, maps each pair through the active
coordinate system, and renders the resulting curve in the desktop view and
SVG. Creating a polar plot without an existing frame creates a polar grid.
Generated rectangular, polar, and `[-1,2]` rectangular-domain files are
preserved in `tests/fixtures/parametric_*_generated.gsp`; each opened with
the expected curve and saved byte-identically in Sketchpad 5. Editing a
defining function updates an imported plot and survives save/reopen.

Double-clicking either a function or parametric curve in the port opens its
numeric domain and sample count properties. Imported code-72/126/127 records
retain their source payload while these three fixed-size fields change.
New plots encode the same fields in their template group. Original and newly
created parametric and function plots with 64 samples and changed domains
opened and saved byte-identically in Sketchpad 5. The verified results are
`tests/fixtures/parametric_range_edited_generated.gsp`,
`parametric_new_range_generated.gsp`,
`function_range_edited_generated.gsp`, and
`function_new_range_generated.gsp`. The renderer limits samples to the stored
count. Smooth expressions no longer lose a segment at a turning point when
the sample count is reduced; reciprocal and tangent pole gaps remain.

Nonnumeric domain expressions, arrowhead properties, and other function types
remain to be implemented.
