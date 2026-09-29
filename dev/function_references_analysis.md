# Referenced functions: dependency integration and native evidence

Checkpoint 40 established the contextual expression parser and binary evidence.
Checkpoint 41 integrates referenced definitions into the document dependency
graph, authoring, editing, plotting, calculations and numeric iteration. The
final corpus scan restores 919 drawing objects, including 229 definitions and
439 calculations, plus three prototype objects through the shared reader.

## Scope established by the corpus

[The reproducible audit](function_reference_audit.rs) inspects unsupported drawing
objects through their actual header offsets, excluding embedded tool prototypes.
It finds 700 referenced primitive definitions with code71, class11/subtype57:
259 numeric-only, 54 function-only and 387 mixed. 694 have at most64 words; six
exceed the current expression bound. At checkpoint 39, of the694,245 in128 files had
all role-correct parents supported, while449 depended on unsupported parents,
including other referenced function definitions. These are prerequisite counts,
not predicted independent recoveries.

Numeric values use `0x6000 + reference index`; referenced function application
uses `0x7000 + reference index`, followed by its argument and `0x000c`. Both
roles share one reference-list namespace. Original GSP/paired HTM evidence:

- `../gsp-rs/tests/fixtures/未实现的系统功能/函数.gsp` object10 refs[7,8,9]:
  `A*x²+B*x+C`; tokens6000/6001/6002. The retained
  [quadratic fixture](../tests/fixtures/function_reference_quadratic_corpus.gsp)
  tests coefficients2,-3,5 at x4, producing25 through the live document API.
- `../gsp-rs/tests/Samples/个人专栏/况永胜作品/2024年年历表.gsp` object5
  refs[1]: `trunc((A-1)/x)`, establishing a compact numeric source operand.
- `../gsp-rs/tests/Samples/未分类档/动滑轮2.gsp` object19 refs[10,16,8,9]:
  `A*sqrt(1+tan(x)^2)+tan(x)*fB(x)-tan(x)*C+D`.
  Tokens6000/6002/6003 are numeric;7001 is the function source. Its
  [retained fixture](../tests/fixtures/function_reference_mixed_corpus.gsp)
  tests the exact slot roles and its numeric result against that expression.

Raw header, wrapper, token and paired HTM dumps are in
`/tmp/function_reference_evidence40.log`,
`/tmp/function_reference_evidence40_quadratic.log` and
`/tmp/function_reference_evidence40_mixed.log`; readiness evidence is
`/tmp/function_reference_ready40.log`. The audit records missing same-ID HTM
explicitly; reordered/omitted HTM objects are not treated as matching IDs.

## Implemented expression API

`SimpleFunction::parse_with_sources` accepts v1/v2/... numeric aliases and
f1(...)/f2(...)/... function aliases at their one-based reference positions.
`source_kinds` validates bounds and rejects a slot used in both roles.
`evaluate_with_sources` resolves numeric/function operands through callbacks,
without freezing their values into constants. Unbound evaluation is undefined.
Primitive `parse` and `decode_simple_expression` retain their zero-source bounds.

`decode_expression_with_sources` supports native compact unary syntax, including
mixed built-in and referenced calls. `encode_expression_record_with_sources`
retains the original92-byte wrapper and writes changed tokens/counts. The old
zero-source writer continues to reject reference-bearing records.

## Native-created and actual record-writer controls

[The retained setup](function_reference_wine_setup.sh) uses native menu selections
on `source_exponents_base_port.gsp` to create f(x)=n*x+1 and g(x)=f(x+1).
The [17-object native-created file](../tests/fixtures/function_reference_controls_native_created.gsp)
is3928 bytes; n is object6, f object15, its plot16 and g object17.
[The capture](../tests/fixtures/function_reference_controls_native_created.png)
shows both formulas and f's live plot. Their actual code71 headers and source
lists are [6] and [15], respectively. Input fields can briefly show an incomplete
preview even when the native-created result is valid; the created capture and
saved payload establish the result.

[The record-writer generator](function_reference_generate.rs) calls the actual
contextual expression API to change f to n*x²-1 and g to f(x-2)+3. It preserves
all reference/header records and replaces only the two expression records. The
[resulting3936-byte file](../tests/fixtures/function_reference_records_writer_native_verified.gsp)
opens and saves byte-identically in Sketchpad5 (exit0, independent `cmp`).
[The verified capture](../tests/fixtures/function_reference_records_writer_native_verified.png)
shows the quadratic plot and both edited formulas. Regression tests compare
both authored records against the actual native-saved payloads and reject
unbound writes. This proves the expression-record API, not a completed
Document authoring path; the following controls establish that separately.

## Document integration and authoring

`Kind::ReferencedFunction` stores an ordered source-ID list and the shared
expression AST. Source indexes refer to that list; they are not baked numeric
values. Decoding validates actual same-page backward dependencies, numeric and
function roles, and supported record layouts. Labels and signed anchors retain
the native function name and position. Calculations, Cartesian and parametric
plots evaluate through the document context. Nested reciprocal functions retain
plot gaps across inherited poles.

The editor captures numeric/function selection order when opening the dialog,
shows aliases and source captions, and supports creating or editing definitions
and plots. Invalid sources or conflicting roles fail before changing the
object list or undo history. Creating a plot and its coordinate frame is one
undo action. Existing expression edits patch the original wrapper and reference
records. New definition groups use the destination drawing's font namespace;
copying the native control's font index3 into a file with only indexes0/1 caused
a partial view, which strict round-trip validation caught. New groups use index0.

In radian mode, Degree predecessors convert to radians before evaluation.
Mapped parameter iterations evaluate function coefficients against the current
step's parameter value, without mutating the source document. Both behaviors
were established by the native controls below.

## Actual Document writer controls

[The reproducible generator](function_references_generate.rs) constructs a
38-object document through the actual public APIs. It covers numeric-only,
function-only and mixed references, nested calculations, plots, angle sources,
inherited reciprocal poles and mapped numeric iteration. Both outputs passed
strict `Document::verify_roundtrip`, opened in Sketchpad5, displayed the expected
values and sequences, and saved byte-identically (exit0 and independent `cmp`).

- [Original output](../tests/fixtures/function_references_writer_original_native_verified.gsp)
  and [capture](../tests/fixtures/function_references_writer_original_native_verified.png):
  7016 bytes, n=2 and radians. Nested calculation values include33.5875 and
  17.5875; the angle source is pi/2 and its sine is1. The mapped iteration labels
  are26,677,458330, reflecting the changing coefficient inside c(x)=n*x+1.
- [Edited output](../tests/fixtures/function_references_writer_edited_native_verified.gsp)
  and [capture](../tests/fixtures/function_references_writer_edited_native_verified.png):
  7018 bytes, n=-1, a cubic source expression and degrees. Nested values become
  9.5875 and -4.175; the angle source is90 and its sine remains1. The iteration
  labels are5,26,677. Regression tests reconstruct both files exactly and check
  three-step undo/redo, captions, dependencies and inherited pole gaps.

The native save logs are
`/tmp/function_references_writer41_original_final_native.log` and
`/tmp/function_references_writer41_edited_final_native.log`. These final controls
use the valid destination font index, rather than the earlier rejected candidate.

## Coverage, checks and remaining limits

The final scan contains1599 documents, with1599 exact original round trips and
zero failed imports. Unsupported slots fall from64206 at checkpoint39 to63284:
62098 drawing slots and1186 prototype slots. The drawing gain consists of229
code71 definitions,439 code48 calculations and251 other descendants. Shared
prototype recovery does not add custom-tool authoring or invocation support.

The scoped gates pass642 library,112 editor,34 numeric-import and4 coverage tests
(792 total), excluding the deferred copying and custom-tool scopes.

Function-call results currently use Scalar units. Native Sketchpad can retain
an angular result unit (for example, a function returning an angle displays90°
in degree mode); full result-unit propagation remains pending even though these
numeric values match. Native cached derivatives with referenced dependencies
are supported, while creating a symbolic derivative through a referenced
function call remains pending.
Wrapper bytes46..48 contain observed high words0/1/2 whose meanings are unproved;
the strict wrapper check and64-word expression bound remain. Further wrappers,
longer expressions and unsupported source parents are still coverage gaps.
Copying and custom tools continue to be deferred; pixel-identical display is
outside the requested scope.
