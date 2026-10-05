//! Immutable update schedules shared across locus samples and frames. Values
//! remain in each sampling scope; plans retain no coordinates or results.
use super::evaluation_plan::EvaluationInput;
use super::{Document, Kind, Point};
use lru::LruCache;
use sketchpad_ir::{DependencyGraph, UpdatePlan};
use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

#[derive(Debug, PartialEq, Eq, Hash)]
struct PlanKey {
    driver: usize,
    roots: Vec<usize>,
    path: Option<usize>,
}

/// Shared by a document and its evaluated views. Only immutable schedules are
/// retained, never sample documents or values. Bound memory across graph edits.
#[derive(Clone, Debug)]
pub(super) struct SamplingPlans {
    schedules: Arc<Mutex<LruCache<PlanKey, Arc<SamplingPlan>>>>,
    mapped_inputs: Arc<Mutex<LruCache<MappedInputsKey, MappedInputs>>>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct MappedInputsKey {
    seeds: Vec<usize>,
    branches: Vec<Vec<usize>>,
    carried: Vec<usize>,
}

#[derive(Clone, Debug)]
struct MappedInputs {
    revision: super::sac_evaluation::Identity,
    external: Option<Arc<[usize]>>,
}

impl Default for SamplingPlans {
    fn default() -> Self {
        Self {
            schedules: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(128).unwrap()))),
            mapped_inputs: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(256).unwrap()))),
        }
    }
}

#[derive(Debug)]
pub(super) struct SamplingPlan {
    driver: usize,
    graph: Arc<DependencyGraph>,
    update: UpdatePlan,
    pub(super) external: Vec<usize>,
}

impl SamplingPlan {
    pub(super) fn new(
        doc: &Document,
        driver: usize,
        roots: &[usize],
        path: Option<usize>,
    ) -> Option<Arc<Self>> {
        let key = PlanKey {
            driver,
            roots: roots.to_vec(),
            path,
        };
        let cached = doc
            .sampling_plans
            .schedules
            .lock()
            .ok()
            .and_then(|mut cache| cache.get(&key).cloned());
        if let Some(plan) = cached.filter(|plan| plan.matches(doc)) {
            return Some(plan);
        }
        let plan = Arc::new(Self::build(doc, driver, roots, path)?);
        if let Ok(mut cache) = doc.sampling_plans.schedules.lock() {
            cache.put(key, plan.clone());
        }
        Some(plan)
    }

    // All connections pass through ObjectStore. Coordinates retain graph identity.
    fn matches(&self, doc: &Document) -> bool {
        Arc::ptr_eq(&self.graph, &doc.objects.dependency_graph())
    }

    fn build(doc: &Document, driver: usize, roots: &[usize], path: Option<usize>) -> Option<Self> {
        let mut roots = roots.to_vec();
        roots.push(driver);
        let graph = doc.objects.dependency_graph();
        let program = graph.slice(&roots).ok()?;
        let update = program.update(&[driver]);
        let mut external: BTreeSet<_> = update.boundaries().iter().copied().collect();
        external.extend(path);
        Some(Self {
            driver,
            graph,
            update,
            external: external.into_iter().collect(),
        })
    }

    pub(super) fn point(&self, sample: &mut Document, point: Point) -> Option<()> {
        sample.execute_evaluation_plan(&self.update, [EvaluationInput::Point(self.driver, point)])
    }

    pub(super) fn kind(&self, sample: &mut Document, kind: Kind) -> Option<()> {
        sample.execute_evaluation_plan(&self.update, [EvaluationInput::Kind(self.driver, kind)])
    }

    pub(super) fn number(&self, sample: &mut Document, value: f64) -> Option<()> {
        sample.execute_evaluation_plan(&self.update, [EvaluationInput::Number(self.driver, value)])
    }
}

impl Document {
    /// Mapping boundaries depend on structure, never on substituted coordinates.
    /// Share these small plans across states while keeping all values local.
    pub(super) fn mapped_external_inputs(
        &self,
        seeds: &[usize],
        branches: &[Vec<usize>],
        carried: &[usize],
    ) -> Option<Arc<[usize]>> {
        super::sac_evaluation::structural_validation(|| {
            let key = MappedInputsKey {
                seeds: seeds.to_vec(),
                branches: branches.to_vec(),
                carried: carried.to_vec(),
            };
            let revision = self.objects.sac_validation_identity();
            let cached = self
                .sampling_plans
                .mapped_inputs
                .lock()
                .unwrap()
                .get(&key)
                .cloned();
            if let Some(cached) = cached.filter(|cached| cached.revision == revision) {
                return cached.external;
            }
            let external = self
                .mapped_external_inputs_uncached(seeds, branches, carried)
                .map(Arc::from);
            self.sampling_plans.mapped_inputs.lock().unwrap().put(
                key,
                MappedInputs {
                    revision,
                    external: external.clone(),
                },
            );
            external
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_boundaries_reuse_coordinates_but_track_types_deletion_and_old_views() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = doc.add(Kind::Free(Point { x: 2., y: 0. }));
        let output = doc.add(Kind::Midpoint(a, b));
        let inputs = |doc: &Document| doc.mapped_external_inputs(&[], &[vec![output]], &[]);
        let initial = inputs(&doc).unwrap();
        assert_eq!(&*initial, &[output]);
        doc.objects.edit(a - 1).kind = Kind::Free(Point { x: 3., y: 0. });
        assert!(Arc::ptr_eq(&initial, &inputs(&doc).unwrap()));
        let old = doc.clone_for_evaluation();
        let edges = doc.objects[output - 1].construction_dependencies();
        doc.objects.edit(output - 1).kind = Kind::IterationBinding(a, b);
        assert_eq!(doc.objects[output - 1].construction_dependencies(), edges);
        assert_eq!(&*inputs(&doc).unwrap(), &[a, b]);
        assert_eq!(&*inputs(&old).unwrap(), &[output]);
        assert_eq!(&*inputs(&doc).unwrap(), &[a, b]);
        doc.objects.edit(a - 1).deleted = true;
        assert!(inputs(&doc).is_none());
        doc.objects.edit(a - 1).deleted = false;
        assert_eq!(&*inputs(&doc).unwrap(), &[a, b]);
    }

    #[test]
    fn plans_survive_coordinate_edits_but_rebuild_for_changed_dependencies() {
        let mut doc = Document::from_bytes(
            include_bytes!("../../../../tests/fixtures/locus_midpoint_original.gsp"),
            None,
        )
        .unwrap();
        let Kind::Locus(refs, _) = doc.objects[7].kind.clone() else {
            panic!()
        };
        let before = doc.locus_segments(8, 8).unwrap();
        let original = SamplingPlan::new(&doc, 4, &refs, Some(3)).unwrap();
        let old_view = doc.evaluated();
        assert!(doc.move_point(1, Point { x: 100., y: 220. }));
        let moved_view = doc.evaluated();
        let reused = SamplingPlan::new(&moved_view, 4, &refs, Some(3)).unwrap();
        assert!(Arc::ptr_eq(&original, &reused));
        assert_ne!(moved_view.locus_segments(8, 8).unwrap(), before);
        assert_eq!(old_view.locus_segments(8, 8).unwrap(), before);

        // Public edits and later references must not bypass cache validation.
        let later = doc.add(Kind::Free(Point { x: 500., y: 400. }));
        doc.checkpoint();
        doc.objects.edit(6).midpoint_host = None;
        doc.objects.edit(6).kind = Kind::Midpoint(4, later);
        let rewired = SamplingPlan::new(&doc, 4, &refs, Some(3)).unwrap();
        assert!(!Arc::ptr_eq(&original, &rewired));
        assert_ne!(doc.locus_segments(8, 8), moved_view.locus_segments(8, 8));
        doc.undo();
        assert_eq!(doc.locus_segments(8, 8), moved_view.locus_segments(8, 8));

        doc.objects.edit(6).midpoint_host = None;
        doc.objects.edit(6).kind = Kind::Midpoint(7, 7);
        assert!(SamplingPlan::new(&doc, 4, &refs, Some(3)).is_none());
    }

    #[test]
    fn repeated_numeric_samples_clear_undefined_values_and_preserve_other_seeds() {
        let mut doc = Document::new();
        let driver = doc.add_named_parameter("n", 2., (0, 0)).unwrap();
        let output = doc
            .add_parameter_calculation(driver, "1/n", (0, 20))
            .unwrap();
        let fixed = doc.add_named_parameter("k", 7., (0, 40)).unwrap();
        let point = doc.add(Kind::Free(Point { x: 2., y: 3. }));
        let seed = Point { x: 40., y: 50. };
        doc.set_transient_point_position(point, seed);
        let mut sample = doc.evaluated().clone_for_evaluation();
        let plan = SamplingPlan::new(&doc, driver, &[output, driver], None).unwrap();
        for (input, expected) in [
            (2., Some(0.5)),
            (0., None),
            (4., Some(0.25)),
            (-2., Some(-0.5)),
        ] {
            plan.number(&mut sample, input).unwrap();
            assert_eq!(sample.numeric_value(output), expected);
            assert_eq!(sample.numeric_value(fixed), Some(7.));
            assert_eq!(sample.position(point), Some(seed));
            assert_eq!(doc.numeric_value(output), Some(0.5));
        }
    }

    #[test]
    fn one_geometric_plan_handles_repeated_driver_substitutions() {
        let mut doc = Document::from_bytes(
            include_bytes!("../../../../tests/fixtures/locus_midpoint_original.gsp"),
            None,
        )
        .unwrap();
        let Kind::Locus(refs, _) = &doc.objects[7].kind else {
            panic!()
        };
        let plan = SamplingPlan::new(&doc, 4, refs, Some(3)).unwrap();
        let mut sample = doc.clone_for_evaluation();
        sample
            .freeze_scoped_evaluation_inputs(&doc, &plan.external)
            .unwrap();
        for fraction in [0., 1., 0.25, 0.75, 0.] {
            let kind = doc.locus_driver_kind_at(4, 3, fraction).unwrap();
            doc.objects.edit(3).kind = kind.clone();
            plan.kind(&mut sample, kind).unwrap();
            assert_eq!(sample.position(7), doc.position(7));
        }
    }

    #[test]
    fn sampled_constraints_update_parameter_measurements_and_keep_nested_samples_isolated() {
        use crate::value_equation_records::ValueEquationKind;
        let mut original = Document::new();
        let center = original.add(Kind::Free(Point { x: 0., y: 0. }));
        let rim = original.add(Kind::Free(Point { x: 10., y: 0. }));
        let circle = original.add(Kind::Circle(center, rim));
        let driver = original.add(Kind::PointOnCircle(circle, 1., 0.));
        let measurement = original.add(Kind::ValueEquation {
            kind: ValueEquationKind::PointValue,
            first: driver,
            second: circle,
        });
        let plan = SamplingPlan::new(&original, driver, &[measurement], Some(circle)).unwrap();
        let view = original.evaluated();
        let mut sample = view.clone_for_evaluation();
        sample
            .freeze_scoped_evaluation_inputs(&view, &plan.external)
            .unwrap();
        let mut expected = original.clone_for_evaluation();
        for fraction in [0.25, 0.75, 0., 0.5] {
            let angle = fraction * std::f64::consts::TAU;
            let kind = original
                .locus_driver_kind_at(driver, circle, angle)
                .unwrap();
            expected.objects.edit(driver - 1).kind = kind.clone();
            plan.kind(&mut sample, kind).unwrap();
            assert_eq!(sample.position(driver), expected.position(driver));
            assert_eq!(
                sample.numeric_value(measurement),
                expected.numeric_value(measurement)
            );
            assert_eq!(sample.numeric_value(measurement), Some(fraction));
            let before = sample.position(driver);
            let mut nested = sample.clone_for_evaluation();
            nested
                .freeze_scoped_evaluation_inputs(&sample, &plan.external)
                .unwrap();
            plan.kind(&mut nested, Kind::PointOnCircle(circle, 1., 0.))
                .unwrap();
            assert_eq!(nested.numeric_value(measurement), Some(0.));
            assert_eq!(sample.position(driver), before);
            assert_eq!(sample.numeric_value(measurement), Some(fraction));
            assert_eq!(view.numeric_value(measurement), Some(0.));
        }
    }
}
