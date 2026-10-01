# Sketchpad 5 function records: arithmetic and trigonometric expressions

## Original application probes

Wine/Xvfb replay in `dev/function_wine_probe.sh` created these files from the
port's six-object Cartesian coordinate system:

| Fixture | Original command | Added objects | Visible result |
| --- | --- | --- | --- |
| `function_definition_identity.gsp` | Number > New Function, enter `x` | 7: code 0 | Formula `f(x)=x`, no curve |
| `function_plot_identity.gsp` | Graph > Plot New Function, enter `x` | 7: code 0; 8: code 72 | Formula and diagonal curve |
| `function_plot_square.gsp` | Graph > Plot New Function, enter `x^2` | 7: code 0; 8: code 72 | Formula and parabola |
| `function_plot_x_plus_1.gsp` | Graph > Plot New Function, enter `x+1` | 7: code 0; 8: code 72 | Shifted diagonal |
| `function_plot_x_cubed.gsp` | Graph > Plot New Function, enter `x^3` | 7: code 0; 8: code 72 | Cubic curve |
| `function_plot_two_times_x_plus_1.gsp` | Graph > Plot New Function, enter `2*(x+1)` | 7: code 0; 8: code 72 | Compound arithmetic curve |
| `function_plot_x_plus_half.gsp` | Graph > Plot New Function, enter `x+0.5` | 7: code 0; 8: code 72 | Decimal arithmetic curve |
| `function_plot_negative_x.gsp` | Graph > Plot New Function, enter `-x` | 7: code 0; 8: code 72 | Negative slope |
| `function_plot_reciprocal_original.gsp` | Graph > Plot New Function, enter `1/x` | 7: code 0; 8: code 72 | Two branches around the vertical asymptote |
| `function_plot_tan_original.gsp` | Graph > Plot New Function, enter `tan(x)` | 7: code 0; 8: code 72 | Branches separated at tangent asymptotes |

The formula is a code-0 object whose `0x0907` record contains a
presentation wrapper and infix expression words. The observed `x` payload
has 94 bytes, with one final word `0x000f`. The `x²` payload has 98 bytes and
three final words `0x000f, 0x1004, 0x0002`. Digits use words `0` through `9`,
the decimal point is `0x000a`, parentheses are `0x000b` and `0x000c`, and
`x` is `0x000f`. Operators `+ − × ÷ ^` use `0x1000` through `0x1004`.
The expression length at payload offset 36 is the payload length minus 32;
offset 44 stores the word count. The intervening presentation bytes vary
between saves, so `src/function_records.rs` retains the original wrapper,
validates the arithmetic grammar, and replaces the token stream when writing.
The formula group
also has `0x07d5` text, `0x07d8` style, and `0x08a3` label placement records.

The code-72 plot has `0x07d2` references `[formula ID, coordinate-system ID]`
and a 40-byte `0x0902` range record. In both original plots, the range is
`[-5.265208333333335, 5.265208333333335]`, the count is 750 samples, and
the low 16 bits of the mode word select Cartesian plotting. The low bytes at
offsets 24 and 32 select continuous/discrete drawing and arrow/endpoint
visibility while their opaque upper bytes are preserved. Native's editable count range is 20–250000.
A trailing
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

Native sine and cosine probes are preserved as `function_plot_sin_original.gsp`
and `function_plot_cos_original.gsp`. After accepting
the application's angle-unit prompt, `sin(x)` stores words
`0x2000, 0x000b, 0x000f, 0x000c, 0x000c`; `cos(x)` uses `0x2001` in place of
`0x2000`. Tangent uses `0x2002` with the same call shape. The top-level
`0x273a` record changes from `1` to `0` for trig functions when radians are selected.
The parser now recognizes this function-call shape, evaluates in the document's
angle unit, and writes new trig expressions in the selected angle unit.
Generated `sin(x)` and `sin(x)+cos(x)` plots opened with the expected
curves and saved byte-identically in Sketchpad 5.
Choosing No in the native prompt leaves `0x273a` at `1` and draws a shallow
degree-based sine curve; that file is `function_plot_sin_degrees_original.gsp`.
The editor now offers both angle-unit choices. The generated degree-based file
uses the same expression tokens with `0x273a=1`.
The native reciprocal and tangent plots leave visible gaps at their
asymptotes. The renderer, SVG export, and curve hit testing now use separate
sampled segments, omitting a segment when interior evaluations leave the
range of its endpoints. Generated reciprocal and tangent files opened in
Sketchpad 5 and saved byte-identically.

Clicking a displayed formula in the port selects its function definition and
changes the Graph menu to Plot Function. The command appends a code-72 plot
that references the existing code-0
definition, including when the coordinate system is created after the
definition. The first result is preserved as
`tests/fixtures/function_plot_existing_generated.gsp`. Both a plot added to
native `function_definition_identity.gsp` and
a plot added to a fresh document opened in Sketchpad 5, displayed the expected
curve, and saved byte-identically. Number > Define Derivative Function now
creates a code-78 object from a selected function. See
`dev/derivative_analysis.md` for the native record and cross-open checks.

Double-clicking a formula in the port opens its expression editor. Editing an
existing code-0 function replaces only its `0x0907` record, retains the
source wrapper, and updates any dependent code-72 plots. Edits from `x²` to
`x+1` and to `2*(x+1)` opened with the expected lines in Sketchpad 5 and
saved byte-identically. The latter grows the token stream and is preserved as
`tests/fixtures/function_plot_edited_compound_generated.gsp`.
Editing `x²` to radians-based `sin(x)` also opened as a sine curve and saved
byte-identically; that fixture is
`tests/fixtures/function_plot_edited_sine_radians_generated.gsp`.

## Writer proof

`function_records::encode_plot_groups` takes the two original groups for `x`
or `x²`, remaps the code-72 dependencies, and preserves all other bytes. For
other arithmetic expressions it replaces the `0x0907` token stream and its
length fields in the `x` template.
`encode_minimal_plot_document` appends them to the port's six-object frame,
updates the page count, and writes `0x2328`. `dev/function_generate.rs` is a
small standalone driver for this proof. The shared document model now parses,
evaluates, and writes these arithmetic function forms. Number > New Function adds a
definition; Graph > Plot New Function adds a definition and a Cartesian plot.
The native renderer and SVG export sample the same evaluated curve, and the
curve can be selected by hit testing.

The generated `2*(x+1)` and `x+0.5` files opened in Sketchpad 5, showed the
expected formula and curve, and saved byte-identically. The shared writer's
`x²` plot and definition-only `x` files also opened and saved in the original.
Dragging the shared writer's plot origin in the original changed `(199,125)`
to `(228,140)`; the edited file reopened in the port with eight supported
objects. The full Rust suite includes tests for original import, save/reopen,
dependency update, and undo/redo. Run it with `cargo test --all-targets`.

The independent record tests can also be run with:

```sh
rustc --edition 2024 --test src/function_records.rs -o /tmp/function_records_tests
/tmp/function_records_tests
```

Arithmetic expressions in `x` and numeric constants, plus sine, cosine, and tangent, in a Cartesian coordinate
system are supported. Parameters, nested definitions, other trigonometric functions,
other function types, nonnumeric plot ranges, and more complex discontinuities still
need original-file probes and a wider expression parser. Unsupported variants
remain read-only through the conservative document safety check.

## Additional built-ins and variable powers

The sibling `gsp-rs/src/runtime/functions/expr.rs` decoder identifies `abs`,
`sqrt`, `ln`, and `log` as opcodes `0x2006` through `0x2009`. Its native
square-root payload regression also demonstrates compact argument tokens
`0x2007, 6, 0x000c`. The port accepts these compact unary arguments when
reading native payloads and stores explicit, terminated argument groups for
new expressions. Nested calls retain each argument boundary. Unknown opcodes
and incomplete argument terminators remain errors.

`function_extensions_generated.gsp` contains
`sqrt(abs(x))+ln(x^2+1)+log(x^2+1)`. The generated file opened in native
Sketchpad with the expected formula and symmetric curve, and saved
byte-identically (2714 bytes). The parser, evaluator, formula display, expression
writer, and editor all accept these four additional built-ins. Square roots
and logarithms outside their real domains produce undefined values; plotting
skips those segments.

The symbolic differentiator implements the abs, sqrt, ln, and log chain rules
and the variable-exponent rule `d(u^v) = u^v*(v'*ln(u)+v*u'/u)`.
The abs derivative is undefined where its argument is zero. Constant powers
retain their existing rule, including powers of negative bases with integer
exponents. General variable powers use the real logarithm and are undefined
where its base is nonpositive. Numeric-slope regressions cover `x^x`, `2^x`,
`(x+1)^(x-1)`, and the additional built-ins. Document regressions cover edits
from `x^x` to `ln(sqrt(x))`, dependent derivative-cache and curve refresh,
save/reopen, undo, and redo. Generated examples are preserved as
`function_variable_derivative_generated.gsp` and
`function_extended_derivative_edited_generated.gsp`.

Both generated derivative examples opened in native Sketchpad with their
source formula, derivative formula, and both curves visible. Native save was
byte-identical for `function_variable_derivative_generated.gsp` (2974 bytes)
and `function_extended_derivative_edited_generated.gsp` (2990 bytes).
`tests/function_extensions_replay.txt` reproduces all three generated files;
`cargo test --lib function` checks the expression and document regressions.
