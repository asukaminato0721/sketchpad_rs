use super::Editor;
use sketchpad_engine::document::Kind;

impl Editor {
    pub(super) fn document_dialogs_ui(&mut self, root: &mut egui::Ui) {
        if self.document_options_dialog {
            let mut add_page = false;
            let mut copy_page = false;
            let mut rename_page = false;
            let mut remove_page = false;
            let mut move_up = false;
            let mut move_down = false;
            let mut close = false;
            let mut selected_page = self.document_options_page;
            egui::Modal::new(egui::Id::new("document_options")).show(root.ctx(), |ui| {
                ui.heading("Document Options");
                egui::ComboBox::from_label("Page")
                    .selected_text(format!("Page {}", selected_page + 1))
                    .show_ui(ui, |ui| {
                        for page in 0..self.doc.page_count {
                            let label = self.doc.page_names[page].as_ref().map_or_else(
                                || format!("Page {}", page + 1),
                                |name| format!("Page {}: {name}", page + 1),
                            );
                            ui.selectable_value(&mut selected_page, page, label);
                        }
                    });
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut self.page_name_draft);
                });
                ui.label("Leave the name empty to use the page number.");
                rename_page = ui.button("Rename Page").clicked();
                add_page = ui.button("Add Blank Page").clicked();
                copy_page = ui.button("Copy Selected Page").clicked();
                remove_page = ui
                    .add_enabled(self.doc.page_count > 1, egui::Button::new("Remove Page…"))
                    .clicked();
                ui.horizontal(|ui| {
                    move_up = ui
                        .add_enabled(selected_page > 0, egui::Button::new("Move Up"))
                        .clicked();
                    move_down = ui
                        .add_enabled(
                            selected_page + 1 < self.doc.page_count,
                            egui::Button::new("Move Down"),
                        )
                        .clicked();
                });
                close = ui.button("Close").clicked();
            });
            if selected_page != self.document_options_page {
                self.document_options_page = selected_page;
                self.page_name_draft = self.doc.page_names[selected_page]
                    .clone()
                    .unwrap_or_default();
            }
            if rename_page {
                match self
                    .doc
                    .set_page_name(self.document_options_page, &self.page_name_draft)
                {
                    Ok(()) => {
                        self.status = format!("Renamed page {}", self.document_options_page + 1)
                    }
                    Err(error) => self.status = error,
                }
            }
            if add_page {
                match self.doc.add_blank_page() {
                    Ok(page) => {
                        self.document_options_page = page - 1;
                        self.page_name_draft.clear();
                        self.selection_order.clear();
                        self.pending = None;
                        self.fit();
                        self.status = format!("Added page {page}");
                    }
                    Err(error) => self.status = error,
                }
            }
            if copy_page {
                match self.doc.copy_page(self.document_options_page) {
                    Ok(page) => {
                        self.document_options_page = page - 1;
                        self.page_name_draft =
                            self.doc.page_names[page - 1].clone().unwrap_or_default();
                        self.selection_order.clear();
                        self.pending = None;
                        self.fit();
                        self.status = format!("Copied page to page {page}");
                    }
                    Err(error) => self.status = error,
                }
            }
            if remove_page {
                self.remove_page_confirmation = Some(self.document_options_page);
            }
            if move_up || move_down {
                let from = self.document_options_page;
                let to = if move_up { from - 1 } else { from + 1 };
                match self.doc.move_page(from, to) {
                    Ok(()) => {
                        self.document_options_page = to;
                        self.page_name_draft = self.doc.page_names[to].clone().unwrap_or_default();
                        self.selection_order.clear();
                        self.pending = None;
                        self.fit();
                        self.status = format!("Moved page {} to position {}", from + 1, to + 1);
                    }
                    Err(error) => self.status = error,
                }
            }
            if close {
                self.document_options_dialog = false;
            }
        }
        if let Some(page) = self.remove_page_confirmation {
            let mut confirm = false;
            let mut cancel = false;
            let affected_links = self
                .doc
                .objects
                .iter()
                .filter(|object| {
                    object.page != page
                        && matches!(object.kind, Kind::LinkPageButton { target_page, .. } if target_page == page)
                })
                .count();
            egui::Modal::new(egui::Id::new("remove_page_confirmation")).show(root.ctx(), |ui| {
                ui.heading(format!("Remove page {}?", page + 1));
                ui.label("The page and its objects will be removed from this sketch.");
                if affected_links > 0 {
                    ui.label(format!(
                        "{affected_links} Page Link(s) to this page will lose their destination."
                    ));
                }
                ui.horizontal(|ui| {
                    confirm = ui.button("Remove Page").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if confirm {
                match self.doc.remove_page(page) {
                    Ok(()) => {
                        self.document_options_page = page.min(self.doc.page_count - 1);
                        self.page_name_draft = self.doc.page_names[self.document_options_page]
                            .clone()
                            .unwrap_or_default();
                        self.selection_order.clear();
                        self.pending = None;
                        self.fit();
                        self.status = format!("Removed page {}", page + 1);
                    }
                    Err(error) => self.status = error,
                }
                self.remove_page_confirmation = None;
            }
            if cancel {
                self.remove_page_confirmation = None;
            }
        }
        if self.pending_action.is_some() {
            enum Decision {
                Save,
                Discard,
                Cancel,
            }
            let mut decision = None;
            egui::Modal::new(egui::Id::new("unsaved_changes")).show(root.ctx(), |ui| {
                ui.heading("Unsaved changes");
                ui.label("Save this sketch before continuing?");
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        decision = Some(Decision::Save);
                    }
                    if ui.button("Discard").clicked() {
                        decision = Some(Decision::Discard);
                    }
                    if ui.button("Cancel").clicked() {
                        decision = Some(Decision::Cancel);
                    }
                });
            });
            match decision {
                Some(Decision::Save) if self.save(false) => self.finish_pending_action(),
                Some(Decision::Discard) => self.finish_pending_action(),
                Some(Decision::Cancel) => self.pending_action = None,
                _ => {}
            }
        }
    }
}
