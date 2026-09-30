# Legacy numeric custom-tool givens

Audited on 2026-09-30. The corpus contains six declared class-11/subtype-51
givens and one class-11/subtype-48 given. Both retain source-free Calculation
groups rather than the newer subtype-33 zero-token placeholder body.

## Native matching categories

Temporary corpus copies changed only the complete `04b4` input order, retaining
every input and every prototype group. With subtype51 first, Native displayed
`1. 匹配度量值 音长`. With the already-supported subtype33 first in the same
tool, it displayed `1. 匹配度量值 音名`. The identical Measurement Value wording
establishes one generic destination category despite the distinct preserved
encodings.

Putting subtype48 first in `三角形在正方形内的滚动.gsp` produced the narrower
prompt `1. 匹配距离度量值 t[1]`: Distance Measurement Value. It therefore carries
a known Length restriction rather than sharing the unit-unknown Numeric root.

## Bounded representation and binding

Subtype51 maps to the existing `Numeric` category. Its exact source-free
Calculation group must pass the ordinary strict parser, but the stored literal
is not treated as the destination value or unit. Invocation specializes from
the actual finite numeric object, and the existing caption picker applies.

Subtype48 maps to a distinct `Distance` category whose prototype numeric unit is
Length. Its body also stores source-free literal zero. Because that inert literal
is scalar, structural validation temporarily projects only the header subtype
to generic 51 before invoking the same strict parser. The original group remains
byte-preserved. Runtime binding and caption picking require a finite object for
which `is_distance_value()` holds.

## Measured recovery

The six subtype51 roots and one newly reachable code-71 function restore seven
slots. The subtype48 root restores 24 more: two code-48 calculations, four
code-16 objects, two code-95 plots, five code-29 objects, five code-26 objects,
one code-24, two code-27, one code-35, and one code-8 object.

Combined prototype coverage rises from 5,026 to 5,057 of 5,356 and unsupported
slots fall from 330 to 299. All-kind-complete documents rise from 1,569 to
1,570; isolated-view completeness stays 47/56 because the distance prototype
retains an independent structural-safety restriction. All 1,599 drawings remain
complete, every original round-trips byte-exactly, and no standalone test case
was added. Full scan: `/tmp/tool_legacy_numeric_distance_given_coverage.log`.
