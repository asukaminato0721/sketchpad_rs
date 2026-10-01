use super::{Editor, action_dialogs::animation_settings_ui};
use sketchpad_engine::action_records::VisibilityAction;
use sketchpad_engine::document::{Document, Kind, LineMode, Object};
use sketchpad_engine::style_records::VisualStyle;

fn object_type_name(doc: &Document, id: usize, object: &Object) -> &'static str {
    if object.kind.is_point() {
        return "Point";
    }
    if let Some((_, _, mode)) = doc.line_points(id) {
        return match mode {
            LineMode::Segment => "Segment",
            LineMode::Ray => "Ray",
            LineMode::Line => "Line",
        };
    }
    if object.kind.is_circle() {
        return "Circle";
    }
    if object.kind.is_polygon() {
        return "Polygon";
    }
    match object.kind {
        Kind::ThreePointArc(..) | Kind::ArcOnCircle(..) | Kind::CenterArc(..) => "Arc",
        Kind::Image { .. }
        | Kind::CroppedImage { .. }
        | Kind::TransformedCroppedImage(..)
        | Kind::SampledImage(..) => "Picture",
        Kind::PlainText { .. } | Kind::PointReferencedText { .. } | Kind::ReferencedText { .. } => {
            "Caption"
        }
        Kind::Parameter(_) => "Parameter",
        Kind::Calculation(_)
        | Kind::MeasuredCalculation(..)
        | Kind::LengthPairCalculation(..)
        | Kind::ParameterCalculation(..) => "Calculation",
        Kind::FunctionDefinition(_)
        | Kind::ReferencedFunction(..)
        | Kind::DerivativeFunction(..) => "Function",
        Kind::FunctionPlot(..) | Kind::ParametricPlot(..) => "Plot",
        Kind::ValueTable(..)
        | Kind::ValueTablePair(..)
        | Kind::DataTable(..)
        | Kind::IterationValues(..) => "Table",
        Kind::AnimationButton { .. }
        | Kind::SoundButton { .. }
        | Kind::HideShowButton { .. }
        | Kind::MovementButton { .. }
        | Kind::ScrollButton { .. }
        | Kind::PresentationButton { .. }
        | Kind::LinkUrlButton { .. }
        | Kind::LinkPageButton { .. } => "Action Button",
        Kind::CoordinateSystem(..) => "Coordinate System",
        Kind::Locus(..) => "Locus",
        _ => "Object",
    }
}

fn object_description(doc: &Document, id: usize) -> String {
    let Some(object) = doc.objects.get(id.saturating_sub(1)) else {
        return format!("Object {id}");
    };
    let object_type = object_type_name(doc, id, object);
    let label = object
        .label
        .as_ref()
        .and_then(|label| label.name_utf8().ok())
        .filter(|name| !name.is_empty());
    match label {
        Some(label) => format!("{object_type} {label} (#{id})"),
        None => format!("{object_type} #{id}"),
    }
}

impl Editor {
    pub(super) fn properties_dialog_ui(&mut self, root: &mut egui::Ui) {
        let Some(id) = self.properties_dialog else {
            return;
        };
        let Some(object) = self
            .doc
            .objects
            .get(id.checked_sub(1).unwrap_or(usize::MAX))
        else {
            self.properties_dialog = None;
            return;
        };
        if object.deleted || object.page != self.doc.active_page {
            self.properties_dialog = None;
            return;
        }

        if self.properties_draft_for != Some(id) {
            self.properties_label_draft = match &object.kind {
                Kind::HideShowButton { label, .. } | Kind::AnimationButton { label, .. } => {
                    label.clone()
                }
                _ => object
                    .label
                    .as_ref()
                    .and_then(|label| label.name_utf8().ok())
                    .unwrap_or_default()
                    .to_owned(),
            };
            self.properties_label_visible_draft =
                object.label.as_ref().is_some_and(|label| label.visible());
            self.properties_opacity_draft = match object.visual_style {
                Some(VisualStyle::Polygon { opacity, .. }) => opacity,
                _ => 32767,
            };
            if let Kind::HideShowButton {
                action, options, ..
            } = &object.kind
            {
                self.properties_visibility_action_draft = *action;
                self.properties_visibility_options_draft = *options;
            }
            if let Kind::AnimationButton { settings, .. } = &object.kind {
                self.properties_animation_settings_draft = settings.clone();
                self.properties_animation_target_draft = 0;
            }
            self.properties_draft_for = Some(id);
        }

        let description = object_description(&self.doc, id);
        let saved_label = match &object.kind {
            Kind::HideShowButton { label, .. } if !label.is_empty() => Some(label.clone()),
            _ => object
                .label
                .as_ref()
                .and_then(|label| label.name_utf8().ok())
                .filter(|name| !name.is_empty())
                .map(str::to_owned),
        };
        let mut hidden = !object.visible;
        let mut arrow_selectable = object.arrow_selectable;
        let parents = self.doc.dependencies_of(id).unwrap_or_default();
        let children = self
            .doc
            .objects
            .iter()
            .filter(|candidate| {
                !candidate.deleted
                    && candidate.page == self.doc.active_page
                    && self
                        .doc
                        .dependencies_of(candidate.id)
                        .is_some_and(|parents| parents.contains(&id))
            })
            .map(|candidate| candidate.id)
            .collect::<Vec<_>>();
        let parent_labels = parents
            .iter()
            .map(|&parent| (parent, object_description(&self.doc, parent)))
            .collect::<Vec<_>>();
        let child_labels = children
            .iter()
            .map(|&child| (child, object_description(&self.doc, child)))
            .collect::<Vec<_>>();

        let mut open = true;
        let mut visibility_changed = false;
        let mut arrow_selectability_changed = false;
        let mut apply_label = false;
        let mut apply_opacity = false;
        let mut apply_hide_show_caption = false;
        let mut reset_hide_show_caption = false;
        let mut hide_show_settings_changed = false;
        let mut apply_animation_properties = false;
        let mut edit_parameter = false;
        let mut navigate_to = None;
        egui::Window::new("Properties")
            .id(egui::Id::new("object_properties"))
            .open(&mut open)
            .resizable(true)
            .show(root.ctx(), |ui| {
                ui.heading(&description);
                egui::Grid::new("object_properties_summary")
                    .num_columns(2)
                    .spacing([12., 6.])
                    .show(ui, |ui| {
                        ui.label("Object ID");
                        ui.monospace(id.to_string());
                        ui.end_row();
                        ui.label("Saved label");
                        ui.label(saved_label.as_deref().unwrap_or("None"));
                        ui.end_row();
                    });
                ui.separator();
                visibility_changed = ui.checkbox(&mut hidden, "Hidden").changed();
                arrow_selectability_changed = ui
                    .checkbox(&mut arrow_selectable, "Arrow selectable")
                    .changed();
                if matches!(object.kind, Kind::Parameter(_)) {
                    edit_parameter = ui.button("Parameter Settings…").clicked();
                }
                if self.doc.can_edit_geometry_label(id) {
                    ui.separator();
                    ui.heading("Label");
                    ui.horizontal(|ui| {
                        ui.label("Name");
                        ui.text_edit_singleline(&mut self.properties_label_draft);
                    });
                    ui.checkbox(&mut self.properties_label_visible_draft, "Show label");
                    apply_label = ui.button("Apply Label").clicked();
                }
                if matches!(object.visual_style, Some(VisualStyle::Polygon { .. })) {
                    ui.separator();
                    ui.heading("Opacity");
                    ui.add(
                        egui::Slider::new(&mut self.properties_opacity_draft, 0..=u16::MAX)
                            .custom_formatter(|value, _| format!("{:.0}%", value * 100. / 65535.)),
                    );
                    apply_opacity = ui.button("Apply Opacity").clicked();
                }
                if matches!(object.kind, Kind::HideShowButton { .. }) {
                    ui.separator();
                    ui.heading("Hide/Show Button");
                    let mut mode = if self.properties_visibility_options_draft.alternates() {
                        2
                    } else if self.properties_visibility_action_draft == VisibilityAction::Show {
                        0
                    } else {
                        1
                    };
                    let before = mode;
                    ui.radio_value(&mut mode, 0, "Always show objects");
                    ui.radio_value(&mut mode, 1, "Always hide objects");
                    ui.radio_value(&mut mode, 2, "Toggle hide/show");
                    if mode != before {
                        if mode < 2 {
                            self.properties_visibility_action_draft = if mode == 0 {
                                VisibilityAction::Show
                            } else {
                                VisibilityAction::Hide
                            };
                        }
                        self.properties_visibility_options_draft = self
                            .properties_visibility_options_draft
                            .with_alternates(mode == 2);
                        hide_show_settings_changed = true;
                    }
                    let mut fade = self.properties_visibility_options_draft.fade();
                    let mut select = self.properties_visibility_options_draft.select_after_show();
                    if ui.checkbox(&mut fade, "Use fade effect").changed() {
                        self.properties_visibility_options_draft =
                            self.properties_visibility_options_draft.with_fade(fade);
                        hide_show_settings_changed = true;
                    }
                    if ui
                        .checkbox(&mut select, "Select objects after showing")
                        .changed()
                    {
                        self.properties_visibility_options_draft = self
                            .properties_visibility_options_draft
                            .with_select_after_show(select);
                        hide_show_settings_changed = true;
                    }
                    ui.horizontal(|ui| {
                        ui.label("Caption");
                        ui.text_edit_singleline(&mut self.properties_label_draft);
                    });
                    ui.horizontal(|ui| {
                        apply_hide_show_caption = ui.button("Apply Caption").clicked();
                        reset_hide_show_caption = ui.button("Use Default Caption").clicked();
                    });
                }
                if let Kind::AnimationButton { targets, .. } = &object.kind {
                    ui.separator();
                    ui.heading("Animation Button");
                    if targets.len() > 1 {
                        ui.label("Choose a target to set its motion:");
                        for (index, target) in targets.iter().enumerate() {
                            ui.selectable_value(
                                &mut self.properties_animation_target_draft,
                                index,
                                format!("Object {target}"),
                            );
                        }
                        ui.separator();
                    }
                    let selected = self
                        .properties_animation_target_draft
                        .min(targets.len().saturating_sub(1));
                    if let (Some(&target), Some(settings)) = (
                        targets.get(selected),
                        self.properties_animation_settings_draft.get_mut(selected),
                    ) {
                        animation_settings_ui(&self.doc, ui, target, settings);
                    }
                    ui.horizontal(|ui| {
                        ui.label("Caption");
                        ui.text_edit_singleline(&mut self.properties_label_draft);
                    });
                    apply_animation_properties = ui.button("Apply Animation Settings").clicked();
                }
                ui.separator();
                ui.label("Parents");
                if parent_labels.is_empty() {
                    ui.weak("None");
                } else {
                    for (parent, label) in &parent_labels {
                        if ui.link(label).clicked() {
                            navigate_to = Some(*parent);
                        }
                    }
                }
                ui.separator();
                ui.label("Children");
                if child_labels.is_empty() {
                    ui.weak("None");
                } else {
                    for (child, label) in &child_labels {
                        if ui.link(label).clicked() {
                            navigate_to = Some(*child);
                        }
                    }
                }
            });

        if visibility_changed {
            match self.doc.set_object_visibility(id, !hidden) {
                Ok(true) => self.status = format!("Updated {description}"),
                Ok(false) => {}
                Err(error) => self.status = error,
            }
        }
        if arrow_selectability_changed {
            match self.doc.set_object_arrow_selectable(id, arrow_selectable) {
                Ok(true) => self.status = format!("Updated {description}"),
                Ok(false) => {}
                Err(error) => self.status = error,
            }
        }
        if apply_label {
            match self.doc.set_object_label(
                id,
                &self.properties_label_draft,
                self.properties_label_visible_draft,
            ) {
                Ok(true) => self.status = format!("Updated the label for {description}"),
                Ok(false) => {}
                Err(error) => self.status = error,
            }
        }
        if apply_opacity {
            match self
                .doc
                .set_object_polygon_opacity(id, self.properties_opacity_draft)
            {
                Ok(true) => self.status = format!("Updated the opacity for {description}"),
                Ok(false) => {}
                Err(error) => self.status = error,
            }
        }
        if hide_show_settings_changed {
            let previous_label = match &self.doc.objects[id - 1].kind {
                Kind::HideShowButton { label, .. } => label.clone(),
                _ => String::new(),
            };
            match self.doc.set_hide_show_button_action_and_options(
                id,
                self.properties_visibility_action_draft,
                self.properties_visibility_options_draft,
            ) {
                Ok(true) => {
                    self.status = format!("Updated {description}");
                    if self.properties_label_draft == previous_label
                        && let Kind::HideShowButton { label, .. } = &self.doc.objects[id - 1].kind
                    {
                        self.properties_label_draft = label.clone();
                    }
                }
                Ok(false) => {}
                Err(error) => self.status = error,
            }
        }
        if apply_hide_show_caption {
            match self
                .doc
                .set_hide_show_button_caption(id, &self.properties_label_draft)
            {
                Ok(()) => self.status = format!("Updated the caption for {description}"),
                Err(error) => self.status = error,
            }
        }
        if reset_hide_show_caption {
            match self.doc.reset_hide_show_button_caption(id) {
                Ok(()) => {
                    self.status = format!("Reset the caption for {description}");
                    if let Kind::HideShowButton { label, .. } = &self.doc.objects[id - 1].kind {
                        self.properties_label_draft = label.clone();
                    }
                }
                Err(error) => self.status = error,
            }
        }
        if apply_animation_properties {
            match self.doc.set_animation_button_properties(
                id,
                &self.properties_animation_settings_draft,
                &self.properties_label_draft,
            ) {
                Ok(true) => self.status = format!("Updated {description}"),
                Ok(false) => {}
                Err(error) => self.status = error,
            }
        }
        if edit_parameter {
            self.properties_dialog = None;
            self.properties_draft_for = None;
            if let Err(error) = self.begin_parameter_dialog(id) {
                self.status = error;
            }
            return;
        }
        if let Some(related) = navigate_to {
            self.select_only(related);
            self.properties_dialog = Some(related);
            self.properties_draft_for = None;
        } else if !open {
            self.properties_dialog = None;
            self.properties_draft_for = None;
        }
    }
}
