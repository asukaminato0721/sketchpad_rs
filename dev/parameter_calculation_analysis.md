# Parameter-dependent calculations

The sibling native `calculation.gsp` fixture was copied to `tests/fixtures/parameter_calculation_chain_native.gsp`. It contains t[1] and five code-48 calculations: t[1], t[1]^2, t[1]^2+1, the same expression in parentheses, and (t[1]^2+1)/2. Each group has a single source reference and a `0907` expression whose u16 tokens are regular infix operators, parentheses, literal digits, and source token `0x6000`. The group grows by two bytes per token.

`ParameterCalculationGroup` reads and writes these tokens with bounded syntax, precedence, and one numeric source. The port accepts a selected parameter's name or `v` for a selected parameter-dependent calculation, plus integer or decimal literals, +, −, ×, ÷, ^, and parentheses, up to 32 tokens. The source parameter is live: editing t[1] from 1 to 2 updates all five imported results. The saved edit opened and saved byte-identically in native Sketchpad 5, showing 2, 4, 5, 5, and 2.5.

Generated `(t[1]^2+3)/2`, `t[1]*12`, `t[1]+3*2`, and `speed^2+1` files opened, displayed the expected values, and saved byte-identically in native Sketchpad. A code-48 group's source reference was changed from t[1] to the preceding `t[1]*12` calculation; native Sketchpad displayed 30.00 for the dependent expression and saved it byte-identically. The port-generated `v+3*2` chain then displayed 30.00, and 42.00 after t[1] changed from 2 to 3; both files saved byte-identically in native Sketchpad. Copy and Paste remap a calculation with its chain of parents. Unary minus, measurement sources, and longer token streams remain to implement.

## Decimal literals

Token `10` represents the decimal point inside numeric literals. A patched expression `t[1]*2.5` displayed 5.00 in native Sketchpad and saved byte-identically. The port now reads and writes decimal literals using digits on both sides of the point, within the existing 32-token bound. It preserves their spelling in the rendered expression and rejects repeated decimal points, missing fractional digits, leading zeros, and initially undefined divisions.

`tests/replay_parameter_decimal_calculation.txt` writes `speed*2.5+0.125` and a dependent calculation divided by `0.5`, then edits speed from 2 to 3. Both generated files opened and saved byte-identically in native Sketchpad. Its results were 5.13/10.25 and 7.63/15.25. The port's parameter-calculation display now matches native rounding at the halfway value 5.125.
