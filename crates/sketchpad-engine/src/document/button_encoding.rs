//! Shared action-group patches run after ordering and before variable-size captions.
use super::embedded_tools::document_record_indices;
use super::{Document, Kind, records};
use crate::action_records::{
    AnimationButtonGroup, HideShowButtonGroup, LinkPageButtonGroup, LinkUrlButtonGroup,
    MovementButtonGroup, PresentationButtonGroup, ScrollButtonGroup, SoundButtonGroup,
};

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
        let original = Self::from_bytes(&self.original, None)?;
        let link_topology_changed = original
            .objects
            .iter()
            .zip(&self.objects)
            .take(self.source_count)
            .any(|(saved, current)| match (&saved.kind, &current.kind) {
                (
                    Kind::LinkPageButton {
                        target_page: saved_page,
                        target_button_ordinal: saved_button,
                        ..
                    },
                    Kind::LinkPageButton {
                        target_page: current_page,
                        target_button_ordinal: current_button,
                        ..
                    },
                ) => {
                    saved.deleted != current.deleted
                        || saved_page != current_page
                        || saved_button != current_button
                }
                (Kind::LinkPageButton { .. }, _) => !saved.deleted,
                (_, Kind::LinkPageButton { .. }) => !current.deleted,
                _ => false,
            });
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
                    if link_topology_changed {
                        let destination_buttons = self.link_destination_buttons(object.page);
                        let linked = self.objects.iter().any(|source| {
                            !source.deleted
                                && matches!(
                                    source.kind,
                                    Kind::LinkPageButton {
                                        target_page,
                                        target_button_ordinal: Some(ordinal),
                                        ..
                                    } if target_page == object.page
                                        && ordinal.checked_sub(1)
                                            .and_then(|index| destination_buttons.get(index))
                                            == Some(&object.id)
                                )
                        });
                        group.set_link_destination(linked);
                    }
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
                Kind::ScrollButton {
                    target,
                    placement,
                    label,
                    anchor,
                } => {
                    let reference = local_id(*target, "Scroll")?;
                    let mut group = ScrollButtonGroup::parse(&bytes[start..end])?;
                    group.set_target(reference)?;
                    group.set_placement(*placement);
                    group.set_label(label)?;
                    group.set_screen_anchor(*anchor);
                    group.set_hidden(!object.visible);
                    Some(group.encode())
                }
                Kind::SoundButton {
                    functions,
                    label,
                    anchor,
                } => {
                    let references = functions
                        .iter()
                        .map(|&function| local_id(function, "Sound"))
                        .collect::<Result<Vec<_>, _>>()?;
                    let mut group = SoundButtonGroup::parse(&bytes[start..end])?;
                    group.set_targets(&references)?;
                    group.set_label(label)?;
                    group.set_screen_anchor(*anchor);
                    group.set_hidden(!object.visible);
                    Some(group.encode())
                }
                Kind::PresentationButton {
                    targets,
                    mode,
                    options,
                    label,
                    anchor,
                } => {
                    let references = targets
                        .iter()
                        .map(|&target| local_id(target, "Presentation"))
                        .collect::<Result<Vec<_>, _>>()?;
                    let mut group = PresentationButtonGroup::parse(&bytes[start..end])?;
                    group.set_targets(&references)?;
                    group.set_options(*mode, *options);
                    group.set_label(label)?;
                    group.set_screen_anchor(*anchor);
                    group.set_hidden(!object.visible);
                    group.set_presentation_suppressed(object.presentation_suppressed);
                    Some(group.encode())
                }
                Kind::LinkUrlButton { url, label, anchor } => {
                    let (mut group, converted) = match LinkUrlButtonGroup::parse(&bytes[start..end])
                    {
                        Ok(group) => (group, false),
                        Err(_) => (
                            LinkUrlButtonGroup::from_original_template(url, label, *anchor)?,
                            true,
                        ),
                    };
                    group.set_url(url)?;
                    group.set_label(label)?;
                    group.set_screen_anchor(*anchor)?;
                    if converted {
                        let object_index = usize::try_from(local_id(object.id, "Link")? - 1)
                            .map_err(|_| "Invalid Link object index")?;
                        group.set_page_object_index(object_index)?;
                    }
                    group.set_hidden(!object.visible);
                    Some(group.encode())
                }
                Kind::LinkPageButton {
                    target_page,
                    target_button_ordinal,
                    label,
                    anchor,
                } => {
                    let (mut group, converted) =
                        match LinkPageButtonGroup::parse(&bytes[start..end]) {
                            Ok(group) => (group, false),
                            Err(_) => (
                                LinkPageButtonGroup::from_original_template(
                                    *target_page,
                                    label,
                                    *anchor,
                                )?,
                                true,
                            ),
                        };
                    group.set_target_page(*target_page)?;
                    group.set_target_button_ordinal(*target_button_ordinal)?;
                    group.set_label(label)?;
                    group.set_screen_anchor(*anchor)?;
                    if converted {
                        let object_index = usize::try_from(local_id(object.id, "Link")? - 1)
                            .map_err(|_| "Invalid Link object index")?;
                        group.set_page_object_index(object_index)?;
                    }
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
