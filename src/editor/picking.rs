use super::Editor;
use sketchpad_rs::document::{Kind, LineMode, Point};
use sketchpad_rs::graph::{GraphPoint, GridForm};
use sketchpad_rs::render::Viewport;

impl Editor {
    pub(super) fn hit_point(&self, v: &Viewport, p: egui::Pos2) -> Option<usize> {
        self.doc
            .objects
            .iter()
            .rev()
            .filter(|o| self.doc.is_displayed(o.id))
            .filter_map(|o| {
                self.doc
                    .position(o.id)
                    .map(|at| (o.id, v.screen(at).distance(p)))
            })
            .filter(|(_, d)| *d < 11.)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|x| x.0)
    }
    pub(super) fn hit_grid(&self, v: &Viewport, p: egui::Pos2) -> Option<usize> {
        let world = v.world(p);
        self.doc
            .objects
            .iter()
            .rev()
            .filter(|object| {
                matches!(object.kind, Kind::CoordinateSystem(_, _))
                    && self.doc.is_displayed(object.id)
            })
            .filter_map(|object| {
                let frame = self.doc.graph_frame(object.id)?;
                let coordinates = frame.coordinates(GraphPoint {
                    x: world.x,
                    y: world.y,
                })?;
                let form = self
                    .doc
                    .frame_styles
                    .get(&object.id)
                    .map_or(self.doc.grid_form[object.page], |style| style.0);
                let (x, y) = if form == GridForm::Polar {
                    let radius = coordinates.x.hypot(coordinates.y).round();
                    let angle = (coordinates.y.atan2(coordinates.x)
                        / (std::f64::consts::TAU / 24.))
                        .round()
                        * std::f64::consts::TAU
                        / 24.;
                    (radius * angle.cos(), radius * angle.sin())
                } else {
                    (coordinates.x.round(), coordinates.y.round())
                };
                let at = frame.world(x, y)?;
                let distance = v.screen(Point { x: at.x, y: at.y }).distance(p);
                (distance < 7.).then_some((object.id, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id)
    }
    pub(super) fn overlay_rows(&self) -> Vec<(usize, String)> {
        let mut rows = Vec::new();
        for object in self.doc.objects.iter().filter(|object| {
            object.page == self.doc.active_page && object.visible && !object.deleted
        }) {
            let id = object.id;
            if let Some(text) = self.doc.measurement_text(id) {
                rows.push((id, text));
            }
            if matches!(object.kind, Kind::SlopeMeasurement(_, _)) {
                let value = self
                    .doc
                    .measurement_value_text(id)
                    .unwrap_or_else(|| "undefined".into());
                rows.push((id, format!("Slope = {value}")));
            }
            if let Some(value) = self.doc.coordinate_measure_text(id) {
                rows.push((id, value));
            }
            if let Some(value) = self.doc.value_equation_text(id) {
                rows.push((id, value));
            }
            if let Some(text) = self.doc.function_display_text(id) {
                rows.push((id, text));
            }
        }
        rows
    }
    pub(super) fn overlay_row_rect(
        &self,
        v: &Viewport,
        row: usize,
        id: usize,
        text: &str,
    ) -> egui::Rect {
        let at = self.doc.objects[id - 1]
            .measurement_anchor
            .or(self.doc.objects[id - 1].function_anchor)
            .map(|(x, y)| v.rect.min + egui::vec2(x as f32 + 2., y as f32 + 7.))
            .unwrap_or_else(|| v.rect.min + egui::vec2(12., 12. + row as f32 * 20.));
        let size = self
            .doc
            .text_font_state(id)
            .map_or(16., |style| f32::from(style.size_code));
        egui::Rect::from_min_size(
            at,
            egui::vec2(text.chars().count() as f32 * size * 0.6, size + 3.),
        )
    }
    pub(super) fn button_offset(&self, v: &Viewport) -> egui::Vec2 {
        egui::vec2(
            ((self.button_view_center.x - v.center.x) * f64::from(v.scale)) as f32,
            ((self.button_view_center.y - v.center.y) * f64::from(v.scale)) as f32,
        )
    }
    pub(super) fn action_button_rect(
        &self,
        v: &Viewport,
        anchor: (i16, i16),
        label: &str,
    ) -> egui::Rect {
        sketchpad_rs::render::action_button_rect(v, anchor, label).translate(self.button_offset(v))
    }
    pub(super) fn hit(&self, v: &Viewport, p: egui::Pos2) -> Option<usize> {
        if let Some(object) = self.doc.objects.iter().rev().find(|object| {
            self.doc.is_displayed(object.id)
                && matches!(&object.kind, Kind::Image { png, transform }
                    if sketchpad_rs::render::picture_hit(v, png, *transform, p))
        }) {
            return Some(object.id);
        }
        if let Some(object) = self.doc.objects.iter().rev().find(|object| {
            self.doc.is_displayed(object.id)
                && self.doc.text_content(object.id).is_some_and(|(text, rect)| {
                    let top_left = v.screen(Point {
                        x: rect.left.into(),
                        y: rect.top.into(),
                    });
                    let saved_bottom_right = v.screen(Point {
                        x: rect.right.into(),
                        y: rect.bottom.into(),
                    });
                    let size = self
                        .doc
                        .text_font_state(object.id)
                        .map_or(16., |style| style.size_code as f32);
                    let (width, height) = if let Some(rows) = self.doc.text_rich_runs(object.id) {
                        let width = rows
                            .iter()
                            .map(|row| {
                                row.iter()
                                    .map(|run| {
                                        run.text.chars().count() as f32
                                            * run.style.size_code.map_or(size, f32::from)
                                            * 0.65
                                    })
                                    .sum::<f32>()
                            })
                            .fold(0., f32::max);
                        let height = rows
                            .iter()
                            .map(|row| {
                                row.iter()
                                    .map(|run| run.style.size_code.map_or(size, f32::from))
                                    .fold(size, f32::max)
                            })
                            .map(|line_size| (line_size * 1.1).max(19.))
                            .sum();
                        (width, height)
                    } else {
                        let width = text
                            .split('\n')
                            .map(|line| line.chars().count())
                            .max()
                            .unwrap_or(0) as f32
                            * size
                            * 0.65;
                        let height = (text.lines().count().saturating_sub(1) as f32 * 19.)
                            + size * 1.1;
                        (width, height)
                    };
                    egui::Rect::from_min_max(
                        top_left,
                        egui::pos2(
                            saved_bottom_right.x.max(top_left.x + width),
                            saved_bottom_right.y.max(top_left.y + height),
                        ),
                    )
                    .contains(p)
                })
        }) {
            return Some(object.id);
        }
        if let Some(object) = self.doc.objects.iter().rev().find(|object| {
            self.doc.is_displayed(object.id)
                && (sketchpad_rs::render::number_screen_rect_for_object(v, &self.doc, object.id)
                    .is_some_and(|rect| rect.contains(p))
                    || sketchpad_rs::render::point_parameter_iteration_label_hit(
                        v, &self.doc, object.id, p,
                    ))
        }) {
            return Some(object.id);
        }
        if let Some((id, _)) = self
            .overlay_rows()
            .iter()
            .enumerate()
            .rev()
            .find(|(row, (id, text))| self.overlay_row_rect(v, *row, *id, text).contains(p))
            .map(|(_, entry)| entry)
        {
            return Some(*id);
        }
        if let Some(object) = self.doc.objects.iter().rev().find(|object| {
            self.doc.is_displayed(object.id)
                && matches!(&object.kind, Kind::HideShowButton { label, anchor, .. }
                    | Kind::AnimationButton { label, anchor, .. }
                    | Kind::MovementButton { label, anchor, .. }
                    | Kind::ScrollButton { label, anchor, .. }
                    | Kind::PresentationButton { label, anchor, .. }
                    | Kind::LinkUrlButton { label, anchor, .. }
                    | Kind::LinkPageButton { label, anchor, .. }
                    if self.action_button_rect(v, *anchor, label).contains(p))
        }) {
            return Some(object.id);
        }
        if let Some(object) = self.doc.objects.iter().rev().find(|object| {
            self.doc.is_displayed(object.id)
                && (sketchpad_rs::render::mapped_numeric_label_hit(v, &self.doc, object.id, p)
                    || sketchpad_rs::render::branch_iteration_hit(v, &self.doc, object.id, p)
                    || sketchpad_rs::render::point_iteration_hit(v, &self.doc, object.id, p)
                    || sketchpad_rs::render::circle_iteration_hit(v, &self.doc, object.id, p))
        }) {
            return Some(object.id);
        }
        if let Some(id) = self.hit_point(v, p) {
            return Some(id);
        }
        self.doc
            .objects
            .iter()
            .rev()
            .filter(|o| self.doc.is_displayed(o.id))
            .filter_map(|o| {
                let distance = if let Some((a, b, mode)) = self.doc.line_points(o.id) {
                    let (a, b) = (v.screen(a), v.screen(b));
                    let ab = b - a;
                    let t = if ab.length_sq() < 1e-6 {
                        0.
                    } else {
                        (p - a).dot(ab) / ab.length_sq()
                    };
                    let t = match mode {
                        LineMode::Segment => t.clamp(0., 1.),
                        LineMode::Ray => t.max(0.),
                        LineMode::Line => t,
                    };
                    (p - (a + ab * t)).length()
                } else if let Some((center, radius)) = self.doc.circle_points(o.id) {
                    ((p - v.screen(center)).length() - radius as f32 * v.scale).abs()
                } else {
                    match o.kind {
                        ref kind if kind.is_arc() => {
                            let points = self.doc.arc_points(o.id, 96)?;
                            points
                                .windows(2)
                                .map(|pair| {
                                    let a = v.screen(pair[0]);
                                    let b = v.screen(pair[1]);
                                    let ab = b - a;
                                    let t = ((p - a).dot(ab) / ab.length_sq()).clamp(0., 1.);
                                    (p - (a + ab * t)).length()
                                })
                                .min_by(f32::total_cmp)?
                        }
                        ref kind if kind.is_polygon() => {
                            let points = self.doc.polygon_points(o.id)?;
                            let points: Vec<_> =
                                points.into_iter().map(|point| v.screen(point)).collect();
                            let inside = points
                                .iter()
                                .zip(points.iter().cycle().skip(1))
                                .take(points.len())
                                .fold(false, |inside, (a, b)| {
                                    inside
                                        ^ ((a.y > p.y) != (b.y > p.y)
                                            && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x)
                                });
                            if inside {
                                0.
                            } else {
                                points
                                    .iter()
                                    .zip(points.iter().cycle().skip(1))
                                    .take(points.len())
                                    .map(|(a, b)| {
                                        let ab = *b - *a;
                                        let t = if ab.length_sq() < 1e-6 {
                                            0.
                                        } else {
                                            ((p - *a).dot(ab) / ab.length_sq()).clamp(0., 1.)
                                        };
                                        (p - (*a + ab * t)).length()
                                    })
                                    .min_by(f32::total_cmp)?
                            }
                        }
                        Kind::FunctionPlot(_, _, _, _, _)
                        | Kind::ParametricPlot(_, _, _, _, _, _, _)
                        | Kind::Locus(_, _)
                        | Kind::ParameterLocus(_, _, _, _) => {
                            if let Some(polygons) = self.doc.locus_polygon_surfaces(o.id, 125)
                                && polygons.iter().any(|points| {
                                    points
                                        .iter()
                                        .zip(points.iter().cycle().skip(1))
                                        .take(points.len())
                                        .fold(false, |inside, (a, b)| {
                                            let a = v.screen(*a);
                                            let b = v.screen(*b);
                                            inside
                                                ^ ((a.y > p.y) != (b.y > p.y)
                                                    && p.x
                                                        < (b.x - a.x) * (p.y - a.y) / (b.y - a.y)
                                                            + a.x)
                                        })
                                })
                            {
                                return Some((o.id, 0.));
                            }
                            let segments = self
                                .doc
                                .function_plot_segments(o.id, 256)
                                .or_else(|| self.doc.parametric_plot_segments(o.id, 256))
                                .map(|segments| {
                                    segments
                                        .into_iter()
                                        .map(|(start, end)| (start, end, LineMode::Segment))
                                        .collect::<Vec<_>>()
                                })
                                .or_else(|| self.doc.locus_strokes(o.id, 256))?;
                            segments
                                .iter()
                                .map(|(start, end, mode)| {
                                    let a = v.screen(*start);
                                    let b = v.screen(*end);
                                    let ab = b - a;
                                    let t = if ab.length_sq() < 1e-6 {
                                        0.
                                    } else {
                                        let raw = (p - a).dot(ab) / ab.length_sq();
                                        match mode {
                                            LineMode::Segment => raw.clamp(0., 1.),
                                            LineMode::Ray => raw.max(0.),
                                            LineMode::Line => raw,
                                        }
                                    };
                                    (p - (a + ab * t)).length()
                                })
                                .min_by(f32::total_cmp)?
                        }
                        _ => return None,
                    }
                };
                (distance < 7.).then_some((o.id, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|x| x.0)
            .or_else(|| self.hit_grid(v, p))
    }
}
