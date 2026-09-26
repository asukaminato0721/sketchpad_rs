# Two simultaneous seeds with a calculated angle

Source:
`../gsp-rs/tests/Samples/个人专栏/贺基旭作品/奇角形迭代(hjx4882).gsp`,
with a paired HTM export. The HTM identifies parameter `N = 17`, calculation
`180°/N`, and the rotation of point 1 about point 2 by that angle. Native
Sketchpad displays a 17-point star and opens/saves the original 3234-byte file
unchanged. Its unrelated conditional action, code 73, remains unsupported.

The iteration prerequisites all decode. Objects 1 and 2 are free seeds, 3 is
their hidden segment, 4 is N, 5 is the degree calculation, and 6 is
`CalculatedAngleRotatedPoint(1,2,5)`. Definition 7 has code 89 and references
`[4,1,2,2,6,3,6]`: live depth, two seeds, image branch `[2,6]`, and carried
segment/point. Its `090a` bytes are
`0000000000000200010000005d7936091100000000000000`. The cached depth is 17;
the live scalar N determines the current depth. Binding 8 carries point 6;
binding 9 carries segment 3.

For an isolated native regression the two unrelated action object groups were
removed and the page preamble's object count changed from 11 to 9. Remaining
object records, styles, labels, references, and caches were retained. Native
Sketchpad opens/saves that 2926-byte file unchanged, and the port imports it
fully. The complete original remains a partial document and preserves code 73.
Both versions are retained as fixtures; the regression checks that boundary.

The mapped graph guard now accepts calculated-angle point rotations whose
numeric dependencies consist only of scalar/angle parameters, constant
calculations, and parameter-calculation chains. Degree typing is checked before
accepting the rotation. Geometric numeric measurements and their formulas are
excluded. Numeric inputs remain fixed within a sequence; changing N recomputes
both angle and depth. Point inputs are substituted simultaneously, so each new
pair is `[old second seed, rotated old first seed about old second seed]`.
The same state evaluator supplies carried point and segment images.

Tests check the native 17-step geometry, N changed to 5, segment lengths,
carried point count, seed movement, point-binding color, copied dependencies,
undefined N=0 suppression with successful save, and rejection of a measured
angle mapping. A fresh five-step sketch and an edited native sketch opened in
Sketchpad as five-point stars and saved byte-identically. Generated and edited
bytes are compared directly with those native saves. Replay is
`tests/mapped_calculated_angle_iteration_replay.txt`; the fixture-generating
test uses `SKETCHPAD_WRITE_MAPPED_ANGLE_FIXTURE=1` for exact native probe inputs.

| Fixture | Bytes | SHA-256 |
| --- | --- | --- |
| `mapped_calculated_angle_iteration_full_native_verified.gsp` | 3234 | `5efb972c3d8f0a55204d166dd6612bb1a74280dd7001d31204384d4ad778bded` |
| `mapped_calculated_angle_iteration_native_verified.gsp` | 2926 | `dc1e8ac7ff87856c96220038c2ca27a7a9405dca9247bb73f6311e48c14d432a` |
| `mapped_calculated_angle_iteration_generated_native_verified.gsp` | 2668 | `bee467a90c8cb3e8e6629a40d43adc8bbbe98f4d341eaeaabb1ebfe4816b8f1f` |
| `mapped_calculated_angle_iteration_edited_native_verified.gsp` | 2924 | `3269208337901caf9177fadce5f43f64e2adf6af8480ed676463ee423e03e40f` |

This does not support arbitrary numeric seed mappings, non-free seeds,
geometry-dependent numeric inputs, or generic rotated/scaled carried segments.
The four-branch Hilbert fixture also has a nonzero definition mode word at
offset 4; its semantics require a separate native probe before extending that
layout.
