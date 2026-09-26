# Parameter-dependent calculations

The sibling native `calculation.gsp` fixture was copied to `tests/fixtures/parameter_calculation_chain_native.gsp`. It contains t[1] and five code-48 calculations: t[1], t[1]^2, t[1]^2+1, the same expression in parentheses, and (t[1]^2+1)/2. Each group has a single source reference and a `0907` expression whose u16 tokens are regular infix operators, parentheses, literal digits, and source token `0x6000`. The group grows by two bytes per token.

`ParameterCalculationGroup` reads and writes these tokens with bounded syntax, precedence, and one numeric source. The port accepts a selected parameter's name or `v` for a selected parameter-dependent calculation, plus integer or decimal literals, +, −, ×, ÷, ^, and parentheses, up to 32 tokens. The source parameter is live: editing t[1] from 1 to 2 updates all five imported results. The saved edit opened and saved byte-identically in native Sketchpad 5, showing 2, 4, 5, 5, and 2.5.

Generated `(t[1]^2+3)/2`, `t[1]*12`, `t[1]+3*2`, and `speed^2+1` files opened, displayed the expected values, and saved byte-identically in native Sketchpad. A code-48 group's source reference was changed from t[1] to the preceding `t[1]*12` calculation; native Sketchpad displayed 30.00 for the dependent expression and saved it byte-identically. The port-generated `v+3*2` chain then displayed 30.00, and 42.00 after t[1] changed from 2 to 3; both files saved byte-identically in native Sketchpad. Copy and Paste remap a calculation with its chain of parents. Unary minus, measurement sources, and longer token streams remain to implement.

## Decimal literals

Token `10` represents the decimal point inside numeric literals. A patched expression `t[1]*2.5` displayed 5.00 in native Sketchpad and saved byte-identically. The port now reads and writes decimal literals using digits on both sides of the point, within the existing 32-token bound. It preserves their spelling in the rendered expression and rejects repeated decimal points, missing fractional digits, leading zeros, and initially undefined divisions.

`tests/replay_parameter_decimal_calculation.txt` writes `speed*2.5+0.125` and a dependent calculation divided by `0.5`, then edits speed from 2 to 3. Both generated files opened and saved byte-identically in native Sketchpad. Its results were 5.13/10.25 and 7.63/15.25. The port's parameter-calculation display now matches native rounding at the halfway value 5.125.

## Degree-valued parameter calculations

The regular polygon fixture stores `360°/n` with header subtype 0x31 (scalar expressions use 0x33). Literal token 0x0101 appends the degree unit to the preceding numeric literal. The parser/writer preserves these tokens and the header subtype. Addition/subtraction require equal units; scalar multiplication/division preserve degrees; degree/degree division produces a scalar. Unsupported dimensions are rejected. Numeric labels retain the saved precision and degree suffix; degree results can supply Mark Angle and calculated-angle point rotation, while live iteration depths require scalar values, truncated to whole steps.

## Measured angle sources

Code-48 degree expressions also accept a code-41 three-point angle or code-43 arc-angle measurement as their reference source. `tests/replay_calculation_angles.txt` creates an angle, arc angle, angle/2, angle+10°, angle/90° (scalar), arc angle/2, and a point rotation driven by angle/2. Both initial and moved files open and save byte-identically in Sketchpad 5. The initial 90° angle yields 45.00°, 100.00°, and 1.00; moving the third point produces 135° and corresponding 67.50°, 145.00°, and 1.50 results. The regression covers byte preservation, unit checks, live evaluation, rotation, copy/paste remapping, deletion, and a degenerate angle becoming undefined.

Number → Calculate accepts a selected angle or arc-angle measurement as `v`. Its initial expression is `v / 2`; a selected degree calculation uses the same default. New calculation anchors account for preceding measurements and numeric captions, reserving 64 pixels after fraction expressions. Existing saved anchors remain intact.

## Scalar measurement and calculation sources

The same code-48 expression accepts signed point ratios, slopes, normalized Value of Point, Cartesian x/y/coordinate-distance values, scalar length-ratio calculations, and literal integer calculations. Coordinate pairs and equations remain excluded because they do not provide one scalar. Source dimensions are checked during import, creation, evaluation, and save. Expressions use `v` for these sources; calculations retain the full source dependency chain and become undefined if their source does.

`tests/replay_calculation_scalars.txt` builds ratio+1 (used as a carried-segment iteration depth), Value of Point × 4, and a length ratio × 3. Editing its free and constrained points changes the calculated depth from 4 to 5, the point-value result from 1 to 2, and the scaled length ratio from 1 to 0.75. Native Sketchpad opens and saves generated and edited files byte-identically. `tests/replay_calculation_coordinates.txt` exercises x+1, y×2, coordinate distance/2, slope+1, and `(2+3)×2`, with original measurement captions hidden to keep the formula results visible. Native open/save is byte-identical before and after moving the second endpoint; slope+1 changes from 1.00 to 0.60. Regression tests cover live values, undefined slope, coordinate-pair rejection, dependency remapping, copied iteration bindings, deletion, and fractional iteration depth.

New caption placement also accounts for fraction sources and dependent fraction expressions. Saved anchors are preserved.

## Length and area expression dimensions

The one-source expression now covers segment length, point distance, polygon perimeter, circle circumference/radius, arc length, circle/polygon area, and existing length/area calculations. Header subtype 48 denotes length, 49 degrees, 50 area, and 51 scalar. Expression inference preserves dimensions through compatible addition/subtraction, scalar scaling, length×length, area/length, and matching-unit quotients. Dimensional powers 0/1 preserve the corresponding scalar/source dimension; length squared produces area. Unsupported dimensions and fractional dimensional powers are rejected. A native exploratory area^0.5 formula evaluates numerically but does not display cm, so treating it as a length is unverified and remains pending.

`tests/replay_calculation_dimensions.txt` creates `v*1.5+v/2` from a measured length, `v^2`, `v^1`, `(v*v)/v`, and an area ratio, plus a point translation driven by the compound length result. It also creates circle area/2, polygon area×1.25, a chained area result×1.5, and a polygon area ratio. All expressions update after source-point edits. Regression tests check inferred units, mixed-unit and exponent rejection, length-calculation chains, marked-distance translation, copy/paste remapping, and deletion cascades. Generated and moved length/area files open and save byte-identically in native Sketchpad 5.

## Built-in functions

The sibling expression-display decoder identifies tokens 0x2000/1/2 as sin/cos/tan and 0x2006..0x200c as abs/sqrt/ln/log/sgn/round/trunc. The native function argument layout is `[function, 11, argument..., 12, 12]`, including the extra function terminator. A scalar sin(n) prototype at n=30 displayed 0.50 and saved byte-identically, proving degree evaluation for these default calculation records. Its verified fixture is `calculation_sin_scalar_native_verified.gsp`.

The parameter-calculation AST and tokenizer now support these functions and nesting, preserving the 32-token record limit. sin/cos/tan accept scalar or degree arguments and return scalars. abs preserves units. sqrt/ln/log/sgn/round/trunc currently accept scalar arguments. Negative square roots and nonpositive logarithms are undefined. Source edits update the expressions and any dependent geometry, including iteration depth. Tests cover nesting, parsing failures, domains, measured-angle evaluation, length-unit preservation, source edits, dependency deletion, copy/paste, and reopen. Further functions, constants, alternate angle modes, and dimensional roots remain pending.

`tests/replay_calculation_functions.txt` produces scalar calculations and edits, measured-angle functions, measured-length abs, and rounding/truncation examples. Native checks confirm sin/cos/tan, abs, sqrt, ln, measured-angle sin/cos/abs, and length abs render the expected results. These files and the edited scalar file save byte-identically in native Sketchpad 5.

The separate rounding fixture makes ln/log/sgn/round/trunc fully visible. Native Sketchpad displays log(100)=2, sgn(-3)=-1, round(4.5)=5, and trunc(4.5)=4. At n=-30 it displays round(-4.5)=-4 and trunc(-4.5)=-4: the round function uses floor(x+0.5), distinct from decimal caption rounding. Both files save byte-identically. This check corrected the initial use of Rust’s round().

The n=90 probe displayed tan(n)=-2.86×10¹⁴ and saved byte-identically. The calculator uses PI=3.1415926535898 (double bytes at executable offset 0x272690), which produces -286411383293069.44 at 90°. The evaluator uses this verified constant and preserves the finite result instead of imposing a mathematical singularity guard. Decimal literals now use direct decimal-to-f64 conversion; summing decimal places had nudged 0.15 above its native value and incorrectly changed round(-30*0.15) at the half-integer boundary.

## Inverse trigonometric functions and constants

Native token probes verify 0x2003/4/5 as inverse sine/cosine/tangent, 0x000d as π, and 0x000e as e. At n=0.5 the native app displays 30.00°, 60.00°, 26.57°, n+π=3.64, and n+e=3.22. The default inverse-angle calculation uses the same degree conversion constant as the direct functions. Created degree-result groups use subtype 49, allowing marked rotation and point iteration. asin/acos reject values outside [-1,1]; atan accepts any finite scalar. All three reject dimensioned arguments.

`tests/replay_calculation_inverse_functions.txt` produces these expressions, a nested sin(asin(n)), negative source edits, and a carried-segment iteration driven by asin(n) with round(abs(n)*10) depth. Both calculation files and the iteration file render correctly and save byte-identically in native Sketchpad. Editing the saved iteration source to n=1.2 makes asin undefined and removes the dependent transformed point and repeated segments; that file also saves byte-identically. Tests cover domain endpoints, angle/scalar units, nested calls, source changes, rotation/iteration, copy/paste, and reopen. Newly constructed undefined geometry remains subject to the existing save validation; the undefined-source regression edits a reopened valid sketch.

The constant lexer accepts pi or π and e, while preserving selected parameters named pi or e as source references. Parameter matching now respects identifier boundaries, so a parameter named p can use p+pi. Source-dependent groups still require a source reference; source-free formulas now use the separate literal-calculation layout described below.

## Source-free expressions

Literal calculation groups (code 0) use the sequence 07d0/0907/07d8/08a3/07d7 with no 07d2 source reference. The existing 2+3 template supports longer expressions by updating the 0907 record length, inner length, token count, and token array. Replacing its three tokens with π+e displays 5.86 and saves byte-identically in native Sketchpad 5; the verified fixture is calculation_constants_only_native_verified.gsp.

Both calculation layouts now share the lexer and expression parser. Literal groups accept 1–32 tokens, decimals (including .5), constants, functions, degree literals, powers, and unary signs. The old small integer builder remains available and preserves its verified presentation bytes. Source-free results infer their units from the expression, carry degree suffixes, and supply calculations, numeric labels, marked rotation, and iteration depth. Scalar depth validation rejects degree results. Invalid-domain results display undefined; division by zero displays infinity while remaining unavailable as a finite geometry value.

`tests/replay_calculation_source_free.txt` generates compound constants/decimal/functions, an inverse-angle rotation with sqrt(25) iteration depth, and signed-power/domain examples. Native screenshots verify -2²=-4, (-2)²=4, 2^-3=0.125 (displayed 0.13), asin(.5)=30°, and the five carried segments. All three generated files display correctly and save byte-identically. Tests cover expression parsing, units, domains, movement, native import, copy/paste, marked-angle selection, and save/reopen. Generic mathematical caption layout and additional unit/literal forms remain pending.

A second native probe verifies -1/0 displays −∞, ln(0) displays −∞, and 0/0 displays undefined; it saves byte-identically (`calculation_source_free_infinite_native_verified.gsp`). The AST retains raw non-finite results for literal captions and filters them from numeric geometry inputs. Integer literals use decimal-to-f64 conversion instead of a 32-bit accumulator, with the native 32-token expression limit still enforced.

## Multiple selected sources

The native code-48 `07d2` record stores a u32 source count followed by that many u32 object references. Expression tokens `0x6000` through `0x600f` index those references. A two-parameter probe displayed x+y=5 and saved byte-identically (`calculation_multi_scalar_native_verified.gsp`). Both record lengths and the source count must be updated when adding references.

The parser, writer, evaluator, dependency traversal, and clipboard remapping now support two to sixteen numeric sources. Creation uses `v1`…`v16` in selection order and `a`/`b` for the first two. Expressions retain the 32-token limit and infer scalar, degree, length, or area units across all sources. The original one-source and simple two-length templates retain their format. Numeric seed mappings in point/parameter iterations also accept multiple-source scalar formulas, with a directly referenced parameter as the mapped seed. See `dev/iteration_analysis.md` for native evaluation evidence.

`tests/replay_calculation_multiple_sources.txt` creates `sqrt((v1+v2)*v3)`, a two-length mean and product, and `round(v1/v2)` as segment iteration depth. It also combines a measured angle and scalar parameter in `v1/v2` for rotation and `sin(v1)*v2` for depth. All six original/edited files opened and saved byte-identically in native Sketchpad 5. The scalar result changes from 4.47 to 5.00, the length-controlled iteration from two to three segments, and the angle-controlled iteration from two 45° steps to three 30° steps. Native screenshots show that the measured angle, although it includes the iteration seed, stays fixed during each iteration rather than being recomputed from every repeated point.

Regressions cover native byte preservation, all source edits and dependencies, three-input evaluation, mixed-unit rejection and scaling, clipboard remapping, dependent deletion, undefined captions, and the two verified iteration sequences.

## Extended parameter values and decimal parsing

Parameter tokens now accept finite decimal values within the 32-token native limit, including larger integers and fractions with more than two decimal places. Imported precision 0–5 and matching step values are preserved; decimal edits promote display precision up to five without rounding the underlying value. `parameter_extended_range_native_verified.gsp` verifies 1000.12345, −123456789.98765, and a 20-digit integer, including their dependent formulas. Generated and edited extended-range files are replayed by `tests/replay_parameter_extended_range.txt`.

Native captions switch to scientific notation at absolute values >10⁶; a separate native probe preserves fixed notation for 999999.98765 and for tiny values down to 0.000001 at five decimal places. This formatting now applies to parameter, calculation, and scalar measurement captions, including attached numeric labels. Exactly one million remains in fixed notation. Snapshot tables retain their verified ±99.99/two-decimal bounds and reject unsupported values atomically.

`parameter_rounding_boundary_native_verified.gsp` verifies that x=0.15 gives round(−30×x)=−4. Parameter digits now use decimal-to-f64 parsing rather than summing decimal places, avoiding an upward floating point drift across the rounding boundary.
