use super::{Editor, MarkedAngle, TranslationMode};

impl Editor {
    pub(super) fn transform_dialogs_ui(&mut self, root: &mut egui::Ui) {
        if self.text_font_dialog {
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("text_font_family")).show(root.ctx(), |ui| {
                ui.heading("Font family");
                ui.text_edit_singleline(&mut self.text_font_input);
                ui.horizontal(|ui| {
                    apply = ui.button("Apply").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if apply {
                let family = self.text_font_input.trim();
                match self.doc.set_selected_text_family(family) {
                    Ok(count) => {
                        self.status = format!("Changed font family on {count} text object(s)");
                        self.text_font_dialog = false;
                    }
                    Err(error) => self.status = error,
                }
            } else if cancel {
                self.text_font_dialog = false;
            }
        }
        if self.translate_dialog {
            let mut angle = self.translation_angle;
            let mut distance = self.translation_distance;
            let mut horizontal = self.translation_horizontal;
            let mut vertical = self.translation_vertical;
            let mut mode = self.translation_mode;
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("polar_translation")).show(root.ctx(), |ui| {
                ui.heading("Translate point");
                ui.radio_value(
                    &mut mode,
                    TranslationMode::FixedPolar,
                    "Polar: fixed distance",
                );
                ui.radio_value(
                    &mut mode,
                    TranslationMode::FixedRectangular,
                    "Rectangular: fixed distances",
                );
                ui.add_enabled_ui(self.marked_distance.is_some(), |ui| {
                    ui.radio_value(
                        &mut mode,
                        TranslationMode::MarkedDistance,
                        "Polar: marked distance",
                    );
                });
                ui.add_enabled_ui(self.marked_rectangular_distances.is_some(), |ui| {
                    ui.radio_value(
                        &mut mode,
                        TranslationMode::RectangularMarked,
                        "Rectangular: marked distances",
                    );
                    ui.radio_value(
                        &mut mode,
                        TranslationMode::RectangularHorizontalFixed,
                        "Rectangular: fixed horizontal, marked vertical",
                    );
                    ui.radio_value(
                        &mut mode,
                        TranslationMode::RectangularVerticalFixed,
                        "Rectangular: marked horizontal, fixed vertical",
                    );
                });
                ui.add_enabled_ui(self.marked_vector.is_some(), |ui| {
                    ui.radio_value(&mut mode, TranslationMode::MarkedVector, "Marked vector");
                });
                if mode == TranslationMode::MarkedVector {
                    if let Some((start, end)) = self.marked_vector {
                        ui.label(format!("Point {start} → point {end}"));
                    }
                } else if mode == TranslationMode::RectangularMarked {
                    if let Some((horizontal, vertical)) = self.marked_rectangular_distances {
                        ui.label(format!(
                            "Horizontal distance {horizontal}; vertical distance {vertical}"
                        ));
                    }
                } else if mode == TranslationMode::RectangularHorizontalFixed {
                    if let Some((_, vertical_id)) = self.marked_rectangular_distances {
                        ui.label(format!("Vertical distance value {vertical_id}"));
                    }
                    ui.horizontal(|ui| {
                        ui.label("Horizontal (cm)");
                        ui.add(egui::DragValue::new(&mut horizontal).speed(0.1));
                    });
                } else if mode == TranslationMode::RectangularVerticalFixed {
                    if let Some((horizontal_id, _)) = self.marked_rectangular_distances {
                        ui.label(format!("Horizontal distance value {horizontal_id}"));
                    }
                    ui.horizontal(|ui| {
                        ui.label("Vertical (cm)");
                        ui.add(egui::DragValue::new(&mut vertical).speed(0.1));
                    });
                } else if mode == TranslationMode::FixedRectangular {
                    ui.horizontal(|ui| {
                        ui.label("Horizontal (cm)");
                        ui.add(egui::DragValue::new(&mut horizontal).speed(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Vertical (cm)");
                        ui.add(egui::DragValue::new(&mut vertical).speed(0.1));
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Angle (degrees)");
                        ui.add(egui::DragValue::new(&mut angle).speed(1.));
                    });
                    if mode == TranslationMode::MarkedDistance {
                        if let Some(id) = self.marked_distance {
                            ui.label(format!("Distance value {id}"));
                        }
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("Distance (cm)");
                            ui.add(egui::DragValue::new(&mut distance).speed(0.1));
                        });
                    }
                }
                ui.horizontal(|ui| {
                    apply = ui.button("Translate").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            self.translation_angle = angle;
            self.translation_distance = distance;
            self.translation_horizontal = horizontal;
            self.translation_vertical = vertical;
            self.translation_mode = mode;
            if apply {
                self.translate_dialog = false;
                self.translate_selected();
            } else if cancel {
                self.translate_dialog = false;
            }
        }
        if self.dilate_dialog {
            let mut factor = self.dilation_factor;
            let mut use_marked = self.dilation_use_marked;
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("dilation_ratio")).show(root.ctx(), |ui| {
                ui.heading("Dilate object");
                ui.radio_value(&mut use_marked, false, "Fixed ratio");
                ui.add_enabled_ui(
                    self.marked_ratio.is_some() || self.marked_ratio_value.is_some(),
                    |ui| {
                        ui.radio_value(&mut use_marked, true, "Marked ratio");
                    },
                );
                if use_marked {
                    if let Some(id) = self.marked_ratio_value {
                        ui.label(format!("Value {id}"));
                    }
                    if let Some([origin, base, target]) = self.marked_ratio {
                        ui.label(format!("Point {origin} → {base} → {target}"));
                    }
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Ratio");
                        ui.add(egui::DragValue::new(&mut factor).speed(0.05));
                    });
                }
                ui.horizontal(|ui| {
                    apply = ui.button("Dilate").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            self.dilation_factor = factor;
            self.dilation_use_marked = use_marked;
            if apply {
                self.dilate_dialog = false;
                self.dilate_selected();
            } else if cancel {
                self.dilate_dialog = false;
            }
        }
        if self.rotate_dialog {
            let mut angle = self.rotation_angle;
            let mut use_marked = self.rotation_use_marked;
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("point_rotation")).show(root.ctx(), |ui| {
                ui.heading("Rotate");
                ui.radio_value(&mut use_marked, false, "Fixed angle");
                ui.add_enabled_ui(self.marked_angle.is_some(), |ui| {
                    ui.radio_value(&mut use_marked, true, "Marked angle");
                });
                if use_marked {
                    if let Some(MarkedAngle::Points([first, vertex, last])) = self.marked_angle {
                        ui.label(format!("Point {first} → {vertex} → {last}"));
                    }
                    if let Some(MarkedAngle::Value(id)) = self.marked_angle {
                        ui.label(format!("Calculated angle {id}"));
                    }
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Angle (degrees)");
                        ui.add(egui::DragValue::new(&mut angle).speed(1.));
                    });
                }
                ui.horizontal(|ui| {
                    apply = ui.button("Rotate").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            self.rotation_angle = angle;
            self.rotation_use_marked = use_marked;
            if apply {
                self.rotate_dialog = false;
                self.rotate_selected();
            } else if cancel {
                self.rotate_dialog = false;
            }
        }
    }
}
