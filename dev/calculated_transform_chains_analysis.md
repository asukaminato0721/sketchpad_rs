# Calculated transforms of transformed segments and circles

Native codes 33 and 29 represent scalar-ratio dilation and degree-valued
rotation. Their geometry evaluators already recurse into supported transformed
segments and circles, but the import, construction, and save guards required
primitive sources. Those guards now use the established supported segment and
circle kinds. The native source header subtypes remain 14 and 28, respectively;
class, reference ordering, numeric units, and other payload checks are retained.
Polygon source variants are a separate ongoing audit.

The regression starts from the native verified vector translation of a rotated
segment or circle. It adds a scalar parameter r=2, dilates the translated shape
about point 1, then rotates that calculated image by a 90-degree calculation.
It measures the result, edits r to -3, and checks the changed positive length or
radius. It also saves/reopens, copies the full dependency chain, and deletes the
original ratio parameter without deleting the copied chain. Existing editor
marked-value transform commands use these document APIs.

All four actual Rust-writer outputs open and save byte-identically in Sketchpad 5:
initial segment and circle documents are each 2,896 bytes; their negative-ratio
edits are each 2,898 bytes. The four retained `calculated_transform_chain_*_native_verified.gsp`
fixtures are exact byte assertions in the regression. Each native script must
exit successfully before comparing its saved output.

Generate the files with
`SKETCHPAD_WRITE_TRANSFORM_CHAIN_FIXTURE=1 cargo test calculated_transforms_accept_live --lib`.
Native capture prefixes are `/tmp/calculated_transform_chain_{segment,circle}_native`
and `/tmp/calculated_transform_chain_segment_edited_native`. The first circle
edit probe ended on an untitled empty window without a native error dialog;
a retry of the same bytes under `/tmp/ctc.gsp` opened and saved byte-identically,
with capture prefix `/tmp/calculated_transform_chain_circle_edited_retry`.
No record checks were relaxed to address that automation failure.
