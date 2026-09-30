# Transient explicit-path locus drivers

## Native structure

A geometric code-35 locus names a target, an explicit path, a driver point, and
its captured dependency closure. Earlier support assumed that a non-free driver
must already be constrained to the named path. The remaining corpus establishes
a second form: the driver's saved construction can use another path or can be a
transformed, calculated, aliased, or colorized point. Sketchpad temporarily
places that existing point on the locus record's explicit path while evaluating
the sweep, without changing the point's ordinary saved construction.

The importer distinguishes the two forms from the typed driver kind. A point on
the same host—including a point on a compatible shortened segment—uses its
stored constraint. Any other supported point is transient. The existing strict
record profile, target ancestry, range and closure checks still apply. A
transient path must be supported by the established segment, ray, line, circle,
arc, polygon, sector, circular-segment, or bounded point-curve sampler; it must
not be the driver and cannot depend on the driver. Stored-path loci retain their
earlier-before-driver ordering requirement.

Sampling replaces only the driver's kind in the cloned locus scope. Point-curve
paths also receive the same nested `PointCurve` evaluation scope used by an
ordinary point-on-curve driver. External inputs remain frozen by the existing
locus evaluator, so the temporary constraint cannot leak into the document or
an enclosing sampled construction.

## Corpus and runtime evidence

The compiled scan restores 16 drawing code-35 roots and 28 downstream drawing
objects. It also restores 13 code-35 embedded-tool roots. The newly admitted
drawing targets include colorized and transformed points, a path-value point,
an iteration point alias, a point-on-curve, a colorized circle, segments, and a
nested locus. A public evaluator probe produced finite geometry for 15 of the
16 roots; the remaining root is a hidden terminal construction whose target was
undefined at all 32 probed positions. Keeping partial samples and hidden
undefined objects is existing locus behavior, not a widened decoding rule.

Three untouched representatives opened and saved byte-identically in the
supplied Sketchpad 5 under Xvfb/Wine:

- transformed/colorized point locus: `20bdfeb9f12fb6d42c4ac8f888b4638ffd7c8d076e3c05e4a2a6cefccf36020e`
- colorized-circle target: `93a6a62d1ae85fcc5cbb5b5505a8bf36bafb8f530995abccec1cf94b8528ddea`
- nested point-locus construction: `d492c1323fc3410ab82ff7b91264de447ca9aa55dd007516904eb69347119b54`

The first native capture visibly renders the same colored curve recovered by
the port evaluator. The complete corpus remains **1,599/1,599** byte-exact with
zero parse failures. Unsupported slots fall **723->666**, fully decoded
documents rise **1,523->1,540**, drawing-complete documents rise
**1,565->1,579**, unsupported documents fall **76->59**, and partial views fall
**795->792**. Embedded-tool decoding rises **4,766->4,779** of 5,356 slots. No
standalone test case was added.
