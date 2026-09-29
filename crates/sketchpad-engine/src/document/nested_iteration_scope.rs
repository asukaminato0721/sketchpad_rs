//! Resolve nested iteration adapters and their explicit construction capture.
use super::{Kind, Object};
use std::collections::BTreeSet;

/// Structural bindings do not choose an environment; their mapped descriptor
/// does. Return descriptors reached through a valid terminal or explicitly
/// captured with their complete mapped root.
pub(super) fn captured_structures(
    objects: &[Object],
    page: usize,
    seeds: &[usize],
    branches: &[Vec<usize>],
    carried: &[usize],
) -> Option<BTreeSet<usize>> {
    let get = |id: usize| {
        objects
            .get(id.checked_sub(1)?)
            .filter(|o| !o.deleted && o.page == page)
    };
    let mut pending: Vec<_> = branches.iter().flatten().chain(carried).copied().collect();
    let mut visited = BTreeSet::new();
    let mut structures = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !visited.insert(id) {
            continue;
        }
        let object = get(id)?;
        let dependencies = object.construction_dependencies();
        if dependencies
            .iter()
            .any(|parent| *parent >= id || get(*parent).is_none())
        {
            return None;
        }
        if seeds.contains(&id) {
            continue;
        }
        if let Kind::IterationPointAlias(binding) = object.kind {
            let (_, root) = super::iteration_endpoint::binding_source(objects, binding, page, id)?;
            structures.insert(binding);
            if structures.insert(root) {
                let Kind::MappedPointIteration {
                    seeds: local_seeds,
                    branches: local_branches,
                    carried: local_carried,
                    ..
                } = &get(root)?.kind
                else {
                    return None;
                };
                if !super::iteration::supported_mapped_point_iteration(
                    objects,
                    local_seeds,
                    local_branches,
                    local_carried,
                ) {
                    return None;
                }
                if carried.contains(&root) {
                    // Mixed omitted active ancestors have a distinct Native
                    // capture phase. Require the proved complete local capture.
                    for &local in local_carried.iter().chain(local_branches.iter().flatten()) {
                        if depends_on_seeds(objects, local, seeds)?
                            && !seeds.contains(&local)
                            && !carried.contains(&local)
                        {
                            return None;
                        }
                    }
                }
            }
        }
        if let Kind::IterationBinding(source, root) = object.kind
            && carried.contains(&id)
        {
            let Kind::MappedPointIteration {
                seeds: local_seeds,
                branches: local_branches,
                carried: local_carried,
                ..
            } = &get(root)?.kind
            else {
                return None;
            };
            if !(source < root
                && root < id
                && carried.contains(&root)
                && local_carried.contains(&source)
                && get(source)
                    .is_some_and(|object| super::iteration::mapped_binding_kind(&object.kind))
                && super::iteration::supported_mapped_point_iteration(
                    objects,
                    local_seeds,
                    local_branches,
                    local_carried,
                ))
            {
                return None;
            }
            structures.insert(id);
            structures.insert(root);
            for &local in local_carried.iter().chain(local_branches.iter().flatten()) {
                if depends_on_seeds(objects, local, seeds)?
                    && !seeds.contains(&local)
                    && !carried.contains(&local)
                {
                    return None;
                }
            }
        }
        pending.extend(dependencies);
    }
    Some(structures)
}

fn depends_on_seeds(objects: &[Object], id: usize, seeds: &[usize]) -> Option<bool> {
    let mut pending = vec![id];
    let mut seen = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if seeds.contains(&id) {
            return Some(true);
        }
        if !seen.insert(id) {
            continue;
        }
        let object = objects.get(id.checked_sub(1)?)?;
        for parent in object.construction_dependencies() {
            if parent >= id {
                return None;
            }
            pending.push(parent);
        }
    }
    Some(false)
}
