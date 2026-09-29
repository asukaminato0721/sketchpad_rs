//! Ephemeral construction values used by mapped evaluation. Native object kinds
//! and serialized records retain their identity while state values are replaced.
use super::kind::{is_arc_kind, is_point_kind, is_straight_kind};
use super::{Document, Kind, LineMode, Point};
use crate::graph::CoordinateFrame;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc as Shared;

type Straight = (Point, Point, LineMode);
type Arc = (Point, f64, f64, f64, bool);

#[derive(Clone, Debug)]
pub(super) struct EvaluationContext {
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
    original: Shared<Document>,
}

impl Document {
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
        let mut baseline = original.clone();
        if !retain_parent {
            baseline.evaluation_context = None;
        }
        let mut context = EvaluationContext {
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
            original: Shared::new(baseline),
        };
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
        self.evaluation_context = Some(context);
        Some(())
    }

    /// A terminal nested mapping overrides its own seeds while retaining unrelated
    /// substitutions established by its caller.
    pub(super) fn inherit_evaluation_seeds(&mut self, original: &Document) -> Option<()> {
        let target = self.evaluation_context.as_mut()?;
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
        let affected: BTreeSet<_> = match self.dependency_index() {
            Ok(index) => index.affected_by(driver).into_iter().collect(),
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
        if let Some(context) = &mut self.evaluation_context {
            for id in affected {
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
        if let Some(context) = &mut self.evaluation_context {
            for id in ids {
                context.seed_points.remove(id);
                context.seed_numbers.remove(id);
            }
        }
    }

    pub(super) fn clear_evaluation_seeds(&mut self) {
        if let Some(context) = &mut self.evaluation_context {
            context.seed_points.clear();
            context.seed_numbers.clear();
        }
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
        let context = self.evaluation_context.as_mut()?;
        context.seed_numbers.remove(&id);
        context.seed_points.insert(id, point);
        Some(())
    }

    pub(super) fn set_evaluation_number(&mut self, id: usize, value: f64) -> Option<()> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted || object.kind.numeric_unit().is_none() || !value.is_finite() {
            return None;
        }
        let context = self.evaluation_context.as_mut()?;
        context.seed_points.remove(&id);
        context.seed_numbers.insert(id, value);
        Some(())
    }

    pub(super) fn evaluation_point(&self, id: usize) -> Option<Option<Point>> {
        let context = self.evaluation_context.as_ref()?;
        if let Some(point) = context.seed_points.get(&id) {
            return Some(Some(*point));
        }
        context.points.get(&id).copied()
    }
    pub(super) fn evaluation_numeric(&self, id: usize) -> Option<Option<f64>> {
        let context = self.evaluation_context.as_ref()?;
        if let Some(value) = context.seed_numbers.get(&id) {
            return Some(Some(*value));
        }
        context.numbers.get(&id).copied()
    }
    pub(super) fn evaluation_straight(&self, id: usize) -> Option<Option<Straight>> {
        self.evaluation_context
            .as_ref()?
            .straights
            .get(&id)
            .copied()
    }
    pub(super) fn evaluation_circle(&self, id: usize) -> Option<Option<(Point, f64)>> {
        self.evaluation_context.as_ref()?.circles.get(&id).copied()
    }
    pub(super) fn evaluation_polygon(&self, id: usize) -> Option<Option<Vec<Point>>> {
        self.evaluation_context.as_ref()?.polygons.get(&id).cloned()
    }
    pub(super) fn evaluation_arc(&self, id: usize) -> Option<Option<Arc>> {
        self.evaluation_context.as_ref()?.arcs.get(&id).copied()
    }
    pub(super) fn evaluation_frame(&self, id: usize) -> Option<Option<CoordinateFrame>> {
        self.evaluation_context.as_ref()?.frames.get(&id).copied()
    }
    /// Functions and degenerate arc policies need the original construction,
    /// rather than a single sampled number or circular geometry.
    pub(super) fn frozen_evaluation_document(&self, id: usize) -> Option<&Document> {
        let context = self.evaluation_context.as_ref()?;
        context
            .frozen
            .contains(&id)
            .then_some(context.original.as_ref())
    }
}
