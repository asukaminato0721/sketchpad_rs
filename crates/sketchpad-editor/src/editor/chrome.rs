use super::{
    Editor, Tool,
    i18n::{Language, tr},
};

pub(super) fn apply_style(ctx: &egui::Context) {
    let mut style = egui::Style {
        visuals: egui::Visuals::light(),
        ..Default::default()
    };
    style.visuals.panel_fill = egui::Color32::from_gray(240);
    style.visuals.window_fill = egui::Color32::from_gray(248);
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(205, 224, 246);
    style.visuals.selection.stroke = egui::Stroke::new(1., egui::Color32::from_rgb(35, 65, 100));
    style.spacing.item_spacing = egui::vec2(6., 5.);
    style.spacing.button_padding = egui::vec2(6., 3.);
    ctx.set_theme(egui::Theme::Light);
    ctx.set_style_of(egui::Theme::Light, style);
}

impl Editor {
    pub(super) fn window_menu_ui(&mut self, ui: &mut egui::Ui) {
        ui.menu_button(tr("Window"), |ui| {
            ui.checkbox(&mut self.show_inspector, tr("Object Information"));
            ui.checkbox(&mut self.show_toolbox, tr("Toolbox"));
            ui.separator();
            ui.menu_button(tr("Language"), |ui| {
                for (language, name) in [
                    (Language::English, "English"),
                    (Language::Chinese, "简体中文"),
                ] {
                    if ui
                        .selectable_label(self.language == language, name)
                        .clicked()
                    {
                        self.language = language;
                        language.activate();
                        if let Err(error) = language.save() {
                            self.status = format!("Could not save language preference: {error}");
                        }
                        ui.ctx().request_repaint();
                        ui.close();
                    }
                }
            });
        });
        ui.menu_button(tr("Help"), |ui| {
            if ui.button(tr("About Sketchpad RS")).clicked() {
                self.show_about = true;
                ui.close();
            }
        });
    }

    pub(super) fn tool_strip_ui(&mut self, root: &mut egui::Ui) {
        if !self.show_toolbox {
            return;
        }
        egui::Panel::left("tool_strip")
            .resizable(false)
            .exact_size(44.)
            .show(root, |ui| {
                for (tool, name) in [
                    (Tool::Select, "Select"),
                    (Tool::Point, "Point"),
                    (Tool::Circle, "Circle"),
                    (Tool::Segment, "Segment"),
                    (Tool::Text, "Text"),
                ] {
                    let (rect, response) =
                        ui.allocate_exact_size(egui::vec2(30., 32.), egui::Sense::click());
                    let selected = self.tool == tool;
                    let visuals = ui.style().interact_selectable(&response, selected);
                    if selected || response.hovered() {
                        ui.painter().rect(
                            rect.shrink(1.),
                            2.,
                            visuals.bg_fill,
                            visuals.bg_stroke,
                            egui::StrokeKind::Inside,
                        );
                    }
                    let p = ui.painter();
                    let center = rect.center();
                    let stroke = egui::Stroke::new(1.5, egui::Color32::from_gray(45));
                    match tool {
                        Tool::Select => {
                            let a = center + egui::vec2(-6., -10.);
                            let b = center + egui::vec2(-5., 9.);
                            let c = center + egui::vec2(0., 3.);
                            let d = center + egui::vec2(8., 2.);
                            p.add(egui::Shape::convex_polygon(
                                vec![a, b, c, d],
                                stroke.color,
                                egui::Stroke::NONE,
                            ));
                        }
                        Tool::Point => {
                            p.circle_filled(center, 3., egui::Color32::from_rgb(45, 95, 180));
                        }
                        Tool::Circle => {
                            p.circle_stroke(center, 9., stroke);
                            p.circle_filled(center, 2., stroke.color);
                        }
                        Tool::Segment => {
                            let a = center + egui::vec2(-8., 8.);
                            let b = center + egui::vec2(8., -8.);
                            p.line_segment([a, b], stroke);
                            p.circle_filled(a, 2., stroke.color);
                            p.circle_filled(b, 2., stroke.color);
                        }
                        Tool::Text => {
                            p.text(
                                center,
                                egui::Align2::CENTER_CENTER,
                                "A",
                                egui::FontId::proportional(24.),
                                stroke.color,
                            );
                        }
                        Tool::Custom(_) => {}
                    }
                    if response.on_hover_text(tr(name)).clicked() {
                        self.tool = tool;
                        self.pending = None;
                        self.custom_tool_inputs.clear();
                        if tool == Tool::Text {
                            self.show_inspector = true;
                        }
                    }
                }
                ui.separator();
                ui.menu_button(">>", |ui| self.custom_tools_menu(ui))
                    .response
                    .on_hover_text(tr("Custom Tools"));
                if ui
                    .selectable_label(self.show_inspector, "i")
                    .on_hover_text(tr("Object Information"))
                    .clicked()
                {
                    self.show_inspector = !self.show_inspector;
                }
            });
    }

    pub(super) fn page_tabs_ui(&mut self, root: &mut egui::Ui) {
        if self.doc.page_count < 2 {
            return;
        }
        egui::Panel::bottom("page_tabs").show(root, |ui| {
            // Reserve space for the bar so it cannot cover short page tabs.
            ui.spacing_mut().scroll.floating = false;
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    for page in 0..self.doc.page_count {
                        let label = self
                            .doc
                            .page_names
                            .get(page)
                            .and_then(|name| name.clone())
                            .unwrap_or_else(|| tr(format!("Page {}", page + 1)));
                        if ui
                            .selectable_label(self.doc.active_page == page, label)
                            .clicked()
                        {
                            self.clear_animation_tracks();
                            self.movements.clear();
                            self.presentations.clear();
                            self.stop_all_sounds();
                            self.doc.active_page = page;
                            self.doc.selected.clear();
                            self.selection_order.clear();
                            self.pending = None;
                            self.custom_tool_inputs.clear();
                            self.fit();
                        }
                    }
                });
            });
        });
    }

    pub(super) fn about_ui(&mut self, root: &mut egui::Ui) {
        egui::Window::new(tr("About Sketchpad RS"))
            .id(egui::Id::new("about"))
            .open(&mut self.show_about)
            .resizable(false)
            .show(root.ctx(), |ui| {
                ui.heading("Sketchpad RS");
                ui.label(tr("A geometry editor for Sketchpad 5 documents."));
                ui.label(tr(
                    "Drag points. Scroll to zoom. Select two points to construct.",
                ));
                ui.label(tr(
                    "Use Window > Language to change the interface language.",
                ));
            });
    }
}
