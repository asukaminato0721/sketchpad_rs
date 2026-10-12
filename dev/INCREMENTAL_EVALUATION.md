# Construction dependency programs

The construction objects are the computation graph. `Object` implements the
IR's `DependencyNode` contract through `construction_dependencies()`. A new
operation declares its input IDs in `Kind::dependencies()` and implements its
geometry or numeric evaluator. It then participates in graph analysis without
an operation-specific invalidation or scheduling implementation.

This static program records construction connections, not derivatives. It
remains useful for reference validation and frozen sampling boundaries. Runtime
value reuse now uses [self-adjusting computation](SELF_ADJUSTING_COMPUTATION.md):
tracked getters record the inputs actually read, including the active branch.
Neither layer instruments arbitrary Rust memory accesses.

## Shared pipeline

1. `ObjectStore::push` registers a construction's connections in `DependencyGraph`.
   A scoped `ObjectStore::edit` records its final connections and changed ID when
   the guard drops. Loading a document registers all imported objects once.
2. `DependencyGraph::program` validates and orders the recorded connections on
   demand and shares the resulting immutable program. `slice` follows ancestors
   of selected outputs for a local sampling program. Forward references can be
   registered before their inputs arrive; validation waits until evaluation.
3. `program.update(changed_inputs)` produces an immutable `UpdatePlan` containing
   the changed inputs, affected nodes in topological order, and unchanged input
   boundaries. Overlapping changes update each descendant once.
4. `Document::execute_evaluation_plan` invalidates affected values, applies
   temporary inputs when present. The SAC runtime materializes values on demand.
5. `EvaluationCache` uses this executor for document edits. `SamplingPlan` uses
   it for point, numeric, and path-parameter substitutions in isolated scopes.
   Initial read-only evaluation creates a lazy SAC view.

For `C = midpoint(A, B)` and `d = distance(C, B)`, changing `A` creates an update
ordered as `A, C, d`, with `B` as a preserved boundary. An unrelated construction
is absent from that update. Repeated changes to `A` reuse the same plan.

## Editing API

```rust
let c = doc.add(Kind::Midpoint(a, b)); // records input connections now
doc.move_point(a, next_position);     // records a change; retains graph identity
doc.edit_object(c, |object| { // positive object ID
    object.kind = Kind::Midpoint(a, another_point);
    object.midpoint_host = None;
})?; // records connections, one undo step, and the document's dirty state
```

`Document::objects()` returns a shared view. The object collection and undo/redo
stacks are private. `edit_object` rejects changes to object identity before it
changes the document. A no-op edit does not change history or the dirty state.
`Document::transaction` applies a compound operation to an isolated candidate.
An error or panic discards that candidate. Successful nested operations produce
one undo step. Use `undo_len()` and `redo_len()` to inspect history availability.

Inside the engine, `ObjectStore` uses editing guards to record connections on
drop, including early returns and unwinding. Mutable indexing and mutable slice
access are unavailable. `get_mut` and `iter_mut` return these guards. Import and
undo can replace the internal collection and register its connections again.

The change journal is bounded to 256 edits. Each evaluated view independently
reads events since its previous revision; events are not consumed by a reader.
Clones share revision tokens until edited, then diverge. Missing history or an
unrelated branch triggers fresh evaluation. Undo and redo restore the object
collection, its connections, and its revision together. No full object comparison
is needed to discover changes. Document-wide settings still use conservative
environment comparisons. Object tables use `Arc<Vec<Object>>` with
`Arc::make_mut` at each mutation guard: read-only scopes share the table and
actual edits detach it. Other snapshot metadata and evaluation work remain.

## Lifetime and invalidation

- Programs and update plans contain identities and edges, never coordinates.
  Coordinate changes preserve them; rewiring rebuilds them. Only the edited
  object's connections are checked at the mutation boundary. No separate manual
  dirty-marking call is required. Sampling plans compare shared graph identity.
- Each evaluated view owns its current values. Sampling baselines share immutable
  data and detach on mutation. Sibling samples and older views retain their values.
- LRU caches bound reusable sampling plans, update plans, and named SAC scopes.
  SAC memo tables bound calculated values and derived geometry. Geometry keys
  include query kind and resolution; actual input reads govern revalidation.
  Parsed formula programs are immutable and shared by clones of their owning
  formula; reparsing an edited formula creates a new program.
- A result cache can contain an explicitly undefined result. A later input
  change must invalidate it just like a defined result.
- Global settings that can affect semantics conservatively rebuild the view.
  Metadata-only edits still refresh the snapshot; font edits invalidate font
  metadata. Object presentation fields are separate from construction edges.

The SAC runtime handles points, numbers, straight hosts, circles, polygons, arcs,
coordinate frames, curve samples and derived display geometry. Equal results
stop propagation; evaluation replaces stale branch traces. Stable nested scope
names allow sampling and iteration computations to survive successive demands.
Static cycles and invalid references prevent an initial compiled dependency
program; conservative evaluation remains available for partial documents.
Runtime cycles produce an invalidatable undefined result. Cyclic constraint
solving would need a separate solver node.

Regression coverage compares incremental views and parallel samples with fresh
evaluation after input changes, formula edits, rewiring, deletion, undo, and redo.
Additional tests cover independent consumers, divergent clones, history overflow,
bulk edits, collection replacement, and edits interrupted by returns or panics.
IR tests cover merged inputs, boundaries, forward references, duplicate IDs,
and cycles.

See [3D sample performance](3D_PERFORMANCE.md) for measurements and the geometry
benchmark command. Reusing unchanged geometry does not establish animation FPS.
