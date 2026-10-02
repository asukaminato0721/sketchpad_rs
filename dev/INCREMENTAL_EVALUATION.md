# Construction dependency programs

The construction objects are the computation graph. `Object` implements the
IR's `DependencyNode` contract through `construction_dependencies()`. A new
operation declares its input IDs in `Kind::dependencies()` and implements its
geometry or numeric evaluator. It then participates in graph analysis without
an operation-specific invalidation or scheduling implementation.

This resembles the graph recording stage of automatic differentiation. It
records construction dependencies, not derivatives. It does not trace arbitrary
Rust reads: every semantic input must be declared. Conditional operations must
declare all possible inputs unless their edges are explicitly rebuilt.

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
   temporary inputs when present, and materializes values in that order.
5. `EvaluationCache` uses this executor for document edits. `SamplingPlan` uses
   it for point, numeric, and path-parameter substitutions in isolated scopes.
   Initial read-only evaluation uses the same dependency program's full order.

For `C = midpoint(A, B)` and `d = distance(C, B)`, changing `A` creates an update
ordered as `A, C, d`, with `B` as a preserved boundary. An unrelated construction
is absent from that update. Repeated changes to `A` reuse the same plan.

## Editing API

```rust
let c = doc.add(Kind::Midpoint(a, b)); // records input connections now
doc.move_point(a, next_position);     // records a change; retains graph identity
{
    let mut object = doc.objects.edit(c - 1); // zero-based storage slot
    object.kind = Kind::Midpoint(a, another_point);
    object.midpoint_host = None;
} // replaces connections and records the change, even on an early return
```

`objects` is a read-only slice view with tracked mutation methods, not a public
`Vec`. Mutable indexing and mutable slice access are unavailable. `get_mut` and
`iter_mut` yield the same editing guards. Replacing the entire collection uses
`Vec<Object>::into()` and registers all connections again. Existing editing,
clipboard, tool expansion, import, save, and undo paths use this boundary.

The change journal is bounded to 256 edits. Each evaluated view independently
reads events since its previous revision; events are not consumed by a reader.
Clones share revision tokens until edited, then diverge. Missing history or an
unrelated branch triggers fresh evaluation. Undo and redo restore the object
collection, its connections, and its revision together. No full object comparison
is needed to discover changes. Document-wide settings still use conservative
environment comparisons; this does not remove snapshot copying or evaluation work.

## Lifetime and invalidation

- Programs and update plans contain identities and edges, never coordinates.
  Coordinate changes preserve them; rewiring rebuilds them. Only the edited
  object's connections are checked at the mutation boundary. No separate manual
  dirty-marking call is required. Sampling plans compare shared graph identity.
- Each evaluated view owns its current values. Sampling baselines share immutable
  data and detach on mutation. Sibling samples and older views retain their values.
- LRU caches bound reusable sampling plans, update plans, and sampled curves.
  Parsed formula programs are immutable and shared by clones of their owning
  formula; reparsing an edited formula creates a new program.
- A result cache can contain an explicitly undefined result. A later input
  change must invalidate it just like a defined result.
- Global settings that can affect semantics conservatively rebuild the view.
  Metadata-only edits still refresh the snapshot; font edits invalidate font
  metadata. Object presentation fields are separate from construction edges.

The executor eagerly caches points, numeric values, straight hosts, and circles.
Other operations continue through their existing evaluators; their dependencies
and invalidation still use the common graph. Cycles and invalid references are
rejected by compilation, and existing conservative evaluation remains the
fallback for unsupported or partial graphs. Cyclic constraint solving would
need a separate solver node or a cycle-aware execution contract.

Regression coverage compares incremental views and parallel samples with fresh
evaluation after input changes, formula edits, rewiring, deletion, undo, and redo.
Additional tests cover independent consumers, divergent clones, history overflow,
bulk edits, collection replacement, and edits interrupted by returns or panics.
IR tests cover merged inputs, boundaries, forward references, duplicate IDs,
and cycles.
