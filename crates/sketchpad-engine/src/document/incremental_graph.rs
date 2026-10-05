//! Thread-confined Incremental graphs backed by shareable, immutable snapshots.
//!
//! The library owns scheduling, necessity and value cutoffs. The adapter records
//! evaluator reads and reconnects expert nodes in a topology callback. Callbacks
//! borrow the current document through scoped TLS; no document or observer is
//! captured by a node, and no non-Send library handle enters a document snapshot.
use super::Document;
use super::sac_evaluation::{Input, Query, Stamp, Value};
use incremental::{Incr, IncrState, Observer, Var, expert};
use rustc_hash::{FxHashMap as Map, FxHashSet as Set};
use std::cell::{Cell, RefCell};
use std::num::NonZeroUsize;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_CHANGE: AtomicU64 = AtomicU64::new(1);
scoped_tls::scoped_thread_local!(static DOCUMENT: Document);
scoped_tls::scoped_thread_local!(static ACTIVE: Live);
thread_local! {
    static GRAPHS: RefCell<lru::LruCache<u64, Rc<Live>>> =
        RefCell::new(lru::LruCache::new(NonZeroUsize::new(64).unwrap()));
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Statistics {
    pub executed: u64,
    pub unchanged: u64,
    pub stabilisations: u64,
}

#[derive(Clone, Debug)]
struct Record {
    value: Value,
    inputs: Vec<(Input, Stamp)>,
    children: Vec<(Query, u64)>,
    changed: u64,
}

#[derive(Clone, Default)]
struct Snapshot {
    records: Arc<Map<Query, Arc<Record>>>,
    retention_limit: usize,
    touched: Set<Query>,
    statistics: Statistics,
    footprints: Map<Query, Arc<Vec<(Input, Stamp)>>>,
    epoch: Option<(u64, Stamp)>,
}

pub(super) struct Graph {
    id: u64,
    family: u64,
    revision: u64,
    coalesce: bool,
    capacity: usize,
    snapshot: Mutex<Snapshot>,
}

impl std::fmt::Debug for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IncrementalGraph")
            .field("id", &self.id)
            .finish()
    }
}

impl Clone for Graph {
    fn clone(&self) -> Self {
        let snapshot = self.snapshot.lock().unwrap();
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            family: self.family,
            revision: self.revision,
            coalesce: self.coalesce,
            capacity: self.capacity,
            snapshot: Mutex::new(Snapshot {
                records: snapshot.records.clone(),
                retention_limit: snapshot.retention_limit,
                ..Snapshot::default()
            }),
        }
    }
}

impl Graph {
    pub(super) fn new(capacity: NonZeroUsize) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            id,
            family: id,
            revision: 1,
            capacity: capacity.get(),
            coalesce: false,
            snapshot: Mutex::new(Snapshot::default()),
        }
    }

    pub(super) fn new_family() -> u64 {
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    }
    pub(super) fn use_family(&mut self, family: u64) {
        self.family = family;
    }
    pub(super) fn detach_execution(&mut self) {
        self.id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        self.family = self.id;
        // New named kernels do not inherit unrelated caller query tables.
        // Their live execution family supplies invariant intermediates.
        *self.snapshot.get_mut().unwrap() = Snapshot::default();
    }

    pub(super) fn revision(&self) -> u64 {
        self.revision
    }
    pub(super) fn advance(&mut self) {
        self.revision += 1;
        self.snapshot.get_mut().unwrap().touched.clear();
        self.snapshot.get_mut().unwrap().epoch = None;
    }
    pub(super) fn coalesce_nested_queries(&mut self, enabled: bool) {
        self.coalesce = enabled;
    }
    pub(super) fn len(&self) -> usize {
        self.snapshot.lock().unwrap().records.len()
    }
    pub(super) fn is_current(&self, key: &Query) -> bool {
        self.snapshot.lock().unwrap().touched.contains(key)
    }
    #[cfg(test)]
    pub(super) fn statistics(&self) -> Statistics {
        self.snapshot.lock().unwrap().statistics
    }

    pub(super) fn snapshot_current(&self) -> Self {
        let source = self.snapshot.lock().unwrap();
        let mut records = Map::default();
        let mut pending: Vec<_> = source.touched.iter().copied().collect();
        while let Some(key) = pending.pop() {
            if records.contains_key(&key) {
                continue;
            }
            if let Some(record) = source.records.get(&key) {
                pending.extend(record.children.iter().map(|(child, _)| *child));
                records.insert(key, record.clone());
            }
        }
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            family: self.family,
            revision: self.revision,
            coalesce: self.coalesce,
            capacity: self.capacity,
            snapshot: Mutex::new(Snapshot {
                records: Arc::new(records),
                ..Snapshot::default()
            }),
        }
    }

    pub(super) fn visit_input_footprint(&self, key: &Query, mut visit: impl FnMut(&Input)) -> bool {
        let inputs = if ACTIVE.is_set() && ACTIVE.with(|live| live.id.get() == self.id) {
            ACTIVE.with(|live| footprint(&live.records.borrow(), *key))
        } else {
            let mut snapshot = self.snapshot.lock().unwrap();
            if let Some(inputs) = snapshot.footprints.get(key) {
                Some(inputs.clone())
            } else {
                let inputs = footprint(&snapshot.records, *key);
                if let Some(inputs) = &inputs {
                    snapshot.footprints.insert(*key, inputs.clone());
                }
                inputs
            }
        };
        let Some(inputs) = inputs else {
            // Missing traces must also observe direct object edits, which can
            // happen without an explicit evaluation-context advance.
            visit(&Input::Objects);
            return false;
        };
        for (input, _) in inputs.iter() {
            visit(input);
        }
        true
    }

    #[cfg(test)]
    pub(super) fn input_footprint(&self, key: &Query) -> Option<Vec<Input>> {
        let mut inputs = Vec::new();
        self.visit_input_footprint(key, |input| inputs.push(*input))
            .then_some(inputs)
    }

    pub(super) fn read_unique_inputs(&self, document: &Document, inputs: &[Input]) {
        ACTIVE.with(|live| {
            let mut frames = live.frames.borrow_mut();
            let frame = frames.last_mut().unwrap();
            frame.inputs.extend(
                inputs
                    .iter()
                    .map(|input| (*input, document.sac_input_stamp(input))),
            );
        });
    }

    pub(super) fn query(&self, document: &Document, key: Query) -> Result<Value, ()> {
        if ACTIVE.is_set() && ACTIVE.with(|live| live.id.get() == self.id) {
            return ACTIVE.with(|live| live.nested(document, key));
        }
        let epoch = (self.revision, document.sac_input_stamp(&Input::Objects));
        {
            let snapshot = self.snapshot.lock().unwrap();
            if snapshot.epoch.as_ref() == Some(&epoch)
                && snapshot.touched.contains(&key)
                && let Some(record) = snapshot.records.get(&key)
            {
                return Ok(record.value.clone());
            }
        }
        // A named immutable result can be reused without switching the live
        // scheduler away from another sample. This is a tuple-input cache;
        // changed inputs go through Incremental for propagation and cutoffs.
        let cached = {
            let mut snapshot = self.snapshot.lock().unwrap();
            if let Some(record) = snapshot.records.get(&key).cloned() {
                let footprint = if let Some(inputs) = snapshot.footprints.get(&key) {
                    Some(inputs.clone())
                } else {
                    let inputs = footprint(&snapshot.records, key);
                    if let Some(inputs) = &inputs {
                        snapshot.footprints.insert(key, inputs.clone());
                    }
                    inputs
                };
                footprint.map(|inputs| (record, inputs))
            } else {
                None
            }
        };
        if let Some((record, inputs)) = cached
            && inputs
                .iter()
                .all(|(input, stamp)| document.sac_input_stamp(input) == *stamp)
        {
            let value = record.value.clone();
            let mut snapshot = self.snapshot.lock().unwrap();
            if snapshot
                .records
                .get(&key)
                .is_none_or(|old| !Arc::ptr_eq(old, &record))
            {
                Arc::make_mut(&mut snapshot.records).insert(key, record);
                snapshot.footprints.clear();
            }
            if snapshot.epoch.as_ref() != Some(&epoch) {
                snapshot.touched.clear();
            }
            snapshot.epoch = Some(epoch);
            snapshot.touched.insert(key);
            return Ok(value);
        }
        let live = GRAPHS.with_borrow_mut(|graphs| {
            if let Some(live) = graphs.get(&self.family)
                && !live.state.is_stabilising()
                && live.frames.borrow().is_empty()
            {
                if live.id.replace(self.id) != self.id {
                    live.statistics
                        .set(self.snapshot.lock().unwrap().statistics);
                    *live.epoch.borrow_mut() = None;
                }
                return live.clone();
            }
            let snapshot = self.snapshot.lock().unwrap();
            let live = Rc::new(Live::new(
                self.id,
                self.coalesce,
                snapshot.records.clone(),
                snapshot.statistics,
            ));
            graphs.put(self.family, live.clone());
            live
        });
        struct DiscardOnPanic(u64);
        impl Drop for DiscardOnPanic {
            fn drop(&mut self) {
                if std::thread::panicking() {
                    GRAPHS.with_borrow_mut(|graphs| {
                        graphs.pop(&self.0);
                    });
                }
            }
        }
        let _discard = DiscardOnPanic(self.family);
        let result = DOCUMENT.set(document, || {
            ACTIVE.set(&live, || live.demand(document, self.revision, key))
        });
        // Nested sample graphs can evict their caller while it is running.
        // Keep the caller hot for the next drawing query after samples finish.
        GRAPHS.with_borrow_mut(|graphs| {
            graphs.put(self.family, live.clone());
        });
        let mut snapshot = self.snapshot.lock().unwrap();
        // Publish only results validated by this reader. Another thread may
        // already have refreshed other keys in the shared snapshot; replacing
        // its entire map with our inherited map would resurrect stale results.
        let records = live.records.borrow();
        let mut changed = false;
        let mut pending = vec![key];
        let mut visited = Set::default();
        while let Some(current) = pending.pop() {
            if !visited.insert(current) {
                continue;
            }
            if let Some(record) = records.get(&current) {
                pending.extend(record.children.iter().map(|(child, _)| *child));
                if snapshot
                    .records
                    .get(&current)
                    .is_none_or(|old| !Arc::ptr_eq(old, record))
                {
                    Arc::make_mut(&mut snapshot.records).insert(current, record.clone());
                    changed = true;
                }
            }
        }
        if changed {
            snapshot.footprints.clear();
        }
        if snapshot.epoch.as_ref() != Some(&epoch) {
            snapshot.touched.clear();
        }
        snapshot.epoch = Some(epoch);
        snapshot.touched.insert(key);
        snapshot.statistics = live.statistics.get();
        if snapshot.records.len() > self.capacity.max(snapshot.retention_limit) {
            // Capacity limits inactive history, not the current working set.
            // Retain every demanded root and its complete dependency subgraph;
            // evicting other roots in the same epoch makes large drawings thrash.
            let mut retained = Map::default();
            let mut pending: Vec<_> = snapshot.touched.iter().copied().collect();
            while let Some(key) = pending.pop() {
                if retained.contains_key(&key) {
                    continue;
                }
                if let Some(record) = snapshot.records.get(&key) {
                    pending.extend(record.children.iter().map(|(child, _)| *child));
                    retained.insert(key, record.clone());
                }
            }
            // Amortise collection when the active working set itself exceeds
            // capacity. Re-scanning it after each query would be quadratic.
            snapshot.retention_limit = retained.len().saturating_add(self.capacity);
            if retained.len() < snapshot.records.len() {
                snapshot.records = Arc::new(retained);
                snapshot.footprints.clear();
                let kept: Set<_> = snapshot.records.keys().copied().collect();
                snapshot.touched.retain(|key| kept.contains(key));
                GRAPHS.with_borrow_mut(|graphs| {
                    graphs.pop(&self.family);
                });
            }
        }
        result
    }
}

fn footprint(records: &Map<Query, Arc<Record>>, key: Query) -> Option<Arc<Vec<(Input, Stamp)>>> {
    let mut pending = vec![key];
    let mut visited = Set::default();
    let mut seen_inputs = Set::default();
    let mut inputs = Vec::new();
    while let Some(key) = pending.pop() {
        if !visited.insert(key) {
            continue;
        }
        let record = records.get(&key)?;
        for (input, stamp) in &record.inputs {
            if seen_inputs.insert(*input) {
                inputs.push((*input, stamp.clone()));
            }
        }
        for (child, changed) in &record.children {
            if records.get(child)?.changed != *changed {
                return None;
            }
            pending.push(*child);
        }
    }
    Some(Arc::new(inputs))
}

struct Frame {
    key: Query,
    inputs: Vec<(Input, Stamp)>,
    children: Vec<(Query, u64)>,
    // A new dynamic edge must be evaluated before it can join the scheduled DAG.
    direct: bool,
    inline_stack: Vec<Query>,
    inline_values: Map<Query, Value>,
}

struct Edges {
    inputs: Vec<expert::Dependency<Stamp>>,
    children: Vec<expert::Dependency<Value>>,
}

struct Live {
    id: Cell<u64>,
    coalesce: bool,
    state: IncrState,
    topology: Var<u64>,
    reconnect_node: Incr<()>,
    topology_revision: Cell<u64>,
    height_limit: Cell<usize>,
    refreshed: RefCell<Set<Query>>,
    records: RefCell<Arc<Map<Query, Arc<Record>>>>,
    nodes: RefCell<Map<Query, expert::Node<Value>>>,
    inputs: RefCell<Map<Input, (Stamp, Var<Stamp>)>>,
    edges: RefCell<Map<Query, Edges>>,
    frames: RefCell<Vec<Frame>>,
    rewired: RefCell<Set<Query>>,
    roots: RefCell<Map<Query, Observer<Value>>>,
    statistics: Cell<Statistics>,
    epoch: RefCell<Option<(u64, Stamp)>>,
    settled: RefCell<Set<Query>>,
}

impl Live {
    fn new(
        id: u64,
        coalesce: bool,
        records: Arc<Map<Query, Arc<Record>>>,
        statistics: Statistics,
    ) -> Self {
        let state = IncrState::new_with_height(4);
        let topology = state.var(0);
        let reconnect_node = topology
            .map(|_| DOCUMENT.with(|document| ACTIVE.with(|live| live.reconnect(document))));
        Self {
            id: Cell::new(id),
            coalesce,
            state,
            topology,
            reconnect_node,
            topology_revision: Cell::new(0),
            height_limit: Cell::new(4),
            refreshed: RefCell::new(Set::default()),
            rewired: RefCell::new(Set::default()),
            records: RefCell::new(records),
            nodes: RefCell::new(Map::default()),
            inputs: RefCell::new(Map::default()),
            edges: RefCell::new(Map::default()),
            frames: RefCell::new(Vec::new()),
            roots: RefCell::new(Map::default()),
            statistics: Cell::new(statistics),
            epoch: RefCell::new(None),
            settled: RefCell::new(Set::default()),
        }
    }

    fn demand(&self, document: &Document, revision: u64, key: Query) -> Result<Value, ()> {
        let epoch = (revision, document.sac_input_stamp(&Input::Objects));
        if self.epoch.borrow().as_ref() == Some(&epoch)
            && self.roots.borrow().contains_key(&key)
            && self.rewired.borrow().is_empty()
            && self.refreshed.borrow().is_empty()
        {
            return Ok(self.roots.borrow()[&key].value());
        }
        if self.epoch.borrow().as_ref() != Some(&epoch) {
            self.settled.borrow_mut().clear();
        }
        // A coalesced sampling kernel with no retained child queries is a
        // pure function of its recorded inputs. Its containing geometry node
        // is scheduled by Incremental; internal arithmetic needs no scheduler.
        if self.coalesce
            && self
                .records
                .borrow()
                .get(&key)
                .is_none_or(|record| record.children.is_empty())
        {
            let value = self.evaluate(document, key, true)?;
            if self.records.borrow()[&key].children.is_empty() {
                *self.epoch.borrow_mut() = Some(epoch);
                return Ok(value);
            }
        }
        for (input, (stamp, variable)) in self.inputs.borrow_mut().iter_mut() {
            let current = document.sac_input_stamp(input);
            if *stamp != current {
                variable.set(current.clone());
                *stamp = current;
            }
        }
        if !self.records.borrow().contains_key(&key) {
            self.evaluate(document, key, true)?;
        }
        if !self.nodes.borrow().contains_key(&key) {
            self.rewired.borrow_mut().insert(key);
        }
        self.prepare();
        if !self.roots.borrow().contains_key(&key) {
            self.roots
                .borrow_mut()
                .insert(key, self.nodes.borrow()[&key].watch().observe());
        }
        loop {
            self.state.stabilise();
            let mut stats = self.statistics.get();
            stats.stabilisations += 1;
            self.statistics.set(stats);
            if self.rewired.borrow().is_empty() && self.refreshed.borrow().is_empty() {
                break;
            }
            self.prepare();
        }
        *self.epoch.borrow_mut() = Some(epoch);
        Ok(self.roots.borrow()[&key].value())
    }

    fn recompute(&self, document: &Document, key: Query) -> Value {
        let record = self.records.borrow()[&key].clone();
        // Imported snapshots and newly connected edges need one library pass.
        // Compare direct dependencies only; Incremental has already scheduled
        // every existing child before this callback and handles transitive work.
        let valid = record
            .inputs
            .iter()
            .all(|(input, stamp)| document.sac_input_stamp(input) == *stamp)
            && record.children.iter().all(|(child, changed)| {
                self.records
                    .borrow()
                    .get(child)
                    .is_some_and(|r| r.changed == *changed)
            });
        if valid {
            self.settled.borrow_mut().insert(key);
            return record.value.clone();
        }
        self.evaluate(document, key, false)
            .expect("scheduled dependency graph is acyclic")
    }

    fn nested(&self, document: &Document, key: Query) -> Result<Value, ()> {
        {
            let frames = self.frames.borrow();
            if frames
                .iter()
                .any(|f| f.key == key || f.inline_stack.contains(&key))
            {
                return Err(());
            }
        }
        if self.coalesce && document.sac_can_inline(&key) {
            if let Some(value) = self
                .frames
                .borrow()
                .last()
                .and_then(|f| f.inline_values.get(&key))
                .cloned()
            {
                return Ok(value);
            }
            self.frames
                .borrow_mut()
                .last_mut()
                .unwrap()
                .inline_stack
                .push(key);
            let value = document.sac_evaluate_body(key);
            let mut frames = self.frames.borrow_mut();
            let frame = frames.last_mut().unwrap();
            frame.inline_stack.pop();
            frame.inline_values.insert(key, value.clone());
            return Ok(value);
        }
        let direct = {
            let frames = self.frames.borrow();
            let parent = frames.last().unwrap();
            parent.direct
                || self
                    .records
                    .borrow()
                    .get(&parent.key)
                    .is_none_or(|r| !r.children.iter().any(|(child, _)| *child == key))
        };
        let value = if (direct && !self.settled.borrow().contains(&key))
            || !self.records.borrow().contains_key(&key)
        {
            self.evaluate(document, key, true)?
        } else {
            self.records.borrow()[&key].value.clone()
        };
        let changed = self.records.borrow()[&key].changed;
        self.frames
            .borrow_mut()
            .last_mut()
            .unwrap()
            .children
            .push((key, changed));
        Ok(value)
    }

    fn evaluate(&self, document: &Document, key: Query, direct: bool) -> Result<Value, ()> {
        self.frames.borrow_mut().push(Frame {
            key,
            direct,
            inputs: Vec::new(),
            children: Vec::new(),
            inline_stack: Vec::new(),
            inline_values: Map::default(),
        });
        struct Pop<'a>(&'a RefCell<Vec<Frame>>);
        impl Drop for Pop<'_> {
            fn drop(&mut self) {
                self.0.borrow_mut().pop();
            }
        }
        let guard = Pop(&self.frames);
        let value = document.sac_execute_query(&key);
        let (mut inputs, mut children) = {
            let mut frames = self.frames.borrow_mut();
            let frame = frames.last_mut().unwrap();
            (
                std::mem::take(&mut frame.inputs),
                std::mem::take(&mut frame.children),
            )
        };
        // Store canonical edge lists once, instead of allocating four hash sets
        // to compare the old and new dependencies at every sampled evaluation.
        inputs.sort_unstable_by_key(|(input, _)| *input);
        inputs.dedup_by_key(|(input, _)| *input);
        children.sort_unstable_by_key(|(query, _)| *query);
        children.dedup_by_key(|(query, _)| *query);
        let previous = self.records.borrow().get(&key).cloned();
        let equal = previous.as_ref().is_some_and(|r| r.value == value);
        let changed = if equal {
            previous.as_ref().unwrap().changed
        } else {
            NEXT_CHANGE.fetch_add(1, Ordering::Relaxed)
        };
        let same_edges = previous.as_ref().is_some_and(|r| {
            r.inputs
                .iter()
                .map(|(i, _)| i)
                .eq(inputs.iter().map(|(i, _)| i))
                && r.children
                    .iter()
                    .map(|(q, _)| q)
                    .eq(children.iter().map(|(q, _)| q))
        });
        Arc::make_mut(&mut self.records.borrow_mut()).insert(
            key,
            Arc::new(Record {
                value: value.clone(),
                inputs,
                children,
                changed,
            }),
        );
        if !same_edges {
            self.rewired.borrow_mut().insert(key);
        }
        if direct {
            self.refreshed.borrow_mut().insert(key);
        }
        let mut stats = self.statistics.get();
        stats.executed += 1;
        stats.unchanged += u64::from(equal);
        self.statistics.set(stats);
        self.settled.borrow_mut().insert(key);
        drop(guard);
        Ok(value)
    }

    fn prepare(&self) {
        let mut nodes = self.nodes.borrow_mut();
        let mut pending: Vec<_> = self.rewired.borrow().iter().copied().collect();
        let mut visited = Set::default();
        while let Some(key) = pending.pop() {
            if !visited.insert(key) {
                continue;
            }
            for (child, _) in &self.records.borrow()[&key].children {
                if !nodes.contains_key(child) {
                    pending.push(*child);
                    self.rewired.borrow_mut().insert(*child);
                }
            }
            nodes.entry(key).or_insert_with(|| {
                let node = expert::Node::new(&self.state.weak(), move || {
                    DOCUMENT.with(|document| ACTIVE.with(|live| live.recompute(document, key)))
                });
                node.add_dependency(&self.reconnect_node);
                node
            });
        }
        if !self.rewired.borrow().is_empty() || !self.refreshed.borrow().is_empty() {
            let revision = self.topology_revision.get() + 1;
            let required_height = (nodes.len() + revision as usize + 8).next_power_of_two();
            if required_height > self.height_limit.get() {
                self.state.set_max_height_allowed(required_height);
                self.height_limit.set(required_height);
            }
            self.topology_revision.set(revision);
            self.topology.set(revision);
        }
    }

    fn reconnect(&self, document: &Document) {
        let changed = std::mem::take(&mut *self.rewired.borrow_mut());
        let nodes = self.nodes.borrow();
        let mut edges = self.edges.borrow_mut();
        // Incremental requires dependency removal during stabilisation. This
        // callback is a predecessor of every expert node, so reconnecting here
        // completes before any affected evaluator callback runs.
        for key in &changed {
            if let Some(old) = edges.remove(key) {
                let node = &nodes[key];
                for edge in old.inputs {
                    node.remove_dependency(edge);
                }
                for edge in old.children {
                    node.remove_dependency(edge);
                }
            }
        }
        for key in std::mem::take(&mut *self.refreshed.borrow_mut()) {
            // A pure kernel can refresh an imported memo without ever creating
            // a scheduler node. If it is demanded as a dependency later,
            // prepare() materializes the node from its latest record.
            if let Some(node) = nodes.get(&key) {
                node.make_stale();
            }
        }
        let records = self.records.borrow();
        let mut inputs = self.inputs.borrow_mut();
        for key in changed {
            let record = &records[&key];
            let node = &nodes[&key];
            let input_edges = record
                .inputs
                .iter()
                .map(|(input, _)| {
                    let (_, variable) = inputs.entry(*input).or_insert_with(|| {
                        let stamp = document.sac_input_stamp(input);
                        (stamp.clone(), self.state.var(stamp))
                    });
                    node.add_dependency(&variable.watch())
                })
                .collect();
            let mut seen = Set::default();
            let child_edges = record
                .children
                .iter()
                .filter(|(child, _)| seen.insert(*child))
                .map(|(child, _)| node.add_dependency(&nodes[child].watch()))
                .collect();
            edges.insert(
                key,
                Edges {
                    inputs: input_edges,
                    children: child_edges,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{EvaluationCache, Kind, Point};

    #[test]
    fn refreshed_kernel_memos_can_be_materialized_later() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 1., y: 2. }));
        let b = doc.add(Kind::Free(Point { x: 3., y: 4. }));
        let view = doc.evaluated();
        view.position(a);
        let records = view
            .sac_evaluation()
            .unwrap()
            .graph
            .snapshot
            .lock()
            .unwrap()
            .records
            .clone();
        let live = Live::new(0, false, records, Statistics::default());
        DOCUMENT.set(&view, || {
            ACTIVE.set(&live, || {
                live.evaluate(&view, Query::Point(a), true).unwrap();
                assert!(live.nodes.borrow().is_empty());
                assert_eq!(
                    live.demand(&view, 1, Query::Point(b)).unwrap(),
                    Value::Point(Some(Point { x: 3., y: 4. }))
                );
                assert_eq!(
                    live.demand(&view, 1, Query::Point(a)).unwrap(),
                    Value::Point(Some(Point { x: 1., y: 2. }))
                );
            })
        });
    }

    #[test]
    fn changing_dag_and_query_order_match_recursive_evaluation() {
        let mut source = Document::new();
        for id in 1..=24 {
            source.add(if id <= 4 {
                Kind::Free(Point {
                    x: id as f64,
                    y: 2.,
                })
            } else {
                Kind::Midpoint(id - 1, id / 2)
            });
        }
        let mut view = source.evaluated().clone_for_evaluation();
        let mut random = 0x1234_5678_u64;
        for step in 0..100 {
            random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
            let id = 1 + (random >> 32) as usize % 24;
            let kind = if id <= 4 || step % 3 == 0 {
                Kind::Free(Point {
                    x: step as f64,
                    y: id as f64,
                })
            } else {
                Kind::Midpoint(1 + step % (id - 1), 1 + (step / 3) % (id - 1))
            };
            source.objects.edit(id - 1).kind = kind.clone();
            view.objects.edit(id - 1).kind = kind;
            for offset in 0..24 {
                let query = 1 + (offset + step) % 24;
                assert_eq!(
                    view.position(query),
                    source.position(query),
                    "step {step}, point {query}"
                );
            }
        }
    }

    #[test]
    fn dynamic_edges_reconnect_to_previously_cached_subgraphs() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 2., y: 0. }));
        let b = doc.add(Kind::Free(Point { x: 8., y: 4. }));
        let c = doc.add(Kind::Midpoint(b, b));
        let output = doc.add(Kind::Midpoint(a, a));
        let mut view = doc.evaluated().clone_for_evaluation();
        assert_eq!(view.position(c), Some(Point { x: 8., y: 4. }));
        assert_eq!(view.position(output), Some(Point { x: 2., y: 0. }));
        for step in 0..20 {
            let at = Point {
                x: step as f64,
                y: 7.,
            };
            view.objects.edit(b - 1).kind = Kind::Free(at);
            view.objects.edit(output - 1).kind = Kind::Midpoint(c, c);
            assert_eq!(view.position(output), Some(at));
            assert_eq!(
                view.position(c),
                Some(at),
                "direct evaluation must reach the library value"
            );
            view.objects.edit(output - 1).kind = Kind::Midpoint(a, a);
            assert_eq!(view.position(output), Some(Point { x: 2., y: 0. }));
        }
        assert!(
            view.sac_evaluation()
                .unwrap()
                .graph
                .statistics()
                .stabilisations
                > 0
        );
    }

    #[test]
    fn simultaneous_edge_reversal_matches_a_fresh_view() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 2., y: 1. }));
        let b = doc.add(Kind::Midpoint(a, a));
        let mut view = doc.evaluated().clone_for_evaluation();
        assert_eq!(view.position(b), view.position(a));
        for step in 0..20 {
            let at = Point {
                x: step as f64,
                y: 3.,
            };
            let (source, derived) = if step % 2 == 0 { (b, a) } else { (a, b) };
            view.objects.edit(source - 1).kind = Kind::Free(at);
            view.objects.edit(derived - 1).kind = Kind::Midpoint(source, source);
            assert_eq!(view.position(derived), Some(at));
            assert_eq!(view.position(source), Some(at));
        }
    }

    #[test]
    fn thread_cache_eviction_rehydrates_a_snapshot_with_current_inputs() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 1., y: 2. }));
        let b = doc.add(Kind::Midpoint(a, a));
        let mut cache = EvaluationCache::default();
        let old = cache.evaluate(&doc);
        assert_eq!(old.position(b), Some(Point { x: 1., y: 2. }));
        GRAPHS.with_borrow_mut(|graphs| graphs.clear());
        doc.objects.edit(a - 1).kind = Kind::Free(Point { x: 3., y: 4. });
        assert_eq!(
            cache.evaluate(&doc).position(b),
            Some(Point { x: 3., y: 4. })
        );
        assert_eq!(old.position(b), Some(Point { x: 1., y: 2. }));
    }

    #[test]
    fn a_panicking_library_callback_discards_only_its_live_graph() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 1., y: 2. }));
        let mut view = doc.evaluated().clone_for_evaluation();
        assert_eq!(view.position(a), Some(Point { x: 1., y: 2. }));
        view.objects.edit(a - 1).kind = Kind::Free(Point { x: 3., y: 4. });
        let graph = &view.sac_evaluation().unwrap().graph;
        let live = GRAPHS.with_borrow_mut(|graphs| graphs.get(&graph.family).unwrap().clone());
        let broken = expert::Node::new(&live.state.weak(), || -> Value {
            panic!("test callback");
        });
        live.roots
            .borrow_mut()
            .insert(Query::Point(a), broken.watch().observe());
        *live.epoch.borrow_mut() = None;
        assert!(std::panic::catch_unwind(|| view.position(a)).is_err());
        assert!(!GRAPHS.with_borrow(|graphs| graphs.contains(&graph.family)));
        assert_eq!(view.position(a), Some(Point { x: 3., y: 4. }));
    }
}
