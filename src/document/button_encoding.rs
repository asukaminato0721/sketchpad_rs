//! Shared action-group patches run after ordering and before variable-size captions.
use super::embedded_tools::document_record_indices;
use super::{Document, Kind, records};
use crate::action_records::HideShowButtonGroup;

impl Document {
    pub(super) fn patch_hide_show_groups(&self, bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        self.patch_hide_show_groups_scoped(bytes, false)
    }

    pub(super) fn patch_hide_show_source_groups(&self, bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        self.patch_hide_show_groups_scoped(bytes, true)
    }

    fn patch_hide_show_groups_scoped(
        &self,
        mut bytes: Vec<u8>,
        source_only: bool,
    ) -> Result<Vec<u8>, String> {
        let mut survivors: Vec<_> = self
            .objects
            .iter()
            .filter(|object| !object.deleted && (!source_only || object.id <= self.source_count))
            .collect();
        if self.page_count > 1 {
            survivors.sort_by_key(|object| (object.page, object.id > self.source_count, object.id));
        }
        let mut ids = vec![0; self.objects.len() + 1];
        let mut page_bases = vec![0; self.page_count];
        for (index, object) in survivors.iter().enumerate() {
            ids[object.id] = index + 1;
            if index == 0 || survivors[index - 1].page != object.page {
                page_bases[object.page] = index;
            }
        }
        let rs = records(&bytes)?;
        let mut ordinal = 0;
        let mut current = None;
        let mut edits = Vec::new();
        for index in document_record_indices(&rs) {
            let record = &rs[index];
            if record.kind == 0x07d0 {
                current = Some(index);
            }
            if record.kind != 0x07d7 {
                continue;
            }
            let start_index = current.take().ok_or("Action group lacks a header")?;
            let object = survivors.get(ordinal).ok_or("Too many action groups")?;
            ordinal += 1;
            let Kind::HideShowButton {
                targets,
                action,
                options,
                caption_flags,
                label,
                anchor,
            } = &object.kind
            else {
                continue;
            };
            let start = rs[start_index].start;
            let end = record.data.end;
            let mut group = HideShowButtonGroup::parse(&bytes[start..end])?;
            let caption_changed = group.action() != *action
                || group.caption_flags() != *caption_flags
                || group.label_bytes() != label.as_bytes();
            let references = targets
                .iter()
                .map(|target| {
                    ids.get(*target)
                        .copied()
                        .and_then(|id| id.checked_sub(page_bases[object.page]))
                        .filter(|id| *id > 0)
                        .and_then(|id| u32::try_from(id).ok())
                        .ok_or_else(|| "Invalid Hide/Show target".to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            group.set_targets(&references)?;
            group.set_options(*options);
            group.set_caption_flags(*caption_flags)?;
            if caption_changed && caption_flags.is_some() {
                group.set_generated_label(label)?;
            } else {
                group.set_label(label)?;
            }
            group.set_action(*action);
            group.set_screen_anchor(*anchor);
            group.set_hidden(!object.visible);
            let replacement = group.encode();
            if replacement != bytes[start..end] {
                edits.push((start, end, replacement));
            }
        }
        if ordinal != survivors.len() {
            return Err("Action group count mismatch".into());
        }
        for (start, end, replacement) in edits.into_iter().rev() {
            bytes.splice(start..end, replacement);
        }
        records(&bytes)?;
        Ok(bytes)
    }
}
