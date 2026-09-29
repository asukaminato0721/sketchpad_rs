# Circular-segment transforms

The remaining code-19/class-13/subtype-69 drawing objects occur in
`方轮小汽车.gsp` and `方轮小汽车2.gsp`. Each file has an intrinsic
`CircularSegmentBoundary` root followed by 22 code-19 images in one recursive
chain. Every group has the ordinary four records: a 28-byte `07d0` header, two
references in `07d2`, a 28-byte `07d3` constraint, and `07d7`. References are
`[source region, marked Length]`.

Code 19 is the native fixed-vertical mixed rectangular flavor. The marked Length
drives horizontal displacement; the constraint stores a fixed vertical
centimeter magnitude and its 37.79527559055118-pixel cache. Both corpus chains
use a zero fixed vertical value. The port represents subtype 69 distinctly from
a sector because its fill closes the transformed arc with the endpoint chord,
not two radii.

The shared affine evaluator now supplies transformed arc geometry to desktop/SVG
fill, picking, area, and perimeter. Header writing uses class 13/subtype 69 and
retains region style and drawing order.

An isolated port-authored file used a 150-pixel marked horizontal distance and a
1-centimeter fixed vertical distance. Sketchpad 5 opened it under Xvfb/Wine and
displayed two identical orange arc-plus-chord regions separated by exactly 150
pixels right and 37.79527559055118 pixels up. The port's generated file is 2,450
bytes and reopens with the same live transform definition.

The compiled 1,599-file scan restores all 44 slots, raises fully decoded
documents from 1,268 to 1,270 and drawing-complete documents from 1,298 to
1,300, and lowers unsupported slots from 2,934 to 2,890. The 598 unsupported
prototype slots are unchanged; all original files remain byte-exact.

## Fixed rotation, dilation, and reflection

The corpus also contains class-13/subtype-69 regions with code 27 fixed
rotation, code 30 fixed dilation, and code 34 reflection. Fourteen roots have
fully decoded source and operation parents. They use the same reference and
constraint layouts already verified for other analytic shapes, so the shared
transform evaluator applies without a second region representation.

A 13-object port writer created one circular segment plus a 180-degree image, a
reflection across a horizontal segment, and a factor-0.5 dilation. Sketchpad 5
opened the 2,692-byte file under Xvfb/Wine and rendered four orange
arc-plus-chord regions at the expected positions, including the halved radius.
The editor exposes these three fixed commands for circular segments while
continuing to reject unverified marked-angle and marked-ratio forms.

This pass restores 37 more drawing objects. Unsupported slots fall from 2,890
to 2,853 (2,255 drawing plus 598 prototype); document-completion and partial-view
counts remain unchanged. All 1,599 originals remain byte-exact.
