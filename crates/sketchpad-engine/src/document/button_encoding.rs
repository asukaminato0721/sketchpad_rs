//! Shared action-group patches run after ordering and before variable-size captions.
use super::embedded_tools::document_record_indices;
use super::{Document, Kind, records};
use crate::action_records::{AnimationButtonGroup, HideShowButtonGroup, MovementButtonGroup};

impl Document {
    pub(super) fn patch_action_groups(&self, bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        self.patch_action_groups_scoped(bytes, false)
    }

    pub(super) fn patch_action_source_groups(&self, bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        self.patch_action_groups_scoped(bytes, true)
    }

    fn patch_action_groups_scoped(
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
            let start = rs[start_index].start;
            let end = record.data.end;
            let local_id = |target: usize, family: &str| {
                ids.get(target)
                    .copied()
                    .and_then(|id| id.checked_sub(page_bases[object.page]))
                    .filter(|id| *id > 0)
                    .and_then(|id| u32::try_from(id).ok())
                    .ok_or_else(|| format!("Invalid {family} target"))
            };
            let replacement = match &object.kind {
                Kind::AnimationButton {
                    targets,
                    label,
                    anchor,
                    settings,
                } => {
                    let references = targets
                        .iter()
                        .map(|&target| local_id(target, "Animation"))
                        .collect::<Result<Vec<_>, _>>()?;
                    let mut group = AnimationButtonGroup::parse(&bytes[start..end])?;
                    group.set_targets(&references)?;
                    group.set_label(label)?;
                    group.set_screen_anchor(*anchor);
                    group.set_settings_list(settings)?;
                    group.set_hidden(!object.visible);
                    Some(group.encode())
                }
                Kind::HideShowButton {
                    targets,
                    action,
                    options,
                    caption_flags,
                    label,
                    anchor,
                } => {
                    let mut group = HideShowButtonGroup::parse(&bytes[start..end])?;
                    let caption_changed = group.action() != *action
                        || group.caption_flags() != *caption_flags
                        || group.label_bytes() != label.as_bytes();
                    let references = targets
                        .iter()
                        .map(|&target| local_id(target, "Hide/Show"))
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
                    group.set_presentation_suppressed(object.presentation_suppressed);
                    Some(group.encode())
                }
                Kind::MovementButton {
                    pairs,
                    speed,
                    follow,
                    label,
                    anchor,
                } => {
                    let references = pairs
                        .iter()
                        .map(|&(source, destination)| {
                            Ok((
                                local_id(source, "Movement")?,
                                local_id(destination, "Movement")?,
                            ))
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                    let mut group = MovementButtonGroup::parse(&bytes[start..end])?;
                    group.set_target_pairs(&references)?;
                    group.set_speed(*speed);
                    group.set_follows_destination(*follow);
                    group.set_label(label)?;
                    group.set_screen_anchor(*anchor);
                    group.set_hidden(!object.visible);
                    Some(group.encode())
                }
                _ => None,
            };
            if let Some(replacement) = replacement
                && replacement != bytes[start..end]
            {
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
