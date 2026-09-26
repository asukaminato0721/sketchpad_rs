# Simultaneous point mappings carrying segments

Evidence is the original Sketchpad 5 document
`../gsp-rs/tests/fixtures/bug/迭代方法2(蚂蚁).gsp` and its paired `.htm`.
The native file is retained as `tests/fixtures/mapped_point_iteration_ant_native.gsp`.
The HTM export describes four independent seed points, four sides, and vector
translations of those points and sides (construction lines 80–117); it omits the
native iteration. Native record parsing and fresh Sketchpad cross-opens therefore
establish the iteration definition, rather than treating the HTM as an iteration
oracle.

Both pages contain a code 89 definition with four seeds and four simultaneous
mapping branches. Its 24-byte `090a` record has four initial zero bytes, a `u16`
image-level flag at offset 4 (0 for all levels, 1 for the final level only), a `u16`
seed count at offset 6, a `u32` branch count at offset 8, an opaque native word at
12, cached depth at 16, and four final zero bytes. References are:

1. The live depth source for code 89 (absent for fixed-depth code 76).
2. The ordered seeds.
3. Each branch's ordered target points, one target per seed.
4. The remaining carried construction objects.

The first page's definition 26 has depth source 25, seeds `[1,4,6,2]`, and
branches `[11,10,1,12]`, `[14,2,15,16]`, `[6,20,19,18]`, `[24,23,22,4]`.
Its remainder is `[3,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24]`.
It includes prerequisite points as well as segments. Eight code 77 bindings
select which segments actually draw; each binding references `[source,26]`.
The second page has the same four-seed/four-branch layout, cached depth 4, and
20 segment bindings. Original reference ordering and opaque cache words survive
saving.

Every branch evaluates all targets against the same previous seed state, then
substitutes all seeds together. External points stay fixed. Carried vector
construction graphs are reevaluated against each resulting state. In a fresh
example, A=(100,160), B=(140,180), and C=B+(B-A), mapping `[A,B] -> [B,C]`
produces segments B–C, C–(220,220), and (220,220)–(260,240) at depth 3.
Native Sketchpad displays precisely that chain. A second branch translating in
the opposite direction produces the expected endpoints (60,120) and (340,260)
for the separate two-branch example. Coincident complete states are drawn once
per generation; this preserves the appearance of these opaque segment bindings.

The implemented family accepts distinct free-point seeds, vector-translated
point construction graphs, and segment/vector-translated-segment bindings.
The bounded extensions in [midpoint evidence](mapped_midpoint_iteration_analysis.md)
and [calculated-angle evidence](mapped_calculated_angle_iteration_analysis.md)
also accept midpoint graphs, rotations driven by pure parameter degree formulas,
and explicitly selected carried-point bindings.
It retains the native carried list, including point prerequisites. Fixed depths
and existing scalar iteration-depth sources work. Undefined depths suppress the
copies while remaining savable. Seed drags, live depth edits, SVG/canvas drawing,
selection through copied segments, hiding, binding stroke styles, clipboard
remapping, and single-page dependency deletion use the existing editor machinery.
The original ant source now decodes all 98 objects, including both URL link
buttons after supporting their native four-byte string padding and opaque
reserved word. Its untouched source and a source seed drag save byte-identically
in Sketchpad 5. The decoder still marks multi-page sources as partial views.
Existing scalar parameter values now use the bounded multi-page edit path
described in `multi_page_parameter_values_analysis.md`: the first-page depth
changes from 3 to 2 while the second-page depth remains 4. The actual Rust output
cross-opens and saves byte-identically in Sketchpad 5 (11,254 bytes), with the
native caption showing `n = 2`. Tests verify the lower iteration count,
independent second-page images, save/reopen, atomic rejected edits, and undo/redo.


Construction is available through `Document::add_mapped_point_iteration` and
`gsp_probe replay` command `iterate_mapped`. See
`tests/mapped_point_iteration_replay.txt`: comma-separated seed/carried IDs,
semicolon-separated branches, then fixed depth or `value:PARAMETER_ID`.
There is no new desktop dialog for specifying general simultaneous mappings.
Numeric state mappings, other construction graphs, and generic carried
polygons/circles remain outside this family. Limits are 2–16 seeds,
1–8 branches, depth 1–128; evaluation suppresses a sequence exceeding 20,000
unique states in one generation or 100,000 generated states across all levels.
The final-level display mode still computes and bounds the complete substitution
chain. The optional visible-binding list follows the depth in the replay command;
the sidebar can switch a supported mapped iteration between all and final levels.

The edited replay changes n from 2 to 4 and B from (140,180) to (150,170).
Native displays the shallow chain through (350,210), confirming both dependency
edits in the saved file.

Native Sketchpad 5 opened and saved all three fresh files and the edited live-depth replay byte-identically,
with screenshots inspected separately from the successful process exits:

| Fixture | Bytes | SHA-256 |
| --- | ---: | --- |
| `mapped_point_iteration_generated_verified.gsp` | 2212 | `c79794edbd6c05d5e050346c9f411d5a526a01d84e4b7f1066525399703acea1` |
| `mapped_point_iteration_branches_verified.gsp` | 2292 | `bab2ab9c5a17c6286124e0713005cac1d2f860fdf075a8ed5421ba43b31f1e4e` |
| `mapped_point_iteration_live_verified.gsp` | 2422 | `9bc438d55d7e07acd2f42afa357d2848791180ca2eee213924ac70e0ae73f270` |
| `mapped_point_iteration_edited_verified.gsp` | 2422 | `596f0181285ca4df2256fc3651a2aad03fff5b0a2e52b925514aaba916d28cb5` |
| `mapped_point_iteration_ant_edited_verified.gsp` | 11254 | `03e34891ca8b3f5aba2f0457355a3766cb0555a87d91f0fd08ac17bd20ab983b` |

Regression tests in `src/document/tests/mapped_iteration.rs` cover the native
four-seed definitions, exact fresh bytes, simultaneous substitution, distinct
branch states, seed/depth edits, undefined domains, saved binding colors,
clipboard remapping, deletion, picking, SVG counts, and invalid mapping inputs.
