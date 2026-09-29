# Legacy short-header circle interiors

`时钟 (2).gsp` contains two code-10 class-5/subtype-16 groups whose headers are
16 bytes rather than the modern 28 bytes. Objects 148 and 149 each have one
`07d2` Circle reference and no constraint. Their sources are supported Circles
146 and 147.

The short headers retain the normal RGB bytes but have neither an opacity field
nor the modern drawing-rank word. This matches the existing GSP4 polygon rule:
the implicit opacity is `0xbfff` (75%) and `interior_order` remains absent while
the original record is preserved. Only the exact code-10/class-5/subtype-16
signature is admitted; other short region headers remain unsupported.

Both objects now evaluate as live analytic disks and preserve their source bytes.
The legacy document-upgrade classifier also maps this exact profile to a modern
disk with opacity `0xbfff` and rank 0 when a complete legacy document needs an
upgrade. The current clock still has unrelated unsupported legacy constructions,
so it correctly remains a partial view.

Sketchpad 5 opened the original under Xvfb/Wine and rendered the two overlapping
gray filled disks above the clock face. The compiled 1,599-file scan restores two
drawing slots: unsupported falls 2,823→2,821 (2,223 drawing plus 598 embedded
prototype slots). Completion and partial-view counts do not change, every source
round-trips byte-exactly, and no parse fails.
