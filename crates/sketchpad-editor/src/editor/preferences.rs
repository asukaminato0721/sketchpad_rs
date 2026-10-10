use super::i18n::tr;
use super::{Editor, SelectionMagnetism};
use sketchpad_engine::document::{MeasurementPrecisionDefaults, ObjectClass, ObjectColorDefaults};
use sketchpad_engine::style_records::{LinePattern, LineWidth, PointStyle, RgbColor};

impl Editor {
    fn rgb(color: [u8; 3]) -> sketchpad_engine::style_records::RgbColor {
        sketchpad_engine::style_records::RgbColor {
            red: color[0],
            green: color[1],
            blue: color[2],
        }
    }

    pub(super) fn sync_document_preferences(&mut self) {
        self.doc.set_object_color_defaults(ObjectColorDefaults {
            point: Self::rgb(self.preferences.point_color),
            straight: Self::rgb(self.preferences.straight_color),
            round: Self::rgb(self.preferences.round_color),
            interior: Self::rgb(self.preferences.interior_color),
            point_locus: Self::rgb(self.preferences.point_locus_color),
            plot: Self::rgb(self.preferences.plot_color),
        });
        self.doc
            .set_randomize_interior_colors(self.preferences.randomize_interior_colors);
        self.doc
            .set_object_style_defaults(self.preferences.object_style_defaults);
        self.doc
            .set_new_polygon_opacity(self.preferences.new_polygon_opacity);
        self.doc.set_measurement_precision_defaults(
            MeasurementPrecisionDefaults::new(
                self.preferences.angle_precision,
                self.preferences.distance_precision,
                self.preferences.other_precision,
            )
            .expect("preference precision is constrained to the native range"),
        );
    }

    fn selected_object_classes(&self) -> std::collections::BTreeSet<ObjectClass> {
        self.doc
            .selected
            .iter()
            .filter_map(|id| self.doc.object_class(*id))
            .collect()
    }

    pub(super) fn apply_selected_color(&mut self, color: RgbColor, update_defaults: bool) {
        let classes = self.selected_object_classes();
        self.status = match self.doc.set_selected_color(color) {
            Ok(0) => "Select a styled geometry object".into(),
            Ok(count) => format!("Changed color on {count} object(s)"),
            Err(error) => return self.status = error,
        };
        if !update_defaults || !self.preferences.update_colors_automatically || classes.is_empty() {
            return;
        }
        let value = [color.red, color.green, color.blue];
        let mut defaults = self.doc.object_color_defaults();
        for class in classes {
            defaults.set_color(class, color);
            match class {
                ObjectClass::Point => self.preferences.point_color = value,
                ObjectClass::Straight => self.preferences.straight_color = value,
                ObjectClass::Round => self.preferences.round_color = value,
                ObjectClass::Interior => self.preferences.interior_color = value,
                ObjectClass::PointLocus => self.preferences.point_locus_color = value,
                ObjectClass::Plot => self.preferences.plot_color = value,
            }
        }
        self.preferences_draft = self.preferences;
        self.doc.set_object_color_defaults(defaults);
    }

    pub(super) fn apply_selected_point_style(&mut self, style: PointStyle, update_defaults: bool) {
        let has_points = self.selected_object_classes().contains(&ObjectClass::Point);
        self.status = match self.doc.set_selected_point_style(style) {
            Ok(0) => "Select at least one point".into(),
            Ok(count) => format!("Changed point style on {count} object(s)"),
            Err(error) => return self.status = error,
        };
        if update_defaults && has_points {
            self.preferences
                .object_style_defaults
                .set_point_style(style);
            self.preferences_draft = self.preferences;
            self.doc
                .set_object_style_defaults(self.preferences.object_style_defaults);
        }
    }

    pub(super) fn apply_selected_line_width(&mut self, width: LineWidth, update_defaults: bool) {
        let classes = self.selected_object_classes();
        self.status = match self.doc.set_selected_line_width(width) {
            Ok(0) => "Select at least one line or circle".into(),
            Ok(count) => format!("Changed line width on {count} object(s)"),
            Err(error) => return self.status = error,
        };
        if update_defaults {
            let mut changed = false;
            for class in classes {
                changed |= self
                    .preferences
                    .object_style_defaults
                    .set_line_width(class, width);
            }
            if changed {
                self.preferences_draft = self.preferences;
                self.doc
                    .set_object_style_defaults(self.preferences.object_style_defaults);
            }
        }
    }

    pub(super) fn apply_selected_line_pattern(
        &mut self,
        pattern: LinePattern,
        update_defaults: bool,
    ) {
        let classes = self.selected_object_classes();
        self.status = match self.doc.set_selected_line_pattern(pattern) {
            Ok(0) => "Select at least one line or circle".into(),
            Ok(count) => format!("Changed line pattern on {count} object(s)"),
            Err(error) => return self.status = error,
        };
        if update_defaults {
            let mut changed = false;
            for class in classes {
                changed |= self
                    .preferences
                    .object_style_defaults
                    .set_line_pattern(class, pattern);
            }
            if changed {
                self.preferences_draft = self.preferences;
                self.doc
                    .set_object_style_defaults(self.preferences.object_style_defaults);
            }
        }
    }

    pub(super) fn selection_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(
            self.preferences.selection_color[0],
            self.preferences.selection_color[1],
            self.preferences.selection_color[2],
        )
    }

    pub(super) fn preferences_dialog_ui(&mut self, root: &mut egui::Ui) {
        if !self.preferences_dialog {
            return;
        }
        let mut apply = false;
        let mut cancel = false;
        egui::Modal::new(egui::Id::new("preferences")).show(root.ctx(), |ui| {
            ui.heading(tr("Preferences"));
            ui.label(tr("Units"));
            ui.separator();
            egui::Grid::new("preference_precision")
                .num_columns(2)
                .spacing([12., 6.])
                .show(ui, |ui| {
                    for (label, precision) in [
                        (
                            "Angle precision",
                            &mut self.preferences_draft.angle_precision,
                        ),
                        (
                            "Distance precision",
                            &mut self.preferences_draft.distance_precision,
                        ),
                        (
                            "Other precision",
                            &mut self.preferences_draft.other_precision,
                        ),
                    ] {
                        ui.label(tr(label));
                        ui.add(egui::Slider::new(precision, 0..=5).text(tr("decimal places")));
                        ui.end_row();
                    }
                });
            ui.add_space(8.);
            ui.label(tr("Tools"));
            ui.separator();
            ui.checkbox(
                &mut self.preferences_draft.double_click_deselection,
                tr("Require a double-click in empty space to deselect"),
            );
            ui.add_space(8.);
            ui.label(tr("Selection magnetism"));
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.preferences_draft.selection_magnetism,
                    SelectionMagnetism::Low,
                    tr("Low"),
                );
                ui.selectable_value(
                    &mut self.preferences_draft.selection_magnetism,
                    SelectionMagnetism::Medium,
                    tr("Medium"),
                );
                ui.selectable_value(
                    &mut self.preferences_draft.selection_magnetism,
                    SelectionMagnetism::High,
                    tr("High"),
                );
            });
            ui.add(
                egui::Slider::new(
                    &mut self.preferences_draft.new_polygon_opacity,
                    0..=u16::MAX,
                )
                .text(tr("New polygon opacity"))
                .custom_formatter(|value, _| format!("{:.0}%", value * 100. / 65535.)),
            );
            ui.add_space(8.);
            ui.label(tr("Color"));
            ui.separator();
            ui.checkbox(
                &mut self.preferences_draft.update_colors_automatically,
                tr("Update defaults when applying colors"),
            );
            ui.checkbox(
                &mut self.preferences_draft.randomize_interior_colors,
                tr("Randomly color new interiors"),
            );
            egui::Grid::new("preference_colors")
                .num_columns(2)
                .spacing([12., 6.])
                .show(ui, |ui| {
                    ui.label(tr("Sketch background"));
                    ui.color_edit_button_srgb(&mut self.preferences_draft.background_color);
                    ui.end_row();
                    ui.label(tr("Selection markers"));
                    ui.color_edit_button_srgb(&mut self.preferences_draft.selection_color);
                    ui.end_row();
                    ui.label(tr("New points"));
                    ui.color_edit_button_srgb(&mut self.preferences_draft.point_color);
                    ui.end_row();
                    ui.label(tr("New lines, rays, and segments"));
                    ui.color_edit_button_srgb(&mut self.preferences_draft.straight_color);
                    ui.end_row();
                    ui.label(tr("New circles and arcs"));
                    ui.color_edit_button_srgb(&mut self.preferences_draft.round_color);
                    ui.end_row();
                    ui.label(tr("New interiors"));
                    ui.color_edit_button_srgb(&mut self.preferences_draft.interior_color);
                    ui.end_row();
                    ui.label(tr("New point loci"));
                    ui.color_edit_button_srgb(&mut self.preferences_draft.point_locus_color);
                    ui.end_row();
                    ui.label(tr("New plots"));
                    ui.color_edit_button_srgb(&mut self.preferences_draft.plot_color);
                    ui.end_row();
                });
            ui.checkbox(
                &mut self.preferences_draft.fade_traces,
                tr("Fade traces over time"),
            );
            ui.add_enabled(
                self.preferences_draft.fade_traces,
                egui::Slider::new(&mut self.preferences_draft.trace_fade_seconds, 0.5..=20.)
                    .logarithmic(true)
                    .suffix(" s")
                    .text(tr("Trace lifetime")),
            );
            ui.add_space(8.);
            ui.label(tr("Text"));
            ui.separator();
            ui.checkbox(
                &mut self.preferences_draft.show_text_palette_for_new_captions,
                tr("Show the Text Palette when creating a caption"),
            );
            ui.add_space(8.);
            ui.weak(tr("These settings apply to this editor session."));
            ui.horizontal(|ui| {
                apply = ui.button(tr("Apply")).clicked();
                cancel = ui.button(tr("Cancel")).clicked();
            });
        });
        if apply {
            self.preferences = self.preferences_draft;
            self.sync_document_preferences();
            self.preferences_dialog = false;
            self.status = "Updated tool preferences".into();
        } else if cancel {
            self.preferences_dialog = false;
        }
    }
}
