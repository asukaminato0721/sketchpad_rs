use super::Editor;
use sketchpad_engine::document::Kind;
use sketchpad_engine::function_records::SimpleFunction;
use sketchpad_engine::plot_points_records::PlotPointsMode;

impl Editor {
    pub(super) fn begin_function_dialog(&mut self, plot: bool, edit_id: Option<usize>) {
        self.function_edit_id = edit_id;
        self.function_dialog = Some(plot);
        self.function_angle_dialog = None;
        self.function_draft_sources = if let Some(id) = edit_id {
            self.doc.function_sources(id).unwrap_or_default().to_vec()
        } else {
            let mut selected = self
                .selection_order
                .iter()
                .copied()
                .filter(|id| self.doc.selected.contains(id))
                .collect::<Vec<_>>();
            for &id in &self.doc.selected {
                if !selected.contains(&id) {
                    selected.push(id);
                }
            }
            selected
                .into_iter()
                .filter(|&id| {
                    self.doc
                        .objects
                        .get(id.saturating_sub(1))
                        .is_some_and(|object| {
                            !object.deleted
                                && object.page == self.doc.active_page
                                && (object.kind.numeric_unit().is_some()
                                    || self.doc.function_sources(id).is_some())
                        })
                })
                .collect()
        };
        if let Some(id) = edit_id {
            self.function_draft = match &self.doc.objects[id - 1].kind {
                Kind::FunctionDefinition(expression) | Kind::ReferencedFunction(_, expression) => {
                    expression
                        .display()
                        .strip_prefix("f(x) = ")
                        .unwrap_or("x")
                        .into()
                }
                _ => "x".into(),
            };
        } else {
            self.function_draft = "x".into();
        }
    }

    pub(super) fn apply_function_expression(
        &mut self,
        expression: SimpleFunction,
        plot: bool,
        edit_id: Option<usize>,
        radians: Option<bool>,
    ) -> Result<usize, String> {
        if let Some(id) = edit_id {
            if let Some(radians) = radians {
                self.doc
                    .set_function_expression_with_angle_unit(id, expression, radians)?;
            } else {
                self.doc.set_function_expression(id, expression)?;
            }
            return Ok(id);
        }
        let sources = &self.function_draft_sources;
        match (sources.is_empty(), plot, radians) {
            (true, false, None) => self.doc.add_function_definition(expression),
            (true, false, Some(radians)) => self
                .doc
                .add_function_definition_with_angle_unit(expression, radians),
            (true, true, None) => self.doc.plot_new_function(expression, self.center),
            (true, true, Some(radians)) => {
                self.doc
                    .plot_new_function_with_angle_unit(expression, self.center, radians)
            }
            (false, false, None) => self
                .doc
                .add_referenced_function_definition(sources, expression),
            (false, false, Some(radians)) => self
                .doc
                .add_referenced_function_definition_with_angle_unit(sources, expression, radians),
            (false, true, None) => {
                self.doc
                    .plot_new_referenced_function(sources, expression, self.center)
            }
            (false, true, Some(radians)) => self.doc.plot_new_referenced_function_with_angle_unit(
                sources,
                expression,
                self.center,
                radians,
            ),
        }
    }

    pub(super) fn function_dialogs_ui(&mut self, root: &mut egui::Ui) {
        if let Some(plot) = self.function_dialog {
            let edit_id = self.function_edit_id;
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("new_function")).show(root.ctx(), |ui| {
                ui.heading(if edit_id.is_some() {
                    "Edit function"
                } else if plot {
                    "Plot new function"
                } else {
                    "New function"
                });
                ui.label("f(x) =");
                ui.text_edit_singleline(&mut self.function_draft);
                ui.label("Arithmetic in x; sin, cos, tan, asin, acos, atan, abs, sqrt, ln, log, sgn, round, trunc; constants pi/π and e");
                for (index, &source) in self.function_draft_sources.iter().enumerate() {
                    let object = &self.doc.objects[source - 1];
                    let alias = if self.doc.function_sources(source).is_some() {
                        format!("f{}(…)", index + 1)
                    } else {
                        format!("v{}", index + 1)
                    };
                    let caption = self.doc.function_display_text(source)
                        .or_else(|| self.doc.measurement_text(source))
                        .or_else(|| self.doc.numeric_value(source).map(|value| format!("{} = {value}", self.doc.parameter_display_name(source).unwrap_or_else(|| format!("Object {source}")))))
                        .unwrap_or_else(|| format!("Object {}", object.id));
                    ui.label(format!("{alias}: {caption}"));
                }
                ui.horizontal(|ui| {
                    apply = ui
                        .button(if edit_id.is_some() {
                            "Update"
                        } else if plot {
                            "Plot"
                        } else {
                            "Create"
                        })
                        .clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.function_dialog = None;
                self.function_edit_id = None;
            }
            if apply {
                let result = SimpleFunction::parse_with_sources(
                    &self.function_draft,
                    self.function_draft_sources.len(),
                )
                .and_then(|expr| {
                    if expr.uses_trigonometry() && !self.doc.angle_unit_radians {
                        self.function_dialog = None;
                        self.function_angle_dialog = Some((expr, plot, edit_id));
                        return Ok(None);
                    }
                    self.apply_function_expression(expr, plot, edit_id, None)
                        .map(Some)
                });
                match result {
                    Ok(None) => {}
                    Ok(Some(id)) => {
                        self.function_dialog = None;
                        self.function_edit_id = None;
                        self.select_only(id);
                        self.status = if edit_id.is_some() {
                            format!("Updated function object {id}")
                        } else {
                            format!("Created function object {id}")
                        };
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if let Some((expression, plot, edit_id)) = self.function_angle_dialog {
            let mut choice = None;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("function_angle_unit")).show(root.ctx(), |ui| {
                ui.heading("Angle unit for this function");
                ui.label("The document currently uses degrees.");
                ui.horizontal(|ui| {
                    if ui.button("Switch to radians").clicked() {
                        choice = Some(true);
                    }
                    if ui.button("Keep degrees").clicked() {
                        choice = Some(false);
                    }
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.function_angle_dialog = None;
                self.function_edit_id = None;
            }
            if let Some(radians) = choice {
                let result =
                    self.apply_function_expression(expression, plot, edit_id, Some(radians));
                match result {
                    Ok(id) => {
                        self.function_angle_dialog = None;
                        self.function_edit_id = None;
                        self.select_only(id);
                        self.status = if edit_id.is_some() {
                            format!("Updated function object {id}")
                        } else {
                            format!("Created function object {id}")
                        };
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if let Some(source) = self.derivative_angle_dialog {
            let mut choice = None;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("derivative_angle_unit")).show(root.ctx(), |ui| {
                ui.heading("Angle unit for this derivative");
                ui.label("The document currently uses degrees.");
                ui.horizontal(|ui| {
                    if ui.button("Switch to radians").clicked() {
                        choice = Some(true);
                    }
                    if ui.button("Keep degrees").clicked() {
                        choice = Some(false);
                    }
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.derivative_angle_dialog = None;
            }
            if let Some(radians) = choice {
                match self
                    .doc
                    .add_derivative_function_with_angle_unit(source, radians)
                {
                    Ok(id) => {
                        self.derivative_angle_dialog = None;
                        self.select_only(id);
                        self.status = format!("Created derivative function {id}");
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if let Some(plot) = self.plot_range_dialog {
            let variable = if matches!(
                self.doc.objects.get(plot - 1).map(|o| &o.kind),
                Some(Kind::FunctionPlot(_, _, _, _, _))
            ) {
                "x"
            } else {
                "t"
            };
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("plot_range")).show(root.ctx(), |ui| {
                ui.heading("Plot properties");
                ui.horizontal(|ui| {
                    ui.label("Domain");
                    ui.add(egui::TextEdit::singleline(&mut self.plot_min_draft).desired_width(70.));
                    ui.label(format!("≤ {variable} ≤"));
                    ui.add(egui::TextEdit::singleline(&mut self.plot_max_draft).desired_width(70.));
                });
                ui.horizontal(|ui| {
                    ui.label("Samples");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.plot_samples_draft).desired_width(70.),
                    );
                });
                ui.horizontal(|ui| {
                    apply = ui.button("Apply").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.plot_range_dialog = None;
            }
            if apply {
                let result = (|| -> Result<(), String> {
                    let min = self
                        .plot_min_draft
                        .trim()
                        .parse::<f64>()
                        .map_err(|_| "Invalid domain start")?;
                    let max = self
                        .plot_max_draft
                        .trim()
                        .parse::<f64>()
                        .map_err(|_| "Invalid domain end")?;
                    let samples = self
                        .plot_samples_draft
                        .trim()
                        .parse::<u32>()
                        .map_err(|_| "Invalid sample count")?;
                    self.doc.set_plot_range(plot, min, max, samples)
                })();
                match result {
                    Ok(()) => {
                        self.plot_range_dialog = None;
                        self.select_only(plot);
                        self.status = format!("Updated plot {plot}");
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if self.parametric_dialog {
            let functions = self
                .doc
                .objects
                .iter()
                .filter_map(|object| {
                    if object.page != self.doc.active_page || object.deleted {
                        return None;
                    }
                    let text = self.doc.function_display_text(object.id)?;
                    Some((object.id, format!("{}: {text}", object.id)))
                })
                .collect::<Vec<_>>();
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("parametric_curve")).show(root.ctx(), |ui| {
                ui.heading("Plot parametric curve");
                ui.horizontal(|ui| {
                    ui.label("x(t) or r(t)");
                    egui::ComboBox::from_id_salt("parametric_x")
                        .selected_text(
                            functions
                                .iter()
                                .find(|(id, _)| *id == self.parametric_x)
                                .map_or("Select function", |(_, label)| label.as_str()),
                        )
                        .show_ui(ui, |ui| {
                            for (id, label) in &functions {
                                ui.selectable_value(&mut self.parametric_x, *id, label);
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.label("y(t) or θ(t)");
                    egui::ComboBox::from_id_salt("parametric_y")
                        .selected_text(
                            functions
                                .iter()
                                .find(|(id, _)| *id == self.parametric_y)
                                .map_or("Select function", |(_, label)| label.as_str()),
                        )
                        .show_ui(ui, |ui| {
                            for (id, label) in &functions {
                                ui.selectable_value(&mut self.parametric_y, *id, label);
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut self.parametric_mode,
                        PlotPointsMode::Cartesian,
                        "Rectangular",
                    );
                    ui.radio_value(&mut self.parametric_mode, PlotPointsMode::Polar, "Polar");
                });
                ui.horizontal(|ui| {
                    ui.label("Domain");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.parametric_min_draft)
                            .desired_width(65.),
                    );
                    ui.label("≤ t ≤");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.parametric_max_draft)
                            .desired_width(65.),
                    );
                });
                ui.horizontal(|ui| {
                    apply = ui.button("Plot").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.parametric_dialog = false;
            }
            if apply {
                let result = (|| -> Result<usize, String> {
                    let min = self
                        .parametric_min_draft
                        .trim()
                        .parse::<f64>()
                        .map_err(|_| "Invalid domain start")?;
                    let max = self
                        .parametric_max_draft
                        .trim()
                        .parse::<f64>()
                        .map_err(|_| "Invalid domain end")?;
                    self.doc.plot_parametric_curve(
                        self.parametric_x,
                        self.parametric_y,
                        self.parametric_mode,
                        min,
                        max,
                        self.center,
                    )
                })();
                match result {
                    Ok(id) => {
                        self.parametric_dialog = false;
                        self.select_only(id);
                        self.status = format!("Plotted parametric curve {id}");
                    }
                    Err(error) => self.status = error,
                }
            }
        }
    }
}
