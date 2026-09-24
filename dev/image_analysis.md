# Embedded pictures and picture commands

## Source and original behavior

The supplied Sketchpad Help describes **Edit > Paste Picture** for imported
`.jpg`, `.gif`, `.png`, `.bmp`, and `.tif` images. With no points selected the
result is independent; with one to three selected points it is attached to
those points. **Edit > Crop Picture to Polygon** requires one picture and one
polygon, hides the original picture, and creates a dynamic cropped image.
These behaviors are described in
`GSP5/GSP5.app/Sketchpad Help/Reference Center/importing_pictures.htm` and
`menus_edit_croppicture.htm`.

The smallest image corpus example found was
`../gsp-rs/tests/fixtures/未实现的系统功能/插入图片.gsp`: one object, 8,446 bytes,
76 records. It is copied to `tests/fixtures/image_original_insert.gsp`.
Sketchpad 5 under Wine opened and saved it without changing any record. Its
picture appeared at raw canvas position `(118,112)` and measured `255×158`.
The code-0 object is distinguishable from a point by these records:

| Record | Payload | Observed role |
| --- | ---: | --- |
| `0x0514` | variable | Global resource: little-endian width and height (`u32`, `u32`) followed immediately by a complete PNG. |
| `0x07d0` | 28 bytes | Code-0 object header; bit `0x10000` of its first word hides the picture. |
| `0x090c` | 16 bytes | Four `u32` dimensions, both pairs equal to PNG width and height in the independent picture. |
| `0x08a8` | 48 bytes | Six little-endian `f64` affine coefficients `[scale_x, shear_x, x, shear_y, scale_y, y]`. |
| `0x1f44` | 2 bytes | Zero-based `u16` index into the document's `0x0514` resources. |
| `0x07d7` | 0 bytes | Object terminator. |

The original fixture has resource index `0` and affine values
`[1,0,118,0,1,112]`. The PNG is `255×158` with a valid PNG chunk stream and
CRCs. An adjacent `0x2328` record records selection; it is not part of the
image group.

## Rust modification cross-open

`src/image_records.rs` replaced the PNG with the 16×16 image in
`tests/fixtures/image_tiny.png`, changed both dimension pairs to 16×16, and
set affine values `[4,0,118,0,4,112]`. The resulting one-picture file is
`tests/fixtures/image_port_tiny.gsp` (2,224 bytes, 76 records). Sketchpad 5
opened it, visibly drew the yellow and blue image at 64×64 screen pixels, and
saved all 76 records byte-identically. Dragging the image 32 pixels right and
20 pixels down changed affine translation to `(150,132)`. The original app
also recompressed the embedded PNG and rewrote style metadata; its saved
result is `tests/fixtures/image_port_dragged_wine.gsp`. The helper parses and
reencodes that file byte-identically.

## Paste and crop variants

In Sketchpad 5, copying the selected Rust-generated image and pasting with
Ctrl+V created a second independent picture. The saved file is
`tests/fixtures/image_original_pasted.gsp` (two objects). It has a second
`0x0514` resource and a second code-0 picture group. The app rasterized the
scaled image into a new 64×64 PNG: the pasted group has 64×64 in `0x090c`, an
identity affine transform, and resource index `1`. The test used Sketchpad's
own clipboard. Direct import from an external X11 clipboard was not tested:
`xclip` and `xsel` are absent in the Wine/Xvfb environment.

For cropping, a Rust-generated file combined the 16×16 picture with three
points and a polygon that overlapped it. Sketchpad 5 opened the five-object
file. After selecting picture and polygon and choosing Edit > Crop Picture to
Polygon, the app hid the source picture and displayed the clipped portion.
The result, `tests/fixtures/image_original_cropped.gsp`, has six objects.
The new code-108 object group contains a 28-byte `0x07d0` header, a 12-byte
`0x07d2` reference list `[source_picture=1, polygon=5]`, a 16-byte `0x090c`
size record, and `0x07d7`. It shares the source PNG resource. The original
source picture's hidden bit changed from clear to set.

## Safe helper scope

`ImageFile::parse` bounds-checks GSP records, verifies PNG chunk lengths and
CRCs, checks dimensions across PNG/resource/object records, and requires
finite affine coefficients. It preserves all unrecognized bytes on exact
reencode. For a document containing exactly one independent affine picture,
including one with unrelated geometry, it can update visibility and transform
or replace the PNG transactionally.
The generated PNG replacement reopened and saved in Sketchpad 5.

The helper decodes the observed crop group for inspection but refuses edits
in a document with a crop, because the derived image's rewrite rules are not
established. It refuses the multi-resource Paste Picture file entirely.
Point-attached images, more than one picture, opacity, pixel
replacement inside a cropped construction, and external clipboard import
remain unimplemented.

## Native integration and generated import

The shared document model now decodes one independent picture alongside
ordinary geometry, imports a PNG into a single-page sketch, moves it with
undo/redo, and writes the `0x0514` resource and code-0 picture group. The
native File menu has **Import PNG Picture…**. The same affine viewport is used
for hit testing, dragging, screen placement, and SVG export, which embeds the
PNG as a data URI. `gsp_probe replay tests/replay_image.txt` creates
`image_port_imported.gsp`, moves the image, and creates
`image_port_imported_moved.gsp`.

Sketchpad 5 under Wine opened the first generated file, showed the 16×16
yellow/blue PNG beside a point, and saved it byte-identically (2,320 bytes).
It then opened the moved file, dragged the picture from raw `(160,155)` to
`(184,170)`, and saved `image_port_imported_dragged_wine.gsp`. The port parsed
that result as an editable picture and preserved the original PNG and affine
translation on another save. Wine recompressed the PNG, reducing the saved
file from 2,320 to 2,112 bytes.

Documents with a crop or multiple picture resources open for inspection and
round-trip byte-identically if untouched. Any edited save is refused with a
read-only picture error. PNG input is decoded before the native menu or CLI
mutates the document; malformed input leaves the sketch unchanged.

Standalone tests:

```sh
rustc --edition=2024 --test src/image_records.rs -o /tmp/sketchpad_image_records_tests
/tmp/sketchpad_image_records_tests
```
