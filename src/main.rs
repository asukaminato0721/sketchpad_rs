use sketchpad_rs::coordinate_measure_records::CoordinateMeasureKind;
use sketchpad_rs::document::{Clipboard, Document, Kind, LineMode, Point};
use sketchpad_rs::function_records::SimpleFunction;
use sketchpad_rs::graph::GridForm;
use sketchpad_rs::plot_points_records::PlotPointsMode;
use sketchpad_rs::render::{GeometryRenderer, Viewport, svg_page};
use sketchpad_rs::style_records::{LinePattern, LineWidth, PointStyle, RgbColor, VisualStyle};
use sketchpad_rs::value_equation_records::ValueEquationKind;
use std::{
    collections::HashMap,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::Arc,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Select,
    Point,
    Segment,
    Circle,
    Text,
}
enum PendingAction {
    New,
    Open(PathBuf),
    Quit,
}
#[derive(Clone, Copy)]
enum OutputAction {
    Svg,
    Pdf,
    Print,
}

fn pdf_from_svg(svg: &str) -> Result<Vec<u8>, String> {
    let path = temporary_pdf_path("convert");
    let result = (|| -> Result<Vec<u8>, String> {
        let mut process = Command::new("rsvg-convert")
            .arg("--format=pdf")
            .arg(format!("--output={}", path.display()))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| error.to_string())?;
        process
            .stdin
            .take()
            .ok_or("Missing PDF converter stdin")?
            .write_all(svg.as_bytes())
            .map_err(|error| error.to_string())?;
        let output = process
            .wait_with_output()
            .map_err(|error| error.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
        }
        let pdf = std::fs::read(&path).map_err(|error| error.to_string())?;
        if !pdf.starts_with(b"%PDF-") {
            return Err("PDF converter returned invalid output".into());
        }
        Ok(pdf)
    })();
    let _ = std::fs::remove_file(path);
    result
}
fn temporary_pdf_path(purpose: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "sketchpad-{purpose}-{}-{unique}.pdf",
        std::process::id()
    ))
}
#[cfg(test)]
mod pdf_tests {
    use super::*;
    #[test]
    fn converts_visible_page_svg_to_pdf_when_converter_is_available() {
        if Command::new("rsvg-convert")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let pdf = pdf_from_svg("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"20\" height=\"20\"><circle cx=\"10\" cy=\"10\" r=\"5\"/></svg>").unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
        assert!(pdf.len() > 100);
    }
}
#[derive(Clone, Copy)]
enum CircleMeasure {
    Circumference,
    Radius,
    Area,
}
#[derive(Clone, Copy)]
enum TraceMark {
    Point(Point, RgbColor),
    Segment(Point, Point, RgbColor),
}
struct Editor {
    doc: Document,
    clipboard: Option<Clipboard>,
    image_textures: HashMap<usize, egui::TextureHandle>,
    tool: Tool,
    pending: Option<usize>,
    selection_order: Vec<usize>,
    marked_center: Option<usize>,
    marked_mirror: Option<usize>,
    trace_marks: Vec<TraceMark>,
    dragging: Option<usize>,
    drag_offset: egui::Vec2,
    text_draft: String,
    inspected: Option<usize>,
    edit_buffer: String,
    scale: f32,
    center: Point,
    status: String,
    pending_action: Option<PendingAction>,
    pending_output: Option<OutputAction>,
    translate_dialog: bool,
    translation_angle: f64,
    translation_distance: f64,
    dilate_dialog: bool,
    dilation_factor: f64,
    rotate_dialog: bool,
    rotation_angle: f64,
    function_dialog: Option<bool>,
    function_draft: String,
    parameter_value_dialog: Option<usize>,
    parameter_value_draft: String,
    plot_axis_dialog: Option<Option<usize>>,
    plot_axis_draft: String,
    exit_requested: bool,
}
impl Editor {
    fn new(doc: Document) -> Self {
        let mut e = Self {
            doc,
            clipboard: None,
            image_textures: HashMap::new(),
            tool: Tool::Select,
            pending: None,
            selection_order: Vec::new(),
            marked_center: None,
            marked_mirror: None,
            trace_marks: Vec::new(),
            dragging: None,
            drag_offset: egui::Vec2::ZERO,
            text_draft: "Text".into(),
            inspected: None,
            edit_buffer: String::new(),
            scale: 1.,
            center: Point { x: 400., y: 300. },
            status: String::new(),
            pending_action: None,
            pending_output: None,
            translate_dialog: false,
            translation_angle: 90.,
            translation_distance: 1.,
            dilate_dialog: false,
            dilation_factor: 0.5,
            rotate_dialog: false,
            rotation_angle: 60.,
            function_dialog: None,
            function_draft: "x".into(),
            parameter_value_dialog: None,
            parameter_value_draft: "1".into(),
            plot_axis_dialog: None,
            plot_axis_draft: "1".into(),
            exit_requested: false,
        };
        e.fit();
        e
    }
    fn fit(&mut self) {
        let mut pts: Vec<_> = self
            .doc
            .objects
            .iter()
            .filter(|o| self.doc.is_displayed(o.id))
            .filter_map(|o| self.doc.position(o.id))
            .collect();
        for object in self
            .doc
            .objects
            .iter()
            .filter(|o| self.doc.is_displayed(o.id))
        {
            if let Kind::PlainText { rect, .. } = &object.kind {
                pts.push(Point {
                    x: rect.left.into(),
                    y: rect.top.into(),
                });
                pts.push(Point {
                    x: rect.right.into(),
                    y: rect.bottom.into(),
                });
            }
            if let Some((anchor, _)) = sketchpad_rs::render::number_lines(&object.kind) {
                pts.push(Point {
                    x: anchor.0 as f64,
                    y: anchor.1 as f64,
                });
            }
            if let Kind::Image { png, transform } = &object.kind
                && let Some(corners) = sketchpad_rs::render::picture_screen_quad(
                    &Viewport {
                        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::ZERO),
                        scale: 1.,
                        center: Point { x: 0., y: 0. },
                    },
                    png,
                    *transform,
                )
            {
                // The zero-sized viewport places world positions at the origin.
                pts.extend(corners.map(|corner| Point {
                    x: corner.x as f64,
                    y: corner.y as f64,
                }));
            }
        }
        if pts.is_empty() {
            return;
        }
        let (mut minx, mut miny, mut maxx, mut maxy) = (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        for p in pts {
            minx = minx.min(p.x);
            miny = miny.min(p.y);
            maxx = maxx.max(p.x);
            maxy = maxy.max(p.y);
        }
        self.center = Point {
            x: (minx + maxx) / 2.,
            y: (miny + maxy) / 2.,
        };
        self.scale = (650. / (maxx - minx).max(100.) as f32)
            .min(480. / (maxy - miny).max(100.) as f32)
            .clamp(0.2, 3.);
    }
    fn open(&mut self, path: PathBuf) {
        match Document::open(&path) {
            Ok(d) => {
                self.replace_document(d);
                self.status = format!("Opened {}", path.display());
            }
            Err(e) => self.status = format!("Open failed: {e}"),
        }
    }
    fn import_picture(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG picture", &["png"])
            .pick_file()
        else {
            return;
        };
        let result = (|| -> Result<usize, String> {
            let png = std::fs::read(&path).map_err(|error| error.to_string())?;
            self.doc.import_png(&png, self.center)
        })();
        match result {
            Ok(id) => {
                self.image_textures.clear();
                self.select_only(id);
                self.status = format!("Imported {}", path.display());
            }
            Err(error) => self.status = format!("Picture import refused: {error}"),
        }
    }
    fn replace_document(&mut self, document: Document) {
        let clipboard = self.clipboard.take();
        *self = Self::new(document);
        self.clipboard = clipboard;
    }
    fn save(&mut self, as_new: bool) -> bool {
        let path = if as_new || self.doc.path.is_none() {
            rfd::FileDialog::new()
                .add_filter("Sketchpad document", &["gsp"])
                .set_file_name("Sketch.gsp")
                .save_file()
        } else {
            self.doc.path.clone()
        };
        let Some(path) = path else { return false };
        match self.doc.save_to(&path) {
            Ok(()) => {
                self.status = format!("Saved {}", path.display());
                true
            }
            Err(e) => {
                self.status = format!("Save refused: {e}");
                false
            }
        }
    }
    fn output_page(&mut self, viewport: &Viewport, action: OutputAction) {
        let svg = match svg_page(&self.doc, viewport) {
            Ok(svg) => svg,
            Err(error) => {
                self.status = format!("Output refused: {error}");
                return;
            }
        };
        if matches!(action, OutputAction::Svg) {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("SVG image", &["svg"])
                .set_file_name("Sketch.svg")
                .save_file()
            else {
                return;
            };
            match std::fs::write(&path, svg) {
                Ok(()) => self.status = format!("Exported {}", path.display()),
                Err(error) => self.status = format!("Export failed: {error}"),
            }
            return;
        }
        let pdf = match pdf_from_svg(&svg) {
            Ok(pdf) => pdf,
            Err(error) => {
                self.status = format!("PDF conversion failed: {error}");
                return;
            }
        };
        if matches!(action, OutputAction::Pdf) {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("PDF document", &["pdf"])
                .set_file_name("Sketch.pdf")
                .save_file()
            else {
                return;
            };
            match std::fs::write(&path, pdf) {
                Ok(()) => self.status = format!("Exported {}", path.display()),
                Err(error) => self.status = format!("Export failed: {error}"),
            }
            return;
        }
        let path = temporary_pdf_path("print");
        if let Err(error) = std::fs::write(&path, pdf) {
            self.status = format!("Print failed: {error}");
            return;
        }
        let result = Command::new("lp").arg(&path).output();
        let _ = std::fs::remove_file(&path);
        self.status = match result {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).trim().to_owned()
            }
            Ok(output) => format!(
                "Print failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            Err(error) => format!("Print failed: {error}"),
        };
    }
    fn finish_pending_action(&mut self) {
        match self.pending_action.take() {
            Some(PendingAction::New) => self.replace_document(Document::new()),
            Some(PendingAction::Open(path)) => self.open(path),
            Some(PendingAction::Quit) => self.exit_requested = true,
            None => {}
        }
    }
    fn hit_point(&self, v: &Viewport, p: egui::Pos2) -> Option<usize> {
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
    fn hit(&self, v: &Viewport, p: egui::Pos2) -> Option<usize> {
        if let Some(object) = self.doc.objects.iter().rev().find(|object| {
            self.doc.is_displayed(object.id)
                && matches!(&object.kind, Kind::Image { png, transform }
                    if sketchpad_rs::render::picture_hit(v, png, *transform, p))
        }) {
            return Some(object.id);
        }
        if let Some(object) = self.doc.objects.iter().rev().find(|object| {
            self.doc.is_displayed(object.id)
                && matches!(&object.kind, Kind::PlainText { rect, .. }
                    if egui::Rect::from_two_pos(
                        v.screen(Point { x: rect.left.into(), y: rect.top.into() }),
                        v.screen(Point { x: rect.right.into(), y: rect.bottom.into() })
                    ).contains(p))
        }) {
            return Some(object.id);
        }
        if let Some(object) = self.doc.objects.iter().rev().find(|object| {
            self.doc.is_displayed(object.id)
                && sketchpad_rs::render::number_screen_rect(v, &object.kind)
                    .is_some_and(|rect| rect.contains(p))
        }) {
            return Some(object.id);
        }
        if let Some(object) = self.doc.objects.iter().rev().find(|object| {
            self.doc.is_displayed(object.id)
                && matches!(&object.kind, Kind::HideShowButton { label, anchor, .. }
                    if sketchpad_rs::render::action_button_rect(v, *anchor, label).contains(p))
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
                } else {
                    match o.kind {
                        Kind::Circle(_, _)
                        | Kind::CircleBySegment(_, _)
                        | Kind::ReflectedCircle(_, _)
                        | Kind::ScaledCircle(_, _, _) => {
                            let (center, radius) = self.doc.circle_points(o.id)?;
                            ((p - v.screen(center)).length() - radius as f32 * v.scale).abs()
                        }
                        Kind::ThreePointArc(_, _, _) | Kind::ArcOnCircle(_, _, _) => {
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
                        Kind::Polygon(_) => {
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
                        Kind::FunctionPlot(_, _, _, _, _) => {
                            let points = self.doc.function_plot_points(o.id, 256)?;
                            points
                                .windows(2)
                                .map(|pair| {
                                    let a = v.screen(pair[0]);
                                    let b = v.screen(pair[1]);
                                    let ab = b - a;
                                    let t = if ab.length_sq() < 1e-6 {
                                        0.
                                    } else {
                                        ((p - a).dot(ab) / ab.length_sq()).clamp(0., 1.)
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
    }
    fn point_at(&mut self, v: &Viewport, p: egui::Pos2) -> usize {
        if let Some(id) = self.hit_point(v, p) {
            return id;
        }
        if let Some(host) = self.hit(v, p)
            && let Some(kind) = self.doc.constrained_point(host, v.world(p))
        {
            return self.doc.add(kind);
        }
        self.doc.add(Kind::Free(self.doc.snap_point(v.world(p))))
    }
    fn select_only(&mut self, id: usize) {
        self.doc.selected.clear();
        self.doc.selected.insert(id);
        self.selection_order.clear();
        self.selection_order.push(id);
    }
    fn select_related(&mut self, children: bool) {
        let current = self.doc.selected.clone();
        let related: Vec<_> = if children {
            self.doc
                .objects
                .iter()
                .filter(|object| {
                    object.page == self.doc.active_page && object.visible && !object.deleted
                })
                .filter(|object| {
                    self.doc
                        .dependencies_of(object.id)
                        .unwrap_or_default()
                        .iter()
                        .any(|id| current.contains(id))
                })
                .map(|object| object.id)
                .collect()
        } else {
            self.doc
                .objects
                .iter()
                .filter(|object| current.contains(&object.id))
                .flat_map(|object| self.doc.dependencies_of(object.id).unwrap_or_default())
                .filter(|id| {
                    self.doc
                        .objects
                        .get(id.saturating_sub(1))
                        .is_some_and(|object| {
                            object.page == self.doc.active_page && object.visible && !object.deleted
                        })
                })
                .collect()
        };
        self.doc.selected = related.iter().copied().collect();
        self.selection_order = related;
    }
    fn clear_selected(&mut self) {
        match self.doc.clear_selected() {
            Ok(removed) if removed.is_empty() => self.status = "Select objects to clear".into(),
            Ok(removed) => {
                self.selection_order.clear();
                self.status = format!("Cleared {} object(s), including dependents", removed.len());
            }
            Err(error) => self.status = error,
        }
    }
    fn measure_length(&mut self) {
        if self.doc.partial_view || self.doc.page_count != 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [segment] = self.selection_order.as_slice() else {
            self.status = "Select one segment to measure".into();
            return;
        };
        let segment = *segment;
        if !matches!(
            self.doc
                .objects
                .get(segment.saturating_sub(1))
                .map(|o| &o.kind),
            Some(Kind::Segment(_, _))
        ) {
            self.status = "Select one segment to measure".into();
            return;
        }
        let id = self.doc.add(Kind::SegmentLength(segment));
        self.select_only(id);
        self.status = format!("Measured segment {segment}");
    }
    fn define_coordinate_system(&mut self) {
        match self.doc.add_default_coordinate_system(self.center) {
            Ok(frame) => {
                self.select_only(frame);
                self.status = format!("Defined Cartesian coordinate system {frame}");
            }
            Err(error) => self.status = error,
        }
    }
    fn measure_slope(&mut self) {
        if self.doc.partial_view || self.doc.page_count != 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [line] = self.selection_order.as_slice() else {
            self.status = "Select one segment, line, or ray to measure".into();
            return;
        };
        let line = *line;
        if !matches!(
            self.doc
                .objects
                .get(line.saturating_sub(1))
                .map(|o| &o.kind),
            Some(Kind::Segment(_, _) | Kind::Line(_, _) | Kind::Ray(_, _))
        ) {
            self.status = "Select one segment, line, or ray to measure".into();
            return;
        }
        let frame = match self.doc.active_coordinate_system() {
            Some(frame) => frame,
            None => match self.doc.add_default_coordinate_system(self.center) {
                Ok(frame) => frame,
                Err(error) => {
                    self.status = error;
                    return;
                }
            },
        };
        let id = self.doc.add(Kind::SlopeMeasurement(line, frame));
        self.select_only(id);
        self.status = format!("Measured slope of object {line}");
    }
    fn measure_value_equation(&mut self, equation: bool) {
        if self.doc.partial_view || self.doc.page_count != 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [first] = self.selection_order.as_slice() else {
            self.status = "Select one constrained point, line, or circle".into();
            return;
        };
        let first = *first;
        let (kind, second) = match self
            .doc
            .objects
            .get(first.saturating_sub(1))
            .map(|object| &object.kind)
        {
            Some(Kind::PointOnSegment(host, _) | Kind::PointOnCircle(host, _, _)) if !equation => {
                (ValueEquationKind::PointValue, *host)
            }
            Some(Kind::Line(_, _)) if equation => (ValueEquationKind::LineEquation, 0),
            Some(Kind::Circle(_, _)) if equation => (ValueEquationKind::CircleEquation, 0),
            _ => {
                self.status = if equation {
                    "Select one line or circle for Equation"
                } else {
                    "Select one point constrained to a segment or circle"
                }
                .into();
                return;
            }
        };
        let second = if equation {
            match self.doc.active_coordinate_system() {
                Some(frame) => frame,
                None => match self.doc.add_default_coordinate_system(self.center) {
                    Ok(frame) => frame,
                    Err(error) => {
                        self.status = error;
                        return;
                    }
                },
            }
        } else {
            second
        };
        let id = self.doc.add(Kind::ValueEquation {
            kind,
            first,
            second,
        });
        if self.doc.value_equation_value(id).is_none() {
            self.doc.undo();
            self.status = "The selected geometry has no supported value or equation".into();
            return;
        }
        self.select_only(id);
        self.status = format!("Measured {kind:?} of object {first}");
    }
    fn measure_distance(&mut self) {
        if self.doc.partial_view || self.doc.page_count != 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [a, b] = self.selection_order.as_slice() else {
            self.status = "Select two points to measure".into();
            return;
        };
        let (a, b) = (*a, *b);
        if self.doc.position(a).is_none() || self.doc.position(b).is_none() {
            self.status = "Select two points to measure".into();
            return;
        }
        let id = self.doc.add(Kind::PointDistance(a, b));
        self.select_only(id);
        self.status = format!("Measured distance between {a} and {b}");
    }
    fn measure_coordinate(&mut self, kind: CoordinateMeasureKind) {
        if self.doc.partial_view || self.doc.page_count != 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let selected = self.selection_order.as_slice();
        let (first, second) = match (kind, selected) {
            (CoordinateMeasureKind::CoordinateDistance, [a, b]) => (*a, Some(*b)),
            (
                CoordinateMeasureKind::Coordinates
                | CoordinateMeasureKind::Abscissa
                | CoordinateMeasureKind::Ordinate,
                [a],
            ) => (*a, None),
            _ => {
                self.status = if kind == CoordinateMeasureKind::CoordinateDistance {
                    "Select two points to measure"
                } else {
                    "Select one point to measure"
                }
                .into();
                return;
            }
        };
        if self.doc.position(first).is_none()
            || second.is_some_and(|id| self.doc.position(id).is_none())
        {
            self.status = "Select point objects to measure".into();
            return;
        }
        let frame = match self.doc.active_coordinate_system() {
            Some(frame) => frame,
            None => match self.doc.add_default_coordinate_system(self.center) {
                Ok(frame) => frame,
                Err(error) => {
                    self.status = error;
                    return;
                }
            },
        };
        let id = self.doc.add(Kind::CoordinateMeasurement {
            kind,
            first,
            second,
            frame,
        });
        self.select_only(id);
        self.status = format!("Measured {kind:?} for point {first}");
    }
    fn measure_ratio(&mut self) {
        if self.doc.partial_view || self.doc.page_count != 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [a, b, c] = self.selection_order.as_slice() else {
            self.status = "Select three collinear points in origin, base, target order".into();
            return;
        };
        let (a, b, c) = (*a, *b, *c);
        let (Some(origin), Some(base), Some(target)) = (
            self.doc.position(a),
            self.doc.position(b),
            self.doc.position(c),
        ) else {
            self.status = "Select three points to measure".into();
            return;
        };
        let (bx, by) = (base.x - origin.x, base.y - origin.y);
        let (cx, cy) = (target.x - origin.x, target.y - origin.y);
        let cross = bx * cy - by * cx;
        let tolerance = 1e-8 * bx.hypot(by) * cx.hypot(cy);
        if self.doc.ratio_for_points(a, b, c).is_none() || cross.abs() > tolerance {
            self.status =
                "Ratio construction needs three collinear points and a nonzero base".into();
            return;
        }
        let id = self.doc.add(Kind::PointRatio(a, b, c));
        self.select_only(id);
        self.status = format!("Measured ratio from points {a}, {b}, {c}");
    }
    fn measure_perimeter(&mut self) {
        if self.doc.partial_view || self.doc.page_count != 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [polygon] = self.selection_order.as_slice() else {
            self.status = "Select one polygon interior to measure".into();
            return;
        };
        let polygon = *polygon;
        if self.doc.polygon_perimeter_cm(polygon).is_none() {
            self.status = "Select one polygon interior to measure".into();
            return;
        }
        let id = self.doc.add(Kind::PolygonPerimeter(polygon));
        self.select_only(id);
        self.status = format!("Measured perimeter of polygon {polygon}");
    }
    fn measure_angle(&mut self) {
        if self.doc.partial_view || self.doc.page_count != 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [a, b, c] = self.selection_order.as_slice() else {
            self.status = "Select three points in angle order".into();
            return;
        };
        let (a, b, c) = (*a, *b, *c);
        if self.doc.angle_for_points(a, b, c).is_none() {
            self.status = "The angle needs two nonzero rays".into();
            return;
        }
        let id = self.doc.add(Kind::AngleMeasurement(a, b, c));
        self.select_only(id);
        self.status = format!("Measured angle {a}-{b}-{c}");
    }
    fn measure_circle(&mut self, measurement: CircleMeasure) {
        if self.doc.partial_view || self.doc.page_count != 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [circle] = self.selection_order.as_slice() else {
            self.status = "Select one circle to measure".into();
            return;
        };
        let circle = *circle;
        if !matches!(
            self.doc
                .objects
                .get(circle.saturating_sub(1))
                .map(|o| &o.kind),
            Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _))
        ) {
            self.status = "Select one circle to measure".into();
            return;
        }
        let (kind, name) = match measurement {
            CircleMeasure::Circumference => (Kind::CircleCircumference(circle), "circumference"),
            CircleMeasure::Radius => (Kind::CircleRadius(circle), "radius"),
            CircleMeasure::Area => (Kind::CircleArea(circle), "area"),
        };
        let id = self.doc.add(kind);
        self.select_only(id);
        self.status = format!("Measured {name} of circle {circle}");
    }
    fn measure_arc_angle(&mut self) {
        if self.doc.partial_view || self.doc.page_count != 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [arc] = self.selection_order.as_slice() else {
            self.status = "Select one arc to measure".into();
            return;
        };
        let arc = *arc;
        if !matches!(
            self.doc.objects.get(arc.saturating_sub(1)).map(|o| &o.kind),
            Some(Kind::ThreePointArc(_, _, _) | Kind::ArcOnCircle(_, _, _))
        ) {
            self.status = "Select one arc to measure".into();
            return;
        }
        let id = self.doc.add(Kind::ArcAngle(arc));
        self.select_only(id);
        self.status = format!("Measured angle of arc {arc}");
    }
    fn measure_arc_length(&mut self) {
        if self.doc.partial_view || self.doc.page_count != 1 {
            self.status = "This document cannot accept a measurement safely".into();
            return;
        }
        let [arc] = self.selection_order.as_slice() else {
            self.status = "Select one arc to measure".into();
            return;
        };
        let arc = *arc;
        if !matches!(
            self.doc.objects.get(arc.saturating_sub(1)).map(|o| &o.kind),
            Some(Kind::ThreePointArc(_, _, _) | Kind::ArcOnCircle(_, _, _))
        ) {
            self.status = "Select one arc to measure".into();
            return;
        }
        let id = self.doc.add(Kind::ArcLength(arc));
        self.select_only(id);
        self.status = format!("Measured length of arc {arc}");
    }
    fn copy_selected(&mut self) -> bool {
        match self.doc.copy_selected() {
            Ok(clipboard) => {
                self.status = format!("Copied {} object(s)", clipboard.len());
                self.clipboard = Some(clipboard);
                true
            }
            Err(error) => {
                self.status = error;
                false
            }
        }
    }
    fn cut_selected(&mut self) {
        let clipboard = match self.doc.copy_selected() {
            Ok(clipboard) => clipboard,
            Err(error) => {
                self.status = error;
                return;
            }
        };
        match self.doc.clear_selected() {
            Ok(removed) => {
                self.clipboard = Some(clipboard);
                self.selection_order.clear();
                self.status = format!("Cut {} object(s)", removed.len());
            }
            Err(error) => self.status = error,
        }
    }
    fn paste(&mut self) {
        let Some(clipboard) = &self.clipboard else {
            self.status = "Clipboard is empty".into();
            return;
        };
        match self.doc.paste(clipboard) {
            Ok(ids) => {
                self.selection_order = ids;
                self.status = format!("Pasted {} object(s)", self.selection_order.len());
            }
            Err(error) => self.status = error,
        }
    }
    fn click(&mut self, v: &Viewport, p: egui::Pos2, multi: bool) {
        if self.doc.partial_view && self.tool != Tool::Select {
            self.status =
                "This page contains unsupported records; construction cannot be saved safely"
                    .into();
            return;
        }
        match self.tool {
            Tool::Select => {
                let hit = self.hit(v, p);
                if !multi
                    && let Some(id) = hit
                    && matches!(self.doc.objects[id - 1].kind, Kind::HideShowButton { .. })
                {
                    match self.doc.activate_hide_show_button(id) {
                        Ok(()) => self.status = format!("Activated button {id}"),
                        Err(error) => self.status = error,
                    }
                    self.selection_order.clear();
                    return;
                }
                if !multi {
                    self.doc.selected.clear();
                    self.selection_order.clear();
                }
                if let Some(id) = hit {
                    if multi && !self.doc.selected.insert(id) {
                        self.doc.selected.remove(&id);
                        self.selection_order.retain(|selected| *selected != id);
                    } else {
                        self.doc.selected.insert(id);
                        self.selection_order.push(id);
                    }
                }
            }
            Tool::Point => {
                let id = self.point_at(v, p);
                self.select_only(id);
            }
            Tool::Text => match self.doc.add_text(&self.text_draft, v.world(p)) {
                Ok(id) => self.select_only(id),
                Err(error) => self.status = error,
            },
            Tool::Segment | Tool::Circle => {
                let id = self.point_at(v, p);
                if let Some(a) = self.pending.take() {
                    if a != id {
                        let k = if self.tool == Tool::Segment {
                            Kind::Segment(a, id)
                        } else {
                            Kind::Circle(a, id)
                        };
                        let made = self.doc.add(k);
                        self.select_only(made);
                    }
                } else {
                    self.pending = Some(id);
                }
            }
        }
    }
    fn pair(&mut self, mode: u8) {
        if self.doc.partial_view {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let ids: Vec<_> = self
            .selection_order
            .iter()
            .copied()
            .filter(|id| self.doc.position(*id).is_some())
            .collect();
        if ids.len() != 2 {
            self.status = "Select exactly two points".into();
            return;
        }
        let (a, b) = (ids[0], ids[1]);
        let kind = match mode {
            0 => Kind::Segment(a, b),
            1 => Kind::Circle(a, b),
            3 => Kind::Line(a, b),
            4 => Kind::Ray(a, b),
            _ => {
                if !self
                    .doc
                    .objects
                    .iter()
                    .any(|o| matches!(o.kind,Kind::Segment(x,y) if (x==a&&y==b)||(x==b&&y==a)))
                {
                    self.doc.add(Kind::Segment(a, b));
                }
                Kind::Midpoint(a, b)
            }
        };
        let id = self.doc.add(kind);
        self.select_only(id);
    }
    fn derived_line(&mut self, perpendicular: bool) {
        if self.doc.partial_view {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let points: Vec<_> = self
            .selection_order
            .iter()
            .copied()
            .filter(|id| self.doc.position(*id).is_some())
            .collect();
        let lines: Vec<_> = self
            .selection_order
            .iter()
            .copied()
            .filter(|id| self.doc.line_points(*id).is_some())
            .collect();
        if points.len() != 1 || lines.len() != 1 || self.doc.selected.len() != 2 {
            self.status = "Select one point and one straight object".into();
            return;
        }
        let kind = if perpendicular {
            Kind::Perpendicular(points[0], lines[0])
        } else {
            Kind::Parallel(points[0], lines[0])
        };
        let id = self.doc.add(kind);
        self.select_only(id);
    }
    fn circle_by_segment(&mut self) {
        if self.doc.partial_view {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let selected = &self.selection_order;
        let centers: Vec<_> = selected
            .iter()
            .copied()
            .filter(|id| self.doc.position(*id).is_some())
            .collect();
        let segments: Vec<_> = selected
            .iter()
            .copied()
            .filter(|id| {
                matches!(
                    self.doc.objects.get(id.saturating_sub(1)).map(|o| &o.kind),
                    Some(Kind::Segment(_, _))
                )
            })
            .collect();
        if selected.len() != 2 || centers.len() != 1 || segments.len() != 1 {
            self.status = "Select one center point and one radius segment".into();
            return;
        }
        let id = self.doc.add(Kind::CircleBySegment(centers[0], segments[0]));
        self.select_only(id);
    }
    fn polygon_interior(&mut self) {
        if self.doc.partial_view || self.doc.page_count != 1 {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let vertices = self.selection_order.clone();
        if vertices.len() < 3 || vertices.iter().any(|&id| self.doc.position(id).is_none()) {
            self.status = "Select at least three points in polygon order".into();
            return;
        }
        let id = self.doc.add(Kind::Polygon(vertices));
        self.select_only(id);
        self.status = format!("Constructed polygon interior {id}");
    }
    fn point_on_object(&mut self) {
        if self.doc.partial_view {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let host = self.selection_order.iter().copied().find(|id| {
            matches!(
                self.doc.objects.get(id.saturating_sub(1)).map(|o| &o.kind),
                Some(
                    Kind::Segment(_, _)
                        | Kind::Circle(_, _)
                        | Kind::CircleBySegment(_, _)
                        | Kind::ReflectedCircle(_, _)
                        | Kind::ScaledCircle(_, _, _)
                        | Kind::ThreePointArc(_, _, _)
                        | Kind::ArcOnCircle(_, _, _)
                )
            )
        });
        let Some(host) = host else {
            self.status = "Select a segment, circle, or arc".into();
            return;
        };
        if self.selection_order.len() > 2 {
            self.status = "Select one host, optionally with a point to project".into();
            return;
        }
        let position = self
            .selection_order
            .iter()
            .copied()
            .filter(|id| *id != host)
            .find_map(|id| self.doc.position(id))
            .or_else(|| {
                if let Some((a, b, _)) = self.doc.line_points(host) {
                    Some(Point {
                        x: (a.x + b.x) / 2.,
                        y: (a.y + b.y) / 2.,
                    })
                } else if let Some((center, radius)) = self.doc.circle_points(host) {
                    Some(Point {
                        x: center.x + radius,
                        y: center.y,
                    })
                } else {
                    self.doc.point_on_arc(host, 0.5)
                }
            });
        let Some(kind) = position.and_then(|p| self.doc.constrained_point(host, p)) else {
            self.status = "Selected host has no usable geometry".into();
            return;
        };
        let id = self.doc.add(kind);
        self.select_only(id);
    }
    fn intersection(&mut self) {
        if self.doc.partial_view {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let ids: Vec<_> = self.selection_order.clone();
        if ids.len() != 2 {
            self.status = "Select two straight objects, a line and circle, or two circles".into();
            return;
        }
        let (a, b) = (ids[0], ids[1]);
        let kinds = if self.doc.line_points(a).is_some() && self.doc.line_points(b).is_some() {
            vec![Kind::Intersection(a, b)]
        } else if (self.doc.line_points(a).is_some() && self.doc.circle_points(b).is_some())
            || (self.doc.circle_points(a).is_some() && self.doc.line_points(b).is_some())
        {
            let (line, circle) = if self.doc.line_points(a).is_some() {
                (a, b)
            } else {
                (b, a)
            };
            vec![
                Kind::LineCircleIntersection(line, circle, 0),
                Kind::LineCircleIntersection(line, circle, 1),
            ]
        } else if self.doc.circle_points(a).is_some() && self.doc.circle_points(b).is_some() {
            vec![
                Kind::CircleCircleIntersection(a, b, 0),
                Kind::CircleCircleIntersection(a, b, 1),
            ]
        } else {
            self.status = "Select two straight objects, a line and circle, or two circles".into();
            return;
        };
        self.doc.selected.clear();
        self.selection_order.clear();
        for kind in kinds {
            let id = self.doc.add(kind);
            self.doc.selected.insert(id);
            self.selection_order.push(id);
        }
    }
    fn angle_bisector(&mut self) {
        if self.doc.partial_view {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let ids = &self.selection_order;
        if ids.len() != 3 || ids.iter().any(|id| self.doc.position(*id).is_none()) {
            self.status = "Select three points in order, with the vertex second".into();
            return;
        }
        let id = self.doc.add(Kind::Bisector(ids[0], ids[1], ids[2]));
        self.select_only(id);
    }
    fn three_point_arc(&mut self) {
        if self.doc.partial_view {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let ids = &self.selection_order;
        if ids.len() != 3 || ids.iter().any(|id| self.doc.position(*id).is_none()) {
            self.status = "Select the arc start, a point on the arc, then its end".into();
            return;
        }
        let id = self.doc.add(Kind::ThreePointArc(ids[0], ids[1], ids[2]));
        self.select_only(id);
    }
    fn arc_on_circle(&mut self) {
        if self.doc.partial_view {
            self.status = "Construction on this document cannot be saved safely".into();
            return;
        }
        let ids = &self.selection_order;
        if ids.len() != 3 {
            self.status = "Select a circle, then its start and end points".into();
            return;
        }
        let (circle, start, end) = (ids[0], ids[1], ids[2]);
        let valid = [start,end].iter().all(|id| matches!(self.doc.objects.get(id.saturating_sub(1)).map(|o| &o.kind), Some(Kind::PointOnCircle(host, _, _)) if *host == circle));
        if !valid || self.doc.circle_points(circle).is_none() {
            self.status = "Select a circle and two points constrained to it".into();
            return;
        }
        let id = self.doc.add(Kind::ArcOnCircle(circle, start, end));
        self.select_only(id);
    }
    fn translate_selected(&mut self) {
        if self.doc.partial_view {
            self.status = "Translation on this document cannot be saved safely".into();
            return;
        }
        if self.selection_order.len() != 1 || self.doc.position(self.selection_order[0]).is_none() {
            self.status = "Select one point to translate".into();
            return;
        }
        if !self.translation_angle.is_finite() || !self.translation_distance.is_finite() {
            self.status = "Translation values must be finite".into();
            return;
        }
        let id = self.doc.add(Kind::PolarTranslatedPoint(
            self.selection_order[0],
            self.translation_angle,
            37.79527559055118,
            self.translation_distance,
        ));
        self.select_only(id);
    }
    fn reflect_selected(&mut self) {
        if self.doc.partial_view {
            self.status = "Reflection on this document cannot be saved safely".into();
            return;
        }
        let (source, mirror) = match self.selection_order.as_slice() {
            [source, mirror] => (*source, *mirror),
            [source] => match self.marked_mirror {
                Some(mirror) => (*source, mirror),
                None => {
                    self.status = "Select a point or circle and mark a mirror line".into();
                    return;
                }
            },
            _ => {
                self.status = "Select a point or circle, then a mirror line".into();
                return;
            }
        };
        if self
            .doc
            .objects
            .get(mirror - 1)
            .is_none_or(|o| o.deleted || o.page != self.doc.active_page)
        {
            self.status = "Marked mirror line is unavailable".into();
            return;
        }
        if self.doc.line_points(mirror).is_none() {
            self.status = "Select a point or circle, then a mirror line".into();
            return;
        }
        let kind = if self.doc.position(source).is_some() {
            Kind::ReflectedPoint(source, mirror)
        } else if self.doc.circle_points(source).is_some() {
            Kind::ReflectedCircle(source, mirror)
        } else {
            self.status = "Select a point or circle, then a mirror line".into();
            return;
        };
        let id = self.doc.add(kind);
        self.select_only(id);
    }
    fn mark_center_selected(&mut self) {
        let [id] = self.selection_order.as_slice() else {
            self.status = "Select one point to mark as center".into();
            return;
        };
        if self.doc.position(*id).is_none() {
            self.status = "Select one point to mark as center".into();
            return;
        }
        self.marked_center = Some(*id);
        self.status = format!("Marked point {id} as center");
    }
    fn mark_mirror_selected(&mut self) {
        let [id] = self.selection_order.as_slice() else {
            self.status = "Select one straight object to mark as mirror".into();
            return;
        };
        if self.doc.line_points(*id).is_none() {
            self.status = "Select one straight object to mark as mirror".into();
            return;
        }
        self.marked_mirror = Some(*id);
        self.status = format!("Marked object {id} as mirror");
    }
    fn dilate_selected(&mut self) {
        if self.doc.partial_view {
            self.status = "Dilation on this document cannot be saved safely".into();
            return;
        }
        if !self.dilation_factor.is_finite() || self.dilation_factor == 0. {
            self.status =
                "Select a point or circle, then a center point; use a nonzero ratio".into();
            return;
        }
        let (source, center) = match self.selection_order.as_slice() {
            [source, center] => (*source, *center),
            [source] => match self.marked_center {
                Some(center) => (*source, center),
                None => {
                    self.status = "Select a source and mark a center point".into();
                    return;
                }
            },
            _ => {
                self.status = "Select a point or circle, then a center point".into();
                return;
            }
        };
        if self
            .doc
            .objects
            .get(center - 1)
            .is_none_or(|o| o.deleted || o.page != self.doc.active_page)
        {
            self.status = "Marked center point is unavailable".into();
            return;
        }
        if self.doc.position(center).is_none() {
            self.status = "Select a point or circle, then a center point".into();
            return;
        }
        let kind = if self.doc.position(source).is_some() {
            Kind::ScaledPoint(source, center, self.dilation_factor)
        } else if self.doc.circle_points(source).is_some() {
            Kind::ScaledCircle(source, center, self.dilation_factor)
        } else {
            self.status = "Select a point or circle, then a center point".into();
            return;
        };
        let id = self.doc.add(kind);
        self.select_only(id);
    }
    fn rotate_selected(&mut self) {
        if self.doc.partial_view {
            self.status = "Rotation on this document cannot be saved safely".into();
            return;
        }
        if !self.rotation_angle.is_finite() {
            self.status = "Select a source point, then a center point; use a finite angle".into();
            return;
        }
        let (source, center) = match self.selection_order.as_slice() {
            [source, center] => (*source, *center),
            [source] => match self.marked_center {
                Some(center) => (*source, center),
                None => {
                    self.status = "Select a source and mark a center point".into();
                    return;
                }
            },
            _ => {
                self.status = "Select a source point, then a center point".into();
                return;
            }
        };
        if self
            .doc
            .objects
            .get(center - 1)
            .is_none_or(|o| o.deleted || o.page != self.doc.active_page)
        {
            self.status = "Marked center point is unavailable".into();
            return;
        }
        if self.doc.position(source).is_none() || self.doc.position(center).is_none() {
            self.status = "Select a source point, then a center point".into();
            return;
        }
        let id = self
            .doc
            .add(Kind::RotatedPoint(source, center, self.rotation_angle));
        self.select_only(id);
    }
    fn capture_traces(&mut self) {
        for object in self
            .doc
            .objects
            .iter()
            .filter(|o| o.tracing && o.visible && !o.deleted && o.page == self.doc.active_page)
        {
            let color = match object.visual_style {
                Some(VisualStyle::Point { color, .. } | VisualStyle::Stroke { color, .. }) => color,
                _ => RgbColor {
                    red: 23,
                    green: 107,
                    blue: 214,
                },
            };
            if let Some(point) = self.doc.position(object.id) {
                self.trace_marks.push(TraceMark::Point(point, color));
            } else if matches!(object.kind, Kind::Segment(_, _))
                && let Some((a, b, LineMode::Segment)) = self.doc.line_points(object.id)
            {
                self.trace_marks.push(TraceMark::Segment(a, b, color));
            }
        }
        if self.trace_marks.len() > 8192 {
            self.trace_marks.drain(..self.trace_marks.len() - 8192);
        }
    }
    fn ui(&mut self, root: &mut egui::Ui) -> Viewport {
        if !root.ctx().egui_wants_keyboard_input() && self.pending_action.is_none() {
            if root.input(|input| input.modifiers.command && input.key_pressed(egui::Key::C)) {
                self.copy_selected();
            }
            if root.input(|input| input.modifiers.command && input.key_pressed(egui::Key::X)) {
                self.cut_selected();
            }
            if root.input(|input| input.modifiers.command && input.key_pressed(egui::Key::V)) {
                self.paste();
            }
        }
        if root.input(|input| {
            input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace)
        }) && !root.ctx().egui_wants_keyboard_input()
            && self.pending_action.is_none()
        {
            self.clear_selected();
        }
        egui::Panel::top("menu").show(root, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        if self.doc.dirty {
                            self.pending_action = Some(PendingAction::New);
                        } else {
                            self.replace_document(Document::new());
                        }
                        ui.close();
                    }
                    if ui.button("Close").clicked() {
                        if self.doc.dirty {
                            self.pending_action = Some(PendingAction::New);
                        } else {
                            self.replace_document(Document::new());
                        }
                        ui.close();
                    }
                    if ui.button("Open…").clicked() {
                        if let Some(p) = rfd::FileDialog::new()
                            .add_filter("Sketchpad document", &["gsp"])
                            .pick_file()
                        {
                            if self.doc.dirty {
                                self.pending_action = Some(PendingAction::Open(p));
                            } else {
                                self.open(p);
                            }
                        }
                        ui.close();
                    }
                    if ui.button("Import PNG Picture…").clicked() {
                        self.import_picture();
                        ui.close();
                    }
                    if ui.button("Save").clicked() {
                        self.save(false);
                        ui.close();
                    }
                    if ui.button("Save As…").clicked() {
                        self.save(true);
                        ui.close();
                    }
                    if ui.button("Export SVG…").clicked() {
                        self.pending_output = Some(OutputAction::Svg);
                        ui.close();
                    }
                    if ui.button("Export PDF…").clicked() {
                        self.pending_output = Some(OutputAction::Pdf);
                        ui.close();
                    }
                    if ui.button("Print…").clicked() {
                        self.pending_output = Some(OutputAction::Print);
                        ui.close();
                    }
                    if ui.button("Quit").clicked() {
                        if self.doc.dirty {
                            self.pending_action = Some(PendingAction::Quit);
                        } else {
                            self.exit_requested = true;
                        }
                        ui.close();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    ui.menu_button("Action Buttons", |ui| {
                        let target = self.selection_order.iter().copied().find(|id| {
                            self.doc.objects.get(id - 1).is_some_and(|object| {
                                object.page == self.doc.active_page
                                    && object.visible
                                    && !object.deleted
                                    && !matches!(
                                        object.kind,
                                        Kind::HideShowButton { .. } | Kind::Unsupported(_)
                                    )
                            })
                        });
                        if ui
                            .add_enabled(
                                target.is_some() && !self.doc.partial_view,
                                egui::Button::new("Hide/Show"),
                            )
                            .clicked()
                        {
                            if let Some(target) = target {
                                match self.doc.add_hide_show_button(target) {
                                    Ok(id) => self.select_only(id),
                                    Err(error) => self.status = error,
                                }
                            }
                            ui.close();
                        }
                    });
                    if ui
                        .add_enabled(!self.doc.undo.is_empty(), egui::Button::new("Undo"))
                        .clicked()
                    {
                        self.doc.undo();
                        self.image_textures.clear();
                        ui.close();
                    }
                    if ui
                        .add_enabled(!self.doc.redo.is_empty(), egui::Button::new("Redo"))
                        .clicked()
                    {
                        self.doc.redo();
                        self.image_textures.clear();
                        ui.close();
                    }
                    if ui.button("Cut").clicked() {
                        self.cut_selected();
                        ui.close();
                    }
                    if ui.button("Copy").clicked() {
                        self.copy_selected();
                        ui.close();
                    }
                    if ui.button("Paste").clicked() {
                        self.paste();
                        ui.close();
                    }
                    if ui.button("Clear").clicked() {
                        self.clear_selected();
                        ui.close();
                    }
                    if ui.button("Select All").clicked() {
                        self.doc.selected = self
                            .doc
                            .objects
                            .iter()
                            .filter(|o| o.page == self.doc.active_page && o.visible && !o.deleted)
                            .map(|o| o.id)
                            .collect();
                        self.selection_order = self.doc.selected.iter().copied().collect();
                        ui.close();
                    }
                    if ui.button("Select Parents").clicked() {
                        self.select_related(false);
                        ui.close();
                    }
                    if ui.button("Select Children").clicked() {
                        self.select_related(true);
                        ui.close();
                    }
                });
                ui.menu_button("Construct", |ui| {
                    if ui.button("Point on Object").clicked() {
                        self.point_on_object();
                        ui.close();
                    }
                    if ui.button("Segment").clicked() {
                        self.pair(0);
                        ui.close();
                    }
                    if ui.button("Ray").clicked() {
                        self.pair(4);
                        ui.close();
                    }
                    if ui.button("Line").clicked() {
                        self.pair(3);
                        ui.close();
                    }
                    if ui.button("Circle by Center and Point").clicked() {
                        self.pair(1);
                        ui.close();
                    }
                    if ui.button("Circle by Center and Radius Segment").clicked() {
                        self.circle_by_segment();
                        ui.close();
                    }
                    if ui.button("Midpoint").clicked() {
                        self.pair(2);
                        ui.close();
                    }
                    if ui.button("Intersection").clicked() {
                        self.intersection();
                        ui.close();
                    }
                    if ui.button("Angle Bisector").clicked() {
                        self.angle_bisector();
                        ui.close();
                    }
                    if ui.button("Arc through 3 Points").clicked() {
                        self.three_point_arc();
                        ui.close();
                    }
                    if ui.button("Arc on Circle").clicked() {
                        self.arc_on_circle();
                        ui.close();
                    }
                    if ui.button("Polygon Interior").clicked() {
                        self.polygon_interior();
                        ui.close();
                    }
                    if ui.button("Parallel Line").clicked() {
                        self.derived_line(false);
                        ui.close();
                    }
                    if ui.button("Perpendicular Line").clicked() {
                        self.derived_line(true);
                        ui.close();
                    }
                });
                ui.menu_button("Measure", |ui| {
                    if ui.button("Length").clicked() {
                        self.measure_length();
                        ui.close();
                    }
                    if ui.button("Distance").clicked() {
                        self.measure_distance();
                        ui.close();
                    }
                    for (label, kind) in [
                        ("Coordinates", CoordinateMeasureKind::Coordinates),
                        ("Abscissa (x)", CoordinateMeasureKind::Abscissa),
                        ("Ordinate (y)", CoordinateMeasureKind::Ordinate),
                        (
                            "Coordinate Distance",
                            CoordinateMeasureKind::CoordinateDistance,
                        ),
                    ] {
                        if ui.button(label).clicked() {
                            self.measure_coordinate(kind);
                            ui.close();
                        }
                    }
                    if ui.button("Ratio").clicked() {
                        self.measure_ratio();
                        ui.close();
                    }
                    if ui.button("Perimeter").clicked() {
                        self.measure_perimeter();
                        ui.close();
                    }
                    if ui.button("Slope").clicked() {
                        self.measure_slope();
                        ui.close();
                    }
                    if ui.button("Value of Point").clicked() {
                        self.measure_value_equation(false);
                        ui.close();
                    }
                    if ui.button("Equation").clicked() {
                        self.measure_value_equation(true);
                        ui.close();
                    }
                    if ui.button("Circumference").clicked() {
                        self.measure_circle(CircleMeasure::Circumference);
                        ui.close();
                    }
                    if ui.button("Angle").clicked() {
                        self.measure_angle();
                        ui.close();
                    }
                    if ui.button("Radius").clicked() {
                        self.measure_circle(CircleMeasure::Radius);
                        ui.close();
                    }
                    if ui.button("Area").clicked() {
                        self.measure_circle(CircleMeasure::Area);
                        ui.close();
                    }
                    if ui.button("Arc Angle").clicked() {
                        self.measure_arc_angle();
                        ui.close();
                    }
                    if ui.button("Arc Length").clicked() {
                        self.measure_arc_length();
                        ui.close();
                    }
                });
                ui.menu_button("Number", |ui| {
                    if ui.button("New Parameter (1.00)").clicked() {
                        let anchor = if self.doc.objects.len() == 1 {
                            (10, 38)
                        } else {
                            (10, 5)
                        };
                        match self.doc.add_default_parameter(anchor) {
                            Ok(id) => self.status = format!("Created parameter {id}"),
                            Err(error) => self.status = error,
                        }
                        ui.close();
                    }
                    if ui.button("Edit Selected Parameter…").clicked() {
                        let selected = self.doc.selected.iter().copied().find(|&id| {
                            self.doc
                                .objects
                                .get(id - 1)
                                .is_some_and(|object| matches!(object.kind, Kind::Parameter(_)))
                        });
                        if let Some(id) = selected {
                            if let Kind::Parameter(group) = &self.doc.objects[id - 1].kind {
                                self.parameter_value_draft = group.value().to_string();
                                self.parameter_value_dialog = Some(id);
                            }
                        } else {
                            self.status = "Select a parameter".into();
                        }
                        ui.close();
                    }
                    if ui.button("Calculate (2 + 3)").clicked() {
                        match self.doc.add_literal_calculation((10, 38)) {
                            Ok(id) => self.status = format!("Created calculation {id}"),
                            Err(error) => self.status = error,
                        }
                        ui.close();
                    }
                    if ui.button("Tabulate Selected Parameter").clicked() {
                        let result =
                            if self.doc.selected.contains(&1) && self.doc.selected.contains(&2) {
                                self.doc.tabulate_default_pair((10, 71))
                            } else {
                                let source = self.doc.selected.iter().next().copied().unwrap_or(0);
                                self.doc.tabulate_default_parameter(source, (10, 38))
                            };
                        match result {
                            Ok(id) => self.status = format!("Created table {id}"),
                            Err(error) => self.status = error,
                        }
                        ui.close();
                    }
                    if ui.button("Add Table Data").clicked() {
                        let id = self.doc.selected.iter().next().copied().unwrap_or(0);
                        if let Err(error) = self.doc.change_table_rows(id, true) {
                            self.status = error;
                        }
                        ui.close();
                    }
                    if ui.button("Remove Table Data").clicked() {
                        let id = self.doc.selected.iter().next().copied().unwrap_or(0);
                        if let Err(error) = self.doc.change_table_rows(id, false) {
                            self.status = error;
                        }
                        ui.close();
                    }
                    if ui.button("New Function…").clicked() {
                        self.function_dialog = Some(false);
                        ui.close();
                    }
                });
                ui.menu_button("Graph", |ui| {
                    let path = self.doc.selected.iter().copied().find(|&id| {
                        self.doc.objects.get(id - 1).is_some_and(|o| {
                            matches!(
                                o.kind,
                                Kind::Segment(_, _)
                                    | Kind::Ray(_, _)
                                    | Kind::Line(_, _)
                                    | Kind::Circle(_, _)
                                    | Kind::CircleBySegment(_, _)
                                    | Kind::ReflectedCircle(_, _)
                                    | Kind::ScaledCircle(_, _, _)
                                    | Kind::GraphAxis(_, _, 0 | 1 | 0x1_0001)
                            )
                        })
                    });
                    if ui
                        .button(if path.is_some() {
                            "Plot Value on Path"
                        } else {
                            "Plot Value on Axis"
                        })
                        .clicked()
                    {
                        let source = self.doc.selected.iter().copied().find(|&id| {
                            self.doc
                                .objects
                                .get(id - 1)
                                .is_some_and(|o| matches!(o.kind, Kind::Parameter(_)))
                        });
                        self.status = match source {
                            Some(source) => match self.doc.plot_parameter_on_path(source, path) {
                                Ok(id) => format!("Plotted path value {id}"),
                                Err(error) => error,
                            },
                            None => {
                                self.plot_axis_dialog = Some(path);
                                self.plot_axis_draft =
                                    match path.map(|id| &self.doc.objects[id - 1].kind) {
                                        Some(Kind::Segment(_, _)) => "0.5",
                                        Some(
                                            Kind::Circle(_, _)
                                            | Kind::CircleBySegment(_, _)
                                            | Kind::ReflectedCircle(_, _)
                                            | Kind::ScaledCircle(_, _, _),
                                        ) => "0.25",
                                        _ => "1",
                                    }
                                    .into();
                                String::new()
                            }
                        };
                        ui.close();
                    }
                    if ui.button("Plot Points as (x, y)").clicked() {
                        match self.doc.plot_default_parameter(PlotPointsMode::Cartesian) {
                            Ok(id) => self.status = format!("Plotted point {id}"),
                            Err(error) => self.status = error,
                        }
                        ui.close();
                    }
                    if ui.button("Plot Points as (r, θ)").clicked() {
                        match self.doc.plot_default_parameter(PlotPointsMode::Polar) {
                            Ok(id) => self.status = format!("Plotted point {id}"),
                            Err(error) => self.status = error,
                        }
                        ui.close();
                    }
                    if ui.button("Plot Table Data").clicked() {
                        let table = self.doc.selected.iter().next().copied().unwrap_or(0);
                        match self.doc.plot_default_pair_table(table) {
                            Ok(ids) => {
                                self.status = format!("Plotted {} table data point(s)", ids.len())
                            }
                            Err(error) => self.status = error,
                        }
                        ui.close();
                    }
                    if ui.button("Plot New Function…").clicked() {
                        self.function_dialog = Some(true);
                        ui.close();
                    }
                    if ui.button("Define Coordinate System").clicked() {
                        self.define_coordinate_system();
                        ui.close();
                    }
                    ui.add_enabled_ui(self.doc.active_coordinate_system().is_some(), |ui| {
                        ui.menu_button("Grid Form", |ui| {
                            for (form, label) in [
                                (GridForm::Polar, "Polar"),
                                (GridForm::Square, "Square"),
                                (GridForm::Rectangular, "Rectangular"),
                            ] {
                                if ui
                                    .selectable_label(
                                        self.doc.grid_form[self.doc.active_page] == form,
                                        label,
                                    )
                                    .clicked()
                                {
                                    if let Err(error) = self.doc.set_grid_form(form) {
                                        self.status = error;
                                    }
                                    ui.close();
                                }
                            }
                            ui.separator();
                            let trigonometric = self.doc.axis_trigonometric();
                            if ui
                                .selectable_label(trigonometric, "Trigonometric X Axis")
                                .clicked()
                            {
                                if let Err(error) = self.doc.set_axis_trigonometric(!trigonometric)
                                {
                                    self.status = error;
                                }
                                ui.close();
                            }
                        });
                    });
                    let mut visible = self.doc.grid_enabled();
                    if ui
                        .add_enabled(
                            self.doc.active_coordinate_system().is_some(),
                            egui::Checkbox::new(&mut visible, "Show Grid"),
                        )
                        .changed()
                    {
                        self.doc.set_grid_visible(visible);
                        ui.close();
                    }
                    let mut dotted = self.doc.grid_dotted[self.doc.active_page];
                    if ui
                        .add_enabled(
                            self.doc.active_coordinate_system().is_some(),
                            egui::Checkbox::new(&mut dotted, "Dotted Grid"),
                        )
                        .changed()
                    {
                        self.doc.set_grid_dotted(dotted);
                        ui.close();
                    }
                    let mut snap = self.doc.snap_points[self.doc.active_page];
                    if ui
                        .add_enabled(
                            self.doc.active_coordinate_system().is_some(),
                            egui::Checkbox::new(&mut snap, "Snap Points"),
                        )
                        .changed()
                    {
                        self.doc.set_snap_points(snap);
                        ui.close();
                    }
                });
                ui.menu_button("Transform", |ui| {
                    if ui.button("Mark Center").clicked() {
                        self.mark_center_selected();
                        ui.close();
                    }
                    if ui.button("Mark Mirror").clicked() {
                        self.mark_mirror_selected();
                        ui.close();
                    }
                    if ui.button("Translate by Angle and Distance…").clicked() {
                        self.translate_dialog = true;
                        ui.close();
                    }
                    if ui.button("Reflect across Line").clicked() {
                        self.reflect_selected();
                        ui.close();
                    }
                    if ui.button("Dilate by Ratio…").clicked() {
                        self.dilate_dialog = true;
                        ui.close();
                    }
                    if ui.button("Rotate Point…").clicked() {
                        self.rotate_dialog = true;
                        ui.close();
                    }
                });
                ui.menu_button("Display", |ui| {
                    ui.menu_button("Point Style", |ui| {
                        for (label, style) in [
                            ("Dot", PointStyle::Dot),
                            ("Small", PointStyle::Small),
                            ("Medium", PointStyle::Medium),
                            ("Thick", PointStyle::Large),
                        ] {
                            if ui.button(label).clicked() {
                                self.status = match self.doc.set_selected_point_style(style) {
                                    Ok(0) => "Select at least one point".into(),
                                    Ok(count) => {
                                        format!("Changed point style on {count} object(s)")
                                    }
                                    Err(error) => error,
                                };
                                ui.close();
                            }
                        }
                    });
                    ui.menu_button("Line Width", |ui| {
                        for (label, width) in [
                            ("Hairline", LineWidth::Hairline),
                            ("Thin", LineWidth::Thin),
                            ("Medium", LineWidth::Medium),
                            ("Thick", LineWidth::Thick),
                        ] {
                            if ui.button(label).clicked() {
                                self.status = match self.doc.set_selected_line_width(width) {
                                    Ok(0) => "Select at least one line or circle".into(),
                                    Ok(count) => format!("Changed line width on {count} object(s)"),
                                    Err(error) => error,
                                };
                                ui.close();
                            }
                        }
                    });
                    ui.menu_button("Line Pattern", |ui| {
                        for (label, pattern) in [
                            ("Solid", LinePattern::Solid),
                            ("Dashed", LinePattern::Dashed),
                            ("Dotted", LinePattern::Dotted),
                        ] {
                            if ui.button(label).clicked() {
                                self.status = match self.doc.set_selected_line_pattern(pattern) {
                                    Ok(0) => "Select at least one line or circle".into(),
                                    Ok(count) => {
                                        format!("Changed line pattern on {count} object(s)")
                                    }
                                    Err(error) => error,
                                };
                                ui.close();
                            }
                        }
                    });
                    ui.menu_button("Color", |ui| {
                        for (label, color) in [
                            (
                                "Red",
                                RgbColor {
                                    red: 255,
                                    green: 0,
                                    blue: 0,
                                },
                            ),
                            (
                                "Orange",
                                RgbColor {
                                    red: 255,
                                    green: 128,
                                    blue: 0,
                                },
                            ),
                            (
                                "Green",
                                RgbColor {
                                    red: 0,
                                    green: 128,
                                    blue: 0,
                                },
                            ),
                            (
                                "Blue",
                                RgbColor {
                                    red: 0,
                                    green: 0,
                                    blue: 255,
                                },
                            ),
                            (
                                "Navy",
                                RgbColor {
                                    red: 0,
                                    green: 0,
                                    blue: 128,
                                },
                            ),
                            (
                                "Purple",
                                RgbColor {
                                    red: 128,
                                    green: 0,
                                    blue: 128,
                                },
                            ),
                            (
                                "Black",
                                RgbColor {
                                    red: 0,
                                    green: 0,
                                    blue: 0,
                                },
                            ),
                        ] {
                            if ui.button(label).clicked() {
                                self.status = match self.doc.set_selected_color(color) {
                                    Ok(0) => "Select a styled geometry object".into(),
                                    Ok(count) => format!("Changed color on {count} object(s)"),
                                    Err(error) => error,
                                };
                                ui.close();
                            }
                        }
                    });
                    ui.separator();
                    if ui.button("Hide Objects").clicked() {
                        self.doc.set_selected_visibility(false);
                        self.selection_order
                            .retain(|id| self.doc.selected.contains(id));
                        ui.close();
                    }
                    if ui.button("Show All Hidden").clicked() {
                        self.doc.show_all_hidden();
                        ui.close();
                    }
                    let tracing = !self.doc.selected.is_empty()
                        && self
                            .doc
                            .selected
                            .iter()
                            .all(|&id| self.doc.objects.get(id - 1).is_some_and(|o| o.tracing));
                    if ui
                        .button(if tracing {
                            "Stop Tracing"
                        } else {
                            "Trace Objects"
                        })
                        .clicked()
                    {
                        self.status = match self.doc.set_selected_tracing(!tracing) {
                            Ok(0) => "Select at least one point or segment".into(),
                            Ok(count) => format!("Updated tracing on {count} object(s)"),
                            Err(error) => error,
                        };
                        ui.close();
                    }
                    if ui.button("Erase Traces").clicked() {
                        self.trace_marks.clear();
                        self.status = "Erased traces".into();
                        ui.close();
                    }
                    if ui.button("Fit Sketch").clicked() {
                        self.fit();
                        ui.close();
                    }
                });
            });
        });
        egui::Panel::left("tools").min_size(110.).show(root, |ui| {
            ui.heading("Tools");
            if self.doc.page_count > 1 {
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(self.doc.active_page > 0, egui::Button::new("‹"))
                        .clicked()
                    {
                        self.doc.active_page -= 1;
                        self.doc.selected.clear();
                        self.selection_order.clear();
                        self.fit();
                    }
                    ui.label(format!(
                        "{}/{}",
                        self.doc.active_page + 1,
                        self.doc.page_count
                    ));
                    if ui
                        .add_enabled(
                            self.doc.active_page + 1 < self.doc.page_count,
                            egui::Button::new("›"),
                        )
                        .clicked()
                    {
                        self.doc.active_page += 1;
                        self.doc.selected.clear();
                        self.selection_order.clear();
                        self.fit();
                    }
                });
                ui.separator();
            }
            for (tool, name) in [
                (Tool::Select, "Select"),
                (Tool::Point, "Point"),
                (Tool::Segment, "Segment"),
                (Tool::Circle, "Circle"),
                (Tool::Text, "Text"),
            ] {
                if ui.selectable_label(self.tool == tool, name).clicked() {
                    self.tool = tool;
                    self.pending = None;
                }
            }
            if self.tool == Tool::Text {
                ui.label("New text");
                ui.text_edit_multiline(&mut self.text_draft);
                ui.label("Click the sketch to place it.");
            }
            if self.marked_center.is_some() || self.marked_mirror.is_some() {
                ui.separator();
                if let Some(id) = self.marked_center {
                    ui.label(format!("Marked center: {id}"));
                }
                if let Some(id) = self.marked_mirror {
                    ui.label(format!("Marked mirror: {id}"));
                }
            }
            if let Some(id) = self.doc.selected.iter().next().copied() {
                if self.inspected != Some(id) {
                    self.inspected = Some(id);
                    self.edit_buffer = match &self.doc.objects[id - 1].kind {
                        Kind::PlainText { text, .. } => text.clone(),
                        _ => self.doc.objects[id - 1]
                            .label
                            .as_ref()
                            .and_then(|label| label.name_utf8().ok())
                            .map(str::to_owned)
                            .unwrap_or_else(|| format!("P{id}")),
                    };
                }
                if matches!(self.doc.objects[id - 1].kind, Kind::PlainText { .. }) {
                    ui.separator();
                    ui.label("Edit selected text");
                    ui.text_edit_multiline(&mut self.edit_buffer);
                    if ui.button("Apply text").clicked() {
                        if let Err(error) = self.doc.edit_text(id, &self.edit_buffer) {
                            self.status = error;
                        }
                    }
                } else if self.doc.position(id).is_some() {
                    ui.separator();
                    ui.label("Point label");
                    ui.text_edit_singleline(&mut self.edit_buffer);
                    let visible = self.doc.objects[id - 1]
                        .label
                        .as_ref()
                        .is_some_and(|label| label.visible());
                    if ui
                        .button(if visible { "Hide label" } else { "Show label" })
                        .clicked()
                    {
                        if let Err(error) =
                            self.doc.set_point_label(id, &self.edit_buffer, !visible)
                        {
                            self.status = error;
                        }
                    }
                    if ui.button("Rename label").clicked() {
                        if let Err(error) = self.doc.set_point_label(id, &self.edit_buffer, visible)
                        {
                            self.status = error;
                        }
                    }
                }
            } else {
                self.inspected = None;
            }
            ui.separator();
            ui.label(format!("{} objects", self.doc.objects.len()));
            if self.doc.partial_view {
                ui.colored_label(egui::Color32::DARK_RED, "Partial view: unsupported objects");
            }
            if self.doc.dirty {
                ui.label("Unsaved changes");
            }
        });
        egui::Panel::bottom("status").show(root, |ui| {
            ui.label(if self.status.is_empty() {
                "Drag points. Scroll to zoom. Select two points to construct."
            } else {
                &self.status
            });
        });
        let mut rect = egui::Rect::NOTHING;
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(root, |ui| {
                let (r, response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
                rect = r;
                let vp = Viewport {
                    rect: r,
                    scale: self.scale,
                    center: self.center,
                };
                if response.hovered() {
                    let dy = ui.input(|i| i.smooth_scroll_delta.y);
                    if dy != 0. {
                        self.scale = (self.scale * (dy * 0.001).exp()).clamp(0.05, 20.);
                    }
                }
                if response.drag_started() && self.tool == Tool::Select {
                    if let Some(p) = ui.input(|i| i.pointer.press_origin()) {
                        if let Some(id) = self.hit(&vp, p) {
                            if matches!(
                                self.doc.objects[id - 1].kind,
                                Kind::Free(_)
                                    | Kind::GraphVerticalUnit(_, _)
                                    | Kind::Image { .. }
                                    | Kind::PlainText { .. }
                                    | Kind::Parameter(_)
                                    | Kind::Calculation(_)
                                    | Kind::ValueTable(_, _)
                                    | Kind::ValueTablePair(_, _, _)
                                    | Kind::PointOnSegment(_, _)
                                    | Kind::PointOnCircle(_, _, _)
                                    | Kind::PointOnArc(_, _)
                            ) {
                                self.drag_offset = match &self.doc.objects[id - 1].kind {
                                    Kind::PlainText { rect, .. } => {
                                        p - vp.screen(Point {
                                            x: rect.left.into(),
                                            y: rect.top.into(),
                                        })
                                    }
                                    Kind::Parameter(_)
                                    | Kind::Calculation(_)
                                    | Kind::ValueTable(_, _)
                                    | Kind::ValueTablePair(_, _, _) => {
                                        let anchor = sketchpad_rs::render::number_lines(
                                            &self.doc.objects[id - 1].kind,
                                        )
                                        .unwrap()
                                        .0;
                                        p - (vp.rect.min
                                            + egui::vec2(anchor.0 as f32, anchor.1 as f32))
                                    }
                                    Kind::Image { transform, .. } => {
                                        p - vp.screen(Point {
                                            x: transform[2],
                                            y: transform[5],
                                        })
                                    }
                                    _ => egui::Vec2::ZERO,
                                };
                                self.doc.checkpoint();
                                self.dragging = Some(id);
                                self.capture_traces();
                            }
                        }
                    }
                }
                if response.dragged() {
                    if let (Some(id), Some(p)) = (self.dragging, response.interact_pointer_pos()) {
                        if matches!(self.doc.objects[id - 1].kind, Kind::Image { .. }) {
                            if let Err(error) =
                                self.doc.drag_image(id, vp.world(p - self.drag_offset))
                            {
                                self.status = error;
                            }
                        } else if matches!(self.doc.objects[id - 1].kind, Kind::PlainText { .. }) {
                            if let Err(error) =
                                self.doc.drag_text(id, vp.world(p - self.drag_offset))
                            {
                                self.status = error;
                            }
                        } else if matches!(
                            self.doc.objects[id - 1].kind,
                            Kind::Parameter(_)
                                | Kind::Calculation(_)
                                | Kind::ValueTable(_, _)
                                | Kind::ValueTablePair(_, _, _)
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
                }
                if response.drag_stopped() {
                    self.dragging = None;
                }
                if response.clicked() {
                    if let Some(p) = response.interact_pointer_pos() {
                        let multi = ui.input(|i| i.modifiers.shift);
                        self.click(&vp, p, multi);
                    }
                }
                for mark in &self.trace_marks {
                    match *mark {
                        TraceMark::Point(at, color) => {
                            let at = vp.screen(at);
                            if r.contains(at) {
                                ui.painter().circle_filled(
                                    at,
                                    2.,
                                    egui::Color32::from_rgb(color.red, color.green, color.blue),
                                );
                            }
                        }
                        TraceMark::Segment(a, b, color) => {
                            ui.painter().line_segment(
                                [vp.screen(a), vp.screen(b)],
                                egui::Stroke::new(
                                    1.,
                                    egui::Color32::from_rgb(color.red, color.green, color.blue),
                                ),
                            );
                        }
                    }
                }
                for object in self.doc.objects.iter().filter(|object| {
                    object.page == self.doc.active_page && object.visible && !object.deleted
                }) {
                    if let Kind::Image { png, transform } = &object.kind {
                        if !self.image_textures.contains_key(&object.id) {
                            if let Ok(decoded) =
                                image::load_from_memory_with_format(png, image::ImageFormat::Png)
                            {
                                let rgba = decoded.to_rgba8();
                                let color = egui::ColorImage::from_rgba_unmultiplied(
                                    [rgba.width() as usize, rgba.height() as usize],
                                    rgba.as_raw(),
                                );
                                let texture = ui.ctx().load_texture(
                                    format!("picture-{}", object.id),
                                    color,
                                    egui::TextureOptions::LINEAR,
                                );
                                self.image_textures.insert(object.id, texture);
                            }
                        }
                        if let Some(texture) = self.image_textures.get(&object.id)
                            && let Some(corners) =
                                sketchpad_rs::render::picture_screen_quad(&vp, png, *transform)
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
                    if let Kind::PlainText { text, rect } = &object.kind {
                        let at = vp.screen(Point {
                            x: rect.left.into(),
                            y: rect.top.into(),
                        });
                        for (line, content) in text.split('\n').enumerate() {
                            ui.painter().text(
                                at + egui::vec2(0., line as f32 * 19.),
                                egui::Align2::LEFT_TOP,
                                content,
                                egui::FontId::proportional(16.),
                                egui::Color32::from_gray(35),
                            );
                        }
                    }
                    if let Some((_, lines)) = sketchpad_rs::render::number_lines(&object.kind)
                        && let Some(number_rect) =
                            sketchpad_rs::render::number_screen_rect(&vp, &object.kind)
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
                        && let Some(point) = self.doc.position(object.id)
                    {
                        let at = vp.screen(point);
                        let offset = label.offset();
                        ui.painter().text(
                            at + egui::vec2(offset.0 as f32, offset.1 as f32),
                            egui::Align2::LEFT_TOP,
                            name,
                            egui::FontId::proportional(13.),
                            egui::Color32::from_gray(35),
                        );
                    }
                    if let Kind::HideShowButton { label, anchor, .. } = &object.kind {
                        let box_rect =
                            sketchpad_rs::render::action_button_rect(&vp, *anchor, label);
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
                for (at, label) in sketchpad_rs::render::trigonometric_axis_labels(&self.doc, &vp) {
                    ui.painter().text(
                        at,
                        egui::Align2::CENTER_TOP,
                        label,
                        egui::FontId::proportional(11.),
                        egui::Color32::from_rgb(36, 48, 61),
                    );
                }
                let mut row = 0;
                for object in self.doc.objects.iter().filter(|object| {
                    object.page == self.doc.active_page && object.visible && !object.deleted
                }) {
                    if let Some(length) = self.doc.segment_length_cm(object.id) {
                        ui.painter().text(
                            r.min + egui::vec2(12., 12. + row as f32 * 20.),
                            egui::Align2::LEFT_TOP,
                            format!("Length = {length:.2} cm"),
                            egui::FontId::proportional(16.),
                            egui::Color32::from_gray(35),
                        );
                        row += 1;
                    }
                    if let Some(distance) = self.doc.point_distance_cm(object.id) {
                        ui.painter().text(
                            r.min + egui::vec2(12., 12. + row as f32 * 20.),
                            egui::Align2::LEFT_TOP,
                            format!("Distance = {distance:.2} cm"),
                            egui::FontId::proportional(16.),
                            egui::Color32::from_gray(35),
                        );
                        row += 1;
                    }
                    if let Some(ratio) = self.doc.ratio_value(object.id) {
                        ui.painter().text(
                            r.min + egui::vec2(12., 12. + row as f32 * 20.),
                            egui::Align2::LEFT_TOP,
                            format!("Ratio = {ratio:.2}"),
                            egui::FontId::proportional(16.),
                            egui::Color32::from_gray(35),
                        );
                        row += 1;
                    }
                    if let Some(perimeter) = self.doc.perimeter_cm(object.id) {
                        ui.painter().text(
                            r.min + egui::vec2(12., 12. + row as f32 * 20.),
                            egui::Align2::LEFT_TOP,
                            format!("Perimeter = {perimeter:.2} cm"),
                            egui::FontId::proportional(16.),
                            egui::Color32::from_gray(35),
                        );
                        row += 1;
                    }
                    if let Some(circumference) = self.doc.circle_circumference_cm(object.id) {
                        ui.painter().text(
                            r.min + egui::vec2(12., 12. + row as f32 * 20.),
                            egui::Align2::LEFT_TOP,
                            format!("Circumference = {circumference:.2} cm"),
                            egui::FontId::proportional(16.),
                            egui::Color32::from_gray(35),
                        );
                        row += 1;
                    }
                    if let Some(radius) = self.doc.circle_radius_cm(object.id) {
                        ui.painter().text(
                            r.min + egui::vec2(12., 12. + row as f32 * 20.),
                            egui::Align2::LEFT_TOP,
                            format!("Radius = {radius:.2} cm"),
                            egui::FontId::proportional(16.),
                            egui::Color32::from_gray(35),
                        );
                        row += 1;
                    }
                    if let Some(area) = self.doc.circle_area_cm2(object.id) {
                        ui.painter().text(
                            r.min + egui::vec2(12., 12. + row as f32 * 20.),
                            egui::Align2::LEFT_TOP,
                            format!("Area = {area:.2} cm²"),
                            egui::FontId::proportional(16.),
                            egui::Color32::from_gray(35),
                        );
                        row += 1;
                    }
                    if let Some(angle) = self.doc.arc_angle_degrees(object.id) {
                        ui.painter().text(
                            r.min + egui::vec2(12., 12. + row as f32 * 20.),
                            egui::Align2::LEFT_TOP,
                            format!("Arc Angle = {angle:.2}°"),
                            egui::FontId::proportional(16.),
                            egui::Color32::from_gray(35),
                        );
                        row += 1;
                    }
                    if let Some(length) = self.doc.arc_length_cm(object.id) {
                        ui.painter().text(
                            r.min + egui::vec2(12., 12. + row as f32 * 20.),
                            egui::Align2::LEFT_TOP,
                            format!("Arc Length = {length:.2} cm"),
                            egui::FontId::proportional(16.),
                            egui::Color32::from_gray(35),
                        );
                        row += 1;
                    }
                    if let Some(angle) = self.doc.angle_degrees(object.id) {
                        ui.painter().text(
                            r.min + egui::vec2(12., 12. + row as f32 * 20.),
                            egui::Align2::LEFT_TOP,
                            format!("Angle = {angle:.2}°"),
                            egui::FontId::proportional(16.),
                            egui::Color32::from_gray(35),
                        );
                        row += 1;
                    }
                    if matches!(object.kind, Kind::SlopeMeasurement(_, _)) {
                        let value = self
                            .doc
                            .slope_value(object.id)
                            .map(|value| format!("{value:.2}"))
                            .unwrap_or_else(|| "undefined".into());
                        ui.painter().text(
                            r.min + egui::vec2(12., 12. + row as f32 * 20.),
                            egui::Align2::LEFT_TOP,
                            format!("Slope = {value}"),
                            egui::FontId::proportional(16.),
                            egui::Color32::from_gray(35),
                        );
                        row += 1;
                    }
                    if let Some(value) = self.doc.coordinate_measure_text(object.id) {
                        ui.painter().text(
                            r.min + egui::vec2(12., 12. + row as f32 * 20.),
                            egui::Align2::LEFT_TOP,
                            value,
                            egui::FontId::proportional(16.),
                            egui::Color32::from_gray(35),
                        );
                        row += 1;
                    }
                    if let Some(value) = self.doc.value_equation_text(object.id) {
                        ui.painter().text(
                            r.min + egui::vec2(12., 12. + row as f32 * 20.),
                            egui::Align2::LEFT_TOP,
                            value,
                            egui::FontId::proportional(16.),
                            egui::Color32::from_gray(35),
                        );
                        row += 1;
                    }
                    if let Kind::FunctionDefinition(expression) = object.kind {
                        ui.painter().text(
                            r.min + egui::vec2(12., 12. + row as f32 * 20.),
                            egui::Align2::LEFT_TOP,
                            expression.display(),
                            egui::FontId::proportional(16.),
                            egui::Color32::from_gray(35),
                        );
                        row += 1;
                    }
                }
            });
        if self.pending_action.is_some() {
            enum Decision {
                Save,
                Discard,
                Cancel,
            }
            let mut decision = None;
            egui::Modal::new(egui::Id::new("unsaved_changes")).show(root.ctx(), |ui| {
                ui.heading("Unsaved changes");
                ui.label("Save this sketch before continuing?");
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        decision = Some(Decision::Save);
                    }
                    if ui.button("Discard").clicked() {
                        decision = Some(Decision::Discard);
                    }
                    if ui.button("Cancel").clicked() {
                        decision = Some(Decision::Cancel);
                    }
                });
            });
            match decision {
                Some(Decision::Save) if self.save(false) => self.finish_pending_action(),
                Some(Decision::Discard) => self.finish_pending_action(),
                Some(Decision::Cancel) => self.pending_action = None,
                _ => {}
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
                ui.label("-99.99 to 99.99, up to two decimal places");
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
        if let Some(plot) = self.function_dialog {
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("new_function")).show(root.ctx(), |ui| {
                ui.heading(if plot {
                    "Plot new function"
                } else {
                    "New function"
                });
                ui.label("f(x) =");
                ui.text_edit_singleline(&mut self.function_draft);
                ui.label("Supported: x, x^2");
                ui.horizontal(|ui| {
                    apply = ui.button(if plot { "Plot" } else { "Create" }).clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.function_dialog = None;
            }
            if apply {
                match SimpleFunction::parse(&self.function_draft).and_then(|expr| {
                    if plot {
                        self.doc.plot_new_function(expr, self.center)
                    } else {
                        self.doc.add_function_definition(expr)
                    }
                }) {
                    Ok(id) => {
                        self.function_dialog = None;
                        self.select_only(id);
                        self.status = format!("Created function object {id}");
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if let Some(path) = self.plot_axis_dialog {
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("plot_axis_value")).show(root.ctx(), |ui| {
                ui.heading("Plot Value on Path");
                ui.horizontal(|ui| {
                    ui.label("Value");
                    ui.text_edit_singleline(&mut self.plot_axis_draft);
                });
                ui.label(match path {
                    Some(id) if matches!(self.doc.objects[id - 1].kind, Kind::Segment(_, _)) => {
                        format!("Segment object {id} (0 to 1)")
                    }
                    Some(id) if matches!(self.doc.objects[id - 1].kind, Kind::Ray(_, _)) => {
                        format!("Ray object {id} (0 or greater)")
                    }
                    Some(id) if matches!(self.doc.objects[id - 1].kind, Kind::Line(_, _)) => {
                        format!("Line object {id}")
                    }
                    Some(id)
                        if matches!(
                            self.doc.objects[id - 1].kind,
                            Kind::Circle(_, _)
                                | Kind::CircleBySegment(_, _)
                                | Kind::ReflectedCircle(_, _)
                                | Kind::ScaledCircle(_, _, _)
                        ) =>
                    {
                        format!("Circle object {id} (counterclockwise turns)")
                    }
                    Some(id) => format!("Axis object {id}"),
                    None => "Active X axis".into(),
                });
                ui.horizontal(|ui| {
                    apply = ui.button("Plot").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.plot_axis_dialog = None;
            }
            if apply {
                let result = self
                    .plot_axis_draft
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| "Enter a numeric value".to_string())
                    .and_then(|value| {
                        if !value.is_finite() {
                            return Err("Enter a finite value".into());
                        }
                        if path.is_none() && self.doc.active_coordinate_system().is_none() {
                            self.doc.add_default_coordinate_system(self.center)?;
                        }
                        self.doc.plot_literal_on_path(value, path)
                    });
                match result {
                    Ok(id) => {
                        self.plot_axis_dialog = None;
                        self.select_only(id);
                        self.status = format!("Plotted path value {id}");
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if self.translate_dialog {
            let mut angle = self.translation_angle;
            let mut distance = self.translation_distance;
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("polar_translation")).show(root.ctx(), |ui| {
                ui.heading("Translate point");
                ui.horizontal(|ui| {
                    ui.label("Angle (degrees)");
                    ui.add(egui::DragValue::new(&mut angle).speed(1.));
                });
                ui.horizontal(|ui| {
                    ui.label("Distance (cm)");
                    ui.add(egui::DragValue::new(&mut distance).speed(0.1));
                });
                ui.horizontal(|ui| {
                    apply = ui.button("Translate").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            self.translation_angle = angle;
            self.translation_distance = distance;
            if apply {
                self.translate_dialog = false;
                self.translate_selected();
            } else if cancel {
                self.translate_dialog = false;
            }
        }
        if self.dilate_dialog {
            let mut factor = self.dilation_factor;
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("dilation_ratio")).show(root.ctx(), |ui| {
                ui.heading("Dilate object");
                ui.horizontal(|ui| {
                    ui.label("Ratio");
                    ui.add(egui::DragValue::new(&mut factor).speed(0.05));
                });
                ui.horizontal(|ui| {
                    apply = ui.button("Dilate").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            self.dilation_factor = factor;
            if apply {
                self.dilate_dialog = false;
                self.dilate_selected();
            } else if cancel {
                self.dilate_dialog = false;
            }
        }
        if self.rotate_dialog {
            let mut angle = self.rotation_angle;
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("point_rotation")).show(root.ctx(), |ui| {
                ui.heading("Rotate point");
                ui.horizontal(|ui| {
                    ui.label("Angle (degrees)");
                    ui.add(egui::DragValue::new(&mut angle).speed(1.));
                });
                ui.horizontal(|ui| {
                    apply = ui.button("Rotate").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            self.rotation_angle = angle;
            if apply {
                self.rotate_dialog = false;
                self.rotate_selected();
            } else if cancel {
                self.rotate_dialog = false;
            }
        }
        let viewport = Viewport {
            rect,
            scale: self.scale,
            center: self.center,
        };
        if let Some(action) = self.pending_output.take() {
            self.output_page(&viewport, action);
        }
        viewport
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traced_point_and_segment_capture_motion_until_erased() {
        let mut editor = Editor::new(Document::new());
        let a = editor.doc.add(Kind::Free(Point { x: 20., y: 30. }));
        let b = editor.doc.add(Kind::Free(Point { x: 80., y: 30. }));
        let segment = editor.doc.add(Kind::Segment(a, b));
        editor.doc.selected.extend([a, segment]);
        assert_eq!(editor.doc.set_selected_tracing(true).unwrap(), 2);
        editor.capture_traces();
        assert_eq!(editor.trace_marks.len(), 2);
        assert!(editor.doc.move_point(a, Point { x: 35., y: 45. }));
        editor.capture_traces();
        assert_eq!(editor.trace_marks.len(), 4);
        assert!(matches!(
            editor.trace_marks[2],
            TraceMark::Point(Point { x: 35., y: 45. }, _)
        ));
        assert!(matches!(
            editor.trace_marks[3],
            TraceMark::Segment(Point { x: 35., y: 45. }, _, _)
        ));
        editor.trace_marks.clear();
        assert!(editor.trace_marks.is_empty());
        editor.capture_traces();
        assert_eq!(editor.trace_marks.len(), 2);
        editor.trace_marks.clear();
        editor.doc.set_selected_tracing(false).unwrap();
        editor.capture_traces();
        assert!(editor.trace_marks.is_empty());
    }

    #[test]
    fn marked_center_and_mirror_drive_transforms_without_changing_source_records() {
        let mut editor = Editor::new(Document::new());
        let center = editor.doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let source = editor.doc.add(Kind::Free(Point { x: 30., y: 0. }));
        let axis_point = editor.doc.add(Kind::Free(Point { x: 0., y: 40. }));
        let mirror = editor.doc.add(Kind::Line(center, axis_point));
        let before = editor.doc.encode().unwrap();

        editor.select_only(center);
        editor.mark_center_selected();
        editor.select_only(mirror);
        editor.mark_mirror_selected();
        assert_eq!(editor.doc.encode().unwrap(), before);

        editor.select_only(source);
        editor.rotation_angle = 90.;
        editor.rotate_selected();
        assert!(
            matches!(editor.doc.objects.last().unwrap().kind, Kind::RotatedPoint(s, c, 90.) if s == source && c == center)
        );

        editor.select_only(source);
        editor.dilation_factor = 2.;
        editor.dilate_selected();
        assert!(
            matches!(editor.doc.objects.last().unwrap().kind, Kind::ScaledPoint(s, c, 2.) if s == source && c == center)
        );

        editor.select_only(source);
        editor.reflect_selected();
        assert!(
            matches!(editor.doc.objects.last().unwrap().kind, Kind::ReflectedPoint(s, m) if s == source && m == mirror)
        );
        assert!(Document::from_bytes(&editor.doc.encode().unwrap(), None).is_ok());
    }
}

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    let doc = match args.get(1).map(PathBuf::from) {
        Some(p) => match Document::open(&p) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Could not open {}: {e}", p.display());
                Document::new()
            }
        },
        None => Document::new(),
    };
    let mut editor = Editor::new(doc);
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let window: Arc<Window> = Arc::new(
        event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("Sketchpad RS")
                    .with_inner_size(winit::dpi::LogicalSize::new(1100., 750.)),
            )
            .unwrap(),
    );
    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(window.clone()).expect("surface");
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }))
    .expect("graphics adapter");
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("graphics device");
    let mut size = window.inner_size();
    let mut config = surface
        .get_default_config(&adapter, size.width.max(1), size.height.max(1))
        .expect("surface config");
    if let Some(f) = surface
        .get_capabilities(&adapter)
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
    {
        config.format = f;
    }
    surface.configure(&device, &config);
    let ctx = egui::Context::default();
    let mut egui_state = egui_winit::State::new(
        ctx.clone(),
        ctx.viewport_id(),
        window.as_ref(),
        Some(window.scale_factor() as f32),
        None,
        None,
    );
    let mut egui_renderer = egui_wgpu::Renderer::new(
        &device,
        config.format,
        egui_wgpu::RendererOptions::default(),
    );
    let geometry = GeometryRenderer::new(&device, config.format);
    window.request_redraw();
    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
                let response = egui_state.on_window_event(&window, &event);
                match event {
                    WindowEvent::CloseRequested => {
                        if editor.doc.dirty {
                            editor.pending_action = Some(PendingAction::Quit);
                            window.request_redraw();
                        } else {
                            elwt.exit();
                        }
                    }
                    WindowEvent::Resized(s) => {
                        size = s;
                        if s.width > 0 && s.height > 0 {
                            config.width = s.width;
                            config.height = s.height;
                            surface.configure(&device, &config);
                        }
                        window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        if size.width == 0 || size.height == 0 {
                            return;
                        }
                        let input = egui_state.take_egui_input(&window);
                        let mut viewport = None;
                        let mut output = ctx.run_ui(input, |ui| viewport = Some(editor.ui(ui)));
                        if editor.exit_requested {
                            elwt.exit();
                            return;
                        }
                        egui_state.handle_platform_output(&window, output.platform_output);
                        let jobs = ctx.tessellate(output.shapes, output.pixels_per_point);
                        let free_textures = output.textures_delta.free.clone();
                        for (id, deltas) in &output.textures_delta.set {
                            for delta in deltas {
                                egui_renderer.update_texture(&device, &queue, *id, delta);
                            }
                        }
                        output.textures_delta.clear();
                        let frame = match surface.get_current_texture() {
                            wgpu::CurrentSurfaceTexture::Success(f)
                            | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
                            wgpu::CurrentSurfaceTexture::Lost
                            | wgpu::CurrentSurfaceTexture::Outdated => {
                                surface.configure(&device, &config);
                                return;
                            }
                            _ => return,
                        };
                        let view = frame.texture.create_view(&Default::default());
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("frame"),
                            });
                        let screen = egui_wgpu::ScreenDescriptor {
                            size_in_pixels: [size.width, size.height],
                            pixels_per_point: output.pixels_per_point,
                        };
                        geometry.render(
                            &device,
                            &mut encoder,
                            &view,
                            &editor.doc,
                            &viewport.unwrap(),
                            [size.width, size.height],
                            output.pixels_per_point,
                        );
                        let mut extra = egui_renderer.update_buffers(
                            &device,
                            &queue,
                            &mut encoder,
                            &jobs,
                            &screen,
                        );
                        {
                            let mut pass = encoder
                                .begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("interface"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &view,
                                        depth_slice: None,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Load,
                                            store: wgpu::StoreOp::Store,
                                        },
                                    })],
                                    depth_stencil_attachment: None,
                                    occlusion_query_set: None,
                                    timestamp_writes: None,
                                    multiview_mask: None,
                                })
                                .forget_lifetime();
                            egui_renderer.render(&mut pass, &jobs, &screen);
                        }
                        extra.push(encoder.finish());
                        queue.submit(extra);
                        queue.present(frame);
                        for id in &free_textures {
                            egui_renderer.free_texture(id);
                        }
                    }
                    _ => {
                        let _ = response;
                        window.request_redraw();
                    }
                }
            }
            _ => {}
        })
        .expect("window loop");
}
