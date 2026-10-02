//! Derived display geometry belongs to an evaluated scope, just like point and
//! numeric values. Clones share immutable payloads; invalidation stays local.
use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum GeometryQuery {
    Points,
    Polygons,
    ColorizedPolygons,
    Disks,
    ColorizedDisks,
    BranchSegments,
    Lines,
    Segments,
    Rays,
    Arcs,
    Circles,
    MappedStrokes,
    MappedSurfaces,
    LocusSurfaces,
    SampledStrokes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Key {
    id: usize,
    query: GeometryQuery,
    resolution: usize,
    value_type: TypeId,
}

type Entries = lru::LruCache<Key, Arc<dyn Any + Send + Sync>>;
pub(super) struct GeometryCache(Mutex<Entries>);

impl Default for GeometryCache {
    fn default() -> Self {
        Self(Mutex::new(lru::LruCache::new(64.try_into().unwrap())))
    }
}

impl Clone for GeometryCache {
    fn clone(&self) -> Self {
        Self(Mutex::new(self.0.lock().unwrap().clone()))
    }
}

impl std::fmt::Debug for GeometryCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeometryCache")
            .field("entries", &self.0.lock().unwrap().len())
            .finish()
    }
}

impl GeometryCache {
    pub(super) fn get_or_evaluate<T: Clone + Send + Sync + 'static>(
        &self,
        id: usize,
        query: GeometryQuery,
        resolution: usize,
        evaluate: impl FnOnce() -> Option<T>,
    ) -> Option<T> {
        let key = Key {
            id,
            query,
            resolution,
            value_type: TypeId::of::<T>(),
        };
        let cached = self.0.lock().unwrap().get(&key).cloned();
        if let Some(value) = cached {
            return value.downcast_ref::<T>().cloned();
        }
        // Dispatch probes several geometry kinds for each object. Do not let
        // cheap non-matching queries evict expensive, successful collections.
        let value = evaluate()?;
        self.0.lock().unwrap().put(key, Arc::new(value.clone()));
        Some(value)
    }

    pub(super) fn invalidate(&mut self, id: usize) {
        let entries = self.0.get_mut().unwrap();
        let keys: Vec<_> = entries
            .iter()
            .filter_map(|(&key, _)| (key.id == id).then_some(key))
            .collect();
        for key in keys {
            entries.pop(&key);
        }
    }

    pub(super) fn clear(&mut self) {
        self.0.get_mut().unwrap().clear();
    }
}
