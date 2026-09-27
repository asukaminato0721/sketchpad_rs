# Coordinate measurement presentation tails

2026-09-27, checkpoint35. Copying and custom-tool authoring remain deferred.

## Dependency evidence

The drawing-only dependency audit linked against checkpoint34 finds1850
unsupported calculations with terminal65/66 blockers;1301 have exclusively those
families. These overlapping root counts are opportunities, not recovery totals.
Of the unsupported coordinate records,148 code65 and116 code66 have all direct
parents supported; all264 fail the format check. Audit output:
`/tmp/broad_measure_roots_checkpoint34.log`; raw examples:
`/tmp/broad_measure_root_examples_checkpoint34.txt`.

Original `y=Asin(wx+v).gsp` object70 has parents45/29, a supported transformed
point and coordinate frame. Its32-byte0903 payload ends with`02003e00`: precision2
followed by a nonzero presentation word. Object83's ordinate has the same tail.
The paired HTML declares Abscissa(22,6) and Ordinate(10,6); object51's exported
calculation is the difference of Abscissa(22,6) and Abscissa(7,6), corresponding to
native object74's sources70/71. Native/GSP IDs differ from exported IDs.

The ordinary measurement reader already preserves any final presentation word.
The coordinate reader had an extra all-zero requirement. Removing that
requirement uses the existing coordinate math and retains prefix, record order,
reference count, numeric display mode, precision and dependency checks. No math
is inferred from the final word, and unchanged source bytes are retained.

## Native controls and actual API writers

Two controls use the existing native nine-object frame and scalar coordinate
writer, replacing only the final0903 word with003e. Native displays x=-1.30 and
y=0.13 respectively. Both controls open/save exactly at2682 bytes; helper exits
and independent comparisons pass. Screenshots were inspected.

The actual API writer imports each control, adds `2*v+1`, moves point1 by
(+37.79527559055118,-75.59055118110236), and sets the coordinate precision to3.
The resulting x=-0.2964583333333337, calculation0.4070833333333326 and
y=2.132291666666667, calculation5.264583333333334 match the Native displayed
-0.296/0.41 and2.132/5.26. Both2862-byte API outputs open/save exactly, with the
nonzero tail retained. Four fixtures and four inspected captures are retained.

Regression covers both controls and exact API writer goldens, live calculation
inputs, precision changes, undo/redo and strict serialization verification.
The original sine corpus regression checks x/y decoding, the dependent
difference and live geometric dependencies, exact source bytes and undo.
That larger source still contains unsupported functions/actions and remains a
partial view; it does not claim strict verification of edits to its whole graph.

## Coverage

The completed checkpoint35 scan recovers2672 drawing slots, including1232
calculations. Prototype counts are unchanged. Total unsupported slots fall from
73059 to70387 (69198 drawing plus1189 prototype). Fully decoded documents remain
89 and all1599 original files preserve exact bytes; no file fails to load.


The original undo regression also exposed a shared serialization issue:
unsupported objects may retain numeric-looking anchors/precision from import.
The display patcher previously treated those as editable measurement metadata,
although `has_measurement_precision` excludes unsupported kinds. It now leaves
those original records untouched. This preserves partial-document source bytes
after a move and undo without assigning new semantics to unknown objects.


Final scoped regression gates pass 615 library tests, 109 editor tests,
34 numeric-import tests and 4 coverage tests, excluding the deferred
copying/custom-tool scope. Existing format/precision rejection tests also pass.
