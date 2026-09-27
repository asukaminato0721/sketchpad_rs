# Many-input calculations

2026-09-27, checkpoint 32. Copying and custom tools remain deferred.

## Dependency evidence

The refreshed drawing dependency audit (`/tmp/broad_measure_roots_checkpoint31.log`)
finds 4749 unsupported calculations in 82 files with a terminal code-48/scalar
root; that category includes overlapping token, unit and input-count failures.
The layout audit (`/tmp/numeric_import_audit_checkpoint31.log`) identifies 46
calculations rejected specifically for reference count. A raw drawing profile
(`/tmp/calculation_source_profile.log`) finds original counts 18–51 above the old
limit and continuous source tokens `0x6000` through `0x6032`. These profiles skip
embedded-tool namespaces. They justify extending the shared calculator rather
than adding shape-dependent calculation variants.

## Change

The existing calculator AST now recognizes `0x6000..=0x60ff`. Input lists and
expression token lists permit up to 256 entries, matching the existing imported
expression bound. Authoring previously used a separate 32-token bound, which
prevented a sum of even 20 inputs. Existing record order, header, numeric cache,
unit inference and declared token-count validation remain intact. Indexes outside
the reference list are rejected; source-free expressions reject indexed tokens
throughout the extended range. `v1`…`v256` follow selection order, and the existing
`a`/`b` aliases retain their meaning.

## Proof

The original `100位e.gsp` object 26 has 20 inputs and ten summed pair products.
The retained `polar_fractional_distance_corpus.gsp` control now imports it as a
calculation, verifies each original input, evaluates the dot product independently
to 2.718281828, and preserves the full original file byte for byte.

An authored 51-parameter sum drives a length calculation (`sum*(1 cm)/100`)
and a marked-distance point translation. Changing input 51 from 1 to 2 changes
the sum 51→52 and the length .51→.52; tests verify live geometry, save/reopen,
undo/redo, index bounds and the 256th-source boundary. The unchanged native
roundtrip helper opened and saved that 13314-byte edited file, exited 0, and an
independent comparison to the untouched input snapshot passed. Its retained
`many_input_calculations_writer_native_verified.gsp` is compared byte for byte
against API reconstruction. This establishes native writer compatibility for
51-input authoring; it does not prove arbitrary 256-input files in the original UI.

The complete corpus scan retains 1599/1599 exact round trips and reduces
unsupported slots from 84900 to 84744, all 156 recovered slots in drawings,
including 80 calculations. Embedded prototype counts do not change.

Scoped validation: 604 library tests, 34 numeric-import tests and 107 editor tests
pass with zero failures. Copy/paste and custom-tool name filters follow the
user-deferred scope. Earlier 32-token rejection assertions were updated to test
the actual 256-token limit; the new native writer regression remains exact.
Whitespace diff checks pass. Logs are `/tmp/sketchpad_many_source_final_lib_retry_gate.log`,
`/tmp/sketchpad_many_source_final_import_gate.log`, and
`/tmp/sketchpad_many_source_editor_gate.log`.
