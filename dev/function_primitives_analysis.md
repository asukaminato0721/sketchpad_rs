# Primitive function roots

Checkpoint 39 targets common predecessor functions rather than isolated drawing
variants. Primitive definitions now share the calculator's 13 native built-ins:
`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `abs`, `sqrt`, `ln`, `log`, `sgn`,
`round`, `trunc`. They also accept native π/e constants. No new shape or transform
enum cases are introduced.

The original calendar sample (`2024年年历表.gsp`, paired `.htm`) defines object 3
as `1 x x @trnc - @sgn_ -`; object 5 is a separate parameter-dependent definition
and remains unsupported. The retained calendar regression checks the first root
at integer and positive/negative fractional inputs without pretending the second
is implemented. Original `music.gsp` and its paired `.htm` identify object 7 as
`5*sin(25*x)`. Its binary caption position is (-1,-18), which the old unsigned
4096 guard incorrectly rejected. Caption setters and readers already use signed
16-bit coordinates; that guard is removed while expression/header/mode guards
and the 64-token bound remain in place.

## Native creation evidence

[The retained setup script](function_primitives_wine_setup.sh) selects each
built-in from the native calculator menu. Typing function names directly can
invoke keyboard mnemonics, so menu selection is essential. The resulting
[26-object native-created document](../tests/fixtures/function_primitives_controls_native_created.gsp)
contains trunc/round/sgn/asin/acos/atan definitions 15/17/19/21/23/25 and their
plots. It imports without partial view and preserves all 5320 bytes.
[The final capture](../tests/fixtures/function_primitives_controls_native_created.png)
shows all six definitions and curves.

## Actual authoring and native save evidence

The regression reconstructs the actual API writer from
`source_exponents_base_port.gsp`, creates eight definitions, eight live function
calls and eight plots, then checks exact fixture equality. π/e are API-authored
and native-verified here; they are not claimed as native-menu creation controls.

- [Original output](../tests/fixtures/function_primitives_writer_original_native_verified.gsp):
  38 objects, 7190 bytes, parameter -0.5; trunc=0, round=0, sgn=-1,
  asin=-π/6, acos=2π/3, atan≈-0.463648, π≈3.141593, e≈2.718282.
- [Edited output](../tests/fixtures/function_primitives_writer_edited_native_verified.gsp):
  38 objects, 7192 bytes; parameter +0.5, trunc source changed to trunc(x+1),
  document angle mode switched to degrees. The calls show 1, 1, 1, 30, 60,
  26.57, 3.14 and 2.72.

Both outputs open and save byte-identically in Sketchpad 5 (`wine_roundtrip.sh`,
exit 0 and independent `cmp`).
[Original capture](../tests/fixtures/function_primitives_writer_original_native_verified.png)
and [edited capture](../tests/fixtures/function_primitives_writer_edited_native_verified.png)
show the actual calls and changed trunc plot. The original inverse-trig results
are radians; the edited capture displays degree results. Function-call result
kinds retain the existing Scalar model.

Hidden constant captions exercise (-10,-30) and (4100,40) without cropping the
visible verification values. They retain their exact bytes through native save
and reopen as editable definitions. Undo/redo of the three edits reproduces both
writer files exactly.

Native rounding uses floor(x+0.5), so -0.5→0 and -1.5→-1. Truncation is toward
zero and sgn(0)=0. Inverse trig returns radians or degrees according to document
mode; asin/acos outside [-1,1] remain undefined. The evaluator shares the
calculator's native π constant, and symbolic constant derivatives are zero.

## Coverage and limits

The final scan `/tmp/sketchpad_corpus_checkpoint39_final.log` preserves all 1599
source files exactly and restores 271 drawing slots, including 119 calculations.
Unsupported slots fall from 64477 to 64206: 63017 drawing and 1189 prototype
slots. Structural counts and prototype coverage are unchanged.

Referenced/parameter-dependent function definitions, other presentation modes
and expressions beyond the 64-token limit still need separate support. Symbolic
derivatives of inverse trig and sgn/round/trunc remain unsupported. Step curves
use existing sample rendering, whose appearance can differ from the original;
exact visual matching is outside the user's requested scope. Copying and custom
tools remain deferred.

Scoped gates pass 631 library, 109 editor, 34 numeric-import and four coverage tests.
