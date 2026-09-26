# Native strokes on point curves

Curve headers use the existing line-style byte at object-header offset 10 and
RGB bytes at offsets 12–14. The supported layouts are class 6 with these
code/subtype pairs:

| Object | Code | Subtype | Authored default |
| --- | --- | --- | --- |
| Point locus | 35 | 42 | Medium solid, RGB 128,0,0 |
| Parameter locus | 97 | 42 | Medium solid, RGB 128,0,0 |
| Function plot | 72 | 58 | Thin solid, RGB 128,0,128 |
| Rectangular parametric plot | 126 | 99 | Medium solid, RGB 128,0,128 |
| Polar parametric plot | 127 | 99 | Medium solid, RGB 128,0,128 |

Width occupies the low nibble (0–3) and pattern the high nibble (0 dotted,
1 dashed, 2 solid), as on straight paths. Imported values are preserved.
The defaults above match the encoder's native templates. Code-35 subtype-43
swept segment, circle, and polygon loci keep their existing special geometry
and presentation; they are not treated as point-curve strokes.

Display color, width, and pattern edits now patch these recognized headers.
SVG and canvas rendering join connected sample segments before drawing the
stroke, so dash phase continues across sample boundaries. Undefined or
disconnected ranges begin separate paths. Picking continues to use the existing
sampled geometry. Stroke widths remain the port's existing pixel approximation.

The regression `native_curve_strokes_are_editable_and_svg_keeps_color_width_and_pattern`
imports five native layouts, verifies unchanged bytes, applies RGB 14,150,210
and Thick width, then saves and reopens each. Polar receives Dotted; the other
four receive Dashed. The test checks saved styles and fitted SVG output.
`authored_curve_defaults_match_native_templates_and_sweeps_keep_their_layout`
checks new curves against their templates and preserves swept-locus behavior.
A renderer regression verifies continuous dash phase and disconnected ranges.

Reproduce the five edited files with:

```sh
SKETCHPAD_WRITE_CURVE_FIXTURE=1 cargo test curve_ --quiet
source /tmp/sketchpad_verify_env.sh
for curve_case in point_locus function parametric polar parameter_locus; do
  cp "/tmp/curve_style_${curve_case}.gsp" "/tmp/curve_style_${curve_case}_native_copy.gsp"
  SKETCHPAD_WINEPREFIX=/home/w/.cache/sketchpad_rs_wine \
    SKETCHPAD_WINE_CAPTURE_PREFIX="/tmp/curve_style_${curve_case}" \
    dev/wine_roundtrip.sh "/tmp/curve_style_${curve_case}_native_copy.gsp" || exit
  cmp "/tmp/curve_style_${curve_case}.gsp" "/tmp/curve_style_${curve_case}_native_copy.gsp" || exit
done
```

Each native script checks that Sketchpad opened the requested filename before
saving. Captures retain native selection outlines, which surround the visible
styled strokes.

All five edited files opened and saved byte-identically in Sketchpad 5. The
saved files are retained under `tests/fixtures/curve_style_*_native_verified.gsp`,
and the existing five-case regression compares its encoded bytes directly with
these native saves:

| Fixture suffix | Bytes | SHA-256 |
| --- | --- | --- |
| `point_locus` | 2376 | `fa67cdecca0efe40f4c4fbb40825ddfcaf20429de9d66937fd35afa32e77165c` |
| `function` | 2734 | `48c656ac72942f2389e5fd356916a7c18370c4c7ee88c068f4d6654298028f05` |
| `parametric` | 2928 | `780d54b365e900100e557769a4e2d4323bc0f3899a598403d70f3da2fe680e7c` |
| `polar` | 2928 | `1b9f036557a91b9953a1287f0f921cce105a0b172e55d6c77196418e3252abf5` |
| `parameter_locus` | 3094 | `30f41af8d924cfda6057f1cfa3314fb492a9bafff4c730897dc64ea4db6d3dda` |

The first four default native captures show the styled curves. The parameter
locus lies outside the default native viewport; a separate 1000-by-800 native
capture shows its thick cyan stroke and saves the same bytes again. That capture
is `/tmp/curve_style_parameter_locus_fullscreen_before_save.png`. Its regression
fits the SVG viewport to the defined sampled geometry.
