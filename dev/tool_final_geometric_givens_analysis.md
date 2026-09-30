# Final code-zero custom-tool given categories

Only class-1/subtypes 6 and 60 remained unknown after the other custom-tool
given categories were established. Each occurs once in the 1,599-document
corpus.

## Subtype 6: Path Object

`7.多面体表面(逆时向)着色` in `颗粒坐标系(3D).gsp` has ordered inputs `[2,1]`.
Native Script View labels local object 2, subtype 6, as `路径对象 面P[1]` and
local object 1, subtype 5, as `路径对象 j`. The two tags therefore share the
port's polymorphic `Path` category. The prototype retains unrelated unsupported
constructions, so only the given root is recovered.

## Subtype 60: segment-backed Distance

`三维空间中的点2` in `也做正n棱锥表面展开.gsp` has ordered inputs
`[8,7,6,5,4,2,1,3]`. Native Script View labels local object 2, subtype 60, as
`距离 r`. After matching the preceding point `O[1]`, Native asks the user to
click a distance. Clicking the visible `OA` distance-value caption does not
advance the tool. Clicking the finite segment matches `r`; Native then matches
the final value and begins the construction.

The prototype's sole direct consumer is `CircleBySegment(4,2)`, where the
given supplies radius magnitude. The port consequently represents subtype 60
as `SegmentDistance`: destination matching accepts a finite segment, while the
abstract root carries Length semantics and not general segment geometry. This
keeps it separate from subtype-14 Segment and subtype-48 Distance Value inputs.

The two roots raise decoded prototype coverage from 5,068 to 5,070 of 5,356
and reduce unsupported slots from 288 to 286. Complete documents and isolated
views remain 1,572 and 49 because both prototypes retain unrelated gaps. All
1,599 originals remain byte-exact. The full scan is
`/tmp/tool_final_given_coverage.txt`.
