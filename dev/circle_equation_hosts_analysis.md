# Cartesian equations of constructed and transformed circles

Native Measure → Equation accepts a circle constructed with a segment radius and
a circle dilated by a marked scalar numeric ratio. Both operations create code 51
with parents `[circle, coordinate system]`, the existing 0x08fc equation text
payload, and a text-style reference. The native files are
`segment_radius_circle_equation_native.gsp` and `ratio_circle_equation_native.gsp`.

Equation creation also adds 0x07d5 circle label metadata and 0x07d8 style references
to the circle groups. The ratio case includes labels on both the original and
transformed circles. These auxiliary records now receive the same bounded
validation as existing polygon labels: parse their label payload, validate the
text-style index, and preserve their source bytes. Numeric-angle and numeric-ratio
shape codes 29/33 are included in this validation. Native fixtures now import
without partial view and save unchanged byte for byte. General curve-label editing
is still outside this equation-host extension.

The desktop command, importer, and save validation accept supported circle kinds,
including circles with a segment radius, reflected/translated/rotated circles,
and fixed/marked/numeric-ratio dilations. Equation evaluation resolves the live
center and radius through the shared geometry and expresses them in the selected
square Cartesian frame. A negative dilation factor uses its absolute radius.
Non-circle geometry and ordinary segments remain ineligible for a circle equation.

Tests cover native-file preservation, live radius edits, signed ratio edits,
transformed centers/radii, coordinate origin edits, copy/remapping, dependent
removal, and desktop selection. Non-square frames and other equation forms retain
their existing restrictions.

`tests/replay_circle_equation_hosts.txt` generates the two creation bases and four
original/edited cross-open probes. Both centers are at (−1.031875,−1.4552083333333)
in the chosen frame. The marked-ratio circle radius changes from 0.79375 to
1.190625 after n=0.5 becomes −0.75. The circle-by-segment radius changes from
1.5875 to 2.38125 after its defining segment grows from 60 to 90 pixels.
The ratio probe hides its source parameter to keep the native equation readable.

To reproduce native creation, use the matching `*_circle_equation_base_port.gsp`
with `dev/wine_roundtrip.sh`, `SKETCHPAD_WINE_SETUP_SCRIPT` set to
`dev/circle_equation_hosts_wine_setup.sh`, and a capture prefix containing `ratio`
or `segment_radius`. The setup script selects the circle and invokes Equation.

All four generated files opened and saved byte-identically in native Sketchpad 5.
Native captions show the expected radius changes, 0.79 → 1.19 and 1.59 → 2.38,
and the edited SVG equation agrees. Both native creation fixtures and all four
port probes verify without partial view. The complete suite passes 585 tests
(483 library, 97 desktop, and five integration).
