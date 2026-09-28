//! Native code88 exposes a single mapped point binding's terminal scope.
use super::kind::is_point_kind;
use super::{Document, Kind, Object, Point};

pub(super) fn binding_source(
    objects: &[Object],
    binding: usize,
    page: usize,
    before: usize,
) -> Option<(usize, usize)> {
    let object = objects.get(binding.checked_sub(1)?)?;
    let Kind::IterationBinding(source, iteration) = object.kind else {
        return None;
    };
    let parent = objects.get(iteration.checked_sub(1)?)?;
    let Kind::MappedPointIteration {
        branches, carried, ..
    } = &parent.kind
    else {
        return None;
    };
    let source_object = objects.get(source.checked_sub(1)?)?;
    (binding < before
        && !object.deleted
        && object.page == page
        && source < iteration
        && iteration < binding
        && !parent.deleted
        && parent.page == page
        && branches.len() == 1
        && carried.contains(&source)
        && !source_object.deleted
        && source_object.page == page
        && is_point_kind(&source_object.kind))
    .then_some((source, iteration))
}

pub(super) fn decode(
    objects: &[Object],
    page: usize,
    id: usize,
    header: &[u8],
    refs: &[usize],
    constraint: Option<&[u8]>,
) -> Option<Kind> {
    let [binding] = refs else {
        return None;
    };
    if header.len() != 28
        || header[..2] != [88, 0]
        || header[6..10] != [1, 0, 4, 0]
        || constraint.is_some()
    {
        return None;
    }
    binding_source(objects, *binding, page, id)?;
    Some(Kind::IterationPointAlias(*binding))
}

impl Document {
    /// Expose the final carried point, including an undefined terminal result.
    pub fn iteration_endpoint(&self, id: usize) -> Option<Point> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationPointAlias(binding) = object.kind else {
            return None;
        };
        if object.deleted {
            return None;
        }
        let (source, iteration) = binding_source(&self.objects, binding, object.page, id)?;
        self.mapped_terminal_point(source, iteration)
    }

    pub fn add_iteration_point_alias(&mut self, binding: usize) -> Result<usize, String> {
        if self.partial_view
            || !self.safe_structure
            || self.page_count != 1
            || binding_source(
                &self.objects,
                binding,
                self.active_page,
                self.objects.len() + 1,
            )
            .is_none()
        {
            return Err("Select a live single-branch mapped point binding".into());
        }
        self.checkpoint();
        Ok(self.append_object(Kind::IterationPointAlias(binding)))
    }
}
