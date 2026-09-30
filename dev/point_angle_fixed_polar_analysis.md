# Point-angle fixed polar translation

Audited 2026-09-30 against the complete 1,599-file corpus and Native Sketchpad
5 under Xvfb/Wine.

## Scope

Exactly two drawing objects use native transform code 22. Both are Point
outputs with four references in the order `[source, first, vertex, last]` and
one 28-byte constraint packet. The first three angle references define the
directed angle; the packet stores a finite literal distance as both pixels and
centimeters, followed by native unit tag 1 and a zero tail.

The importer represents this as a polar translation whose angle is
`DirectedAngle(first, vertex, last)` and whose distance is a literal Length.
It requires an earlier same-page Point source, three earlier same-page Point
angle operands, the exact Point output header, the exact packet length and code
prefix, a known unit tag, a finite magnitude, a positive finite scale, and the
observed zero tail. Repeated references remain valid: the angle-bisector sample
uses the source itself as the angle vertex.

The operation is import-only. Fresh authoring and clipboard reconstruction are
still rejected because the corpus establishes decoding and evaluation, not the
full native creation workflow. Unchanged source bytes continue through the
preserving serializer.

## Evidence

`四圆极限集[31].gsp` decodes its object 21 as a 0.000001 cm translation of
point 2 through angle `(18, 12, 17)`. `角平分线的尺规作图（雪山飞狐）.gsp`
decodes object 12 as a 0.5 cm translation of point 1 through angle `(4, 1,
10)`. That recovery also unlocks object 13, a vector translation that depends
on the code-22 result. The latter file consequently has all 96 drawing slots
decoded, and its port SVG reproduces the native O/A/B construction and its
dependent geometry.

Both untouched originals open and Ctrl-S byte-for-byte in Native Sketchpad 5:

- `四圆极限集[31].gsp`:
  `63e8489d93291ac4f518bca3ad0003ecdffadd4a37d83d0837389918b5f67ec6`
- `角平分线的尺规作图（雪山飞狐）.gsp`:
  `818314f651a131a031d88961fa2790fc11ae3fcce10baf9c9b34df7e8410610d`

The compiled full scan restores the two code-22 roots and the one dependent
code-16 object. Unsupported slots fall **583→580**, complete documents rise
**1,555→1,556**, drawing-complete documents rise **1,594→1,596**, unsupported
documents fall **44→43**, and partial views fall **789→788**. All **1,599**
files remain whole-file exact with zero import failures. Embedded-tool coverage
is unchanged at **4,779/5,356**. No standalone test case was added.

Full scan: `/tmp/code22_coverage_all_tests.log`. Native captures:
`/tmp/code22_four_circle_native_before_save.png` and
`/tmp/code22_angle_native_before_save.png`. Port render:
`/tmp/code22_angle_bisector_port.svg`.
