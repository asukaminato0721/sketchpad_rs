# Incremental geometry evaluation

The engine uses [`incremental` 0.2.8](https://docs.rs/incremental/0.2.8/incremental/),
the Rust port of Jane Street Incremental. The former public
`sketchpad_engine::sac::Runtime` and its recursive validation scheduler have been
removed. This is an intentional API break.

`document/incremental_graph.rs` adapts geometry queries to the library's input
variables and expert nodes. Incremental schedules dependencies before consumers,
limits work to observed graphs, and stops propagation when results are equal.
`document/sac_evaluation.rs` supplies geometry operations, exact result equality,
and tracking at object and temporary-value reads. Ordinary construction code
continues to use these tracked APIs; arbitrary Rust memory reads are not tracked.

## Execution and dynamic dependencies

The first demand discovers a query's actual input and child-query reads. The
adapter connects those reads to input variables and expert nodes. Later changes
update input variables before stabilisation. Expert callbacks execute geometry
operations, and the library propagates their results.

A previously unseen dynamic dependency is evaluated directly to discover its
trace. Repeated reads within that discovery share settled values. The adapter
then reconnects the affected nodes in a topology callback that precedes the
geometry callbacks. Incremental requires dependency removal during
stabilisation, so connections are not removed arbitrarily between passes.
Replaced edges are detached before new edges are attached, including simultaneous
edge reversals. A further pass publishes directly discovered results into the
library's nodes. Cyclic geometry reads produce an undefined result that observes
structural and scope inputs, allowing a later edit to recover.

Snapshot imports compare direct input stamps and child-result identities in an
expert callback. They do not recursively schedule children: the library orders
those callbacks. An immutable cached result also has a transitive input footprint.
When all those inputs match, the adapter can return that result without switching
a live scheduler away from another sample. When inputs differ, Incremental handles
recomputation and value cutoffs.

## Sampling and snapshots

An evaluated document contains immutable, shareable result and dependency records;
it does not contain Incremental's `Rc` handles. Each thread keeps an LRU of live
execution graphs. Scoped TLS lends a document to callbacks only for the duration
of a query. Callbacks capture neither documents nor observers. A panic discards
the affected live graph; the previously published snapshot remains usable.
Parallel readers have independent live graphs and can duplicate pure work.
Publication merges the demanded root and its complete dependency closure into
the shared snapshot. It does not replace unrelated records with a reader's older
copies. This preserves concurrent readers' updates and complete input footprints.

Mappings, iteration steps, locus samples, curve parameters and image-grid samples
retain named result snapshots. A name contains the call site and logical sample,
not its current coordinates. Sibling samples with the same construction share an
execution family: sample indices, fractions, iteration steps and image-grid
positions are removed from the execution-family name, while retained result names
remain distinct.
Consequently, moving between samples updates inputs on one live graph instead of
rebuilding a scheduler for every point. Returning to an unchanged named sample can
reuse its immutable result immediately. Old views validate against their own
inputs even when a live execution family has most recently served a newer view.

Iteration and mapping scopes coalesce small arithmetic and point operations into
one traced computation, with a temporary memo table for intermediate values.
When a coalesced kernel has no retained child queries, it executes as a pure
function of its recorded input tuple, without constructing library nodes for its
internal arithmetic. Its containing geometry node remains scheduled by
Incremental. Expensive geometry queries remain separate nodes. Locus and
intersection sampling retain finer dependencies so invariant intermediates can
survive a sample change.
Cross-scope calls expose their underlying reads to the caller. A missing or
inconsistent footprint conservatively observes the object table and scope.

Object-table replacements remain sparse copy-on-write overlays. Each replacement
has its own revision identity; point-value measurements see substituted constraint
parameters as well as positions. This sharing is independent of the scheduler.
Normal edits and callers requiring contiguous slices materialize the table.

## Retention

Result tables share their backing map and immutable records through `Arc`. Named
scope publication retains the demanded records and their transitive dependencies.
A table exceeding its 4,096-record target drops unrelated records while preserving
the complete last-demanded subgraph. A single larger subgraph may exceed that
soft target. The named-result LRU has limits of 8,192 entries and 200,000 retained
records; the execution-family name cache has 8,192 entries. Each thread retains
at most 64 live graphs. These are count limits, not byte limits. Retaining old
views intentionally keeps their snapshots alive.

New named scopes do not inherit unrelated caller query tables; the live family
provides reusable intermediates. The library's height queues start small and grow
as graphs are discovered or rewired. A caller is returned to the live LRU after nested sampling, so a long
sample sequence does not evict the graph needed by the next drawing query.

## Correctness boundaries

- Inputs remain immutable during one query. Temporary-input edits advance the
  context revision; object edits supply a separate object-table identity.
- Every semantic input must cross the tracked access boundary.
- Result equality means downstream computations cannot distinguish the values.
  Floating-point comparison preserves signed zero and other bit distinctions.
- A result snapshot is independent of which thread or execution family last
  evaluated the construction. Mutation, rewiring, undefined values and old-view
  reads must agree with a fresh recursive evaluation.
- Construction dependency programs still validate references, select frozen
  boundaries and clear temporary substitutions. They do not eagerly evaluate
  every affected object.

There is no JIT or automatic derivative generation. Incremental scheduling alone
does not make fully changing numerical workloads cheap or establish 60 FPS.
Current measurements, including regressions, are in
[Incremental performance](INCREMENTAL_PERFORMANCE.md).
Historical custom-runtime measurements are in [SAC performance](SAC_PERFORMANCE.md).
