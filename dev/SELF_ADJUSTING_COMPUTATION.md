# Self-adjusting evaluation

`sketchpad_engine::sac::Runtime` records input reads and child queries while a
pure resolver runs. After inputs change, a demand validates the previous trace.
Unchanged inputs reuse the result. A changed child that produces the same result
retains its change stamp, so consumers can remain valid. Reevaluation replaces
the old trace, including dependencies belonging to a formerly active branch.

## Engine integration

`document/sac_evaluation.rs` connects ordinary point, number, straight, circle,
polygon, arc, coordinate-frame, curve-sample and display-geometry queries to
this runtime.
Object getters and temporary-value getters record reads automatically. Helpers
shared with the parser can accept `ObjectRead` to retain individual reads;
borrowing the whole object table conservatively records the entire table.
Locus, curve-point, sampled-mapping and colorized validation helpers use
`ObjectRead` instead of coercing the store to a slice.
Coordinate-system lookups use a candidate index. Its own revision tracks changes
to membership; candidate-object reads track axes, pages and deletion. Moving an
unrelated point therefore does not invalidate every coordinate-based curve.

`EvaluationCache` produces immutable views. Object mutation guards supply
per-object revision identities. A separate structural identity covers transform
reference validation. Changes to document-wide settings conservatively create
a fresh view. Sampling and mapping scopes keep their seed values and frozen
external inputs separate from their computation records.

Mapped-element legality is a pure property of the stored construction table.
A table shares cached boolean results across sampling scopes. Every object edit
replaces this cache; older snapshots retain their own cache. Queries observe a
`MappedElementValidity(id)` input, so changes from valid to invalid (or back)
propagate without adding every validation read to every sample trace. Temporary
seed coordinates are not inputs to this construction-only check.

Geometric sampling sometimes substitutes a point's constraint parameters. These
substitutions use a sparse copy-on-write map over the shared object table, with
an individual revision token for each replaced object. A sample does not clone
the full object table or the full vector of revision tokens. Object lookup and
iteration include the replacements, so point-value measurements still read the
sampled circle, arc or polygon parameters. Nested scopes share the sparse map
until a replacement changes; older snapshots retain their own values.

Callers that require a contiguous slice lazily materialize the merged table.
Ordinary object edits also materialize it and preserve the existing per-object
revision tokens. Both sparse replacements and regular edits use the same
dependency reconnection and validation invalidation rules.

The construction dependency program remains useful for validating references,
selecting frozen boundaries and clearing temporary substitutions. It no longer
eagerly evaluates every affected object. Query demands drive value computation.

## Scope identity and sharing

Mappings, iteration steps and branches, locus samples, point-curve parameters,
sampled mappings and image-grid samples have explicit names under their caller's
scope. Coordinates are inputs rather than part of an iteration's identity.
Revisiting a name retrieves its trace and revalidates it against current inputs.
Cross-scope calls expose their underlying input reads to the caller. Cached
transitive input summaries avoid repeatedly walking unchanged child traces.
The engine visits these immutable summaries without copying their input/stamp
vectors. Visitor callbacks run outside the runtime lock. Already-unique engine
recordings are submitted once without a second hash-set deduplication pass.
A summary is rejected if a child has changed since its parent last ran. If retained
children are missing, validation falls back conservatively to a scope revision.

Memo tables use `Arc` sharing and copy on write, with immutable `Arc` payloads.
Cloning a table is cheap; its first write detaches the table while sharing the
individual values and traces. `rustc-hash` handles internal query identities.
Named-scope publication retains records demanded during the current revision
and their transitive child records, even when the parent was a cache hit. It
excludes unrelated inherited lookup entries. Undefined geometry probes are
retained too, so their dependency summaries remain available across scopes.

Each query table has a 4,096-record FIFO bound. The shared named-scope LRU retains
at most 8,192 entries and 200,000 memo records. These are record-count limits,
not byte limits. Eviction can cause recomputation but cannot revive an obsolete
change stamp. Keeping older evaluated views intentionally retains their data.

Recording hash sets are recycled per thread. The pool keeps at most 32 empty
sets, each with capacity at most 256 entries; larger sets are released.
Unwinding clears recordings before reuse. The pool retains allocations, not
query dependencies or results.

## Trace granularity

Iteration and mapping scopes coalesce nested point and arithmetic queries into
the enclosing demanded trace. A bounded temporary memo table shares intermediate
values during that execution; it does not retain a separate persistent node for
every intermediate. Branch input reads still replace the old trace on execution.

Display geometry and curve tessellation remain independent memoized queries.
Sampling and curve-intersection scopes retain fine-grained traces, allowing them
to reuse invariant intermediates between samples. Calls into a different frozen
or caller scope also retain their own records and export their input summaries.
Coalescing across those boundaries would lose useful reuse or misattribute reads.

`Statistics::executed` counts executed persistent trace roots; it does not count
arithmetic queries inlined into those roots. Coalescing trades some fine-grained
equality cutoffs for less recording and allocation. It is not native-code inlining.

## Resolver contract

- Queries must be deterministic and free of externally visible effects. Inputs
  stay immutable throughout a query. Call `advance` after changing inputs, or
  provide a changing `revision_stamp` from the input mutation boundary.
- Every semantic input must be read through the resolver's tracking boundary.
  This runtime does not instrument arbitrary Rust memory accesses.
- An overridden `evaluate_inline` must preserve the results and semantic input
  reads of `evaluate`. `can_inline` can retain expensive child query boundaries.
- Result equality must mean that downstream computations cannot distinguish the
  values. The engine compares floating-point bits, including signed zeros.
- Runtime clones have independent revisions and traces. Parallel readers may
  duplicate a pure calculation, but publication preserves equal-result stamps.
  Resolver execution runs outside the state lock; unwinding discards partial
  traces. Cycles return an error, translated by the engine to an invalidatable
  undefined value.

This is an interpreted SAC runtime, with no native-code JIT or derivative
generation. Fully changing graphs still require computation and pay tracking
overhead. The historical timings in `3D_PERFORMANCE.md` predate this runtime;
they must not be used as measurements of SAC or evidence of 60 FPS.
Current measurements and verification are in [SAC performance](SAC_PERFORMANCE.md).
