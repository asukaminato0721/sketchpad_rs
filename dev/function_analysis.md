# Sketchpad 5 function records: initial verified subset

## Original application probes

Wine/Xvfb replay in `dev/function_wine_probe.sh` created these files from the
port's six-object Cartesian coordinate system:

| Fixture | Original command | Added objects | Visible result |
| --- | --- | --- | --- |
| `function_definition_identity.gsp` | Number > New Function, enter `x` | 7: code 0 | Formula `f(x)=x`, no curve |
| `function_plot_identity.gsp` | Graph > Plot New Function, enter `x` | 7: code 0; 8: code 72 | Formula and diagonal curve |
| `function_plot_square.gsp` | Graph > Plot New Function, enter `x^2` | 7: code 0; 8: code 72 | Formula and parabola |

The simple formula is a code-0 object whose `0x0907` record contains a
presentation wrapper and postfix expression words. The observed `x` payload
has 94 bytes, with one final word `0x000f`. The `x²` payload has 98 bytes and
three final words `0x000f, 0x1004, 0x0002`. The intervening presentation bytes
vary between saves, so `src/function_records.rs` retains the original record
template and identifies only these exact postfix variants. The formula group
also has `0x07d5` text, `0x07d8` style, and `0x08a3` label placement records.

The code-72 plot has `0x07d2` references `[formula ID, coordinate-system ID]`
and a 40-byte `0x0902` range record. In both original plots, the range is
`[-5.265208333333335, 5.265208333333335]`, the count is 750 samples, and
the low 16 bits of the mode word select Cartesian plotting. A trailing
`0x2328` record names the last object (ID 8), while `0x232d` still names the
coordinate system (ID 6).

Dragging the coordinate origin in a Rust-assembled `x²` file from screen
`(259,194)` to `(300,220)` moved the original app's parabola vertex to the
new origin. The saved file is `function_port_square_moved_wine.gsp`; the
standalone decoder still reads its expression and dependencies. This confirms
that the curve is evaluated from the coordinate-system dependency, rather
than saved as fixed sample points.

The sibling corpus has `../gsp-rs/tests/fixtures/未实现的系统功能/函数.gsp`, which
contains an additional code-71 function definition with multiple references.
Its nested and parameterized expression format is outside this subset.

## Writer proof

`function_records::encode_plot_groups` takes the two original groups for `x`
or `x²`, remaps the code-72 dependencies, and preserves all other bytes.
`encode_minimal_plot_document` appends them to the port's six-object frame,
updates the page count, and writes `0x2328`. `dev/function_generate.rs` is a
small standalone driver for this proof; the shared editor model does not yet
expose function objects.

Both generated files opened in Sketchpad 5, showed the expected formula and
curve, and saved with **zero changed record slots**. Five standalone tests
cover expression rejection, original and Number-command decoding, generated
document reparse, and the edited original file. Run them with:

```sh
rustc --edition 2024 --test src/function_records.rs -o /tmp/function_records_tests
/tmp/function_records_tests
```

Only `x` and `x²` in a Cartesian coordinate system are safe to encode here.
General arithmetic, parameters, nested definitions, trigonometric functions,
polar/parametric plots, and function editing still need original-file probes
and a wider expression parser before integration.
