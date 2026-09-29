use super::Editor;
use sketchpad_engine::document::Kind;

impl Editor {
    pub(super) fn begin_edit_definition(&mut self, id: usize) -> Result<(), String> {
        let object = self
            .doc
            .objects
            .get(id.checked_sub(1).ok_or("Invalid object ID")?)
            .filter(|object| !object.deleted && object.page == self.doc.active_page)
            .ok_or("Select one editable definition on the active page")?;
        if matches!(
            object.kind,
            Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _)
        ) {
            self.begin_function_dialog(false, Some(id));
            return Ok(());
        }
        if let Kind::Parameter(group) = &object.kind {
            self.parameter_value_draft = group.value().to_string();
            self.parameter_value_dialog = Some(id);
            return Ok(());
        }
        if let Kind::Calculation(group) = &object.kind {
            self.calculation_draft = group.expression();
            self.calculation_edit_id = Some(id);
            self.calculation_dialog = true;
            return Ok(());
        }
        Err("Select one supported calculation, function, or numeric parameter".into())
    }

    pub(super) fn calculation_dialogs_ui(&mut self, root: &mut egui::Ui) {
        if self.calculation_dialog {
            let edit_id = self.calculation_edit_id;
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("calculation_expression")).show(root.ctx(), |ui| {
                ui.heading(if edit_id.is_some() {
                    "Edit calculation"
                } else {
                    "Calculate"
                });
                ui.text_edit_singleline(&mut self.calculation_draft);
                ui.label(
                    if self.doc.selected.iter().any(|&id| self.doc.objects.get(id - 1).is_some_and(|object| matches!(object.kind, Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) | Kind::DerivativeFunction(_, _)))) {
                        "Use f1(…), f2(…), … for selected functions and v1, v2, … for values, in selection order"
                    } else if self.doc.selected.len() >= 2 {
                        "Use v1, v2, … in selection order; a and b also name the first two values"
                    } else if self.doc.selected.len() == 1
                        && self.doc.selected.iter().next().is_some_and(|&id| {
                            self.doc
                                .objects
                                .get(id - 1)
                                .is_some_and(|object| object.kind.is_distance_value())
                        })
                    {
                        "Selected length: × factor, ÷ divisor, or an expression using v"
                    } else if self.doc.selected.len() == 1 && self.doc.selected.iter().next().is_some_and(|&id| self.doc.objects.get(id - 1).is_some_and(|object| object.kind.numeric_unit().is_some())) {
                        "Use a parameter by name, or a selected numeric value as v; degree literals use °"
                    } else {
                        "Enter numbers, arithmetic, functions, pi/π, e, and cm, degree, or radian literals"
                    },
                );
                if self.doc.selected.iter().all(|&id| self.doc.objects.get(id - 1).is_some_and(|o| o.kind.numeric_unit().is_some())) {
                    ui.label("Functions: sin, cos, tan (degrees), asin, acos, atan, abs, sqrt, ln, log, sgn, round, trunc; constants: pi, π, e");
                }
                ui.horizontal(|ui| {
                    apply = ui
                        .button(if edit_id.is_some() { "Update" } else { "Create" })
                        .clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.calculation_dialog = false;
                self.calculation_edit_id = None;
            }
            if apply {
                if let Some(id) = edit_id {
                    match self
                        .doc
                        .set_calculation_expression(id, self.calculation_draft.trim())
                    {
                        Ok(()) => {
                            self.calculation_dialog = false;
                            self.calculation_edit_id = None;
                            self.status = format!("Updated calculation {id}");
                        }
                        Err(error) => self.status = error,
                    }
                    return;
                }
                let (_, y) = self.doc.next_calculation_anchor();
                let selected_length = if self.doc.selected.len() == 1 {
                    self.doc.selected.iter().next().copied().filter(|&id| {
                        self.doc
                            .objects
                            .get(id - 1)
                            .is_some_and(|object| object.kind.is_distance_value())
                    })
                } else {
                    None
                };
                let selected_parameter = if self.doc.selected.len() == 1 {
                    self.doc.selected.iter().next().copied().filter(|&id| {
                        self.doc
                            .objects
                            .get(id - 1)
                            .is_some_and(|object| object.kind.numeric_unit().is_some())
                    })
                } else {
                    None
                };
                let draft = self.calculation_draft.trim();
                let measured = draft
                    .strip_prefix(['*', '×', '·'])
                    .map(|value| (value, false))
                    .or_else(|| draft.strip_prefix(['/', '÷']).map(|value| (value, true)));
                let pair_ids = self
                    .selection_order
                    .iter()
                    .copied()
                    .filter(|id| self.doc.selected.contains(id))
                    .collect::<Vec<_>>();
                let result = if self.doc.selected.len() >= 2 && pair_ids.len() >= 2 {
                    let compact = draft
                        .chars()
                        .filter(|ch| !ch.is_whitespace())
                        .collect::<String>();
                    if pair_ids.len() == 2
                        && pair_ids
                            .iter()
                            .all(|id| self.doc.objects[*id - 1].kind.is_distance_value())
                    {
                        match compact.as_str() {
                            "a+b" => self.doc.add_length_pair_calculation(
                                pair_ids[0],
                                pair_ids[1],
                                false,
                                (10, y),
                            ),
                            "a-b" | "a−b" => self.doc.add_length_pair_calculation(
                                pair_ids[0],
                                pair_ids[1],
                                true,
                                (10, y),
                            ),
                            "a*b" | "a×b" | "a·b" => self.doc.add_length_pair_operation(
                                pair_ids[0],
                                pair_ids[1],
                                2,
                                (10, y),
                            ),
                            "a/b" | "a÷b" => self.doc.add_length_pair_operation(
                                pair_ids[0],
                                pair_ids[1],
                                3,
                                (10, y),
                            ),
                            _ => self.doc.add_multi_calculation(&pair_ids, draft, (10, y)),
                        }
                    } else {
                        self.doc.add_multi_calculation(&pair_ids, draft, (10, y))
                    }
                } else if let Some((source, (factor, divide))) = selected_length.zip(measured) {
                    factor
                        .trim()
                        .parse::<u8>()
                        .map_err(|_| "Enter an integer from 0 to 99".to_string())
                        .and_then(|factor| {
                            if divide {
                                self.doc
                                    .add_measured_length_division(source, factor, (10, y))
                            } else {
                                self.doc
                                    .add_measured_length_calculation(source, factor, (10, y))
                            }
                        })
                } else if let Some(source) = selected_parameter {
                    self.doc.add_parameter_calculation(source, draft, (10, y))
                } else if self.doc.selected.len() == 1
                    && self.doc.selected.iter().next().is_some_and(|&id| {
                        self.doc.objects.get(id - 1).is_some_and(|object| {
                            matches!(
                                object.kind,
                                Kind::FunctionDefinition(_)
                                    | Kind::ReferencedFunction(_, _)
                                    | Kind::DerivativeFunction(_, _)
                            )
                        })
                    })
                {
                    self.doc.add_multi_calculation(
                        &[*self.doc.selected.iter().next().unwrap()],
                        draft,
                        (10, y),
                    )
                } else {
                    self.doc.add_calculation_expression(draft, (10, y))
                };
                match result {
                    Ok(id) => {
                        self.calculation_dialog = false;
                        self.calculation_edit_id = None;
                        self.status = format!("Created calculation {id}");
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if self.iterate_dialog {
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("point_iteration")).show(root.ctx(), |ui| {
                ui.heading("Iterate");
                ui.label("Select a free seed point and its translated, rotated, dilated, or reflected image. Add a scalar parameter or calculation for live point depth, or their segment or circle with an optional depth value. Rays also accept a scalar parameter or calculation for live depth.");
                ui.label("For other carried shapes, select a seed point, its transformed image, and the circles, polygons, arcs, rays or lines to repeat. Derived seed points are supported. An optional scalar depth value updates the number of images live.");
                ui.label("For numeric mappings, also select the parameter, calculation, and attached label, with an optional separate depth value. For a branching triangle, select its seed, two translated images, interior, optional sides, and optional depth parameter.");
                ui.horizontal(|ui| {
                    ui.label("Depth");
                    ui.text_edit_singleline(&mut self.iteration_depth_draft);
                });
                ui.horizontal(|ui| {
                    apply = ui.button("Create").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.iterate_dialog = false;
            }
            if apply {
                match self.iteration_depth_draft.trim().parse::<u32>() {
                    Ok(depth) => {
                        self.iterate_selected(depth);
                        if self.status.starts_with("Created iteration ") {
                            self.iterate_dialog = false;
                        }
                    }
                    Err(_) => self.status = "Enter an iteration depth from 1 to 128".into(),
                }
            }
        }
        if self.named_parameter_dialog {
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("named_parameter")).show(root.ctx(), |ui| {
                ui.heading("New named parameter");
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut self.parameter_name_draft);
                });
                ui.horizontal(|ui| {
                    ui.label("Value");
                    ui.text_edit_singleline(&mut self.parameter_value_draft);
                });
                ui.label("Finite values, up to 32 native numeric tokens");
                ui.horizontal(|ui| {
                    apply = ui.button("Create").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.named_parameter_dialog = false;
            }
            if apply {
                let result = self
                    .parameter_value_draft
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| "Enter a numeric value".to_string())
                    .and_then(|value| {
                        let parameter_count = self
                            .doc
                            .objects
                            .iter()
                            .filter(|object| matches!(object.kind, Kind::Parameter(_)))
                            .count();
                        let y = parameter_count
                            .checked_mul(33)
                            .and_then(|offset| offset.checked_add(5))
                            .and_then(|y| i16::try_from(y).ok())
                            .ok_or("Too many parameters")?;
                        self.doc.add_named_parameter(
                            self.parameter_name_draft.trim(),
                            value,
                            (10, y),
                        )
                    });
                match result {
                    Ok(id) => {
                        self.named_parameter_dialog = false;
                        self.status = format!("Created parameter {id}");
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if let Some(id) = self.parameter_value_dialog {
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("parameter_value")).show(root.ctx(), |ui| {
                ui.heading(format!("Edit parameter {id}"));
                ui.horizontal(|ui| {
                    ui.label("Value");
                    ui.text_edit_singleline(&mut self.parameter_value_draft);
                });
                ui.label("Finite values, up to 32 native numeric tokens");
                ui.horizontal(|ui| {
                    apply = ui.button("Apply").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.parameter_value_dialog = None;
            }
            if apply {
                let result = self
                    .parameter_value_draft
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| "Enter a numeric value".to_string())
                    .and_then(|value| self.doc.set_parameter_value(id, value));
                match result {
                    Ok(()) => {
                        self.parameter_value_dialog = None;
                        self.status = format!("Updated parameter {id}");
                    }
                    Err(error) => self.status = error,
                }
            }
        }
    }
}
