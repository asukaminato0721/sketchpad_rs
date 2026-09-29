//! Stored single-application dependency mappings. These are ordinary live
//! constructions; tool authoring and invocation have separate semantics.
use super::iteration::{MappedValue, supported_mapped_point_iteration};
use super::kind::is_point_kind;
use super::transform_model::ElementType;
use super::{Document, Kind, Object, Point};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq)]
pub struct MappedElementDefinition {
    output: usize,
    element: ElementType,
    seed: usize,
    replacement: usize,
    carried: Vec<usize>,
}

impl MappedElementDefinition {
    pub fn new(
        output: usize,
        element: ElementType,
        seed: usize,
        replacement: usize,
        carried: Vec<usize>,
    ) -> Result<Self, String> {
        if element != ElementType::Point
            || [output, seed, replacement].contains(&0)
            || carried.last() != Some(&output)
            || carried.contains(&0)
            || carried.iter().copied().collect::<BTreeSet<_>>().len() != carried.len()
        {
            return Err(
                "Select a point output and an ordered carried graph ending at that output".into(),
            );
        }
        Ok(Self {
            output,
            element,
            seed,
            replacement,
            carried,
        })
    }
    pub fn output(&self) -> usize {
        self.output
    }
    pub fn element(&self) -> ElementType {
        self.element
    }
    pub fn seed(&self) -> usize {
        self.seed
    }
    pub fn replacement(&self) -> usize {
        self.replacement
    }
    pub fn carried(&self) -> &[usize] {
        &self.carried
    }
    pub fn native_references(&self) -> Vec<usize> {
        let mut refs = vec![self.output, self.replacement, self.seed];
        refs.extend(&self.carried);
        refs
    }
    pub(super) fn remap(&self, ids: &[usize]) -> Self {
        Self {
            output: ids[self.output],
            element: self.element,
            seed: ids[self.seed],
            replacement: ids[self.replacement],
            carried: self.carried.iter().map(|id| ids[*id]).collect(),
        }
    }
    pub(super) fn supported(&self, objects: &[Object], page: usize, object_id: usize) -> bool {
        if self.native_references().iter().any(|id| {
            objects.get(id.saturating_sub(1)).is_none_or(|o| {
                *id == 0 || *id >= object_id || o.id != *id || o.page != page || o.deleted
            })
        }) {
            return false;
        }
        let point = |id: usize| {
            is_point_kind(&objects[id - 1].kind)
                && !matches!(
                    objects[id - 1].kind,
                    Kind::ToolInput(_) | Kind::ToolPointOnPath(_)
                )
        };
        // Native code101 point roles are independently proved. Numeric locus
        // packets (code97) are not single-point applications.
        point(self.output)
            && point(self.seed)
            && point(self.replacement)
            && supported_mapped_point_iteration(
                objects,
                &[self.seed],
                &[vec![self.replacement]],
                &self.carried,
            )
    }
}

pub(super) fn decode_mapped_element(
    objects: &[Object],
    page: usize,
    object_id: usize,
    header: &[u8],
    refs: &[usize],
    constraint: Option<&[u8]>,
) -> Option<MappedElementDefinition> {
    if header.len() != 28
        || header[..2] != [101, 0]
        || header[6..10] != [1, 0, 89, 0]
        || constraint.is_some()
        || refs.len() < 4
    {
        return None;
    }
    let definition = MappedElementDefinition::new(
        refs[0],
        ElementType::Point,
        refs[2],
        refs[1],
        refs[3..].to_vec(),
    )
    .ok()?;
    definition
        .supported(objects, page, object_id)
        .then_some(definition)
}

impl Document {
    /// Return the ordered construction slice that changes when `seed` is
    /// substituted and produces `output`. This is the dependency model used by
    /// Sketchpad custom point transforms and by native code-101 applications.
    pub fn custom_transform_carried(
        &self,
        seed: usize,
        output: usize,
    ) -> Result<Vec<usize>, String> {
        if self.partial_view || !self.safe_structure || self.page_count != 1 {
            return Err("Custom transforms require a supported single-page sketch".into());
        }
        let point = |id: usize| {
            self.objects
                .get(id.saturating_sub(1))
                .is_some_and(|object| {
                    !object.deleted
                        && object.page == self.active_page
                        && is_point_kind(&object.kind)
                        && !matches!(object.kind, Kind::ToolInput(_) | Kind::ToolPointOnPath(_))
                })
        };
        if seed == output || seed >= output || !point(seed) || !point(output) {
            return Err("Select a point followed by a later point that depends on it".into());
        }

        // Object IDs are native construction order, so a single forward pass
        // identifies every node affected by substituting the seed.
        let mut dependent = vec![false; self.objects.len() + 1];
        dependent[seed] = true;
        for id in seed + 1..=output {
            let object = &self.objects[id - 1];
            if object.deleted || object.page != self.active_page {
                continue;
            }
            let parents = object.construction_dependencies();
            if parents.iter().any(|parent| {
                *parent == 0
                    || *parent >= id
                    || self
                        .objects
                        .get(parent.saturating_sub(1))
                        .is_none_or(|parent| parent.deleted || parent.page != self.active_page)
            }) {
                return Err("The selected construction has invalid dependencies".into());
            }
            dependent[id] = parents
                .iter()
                .any(|parent| dependent.get(*parent).copied().unwrap_or(false));
        }
        if !dependent[output] {
            return Err("The second point does not depend on the first point".into());
        }

        // Keep only the affected ancestors of the chosen output. Independent
        // operands remain external inputs and are frozen by mapped evaluation.
        let mut carried = BTreeSet::new();
        let mut pending = vec![output];
        while let Some(id) = pending.pop() {
            if id == seed || !dependent.get(id).copied().unwrap_or(false) || !carried.insert(id) {
                continue;
            }
            pending.extend(
                self.objects[id - 1]
                    .construction_dependencies()
                    .into_iter()
                    .filter(|parent| dependent.get(*parent).copied().unwrap_or(false)),
            );
        }
        let carried = carried.into_iter().collect::<Vec<_>>();
        if !supported_mapped_point_iteration(&self.objects, &[seed], &[vec![seed]], &carried) {
            return Err("This point dependency graph cannot be used as a custom transform".into());
        }
        Ok(carried)
    }

    pub fn add_mapped_element(
        &mut self,
        output: usize,
        seed: usize,
        replacement: usize,
        carried: &[usize],
    ) -> Result<usize, String> {
        let definition = MappedElementDefinition::new(
            output,
            ElementType::Point,
            seed,
            replacement,
            carried.to_vec(),
        )?;
        if self.partial_view
            || !self.safe_structure
            || self.page_count != 1
            || !definition.supported(&self.objects, self.active_page, self.objects.len() + 1)
        {
            return Err(
                "Select a supported point substitution and carried dependency graph".into(),
            );
        }
        self.checkpoint();
        Ok(self.append_object(Kind::MappedElement(definition)))
    }
    pub(super) fn mapped_element_point(&self, id: usize) -> Option<Point> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::MappedElement(definition) = &object.kind else {
            return None;
        };
        if object.deleted || !definition.supported(&self.objects, object.page, id) {
            return None;
        }
        let value = MappedValue::read(self, definition.replacement)?;
        let external = self.mapped_external_inputs(
            &[definition.seed],
            &[vec![definition.replacement]],
            &definition.carried,
        )?;
        let mut sample = self.clone();
        sample.freeze_scoped_evaluation_inputs(self, &external)?;
        value.substitute(&mut sample, definition.seed)?;
        sample.position(definition.output)
    }
}
