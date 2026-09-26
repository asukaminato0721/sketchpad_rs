# Measurement-dependent calculation

The native fixture `tests/fixtures/calculation_length_times_two_native.gsp` was made by selecting a segment length in Sketchpad 5's Calculator and multiplying it by 2. Its code-48 object group is `assets/measurement_times_two_group.bin` (176 bytes): header, one-reference record, expression record, text-style record, end. The reference at group offset 48 points to the source measurement. The expression's final three words at offsets 152–157 encode source operand `0x6000`, multiplication `0x1002`, and a one-digit literal. The on-page anchor is at offsets 72–75. A generated factor-3 fixture is `tests/fixtures/calculation_length_times_three_port.gsp`.

`MeasuredCalculationGroup` accepts this measured-length × one-digit form. `Document` resolves it from the current segment geometry, so dragging either endpoint updates the value and a calculation may supply a marked distance. Import and byte-preserving save of the native file, import of the generated file, live source edits, and native Sketchpad open/save were checked. A generated sketch with the source measurement as object 5 also displayed the expected 14.98 cm in native Sketchpad.

The native `calculation_distance_times_two_native.gsp` uses the same code-48 format with a point-distance source. Its group differs in source reference, anchor, and opaque cache words; the generated `calculation_distance_times_three_port.gsp` opened, displayed 7.94 cm, and saved byte-identically in Sketchpad 5.

The native `calculation_perimeter_times_two_native.gsp` also uses the same code-48 formula linked to a polygon-perimeter source. `calculation_perimeter_times_three_port.gsp` displayed 28.58 cm and saved byte-identically in Sketchpad 5.

Native `calculation_circumference_times_two_native.gsp` and `calculation_radius_times_two_native.gsp` share the code-48 layout. Generated factor-three versions displayed 49.87 cm and 7.94 cm and saved byte-identically in Sketchpad 5.

Native `calculation_arc_length_times_two_native.gsp` uses the same code-48 format. The generated factor-three file displayed 13.46 cm and saved byte-identically in Sketchpad 5.

`calculation_marked_distance_port.gsp` uses the length calculation as a marked distance for point translation. It opened and saved byte-identically in Sketchpad 5; moving the segment endpoint in the port updates the translated point. The calculation also survives copying and pasting with its measurement and geometry parents, and renumbering after an earlier unrelated object is deleted.

Native `calculation_length_divide_two_native.gsp` differs from multiplication by operator word `0x1003` at group offsets 154–155 and opaque cache bytes. The generated divided-by-three file displayed 1.66 cm and saved byte-identically. Division by zero is rejected.

A two-digit factor adds one u16 digit token: the code-48 group grows from 176 to 178 bytes, the `0907` payload from 98 to 100, and the expression token count from three to four. `assets/measurement_times_twelve_group.bin` is the native template. Native length × 12 and length ÷ 12 files import byte-identically; generated length × 23 and length ÷ 12 files displayed 114.85 cm and 0.42 cm and saved byte-identically in Sketchpad 5.

A code-48 length calculation can itself be the source of another code-48 length calculation. Patching a second calculation's one-reference record from the original length measurement to the first calculated value made native Sketchpad display `(AB·3)·2 = 29.96 cm`; it saved the file byte-identically. The port now imports, evaluates, creates, and remaps these chains. Its generated multiplication and subsequent division files displayed 29.96 cm and 9.99 cm in native Sketchpad, and both saved byte-identically. After moving the original endpoint, native Sketchpad displayed 6.55 cm, 19.64 cm, 39.28 cm, and 13.09 cm for the measurement and its three dependent results; the moved file also saved byte-identically.

Other source types, operators, operands above 99, and expression nesting still need native fixtures before broadening the writer.

## Two length sources

The native-verified sum fixture was constructed by replacing the literal operand in a length calculation with a second source operand. Sketchpad 5 displayed `AB + CD = 7.94 cm` for 100- and 200-pixel segment lengths and saved the file byte-identically. The 180-byte code-48 group has a 12-byte `07d2` payload `[2, first, second]` and three `0907` tokens: `0x6000`, addition `0x1000` or subtraction `0x1001`, and `0x6001`. The expression begins four bytes later than the single-source form. Its verified template is `assets/length_sum_pair_group.bin`.

The port imports and writes this form, records both dependencies, evaluates live sums and signed differences, and remaps both embedded references on Copy/Paste and Clear. Number > Calculate uses `a + b` or `a - b` for two selected lengths, preserving selection order. `tests/replay_calculation_two_lengths.txt` generates both operations and moves one endpoint. Native Sketchpad displayed 7.94 cm and −2.65 cm before the move, then 9.26 cm and −1.32 cm afterward; both generated files opened and saved byte-identically. Mixed-unit pairs need separate unit and record verification.

The same two-source group also supports product token `0x1002` and quotient token `0x1003`. Native Sketchpad displayed 14.00 cm² for the product and 0.50 without units for the quotient, saving both patched probe files byte-identically. The port now preserves those output dimensions: products and ratios cannot supply a marked distance or a length-only calculation. `tests/replay_calculation_two_lengths_product_ratio.txt` generates both forms and moves an endpoint; native Sketchpad displayed 21.00 cm² and 0.75 after the move, and both generated files saved byte-identically. A denominator that later becomes zero displays undefined; creation with a zero denominator is rejected. Mixed-unit operands and further area or unitless calculation chains remain pending.
