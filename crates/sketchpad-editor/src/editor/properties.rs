use super::Editor;
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
            self.properties_label_draft = object
                .label
                .as_ref()
                .and_then(|label| label.name_utf8().ok())
                .unwrap_or_default()
                .to_owned();
            self.properties_label_visible_draft =
                object.label.as_ref().is_some_and(|label| label.visible());
            self.properties_opacity_draft = match object.visual_style {
                Some(VisualStyle::Polygon { opacity, .. }) => opacity,
                _ => 32767,
            };
            self.properties_draft_for = Some(id);
        }

        let description = object_description(&self.doc, id);
        let saved_label = object
            .label
            .as_ref()
            .and_then(|label| label.name_utf8().ok())
            .filter(|name| !name.is_empty())
            .map(str::to_owned);
        let mut hidden = !object.visible;
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
        let mut apply_label = false;
        let mut apply_opacity = false;
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
