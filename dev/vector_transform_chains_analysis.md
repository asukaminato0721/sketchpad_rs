# Vector translations: names and transformed sources

The sibling fixture `两个三角形标记全等.gsp` and its paired HTM explicitly
construct vector translations of B and C by A→A′, naming the resulting points
`B'` and `C'`. Their native code 16 headers have class 1 and point subtype 4;
their 0x07d5 label and 0x07d8 style records use the same checked label schema
as other supported transformed objects. The existing decoder recovered their
geometry but discarded their names and marked the document unsafe because
class 1 was excluded from the label predicate.

The first twelve construction objects, excluding the subsequent unsupported
congruence annotations, form `labeled_vector_points_native.gsp`. Native Sketchpad
opened and saved this document byte-identically (2,842 bytes). Tests retain the
original whole source too, verify HTM positions B′=(467,250) and C′=(404,161),
move the live vector endpoint, save/reopen, copy with dependency remapping, and
delete the original vector endpoint with its dependents. The typed point label
record and its opaque suffix survive these operations.

A separate code 16 import gap rejected supported transformed segment and circle
sources. For example, the Hilbert-curve tutorial's second-page object 21 translates
a rotated segment; `平面截圆柱面的展开图.gsp` contains a vector translation
of a calculated-ratio scaled segment. The existing geometry evaluator already
recurses into these sources. The decoder and validator now accept supported
segment and circle kinds rather than requiring their primitive construction
kinds. Ray and line transformations remain distinct, unimplemented families.

`vector_rotated_{segment,circle}_native.gsp` append the established native code 16
record to native fixed-rotation documents, referencing the rotated shape and
free vector endpoints. Both cross-open and save byte-identically (2,268 bytes).
The vector translates the rotated segment from (170,110)→(170,70) to
(210,150)→(210,110), and translates the rotated circle to center (210,150),
radius 40. Native captures are `/tmp/vector_rotated_segment_native_retry_before_save.png`
and `/tmp/vector_rotated_circle_native_before_save.png`.

`tests/transform_vector_chained_shapes_replay.txt` exercises the actual Rust writer
for both initial constructions and source edits. All four actual Rust-authored
port files cross-open and save byte-identically in native Sketchpad (2,268 bytes
each); before-save captures use the matching `/tmp/vector_rotated_*_port` names. The writer already inserts new object groups before
existing 0x2328/0x232d page markers. An initial handcrafted candidate incorrectly
inserted the group after the marker and native Sketchpad rejected it; correcting
that probe ordering resolves the rejection without loosening any schema checks.
The failed capture remains `/tmp/vector_rotated_segment_native_open_failed.png`.

Regression tests additionally add constrained points on the translated shapes,
measure their live values, edit the underlying source geometry, copy/paste the
shape chains, reopen the saved result, and delete a source with its dependents.
