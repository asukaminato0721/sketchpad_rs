use sketchpad_rs::action_records::{
    AnimationDirection, AnimationSettings, MovementSpeed, PresentationMode, ScrollPlacement,
};
use sketchpad_rs::coordinate_measure_records::CoordinateMeasureKind;
use sketchpad_rs::document::{Clipboard, Document, Kind, LineMode, Point, TextStyleCommand};
use sketchpad_rs::function_records::SimpleFunction;
use sketchpad_rs::graph::{GraphPoint, GridForm};
use sketchpad_rs::plot_points_records::PlotPointsMode;
use sketchpad_rs::render::{GeometryRenderer, TraceMark, Viewport, svg_page_with_button_offset};
use sketchpad_rs::style_records::{LinePattern, LineWidth, PointStyle, RgbColor, VisualStyle};
use sketchpad_rs::value_equation_records::ValueEquationKind;
use std::{
    collections::HashMap,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TranslationMode {
    FixedPolar,
    FixedRectangular,
    MarkedDistance,
    RectangularMarked,
    RectangularHorizontalFixed,
    RectangularVerticalFixed,
    MarkedVector,
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

fn system_font_data(family: &str) -> Option<egui::FontData> {
    let lookup = family.strip_prefix('@').unwrap_or(family);
    let output = Command::new("fc-match")
        .arg("-f")
        .arg("%{file}\n%{index}\n")
        .arg(lookup)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let result = std::str::from_utf8(&output.stdout).ok()?;
    let mut lines = result.lines();
    let path = lines.next()?;
    let index = lines.next()?.parse::<u32>().ok()? & 0xffff;
    if path.is_empty() {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let mut data = egui::FontData::from_owned(bytes);
    data.index = index;
    Some(data)
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
struct AnimationTrack {
    id: usize,
    phase: f64,
    direction: f64,
    free_velocity: Option<Point>,
    free_turn_remaining: f64,
    random_state: u64,
    button_settings: Option<AnimationSettings>,
    start_phase: f64,
    distance_travelled: f64,
}
#[derive(Clone)]
struct AnimationButtonDraft {
    targets: Vec<usize>,
    settings: Vec<AnimationSettings>,
    selected: usize,
}
#[derive(Clone, Copy)]
struct MovementTrack {
    button: usize,
    source: usize,
    destination: usize,
    initial_destination: Option<Point>,
    speed: MovementSpeed,
}
#[derive(Clone, Copy)]
struct MovementButtonDraft {
    source: usize,
    destination: usize,
    speed: MovementSpeed,
    follow: bool,
}
#[derive(Clone, Copy)]
struct ScrollButtonDraft {
    target: usize,
    placement: ScrollPlacement,
}
#[derive(Clone)]
struct PresentationButtonDraft {
    targets: Vec<usize>,
    mode: PresentationMode,
}
#[derive(Clone)]
struct LinkButtonDraft {
    use_page: bool,
    target_page: usize,
    target_button_ordinal: Option<usize>,
    url: String,
}
#[derive(Clone)]
struct PresentationTrack {
    id: usize,
    targets: Vec<usize>,
    mode: PresentationMode,
    next: usize,
    active: Vec<usize>,
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
    marked_vector: Option<(usize, usize)>,
    marked_distance: Option<usize>,
    marked_rectangular_distances: Option<(usize, usize)>,
    marked_angle: Option<[usize; 3]>,
    marked_ratio: Option<[usize; 3]>,
    trace_marks: Vec<TraceMark>,
    animations: Vec<AnimationTrack>,
    movements: Vec<MovementTrack>,
    presentations: Vec<PresentationTrack>,
    animation_speed: f64,
    animation_paused: bool,
    animation_last_tick: Instant,
    animation_bounds: Option<(Point, Point)>,
    viewport_rect: Option<egui::Rect>,
    show_motion_controller: bool,
    animation_button_dialog: Option<AnimationButtonDraft>,
    movement_button_dialog: Option<MovementButtonDraft>,
    scroll_button_dialog: Option<ScrollButtonDraft>,
    presentation_button_dialog: Option<PresentationButtonDraft>,
    link_dialog: Option<LinkButtonDraft>,
    dragging: Option<usize>,
    drag_offset: egui::Vec2,
    text_draft: String,
    font_definitions: egui::FontDefinitions,
    font_definitions_applied: bool,
    loaded_fonts: HashMap<String, bool>,
    text_font_dialog: bool,
    text_font_input: String,
    text_selected_range: Option<(usize, usize, usize)>,
    inspected: Option<usize>,
    edit_buffer: String,
    scale: f32,
    center: Point,
    button_view_center: Point,
    status: String,
    pending_action: Option<PendingAction>,
    pending_output: Option<OutputAction>,
    translate_dialog: bool,
    translation_angle: f64,
    translation_distance: f64,
    translation_horizontal: f64,
    translation_vertical: f64,
    translation_mode: TranslationMode,
    dilate_dialog: bool,
    dilation_factor: f64,
    dilation_use_marked: bool,
    rotate_dialog: bool,
    rotation_angle: f64,
    rotation_use_marked: bool,
    function_dialog: Option<bool>,
    function_edit_id: Option<usize>,
    function_draft: String,
    function_angle_dialog: Option<(SimpleFunction, bool, Option<usize>)>,
    derivative_angle_dialog: Option<usize>,
    parametric_dialog: bool,
    parametric_x: usize,
    parametric_y: usize,
    parametric_mode: PlotPointsMode,
    parametric_min_draft: String,
    parametric_max_draft: String,
    plot_range_dialog: Option<usize>,
    plot_min_draft: String,
    plot_max_draft: String,
    plot_samples_draft: String,
    parameter_value_dialog: Option<usize>,
    parameter_value_draft: String,
    named_parameter_dialog: bool,
    parameter_name_draft: String,
    plot_axis_dialog: Option<Option<usize>>,
    plot_axis_draft: String,
    plot_points_dialog: Option<PlotPointsMode>,
    plot_points_first_draft: String,
    plot_points_second_draft: String,
    plot_table_dialog: Option<usize>,
    plot_table_mode: PlotPointsMode,
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
            marked_vector: None,
            marked_distance: None,
            marked_rectangular_distances: None,
            marked_angle: None,
            marked_ratio: None,
            trace_marks: Vec::new(),
            animations: Vec::new(),
            movements: Vec::new(),
            presentations: Vec::new(),
            animation_speed: 1.,
            animation_paused: false,
            animation_last_tick: Instant::now(),
            animation_bounds: None,
            viewport_rect: None,
            show_motion_controller: false,
            animation_button_dialog: None,
            movement_button_dialog: None,
            scroll_button_dialog: None,
            presentation_button_dialog: None,
            link_dialog: None,
            dragging: None,
            drag_offset: egui::Vec2::ZERO,
            text_draft: "Text".into(),
            font_definitions: egui::FontDefinitions::default(),
            font_definitions_applied: false,
            loaded_fonts: HashMap::new(),
            text_font_dialog: false,
            text_font_input: "Tahoma".into(),
            text_selected_range: None,
            inspected: None,
            edit_buffer: String::new(),
            scale: 1.,
            center: Point { x: 400., y: 300. },
            button_view_center: Point { x: 400., y: 300. },
            status: String::new(),
            pending_action: None,
            pending_output: None,
            translate_dialog: false,
            translation_angle: 90.,
            translation_distance: 1.,
            translation_horizontal: 1.,
            translation_vertical: 1.,
            translation_mode: TranslationMode::FixedPolar,
            dilate_dialog: false,
            dilation_factor: 0.5,
            dilation_use_marked: false,
            rotate_dialog: false,
            rotation_angle: 60.,
            rotation_use_marked: false,
            function_dialog: None,
            function_edit_id: None,
            function_draft: "x".into(),
            function_angle_dialog: None,
            derivative_angle_dialog: None,
            parametric_dialog: false,
            parametric_x: 0,
            parametric_y: 0,
            parametric_mode: PlotPointsMode::Cartesian,
            parametric_min_draft: "0".into(),
            parametric_max_draft: "1".into(),
            plot_range_dialog: None,
            plot_min_draft: String::new(),
            plot_max_draft: String::new(),
            plot_samples_draft: String::new(),
            parameter_value_dialog: None,
            parameter_value_draft: "1".into(),
            named_parameter_dialog: false,
            parameter_name_draft: "a".into(),
            plot_axis_dialog: None,
            plot_axis_draft: "1".into(),
            plot_points_dialog: None,
            plot_points_first_draft: "1".into(),
            plot_points_second_draft: "1".into(),
            plot_table_dialog: None,
            plot_table_mode: PlotPointsMode::Cartesian,
            exit_requested: false,
        };
        e.fit();
        e
    }

    fn sync_document_fonts(&mut self, ctx: &egui::Context) {
        let families = self
            .doc
            .objects
            .iter()
            .filter(|object| !object.deleted)
            .filter_map(|object| self.doc.text_font_state(object.id).map(|font| font.family))
            .collect::<Vec<_>>();
        let mut changed = !self.font_definitions_applied;
        for family in families {
            if self.loaded_fonts.contains_key(&family) {
                continue;
            }
            let Some(data) = system_font_data(&family) else {
                self.loaded_fonts.insert(family, false);
                continue;
            };
            let key = format!("system:{family}");
            self.font_definitions
                .font_data
                .insert(key.clone(), Arc::new(data));
            let mut chain = vec![key];
            if let Some(fallbacks) = self
                .font_definitions
                .families
                .get(&egui::FontFamily::Proportional)
            {
                chain.extend(fallbacks.iter().cloned());
            }
            self.font_definitions
                .families
                .insert(egui::FontFamily::Name(family.clone().into()), chain);
            self.loaded_fonts.insert(family, true);
            changed = true;
        }
        if changed {
            ctx.set_fonts(self.font_definitions.clone());
            self.font_definitions_applied = true;
        }
    }

    fn canvas_font_id(&self, family: Option<&str>, size: f32) -> egui::FontId {
        let font_family = family
            .filter(|family| self.loaded_fonts.get(*family) == Some(&true))
            .map_or(egui::FontFamily::Proportional, |family| {
                egui::FontFamily::Name(family.into())
            });
        egui::FontId::new(size, font_family)
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
            if let Some((anchor, _)) =
                sketchpad_rs::render::number_lines_for_object(&self.doc, object.id)
            {
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
        self.button_view_center = self.center;
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
        let inserted_multi_page_links = self.doc.has_pending_multi_page_links();
        match self.doc.save_to(&path) {
            Ok(()) => {
                if inserted_multi_page_links {
                    self.selection_order.clear();
                    self.image_textures.clear();
                    self.pending = None;
                    self.dragging = None;
                    self.inspected = None;
                    self.text_selected_range = None;
                    self.marked_center = None;
                    self.marked_mirror = None;
                    self.marked_vector = None;
                    self.marked_distance = None;
                    self.marked_rectangular_distances = None;
                    self.marked_angle = None;
                    self.marked_ratio = None;
                    self.trace_marks.clear();
                    self.animations.clear();
                    self.movements.clear();
                    self.presentations.clear();
                    self.animation_button_dialog = None;
                    self.movement_button_dialog = None;
                    self.scroll_button_dialog = None;
                    self.presentation_button_dialog = None;
                }
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
        let svg =
            match svg_page_with_button_offset(&self.doc, viewport, self.button_offset(viewport)) {
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
    fn hit_grid(&self, v: &Viewport, p: egui::Pos2) -> Option<usize> {
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
    fn overlay_rows(&self) -> Vec<(usize, String)> {
        let mut rows = Vec::new();
        for object in self.doc.objects.iter().filter(|object| {
            object.page == self.doc.active_page && object.visible && !object.deleted
        }) {
            let id = object.id;
            if let Some(value) = self.doc.segment_length_cm(id) {
                rows.push((id, format!("Length = {value:.2} cm")));
            }
            if let Some(value) = self.doc.point_distance_cm(id) {
                rows.push((id, format!("Distance = {value:.2} cm")));
            }
            if let Some(value) = self.doc.ratio_value(id) {
                rows.push((id, format!("Ratio = {value:.2}")));
            }
            if let Some(value) = self.doc.perimeter_cm(id) {
                rows.push((id, format!("Perimeter = {value:.2} cm")));
            }
            if let Some(value) = self.doc.circle_circumference_cm(id) {
                rows.push((id, format!("Circumference = {value:.2} cm")));
            }
            if let Some(value) = self.doc.circle_radius_cm(id) {
                rows.push((id, format!("Radius = {value:.2} cm")));
            }
            if let Some(value) = self.doc.circle_area_cm2(id) {
                rows.push((id, format!("Area = {value:.2} cm²")));
            }
            if let Some(value) = self.doc.polygon_area_cm2(id) {
                rows.push((id, format!("Area = {value:.2} cm²")));
            }
            if let Some(value) = self.doc.arc_angle_degrees(id) {
                rows.push((id, format!("Arc Angle = {value:.2}°")));
            }
            if let Some(value) = self.doc.arc_length_cm(id) {
                rows.push((id, format!("Arc Length = {value:.2} cm")));
            }
            if let Some(value) = self.doc.angle_degrees(id) {
                rows.push((id, format!("Angle = {value:.2}°")));
            }
            if matches!(object.kind, Kind::SlopeMeasurement(_, _)) {
                let value = self
                    .doc
                    .slope_value(id)
                    .map(|value| format!("{value:.2}"))
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
    fn overlay_row_rect(&self, v: &Viewport, row: usize, id: usize, text: &str) -> egui::Rect {
        let at = self.doc.objects[id - 1]
            .function_anchor
            .map(|(x, y)| v.rect.min + egui::vec2(x as f32 + 2., y as f32 + 7.))
            .unwrap_or_else(|| v.rect.min + egui::vec2(12., 12. + row as f32 * 20.));
        egui::Rect::from_min_size(at, egui::vec2(text.chars().count() as f32 * 9., 19.))
    }
    fn button_offset(&self, v: &Viewport) -> egui::Vec2 {
        egui::vec2(
            ((self.button_view_center.x - v.center.x) * f64::from(v.scale)) as f32,
            ((self.button_view_center.y - v.center.y) * f64::from(v.scale)) as f32,
        )
    }
    fn action_button_rect(&self, v: &Viewport, anchor: (i16, i16), label: &str) -> egui::Rect {
        sketchpad_rs::render::action_button_rect(v, anchor, label).translate(self.button_offset(v))
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
                && matches!(&object.kind, Kind::PlainText { text, rect }
                if {
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
                && sketchpad_rs::render::number_screen_rect_for_object(v, &self.doc, object.id)
                    .is_some_and(|rect| rect.contains(p))
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
                        Kind::Polygon(_)
                        | Kind::ReflectedPolygon(_, _)
                        | Kind::VectorTranslatedPolygon(_, _, _)
                        | Kind::PolarTranslatedPolygon(_, _, _, _)
                        | Kind::FixedRectangularTranslatedPolygon(_, _, _, _, _)
                        | Kind::MarkedDistanceTranslatedPolygon(_, _, _)
                        | Kind::RectangularMarkedTranslatedPolygon(_, _, _)
                        | Kind::MixedRectangularTranslatedPolygon(_, _, _, _)
                        | Kind::RotatedPolygon(_, _, _)
                        | Kind::AngleRotatedPolygon(_, _, _, _, _)
                        | Kind::ScaledPolygon(_, _, _)
                        | Kind::RatioScaledPolygon(_, _, _, _, _) => {
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
                        | Kind::Locus(_, _) => {
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
    fn coordinate_selection(&self) -> (Option<usize>, Vec<usize>, bool) {
        let mut selected_origin = None;
        let mut selected_distances = Vec::new();
        let mut valid_distance_selection = true;
        for &id in &self.selection_order {
            match self
                .doc
                .objects
                .get(id.saturating_sub(1))
                .map(|object| &object.kind)
            {
                Some(Kind::Segment(_, _) | Kind::SegmentLength(_) | Kind::PointDistance(_, _)) => {
                    selected_distances.push(id);
                }
                Some(_) if selected_origin.is_none() && self.doc.position(id).is_some() => {
                    selected_origin = Some(id);
                }
                _ => valid_distance_selection = false,
            }
        }
        (
            selected_origin,
            selected_distances,
            valid_distance_selection,
        )
    }
    fn define_coordinate_system_label(&self) -> &'static str {
        match self.selection_order.as_slice() {
            [circle] if self.doc.circle_points(*circle).is_some() => "Define Unit Circle",
            [origin] if self.doc.position(*origin).is_some() => "Define Origin",
            _ => {
                let (_, distances, valid) = self.coordinate_selection();
                if valid {
                    match distances.len() {
                        1 => "Define Unit Distance",
                        2 => "Define Unit Distances",
                        _ => "Define Coordinate System",
                    }
                } else {
                    "Define Coordinate System"
                }
            }
        }
    }
    fn define_coordinate_system(&mut self) {
        let (selected_origin, selected_distances, valid_distance_selection) =
            self.coordinate_selection();
        let result = match self.selection_order.as_slice() {
            [circle] if self.doc.circle_points(*circle).is_some() => {
                self.doc.add_coordinate_system_from_circle(*circle)
            }
            [origin] if self.doc.position(*origin).is_some() => {
                self.doc.add_coordinate_system_at_point(*origin)
            }
            _ if valid_distance_selection && matches!(selected_distances.len(), 1 | 2) => {
                self.doc.add_coordinate_system_from_distances(
                    selected_origin,
                    &selected_distances,
                    self.center,
                )
            }
            _ => self.doc.add_default_coordinate_system(self.center),
        };
        match result {
            Ok(frame) => {
                self.select_only(frame);
                self.status = format!("Defined coordinate system {frame}");
            }
            Err(error) => self.status = error,
        }
    }
    fn mark_coordinate_system(&mut self) {
        let [selected] = self.selection_order.as_slice() else {
            self.status = "Select one coordinate system axis, origin, or unit point".into();
            return;
        };
        match self.doc.mark_coordinate_system(*selected) {
            Ok(frame) => self.status = format!("Marked coordinate system {frame}"),
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
        let [host] = self.selection_order.as_slice() else {
            self.status = "Select one circle or polygon interior to measure".into();
            return;
        };
        let host = *host;
        let source = self
            .doc
            .objects
            .get(host.saturating_sub(1))
            .map(|o| &o.kind);
        let (kind, name) = match (measurement, source) {
            (
                CircleMeasure::Circumference,
                Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)),
            ) => (Kind::CircleCircumference(host), "circumference"),
            (CircleMeasure::Radius, Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _))) => {
                (Kind::CircleRadius(host), "radius")
            }
            (CircleMeasure::Area, Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _))) => {
                (Kind::CircleArea(host), "area")
            }
            (CircleMeasure::Area, Some(kind)) if kind.is_polygon() => {
                (Kind::PolygonArea(host), "area")
            }
            _ => {
                self.status = "Select one circle or polygon interior to measure".into();
                return;
            }
        };
        let id = self.doc.add(kind);
        self.select_only(id);
        self.status = format!("Measured {name} of object {host}");
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
                if !multi
                    && let Some(id) = hit
                    && let Kind::AnimationButton {
                        targets, settings, ..
                    } = &self.doc.objects[id - 1].kind
                {
                    let targets = targets.clone();
                    let settings = settings.clone();
                    self.animate_button_targets(&targets, &settings);
                    self.selection_order.clear();
                    return;
                }
                if !multi
                    && let Some(id) = hit
                    && matches!(self.doc.objects[id - 1].kind, Kind::MovementButton { .. })
                {
                    self.activate_movement_button(id);
                    self.selection_order.clear();
                    return;
                }
                if !multi
                    && let Some(id) = hit
                    && let Kind::ScrollButton {
                        target, placement, ..
                    } = self.doc.objects[id - 1].kind
                {
                    if let Err(error) = self.scroll_to_button_target(target, placement, v) {
                        self.status = error;
                    }
                    self.selection_order.clear();
                    return;
                }
                if !multi
                    && let Some(id) = hit
                    && matches!(
                        self.doc.objects[id - 1].kind,
                        Kind::PresentationButton { .. }
                    )
                {
                    if let Err(error) = self.activate_presentation_button(id, v, 0) {
                        self.status = error;
                    }
                    self.selection_order.clear();
                    return;
                }
                if !multi
                    && let Some(id) = hit
                    && let Kind::LinkUrlButton { url, .. } = &self.doc.objects[id - 1].kind
                {
                    let url = url.clone();
                    if let Err(error) = self.activate_link_url(&url) {
                        self.status = error;
                    }
                    self.selection_order.clear();
                    return;
                }
                if !multi
                    && let Some(id) = hit
                    && let Kind::LinkPageButton {
                        target_page,
                        target_button_ordinal,
                        ..
                    } = self.doc.objects[id - 1].kind
                {
                    if let Err(error) =
                        self.activate_link_page(target_page, target_button_ordinal, v, 0)
                    {
                        self.status = error;
                    }
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
    fn construct_locus(&mut self) {
        let ids = self.selection_order.clone();
        if ids.len() != 2 {
            self.status = "Select a moving point and a dependent object".into();
            return;
        }
        let (driver, target) = if matches!(
            self.doc.objects[ids[0] - 1].kind,
            Kind::PointOnSegment(_, _) | Kind::PointOnCircle(_, _, _) | Kind::PointOnArc(_, _)
        ) {
            (ids[0], ids[1])
        } else {
            (ids[1], ids[0])
        };
        match self.doc.construct_locus(driver, target) {
            Ok(id) => {
                self.select_only(id);
                self.status = format!("Constructed locus {id}");
            }
            Err(error) => self.status = error,
        }
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
                        | Kind::RatioScaledCircle(_, _, _, _, _)
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
        if self.selection_order.len() != 1 {
            self.status = "Select one point, segment, or circle to translate".into();
            return;
        }
        let source = self.selection_order[0];
        let source_is_segment = matches!(
            self.doc
                .objects
                .get(source.saturating_sub(1))
                .map(|object| &object.kind),
            Some(Kind::Segment(_, _))
        );
        let source_is_circle = matches!(
            self.doc
                .objects
                .get(source.saturating_sub(1))
                .map(|object| &object.kind),
            Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _))
        );
        let source_is_polygon = matches!(
            self.doc
                .objects
                .get(source.saturating_sub(1))
                .map(|object| &object.kind),
            Some(Kind::Polygon(_))
        );
        if self.doc.position(source).is_none()
            && !source_is_segment
            && !source_is_circle
            && !(source_is_polygon
                && matches!(
                    self.translation_mode,
                    TranslationMode::FixedPolar
                        | TranslationMode::FixedRectangular
                        | TranslationMode::MarkedVector
                        | TranslationMode::MarkedDistance
                        | TranslationMode::RectangularMarked
                        | TranslationMode::RectangularHorizontalFixed
                        | TranslationMode::RectangularVerticalFixed
                ))
        {
            self.status = "Select one point, segment, or circle to translate".into();
            return;
        }
        if (source_is_segment
            && !matches!(
                self.translation_mode,
                TranslationMode::FixedPolar
                    | TranslationMode::FixedRectangular
                    | TranslationMode::MarkedVector
                    | TranslationMode::MarkedDistance
            ))
            || (source_is_circle
                && !matches!(
                    self.translation_mode,
                    TranslationMode::FixedPolar
                        | TranslationMode::FixedRectangular
                        | TranslationMode::MarkedVector
                        | TranslationMode::MarkedDistance
                ))
        {
            self.status = "This translation mode currently supports points only".into();
            return;
        }
        let kind = match self.translation_mode {
            TranslationMode::FixedPolar => {
                if !self.translation_angle.is_finite() || !self.translation_distance.is_finite() {
                    self.status = "Translation values must be finite".into();
                    return;
                }
                if source_is_segment {
                    Kind::PolarTranslatedSegment(
                        source,
                        self.translation_angle,
                        37.79527559055118,
                        self.translation_distance,
                    )
                } else if source_is_circle {
                    Kind::PolarTranslatedCircle(
                        source,
                        self.translation_angle,
                        37.79527559055118,
                        self.translation_distance,
                    )
                } else if source_is_polygon {
                    Kind::PolarTranslatedPolygon(
                        source,
                        self.translation_angle,
                        37.79527559055118,
                        self.translation_distance,
                    )
                } else {
                    Kind::PolarTranslatedPoint(
                        source,
                        self.translation_angle,
                        37.79527559055118,
                        self.translation_distance,
                    )
                }
            }
            TranslationMode::FixedRectangular => {
                if !self.translation_horizontal.is_finite()
                    || !self.translation_vertical.is_finite()
                {
                    self.status = "Translation values must be finite".into();
                    return;
                }
                if source_is_segment {
                    Kind::FixedRectangularTranslatedSegment(
                        source,
                        37.79527559055118,
                        self.translation_horizontal,
                        37.79527559055118,
                        self.translation_vertical,
                    )
                } else if source_is_circle {
                    Kind::FixedRectangularTranslatedCircle(
                        source,
                        37.79527559055118,
                        self.translation_horizontal,
                        37.79527559055118,
                        self.translation_vertical,
                    )
                } else if source_is_polygon {
                    Kind::FixedRectangularTranslatedPolygon(
                        source,
                        37.79527559055118,
                        self.translation_horizontal,
                        37.79527559055118,
                        self.translation_vertical,
                    )
                } else {
                    Kind::FixedRectangularTranslatedPoint(
                        source,
                        37.79527559055118,
                        self.translation_horizontal,
                        37.79527559055118,
                        self.translation_vertical,
                    )
                }
            }
            TranslationMode::MarkedDistance => {
                let Some(distance) = self.marked_distance else {
                    self.status = "Mark a distance value first".into();
                    return;
                };
                if !self.translation_angle.is_finite()
                    || self
                        .doc
                        .objects
                        .get(distance.saturating_sub(1))
                        .is_none_or(|object| object.deleted || object.page != self.doc.active_page)
                    || self.doc.distance_value_pixels(distance).is_none()
                {
                    self.status = "Marked distance is unavailable".into();
                    return;
                }
                if source_is_segment {
                    Kind::MarkedDistanceTranslatedSegment(source, distance, self.translation_angle)
                } else if source_is_circle {
                    Kind::MarkedDistanceTranslatedCircle(source, distance, self.translation_angle)
                } else if source_is_polygon {
                    Kind::MarkedDistanceTranslatedPolygon(source, distance, self.translation_angle)
                } else {
                    Kind::MarkedDistanceTranslatedPoint(source, distance, self.translation_angle)
                }
            }
            TranslationMode::RectangularMarked => {
                let Some((horizontal, vertical)) = self.marked_rectangular_distances else {
                    self.status = "Mark two distance values first".into();
                    return;
                };
                if [horizontal, vertical].iter().any(|&id| {
                    self.doc
                        .objects
                        .get(id.saturating_sub(1))
                        .is_none_or(|object| object.deleted || object.page != self.doc.active_page)
                        || self.doc.distance_value_pixels(id).is_none()
                }) {
                    self.status = "Marked distances are unavailable".into();
                    return;
                }
                if source_is_polygon {
                    Kind::RectangularMarkedTranslatedPolygon(source, horizontal, vertical)
                } else {
                    Kind::RectangularMarkedTranslatedPoint(source, horizontal, vertical)
                }
            }
            TranslationMode::RectangularHorizontalFixed
            | TranslationMode::RectangularVerticalFixed => {
                let Some((horizontal, vertical)) = self.marked_rectangular_distances else {
                    self.status = "Mark two distance values first".into();
                    return;
                };
                let fixed_horizontal =
                    self.translation_mode == TranslationMode::RectangularHorizontalFixed;
                let distance = if fixed_horizontal {
                    vertical
                } else {
                    horizontal
                };
                let fixed_cm = if fixed_horizontal {
                    self.translation_horizontal
                } else {
                    self.translation_vertical
                };
                if !fixed_cm.is_finite()
                    || self
                        .doc
                        .objects
                        .get(distance.saturating_sub(1))
                        .is_none_or(|object| object.deleted || object.page != self.doc.active_page)
                    || self.doc.distance_value_pixels(distance).is_none()
                {
                    self.status = "Mixed rectangular translation values are unavailable".into();
                    return;
                }
                if source_is_polygon {
                    Kind::MixedRectangularTranslatedPolygon(
                        source,
                        distance,
                        fixed_cm,
                        fixed_horizontal,
                    )
                } else {
                    Kind::MixedRectangularTranslatedPoint(
                        source,
                        distance,
                        fixed_cm,
                        fixed_horizontal,
                    )
                }
            }
            TranslationMode::MarkedVector => {
                let Some((start, end)) = self.marked_vector else {
                    self.status = "Mark two points as a vector first".into();
                    return;
                };
                if [start, end].iter().any(|&id| {
                    self.doc
                        .objects
                        .get(id.saturating_sub(1))
                        .is_none_or(|object| object.deleted || object.page != self.doc.active_page)
                        || self.doc.position(id).is_none()
                }) {
                    self.status = "Marked vector is unavailable".into();
                    return;
                }
                if source_is_segment {
                    Kind::VectorTranslatedSegment(source, start, end)
                } else if source_is_circle {
                    Kind::VectorTranslatedCircle(source, start, end)
                } else if source_is_polygon {
                    Kind::VectorTranslatedPolygon(source, start, end)
                } else {
                    Kind::VectorTranslatedPoint(source, start, end)
                }
            }
        };
        let id = self.doc.add(kind);
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
                    self.status =
                        "Select a point, segment, circle, or interior and mark a mirror line"
                            .into();
                    return;
                }
            },
            _ => {
                self.status =
                    "Select a point, segment, circle, or interior, then a mirror line".into();
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
            self.status = "Select a point, segment, circle, or interior, then a mirror line".into();
            return;
        }
        let kind = if self.doc.position(source).is_some() {
            Kind::ReflectedPoint(source, mirror)
        } else if self.doc.circle_points(source).is_some() {
            Kind::ReflectedCircle(source, mirror)
        } else if matches!(
            self.doc
                .objects
                .get(source.saturating_sub(1))
                .map(|o| &o.kind),
            Some(Kind::Segment(_, _))
        ) {
            Kind::ReflectedSegment(source, mirror)
        } else if matches!(
            self.doc
                .objects
                .get(source.saturating_sub(1))
                .map(|o| &o.kind),
            Some(Kind::Polygon(_))
        ) {
            Kind::ReflectedPolygon(source, mirror)
        } else {
            self.status = "Select a point, segment, circle, or interior, then a mirror line".into();
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
    fn mark_vector_selected(&mut self) {
        let points = self
            .selection_order
            .iter()
            .copied()
            .filter(|&id| self.doc.position(id).is_some())
            .collect::<Vec<_>>();
        if points.len() < 2 {
            self.status = "Select two points to mark a vector".into();
            return;
        }
        let start = points[points.len() - 2];
        let end = points[points.len() - 1];
        self.marked_vector = Some((start, end));
        self.translation_mode = TranslationMode::MarkedVector;
        self.doc.selected.remove(&start);
        self.doc.selected.remove(&end);
        self.selection_order.retain(|&id| id != start && id != end);
        self.status = format!("Marked vector from point {start} to point {end}");
    }
    fn mark_distance_selected(&mut self) {
        let values = self
            .selection_order
            .iter()
            .copied()
            .filter(|&id| self.doc.distance_value_pixels(id).is_some())
            .collect::<Vec<_>>();
        match values.as_slice() {
            [id] => {
                self.marked_distance = Some(*id);
                self.translation_mode = TranslationMode::MarkedDistance;
                self.doc.selected.remove(id);
                self.selection_order.retain(|selected| selected != id);
                self.status = format!("Marked distance value {id}");
            }
            [horizontal, vertical] => {
                self.marked_rectangular_distances = Some((*horizontal, *vertical));
                self.translation_mode = TranslationMode::RectangularMarked;
                for id in [horizontal, vertical] {
                    self.doc.selected.remove(id);
                }
                self.selection_order
                    .retain(|id| id != horizontal && id != vertical);
                self.status = format!(
                    "Marked horizontal distance {horizontal} and vertical distance {vertical}"
                );
            }
            _ => self.status = "Select one or two distance measurements".into(),
        }
    }
    fn mark_angle_selected(&mut self) {
        let points = self
            .selection_order
            .iter()
            .copied()
            .filter(|&id| self.doc.position(id).is_some())
            .collect::<Vec<_>>();
        if points.len() < 3 {
            self.status = "Select three points to mark an angle".into();
            return;
        }
        let angle = [
            points[points.len() - 3],
            points[points.len() - 2],
            points[points.len() - 1],
        ];
        if self
            .doc
            .angle_for_points(angle[0], angle[1], angle[2])
            .is_none()
        {
            self.status = "Marked angle needs distinct sides".into();
            return;
        }
        self.marked_angle = Some(angle);
        self.rotation_use_marked = true;
        for &id in &angle {
            self.doc.selected.remove(&id);
        }
        self.selection_order.retain(|id| !angle.contains(id));
        self.status = format!("Marked angle {}–{}–{}", angle[0], angle[1], angle[2]);
    }
    fn mark_ratio_selected(&mut self) {
        let points = self
            .selection_order
            .iter()
            .copied()
            .filter(|&id| self.doc.position(id).is_some())
            .collect::<Vec<_>>();
        if points.len() < 3 {
            self.status = "Select three collinear points to mark a ratio".into();
            return;
        }
        let ratio = [
            points[points.len() - 3],
            points[points.len() - 2],
            points[points.len() - 1],
        ];
        let [Some(origin), Some(base), Some(target)] = ratio.map(|id| self.doc.position(id)) else {
            self.status = "Selected ratio points are unavailable".into();
            return;
        };
        let (bx, by) = (base.x - origin.x, base.y - origin.y);
        let (cx, cy) = (target.x - origin.x, target.y - origin.y);
        let cross = bx * cy - by * cx;
        let tolerance = 1e-8 * bx.hypot(by) * cx.hypot(cy);
        if self
            .doc
            .ratio_for_points(ratio[0], ratio[1], ratio[2])
            .is_none()
            || cross.abs() > tolerance
        {
            self.status = "Marked ratio needs a nonzero base and collinear points".into();
            return;
        }
        self.marked_ratio = Some(ratio);
        self.dilation_use_marked = true;
        for &id in &ratio {
            self.doc.selected.remove(&id);
        }
        self.selection_order.retain(|id| !ratio.contains(id));
        self.status = format!("Marked ratio {}–{}–{}", ratio[0], ratio[1], ratio[2]);
    }
    fn dilate_selected(&mut self) {
        if self.doc.partial_view {
            self.status = "Dilation on this document cannot be saved safely".into();
            return;
        }
        if !self.dilation_use_marked
            && (!self.dilation_factor.is_finite() || self.dilation_factor == 0.)
        {
            self.status =
                "Select a point, segment, or circle and a center point; use a nonzero ratio".into();
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
                self.status = "Select a point, segment, or circle, then a center point".into();
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
            self.status = "Select a point, segment, or circle, then a center point".into();
            return;
        }
        let kind = if self.dilation_use_marked {
            let Some([origin, base, target]) = self.marked_ratio else {
                self.status = "Mark a ratio first".into();
                return;
            };
            if [origin, base, target].iter().any(|&id| {
                self.doc
                    .objects
                    .get(id.saturating_sub(1))
                    .is_none_or(|object| object.deleted || object.page != self.doc.active_page)
                    || self.doc.position(id).is_none()
            }) || self.doc.ratio_for_points(origin, base, target).is_none()
            {
                self.status = "Marked ratio is unavailable".into();
                return;
            }
            if self.doc.position(source).is_some() {
                Kind::RatioScaledPoint(source, center, origin, base, target)
            } else if self.doc.circle_points(source).is_some() {
                Kind::RatioScaledCircle(source, center, origin, base, target)
            } else if matches!(
                self.doc
                    .objects
                    .get(source.saturating_sub(1))
                    .map(|o| &o.kind),
                Some(Kind::Segment(_, _))
            ) {
                Kind::RatioScaledSegment(source, center, origin, base, target)
            } else if matches!(
                self.doc
                    .objects
                    .get(source.saturating_sub(1))
                    .map(|o| &o.kind),
                Some(Kind::Polygon(_))
            ) {
                Kind::RatioScaledPolygon(source, center, origin, base, target)
            } else {
                self.status =
                    "Select a point, segment, or circle to dilate by the marked ratio".into();
                return;
            }
        } else if self.doc.position(source).is_some() {
            Kind::ScaledPoint(source, center, self.dilation_factor)
        } else if self.doc.circle_points(source).is_some() {
            Kind::ScaledCircle(source, center, self.dilation_factor)
        } else if matches!(
            self.doc
                .objects
                .get(source.saturating_sub(1))
                .map(|o| &o.kind),
            Some(Kind::Segment(_, _))
        ) {
            Kind::ScaledSegment(source, center, self.dilation_factor)
        } else if matches!(
            self.doc
                .objects
                .get(source.saturating_sub(1))
                .map(|o| &o.kind),
            Some(Kind::Polygon(_))
        ) {
            Kind::ScaledPolygon(source, center, self.dilation_factor)
        } else {
            self.status = "Select a point, segment, or circle, then a center point".into();
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
        if !self.rotation_use_marked && !self.rotation_angle.is_finite() {
            self.status = "Select a source and center point; use a finite angle".into();
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
                self.status = "Select a point, segment, or circle, then a center point".into();
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
        let source_kind = self
            .doc
            .objects
            .get(source.saturating_sub(1))
            .map(|o| &o.kind);
        let source_point = self.doc.position(source).is_some();
        let source_segment = matches!(source_kind, Some(Kind::Segment(_, _)))
            && self.doc.line_points(source).is_some();
        let source_circle = matches!(
            source_kind,
            Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _))
        ) && self.doc.circle_points(source).is_some();
        let source_polygon = matches!(source_kind, Some(Kind::Polygon(_)))
            && self.doc.polygon_points(source).is_some();
        if !(source_point || source_segment || source_circle || source_polygon)
            || self.doc.position(center).is_none()
        {
            self.status = "Select a point, segment, or circle and a center point".into();
            return;
        }
        let kind = if self.rotation_use_marked {
            let Some([first, vertex, last]) = self.marked_angle else {
                self.status = "Mark an angle first".into();
                return;
            };
            if [first, vertex, last].iter().any(|&id| {
                self.doc
                    .objects
                    .get(id.saturating_sub(1))
                    .is_none_or(|object| object.deleted || object.page != self.doc.active_page)
                    || self.doc.position(id).is_none()
            }) || self.doc.angle_for_points(first, vertex, last).is_none()
            {
                self.status = "Marked angle is unavailable".into();
                return;
            }
            if source_point {
                Kind::AngleRotatedPoint(source, center, first, vertex, last)
            } else if source_segment {
                Kind::AngleRotatedSegment(source, center, first, vertex, last)
            } else if source_polygon {
                Kind::AngleRotatedPolygon(source, center, first, vertex, last)
            } else {
                Kind::AngleRotatedCircle(source, center, first, vertex, last)
            }
        } else {
            if source_point {
                Kind::RotatedPoint(source, center, self.rotation_angle)
            } else if source_segment {
                Kind::RotatedSegment(source, center, self.rotation_angle)
            } else if source_polygon {
                Kind::RotatedPolygon(source, center, self.rotation_angle)
            } else {
                Kind::RotatedCircle(source, center, self.rotation_angle)
            }
        };
        let id = self.doc.add(kind);
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
                Some(
                    VisualStyle::Point { color, .. }
                    | VisualStyle::Stroke { color, .. }
                    | VisualStyle::Polygon { color },
                ) => color,
                _ => RgbColor {
                    red: 23,
                    green: 107,
                    blue: 214,
                },
            };
            if let Some(point) = self.doc.position(object.id) {
                self.trace_marks.push(TraceMark::Point(point, color));
            } else if let Some((a, b, mode)) = self.doc.line_points(object.id) {
                self.trace_marks
                    .push(TraceMark::Straight(a, b, mode, color));
            } else if let Some((center, radius)) = self.doc.circle_points(object.id) {
                self.trace_marks
                    .push(TraceMark::Circle(center, radius, color));
            } else if let Some(points) = self.doc.arc_points(object.id, 128) {
                self.trace_marks.push(TraceMark::Arc(points, color));
            } else if let Some(points) = self.doc.polygon_points(object.id) {
                self.trace_marks.push(TraceMark::Polygon(points, color));
            }
        }
        if self.trace_marks.len() > 8192 {
            self.trace_marks.drain(..self.trace_marks.len() - 8192);
        }
    }
    fn next_animation_random(state: &mut u64) -> f64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        (*state >> 11) as f64 / (1u64 << 53) as f64
    }
    fn animation_track(&self, id: usize) -> Option<AnimationTrack> {
        let object = self.doc.objects.get(id.checked_sub(1)?)?;
        if object.deleted || !object.visible || object.page != self.doc.active_page {
            return None;
        }
        if let Kind::Free(point) = object.kind {
            let mut random_state = ((id as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                ^ point.x.to_bits().rotate_left(17)
                ^ point.y.to_bits().rotate_left(41))
                | 1;
            let angle = Self::next_animation_random(&mut random_state) * std::f64::consts::TAU;
            return Some(AnimationTrack {
                id,
                phase: 0.,
                direction: 1.,
                free_velocity: Some(Point {
                    x: angle.cos(),
                    y: angle.sin(),
                }),
                free_turn_remaining: 0.35,
                random_state,
                button_settings: None,
                start_phase: 0.,
                distance_travelled: 0.,
            });
        }
        let phase = match object.kind {
            Kind::PointOnSegment(host, t)
                if matches!(self.doc.line_points(host), Some((_, _, LineMode::Segment))) =>
            {
                t
            }
            Kind::PointOnCircle(host, x, y) if self.doc.circle_points(host).is_some() => {
                y.atan2(x).rem_euclid(std::f64::consts::TAU) / std::f64::consts::TAU
            }
            Kind::PointOnArc(host, t) if self.doc.point_on_arc(host, t).is_some() => t,
            _ => return None,
        };
        phase.is_finite().then_some(AnimationTrack {
            id,
            phase,
            direction: 1.,
            free_velocity: None,
            free_turn_remaining: 0.,
            random_state: ((id as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ phase.to_bits()) | 1,
            button_settings: None,
            start_phase: phase,
            distance_travelled: 0.,
        })
    }
    fn animate_targets(&mut self, targets: &[usize], show_controller: bool) {
        let added = targets
            .iter()
            .copied()
            .filter(|id| !self.animations.iter().any(|track| track.id == *id))
            .filter_map(|id| self.animation_track(id))
            .collect::<Vec<_>>();
        if added.is_empty() {
            self.status = "Select a free point or a point on a segment, circle, or arc".into();
            return;
        }
        if self.animations.is_empty() {
            self.doc.checkpoint();
        }
        self.status = format!("Animating {} point(s)", added.len());
        self.animations.extend(added);
        self.animation_paused = false;
        self.animation_last_tick = Instant::now();
        self.show_motion_controller |= show_controller;
    }
    fn animate_selected(&mut self) {
        let selected = self.doc.selected.iter().copied().collect::<Vec<_>>();
        self.animate_targets(&selected, true);
    }
    fn animate_button_targets(&mut self, targets: &[usize], settings: &[AnimationSettings]) {
        if targets
            .iter()
            .any(|target| self.animations.iter().any(|track| track.id == *target))
        {
            self.animations.retain(|track| !targets.contains(&track.id));
            self.status = "Stopped animation".into();
            return;
        }
        self.animate_targets(targets, false);
        for (&target, &setting) in targets.iter().zip(settings) {
            if let Some(track) = self.animations.iter_mut().find(|track| track.id == target) {
                track.button_settings = Some(setting);
                track.direction = if setting.direction == AnimationDirection::Backward {
                    -1.
                } else {
                    1.
                };
            }
        }
    }
    fn activate_movement_button(&mut self, id: usize) {
        if self.movements.iter().any(|track| track.button == id) {
            self.movements.retain(|track| track.button != id);
            self.status = "Stopped movement".into();
            return;
        }
        let Kind::MovementButton {
            source,
            destination,
            speed,
            follow,
            ..
        } = self.doc.objects[id - 1].kind
        else {
            return;
        };
        let Some(target) = self.doc.position(destination) else {
            self.status = "Movement destination is unavailable".into();
            return;
        };
        if !matches!(self.doc.objects[source - 1].kind, Kind::Free(_)) {
            self.status = "Movement source is unavailable".into();
            return;
        }
        self.doc.checkpoint();
        if speed == MovementSpeed::Instant {
            self.doc.move_point(source, target);
            self.status = "Moved point to destination".into();
        } else {
            self.movements.push(MovementTrack {
                button: id,
                source,
                destination,
                initial_destination: (!follow).then_some(target),
                speed,
            });
            self.animation_last_tick = Instant::now();
            self.status = "Moving point to destination".into();
        }
    }
    fn scroll_to_button_target(
        &mut self,
        target: usize,
        placement: ScrollPlacement,
        viewport: &Viewport,
    ) -> Result<(), String> {
        let point = self
            .doc
            .position(target)
            .ok_or("Scroll target is unavailable")?;
        self.center = match placement {
            ScrollPlacement::Center => point,
            ScrollPlacement::UpperLeft => Point {
                x: point.x + f64::from(viewport.rect.width()) / (2. * f64::from(self.scale)),
                y: point.y + f64::from(viewport.rect.height()) / (2. * f64::from(self.scale)),
            },
        };
        self.status = format!("Scrolled to point {target}");
        Ok(())
    }

    fn activate_link_url(&mut self, url: &str) -> Result<(), String> {
        if let Some(relative) = url.strip_prefix("sketchdoc://") {
            if relative.is_empty() || std::path::Path::new(relative).is_absolute() {
                return Err("Invalid sketchdoc link path".into());
            }
            let base = self
                .doc
                .path
                .as_ref()
                .and_then(|path| path.parent())
                .ok_or("Save this sketch before following its document link")?;
            let path = base.join(relative);
            if self.doc.dirty {
                self.pending_action = Some(PendingAction::Open(path));
            } else {
                self.open(path);
            }
            return Ok(());
        }
        if !["http://", "https://", "file://"]
            .iter()
            .any(|prefix| url.starts_with(prefix))
        {
            return Err("This Link URL scheme is not yet supported".into());
        }
        #[cfg(target_os = "windows")]
        let mut command = {
            let mut command = Command::new("rundll32");
            command.arg("url.dll,FileProtocolHandler");
            command
        };
        #[cfg(target_os = "macos")]
        let mut command = Command::new("open");
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let mut command = Command::new("xdg-open");
        command
            .arg(url)
            .spawn()
            .map_err(|error| format!("Could not open link: {error}"))?;
        self.status = format!("Opened link {url}");
        Ok(())
    }

    fn activate_link_page(
        &mut self,
        target_page: usize,
        target_button_ordinal: Option<usize>,
        viewport: &Viewport,
        depth: usize,
    ) -> Result<(), String> {
        if target_page >= self.doc.page_count {
            return Err("Link destination page is unavailable".into());
        }
        let destination_button = target_button_ordinal
            .map(|ordinal| {
                self.doc
                    .link_destination_buttons(target_page)
                    .get(
                        ordinal
                            .checked_sub(1)
                            .ok_or("Invalid destination button ordinal")?,
                    )
                    .copied()
                    .ok_or("Link destination button is unavailable")
            })
            .transpose()?;
        self.animations.clear();
        self.movements.clear();
        self.presentations.clear();
        self.doc.active_page = target_page;
        self.doc.selected.clear();
        self.selection_order.clear();
        self.fit();
        self.status = format!("Opened page {}", target_page + 1);
        if let Some(id) = destination_button {
            let destination_viewport = Viewport {
                rect: viewport.rect,
                scale: self.scale,
                center: self.center,
            };
            self.activate_child_action(id, &destination_viewport, depth + 1)?;
        }
        Ok(())
    }

    fn action_is_running(&self, id: usize) -> bool {
        match self
            .doc
            .objects
            .get(id.saturating_sub(1))
            .map(|object| &object.kind)
        {
            Some(Kind::AnimationButton { targets, .. }) => self
                .animations
                .iter()
                .any(|track| targets.contains(&track.id)),
            Some(Kind::MovementButton { .. }) => {
                self.movements.iter().any(|track| track.button == id)
            }
            Some(Kind::PresentationButton { .. }) => {
                self.presentations.iter().any(|track| track.id == id)
            }
            _ => false,
        }
    }

    fn stop_child_action(&mut self, id: usize, depth: usize) {
        if depth > 32 {
            return;
        }
        match self
            .doc
            .objects
            .get(id.saturating_sub(1))
            .map(|object| &object.kind)
        {
            Some(Kind::AnimationButton { targets, .. }) => {
                let targets = targets.clone();
                self.animations.retain(|track| !targets.contains(&track.id));
            }
            Some(Kind::MovementButton { .. }) => {
                self.movements.retain(|track| track.button != id);
            }
            Some(Kind::PresentationButton { .. }) => {
                if let Some(index) = self.presentations.iter().position(|track| track.id == id) {
                    let track = self.presentations.remove(index);
                    for child in track.active {
                        self.stop_child_action(child, depth + 1);
                    }
                }
            }
            _ => {}
        }
    }

    fn activate_child_action(
        &mut self,
        id: usize,
        viewport: &Viewport,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 32 {
            return Err("Presentation buttons are nested too deeply".into());
        }
        let object = self
            .doc
            .objects
            .get(id.checked_sub(1).ok_or("Invalid action button ID")?)
            .ok_or("Invalid action button ID")?;
        if object.deleted || !object.visible || object.page != self.doc.active_page {
            return Err("A Presentation parent button is unavailable".into());
        }
        match object.kind.clone() {
            Kind::HideShowButton { .. } => self.doc.activate_hide_show_button(id),
            Kind::AnimationButton {
                targets, settings, ..
            } => {
                self.animate_button_targets(&targets, &settings);
                Ok(())
            }
            Kind::MovementButton { .. } => {
                self.activate_movement_button(id);
                Ok(())
            }
            Kind::ScrollButton {
                target, placement, ..
            } => self.scroll_to_button_target(target, placement, viewport),
            Kind::LinkUrlButton { url, .. } => self.activate_link_url(&url),
            Kind::LinkPageButton {
                target_page,
                target_button_ordinal,
                ..
            } => self.activate_link_page(target_page, target_button_ordinal, viewport, depth + 1),
            Kind::PresentationButton { .. } => {
                self.activate_presentation_button(id, viewport, depth + 1)
            }
            _ => Err("Object is not an action button".into()),
        }
    }

    fn run_sequential_presentation(
        &mut self,
        mut track: PresentationTrack,
        viewport: &Viewport,
        depth: usize,
    ) -> Result<(), String> {
        while track.next < track.targets.len() {
            let child = track.targets[track.next];
            track.next += 1;
            self.activate_child_action(child, viewport, depth + 1)?;
            if self.action_is_running(child) {
                track.active = vec![child];
                self.presentations.push(track);
                return Ok(());
            }
        }
        Ok(())
    }

    fn activate_presentation_button(
        &mut self,
        id: usize,
        viewport: &Viewport,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 32 {
            return Err("Presentation buttons are nested too deeply".into());
        }
        if let Some(index) = self.presentations.iter().position(|track| track.id == id) {
            let mut track = self.presentations.remove(index);
            for child in track.active.drain(..) {
                self.stop_child_action(child, depth + 1);
            }
            if track.mode == PresentationMode::Sequential {
                self.run_sequential_presentation(track, viewport, depth + 1)?;
            }
            return Ok(());
        }
        let Some(object) = self
            .doc
            .objects
            .get(id.checked_sub(1).ok_or("Invalid button ID")?)
        else {
            return Err("Invalid button ID".into());
        };
        if object.deleted || !object.visible || object.page != self.doc.active_page {
            return Err("Presentation button is unavailable".into());
        }
        let Kind::PresentationButton { targets, mode, .. } = &object.kind else {
            return Err("Object is not a Presentation button".into());
        };
        let targets = targets.clone();
        let mode = *mode;
        let mut track = PresentationTrack {
            id,
            targets,
            mode,
            next: 0,
            active: Vec::new(),
        };
        match mode {
            PresentationMode::Simultaneous => {
                for &child in &track.targets {
                    self.activate_child_action(child, viewport, depth + 1)?;
                    if self.action_is_running(child) {
                        track.active.push(child);
                    }
                }
                if !track.active.is_empty() {
                    self.presentations.push(track);
                }
            }
            PresentationMode::Sequential => {
                self.run_sequential_presentation(track, viewport, depth + 1)?
            }
        }
        self.status = format!("Activated Presentation button {id}");
        Ok(())
    }

    fn tick_presentations(&mut self) {
        let Some(rect) = self.viewport_rect else {
            return;
        };
        let tracks = std::mem::take(&mut self.presentations);
        for mut track in tracks {
            match track.mode {
                PresentationMode::Simultaneous => {
                    track.active.retain(|&id| self.action_is_running(id));
                    if !track.active.is_empty() {
                        self.presentations.push(track);
                    }
                }
                PresentationMode::Sequential => {
                    if track.active.iter().all(|&id| !self.action_is_running(id)) {
                        track.active.clear();
                        let viewport = Viewport {
                            rect,
                            scale: self.scale,
                            center: self.center,
                        };
                        if let Err(error) = self.run_sequential_presentation(track, &viewport, 0) {
                            self.status = error;
                        }
                    } else {
                        self.presentations.push(track);
                    }
                }
            }
        }
    }
    fn tick_movement(&mut self, elapsed_seconds: f64) {
        if self.movements.is_empty() || !elapsed_seconds.is_finite() {
            return;
        }
        self.capture_traces();
        let mut tracks = std::mem::take(&mut self.movements);
        tracks.retain(|track| {
            let Some(current) = self.doc.position(track.source) else {
                return false;
            };
            let Some(target) = track
                .initial_destination
                .or_else(|| self.doc.position(track.destination))
            else {
                return false;
            };
            let delta_x = target.x - current.x;
            let delta_y = target.y - current.y;
            let distance = delta_x.hypot(delta_y);
            if !distance.is_finite() {
                return false;
            }
            let pixels_per_second = match track.speed {
                MovementSpeed::Slow => 60.,
                MovementSpeed::Medium => 150.,
                MovementSpeed::Fast => 360.,
                MovementSpeed::Instant => f64::INFINITY,
            };
            let step = pixels_per_second * elapsed_seconds.clamp(0., 0.1) / f64::from(self.scale);
            let fraction = if distance <= step || distance < 1e-9 {
                1.
            } else {
                step / distance
            };
            let moved = self.doc.move_point(
                track.source,
                Point {
                    x: current.x + delta_x * fraction,
                    y: current.y + delta_y * fraction,
                },
            );
            moved && fraction < 1.
        });
        self.movements = tracks;
    }
    fn stop_animation(&mut self) {
        self.animations.clear();
        self.movements.clear();
        self.animation_paused = false;
        self.status = "Stopped animation".into();
    }
    fn tick_animation(&mut self, elapsed_seconds: f64) {
        if self.animations.is_empty() || self.animation_paused || !elapsed_seconds.is_finite() {
            return;
        }
        let base_step = elapsed_seconds.clamp(0., 0.1) * self.animation_speed / 2.;
        if base_step == 0. {
            return;
        }
        self.capture_traces();
        let mut tracks = std::mem::take(&mut self.animations);
        tracks.retain_mut(|track| {
            let Some(object) = self.doc.objects.get(track.id - 1) else {
                return false;
            };
            if object.deleted || !object.visible || object.page != self.doc.active_page {
                return false;
            }
            let kind = &object.kind;
            let speed_scale = track
                .button_settings
                .map_or(1., |settings| f64::from(settings.speed) / 24.);
            let step = base_step * speed_scale;
            if let Some(mut velocity) = track.free_velocity {
                let Kind::Free(current) = *kind else {
                    return false;
                };
                let elapsed = elapsed_seconds.clamp(0., 0.1);
                track.free_turn_remaining -= elapsed * self.animation_speed * speed_scale;
                if track.free_turn_remaining <= 0. {
                    let angle = Self::next_animation_random(&mut track.random_state)
                        * std::f64::consts::TAU;
                    velocity = Point {
                        x: angle.cos(),
                        y: angle.sin(),
                    };
                    track.free_turn_remaining +=
                        0.35 + 0.45 * Self::next_animation_random(&mut track.random_state);
                }
                let distance =
                    90. * elapsed * self.animation_speed * speed_scale / f64::from(self.scale);
                let mut destination = Point {
                    x: current.x + velocity.x * distance,
                    y: current.y + velocity.y * distance,
                };
                if let Some((min, max)) = self.animation_bounds {
                    let margin = 10. / f64::from(self.scale);
                    let left = min.x.min(max.x) + margin;
                    let right = min.x.max(max.x) - margin;
                    let top = min.y.min(max.y) + margin;
                    let bottom = min.y.max(max.y) - margin;
                    if left < right && top < bottom {
                        if destination.x < left {
                            destination.x = left;
                            velocity.x = velocity.x.abs();
                        } else if destination.x > right {
                            destination.x = right;
                            velocity.x = -velocity.x.abs();
                        }
                        if destination.y < top {
                            destination.y = top;
                            velocity.y = velocity.y.abs();
                        } else if destination.y > bottom {
                            destination.y = bottom;
                            velocity.y = -velocity.y.abs();
                        }
                    }
                }
                track.free_velocity = Some(velocity);
                let moved = self.doc.move_point(track.id, destination);
                return moved && !track.button_settings.is_some_and(|settings| settings.once);
            }
            let circular = matches!(kind, Kind::PointOnCircle(_, _, _));
            match track.button_settings.map(|settings| settings.direction) {
                Some(AnimationDirection::Forward) => {
                    track.phase = (track.phase + step).rem_euclid(1.);
                    track.distance_travelled += step;
                }
                Some(AnimationDirection::Backward) => {
                    track.phase = (track.phase - step).rem_euclid(1.);
                    track.distance_travelled += step;
                }
                Some(AnimationDirection::Random) => {
                    track.phase = Self::next_animation_random(&mut track.random_state);
                }
                Some(AnimationDirection::Bidirectional) | None => {
                    track.phase += track.direction * step;
                    if circular {
                        track.phase = track.phase.rem_euclid(1.);
                    } else if track.phase > 1. {
                        track.phase = 2. - track.phase;
                        track.direction = -1.;
                    } else if track.phase < 0. {
                        track.phase = -track.phase;
                        track.direction = 1.;
                    }
                }
            }
            let once_complete = track.button_settings.is_some_and(|settings| {
                settings.once
                    && (settings.direction == AnimationDirection::Random
                        || track.distance_travelled >= 1.)
            });
            if once_complete
                && !matches!(
                    track.button_settings,
                    Some(AnimationSettings {
                        direction: AnimationDirection::Random,
                        ..
                    })
                )
            {
                track.phase = track.start_phase;
            }
            let destination = match *kind {
                Kind::PointOnSegment(host, _) => self
                    .doc
                    .line_points(host)
                    .filter(|(_, _, mode)| *mode == LineMode::Segment)
                    .map(|(a, b, _)| Point {
                        x: a.x + (b.x - a.x) * track.phase,
                        y: a.y + (b.y - a.y) * track.phase,
                    }),
                Kind::PointOnCircle(host, _, _) => self.doc.circle_points(host).map(|(c, r)| {
                    let angle = track.phase * std::f64::consts::TAU;
                    Point {
                        x: c.x + r * angle.cos(),
                        y: c.y - r * angle.sin(),
                    }
                }),
                Kind::PointOnArc(host, _) => self.doc.point_on_arc(host, track.phase),
                _ => None,
            };
            let moved = destination.is_some_and(|point| self.doc.move_point(track.id, point));
            moved && !once_complete
        });
        self.animations = tracks;
    }
    fn tick_animation_clock(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.animation_last_tick).as_secs_f64();
        self.animation_last_tick = now;
        self.tick_animation(elapsed);
        self.tick_movement(elapsed);
        self.tick_presentations();
    }
    fn ui(&mut self, root: &mut egui::Ui) -> Viewport {
        if self.pending_action.is_none() {
            if root.input_mut(|input| {
                input.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::S,
                )
            }) {
                self.save(true);
            } else if root
                .input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::S))
            {
                self.save(false);
            }
        }
        if !root.ctx().egui_wants_keyboard_input() && self.pending_action.is_none() {
            if root.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::C)) {
                self.copy_selected();
            }
            if root.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::X)) {
                self.cut_selected();
            }
            if root.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::V)) {
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
                        let targets: Vec<_> = self
                            .selection_order
                            .iter()
                            .copied()
                            .filter(|id| {
                                self.doc.objects.get(id - 1).is_some_and(|object| {
                                object.page == self.doc.active_page
                                    && object.visible
                                    && !object.deleted
                                    && !matches!(
                                        object.kind,
                                        Kind::HideShowButton { .. }
                                            | Kind::AnimationButton { .. }
                                            | Kind::MovementButton { .. }
                                            | Kind::ScrollButton { .. }
                                            | Kind::PresentationButton { .. }
                                            | Kind::LinkUrlButton { .. }
                                            | Kind::LinkPageButton { .. }
                                            | Kind::Unsupported(_)
                                    )
                                })
                            })
                            .collect();
                        if ui
                            .add_enabled(
                                !targets.is_empty() && !self.doc.partial_view,
                                egui::Button::new("Hide/Show"),
                            )
                            .clicked()
                        {
                            match self.doc.add_hide_show_button_for_targets(&targets) {
                                Ok(id) => self.select_only(id),
                                Err(error) => self.status = error,
                            }
                            ui.close();
                        }
                        let animation_targets = self
                            .selection_order
                            .iter()
                            .copied()
                            .filter_map(|id| {
                                self.animation_track(id)?;
                                Some((id, self.doc.animation_button_default_settings(id)?))
                            })
                            .collect::<Vec<_>>();
                        if ui
                            .add_enabled(
                                !animation_targets.is_empty() && !self.doc.partial_view,
                                egui::Button::new("Animation"),
                            )
                            .clicked()
                        {
                            self.animation_button_dialog = Some(AnimationButtonDraft {
                                targets: animation_targets.iter().map(|(id, _)| *id).collect(),
                                settings: animation_targets.iter().map(|(_, value)| *value).collect(),
                                selected: 0,
                            });
                            ui.close();
                        }
                        let movement_pair = match self.selection_order.as_slice() {
                            [source, destination]
                                if matches!(self.doc.objects[*source - 1].kind, Kind::Free(_))
                                    && self.doc.position(*destination).is_some() =>
                            {
                                Some((*source, *destination))
                            }
                            _ => None,
                        };
                        if ui.add_enabled(movement_pair.is_some() && !self.doc.partial_view,
                            egui::Button::new("Movement")).clicked()
                        {
                            let (source, destination) = movement_pair.unwrap();
                            self.movement_button_dialog = Some(MovementButtonDraft {
                                source, destination, speed: MovementSpeed::Medium, follow: true,
                            });
                            ui.close();
                        }
                        let presentation_targets = self.selection_order.iter().copied()
                            .filter(|id| self.doc.objects.get(id - 1).is_some_and(|object| {
                                object.visible && !object.deleted && object.page == self.doc.active_page
                                    && object.kind.is_action_button()
                            }))
                            .collect::<Vec<_>>();
                        if ui.add_enabled(!presentation_targets.is_empty()
                            && presentation_targets.len() == self.selection_order.len()
                            && !self.doc.partial_view,
                            egui::Button::new("Presentation")).clicked()
                        {
                            self.presentation_button_dialog = Some(PresentationButtonDraft {
                                targets: presentation_targets,
                                mode: PresentationMode::Simultaneous,
                            });
                            ui.close();
                        }
                        if ui.add_enabled(
                            (!self.doc.partial_view && self.doc.page_count == 1)
                                || (self.doc.page_count > 1
                                    && !self.doc.objects.iter().any(|object| matches!(object.kind, Kind::Unsupported(_)))),
                            egui::Button::new("Link…")).clicked()
                        {
                            self.link_dialog = Some(LinkButtonDraft {
                                use_page: self.doc.page_count > 1,
                                target_page: usize::from(self.doc.active_page == 0),
                                target_button_ordinal: None,
                                url: "https://".into(),
                            });
                            ui.close();
                        }
                        let scroll_target = match self.selection_order.as_slice() {
                            [target] if self.doc.position(*target).is_some() => Some(*target),
                            _ => None,
                        };
                        if ui.add_enabled(scroll_target.is_some() && !self.doc.partial_view,
                            egui::Button::new("Scroll")).clicked()
                        {
                            self.scroll_button_dialog = Some(ScrollButtonDraft {
                                target: scroll_target.unwrap(),
                                placement: ScrollPlacement::UpperLeft,
                            });
                            ui.close();
                        }
                    });
                    if ui
                        .add_enabled(!self.doc.undo.is_empty(), egui::Button::new("Undo"))
                        .clicked()
                    {
                        self.animations.clear();
                        self.movements.clear();
                        self.presentations.clear();
                        self.doc.undo();
                        self.image_textures.clear();
                        ui.close();
                    }
                    if ui
                        .add_enabled(!self.doc.redo.is_empty(), egui::Button::new("Redo"))
                        .clicked()
                    {
                        self.animations.clear();
                        self.movements.clear();
                        self.presentations.clear();
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
                    if ui.button("Locus").clicked() {
                        self.construct_locus();
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
                    if ui.button("New Named Parameter…").clicked() {
                        self.parameter_name_draft = "a".into();
                        self.parameter_value_draft = "1".into();
                        self.named_parameter_dialog = true;
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
                        let pair_selected = self.doc.selected.len() == 2
                            && ["t[1]", "t[2]"].iter().all(|name| {
                                self.doc.selected.iter().any(|id| {
                                    self.doc.objects.get(id - 1).is_some_and(|object| {
                                        matches!(&object.kind, Kind::Parameter(group) if group.name() == *name)
                                    })
                                })
                            });
                        let result =
                            if pair_selected {
                                self.doc.tabulate_default_pair((10, 71))
                            } else {
                                let source = self.doc.selected.iter().next().copied().unwrap_or(0);
                                let anchor = source
                                    .checked_sub(1)
                                    .and_then(|index| self.doc.objects.get(index))
                                    .and_then(|object| match &object.kind {
                                        Kind::Parameter(group) if group.name() != "t[1]" => {
                                            let bottom = group.bbox().1.1;
                                            bottom.checked_add(8).map(|y| (group.anchor().0, y))
                                        }
                                        _ => None,
                                    })
                                    .unwrap_or((10, 38));
                                self.doc.tabulate_default_parameter(source, anchor)
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
                    let selected_function = if self.doc.selected.len() == 1 {
                        self.doc.selected.iter().copied().next().filter(|&id| {
                            self.doc.objects.get(id - 1).is_some_and(|object| {
                                !object.deleted
                                    && matches!(
                                        object.kind,
                                        Kind::FunctionDefinition(_)
                                            | Kind::DerivativeFunction(_, _)
                                    )
                            })
                        })
                    } else {
                        None
                    };
                    if ui
                        .add_enabled(
                            selected_function.is_some(),
                            egui::Button::new("Define Derivative Function"),
                        )
                        .clicked()
                    {
                        let source = selected_function.unwrap();
                        let uses_trigonometry = match self.doc.objects[source - 1].kind {
                            Kind::FunctionDefinition(expression)
                            | Kind::DerivativeFunction(_, expression) => {
                                expression.uses_trigonometry()
                            }
                            _ => false,
                        };
                        if uses_trigonometry && !self.doc.angle_unit_radians {
                            self.derivative_angle_dialog = Some(source);
                        } else {
                            match self.doc.add_derivative_function(source) {
                                Ok(id) => {
                                    self.select_only(id);
                                    self.status = format!("Created derivative function {id}");
                                }
                                Err(error) => self.status = error,
                            }
                        }
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
                                    | Kind::RatioScaledCircle(_, _, _, _, _)
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
                                            | Kind::ScaledCircle(_, _, _)
                                            | Kind::RatioScaledCircle(_, _, _, _, _),
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
                        self.plot_points_dialog = Some(PlotPointsMode::Cartesian);
                        self.plot_points_first_draft = "1".into();
                        self.plot_points_second_draft = "1".into();
                        ui.close();
                    }
                    if ui.button("Plot Points as (r, θ)").clicked() {
                        self.plot_points_dialog = Some(PlotPointsMode::Polar);
                        self.plot_points_first_draft = "1".into();
                        self.plot_points_second_draft = "1".into();
                        ui.close();
                    }
                    if ui.button("Plot Table Data").clicked() {
                        let table = self.doc.selected.iter().next().copied().unwrap_or(0);
                        self.plot_table_dialog = Some(table);
                        self.plot_table_mode = PlotPointsMode::Cartesian;
                        ui.close();
                    }
                    let selected_function = self.doc.selected.iter().copied().find(|&id| {
                        self.doc.objects.get(id - 1).is_some_and(|o| {
                            !o.deleted
                                && matches!(
                                    o.kind,
                                    Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _)
                                )
                        })
                    });
                    if ui
                        .button(if selected_function.is_some() {
                            "Plot Function"
                        } else {
                            "Plot New Function…"
                        })
                        .clicked()
                    {
                        if let Some(definition) = selected_function {
                            match self.doc.plot_function(definition, self.center) {
                                Ok(id) => {
                                    self.select_only(id);
                                    self.status = format!("Plotted function {id}");
                                }
                                Err(error) => self.status = error,
                            }
                        } else {
                            self.function_dialog = Some(true);
                        }
                        ui.close();
                    }
                    if ui.button("Plot Parametric Curve…").clicked() {
                        let functions = self
                            .doc
                            .objects
                            .iter()
                            .filter(|object| {
                                object.page == self.doc.active_page
                                    && !object.deleted
                                    && matches!(
                                        object.kind,
                                        Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _)
                                    )
                            })
                            .map(|object| object.id)
                            .collect::<Vec<_>>();
                        if functions.len() < 2 {
                            self.status = "Define two functions before plotting a parametric curve".into();
                        } else {
                            let mut chosen = self
                                .selection_order
                                .iter()
                                .copied()
                                .filter(|id| functions.contains(id))
                                .collect::<Vec<_>>();
                            for id in functions {
                                if !chosen.contains(&id) {
                                    chosen.push(id);
                                }
                            }
                            self.parametric_x = chosen[0];
                            self.parametric_y = chosen[1];
                            self.parametric_mode = PlotPointsMode::Cartesian;
                            self.parametric_min_draft = "0".into();
                            self.parametric_max_draft = "1".into();
                            self.parametric_dialog = true;
                        }
                        ui.close();
                    }
                    if ui.button(self.define_coordinate_system_label()).clicked() {
                        self.define_coordinate_system();
                        ui.close();
                    }
                    if ui.button("Mark Coordinate System").clicked() {
                        self.mark_coordinate_system();
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
                    if ui.button("Mark Angle").clicked() {
                        self.mark_angle_selected();
                        ui.close();
                    }
                    if ui.button("Mark Ratio").clicked() {
                        self.mark_ratio_selected();
                        ui.close();
                    }
                    if ui.button("Mark Vector").clicked() {
                        self.mark_vector_selected();
                        ui.close();
                    }
                    if ui.button("Mark Distance").clicked() {
                        self.mark_distance_selected();
                        ui.close();
                    }
                    if ui.button("Translate…").clicked() {
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
                    if ui.button("Rotate…").clicked() {
                        self.rotate_dialog = true;
                        ui.close();
                    }
                });
                ui.menu_button("Display", |ui| {
                    ui.menu_button("Text", |ui| {
                        if ui.button("Font Family…").clicked() {
                            self.text_font_input = self
                                .doc
                                .selected
                                .iter()
                                .find_map(|id| self.doc.text_font_state(*id))
                                .map(|style| style.family)
                                .unwrap_or_else(|| "Tahoma".into());
                            self.text_font_dialog = true;
                            ui.close();
                        }
                        for (label, command) in [
                            ("Bold", TextStyleCommand::Bold),
                            ("Italic", TextStyleCommand::Italic),
                            ("Underline", TextStyleCommand::Underline),
                            ("Increase Size", TextStyleCommand::IncreaseSize),
                            ("Decrease Size", TextStyleCommand::DecreaseSize),
                        ] {
                            if ui.button(label).clicked() {
                                self.status = match self.doc.toggle_selected_text_style(command) {
                                    Ok(count) => format!("Formatted {count} text object(s)"),
                                    Err(error) => error,
                                };
                                ui.close();
                            }
                        }
                    });
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
                            Ok(0) => "Select a geometric object".into(),
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
                    ui.separator();
                    if ui.button("Animate Objects").clicked() {
                        self.animate_selected();
                        ui.close();
                    }
                    if ui
                        .add_enabled(!self.animations.is_empty(), egui::Button::new("Increase Speed"))
                        .clicked()
                    {
                        self.animation_speed = (self.animation_speed * 2.).min(8.);
                        ui.close();
                    }
                    if ui
                        .add_enabled(!self.animations.is_empty(), egui::Button::new("Decrease Speed"))
                        .clicked()
                    {
                        self.animation_speed = (self.animation_speed / 2.).max(0.125);
                        ui.close();
                    }
                    if ui
                        .add_enabled(!self.animations.is_empty() || !self.movements.is_empty(), egui::Button::new("Stop Animation"))
                        .clicked()
                    {
                        self.stop_animation();
                        ui.close();
                    }
                    if ui
                        .button(if self.show_motion_controller {
                            "Hide Motion Controller"
                        } else {
                            "Show Motion Controller"
                        })
                        .clicked()
                    {
                        self.show_motion_controller = !self.show_motion_controller;
                        ui.close();
                    }
                    if ui.button("Fit Sketch").clicked() {
                        self.fit();
                        ui.close();
                    }
                });
            });
        });
        if self.show_motion_controller {
            let mut open = true;
            egui::Window::new("Motion Controller")
                .default_pos(egui::pos2(220., 45.))
                .open(&mut open)
                .show(root.ctx(), |ui| {
                    ui.label(format!("{} point(s)", self.animations.len()));
                    ui.horizontal(|ui| {
                        if ui
                            .button(if self.animation_paused {
                                "Resume"
                            } else {
                                "Pause"
                            })
                            .clicked()
                            && !self.animations.is_empty()
                        {
                            self.animation_paused = !self.animation_paused;
                            self.animation_last_tick = Instant::now();
                        }
                        if ui.button("Stop").clicked() {
                            self.stop_animation();
                        }
                    });
                    ui.add(egui::Slider::new(&mut self.animation_speed, 0.125..=8.).text("Speed"));
                });
            self.show_motion_controller = open;
        }
        egui::Panel::left("tools").min_size(110.).show(root, |ui| {
            ui.heading("Tools");
            if self.doc.page_count > 1 {
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(self.doc.active_page > 0, egui::Button::new("‹"))
                        .clicked()
                    {
                        self.animations.clear();
                        self.movements.clear();
                        self.presentations.clear();
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
                        self.animations.clear();
                        self.movements.clear();
                        self.presentations.clear();
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
            if self.marked_center.is_some()
                || self.marked_mirror.is_some()
                || self.marked_vector.is_some()
                || self.marked_distance.is_some()
                || self.marked_rectangular_distances.is_some()
                || self.marked_angle.is_some()
                || self.marked_ratio.is_some()
            {
                ui.separator();
                if let Some(id) = self.marked_center {
                    ui.label(format!("Marked center: {id}"));
                }
                if let Some(id) = self.marked_mirror {
                    ui.label(format!("Marked mirror: {id}"));
                }
                if let Some((start, end)) = self.marked_vector {
                    ui.label(format!("Marked vector: {start} → {end}"));
                }
                if let Some(id) = self.marked_distance {
                    ui.label(format!("Marked distance: {id}"));
                }
                if let Some((horizontal, vertical)) = self.marked_rectangular_distances {
                    ui.label(format!(
                        "Marked distances: horizontal {horizontal}, vertical {vertical}"
                    ));
                }
                if let Some([first, vertex, last]) = self.marked_angle {
                    ui.label(format!("Marked angle: {first}–{vertex}–{last}"));
                }
                if let Some([origin, base, target]) = self.marked_ratio {
                    ui.label(format!("Marked ratio: {origin}–{base}–{target}"));
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
                    let edit = egui::TextEdit::multiline(&mut self.edit_buffer).show(ui);
                    if let Some(range) = edit.cursor_range {
                        if !range.is_empty() {
                            let chars = range.as_sorted_char_range();
                            self.text_selected_range = Some((id, chars.start.0, chars.end.0));
                        } else if edit.response.has_focus() {
                            self.text_selected_range = None;
                        }
                    }
                    let selected_range = self
                        .text_selected_range
                        .and_then(|(target, start, end)| (target == id).then_some((start, end)));
                    if ui.button("Apply text").clicked() {
                        if let Err(error) = self.doc.edit_text(id, &self.edit_buffer) {
                            self.status = error;
                        }
                    }
                    ui.horizontal(|ui| {
                        for (label, command) in [
                            ("Bold", TextStyleCommand::Bold),
                            ("Italic", TextStyleCommand::Italic),
                            ("Underline", TextStyleCommand::Underline),
                        ] {
                            if ui.button(label).clicked() {
                                let current = match &self.doc.objects[id - 1].kind {
                                    Kind::PlainText { text, .. } => text.as_str(),
                                    _ => "",
                                };
                                self.status = if self.edit_buffer != current {
                                    "Apply the text edit before formatting a selection".into()
                                } else if let Some((start, end)) = selected_range {
                                    match self.doc.format_text_range(id, start, end, command) {
                                        Ok(()) => format!("Formatted characters {start}–{end}"),
                                        Err(error) => error,
                                    }
                                } else {
                                    "Select characters in the text field first".into()
                                };
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        for (label, command) in [
                            ("Larger", TextStyleCommand::IncreaseSize),
                            ("Smaller", TextStyleCommand::DecreaseSize),
                        ] {
                            if ui.button(label).clicked() {
                                let current = match &self.doc.objects[id - 1].kind {
                                    Kind::PlainText { text, .. } => text.as_str(),
                                    _ => "",
                                };
                                self.status = if self.edit_buffer != current {
                                    "Apply the text edit before formatting a selection".into()
                                } else if let Some((start, end)) = selected_range {
                                    match self.doc.format_text_range(id, start, end, command) {
                                        Ok(()) => format!("Formatted characters {start}–{end}"),
                                        Err(error) => error,
                                    }
                                } else {
                                    "Select characters in the text field first".into()
                                };
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.text_font_input)
                                .hint_text("Font family"),
                        );
                        if ui.button("Font for selection").clicked() {
                            let current = match &self.doc.objects[id - 1].kind {
                                Kind::PlainText { text, .. } => text.as_str(),
                                _ => "",
                            };
                            self.status = if self.edit_buffer != current {
                                "Apply the text edit before formatting a selection".into()
                            } else if let Some((start, end)) = selected_range {
                                match self.doc.format_text_range_family(
                                    id,
                                    start,
                                    end,
                                    self.text_font_input.trim(),
                                ) {
                                    Ok(()) => format!("Changed font on characters {start}–{end}"),
                                    Err(error) => error,
                                }
                            } else {
                                "Select characters in the text field first".into()
                            };
                        }
                    });
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
                let reason = if self.doc.page_count > 1
                    && self
                        .doc
                        .objects
                        .iter()
                        .all(|object| !matches!(object.kind, Kind::Unsupported(_)))
                {
                    "Multi-page editing is limited"
                } else {
                    "Partial view: unsupported objects"
                };
                ui.colored_label(egui::Color32::DARK_RED, reason);
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
                                        let anchor = sketchpad_rs::render::number_lines_for_object(
                                            &self.doc, id,
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
                                self.animations.retain(|track| track.id != id);
                                self.movements.retain(|track| track.source != id);
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
                if response.double_clicked()
                    && let Some(p) = response.interact_pointer_pos()
                    && let Some(id) = self.hit(&vp, p)
                {
                    match self.doc.objects[id - 1].kind {
                        Kind::FunctionDefinition(expression) => {
                            self.function_edit_id = Some(id);
                            self.function_dialog = Some(false);
                            self.function_draft = expression
                                .display()
                                .strip_prefix("f(x) = ")
                                .unwrap_or("x")
                                .into();
                        }
                        Kind::FunctionPlot(_, _, min, max, samples)
                        | Kind::ParametricPlot(_, _, _, min, max, samples, _) => {
                            self.plot_range_dialog = Some(id);
                            self.plot_min_draft = min.to_string();
                            self.plot_max_draft = max.to_string();
                            self.plot_samples_draft = samples.to_string();
                        }
                        _ => {}
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
                                run.style != sketchpad_rs::text_object::RichRunStyle::default()
                            })
                        });
                        if has_mixed_runs {
                            let mut line_y = at.y;
                            for row in rows.unwrap().iter() {
                                let line_size = row
                                    .iter()
                                    .map(|run| run.style.size_code.map_or(size, f32::from))
                                    .fold(size, f32::max);
                                let mut x = at.x;
                                for run in row {
                                    let run_size = run.style.size_code.map_or(size, f32::from);
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
                                    let color = egui::Color32::from_gray(35);
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
                                    let position = egui::pos2(x, line_y + line_size - run_size);
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
                    if let Some((_, lines)) =
                        sketchpad_rs::render::number_lines_for_object(&self.doc, object.id)
                        && let Some(number_rect) =
                            sketchpad_rs::render::number_screen_rect_for_object(
                                &vp, &self.doc, object.id,
                            )
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
                        let position = at + egui::vec2(offset.0 as f32, offset.1 as f32);
                        let style = self.doc.text_font_state(object.id);
                        let size = style.as_ref().map_or(13., |style| style.size_code as f32);
                        let font_id = self.canvas_font_id(
                            style.as_ref().map(|style| style.family.as_str()),
                            size,
                        );
                        let bounds = ui.painter().text(
                            position,
                            egui::Align2::LEFT_TOP,
                            name,
                            font_id.clone(),
                            egui::Color32::from_gray(35),
                        );
                        if style.as_ref().is_some_and(|style| style.bold) {
                            ui.painter().text(
                                position + egui::vec2(0.7, 0.),
                                egui::Align2::LEFT_TOP,
                                name,
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
                for (at, label) in sketchpad_rs::render::trigonometric_axis_labels(&self.doc, &vp) {
                    ui.painter().text(
                        at,
                        egui::Align2::CENTER_TOP,
                        label,
                        egui::FontId::proportional(11.),
                        egui::Color32::from_rgb(36, 48, 61),
                    );
                }
                for (at, label, vertical) in
                    sketchpad_rs::render::defined_axis_labels(&self.doc, &vp)
                {
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
                        ui.painter()
                            .rect_filled(rect, 0., egui::Color32::from_rgb(255, 188, 230));
                    }
                    ui.painter().text(
                        rect.min,
                        egui::Align2::LEFT_TOP,
                        text,
                        egui::FontId::proportional(16.),
                        egui::Color32::from_gray(35),
                    );
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
        if let Some(mut draft) = self.animation_button_dialog.clone() {
            let mut create = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("animation_button_settings")).show(root.ctx(), |ui| {
                ui.heading("Animation Button");
                if draft.targets.len() > 1 {
                    ui.label("Choose a point to set its motion:");
                    for (index, target) in draft.targets.iter().enumerate() {
                        ui.selectable_value(&mut draft.selected, index, format!("Point {target}"));
                    }
                    ui.separator();
                }
                let target = draft.targets[draft.selected];
                let settings = &mut draft.settings[draft.selected];
                let free = matches!(self.doc.objects[target - 1].kind, Kind::Free(_));
                let circle = matches!(
                    self.doc.objects[target - 1].kind,
                    Kind::PointOnCircle(_, _, _)
                );
                ui.label(format!("Animate point {target}"));
                if free {
                    ui.label("Free points move at random");
                } else {
                    let direction_name = match (circle, settings.direction) {
                        (true, AnimationDirection::Forward) => "Counterclockwise",
                        (true, AnimationDirection::Backward) => "Clockwise",
                        (_, AnimationDirection::Forward) => "Forward",
                        (_, AnimationDirection::Backward) => "Backward",
                        (_, AnimationDirection::Bidirectional) => "Bidirectional",
                        (_, AnimationDirection::Random) => "Random",
                    };
                    egui::ComboBox::from_label("Direction")
                        .selected_text(direction_name)
                        .show_ui(ui, |ui| {
                            for (direction, name) in [
                                (
                                    AnimationDirection::Forward,
                                    if circle {
                                        "Counterclockwise"
                                    } else {
                                        "Forward"
                                    },
                                ),
                                (
                                    AnimationDirection::Backward,
                                    if circle { "Clockwise" } else { "Backward" },
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
                    .add_animation_button_for_targets_with_settings(&values)
                {
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
        if let Some(mut draft) = self.movement_button_dialog {
            let mut create = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("movement_button_settings")).show(root.ctx(), |ui| {
                ui.heading("Movement Button");
                ui.label(format!(
                    "Move point {} toward point {}",
                    draft.source, draft.destination
                ));
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
                match self.doc.add_movement_button(
                    draft.source,
                    draft.destination,
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
                ui.label("Value: -99.99 to 99.99, up to two decimal places");
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
            let edit_id = self.function_edit_id;
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("new_function")).show(root.ctx(), |ui| {
                ui.heading(if edit_id.is_some() {
                    "Edit function"
                } else if plot {
                    "Plot new function"
                } else {
                    "New function"
                });
                ui.label("f(x) =");
                ui.text_edit_singleline(&mut self.function_draft);
                ui.label("Supported: arithmetic in x, sin(x), cos(x), tan(x)");
                ui.horizontal(|ui| {
                    apply = ui
                        .button(if edit_id.is_some() {
                            "Update"
                        } else if plot {
                            "Plot"
                        } else {
                            "Create"
                        })
                        .clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.function_dialog = None;
                self.function_edit_id = None;
            }
            if apply {
                let result = SimpleFunction::parse(&self.function_draft).and_then(|expr| {
                    if expr.uses_trigonometry() && !self.doc.angle_unit_radians {
                        self.function_dialog = None;
                        self.function_angle_dialog = Some((expr, plot, edit_id));
                        return Ok(None);
                    }
                    let id = if let Some(id) = edit_id {
                        self.doc.set_function_expression(id, expr)?;
                        id
                    } else if plot {
                        self.doc.plot_new_function(expr, self.center)?
                    } else {
                        self.doc.add_function_definition(expr)?
                    };
                    Ok(Some(id))
                });
                match result {
                    Ok(None) => {}
                    Ok(Some(id)) => {
                        self.function_dialog = None;
                        self.function_edit_id = None;
                        self.select_only(id);
                        self.status = if edit_id.is_some() {
                            format!("Updated function object {id}")
                        } else {
                            format!("Created function object {id}")
                        };
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if let Some((expression, plot, edit_id)) = self.function_angle_dialog {
            let mut choice = None;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("function_angle_unit")).show(root.ctx(), |ui| {
                ui.heading("Angle unit for this function");
                ui.label("The document currently uses degrees.");
                ui.horizontal(|ui| {
                    if ui.button("Switch to radians").clicked() {
                        choice = Some(true);
                    }
                    if ui.button("Keep degrees").clicked() {
                        choice = Some(false);
                    }
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.function_angle_dialog = None;
                self.function_edit_id = None;
            }
            if let Some(radians) = choice {
                let result = if let Some(id) = edit_id {
                    self.doc
                        .set_function_expression_with_angle_unit(id, expression, radians)
                        .map(|()| id)
                } else if plot {
                    self.doc
                        .plot_new_function_with_angle_unit(expression, self.center, radians)
                } else {
                    self.doc
                        .add_function_definition_with_angle_unit(expression, radians)
                };
                match result {
                    Ok(id) => {
                        self.function_angle_dialog = None;
                        self.function_edit_id = None;
                        self.select_only(id);
                        self.status = if edit_id.is_some() {
                            format!("Updated function object {id}")
                        } else {
                            format!("Created function object {id}")
                        };
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if let Some(source) = self.derivative_angle_dialog {
            let mut choice = None;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("derivative_angle_unit")).show(root.ctx(), |ui| {
                ui.heading("Angle unit for this derivative");
                ui.label("The document currently uses degrees.");
                ui.horizontal(|ui| {
                    if ui.button("Switch to radians").clicked() {
                        choice = Some(true);
                    }
                    if ui.button("Keep degrees").clicked() {
                        choice = Some(false);
                    }
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.derivative_angle_dialog = None;
            }
            if let Some(radians) = choice {
                match self
                    .doc
                    .add_derivative_function_with_angle_unit(source, radians)
                {
                    Ok(id) => {
                        self.derivative_angle_dialog = None;
                        self.select_only(id);
                        self.status = format!("Created derivative function {id}");
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if let Some(plot) = self.plot_range_dialog {
            let variable = if matches!(
                self.doc.objects.get(plot - 1).map(|o| &o.kind),
                Some(Kind::FunctionPlot(_, _, _, _, _))
            ) {
                "x"
            } else {
                "t"
            };
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("plot_range")).show(root.ctx(), |ui| {
                ui.heading("Plot properties");
                ui.horizontal(|ui| {
                    ui.label("Domain");
                    ui.add(egui::TextEdit::singleline(&mut self.plot_min_draft).desired_width(70.));
                    ui.label(format!("≤ {variable} ≤"));
                    ui.add(egui::TextEdit::singleline(&mut self.plot_max_draft).desired_width(70.));
                });
                ui.horizontal(|ui| {
                    ui.label("Samples");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.plot_samples_draft).desired_width(70.),
                    );
                });
                ui.horizontal(|ui| {
                    apply = ui.button("Apply").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.plot_range_dialog = None;
            }
            if apply {
                let result = (|| -> Result<(), String> {
                    let min = self
                        .plot_min_draft
                        .trim()
                        .parse::<f64>()
                        .map_err(|_| "Invalid domain start")?;
                    let max = self
                        .plot_max_draft
                        .trim()
                        .parse::<f64>()
                        .map_err(|_| "Invalid domain end")?;
                    let samples = self
                        .plot_samples_draft
                        .trim()
                        .parse::<u32>()
                        .map_err(|_| "Invalid sample count")?;
                    self.doc.set_plot_range(plot, min, max, samples)
                })();
                match result {
                    Ok(()) => {
                        self.plot_range_dialog = None;
                        self.select_only(plot);
                        self.status = format!("Updated plot {plot}");
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if self.parametric_dialog {
            let functions = self
                .doc
                .objects
                .iter()
                .filter_map(|object| {
                    if object.page != self.doc.active_page || object.deleted {
                        return None;
                    }
                    let text = self.doc.function_display_text(object.id)?;
                    Some((object.id, format!("{}: {text}", object.id)))
                })
                .collect::<Vec<_>>();
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("parametric_curve")).show(root.ctx(), |ui| {
                ui.heading("Plot parametric curve");
                ui.horizontal(|ui| {
                    ui.label("x(t) or r(t)");
                    egui::ComboBox::from_id_salt("parametric_x")
                        .selected_text(
                            functions
                                .iter()
                                .find(|(id, _)| *id == self.parametric_x)
                                .map_or("Select function", |(_, label)| label.as_str()),
                        )
                        .show_ui(ui, |ui| {
                            for (id, label) in &functions {
                                ui.selectable_value(&mut self.parametric_x, *id, label);
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.label("y(t) or θ(t)");
                    egui::ComboBox::from_id_salt("parametric_y")
                        .selected_text(
                            functions
                                .iter()
                                .find(|(id, _)| *id == self.parametric_y)
                                .map_or("Select function", |(_, label)| label.as_str()),
                        )
                        .show_ui(ui, |ui| {
                            for (id, label) in &functions {
                                ui.selectable_value(&mut self.parametric_y, *id, label);
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut self.parametric_mode,
                        PlotPointsMode::Cartesian,
                        "Rectangular",
                    );
                    ui.radio_value(&mut self.parametric_mode, PlotPointsMode::Polar, "Polar");
                });
                ui.horizontal(|ui| {
                    ui.label("Domain");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.parametric_min_draft)
                            .desired_width(65.),
                    );
                    ui.label("≤ t ≤");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.parametric_max_draft)
                            .desired_width(65.),
                    );
                });
                ui.horizontal(|ui| {
                    apply = ui.button("Plot").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.parametric_dialog = false;
            }
            if apply {
                let result = (|| -> Result<usize, String> {
                    let min = self
                        .parametric_min_draft
                        .trim()
                        .parse::<f64>()
                        .map_err(|_| "Invalid domain start")?;
                    let max = self
                        .parametric_max_draft
                        .trim()
                        .parse::<f64>()
                        .map_err(|_| "Invalid domain end")?;
                    self.doc.plot_parametric_curve(
                        self.parametric_x,
                        self.parametric_y,
                        self.parametric_mode,
                        min,
                        max,
                        self.center,
                    )
                })();
                match result {
                    Ok(id) => {
                        self.parametric_dialog = false;
                        self.select_only(id);
                        self.status = format!("Plotted parametric curve {id}");
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
                                | Kind::RatioScaledCircle(_, _, _, _, _)
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
        if let Some(mode) = self.plot_points_dialog {
            let mut apply = false;
            let mut done = false;
            egui::Modal::new(egui::Id::new("plot_points")).show(root.ctx(), |ui| {
                ui.heading("Plot Points");
                ui.horizontal(|ui| {
                    ui.label(if mode == PlotPointsMode::Cartesian {
                        "X"
                    } else {
                        "Radius"
                    });
                    ui.text_edit_singleline(&mut self.plot_points_first_draft);
                });
                ui.horizontal(|ui| {
                    ui.label(if mode == PlotPointsMode::Cartesian {
                        "Y"
                    } else {
                        "Angle (degrees)"
                    });
                    ui.text_edit_singleline(&mut self.plot_points_second_draft);
                });
                ui.horizontal(|ui| {
                    apply = ui.button("Plot").clicked();
                    done = ui.button("Done").clicked();
                });
            });
            if done {
                self.plot_points_dialog = None;
            }
            if apply {
                let coordinates = [
                    self.plot_points_first_draft.trim().parse::<f64>(),
                    self.plot_points_second_draft.trim().parse::<f64>(),
                ];
                let result = match coordinates {
                    [Ok(first), Ok(second)] => self.doc.plot_points_at(mode, [first, second]),
                    _ => Err("Enter two numeric coordinates".into()),
                };
                match result {
                    Ok(id) => {
                        self.select_only(id);
                        self.status = format!("Plotted point {id}");
                        self.plot_points_first_draft.clear();
                        self.plot_points_second_draft.clear();
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if let Some(table) = self.plot_table_dialog {
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("plot_table_data")).show(root.ctx(), |ui| {
                ui.heading("Plot Table Data");
                ui.label("First coordinate: t₂");
                ui.label("Second coordinate: t₁");
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut self.plot_table_mode,
                        PlotPointsMode::Cartesian,
                        "Cartesian (x, y)",
                    );
                    ui.radio_value(
                        &mut self.plot_table_mode,
                        PlotPointsMode::Polar,
                        "Polar (r, θ in degrees)",
                    );
                });
                ui.horizontal(|ui| {
                    apply = ui.button("Plot").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if cancel {
                self.plot_table_dialog = None;
            }
            if apply {
                match self.doc.plot_pair_table_data(table, self.plot_table_mode) {
                    Ok(ids) => {
                        self.plot_table_dialog = None;
                        if let Some(id) = ids.last() {
                            self.select_only(*id);
                        }
                        self.status = format!("Plotted {} table data point(s)", ids.len());
                    }
                    Err(error) => self.status = error,
                }
            }
        }
        if self.text_font_dialog {
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("text_font_family")).show(root.ctx(), |ui| {
                ui.heading("Font family");
                ui.text_edit_singleline(&mut self.text_font_input);
                ui.horizontal(|ui| {
                    apply = ui.button("Apply").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if apply {
                let family = self.text_font_input.trim();
                match self.doc.set_selected_text_family(family) {
                    Ok(count) => {
                        self.status = format!("Changed font family on {count} text object(s)");
                        self.text_font_dialog = false;
                    }
                    Err(error) => self.status = error,
                }
            } else if cancel {
                self.text_font_dialog = false;
            }
        }
        if self.translate_dialog {
            let mut angle = self.translation_angle;
            let mut distance = self.translation_distance;
            let mut horizontal = self.translation_horizontal;
            let mut vertical = self.translation_vertical;
            let mut mode = self.translation_mode;
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("polar_translation")).show(root.ctx(), |ui| {
                ui.heading("Translate point");
                ui.radio_value(
                    &mut mode,
                    TranslationMode::FixedPolar,
                    "Polar: fixed distance",
                );
                ui.radio_value(
                    &mut mode,
                    TranslationMode::FixedRectangular,
                    "Rectangular: fixed distances",
                );
                ui.add_enabled_ui(self.marked_distance.is_some(), |ui| {
                    ui.radio_value(
                        &mut mode,
                        TranslationMode::MarkedDistance,
                        "Polar: marked distance",
                    );
                });
                ui.add_enabled_ui(self.marked_rectangular_distances.is_some(), |ui| {
                    ui.radio_value(
                        &mut mode,
                        TranslationMode::RectangularMarked,
                        "Rectangular: marked distances",
                    );
                    ui.radio_value(
                        &mut mode,
                        TranslationMode::RectangularHorizontalFixed,
                        "Rectangular: fixed horizontal, marked vertical",
                    );
                    ui.radio_value(
                        &mut mode,
                        TranslationMode::RectangularVerticalFixed,
                        "Rectangular: marked horizontal, fixed vertical",
                    );
                });
                ui.add_enabled_ui(self.marked_vector.is_some(), |ui| {
                    ui.radio_value(&mut mode, TranslationMode::MarkedVector, "Marked vector");
                });
                if mode == TranslationMode::MarkedVector {
                    if let Some((start, end)) = self.marked_vector {
                        ui.label(format!("Point {start} → point {end}"));
                    }
                } else if mode == TranslationMode::RectangularMarked {
                    if let Some((horizontal, vertical)) = self.marked_rectangular_distances {
                        ui.label(format!(
                            "Horizontal distance {horizontal}; vertical distance {vertical}"
                        ));
                    }
                } else if mode == TranslationMode::RectangularHorizontalFixed {
                    if let Some((_, vertical_id)) = self.marked_rectangular_distances {
                        ui.label(format!("Vertical distance value {vertical_id}"));
                    }
                    ui.horizontal(|ui| {
                        ui.label("Horizontal (cm)");
                        ui.add(egui::DragValue::new(&mut horizontal).speed(0.1));
                    });
                } else if mode == TranslationMode::RectangularVerticalFixed {
                    if let Some((horizontal_id, _)) = self.marked_rectangular_distances {
                        ui.label(format!("Horizontal distance value {horizontal_id}"));
                    }
                    ui.horizontal(|ui| {
                        ui.label("Vertical (cm)");
                        ui.add(egui::DragValue::new(&mut vertical).speed(0.1));
                    });
                } else if mode == TranslationMode::FixedRectangular {
                    ui.horizontal(|ui| {
                        ui.label("Horizontal (cm)");
                        ui.add(egui::DragValue::new(&mut horizontal).speed(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Vertical (cm)");
                        ui.add(egui::DragValue::new(&mut vertical).speed(0.1));
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Angle (degrees)");
                        ui.add(egui::DragValue::new(&mut angle).speed(1.));
                    });
                    if mode == TranslationMode::MarkedDistance {
                        if let Some(id) = self.marked_distance {
                            ui.label(format!("Distance value {id}"));
                        }
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("Distance (cm)");
                            ui.add(egui::DragValue::new(&mut distance).speed(0.1));
                        });
                    }
                }
                ui.horizontal(|ui| {
                    apply = ui.button("Translate").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            self.translation_angle = angle;
            self.translation_distance = distance;
            self.translation_horizontal = horizontal;
            self.translation_vertical = vertical;
            self.translation_mode = mode;
            if apply {
                self.translate_dialog = false;
                self.translate_selected();
            } else if cancel {
                self.translate_dialog = false;
            }
        }
        if self.dilate_dialog {
            let mut factor = self.dilation_factor;
            let mut use_marked = self.dilation_use_marked;
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("dilation_ratio")).show(root.ctx(), |ui| {
                ui.heading("Dilate object");
                ui.radio_value(&mut use_marked, false, "Fixed ratio");
                ui.add_enabled_ui(self.marked_ratio.is_some(), |ui| {
                    ui.radio_value(&mut use_marked, true, "Marked ratio");
                });
                if use_marked {
                    if let Some([origin, base, target]) = self.marked_ratio {
                        ui.label(format!("Point {origin} → {base} → {target}"));
                    }
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Ratio");
                        ui.add(egui::DragValue::new(&mut factor).speed(0.05));
                    });
                }
                ui.horizontal(|ui| {
                    apply = ui.button("Dilate").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            self.dilation_factor = factor;
            self.dilation_use_marked = use_marked;
            if apply {
                self.dilate_dialog = false;
                self.dilate_selected();
            } else if cancel {
                self.dilate_dialog = false;
            }
        }
        if self.rotate_dialog {
            let mut angle = self.rotation_angle;
            let mut use_marked = self.rotation_use_marked;
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("point_rotation")).show(root.ctx(), |ui| {
                ui.heading("Rotate");
                ui.radio_value(&mut use_marked, false, "Fixed angle");
                ui.add_enabled_ui(self.marked_angle.is_some(), |ui| {
                    ui.radio_value(&mut use_marked, true, "Marked angle");
                });
                if use_marked {
                    if let Some([first, vertex, last]) = self.marked_angle {
                        ui.label(format!("Point {first} → {vertex} → {last}"));
                    }
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Angle (degrees)");
                        ui.add(egui::DragValue::new(&mut angle).speed(1.));
                    });
                }
                ui.horizontal(|ui| {
                    apply = ui.button("Rotate").clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            self.rotation_angle = angle;
            self.rotation_use_marked = use_marked;
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
    fn define_coordinate_system_uses_selected_distances_in_order() {
        let doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/graph_two_segments_base.gsp"),
            None,
        )
        .unwrap();
        let mut editor = Editor::new(doc);
        editor.selection_order = vec![1, 3, 6];
        editor.doc.selected.extend([1, 3, 6]);
        assert_eq!(
            editor.define_coordinate_system_label(),
            "Define Unit Distances"
        );
        editor.define_coordinate_system();
        assert_eq!(editor.doc.active_coordinate_system(), Some(9));
        assert_eq!(editor.doc.objects[6].kind, Kind::DistanceAxis(1, 3, 1));
        assert_eq!(editor.doc.objects[7].kind, Kind::DistanceAxis(1, 6, 0));
        assert_eq!(editor.doc.grid_form[0], GridForm::Rectangular);
    }

    #[test]
    fn clicking_a_grid_intersection_selects_its_coordinate_system() {
        let doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/graph_two_coordinate_systems_original.gsp"),
            None,
        )
        .unwrap();
        let mut editor = Editor::new(doc);
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 199., y: 125. },
        };
        for (frame_id, x, y) in [(9, 2., 2.), (14, 2., 3.)] {
            let at = editor
                .doc
                .graph_frame(frame_id)
                .unwrap()
                .world(x, y)
                .unwrap();
            let click = viewport.screen(Point { x: at.x, y: at.y });
            assert_eq!(editor.hit(&viewport, click), Some(frame_id));
        }
        editor.select_only(9);
        editor.mark_coordinate_system();
        assert_eq!(editor.doc.active_coordinate_system(), Some(9));
        editor.doc.mark_coordinate_system(1).unwrap();
        editor.doc.set_grid_visible(false);
        let at = editor.doc.graph_frame(14).unwrap().world(2., 3.).unwrap();
        assert_ne!(
            editor.hit_grid(&viewport, viewport.screen(Point { x: at.x, y: at.y })),
            Some(14)
        );

        let mixed = Editor::new(
            Document::from_bytes(
                include_bytes!("../tests/fixtures/graph_two_frames_mixed_grid_generated.gsp"),
                None,
            )
            .unwrap(),
        );
        let radius = 3. / std::f64::consts::SQRT_2;
        let at = mixed
            .doc
            .graph_frame(14)
            .unwrap()
            .world(radius, radius)
            .unwrap();
        assert_eq!(
            mixed.hit_grid(&viewport, viewport.screen(Point { x: at.x, y: at.y })),
            Some(14)
        );
    }

    #[test]
    fn clicking_function_label_selects_definition_for_plotting() {
        let doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/function_definition_identity.gsp"),
            None,
        )
        .unwrap();
        let mut editor = Editor::new(doc);
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 199., y: 125. },
        };
        let label = viewport.rect.min + egui::vec2(20., 20.);
        editor.click(&viewport, label, false);
        assert_eq!(
            editor.doc.selected.iter().copied().collect::<Vec<_>>(),
            vec![7]
        );
        let plot = editor
            .doc
            .plot_function(7, Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(plot, 8);
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );

        let mut mixed = Document::new();
        let a = mixed.add(Kind::Free(Point { x: 100., y: 100. }));
        let b = mixed.add(Kind::Free(Point { x: 150., y: 100. }));
        let segment = mixed.add(Kind::Segment(a, b));
        mixed.add(Kind::SegmentLength(segment));
        let definition = mixed
            .add_function_definition(SimpleFunction::Square)
            .unwrap();
        let mut editor = Editor::new(mixed);
        assert_eq!(editor.overlay_rows().len(), 2);
        editor.click(&viewport, viewport.rect.min + egui::vec2(20., 40.), false);
        assert_eq!(
            editor.doc.selected.iter().copied().collect::<Vec<_>>(),
            vec![definition]
        );
    }

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
            TraceMark::Straight(Point { x: 35., y: 45. }, _, LineMode::Segment, _)
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
    fn traced_circle_arc_and_polygon_capture_geometry_changes() {
        let mut editor = Editor::new(Document::new());
        let a = editor.doc.add(Kind::Free(Point { x: 20., y: 30. }));
        let b = editor.doc.add(Kind::Free(Point { x: 80., y: 30. }));
        let c = editor.doc.add(Kind::Free(Point { x: 45., y: 70. }));
        let circle = editor.doc.add(Kind::Circle(a, b));
        let arc = editor.doc.add(Kind::ThreePointArc(a, c, b));
        let polygon = editor.doc.add(Kind::Polygon(vec![a, b, c]));
        editor.doc.selected.extend([circle, arc, polygon]);
        assert_eq!(editor.doc.set_selected_tracing(true).unwrap(), 3);
        editor.capture_traces();
        assert_eq!(editor.trace_marks.len(), 3);
        assert!(matches!(editor.trace_marks[0], TraceMark::Circle(_, _, _)));
        assert!(matches!(editor.trace_marks[1], TraceMark::Arc(_, _)));
        assert!(matches!(editor.trace_marks[2], TraceMark::Polygon(_, _)));
        assert!(editor.doc.move_point(b, Point { x: 90., y: 40. }));
        editor.capture_traces();
        assert_eq!(editor.trace_marks.len(), 6);
        match (&editor.trace_marks[0], &editor.trace_marks[3]) {
            (TraceMark::Circle(_, first, _), TraceMark::Circle(_, second, _)) => {
                assert_ne!(first, second);
            }
            _ => panic!("Expected two circle trace samples"),
        }
        match (&editor.trace_marks[1], &editor.trace_marks[4]) {
            (TraceMark::Arc(first, _), TraceMark::Arc(second, _)) => {
                assert_ne!(first, second);
            }
            _ => panic!("Expected two arc trace samples"),
        }
        match (&editor.trace_marks[2], &editor.trace_marks[5]) {
            (TraceMark::Polygon(first, _), TraceMark::Polygon(second, _)) => {
                assert_ne!(first, second);
            }
            _ => panic!("Expected two polygon trace samples"),
        }
    }

    #[test]
    fn traced_ray_and_line_capture_modes_and_motion() {
        let mut editor = Editor::new(Document::new());
        let a = editor.doc.add(Kind::Free(Point { x: 20., y: 30. }));
        let b = editor.doc.add(Kind::Free(Point { x: 80., y: 30. }));
        let ray = editor.doc.add(Kind::Ray(a, b));
        let line = editor.doc.add(Kind::Line(a, b));
        editor.doc.selected.extend([ray, line]);
        assert_eq!(editor.doc.set_selected_tracing(true).unwrap(), 2);
        editor.capture_traces();
        assert_eq!(editor.trace_marks.len(), 2);
        assert!(matches!(
            editor.trace_marks[0],
            TraceMark::Straight(_, _, LineMode::Ray, _)
        ));
        assert!(matches!(
            editor.trace_marks[1],
            TraceMark::Straight(_, _, LineMode::Line, _)
        ));
        assert!(editor.doc.move_point(a, Point { x: 35., y: 45. }));
        editor.capture_traces();
        assert!(matches!(
            editor.trace_marks[2],
            TraceMark::Straight(Point { x: 35., y: 45. }, _, LineMode::Ray, _)
        ));
        assert!(matches!(
            editor.trace_marks[3],
            TraceMark::Straight(Point { x: 35., y: 45. }, _, LineMode::Line, _)
        ));
    }

    #[test]
    fn presentation_buttons_activate_parents_and_wait_for_motion() {
        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/presentation_two_hide_show_base.gsp"),
            None,
        )
        .unwrap();
        let presentation = doc
            .add_presentation_button_for_targets(&[3, 4], PresentationMode::Simultaneous)
            .unwrap();
        let mut editor = Editor::new(doc);
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: editor.center,
        };
        let Kind::PresentationButton { label, anchor, .. } =
            &editor.doc.objects[presentation - 1].kind
        else {
            panic!()
        };
        let click = editor
            .action_button_rect(&viewport, *anchor, label)
            .center();
        editor.click(&viewport, click, false);
        assert!(!editor.doc.objects[0].visible);
        assert!(!editor.doc.objects[1].visible);
        assert!(editor.presentations.is_empty());

        let mut doc = Document::new();
        let source = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let destination = doc.add(Kind::Free(Point { x: 120., y: 0. }));
        let movement = doc
            .add_movement_button(source, destination, MovementSpeed::Medium, true)
            .unwrap();
        let hide = doc.add_hide_show_button(destination).unwrap();
        let presentation = doc
            .add_presentation_button_for_targets(&[movement, hide], PresentationMode::Sequential)
            .unwrap();
        let mut editor = Editor::new(doc);
        editor.scale = 1.;
        editor.viewport_rect = Some(viewport.rect);
        editor
            .activate_presentation_button(presentation, &viewport, 0)
            .unwrap();
        assert_eq!(editor.presentations.len(), 1);
        assert!(editor.doc.objects[destination - 1].visible);
        for _ in 0..9 {
            editor.tick_movement(0.1);
            editor.tick_presentations();
        }
        assert_eq!(editor.doc.position(source), Some(Point { x: 120., y: 0. }));
        assert!(!editor.doc.objects[destination - 1].visible);
        assert!(editor.presentations.is_empty());
    }

    #[test]
    fn sketchdoc_link_button_opens_sibling_document() {
        let path = std::path::Path::new("tests/fixtures/action_link_url_port_sketchdoc.gsp");
        let mut editor = Editor::new(Document::open(path).unwrap());
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
            scale: editor.scale,
            center: editor.center,
        };
        let Kind::LinkUrlButton { label, anchor, .. } = &editor.doc.objects[1].kind else {
            panic!();
        };
        let click = editor
            .action_button_rect(&viewport, *anchor, label)
            .center();
        editor.click(&viewport, click, false);
        assert_eq!(editor.doc.objects.len(), 1);
        assert_eq!(
            editor.doc.path.as_ref().unwrap().file_name().unwrap(),
            "point.gsp"
        );
    }

    #[test]
    fn page_link_buttons_switch_pages() {
        for (bytes, source_page, destination_page) in [
            (
                include_bytes!("../tests/fixtures/action_link_page_default_original.gsp")
                    .as_slice(),
                1,
                0,
            ),
            (
                include_bytes!("../tests/fixtures/action_link_page_two_original.gsp").as_slice(),
                0,
                1,
            ),
            (
                include_bytes!("../tests/fixtures/action_link_page_port_generated.gsp").as_slice(),
                1,
                0,
            ),
        ] {
            let mut editor = Editor::new(Document::from_bytes(bytes, None).unwrap());
            editor.doc.active_page = source_page;
            editor.fit();
            let viewport = Viewport {
                rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
                scale: editor.scale,
                center: editor.center,
            };
            let Kind::LinkPageButton { label, anchor, .. } = &editor.doc.objects[1].kind else {
                panic!();
            };
            let click = editor
                .action_button_rect(&viewport, *anchor, label)
                .center();
            editor.click(&viewport, click, false);
            assert_eq!(editor.doc.active_page, destination_page);
        }
    }

    #[test]
    fn page_link_activates_destination_hide_show_button() {
        for (index, bytes) in [
            include_bytes!(
                "../tests/fixtures/action_link_page_activate_hide_show_native_created.gsp"
            )
            .as_slice(),
            include_bytes!(
                "../tests/fixtures/action_link_page_activate_hide_show_port_generated.gsp"
            )
            .as_slice(),
        ]
        .into_iter()
        .enumerate()
        {
            let mut editor = Editor::new(Document::from_bytes(bytes, None).unwrap());
            let path = std::env::temp_dir().join(format!(
                "sketchpad_rs_link_click_save_{}_{}.gsp",
                std::process::id(),
                index
            ));
            editor.doc.path = Some(path.clone());
            editor.doc.active_page = 1;
            editor.fit();
            let viewport = Viewport {
                rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
                scale: editor.scale,
                center: editor.center,
            };
            let Kind::LinkPageButton { label, anchor, .. } = &editor.doc.objects[3].kind else {
                panic!();
            };
            let click = editor
                .action_button_rect(&viewport, *anchor, label)
                .center();
            editor.click(&viewport, click, false);
            assert_eq!(editor.doc.active_page, 0);
            assert!(!editor.doc.objects[0].visible);
            assert!(matches!(
                editor.doc.objects[1].kind,
                Kind::HideShowButton {
                    action: sketchpad_rs::action_records::VisibilityAction::Show,
                    ..
                }
            ));
            assert!(editor.save(false), "{}", editor.status);
            let reopened = Document::open(&path).unwrap();
            assert!(!reopened.objects[0].visible);
            assert!(matches!(
                reopened.objects[1].kind,
                Kind::HideShowButton {
                    action: sketchpad_rs::action_records::VisibilityAction::Show,
                    ..
                }
            ));
            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn scroll_buttons_place_target_at_corner_or_center() {
        let mut doc = Document::new();
        let point = doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let corner = doc
            .add_scroll_button(point, ScrollPlacement::UpperLeft)
            .unwrap();
        let centered = doc
            .add_scroll_button(point, ScrollPlacement::Center)
            .unwrap();
        let mut editor = Editor::new(doc);
        editor.scale = 2.;
        let viewport = |center| Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 2.,
            center,
        };
        let v = viewport(editor.center);
        let Kind::ScrollButton { label, anchor, .. } = &editor.doc.objects[corner - 1].kind else {
            panic!()
        };
        let click = sketchpad_rs::render::action_button_rect(&v, *anchor, label).center();
        editor.click(&v, click, false);
        assert_eq!(editor.center, Point { x: 250., y: 195. });
        let scrolled = viewport(editor.center);
        let Kind::ScrollButton { label, anchor, .. } = &editor.doc.objects[corner - 1].kind else {
            panic!()
        };
        assert_eq!(
            editor.action_button_rect(&scrolled, *anchor, label).min,
            egui::pos2(-190., -145.)
        );
        editor.fit();
        editor.scale = 2.;
        let v = viewport(editor.center);
        let Kind::ScrollButton { label, anchor, .. } = &editor.doc.objects[centered - 1].kind
        else {
            panic!()
        };
        let click = sketchpad_rs::render::action_button_rect(&v, *anchor, label).center();
        editor.click(&v, click, false);
        assert_eq!(editor.center, Point { x: 150., y: 120. });
    }

    #[test]
    fn movement_button_reaches_instant_and_initial_destinations() {
        let mut doc = Document::new();
        let source = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let destination = doc.add(Kind::Free(Point { x: 120., y: 0. }));
        let instant = doc
            .add_movement_button(source, destination, MovementSpeed::Instant, true)
            .unwrap();
        let mut editor = Editor::new(doc);
        editor.scale = 1.;
        editor.activate_movement_button(instant);
        assert_eq!(editor.doc.position(source), Some(Point { x: 120., y: 0. }));
        assert!(editor.movements.is_empty());

        editor.doc.move_point(source, Point { x: 0., y: 0. });
        let initial = editor
            .doc
            .add_movement_button(source, destination, MovementSpeed::Medium, false)
            .unwrap();
        editor.activate_movement_button(initial);
        editor.doc.move_point(destination, Point { x: 200., y: 0. });
        for _ in 0..10 {
            editor.tick_movement(0.1);
        }
        assert_eq!(editor.doc.position(source), Some(Point { x: 120., y: 0. }));
        assert!(editor.movements.is_empty());
    }

    #[test]
    fn clicking_animation_button_starts_its_target_without_controller() {
        let doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/animation_button_base.gsp"),
            None,
        )
        .unwrap();
        let mut editor = Editor::new(doc);
        let button = editor.doc.add_animation_button(4).unwrap();
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 250., y: 150. },
        };
        let Kind::AnimationButton { label, anchor, .. } = &editor.doc.objects[button - 1].kind
        else {
            panic!("Expected animation button")
        };
        let click = sketchpad_rs::render::action_button_rect(&viewport, *anchor, label).center();
        let before = editor.doc.position(4).unwrap();
        editor.click(&viewport, click, false);
        assert_eq!(editor.animations.len(), 1);
        assert!(!editor.show_motion_controller);
        editor.tick_animation(0.1);
        assert_ne!(editor.doc.position(4).unwrap(), before);
    }

    #[test]
    fn animation_button_direction_speed_once_and_second_click_control_motion() {
        let base = include_bytes!("../tests/fixtures/animation_button_base.gsp");
        for (direction, speed, expected_phase) in [
            (AnimationDirection::Forward, 24., 0.55),
            (AnimationDirection::Backward, 24., 0.45),
            (AnimationDirection::Forward, 80., 0.5 + 0.05 * 80. / 24.),
            (AnimationDirection::Forward, 4., 0.5 + 0.05 * 4. / 24.),
        ] {
            let mut editor = Editor::new(Document::from_bytes(base, None).unwrap());
            let settings = AnimationSettings {
                direction,
                speed,
                once: false,
            };
            editor
                .doc
                .add_animation_button_with_settings(4, settings)
                .unwrap();
            editor.animate_button_targets(&[4], &[settings]);
            editor.tick_animation(0.1);
            assert!((editor.animations[0].phase - expected_phase).abs() < 1e-9);
            editor.animate_button_targets(&[4], &[settings]);
            assert!(editor.animations.is_empty());
        }
        let mut editor = Editor::new(Document::from_bytes(base, None).unwrap());
        let settings = AnimationSettings {
            direction: AnimationDirection::Forward,
            speed: 80.,
            once: true,
        };
        editor.animate_button_targets(&[4], &[settings]);
        for _ in 0..8 {
            editor.tick_animation(0.1);
        }
        assert!(editor.animations.is_empty());
        assert!((editor.doc.position(4).unwrap().x - 250.).abs() < 1e-9);
    }

    #[test]
    fn two_point_animation_button_starts_and_stops_both_tracks() {
        let doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/animation_button_two_points_base.gsp"),
            None,
        )
        .unwrap();
        let mut editor = Editor::new(doc);
        let settings = [
            AnimationSettings::default(),
            AnimationSettings {
                direction: AnimationDirection::Backward,
                ..AnimationSettings::default()
            },
        ];
        let button = editor
            .doc
            .add_animation_button_for_targets_with_settings(&[(4, settings[0]), (5, settings[1])])
            .unwrap();
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 250., y: 150. },
        };
        let Kind::AnimationButton { label, anchor, .. } = &editor.doc.objects[button - 1].kind
        else {
            panic!("Expected Animation button")
        };
        let click = sketchpad_rs::render::action_button_rect(&viewport, *anchor, label).center();
        editor.click(&viewport, click, false);
        assert_eq!(editor.animations.len(), 2);
        assert!(!editor.show_motion_controller);
        editor.tick_animation(0.1);
        assert!((editor.doc.position(4).unwrap().x - 235.).abs() < 1e-9);
        assert!((editor.doc.position(5).unwrap().x - 265.).abs() < 1e-9);
        editor.click(&viewport, click, false);
        assert!(editor.animations.is_empty());
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

    #[test]
    fn marked_vector_drives_dynamic_translation() {
        let mut editor = Editor::new(Document::new());
        let start = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let end = editor.doc.add(Kind::Free(Point { x: 220., y: 130. }));
        let source = editor.doc.add(Kind::Free(Point { x: 100., y: 200. }));
        let before = editor.doc.encode().unwrap();

        editor.select_only(start);
        editor.selection_order.push(end);
        editor.doc.selected.insert(end);
        editor.mark_vector_selected();
        assert_eq!(editor.marked_vector, Some((start, end)));
        assert!(editor.selection_order.is_empty());
        assert_eq!(editor.doc.encode().unwrap(), before);

        editor.select_only(source);
        editor.translate_selected();
        let image = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects[image - 1].kind,
            Kind::VectorTranslatedPoint(source, start, end)
        );
        assert_eq!(editor.doc.position(image), Some(Point { x: 170., y: 210. }));
        assert!(editor.doc.move_point(end, Point { x: 250., y: 130. }));
        assert_eq!(editor.doc.position(image), Some(Point { x: 200., y: 210. }));
    }

    #[test]
    fn marked_distance_drives_dynamic_polar_translation() {
        let mut editor = Editor::new(Document::new());
        let start = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let end = editor.doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let source = editor.doc.add(Kind::Free(Point { x: 100., y: 220. }));
        let distance = editor.doc.add(Kind::PointDistance(start, end));
        let before = editor.doc.encode().unwrap();

        editor.select_only(distance);
        editor.mark_distance_selected();
        assert_eq!(editor.marked_distance, Some(distance));
        assert_eq!(editor.translation_mode, TranslationMode::MarkedDistance);
        assert!(editor.selection_order.is_empty());
        assert_eq!(editor.doc.encode().unwrap(), before);

        editor.select_only(source);
        editor.translate_selected();
        let image = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects[image - 1].kind,
            Kind::MarkedDistanceTranslatedPoint(source, distance, 90.)
        );
        assert_eq!(editor.doc.position(image), Some(Point { x: 100., y: 120. }));
        assert!(editor.doc.move_point(end, Point { x: 300., y: 120. }));
        let moved = editor.doc.position(image).unwrap();
        assert!((moved.x - 100.).abs() < 1e-9 && (moved.y - 70.).abs() < 1e-9);
    }
    #[test]
    fn marked_distance_translates_selected_segment_and_circle() {
        let mut editor = Editor::new(Document::new());
        let measure_start = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let measure_end = editor.doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let source_start = editor.doc.add(Kind::Free(Point { x: 100., y: 220. }));
        let source_end = editor.doc.add(Kind::Free(Point { x: 200., y: 220. }));
        let distance = editor
            .doc
            .add(Kind::PointDistance(measure_start, measure_end));
        let segment = editor.doc.add(Kind::Segment(source_start, source_end));
        let circle = editor.doc.add(Kind::Circle(source_start, source_end));
        editor.marked_distance = Some(distance);
        editor.translation_mode = TranslationMode::MarkedDistance;

        editor.select_only(segment);
        editor.translate_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::MarkedDistanceTranslatedSegment(segment, distance, 90.)
        );
        editor.select_only(circle);
        editor.translate_selected();
        let translated_circle = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::MarkedDistanceTranslatedCircle(circle, distance, 90.)
        );
        assert!(
            editor
                .doc
                .move_point(measure_end, Point { x: 300., y: 120. })
        );
        let (center, _) = editor.doc.circle_points(translated_circle).unwrap();
        assert!((center.y - 70.).abs() < 1e-9);
    }

    #[test]
    fn two_marked_distances_drive_rectangular_translation() {
        let mut editor = Editor::new(Document::new());
        let h_start = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let h_end = editor.doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let v_start = editor.doc.add(Kind::Free(Point { x: 150., y: 180. }));
        let v_end = editor.doc.add(Kind::Free(Point { x: 150., y: 230. }));
        let source = editor.doc.add(Kind::Free(Point { x: 100., y: 210. }));
        let horizontal = editor.doc.add(Kind::PointDistance(h_start, h_end));
        let vertical = editor.doc.add(Kind::PointDistance(v_start, v_end));
        let before = editor.doc.encode().unwrap();

        editor.select_only(horizontal);
        editor.doc.selected.insert(vertical);
        editor.selection_order.push(vertical);
        editor.mark_distance_selected();
        assert_eq!(
            editor.marked_rectangular_distances,
            Some((horizontal, vertical))
        );
        assert_eq!(editor.translation_mode, TranslationMode::RectangularMarked);
        assert!(editor.selection_order.is_empty());
        assert_eq!(editor.doc.encode().unwrap(), before);

        editor.select_only(source);
        editor.translate_selected();
        let image = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects[image - 1].kind,
            Kind::RectangularMarkedTranslatedPoint(source, horizontal, vertical)
        );
        assert_eq!(editor.doc.position(image), Some(Point { x: 200., y: 160. }));
    }

    #[test]
    fn fixed_rectangular_translation_uses_signed_offsets() {
        let mut editor = Editor::new(Document::new());
        let source = editor.doc.add(Kind::Free(Point { x: 100., y: 210. }));
        editor.select_only(source);
        editor.translation_mode = TranslationMode::FixedRectangular;
        editor.translation_horizontal = 2.;
        editor.translation_vertical = -0.5;
        editor.translate_selected();
        let image = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects[image - 1].kind,
            Kind::FixedRectangularTranslatedPoint(
                source,
                37.79527559055118,
                2.,
                37.79527559055118,
                -0.5
            )
        );
        let translated = editor.doc.position(image).unwrap();
        assert!((translated.x - 175.59055118110236).abs() < 1e-9);
        assert!((translated.y - 228.89763779527559).abs() < 1e-9);
    }

    #[test]
    fn mixed_rectangular_translation_uses_the_other_marked_axis() {
        let mut editor = Editor::new(Document::new());
        let h_start = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let h_end = editor.doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let v_start = editor.doc.add(Kind::Free(Point { x: 150., y: 180. }));
        let v_end = editor.doc.add(Kind::Free(Point { x: 150., y: 230. }));
        let source = editor.doc.add(Kind::Free(Point { x: 100., y: 210. }));
        let horizontal = editor.doc.add(Kind::PointDistance(h_start, h_end));
        let vertical = editor.doc.add(Kind::PointDistance(v_start, v_end));
        editor.marked_rectangular_distances = Some((horizontal, vertical));
        editor.translation_horizontal = 2.;
        editor.translation_vertical = -0.5;

        editor.select_only(source);
        editor.translation_mode = TranslationMode::RectangularHorizontalFixed;
        editor.translate_selected();
        let fixed_x = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects[fixed_x - 1].kind,
            Kind::MixedRectangularTranslatedPoint(source, vertical, 2., true)
        );

        editor.select_only(source);
        editor.translation_mode = TranslationMode::RectangularVerticalFixed;
        editor.translate_selected();
        let fixed_y = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects[fixed_y - 1].kind,
            Kind::MixedRectangularTranslatedPoint(source, horizontal, -0.5, false)
        );
        assert!(editor.doc.move_point(h_end, Point { x: 300., y: 120. }));
        assert!(editor.doc.move_point(v_end, Point { x: 150., y: 250. }));
        let fixed_x_position = editor.doc.position(fixed_x).unwrap();
        let fixed_y_position = editor.doc.position(fixed_y).unwrap();
        assert!((fixed_x_position.x - (100. + 2. * 37.79527559055118)).abs() < 1e-9);
        assert_eq!(fixed_x_position.y, 140.);
        assert_eq!(fixed_y_position.x, 250.);
        assert!((fixed_y_position.y - (210. + 0.5 * 37.79527559055118)).abs() < 1e-9);
    }

    #[test]
    fn fixed_polar_translation_accepts_a_segment() {
        let mut editor = Editor::new(Document::new());
        let start = editor.doc.add(Kind::Free(Point { x: 100., y: 200. }));
        let end = editor.doc.add(Kind::Free(Point { x: 200., y: 200. }));
        let segment = editor.doc.add(Kind::Segment(start, end));
        editor.select_only(segment);
        editor.translation_mode = TranslationMode::FixedPolar;
        editor.translation_angle = 90.;
        editor.translation_distance = 1.;
        editor.translate_selected();
        let translated = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects[translated - 1].kind,
            Kind::PolarTranslatedSegment(segment, 90., 37.79527559055118, 1.)
        );
        let (a, b, mode) = editor.doc.line_points(translated).unwrap();
        assert_eq!(mode, sketchpad_rs::document::LineMode::Segment);
        assert!((a.y - (200. - 37.79527559055118)).abs() < 1e-9);
        assert!((b.y - (200. - 37.79527559055118)).abs() < 1e-9);
    }
    #[test]
    fn fixed_polar_circle_and_rectangular_segment_translation() {
        let mut editor = Editor::new(Document::new());
        let center = editor.doc.add(Kind::Free(Point { x: 100., y: 200. }));
        let radius_point = editor.doc.add(Kind::Free(Point { x: 150., y: 200. }));
        let circle = editor.doc.add(Kind::Circle(center, radius_point));
        let end = editor.doc.add(Kind::Free(Point { x: 200., y: 200. }));
        let segment = editor.doc.add(Kind::Segment(center, end));

        editor.select_only(circle);
        editor.translation_mode = TranslationMode::FixedPolar;
        editor.translate_selected();
        let translated_circle = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects[translated_circle - 1].kind,
            Kind::PolarTranslatedCircle(circle, 90., 37.79527559055118, 1.)
        );
        let (translated_center, radius) = editor.doc.circle_points(translated_circle).unwrap();
        assert!((translated_center.y - (200. - 37.79527559055118)).abs() < 1e-9);
        assert_eq!(radius, 50.);

        editor.select_only(segment);
        editor.translation_mode = TranslationMode::FixedRectangular;
        editor.translation_horizontal = 2.;
        editor.translation_vertical = -0.5;
        editor.translate_selected();
        let translated_segment = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects[translated_segment - 1].kind,
            Kind::FixedRectangularTranslatedSegment(
                segment,
                37.79527559055118,
                2.,
                37.79527559055118,
                -0.5
            )
        );
        let (a, b, _) = editor.doc.line_points(translated_segment).unwrap();
        assert!((a.x - (100. + 2. * 37.79527559055118)).abs() < 1e-9);
        assert!((b.y - (200. + 0.5 * 37.79527559055118)).abs() < 1e-9);
    }
    #[test]
    fn rectangular_circle_and_vector_shape_translation() {
        let mut editor = Editor::new(Document::new());
        let start = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let end = editor.doc.add(Kind::Free(Point { x: 220., y: 130. }));
        let segment = editor.doc.add(Kind::Segment(start, end));
        let center = editor.doc.add(Kind::Free(Point { x: 100., y: 200. }));
        let radius_point = editor.doc.add(Kind::Free(Point { x: 150., y: 200. }));
        let circle = editor.doc.add(Kind::Circle(center, radius_point));
        editor.marked_vector = Some((start, end));

        editor.select_only(segment);
        editor.translation_mode = TranslationMode::MarkedVector;
        editor.translate_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::VectorTranslatedSegment(segment, start, end)
        );

        editor.select_only(circle);
        editor.translate_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::VectorTranslatedCircle(circle, start, end)
        );

        editor.select_only(circle);
        editor.translation_mode = TranslationMode::FixedRectangular;
        editor.translate_selected();
        let translated = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::FixedRectangularTranslatedCircle(
                circle,
                37.79527559055118,
                1.,
                37.79527559055118,
                1.
            )
        );
        assert!(editor.doc.circle_points(translated).is_some());
    }

    #[test]
    fn marked_angle_drives_dynamic_rotation() {
        let mut editor = Editor::new(Document::new());
        let center = editor.doc.add(Kind::Free(Point { x: 130., y: 150. }));
        let first = editor.doc.add(Kind::Free(Point { x: 230., y: 160. }));
        let vertex = editor.doc.add(Kind::Free(Point { x: 230., y: 220. }));
        let last = editor.doc.add(Kind::Free(Point { x: 290., y: 220. }));
        let source = editor.doc.add(Kind::Free(Point { x: 170., y: 150. }));
        let before = editor.doc.encode().unwrap();

        editor.select_only(center);
        editor.mark_center_selected();
        editor.select_only(first);
        editor.selection_order.extend([vertex, last]);
        editor.doc.selected.extend([vertex, last]);
        editor.mark_angle_selected();
        assert_eq!(editor.marked_angle, Some([first, vertex, last]));
        assert!(editor.selection_order.is_empty());
        assert_eq!(editor.doc.encode().unwrap(), before);

        editor.select_only(source);
        editor.rotate_selected();
        let image = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects[image - 1].kind,
            Kind::AngleRotatedPoint(source, center, first, vertex, last)
        );
        assert_eq!(editor.doc.position(image), Some(Point { x: 130., y: 190. }));
        assert!(editor.doc.move_point(last, Point { x: 290., y: 190. }));
        assert_ne!(editor.doc.position(image), Some(Point { x: 130., y: 190. }));
    }

    #[test]
    fn marked_angle_rotates_selected_segment_and_circle() {
        let mut editor = Editor::new(Document::new());
        let center = editor.doc.add(Kind::Free(Point { x: 130., y: 150. }));
        let first = editor.doc.add(Kind::Free(Point { x: 230., y: 160. }));
        let vertex = editor.doc.add(Kind::Free(Point { x: 230., y: 220. }));
        let last = editor.doc.add(Kind::Free(Point { x: 290., y: 220. }));
        let a = editor.doc.add(Kind::Free(Point { x: 170., y: 190. }));
        let b = editor.doc.add(Kind::Free(Point { x: 210., y: 190. }));
        let segment = editor.doc.add(Kind::Segment(a, b));
        let circle = editor.doc.add(Kind::Circle(a, b));
        editor.marked_center = Some(center);
        editor.marked_angle = Some([first, vertex, last]);
        editor.rotation_use_marked = true;
        editor.select_only(segment);
        editor.rotate_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::AngleRotatedSegment(segment, center, first, vertex, last)
        );
        editor.select_only(circle);
        editor.rotate_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::AngleRotatedCircle(circle, center, first, vertex, last)
        );
        assert_eq!(
            editor
                .doc
                .circle_points(editor.doc.objects.len())
                .unwrap()
                .1,
            40.
        );
    }

    #[test]
    fn fixed_angle_rotates_selected_segment_and_circle() {
        let mut editor = Editor::new(Document::new());
        let center = editor.doc.add(Kind::Free(Point { x: 130., y: 150. }));
        let a = editor.doc.add(Kind::Free(Point { x: 170., y: 190. }));
        let b = editor.doc.add(Kind::Free(Point { x: 210., y: 190. }));
        let segment = editor.doc.add(Kind::Segment(a, b));
        let circle = editor.doc.add(Kind::Circle(a, b));
        editor.marked_center = Some(center);
        editor.rotation_angle = 90.;
        editor.rotation_use_marked = false;
        editor.select_only(segment);
        editor.rotate_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::RotatedSegment(segment, center, 90.)
        );
        editor.select_only(circle);
        editor.rotate_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::RotatedCircle(circle, center, 90.)
        );
        assert_eq!(
            editor
                .doc
                .circle_points(editor.doc.objects.len())
                .unwrap()
                .1,
            40.
        );
    }

    #[test]
    fn segment_reflection_and_dilation_commands() {
        let mut editor = Editor::new(Document::new());
        let center = editor.doc.add(Kind::Free(Point { x: 100., y: 120. }));
        let origin = editor.doc.add(Kind::Free(Point { x: 180., y: 100. }));
        let base = editor.doc.add(Kind::Free(Point { x: 230., y: 100. }));
        let target = editor.doc.add(Kind::Free(Point { x: 205., y: 100. }));
        let a = editor.doc.add(Kind::Free(Point { x: 140., y: 120. }));
        let b = editor.doc.add(Kind::Free(Point { x: 160., y: 120. }));
        let segment = editor.doc.add(Kind::Segment(a, b));
        let mirror_a = editor.doc.add(Kind::Free(Point { x: 100., y: 100. }));
        let mirror_b = editor.doc.add(Kind::Free(Point { x: 100., y: 220. }));
        let mirror = editor.doc.add(Kind::Segment(mirror_a, mirror_b));
        editor.marked_center = Some(center);
        editor.dilation_use_marked = false;
        editor.dilation_factor = 0.5;
        editor.select_only(segment);
        editor.dilate_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::ScaledSegment(segment, center, 0.5)
        );
        editor.marked_ratio = Some([origin, base, target]);
        editor.dilation_use_marked = true;
        editor.select_only(segment);
        editor.dilate_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::RatioScaledSegment(segment, center, origin, base, target)
        );
        editor.marked_mirror = Some(mirror);
        editor.select_only(segment);
        editor.reflect_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::ReflectedSegment(segment, mirror)
        );
    }

    #[test]
    fn polygon_interior_reflection_command() {
        let mut editor = Editor::new(Document::new());
        let a = editor.doc.add(Kind::Free(Point { x: 150., y: 100. }));
        let b = editor.doc.add(Kind::Free(Point { x: 150., y: 230. }));
        let mirror = editor.doc.add(Kind::Segment(a, b));
        let p = editor.doc.add(Kind::Free(Point { x: 200., y: 130. }));
        let q = editor.doc.add(Kind::Free(Point { x: 240., y: 180. }));
        let r = editor.doc.add(Kind::Free(Point { x: 200., y: 230. }));
        let polygon = editor.doc.add(Kind::Polygon(vec![p, q, r]));
        editor.marked_mirror = Some(mirror);
        editor.select_only(polygon);
        editor.reflect_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::ReflectedPolygon(polygon, mirror)
        );
        assert_eq!(
            editor
                .doc
                .polygon_points(editor.doc.objects.len())
                .unwrap()
                .len(),
            3
        );
    }

    #[test]
    fn transformed_circles_are_selectable_on_canvas() {
        let mut editor = Editor::new(Document::new());
        let center = editor.doc.add(Kind::Free(Point { x: 100., y: 100. }));
        let source_center = editor.doc.add(Kind::Free(Point { x: 180., y: 100. }));
        let radius_point = editor.doc.add(Kind::Free(Point { x: 200., y: 100. }));
        let circle = editor.doc.add(Kind::Circle(source_center, radius_point));
        let rotated = editor.doc.add(Kind::RotatedCircle(circle, center, 90.));
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 150., y: 100. },
        };
        assert_eq!(
            editor.hit(&viewport, viewport.screen(Point { x: 120., y: 20. })),
            Some(rotated)
        );
    }

    #[test]
    fn polygon_locus_interior_is_selectable_on_canvas() {
        let base = include_bytes!("../tests/fixtures/locus_polygon_target_base.gsp");
        let mut document = Document::from_bytes(base, None).unwrap();
        let locus = document.construct_locus(4, 7).unwrap();
        let editor = Editor::new(document);
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 210., y: 140. },
        };
        assert_eq!(
            editor.hit(&viewport, viewport.screen(Point { x: 140., y: 150. })),
            Some(locus)
        );
        assert_eq!(
            editor.hit(
                &viewport,
                viewport.screen(Point {
                    x: 180.72,
                    y: 179.9
                })
            ),
            Some(locus)
        );
    }

    #[test]
    fn polygon_rotation_and_dilation_commands_use_live_interiors() {
        let mut editor = Editor::new(Document::new());
        let center = editor.doc.add(Kind::Free(Point { x: 100., y: 120. }));
        let first = editor.doc.add(Kind::Free(Point { x: 230., y: 160. }));
        let vertex = editor.doc.add(Kind::Free(Point { x: 230., y: 220. }));
        let last = editor.doc.add(Kind::Free(Point { x: 290., y: 220. }));
        let a = editor.doc.add(Kind::Free(Point { x: 140., y: 150. }));
        let b = editor.doc.add(Kind::Free(Point { x: 180., y: 150. }));
        let c = editor.doc.add(Kind::Free(Point { x: 160., y: 190. }));
        let polygon = editor.doc.add(Kind::Polygon(vec![a, b, c]));
        let origin = editor.doc.add(Kind::Free(Point { x: 180., y: 100. }));
        let base = editor.doc.add(Kind::Free(Point { x: 230., y: 100. }));
        let target = editor.doc.add(Kind::Free(Point { x: 205., y: 100. }));
        editor.marked_center = Some(center);
        editor.marked_angle = Some([first, vertex, last]);
        editor.marked_ratio = Some([origin, base, target]);

        editor.rotation_angle = 90.;
        editor.select_only(polygon);
        editor.rotate_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::RotatedPolygon(polygon, center, 90.)
        );

        editor.rotation_use_marked = true;
        editor.select_only(polygon);
        editor.rotate_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::AngleRotatedPolygon(polygon, center, first, vertex, last)
        );

        editor.dilation_factor = 0.5;
        editor.select_only(polygon);
        editor.dilate_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::ScaledPolygon(polygon, center, 0.5)
        );

        editor.dilation_use_marked = true;
        editor.select_only(polygon);
        editor.dilate_selected();
        let image = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::RatioScaledPolygon(polygon, center, origin, base, target)
        );
        let before = editor.doc.polygon_points(image).unwrap();
        assert!(editor.doc.move_point(target, Point { x: 255., y: 100. }));
        assert_ne!(editor.doc.polygon_points(image).unwrap(), before);
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }

    #[test]
    fn marked_vector_translates_selected_polygon() {
        let mut editor = Editor::new(Document::new());
        let start = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let end = editor.doc.add(Kind::Free(Point { x: 220., y: 130. }));
        let a = editor.doc.add(Kind::Free(Point { x: 100., y: 200. }));
        let b = editor.doc.add(Kind::Free(Point { x: 140., y: 200. }));
        let c = editor.doc.add(Kind::Free(Point { x: 120., y: 240. }));
        let source = editor.doc.add(Kind::Polygon(vec![a, b, c]));
        editor.marked_vector = Some((start, end));
        editor.translation_mode = TranslationMode::MarkedVector;
        editor.select_only(source);
        editor.translate_selected();
        let image = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::VectorTranslatedPolygon(source, start, end)
        );
        assert_eq!(
            editor.doc.polygon_points(image).unwrap()[0],
            Point { x: 170., y: 210. }
        );
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );

        editor.translation_mode = TranslationMode::FixedPolar;
        editor.translation_angle = 90.;
        editor.translation_distance = 1.;
        editor.select_only(source);
        editor.translate_selected();
        let polar = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::PolarTranslatedPolygon(source, 90., 37.79527559055118, 1.)
        );
        assert!(
            (editor.doc.polygon_points(polar).unwrap()[0].y - (200. - 37.79527559055118)).abs()
                < 1e-9
        );

        editor.translation_mode = TranslationMode::FixedRectangular;
        editor.translation_horizontal = 2.;
        editor.translation_vertical = -0.5;
        editor.select_only(source);
        editor.translate_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::FixedRectangularTranslatedPolygon(
                source,
                37.79527559055118,
                2.,
                37.79527559055118,
                -0.5
            )
        );
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }

    #[test]
    fn marked_distance_translates_selected_polygon() {
        let mut editor = Editor::new(Document::new());
        let first = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let last = editor.doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let a = editor.doc.add(Kind::Free(Point { x: 100., y: 190. }));
        let b = editor.doc.add(Kind::Free(Point { x: 140., y: 190. }));
        let c = editor.doc.add(Kind::Free(Point { x: 120., y: 230. }));
        let distance = editor.doc.add(Kind::PointDistance(first, last));
        let source = editor.doc.add(Kind::Polygon(vec![a, b, c]));
        editor.marked_distance = Some(distance);
        editor.translation_mode = TranslationMode::MarkedDistance;
        editor.translation_angle = 90.;
        editor.select_only(source);
        editor.translate_selected();
        let image = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::MarkedDistanceTranslatedPolygon(source, distance, 90.)
        );
        assert!((editor.doc.polygon_points(image).unwrap()[0].y - 90.).abs() < 1e-9);
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }

    #[test]
    fn area_command_measures_selected_polygon() {
        let mut editor = Editor::new(Document::new());
        let a = editor.doc.add(Kind::Free(Point { x: 170., y: 190. }));
        let b = editor.doc.add(Kind::Free(Point { x: 210., y: 190. }));
        let c = editor.doc.add(Kind::Free(Point { x: 190., y: 230. }));
        let polygon = editor.doc.add(Kind::Polygon(vec![a, b, c]));
        editor.select_only(polygon);
        editor.measure_circle(CircleMeasure::Area);
        let area = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects[area - 1].kind,
            Kind::PolygonArea(polygon)
        );
        assert!(
            editor
                .overlay_rows()
                .iter()
                .any(|(_, text)| text == "Area = 0.56 cm²")
        );
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );

        let rotated = editor.doc.add(Kind::RotatedPolygon(polygon, a, 90.));
        editor.select_only(rotated);
        editor.measure_perimeter();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::PolygonPerimeter(rotated)
        );
        editor.select_only(rotated);
        editor.measure_circle(CircleMeasure::Area);
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::PolygonArea(rotated)
        );
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }

    #[test]
    fn rectangular_marked_and_mixed_translate_selected_polygon() {
        let mut editor = Editor::new(Document::new());
        let a = editor.doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let b = editor.doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let c = editor.doc.add(Kind::Free(Point { x: 150., y: 180. }));
        let d = editor.doc.add(Kind::Free(Point { x: 150., y: 230. }));
        let horizontal = editor.doc.add(Kind::PointDistance(a, b));
        let vertical = editor.doc.add(Kind::PointDistance(c, d));
        let p = editor.doc.add(Kind::Free(Point { x: 100., y: 190. }));
        let q = editor.doc.add(Kind::Free(Point { x: 140., y: 190. }));
        let r = editor.doc.add(Kind::Free(Point { x: 120., y: 230. }));
        let source = editor.doc.add(Kind::Polygon(vec![p, q, r]));
        editor.marked_rectangular_distances = Some((horizontal, vertical));

        editor.translation_mode = TranslationMode::RectangularMarked;
        editor.select_only(source);
        editor.translate_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::RectangularMarkedTranslatedPolygon(source, horizontal, vertical)
        );

        editor.translation_horizontal = 2.;
        editor.translation_mode = TranslationMode::RectangularHorizontalFixed;
        editor.select_only(source);
        editor.translate_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::MixedRectangularTranslatedPolygon(source, vertical, 2., true)
        );

        editor.translation_vertical = 2.;
        editor.translation_mode = TranslationMode::RectangularVerticalFixed;
        editor.select_only(source);
        editor.translate_selected();
        assert_eq!(
            editor.doc.objects.last().unwrap().kind,
            Kind::MixedRectangularTranslatedPolygon(source, horizontal, 2., false)
        );
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }

    #[test]
    fn marked_ratio_drives_dynamic_dilation() {
        let mut editor = Editor::new(Document::new());
        let center = editor.doc.add(Kind::Free(Point { x: 100., y: 120. }));
        let origin = editor.doc.add(Kind::Free(Point { x: 180., y: 100. }));
        let base = editor.doc.add(Kind::Free(Point { x: 230., y: 100. }));
        let target = editor.doc.add(Kind::Free(Point { x: 205., y: 100. }));
        let source = editor.doc.add(Kind::Free(Point { x: 140., y: 120. }));
        let before = editor.doc.encode().unwrap();

        editor.select_only(center);
        editor.mark_center_selected();
        editor.select_only(origin);
        editor.selection_order.extend([base, target]);
        editor.doc.selected.extend([base, target]);
        editor.mark_ratio_selected();
        assert_eq!(editor.marked_ratio, Some([origin, base, target]));
        assert!(editor.selection_order.is_empty());
        assert_eq!(editor.doc.encode().unwrap(), before);

        editor.select_only(source);
        editor.dilate_selected();
        let image = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects[image - 1].kind,
            Kind::RatioScaledPoint(source, center, origin, base, target)
        );
        assert_eq!(editor.doc.position(image), Some(Point { x: 120., y: 120. }));
        assert!(editor.doc.move_point(target, Point { x: 255., y: 100. }));
        assert_eq!(editor.doc.position(image), Some(Point { x: 160., y: 120. }));

        let radius_point = editor.doc.add(Kind::Free(Point { x: 160., y: 120. }));
        let circle = editor.doc.add(Kind::Circle(source, radius_point));
        editor.select_only(circle);
        editor.dilate_selected();
        let circle_image = editor.doc.objects.len();
        assert_eq!(
            editor.doc.objects[circle_image - 1].kind,
            Kind::RatioScaledCircle(circle, center, origin, base, target)
        );
        assert_eq!(
            editor.doc.circle_points(circle_image),
            Some((Point { x: 160., y: 120. }, 30.))
        );
    }

    #[test]
    fn animation_moves_constrained_points_and_their_dependents() {
        let mut editor = Editor::new(Document::new());
        let a = editor.doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = editor.doc.add(Kind::Free(Point { x: 100., y: 0. }));
        let path = editor.doc.add(Kind::Segment(a, b));
        let moving = editor.doc.add(Kind::PointOnSegment(path, 0.25));
        editor.doc.add(Kind::Segment(a, moving));
        let midpoint = editor.doc.add(Kind::Midpoint(a, moving));
        let center = editor.doc.add(Kind::Free(Point { x: 200., y: 200. }));
        let edge = editor.doc.add(Kind::Free(Point { x: 220., y: 200. }));
        let circle = editor.doc.add(Kind::Circle(center, edge));
        let orbit = editor.doc.add(Kind::PointOnCircle(circle, 1., 0.));
        editor.doc.selected.extend([moving, orbit]);
        editor.animate_selected();
        assert_eq!(editor.animations.len(), 2);
        for _ in 0..10 {
            editor.tick_animation(0.1);
        }
        assert!((editor.doc.position(moving).unwrap().x - 75.).abs() < 1e-9);
        assert!((editor.doc.position(midpoint).unwrap().x - 37.5).abs() < 1e-9);
        assert!((editor.doc.position(orbit).unwrap().x - 180.).abs() < 1e-9);
        assert!((editor.doc.position(orbit).unwrap().y - 200.).abs() < 1e-9);
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
        editor.stop_animation();
        editor.doc.undo();
        assert_eq!(editor.doc.position(moving), Some(Point { x: 25., y: 0. }));
        assert_eq!(editor.doc.position(orbit), Some(Point { x: 220., y: 200. }));
    }

    #[test]
    fn arc_animation_bounces_and_pause_holds_position() {
        let mut editor = Editor::new(Document::new());
        let a = editor.doc.add(Kind::Free(Point { x: 100., y: 0. }));
        let b = editor.doc.add(Kind::Free(Point { x: 70., y: 70. }));
        let c = editor.doc.add(Kind::Free(Point { x: 0., y: 100. }));
        let arc = editor.doc.add(Kind::ThreePointArc(a, b, c));
        let moving = editor.doc.add(Kind::PointOnArc(arc, 0.9));
        editor.select_only(moving);
        editor.animate_selected();
        editor.animation_speed = 2.;
        editor.tick_animation(0.1);
        assert!((editor.animations[0].phase - 1.).abs() < 1e-9);
        editor.tick_animation(0.1);
        assert!((editor.animations[0].phase - 0.9).abs() < 1e-9);
        assert_eq!(editor.animations[0].direction, -1.);
        let at = editor.doc.position(moving);
        editor.animation_paused = true;
        editor.tick_animation(0.1);
        assert_eq!(editor.doc.position(moving), at);
        editor.stop_animation();
        assert!(editor.animations.is_empty());
    }

    #[test]
    fn animation_stopped_file_from_desktop_roundtrips() {
        let bytes = include_bytes!("../tests/fixtures/animation_segment_locus_saved_by_port.gsp");
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        assert_eq!(document.encode().unwrap(), bytes);
        let Kind::PointOnSegment(3, t) = document.objects[3].kind else {
            panic!("Animated point lost its segment constraint");
        };
        assert!((0.6..0.9).contains(&t));
        assert!((document.position(4).unwrap().x - (120. + 180. * t)).abs() < 1e-9);
    }

    #[test]
    fn free_point_animation_wanders_within_canvas_and_saves() {
        let mut editor = Editor::new(Document::new());
        let moving = editor.doc.add(Kind::Free(Point { x: 100., y: 100. }));
        editor.animation_bounds = Some((Point { x: 0., y: 0. }, Point { x: 200., y: 200. }));
        editor.select_only(moving);
        editor.animate_selected();
        assert!(editor.animations[0].free_velocity.is_some());
        for _ in 0..150 {
            editor.tick_animation(0.1);
            let at = editor.doc.position(moving).unwrap();
            assert!((10.0..=190.).contains(&at.x));
            assert!((10.0..=190.).contains(&at.y));
        }
        let stopped_at = editor.doc.position(moving).unwrap();
        assert_ne!(stopped_at, Point { x: 100., y: 100. });
        editor.animation_paused = true;
        editor.tick_animation(0.1);
        assert_eq!(editor.doc.position(moving), Some(stopped_at));
        editor.stop_animation();
        let reopened = Document::from_bytes(&editor.doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(moving), Some(stopped_at));
        editor.doc.undo();
        assert_eq!(
            editor.doc.position(moving),
            Some(Point { x: 100., y: 100. })
        );
    }

    #[test]
    fn free_point_animation_desktop_file_roundtrips() {
        let bytes = include_bytes!("../tests/fixtures/animation_free_point_saved_by_port.gsp");
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        assert_eq!(document.encode().unwrap(), bytes);
        let at = document.position(1).unwrap();
        assert_ne!(at, Point { x: 200., y: 150. });
        assert!((at.x - 169.69428734643552).abs() < 1e-9);
        assert!((at.y - 146.60481682616518).abs() < 1e-9);
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
    let mut next_animation_frame = Instant::now();
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
                        editor.tick_animation_clock();
                        let input = egui_state.take_egui_input(&window);
                        editor.sync_document_fonts(&ctx);
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
                            &editor.trace_marks,
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
            Event::AboutToWait => {
                if (!editor.animations.is_empty() && !editor.animation_paused)
                    || !editor.movements.is_empty()
                    || !editor.presentations.is_empty()
                {
                    let now = Instant::now();
                    if now >= next_animation_frame {
                        window.request_redraw();
                        next_animation_frame = now + Duration::from_millis(16);
                    }
                    elwt.set_control_flow(ControlFlow::WaitUntil(next_animation_frame));
                } else {
                    next_animation_frame = Instant::now();
                    elwt.set_control_flow(ControlFlow::Wait);
                }
            }
            _ => {}
        })
        .expect("window loop");
}
