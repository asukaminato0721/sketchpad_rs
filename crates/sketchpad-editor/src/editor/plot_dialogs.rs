use super::{Editor, is_circle_construction_host};
use sketchpad_engine::document::Kind;
use sketchpad_engine::plot_points_records::PlotPointsMode;

impl Editor {
    pub(super) fn plot_dialogs_ui(&mut self, root: &mut egui::Ui) {
        if let Some(path) = self.plot_axis_dialog {
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("plot_axis_value")).show(root.ctx(), |ui| {
                ui.heading("Plot Value on Path");
                ui.horizontal(|ui| {
                    ui.label("Value");
                    ui.text_edit_singleline(&mut self.plot_axis_draft);
                });
                ui.label(match path {
                    Some(id) if matches!(self.doc.objects[id - 1].kind, Kind::Segment(_, _)) => {
                        format!("Segment object {id} (0 to 1)")
                    }
                    Some(id) if matches!(self.doc.objects[id - 1].kind, Kind::Ray(_, _)) => {
                        format!("Ray object {id} (0 or greater)")
                    }
                    Some(id) if matches!(self.doc.objects[id - 1].kind, Kind::Line(_, _)) => {
                        format!("Line object {id}")
                    }
                    Some(id) if is_circle_construction_host(&self.doc.objects[id - 1].kind) => {
                        format!("Circle object {id} (counterclockwise turns)")
                    }
                    Some(id) => format!("Axis object {id}"),
                    None => "Active X axis".into(),
                });
                ui.horizontal(|ui| {
                    apply = ui.button("Plot").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.plot_axis_dialog = None;
            }
            if apply {
                let result = self
                    .plot_axis_draft
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| "Enter a numeric value".to_string())
                    .and_then(|value| {
                        if !value.is_finite() {
                            return Err("Enter a finite value".into());
                        }
                        if path.is_none() && self.doc.active_coordinate_system().is_none() {
                            self.doc.add_default_coordinate_system(self.center)?;
                        }
                        self.doc.plot_literal_on_path(value, path)
                    });
                match result {
                    Ok(id) => {
                        self.plot_axis_dialog = None;
                        self.select_only(id);
                        self.status = format!("Plotted path value {id}");
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if let Some(mode) = self.plot_points_dialog {
            let mut apply = false;
            let mut done = false;
            egui::Modal::new(egui::Id::new("plot_points")).show(root.ctx(), |ui| {
                ui.heading("Plot Points");
                ui.horizontal(|ui| {
                    ui.label(if mode == PlotPointsMode::Cartesian {
                        "X"
                    } else {
                        "Radius"
                    });
                    ui.text_edit_singleline(&mut self.plot_points_first_draft);
                });
                ui.horizontal(|ui| {
                    ui.label(if mode == PlotPointsMode::Cartesian {
                        "Y"
                    } else {
                        "Angle (degrees)"
                    });
                    ui.text_edit_singleline(&mut self.plot_points_second_draft);
                });
                ui.horizontal(|ui| {
                    apply = ui.button("Plot").clicked();
                    done = ui.button("Done").clicked();
                });
            });
            if done {
                self.plot_points_dialog = None;
            }
            if apply {
                let coordinates = [
                    self.plot_points_first_draft.trim().parse::<f64>(),
                    self.plot_points_second_draft.trim().parse::<f64>(),
                ];
                let result = match coordinates {
                    [Ok(first), Ok(second)] => self.doc.plot_points_at(mode, [first, second]),
                    _ => Err("Enter two numeric coordinates".into()),
                };
                match result {
                    Ok(id) => {
                        self.select_only(id);
                        self.status = format!("Plotted point {id}");
                        self.plot_points_first_draft.clear();
                        self.plot_points_second_draft.clear();
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if let Some(table) = self.plot_table_dialog {
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("plot_table_data")).show(root.ctx(), |ui| {
                ui.heading("Plot Table Data");
                ui.label("First coordinate: t₂");
                ui.label("Second coordinate: t₁");
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut self.plot_table_mode,
                        PlotPointsMode::Cartesian,
                        "Cartesian (x, y)",
                    );
                    ui.radio_value(
                        &mut self.plot_table_mode,
                        PlotPointsMode::Polar,
                        "Polar (r, θ in degrees)",
                    );
                });
                ui.horizontal(|ui| {
                    apply = ui.button("Plot").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.plot_table_dialog = None;
            }
            if apply {
                match self.doc.plot_pair_table_data(table, self.plot_table_mode) {
                    Ok(ids) => {
                        self.plot_table_dialog = None;
                        if let Some(id) = ids.last() {
                            self.select_only(*id);
                        }
                        self.status = format!("Plotted {} table data point(s)", ids.len());
                    }
                    Err(error) => self.status = error,
                }
            }
        }
    }
}
