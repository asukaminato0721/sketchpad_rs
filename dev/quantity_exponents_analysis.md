# Quantity exponents and pixel calculation literals

Checkpoint 37 extends the shared calculator AST to exponents carrying physical
or angle units. It does not introduce shape/transform enum combinations.
Copying and custom-tool implementation remain deferred.

The checkpoint 36 audit found 92 calculation unit failures with known numeric
inputs, including 82 instances of `trunc(v1 * 5^v2)+1` with two Length inputs.
These are dependency roots; their recovery also makes downstream calculations
and geometry available. The complete corpus scan recovers 398 drawing objects,
including 392 calculations. These counts include dependency recovery rather
than 398 independent expression variants.

## Native evidence

`quantity_exponents_native_created.gsp` was created in Native Calculator from
`source_exponents_base_port.gsp`. The controls are `2^radius`, `2^area`,
`2^angle`, and `radius^angle`. All results are unitless; the source magnitudes
remain in centimeters, square centimeters and degrees respectively. Native
values at the base geometry are 3.01, 241.74, 1.24×10²⁷, and 1.16×10¹⁸.
The four retained `quantity_*_exponent_native_created.png` captures show the
calculator outputs. Reproduce with `quantity_exponents_wine_setup.sh` through
the serialized `wine_roundtrip.sh` helper on a temporary copy of the base.

`quantity_literal_exponents_native_created.gsp` supplies independent controls
for `radius^2°`, `radius^(2px)`, `n^2°`, and `n^(2px)`. The native pixel suffix
is `0x0200`, and the literal magnitude becomes `2/37.79527559055118` centimeters.
Native canonical captions show `0.0529167 cm`; the results are unitless 2.52,
1.02, 4.00, and 1.04. The retained captures whose names end in `radius_length`
and `scalar_length` refer to these **pixel** controls, not centimeter literals.
The reproduction script `quantity_literal_exponents_wine_setup.sh` labels them
`radius_pixels` and `scalar_pixels` to remove that ambiguity. No claim is made
for suffix `0x0202` or pixel Parameter records.

The original `三角函数线.gsp` and its paired HTM define object 71 as
`A 5 B ^ * @trnc 1 +`, with Distance objects 68 and 70 as its sources. The
retained `quantity_exponent_trigonometry_corpus.gsp` now imports that calculation
as Scalar. Moving constrained source point 69 updates the distance and formula;
undo/redo restores them. Unedited encoding preserves the whole original file.
This original also has a constrained-source Movement button outside the
currently supported edited button layout; this control does not claim edited
serialization for that unrelated button.

## Implementation and writer verification

The shared unit inference uses the exponent's numeric magnitude and returns
Scalar for a typed exponent. Existing dimensional rules are unchanged for
bare, dimensionless integer exponents: `radius^2` remains Area. Referenced,
grouped, computed and negative exponents keep the previously established Scalar
classification. Nonfinite and invalid numeric domains remain undefined.

The same calculation parser recognizes `0x0200` Length literals and converts
pixel magnitude into centimeters; the authoring lexer accepts `px`. Original
token bytes are preserved. Port captions may retain pixel notation while Native
shows its centimeter normalization; pixel-identical display is outside scope.

The actual API writer uses `add_parameter_calculation`, `add_multi_calculation`
and `add_calculation_expression` for all eight exponent controls and a standalone
`2px` Length calculation. Both original and edited 17-object outputs were
opened and saved by Native Sketchpad using the unchanged serialized helper.
The helper returned 0 and independent `cmp` passed for 4174 and 4176 bytes.
The retained `quantity_exponents_writer_{original,edited}_native_verified.gsp`
and PNG files show those actual outputs, not calculator-created substitutes.
The edit increases the radius from 60 to 90 pixels, changes the angle from 90°
to 45°, and changes n from 2 to −2. Native and port agree that `n^2°` stays 4
while `n^(2px)` becomes undefined. A regression reconstructs both exact files
through the API and exercises strict round trips and undo/redo.

The final scan `/tmp/sketchpad_corpus_checkpoint37_final.log` retains all 1599
exact source round trips, 89 fully decoded documents and unchanged prototype
counts. Unsupported slots fall from 65014 to 64616 (63427 drawing, 1189
prototype). Pixel support itself adds no further corpus recoveries beyond the
typed-exponent change.

Scoped regression gates pass 623 library, 109 editor, 34 numeric-import and
4 coverage tests, excluding the deferred copying/custom-tool scope. The four
focused quantity-exponent regressions pass, reproduction scripts pass `bash -n`,
and the changed tracked source/documentation files pass `git diff --check`.
