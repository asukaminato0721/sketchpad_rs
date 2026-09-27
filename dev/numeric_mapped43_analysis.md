# Scalar mapped iteration evidence (checkpoint 43)

The checkpoint42 drawing-only baseline audit has 706 unsupported code-76/89 roots
whose direct predecessors decode. Of these, 234 have exclusively numeric
seeds, including 80 with two or three seeds. The multi-seed cohort is 77
Scalar pairs, two Scalar triples, and one Degree pair. No ready multi-seed
Length/Area/Compound cohort was found. These are prerequisite counts, not
claims that their complete carried DAG is supported.

Thirty-two live Scalar-pair roots (25 Parameter seed pairs, seven
ParameterCalculation seed pairs) have only these dependency kinds: Free,
Parameter, ParameterCalculation, CoordinateSystem, GraphAxis,
GraphHorizontalUnit, GraphVerticalUnit, NumericCoordinatePoint. Eight more
multi-seed roots have only Parameter/ParameterCalculation prerequisites.
The other ready numeric roots need broader transform, path, measurement,
intersection, polygon, or locus evaluation.

The raw reference partition is unchanged from point mapping:

- Code 76 references `[seeds..., branch images..., carried...]`.
- Code 89 prepends its scalar depth object.
- Record 0x090a stores u16 seed count at offset 6 and u32 branch count at 8;
  its 24-byte bounded definition has final-level flag at 4 and depth at 16.
- Mappings are simultaneous. With `[a,b]=[1,1]` and image `[b,a+b]`, states
  become `[1,2]`, `[2,3]`, `[3,5]` at depth three.

`dev/numeric_mapped43_generate.rs` independently reproduces the existing
native record templates around a public-API base drawing. Groups must be
inserted before the page's 0x2328/0x232d records, exactly as the production
writer does. Inserting them directly before 0x03e9 leaves them after page
state records and does not open in Native; malformed early controls were
not retained as evidence.

Three raw controls opened and saved byte-identically in Native Sketchpad:

| Retained fixture | Bytes | Evidence |
| --- | ---: | --- |
| `numeric_mapped43_fibonacci_native_verified.gsp` | 3380 | Scalar parameters a=b=1 mapped to b,a+b; carried numeric-coordinate p=(a,b) and p--q segment with q=(b,a+b). |
| `numeric_mapped43_calculated_native_verified.gsp` | 3312 | Same mapping with both seeds source-free Scalar calculations. |
| `numeric_mapped43_single_native_verified.gsp` | 3372 | One Scalar seed a mapped to a+b with independent b=1; retained as future evidence, not a current single-seed implementation claim. |

Each fixture has a corresponding PNG capture. The first two captures show
red point-binding markers at (2,3) and (3,5), and the blue segment ending
at (5,8). The first point image (1,2) lies below that initial small-window
capture. This distinction matters: p's images are (1,2),(2,3),(3,5), while
q supplies segment endpoints (2,3),(3,5),(5,8). p does not acquire an extra
iteration step.

`dev/numeric_mapped43_api_generate.rs` generates actual API controls for
parameter and calculated pairs, live depth, changed live inputs/depth, and
a mixed point/Scalar mapping. It checks strict roundtrip before writing.
Native logs and enlarged screenshots for these controls are recorded
separately from the independent raw controls.

All five actual API controls opened, displayed the expected sequences, and
saved byte-identically in Native (enlarged captures inspect every endpoint):

| Actual API fixture | Bytes | Verified Native geometry |
| --- | ---: | --- |
| `numeric_mapped43_api_fibonacci_native_verified.gsp` | 3380 | p=(1,2),(2,3),(3,5); segment ends at (5,8). Exactly identical bytes to independent raw Fibonacci writer. |
| `numeric_mapped43_api_calculated_native_verified.gsp` | 3312 | Same geometry with calculated Scalar seeds; exactly identical to independent raw calculated writer. |
| `numeric_mapped43_api_live_native_verified.gsp` | 3590 | Same geometry with scalar n=3 live depth. |
| `numeric_mapped43_api_edited_native_verified.gsp` | 3590 | a=2,b=1,n=2 gives p=(1,3),(3,4); segments connect (1,3)--(3,4) and (3,4)--(4,7). |
| `numeric_mapped43_api_mixed_native_verified.gsp` | 3170 | Free point A plus Scalar b mapped simultaneously to Q=(b,b) and b+1; point images (1,1),(2,2),(3,3), segment final endpoint (4,4). |

The mixed control proves point and Scalar states retain their distinct roles.
The calculated-seed control proves a calculated Scalar seed must behave as an
independent iteration input at runtime; reevaluating its original formula
instead of substituting its carried value would keep both source-free seeds
at one and collapse the orbit. Live edits prove depth and initial scalar
values are read from current drawing state rather than cached in the record.
Production remains bounded to the separately supported DAG kinds; this
proof does not authorize arbitrary transformed/locus/path constructions.


Checkpoint43 production stores Point/Scalar state values only inside the cloned
runtime sample. Scalar substitution uses the existing exact floating-point
parameter evaluation override, avoiding decimal display rounding. Seed kinds and
saved formula records in the source document remain unchanged. Branch targets
must match the ordered seed roles; supported predecessor edges must be backward,
undeleted and on the same page. The existing generation/state resource limits
and final-level semantics apply to both point and scalar mappings.

Six new document regressions reconstruct all five actual numeric API files and
the two derived-point transformed-segment writer files, exercise simultaneous
states, live source/depth edits, exact saves, undo/redo, poles and restoration,
and retain the original pentagon's byte identity and structural edit restriction.
The final scan restores269 drawing objects (110 iteration definitions and159
bindings), with unchanged prototype coverage. All1599 source files remain exact.
No copying or custom-tool implementation is added. Single-seed layouts have
Native evidence but require consolidation with the specialized iteration kinds;
geometric numeric inputs and wider carried shapes remain follow-up work.

The final scoped gates pass656 library,115 editor,34 numeric-import and four
coverage tests (809 total). Logs use `/tmp/sketchpad_checkpoint43_*_gate.log`,
with the final library run at `/tmp/sketchpad_checkpoint43_lib_final_gate2.log`.
The complete corpus scan is `/tmp/sketchpad_corpus_checkpoint43_final.log`.
Unsupported slots remain59173; this checkpoint is progress rather than command
parity completion. Display need not match Native pixel-for-pixel.
