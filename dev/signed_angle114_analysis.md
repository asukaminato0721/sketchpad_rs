# Legacy signed-angle values (code 114)

## Scope and layout

Two corpus objects use native code 114 with class 18 / subtype 95. Both have
three earlier same-page point references and the exact record sequence
`07d0(28), 07d2(16), 090e(12), 07d7(0)`. The marker payload is
`01 00 01 00 00 00 00 00 fe fe fe 00`. Each value feeds a legacy code-119
numeric point rotation, which in turn feeds a code-97 locus.

Fixtures:

- `../gsp-rs/tests/Samples/个人专栏/方小庆作品/(inRm)摆线.gsp`
- `../gsp-rs/tests/Samples/个人专栏/方小庆作品/圆的滚动全解(inRm).gsp`

The semantic model keeps a distinct `SignedAngleValue(first, vertex, last)` so
this hidden numeric construction is not conflated with visible code-113 angle
marks or the positive-turn code-115/116 variants. It has degree units, live
three-point dependencies, signed-shortest-turn evaluation, marker presentation,
copy/remapping support, and an exact code-114 writer.

## Native direction evidence

In the smaller fixture, the referenced points produce these candidate turns:

```text
signed(B, C, D) = -56.00023691591092°
signed(D, C, B) =  56.00023691591092°
```

Code 119 rotates object 16 around object 11. Applying the negative turn places
the result at document coordinate `(220.772238, 63.014924)`. A real Sketchpad 5
run under Xvfb/Wine with Show All Hidden places the dependent point at that
location; the positive and alternate-vertex candidates do not match.

A separate untouched Native open/save remains byte-identical with SHA-256
`e4fc80205c12fd460f8ce849a09bd8ca55aa7e57e2c2166e3ba56c703ae5ab20`.
The port also copies the seven-object dependency closure, writes a fresh
code-114 group, and reopens it as the same signed value.

## Coverage

The full scan is `/tmp/signed_angle114_coverage.log`. Two code-114 values, two
code-119 rotations, and two downstream loci are restored. Unsupported slots
fall 775→769; complete documents/drawings rise 1,508/1,549→1,510/1,551;
unsupported documents fall 91→89; and partial views fall 801→800. Embedded-tool
coverage remains 4,766/5,356. All 1,599 originals retain byte-exact round trips.
No standalone test case was added.
