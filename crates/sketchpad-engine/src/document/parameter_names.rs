use super::{Document, Kind, records};

pub(super) fn parameter_name_counter(bytes: &[u8], target_page: usize) -> Option<u32> {
    page_name_counter(bytes, target_page, 0x0960)
}

pub(super) fn measurement_name_counter(bytes: &[u8], target_page: usize) -> Option<u32> {
    page_name_counter(bytes, target_page, 0x0967)
}

fn page_name_counter(bytes: &[u8], target_page: usize, kind: u32) -> Option<u32> {
    let mut page = None;
    let mut counter = None;
    let entries = records(bytes).ok()?;
    for record in super::embedded_tools::document_records(&entries) {
        if record.kind == 0x03e8 {
            page = Some(page.map_or(0, |page| page + 1));
            if page.is_some_and(|page| page > target_page) {
                break;
            }
        } else if page == Some(target_page) && record.kind == kind {
            let payload = &bytes[record.data.clone()];
            if payload.len() != 6 || payload[..2] != [0, 0] || counter.is_some() {
                return None;
            }
            counter = Some(u32::from_le_bytes(payload[2..6].try_into().ok()?));
        }
    }
    counter.filter(|counter| *counter > 0)
}

impl Document {
    /// Materialize the current page's generated names as one transaction.
    pub(super) fn materialize_parameter_names(&mut self) -> Result<(), String> {
        let unnamed = self
            .objects
            .iter()
            .filter(|object| !object.deleted && object.page == self.active_page)
            .filter(
                |object| matches!(&object.kind, Kind::Parameter(group) if group.name().is_empty()),
            )
            .map(|object| object.id)
            .collect::<Vec<_>>();
        if unnamed.is_empty() {
            return Ok(());
        }
        let counter = parameter_name_counter(&self.original, self.active_page)
            .ok_or("Invalid parameter naming counter")?;
        let next = counter
            .checked_add(u32::try_from(unnamed.len()).map_err(|_| "Too many parameters")?)
            .ok_or("Parameter naming counter overflow")?;
        let mut changes = Vec::new();
        for (ordinal, id) in unnamed.iter().enumerate() {
            let Kind::Parameter(group) = &self.objects[*id - 1].kind else {
                unreachable!()
            };
            let font = self
                .text_font_state(*id)
                .ok_or("Unsupported generated parameter font")?;
            let group = group.with_generated_name(counter + ordinal as u32, &font)?;
            changes.push((*id, group));
        }
        let entries = records(&self.original)?;
        let mut page = None;
        let mut location = None;
        for record in super::embedded_tools::document_records(&entries) {
            if record.kind == 0x03e8 {
                page = Some(page.map_or(0, |page| page + 1));
            }
            if page == Some(self.active_page) && record.kind == 0x0960 {
                location = Some(record.data.start + 2);
                break;
            }
        }
        let location = location.ok_or("Missing parameter naming counter")?;
        for (id, group) in changes {
            self.objects[id - 1].kind = Kind::Parameter(group);
        }
        self.original[location..location + 4].copy_from_slice(&next.to_le_bytes());
        self.dirty = true;
        Ok(())
    }

    /// Native parameter captions can use a literal definition or a generated
    /// page name even when the original group has no stored label record.
    pub fn parameter_display_name(&self, id: usize) -> Option<String> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::Parameter(parameter) = &object.kind else {
            return None;
        };
        if parameter.display_mode() == 0 {
            return Some(parameter.literal_definition());
        }
        if !parameter.name().is_empty() {
            return Some(parameter.name().to_owned());
        }
        let ordinal = u32::try_from(self.objects.iter().take(id - 1).filter(|candidate| {
            !candidate.deleted && candidate.page == object.page
                && matches!(&candidate.kind, Kind::Parameter(group) if group.name().is_empty())
        }).count()).ok()?;
        let number = parameter_name_counter(&self.original, object.page)?.checked_add(ordinal)?;
        Some(format!("t[{number}]"))
    }
}
