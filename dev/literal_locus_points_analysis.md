# Literal points on ordinary point loci

## Corpus boundary

A fresh drawing-only dependency audit at the 2,821-slot baseline found four
unsupported code-123 roots whose sole parent was an already-supported ordinary
point-output `Locus`. They occur in two files and reach 76 distinct unsupported
objects including the roots. Every group has the same strict native signature:

- 28-byte code-123 class-1/subtype-56 header;
- one counted `0x07d2` reference to the earlier same-page locus;
- one 12-byte `0x07d3` payload containing tag 123 and a finite `f64`;
- terminator, with no additional records.

The stored values are `1` and `2/3`. The existing point-curve evaluator already
defines the normalized numeric domain used by code 95 and by code 123 on
intrinsic sampled curves. Admission is deliberately limited to `SampledMapped`
and ordinary geometric `Locus` hosts. The separately observed code-123
`ParameterLocus` root is excluded because its raw numeric domain has not been
proved; FunctionPlot literals remain excluded for the same reason.

## Implementation and regression

`curve_points::literal_supported` centralizes that narrow host rule. Decode,
geometry, undefined-state validation, authoring, and save/reopen all use the
same predicate. The existing midpoint-locus regression was extended with a
`2/3` literal point, so no additional test function was introduced.

## Native verification

The public writer added a literal `2/3` point to the existing midpoint-locus
fixture, producing `/tmp/sketchpad_locus_literal.gsp` (2,456 bytes, nine
objects). Wine's file-association launch opened it directly in Sketchpad 5 under
Xvfb. The native canvas visibly placed the new red point at two-thirds of the
magenta midpoint locus. The window title named the generated file.

After assigning X focus to the Sketchpad window, native Ctrl+S produced an exact
copy. Both source and native-saved files have SHA-256
`23c703216253a74ef1459ee9114190c027f4cd1ad507a931174a235d92f915eb`.
The retained screenshot is `/tmp/sketchpad_locus_literal_start.png`.

## Measured result

The complete 1,599-file scan restores exactly the four roots and their 72
descendants. Unsupported slots fall 2,821→2,745 (2,147 drawing plus 598 tool
prototype slots). All-kind-complete documents rise 1,272→1,274 and
drawing-complete documents rise 1,302→1,304. Partial views remain 865. Every
original remains byte-identical and there are zero import failures. Full log:
`/tmp/sketchpad_locus_literal_full_coverage.log`.
