use super::{Editor, Tool};
use crate::render::Viewport;
use sketchpad_engine::document::Kind;

impl Editor {
    fn point_at(&mut self, v: &Viewport, p: egui::Pos2) -> usize {
        if let Some(id) = self.hit_point(v, p) {
            return id;
        }
        if let Some(host) = self.hit(v, p)
            && let Some(kind) = self.doc.constrained_point(host, v.world(p))
        {
            return self.doc.add(kind);
        }
        self.doc.add(Kind::Free(self.doc.snap_point(v.world(p))))
    }
    pub(super) fn select_only(&mut self, id: usize) {
        self.doc.selected.clear();
        self.doc.selected.insert(id);
        self.selection_order.clear();
        self.selection_order.push(id);
    }
    pub(super) fn select_related(&mut self, children: bool) {
        let current = self.doc.selected.clone();
        let related: Vec<_> = if children {
            self.doc
                .objects
                .iter()
                .filter(|object| {
                    object.page == self.doc.active_page && object.visible && !object.deleted
                })
                .filter(|object| {
                    self.doc
                        .dependencies_of(object.id)
                        .unwrap_or_default()
                        .iter()
                        .any(|id| current.contains(id))
                })
                .map(|object| object.id)
                .collect()
        } else {
            self.doc
                .objects
                .iter()
                .filter(|object| current.contains(&object.id))
                .flat_map(|object| self.doc.dependencies_of(object.id).unwrap_or_default())
                .filter(|id| {
                    self.doc
                        .objects
                        .get(id.saturating_sub(1))
                        .is_some_and(|object| {
                            object.page == self.doc.active_page && object.visible && !object.deleted
                        })
                })
                .collect()
        };
        self.doc.selected = related.iter().copied().collect();
        self.selection_order = related;
    }
    pub(super) fn clear_selected(&mut self) {
        match self.doc.clear_selected() {
            Ok(removed) if removed.is_empty() => self.status = "Select objects to clear".into(),
            Ok(removed) => {
                self.stop_all_sounds();
                self.selection_order.clear();
                self.status = format!("Cleared {} object(s), including dependents", removed.len());
            }
            Err(error) => self.status = error,
        }
    }
    pub(super) fn copy_selected(&mut self) -> bool {
        match self.doc.copy_selected() {
            Ok(clipboard) => {
                self.status = format!("Copied {} object(s)", clipboard.len());
                self.clipboard = Some(clipboard);
                true
            }
            Err(error) => {
                self.status = error;
                false
            }
        }
    }
    pub(super) fn cut_selected(&mut self) {
        let clipboard = match self.doc.copy_selected() {
            Ok(clipboard) => clipboard,
            Err(error) => {
                self.status = error;
                return;
            }
        };
        match self.doc.clear_selected() {
            Ok(removed) => {
                self.stop_all_sounds();
                self.clipboard = Some(clipboard);
                self.selection_order.clear();
                self.status = format!("Cut {} object(s)", removed.len());
            }
            Err(error) => self.status = error,
        }
    }
    pub(super) fn paste(&mut self) {
        let Some(clipboard) = &self.clipboard else {
            self.status = "Clipboard is empty".into();
            return;
        };
        match self.doc.paste(clipboard) {
            Ok(ids) => {
                self.selection_order = ids;
                self.status = format!("Pasted {} object(s)", self.selection_order.len());
            }
            Err(error) => self.status = error,
        }
    }
    pub(super) fn click(&mut self, v: &Viewport, p: egui::Pos2, multi: bool) {
        if self.doc.partial_view
            && !matches!(self.tool, Tool::Select | Tool::Custom(_))
            && !(self.doc.page_count > 1
                && matches!(
                    self.tool,
                    Tool::Point | Tool::Segment | Tool::Circle | Tool::Text
                ))
        {
            self.status =
                "This page contains unsupported records; construction cannot be saved safely"
                    .into();
            return;
        }
        match self.tool {
            Tool::Custom(source) => self.custom_tool_click(source, v, p),
            Tool::Select => {
                if !multi && let Some(id) = self.referenced_action_hit(v, p) {
                    if let Err(error) = self.activate_referenced_action(id, v) {
                        self.status = error;
                    }
                    self.selection_order.clear();
                    return;
                }
                let hit = self.hit(v, p);
                if !multi
                    && let Some(id) = hit
                    && matches!(self.doc.objects[id - 1].kind, Kind::HideShowButton { .. })
                {
                    match self.doc.activate_hide_show_button(id) {
                        Ok(()) => self.status = format!("Activated button {id}"),
                        Err(error) => self.status = error,
                    }
                    self.selection_order.clear();
                    return;
                }
                if !multi
                    && let Some(id) = hit
                    && let Kind::AnimationButton {
                        targets, settings, ..
                    } = &self.doc.objects[id - 1].kind
                {
                    let targets = targets.clone();
                    let settings = settings.clone();
                    self.animate_button_targets(&targets, &settings);
                    self.selection_order.clear();
                    return;
                }
                if !multi
                    && let Some(id) = hit
                    && matches!(self.doc.objects[id - 1].kind, Kind::SoundButton { .. })
                {
                    if let Err(error) = self.activate_sound_button(id) {
                        self.status = error;
                    }
                    self.selection_order.clear();
                    return;
                }
                if !multi
                    && let Some(id) = hit
                    && matches!(self.doc.objects[id - 1].kind, Kind::MovementButton { .. })
                {
                    self.activate_movement_button(id);
                    self.selection_order.clear();
                    return;
                }
                if !multi
                    && let Some(id) = hit
                    && let Kind::ScrollButton {
                        target, placement, ..
                    } = self.doc.objects[id - 1].kind
                {
                    if let Err(error) = self.scroll_to_button_target(target, placement, v) {
                        self.status = error;
                    }
                    self.selection_order.clear();
                    return;
                }
                if !multi
                    && let Some(id) = hit
                    && matches!(
                        self.doc.objects[id - 1].kind,
                        Kind::PresentationButton { .. }
                    )
                {
                    if let Err(error) = self.activate_presentation_button(id, v, 0) {
                        self.status = error;
                    }
                    self.selection_order.clear();
                    return;
                }
                if !multi
                    && let Some(id) = hit
                    && let Kind::LinkUrlButton { url, .. } = &self.doc.objects[id - 1].kind
                {
                    let url = url.clone();
                    if let Err(error) = self.activate_link_url(&url) {
                        self.status = error;
                    }
                    self.selection_order.clear();
                    return;
                }
                if !multi
                    && let Some(id) = hit
                    && let Kind::LinkPageButton {
                        target_page,
                        target_button_ordinal,
                        ..
                    } = self.doc.objects[id - 1].kind
                {
                    if let Err(error) =
                        self.activate_link_page(target_page, target_button_ordinal, v, 0)
                    {
                        self.status = error;
                    }
                    return;
                }
                let hit = hit.filter(|id| self.doc.objects[*id - 1].arrow_selectable);
                if !multi && (hit.is_some() || !self.preferences.double_click_deselection) {
                    self.doc.selected.clear();
                    self.selection_order.clear();
                }
                if let Some(id) = hit {
                    if multi && !self.doc.selected.insert(id) {
                        self.doc.selected.remove(&id);
                        self.selection_order.retain(|selected| *selected != id);
                    } else {
                        self.doc.selected.insert(id);
                        self.selection_order.push(id);
                    }
                }
            }
            Tool::Point => {
                let id = self.point_at(v, p);
                self.select_only(id);
            }
            Tool::Text => match self.doc.add_text(&self.text_draft, v.world(p)) {
                Ok(id) => {
                    self.select_only(id);
                    if self.preferences.show_text_palette_for_new_captions {
                        self.show_text_palette = true;
                    }
                }
                Err(error) => self.status = error,
            },
            Tool::Segment | Tool::Circle => {
                let id = self.point_at(v, p);
                if let Some(a) = self.pending.take() {
                    if a != id {
                        let k = if self.tool == Tool::Segment {
                            Kind::Segment(a, id)
                        } else {
                            Kind::Circle(a, id)
                        };
                        let made = self.doc.add(k);
                        self.select_only(made);
                    }
                } else {
                    self.pending = Some(id);
                }
            }
        }
    }
}
