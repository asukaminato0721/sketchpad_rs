//! The mutation boundary for construction objects. No mutable slice escapes:
//! every insertion or completed edit records connections and a change event.
use super::Object;
use sketchpad_ir::{DependencyError, DependencyGraph, DependencyProgram};
use std::collections::{BTreeSet, VecDeque};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

const HISTORY_LIMIT: usize = 256;

#[derive(Clone, Debug, Default)]
struct Tracking {
    graph: Arc<DependencyGraph>,
    revision: Arc<()>,
    // Each entry identifies the revision before a mutation. Shared tokens make
    // cloned branches distinguishable without a global counter or shared state.
    history: VecDeque<(Arc<()>, usize)>,
}

impl Tracking {
    fn changed(&mut self, id: usize) {
        self.history.push_back((self.revision.clone(), id));
        self.revision = Arc::new(());
        if self.history.len() > HISTORY_LIMIT {
            self.history.pop_front();
        }
    }
}

/// Read as a slice; mutate with `edit`, `get_mut`, or `iter_mut`. Editing guards
/// register the resulting inputs on drop, including early returns and unwinding.
/// Untracked mutable indexing is deliberately unavailable:
/// ```compile_fail
/// use sketchpad_engine::document::{Document, Kind, Point};
/// let mut doc = Document::new();
/// doc.add(Kind::Free(Point { x: 0., y: 0. }));
/// doc.objects[0].deleted = true;
/// ```
#[derive(Debug)]
pub struct ObjectStore {
    // Evaluation scopes only substitute values. Share their construction table
    // until an actual edit goes through one of the mutation guards below.
    objects: Arc<Vec<Object>>,
    tracking: Mutex<Tracking>,
}

impl Clone for ObjectStore {
    fn clone(&self) -> Self {
        Self {
            objects: self.objects.clone(),
            tracking: Mutex::new(self.tracking.lock().unwrap().clone()),
        }
    }
}

impl PartialEq for ObjectStore {
    fn eq(&self, other: &Self) -> bool {
        self.objects == other.objects
    }
}

impl From<Vec<Object>> for ObjectStore {
    fn from(objects: Vec<Object>) -> Self {
        let mut graph = DependencyGraph::default();
        for object in &objects {
            graph.register(object);
        }
        Self {
            objects: Arc::new(objects),
            tracking: Mutex::new(Tracking {
                graph: Arc::new(graph),
                ..Tracking::default()
            }),
        }
    }
}

impl Deref for ObjectStore {
    type Target = [Object];
    fn deref(&self) -> &Self::Target {
        &self.objects
    }
}

impl ObjectStore {
    pub(super) fn shares_objects(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.objects, &other.objects)
    }

    pub fn push(&mut self, object: Object) {
        let tracking = self.tracking.get_mut().unwrap();
        Arc::make_mut(&mut tracking.graph).register(&object);
        tracking.changed(object.id);
        Arc::make_mut(&mut self.objects).push(object);
    }

    pub fn edit(&mut self, slot: usize) -> ObjectEdit<'_> {
        self.get_mut(slot).expect("object slot out of bounds")
    }

    pub fn get_mut(&mut self, slot: usize) -> Option<ObjectEdit<'_>> {
        Some(ObjectEdit {
            object: Arc::make_mut(&mut self.objects).get_mut(slot)?,
            slot,
            tracking: &self.tracking,
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = ObjectEdit<'_>> {
        Arc::make_mut(&mut self.objects)
            .iter_mut()
            .enumerate()
            .map(|(slot, object)| ObjectEdit {
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

impl<'a> IntoIterator for &'a ObjectStore {
    type Item = &'a Object;
    type IntoIter = std::slice::Iter<'a, Object>;
    fn into_iter(self) -> Self::IntoIter {
        self.objects.iter()
    }
}

impl IntoIterator for ObjectStore {
    type Item = Object;
    type IntoIter = std::vec::IntoIter<Object>;
    fn into_iter(self) -> Self::IntoIter {
        Arc::unwrap_or_clone(self.objects).into_iter()
    }
}

/// A scoped edit. The store cannot be evaluated while this guard is borrowed.
pub struct ObjectEdit<'a> {
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
        if !tracking.graph.matches_at(self.slot, self.object) {
            Arc::make_mut(&mut tracking.graph).reconnect(self.slot, self.object);
        }
        tracking.changed(self.object.id);
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
