# Legacy polygon fill styles

The isolated `tests/fixtures/legacy_polygon_rank_original.gsp` retains the
authentic 16-byte class-4 polygon header from Clock(2), object 79. Its native
header is `080000000000040035000100ff000000`: code 8, subtype 53, and stored
RGB (255,0,0) at offsets 12–14. There is no opacity or drawing-order field.
The existing color reader already recognized this header. The actual gap was
the fallback opacity of 32767 (50%), rather than the native legacy fill.

Sketchpad 5 opens and saves the untouched 1,492-byte file byte-identically.
An interior screenshot pixel is (255,64,64) over a white background, establishing
its implicit 75% fill. A change only to RGB (0,128,255) also opens and saves
byte-identically at 1,492 bytes, keeping every short header intact. Its interior
pixel is (64,159,255).

A native Ctrl+C/Ctrl+V probe supplies exact integer evidence for that opacity.
Sketchpad upgrades the complete legacy document to modern 28-byte headers and
metadata. Both the original and copied polygon have explicit opacity bytes
`ff bf` at offsets 16–17: `0xbfff`, or 49151. Both have drawing rank 0, and the
copy is translated by (20,20). That complete upgrade is separate from this
narrow color/header change; the native conversion also changes old point-style
fields.

The reader now gives the proven short code-8/class-4 polygon layout its native
implicit opacity of 49151. Color edits write only offsets 12–14, preserving
the 16-byte header and `interior_order = None`. The low-level style writer
rejects any other requested opacity before changing RGB, so an unstored setting
cannot be silently lost. Modern polygon headers continue to store their explicit
opacity as before.

Copying the polygon and its vertices into a fresh modern `Document` retains
its effective RGB/opacity in an ordinary modern polygon header with rank 0.
The actual Rust writer output opens and saves byte-identically at 2,336 bytes,
with the same native blue interior pixel (64,159,255). This proves the bounded
modern clipboard destination without extending the original short header.

| Retained fixture | Bytes | SHA-256 |
| --- | ---: | --- |
| `legacy_polygon_blue_native_verified.gsp` | 1492 | `812a022097f5e2128092cfcb41a029bee76ad9f2849a16ea72cbda3e541ee37d` |
| `legacy_polygon_copy_native_verified.gsp` | 2688 | `52ded035413b1ba7395a7e348cead40e849030562abcc9f0af0b10c0e0e8c664` |
| `legacy_polygon_modern_copy_native_verified.gsp` | 2336 | `262a5b460f358ee51ec2f881a2f9e9b306e96834947f3fc935b5bd51c56c9714` |

The native copy fixture intentionally contains the application's full upgrade;
it is evidence for the implicit opacity, rather than an exact Rust conversion
golden. The RGB-only fixture and fresh-modern clipboard fixture are exact byte
assertions of the actual document writer. Native process success was checked
before each byte comparison; screenshots and pixels were inspected separately.

`style_records` regressions cover RGB, short/modern opacity equivalence, and
atomic rejection of unsupported opacity changes. The rendering/API regression
checks red and edited blue SVG fills at the native opacity, undo/redo, preserved
short-header/rank metadata, and fresh-modern clipboard save/reopen. Canvas uses
the same decoded `VisualStyle::Polygon` through its existing fill path; no color
or fallback rendering changes were needed. Generate the modern clipboard file
with `SKETCHPAD_WRITE_LEGACY_STYLE_FIXTURE=1 cargo test legacy_polygon_`.
Native capture prefixes are `/tmp/legacy_polygon_style_original`,
`/tmp/legacy_polygon_style_blue`, `/tmp/legacy_polygon_style_copy`, and
`/tmp/legacy_polygon_modern_copy`.

The modern clipboard golden was regenerated after the independent legacy
free-point audit established implicit Small vertices. The actual writer output
again passed native byte-identical saving at 2,336 bytes; its four point size
bytes are now 1. See [legacy point styles](legacy_free_point_style_analysis.md)
and [same-document paste conversion](legacy_document_paste_analysis.md).
