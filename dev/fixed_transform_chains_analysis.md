# Fixed transforms of transformed segments and circles

Fixed rotation (27), fixed dilation (30), and reflection (34) use the same native
shape classes when their source is already transformed: segment class 2/subtype
14, or circle class 3/subtype 28. Each group has two ordered references:
`[source, center]` for rotation/dilation, or `[source, mirror]` for reflection.
The rotation constraint remains 36 bytes with its degree angle and cached sine
and cosine; dilation remains 12 bytes with a scalar factor; reflection has no
constraint payload. No additional source-class or source-coordinate cache is
introduced for these chains.

The controlled source fixtures are the previously native-verified
`vector_rotated_segment_native.gsp` and `vector_rotated_circle_native.gsp`.
Each has a primitive shape 4, its fixed rotation 5, and vector translation 6.
The new chain rotates image 6 by 45 degrees about point 1 (object 7), dilates
image 7 by 1.25 about point 1 (object 8), constructs mirror segment 9 from points
1 and 3, and reflects image 8 in that mirror (object 10). Native Sketchpad draws
all three derived images. The edit moves point 3 from (210,190) to (250,190) and
point 1 from (130,150) to (150,170), changing the source geometry, center, vector,
and mirror. Both edited native views show the changed chain.

Before broadening the guards, the audit generated ordinary primitive-source
transform groups with the existing writer and changed only their source
references to 6, 7, and 8. Native open/save established those source semantics.
The final regression now creates the chains directly through the normal writer
and compares every byte with those retained native files; it needs no reference
patching. No encoder changes were required.

All four native scripts exited successfully, and each subsequent saved-byte
comparison was identical. Fullscreen screenshots were inspected separately:

| Retained fixture | Bytes | SHA-256 |
| --- | ---: | --- |
| `fixed_transform_chain_segment_native_verified.gsp` | 2588 | `beed75c3a8fdb80df7653c37f7a182fe7fc4d1ce53598ce0eed3b0b6a323fcbf` |
| `fixed_transform_chain_segment_edited_native_verified.gsp` | 2588 | `e5c5aaee0d56cd8f1dc4d84a9f9f79ef1c73111f25a675b1c12a5e1abab6e07b` |
| `fixed_transform_chain_circle_native_verified.gsp` | 2588 | `3a58a739834b9af137c494f9fa59d2d6404f3b44d795756adda083d7a1bc01b1` |
| `fixed_transform_chain_circle_edited_native_verified.gsp` | 2588 | `2a645326075d5f8fc1e9f40dce42d5837a48290eb53cff71eb30e46efa5a4140` |

Import, validation, and editor fixed-transform guards now accept the established
supported segment and circle kinds. Rays, lines, iteration bindings, and
unsupported source families are excluded by the closed shape-kind predicates.
The fixed segment/circle rotation decoder also retains exact target-class and
subtype checks. Polygon branches and marked-point-angle transform source guards
are unchanged. Calculated-value chains have separate evidence in
[calculated transform chains](calculated_transform_chains_analysis.md).

The existing recursive geometry paths carry the changes to drawing, picking,
measurement, and editing. Regressions in `src/document/tests/transforms.rs`
check explicit rotated/scaled/reflected point coordinates, circle radius,
individual source/center edits, save/reopen, the four native goldens, clipboard
translation, and dependency deletion. `src/editor/tests/transforms.rs` checks
the actual fixed commands and continued ray/line rejection. Generate the writer
files with `SKETCHPAD_WRITE_FIXED_CHAIN_FIXTURE=1 cargo test fixed_`.
Their paths are `/tmp/fixed_transform_chain_{segment,circle}_writer.gsp` and
`/tmp/fixed_transform_chain_{segment,circle}_edited_writer.gsp`. Native capture
prefixes are `/tmp/fixed_transform_chain_{segment,circle}` and the corresponding
`_edited` suffix.
