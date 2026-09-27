# Source-free quantity calculations

2026-09-27, checkpoint 31. Copying and custom tools remain deferred by user request.

## Implementation

`CalculationGroup` reuses the existing calculator tokenizer, AST, dimensional
algebra and evaluation. It accepts native numeric headers 48–51. Headers 48/49/50
must match the inferred Length/Degrees/Area result; header 51 retains its existing
legacy generic semantics. Existing field, record ordering, cache and presentation
validation remains in place. Expression editing requires the same numeric unit
and preserves presentation records. Invalid edits leave the document unchanged.

Centimeter literals bind to their magnitude: `9 cm^2` squares `9 cm`.
`sqrt(3 cm*3 cm)` yields 3 cm. Radian literals convert into the shared degree-based
angle evaluator, so `sin(pi rad/6)` yields a scalar 0.5. Negative square roots
remain undefined while retaining the inferred unit. Length results drive the
existing matrix-sequence transformation path and update on edit, undo and redo.

## Evidence

Original `(inRm)重叠面积8例.gsp` object 3 stores `3 cm*sqrt(3)` in a Length
header; `课程4笔记.gsp` global object 101 (page 2/local 31) stores `atan(6)` in
an angle header without a radian suffix. Both are retained as corpus fixtures and
verified for numeric value, unit and exact file preservation. The read-only audit
found 6 Length and 12 angle roots previously rejected, across 12 files; dependency
counts overlap. No original source-free Area or trailing-decimal failing control
was found, so those capabilities are verified separately through authoring.

`source_free_quantity_writer_native_verified.gsp` is built through public
Document APIs: `.3 cm`, `2 cm*3 cm`, `sin(pi rad/6)`, `abs(-pi rad/6)`, `1.+2.`,
a free point and its marked-length translation. The unchanged
`dev/wine_roundtrip.sh` opened and saved the 2856-byte file, exited 0, and an
independent comparison against the untouched snapshot matched exactly. A
reconstruction regression checks every byte against that saved golden.

Focused tests also verify expression editing, save/reopen, undo/redo, live
transform updates, domain failure and atomic incompatible-unit rejection.
The corpus scanner reports 138 fewer unsupported drawing objects, all 1599
original files preserved exactly. Copy/paste and custom-tool tests are excluded
from the scoped gates to respect the user's deferred scope.

Validation logs: scoped library 601 passed; focused quantity/golden gate 4 passed;
editor 107 passed; numeric import 33 passed, all with zero failures. The library
count precedes the additional writer golden test, which passes in the focused gate.
Legacy rejection assertions were updated to accept the newly supported operations;
invalid numeric headers, unknown display mode 2, unsupported literal forms and
incompatible units remain covered. Whitespace diff checks pass.
