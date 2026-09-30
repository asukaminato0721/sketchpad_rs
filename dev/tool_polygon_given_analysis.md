# Polygon custom-tool given

Audited on 2026-09-30. The corpus contains one declared code-0/class-1/subtype-15
given: local object 2 of tool 0 in
`Samples/个人专栏/李章博作品/任意边数凸多边形在直线上滚动工具（李章博）.gsp`.
Its placeholder label is `P[1]`, and the tool declares inputs `[2,1]`.

## Native category evidence

The Native Custom Tools menu names the tool `凸多边形在直线上的滚动（李章博）`.
Selecting it under Xvfb displayed the status `1. 匹配多边形 P[1]` (match polygon
`P[1]`). Clicking the interior of the visible green polygon satisfied that input
and advanced to the point input. Clicking a source vertex then materialized a
magenta copy of the polygon, its line, labels, and rolling controls. The saved
probe grew from 35,946 to 51,334 bytes; its SHA-256 changed from
`4f2030604b8ff4c3181efe3e9b99fa4187384b247d02f9490c11a97df97e756b` to
`08af36e35584671bca562587f643496dcdda3ed2c52fc26c84b0755e06695cd8`.
Scratch captures are `/tmp/subtype15_{selected,after_polygon,after_point,after_save}.png`.

This establishes a Polygon category, not the broader Path category. The tool's
name and construction also require arbitrary vertex counts; no particular
polygon subtype can be inferred from its placeholder.

## Representation and binding

The given uses the strict class-1 placeholder family
`07d0:28, 07d5:30, 07d8:2, 0899:16, 07d7:0` and has no dependencies. Contextual
decoding assigns `ToolInput(Polygon)`. It participates in the general polygon
predicate so its code-39 perimeter and unit-aware descendants can decode, but
`polygon_vertex_count` remains unavailable until invocation binds a real object.
No synthetic vertices or cached geometry are introduced.

Engine invocation requires a supported actual polygon on the active page. The
desktop input picker applies the same type filter and uses ordinary filled-region
hit testing, so an interior click can select it as in Native.

## Measured recovery and limits

The affected 120-object prototype now decodes through local object 67 before
reaching independent mapped-iteration roots. It remains partial, and port-side
execution is therefore still rejected. The full scan restores 23 prototype
slots, raising decoded prototype coverage from 4,913 to 4,936 of 5,356 and
lowering unsupported slots from 443 to 420. Drawing coverage stays 1,599/1,599,
all originals round-trip exactly, and there are no extraction failures. No
standalone test case was added. Full scan:
`/tmp/tool_polygon_given_coverage.log`.
