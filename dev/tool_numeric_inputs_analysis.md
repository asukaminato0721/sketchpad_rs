# Native numeric custom-tool givens

This slice establishes the generic numeric input category `code=0, class=11,
subtype=33` in a registered Custom Tool. It does not establish the other numeric
or function given subtypes in the corpus. The earlier given audit counted 248
such subtype33 givens out of 267 class11 givens; these counts describe raw tags,
not the eligibility of every surrounding prototype.

## Independent Native controls

The input drawing was authored through the existing public APIs: named scalar
parameter `t=2` at `(20,20)` and calculation `t+1` at `(20,60)`. Native selected
both objects with Ctrl+A and created “Scalar plus one”, with Script View enabled.
The retained creation file shows the given and the dependent calculation in
Script View; the main drawing still shows `t=2.0`, `t+1=3.00`.

Native changed the parameter's ordinary subtype79 to a code0/class11/subtype33
placeholder and changed the dependent code48 result from subtype51 to subtype33.
The given has exactly `07d0,0907,07d5,07d8,08a3,07d7`, a 94-byte numeric payload
with one zero token, display mode3, and no `07d2` dependencies. Its bbox is
`(10,5)..(18,31)`. The expression template has refs `[1]` and tokens
`[6000,1000,0001]`. The ordered declaration is `[1]`. Native cache words inside
these payloads vary when reopened or when tools are invoked; they are preserved,
not interpreted as numeric values or animation metadata.

The tool was invoked against each of the following concrete inputs. Every listed
result was positively captured, saved by Native, independently reopened, and
saved byte for byte without a change. An ordinary calculator control used the
same source and `+1` for degree, measured length, and measured area.

| Input | Native result | Ordinary result header | Retained fixture |
| --- | --- | --- | --- |
| Scalar parameter 5 | 6.00 | 51 (Scalar) | `embedded_tool_scalar_invoked_native_verified.gsp` |
| Length parameter 5 cm | 6.00 | 51 (Scalar) | `embedded_tool_scalar_on_length_native_verified.gsp` |
| Degree parameter 35° | 36.00 | 51 (Scalar) | `embedded_tool_scalar_on_degree_native_verified.gsp` |
| Segment length 5 cm | 6.00 | 51 (Scalar) | `embedded_tool_scalar_on_measure_native_verified.gsp` |
| Polygon area 5 cm² | 6.00 | 51 (Scalar) | `embedded_tool_scalar_on_area_native_verified.gsp` |

The ordinary controls are retained as
`embedded_tool_numeric_regular_{degree,measure,area}_native_verified.gsp`.
Changing the bound scalar parameter from 5 to 7 in Native updates the result
from 6 to 8; `embedded_tool_scalar_live_edit_native_verified.gsp` retains the
actual edited drawing. Matching PNGs preserve the positive captures. Assets
reside in `tests/fixtures/`.

A separately created “Scalar times two” tool has the same abstract given33 and
abstract result33, but tokens `[6000,1002,0002]`. Bound to the 5 cm segment
measurement, it produces 10.00 cm and ordinary result subtype48. The ordinary
calculator likewise produces subtype48 and 10.00 cm. The tool's Native saved and
independently reopened result is retained as
`embedded_tool_numeric_multiply_length_native_verified.gsp` and its matching PNG.
The same separately created tool, bound to the 35° parameter, produces 70.00°
and ordinary subtype49. Its saved, independently reopened result is retained as
`embedded_tool_numeric_multiply_degree_native_verified.gsp` and its matching PNG.
These controls distinguish dynamic unit specialization from discarding all input units.

## Bounded implementation

Only contextual isolated tool decoding recognizes this given. Ordinary drawings
continue to reject code0/subtype33 as an unsupported construction. The new
`EmbeddedToolInputKind::Numeric` has no numeric unit or value while unbound.
The given validates the exact observed record sequence, payload size, immutable
numeric prefix, one zero token, mode3, and supported precision. Unknown given
subtypes remain unsupported. No cache bytes are converted into numbers.

A contextual code48/class11/subtype33 expression is parsed through the existing
calculation parser after a temporary subtype51 projection. Its source IDs must
match the native reference record exactly, and its parents must be numeric
placeholders or supported numeric constructions. It stays an ordinary
`ParameterCalculation` in the isolated semantic graph. The raw registered tool
bytes in the host document remain unchanged. Abstract prototype inputs cannot
be saved as ordinary drawing objects.

`ParameterCalculationGroup::specialize_tool_sources` remaps bound sources,
infers the expression's result unit, and writes the ordinary native result
subtype. It preserves expression tokens and presentation metadata. The runtime
specializes in dependency order, so a later expression sees the actual unit of
an earlier result. The same shared expression-unit rule now matches the
independent ordinary Native calculator: adding a bare decimal to
Length, Degrees, or Area produces Scalar. Native length subtraction has a separate
retained control (`numeric_length_difference_native_verified.gsp`) that keeps
Length; the addition proof does not broaden subtraction rules. Multiplication still combines units.
A scalar variable remains distinct from a bare decimal; mismatched variable
units remain guarded pending a separate proof.

The corpus also contains raw class11/subtypes51,57,79,80,81,48 givens. Their
category restrictions and placeholder representations are separate work. This
slice does not infer them from ordinary construction headers.
