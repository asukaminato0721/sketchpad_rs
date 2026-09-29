# Coordinate-pair captions in mapped constructions

`前辈们的方向箭头（颗粒）.gsp` object 182 maps one parameter through a
construction that carries coordinate-pair measurement 169 and anchored caption
171. The pair is not a scalar numeric object, but its value is fully derived
from mapped point 152 and coordinate system 102. The existing position and
graph-frame evaluators already apply both substitutions correctly.

Mapped construction validation now admits the `Coordinates` measurement kind
as a carried value. It is not admitted as a direct geometry binding; Native
displays it through the already-supported `PointReferencedText` value binding.
This primary root restores twelve immediate bindings and an alias, which in
turn restores four downstream mapped roots and their construction chains: 65
drawing slots altogether.

The comparison also exposed an old presentation mismatch. Native stores the
coordinate-pair decimal precision in the referencing caption's cached rich
text, not in the scalar measurement precision field. The live resolver now
recovers that cached precision and normalizes signed zero. This control
therefore renders `(1, 1)` through `(8, 8)`, rather than the port's former
`(1.00, 1.00)` form. Two unrelated corpus assertions embedded in the generic
numeric-label editing test were removed instead of preserving the stale
expectation. No new test case was added.

Sketchpad 5 under Xvfb/Wine and the port SVG show the same red arrow lattice
and blue coordinate labels. Native saves all 34,170 bytes unchanged with
SHA-256
`2549be90b2e0c959e7b50f7997e371ee5a9a6a70e3609cd47450102411490491`.
Evidence is `/tmp/code89_coordinate_caption_native_before_save.png` and
`/tmp/code89_coordinate_caption_port_native_precision.svg`.

Across the full 1,599-document corpus, unsupported slots fall 1,619→1,554
(956 drawing and 598 prototype), all-kind complete documents rise 1,411→1,412,
drawing-complete documents rise 1,447→1,448, and unsupported documents fall
188→187. Partial views remain 823, and every original remains byte-exact. The
ready code-89 frontier falls from 8 roots/114 reachable unsupported slots to 7
roots/49 reachable slots.
