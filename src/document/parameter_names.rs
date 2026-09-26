use super::{Document, Kind, records};

pub(super) fn parameter_name_counter(bytes: &[u8], target_page: usize) -> Option<u32> {
    let mut page = None;
    let mut counter = None;
    for record in records(bytes).ok()? {
        if record.kind == 0x03e8 {
            page = Some(page.map_or(0, |page| page + 1));
            if page.is_some_and(|page| page > target_page) {
                break;
            }
        } else if page == Some(target_page) && record.kind == 0x0960 {
            let payload = &bytes[record.data];
            if payload.len() != 6 || payload[..2] != [0, 0] || counter.is_some() {
                return None;
            }
            counter = Some(u32::from_le_bytes(payload[2..6].try_into().ok()?));
        }
    }
    counter.filter(|counter| *counter > 0)
}

impl Document {
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
