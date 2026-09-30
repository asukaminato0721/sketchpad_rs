# Function sources in custom-tool calculations

Audited on 2026-09-30. Three embedded prototypes contain the same wide code-48
calculation shape:

- `Samples/个人专栏/钮炳坤作品/500位PI螺旋.gsp`, tool 0;
- `Samples/个人专栏/钮炳坤作品/根号2(钮炳坤).gsp`, tool 0;
- `Samples/热研系列/迭代系列/500位PI螺旋.gsp`, tool 0.

Each tool declares 39 inputs. Local object 45 has 35 references and a 546-byte
`0907` expression record. The expression parser identifies those references as
exactly one Function source followed by 34 Numeric sources. The function is the
already-decoded referenced definition at local object 44; the remaining
operands are generic Numeric tool givens.

## Previous rejection

Abstract code-48 tool calculations are parsed before their numeric givens have
destination units. The contextual branch correctly retained the native
expression and reference IDs, but admitted only parents with a concrete numeric
unit or `ToolInput(Numeric)`. It therefore rejected the referenced function
before consulting the expression's independently parsed source roles.

This was narrower than ordinary code-48 decoding. The shared expression tree
already distinguishes `FunctionSource(index, argument)` from a numeric source,
evaluates supported referenced functions, and rejects a source used in both
roles.

## Bounded correction

Contextual decoding now zips the parsed source-role vector with the exact native
reference vector. A Numeric role still requires a numeric object or generic
Numeric tool placeholder. A Function role requires a supported function kind.
Counts, order, source IDs, expression framing, token parsing, and the strict
placeholder layouts remain unchanged.

Invocation applies the same role check after remapping. Numeric operands provide
their real destination units. A verified function operand occupies its native
source slot with Scalar as the unit placeholder; the expression tree ignores
that slot's magnitude unit and independently validates the function argument,
returning the scalar function result. A non-function object in that role is
rejected transactionally.

At this checkpoint the three prototypes remained partial because of their
unrelated final code-100 objects. The later
[attached-picture checkpoint](tool_attached_picture_analysis.md) decodes and
materializes those final objects, completing invocation of all 69 prototype
objects while retaining the isolated PNG resources as read-only.

## Measured recovery

The corpus scan restores 42 prototype slots: nine code-48 calculations, three
code-71 referenced functions, twelve code-33 objects, nine code-16 objects, and
nine code-29 objects. Decoded prototype coverage rises from 4,962 to 5,004 of
5,356 and unsupported slots fall from 394 to 352. Complete-document and isolated
tool counts do not change. All 1,599 drawings remain complete, all 1,599 source
files round-trip byte-exactly, and no standalone test case was added. Full scan:
`/tmp/tool_function_calculation_coverage.log`.
