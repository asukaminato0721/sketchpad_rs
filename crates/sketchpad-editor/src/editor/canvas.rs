use super::{Editor, Tool};
use crate::render::Viewport;
use sketchpad_engine::document::{Kind, Point};
use sketchpad_engine::text_records::display_label_name;

impl Editor {
    pub(super) fn canvas_ui(&mut self, root: &mut egui::Ui) -> egui::Rect {
        let mut rect = egui::Rect::NOTHING;
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(root, |ui| {
                let (r, response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
                rect = r;
                self.viewport_rect = Some(r);
                let vp = Viewport {
                    rect: r,
                    scale: self.scale,
                    center: self.center,
                };
                self.animation_bounds = Some((vp.world(r.min), vp.world(r.max)));
                if response.hovered() {
                    let dy = ui.input(|i| i.smooth_scroll_delta.y);
                    if dy != 0. {
                        self.scale = (self.scale * (dy * 0.001).exp()).clamp(0.05, 20.);
                    }
                }
                if response.drag_started()
                    && self.tool == Tool::Select
                    && let Some(p) = ui.input(|i| i.pointer.press_origin())
                    && let Some(id) = self.hit(&vp, p)
                    && (self.doc.objects[id - 1].measurement_anchor.is_some()
                        || matches!(
                            self.doc.objects[id - 1].kind,
                            Kind::Free(_)
                                | Kind::GraphVerticalUnit(_, _)
                                | Kind::Image { .. }
                                | Kind::PlainText { .. }
                                | Kind::ReferencedText { .. }
                                | Kind::Parameter(_)
                                | Kind::Calculation(_)
                                | Kind::MeasuredCalculation(_, _)
                                | Kind::LengthPairCalculation(_, _, _)
                                | Kind::ParameterCalculation(_, _)
                                | Kind::ValueTable(_, _)
                                | Kind::ValueTablePair(_, _, _)
                                | Kind::DataTable(_, _)
                                | Kind::PointOnSegment(_, _)
                                | Kind::PointOnRay(_, _)
                                | Kind::PointOnLine(_, _)
                                | Kind::PointOnCircle(_, _, _)
                                | Kind::PointOnArc(_, _)
                                | Kind::PointOnCurve(..)
                                | Kind::PointOnSector(..)
                        ))
                {
                    self.drag_offset = if let Some((x, y)) =
                        self.doc.objects[id - 1].measurement_anchor
                    {
                        p - (vp.rect.min + egui::vec2(f32::from(x), f32::from(y)))
                    } else {
                        match &self.doc.objects[id - 1].kind {
                            Kind::ReferencedText { text, .. } => {
                                let (x, y) = text.stored_origin();
                                p - vp.screen(Point {
                                    x: f64::from(x),
                                    y: f64::from(y),
                                })
                            }
                            Kind::PlainText { rect, .. } => {
                                p - vp.screen(Point {
                                    x: rect.left.into(),
                                    y: rect.top.into(),
                                })
                            }
                            Kind::Parameter(_)
                            | Kind::Calculation(_)
                            | Kind::MeasuredCalculation(_, _)
                            | Kind::LengthPairCalculation(_, _, _)
                            | Kind::ParameterCalculation(_, _)
                            | Kind::ValueTable(_, _)
                            | Kind::ValueTablePair(_, _, _)
                            | Kind::DataTable(_, _) => {
                                let anchor = crate::render::number_lines_for_object(&self.doc, id)
                                    .unwrap()
                                    .0;
                                p - (vp.rect.min + egui::vec2(anchor.0 as f32, anchor.1 as f32))
                            }
                            Kind::Image { .. } => self
                                .doc
                                .image_transform(id)
                                .map(|transform| {
                                    p - vp.screen(Point {
                                        x: transform[2],
                                        y: transform[5],
                                    })
                                })
                                .unwrap_or_default(),
                            _ => egui::Vec2::ZERO,
                        }
                    };
                    self.doc.checkpoint();
                    self.retain_animation_tracks(|track| track.id != id);
                    self.movements.retain(|track| track.source != id);
                    self.dragging = Some(id);
                    self.capture_traces();
                }
                if response.dragged()
                    && let (Some(id), Some(p)) = (self.dragging, response.interact_pointer_pos())
                {
                    if self.doc.objects[id - 1].measurement_anchor.is_some() {
                        let at = p - self.drag_offset - vp.rect.min;
                        if let Err(error) = self.doc.drag_measurement_caption(
                            id,
                            (at.x.round() as i16, at.y.round() as i16),
                        ) {
                            self.status = error;
                        }
                    } else if matches!(self.doc.objects[id - 1].kind, Kind::Image { .. }) {
                        if let Err(error) = self.doc.drag_image(id, vp.world(p - self.drag_offset))
                        {
                            self.status = error;
                        }
                    } else if matches!(
                        self.doc.objects[id - 1].kind,
                        Kind::PlainText { .. } | Kind::ReferencedText { .. }
                    ) {
                        if let Err(error) = self.doc.drag_text(id, vp.world(p - self.drag_offset)) {
                            self.status = error;
                        }
                    } else if matches!(
                        self.doc.objects[id - 1].kind,
                        Kind::Parameter(_)
                            | Kind::Calculation(_)
                            | Kind::MeasuredCalculation(_, _)
                            | Kind::LengthPairCalculation(_, _, _)
                            | Kind::ParameterCalculation(_, _)
                            | Kind::ValueTable(_, _)
                            | Kind::ValueTablePair(_, _, _)
                            | Kind::DataTable(_, _)
                    ) {
                        let at = p - self.drag_offset - vp.rect.min;
                        let anchor = (at.x.round() as i16, at.y.round() as i16);
                        if let Err(error) = self.doc.move_number(id, anchor) {
                            self.status = error;
                        }
                    } else {
                        self.doc.move_point(id, vp.world(p));
                    }
                    self.capture_traces();
                }
                if response.drag_stopped() {
                    self.dragging = None;
                }
                if response.clicked()
                    && let Some(p) = response.interact_pointer_pos()
                {
                    let multi = ui.input(|i| i.modifiers.shift);
                    self.click(&vp, p, multi);
                }
                if response.double_clicked()
                    && let Some(p) = response.interact_pointer_pos()
                {
                    if let Some(id) = self.hit(&vp, p) {
                        match self.doc.objects[id - 1].kind {
                            Kind::FunctionDefinition(_)
                            | Kind::ReferencedFunction(_, _)
                            | Kind::Calculation(_) => {
                                if let Err(error) = self.begin_edit_definition(id) {
                                    self.status = error;
                                }
                            }
                            Kind::FunctionPlot(..)
                            | Kind::ParametricPlot(..)
                            | Kind::ParameterLocus(..) => {
                                if let Some((min, max, samples)) = self.doc.plot_range(id) {
                                    self.plot_range_dialog = Some(id);
                                    self.plot_min_draft = min.to_string();
                                    self.plot_max_draft = max.to_string();
                                    self.plot_samples_draft = samples.to_string();
                                }
                            }
                            _ => {}
                        }
                    } else if self.tool == Tool::Select && self.preferences.double_click_deselection
                    {
                        self.doc.selected.clear();
                        self.selection_order.clear();
                    }
                }
                for object in self
                    .doc
                    .objects
                    .iter()
                    .filter(|object| self.doc.is_displayed(object.id))
                {
                    if let Some((source, png, transform, polygon)) =
                        self.doc.cropped_image(object.id)
                    {
                        if !self.image_textures.contains_key(&source)
                            && let Ok(decoded) =
                                image::load_from_memory_with_format(png, image::ImageFormat::Png)
                        {
                            let rgba = decoded.to_rgba8();
                            let color = egui::ColorImage::from_rgba_unmultiplied(
                                [rgba.width() as usize, rgba.height() as usize],
                                rgba.as_raw(),
                            );
                            self.image_textures.insert(
                                source,
                                ui.ctx().load_texture(
                                    format!("picture-{source}"),
                                    color,
                                    egui::TextureOptions::LINEAR,
                                ),
                            );
                        }
                        if let Some(texture) = self.image_textures.get(&source)
                            && let Some(vertices) = crate::render::cropped_picture_vertices(
                                &vp, png, transform, &polygon,
                            )
                        {
                            let points =
                                vertices.iter().map(|(point, _)| *point).collect::<Vec<_>>();
                            let mut mesh = egui::Mesh::with_texture(texture.id());
                            for (position, uv) in vertices {
                                mesh.vertices.push(egui::epaint::Vertex {
                                    pos: position,
                                    uv,
                                    color: egui::Color32::WHITE,
                                });
                            }
                            for triangle in crate::render::polygon_triangles(&points) {
                                mesh.indices.extend(triangle.map(|index| index as u32));
                            }
                            ui.painter().add(egui::Shape::mesh(mesh));
                        }
                    }
                    if let Some((source, png, transforms)) = self.doc.image_instances(object.id) {
                        if !self.image_textures.contains_key(&source)
                            && let Ok(decoded) =
                                image::load_from_memory_with_format(png, image::ImageFormat::Png)
                        {
                            let rgba = decoded.to_rgba8();
                            let color = egui::ColorImage::from_rgba_unmultiplied(
                                [rgba.width() as usize, rgba.height() as usize],
                                rgba.as_raw(),
                            );
                            let texture = ui.ctx().load_texture(
                                format!("picture-{source}"),
                                color,
                                egui::TextureOptions::LINEAR,
                            );
                            self.image_textures.insert(source, texture);
                        }
                        for transform in transforms {
                            if let Some(texture) = self.image_textures.get(&source)
                                && let Some(corners) =
                                    crate::render::picture_screen_quad(&vp, png, transform)
                            {
                                let mut mesh = egui::Mesh::with_texture(texture.id());
                                for (position, uv) in corners.into_iter().zip([
                                    egui::pos2(0., 0.),
                                    egui::pos2(1., 0.),
                                    egui::pos2(1., 1.),
                                    egui::pos2(0., 1.),
                                ]) {
                                    mesh.vertices.push(egui::epaint::Vertex {
                                        pos: position,
                                        uv,
                                        color: egui::Color32::WHITE,
                                    });
                                }
                                mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
                                ui.painter().add(egui::Shape::mesh(mesh));
                            }
                        }
                    }
                    if let Some((text, rect)) = self.doc.text_content(object.id) {
                        let at = vp.screen(Point {
                            x: rect.left.into(),
                            y: rect.top.into(),
                        });
                        let style = self.doc.text_font_state(object.id);
                        let size = style.as_ref().map_or(16., |style| style.size_code as f32);
                        let font_id = self.canvas_font_id(
                            style.as_ref().map(|style| style.family.as_str()),
                            size,
                        );
                        let rows = self.doc.text_rich_runs(object.id);
                        let families = self.doc.text_font_families().unwrap_or_default();
                        let has_mixed_runs = rows.as_ref().is_some_and(|rows| {
                            rows.iter().flatten().any(|run| {
                                run.style != sketchpad_engine::text_object::RichRunStyle::default()
                            })
                        });
                        if let Some(layout) = crate::render::math_text::measured_layout(
                            &self.doc,
                            object.id,
                            ui.ctx(),
                        ) {
                            layout.paint(ui.painter(), at);
                        } else if has_mixed_runs {
                            let mut line_y = at.y;
                            for row in rows.unwrap().iter() {
                                let line_size = row
                                    .iter()
                                    .map(|run| run.style.size_code.map_or(size, f32::from))
                                    .fold(size, f32::max);
                                let mut x = at.x;
                                for run in row {
                                    let (run_size, baseline_shift) = run.style.script_metrics(
                                        run.style.size_code.map_or(size, f32::from),
                                    );
                                    let run_family = run
                                        .style
                                        .family_index
                                        .and_then(|index| families.get(usize::from(index)))
                                        .map(String::as_str)
                                        .or_else(|| {
                                            style.as_ref().map(|style| style.family.as_str())
                                        });
                                    let run_font_id = self.canvas_font_id(run_family, run_size);
                                    let bold = run.style.bold.unwrap_or_else(|| {
                                        style.as_ref().is_some_and(|style| style.bold)
                                    });
                                    let italic = run.style.italic.unwrap_or_else(|| {
                                        style.as_ref().is_some_and(|style| style.italic)
                                    });
                                    let underline = run.style.underline.unwrap_or_else(|| {
                                        style.as_ref().is_some_and(|style| style.underline)
                                    });
                                    let color = run
                                        .style
                                        .rgb_color()
                                        .map(|[red, green, blue]| {
                                            egui::Color32::from_rgb(red, green, blue)
                                        })
                                        .unwrap_or_else(|| egui::Color32::from_gray(35));
                                    let format = egui::text::TextFormat {
                                        font_id: run_font_id,
                                        color,
                                        italics: italic,
                                        underline: if underline {
                                            egui::Stroke::new(1., color)
                                        } else {
                                            egui::Stroke::NONE
                                        },
                                        ..Default::default()
                                    };
                                    let galley = ui.painter().layout_job(
                                        egui::text::LayoutJob::single_section(
                                            run.text.clone(),
                                            format,
                                        ),
                                    );
                                    let width = galley.rect.width();
                                    let position = egui::pos2(
                                        x,
                                        line_y + line_size - run_size - baseline_shift,
                                    );
                                    ui.painter().galley(position, galley.clone(), color);
                                    if bold {
                                        ui.painter().galley(
                                            position + egui::vec2(0.7, 0.),
                                            galley,
                                            color,
                                        );
                                    }
                                    x += width;
                                }
                                line_y += (line_size * 1.1).max(19.);
                            }
                        } else {
                            for (line, content) in text.split('\n').enumerate() {
                                let position = at + egui::vec2(0., line as f32 * 19.);
                                let bounds = ui.painter().text(
                                    position,
                                    egui::Align2::LEFT_TOP,
                                    content,
                                    font_id.clone(),
                                    egui::Color32::from_gray(35),
                                );
                                if style.as_ref().is_some_and(|style| style.bold) {
                                    ui.painter().text(
                                        position + egui::vec2(0.7, 0.),
                                        egui::Align2::LEFT_TOP,
                                        content,
                                        font_id.clone(),
                                        egui::Color32::from_gray(35),
                                    );
                                }
                                if style.as_ref().is_some_and(|style| style.underline) {
                                    ui.painter().line_segment(
                                        [
                                            egui::pos2(bounds.min.x, bounds.max.y - 1.),
                                            egui::pos2(bounds.max.x, bounds.max.y - 1.),
                                        ],
                                        egui::Stroke::new(1., egui::Color32::from_gray(35)),
                                    );
                                }
                            }
                        }
                    }
                    if let Some(labels) = self.doc.point_parameter_iteration_labels(object.id) {
                        for (point, text) in labels {
                            ui.painter().text(
                                vp.screen(point),
                                egui::Align2::CENTER_CENTER,
                                text,
                                egui::FontId::proportional(16.),
                                if self.doc.selected.contains(&object.id) {
                                    self.selection_color()
                                } else {
                                    egui::Color32::BLACK
                                },
                            );
                        }
                    }
                    if let Some(labels) = self.doc.point_math_labels(object.id) {
                        for (point, nodes) in labels {
                            let layout = crate::render::math_text::measured_nodes(
                                &self.doc,
                                object.id,
                                &nodes,
                                ui.ctx(),
                            );
                            let at =
                                vp.screen(point) - egui::vec2(layout.width, layout.height()) * 0.5;
                            layout.paint(ui.painter(), at);
                        }
                    } else {
                        for (point, text) in self
                            .doc
                            .point_numeric_label(object.id)
                            .map(|label| vec![label])
                            .or_else(|| self.doc.mapped_numeric_label_images(object.id))
                            .unwrap_or_default()
                        {
                            let style = self.doc.numeric_label_font_state(object.id);
                            let size = style.as_ref().map_or(16, |style| style.size_code);
                            let font_id = self.canvas_font_id(
                                style.as_ref().map(|style| style.family.as_str()),
                                size as f32,
                            );
                            ui.painter().text(
                                vp.screen(point),
                                egui::Align2::CENTER_CENTER,
                                text,
                                font_id,
                                if self.doc.selected.contains(&object.id) {
                                    self.selection_color()
                                } else {
                                    egui::Color32::BLACK
                                },
                            );
                        }
                    }
                    if let Some((_, lines)) =
                        crate::render::number_lines_for_object(&self.doc, object.id)
                        && let Some(number_rect) =
                            crate::render::number_screen_rect_for_object(&vp, &self.doc, object.id)
                    {
                        ui.painter()
                            .rect_filled(number_rect, 0., egui::Color32::WHITE);
                        ui.painter().rect_stroke(
                            number_rect,
                            0.,
                            egui::Stroke::new(1., egui::Color32::from_gray(70)),
                            egui::StrokeKind::Inside,
                        );
                        for (row, line) in lines.iter().enumerate() {
                            ui.painter().text(
                                number_rect.min + egui::vec2(5., 3. + row as f32 * 21.),
                                egui::Align2::LEFT_TOP,
                                line,
                                egui::FontId::proportional(16.),
                                egui::Color32::BLACK,
                            );
                        }
                    }
                    if let Some(label) = &object.label
                        && label.visible()
                        && let Ok(name) = label.name_utf8()
                        && let Some(point) = self.doc.object_label_anchor(object.id)
                    {
                        let at = vp.screen(point);
                        let offset = label.offset();
                        let position = at + egui::vec2(offset.0 as f32, offset.1 as f32);
                        let alignment = if object.angle_marker_style.is_some() {
                            egui::Align2::CENTER_TOP
                        } else {
                            egui::Align2::LEFT_TOP
                        };
                        let style = self.doc.text_font_state(object.id);
                        let size = style.as_ref().map_or(13., |style| style.size_code as f32);
                        let font_id = self.canvas_font_id(
                            style.as_ref().map(|style| style.family.as_str()),
                            size,
                        );
                        let name = display_label_name(name);
                        let bounds = ui.painter().text(
                            position,
                            alignment,
                            name.as_ref(),
                            font_id.clone(),
                            egui::Color32::from_gray(35),
                        );
                        if style.as_ref().is_some_and(|style| style.bold) {
                            ui.painter().text(
                                position + egui::vec2(0.7, 0.),
                                alignment,
                                name.as_ref(),
                                font_id.clone(),
                                egui::Color32::from_gray(35),
                            );
                        }
                        if style.as_ref().is_some_and(|style| style.underline) {
                            ui.painter().line_segment(
                                [
                                    egui::pos2(bounds.min.x, bounds.max.y - 1.),
                                    egui::pos2(bounds.max.x, bounds.max.y - 1.),
                                ],
                                egui::Stroke::new(1., egui::Color32::from_gray(35)),
                            );
                        }
                    }
                    if let Kind::HideShowButton { label, anchor, .. }
                    | Kind::AnimationButton { label, anchor, .. }
                    | Kind::MovementButton { label, anchor, .. }
                    | Kind::ScrollButton { label, anchor, .. }
                    | Kind::PresentationButton { label, anchor, .. }
                    | Kind::LinkUrlButton { label, anchor, .. }
                    | Kind::LinkPageButton { label, anchor, .. } = &object.kind
                    {
                        let box_rect = self.action_button_rect(&vp, *anchor, label);
                        ui.painter().rect_filled(
                            box_rect,
                            3.,
                            egui::Color32::from_rgb(231, 233, 237),
                        );
                        ui.painter().rect_stroke(
                            box_rect,
                            3.,
                            egui::Stroke::new(1., egui::Color32::from_rgb(94, 102, 114)),
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            box_rect.min + egui::vec2(8., 5.),
                            egui::Align2::LEFT_TOP,
                            label,
                            egui::FontId::proportional(14.),
                            egui::Color32::from_rgb(32, 36, 42),
                        );
                    }
                }
                for o in self
                    .doc
                    .objects
                    .iter()
                    .filter(|o| self.doc.is_displayed(o.id))
                {
                    if o.label.is_none()
                        && let Some(p) = self.doc.position(o.id)
                    {
                        let at = vp.screen(p);
                        if r.contains(at) {
                            ui.painter().text(
                                at + egui::vec2(7., -9.),
                                egui::Align2::LEFT_TOP,
                                format!("{}", o.id),
                                egui::FontId::proportional(11.),
                                egui::Color32::from_gray(90),
                            );
                        }
                    }
                }
                for (at, label) in crate::render::trigonometric_axis_labels(&self.doc, &vp) {
                    ui.painter().text(
                        at,
                        egui::Align2::CENTER_TOP,
                        label,
                        egui::FontId::proportional(11.),
                        egui::Color32::from_rgb(36, 48, 61),
                    );
                }
                for (at, label, vertical) in crate::render::defined_axis_labels(&self.doc, &vp) {
                    ui.painter().text(
                        at,
                        if vertical {
                            egui::Align2::RIGHT_CENTER
                        } else {
                            egui::Align2::CENTER_TOP
                        },
                        label,
                        egui::FontId::proportional(11.),
                        egui::Color32::from_rgb(36, 48, 61),
                    );
                }
                for (row, (id, text)) in self.overlay_rows().into_iter().enumerate() {
                    let rect = self.overlay_row_rect(&vp, row, id, &text);
                    if self.doc.selected.contains(&id) {
                        let selected = self.selection_color();
                        ui.painter().rect_filled(
                            rect,
                            0.,
                            egui::Color32::from_rgba_unmultiplied(
                                selected.r(),
                                selected.g(),
                                selected.b(),
                                80,
                            ),
                        );
                    }
                    let style = self.doc.text_font_state(id);
                    let size = style
                        .as_ref()
                        .map_or(16., |style| f32::from(style.size_code));
                    let font_id = self
                        .canvas_font_id(style.as_ref().map(|style| style.family.as_str()), size);
                    let color = egui::Color32::from_gray(35);
                    let format = egui::text::TextFormat {
                        font_id,
                        color,
                        italics: style.as_ref().is_some_and(|style| style.italic),
                        underline: if style.as_ref().is_some_and(|style| style.underline) {
                            egui::Stroke::new(1., color)
                        } else {
                            egui::Stroke::NONE
                        },
                        ..Default::default()
                    };
                    let galley = ui
                        .painter()
                        .layout_job(egui::text::LayoutJob::single_section(text, format));
                    ui.painter().galley(rect.min, galley.clone(), color);
                    if style.as_ref().is_some_and(|style| style.bold) {
                        ui.painter()
                            .galley(rect.min + egui::vec2(0.7, 0.), galley, color);
                    }
                }
                if matches!(self.tool, Tool::Custom(_)) {
                    let start = self.custom_tool_path_start.and_then(|start| match start {
                        super::custom_tools::PathStart::Existing(id) => self.doc.position(id),
                        super::custom_tools::PathStart::New(point) => Some(point),
                    });
                    if let Some(start) = start {
                        ui.painter().circle_stroke(
                            vp.screen(start),
                            5.,
                            egui::Stroke::new(1.5, egui::Color32::from_rgb(180, 40, 100)),
                        );
                    }
                }
            });
        rect
    }
}
