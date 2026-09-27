# Function calls in calculations

Checkpoint 38 adds native referenced function calls to the shared calculator
AST and source list. Function and numeric operands occupy the same reference
namespace, with `0x7000 + index` for a function call and `0x6000 + index` for a
numeric operand. Calls have an implicit opener and one unmatched `0x000c`
closer, as confirmed by Native Calculator-created files. Constant calls have a
function reference even without a numeric operand; they are not source-free
calculations. No shape/transform combinations or custom-tool/copying authoring
are introduced.

The checkpoint 37 token audit found 395 unsupported expressions and 180 invalid
source-token expressions. These primarily used referenced function calls.
The shared parser now accepts them, including nested calls, repeated calls and
calls at other reference indexes. Unsupported function definitions remain
unsupported parents rather than being treated as numeric constants.

## Native controls

`function_application_base_port.gsp` contains the existing quantity base plus
f(x)=x²+1, g(x)=sqrt(x), and f′(x)=2x. The three literal fixtures
`function_application_{literal,length_literal,degree_literal}_native_created.gsp`
were created using Native's Functions menu; they show f(2), f(2cm) and f(2°)
all producing unitless 5 in the base's degree mode. The degree literal control
does not establish degree-to-radian conversion in radian mode on its own.

`function_application_controls_native_created.gsp` establishes f(n)=5,
g(n)=1.414…, f′(n)=4, and f(radius)=3.52015625 at n=2 and radius=1.5875cm.
Retained screenshots independently show all four Native results. The shared
call evaluator uses Length magnitudes in centimeters and returns Scalar.
Changing n to −2 makes g(n) undefined, while changing f to x³+1 refreshes both
its calls and derivative calls. Import/edit/undo/redo regressions retain the
original function references and exact unedited bytes.

The original `函数图像五类基本变换.gsp` has f(x)=x²−4x+3, as stated in its
paired HTM. Its object 94 now calls that definition at x=−1.25 and produces
9.5625. `function_application_transform_corpus.gsp` retains the original file.

Reproduction scripts are `function_applications_wine_setup.sh` and the three
`function_application_*literal_wine_setup.sh` scripts. Run them via the
serialized `wine_roundtrip.sh` helper on a temporary copy of the base. Function
entries are in the Calculator Functions menu, not the Values menu.

## Evaluation and interaction

`CalculationSourceKind` records whether each reference is numeric or callable.
The parser rejects out-of-range indexes, missing closers and a source used both
ways. The decoder, authoring API and save validator require the referenced
object to have the matching type. Function slots use placeholders during value
collection and are evaluated by callback; they never read a function object as
a numeric value. All actual dependencies stay in the DAG. `add_multi_calculation`
accepts f1(…), f2(…), … at the corresponding source indexes, alongside v1, v2,
… numeric operands. A single function source supports constant calls. The
editor exposes this syntax for selected functions and derivative functions.

Native writer verification exposed an angle distinction: in radian mode,
typed Degree arguments are converted to radians before calling the function;
in degree mode they keep their degree magnitude. A scalar argument remains its
raw magnitude. The callback now receives the argument unit from the AST.
This makes a sine function at a measured 90° return 1 in both modes, while
sin-function(90) in radian mode remains sin(90 radians). Nested functions return
Scalar, so their output is not converted a second time. Invalid calls suppress
their dependent calculations and numeric labels.

The point/parameter iteration evaluator uses the same callback and reference
roles for labels. Mapped numeric expressions can keep supported functions as
external dependencies; each mapped parameter advances without replacing the
function slot. Existing generic transform evaluation is reused.

## Actual writer proof

The actual API writer authors ten calls, a sine definition, a translated point,
an attached numeric label and a three-step point/parameter iteration. The two
28-object outputs use existing function definitions plus both nesting orders,
combined functions, derivative calls, Length and Degree sources and a typed
degree literal. They were opened and saved in Native by the unchanged serialized
helper, with exit 0 and independent `cmp` passing for 5832 and 5838 bytes. The
retained `function_applications_writer_{original,edited}_native_verified.gsp`
and PNG files are actual API output, not Native-created calculator substitutes.

The original labels are 26.00, 677.00 and 458330.00 for repeated x²+1 from n=2.
Changing n to −2, f to x³+1, the angle mode to degrees and the radius source
point by 30 pixels yields −342.00, −4.00×10⁷ and −6.40×10²². Native and port
agree that g(n), f(g(n)), g(f(n)) and the g(n)+f′(n) sum become undefined.
The sine call with a 90° source remains 1 in either angle mode; f(2°) changes
from 1.0012184696791468 in radian mode to 9 with the edited cubic in degree mode.
Both captures show these values and the separated iteration labels. The writer
regression reconstructs both exact files, checks source/function/domain edits,
and verifies strict save/reopen plus four undo/redo steps.

The final scan `/tmp/sketchpad_corpus_checkpoint38_final.log` preserves all 1599
original files exactly and recovers 139 drawing objects, including 75
calculations. Unsupported slots fall from 64616 to 64477 (63288 drawing and
1189 prototype); fully decoded document and prototype counts are unchanged.
There are still 5982 unsupported drawing calculations. The role-aware numeric
audit identifies 5968 missing-parent/type cases, nine header cases and five
function-unit cases. Dependency-root counts overlap: code-59 roots affect 1566
calculations, code-101 roots 1140, and referenced code-71 functions 812 across
86 files. Those counts are priority evidence, not projected independent gains.
Referenced function definitions and broader function plot dependencies remain
required follow-up work.

Scoped gates pass 628 library, 109 editor, 34 numeric-import and four coverage
tests, excluding the deferred copying/custom-tool scope. All five focused
function-call regressions pass. Reproduction scripts pass `bash -n`, and the
changed tracked source/documentation files pass `git diff --check`.
