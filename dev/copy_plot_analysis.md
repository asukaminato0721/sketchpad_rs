# Copying saved function plots

`tests/copy_plot_replay.txt` copies a saved Cartesian function plot with its
coordinate frame and definition, and a saved derivative with its source
definition. The resulting files are
`tests/fixtures/function_plot_copied_generated.gsp` and
`tests/fixtures/function_derivative_copied_generated.gsp`.

The clipboard includes dependencies and now also carries per-frame grid style.
Without the copied style, the saved file reparsed with an extra frame style and
Paste correctly rejected the edit. Both files now reopen as fully editable
documents in the port. Sketchpad 5 opened and saved each byte-identically.

The function display position has two records. The `0x0907` expression payload
stores its `(x, y)` anchor at offsets 12 and 14. Function definitions also
carry a `0x08a3` text rectangle; all four coordinates must shift with the
anchor. The native app's own Copy/Paste moves these coordinates by `(20, 20)`.
The port now does so for copied definitions and derivatives, and native
screenshots confirm the duplicate formulas appear separately. The Rust canvas
and SVG exporter use the saved expression anchor. The native copy sample is
`tests/fixtures/function_plot_copied_native.gsp`.

The native copy sample now reparses as a fully editable document after the
function expression decoder accepts its moved anchor. Moving the copied
frame's origin in the Rust port shifted the copied curve, and Sketchpad 5
opened and saved `tests/fixtures/function_plot_copied_native_moved_generated.gsp`
byte-identically.

Selecting the function formula in Sketchpad also copies its plotted curve and
the curve's parents. Its duplicated origin and formula stay visible, while the
copied frame's axis and coordinate-system groups are hidden. The port now
includes a selected function's Cartesian plots and applies those visibility
flags to copied frame parts. A hidden vertical unit needs its `0x40000`
presentation flag cleared: combining that flag with the hide bit made the
generated file fail to open in Sketchpad. With the native header value, the
selected-function replay opens and saves byte-identically in Sketchpad and
shows the same two formulas and curves without duplicated axes. The selected
function and selected plot cases currently generate identical files.

Selecting either source formula of a native rectangular parametric curve also
copies both formulas, the curve, and a hidden frame. The same replay now works
in the port. `tests/fixtures/parametric_function_copied_native.gsp` records
Sketchpad's result; `tests/fixtures/parametric_function_copied_generated.gsp`
is the port's result. Sketchpad opened and saved the generated file
byte-identically, with the copied curve and formulas visible and the duplicate
axes hidden. Function names come from the `0x07d5` label record. When one of
the two parametric sources is selected, Sketchpad keeps the unselected copied
formula named `f` and renames the selected one to `g`; the port now writes and
displays those names. Selecting the second source has its own native and
generated fixtures and also opened and saved byte-identically in Sketchpad.
Other graph constructions and native clipboard ordering remain to be checked.
