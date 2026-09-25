# Polar translation of a selected segment

`tests/transform_segment_base_replay.txt` creates a segment with endpoints `(100, 200)` and `(200, 200)`. Selecting its interior and using the native Transform > Translate dialog with the default 1 cm distance and 90° angle produces `tests/fixtures/polar_segment_translation_original.gsp`. The translated line appears about 37.795 pixels above the source, without visible endpoint objects.

The native file contains one additional object, code 21, referencing source segment 3. Its object header has subtype 14 and the segment stroke style, while its 60-byte `0x07d3` payload matches fixed polar point translation. The port now distinguishes code 21 point and segment objects by header subtype and source kind. The translated segment uses the live source endpoints plus the polar offset, and is included in stroke rendering, hit testing, styles, and save validation.

`tests/transform_segment_translation_replay.txt` creates the translated segment in Rust, moves one source endpoint, and saves. Sketchpad 5 opened the generated file, displayed the updated pair of segments, and saved it byte-identically. Import and editor tests verify the construction and its dependency on both source endpoints.
