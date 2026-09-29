# Transformed cropped pictures

## Native layout and model

Eleven drawing code-29 groups rotate a cropped picture rather than a complete
picture: one in `一款精致的小车广告.gsp` and ten in `广告片断.gsp`. Each uses
the strict numeric-rotation layout already supported for pictures: a 28-byte
header, a three-reference record `[source, center, degree value]`, a 16-byte
`090c` picture cache, an optional empty `232e`, and a terminator. In every
case, the cache exactly matches the source crop cache. The car document also
contains one downstream code-16 vector translation whose source is the rotated
crop and whose cache remains identical.

A transformed crop is represented separately from a transformed full picture.
Evaluation recursively composes the source pixel transform with the native
affine operation and applies the same operation to the live clipping polygon.
This preserves the crop dependency instead of widening it into a transformed
copy of the complete PNG. Admission remains bounded by the existing transform
header, record, reference-role, ordering, page, unit, and finite-matrix checks;
the crop-chain cache must also equal the exact source cache. Plain transformed
pictures retain their prior width/height cache validation.

Existing objects remain byte-preserved. Creating a new transformed crop is
explicitly refused by the writer because the generic transform encoder does not
emit the required picture cache.

## Rendering and Native evidence

Native picture and filled-region headers share a back-to-front rank. In the car
document, the gray source picture is rank 21, the yellow picture rank 22, the
hidden half-car polygon rank 23, its visible crop rank 24, the wheel polygon
rank 25, its hidden crop rank 26, and the two transformed copies ranks 27 and
28. SVG export now follows that shared interior stack. Cropped SVG images use a
screen-space clipping group, avoiding the image transform being applied twice
to the clip polygon. The live canvas orders picture meshes by the same saved
picture rank.

Sketchpad 5 under Wine displays the intended yellow left half, gray right half,
and two gray wheels. The port SVG reproduces that composition. An untouched
Native open/save preserves all bytes with SHA-256
`40d7b467ec4cfed98af88d36c4a4c258502c5134802e5214d9ee1124f32ffb73`.

## Corpus result

The decoder restores **eleven code-29 roots and one code-16 dependent**.
Unsupported falls **789→777**, complete documents/drawings rise
**1,504/1,545→1,506/1,547**, unsupported documents fall **95→93**, and partial
views remain **802**. Embedded-tool coverage remains **4,766/5,356**, and all
**1,599** originals remain byte-exact with zero failures. Full scan:
`/tmp/transformed_crop_final_full_coverage.log`.

No new test case was added.
