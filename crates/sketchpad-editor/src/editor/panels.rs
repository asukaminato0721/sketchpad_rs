use super::i18n::tr;
use super::{Editor, MarkedAngle, Tool};
use sketchpad_engine::document::{EditRestriction, Kind, TextStyleCommand};
use std::time::Instant;

impl Editor {
    pub(super) fn motion_controller_ui(&mut self, root: &mut egui::Ui) {
        if self.show_motion_controller {
            let mut open = true;
            egui::Window::new(tr("Motion Controller"))
                .id(egui::Id::new("Motion Controller"))
                .default_pos(egui::pos2(220., 45.))
                .open(&mut open)
                .show(root.ctx(), |ui| {
                    ui.label(tr(format!("{} point(s)", self.animations.len())));
                    ui.horizontal(|ui| {
                        if ui
                            .button(tr(if self.animation_paused {
                                "Resume"
                            } else {
                                "Pause"
                            }))
                            .clicked()
                            && !self.animations.is_empty()
                        {
                            self.animation_paused = !self.animation_paused;
                            self.animation_last_tick = Instant::now();
                        }
                        if ui.button(tr("Stop")).clicked() {
                            self.stop_animation();
                        }
                    });
                    ui.add(
                        egui::Slider::new(&mut self.animation_speed, 0.125..=8.).text(tr("Speed")),
                    );
                });
            self.show_motion_controller = open;
        }
    }
}

impl Editor {
    pub(super) fn apply_selected_text_style(&mut self, command: TextStyleCommand) {
        self.status = match self.doc.toggle_selected_text_style(command) {
            Ok(count) => format!("Formatted {count} text object(s)"),
            Err(error) => error,
        };
    }

    pub(super) fn apply_selected_text_family(&mut self) {
        self.status = match self
            .doc
            .set_selected_text_family(self.text_font_input.trim())
        {
            Ok(count) => format!("Changed font on {count} text object(s)"),
            Err(error) => error,
        };
    }

    pub(super) fn text_palette_ui(&mut self, root: &mut egui::Ui) {
        if !self.show_text_palette {
            return;
        }

        let mut open = true;
        egui::Window::new(tr("Text Palette"))
            .id(egui::Id::new("Text Palette"))
            .default_pos(egui::pos2(220., 175.))
            .resizable(false)
            .open(&mut open)
            .show(root.ctx(), |ui| {
                if let Some(style) = self
                    .doc
                    .selected
                    .iter()
                    .find_map(|id| self.doc.text_font_state(*id))
                {
                    ui.label(tr(format!("{} — {} pt", style.family, style.size_code)));
                } else {
                    ui.label(tr("Select text, an object label, or a numeric caption."));
                }
                ui.horizontal(|ui| {
                    for (label, command) in [
                        ("Bold", TextStyleCommand::Bold),
                        ("Italic", TextStyleCommand::Italic),
                        ("Underline", TextStyleCommand::Underline),
                    ] {
                        if ui.button(tr(label)).clicked() {
                            self.apply_selected_text_style(command);
                        }
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button(tr("Smaller")).clicked() {
                        self.apply_selected_text_style(TextStyleCommand::DecreaseSize);
                    }
                    if ui.button(tr("Larger")).clicked() {
                        self.apply_selected_text_style(TextStyleCommand::IncreaseSize);
                    }
                });
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.text_font_input)
                            .hint_text(tr("Font family")),
                    );
                    if ui.button(tr("Apply Font")).clicked() {
                        self.apply_selected_text_family();
                    }
                });
            });
        self.show_text_palette = open;
    }
}

impl Editor {
    pub(super) fn tools_ui(&mut self, root: &mut egui::Ui) {
        if !self.show_inspector {
            return;
        }
        let mut open = true;
        egui::Window::new(tr("Object Information"))
            .id(egui::Id::new("object_information"))
            .default_width(380.)
            .default_height(620.)
            .open(&mut open)
            .vscroll(true)
            .show(root.ctx(), |ui| {
                self.dependency_inspector_ui(ui);
                ui.separator();
                if self.tool == Tool::Text {
                    ui.label(tr("New text"));
                    ui.text_edit_multiline(&mut self.text_draft);
                    ui.label(tr("Click the sketch to place it."));
                }
                if self.marked_center.is_some()
                    || self.marked_mirror.is_some()
                    || self.marked_vector.is_some()
                    || self.marked_distance.is_some()
                    || self.marked_rectangular_distances.is_some()
                    || self.marked_angle.is_some()
                    || self.marked_ratio.is_some()
                    || self.marked_ratio_value.is_some()
                {
                    ui.separator();
                    if let Some(id) = self.marked_center {
                        ui.label(tr(format!("Marked center: {id}")));
                    }
                    if let Some(id) = self.marked_mirror {
                        ui.label(tr(format!("Marked mirror: {id}")));
                    }
                    if let Some((start, end)) = self.marked_vector {
                        ui.label(tr(format!("Marked vector: {start} → {end}")));
                    }
                    if let Some(id) = self.marked_distance {
                        ui.label(tr(format!("Marked distance: {id}")));
                    }
                    if let Some((horizontal, vertical)) = self.marked_rectangular_distances {
                        ui.label(tr(format!(
                            "Marked distances: horizontal {horizontal}, vertical {vertical}"
                        )));
                    }
                    if let Some(MarkedAngle::Points([first, vertex, last])) = self.marked_angle {
                        ui.label(tr(format!("Marked angle: {first}–{vertex}–{last}")));
                    }
                    if let Some(MarkedAngle::Value(id)) = self.marked_angle {
                        ui.label(tr(format!("Marked angle value: {id}")));
                    }
                    if let Some(id) = self.marked_ratio_value {
                        ui.label(tr(format!("Marked ratio value: {id}")));
                    }
                    if let Some([origin, base, target]) = self.marked_ratio {
                        ui.label(tr(format!("Marked ratio: {origin}–{base}–{target}")));
                    }
                }
                if let Some(id) = self.doc.selected.iter().next().copied() {
                    let iteration_id = match self.doc.objects()[id - 1].kind {
                        Kind::MappedPointIteration { .. } => Some(id),
                        Kind::IterationBinding(_, parent) => Some(parent),
                        _ => None,
                    };
                    if let Some(iteration_id) = iteration_id
                        && let Some(object) = self.doc.objects().get(iteration_id - 1)
                        && let Kind::MappedPointIteration { mut final_only, .. } = object.kind
                    {
                        ui.separator();
                        if ui
                            .checkbox(&mut final_only, tr("Show only final iteration"))
                            .changed()
                        {
                            self.status = match self
                                .doc
                                .set_mapped_iteration_final_only(iteration_id, final_only)
                            {
                                Ok(()) => "Updated iteration images".into(),
                                Err(error) => error,
                            };
                        }
                    }
                    if self.inspected != Some(id) {
                        self.inspected = Some(id);
                        self.edit_buffer = match &self.doc.objects()[id - 1].kind {
                            Kind::PlainText { text, .. } => text.clone(),
                            _ => self.doc.objects()[id - 1]
                                .label
                                .as_ref()
                                .and_then(|label| label.name_utf8().ok())
                                .map(str::to_owned)
                                .unwrap_or_else(|| format!("P{id}")),
                        };
                    }
                    if matches!(self.doc.objects()[id - 1].kind, Kind::PlainText { .. }) {
                        ui.separator();
                        ui.label(tr("Edit selected text"));
                        let edit = egui::TextEdit::multiline(&mut self.edit_buffer).show(ui);
                        if let Some(range) = edit.cursor_range {
                            if !range.is_empty() {
                                let chars = range.as_sorted_char_range();
                                self.text_selected_range = Some((id, chars.start.0, chars.end.0));
                            } else if edit.response.has_focus() {
                                self.text_selected_range = None;
                            }
                        }
                        let selected_range =
                            self.text_selected_range.and_then(|(target, start, end)| {
                                (target == id).then_some((start, end))
                            });
                        if ui.button(tr("Apply text")).clicked()
                            && let Err(error) = self.doc.edit_text(id, &self.edit_buffer)
                        {
                            self.status = error;
                        }
                        ui.horizontal(|ui| {
                            for (label, command) in [
                                ("Bold", TextStyleCommand::Bold),
                                ("Italic", TextStyleCommand::Italic),
                                ("Underline", TextStyleCommand::Underline),
                            ] {
                                if ui.button(tr(label)).clicked() {
                                    let current = match &self.doc.objects()[id - 1].kind {
                                        Kind::PlainText { text, .. } => text.as_str(),
                                        _ => "",
                                    };
                                    self.status = if self.edit_buffer != current {
                                        "Apply the text edit before formatting a selection".into()
                                    } else if let Some((start, end)) = selected_range {
                                        match self.doc.format_text_range(id, start, end, command) {
                                            Ok(()) => format!("Formatted characters {start}–{end}"),
                                            Err(error) => error,
                                        }
                                    } else {
                                        "Select characters in the text field first".into()
                                    };
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            for (label, command) in [
                                ("Larger", TextStyleCommand::IncreaseSize),
                                ("Smaller", TextStyleCommand::DecreaseSize),
                            ] {
                                if ui.button(tr(label)).clicked() {
                                    let current = match &self.doc.objects()[id - 1].kind {
                                        Kind::PlainText { text, .. } => text.as_str(),
                                        _ => "",
                                    };
                                    self.status = if self.edit_buffer != current {
                                        "Apply the text edit before formatting a selection".into()
                                    } else if let Some((start, end)) = selected_range {
                                        match self.doc.format_text_range(id, start, end, command) {
                                            Ok(()) => format!("Formatted characters {start}–{end}"),
                                            Err(error) => error,
                                        }
                                    } else {
                                        "Select characters in the text field first".into()
                                    };
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.text_font_input)
                                    .hint_text(tr("Font family")),
                            );
                            if ui.button(tr("Font for selection")).clicked() {
                                let current = match &self.doc.objects()[id - 1].kind {
                                    Kind::PlainText { text, .. } => text.as_str(),
                                    _ => "",
                                };
                                self.status = if self.edit_buffer != current {
                                    "Apply the text edit before formatting a selection".into()
                                } else if let Some((start, end)) = selected_range {
                                    match self.doc.format_text_range_family(
                                        id,
                                        start,
                                        end,
                                        self.text_font_input.trim(),
                                    ) {
                                        Ok(()) => {
                                            format!("Changed font on characters {start}–{end}")
                                        }
                                        Err(error) => error,
                                    }
                                } else {
                                    "Select characters in the text field first".into()
                                };
                            }
                        });
                    } else if self.doc.can_edit_geometry_label(id) {
                        ui.separator();
                        ui.label(tr("Object label"));
                        ui.text_edit_singleline(&mut self.edit_buffer);
                        let visible = self.doc.objects()[id - 1]
                            .label
                            .as_ref()
                            .is_some_and(|label| label.visible());
                        if ui
                            .button(tr(if visible { "Hide label" } else { "Show label" }))
                            .clicked()
                        {
                            self.status =
                                match self.doc.set_object_label(id, &self.edit_buffer, !visible) {
                                    Ok(true) => "Updated object label".into(),
                                    Ok(false) => "Object label is unchanged".into(),
                                    Err(error) => error,
                                };
                        }
                        if ui.button(tr("Rename label")).clicked() {
                            self.status =
                                match self.doc.set_object_label(id, &self.edit_buffer, visible) {
                                    Ok(true) => "Renamed object label".into(),
                                    Ok(false) => "Object label is unchanged".into(),
                                    Err(error) => error,
                                };
                        }
                    }
                } else {
                    self.inspected = None;
                }
                ui.separator();
                if let Some(restriction) = self.doc.edit_restriction() {
                    if restriction == EditRestriction::UnsupportedObjects {
                        ui.colored_label(egui::Color32::DARK_RED, tr("Unsupported objects:"));
                        for object in self.doc.objects().iter().filter(|object| !object.deleted) {
                            let Kind::Unsupported(code) = object.kind else {
                                continue;
                            };
                            let mut description = tr(format!(
                                "Page {page}: object #{id} (type code {code})",
                                page = object.page + 1,
                                id = object.id,
                            ));
                            if let Some(name) = object
                                .label
                                .as_ref()
                                .and_then(|label| label.name_utf8().ok())
                                .filter(|name| !name.is_empty())
                            {
                                description.push_str(" · ");
                                description
                                    .push_str(&super::inspector_description::single_line(name));
                            }
                            ui.colored_label(egui::Color32::DARK_RED, description);
                        }
                    } else {
                        let reason = match restriction {
                            EditRestriction::ReadOnlyImages => "Editing imported images is limited",
                            EditRestriction::MultiPageStructure => "Multi-page editing is limited",
                            EditRestriction::UnrecognizedRecords => {
                                "Editing is limited to preserve unfamiliar records"
                            }
                            EditRestriction::UnsupportedObjects => unreachable!(),
                        };
                        ui.colored_label(egui::Color32::DARK_RED, tr(reason));
                    }
                }
                if self.doc.dirty {
                    ui.label(tr("Unsaved changes"));
                }
            });
        self.show_inspector = open;
    }
}
impl Editor {
    pub(super) fn status_ui(&mut self, root: &mut egui::Ui) {
        egui::Panel::bottom("status").show(root, |ui| {
            ui.label(tr(if self.status.is_empty() {
                "Drag points. Scroll to zoom. Select two points to construct."
            } else {
                &self.status
            }));
        });
    }
}
