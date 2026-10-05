//! Bridge evaluator reads into the self-adjusting runtime. Observers are thread
//! local; values and traces belong to each immutable evaluated scope.
pub(super) use super::incremental_graph::Graph;
use super::{Document, LineMode, Point};
use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Query {
    Point(usize),
    CurveIntersections(usize, usize),
    Number(usize),
    Straight(usize),
    Circle(usize),
    Polygon(usize),
    Arc(usize),
    Frame(usize),
    Geometry(usize, super::geometry_cache::GeometryQuery, usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Input {
    Validation,
    CoordinateSystems,
    Object(usize),
    Objects,
    Length,
    Point(usize),
    Number(usize),
    Straight(usize),
    Circle(usize),
    Polygon(usize),
    Arc(usize),
    Frame(usize),
    Frozen(usize),
    Scope,
    MappedElementValidity(usize),
}

#[derive(Clone, Debug)]
pub(super) enum Value {
    Point(Option<Point>),
    Number(Option<f64>),
    Straight(Option<(Point, Point, LineMode)>),
    Circle(Option<(Point, f64)>),
    Polygon(Option<Vec<Point>>),
    Arc(Option<(Point, f64, f64, f64, bool)>),
    Frame(Option<crate::graph::CoordinateFrame>),
    Geometry(Option<Geometry>),
}

// Equal results must be interchangeable by every downstream operation. IEEE
// equality is insufficient: atan2 and division can distinguish signed zeros.
trait ExactEq {
    fn exact_eq(&self, other: &Self) -> bool;
}
impl ExactEq for f64 {
    fn exact_eq(&self, other: &Self) -> bool {
        self.to_bits() == other.to_bits()
    }
}
impl<T: ExactEq> ExactEq for Option<T> {
    fn exact_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Some(a), Some(b)) => a.exact_eq(b),
            (None, None) => true,
            _ => false,
        }
    }
}
impl<T: ExactEq> ExactEq for Vec<T> {
    fn exact_eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other).all(|(a, b)| a.exact_eq(b))
    }
}
impl<T: ExactEq> ExactEq for Arc<T> {
    fn exact_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(self, other) || self.as_ref().exact_eq(other.as_ref())
    }
}
macro_rules! exact_fields {
    ($ty:ty; $($field:tt),+) => {
        impl ExactEq for $ty {
            fn exact_eq(&self, other: &Self) -> bool { true $(&& self.$field.exact_eq(&other.$field))+ }
        }
    };
}
macro_rules! exact_tuple {
    ($($ty:ident:$index:tt),+) => {
        impl<$($ty:ExactEq),+> ExactEq for ($($ty,)+) {
            fn exact_eq(&self, other:&Self)->bool { true $(&& self.$index.exact_eq(&other.$index))+ }
        }
    };
}
exact_tuple!(A:0, B:1);
exact_tuple!(A:0, B:1, C:2);
exact_tuple!(A:0, B:1, C:2, D:3, E:4);
exact_fields!(Point; x, y);
exact_fields!(crate::graph::GraphPoint; x, y);
exact_fields!(crate::graph::CoordinateFrame; origin, x_unit, y_unit);
exact_fields!(super::locus::LocusStroke; start, end, mode, style);
exact_fields!(super::locus::LocusSurface; points, style);
macro_rules! exact_plain {
    ($($ty:ty),+) => { $(impl ExactEq for $ty {
        fn exact_eq(&self, other: &Self) -> bool { self == other }
    })+ };
}
exact_plain!(
    bool,
    usize,
    LineMode,
    crate::style_records::VisualStyle,
    crate::style_records::RgbColor,
    Geometry
);
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Point(a), Self::Point(b)) => a.exact_eq(b),
            (Self::Number(a), Self::Number(b)) => a.exact_eq(b),
            (Self::Straight(a), Self::Straight(b)) => a.exact_eq(b),
            (Self::Circle(a), Self::Circle(b)) => a.exact_eq(b),
            (Self::Polygon(a), Self::Polygon(b)) => a.exact_eq(b),
            (Self::Arc(a), Self::Arc(b)) => a.exact_eq(b),
            (Self::Frame(a), Self::Frame(b)) => a.exact_eq(b),
            (Self::Geometry(a), Self::Geometry(b)) => a.exact_eq(b),
            _ => false,
        }
    }
}

#[derive(Clone)]
pub(super) struct Geometry {
    value: Arc<dyn std::any::Any + Send + Sync>,
    equal: fn(&dyn std::any::Any, &dyn std::any::Any) -> bool,
}
impl std::fmt::Debug for Geometry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Geometry(..)")
    }
}
impl PartialEq for Geometry {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.value, &other.value)
            || (self.equal)(self.value.as_ref(), other.value.as_ref())
    }
}
impl Geometry {
    fn new<T: ExactEq + Send + Sync + 'static>(value: T) -> Self {
        Self {
            value: Arc::new(value),
            equal: |a, b| {
                a.downcast_ref::<T>()
                    .zip(b.downcast_ref::<T>())
                    .is_some_and(|(a, b)| a.exact_eq(b))
            },
        }
    }
    pub(super) fn get<T: Clone + 'static>(&self) -> Option<T> {
        self.value.downcast_ref::<T>().cloned()
    }
}

#[derive(Clone, Debug)]
pub(super) struct Identity(pub Arc<()>);
impl PartialEq for Identity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Stamp {
    Identity(Identity),
    Validity(bool),
    Length(usize),
    Slot(Option<Value>),
    Revision(u64),
    Frozen(Option<(Identity, Option<(u64, u64)>)>),
}

static NEXT_SCOPE: AtomicU64 = AtomicU64::new(1);

/// Names describe a call site and its logical sample, never current coordinates.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum Scope {
    Mapping(usize),
    Iteration {
        owner: usize,
        step: u32,
        branch: Vec<usize>,
    },
    Sample {
        owner: usize,
        index: usize,
        count: usize,
    },
    MappedSample {
        owner: usize,
        fraction: u64,
    },
    CurveRefinement {
        owner: usize,
        fraction: u64,
    },
    ImageSample {
        owner: usize,
        column: usize,
        row: usize,
        columns: usize,
        rows: usize,
    },
}
type ScopePath = Arc<Vec<Scope>>;

#[derive(Debug)]
struct Scopes {
    entries: lru::LruCache<ScopePath, Arc<Graph>>,
    nodes: usize,
    families: lru::LruCache<ScopePath, u64>,
}
impl Default for Scopes {
    fn default() -> Self {
        Self {
            entries: lru::LruCache::sparse(8192.try_into().unwrap()),
            nodes: 0,
            families: lru::LruCache::sparse(8192.try_into().unwrap()),
        }
    }
}
impl Scopes {
    fn publish(&mut self, path: ScopePath, graph: Graph) {
        self.nodes += graph.len();
        if let Some((_, old)) = self.entries.push(path, Arc::new(graph)) {
            self.nodes -= old.len();
        }
        // Bound retained computation records across all nested scopes, rather
        // than allowing an independent unbounded subtree for each sample.
        while self.nodes > 200_000 {
            if let Some((_, old)) = self.entries.pop_lru() {
                self.nodes -= old.len();
            } else {
                break;
            }
        }
    }
}

#[derive(Debug)]
pub(super) struct Evaluation {
    id: u64,
    pub(super) graph: Graph,
    path: ScopePath,
    scopes: Arc<Mutex<Scopes>>,
    transient: bool,
}
impl Default for Evaluation {
    fn default() -> Self {
        Self {
            id: NEXT_SCOPE.fetch_add(1, Ordering::Relaxed),
            graph: Graph::new(65536.try_into().unwrap()),
            path: Arc::new(Vec::new()),
            scopes: Arc::new(Mutex::new(Scopes::default())),
            transient: false,
        }
    }
}
impl Clone for Evaluation {
    fn clone(&self) -> Self {
        Self {
            id: NEXT_SCOPE.fetch_add(1, Ordering::Relaxed),
            graph: self.graph.clone(),
            path: self.path.clone(),
            scopes: self.scopes.clone(),
            transient: self.transient,
        }
    }
}

impl Evaluation {
    pub(super) fn input_revision(&self) -> (u64, u64) {
        (self.id, self.graph.revision())
    }

    pub(super) fn child(&self, name: Scope, fallback: Graph) -> Self {
        // Sequential samples discard their arithmetic after each result.
        // Root finding revisits fractions, so retain its intermediate traces.
        let transient = !matches!(name, Scope::CurveRefinement { .. });
        let coalesce = matches!(name, Scope::Iteration { .. } | Scope::Mapping(_));
        let mut path = (*self.path).clone();
        path.push(name);
        let path = Arc::new(path);
        let mut execution_path = (*path).clone();
        for scope in &mut execution_path {
            match scope {
                Scope::Sample { index, .. } => *index = 0,
                Scope::Iteration { step, .. } => *step = 0,
                Scope::MappedSample { fraction, .. } | Scope::CurveRefinement { fraction, .. } => {
                    *fraction = 0
                }
                Scope::ImageSample { column, row, .. } => {
                    *column = 0;
                    *row = 0;
                }
                _ => {}
            }
        }
        let execution_path = Arc::new(execution_path);
        let (previous, family) = {
            let mut scopes = self.scopes.lock().unwrap();
            let previous = scopes.entries.get(&path).cloned();
            let family = if let Some(family) = scopes.families.get(&execution_path) {
                *family
            } else {
                let family = Graph::new_family();
                scopes.families.put(execution_path, family);
                family
            };
            (previous, family)
        };
        let mut graph = if let Some(previous) = previous {
            (*previous).clone()
        } else {
            let mut graph = fallback;
            graph.detach_execution();
            graph
        };
        // Sibling samples run the same construction with different inputs.
        // Reuse their scheduler while keeping named result snapshots separate.
        graph.use_family(family);
        // The same logical scope may now have different seeds or frozen inputs.
        graph.advance();
        graph.coalesce_nested_queries(coalesce);
        Self {
            id: NEXT_SCOPE.fetch_add(1, Ordering::Relaxed),
            graph,
            path,
            scopes: self.scopes.clone(),
            transient,
        }
    }
    pub(super) fn publish(&self) {
        if self.graph.len() == 0 {
            return;
        }
        let snapshot = self.graph.snapshot_current();
        self.scopes
            .lock()
            .unwrap()
            .publish(self.path.clone(), snapshot);
    }

    fn transient_query(&self, doc: &Document, key: Query) -> Value {
        if KERNEL.is_set() && KERNEL.with(|kernel| kernel.matches(doc)) {
            return KERNEL.with(|kernel| kernel.query(doc, key));
        }
        // Retain precise reads without storing versioned arithmetic records.
        let _recording = Recording::transient(self.id);
        let kernel = Kernel::new(doc);
        KERNEL.set(&kernel, || kernel.query(doc, key))
    }
}

pub(super) fn evaluate_scope<T>(
    sample: &mut Document,
    evaluate: impl FnOnce(&mut Document) -> T,
) -> T {
    let scope = sample.sac_evaluation().unwrap();
    if !scope.transient {
        return evaluate(sample);
    }
    let _recording = Recording::transient(scope.id);
    let kernel = Kernel::new(sample);
    KERNEL.set(&kernel, || evaluate(sample))
}

scoped_tls::scoped_thread_local!(static KERNEL: Kernel);
type KernelMap = FxHashMap<Query, Option<Value>>;
thread_local! {
    static KERNEL_MAPS: RefCell<Vec<KernelMap>> = const { RefCell::new(Vec::new()) };
}
struct Kernel {
    document: *const Document,
    recording_depth: usize,
    running: Cell<usize>,
    epoch: RefCell<Option<((u64, u64), Identity)>>,
    values: RefCell<KernelMap>,
}
impl Kernel {
    fn new(doc: &Document) -> Self {
        Self {
            document: doc,
            recording_depth: READS.with_borrow(Vec::len),
            running: Cell::new(0),
            epoch: RefCell::default(),
            values: RefCell::new(
                KERNEL_MAPS.with_borrow_mut(|maps| maps.pop().unwrap_or_default()),
            ),
        }
    }
    fn matches(&self, doc: &Document) -> bool {
        std::ptr::eq(self.document, doc)
            && READS.with_borrow(|frames| {
                frames.len() == self.recording_depth
                    && frames.last().is_some_and(|frame| frame.transient)
            })
    }
    fn query(&self, doc: &Document, key: Query) -> Value {
        if self.running.get() == 0 {
            let epoch = (
                doc.sac_evaluation().unwrap().input_revision(),
                doc.objects.sac_identity(None),
            );
            if self.epoch.borrow().as_ref() != Some(&epoch) {
                self.values.borrow_mut().clear();
                *self.epoch.borrow_mut() = Some(epoch);
            }
        }
        if let Some(value) = self.values.borrow().get(&key) {
            return value.clone().unwrap_or_else(|| {
                observe(Input::Objects);
                observe(Input::Scope);
                key.undefined()
            });
        }
        self.values.borrow_mut().insert(key, None);
        struct Running<'a>(&'a Cell<usize>);
        impl Drop for Running<'_> {
            fn drop(&mut self) {
                self.0.set(self.0.get() - 1);
            }
        }
        self.running.set(self.running.get() + 1);
        let _running = Running(&self.running);
        let value = doc.sac_evaluate_body(key);
        self.values.borrow_mut().insert(key, Some(value.clone()));
        value
    }
}

impl Drop for Kernel {
    fn drop(&mut self) {
        let mut values = self.values.take();
        if values.capacity() <= 8192 {
            values.clear();
            KERNEL_MAPS.with_borrow_mut(|maps| {
                if maps.len() < 32 {
                    maps.push(values);
                }
            });
        }
    }
}

impl Query {
    fn undefined(self) -> Value {
        match self {
            Self::Point(_) => Value::Point(None),
            Self::Number(_) => Value::Number(None),
            Self::Straight(_) => Value::Straight(None),
            Self::Circle(_) => Value::Circle(None),
            Self::Polygon(_) => Value::Polygon(None),
            Self::Arc(_) => Value::Arc(None),
            Self::Frame(_) => Value::Frame(None),
            Self::Geometry(..) | Self::CurveIntersections(..) => Value::Geometry(None),
        }
    }
}

struct Reads {
    scope: u64,
    transient: bool,
    inputs: InputSet,
}

/// Object IDs are dense in ordinary sketches. Record repeated reads with a bit
/// per input kind, while retaining a compact list for stamping and clearing.
/// Sparse or invalid IDs use a hash set without allocating up to that ID.
#[derive(Default)]
struct InputSet {
    masks: Vec<u16>,
    values: Vec<Input>,
    sparse: FxHashSet<Input>,
}

impl Input {
    fn bit(self) -> (usize, u16) {
        match self {
            Self::Validation => (0, 1),
            Self::CoordinateSystems => (0, 1 << 1),
            Self::Object(id) => (id, 1 << 2),
            Self::Objects => (0, 1 << 3),
            Self::Length => (0, 1 << 4),
            Self::Point(id) => (id, 1 << 5),
            Self::Number(id) => (id, 1 << 6),
            Self::Straight(id) => (id, 1 << 7),
            Self::Circle(id) => (id, 1 << 8),
            Self::Polygon(id) => (id, 1 << 9),
            Self::Arc(id) => (id, 1 << 10),
            Self::Frame(id) => (id, 1 << 11),
            Self::Frozen(id) => (id, 1 << 12),
            Self::Scope => (0, 1 << 13),
            Self::MappedElementValidity(id) => (id, 1 << 14),
        }
    }
}

impl InputSet {
    fn insert(&mut self, input: Input) {
        let (index, bit) = input.bit();
        let fresh = if index < 16384 {
            if index >= self.masks.len() {
                self.masks.resize(index + 1, 0);
            }
            let fresh = self.masks[index] & bit == 0;
            self.masks[index] |= bit;
            fresh
        } else {
            self.sparse.insert(input)
        };
        if fresh {
            self.values.push(input);
        }
    }

    fn clear(&mut self) {
        for input in self.values.drain(..) {
            let (index, bit) = input.bit();
            if let Some(mask) = self.masks.get_mut(index) {
                *mask &= !bit;
            }
        }
        self.sparse.clear();
    }
}
thread_local! {
    static READS: RefCell<Vec<Reads>> = const { RefCell::new(Vec::new()) };
    // Reuse recording allocations across the many short-lived sample scopes.
    static INPUT_SETS: RefCell<Vec<InputSet>> = const { RefCell::new(Vec::new()) };
    static STRUCTURAL: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Transform reference validation reads only IDs, pages, unsupported/deleted
/// flags and edges. ObjectStore gives these a separate revision from coordinates.
pub(super) fn structural_validation<T>(validate: impl FnOnce() -> T) -> T {
    observe(Input::Validation);
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            STRUCTURAL.set(STRUCTURAL.get() - 1);
        }
    }
    STRUCTURAL.set(STRUCTURAL.get() + 1);
    let _guard = Guard;
    validate()
}

pub(super) fn observe(input: Input) {
    if STRUCTURAL.get() != 0 {
        return;
    }
    READS.with_borrow_mut(|frames| {
        if let Some(frame) = frames.last_mut() {
            frame.inputs.insert(input);
        }
    });
}

struct Recording;
impl Recording {
    fn new(scope: u64) -> Self {
        Self::with_mode(scope, false)
    }
    fn transient(scope: u64) -> Self {
        Self::with_mode(scope, true)
    }
    fn with_mode(scope: u64, transient: bool) -> Self {
        let inputs = INPUT_SETS.with_borrow_mut(|sets| sets.pop().unwrap_or_default());
        READS.with_borrow_mut(|frames| {
            frames.push(Reads {
                scope,
                transient,
                inputs,
            })
        });
        Self
    }
    fn record_inputs(&self, graph: &Graph, document: &Document) {
        // Document::input_stamp reads raw slots and never invokes observation.
        READS.with_borrow(|frames| {
            graph.read_unique_inputs(document, &frames.last().unwrap().inputs.values);
        });
    }
}
impl Drop for Recording {
    fn drop(&mut self) {
        let mut inputs = READS.with_borrow_mut(|frames| {
            let frame = frames.pop().unwrap();
            if frame.transient
                && let Some(parent) = frames.last_mut()
            {
                for &input in &frame.inputs.values {
                    parent.inputs.insert(input);
                }
            }
            frame.inputs
        });
        if inputs.values.capacity() <= 8192 && inputs.sparse.capacity() <= 8192 {
            inputs.clear();
            INPUT_SETS.with_borrow_mut(|sets| {
                if sets.len() < 64 {
                    sets.push(inputs);
                }
            });
        }
    }
}

impl Document {
    pub(super) fn sac_can_inline(&self, key: &Query) -> bool {
        !matches!(key, Query::Geometry(..) | Query::CurveIntersections(..))
            && READS.with_borrow(|frames| {
                frames.last().is_some_and(|frame| {
                    self.sac_evaluation()
                        .is_some_and(|scope| frame.scope == scope.id)
                })
            })
    }
    pub(super) fn sac_input_stamp(&self, input: &Input) -> Stamp {
        // Verification must not manufacture new dependencies.
        match *input {
            Input::MappedElementValidity(id) => {
                Stamp::Validity(self.objects.mapped_element_validity(id))
            }
            Input::CoordinateSystems => Stamp::Identity(self.objects.sac_coordinate_identity()),
            Input::Validation => Stamp::Identity(self.objects.sac_validation_identity()),
            Input::Object(slot) => Stamp::Identity(self.objects.sac_identity(Some(slot))),
            Input::Objects => Stamp::Identity(self.objects.sac_identity(None)),
            Input::Length => Stamp::Length(self.objects.untracked_len()),
            Input::Scope => Stamp::Revision(self.sac_evaluation().unwrap().graph.revision()),
            Input::Frozen(id) => self.sac_frozen_stamp(id),
            input => Stamp::Slot(self.sac_slot(input)),
        }
    }
    pub(super) fn sac_execute_query(&self, key: &Query) -> Value {
        let scope = self.sac_evaluation().unwrap();
        let recording = Recording::new(scope.id);
        let value = self.sac_evaluate_body(*key);
        recording.record_inputs(&scope.graph, self);
        value
    }
}

impl Document {
    pub(super) fn sac_evaluate_body(&self, key: Query) -> Value {
        match key {
            Query::Point(id) => Value::Point(self.position_untracked(id)),
            Query::CurveIntersections(carrier, curve) => Value::Geometry(
                self.curve_intersections_uncached(carrier, curve)
                    .map(|points| Geometry::new(Arc::new(points))),
            ),
            Query::Number(id) => Value::Number(self.numeric_value_untracked(id)),
            Query::Straight(id) => Value::Straight(self.line_points_untracked(id)),
            Query::Circle(id) => Value::Circle(self.circle_points_untracked(id)),
            Query::Polygon(id) => Value::Polygon(self.polygon_points_untracked(id)),
            Query::Arc(id) => Value::Arc(self.arc_geometry_untracked(id)),
            Query::Frame(id) => Value::Frame(self.graph_frame_untracked(id)),
            Query::Geometry(id, kind, resolution) => {
                use super::geometry_cache::GeometryQuery as G;
                Value::Geometry(match kind {
                    G::CurveSamples => super::point_curve::PointCurve::new(self, id)
                        .map(|mut curve| Geometry::new(curve.sample_points_uncached(resolution))),
                    G::Points => self.point_iteration_images_uncached(id).map(Geometry::new),
                    G::Polygons => self
                        .polygon_iteration_images_uncached(id)
                        .map(Geometry::new),
                    G::ColorizedPolygons => self
                        .colorized_polygon_iteration_images_uncached(id)
                        .map(Geometry::new),
                    G::ColorizedPolygonBatch => self
                        .colorized_polygon_iteration_batch_uncached(id)
                        .map(Geometry::new),
                    G::Disks => self
                        .circle_interior_iteration_images_uncached(id)
                        .map(Geometry::new),
                    G::ColorizedDisks => self
                        .colorized_disk_iteration_images_uncached(id)
                        .map(Geometry::new),
                    G::BranchSegments => self
                        .branch_segment_iteration_images_uncached(id)
                        .map(Geometry::new),
                    G::Lines => self.line_iteration_images_uncached(id).map(Geometry::new),
                    G::Segments => self
                        .segment_iteration_images_uncached(id)
                        .map(Geometry::new),
                    G::Rays => self.ray_iteration_images_uncached(id).map(Geometry::new),
                    G::Arcs => self
                        .arc_iteration_images_uncached(id, resolution)
                        .map(Geometry::new),
                    G::Circles => self.circle_iteration_images_uncached(id).map(Geometry::new),
                    G::MappedStrokes => self
                        .mapped_collection_strokes_uncached(id, resolution)
                        .map(Geometry::new),
                    G::MappedSurfaces => self
                        .mapped_collection_surfaces_uncached(id, resolution)
                        .map(Geometry::new),
                    G::LocusSurfaces => self
                        .locus_surfaces_uncached(id, resolution)
                        .map(Geometry::new),
                    G::SampledStrokes => self
                        .sampled_mapped_styled_strokes_uncached(id, resolution)
                        .map(Geometry::new),
                    G::LocusStrokes => self
                        .locus_styled_strokes_untracked(id, resolution)
                        .map(Geometry::new),
                })
            }
        }
    }

    pub(super) fn sac_query(&self, key: Query) -> Option<Value> {
        let scope = self.sac_evaluation()?;
        if scope.transient
            && !matches!(key, Query::Geometry(..))
            && READS.with_borrow(|frames| {
                frames
                    .last()
                    .is_none_or(|frame| frame.scope != scope.id || frame.transient)
            })
        {
            return Some(scope.transient_query(self, key));
        }
        let result = scope.graph.query(self, key).unwrap_or_else(|_| {
            // A cycle has no published child edge. Keep its undefined fallback
            // sensitive to both structural edits and temporary seed changes.
            observe(Input::Objects);
            observe(Input::Scope);
            key.undefined()
        });

        // A transient caller has no retained graph edge, even when it shares
        // this scope. Both cold queries and memo hits must expose their reads.
        let forward = READS.with_borrow(|frames| {
            frames
                .last()
                .is_some_and(|frame| frame.scope != scope.id || frame.transient)
        });
        if forward {
            READS.with_borrow_mut(|frames| {
                if let Some(frame) = frames.last_mut()
                    && !scope.graph.visit_input_footprint(&key, |input| {
                        frame.inputs.insert(*input);
                    })
                {
                    frame.inputs.insert(Input::Scope);
                }
            });
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{EvaluationCache, Kind};

    #[test]
    fn transient_callers_track_retained_geometry_on_misses_and_hits_in_the_same_scope() {
        let mut doc = Document::new();
        let seed = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let fixed = doc.add(Kind::Free(Point { x: 8., y: 2. }));
        let middle = doc.add(Kind::Midpoint(seed, fixed));
        let iteration = doc.add(Kind::MappedPointIteration {
            seeds: vec![seed],
            branches: vec![vec![middle]],
            carried: vec![middle],
            depth: crate::document::IterationDepth::Fixed(2),
            final_only: false,
        });
        let binding = doc.add(Kind::IterationBinding(middle, iteration));
        let parent = doc.evaluated();
        let mut sample = parent.clone_for_evaluation();
        parent.with_sac_scope(
            &mut sample,
            Scope::Sample {
                owner: binding,
                index: 0,
                count: 2,
            },
            |sample| {
                let mut previous = None;
                for x in [8., 12.] {
                    sample.objects.edit(fixed - 1).kind = Kind::Free(Point { x, y: 2. });
                    for _ in 0..2 {
                        let _recording = Recording::transient(sample.sac_evaluation().unwrap().id);
                        let points = sample.point_iteration_images(binding).unwrap();
                        assert_eq!(
                            points,
                            vec![
                                Point {
                                    x: x * 0.75,
                                    y: 1.5
                                },
                                Point {
                                    x: x * 0.875,
                                    y: 1.75
                                }
                            ]
                        );
                        if x == 12. {
                            assert_ne!(Some(&points), previous.as_ref());
                        }
                        READS.with_borrow(|frames| {
                            assert!(
                                frames
                                    .last()
                                    .unwrap()
                                    .inputs
                                    .values
                                    .contains(&Input::Object(fixed - 1))
                            );
                        });
                        if x == 8. {
                            previous = Some(points);
                        }
                    }
                }
            },
        );
    }

    #[test]
    fn transient_kernels_forward_reads_and_do_not_reuse_changed_seeds() {
        let mut doc = Document::new();
        let seed = doc.add(Kind::Free(Point { x: 2., y: 4. }));
        let middle = doc.add(Kind::Midpoint(seed, seed));
        let parent = doc.evaluated();
        let mut sample = parent.clone_for_evaluation();
        parent.with_sac_scope(&mut sample, Scope::Mapping(middle), |sample| {
            for x in [4., 8., -2.] {
                sample
                    .set_evaluation_point(seed, Point { x, y: 4. })
                    .unwrap();
                let _recording = Recording::new(parent.sac_evaluation().unwrap().id);
                assert_eq!(sample.position(middle), Some(Point { x, y: 4. }));
                READS.with_borrow(|frames| {
                    let inputs = &frames.last().unwrap().inputs.values;
                    assert!(inputs.contains(&Input::Point(seed)));
                    assert!(inputs.contains(&Input::Object(middle - 1)));
                    assert!(!inputs.contains(&Input::Objects));
                });
            }
            let old = sample.clone_for_evaluation();
            sample.objects.edit(middle - 1).deleted = true;
            assert_eq!(sample.position(middle), None);
            assert_eq!(old.position(middle), Some(Point { x: -2., y: 4. }));
        });
    }

    #[test]
    fn transient_cycles_and_unwinding_do_not_poison_later_queries() {
        let mut doc = Document::new();
        let point = doc.add(Kind::Free(Point { x: 2., y: 4. }));
        let parent = doc.evaluated();
        let mut sample = parent.clone_for_evaluation();
        parent.with_sac_scope(&mut sample, Scope::Mapping(point), |sample| {
            sample.objects.edit(point - 1).kind = Kind::Midpoint(point, point);
            assert_eq!(sample.position(point), None);
            sample.objects.edit(point - 1).kind = Kind::Free(Point { x: 6., y: 8. });
            assert_eq!(sample.position(point), Some(Point { x: 6., y: 8. }));
            let outer_kernel = KERNEL.with(|kernel| kernel as *const Kernel);
            let outcome = std::panic::catch_unwind(|| {
                let _recording = Recording::transient(sample.sac_evaluation().unwrap().id);
                let kernel = Kernel::new(sample);
                KERNEL.set(&kernel, || {
                    kernel.values.borrow_mut().insert(Query::Point(point), None);
                    panic!("interrupted kernel");
                });
            });
            assert!(outcome.is_err());
            assert!(KERNEL.with(|kernel| std::ptr::eq(kernel, outer_kernel)));
            assert_eq!(sample.position(point), Some(Point { x: 6., y: 8. }));
        });
    }

    #[test]
    fn dense_input_sets_preserve_kinds_sparse_ids_and_reuse() {
        let mut inputs = vec![
            Input::Validation,
            Input::CoordinateSystems,
            Input::Objects,
            Input::Length,
            Input::Scope,
        ];
        for id in [0, 1, 99, 16383, 16384, usize::MAX] {
            inputs.extend([
                Input::Object(id),
                Input::Point(id),
                Input::Number(id),
                Input::Straight(id),
                Input::Circle(id),
                Input::Polygon(id),
                Input::Arc(id),
                Input::Frame(id),
                Input::Frozen(id),
                Input::MappedElementValidity(id),
            ]);
        }
        let mut set = InputSet::default();
        for _ in 0..3 {
            for &input in inputs.iter().chain(inputs.iter().rev()) {
                set.insert(input);
            }
            assert_eq!(set.values, inputs);
            assert!(set.masks.len() <= 16384);
            set.clear();
            assert!(set.values.is_empty() && set.sparse.is_empty());
            assert!(set.masks.iter().all(|mask| *mask == 0));
        }
    }

    #[test]
    fn rejected_geometry_keeps_the_parent_read_dependency() {
        let mut doc = Document::new();
        let id = doc.add(Kind::Free(Point { x: 1., y: 2. }));
        let mut view = doc.evaluated().clone_for_evaluation();
        let input = Input::Object(id - 1);
        let before = view.sac_input_stamp(&input);
        {
            let _parent = Recording::new(view.sac_evaluation().unwrap().id);
            assert!(view.point_iteration_images(id).is_none());
            READS.with_borrow(|frames| {
                assert!(frames.last().unwrap().inputs.values.contains(&input))
            });
        }
        view.objects.edit(id - 1).kind =
            Kind::PointIteration(id, id, crate::document::IterationDepth::Value(1));
        assert_ne!(view.sac_input_stamp(&input), before);
    }

    #[test]
    fn shared_mapping_validation_tracks_edits_and_preserves_old_snapshots() {
        let mut doc = Document::new();
        let seed = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let fixed = doc.add(Kind::Free(Point { x: 10., y: 0. }));
        let segment = doc.add(Kind::Segment(seed, fixed));
        let output = doc.add(Kind::SegmentMidpoint(segment));
        let replacement = doc.add(Kind::Free(Point { x: 30., y: 20. }));
        let mapped = doc
            .add_mapped_element(output, seed, replacement, &[segment, output])
            .unwrap();
        let mut cache = EvaluationCache::default();
        let old = cache.evaluate(&doc);
        assert_eq!(old.position(mapped), Some(Point { x: 20., y: 10. }));
        doc.objects.edit(fixed - 1).deleted = true;
        assert_eq!(cache.evaluate(&doc).position(mapped), None);
        doc.objects.edit(fixed - 1).deleted = false;
        assert_eq!(cache.evaluate(&doc).position(mapped), old.position(mapped));
        doc.move_point(replacement, Point { x: 50., y: 20. });
        assert_eq!(
            cache.evaluate(&doc).position(mapped),
            Some(Point { x: 30., y: 10. })
        );
        assert_eq!(old.position(mapped), Some(Point { x: 20., y: 10. }));
        doc.objects.edit(replacement - 1).kind = Kind::Segment(seed, fixed);
        assert_eq!(cache.evaluate(&doc).position(mapped), None);
        assert_eq!(doc.evaluated().position(mapped), None);
    }

    #[test]
    fn recycled_recordings_do_not_leak_inputs_after_unwinding() {
        assert!(
            std::panic::catch_unwind(|| {
                let _outer = Recording::new(1);
                observe(Input::Point(12));
                let _inner = Recording::new(2);
                observe(Input::Object(34));
                panic!("interrupted recording");
            })
            .is_err()
        );
        let _recording = Recording::new(3);
        observe(Input::Number(56));
        READS.with_borrow(|frames| {
            assert_eq!(frames.len(), 1);
            assert_eq!(frames[0].inputs.values, [Input::Number(56)]);
        });
    }

    #[test]
    fn demand_reads_skip_unrelated_edits_and_cut_off_equal_intermediates() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = doc.add(Kind::Free(Point { x: 10., y: 10. }));
        let c = doc.add(Kind::Free(Point { x: 20., y: 20. }));
        let unrelated = doc.add(Kind::Free(Point { x: 99., y: 99. }));
        let middle = doc.add(Kind::Midpoint(a, b));
        let output = doc.add(Kind::Midpoint(middle, c));
        let mut cache = EvaluationCache::default();
        let before = cache.evaluate(&doc);
        assert_eq!(
            before.sac_evaluation().unwrap().graph.statistics().executed,
            0
        );
        let expected = before.position(output);
        assert_eq!(expected, Some(Point { x: 12.5, y: 12.5 }));
        doc.objects.edit(unrelated - 1).kind = Kind::Free(Point { x: 100., y: 100. });
        let independent = cache.evaluate(&doc);
        assert_eq!(independent.position(output), expected);
        assert_eq!(
            independent
                .sac_evaluation()
                .unwrap()
                .graph
                .statistics()
                .executed,
            0
        );
        doc.objects.edit(a - 1).kind = Kind::Free(Point { x: 2., y: 0. });
        doc.objects.edit(b - 1).kind = Kind::Free(Point { x: 8., y: 10. });
        let cancelled = cache.evaluate(&doc);
        assert_eq!(cancelled.position(output), expected);
        let stats = cancelled.sac_evaluation().unwrap().graph.statistics();
        assert_eq!(
            stats.executed, 3,
            "only the two inputs and their midpoint execute: {stats:?}"
        );
        assert_eq!(stats.unchanged, 1);
        assert_eq!(before.position(a), Some(Point { x: 0., y: 0. }));
    }

    #[test]
    fn coordinate_lookup_ignores_unrelated_values_and_tracks_candidate_changes() {
        let mut doc = Document::new();
        let origin = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let horizontal = doc.add(Kind::GraphHorizontalUnit(origin, 10.));
        let vertical = doc.add(Kind::GraphVerticalUnit(horizontal, 20.));
        let x_axis = doc.add(Kind::GraphAxis(origin, horizontal, 1));
        let y_axis = doc.add(Kind::GraphAxis(origin, vertical, 0));
        let unrelated = doc.add(Kind::Free(Point { x: 50., y: 50. }));
        doc.grid_form[0] = crate::graph::GridForm::Rectangular;
        let mut view = doc.evaluated().clone_for_evaluation();
        assert_eq!(view.position(vertical).unwrap().y, -20.);
        let graph = &view.sac_evaluation().unwrap().graph;
        let reads = graph.input_footprint(&Query::Point(vertical)).unwrap();
        assert!(reads.contains(&Input::CoordinateSystems));
        assert!(!reads.contains(&Input::Object(unrelated - 1)));
        let executed = graph.statistics().executed;
        view.objects.edit(unrelated - 1).kind = Kind::Free(Point { x: 80., y: 50. });
        assert_eq!(view.position(vertical).unwrap().y, -20.);
        assert_eq!(
            view.sac_evaluation().unwrap().graph.statistics().executed,
            executed
        );

        // Append, delete, restore, change page and replace a coordinate-system
        // kind in the same evaluated scope: each must invalidate the lookup.
        let frame = view.add(Kind::CoordinateSystem(x_axis, y_axis));
        view.frame_styles
            .insert(frame, (crate::graph::GridForm::Square, false));
        assert_eq!(view.position(vertical).unwrap().y, -10.);
        let old = view.clone_for_evaluation();
        view.objects.edit(frame - 1).deleted = true;
        assert_eq!(view.position(vertical).unwrap().y, -20.);
        view.objects.edit(frame - 1).deleted = false;
        assert_eq!(view.position(vertical).unwrap().y, -10.);
        view.objects.edit(frame - 1).page = 1;
        assert_eq!(view.position(vertical).unwrap().y, -20.);
        view.objects.edit(frame - 1).page = 0;
        assert_eq!(view.position(vertical).unwrap().y, -10.);
        view.objects.edit(frame - 1).kind = Kind::Midpoint(origin, horizontal);
        assert_eq!(view.position(vertical).unwrap().y, -20.);
        view.objects.edit(frame - 1).kind = Kind::CoordinateSystem(x_axis, y_axis);
        assert_eq!(view.position(vertical).unwrap().y, -10.);
        assert_eq!(old.position(vertical).unwrap().y, -10.);
    }

    #[test]
    fn locus_cache_survives_unrelated_edits_but_updates_after_driver_edits() {
        use super::super::{EvaluationCache, geometry_cache::GeometryQuery};
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/locus_midpoint_original.gsp");
        let mut doc = Document::open(&path).unwrap();
        let locus = doc
            .objects
            .iter()
            .find(|object| matches!(object.kind, Kind::Locus(..)))
            .unwrap()
            .id;
        let unrelated = doc.add(Kind::Free(Point { x: 123., y: 456. }));
        let mut cache = EvaluationCache::default();
        let old = cache.evaluate(&doc);
        let geometry = old.locus_styled_strokes(locus, 32).unwrap();
        assert!(!geometry.is_empty());
        let reads = old
            .sac_evaluation()
            .unwrap()
            .graph
            .input_footprint(&Query::Geometry(locus, GeometryQuery::LocusStrokes, 32))
            .unwrap();
        assert!(!reads.contains(&Input::Objects));
        assert!(!reads.contains(&Input::Object(unrelated - 1)));
        doc.move_point(unrelated, Point { x: 456., y: 123. });
        let updated = cache.evaluate(&doc);
        assert_eq!(updated.locus_styled_strokes(locus, 32).unwrap(), geometry);
        assert_eq!(
            updated
                .sac_evaluation()
                .unwrap()
                .graph
                .statistics()
                .executed,
            0
        );
        let driver = doc
            .objects
            .iter()
            .find_map(|object| match object.kind {
                Kind::Free(point) if object.id != unrelated => Some((object.id, point)),
                _ => None,
            })
            .unwrap();
        doc.move_point(
            driver.0,
            Point {
                x: driver.1.x + 30.,
                y: driver.1.y,
            },
        );
        let changed = cache
            .evaluate(&doc)
            .locus_styled_strokes(locus, 32)
            .unwrap();
        assert_ne!(changed, geometry);
        assert_eq!(
            changed,
            doc.evaluated().locus_styled_strokes(locus, 32).unwrap()
        );
        assert_eq!(old.locus_styled_strokes(locus, 32).unwrap(), geometry);
    }

    #[test]
    fn stable_samples_reuse_traces_and_keep_substitutions_isolated() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = doc.add(Kind::Free(Point { x: 10., y: 10. }));
        let output = doc.add(Kind::Midpoint(a, b));
        let view = doc.evaluated();
        let mut sample = view.clone_for_evaluation();
        // Refinement revisits parameters and retains its arithmetic traces.
        for (index, x, executed) in [(0, 2., 3), (1, 4., 2), (0, 2., 0), (0, 6., 2)] {
            view.with_sac_scope(
                &mut sample,
                Scope::CurveRefinement {
                    owner: output,
                    fraction: index as u64,
                },
                |sample| {
                    sample.set_evaluation_point(a, Point { x, y: 0. }).unwrap();
                    let before = sample.sac_evaluation().unwrap().graph.statistics().executed;
                    assert_eq!(
                        sample.position(output),
                        Some(Point {
                            x: (x + 10.) / 2.,
                            y: 5.
                        })
                    );
                    let after = sample.sac_evaluation().unwrap().graph.statistics().executed;
                    assert_eq!(after - before, executed, "sample {index}, seed {x}");
                    assert_eq!(
                        sample.position(output),
                        Some(Point {
                            x: (x + 10.) / 2.,
                            y: 5.
                        })
                    );
                    assert_eq!(
                        sample.sac_evaluation().unwrap().graph.statistics().executed,
                        after
                    );
                },
            );
        }
        assert_eq!(view.position(output), Some(Point { x: 5., y: 5. }));
    }

    #[test]
    fn signed_zero_is_an_observable_change_in_values_and_slots() {
        assert_ne!(Value::Number(Some(0.)), Value::Number(Some(-0.)));
        let a = Geometry::new(vec![Point { x: 0., y: 1. }]);
        let b = Geometry::new(vec![Point { x: -0., y: 1. }]);
        assert_ne!(a, b);
    }

    #[test]
    fn frozen_inputs_distinguish_parent_substitutions_over_the_same_object_table() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let mut parent = doc.evaluated().clone_for_evaluation();
        parent
            .set_evaluation_point(a, Point { x: 2., y: 0. })
            .unwrap();
        let mut first = parent.clone_for_evaluation();
        first
            .freeze_scoped_evaluation_inputs(&parent, &[a])
            .unwrap();
        let stamp = first.sac_frozen_stamp(a);
        parent
            .set_evaluation_point(a, Point { x: 4., y: 0. })
            .unwrap();
        let mut second = parent.clone_for_evaluation();
        second
            .freeze_scoped_evaluation_inputs(&parent, &[a])
            .unwrap();
        assert_ne!(stamp, second.sac_frozen_stamp(a));
        assert_eq!(first.position(a), Some(Point { x: 2., y: 0. }));
        assert_eq!(second.position(a), Some(Point { x: 4., y: 0. }));
    }

    #[test]
    fn object_edit_guards_revalidate_an_existing_scoped_document() {
        let mut source = Document::new();
        let a = source.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = source.add(Kind::Free(Point { x: 10., y: 10. }));
        let midpoint = source.add(Kind::Midpoint(a, b));
        let mut scoped = source.evaluated().clone_for_evaluation();
        assert_eq!(scoped.position(midpoint), Some(Point { x: 5., y: 5. }));
        scoped.objects.edit(a - 1).kind = Kind::Free(Point { x: 4., y: 4. });
        assert_eq!(scoped.position(midpoint), Some(Point { x: 7., y: 7. }));
        scoped.objects.edit(b - 1).deleted = true;
        assert_eq!(scoped.position(midpoint), None);
        scoped.objects.edit(b - 1).deleted = false;
        assert_eq!(scoped.position(midpoint), Some(Point { x: 7., y: 7. }));
    }

    #[test]
    fn numeric_calculations_do_not_depend_on_the_whole_object_table() {
        let mut doc = Document::new();
        let n = doc.add_named_parameter("n", 2., (0, 0)).unwrap();
        let square = doc.add_parameter_calculation(n, "n*n", (0, 20)).unwrap();
        let unrelated = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let mut cache = EvaluationCache::default();
        let first = cache.evaluate(&doc);
        assert_eq!(first.numeric_value(square), Some(4.));
        doc.objects.edit(unrelated - 1).kind = Kind::Free(Point { x: 7., y: 5. });
        let second = cache.evaluate(&doc);
        assert_eq!(second.numeric_value(square), Some(4.));
        assert_eq!(
            second.sac_evaluation().unwrap().graph.statistics().executed,
            0
        );
        doc.set_parameter_value(n, -2.).unwrap();
        let third = cache.evaluate(&doc);
        assert_eq!(third.numeric_value(square), Some(4.));
        assert!(third.sac_evaluation().unwrap().graph.statistics().unchanged > 0);
    }

    #[test]
    fn a_cached_undefined_cycle_recovers_when_another_node_is_rewired() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = doc.add(Kind::Midpoint(a, a));
        let mut scoped = doc.evaluated().clone_for_evaluation();
        scoped.objects.edit(a - 1).kind = Kind::Midpoint(b, b);
        assert_eq!(scoped.position(a), None);
        scoped.objects.edit(a - 1).kind = Kind::Free(Point { x: 2., y: 3. });
        assert_eq!(scoped.position(b), Some(Point { x: 2., y: 3. }));
    }
}
