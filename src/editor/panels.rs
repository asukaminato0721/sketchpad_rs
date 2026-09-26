use super::{Editor, MarkedAngle, Tool};
use sketchpad_rs::document::{Kind, TextStyleCommand};
use std::time::Instant;

impl Editor {
    pub(super) fn motion_controller_ui(&mut self, root: &mut egui::Ui) {
        if self.show_motion_controller {
            let mut open = true;
            egui::Window::new("Motion Controller")
                .default_pos(egui::pos2(220., 45.))
                .open(&mut open)
                .show(root.ctx(), |ui| {
                    ui.label(format!("{} point(s)", self.animations.len()));
                    ui.horizontal(|ui| {
                        if ui
                            .button(if self.animation_paused {
                                "Resume"
                            } else {
                                "Pause"
                            })
                            .clicked()
                            && !self.animations.is_empty()
                        {
                            self.animation_paused = !self.animation_paused;
                            self.animation_last_tick = Instant::now();
                        }
                        if ui.button("Stop").clicked() {
                            self.stop_animation();
                        }
                    });
                    ui.add(egui::Slider::new(&mut self.animation_speed, 0.125..=8.).text("Speed"));
                });
            self.show_motion_controller = open;
        }
    }
}
impl Editor {
    pub(super) fn tools_ui(&mut self, root: &mut egui::Ui) {
        egui::Panel::left("tools").min_size(110.).show(root, |ui| {
            if self.show_toolbox {
                ui.heading("Tools");
            }
            if self.doc.page_count > 1 {
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(self.doc.active_page > 0, egui::Button::new("‹"))
                        .clicked()
                    {
                        self.animations.clear();
                        self.movements.clear();
                        self.presentations.clear();
                        self.doc.active_page -= 1;
                        self.doc.selected.clear();
                        self.selection_order.clear();
                        self.fit();
                    }
                    ui.label(format!(
                        "{}/{}",
                        self.doc.active_page + 1,
                        self.doc.page_count
                    ));
                    if let Some(name) = &self.doc.page_names[self.doc.active_page] {
                        ui.label(name);
                    }
                    if ui
                        .add_enabled(
                            self.doc.active_page + 1 < self.doc.page_count,
                            egui::Button::new("›"),
                        )
                        .clicked()
                    {
                        self.animations.clear();
                        self.movements.clear();
                        self.presentations.clear();
                        self.doc.active_page += 1;
                        self.doc.selected.clear();
                        self.selection_order.clear();
                        self.fit();
                    }
                });
                ui.separator();
            }
            if self.show_toolbox {
                for (tool, name) in [
                    (Tool::Select, "Select"),
                    (Tool::Point, "Point"),
                    (Tool::Segment, "Segment"),
                    (Tool::Circle, "Circle"),
                    (Tool::Text, "Text"),
                ] {
                    if ui.selectable_label(self.tool == tool, name).clicked() {
                        self.tool = tool;
                        self.pending = None;
                    }
                }
                if self.tool == Tool::Text {
                    ui.label("New text");
                    ui.text_edit_multiline(&mut self.text_draft);
                    ui.label("Click the sketch to place it.");
                }
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
                    ui.label(format!("Marked center: {id}"));
                }
                if let Some(id) = self.marked_mirror {
                    ui.label(format!("Marked mirror: {id}"));
                }
                if let Some((start, end)) = self.marked_vector {
                    ui.label(format!("Marked vector: {start} → {end}"));
                }
                if let Some(id) = self.marked_distance {
                    ui.label(format!("Marked distance: {id}"));
                }
                if let Some((horizontal, vertical)) = self.marked_rectangular_distances {
                    ui.label(format!(
                        "Marked distances: horizontal {horizontal}, vertical {vertical}"
                    ));
                }
                if let Some(MarkedAngle::Points([first, vertex, last])) = self.marked_angle {
                    ui.label(format!("Marked angle: {first}–{vertex}–{last}"));
                }
                if let Some(MarkedAngle::Value(id)) = self.marked_angle {
                    ui.label(format!("Marked angle value: {id}"));
                }
                if let Some(id) = self.marked_ratio_value {
                    ui.label(format!("Marked ratio value: {id}"));
                }
                if let Some([origin, base, target]) = self.marked_ratio {
                    ui.label(format!("Marked ratio: {origin}–{base}–{target}"));
                }
            }
            if let Some(id) = self.doc.selected.iter().next().copied() {
                let iteration_id = match self.doc.objects[id - 1].kind {
                    Kind::MappedPointIteration { .. } => Some(id),
                    Kind::IterationBinding(_, parent) => Some(parent),
                    _ => None,
                };
                if let Some(iteration_id) = iteration_id
                    && let Some(object) = self.doc.objects.get(iteration_id - 1)
                    && let Kind::MappedPointIteration { mut final_only, .. } = object.kind
                {
                    ui.separator();
                    if ui
                        .checkbox(&mut final_only, "Show only final iteration")
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
                    self.edit_buffer = match &self.doc.objects[id - 1].kind {
                        Kind::PlainText { text, .. } => text.clone(),
                        _ => self.doc.objects[id - 1]
                            .label
                            .as_ref()
                            .and_then(|label| label.name_utf8().ok())
                            .map(str::to_owned)
                            .unwrap_or_else(|| format!("P{id}")),
                    };
                }
                if matches!(self.doc.objects[id - 1].kind, Kind::PlainText { .. }) {
                    ui.separator();
                    ui.label("Edit selected text");
                    let edit = egui::TextEdit::multiline(&mut self.edit_buffer).show(ui);
                    if let Some(range) = edit.cursor_range {
                        if !range.is_empty() {
                            let chars = range.as_sorted_char_range();
                            self.text_selected_range = Some((id, chars.start.0, chars.end.0));
                        } else if edit.response.has_focus() {
                            self.text_selected_range = None;
                        }
                    }
                    let selected_range = self
                        .text_selected_range
                        .and_then(|(target, start, end)| (target == id).then_some((start, end)));
                    if ui.button("Apply text").clicked() {
                        if let Err(error) = self.doc.edit_text(id, &self.edit_buffer) {
                            self.status = error;
                        }
                    }
                    ui.horizontal(|ui| {
                        for (label, command) in [
                            ("Bold", TextStyleCommand::Bold),
                            ("Italic", TextStyleCommand::Italic),
                            ("Underline", TextStyleCommand::Underline),
                        ] {
                            if ui.button(label).clicked() {
                                let current = match &self.doc.objects[id - 1].kind {
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
                            if ui.button(label).clicked() {
                                let current = match &self.doc.objects[id - 1].kind {
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
                                .hint_text("Font family"),
                        );
                        if ui.button("Font for selection").clicked() {
                            let current = match &self.doc.objects[id - 1].kind {
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
                                    Ok(()) => format!("Changed font on characters {start}–{end}"),
                                    Err(error) => error,
                                }
                            } else {
                                "Select characters in the text field first".into()
                            };
                        }
                    });
                } else if self.doc.position(id).is_some() {
                    ui.separator();
                    ui.label("Point label");
                    ui.text_edit_singleline(&mut self.edit_buffer);
                    let visible = self.doc.objects[id - 1]
                        .label
                        .as_ref()
                        .is_some_and(|label| label.visible());
                    if ui
                        .button(if visible { "Hide label" } else { "Show label" })
                        .clicked()
                    {
                        if let Err(error) =
                            self.doc.set_point_label(id, &self.edit_buffer, !visible)
                        {
                            self.status = error;
                        }
                    }
                    if ui.button("Rename label").clicked() {
                        if let Err(error) = self.doc.set_point_label(id, &self.edit_buffer, visible)
                        {
                            self.status = error;
                        }
                    }
                }
            } else {
                self.inspected = None;
            }
            ui.separator();
            ui.label(format!("{} objects", self.doc.objects.len()));
            if self.doc.partial_view {
                let reason = if self.doc.page_count > 1
                    && self
                        .doc
                        .objects
                        .iter()
                        .all(|object| !matches!(object.kind, Kind::Unsupported(_)))
                {
                    "Multi-page editing is limited"
                } else {
                    "Partial view: unsupported objects"
                };
                ui.colored_label(egui::Color32::DARK_RED, reason);
            }
            if self.doc.dirty {
                ui.label("Unsaved changes");
            }
        });
    }
}
impl Editor {
    pub(super) fn status_ui(&mut self, root: &mut egui::Ui) {
        egui::Panel::bottom("status").show(root, |ui| {
            ui.label(if self.status.is_empty() {
                "Drag points. Scroll to zoom. Select two points to construct."
            } else {
                &self.status
            });
        });
    }
}
