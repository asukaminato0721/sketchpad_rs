use super::i18n::tr;
use super::{Editor, is_circle_construction_host};
use sketchpad_engine::document::Kind;
use sketchpad_engine::plot_points_records::PlotPointsMode;

impl Editor {
    pub(super) fn plot_dialogs_ui(&mut self, root: &mut egui::Ui) {
        if let Some(path) = self.dialogs.plot_axis_dialog {
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("plot_axis_value")).show(root.ctx(), |ui| {
                ui.heading(tr("Plot Value on Path"));
                ui.horizontal(|ui| {
                    ui.label(tr("Value"));
                    ui.text_edit_singleline(&mut self.dialogs.plot_axis_draft);
                });
                ui.label(tr(match path {
                    Some(id) if matches!(self.doc.objects()[id - 1].kind, Kind::Segment(_, _)) => {
                        format!("Segment object {id} (0 to 1)")
                    }
                    Some(id) if matches!(self.doc.objects()[id - 1].kind, Kind::Ray(_, _)) => {
                        format!("Ray object {id} (0 or greater)")
                    }
                    Some(id) if matches!(self.doc.objects()[id - 1].kind, Kind::Line(_, _)) => {
                        format!("Line object {id}")
                    }
                    Some(id) if is_circle_construction_host(&self.doc.objects()[id - 1].kind) => {
                        format!("Circle object {id} (counterclockwise turns)")
                    }
                    Some(id) => format!("Axis object {id}"),
                    None => "Active X axis".into(),
                }));
                ui.horizontal(|ui| {
                    apply = ui.button(tr("Plot")).clicked();
                    cancel = ui.button(tr("Cancel")).clicked();
                });
            });
            if cancel {
                self.dialogs.plot_axis_dialog = None;
            }
            if apply {
                let result = self
                    .dialogs
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
                        self.dialogs.plot_axis_dialog = None;
                        self.select_only(id);
                        self.status = format!("Plotted path value {id}");
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if let Some(mode) = self.dialogs.plot_points_dialog {
            let mut apply = false;
            let mut done = false;
            egui::Modal::new(egui::Id::new("plot_points")).show(root.ctx(), |ui| {
                ui.heading(tr("Plot Points"));
                ui.horizontal(|ui| {
                    ui.label(tr(if mode == PlotPointsMode::Cartesian {
                        "X"
                    } else {
                        "Radius"
                    }));
                    ui.text_edit_singleline(&mut self.dialogs.plot_points_first_draft);
                });
                ui.horizontal(|ui| {
                    ui.label(tr(if mode == PlotPointsMode::Cartesian {
                        "Y"
                    } else {
                        "Angle (degrees)"
                    }));
                    ui.text_edit_singleline(&mut self.dialogs.plot_points_second_draft);
                });
                ui.horizontal(|ui| {
                    apply = ui.button(tr("Plot")).clicked();
                    done = ui.button(tr("Done")).clicked();
                });
            });
            if done {
                self.dialogs.plot_points_dialog = None;
            }
            if apply {
                let coordinates = [
                    self.dialogs.plot_points_first_draft.trim().parse::<f64>(),
                    self.dialogs.plot_points_second_draft.trim().parse::<f64>(),
                ];
                let result = match coordinates {
                    [Ok(first), Ok(second)] => self.doc.plot_points_at(mode, [first, second]),
                    _ => Err("Enter two numeric coordinates".into()),
                };
                match result {
                    Ok(id) => {
                        self.select_only(id);
                        self.status = format!("Plotted point {id}");
                        self.dialogs.plot_points_first_draft.clear();
                        self.dialogs.plot_points_second_draft.clear();
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if let Some(table) = self.dialogs.plot_table_dialog {
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("plot_table_data")).show(root.ctx(), |ui| {
                ui.heading(tr("Plot Table Data"));
                ui.label(tr("First coordinate: t₂"));
                ui.label(tr("Second coordinate: t₁"));
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut self.dialogs.plot_table_mode,
                        PlotPointsMode::Cartesian,
                        tr("Cartesian (x, y)"),
                    );
                    ui.radio_value(
                        &mut self.dialogs.plot_table_mode,
                        PlotPointsMode::Polar,
                        tr("Polar (r, θ in degrees)"),
                    );
                });
                ui.horizontal(|ui| {
                    apply = ui.button(tr("Plot")).clicked();
                    cancel = ui.button(tr("Cancel")).clicked();
                });
            });
            if cancel {
                self.dialogs.plot_table_dialog = None;
            }
            if apply {
                match self
                    .doc
                    .plot_pair_table_data(table, self.dialogs.plot_table_mode)
                {
                    Ok(ids) => {
                        self.dialogs.plot_table_dialog = None;
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
