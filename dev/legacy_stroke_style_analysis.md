# Legacy segment, circle, ray, and line pen styles

The authentic 16-byte headers come from Clock(2): segment objects 10 and 39,
circle objects 6 and 8, ray object 3 (native code 64), and perpendicular-line
object 29 (native code 5). Their existing groups were isolated with authentic
legacy metadata and four cloned known free-point groups. Only coordinates,
parent references, page object count, and the hidden flags of the ray/line were
changed. The resulting ten-object source is fully known to the port and to
native Sketchpad: four free points, two segments, two circles, a ray, and a
perpendicular line. No ordinary legacy code-63 line or code-4 circle-by-segment
was discovered, and those conversion families are not inferred from this proof.

Native visible geometry proves the code-64 object is a ray: it extends beyond
its second defining point in one direction. The code-5 perpendicular line spans
the viewport in both directions. Segment/circle pen styles are visibly solid,
with different widths. Native copy/paste upgrades all original and copied
headers to 28 bytes, preserving the source RGB bytes and live parent graph.
The original and copied style fields establish an enum rather than a packed
modern width/pattern byte:

| Old word at bytes 10–11 | Native modern byte 10 | Style |
| --- | --- | --- |
| `00 00` | `11` | Thin Dashed |
| `01 00` | `21` | Thin Solid |
| `02 00` | `22` | Medium Solid |

The authentic segment and circle headers independently supply values 1 and 2.
A controlled value-2 ray/perpendicular-line source also upgrades to 22, with
visible medium solid strokes. A separate all-zero source independently proves
Thin Dashed for all four families, including dashed circles, rays, and lines.
Every controlled copy output is 3284 bytes and contains twenty objects. These
are conversion/copy proofs, so original and converted output intentionally
differ. Successful script exits and captures establish the edits actually took
place; no unchanged-file comparison is substituted for conversion evidence.
The earlier eight-object segment/circle-only probe converted/copied to 3028 bytes.

A separate segment changed to old raw word 3 is rejected with a native format
error, as is raw word 11. The latter is valid packed modern Thin Dashed, but it
is not the legacy encoding of that style. A mixed zero/three diagnostic was
also rejected; independent zero acceptance and three rejection resolve its
ambiguity. Failed captures remain at
`/tmp/legacy_segment_style_{three,dashed}_native_open_failed.png`.

Retained fixtures are:

- `legacy_stroke_styles_original.gsp` and `legacy_stroke_styles_copied_native.gsp`:
  authentic mixed Thin/Medium Solid styles.
- `legacy_stroke_dashed_original.gsp` and `legacy_stroke_dashed_copied_native.gsp`:
  independently controlled zero for every stroke family.
- `legacy_stroke_medium_paths_original.gsp` and
  `legacy_stroke_medium_paths_copied_native.gsp`: Medium Solid ray and line.

The private low-level predicate requires a 16-byte header, byte 11 equal to zero,
a proved enum value 0–2, and exactly one of `(code,class,subtype)`:
`(2,2,14)`, `(3,3,28)`, `(64,2,13)`, or `(5,2,47)`. Getters map only these proved
headers. Setters support exactly Thin Dashed, Thin Solid, and Medium Solid and
write only byte 10. Other combinations fail before mutation and require a
modern header. Opaque byte 15, colors, flags, and all other bytes stay untouched.
Other short layouts retain their prior interpretation. In particular modern
short-header prefixes with byte 11 equal to 1 still use the packed modern
pattern/width byte; compatibility tests preserve that existing behavior.

Whole-document conversion must validate the exact legacy families and enum
values, map byte 10 with the table above, and use the corresponding modern
header layout. Native converted stroke headers have byte 11 equal to 1,
byte 15 zero, bytes 16–17 `ff ff`, and the standard modern cache tail
`c1 bd f0 ff 00 00 00 00`. Header marker bytes are presentation state and remain
subject to the independently proved converter/template behavior. This audit
adds no guessed packed legacy bits and does not convert unsupported objects.

The regression compares every source stroke with both its native-converted
original and copy, checks RGB, and verifies atomic setter rejection and exact
preservation of other fields. `tests/legacy_stroke_style_replay.txt` applies
fitting legacy styles through the actual public APIs and saves a source whose
headers remain 16 bytes. The ordinary modern-size/style conversion policy is
handled separately by the document converter.

The actual port-authored replay output opened successfully in the native
application and saved byte for byte (1744 bytes). The capture shows the first
circle and segment solid Thin, the second circle/segment and ray dashed Thin,
and the perpendicular line solid Medium. The retained
`legacy_stroke_styles_edited_native_verified.gsp` is compared with the actual
public API edits in a regression, including all ten headers remaining 16 bytes.
This proves edits that fit the old enum independently of modern-header copying.
