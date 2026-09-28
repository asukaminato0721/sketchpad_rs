use sketchpad_ir::{DependencyError, DependencyIndex, DependencyNode, ObjectNode, Point};

#[derive(Clone)]
struct TestNode(ObjectNode<Vec<usize>>);

impl DependencyNode for TestNode {
    fn id(&self) -> usize {
        self.0.id
    }
    fn page(&self) -> usize {
        self.0.page
    }
    fn dependencies(&self) -> Vec<usize> {
        self.0.kind.clone()
    }
}
fn node(id: usize, parents: &[usize]) -> TestNode {
    TestNode(ObjectNode::new(id, 1, parents.to_vec()))
}

#[test]
fn branching_graph_orders_dependencies_without_assuming_id_order() {
    // Two branches reconverge; the ancestor's ID exceeds both child IDs.
    let nodes = [
        node(3, &[2, 1]),
        node(1, &[9]),
        node(9, &[]),
        node(2, &[9]),
        node(8, &[]),
    ];
    let index = DependencyIndex::build(&nodes).unwrap();
    assert_eq!(index.topological, [8, 9, 1, 2, 3]);
    assert_eq!(index.forward[&3], [1, 2]);
    assert_eq!(index.reverse[&9], [1, 2]);
    assert_eq!(index.affected_by(9), [9, 1, 2, 3]);
    assert_eq!(index.affected_by(1), [1, 3]);
    assert_eq!(index.affected_by(8), [8]);
    assert!(index.affected_by(100).is_empty());
    let reversed: Vec<_> = nodes.into_iter().rev().collect();
    assert_eq!(DependencyIndex::build(&reversed).unwrap(), index);
}

#[test]
fn missing_or_removed_parent_is_rejected() {
    assert_eq!(
        DependencyIndex::build(&[node(1, &[2])]),
        Err(DependencyError::MissingDependency {
            node: 1,
            dependency: 2
        })
    );
    assert_eq!(
        DependencyIndex::build(&[node(1, &[0])]),
        Err(DependencyError::MissingDependency {
            node: 1,
            dependency: 0
        })
    );
}

#[test]
fn cycles_and_self_references_are_rejected() {
    assert_eq!(
        DependencyIndex::build(&[node(1, &[2]), node(2, &[1]), node(3, &[2]), node(4, &[])]),
        Err(DependencyError::Cycle {
            nodes: vec![1, 2, 3]
        })
    );
    assert_eq!(
        DependencyIndex::build(&[node(8, &[8])]),
        Err(DependencyError::Cycle { nodes: vec![8] })
    );
}

#[test]
fn positive_unique_id_and_page_boundaries_are_enforced() {
    assert_eq!(
        DependencyIndex::build(&[node(0, &[])]),
        Err(DependencyError::InvalidId)
    );
    assert_eq!(
        DependencyIndex::build(&[node(1, &[]), node(1, &[])]),
        Err(DependencyError::DuplicateId { id: 1 })
    );
    assert_eq!(
        DependencyIndex::build(&[node(1, &[]), TestNode(ObjectNode::new(2, 2, vec![1]))]),
        Err(DependencyError::CrossPageDependency {
            node: 2,
            dependency: 1,
            page: 2,
            dependency_page: 1
        })
    );
}

#[test]
fn duplicate_edges_are_coalesced_and_empty_graph_is_valid() {
    let index = DependencyIndex::build(&[node(2, &[1, 1]), node(1, &[])]).unwrap();
    assert_eq!(index.forward[&2], [1]);
    assert_eq!(index.reverse[&1], [2]);
    assert_eq!(index.affected_by(1), [1, 2]);
    let empty: [TestNode; 0] = [];
    assert!(
        DependencyIndex::build(&empty)
            .unwrap()
            .topological
            .is_empty()
    );
}

#[test]
fn point_distance_preserves_geometric_behavior() {
    let a = Point { x: -1., y: 2. };
    let b = Point { x: 2., y: 6. };
    assert_eq!(a.distance(b), 5.);
    assert_eq!(a.distance(a), 0.);
}
