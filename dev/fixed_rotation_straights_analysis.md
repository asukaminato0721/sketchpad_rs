# Fixed rotation of rays and lines

Controlled Sketchpad 5 construction establishes two additional code-27
families. A rotated ray uses a 28-byte class-2 header with subtype 13; a rotated
line uses class 2/subtype 12. Both retain ordered references `[source, center]`
and the existing 36-byte fixed-rotation constraint, including degree angle,
cached sine/cosine, and native flag/cache words. Subtype 14 remains a finite
segment and must not represent these extended paths.

The native sources have center (130,150), source start (170,190), and source end
(210,190). The source objects are primitive code-64 rays or code-63 lines. The
native Transform menu marks the center and creates a fixed 90-degree rotation.
Both image control points become (170,110) and (170,70). Native ray rendering
starts at (170,110) and extends upward only; native line rendering extends
through both sides of the viewport. The screenshots distinguish these modes
from a finite segment and from each other.

The actual document writer moves the center to (150,170), source start to
(180,220), and source end to (260,200). Native saves preserve all 2,188 bytes
of each edited file. The rotated control points become (200,140) and (180,60),
with the ray extending from the first point toward the second and the line
extending in both directions. The native screenshots confirm that geometry.

Fresh ordinary Rust-writer files use 45 degrees and contain no native mark-center
label. Each opens and saves byte-identically at 2,144 bytes. Their image start
is (186.56854249,150), with direction toward the upper right. Again, Sketchpad
displays one-sided ray extent and two-sided line extent. Native script success
was checked before saved-byte comparisons; screenshots were inspected
separately.

| Retained fixture | Bytes | SHA-256 |
| --- | ---: | --- |
| `fixed_rotation_ray_original.gsp` | 2188 | `d393bd8c2c92a628025867b845f3836af00cb77fe91058ce7a656ed21636ae49` |
| `fixed_rotation_ray_edited_native_verified.gsp` | 2188 | `edab1c805e03bb13255907556c8cff4944c615772203b4cb80570b58450022a3` |
| `fixed_rotation_ray_generated_native_verified.gsp` | 2144 | `b1bdd1934550b4d7bac3a4659837ce6daefb404e8e040ef6b34e52561bbd9ea8` |
| `fixed_rotation_line_original.gsp` | 2188 | `fdd9f0a3354dd4f244e5ba478fce76e5d28037eeb5836c5f1da162735a62c069` |
| `fixed_rotation_line_edited_native_verified.gsp` | 2188 | `9b97a6930b89d0daa92ca5c775dfa20af9b0f6912488a7a36575a6cdc522513e` |
| `fixed_rotation_line_generated_native_verified.gsp` | 2144 | `96b233cc70ea23e51e60ce5b1898dc591e9f5b5ef4f2a3079bb823345a787d33` |

An independent mirror-host probe adds a reflected primitive point to each fresh
45-degree document. Its code-34 group retains references `[point 1, rotated
host 5]` and has no constraint payload. Both native saves are byte-identical at
2,208 bytes. Sketchpad shows the reflected point at
(186.56854249,206.56854249) for both hosts. The perpendicular projection lies
behind the ray's origin, so this also establishes that reflection uses the ray's
supporting line rather than clipping to its visible extent. The ray and line
remain distinct for drawing and picking.

| Mirror-host fixture | Bytes | SHA-256 |
| --- | ---: | --- |
| `reflected_point_rotated_ray_mirror_native_verified.gsp` | 2208 | `74b432a7704457c3db47ac766447f3cbf6c2cb5c625f853670182c22c2e35da5` |
| `reflected_point_rotated_line_mirror_native_verified.gsp` | 2208 | `cd7c995265e9a99d2c70cbc02a1e49c2bf21bd2e989e38a0317a864d825a3a74` |

The model uses explicit `RotatedRay` and `RotatedLine` variants. Dependencies,
clipboard remapping, geometry, drawing, and picking preserve the corresponding
`LineMode`. Neither kind enters the finite-segment predicate. The native writer
uses code 27 with subtype 13 or 12 and the same fixed-angle payload as the
previous supported families. Stroke color/width/pattern come from the observed
modern class-2 header fields, including the authored default medium navy stroke.

The source guard accepts primitive and fixed transformed `Ray` or `Line` objects, with known
point centers and finite fixed angles. Fixed rotation/dilation chains have independent
native proof in [fixed_straight_transform_chains_analysis.md](fixed_straight_transform_chains_analysis.md).
Reflected, calculated and marked-angle ray/line source constructions remain excluded. The existing fixed rotation editor
command and `gsp_probe replay` command `rotate` select the explicit variant.
`tests/transform_fixed_rotation_straights_replay.txt` generates both fresh files;
its bytes match the native-verified fresh fixtures.

Regressions in `src/document/tests/transforms.rs` cover unchanged native bytes,
the edited native goldens, explicit geometry/mode, fresh native writer bytes,
clipboard translation, dependency deletion, wrong source modes and target
subtypes, and the initially bounded primitive-source writer. Fixed rotation/dilation source chains now have independent native proof in [fixed_straight_transform_chains_analysis.md](fixed_straight_transform_chains_analysis.md).
The same test compares explicit reflected-point geometry and exact native
mirror-host bytes, retaining the consistent straight-host classification.
`src/editor/tests/transforms.rs` covers the fixed command, ray/line picking
outside the defining control points, one-sided ray versus two-sided line SVG
clipping, and retained dilation/reflection/marked-angle source exclusions.
Generate the writer files with
`SKETCHPAD_WRITE_ROTATED_STRAIGHT_FIXTURE=1 cargo test fixed_rotated_ray_and_line`.
Capture prefixes are `/tmp/fixed_rotation_{ray,line}_create`,
`/tmp/fixed_rotation_{ray,line}_edited`, and
`/tmp/fixed_rotation_{ray,line}_generated`.
Mirror-host capture prefixes are
`/tmp/reflected_point_rotated_{ray,line}_mirror`.
