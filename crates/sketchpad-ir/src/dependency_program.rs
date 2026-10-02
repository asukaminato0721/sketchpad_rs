//! Value-independent dependency programs for incremental and sampled evaluation.
use crate::{DependencyError, DependencyIndex, DependencyNode};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, OnceLock};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Signature {
    id: usize,
    page: usize,
    dependencies: Vec<usize>,
}

/// Connections recorded when construction inputs are attached or replaced.
/// Validation and topological scheduling are deferred until a program is needed,
/// so imports and batch edits can contain forward references temporarily.
#[derive(Clone, Debug, Default)]
pub struct DependencyGraph {
    nodes: Vec<Signature>,
    compiled: OnceLock<Result<Arc<DependencyProgram>, DependencyError>>,
}

impl DependencyGraph {
    pub fn register(&mut self, node: &impl DependencyNode) {
        self.nodes.push(Signature::of(node));
        self.compiled.take();
    }

    pub fn matches_at(&self, slot: usize, node: &impl DependencyNode) -> bool {
        self.nodes
            .get(slot)
            .is_some_and(|old| *old == Signature::of(node))
    }

    pub fn reconnect(&mut self, slot: usize, node: &impl DependencyNode) {
        let signature = Signature::of(node);
        if self.nodes[slot] != signature {
            self.nodes[slot] = signature;
            self.compiled.take();
        }
    }

    pub fn program(&self) -> Result<Arc<DependencyProgram>, DependencyError> {
        self.compiled
            .get_or_init(|| DependencyProgram::from_signatures(self.nodes.clone()).map(Arc::new))
            .clone()
    }

    pub fn slice(&self, roots: &[usize]) -> Result<DependencyProgram, DependencyError> {
        DependencyProgram::compile_slice(&self.nodes, roots)
    }
}

impl Signature {
    fn of(node: &impl DependencyNode) -> Self {
        Self {
            id: node.id(),
            page: node.page(),
            dependencies: node.dependencies(),
        }
    }
}

impl DependencyNode for Signature {
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

/// An immutable construction graph. Input values and evaluation caches belong
/// to the caller, so one program can serve many independent sampling scopes.
#[derive(Clone, Debug)]
pub struct DependencyProgram {
    nodes: BTreeMap<usize, Signature>,
    index: DependencyIndex,
}

/// A specialized update for a set of changed inputs. Each affected node appears
/// once, after its dependencies. Boundaries retain the caller's current values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdatePlan {
    inputs: Vec<usize>,
    order: Vec<usize>,
    boundaries: Vec<usize>,
}

impl UpdatePlan {
    pub fn inputs(&self) -> &[usize] {
        &self.inputs
    }
    pub fn order(&self) -> &[usize] {
        &self.order
    }
    pub fn boundaries(&self) -> &[usize] {
        &self.boundaries
    }
}

impl DependencyProgram {
    pub fn compile(nodes: &[impl DependencyNode]) -> Result<Self, DependencyError> {
        Self::from_signatures(nodes.iter().map(Signature::of).collect())
    }

    /// Compile only the ancestors of the requested outputs, including those
    /// outputs. Unrelated malformed edges do not prevent a valid slice running.
    pub fn compile_slice(
        nodes: &[impl DependencyNode],
        roots: &[usize],
    ) -> Result<Self, DependencyError> {
        let mut lookup = BTreeMap::new();
        for node in nodes {
            if lookup.insert(node.id(), node).is_some() {
                return Err(DependencyError::DuplicateId { id: node.id() });
            }
        }
        let mut selected = BTreeMap::new();
        let mut pending: Vec<_> = roots.iter().map(|&id| (id, id)).collect();
        while let Some((child, id)) = pending.pop() {
            if selected.contains_key(&id) {
                continue;
            }
            let node = lookup.get(&id).ok_or(DependencyError::MissingDependency {
                node: child,
                dependency: id,
            })?;
            let signature = Signature::of(*node);
            pending.extend(signature.dependencies.iter().map(|&parent| (id, parent)));
            selected.insert(id, signature);
        }
        Self::from_signatures(selected.into_values().collect())
    }

    fn from_signatures(nodes: Vec<Signature>) -> Result<Self, DependencyError> {
        let index = DependencyIndex::build(&nodes)?;
        Ok(Self {
            nodes: nodes.into_iter().map(|node| (node.id, node)).collect(),
            index,
        })
    }

    pub fn order(&self) -> &[usize] {
        &self.index.topological
    }

    pub fn index(&self) -> &DependencyIndex {
        &self.index
    }

    /// Compare identities and edges, independently of coordinates and values.
    pub fn matches_node(&self, node: &impl DependencyNode) -> bool {
        self.nodes
            .get(&node.id())
            .is_some_and(|old| old.page == node.page() && old.dependencies == node.dependencies())
    }

    /// Changes propagate through every declared dependency, irrespective of the
    /// operation type. Unknown input IDs are ignored, as in affected_by().
    pub fn update(&self, inputs: &[usize]) -> UpdatePlan {
        let inputs: BTreeSet<_> = inputs
            .iter()
            .copied()
            .filter(|id| self.index.reverse.contains_key(id))
            .collect();
        let mut affected = BTreeSet::new();
        let mut pending: Vec<_> = inputs.iter().copied().collect();
        while let Some(id) = pending.pop() {
            if affected.insert(id) {
                pending.extend(&self.index.reverse[&id]);
            }
        }
        let boundaries: BTreeSet<_> = affected
            .iter()
            .flat_map(|id| &self.index.forward[id])
            .copied()
            .filter(|id| !affected.contains(id))
            .collect();
        UpdatePlan {
            inputs: inputs.into_iter().collect(),
            order: self
                .order()
                .iter()
                .copied()
                .filter(|id| affected.contains(id))
                .collect(),
            boundaries: boundaries.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: usize, dependencies: &[usize]) -> Signature {
        Signature {
            id,
            page: 0,
            dependencies: dependencies.to_vec(),
        }
    }

    #[test]
    fn overlapping_changes_merge_into_one_ordered_update() {
        let nodes = [
            node(90, &[30, 50]),
            node(30, &[10, 20]),
            node(50, &[20]),
            node(10, &[]),
            node(20, &[]),
            node(99, &[]),
        ];
        let program = DependencyProgram::compile(&nodes).unwrap();
        let update = program.update(&[20, 10, 20, 1000]);
        assert_eq!(update.inputs(), &[10, 20]);
        assert_eq!(update.order(), &[10, 20, 30, 50, 90]);
        assert!(update.boundaries().is_empty());
        let partial = program.update(&[10]);
        assert_eq!(partial.order(), &[10, 30, 90]);
        assert_eq!(partial.boundaries(), &[20, 50]);
    }

    #[test]
    fn slicing_follows_forward_references_and_detects_rewiring() {
        let nodes = [node(1, &[8]), node(8, &[]), node(99, &[100])];
        assert!(DependencyProgram::compile(&nodes).is_err());
        let program = DependencyProgram::compile_slice(&nodes, &[1]).unwrap();
        assert_eq!(program.order(), &[8, 1]);
        assert!(program.matches_node(&nodes[0]));
        assert!(!program.matches_node(&node(1, &[99])));
        assert!(DependencyProgram::compile_slice(&[node(1, &[8]), node(8, &[1])], &[1]).is_err());
        assert_eq!(
            DependencyProgram::compile_slice(&[node(1, &[]), node(1, &[])], &[1]).unwrap_err(),
            DependencyError::DuplicateId { id: 1 },
        );
    }
}
