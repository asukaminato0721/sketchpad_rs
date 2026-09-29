# Data-defined picture functions

Scope: `Number > Define Function from Drawing or Picture`, currently bounded to
independent PNG pictures whose affine transform is axis-aligned and whose alpha
mask has one top-edge sample in every pixel column.

## Native evidence

Sketchpad 5 creates native object code 122 with two references: the source
picture and the active coordinate system. If the page has no coordinate system,
the command creates the standard six-object Cartesian frame first. The observed
group uses records `07d0, 07d2, 07d3, 0907, 07d5, 07d8, 07d7`; the second word
of `07d3` stores the default smoothing value 3. A native-created picture
function decodes as `DataFunction { source: 1, frame: 7, smoothing: 3 }`.

The port generated an eight-object file from `image_tiny.png`: picture, default
frame, and code-122 function. Native Sketchpad opened it, displayed the
picture-function caption, and saved the 3,056-byte file without changing a
byte. Its SHA-256 was
`86d50ed4e18358679a0573bfb82829b618e481dc6781fd6e3d65d60dfde62fe6` before
and after the native save.

## Supported behavior and boundary

The evaluator samples the first nontransparent pixel in each picture column,
applies the stored moving-average smoothing radius, converts the samples through
the referenced coordinate frame, and linearly interpolates between columns.
Creation rejects rotated/sheared images, empty columns, and shapes that cannot
be represented as a single-valued function.

Native drawing sources use the same code-122 record family, but their source
object and three opaque `0907` fields differ. Drawing authoring, non-alpha image
edge extraction, the smoothing property UI, approximate derivatives, and direct
graphing remain outside this proved writer boundary.

Reference: [Functions and data-defined functions](https://keyonline.keypress.com/gsp-reference-center/objects_functions.htm).
