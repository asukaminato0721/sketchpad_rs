# Parameter-dependent calculations

The sibling native `calculation.gsp` fixture was copied to `tests/fixtures/parameter_calculation_chain_native.gsp`. It contains t[1] and five code-48 calculations: t[1], t[1]^2, t[1]^2+1, the same expression in parentheses, and (t[1]^2+1)/2. Each group has a single source reference and a `0907` expression whose u16 tokens are regular infix operators, parentheses, literal digits, and source token `0x6000`. The group grows by two bytes per token.

`ParameterCalculationGroup` reads and writes these tokens with bounded syntax, precedence, and a single parameter source. The port accepts a selected parameter's name, integer literals, +, −, ×, ÷, ^, and parentheses, up to 32 tokens. The source parameter is live: editing t[1] from 1 to 2 updates all five imported results. The saved edit opened and saved byte-identically in native Sketchpad 5, showing 2, 4, 5, 5, and 2.5.

Generated `(t[1]^2+3)/2`, `t[1]*12`, `t[1]+3*2`, and `speed^2+1` files opened, displayed the expected values, and saved byte-identically in native Sketchpad. Copy and Paste remap a calculation with its parameter parent. Expressions with other calculation sources, decimal literals, unary minus, named measurements, and longer token streams remain to implement.
