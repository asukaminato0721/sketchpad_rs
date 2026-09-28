use super::Editor;
use sketchpad_engine::action_records::{
    AnimationDirection, MovementSpeed, PresentationMode, ScrollPlacement,
};
use sketchpad_engine::document::Kind;

impl Editor {
    pub(super) fn action_dialogs_ui(&mut self, root: &mut egui::Ui) {
        if let Some(mut draft) = self.animation_button_dialog.clone() {
            let mut create = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("animation_button_settings")).show(root.ctx(), |ui| {
                ui.heading("Animation Button");
                if draft.targets.len() > 1 {
                    ui.label("Choose a target to set its motion:");
                    for (index, target) in draft.targets.iter().enumerate() {
                        ui.selectable_value(&mut draft.selected, index, format!("Object {target}"));
                    }
                    ui.separator();
                }
                let target = draft.targets[draft.selected];
                let settings = &mut draft.settings[draft.selected];
                let free = matches!(self.doc.objects[target - 1].kind, Kind::Free(_));
                let parameter = matches!(self.doc.objects[target - 1].kind, Kind::Parameter(_));
                let circle = matches!(
                    self.doc.objects[target - 1].kind,
                    Kind::PointOnCircle(_, _, _)
                );
                ui.label(format!(
                    "Animate {} {target}",
                    if parameter { "parameter" } else { "point" }
                ));
                if free {
                    ui.label("Free points move at random");
                } else {
                    let direction_name = match (circle, parameter, settings.direction) {
                        (true, _, AnimationDirection::Forward) => "Counterclockwise",
                        (true, _, AnimationDirection::Backward) => "Clockwise",
                        (_, true, AnimationDirection::Forward) => "Increasing",
                        (_, true, AnimationDirection::Backward) => "Decreasing",
                        (_, _, AnimationDirection::Forward) => "Forward",
                        (_, _, AnimationDirection::Backward) => "Backward",
                        (_, _, AnimationDirection::Bidirectional) => "Bidirectional",
                        (_, _, AnimationDirection::Random) => "Random",
                    };
                    egui::ComboBox::from_label("Direction")
                        .selected_text(direction_name)
                        .show_ui(ui, |ui| {
                            for (direction, name) in [
                                (
                                    AnimationDirection::Forward,
                                    if circle {
                                        "Counterclockwise"
                                    } else if parameter {
                                        "Increasing"
                                    } else {
                                        "Forward"
                                    },
                                ),
                                (
                                    AnimationDirection::Backward,
                                    if circle {
                                        "Clockwise"
                                    } else if parameter {
                                        "Decreasing"
                                    } else {
                                        "Backward"
                                    },
                                ),
                                (AnimationDirection::Bidirectional, "Bidirectional"),
                                (AnimationDirection::Random, "Random"),
                            ] {
                                ui.selectable_value(&mut settings.direction, direction, name);
                            }
                        });
                }
                if !free && settings.direction == AnimationDirection::Bidirectional {
                    settings.once = false;
                }
                ui.add_enabled(
                    free || settings.direction != AnimationDirection::Bidirectional,
                    egui::Checkbox::new(&mut settings.once, "Once Only"),
                );
                if parameter {
                    ui.horizontal(|ui| {
                        ui.label("Change per second");
                        ui.add(egui::DragValue::new(&mut settings.speed).range(0.025..=2_500.));
                    });
                    if let Some(options) = settings.parameter.as_mut() {
                        ui.checkbox(&mut options.discrete, "Discrete changes");
                        ui.horizontal(|ui| {
                            ui.label("Step");
                            ui.add_enabled(
                                options.discrete,
                                egui::DragValue::new(&mut options.step).range(0.001..=10_000.),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Domain");
                            ui.add(egui::DragValue::new(&mut options.min).speed(0.1));
                            ui.label("to");
                            ui.add(egui::DragValue::new(&mut options.max).speed(0.1));
                        });
                    }
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Speed");
                        for (speed, name) in [(4., "Slow"), (24., "Medium"), (80., "Fast")] {
                            if ui.selectable_label(settings.speed == speed, name).clicked() {
                                settings.speed = speed;
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Custom speed");
                        ui.add(egui::DragValue::new(&mut settings.speed).range(0.1..=10_000.));
                    });
                }
                ui.horizontal(|ui| {
                    create = ui.button("Create").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.animation_button_dialog = None;
            } else if create {
                let values = draft
                    .targets
                    .iter()
                    .copied()
                    .zip(draft.settings.iter().copied())
                    .collect::<Vec<_>>();
                match self
                    .doc
                    .add_animation_button_for_targets_with_settings_and_label(
                        &values,
                        draft.label.as_deref(),
                    ) {
                    Ok(id) => {
                        self.select_only(id);
                        self.animation_button_dialog = None;
                    }
                    Err(error) => {
                        self.status = error;
                        self.animation_button_dialog = Some(draft);
                    }
                }
            } else {
                self.animation_button_dialog = Some(draft);
            }
        }
        if let Some(mut draft) = self.movement_button_dialog.clone() {
            let mut create = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("movement_button_settings")).show(root.ctx(), |ui| {
                ui.heading("Movement Button");
                for &(source, destination) in &draft.pairs {
                    ui.label(format!("Move object {source} toward object {destination}"));
                }
                egui::ComboBox::from_label("Speed")
                    .selected_text(match draft.speed {
                        MovementSpeed::Slow => "Slow",
                        MovementSpeed::Medium => "Medium",
                        MovementSpeed::Fast => "Fast",
                        MovementSpeed::Instant => "Instant",
                    })
                    .show_ui(ui, |ui| {
                        for (speed, name) in [
                            (MovementSpeed::Slow, "Slow"),
                            (MovementSpeed::Medium, "Medium"),
                            (MovementSpeed::Fast, "Fast"),
                            (MovementSpeed::Instant, "Instant"),
                        ] {
                            ui.selectable_value(&mut draft.speed, speed, name);
                        }
                    });
                ui.radio_value(&mut draft.follow, true, "Follow moving destination");
                ui.radio_value(&mut draft.follow, false, "Move toward initial destination");
                ui.horizontal(|ui| {
                    create = ui.button("Create").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.movement_button_dialog = None;
            } else if create {
                match self.doc.add_movement_button_for_pairs(
                    &draft.pairs,
                    draft.speed,
                    draft.follow,
                ) {
                    Ok(id) => {
                        self.select_only(id);
                        self.movement_button_dialog = None;
                    }
                    Err(error) => self.status = error,
                }
            } else {
                self.movement_button_dialog = Some(draft);
            }
        }
        if let Some(mut draft) = self.scroll_button_dialog {
            let mut create = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("scroll_button_settings")).show(root.ctx(), |ui| {
                ui.heading("Scroll Button");
                ui.label(format!(
                    "Position point {} in the sketch window:",
                    draft.target
                ));
                ui.radio_value(
                    &mut draft.placement,
                    ScrollPlacement::UpperLeft,
                    "Upper left",
                );
                ui.radio_value(&mut draft.placement, ScrollPlacement::Center, "Center");
                ui.horizontal(|ui| {
                    create = ui.button("Create").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.scroll_button_dialog = None;
            } else if create {
                match self.doc.add_scroll_button(draft.target, draft.placement) {
                    Ok(id) => {
                        self.select_only(id);
                        self.scroll_button_dialog = None;
                    }
                    Err(error) => self.status = error,
                }
            } else {
                self.scroll_button_dialog = Some(draft);
            }
        }
        if let Some(mut draft) = self.presentation_button_dialog.clone() {
            let mut create = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("presentation_button_settings")).show(
                root.ctx(),
                |ui| {
                    ui.heading("Presentation Button");
                    ui.label(format!(
                        "Present {} selected action button(s)",
                        draft.targets.len()
                    ));
                    ui.radio_value(
                        &mut draft.mode,
                        PresentationMode::Simultaneous,
                        "Simultaneously",
                    );
                    ui.radio_value(
                        &mut draft.mode,
                        PresentationMode::Sequential,
                        "Sequentially",
                    );
                    ui.horizontal(|ui| {
                        create = ui.button("Create").clicked();
                        cancel = ui.button("Cancel").clicked();
                    });
                },
            );
            if cancel {
                self.presentation_button_dialog = None;
            } else if create {
                match self
                    .doc
                    .add_presentation_button_for_targets(&draft.targets, draft.mode)
                {
                    Ok(id) => {
                        self.select_only(id);
                        self.presentation_button_dialog = None;
                    }
                    Err(error) => self.status = error,
                }
            } else {
                self.presentation_button_dialog = Some(draft);
            }
        }
        if let Some(mut draft) = self.link_dialog.clone() {
            let mut create = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("link_button_settings")).show(root.ctx(), |ui| {
                ui.heading("Link Button");
                if self.doc.page_count > 1 {
                    ui.radio_value(&mut draft.use_page, true, "Page");
                    if draft.use_page {
                        egui::ComboBox::from_id_salt("link_target_page")
                            .selected_text(format!("Page {}", draft.target_page + 1))
                            .show_ui(ui, |ui| {
                                for page in 0..self.doc.page_count {
                                    if page != self.doc.active_page {
                                        if ui
                                            .selectable_value(
                                                &mut draft.target_page,
                                                page,
                                                format!("Page {}", page + 1),
                                            )
                                            .clicked()
                                        {
                                            draft.target_button_ordinal = None;
                                        }
                                    }
                                }
                            });
                        let destination_buttons =
                            self.doc.link_destination_buttons(draft.target_page);
                        if destination_buttons.iter().any(|id| {
                            matches!(self.doc.objects[id - 1].kind, Kind::HideShowButton { .. })
                        }) {
                            egui::ComboBox::from_id_salt("link_target_button")
                                .selected_text(match draft.target_button_ordinal {
                                    Some(ordinal) => format!("Button {ordinal} on page"),
                                    None => "No button on page".into(),
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut draft.target_button_ordinal,
                                        None,
                                        "No button on page",
                                    );
                                    for (index, id) in destination_buttons.iter().enumerate() {
                                        let label = match &self.doc.objects[id - 1].kind {
                                            Kind::HideShowButton { label, .. } => label.as_str(),
                                            _ => continue,
                                        };
                                        ui.selectable_value(
                                            &mut draft.target_button_ordinal,
                                            Some(index + 1),
                                            label,
                                        );
                                    }
                                });
                        }
                    }
                    ui.radio_value(&mut draft.use_page, false, "URL");
                }
                if !draft.use_page {
                    ui.label("URL");
                    ui.text_edit_singleline(&mut draft.url);
                }
                ui.horizontal(|ui| {
                    create = ui.button("Create").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.link_dialog = None;
            } else if create {
                let result = if draft.use_page {
                    self.doc.add_link_page_button_with_target(
                        draft.target_page,
                        draft.target_button_ordinal,
                    )
                } else {
                    self.doc.add_link_url_button(&draft.url)
                };
                match result {
                    Ok(id) => {
                        self.select_only(id);
                        self.link_dialog = None;
                    }
                    Err(error) => {
                        self.status = error;
                        self.link_dialog = Some(draft);
                    }
                }
            } else {
                self.link_dialog = Some(draft);
            }
        }
    }
}
