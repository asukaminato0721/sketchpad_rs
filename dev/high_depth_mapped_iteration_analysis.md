# High-depth general mapped iterations

## Admission boundary

The last two unsupported drawing code-76 roots use the established general
`MappedPointIteration` packet and evaluator. Their exact 24-byte `090a`
definitions differ from already admitted records only in the fixed-depth word:

- `量角器的刻度迭代（hjx4882）2.gsp` stores depth 179 for one scalar seed and
  one branch.
- `角谷猜想（优化）.gsp` stores depth 800 for two scalar seeds and one branch.

The general mapped evaluator already accepts live depths through 4,096 and
enforces independent limits of 20,000 states per generation and 100,000 total
generated states. Imported fixed depths now use the same 4,096 ceiling in
decoding, validation, evaluation, and regenerated packet caches. The public
desktop authoring path and specialized iteration families continue to use the
existing 1–128 limit.

No packet shape, seed type, branch graph, or carried-construction guard was
relaxed. Both mappings use one state per generation, so the raised depth does
not increase branch width.

## Rendering and Native evidence

The protractor mapping evaluates 179 images and exposes its carried segment and
numeric-label bindings. The port SVG contains 182 line elements and 197 text
elements overall, including the full 1–180 scale. Its geometry and labels match
the native semicircular protractor.

The depth-800 Collatz root terminates early when its repeated two-scalar state
deduplicates. It unlocks an endpoint alias, point ratio, two live-depth mapped
iterations, and their colorized circle/disk and numeric-label bindings. The port
produces the native sequence beginning `1541, 4624, 2312, 1156` and ending
`16, 8, 4, 2, 1`, wrapped into ten columns. The headless SVG probe's diagnostic
auto-fit selected its documented maximum 8x scale because the ordinary visible
bounds are small; inverse viewport coordinates retain the native 75.795-unit
column spacing and 7.559-unit circle radius.

Sketchpad 5 under Xvfb/Wine visibly rendered both untouched originals. Saving
them retained their exact SHA-256 values:

- `91bbd1a80bc1a983ce24ef75ecdb8bede9e617a3686c634725c89c0958339b4f`
- `abce071097c876be745610e25ef59e7ce66265ffe6a1b56c874d4a6caba23c92`

## Corpus effect

The two code-76 roots restore ten code-77 bindings, two code-89 mappings, and
one each of codes 47, 48, and 88: 17 drawing objects. Unsupported falls
**604->587**, fully decoded documents rise **1,551->1,553**,
drawing-complete documents rise **1,590->1,592**, unsupported documents fall
**48->46**, and partial views fall **792->790**. Embedded-tool decoding remains
**4,779/5,356**, and all **1,599** originals remain byte-exact with zero parse
failures. No standalone test case was added.
