# Dimensional calculator roots and powers

Native Calculator probes were created from circle area, circle radius, and a
90° angle. The saved creation fixtures are `area_square_root_native.gsp`,
`area_fractional_power_native.gsp`, and `dimensional_roots_powers_native.gsp`.
The UI establishes these rules:

| Expression | Native result units |
| --- | --- |
| sqrt(area) | cm |
| area^0.5 | none |
| area^1 | cm² |
| radius^2 | cm² |
| sqrt(radius) | none |
| sqrt(angle) | none |
| angle^2 | °² |
| (area^1)^2 | cm⁴ |
| sqrt((area^1)^2) | cm² |
| sqrt(sqrt((area^1)^2)) | cm |

Integer literal exponents and products/quotients now carry integer length and degree
exponents. Square roots halve the exponents when both divide evenly; otherwise
they return a scalar. Fractional powers return a scalar even when a dimensional
interpretation would be possible. Literal integer exponents retain units; exponent overflow is rejected. Later
native probes corrected the treatment of computed, grouped, negative, and
source-dependent exponents: all produce scalar results, even for an integer
value. See [exponent evidence](source_exponents_analysis.md).

Result subtype 51 is a generic quantity category: native area^1 and radius^2
both use it despite having area units. Native angle^2 uses subtype 50, which is
also used for areas. The importer infers units from the expression and source
units, while retaining these original bytes. Inferred units survive source
remapping, precision changes, clipboard visibility changes, and page copying.
Specific header mismatches outside the observed categories are still rejected.
Native function tokens can have an implicit opening parenthesis, e.g.
[0x2007, 0x6000, 12], alongside the previously supported explicit argument form.
Both are parsed and preserved.

`tests/replay_dimensional_roots.txt` generates five cross-open probes. The area
chain includes the length root, scalar fractional power, area identity, cm⁴
power, and both nested roots. Moving the radius from 60 to 90 pixels changes
area 7.9173 → 17.8139, the roots 2.81377 → 4.22065, and cm⁴ output
62.6837 → 317.336. Native captions show 7.92 → 17.81, 2.81 → 4.22,
and 62.68 → 317.34 with the expected units.

The iteration probes use sqrt(area) as marked translation distance and
area^0.5 as scalar live depth. Changing radius 40 → 60 pixels changes distance
1.88 → 2.81 cm and depth one → two. Numeric display precision does not round
the source value used for iteration. The unit probe includes sqrt(radius),
sqrt(angle), and angle^2. Generated sketches are checked by opening and saving
copies with `dev/wine_roundtrip.sh` and comparing the native saves with originals.

To reproduce native creation, use `dimensional_roots_base_port.gsp` with
`dev/dimensional_roots_wine_setup.sh`. Use a capture prefix containing
`fractional` for area^0.5; otherwise the script creates sqrt(area). For the five
measurement probes, use `dimensional_roots_units_base_port.gsp` and a capture
prefix containing `units`. The script hides each newly created calculation to
keep the original measurement captions selectable. Use the Power button:
typing a caret in the native calculator opens New Parameter.

Tests cover native byte preservation, source edits, higher units, negative root
domains, source remapping, precision, copy/paste, page copying, dependent removal,
and live marked-distance iteration. Existing restrictions on degenerate circle
measurements and copying some native text styles are outside this extension.

All five generated single-page probes displayed the expected native captions
and saved byte-identically in Sketchpad 5. All three native creation fixtures,
five generated probes, and two creation bases verify without partial view.
Page-copy unit tests retain the existing multi-page partial-view guard.
The complete suite passes 595 tests (492 library, 98 desktop, and five
integration). `git diff --check` is clean.
