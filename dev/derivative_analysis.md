# Derivative function records

Sketchpad 5's Number > Define Derivative Function creates an object group with
code `78`. Its `0x07d2` record contains one source function ID, and its
`0x0907` record caches the derivative expression. The cache matters: changing
the source expression bytes without updating the derivative record leaves the
old derivative visible after reopening in Sketchpad 5.

Native fixtures cover `x`, `x²`, `x³`, `sin(x)` in radians, and `sin(x)` in
degrees. They are in `tests/fixtures/function_derivative_*_original.gsp`.
The native direct trig derivative uses a compact three-token `cos(x)` stream;
the parser normalizes it to the same function as the usual five-token stream.
All five fixtures decode as editable documents and round-trip byte-identically.

The port differentiates its supported expression grammar symbolically, writes
the cached expression into a code-78 group, and updates existing derivative
caches when their source formula changes. It offers the native radians/degrees
choice for a trigonometric source when the sketch is in degrees. Both angle
settings retain Sketchpad's formal `cos(x)` derivative of `sin(x)`; the degree
setting does not add a conversion factor. Derivatives can feed Cartesian and
parametric plots.

Cross-open checks with `dev/wine_roundtrip.sh` confirmed these generated
files display correctly in Sketchpad 5 and save byte-identically: a square
derivative (`2·x`), sine derivatives in radians and degrees (`cos(x)`), Cartesian and parametric
plots using the square derivative, and an imported square derivative after
changing its source to `x³` (`3·x²`). Screenshots and temporary `.gsp` files are under
`/tmp/derivative_*_native*` and `/tmp/derivative_*_generated.gsp`.

The current symbolic differentiator covers constants, `x`, unary signs,
arithmetic, constant exponents, `sin`, `cos`, and `tan`, with product, quotient,
power, and chain rules. It rejects variable exponents and expressions whose
result exceeds the supported 256-token limit. The broader native function
language remains to be decoded.

Native code-78 records can contain more than the differentiated function in
their dependency list. The first dependency remains the source function; later
slots are live inputs used by the cached derivative expression. The corpus
example `过笛卡尔心形线外一点的切线.gsp` retains a parameter in its two derivative
caches. Those records now decode, evaluate, display, remap, copy, and serialize
through the same contextual-expression path as referenced definitions. Their
values at `x=1` agree with centered finite differences of the source functions
to about `1e-10`, and the original opens and saves byte-identically in
Sketchpad 5.

This does not yet symbolically differentiate through a referenced function
call. Imported native caches remain authoritative for that case.
