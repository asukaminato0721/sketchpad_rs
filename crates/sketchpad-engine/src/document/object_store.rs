//! The mutation boundary for construction objects. No mutable slice escapes:
//! every insertion or completed edit records connections and a change event.
use super::sac_evaluation::{Identity, Input, observe};
use super::{Kind, Object};
use sketchpad_ir::{DependencyError, DependencyGraph, DependencyProgram};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ops::{Deref, DerefMut, Index};
use std::sync::{Arc, Mutex, OnceLock};

const HISTORY_LIMIT: usize = 256;

/// Helpers shared by loading and evaluation keep per-object reads observable.
/// Taking a raw slice from ObjectStore would register the entire table instead.
pub(super) trait ObjectRead: Index<usize, Output = Object> {
    fn get(&self, slot: usize) -> Option<&Object>;
    fn len(&self) -> usize;
}
impl ObjectRead for [Object] {
    fn len(&self) -> usize {
        <[Object]>::len(self)
    }
    fn get(&self, slot: usize) -> Option<&Object> {
        <[Object]>::get(self, slot)
    }
}
impl ObjectRead for Vec<Object> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
    fn get(&self, slot: usize) -> Option<&Object> {
        self.as_slice().get(slot)
    }
}
impl ObjectRead for ObjectStore {
    fn len(&self) -> usize {
        ObjectStore::len(self)
    }
    fn get(&self, slot: usize) -> Option<&Object> {
        ObjectStore::get(self, slot)
    }
}

// Stamp verification must inspect substitutions without observing new inputs.
struct Untracked<'a>(&'a ObjectStore);
impl Index<usize> for Untracked<'_> {
    type Output = Object;
    fn index(&self, slot: usize) -> &Object {
        self.0
            .get_untracked(slot)
            .expect("object slot out of bounds")
    }
}
impl ObjectRead for Untracked<'_> {
    fn get(&self, slot: usize) -> Option<&Object> {
        self.0.get_untracked(slot)
    }
    fn len(&self) -> usize {
        self.0.untracked_len()
    }
}

/// Iteration includes temporary substitutions without materializing a slice.
pub struct ObjectIter<'a> {
    store: &'a ObjectStore,
    slots: std::ops::Range<usize>,
}
impl<'a> Iterator for ObjectIter<'a> {
    type Item = &'a Object;
    fn next(&mut self) -> Option<Self::Item> {
        self.store.get_untracked(self.slots.next()?)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.slots.size_hint()
    }
}
impl DoubleEndedIterator for ObjectIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.store.get_untracked(self.slots.next_back()?)
    }
}
impl ExactSizeIterator for ObjectIter<'_> {}
impl std::iter::FusedIterator for ObjectIter<'_> {}

#[derive(Clone, Debug, Default)]
struct Tracking {
    graph: Arc<DependencyGraph>,
    revision: Arc<()>,
    object_revisions: Arc<Vec<Arc<()>>>,
    validation_revision: Arc<()>,
    coordinate_systems: Arc<Vec<usize>>,
    coordinate_revision: Arc<()>,
    mapped_validity: Arc<Mutex<rustc_hash::FxHashMap<usize, bool>>>,
    // Each entry identifies the revision before a mutation. Shared tokens make
    // cloned branches distinguishable without a global counter or shared state.
    history: VecDeque<(Arc<()>, usize)>,
}

impl Tracking {
    fn coordinate_membership(&mut self, slot: usize, present: bool) {
        match (self.coordinate_systems.binary_search(&slot), present) {
            (Err(index), true) => Arc::make_mut(&mut self.coordinate_systems).insert(index, slot),
            (Ok(index), false) => {
                Arc::make_mut(&mut self.coordinate_systems).remove(index);
            }
            _ => return,
        }
        self.coordinate_revision = Arc::new(());
    }

    fn changed(&mut self, id: usize, slot: usize) {
        self.changed_revision(id);
        let versions = Arc::make_mut(&mut self.object_revisions);
        if slot == versions.len() {
            versions.push(self.revision.clone());
        } else if let Some(version) = versions.get_mut(slot) {
            *version = self.revision.clone();
        }
    }

    fn changed_revision(&mut self, id: usize) {
        self.mapped_validity = Arc::new(Mutex::new(Default::default()));
        self.history.push_back((self.revision.clone(), id));
        self.revision = Arc::new(());
        if self.history.len() > HISTORY_LIMIT {
            self.history.pop_front();
        }
    }

    fn reconnect(
        &mut self,
        slot: usize,
        object: &Object,
        validity: (bool, std::mem::Discriminant<Kind>),
    ) {
        if !self.graph.matches_at(slot, object) {
            Arc::make_mut(&mut self.graph).reconnect(slot, object);
            self.validation_revision = Arc::new(());
        } else if validity != (object.deleted, std::mem::discriminant(&object.kind)) {
            self.validation_revision = Arc::new(());
        }
        self.coordinate_membership(slot, matches!(object.kind, Kind::CoordinateSystem(..)));
    }
}

#[derive(Clone, Debug)]
struct Substitution {
    object: Arc<Object>,
    revision: Arc<()>,
}

/// Read as a slice; mutate with `edit`, `get_mut`, or `iter_mut`. Editing guards
/// register the resulting inputs on drop, including early returns and unwinding.
/// Untracked mutable indexing is deliberately unavailable:
/// ```compile_fail
/// use sketchpad_engine::document::{Document, Kind, Point};
/// let mut doc = Document::new();
/// doc.add(Kind::Free(Point { x: 0., y: 0. }));
/// doc.objects()[0].deleted = true;
/// ```
#[derive(Debug)]
pub struct ObjectStore {
    // Sampled constraints retain their parameters as well as their coordinates.
    // Keep those few replacements separate from the shared construction table.
    objects: Arc<Vec<Object>>,
    substitutions: Option<Arc<BTreeMap<usize, Substitution>>>,
    materialized: OnceLock<Arc<Vec<Object>>>,
    tracking: Mutex<Tracking>,
}

impl Clone for ObjectStore {
    fn clone(&self) -> Self {
        Self {
            objects: self.objects.clone(),
            substitutions: self.substitutions.clone(),
            materialized: self.materialized.clone(),
            tracking: Mutex::new(self.tracking.lock().unwrap().clone()),
        }
    }
}

impl PartialEq for ObjectStore {
    fn eq(&self, other: &Self) -> bool {
        self.untracked_iter().eq(other.untracked_iter())
    }
}

impl From<Vec<Object>> for ObjectStore {
    fn from(objects: Vec<Object>) -> Self {
        let object_revisions = Arc::new(objects.iter().map(|_| Arc::new(())).collect());
        let coordinate_systems = Arc::new(
            objects
                .iter()
                .enumerate()
                .filter_map(|(slot, object)| {
                    matches!(object.kind, super::Kind::CoordinateSystem(..)).then_some(slot)
                })
                .collect(),
        );
        let mut graph = DependencyGraph::default();
        for object in &objects {
            graph.register(object);
        }
        Self {
            objects: Arc::new(objects),
            substitutions: None,
            materialized: OnceLock::new(),
            tracking: Mutex::new(Tracking {
                graph: Arc::new(graph),
                object_revisions,
                coordinate_systems,
                ..Tracking::default()
            }),
        }
    }
}

impl Deref for ObjectStore {
    type Target = [Object];
    fn deref(&self) -> &Self::Target {
        observe(Input::Objects);
        self.as_slice_untracked()
    }
}

impl ObjectStore {
    fn get_untracked(&self, slot: usize) -> Option<&Object> {
        self.substitutions
            .as_ref()
            .and_then(|objects| objects.get(&slot))
            .map(|entry| entry.object.as_ref())
            .or_else(|| self.objects.get(slot))
    }

    fn untracked_iter(&self) -> ObjectIter<'_> {
        ObjectIter {
            store: self,
            slots: 0..self.objects.len(),
        }
    }

    // Slice compatibility is paid for only by callers that require contiguity.
    fn as_slice_untracked(&self) -> &[Object] {
        if self.substitutions.is_none() {
            return &self.objects;
        }
        self.materialized
            .get_or_init(|| Arc::new(self.untracked_iter().cloned().collect()))
    }

    fn materialize(&mut self) {
        if self.substitutions.is_none() {
            return;
        }
        let objects = self
            .materialized
            .take()
            .unwrap_or_else(|| Arc::new(self.untracked_iter().cloned().collect()));
        let substitutions = self.substitutions.take().unwrap();
        let versions = Arc::make_mut(&mut self.tracking.get_mut().unwrap().object_revisions);
        for (&slot, entry) in substitutions.iter() {
            versions[slot] = entry.revision.clone();
        }
        self.objects = objects;
    }

    /// Sampling changes a constraint without copying all objects or all stamps.
    /// Every read API sees the replacement, including direct parameter reads.
    pub(super) fn substitute_kind(&mut self, slot: usize, kind: Kind) -> Option<()> {
        let original = self.objects.get(slot)?;
        self.materialized.take();
        let entries = Arc::make_mut(self.substitutions.get_or_insert_with(Default::default));
        let entry = entries.entry(slot).or_insert_with(|| Substitution {
            object: Arc::new(original.clone()),
            revision: Arc::new(()),
        });
        let object = Arc::make_mut(&mut entry.object);
        let validity = (object.deleted, std::mem::discriminant(&object.kind));
        object.kind = kind;
        let tracking = self.tracking.get_mut().unwrap();
        tracking.reconnect(slot, object, validity);
        tracking.changed_revision(object.id);
        entry.revision = tracking.revision.clone();
        Some(())
    }

    /// Pure construction validation is shared by scopes over this object table.
    /// Mutation replaces the cache, preserving old snapshots. This raw accessor
    /// also serves input-stamp validation and must not record nested reads.
    pub(super) fn mapped_element_validity(&self, id: usize) -> bool {
        let Some(object) = id.checked_sub(1).and_then(|slot| self.get_untracked(slot)) else {
            return false;
        };
        let super::Kind::MappedElement(definition) = &object.kind else {
            return false;
        };
        let cache = self.tracking.lock().unwrap().mapped_validity.clone();
        if let Some(valid) = cache.lock().unwrap().get(&id) {
            return *valid;
        }
        let valid = !object.deleted && definition.supported(&Untracked(self), object.page, id);
        cache.lock().unwrap().insert(id, valid);
        valid
    }

    pub(super) fn sac_identity(&self, slot: Option<usize>) -> Identity {
        if let Some(entry) = slot.and_then(|slot| self.substitutions.as_ref()?.get(&slot)) {
            return Identity(entry.revision.clone());
        }
        let tracking = self.tracking.lock().unwrap();
        Identity(
            slot.and_then(|slot| tracking.object_revisions.get(slot))
                .unwrap_or(&tracking.revision)
                .clone(),
        )
    }
    pub(super) fn sac_validation_identity(&self) -> Identity {
        Identity(self.tracking.lock().unwrap().validation_revision.clone())
    }
    pub(super) fn sac_coordinate_identity(&self) -> Identity {
        Identity(self.tracking.lock().unwrap().coordinate_revision.clone())
    }
    /// Track the candidate set separately from the contents of unrelated objects.
    pub(super) fn coordinate_systems(&self) -> impl DoubleEndedIterator<Item = &Object> {
        observe(Input::CoordinateSystems);
        let slots = self.tracking.lock().unwrap().coordinate_systems.clone();
        (0..slots.len()).map(move |index| &self[slots[index]])
    }
    pub(super) fn untracked_len(&self) -> usize {
        self.objects.len()
    }
    pub fn len(&self) -> usize {
        observe(Input::Length);
        self.objects.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn get(&self, slot: usize) -> Option<&Object> {
        observe(Input::Object(slot));
        self.get_untracked(slot)
    }
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Object> + ExactSizeIterator {
        observe(Input::Length);
        self.untracked_iter().enumerate().map(|(slot, object)| {
            observe(Input::Object(slot));
            object
        })
    }
    pub(super) fn shares_objects(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.objects, &other.objects)
            && match (&self.substitutions, &other.substitutions) {
                (None, None) => true,
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                _ => false,
            }
    }

    pub fn push(&mut self, object: Object) {
        self.materialize();
        let tracking = self.tracking.get_mut().unwrap();
        Arc::make_mut(&mut tracking.graph).register(&object);
        tracking.validation_revision = Arc::new(());
        tracking.coordinate_membership(
            self.objects.len(),
            matches!(object.kind, super::Kind::CoordinateSystem(..)),
        );
        tracking.changed(object.id, self.objects.len());
        Arc::make_mut(&mut self.objects).push(object);
    }

    pub fn edit(&mut self, slot: usize) -> ObjectEdit<'_> {
        self.get_mut(slot).expect("object slot out of bounds")
    }

    pub fn get_mut(&mut self, slot: usize) -> Option<ObjectEdit<'_>> {
        self.materialize();
        let object = Arc::make_mut(&mut self.objects).get_mut(slot)?;
        let validity = (object.deleted, std::mem::discriminant(&object.kind));
        Some(ObjectEdit {
            object,
            validity,
            slot,
            tracking: &self.tracking,
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = ObjectEdit<'_>> {
        self.materialize();
        Arc::make_mut(&mut self.objects)
            .iter_mut()
            .enumerate()
            .map(|(slot, object)| ObjectEdit {
                validity: (object.deleted, std::mem::discriminant(&object.kind)),
                object,
                slot,
                tracking: &self.tracking,
            })
    }

    pub fn dependency_program(&self) -> Result<Arc<DependencyProgram>, DependencyError> {
        self.dependency_graph().program()
    }

    pub(super) fn dependency_graph(&self) -> Arc<DependencyGraph> {
        self.tracking.lock().unwrap().graph.clone()
    }

    /// No object comparison. Missing history or a divergent clone requests a
    /// fresh evaluation; a slow consumer can never retain stale results.
    pub(super) fn changes_since(&self, previous: &Self) -> Option<Vec<usize>> {
        let previous_revision = previous.tracking.lock().unwrap().revision.clone();
        let tracking = self.tracking.lock().unwrap();
        if Arc::ptr_eq(&tracking.revision, &previous_revision) {
            return Some(Vec::new());
        }
        let start = tracking
            .history
            .iter()
            .position(|(revision, _)| Arc::ptr_eq(revision, &previous_revision))?;
        Some(
            tracking
                .history
                .iter()
                .skip(start)
                .map(|(_, id)| *id)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        )
    }
}

impl Index<usize> for ObjectStore {
    type Output = Object;
    fn index(&self, slot: usize) -> &Object {
        observe(Input::Object(slot));
        self.get_untracked(slot).expect("object slot out of bounds")
    }
}

macro_rules! slice_index {
    ($($index:ty),*) => {$(
        impl Index<$index> for ObjectStore {
            type Output = [Object];
            fn index(&self, index: $index) -> &[Object] {
                observe(Input::Objects);
                &self.as_slice_untracked()[index]
            }
        }
    )*};
}
slice_index!(
    std::ops::Range<usize>,
    std::ops::RangeFrom<usize>,
    std::ops::RangeTo<usize>,
    std::ops::RangeFull,
    std::ops::RangeInclusive<usize>,
    std::ops::RangeToInclusive<usize>
);

impl<'a> IntoIterator for &'a ObjectStore {
    type Item = &'a Object;
    type IntoIter = ObjectIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        observe(Input::Objects);
        self.untracked_iter()
    }
}

impl IntoIterator for ObjectStore {
    type Item = Object;
    type IntoIter = std::vec::IntoIter<Object>;
    fn into_iter(mut self) -> Self::IntoIter {
        self.materialize();
        Arc::unwrap_or_clone(self.objects).into_iter()
    }
}

/// A scoped edit. The store cannot be evaluated while this guard is borrowed.
pub struct ObjectEdit<'a> {
    validity: (bool, std::mem::Discriminant<Kind>),
    object: &'a mut Object,
    slot: usize,
    tracking: &'a Mutex<Tracking>,
}

impl Deref for ObjectEdit<'_> {
    type Target = Object;
    fn deref(&self) -> &Object {
        self.object
    }
}

impl DerefMut for ObjectEdit<'_> {
    fn deref_mut(&mut self) -> &mut Object {
        self.object
    }
}

impl Drop for ObjectEdit<'_> {
    fn drop(&mut self) {
        let mut tracking = self.tracking.lock().unwrap();
        tracking.reconnect(self.slot, self.object, self.validity);
        tracking.changed(self.object.id, self.slot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Document, EvaluationCache, Kind, Point};

    fn sketch() -> Document {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = doc.add(Kind::Free(Point { x: 10., y: 20. }));
        doc.add(Kind::Midpoint(a, b));
        doc
    }

    #[test]
    fn sparse_substitutions_preserve_snapshots_and_all_object_read_apis() {
        let doc = sketch();
        let mut sample = doc.objects.clone();
        let old_stamp = sample.sac_identity(Some(0));
        let fixed_stamp = sample.sac_identity(Some(1));
        sample
            .substitute_kind(0, Kind::Free(Point { x: 20., y: 30. }))
            .unwrap();
        // Sampling must not detach either full table, even with old views alive.
        assert!(Arc::ptr_eq(&sample.objects, &doc.objects.objects));
        assert!(Arc::ptr_eq(
            &sample.tracking.lock().unwrap().object_revisions,
            &doc.objects.tracking.lock().unwrap().object_revisions,
        ));
        assert_ne!(sample.sac_identity(Some(0)), old_stamp);
        assert_eq!(sample.sac_identity(Some(1)), fixed_stamp);
        assert_eq!(sample.changes_since(&doc.objects), Some(vec![1]));
        let snapshot = sample.clone();
        let mut expected = doc.objects.to_vec();
        expected[0].kind = Kind::Free(Point { x: 20., y: 30. });
        assert_eq!(sample.get(0), Some(&expected[0]));
        assert_eq!(sample[0], expected[0]);
        assert!(sample.iter().eq(expected.iter()));
        assert!((&sample).into_iter().rev().eq(expected.iter().rev()));
        assert!(sample.materialized.get().is_none());
        assert_eq!(&sample[..], expected.as_slice());
        assert_eq!(&*sample, expected.as_slice());
        assert_eq!(sample.clone().into_iter().collect::<Vec<_>>(), expected);

        let stamp = sample.sac_identity(Some(0));
        sample
            .substitute_kind(0, Kind::Free(Point { x: 50., y: 60. }))
            .unwrap();
        assert!(sample.materialized.get().is_none());
        assert_ne!(sample.sac_identity(Some(0)), stamp);
        assert_eq!(snapshot.sac_identity(Some(0)), stamp);
        assert_eq!(snapshot.to_vec(), expected);
        assert_ne!(sample, snapshot);
        assert_eq!(doc.objects[0].kind, Kind::Free(Point { x: 0., y: 0. }));

        // Returning to ordinary edits must retain the sparse values and stamps.
        let stamp = sample.sac_identity(Some(0));
        sample.edit(1).kind = Kind::Free(Point { x: 80., y: 90. });
        assert_eq!(sample.sac_identity(Some(0)), stamp);
        assert_eq!(sample[0].kind, Kind::Free(Point { x: 50., y: 60. }));
        assert_ne!(sample.sac_identity(Some(1)), fixed_stamp);
        assert_eq!(sample.changes_since(&snapshot), Some(vec![1, 2]));
        assert_eq!(snapshot.to_vec(), expected);
    }

    #[test]
    fn sparse_rewiring_and_regular_mutations_keep_the_dependency_graph_current() {
        let doc = sketch();
        let mut sample = doc.objects.clone();
        let old_graph = sample.dependency_graph();
        let old_validation = sample.sac_validation_identity();
        sample.substitute_kind(2, Kind::Midpoint(2, 2)).unwrap();
        assert_eq!(
            sample.dependency_program().unwrap().index().forward[&3],
            [2]
        );
        assert_eq!(old_graph.program().unwrap().index().forward[&3], [1, 2]);
        assert_ne!(sample.sac_validation_identity(), old_validation);
        sample.substitute_kind(2, Kind::Midpoint(3, 3)).unwrap();
        assert!(sample.dependency_program().is_err());
        sample.substitute_kind(2, Kind::Midpoint(1, 4)).unwrap();
        let mut added = doc.objects[0].clone();
        added.id = 4;
        sample.push(added);
        assert_eq!(
            sample.dependency_program().unwrap().index().forward[&3],
            [1, 4]
        );
        sample.substitute_kind(2, Kind::Midpoint(2, 4)).unwrap();
        for mut object in sample.iter_mut().skip(2).take(1) {
            object.kind = Kind::Midpoint(1, 2);
        }
        assert_eq!(
            sample.dependency_program().unwrap().index().forward[&3],
            [1, 2]
        );
        assert!(
            sample
                .substitute_kind(99, Kind::Free(Point { x: 0., y: 0. }))
                .is_none()
        );
    }

    #[test]
    fn connections_are_recorded_on_insertion_and_scoped_rewiring() {
        let mut doc = sketch();
        let graph = doc.objects.dependency_graph();
        let original = graph.program().unwrap();
        assert_eq!(original.index().forward[&3], [1, 2]);
        let old = doc.objects.clone();
        doc.move_point(1, Point { x: 4., y: 2. });
        assert!(Arc::ptr_eq(&graph, &doc.objects.dependency_graph()));
        assert_eq!(doc.objects.changes_since(&old), Some(vec![1]));

        // Holding a guard permits an atomic edit of all construction fields.
        {
            let mut object = doc.objects.edit(2);
            object.kind = Kind::Midpoint(2, 2);
            object.midpoint_host = None;
        }
        let rewired = doc.objects.dependency_program().unwrap();
        assert_eq!(rewired.index().reverse[&1], []);
        assert_eq!(rewired.index().reverse[&2], [3]);
        assert_eq!(original.index().forward[&3], [1, 2]);
        assert_eq!(doc.objects.changes_since(&old), Some(vec![1, 3]));
        assert_eq!(doc.evaluated().position(3), doc.position(2));

        // Forward references may be recorded before the input is inserted.
        doc.objects.edit(2).kind = Kind::Midpoint(1, 4);
        assert!(doc.objects.dependency_program().is_err());
        doc.add(Kind::Free(Point { x: 20., y: 30. }));
        assert_eq!(doc.dependency_index().unwrap().forward[&3], [1, 4]);
        doc.objects.edit(2).kind = Kind::Midpoint(3, 4);
        assert!(matches!(
            doc.objects.dependency_program(),
            Err(DependencyError::Cycle { .. })
        ));
        doc.objects.edit(2).kind = Kind::Midpoint(1, 4);
        assert!(doc.objects.dependency_program().is_ok());
    }

    #[test]
    fn consumers_clones_history_overflow_and_undo_never_share_stale_values() {
        let mut doc = sketch();
        let mut fast = EvaluationCache::default();
        let mut slow = EvaluationCache::default();
        let original = fast.evaluate(&doc);
        slow.evaluate(&doc);
        let baseline = doc.objects.clone();
        let mut branch = doc.clone();
        branch.move_point(1, Point { x: 100., y: 0. });
        doc.checkpoint();
        for step in 1..=HISTORY_LIMIT + 1 {
            doc.move_point(
                1,
                Point {
                    x: step as f64,
                    y: 0.,
                },
            );
            assert_eq!(fast.evaluate(&doc).position(3), doc.position(3));
        }
        assert!(doc.objects.changes_since(&baseline).is_none());
        assert_eq!(slow.evaluate(&doc).position(3), doc.position(3));
        assert!(branch.objects.changes_since(&doc.objects).is_none());
        assert_eq!(fast.evaluate(&branch).position(3), branch.position(3));
        assert_eq!(fast.evaluate(&doc).position(3), doc.position(3));
        assert_eq!(original.position(3), Some(Point { x: 5., y: 10. }));

        doc.undo();
        assert_eq!(fast.evaluate(&doc).position(3), original.position(3));
        doc.redo();
        assert_eq!(
            fast.evaluate(&doc).position(3),
            slow.evaluate(&doc).position(3)
        );

        // Bulk edits and replacement imports also pass the mutation boundary.
        for mut object in doc.objects.iter_mut().take(2) {
            object.kind = Kind::Free(Point { x: 8., y: 6. });
        }
        assert_eq!(
            fast.evaluate(&doc).position(3),
            Some(Point { x: 8., y: 6. })
        );
        let mut objects = doc.objects.to_vec();
        objects[0].kind = Kind::Free(Point { x: 12., y: 6. });
        doc.objects = objects.into();
        assert_eq!(
            fast.evaluate(&doc).position(3),
            Some(Point { x: 10., y: 6. })
        );
    }

    #[test]
    fn scoped_edits_register_connections_on_early_return_and_unwind() {
        fn rewire(doc: &mut Document) -> Result<(), ()> {
            let mut object = doc.objects.get_mut(2).unwrap();
            object.kind = Kind::Midpoint(2, 2);
            Err(())
        }
        let mut doc = sketch();
        assert!(rewire(&mut doc).is_err());
        assert_eq!(doc.dependency_index().unwrap().forward[&3], [2]);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut object = doc.objects.edit(2);
            object.kind = Kind::Midpoint(1, 1);
            panic!("interrupted edit");
        }));
        assert!(result.is_err());
        assert_eq!(doc.dependency_index().unwrap().forward[&3], [1]);
        assert_eq!(doc.evaluated().position(3), doc.position(1));
    }
}
