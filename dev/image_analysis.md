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

## Pictures attached to two or three points

The native reference documentation defines picture attachment to one, two, or
three selected points. Two points place opposite corners of an imaginary
rectangle. With three points, the first, second, and third selected points are
the bottom-left, bottom-right, and top-left corners of a parallelogram. See
<https://referencecenter.dynamicgeometry.com/gsp-reference-center/importing_pictures.htm>.

The corpus stores the two-point code-85 references as
`[bottom_right, top_left]` and the three-point code-100 references as
`[top_left, bottom_right, bottom_left]`. Both use the same class-16/subtype-78
header and embedded resource index as other pictures. Their strict groups are
`07d0, 07d2, 090c, [optional empty 232e], 1f44, 07d7`. The final dimension
pair in `090c` must match the selected PNG resource. The first pair is a live
cached extent and may be zero when the attachment is collapsed.

The document model represents independent, one-point, rectangle, and
parallelogram placements explicitly. It resolves the attached affine matrix
from current point positions and PNG dimensions, so canvas rendering, SVG,
picking, iteration, and dependent geometry share the same live result. The
resource table remains read-only. The strict import recovers all 30 drawing
code-85 and all 106 drawing code-100 pictures in the corpus. Their newly live
dependencies recover another 115 objects.

Crop groups are also cataloged independently of the single-picture writable
wrapper. The strict code-108 layout accepts its observed optional empty
`0x232e` marker, checks the recorded source dimensions against the referenced
PNG, and still requires a live polygon parent. This restores all 39 remaining
drawing crops. Together, the attached pictures, crops, and downstream objects
recover 290 slots.

## Safe helper scope

`ImageFile::parse` bounds-checks GSP records, verifies PNG chunk lengths and
CRCs, checks dimensions across PNG/resource/object records, and requires
finite affine coefficients. It preserves all unrecognized bytes on exact
reencode. For a document containing exactly one independent affine picture,
including one with unrelated geometry, it can update visibility and transform
or replace the PNG transactionally.
The generated PNG replacement reopened and saved in Sketchpad 5.

The helper decodes and writes the observed crop group. The document model
keeps its picture and polygon dependencies live, clips it in the canvas and
SVG renderer, and safely updates the known source visibility and transform
fields. Pixel replacement inside a cropped construction remains guarded. The
writable helper refuses the multi-resource Paste Picture file entirely. More
than one writable picture, opacity, external clipboard import, and fresh
attached-picture authoring remain unimplemented; imported one-, two-, and
three-point attachments are readable and live.

## Native integration and generated import

The shared document model now decodes one independent picture alongside
ordinary geometry, imports a PNG into a single-page sketch, moves it with
undo/redo, and writes the `0x0514` resource and code-0 picture group. The
native File menu has **Import PNG Picture…**, and Edit has **Crop Picture to
Polygon** when one supported picture and polygon are selected. The same affine
viewport is used for hit testing, dragging, screen placement, crop clipping,
and SVG export, which embeds the PNG as a data URI. `gsp_probe replay tests/replay_image.txt` creates
`image_port_imported.gsp`, moves the image, and creates
`image_port_imported_moved.gsp`.

Sketchpad 5 under Wine opened the first generated file, showed the 16×16
yellow/blue PNG beside a point, and saved it byte-identically (2,320 bytes).
It then opened the moved file, dragged the picture from raw `(160,155)` to
`(184,170)`, and saved `image_port_imported_dragged_wine.gsp`. The port parsed
that result as an editable picture and preserved the original PNG and affine
translation on another save. Wine recompressed the PNG, reducing the saved
file from 2,320 to 2,112 bytes.

The Linux Wayland desktop smoke capture (`/tmp/sketchpad_native_image.png`)
showed the imported raster and point in the native window. The generated SVG
was converted to PDF with `rsvg-convert`; rasterizing the PDF showed the same
picture, so the print pipeline includes it.

Documents with a recognized crop decode as editable live dependencies and
round-trip byte-identically if untouched. Multiple picture resources still
open read-only. PNG input is decoded before the native menu or CLI mutates the
document; malformed input leaves the sketch unchanged.

Standalone tests:

```sh
rustc --edition=2024 --test src/image_records.rs -o /tmp/sketchpad_image_records_tests
/tmp/sketchpad_image_records_tests
```

## Vector-translated attached pictures

The reachable code-16 picture family uses class 16/subtype 78, references
`[source picture, vector start, vector end]`, and the exact record layout
`07d0(28), 07d2(16), 090c(16), 07d7(0)`. The `090c` payload repeats the source
PNG width and height twice. Every observed source is a previously validated
code-99 point attachment or code-100 parallelogram attachment.

Import requires an earlier decoded picture, two earlier live points on the same
page, and four cache words equal to
`[width, height, width, height]`. The resulting placement reuses the source PNG
and adds the live endpoint displacement to the source's current affine
translation. The full scan decodes all 16 currently reachable roots and five
dependent objects, lowering unsupported slots 3,815→3,794. It leaves all 1,599
files byte-exact. Fresh authoring and picture-resource mutation remain guarded.

## Shared affine picture transforms

Picture placement now stores the same typed `TransformDefinition` used by
geometric transformations. The evaluator resolves one live operation matrix
and composes it after the source picture's current pixel-to-document matrix.
This keeps translations, rotations, dilations, and reflections consistent
across canvas drawing, picking, SVG, and mapped image instances without adding
one placement variant per native opcode.

The strict importer accepts class-16/subtype-78 transform groups only when the
source is an earlier decoded picture on the same page, every transform operand
has its established point or numeric role, the group contains only its header,
reference, optional literal constraint, one 16-byte size cache, and terminator,
and that cache equals `[width, height, width, height]`. The compiled scan
restores 37 roots—codes 16, 19, 24, 28, 29, 30, 33, and 34—and 22 downstream
objects. Unsupported slots fall 3,794→3,735; ten additional documents decode
completely, and all 1,599 originals remain byte-exact. Code-29 transforms of
cropped pictures remain separate: their source is a clip object rather than a
plain picture and their four cache words encode two different rectangles.

## Rotated point attachments

The original code-99 admission required the stored rotation to be zero. Four
later corpus objects use the same strict class-16/subtype-78 attachment layout
with finite rotations. Their point parents include both ordinary transformed
points and a free point; their duplicated `090c` dimensions still match the PNG.

The decoder now constructs the point-relative pixel affine from the stored
positive scales and rotation, translating the pixel center to the origin before
the rotation and scale. `image_transform` then adds the current anchor position,
so movement, canvas rendering, SVG, picking, and mapped instances share the
same result. Two of the four groups carry one additional hidden `07d5` internal
metadata record with the exact native name `[1]`; the other two carry the
already-supported empty `232e` marker. Only that exact observed payload is
accepted, preserved, and kept out of the visible object-label model.

This bounded extension restores four code-99 roots and one downstream object.
Unsupported slots fall 3,735→3,730, all-kind-complete documents rise
1,167→1,170, drawing-complete documents rise 1,196→1,199, and all 1,599
originals remain byte-exact. The compiled audit finds no remaining unsupported
class-16/code-99 picture profile on a drawing page. Twelve code-99 embedded-tool
prototype slots remain outside this admission. Fresh attachment authoring and
resource-table mutation remain guarded.
