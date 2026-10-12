use super::i18n::tr;
use super::{Editor, LinkButtonDraft};
use sketchpad_engine::action_records::{
    AnimationDirection, AnimationSettings, MovementSpeed, PresentationMode, PresentationOptions,
    PresentationStop, ScrollPlacement,
};
use sketchpad_engine::document::{Document, Kind};

pub(super) fn animation_settings_ui(
    doc: &Document,
    ui: &mut egui::Ui,
    target: usize,
    settings: &mut AnimationSettings,
) {
    let free = matches!(doc.objects()[target - 1].kind, Kind::Free(_));
    let parameter = matches!(doc.objects()[target - 1].kind, Kind::Parameter(_));
    let circle = matches!(doc.objects()[target - 1].kind, Kind::PointOnCircle(_, _, _));
    ui.label(tr(format!(
        "Animate {} {target}",
        if parameter { "parameter" } else { "point" }
    )));
    if free {
        ui.label(tr("Free points move at random"));
        settings.direction = AnimationDirection::Forward;
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
        egui::ComboBox::from_label(tr("Direction"))
            .selected_text(tr(direction_name))
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
                    ui.selectable_value(&mut settings.direction, direction, tr(name));
                }
            });
    }
    if !free && settings.direction == AnimationDirection::Bidirectional {
        settings.once = false;
    }
    ui.add_enabled(
        free || settings.direction != AnimationDirection::Bidirectional,
        egui::Checkbox::new(&mut settings.once, tr("Once Only")),
    );
    if parameter {
        ui.horizontal(|ui| {
            ui.label(tr("Change per second"));
            ui.add(egui::DragValue::new(&mut settings.speed).range(0.025..=2_500.));
        });
        if let Some(options) = settings.parameter.as_mut() {
            ui.checkbox(&mut options.discrete, tr("Discrete changes"));
            ui.horizontal(|ui| {
                ui.label(tr("Step"));
                ui.add_enabled(
                    options.discrete,
                    egui::DragValue::new(&mut options.step).range(0.001..=10_000.),
                );
            });
            ui.horizontal(|ui| {
                ui.label(tr("Domain"));
                ui.add(egui::DragValue::new(&mut options.min).speed(0.1));
                ui.label(tr("to"));
                ui.add(egui::DragValue::new(&mut options.max).speed(0.1));
            });
        }
    } else {
        ui.horizontal(|ui| {
            ui.label(tr("Speed"));
            for (speed, name) in [(4., "Slow"), (24., "Medium"), (80., "Fast")] {
                if ui
                    .selectable_label(settings.speed == speed, tr(name))
                    .clicked()
                {
                    settings.speed = speed;
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label(tr("Custom speed"));
            ui.add(egui::DragValue::new(&mut settings.speed).range(0.1..=10_000.));
        });
    }
}

pub(super) fn movement_settings_ui(
    ui: &mut egui::Ui,
    speed: &mut MovementSpeed,
    follow: &mut bool,
) {
    egui::ComboBox::from_label(tr("Speed"))
        .selected_text(tr(match speed {
            MovementSpeed::Slow => "Slow",
            MovementSpeed::Medium => "Medium",
            MovementSpeed::Fast => "Fast",
            MovementSpeed::Instant => "Instant",
        }))
        .show_ui(ui, |ui| {
            for (value, name) in [
                (MovementSpeed::Slow, "Slow"),
                (MovementSpeed::Medium, "Medium"),
                (MovementSpeed::Fast, "Fast"),
                (MovementSpeed::Instant, "Instant"),
            ] {
                ui.selectable_value(speed, value, tr(name));
            }
        });
    ui.radio_value(follow, true, tr("Follow moving destination"));
    ui.radio_value(follow, false, tr("Move toward initial destination"));
}

pub(super) fn scroll_settings_ui(
    ui: &mut egui::Ui,
    target: usize,
    placement: &mut ScrollPlacement,
) {
    ui.label(tr(format!("Position point {target} in the sketch window:")));
    ui.radio_value(placement, ScrollPlacement::UpperLeft, tr("Upper left"));
    ui.radio_value(placement, ScrollPlacement::Center, tr("Center"));
}

pub(super) fn presentation_settings_ui(
    ui: &mut egui::Ui,
    target_count: usize,
    mode: &mut PresentationMode,
    options: &mut PresentationOptions,
) {
    ui.label(tr(format!("Present {target_count} action button(s)")));
    let previous_mode = *mode;
    ui.radio_value(mode, PresentationMode::Simultaneous, tr("Simultaneously"));
    ui.radio_value(mode, PresentationMode::Sequential, tr("Sequentially"));
    if *mode != previous_mode {
        let before = (
            options.deselect,
            options.stop_animations,
            options.erase_traces,
            options.native_state,
        );
        *options = PresentationOptions::for_mode(*mode);
        options.deselect = before.0;
        options.stop_animations = before.1;
        options.erase_traces = before.2;
        options.native_state = before.3;
    }
    ui.separator();
    ui.label(tr("Before starting:"));
    ui.checkbox(&mut options.deselect, tr("Deselect objects"));
    ui.checkbox(&mut options.stop_animations, tr("Stop running animations"));
    ui.checkbox(&mut options.erase_traces, tr("Erase traces"));
    ui.separator();
    if *mode == PresentationMode::Simultaneous {
        ui.label(tr("Stop after:"));
        ui.radio_value(
            &mut options.stop,
            PresentationStop::LastAction,
            tr("Last action stops"),
        );
        ui.radio_value(
            &mut options.stop,
            PresentationStop::FirstAction,
            tr("First action stops"),
        );
        ui.radio_value(
            &mut options.stop,
            PresentationStop::ElapsedTime,
            tr("Elapsed time"),
        );
    } else {
        options.stop = PresentationStop::LastAction;
        ui.label(tr("Pause between actions:"));
    }
    if *mode == PresentationMode::Sequential || options.stop == PresentationStop::ElapsedTime {
        let mut seconds = f64::from(options.time_tenths) / 10.;
        if ui
            .add(
                egui::DragValue::new(&mut seconds)
                    .range(0.0..=6553.5)
                    .speed(0.1)
                    .suffix(" s"),
            )
            .changed()
        {
            options.time_tenths = (seconds * 10.).round() as u16;
        }
    }
}

pub(super) fn link_settings_ui(doc: &Document, ui: &mut egui::Ui, draft: &mut LinkButtonDraft) {
    if doc.page_count > 1 {
        ui.radio_value(&mut draft.use_page, true, tr("Page"));
        if draft.use_page {
            egui::ComboBox::from_id_salt("link_target_page")
                .selected_text(tr(format!("Page {}", draft.target_page + 1)))
                .show_ui(ui, |ui| {
                    for page in 0..doc.page_count {
                        if page != doc.active_page
                            && ui
                                .selectable_value(
                                    &mut draft.target_page,
                                    page,
                                    tr(format!("Page {}", page + 1)),
                                )
                                .clicked()
                        {
                            draft.target_button_ordinal = None;
                        }
                    }
                });
            let destination_buttons = doc.link_destination_buttons(draft.target_page);
            if draft.target_button_ordinal.is_some()
                || destination_buttons
                    .iter()
                    .any(|id| matches!(doc.objects()[id - 1].kind, Kind::HideShowButton { .. }))
            {
                egui::ComboBox::from_id_salt("link_target_button")
                    .selected_text(tr(match draft.target_button_ordinal {
                        Some(ordinal) => format!("Button {ordinal} on page"),
                        None => "No button on page".into(),
                    }))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut draft.target_button_ordinal,
                            None,
                            tr("No button on page"),
                        );
                        for (index, id) in destination_buttons.iter().enumerate() {
                            let label = match &doc.objects()[id - 1].kind {
                                Kind::HideShowButton { label, .. } => label.as_str(),
                                _ => continue,
                            };
                            ui.selectable_value(
                                &mut draft.target_button_ordinal,
                                Some(index + 1),
                                tr(label),
                            );
                        }
                    });
            }
        }
        ui.radio_value(&mut draft.use_page, false, tr("URL"));
    } else {
        draft.use_page = false;
    }
    if !draft.use_page {
        ui.label(tr("URL"));
        ui.text_edit_singleline(&mut draft.url);
    }
}

impl Editor {
    pub(super) fn action_dialogs_ui(&mut self, root: &mut egui::Ui) {
        if let Some(mut draft) = self.animation_button_dialog.clone() {
            let mut create = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("animation_button_settings")).show(root.ctx(), |ui| {
                ui.heading(tr("Animation Button"));
                if draft.targets.len() > 1 {
                    ui.label(tr("Choose a target to set its motion:"));
                    for (index, target) in draft.targets.iter().enumerate() {
                        ui.selectable_value(
                            &mut draft.selected,
                            index,
                            tr(format!("Object {target}")),
                        );
                    }
                    ui.separator();
                }
                let target = draft.targets[draft.selected];
                let settings = &mut draft.settings[draft.selected];
                animation_settings_ui(&self.doc, ui, target, settings);
                ui.horizontal(|ui| {
                    create = ui.button(tr("Create")).clicked();
                    cancel = ui.button(tr("Cancel")).clicked();
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
                ui.heading(tr("Movement Button"));
                for &(source, destination) in &draft.pairs {
                    ui.label(tr(format!(
                        "Move object {source} toward object {destination}"
                    )));
                }
                movement_settings_ui(ui, &mut draft.speed, &mut draft.follow);
                ui.horizontal(|ui| {
                    create = ui.button(tr("Create")).clicked();
                    cancel = ui.button(tr("Cancel")).clicked();
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
                ui.heading(tr("Scroll Button"));
                scroll_settings_ui(ui, draft.target, &mut draft.placement);
                ui.horizontal(|ui| {
                    create = ui.button(tr("Create")).clicked();
                    cancel = ui.button(tr("Cancel")).clicked();
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
                    ui.heading(tr("Presentation Button"));
                    presentation_settings_ui(
                        ui,
                        draft.targets.len(),
                        &mut draft.mode,
                        &mut draft.options,
                    );
                    ui.horizontal(|ui| {
                        create = ui.button(tr("Create")).clicked();
                        cancel = ui.button(tr("Cancel")).clicked();
                    });
                },
            );
            if cancel {
                self.presentation_button_dialog = None;
            } else if create {
                match self.doc.add_presentation_button_for_targets_with_options(
                    &draft.targets,
                    draft.mode,
                    draft.options,
                ) {
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
                ui.heading(tr("Link Button"));
                link_settings_ui(&self.doc, ui, &mut draft);
                ui.horizontal(|ui| {
                    create = ui.button(tr("Create")).clicked();
                    cancel = ui.button(tr("Cancel")).clicked();
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
