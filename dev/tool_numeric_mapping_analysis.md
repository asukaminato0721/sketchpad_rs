# Numeric mapped-tool operands and path values

## Scope

Three remaining custom-tool families used declared inputs in positions that are
ordinary typed objects after a tool is invoked but placeholders while its
prototype is decoded:

- `排列与组合数.gsp` uses generic Numeric inputs as mapped seeds and replaces
  them with calculated numeric branch values.
- `也做正n棱锥表面展开.gsp` measures a concrete prototype point along four
  declared Path inputs.
- `m×n网格的交错填充.gsp` uses generic Numeric inputs as the coordinates of
  three prototype coordinate points. Those points feed a fixed dilation and a
  mapped iteration.

The admissions are restricted to isolated tool decoding. Ordinary documents
retain the existing numeric-coordinate, point-value, and mapped-seed type
checks. Invocation must bind every placeholder to a live object of the declared
type before any construction is materialized.

## Decoder and runtime evidence

Mapped-iteration validation now permits a declared generic Numeric placeholder
as a seed and requires its branch image to be either another Numeric
placeholder or a typed numeric result. The combinations prototype consequently
decodes all 25 previously blocked slots. Binding `n=5`, `m=2`, and the tool's
two control values to `0` and `1` creates 32 outputs in a 37-object, 7,048-byte
drawing. It reopens non-partially and re-encodes exactly.

Sketchpad 5 under Xvfb/Wine visibly displays the expected `C = 10` result.
Ctrl+S preserves SHA-256
`12b30c07a483d7b9bc1f9159600ecd7e17aba37e8a97f0da66e08cfce09acfee`.

Point Value accepts a concrete point paired with a declared Path only in a tool
prototype. Four such roots and their four calculations restore all eight slots
in the interval-hiding tool. A real invocation with four finite segments and a
point creates eight outputs in a 21-object, 4,152-byte drawing; reopen and
re-encode are exact.

Numeric Coordinate Point similarly accepts declared Numeric placeholders for
its x/y operands only while decoding a tool. This restores three coordinate
points, their fixed midpoint/dilation result, the mapped root, and its binding:
all six blocked slots in the grid tool. The complete prototype evaluates its
positions and iteration graph. Full materialization currently reaches the
separate guarded limitation for laying out `trunc(...)` formula captions; no
unverified caption geometry was introduced as part of this decoding pass.

## Corpus result

The pass restores 39 prototype slots: coverage rises
**5,290→5,329/5,356**, unsupported slots fall **66→27**, fully decoded
documents rise **1,591→1,593/1,599**, and complete isolated tool views rise
**75→78/103**. All 1,599 drawings remain complete and all 1,599 source files
round-trip byte-exactly. No standalone test case was added.

Full scan: `/tmp/tool_numeric_mapping_path_values_coverage.log`.
