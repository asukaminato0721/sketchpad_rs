# Tool-prototype mapped elements and nested mappings

Snapshot: 2026-09-30.

## Observed structures

The remaining code-101 roots are single-application point mappings inside
embedded tools. Their seed and replacement may both be declared
`ToolPointOnPath` placeholders. Their carried dependency graph may also read a
declared numeric tool input that is invariant across the substitution. Several
of the same prototypes contain code-89 mappings nested inside the code-101
capture, so the placeholder context must follow the captured mapping rather
than stop at the outer descriptor.

These layouts are valid only while decoding a tool prototype. Ordinary drawing
code-101 and code-89 records retain the existing strict validator.

## Admission and serialization

The prototype-only validator admits a path-point placeholder in the mapped seed
or image role and treats other declared tool inputs outside the seed, branch,
and carried sets as frozen predecessors. It still checks object order, page,
dependency closure, mapped output type, and every nested captured structure.
Invocation materializes the path points and binds the numeric inputs before the
ordinary mapped evaluator runs.

Code-101 objects also retain their native point name and font records. Native
stores these records after the reference list, not immediately after the object
header; the writer now emits that order. The observed normal Times New Roman
size-24 point-value caption advances the following numeric row by 19 pixels.

## Verification

The corpus scan restores 44 prototype slots:

- all seven code-101 roots;
- six code-89 descriptors, nine code-77 bindings, and two code-88 aliases;
- four code-37, two code-69, two code-24, two code-35, six code-30, two code-33,
  and two code-16 dependents.

Decoded prototype coverage rises **5,207→5,251/5,356** and unsupported slots
fall **149→105**. Fully decoded documents rise **1,581→1,585/1,599**,
complete isolated tool views rise **67→71/103**, and partial source views fall
**769→765**. Every drawing remains complete, and all 1,599 originals remain
byte-exact.

A representative rolling-ellipse tool invocation creates 35 outputs and 41
total objects, including two mapped elements and one mapped iteration. The
6,284-byte result reopens non-partially and re-encodes byte-for-byte. Sketchpad
5 under Wine and Xvfb renders the coordinate frame, parameter, labeled points,
and mapped curve. Ctrl+S preserves SHA-256
`09394130b5c28571480ac0e33663f1fb7fca2bfc759c186931bbebf149ecf562`.

No standalone test case was added. The existing workspace suite, strict Clippy,
corpus scan, invocation round trip, and Native open/save control are the
regression gates.
