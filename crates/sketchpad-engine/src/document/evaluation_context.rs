//! Ephemeral construction values used by mapped evaluation. Native object kinds
//! and serialized records retain their identity while state values are replaced.
use super::kind::{is_arc_kind, is_point_kind, is_straight_kind};
use super::{Document, Kind, LineMode, Point};
use crate::graph::CoordinateFrame;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc as Shared;

type Straight = (Point, Point, LineMode);
type Arc = (Point, f64, f64, f64, bool);

/// An immutable evaluated view. Create a new view after editing its source.
/// Calculation caches and history are never written back to the source document.
#[derive(Clone, Debug)]
pub struct EvaluatedDocument(pub(super) Document);

impl std::ops::Deref for EvaluatedDocument {
    type Target = Document;

    fn deref(&self) -> &Document {
        &self.0
    }
}

impl EvaluatedDocument {
    /// Warm independent visible traces before rendering. Each sample owns its
    /// substitutions; only immutable results enter this view's bounded cache.
    pub fn prepare_locus_strokes(&self, subdivisions: usize) {
        let Some(context) = &self.evaluation_context else {
            return;
        };
        let pending: Vec<_> = {
            self.objects
                .iter()
                .filter(|o| matches!(o.kind, Kind::Locus(..) | Kind::ParameterLocus(..)))
                .filter(|o| self.is_displayed(o.id))
                .filter(|o| {
                    !context
                        .sac
                        .graph
                        .is_current(&super::sac_evaluation::Query::Geometry(
                            o.id,
                            super::geometry_cache::GeometryQuery::LocusStrokes,
                            subdivisions,
                        ))
                })
                .map(|o| o.id)
                .take(64)
                .collect()
        };
        if pending.is_empty() {
            return;
        }
        let workers = std::thread::available_parallelism()
            .map_or(1, |count| count.get())
            .min(4)
            .min(pending.len());
        let evaluate = |ids: &[usize]| {
            for &id in ids {
                let _ = self.locus_styled_strokes(id, subdivisions);
            }
        };
        if workers == 1 {
            evaluate(&pending);
        } else {
            std::thread::scope(|scope| {
                let mut chunks = pending.chunks(pending.len().div_ceil(workers));
                let local = chunks.next().unwrap();
                for chunk in chunks {
                    let evaluate = &evaluate;
                    scope.spawn(move || evaluate(chunk));
                }
                evaluate(local);
            });
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct EvaluationContext {
    sac: super::sac_evaluation::Evaluation,
    sampling_baseline: std::sync::OnceLock<Shared<Document>>,
    validated: ValidatedDependencies,
    arc_dependencies: ValidatedDependencies,
    fonts: FontCache,
    points: BTreeMap<usize, Option<Point>>,
    numbers: BTreeMap<usize, Option<f64>>,
    straights: BTreeMap<usize, Option<Straight>>,
    circles: BTreeMap<usize, Option<(Point, f64)>>,
    polygons: BTreeMap<usize, Option<Vec<Point>>>,
    arcs: BTreeMap<usize, Option<Arc>>,
    frames: BTreeMap<usize, Option<CoordinateFrame>>,
    seed_points: BTreeMap<usize, Point>,
    seed_numbers: BTreeMap<usize, f64>,
    frozen: BTreeSet<usize>,
    pub(super) original: Shared<Document>,
}

// Validation contains only structural facts. Sharing the revision-checked set
// avoids copying it for each sample; old views and parallel readers still check
// their own identity while holding the same lock.
type ValidationState = (Option<super::sac_evaluation::Identity>, BTreeSet<usize>);
#[derive(Clone, Debug, Default)]
struct ValidatedDependencies(Shared<std::sync::Mutex<ValidationState>>);
#[derive(Debug, Default)]
struct FontCache(std::sync::Mutex<BTreeMap<usize, Option<super::FontState>>>);
impl Clone for FontCache {
    fn clone(&self) -> Self {
        Self(std::sync::Mutex::new(self.0.lock().unwrap().clone()))
    }
}
impl EvaluationContext {
    fn new(original: Shared<Document>) -> Self {
        Self {
            sac: Default::default(),
            sampling_baseline: std::sync::OnceLock::new(),
            validated: ValidatedDependencies::default(),
            arc_dependencies: ValidatedDependencies::default(),
            fonts: FontCache::default(),
            points: BTreeMap::new(),
            numbers: BTreeMap::new(),
            straights: BTreeMap::new(),
            circles: BTreeMap::new(),
            polygons: BTreeMap::new(),
            arcs: BTreeMap::new(),
            frames: BTreeMap::new(),
            seed_points: BTreeMap::new(),
            seed_numbers: BTreeMap::new(),
            frozen: BTreeSet::new(),
            original,
        }
    }
}

impl Document {
    pub(super) fn with_sac_scope<T>(
        &self,
        sample: &mut Document,
        name: super::sac_evaluation::Scope,
        evaluate: impl FnOnce(&mut Document) -> T,
    ) -> T {
        if let Some(parent) = self.sac_evaluation()
            && let Some(context) = sample.evaluation_context_mut()
        {
            let fallback = std::mem::replace(
                &mut context.sac.graph,
                super::sac_evaluation::Graph::new(4096.try_into().unwrap()),
            );
            context.sac = parent.child(name, fallback);
            let result = super::sac_evaluation::evaluate_scope(sample, evaluate);
            sample.sac_evaluation().unwrap().publish();
            result
        } else {
            evaluate(sample)
        }
    }
    pub(super) fn sac_evaluation(&self) -> Option<&super::sac_evaluation::Evaluation> {
        self.evaluation_context.as_ref().map(|context| &context.sac)
    }

    pub(super) fn sac_frozen_stamp(&self, id: usize) -> super::sac_evaluation::Stamp {
        let context = self.evaluation_context.as_ref().unwrap();
        super::sac_evaluation::Stamp::Frozen(context.frozen.contains(&id).then(|| {
            (
                context.original.objects.sac_identity(None),
                context
                    .original
                    .sac_evaluation()
                    .map(|scope| scope.input_revision()),
            )
        }))
    }

    pub(super) fn sac_slot(
        &self,
        input: super::sac_evaluation::Input,
    ) -> Option<super::sac_evaluation::Value> {
        use super::sac_evaluation::{Input, Value};
        let context = self.evaluation_context.as_ref()?;
        match input {
            Input::Point(id) => context
                .seed_points
                .get(&id)
                .copied()
                .map(Some)
                .or_else(|| context.points.get(&id).copied())
                .map(Value::Point),
            Input::Number(id) => context
                .seed_numbers
                .get(&id)
                .copied()
                .map(Some)
                .or_else(|| context.numbers.get(&id).copied())
                .map(Value::Number),
            Input::Straight(id) => context.straights.get(&id).copied().map(Value::Straight),
            Input::Circle(id) => context.circles.get(&id).copied().map(Value::Circle),
            Input::Polygon(id) => context.polygons.get(&id).cloned().map(Value::Polygon),
            Input::Arc(id) => context.arcs.get(&id).copied().map(Value::Arc),
            Input::Frame(id) => context.frames.get(&id).copied().map(Value::Frame),
            _ => None,
        }
    }
    pub(super) fn cached_geometry<T: Clone + Send + Sync + 'static>(
        &self,
        id: usize,
        query: super::geometry_cache::GeometryQuery,
        resolution: usize,
        evaluate: impl FnOnce() -> Option<T>,
    ) -> Option<T> {
        use super::sac_evaluation::{Query, Value};
        // Keep this read tracked: a parent query that sees an unsupported kind
        // must be invalidated when an edit makes that kind applicable.
        if !query.accepts(&self.objects.get(id.checked_sub(1)?)?.kind) {
            return None;
        }
        if let Some(Value::Geometry(value)) = self.sac_query(Query::Geometry(id, query, resolution))
        {
            return value.and_then(|value| value.get::<T>());
        }
        evaluate()
    }

    // Sampling baselines share immutable values. Detach before changing seeds
    // or invalidating results so older views and sibling samples stay valid.
    pub(super) fn evaluation_context_mut(&mut self) -> Option<&mut EvaluationContext> {
        let context = self.evaluation_context.as_mut().map(Shared::make_mut)?;
        context.sac.graph.advance();
        context.sampling_baseline.take();
        Some(context)
    }

    fn sampling_baseline(&self) -> Shared<Document> {
        let Some(context) = &self.evaluation_context else {
            return Shared::new(self.clone_for_evaluation());
        };
        context
            .sampling_baseline
            .get_or_init(|| {
                let mut baseline = self.clone_for_evaluation();
                // Detach the context before storing the baseline; otherwise its
                // shared context would own itself through this OnceLock.
                baseline.evaluation_context_mut();
                Shared::new(baseline)
            })
            .clone()
    }

    pub(super) fn with_validated_dependencies<T>(
        &self,
        validate: impl FnOnce(&mut BTreeSet<usize>) -> T,
    ) -> T {
        super::sac_evaluation::structural_validation(|| {
            if let Some(context) = &self.evaluation_context {
                let identity = self.objects.sac_validation_identity();
                let mut cached = context.validated.0.lock().unwrap();
                if cached.0.as_ref() != Some(&identity) {
                    cached.0 = Some(identity);
                    cached.1.clear();
                }
                validate(&mut cached.1)
            } else {
                validate(&mut BTreeSet::new())
            }
        })
    }

    pub(super) fn with_available_arc_dependencies<T>(
        &self,
        validate: impl FnOnce(&mut BTreeSet<usize>) -> T,
    ) -> T {
        super::sac_evaluation::structural_validation(|| {
            if let Some(context) = &self.evaluation_context {
                let identity = self.objects.sac_validation_identity();
                let mut cached = context.arc_dependencies.0.lock().unwrap();
                if cached.0.as_ref() != Some(&identity) {
                    cached.0 = Some(identity);
                    cached.1.clear();
                }
                validate(&mut cached.1)
            } else {
                validate(&mut BTreeSet::new())
            }
        })
    }

    pub(super) fn cached_text_font(
        &self,
        id: usize,
        evaluate: impl FnOnce() -> Option<super::FontState>,
    ) -> Option<super::FontState> {
        let Some(context) = &self.evaluation_context else {
            return evaluate();
        };
        let cached = context.fonts.0.lock().unwrap().get(&id).cloned();
        if let Some(value) = cached {
            return value;
        }
        let value = evaluate();
        context.fonts.0.lock().unwrap().insert(id, value.clone());
        value
    }

    pub(super) fn invalidate_text_fonts(&mut self, ids: &[usize]) {
        if let Some(context) = self.evaluation_context_mut() {
            let fonts = context.fonts.0.get_mut().unwrap();
            for id in ids {
                fonts.remove(id);
            }
        }
    }

    /// Create an isolated SAC view. Values are evaluated when first demanded
    /// by drawing, fitting, export or another construction.
    pub fn evaluated(&self) -> EvaluatedDocument {
        let mut snapshot = self.clone_for_evaluation();
        if self.objects.dependency_program().is_ok() {
            let _ = snapshot.ensure_evaluation_context(self);
        }
        EvaluatedDocument(snapshot)
    }

    // Read-only evaluation needs construction and presentation state, but never
    // undo history. Copying history here would make every frame more expensive
    // after each drag, even though the construction size had not changed.
    pub(super) fn clone_for_evaluation(&self) -> Self {
        Self {
            objects: self.objects.clone(),
            path: self.path.clone(),
            selected: self.selected.clone(),
            dirty: self.dirty,
            partial_view: self.partial_view,
            active_page: self.active_page,
            page_count: self.page_count,
            snap_points: self.snap_points.clone(),
            grid_dotted: self.grid_dotted.clone(),
            grid_form: self.grid_form.clone(),
            page_names: self.page_names.clone(),
            angle_unit_radians: self.angle_unit_radians,
            marked_coordinate_system: self.marked_coordinate_system.clone(),
            frame_styles: self.frame_styles.clone(),
            object_color_defaults: self.object_color_defaults,
            object_style_defaults: self.object_style_defaults,
            measurement_precision_defaults: self.measurement_precision_defaults,
            new_polygon_opacity: self.new_polygon_opacity,
            randomize_interior_colors: self.randomize_interior_colors,
            interior_color_cursor: self.interior_color_cursor,
            merge_aliases: self.merge_aliases.clone(),
            measurement_names: self.measurement_names.clone(),
            evaluation_context: self.evaluation_context.clone(),
            sampling_plans: self.sampling_plans.clone(),
            original: self.original.clone(),
            run_font_families: self.run_font_families.clone(),
            source_count: self.source_count,
            last_moved_object: self.last_moved_object,
            custom_transform_definitions: self.custom_transform_definitions.clone(),
            safe_structure: self.safe_structure,
            image_read_only: self.image_read_only,
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    /// Temporarily place a constructed point without changing its native
    /// construction. Serialization and document history ignore this value.
    pub fn set_transient_point_position(&mut self, id: usize, point: Point) -> bool {
        if self.evaluation_context.is_none() {
            let mut original = self.clone();
            original.evaluation_context = None;
            self.evaluation_context =
                Some(Shared::new(EvaluationContext::new(Shared::new(original))));
        }
        self.invalidate_evaluation_dependents(id);
        self.set_evaluation_point(id, point).is_some()
    }

    /// Return temporarily placed points to their construction-defined positions.
    pub fn clear_transient_point_positions(&mut self, ids: &[usize]) {
        let Some(context) = self.evaluation_context_mut() else {
            return;
        };
        for id in ids {
            context.seed_points.remove(id);
        }
        if context.seed_points.is_empty()
            && context.seed_numbers.is_empty()
            && context.frozen.is_empty()
            && context.points.is_empty()
            && context.numbers.is_empty()
            && context.straights.is_empty()
            && context.circles.is_empty()
            && context.polygons.is_empty()
            && context.arcs.is_empty()
            && context.frames.is_empty()
        {
            self.evaluation_context = None;
        }
    }

    /// Establish a seedable evaluation context without replacing an existing
    /// scoped context supplied by a caller.
    pub(super) fn ensure_evaluation_context(&mut self, original: &Document) -> Option<()> {
        if self.evaluation_context.is_none() {
            self.freeze_evaluation_inputs(original, &[])?;
        }
        Some(())
    }

    /// Freeze precisely the external construction nodes supplied by the caller.
    /// A cached undefined value is different from a construction without a cache.
    pub(super) fn freeze_evaluation_inputs(
        &mut self,
        original: &Document,
        external_ids: &[usize],
    ) -> Option<()> {
        self.freeze_evaluation_inputs_with_parent(original, external_ids, false)
    }

    /// A nested stored mapping samples the caller's existing substitutions.
    /// Retain that parent context when caching omitted construction boundaries.
    pub(super) fn freeze_scoped_evaluation_inputs(
        &mut self,
        original: &Document,
        external_ids: &[usize],
    ) -> Option<()> {
        self.freeze_evaluation_inputs_with_parent(original, external_ids, true)
    }

    fn freeze_evaluation_inputs_with_parent(
        &mut self,
        original: &Document,
        external_ids: &[usize],
        retain_parent: bool,
    ) -> Option<()> {
        let baseline = if retain_parent {
            original.sampling_baseline()
        } else {
            let mut baseline = original.clone_for_evaluation();
            baseline.evaluation_context = None;
            Shared::new(baseline)
        };
        let mut context = EvaluationContext::new(baseline);
        if let Some(parent) = original.sac_evaluation() {
            context.sac = parent.clone();
            context.sac.graph.advance();
        }
        // Validation concerns references, pages and object kinds, not sampled
        // coordinates. A scope over the same immutable table can reuse it.
        if self.objects.shares_objects(&original.objects)
            && let Some(parent) = &original.evaluation_context
        {
            context.validated = parent.validated.clone();
            context.arc_dependencies = parent.arc_dependencies.clone();
        }
        for &id in external_ids {
            let object = context.original.objects.get(id.checked_sub(1)?)?;
            if object.deleted {
                return None;
            }
            context.frozen.insert(id);
            let kind = &object.kind;
            if is_point_kind(kind) {
                context.points.insert(id, context.original.position(id));
            }
            if kind.numeric_unit().is_some() {
                context
                    .numbers
                    .insert(id, context.original.numeric_value(id));
            }
            if is_straight_kind(kind) {
                context
                    .straights
                    .insert(id, context.original.line_points(id));
            }
            if kind.is_circle() {
                context
                    .circles
                    .insert(id, context.original.circle_points(id));
            }
            if kind.is_circle_interior() {
                context
                    .circles
                    .insert(id, context.original.circle_interior_geometry(id));
            }
            if kind.is_polygon() {
                context
                    .polygons
                    .insert(id, context.original.polygon_points(id));
            }
            if kind.is_sector() {
                context
                    .arcs
                    .insert(id, context.original.sector_arc_geometry(id));
            }
            if is_arc_kind(kind) {
                context.arcs.insert(id, context.original.arc_geometry(id));
            }
            if matches!(kind, Kind::CoordinateSystem(..)) {
                context.frames.insert(id, context.original.graph_frame(id));
            }
        }
        self.evaluation_context = Some(Shared::new(context));
        Some(())
    }

    /// A terminal nested mapping overrides its own seeds while retaining unrelated
    /// substitutions established by its caller.
    pub(super) fn inherit_evaluation_seeds(&mut self, original: &Document) -> Option<()> {
        let target = self.evaluation_context_mut()?;
        if let Some(parent) = &original.evaluation_context {
            target
                .seed_points
                .extend(parent.seed_points.iter().map(|(&id, &value)| (id, value)));
            target
                .seed_numbers
                .extend(parent.seed_numbers.iter().map(|(&id, &value)| (id, value)));
        }
        Some(())
    }
    /// Re-evaluate a local trace driver and its descendants while preserving
    /// unrelated frozen inputs and caller substitutions.
    pub(super) fn invalidate_evaluation_dependents(&mut self, driver: usize) {
        if self.evaluation_context.is_none() {
            return;
        }
        let affected: BTreeSet<_> = match self.objects.dependency_program() {
            Ok(program) => program.index().affected_by(driver).into_iter().collect(),
            // Imported partial documents may have unresolved construction edges.
            // Preserve their existing conservative invalidation behavior.
            Err(_) => {
                let mut affected = BTreeSet::from([driver]);
                for object in &self.objects {
                    if object
                        .construction_dependencies()
                        .iter()
                        .any(|id| affected.contains(id))
                    {
                        affected.insert(object.id);
                    }
                }
                affected
            }
        };
        self.invalidate_evaluation_ids(driver, affected);
    }

    pub(super) fn invalidate_evaluation_ids(
        &mut self,
        driver: usize,
        ids: impl IntoIterator<Item = usize>,
    ) {
        if let Some(context) = self.evaluation_context_mut() {
            for id in ids {
                context.points.remove(&id);
                context.numbers.remove(&id);
                context.straights.remove(&id);
                context.circles.remove(&id);
                context.polygons.remove(&id);
                context.arcs.remove(&id);
                context.frames.remove(&id);
                context.frozen.remove(&id);
            }
            context.seed_points.remove(&driver);
            context.seed_numbers.remove(&driver);
        }
    }

    pub(super) fn clear_evaluation_seed_ids(&mut self, ids: &[usize]) {
        if let Some(context) = self.evaluation_context_mut() {
            for id in ids {
                context.seed_points.remove(id);
                context.seed_numbers.remove(id);
            }
        }
    }

    pub(super) fn clear_evaluation_seeds(&mut self) {
        if let Some(context) = self.evaluation_context_mut() {
            context.seed_points.clear();
            context.seed_numbers.clear();
        }
    }

    /// Discard values derived from a previous substitution while preserving
    /// frozen boundaries and the caller's current seed values.
    pub(super) fn clear_evaluation_results(&mut self) {
        let Some(context) = self.evaluation_context_mut() else {
            return;
        };
        let frozen = &context.frozen;
        context.points.retain(|id, _| frozen.contains(id));
        context.numbers.retain(|id, _| frozen.contains(id));
        context.straights.retain(|id, _| frozen.contains(id));
        context.circles.retain(|id, _| frozen.contains(id));
        context.polygons.retain(|id, _| frozen.contains(id));
        context.arcs.retain(|id, _| frozen.contains(id));
        context.frames.retain(|id, _| frozen.contains(id));
    }

    /// Install derived batch results after seeding one iteration state. These
    /// are ordinary cached values and are cleared before the next state.
    pub(super) fn cache_evaluation_points(
        &mut self,
        values: impl Iterator<Item = (usize, Option<Point>)>,
    ) -> Option<()> {
        self.evaluation_context_mut()?.points.extend(values);
        Some(())
    }

    pub(super) fn set_evaluation_point(&mut self, id: usize, point: Point) -> Option<()> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted
            || !is_point_kind(&object.kind)
            || !point.x.is_finite()
            || !point.y.is_finite()
        {
            return None;
        }
        let context = self.evaluation_context_mut()?;
        context.seed_numbers.remove(&id);
        context.seed_points.insert(id, point);
        Some(())
    }

    pub(super) fn set_evaluation_number(&mut self, id: usize, value: f64) -> Option<()> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted || object.kind.numeric_unit().is_none() || !value.is_finite() {
            return None;
        }
        self.invalidate_evaluation_dependents(id);
        self.seed_evaluation_number(id, value)
    }

    // Callers with a fixed sampling plan invalidate its affected slots first.
    pub(super) fn seed_evaluation_number(&mut self, id: usize, value: f64) -> Option<()> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted || object.kind.numeric_unit().is_none() || !value.is_finite() {
            return None;
        }
        let context = self.evaluation_context_mut()?;
        context.seed_points.remove(&id);
        context.seed_numbers.insert(id, value);
        Some(())
    }

    pub(super) fn evaluation_point(&self, id: usize) -> Option<Option<Point>> {
        super::sac_evaluation::observe(super::sac_evaluation::Input::Point(id));
        let context = self.evaluation_context.as_ref()?;
        if let Some(point) = context.seed_points.get(&id) {
            return Some(Some(*point));
        }
        context.points.get(&id).copied()
    }
    pub(super) fn evaluation_numeric(&self, id: usize) -> Option<Option<f64>> {
        super::sac_evaluation::observe(super::sac_evaluation::Input::Number(id));
        let context = self.evaluation_context.as_ref()?;
        if let Some(value) = context.seed_numbers.get(&id) {
            return Some(Some(*value));
        }
        context.numbers.get(&id).copied()
    }
    pub(super) fn evaluation_straight(&self, id: usize) -> Option<Option<Straight>> {
        super::sac_evaluation::observe(super::sac_evaluation::Input::Straight(id));
        self.evaluation_context
            .as_ref()?
            .straights
            .get(&id)
            .copied()
    }
    pub(super) fn evaluation_circle(&self, id: usize) -> Option<Option<(Point, f64)>> {
        super::sac_evaluation::observe(super::sac_evaluation::Input::Circle(id));
        self.evaluation_context.as_ref()?.circles.get(&id).copied()
    }
    pub(super) fn evaluation_polygon(&self, id: usize) -> Option<Option<Vec<Point>>> {
        super::sac_evaluation::observe(super::sac_evaluation::Input::Polygon(id));
        self.evaluation_context.as_ref()?.polygons.get(&id).cloned()
    }
    pub(super) fn evaluation_arc(&self, id: usize) -> Option<Option<Arc>> {
        super::sac_evaluation::observe(super::sac_evaluation::Input::Arc(id));
        self.evaluation_context.as_ref()?.arcs.get(&id).copied()
    }
    pub(super) fn evaluation_frame(&self, id: usize) -> Option<Option<CoordinateFrame>> {
        super::sac_evaluation::observe(super::sac_evaluation::Input::Frame(id));
        self.evaluation_context.as_ref()?.frames.get(&id).copied()
    }
    /// Functions and degenerate arc policies need the original construction,
    /// rather than a single sampled number or circular geometry.
    pub(super) fn frozen_evaluation_document(&self, id: usize) -> Option<&Document> {
        super::sac_evaluation::observe(super::sac_evaluation::Input::Frozen(id));
        let context = self.evaluation_context.as_ref()?;
        context
            .frozen
            .contains(&id)
            .then_some(context.original.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arc_dependency_cache_handles_shared_ancestors_and_structural_edits() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = doc.add(Kind::Free(Point { x: 2., y: 0. }));
        let c = doc.add(Kind::Free(Point { x: 0., y: 2. }));
        let mut shared = a;
        for _ in 0..20 {
            shared = doc.add(Kind::Midpoint(shared, shared));
        }
        let arc = doc.add(Kind::ThreePointArc(shared, b, c));
        let mut view = doc.evaluated().0;
        assert!(view.plot_arc_host_available(arc));
        let old = view.clone_for_evaluation();
        view.objects.edit(a - 1).deleted = true;
        assert!(!view.plot_arc_host_available(arc));
        view.objects.edit(a - 1).deleted = false;
        assert!(view.plot_arc_host_available(arc));
        view.objects.edit(a - 1).page = 1;
        assert!(!view.plot_arc_host_available(arc));
        view.objects.edit(a - 1).page = 0;
        view.objects.edit(a - 1).kind = Kind::Midpoint(shared, shared);
        assert!(!view.plot_arc_host_available(arc));
        assert!(old.plot_arc_host_available(arc));
        view.objects.edit(a - 1).kind = Kind::Free(Point { x: 0., y: 0. });
        assert!(view.arc_geometry(arc).is_some());
        view.objects.edit(c - 1).kind = Kind::Free(Point { x: 4., y: 0. });
        assert!(view.arc_geometry(arc).is_none());
        assert!(old.arc_geometry(arc).is_some());

        // The structural cache is shared, but each concurrent view must use
        // its own revision. A negative result in one must not poison another.
        view.objects.edit(a - 1).deleted = true;
        std::thread::scope(|scope| {
            scope.spawn(|| {
                for _ in 0..100 {
                    assert!(old.plot_arc_host_available(arc));
                }
            });
            for _ in 0..100 {
                assert!(!view.plot_arc_host_available(arc));
            }
        });
    }

    #[test]
    fn a_working_set_larger_than_capacity_stays_cached_across_views() {
        let mut doc = Document::new();
        let mut outputs = Vec::new();
        for i in 0..24 {
            let seed = doc.add(Kind::Free(Point { x: i as f64, y: 2. }));
            outputs.push((seed, doc.add(Kind::Midpoint(seed, seed))));
        }
        let mut view = doc.evaluated().0;
        view.evaluation_context_mut().unwrap().sac.graph =
            super::super::incremental_graph::Graph::new(8.try_into().unwrap());
        let check = |view: &Document, offset: f64| {
            for (i, &(_, output)) in outputs.iter().enumerate() {
                assert_eq!(
                    view.position(output),
                    Some(Point {
                        x: i as f64 + offset,
                        y: 2.
                    })
                );
            }
            let graph = &view.sac_evaluation().unwrap().graph;
            assert_eq!(graph.len(), outputs.len() * 2);
            let executed = graph.statistics().executed;
            for &(_, output) in outputs.iter().rev() {
                view.position(output);
            }
            assert_eq!(graph.statistics().executed, executed);
        };
        check(&view, 0.);
        let old = view.clone_for_evaluation();
        view.evaluation_context_mut();
        for (i, &(seed, _)) in outputs.iter().enumerate() {
            view.objects.edit(seed - 1).kind = Kind::Free(Point {
                x: i as f64 + 10.,
                y: 2.,
            });
        }
        check(&view, 10.);
        check(&old, 0.);
        check(&view, 10.);
    }

    #[test]
    fn inactive_history_is_reclaimed_without_splitting_dependencies() {
        let mut doc = Document::new();
        let mut outputs = Vec::new();
        for i in 0..160 {
            let seed = doc.add(Kind::Free(Point { x: i as f64, y: 2. }));
            outputs.push(doc.add(Kind::Midpoint(seed, seed)));
        }
        let mut view = doc.evaluated().0;
        view.evaluation_context_mut().unwrap().sac.graph =
            super::super::incremental_graph::Graph::new(8.try_into().unwrap());
        for batch in outputs.chunks(16) {
            view.evaluation_context_mut();
            for &output in batch {
                assert_eq!(view.position(output), doc.position(output));
            }
            let graph = &view.sac_evaluation().unwrap().graph;
            assert!(
                graph.len() <= 72,
                "inactive records accumulated: {}",
                graph.len()
            );
            for &output in batch {
                assert!(
                    graph
                        .input_footprint(&super::super::sac_evaluation::Query::Point(output))
                        .is_some()
                );
            }
        }
    }

    #[test]
    fn unsupported_geometry_does_not_cache_and_tracks_type_changes() {
        let doc = Document::from_bytes(
            include_bytes!("../../../../tests/fixtures/numeric_ratio_measured_iteration_port.gsp"),
            None,
        )
        .unwrap();
        let output = 6;
        let iteration = doc.objects[output - 1].kind.clone();
        let mut view = doc.evaluated().0;
        view.objects.edit(output - 1).kind = Kind::Free(Point { x: 5., y: 6. });
        assert_eq!(view.point_iteration_images(output), None);
        assert_eq!(view.sac_evaluation().unwrap().graph.len(), 0);
        view.objects.edit(output - 1).kind = iteration;
        assert_eq!(
            view.point_iteration_images(output),
            view.point_iteration_images_uncached(output)
        );
        assert!(view.point_iteration_images(output).is_some());
        view.objects.edit(output - 1).kind = Kind::Free(Point { x: 5., y: 6. });
        assert_eq!(view.point_iteration_images(output), None);
    }

    #[test]
    fn evaluated_views_match_recursive_geometry_and_sampled_scopes() {
        for name in [
            "locus_midpoint_original.gsp",
            "curve_style_parameter_locus_native_verified.gsp",
            "numeric_cood_trace_native.gsp",
            "locus_undefined_sample_verified.gsp",
            "mapped_calculated_angle_iteration_full_native_verified.gsp",
            "source_exponents_numeric_iteration_port.gsp",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures")
                .join(name);
            let doc = Document::open(&path).unwrap();
            let evaluated = doc.evaluated();
            for object in &doc.objects {
                let id = object.id;
                assert_eq!(
                    evaluated.position(id),
                    doc.position(id),
                    "{name}: point {id}"
                );
                assert_eq!(
                    evaluated.numeric_value(id),
                    doc.numeric_value(id),
                    "{name}: number {id}"
                );
                assert_eq!(
                    evaluated.line_points(id),
                    doc.line_points(id),
                    "{name}: line {id}"
                );
                assert_eq!(
                    evaluated.is_displayed(id),
                    doc.is_displayed(id),
                    "{name}: display {id}"
                );
                assert_eq!(
                    evaluated.locus_segments(id, 8),
                    doc.locus_segments(id, 8),
                    "{name}: locus {id}"
                );
                assert_eq!(
                    evaluated.mapped_collection_strokes(id, 8),
                    doc.mapped_collection_strokes(id, 8),
                    "{name}: mapping {id}"
                );
            }
            assert_eq!(evaluated.encode(), doc.encode(), "{name}: saved bytes");
        }
    }

    #[test]
    fn evaluated_views_are_isolated_from_edits_history_and_transient_inputs() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = doc.add(Kind::Free(Point { x: 10., y: 20. }));
        let middle = doc.add(Kind::Midpoint(a, b));
        let parameter = doc.add_named_parameter("n", 2., (0, 0)).unwrap();
        let reciprocal = doc
            .add_parameter_calculation(parameter, "1/n", (0, 20))
            .unwrap();
        let view = doc.evaluated();
        assert_eq!(view.position(middle), Some(Point { x: 5., y: 10. }));
        assert_eq!(view.numeric_value(reciprocal), Some(0.5));
        assert!(view.undo.is_empty() && view.redo.is_empty());
        assert!(!doc.undo.is_empty());
        assert!(doc.evaluation_context.is_none());

        assert!(doc.move_point(a, Point { x: 20., y: 30. }));
        doc.set_parameter_value(parameter, 0.).unwrap();
        assert_eq!(doc.evaluated().numeric_value(reciprocal), None);
        doc.set_parameter_value(parameter, 4.).unwrap();
        let updated = doc.evaluated();
        assert_eq!(updated.position(middle), Some(Point { x: 15., y: 25. }));
        assert_eq!(updated.numeric_value(reciprocal), Some(0.25));
        assert_eq!(view.position(middle), Some(Point { x: 5., y: 10. }));
        assert_eq!(view.numeric_value(reciprocal), Some(0.5));

        assert!(doc.set_transient_point_position(a, Point { x: 30., y: 40. }));
        assert_eq!(
            doc.evaluated().position(middle),
            Some(Point { x: 20., y: 30. })
        );
        doc.clear_transient_point_positions(&[a]);
        assert_eq!(
            doc.evaluated().position(middle),
            Some(Point { x: 15., y: 25. })
        );
    }
}
