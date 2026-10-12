use super::i18n::tr;
use super::{Editor, ParameterAnimationSettings};
use sketchpad_engine::document::Kind;

impl Editor {
    pub(super) fn begin_parameter_dialog(&mut self, id: usize) -> Result<(), String> {
        let Some(Kind::Parameter(group)) = self
            .doc
            .objects()
            .get(id.checked_sub(1).ok_or("Invalid parameter ID")?)
            .filter(|object| !object.deleted && object.page == self.doc.active_page)
            .map(|object| &object.kind)
        else {
            return Err("Select a parameter on the active page".into());
        };
        let settings = group.animation_settings();
        self.dialogs.parameter_value_draft = group.value().to_string();
        self.dialogs.parameter_precision_draft = group.precision();
        self.dialogs.parameter_display_mode_draft = group.display_mode();
        self.dialogs.parameter_original_display_mode = group.display_mode() == 0;
        self.dialogs.parameter_edit_box_draft = group.includes_edit_box();
        self.dialogs.parameter_discrete_draft = settings.discrete;
        self.dialogs.parameter_rate_units_draft = settings.rate_units.to_string();
        self.dialogs.parameter_rate_seconds_draft = settings.rate_seconds.to_string();
        self.dialogs.parameter_min_draft = settings.domain.0.to_string();
        self.dialogs.parameter_max_draft = settings.domain.1.to_string();
        self.dialogs.parameter_keyboard_step_draft = settings.keyboard_step.to_string();
        self.dialogs.parameter_value_dialog = Some(id);
        self.dialogs.numeric_dialog_error = None;
        Ok(())
    }

    pub(super) fn begin_edit_definition(&mut self, id: usize) -> Result<(), String> {
        let object = self
            .doc
            .objects()
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
        if matches!(object.kind, Kind::Parameter(_)) {
            return self.begin_parameter_dialog(id);
        }
        if matches!(
            object.kind,
            Kind::Calculation(_)
                | Kind::ParameterCalculation(..)
                | Kind::MeasuredCalculation(..)
                | Kind::LengthPairCalculation(..)
        ) {
            self.dialogs.calculation_draft = self.doc.calculation_expression(id)?;
            self.dialogs.calculation_edit_id = Some(id);
            self.dialogs.calculation_dialog = true;
            self.dialogs.numeric_dialog_error = None;
            return Ok(());
        }
        Err("Select one supported calculation, function, or numeric parameter".into())
    }

    pub(super) fn calculation_dialogs_ui(&mut self, root: &mut egui::Ui) {
        if self.dialogs.calculation_dialog {
            let edit_id = self.dialogs.calculation_edit_id;
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("calculation_expression")).show(root.ctx(), |ui| {
                ui.heading(tr(if edit_id.is_some() {
                    "Edit calculation"
                } else {
                    "Calculate"
                }));
                ui.text_edit_singleline(&mut self.dialogs.calculation_draft);
                if let Some(id) = edit_id {
                    ui.label(tr("Use v1, v2, … for saved numeric sources and f1, f2, … for saved functions"));
                    for (index, source) in self.doc.objects()[id - 1].kind.dependencies().iter().enumerate() {
                        let prefix = if self.doc.objects()[*source - 1].kind.is_function() { "f" } else { "v" };
                        ui.label(format!("{prefix}{} → #{source}", index + 1));
                    }
                } else {
                    ui.label(tr(
                    if self.doc.selected.iter().any(|&id| self.doc.objects().get(id - 1).is_some_and(|object| matches!(object.kind, Kind::FunctionDefinition(_) | Kind::ReferencedFunction(_, _) | Kind::DerivativeFunction(_, _)))) {
                        "Use f1(…), f2(…), … for selected functions and v1, v2, … for values, in selection order"
                    } else if self.doc.selected.len() >= 2 {
                        "Use v1, v2, … in selection order; a and b also name the first two values"
                    } else if self.doc.selected.len() == 1
                        && self.doc.selected.iter().next().is_some_and(|&id| {
                            self.doc
                                .objects()
                                .get(id - 1)
                                .is_some_and(|object| object.kind.is_distance_value())
                        })
                    {
                        "Selected length: × factor, ÷ divisor, or an expression using v"
                    } else if self.doc.selected.len() == 1 && self.doc.selected.iter().next().is_some_and(|&id| self.doc.objects().get(id - 1).is_some_and(|object| object.kind.numeric_unit().is_some())) {
                        "Use a parameter by name, or a selected numeric value as v; degree literals use °"
                    } else {
                        "Enter numbers, arithmetic, functions, pi/π, e, and cm, degree, or radian literals"
                    }),
                    );
                }
                if self.doc.selected.iter().all(|&id| self.doc.objects().get(id - 1).is_some_and(|o| o.kind.numeric_unit().is_some())) {
                    ui.label(tr("Functions: sin, cos, tan (degrees), asin, acos, atan, abs, sqrt, ln, log, sgn, round, trunc; constants: pi, π, e"));
                }
                ui.horizontal(|ui| {
                    apply = ui
                        .button(tr(if edit_id.is_some() { "Update" } else { "Create" }))
                        .clicked();
                    cancel = ui.button(tr("Cancel")).clicked();
                });
                if let Some(error) = &self.dialogs.numeric_dialog_error {
                    ui.colored_label(ui.visuals().error_fg_color, tr(error));
                }
            });
            if cancel {
                self.dialogs.calculation_dialog = false;
                self.dialogs.calculation_edit_id = None;
                self.dialogs.numeric_dialog_error = None;
            }
            if apply {
                if let Some(id) = edit_id {
                    match self
                        .doc
                        .set_calculation_expression(id, self.dialogs.calculation_draft.trim())
                    {
                        Ok(()) => {
                            self.dialogs.calculation_dialog = false;
                            self.dialogs.calculation_edit_id = None;
                            self.status = format!("Updated calculation {id}");
                            self.dialogs.numeric_dialog_error = None;
                        }
                        Err(error) => {
                            self.dialogs.numeric_dialog_error = Some(error.clone());
                            self.status = error;
                        }
                    }
                    return;
                }
                let (_, y) = self.doc.next_calculation_anchor();
                let selected_length = if self.doc.selected.len() == 1 {
                    self.doc.selected.iter().next().copied().filter(|&id| {
                        self.doc
                            .objects()
                            .get(id - 1)
                            .is_some_and(|object| object.kind.is_distance_value())
                    })
                } else {
                    None
                };
                let selected_parameter = if self.doc.selected.len() == 1 {
                    self.doc.selected.iter().next().copied().filter(|&id| {
                        self.doc
                            .objects()
                            .get(id - 1)
                            .is_some_and(|object| object.kind.numeric_unit().is_some())
                    })
                } else {
                    None
                };
                let draft = self.dialogs.calculation_draft.trim();
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
                            .all(|id| self.doc.objects()[*id - 1].kind.is_distance_value())
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
                        self.doc.objects().get(id - 1).is_some_and(|object| {
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
                        self.dialogs.calculation_dialog = false;
                        self.dialogs.calculation_edit_id = None;
                        self.status = format!("Created calculation {id}");
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if self.dialogs.iterate_dialog {
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("point_iteration")).show(root.ctx(), |ui| {
                ui.heading(tr("Iterate"));
                ui.label(tr("Select a free seed point and its translated, rotated, dilated, or reflected image. Add a scalar parameter or calculation for live point depth, or their segment or circle with an optional depth value. Rays also accept a scalar parameter or calculation for live depth."));
                ui.label(tr("For other carried shapes, select a seed point, its transformed image, and the circles, polygons, arcs, rays or lines to repeat. Derived seed points are supported. An optional scalar depth value updates the number of images live."));
                ui.label(tr("For numeric mappings, also select the parameter, calculation, and attached label, with an optional separate depth value. For a branching triangle, select its seed, two translated images, interior, optional sides, and optional depth parameter."));
                ui.horizontal(|ui| {
                    ui.label(tr("Depth"));
                    ui.text_edit_singleline(&mut self.dialogs.iteration_depth_draft);
                });
                ui.horizontal(|ui| {
                    apply = ui.button(tr("Create")).clicked();
                    cancel = ui.button(tr("Cancel")).clicked();
                });
            });
            if cancel {
                self.dialogs.iterate_dialog = false;
            }
            if apply {
                match self.dialogs.iteration_depth_draft.trim().parse::<u32>() {
                    Ok(depth) => {
                        self.iterate_selected(depth);
                        if self.status.starts_with("Created iteration ") {
                            self.dialogs.iterate_dialog = false;
                        }
                    }
                    Err(_) => self.status = "Enter an iteration depth from 1 to 128".into(),
                }
            }
        }
        if self.dialogs.named_parameter_dialog {
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("named_parameter")).show(root.ctx(), |ui| {
                ui.heading(tr("New named parameter"));
                ui.horizontal(|ui| {
                    ui.label(tr("Name"));
                    ui.text_edit_singleline(&mut self.dialogs.parameter_name_draft);
                });
                ui.horizontal(|ui| {
                    ui.label(tr("Value"));
                    ui.text_edit_singleline(&mut self.dialogs.parameter_value_draft);
                });
                ui.label(tr("Finite values, up to 32 native numeric tokens"));
                ui.horizontal(|ui| {
                    apply = ui.button(tr("Create")).clicked();
                    cancel = ui.button(tr("Cancel")).clicked();
                });
            });
            if cancel {
                self.dialogs.named_parameter_dialog = false;
            }
            if apply {
                let result = self
                    .dialogs
                    .parameter_value_draft
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| "Enter a numeric value".to_string())
                    .and_then(|value| {
                        let parameter_count = self
                            .doc
                            .objects()
                            .iter()
                            .filter(|object| matches!(object.kind, Kind::Parameter(_)))
                            .count();
                        let y = parameter_count
                            .checked_mul(33)
                            .and_then(|offset| offset.checked_add(5))
                            .and_then(|y| i16::try_from(y).ok())
                            .ok_or("Too many parameters")?;
                        self.doc.add_named_parameter(
                            self.dialogs.parameter_name_draft.trim(),
                            value,
                            (10, y),
                        )
                    });
                match result {
                    Ok(id) => {
                        self.dialogs.named_parameter_dialog = false;
                        self.status = format!("Created parameter {id}");
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if let Some(id) = self.dialogs.parameter_value_dialog {
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("parameter_value")).show(root.ctx(), |ui| {
                ui.heading(tr(format!("Edit parameter {id}")));
                ui.horizontal(|ui| {
                    ui.label(tr("Value"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.dialogs.parameter_value_draft)
                            .desired_width(90.),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(tr("Precision"));
                    egui::ComboBox::from_id_salt("parameter_precision")
                        .selected_text(tr(match self.dialogs.parameter_precision_draft {
                            0 => "units",
                            1 => "tenths",
                            2 => "hundredths",
                            3 => "thousandths",
                            4 => "ten-thousandths",
                            _ => "hundred-thousandths",
                        }))
                        .show_ui(ui, |ui| {
                            for (precision, name) in [
                                (0, "units"),
                                (1, "tenths"),
                                (2, "hundredths"),
                                (3, "thousandths"),
                                (4, "ten-thousandths"),
                                (5, "hundred-thousandths"),
                            ] {
                                ui.selectable_value(
                                    &mut self.dialogs.parameter_precision_draft,
                                    precision,
                                    name,
                                );
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.label(tr("Display"));
                    egui::ComboBox::from_id_salt("parameter_display_mode")
                        .selected_text(tr(match self.dialogs.parameter_display_mode_draft {
                            0 => "Original definition",
                            1 => "Value only",
                            _ => "Current name",
                        }))
                        .show_ui(ui, |ui| {
                            if self.dialogs.parameter_original_display_mode {
                                ui.selectable_value(
                                    &mut self.dialogs.parameter_display_mode_draft,
                                    0,
                                    tr("Original definition"),
                                );
                            }
                            ui.selectable_value(
                                &mut self.dialogs.parameter_display_mode_draft,
                                1,
                                tr("Value only"),
                            );
                            ui.selectable_value(
                                &mut self.dialogs.parameter_display_mode_draft,
                                3,
                                tr("Current name"),
                            );
                        });
                });
                ui.checkbox(
                    &mut self.dialogs.parameter_edit_box_draft,
                    tr("Show edit box (double-click to edit)"),
                );
                ui.horizontal(|ui| {
                    ui.label(tr("Change"));
                    ui.radio_value(
                        &mut self.dialogs.parameter_discrete_draft,
                        false,
                        tr("Continuously"),
                    );
                    ui.radio_value(
                        &mut self.dialogs.parameter_discrete_draft,
                        true,
                        tr("Discretely"),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(tr("By"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.dialogs.parameter_rate_units_draft)
                            .desired_width(70.),
                    );
                    ui.label(tr("units per"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.dialogs.parameter_rate_seconds_draft)
                            .desired_width(70.),
                    );
                    ui.label(tr("seconds"));
                });
                ui.horizontal(|ui| {
                    ui.label(tr("Animation domain"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.dialogs.parameter_min_draft)
                            .desired_width(70.),
                    );
                    ui.label(tr("to"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.dialogs.parameter_max_draft)
                            .desired_width(70.),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(tr("Keyboard (+/−) adjustment"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.dialogs.parameter_keyboard_step_draft)
                            .desired_width(70.),
                    );
                });
                ui.label(tr("The domain affects animation only"));
                if let Some(error) = &self.dialogs.numeric_dialog_error {
                    ui.colored_label(ui.visuals().error_fg_color, tr(error));
                }
                ui.horizontal(|ui| {
                    apply = ui.button(tr("Apply")).clicked();
                    cancel = ui.button(tr("Cancel")).clicked();
                });
            });
            if cancel {
                self.dialogs.parameter_value_dialog = None;
                self.dialogs.numeric_dialog_error = None;
            }
            if apply {
                let parse = |name: &str, draft: &str| {
                    draft
                        .trim()
                        .parse::<f64>()
                        .map_err(|_| format!("Enter a numeric {name}"))
                };
                let result = (|| {
                    let value = parse("value", &self.dialogs.parameter_value_draft)?;
                    let settings = ParameterAnimationSettings {
                        discrete: self.dialogs.parameter_discrete_draft,
                        rate_units: parse("rate", &self.dialogs.parameter_rate_units_draft)?,
                        rate_seconds: parse(
                            "rate interval",
                            &self.dialogs.parameter_rate_seconds_draft,
                        )?,
                        domain: (
                            parse("domain minimum", &self.dialogs.parameter_min_draft)?,
                            parse("domain maximum", &self.dialogs.parameter_max_draft)?,
                        ),
                        keyboard_step: parse(
                            "keyboard adjustment",
                            &self.dialogs.parameter_keyboard_step_draft,
                        )?,
                    };
                    self.doc.set_parameter_properties(
                        id,
                        value,
                        self.dialogs.parameter_precision_draft,
                        self.dialogs.parameter_display_mode_draft,
                        self.dialogs.parameter_edit_box_draft,
                        settings,
                    )
                })();
                match result {
                    Ok(()) => {
                        self.dialogs.parameter_value_dialog = None;
                        self.status = format!("Updated parameter {id}");
                        self.dialogs.numeric_dialog_error = None;
                    }
                    Err(error) => {
                        self.dialogs.numeric_dialog_error = Some(error.clone());
                        self.status = error;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, FullOutput, PointerButton, Pos2};
    use sketchpad_engine::document::Document;

    fn frame(editor: &mut Editor, ctx: &egui::Context, events: Vec<Event>) -> FullOutput {
        editor.language.activate();
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    egui::vec2(1100., 750.),
                )),
                events,
                ..Default::default()
            },
            |ui| editor.calculation_dialogs_ui(ui),
        );
        output.textures_delta.clear();
        output
    }

    fn click_text(editor: &mut Editor, ctx: &egui::Context, label: &str) {
        // Modal layout needs a frame before its stable position is available.
        frame(editor, ctx, vec![]);
        let output = frame(editor, ctx, vec![]);
        let pos = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) if text.galley.text() == label => {
                    Some(text.galley.rect.translate(text.pos.to_vec2()).center())
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("Missing dialog text: {label}"));
        for pressed in [true, false] {
            frame(
                editor,
                ctx,
                vec![
                    Event::PointerMoved(pos),
                    Event::PointerButton {
                        pos,
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
            );
        }
    }

    #[test]
    fn imported_binary_dialogs_apply_changes_and_show_errors() {
        let doc = Document::from_bytes(
            include_bytes!("../../../../tests/fixtures/decimal_to_binary_imported.gsp"),
            None,
        )
        .unwrap();
        let mut editor = Editor::new(doc);
        let ctx = egui::Context::default();
        editor.begin_edit_definition(1).unwrap();
        editor.dialogs.parameter_value_draft = "293".into();
        click_text(&mut editor, &ctx, "Apply");
        assert_eq!(
            editor.dialogs.parameter_value_dialog, None,
            "{}",
            editor.status
        );
        assert_eq!(editor.doc.numeric_value(1), Some(293.));
        // Keep a different selection to detect accidental rebinding to UI selection.
        editor.select_only(15);
        editor.begin_edit_definition(4).unwrap();
        assert!(editor.dialogs.calculation_draft.contains("v2"));
        editor.dialogs.calculation_draft = "v1-2*v2+1".into();
        click_text(&mut editor, &ctx, "Update");
        assert!(!editor.dialogs.calculation_dialog, "{}", editor.status);
        assert_eq!(editor.doc.numeric_value(4), Some(2.));
        editor.begin_edit_definition(4).unwrap();
        editor.dialogs.calculation_draft = "v3+1".into();
        click_text(&mut editor, &ctx, "Update");
        assert!(editor.dialogs.calculation_dialog);
        assert_eq!(editor.doc.numeric_value(4), Some(2.));
        let error = editor.dialogs.numeric_dialog_error.clone().unwrap();
        let output = frame(&mut editor, &ctx, vec![]);
        assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,
            egui::epaint::Shape::Text(text) if text.galley.text() == error)));
    }
}
