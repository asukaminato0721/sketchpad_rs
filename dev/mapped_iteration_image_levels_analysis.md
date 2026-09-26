# Mapped iteration image levels

The native 24-byte `090a` definition stores a `u16` image-level flag at offset
4. Controlled Sketchpad 5 probes establish `0` as all generated levels and `1`
as the final generated level only. Original construction objects remain visible
in either mode. The flag does not change the mapping, its depth, the carried
objects, or the complete-state deduplication within each generation.

The initial audit found flag 1 in the four-branch Hilbert definitions of
`../gsp-rs/tests/Samples/个人专栏/方小庆作品/[希尔伯特曲线]制作详解(inRm).gsp`
and its paired `.htm`. For example, definition 106 has seeds `[87,85]`, four
simultaneous branches, and definition bytes
`000000000100020004000000393654090300000000000000`.
That source also requires fixed dilation and rotation graphs that remain outside
the current mapped-graph guard. Its flag alone therefore does not make the
Hilbert source editable.

Two previously verified files isolate the flag without introducing those graph
dependencies. Each disposable source was changed only at offset 4, opened and
saved successfully in Sketchpad, and compared after the native process exited:

| Retained fixture | Bytes | Native result |
| --- | ---: | --- |
| `mapped_midpoint_triangle_final_native_verified.gsp` | 2864 | At depth 2, only the final small triangle and its carried midpoint images remain; the intermediate medial triangle disappears. |
| `mapped_branches_final_native_verified.gsp` | 2292 | At depth 3, all four distinct final branch states remain; the earlier generated segments disappear. |
| `mapped_final_level_live_native_verified.gsp` | 2422 | A fresh code 89 mapping with live `n = 4` displays only its final segment, (260,240)–(300,260), along with the original construction. |

For the triangle, byte 2428 changed from 0 to 1. For the branching file, byte
2176 changed from 0 to 1. Every other byte, including reference ordering, opaque
cache words, and cached depth, was preserved by the native save. The respective
SHA-256 values are
`d61f719c590921db35e3fd95203df06afa16033bbfdff6e55f266c2c05b378c5`
and `4af6af06869cd997a0f703ba0857dd0b2bc3682d349cd7442cf4e6c91501e9df`.
The freshly generated live-depth file also opened and saved successfully with
all 2,422 bytes identical; its SHA-256 is
`c08bde2bbb2e24e3443e706ebb1db9bd041f653c9b09c4c95f44e92f3aa1c11b`.

`Kind::MappedPointIteration::final_only` retains this property. New mappings
default to all levels. `Document::set_mapped_iteration_final_only` verifies a
candidate save before changing the document and supports undo/redo. The editor
sidebar and iteration tool expose the property for a supported mapped parent or
its selected carried binding. Clipboard remapping preserves the property.
Saved existing definitions patch only the two-byte flag; unknown flag values
remain unsupported and byte-preserved.

Both modes compute every substitution level. The existing limits of 20,000
unique states per generation and 100,000 generated states across all generations
therefore apply even when only the final level draws. Point and segment bindings
share the same evaluator, so final-level selection also reaches SVG, canvas,
and picking through their existing geometry paths. Live depth edits recompute
the final level.

Regression coverage is in `src/document/tests/mapped_iteration.rs`: exact native
bytes, both display modes, point and segment images, multiple branches, live
depth, undo/redo, clipboard, and rejection of unknown flags. Numeric
point/parameter iteration still requires a scalar parameter seed; typed degree
and distance seeds remain rejected until their repeated substitution semantics
have native evidence.
