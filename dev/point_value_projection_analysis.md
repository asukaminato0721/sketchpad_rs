# Value of Point: names and independent anchors

The native code 94 records in sibling corpus samples `光的折射(蚂蚁制作).gsp`
and `10福建宁德26题(蚂蚁).gsp` contain an optional 0x07d5 name record
between numeric format and text style. The former names its ray value `折射率n`;
its point constraint is attached to the same ray. The latter names its value `M`
and references a rotated point D and a separate segment, rather than a
point constrained to that segment. Its caption x coordinate is 42, where existing fixtures happened to use 10
or 74. Caption positions are signed screen coordinates; they impose no semantic
restriction on supported values. Imported label data and all opaque presentation
bytes remain preserved.

Native Sketchpad 5 was used to open and save four documents with independent
free point anchors, keeping the original code 94 records and replacing the
constrained point group with a free point:

- `point_value_free_segment_native.gsp`: segment (150,120)→(250,120),
  point (200,170), caption 0.50. This distinguishes signed orthogonal projection
  from Euclidean distance along the path. The live value remains unbounded.
- `point_value_free_circle_native.gsp`: center (323,217), point (250,20),
  caption 0.31. Radial projection gives
  atan2(197,−73)/2π≈0.3065. Radius does not affect the angular value.
- `point_value_free_arc_native.gsp`: the existing three-point arc through
  (323,217),(191,328),(497,311), independent point (300,300), caption 0.08.
  Normalized angular progress in the arc sweep is 0.077340094557163.
- `point_value_free_arc_outside_native.gsp`: same arc, point (323,150),
  caption 0.00. The angle lies outside the sweep and projects to the nearer
  endpoint, matching the existing constrained-point arc projection helper.

The four source documents were accepted and saved by native Sketchpad using
`dev/wine_roundtrip.sh`; captures are `/tmp/value_{segment,circle,arc}_free_before_save.png`
and `/tmp/value_arc_outside_before_save.png`. The shell needs Wine wow64,
xdotool, xvfb-run, and ImageMagick in PATH; this host's default Wine executable
cannot use the existing 64-bit prefix.

Evaluation keeps exact constraint parameters for matching circle/arc hosts.
Other supported point anchors use live geometry. A point at a circle center
has no direction and therefore yields an undefined value. Polygon values are
only accepted for a matching point-on-polygon constraint, whose local edge
fraction is converted to whole-perimeter progress by the separately verified
polygon implementation. There is no unsupported-object fallback.

Tests verify byte preservation of imported groups and whole documents,
labelled corpus references, live point edits, save/reopen, and copied values
with remapped dependencies. Malformed label byte counts are rejected.

The whole Ningde source also opens and saves byte-identically in native Sketchpad
(24,684 bytes). Its second-page caption displays `M = 0.35` and the independent
point D lies on the rectangle's top edge. Two upstream code 24 measured-distance
translations contain an opaque final four-byte trailer `b9 5c 2e 97`; their
36-byte payloads otherwise match the established vector/angle schema. Requiring
those trailing bytes to be zero blocked D before the value could be imported.
The trailer is preserved independently of the checked vector and distance
semantics. The numeric format also retains its two separately stored u16 fields
at offsets 24 and 26: type 6 and observed presentation variant 0, 1, or 3,
rather than interpreting both as a single u32.

## Sampled-path and sector values

The decoded-parent frontier contains 22 further native code-94 measurements.
Their second references are ordinary Locus, ParameterLocus, FunctionPlot,
SampledMapped, or SectorBoundary objects; their first references cover free,
constructed, transformed, colorized, coordinate, and path-derived points. Every
group has the established 32-byte `0x0903` Value of Point layout.

Sampled curve hosts use the existing live nearest-point search, then convert its
normalized fraction through the host's canonical value domain: normalized for
ordinary loci, the declared range for parameter loci and function plots, and the
mapped path's logical value for sampled mappings. Undefined live curves retain
their structurally valid measurement DAG. A matching PointOnSector uses its
stored normalized physical-perimeter parameter; arbitrary sector projection is
not inferred from this corpus.

The shared structural predicate is used by decoding, validation, and editor
selection. The existing point-value regression now includes one six-value locus
document and one sector document without adding another test function. The full
corpus restores 22 code-94 roots and 143 dependents: unsupported slots fall
2,639→2,474 (1,876 drawing plus 598 prototype), all-kind-complete documents rise
1,277→1,285, and drawing-complete documents rise 1,307→1,315. Unsupported
documents fall 322→314; all 1,599 originals remain byte-exact. Eight code-94
objects whose point or axis parents are still unsupported remain opaque. Full
scan: `/tmp/sketchpad_curve_point_values_full_coverage.log`.

A writer control created a sector, a constrained boundary point, and its
Value-of-Point measurement through the public document API. The supplied Windows
Sketchpad 5 executable opened `/tmp/sector_point_value_port.gsp` under Xvfb/Wine
and Ctrl+S left all 2,294 bytes unchanged (SHA-256
`5091d091477ae3f18f883be1b57823abf4b10506feccfbbb30a628fbf4efeb23`).
The temporary file and Wine prefix were removed after the check. This proves the
authored record's Native open/save acceptance; it is not a screenshot-based
pixel claim.
