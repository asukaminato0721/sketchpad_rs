use super::{CustomTransform, Editor};

impl Editor {
    pub(super) fn can_define_custom_transform(&self) -> bool {
        let [seed, output] = self.selection_order.as_slice() else {
            return false;
        };
        self.doc.custom_transform_carried(*seed, *output).is_ok()
    }

    pub(super) fn begin_define_custom_transform(&mut self) {
        let [seed, output] = self.selection_order.as_slice() else {
            self.status = "Select a pre-image point followed by its dependent image point".into();
            return;
        };
        if let Err(error) = self.doc.custom_transform_carried(*seed, *output) {
            self.status = error;
            return;
        }
        self.define_custom_transform_dialog = Some((*seed, *output));
        self.custom_transform_name_draft =
            format!("Custom Transform {}", self.custom_transforms.len() + 1);
    }

    pub(super) fn apply_custom_transform(&mut self, index: usize) {
        let Some(definition) = self.custom_transforms.get(index).cloned() else {
            self.status = "Custom transform is unavailable".into();
            return;
        };
        let [replacement] = self.selection_order.as_slice() else {
            self.status = "Select one point to transform".into();
            return;
        };
        let replacement = *replacement;
        if self.doc.position(replacement).is_none()
            || self
                .doc
                .objects
                .get(replacement.saturating_sub(1))
                .is_none_or(|object| object.deleted || object.page != self.doc.active_page)
        {
            self.status = "Select one point to transform".into();
            return;
        }
        let carried = match self
            .doc
            .custom_transform_carried(definition.seed, definition.output)
        {
            Ok(carried) => carried,
            Err(error) => {
                self.status = format!("{} is unavailable: {error}", definition.name);
                return;
            }
        };
        match self
            .doc
            .add_mapped_element(definition.output, definition.seed, replacement, &carried)
        {
            Ok(id) => {
                self.select_only(id);
                self.status = format!("Applied {} to point {replacement}", definition.name);
            }
            Err(error) => self.status = error,
        }
    }

    pub(super) fn custom_transform_dialogs_ui(&mut self, root: &mut egui::Ui) {
        if let Some((seed, output)) = self.define_custom_transform_dialog {
            let mut create = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("define_custom_transform")).show(root.ctx(), |ui| {
                ui.heading("Define Custom Transform");
                ui.label(format!("Point {seed} maps to point {output}"));
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut self.custom_transform_name_draft);
                });
                ui.horizontal(|ui| {
                    create = ui
                        .add_enabled(
                            !self.custom_transform_name_draft.trim().is_empty(),
                            egui::Button::new("Define"),
                        )
                        .clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if create {
                match self.doc.custom_transform_carried(seed, output) {
                    Ok(_) => {
                        let name = self.custom_transform_name_draft.trim().to_owned();
                        self.custom_transforms.push(CustomTransform {
                            name: name.clone(),
                            seed,
                            output,
                        });
                        self.define_custom_transform_dialog = None;
                        self.status = format!("Defined {name}");
                    }
                    Err(error) => self.status = error,
                }
            } else if cancel {
                self.define_custom_transform_dialog = None;
            }
        }

        if self.edit_custom_transforms_dialog {
            let mut close = false;
            let mut move_up = false;
            let mut move_down = false;
            let mut remove = false;
            let mut rename = None;
            let count = self.custom_transforms.len();
            self.custom_transform_edit_selected = self
                .custom_transform_edit_selected
                .min(count.saturating_sub(1));
            let selected = self.custom_transform_edit_selected;
            egui::Modal::new(egui::Id::new("edit_custom_transforms")).show(root.ctx(), |ui| {
                ui.heading("Edit Custom Transforms");
                if count == 0 {
                    ui.label("No custom transforms are defined.");
                } else {
                    for (index, definition) in self.custom_transforms.iter().enumerate() {
                        let valid = self
                            .doc
                            .custom_transform_carried(definition.seed, definition.output)
                            .is_ok();
                        let label = if valid {
                            definition.name.clone()
                        } else {
                            format!("{} (unavailable)", definition.name)
                        };
                        if ui.selectable_label(index == selected, label).clicked() {
                            self.custom_transform_edit_selected = index;
                        }
                    }
                    let selected = self.custom_transform_edit_selected;
                    let definition = &self.custom_transforms[selected];
                    ui.separator();
                    ui.label(format!(
                        "Point {} maps to point {}",
                        definition.seed, definition.output
                    ));
                    let mut name = definition.name.clone();
                    if ui.text_edit_singleline(&mut name).changed() && !name.trim().is_empty() {
                        rename = Some(name);
                    }
                    ui.horizontal(|ui| {
                        move_up = ui
                            .add_enabled(selected > 0, egui::Button::new("Move Up"))
                            .clicked();
                        move_down = ui
                            .add_enabled(selected + 1 < count, egui::Button::new("Move Down"))
                            .clicked();
                        remove = ui.button("Remove").clicked();
                    });
                }
                close = ui.button("Close").clicked();
            });
            let selected = self.custom_transform_edit_selected;
            if let Some(name) = rename {
                self.custom_transforms[selected].name = name.trim().to_owned();
            }
            if move_up {
                self.custom_transforms.swap(selected, selected - 1);
                self.custom_transform_edit_selected -= 1;
            } else if move_down {
                self.custom_transforms.swap(selected, selected + 1);
                self.custom_transform_edit_selected += 1;
            } else if remove {
                let removed = self.custom_transforms.remove(selected);
                self.custom_transform_edit_selected =
                    selected.min(self.custom_transforms.len().saturating_sub(1));
                self.status = format!("Removed {}", removed.name);
            }
            if close {
                self.edit_custom_transforms_dialog = false;
            }
        }
    }
}
