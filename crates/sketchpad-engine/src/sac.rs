//! Demand-driven self-adjusting computations with explicit, stable identities.
//!
//! A resolver reads every mutable input through `Runtime::read`. Queries record
//! the inputs and child queries actually read, including branch conditions. A new
//! revision validates those reads before reusing a value. Equal child results
//! retain their change stamp, so propagation stops at that child. Reevaluation
//! replaces the old trace, rather than accumulating obsolete branch edges.
//!
//! Inputs must remain immutable during a revision. Start a new revision after
//! changing them. A resolver must be deterministic and must not perform effects.
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::collections::VecDeque;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;

// A scope shares its memo table until its first write. Values and traces stay
// individually shared when that write detaches the table.
#[derive(Clone)]
struct MemoMap<K, V>(Arc<HashMap<K, V>>);
impl<K, V> Default for MemoMap<K, V> {
    fn default() -> Self {
        Self(Arc::new(HashMap::default()))
    }
}
impl<K, V> std::ops::Deref for MemoMap<K, V> {
    type Target = HashMap<K, V>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<K: Clone, V: Clone> std::ops::DerefMut for MemoMap<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
    }
}
impl<K: Eq + std::hash::Hash, V> FromIterator<(K, V)> for MemoMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(values: T) -> Self {
        Self(Arc::new(values.into_iter().collect()))
    }
}

pub trait Resolver<K, I, V, S> {
    /// Optional input-table epoch for owners whose mutation boundary is separate
    /// from this runtime. Inputs still must be immutable throughout one query.
    fn revision_stamp(&self) -> Option<S> {
        None
    }
    /// Return false to retain a nested query as an independently reusable trace.
    fn can_inline(&self, _key: &K) -> bool {
        true
    }
    /// Called for an inlined query; input reads belong to its enclosing trace.
    fn evaluate_inline(&self, runtime: &Runtime<K, I, V, S>, key: &K) -> Result<V, Cycle<K>> {
        self.evaluate(runtime, key)
    }
    /// Cheap probes can opt out of retention without changing trace semantics.
    fn retain(&self, _key: &K, _value: &V) -> bool {
        true
    }
    fn input_stamp(&self, input: &I) -> S;
    fn evaluate(&self, runtime: &Runtime<K, I, V, S>, key: &K) -> Result<V, Cycle<K>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cycle<K>(pub K);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    pub hits: u64,
    pub verified: u64,
    pub executed: u64,
    pub unchanged: u64,
}

#[derive(Clone, Debug)]
enum Read<K, I, S> {
    Input(I, S),
    Query(K, u64),
}

#[derive(Clone, Debug)]
struct Node<K, I, V, S> {
    value: V,
    reads: Arc<Vec<Read<K, I, S>>>,
    changed: u64,
    verified: u64,
    epoch: Option<S>,
}
type NodeResult<K, I, V, S> = Result<Arc<Node<K, I, V, S>>, Cycle<K>>;

struct Frame<K, I, V, S> {
    key: K,
    recording: bool,
    reads: Vec<Read<K, I, S>>,
    inline_stack: Vec<K>,
    inline_values: HashMap<K, V>,
}

struct State<K, I, V, S> {
    nodes: MemoMap<K, Arc<Node<K, I, V, S>>>,
    insertion_order: Arc<VecDeque<K>>,
    capacity: usize,
    stacks: HashMap<ThreadId, Vec<Frame<K, I, V, S>>>,
    revision: u64,
    clock: u64,
    statistics: Statistics,
    touched: HashSet<K>,
    footprints: MemoMap<K, Arc<Vec<(I, S)>>>,
}

/// Clones share immutable memo payloads, but have independent revisions/traces.
/// Query evaluation never holds the state lock while calling user code. Several
/// readers may evaluate the same pure query concurrently; publication preserves
/// the change stamp when their results agree.
pub struct Runtime<K, I, V, S> {
    coalesce: bool,
    state: Mutex<State<K, I, V, S>>,
}

impl<K: Clone + Eq + Hash, I, V, S> Clone for Runtime<K, I, V, S> {
    fn clone(&self) -> Self {
        let state = self.state.lock().unwrap();
        Self {
            coalesce: self.coalesce,
            state: Mutex::new(State {
                nodes: state.nodes.clone(),
                insertion_order: state.insertion_order.clone(),
                capacity: state.capacity,
                stacks: HashMap::default(),
                revision: state.revision,
                clock: state.clock,
                statistics: Statistics::default(),
                touched: HashSet::default(),
                footprints: state.footprints.clone(),
            }),
        }
    }
}

impl<K, I, V, S> std::fmt::Debug for Runtime<K, I, V, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.state.lock().unwrap();
        f.debug_struct("Runtime")
            .field("revision", &state.revision)
            .field("statistics", &state.statistics)
            .finish_non_exhaustive()
    }
}

impl<K: Clone + Eq + Hash, I: Clone + Eq + Hash, V: Clone + PartialEq, S: Clone + PartialEq>
    Runtime<K, I, V, S>
{
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            coalesce: false,
            state: Mutex::new(State {
                nodes: MemoMap::default(),
                insertion_order: Arc::new(VecDeque::new()),
                capacity: capacity.get(),
                stacks: HashMap::default(),
                revision: 1,
                clock: 0,
                statistics: Statistics::default(),
                touched: HashSet::default(),
                footprints: MemoMap::default(),
            }),
        }
    }

    /// Record each externally demanded query as one trace. Nested queries use
    /// a temporary memo table for this execution and contribute their input
    /// reads to the enclosing trace. Equality cutoffs remain at trace boundaries.
    pub fn coalesce_nested_queries(&mut self, enabled: bool) {
        let state = self.state.get_mut().unwrap();
        debug_assert!(state.stacks.is_empty());
        self.coalesce = enabled;
    }

    pub fn advance(&mut self) {
        let state = self.state.get_mut().unwrap();
        debug_assert!(state.stacks.is_empty());
        state.touched.clear();
        state.revision = state
            .revision
            .checked_add(1)
            .expect("SAC revision overflow");
    }

    pub fn statistics(&self) -> Statistics {
        self.state.lock().unwrap().statistics
    }

    /// Keep demanded records and their transitive children when publishing a
    /// named scope. Inherited lookup entries remain useful locally, but retaining an
    /// entire caller table for every sample would crowd useful scopes out.
    pub fn snapshot_current(&self) -> Self {
        let state = self.state.lock().unwrap();
        let mut snapshot = State {
            nodes: MemoMap::default(),
            insertion_order: Arc::new(VecDeque::new()),
            capacity: state.capacity,
            stacks: HashMap::default(),
            revision: state.revision,
            clock: state.clock,
            statistics: Statistics::default(),
            touched: HashSet::default(),
            footprints: MemoMap::default(),
        };
        let mut pending: Vec<_> = state.touched.iter().cloned().collect();
        while let Some(key) = pending.pop() {
            if snapshot.nodes.contains_key(&key) {
                continue;
            }
            if let Some(node) = state.nodes.get(&key) {
                pending.extend(node.reads.iter().filter_map(|read| match read {
                    Read::Query(child, _) => Some(child.clone()),
                    _ => None,
                }));
                Self::publish(&mut snapshot, key, node.clone());
            }
        }
        snapshot.footprints = state
            .footprints
            .iter()
            .filter(|(key, _)| snapshot.nodes.contains_key(*key))
            .map(|(key, inputs)| (key.clone(), inputs.clone()))
            .collect();
        Self {
            coalesce: self.coalesce,
            state: Mutex::new(snapshot),
        }
    }

    pub fn revision(&self) -> u64 {
        self.state.lock().unwrap().revision
    }

    pub fn is_current(&self, key: &K) -> bool {
        let state = self.state.lock().unwrap();
        state
            .nodes
            .get(key)
            .is_some_and(|node| node.verified == state.revision)
    }

    pub fn len(&self) -> usize {
        self.state.lock().unwrap().nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Inputs observed by this query and its retained child computations.
    pub fn input_footprint(&self, key: &K) -> Option<Vec<I>> {
        self.stamped_footprint(key)
            .map(|inputs| inputs.iter().map(|(input, _)| input.clone()).collect())
    }

    /// Visit an immutable input summary without copying its inputs or stamps.
    /// The callback runs outside the state lock and may query the runtime.
    pub fn visit_input_footprint(&self, key: &K, mut visit: impl FnMut(&I)) -> bool {
        let Some(inputs) = self.stamped_footprint(key) else {
            return false;
        };
        for (input, _) in inputs.iter() {
            visit(input);
        }
        true
    }

    fn stamped_footprint(&self, key: &K) -> Option<Arc<Vec<(I, S)>>> {
        let mut state = self.state.lock().unwrap();
        if let Some(inputs) = state.footprints.get(key) {
            return Some(inputs.clone());
        }
        let mut visited = HashSet::default();
        let mut pending = vec![key.clone()];
        let mut inputs = Vec::new();
        let mut seen_inputs = HashSet::default();
        while let Some(key) = pending.pop() {
            if !visited.insert(key.clone()) {
                continue;
            }
            let node = state.nodes.get(&key)?;
            for read in node.reads.iter() {
                match read {
                    Read::Input(input, stamp) => {
                        if seen_inputs.insert(input.clone()) {
                            inputs.push((input.clone(), stamp.clone()));
                        }
                    }
                    Read::Query(child, changed) => {
                        // A child may have been demanded independently since
                        // this parent last ran. Its new inputs cannot justify
                        // reuse of the parent's old result.
                        if state.nodes.get(child)?.changed != *changed {
                            return None;
                        }
                        pending.push(child.clone());
                    }
                }
            }
        }
        let inputs = Arc::new(inputs);
        state.footprints.insert(key.clone(), inputs.clone());
        Some(inputs)
    }

    /// Observe an input before using its value. Stamps need equality, not order;
    /// immutable value identities or exact values are both valid stamps.
    pub fn read(&self, resolver: &impl Resolver<K, I, V, S>, input: I) {
        let stamp = resolver.input_stamp(&input);
        self.record(Read::Input(input, stamp));
    }

    /// Register a batch of observed inputs with one recording lock.
    pub fn read_many(
        &self,
        resolver: &impl Resolver<K, I, V, S>,
        inputs: impl IntoIterator<Item = I>,
    ) {
        let reads: Vec<_> = inputs
            .into_iter()
            .map(|input| {
                let stamp = resolver.input_stamp(&input);
                Read::Input(input, stamp)
            })
            .collect();
        let mut state = self.state.lock().unwrap();
        if let Some(frame) = state
            .stacks
            .get_mut(&std::thread::current().id())
            .and_then(|s| s.last_mut())
            && frame.recording
        {
            let mut seen: HashSet<_> = frame
                .reads
                .iter()
                .filter_map(|read| match read {
                    Read::Input(input, _) => Some(input.clone()),
                    _ => None,
                })
                .collect();
            frame
                .reads
                .extend(reads.into_iter().filter(|read| match read {
                    Read::Input(input, _) => seen.insert(input.clone()),
                    _ => unreachable!(),
                }));
        }
    }

    /// Engine recordings already contain unique inputs. They are submitted
    /// once, after evaluating the query; existing frame reads are child edges.
    pub(crate) fn read_unique_inputs(
        &self,
        resolver: &impl Resolver<K, I, V, S>,
        inputs: &HashSet<I>,
    ) {
        let reads = inputs
            .iter()
            .map(|input| Read::Input(input.clone(), resolver.input_stamp(input)))
            .collect::<Vec<_>>();
        let mut state = self.state.lock().unwrap();
        if let Some(frame) = state
            .stacks
            .get_mut(&std::thread::current().id())
            .and_then(|stack| stack.last_mut())
            .filter(|frame| frame.recording)
        {
            debug_assert!(
                frame
                    .reads
                    .iter()
                    .all(|read| matches!(read, Read::Query(..)))
            );
            frame.reads.extend(reads);
        }
    }

    pub fn query(&self, resolver: &impl Resolver<K, I, V, S>, key: K) -> Result<V, Cycle<K>> {
        if let Some(result) = self.query_inline(resolver, &key) {
            return result;
        }
        let node = self.query_node(resolver, key.clone())?;
        self.record(Read::Query(key, node.changed));
        Ok(node.value.clone())
    }

    fn query_inline(
        &self,
        resolver: &impl Resolver<K, I, V, S>,
        key: &K,
    ) -> Option<Result<V, Cycle<K>>> {
        if !self.coalesce || !resolver.can_inline(key) {
            return None;
        }
        let thread = std::thread::current().id();
        {
            let mut state = self.state.lock().unwrap();
            let stack = state.stacks.get_mut(&thread)?;
            if stack
                .iter()
                .any(|frame| &frame.key == key || frame.inline_stack.contains(key))
            {
                return Some(Err(Cycle(key.clone())));
            }
            let frame = stack.last_mut()?;
            if !frame.recording {
                return None;
            }
            if let Some(value) = frame.inline_values.get(key) {
                return Some(Ok(value.clone()));
            }
            frame.inline_stack.push(key.clone());
        }
        let _guard = InlineGuard {
            runtime: self,
            thread,
        };
        let result = resolver.evaluate_inline(self, key);
        if let Ok(value) = &result {
            let mut state = self.state.lock().unwrap();
            let capacity = state.capacity;
            let frame = state.stacks.get_mut(&thread).unwrap().last_mut().unwrap();
            if frame.inline_values.len() == capacity {
                frame.inline_values.clear();
            }
            frame.inline_values.insert(key.clone(), value.clone());
        }
        Some(result)
    }

    fn query_node(&self, resolver: &impl Resolver<K, I, V, S>, key: K) -> NodeResult<K, I, V, S> {
        let thread = std::thread::current().id();
        let epoch = resolver.revision_stamp();
        let (old, revision, footprint) = {
            let mut state = self.state.lock().unwrap();
            state.touched.insert(key.clone());
            if state.stacks.get(&thread).is_some_and(|stack| {
                stack
                    .iter()
                    .any(|f| f.key == key || f.inline_stack.contains(&key))
            }) {
                return Err(Cycle(key));
            }
            let old = state.nodes.get(&key).cloned();
            if let Some(node) = &old
                && node.verified == state.revision
                && node.epoch == epoch
            {
                state.statistics.hits += 1;
                return Ok(node.clone());
            }
            state.stacks.entry(thread).or_default().push(Frame {
                key: key.clone(),
                recording: false,
                reads: Vec::new(),
                inline_stack: Vec::new(),
                inline_values: HashMap::default(),
            });
            let footprint = state.footprints.get(&key).cloned();
            (old, state.revision, footprint)
        };
        // Pop even when the resolver panics or returns a cycle. No partially
        // collected trace is published; the previous memo remains unverified.
        let guard = FrameGuard {
            runtime: self,
            thread,
        };
        if let Some(node) = &old {
            // A previously exported footprint contains the exact transitive
            // input stamps. Identical inputs permit reuse without visiting each
            // child; changed inputs still use ordinary value-based propagation.
            let identical = footprint.as_ref().is_some_and(|inputs| {
                inputs
                    .iter()
                    .all(|(input, stamp)| resolver.input_stamp(input) == *stamp)
            });
            let mut valid = true;
            // Validate local branch conditions before walking old child edges.
            for read in node
                .reads
                .iter()
                .filter(|read| matches!(read, Read::Input(..)))
                .chain(
                    node.reads
                        .iter()
                        .filter(|read| matches!(read, Read::Query(..))),
                )
            {
                if identical {
                    break;
                }
                valid = match read {
                    Read::Input(input, stamp) => resolver.input_stamp(input) == *stamp,
                    Read::Query(child, stamp) => {
                        self.query_node(resolver, child.clone())?.changed == *stamp
                    }
                };
                if !valid {
                    break;
                }
            }
            if valid {
                let mut verified = (**node).clone();
                verified.verified = revision;
                verified.epoch = epoch;
                let verified = Arc::new(verified);
                let mut state = self.state.lock().unwrap();
                state.statistics.verified += 1;
                Self::publish(&mut state, key, verified.clone());
                return Ok(verified);
            }
        }
        {
            let mut state = self.state.lock().unwrap();
            state
                .stacks
                .get_mut(&thread)
                .unwrap()
                .last_mut()
                .unwrap()
                .recording = true;
            state.statistics.executed += 1;
        }
        let value = resolver.evaluate(self, &key)?;
        let retain = resolver.retain(&key, &value);
        let reads = {
            let mut state = self.state.lock().unwrap();
            std::mem::take(
                &mut state
                    .stacks
                    .get_mut(&thread)
                    .unwrap()
                    .last_mut()
                    .unwrap()
                    .reads,
            )
        };
        drop(guard);
        let mut state = self.state.lock().unwrap();
        // Equal values can still acquire different branch dependencies.
        state.footprints = MemoMap::default();
        let previous = state.nodes.get(&key).cloned().or(old);
        let changed = if let Some(previous) = previous.filter(|node| node.value == value) {
            state.statistics.unchanged += 1;
            previous.changed
        } else {
            state.clock = state
                .clock
                .checked_add(1)
                .expect("SAC change stamp overflow");
            state.clock
        };
        let node = Arc::new(Node {
            value,
            reads: Arc::new(reads),
            changed,
            verified: revision,
            epoch,
        });
        if retain {
            Self::publish(&mut state, key, node.clone());
        }
        Ok(node)
    }

    fn publish(state: &mut State<K, I, V, S>, key: K, node: Arc<Node<K, I, V, S>>) {
        if !state.nodes.contains_key(&key) {
            state.footprints = MemoMap::default();
            if state.nodes.len() == state.capacity {
                let oldest = Arc::make_mut(&mut state.insertion_order)
                    .pop_front()
                    .unwrap();
                state.nodes.remove(&oldest);
            }
            Arc::make_mut(&mut state.insertion_order).push_back(key.clone());
        }
        state.nodes.insert(key, node);
    }

    fn record(&self, read: Read<K, I, S>) {
        self.record_many(std::iter::once(read));
    }

    fn record_many(&self, reads: impl IntoIterator<Item = Read<K, I, S>>) {
        let mut state = self.state.lock().unwrap();
        if let Some(frame) = state
            .stacks
            .get_mut(&std::thread::current().id())
            .and_then(|s| s.last_mut())
            && frame.recording
        {
            // Repeated reads do not grow traces. Input immutability during a
            // revision guarantees that the first observed stamp remains valid.
            for read in reads {
                if !frame.reads.iter().any(|old| match (old, &read) {
                    (Read::Input(a, _), Read::Input(b, _)) => a == b,
                    (Read::Query(a, _), Read::Query(b, _)) => a == b,
                    _ => false,
                }) {
                    frame.reads.push(read);
                }
            }
        }
    }

    /// Optional collection after consumers have published their current roots.
    /// Removing an old branch never permits a stale reuse: an evicted child gets
    /// a new change stamp when reconstructed. FIFO eviction also bounds uncollected traces.
    pub fn retain_reachable(&mut self, roots: impl IntoIterator<Item = K>) {
        let state = self.state.get_mut().unwrap();
        state.footprints = MemoMap::default();
        let mut reachable = HashSet::default();
        let mut pending: Vec<_> = roots.into_iter().collect();
        while let Some(key) = pending.pop() {
            if !reachable.insert(key.clone()) {
                continue;
            }
            if let Some(node) = state.nodes.get(&key) {
                pending.extend(node.reads.iter().filter_map(|read| match read {
                    Read::Query(child, _) => Some(child.clone()),
                    _ => None,
                }));
            }
        }
        let dead: Vec<_> = state
            .nodes
            .iter()
            .filter(|(k, _)| !reachable.contains(*k))
            .map(|(k, _)| k.clone())
            .collect();
        for key in dead {
            state.nodes.remove(&key);
        }
        Arc::make_mut(&mut state.insertion_order).retain(|key| reachable.contains(key));
    }
}

struct InlineGuard<'a, K, I, V, S> {
    runtime: &'a Runtime<K, I, V, S>,
    thread: ThreadId,
}
impl<K, I, V, S> Drop for InlineGuard<'_, K, I, V, S> {
    fn drop(&mut self) {
        self.runtime
            .state
            .lock()
            .unwrap()
            .stacks
            .get_mut(&self.thread)
            .unwrap()
            .last_mut()
            .unwrap()
            .inline_stack
            .pop();
    }
}

struct FrameGuard<'a, K, I, V, S> {
    runtime: &'a Runtime<K, I, V, S>,
    thread: ThreadId,
}

impl<K, I, V, S> Drop for FrameGuard<'_, K, I, V, S> {
    fn drop(&mut self) {
        let mut state = self.runtime.state.lock().unwrap();
        let stack = state.stacks.get_mut(&self.thread).unwrap();
        stack.pop();
        if stack.is_empty() {
            state.stacks.remove(&self.thread);
        }
    }
}

#[cfg(test)]
mod tests;
