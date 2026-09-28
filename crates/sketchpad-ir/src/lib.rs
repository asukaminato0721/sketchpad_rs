//! Native-format-independent object identity and dependency graph.
//!
//! Geometry kinds remain generic. Selection, visual state and native record
//! offsets belong to the engine's wrapper around an [`ObjectNode`].
pub mod affine;
pub use affine::{AffineMatrix, TransformEvaluationError, TransformGeometry, TransformSequence};
pub mod transform_model;
pub mod units;
pub use units::NumericUnit;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectNode<K> {
    pub id: usize,
    pub page: usize,
    pub kind: K,
}

impl<K> ObjectNode<K> {
    pub fn new(id: usize, page: usize, kind: K) -> Self {
        Self { id, page, kind }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
impl Point {
    pub fn distance(self, p: Self) -> f64 {
        (self.x - p.x).hypot(self.y - p.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineMode {
    Segment,
    Line,
    Ray,
}

/// Construction dependencies only; presentation metadata need not form edges.
/// Every dependency must name a node supplied to [`DependencyIndex::build`].
pub trait DependencyNode {
    fn id(&self) -> usize;
    fn page(&self) -> usize;
    fn dependencies(&self) -> Vec<usize>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DependencyError {
    InvalidId,
    DuplicateId {
        id: usize,
    },
    MissingDependency {
        node: usize,
        dependency: usize,
    },
    CrossPageDependency {
        node: usize,
        dependency: usize,
        page: usize,
        dependency_page: usize,
    },
    /// Nodes left after topological elimination, including downstream nodes.
    Cycle {
        nodes: Vec<usize>,
    },
}
impl fmt::Display for DependencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId => write!(f, "Object IDs must be positive"),
            Self::DuplicateId { id } => write!(f, "Duplicate object ID {id}"),
            Self::MissingDependency { node, dependency } => {
                write!(f, "Object {node} references missing object {dependency}")
            }
            Self::CrossPageDependency {
                node,
                dependency,
                page,
                dependency_page,
            } => write!(
                f,
                "Object {node} on page {page} references object {dependency} on page {dependency_page}"
            ),
            Self::Cycle { nodes } => {
                write!(f, "Dependency cycle prevents ordering objects {nodes:?}")
            }
        }
    }
}
impl std::error::Error for DependencyError {}

/// Deterministic graph with dependencies preceding their dependents.
///
/// IDs are identities, not positions or a presumed topological order. Duplicate
/// edges are coalesced; missing or cross-page references are rejected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyIndex {
    /// Node → its immediate dependencies, sorted by ID.
    pub forward: BTreeMap<usize, Vec<usize>>,
    /// Node → its immediate dependents, sorted by ID.
    pub reverse: BTreeMap<usize, Vec<usize>>,
    /// A deterministic topological order; smallest ID wins among ready nodes.
    pub topological: Vec<usize>,
}
impl DependencyIndex {
    pub fn build(nodes: &[impl DependencyNode]) -> Result<Self, DependencyError> {
        let mut pages = BTreeMap::new();
        for node in nodes {
            let id = node.id();
            if id == 0 {
                return Err(DependencyError::InvalidId);
            }
            if pages.insert(id, node.page()).is_some() {
                return Err(DependencyError::DuplicateId { id });
            }
        }
        let mut forward = BTreeMap::new();
        let mut reverse: BTreeMap<usize, BTreeSet<usize>> =
            pages.keys().map(|&id| (id, BTreeSet::new())).collect();
        for node in nodes {
            let id = node.id();
            let dependencies: BTreeSet<_> = node.dependencies().into_iter().collect();
            for &dependency in &dependencies {
                let Some(&dependency_page) = pages.get(&dependency) else {
                    return Err(DependencyError::MissingDependency {
                        node: id,
                        dependency,
                    });
                };
                if dependency_page != node.page() {
                    return Err(DependencyError::CrossPageDependency {
                        node: id,
                        dependency,
                        page: node.page(),
                        dependency_page,
                    });
                }
                reverse.get_mut(&dependency).unwrap().insert(id);
            }
            forward.insert(id, dependencies.into_iter().collect::<Vec<_>>());
        }
        let reverse: BTreeMap<_, Vec<_>> = reverse
            .into_iter()
            .map(|(id, dependents)| (id, dependents.into_iter().collect()))
            .collect();
        let mut remaining: BTreeMap<_, _> = forward
            .iter()
            .map(|(&id, dependencies)| (id, dependencies.len()))
            .collect();
        let mut ready: BTreeSet<_> = remaining
            .iter()
            .filter_map(|(&id, &count)| (count == 0).then_some(id))
            .collect();
        let mut topological = Vec::with_capacity(nodes.len());
        while let Some(id) = ready.pop_first() {
            topological.push(id);
            for dependent in &reverse[&id] {
                let count = remaining.get_mut(dependent).unwrap();
                *count -= 1;
                if *count == 0 {
                    ready.insert(*dependent);
                }
            }
        }
        if topological.len() != nodes.len() {
            return Err(DependencyError::Cycle {
                nodes: remaining
                    .into_iter()
                    .filter_map(|(id, count)| (count > 0).then_some(id))
                    .collect(),
            });
        }
        Ok(Self {
            forward,
            reverse,
            topological,
        })
    }

    /// Return this node and all transitive dependents in topological order.
    /// Unknown IDs return an empty vector.
    pub fn affected_by(&self, id: usize) -> Vec<usize> {
        if !self.reverse.contains_key(&id) {
            return Vec::new();
        }
        let mut affected = BTreeSet::new();
        let mut pending = vec![id];
        while let Some(node) = pending.pop() {
            if affected.insert(node) {
                pending.extend(&self.reverse[&node]);
            }
        }
        self.topological
            .iter()
            .copied()
            .filter(|id| affected.contains(id))
            .collect()
    }
}
