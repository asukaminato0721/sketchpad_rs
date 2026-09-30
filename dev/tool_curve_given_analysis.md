# Legacy Curve custom-tool given

Audited on 2026-09-30. The corpus contains one declared class-1/subtype-8 given:
local object 1 of `直尺画线工具` in `圆弧展平（修改）.gsp`.

## Native category and eligible hosts

The original declaration orders its inputs `[2,1]`, initially prompting for
point `A`. A same-size temporary copy changes only that complete `04b4` payload
to `[1,2]`; `cmp` reports two changed low bytes. Native then displays
`1. 匹配曲线 c[1]`: match Curve `c[1]`.

While awaiting that input, independent hover controls mark an ordinary circle,
an arc, a full line, and a finite segment as eligible. Subtype 8 therefore uses
the same polymorphic destination family represented by the port's `Path`
category, despite retaining a distinct native subtype from the more common
class-1/subtype-5 placeholder.

## Bounded recovery

The exact class-1 placeholder body remains subject to the existing strict tool
input layout. Typing the root makes its source-free code-15 point-on-path child
reachable. A later code-11 object represents a generic curve intersection;
because the current semantic model has only fixed straight/circle intersection
variants, that object remains unsupported rather than receiving invented
geometry.

Prototype coverage rises from 5,059 to 5,061 of 5,356 and unsupported slots
fall from 297 to 295. Complete documents remain 1,570 and isolated-tool
completeness stays 47/56. All 1,599 drawings remain complete, every original
round-trips byte-exactly, and no standalone test case was added. Full scan:
`/tmp/tool_curve_given_coverage.log`.
