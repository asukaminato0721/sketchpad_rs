//! Immutable update schedules shared across locus samples and frames. Values
//! remain in each sampling scope; plans retain no coordinates or results.
use super::{Document, Kind, Point};
use lru::LruCache;
use sketchpad_ir::{DependencyIndex, DependencyNode};
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct Node {
    id: usize,
    page: usize,
    dependencies: Vec<usize>,
}
impl DependencyNode for Node {
    fn id(&self) -> usize {
        self.id
    }
    fn page(&self) -> usize {
        self.page
    }
    fn dependencies(&self) -> Vec<usize> {
        self.dependencies.clone()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct PlanKey {
    driver: usize,
    roots: Vec<usize>,
    path: Option<usize>,
}

/// Shared by a document and its evaluated views. Only immutable schedules are
/// retained, never sample documents or values. Bound memory across graph edits.
#[derive(Clone, Debug)]
pub(super) struct SamplingPlans(Arc<Mutex<LruCache<PlanKey, Arc<SamplingPlan>>>>);

impl Default for SamplingPlans {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(LruCache::new(
            NonZeroUsize::new(128).unwrap(),
        ))))
    }
}

#[derive(Debug)]
pub(super) struct SamplingPlan {
    driver: usize,
    nodes: Vec<Node>,
    affected: Vec<usize>,
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
            .0
            .lock()
            .ok()
            .and_then(|mut cache| cache.get(&key).cloned());
        if let Some(plan) = cached.filter(|plan| plan.matches(doc)) {
            return Some(plan);
        }
        let plan = Arc::new(Self::build(doc, driver, roots, path)?);
        if let Ok(mut cache) = doc.sampling_plans.0.lock() {
            cache.put(key, plan.clone());
        }
        Some(plan)
    }

    // Objects are publicly editable, so validate actual edges instead of relying
    // on UI dirty flags. Coordinate changes retain the plan; rewiring rebuilds it.
    fn matches(&self, doc: &Document) -> bool {
        self.nodes.iter().all(|node| {
            doc.objects.get(node.id - 1).is_some_and(|object| {
                object.id == node.id
                    && object.page == node.page
                    && object.construction_dependencies() == node.dependencies
            })
        })
    }

    fn build(doc: &Document, driver: usize, roots: &[usize], path: Option<usize>) -> Option<Self> {
        // Follow actual edges, including forward references. Unrelated objects
        // elsewhere in the sketch do not contribute to this sampling program.
        let mut nodes = BTreeMap::new();
        let mut pending = roots.to_vec();
        pending.push(driver);
        while let Some(id) = pending.pop() {
            if nodes.contains_key(&id) {
                continue;
            }
            let object = doc.objects.get(id.checked_sub(1)?)?;
            if object.id != id {
                return None;
            }
            let dependencies = object.construction_dependencies();
            pending.extend(dependencies.iter().copied());
            nodes.insert(
                id,
                Node {
                    id,
                    page: object.page,
                    dependencies,
                },
            );
        }
        let nodes = nodes.into_values().collect::<Vec<_>>();
        let index = DependencyIndex::build(&nodes).ok()?;
        let affected = index.affected_by(driver);
        let changing: BTreeSet<_> = affected.iter().copied().collect();
        let mut external: BTreeSet<_> = path.into_iter().collect();
        for id in roots.iter().filter(|id| changing.contains(id)) {
            external.extend(
                index.forward[id]
                    .iter()
                    .copied()
                    .filter(|id| !changing.contains(id)),
            );
        }
        Some(Self {
            driver,
            nodes,
            affected,
            external: external.into_iter().collect(),
        })
    }

    fn invalidate(&self, sample: &mut Document) {
        sample.invalidate_evaluation_ids(self.driver, self.affected.iter().copied());
    }

    pub(super) fn point(&self, sample: &mut Document, point: Point) -> Option<()> {
        self.invalidate(sample);
        sample.set_evaluation_point(self.driver, point)?;
        sample.cache_evaluation_values(&self.affected)
    }

    pub(super) fn kind(&self, sample: &mut Document, kind: Kind) -> Option<()> {
        self.invalidate(sample);
        sample.objects[self.driver - 1].kind = kind;
        sample.cache_evaluation_values(&self.affected)
    }

    pub(super) fn number(&self, sample: &mut Document, value: f64) -> Option<()> {
        self.invalidate(sample);
        sample.seed_evaluation_number(self.driver, value)?;
        sample.cache_evaluation_values(&self.affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        doc.objects[6].midpoint_host = None;
        doc.objects[6].kind = Kind::Midpoint(4, later);
        let rewired = SamplingPlan::new(&doc, 4, &refs, Some(3)).unwrap();
        assert!(!Arc::ptr_eq(&original, &rewired));
        assert_ne!(doc.locus_segments(8, 8), moved_view.locus_segments(8, 8));
        doc.undo();
        assert_eq!(doc.locus_segments(8, 8), moved_view.locus_segments(8, 8));

        doc.objects[6].midpoint_host = None;
        doc.objects[6].kind = Kind::Midpoint(7, 7);
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
            doc.objects[3].kind = kind.clone();
            plan.kind(&mut sample, kind).unwrap();
            assert_eq!(sample.position(7), doc.position(7));
        }
    }
}
