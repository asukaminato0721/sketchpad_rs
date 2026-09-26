# Source-dependent exponents and native quantity syntax

Native Calculator probes establish that area^n, radius^n, and angle^n all produce
unitless results, even when the current parameter n is the integer 2. This is a
fixed unit classification; changing n does not change the output unit. The
native creation fixtures are `source_exponent_area_native.gsp` and
`source_exponent_quantities_native.gsp`. All sources remain dependencies, and
both geometric edits and numeric edits refresh their values.

Further probes expose a syntactic rule for dimensional powers:

| Native expression | Result unit |
| --- | --- |
| area^2 | cm⁴ |
| area^2.0 | cm⁴ |
| area^(2) | none |
| area^(1+1) | none |
| area^-2 | none |
| area^n, n=2 | none |
| area^2/1 | cm⁴ (the quotient is outside the power) |

These cases are preserved in `dimensional_exponent_forms_native.gsp`; the
negative-power preview was also verified in an independent probe. Native
Calculator ignores a leading plus entered at the beginning of the exponent,
so its +2 probe is saved as the same literal 2. Integer literal exponents retain
dimensions; fractional literals, unary minus, explicit grouping, computed
expressions, and referenced numeric values discard dimensions. The previous
implementation incorrectly evaluated constant exponent expressions when
inferring units. That rule has been replaced with the native syntactic rule.
The expression tree retains explicit grouping for unit inference while numeric
evaluation remains unchanged. Existing native expression bytes are preserved.

`tests/replay_source_exponents.txt` creates original and edited probes for
three quantity powers, scalar live iteration depth, and a numeric iteration.
The quantity files keep area^(1+1) alongside area^n, radius^n, and angle^n.
Changing n from 2 to 0.5 leaves all four results scalar. The numeric results
change from 62.6837, 2.52016, and 8100 to 2.81377, 1.25996, and 9.48683;
the constant expression stays 62.6837.

The live-depth probes translate by sqrt(area) while area^n supplies the depth.
With a 30-pixel circle radius, n=0.5 gives one step; n=2 gives three steps.
The numeric iteration maps n to area^n, with the area fixed within each
sequence. It advances a referenced exponent at the second source index and
refreshes its labels when n changes. The example uses zero-decimal labels
without rounding the carried state: n=0.5 gives labels 3, 6, and 58.
The exponent parameter is placed below the calculator captions to avoid
overlap. Tests also cover a negative dimensional base: a fractional exponent becomes undefined and suppresses dependent
iteration; restoring an integer exponent recovers the value and geometry.

Reproduce native creation with `source_exponents_base_port.gsp` and
`dev/source_exponents_wine_setup.sh`. The script creates the three source powers
and area^(1+1). For the five syntax probes, use the same base with
`dev/exponent_forms_wine_setup.sh`. Both scripts hide new calculations between
probes to keep the original measurement captions selectable. Native cross-open
checks operate on temporary copies via `dev/wine_roundtrip.sh` and compare each
native save with its original port-generated file.

All six generated originals/edits displayed the expected native values and
saved byte-identically in Sketchpad 5. The four iteration files were checked
again after moving the parameter and reducing numeric-label display precision;
the captions are readable and the native numeric labels agree with the port.
All three creation fixtures, six generated probes, and the creation base pass
unedited import/byte-preservation verification without partial view. The final
complete suite passes 599 tests (496 library, 98 desktop, five integration).
`git diff --check` is clean.
