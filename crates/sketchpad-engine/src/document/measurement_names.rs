//! Native measurement names requested by active referenced-text operands.
//! Assignment survives visibility changes; only a dirty save writes the labels.
use super::parameter_names::measurement_name_counter;
use super::referenced_text::ReferenceRole;
use super::{Document, Kind};
use std::collections::{BTreeMap, BTreeSet};

impl Document {
    fn allocated_measurement_names(&self) -> BTreeMap<usize, u32> {
        let mut names = self.measurement_names.clone();
        let mut next = (0..self.page_count)
            .map(|page| measurement_name_counter(&self.original, page))
            .collect::<Vec<_>>();
        for (&id, &number) in &names {
            if let Some(object) = self.objects.get(id.saturating_sub(1))
                && let Some(counter) = next.get_mut(object.page)
            {
                *counter =
                    counter.and_then(|counter| number.checked_add(1).map(|n| counter.max(n)));
            }
        }
        // A hidden caption participates when a visible construction uses it.
        // Scan the closure in object order, not recursive evaluation order.
        let mut active = BTreeSet::new();
        let mut pending = self
            .objects
            .iter()
            .filter(|o| !o.deleted && o.visible && !o.presentation_suppressed)
            .map(|o| o.id)
            .collect::<Vec<_>>();
        while let Some(id) = pending.pop() {
            let Some(object) = id.checked_sub(1).and_then(|i| self.objects.get(i)) else {
                continue;
            };
            if object.deleted || !active.insert(id) {
                continue;
            }
            pending.extend(
                object
                    .construction_dependencies()
                    .into_iter()
                    .filter(|parent| {
                        self.objects
                            .get(parent.saturating_sub(1))
                            .is_some_and(|p| !p.deleted && p.page == object.page && *parent < id)
                    }),
            );
        }
        for object in self.objects.iter().filter(|o| active.contains(&o.id)) {
            let (parents, text) = match &object.kind {
                Kind::ReferencedText { parents, text } => (parents.clone(), text),
                Kind::PointReferencedText {
                    point,
                    source,
                    text,
                } => (vec![*point, *source], text),
                _ => continue,
            };
            for (slot, role) in text.reference_occurrences() {
                if role != ReferenceRole::Name {
                    continue;
                }
                let Some(&id) = parents.get(slot - 1) else {
                    continue;
                };
                let Some(source) = id.checked_sub(1).and_then(|i| self.objects.get(i)) else {
                    continue;
                };
                if source.deleted
                    || source.page != object.page
                    || !matches!(
                        source.kind,
                        Kind::ParameterCalculation(..)
                            | Kind::SegmentRatio(..)
                            | Kind::PointRatio(..)
                            | Kind::SegmentLength(..)
                    )
                    || source
                        .label
                        .as_ref()
                        .is_some_and(|label| label.name_utf8().is_ok_and(|s| !s.is_empty()))
                    || names.contains_key(&id)
                {
                    continue;
                }
                let Some(number) = next[source.page] else {
                    continue;
                };
                let Some(after) = number.checked_add(1) else {
                    continue;
                };
                names.insert(id, number);
                next[source.page] = Some(after);
            }
        }
        names
    }

    pub(super) fn refresh_measurement_names(&mut self) {
        self.measurement_names = self.allocated_measurement_names();
    }

    pub(super) fn measurement_reference_name(&self, source: usize) -> Option<String> {
        let object = self.objects.get(source.checked_sub(1)?)?;
        if let Some(name) = object
            .label
            .as_ref()
            .and_then(|l| l.name_utf8().ok())
            .filter(|s| !s.is_empty())
        {
            return Some(name.to_owned());
        }
        let number = self
            .measurement_names
            .get(&source)
            .copied()
            .or_else(|| self.allocated_measurement_names().get(&source).copied())?;
        Some(format!("m[{number}]"))
    }

    /// Produce the semantic save state without mutating a read-only evaluation.
    /// Original bytes and runtime assignments stay in their own history domains.
    pub(super) fn materialized_measurement_names(&self) -> Result<Option<Self>, String> {
        if !self.dirty {
            return Ok(None);
        }
        let names = self.allocated_measurement_names();
        if !names.iter().any(|(&id, _)| {
            self.objects.get(id.saturating_sub(1)).is_some_and(|o| {
                !o.deleted && o.label.as_ref().is_none_or(|l| l.name_bytes().is_empty())
            })
        }) {
            return Ok(None);
        }
        let mut prepared = self.clone();
        prepared.measurement_names = names.clone();
        let mut counters = BTreeMap::<usize, u32>::new();
        for (id, number) in names {
            let Some(object) = prepared.objects.get_mut(id.saturating_sub(1)) else {
                continue;
            };
            let after = number
                .checked_add(1)
                .ok_or("Measurement name counter overflow")?;
            counters
                .entry(object.page)
                .and_modify(|next| *next = (*next).max(after))
                .or_insert(after);
            if object.deleted
                || object
                    .label
                    .as_ref()
                    .is_some_and(|l| !l.name_bytes().is_empty())
            {
                continue;
            }
            let name = format!("m[{number}]");
            if let Some(label) = &mut object.label {
                label.set_name(&name)?;
            } else {
                object.label = Some(crate::text_records::LabelAux::new_point(
                    &name,
                    false,
                    (8, 5),
                )?);
            }
        }
        for object in &mut prepared.objects {
            if let (Kind::ParameterCalculation(source, group), Some(label)) =
                (&object.kind, &object.label)
            {
                object.kind = Kind::ParameterCalculation(*source, group.with_label(label)?);
            }
        }
        let entries = super::records::records(&prepared.original)?;
        let mut page = None;
        for record in super::embedded_tools::document_records(&entries) {
            if record.kind == 0x03e8 {
                page = Some(page.map_or(0, |p| p + 1));
            }
            if record.kind == 0x0967
                && let Some((page, next)) = page.and_then(|p| counters.get(&p).map(|&n| (p, n)))
            {
                let counter = measurement_name_counter(&prepared.original, page)
                    .ok_or("Invalid measurement name counter")?;
                prepared.original[record.data.start + 2..record.data.start + 6]
                    .copy_from_slice(&counter.max(next).to_le_bytes());
            }
        }
        Ok(Some(prepared))
    }
}
