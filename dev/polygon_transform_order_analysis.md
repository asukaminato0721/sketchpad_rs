# Native polygon drawing order and transform chains

The signed 32-bit value at offset 20 of a 28-byte class-4 object header is
interior drawing order. It is neither a vertex count nor a transform flag.
The corpus contains native values from negative ranks through positive ranks,
including different ranks for the same polygon subtype. Vertex count remains
encoded by subtype `49 + count`.

Native coincident orange and green polygons prove the field's effect directly.
`polygon_order_original_native.gsp` uses primitive rank 0 and calculated dilation
rank 1, and the green interior draws last. Swapping just those four bytes in each
header produces `polygon_order_swapped_native.gsp`, where orange draws last.
`polygon_order_negative_native.gsp` assigns the green polygon rank -1; orange
again draws last. This distinguishes ascending signed order from unsigned order.
All three files opened and saved byte-identically in Sketchpad (2326 bytes).
The source geometry is identical in all three fixtures. Native copy/paste in
`polygon_order_copied_native.gsp` preserves the source ranks 0 and 1 in copied
polygons 11 and 12, rather than allocating ranks above existing objects.
`tests/polygon_order_replay.txt` authors the coincident source geometry.

Objects and clipboard objects now retain optional signed interior order.
Import, source patching, new-object writing, clipboard remapping, multi-page
writing, and roundtrip validation preserve it. New primitives retain the
existing writer default 0 and new transformed polygons retain default 1.
SVG and GPU rendering stably sort visible polygon fills by signed rank within
the existing polygon fill slots; other drawing slots and picking order remain
unchanged. Ties retain object order. Existing supported polygon transform
schemas accept any signed rank while still checking header length/class,
subtype, source family, references, and constraint payloads.

Native code 16, 29, and 33 polygon sources may themselves be supported transformed
polygons. Vertex count is recovered by following their source chain to a known
primitive polygon, requiring decreasing object IDs, the same page, and live
known objects. No count is inferred from rank or from opaque unsupported kinds.
Other transform families retain their previous source-kind constraints.
`polygon_rotation_of_vector_corpus_native.gsp` isolates actual objects 22 and 23
from the sibling corpus's 密铺(万缘风): translated quadrilateral 22 has rank 1,
and calculated rotation 23 of that translated source has rank 2. This 4480-byte
file opened and saved byte-identically in Sketchpad. Its geometry lies outside
the cropped probe window; the capture is a schema/file proof, not a claim about
visible coordinates.

`tests/polygon_transform_chains_replay.txt` uses the actual document APIs to
create calculated rotation, calculated dilation of that rotation, vector
translation of that dilation, and further calculated rotation/dilation.
Both generated and source-edited outputs opened and saved byte-identically in
Sketchpad (2792 and 2794 bytes). The retained verified fixtures check live
geometry, source movement, parameter changes, copy/remapping, and deletion.
Regression tests also reject cyclic, cross-page, deleted, and unsupported count
chains, and verify arbitrary signed ranks across existing transform families.

Legacy GSP4 headers are a separate observed layout. Clock(2), object 79, is a
native polygon with a 16-byte class-4 header and no drawing-order field. The
whole original is retained as `legacy_clock_original.gsp`. Its rank remains
`None`; an unchanged save preserves all original bytes. The isolated
`legacy_polygon_rank_original.gsp` retains the authentic GSP4 metadata, free
point headers, polygon header, and reference schema, rebinding only the four
vertices. Sketchpad opened and saved it byte-identically (1492 bytes). Source
geometry edits preserve the short polygon header. The actual document writer's
point-moved output also opened and saved byte-identically (1492 bytes), retained
as `legacy_polygon_rank_edited_native_verified.gsp`. An attempt to assign an
absent rank fails rather than extending an opaque legacy header. A diagnostic
that mixed this old header with GSP5 point/header metadata failed native opening;
it was discarded in favor of the coherent GSP4 fixture. The separate
[legacy fill-style audit](legacy_polygon_styles_analysis.md) establishes its
implicit 75% opacity, RGB edits that preserve the short header, and copied
polygon style in a fresh modern document. Native copying within the legacy
document performs a complete header/metadata upgrade.
