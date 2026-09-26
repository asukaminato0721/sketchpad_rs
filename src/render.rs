use crate::document::{Document, Kind, LineMode, Point};
use crate::graph::GridForm;
use crate::style_records::{
    LinePattern, LineWidth, PointStyle, RgbColor, SegmentLineStyle, VisualStyle,
};
use egui::{Pos2, Rect};
use std::fmt::Write as _;
use wgpu::util::DeviceExt;

fn rgba(color: RgbColor, alpha: f32) -> [f32; 4] {
    [
        color.red as f32 / 255.,
        color.green as f32 / 255.,
        color.blue as f32 / 255.,
        alpha,
    ]
}
fn css_color(color: RgbColor) -> String {
    format!("#{:02x}{:02x}{:02x}", color.red, color.green, color.blue)
}
fn point_radius(style: PointStyle) -> f32 {
    match style {
        PointStyle::Dot => 1.4,
        PointStyle::Small => 2.7,
        PointStyle::Medium => 4.,
        PointStyle::Large => 6.,
    }
}
fn line_width(style: LineWidth) -> f32 {
    match style {
        LineWidth::Hairline => 1.,
        LineWidth::Thin => 1.5,
        LineWidth::Medium => 2.,
        LineWidth::Thick => 4.,
    }
}
fn svg_stroke(style: Option<VisualStyle>) -> String {
    if let Some(VisualStyle::Stroke { line, color }) = style {
        let dash = match line.pattern {
            LinePattern::Solid => String::new(),
            LinePattern::Dashed => " stroke-dasharray=\"9 6\"".into(),
            LinePattern::Dotted => " stroke-dasharray=\"2 5\"".into(),
        };
        format!(
            " stroke=\"{}\" stroke-width=\"{}\"{}",
            css_color(color),
            line_width(line.width),
            dash
        )
    } else {
        String::new()
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 4],
}

pub struct Viewport {
    pub rect: Rect,
    pub scale: f32,
    pub center: Point,
}
impl Viewport {
    pub fn screen(&self, p: Point) -> Pos2 {
        self.rect.center()
            + egui::vec2(
                (p.x - self.center.x) as f32 * self.scale,
                (p.y - self.center.y) as f32 * self.scale,
            )
    }
    pub fn world(&self, p: Pos2) -> Point {
        let d = p - self.rect.center();
        Point {
            x: self.center.x + d.x as f64 / self.scale as f64,
            y: self.center.y + d.y as f64 / self.scale as f64,
        }
    }
}
pub fn action_button_rect(vp: &Viewport, anchor: (i16, i16), label: &str) -> Rect {
    let min = vp.rect.min + egui::vec2(anchor.0 as f32, anchor.1 as f32);
    let width = (label.chars().count() as f32 * 8. + 20.).max(88.);
    Rect::from_min_size(min, egui::vec2(width, 24.))
}
pub fn number_lines(kind: &Kind) -> Option<((i16, i16), Vec<String>)> {
    match kind {
        Kind::Parameter(group) => Some((
            group.anchor(),
            vec![format!(
                "{} = {:.*}",
                match group.name() {
                    "t[1]" => "t₁",
                    "t[2]" => "t₂",
                    name => name,
                },
                group.precision(),
                group.value()
            )],
        )),
        Kind::Calculation(group) => Some((
            group.anchor(),
            vec![format!("{} = {:.2}", group.expression(), group.value())],
        )),
        Kind::ValueTable(_, group) => {
            if let Some((name, values)) = group.named_label_values() {
                let mut lines = vec![name];
                lines.extend(values);
                return Some((group.anchor(), lines));
            }
            let mut lines = vec!["t₁".into()];
            lines.extend(
                group
                    .default_values()?
                    .into_iter()
                    .map(|value| format!("{value:.2}")),
            );
            Some((group.anchor(), lines))
        }
        Kind::ValueTablePair(_, _, group) => {
            let mut lines = vec!["t₂       t₁".into()];
            lines.extend(
                group
                    .pair_values()?
                    .into_iter()
                    .map(|[x, y]| format!("{x:.2}    {y:.2}")),
            );
            Some((group.anchor(), lines))
        }
        _ => None,
    }
}

pub fn number_lines_for_object(doc: &Document, id: usize) -> Option<((i16, i16), Vec<String>)> {
    let object = doc.objects.get(id.checked_sub(1)?)?;
    if let Kind::MeasuredCalculation(source, group) = &object.kind {
        let value = doc.measured_calculation_cm(id)?;
        let name = doc
            .length_measurement_name(*source)
            .unwrap_or_else(|| "Length".into());
        return Some((
            group.anchor(),
            vec![format!(
                "{name} {} {} = {value:.2} cm",
                if group.divides() { "÷" } else { "·" },
                group.factor()
            )],
        ));
    }
    if let Kind::ValueTable(source, group) = &object.kind
        && let Some((name, mut values)) = group.named_label_values()
    {
        if let Some(Kind::Parameter(parameter)) = doc
            .objects
            .get(source.checked_sub(1)?)
            .map(|object| &object.kind)
            && let Some(last) = values.last_mut()
        {
            *last = format!("{:.*}", parameter.precision(), parameter.value());
        }
        let mut lines = vec![name];
        lines.extend(values);
        return Some((group.anchor(), lines));
    }
    number_lines(&object.kind)
}

fn number_rect(vp: &Viewport, anchor: (i16, i16), lines: &[String]) -> Rect {
    let width = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0) as f32
        * 10.
        + 16.;
    Rect::from_min_size(
        vp.rect.min + egui::vec2(anchor.0 as f32, anchor.1 as f32),
        egui::vec2(width.max(66.), lines.len() as f32 * 21. + 8.),
    )
}

pub fn number_screen_rect(vp: &Viewport, kind: &Kind) -> Option<Rect> {
    let (anchor, lines) = number_lines(kind)?;
    Some(number_rect(vp, anchor, &lines))
}

pub fn number_screen_rect_for_object(vp: &Viewport, doc: &Document, id: usize) -> Option<Rect> {
    let (anchor, lines) = number_lines_for_object(doc, id)?;
    Some(number_rect(vp, anchor, &lines))
}
pub fn picture_screen_quad(vp: &Viewport, png: &[u8], transform: [f64; 6]) -> Option<[Pos2; 4]> {
    if png.len() < 24 {
        return None;
    }
    let width = u32::from_be_bytes(png[16..20].try_into().ok()?) as f64;
    let height = u32::from_be_bytes(png[20..24].try_into().ok()?) as f64;
    let world = |u: f64, v: f64| Point {
        x: transform[0] * u + transform[1] * v + transform[2],
        y: transform[3] * u + transform[4] * v + transform[5],
    };
    Some([
        vp.screen(world(0., 0.)),
        vp.screen(world(width, 0.)),
        vp.screen(world(width, height)),
        vp.screen(world(0., height)),
    ])
}
pub fn picture_hit(vp: &Viewport, png: &[u8], transform: [f64; 6], at: Pos2) -> bool {
    if png.len() < 24 {
        return false;
    }
    let world = vp.world(at);
    let x = world.x - transform[2];
    let y = world.y - transform[5];
    let det = transform[0] * transform[4] - transform[1] * transform[3];
    if det.abs() < 1e-12 {
        return false;
    }
    let u = (transform[4] * x - transform[1] * y) / det;
    let v = (-transform[3] * x + transform[0] * y) / det;
    let width = u32::from_be_bytes(png[16..20].try_into().unwrap()) as f64;
    let height = u32::from_be_bytes(png[20..24].try_into().unwrap()) as f64;
    u >= 0. && u <= width && v >= 0. && v <= height
}

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = ((chunk[0] as u32) << 16)
            | ((chunk.get(1).copied().unwrap_or(0) as u32) << 8)
            | chunk.get(2).copied().unwrap_or(0) as u32;
        out.push(ALPHABET[((n >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
fn clipped_line(a: Pos2, b: Pos2, mode: LineMode, rect: Rect) -> Option<(Pos2, Pos2)> {
    let d = b - a;
    if d.length_sq() < 1e-8 {
        return None;
    }
    let (mut lo, mut hi) = match mode {
        LineMode::Segment => (0., 1.),
        LineMode::Ray => (0., f32::INFINITY),
        LineMode::Line => (f32::NEG_INFINITY, f32::INFINITY),
    };
    for (origin, delta, min, max) in [
        (a.x, d.x, rect.min.x, rect.max.x),
        (a.y, d.y, rect.min.y, rect.max.y),
    ] {
        if delta.abs() < 1e-8 {
            if origin < min || origin > max {
                return None;
            }
            continue;
        }
        let t0 = (min - origin) / delta;
        let t1 = (max - origin) / delta;
        lo = lo.max(t0.min(t1));
        hi = hi.min(t0.max(t1));
    }
    (lo <= hi && lo.is_finite() && hi.is_finite()).then_some((a + d * lo, a + d * hi))
}

fn grid_lines_for_frame(doc: &Document, vp: &Viewport, frame_id: usize) -> Vec<(Pos2, Pos2)> {
    let Some(frame) = doc.graph_frame(frame_id) else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    let corners = [
        vp.rect.left_top(),
        vp.rect.right_top(),
        vp.rect.left_bottom(),
        vp.rect.right_bottom(),
    ];
    let coordinates = corners
        .into_iter()
        .filter_map(|corner| {
            let world = vp.world(corner);
            frame.coordinates(crate::graph::GraphPoint {
                x: world.x,
                y: world.y,
            })
        })
        .collect::<Vec<_>>();
    if coordinates.len() != 4 {
        return Vec::new();
    }
    let (xmin, xmax) = coordinates
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| {
            (lo.min(p.x), hi.max(p.x))
        });
    let (ymin, ymax) = coordinates
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| {
            (lo.min(p.y), hi.max(p.y))
        });
    let bounds = [
        (xmin.floor() - 1., xmax.ceil() + 1.),
        (ymin.floor() - 1., ymax.ceil() + 1.),
    ];
    for axis in 0..2 {
        let (lo, hi) = bounds[axis];
        if !lo.is_finite() || !hi.is_finite() {
            continue;
        }
        let interval = ((hi - lo) / 200.).ceil().max(1.) as i64;
        let mut value = lo as i64;
        while value <= hi as i64 {
            let pair = if axis == 0 {
                (
                    frame.world(value as f64, ymin - 1.),
                    frame.world(value as f64, ymax + 1.),
                )
            } else {
                (
                    frame.world(xmin - 1., value as f64),
                    frame.world(xmax + 1., value as f64),
                )
            };
            if let (Some(a), Some(b)) = pair {
                if let Some(line) = clipped_line(
                    vp.screen(Point { x: a.x, y: a.y }),
                    vp.screen(Point { x: b.x, y: b.y }),
                    LineMode::Line,
                    vp.rect,
                ) {
                    lines.push(line);
                }
            }
            value = value.saturating_add(interval);
            if value == i64::MAX {
                break;
            }
        }
    }
    lines
}

fn polar_grid_for_frame(
    doc: &Document,
    vp: &Viewport,
    frame_id: usize,
) -> (Vec<Vec<Pos2>>, Vec<Pos2>) {
    let Some(frame) = doc.graph_frame(frame_id) else {
        return (Vec::new(), Vec::new());
    };
    let mut radius: f64 = 0.0;
    for corner in [
        vp.rect.left_top(),
        vp.rect.right_top(),
        vp.rect.left_bottom(),
        vp.rect.right_bottom(),
    ] {
        let at = vp.world(corner);
        if let Some(point) = frame.coordinates(crate::graph::GraphPoint { x: at.x, y: at.y }) {
            radius = radius.max(point.x.hypot(point.y));
        }
    }
    if !radius.is_finite() {
        return (Vec::new(), Vec::new());
    }
    let radius = radius.ceil().min(200.) as usize;
    let mut paths = Vec::new();
    let mut dots = Vec::new();
    for r in 1..=radius {
        let mut circle = Vec::with_capacity(97);
        for sample in 0..=96 {
            let angle = std::f64::consts::TAU * sample as f64 / 96.;
            if let Some(at) = frame.world(r as f64 * angle.cos(), r as f64 * angle.sin()) {
                circle.push(vp.screen(Point { x: at.x, y: at.y }));
            }
        }
        if circle.len() == 97 {
            paths.push(circle);
        }
        for spoke in 0..24 {
            let angle = std::f64::consts::TAU * spoke as f64 / 24.;
            if let Some(at) = frame.world(r as f64 * angle.cos(), r as f64 * angle.sin()) {
                let screen = vp.screen(Point { x: at.x, y: at.y });
                if vp.rect.contains(screen) {
                    dots.push(screen);
                }
            }
        }
    }
    let origin = vp.screen(Point {
        x: frame.origin.x,
        y: frame.origin.y,
    });
    for spoke in 0..24 {
        let angle = std::f64::consts::TAU * spoke as f64 / 24.;
        if let Some(at) = frame.world(radius as f64 * angle.cos(), radius as f64 * angle.sin()) {
            paths.push(vec![origin, vp.screen(Point { x: at.x, y: at.y })]);
        }
    }
    (paths, dots)
}

fn square_grid_points(lines: &[(Pos2, Pos2)], vp: &Viewport) -> Vec<Pos2> {
    let mut points = Vec::new();
    for (index, (a, b)) in lines.iter().enumerate() {
        let u = *b - *a;
        for (c, d) in &lines[index + 1..] {
            let v = *d - *c;
            let determinant = u.x * v.y - u.y * v.x;
            if determinant.abs() < 1e-4 {
                continue;
            }
            let w = *c - *a;
            let t = (w.x * v.y - w.y * v.x) / determinant;
            let s = (w.x * u.y - w.y * u.x) / determinant;
            if (-1e-4..=1.0001).contains(&t) && (-1e-4..=1.0001).contains(&s) {
                let point = *a + u * t;
                if vp.rect.contains(point) {
                    points.push(point);
                }
            }
        }
    }
    points
}

#[derive(Default)]
struct GridGeometry {
    lines: Vec<(Pos2, Pos2)>,
    polar_paths: Vec<Vec<Pos2>>,
    dots: Vec<Pos2>,
    selected_lines: Vec<(Pos2, Pos2)>,
    selected_polar_paths: Vec<Vec<Pos2>>,
    selected_dots: Vec<Pos2>,
}

fn grid_geometry(doc: &Document, vp: &Viewport) -> GridGeometry {
    let mut geometry = GridGeometry::default();
    for object in doc.objects.iter().filter(|object| {
        matches!(object.kind, Kind::CoordinateSystem(_, _)) && doc.is_displayed(object.id)
    }) {
        let (form, dotted) = doc
            .frame_styles
            .get(&object.id)
            .copied()
            .unwrap_or((doc.grid_form[object.page], doc.grid_dotted[object.page]));
        let selected = doc.selected.contains(&object.id);
        if form == GridForm::Polar {
            let (paths, dots) = polar_grid_for_frame(doc, vp, object.id);
            if dotted {
                if selected {
                    geometry.selected_dots.extend(dots);
                } else {
                    geometry.dots.extend(dots);
                }
            } else {
                if selected {
                    geometry.selected_polar_paths.extend(paths);
                } else {
                    geometry.polar_paths.extend(paths);
                }
            }
        } else {
            let lines = grid_lines_for_frame(doc, vp, object.id);
            if dotted {
                let dots = square_grid_points(&lines, vp);
                if selected {
                    geometry.selected_dots.extend(dots);
                } else {
                    geometry.dots.extend(dots);
                }
            } else {
                if selected {
                    geometry.selected_lines.extend(lines);
                } else {
                    geometry.lines.extend(lines);
                }
            }
        }
    }
    geometry
}

fn defined_axis_annotations(
    doc: &Document,
    vp: &Viewport,
) -> (Vec<(Pos2, Pos2, bool)>, Vec<(Pos2, String, bool)>) {
    let mut ticks = Vec::new();
    let mut labels = Vec::new();
    for object in doc.objects.iter().filter(|object| {
        matches!(object.kind, Kind::CoordinateSystem(_, _)) && doc.is_displayed(object.id)
    }) {
        let Kind::CoordinateSystem(x_axis, y_axis) = object.kind else {
            continue;
        };
        let x_kind = x_axis
            .checked_sub(1)
            .and_then(|id| doc.objects.get(id))
            .map(|axis| &axis.kind);
        let circle_defined = matches!(x_kind, Some(Kind::CircleAxis(_, 1)));
        let distance_defined = matches!(x_kind, Some(Kind::DistanceAxis(_, _, 1)));
        if !circle_defined && !distance_defined {
            continue;
        }
        let Some(frame) = doc.graph_frame(object.id) else {
            continue;
        };
        let center = vp.screen(Point {
            x: frame.origin.x,
            y: frame.origin.y,
        });
        let x_unit = vp
            .screen(Point {
                x: frame.x_unit.x,
                y: frame.x_unit.y,
            })
            .distance(center);
        let y_unit = vp
            .screen(Point {
                x: frame.y_unit.x,
                y: frame.y_unit.y,
            })
            .distance(center);
        if !x_unit.is_finite() || x_unit < 1. || !y_unit.is_finite() || y_unit < 1. {
            continue;
        }
        let divisions = |unit: f32| {
            if unit >= 50. {
                10
            } else if unit >= 20. {
                5
            } else {
                1
            }
        };
        let x_divisions = divisions(x_unit);
        let y_divisions = divisions(y_unit);
        let x_spacing = x_unit / x_divisions as f32;
        let y_spacing = y_unit / y_divisions as f32;
        if !x_spacing.is_finite() || x_spacing < 1. || !y_spacing.is_finite() || y_spacing < 1. {
            continue;
        }
        let selected_x = doc.selected.contains(&x_axis);
        let selected_y = doc.selected.contains(&y_axis);
        let first_x = (((vp.rect.min.x - center.x) / x_spacing).ceil() as i32).max(-1000);
        let last_x = (((vp.rect.max.x - center.x) / x_spacing).floor() as i32).min(1000);
        let first_y = (((center.y - vp.rect.max.y) / y_spacing).ceil() as i32).max(-1000);
        let last_y = (((center.y - vp.rect.min.y) / y_spacing).floor() as i32).min(1000);
        if doc.is_displayed(x_axis) && vp.rect.min.y <= center.y && center.y <= vp.rect.max.y {
            for i in first_x..=last_x {
                if i == 0 {
                    continue;
                }
                let at = egui::pos2(center.x + i as f32 * x_spacing, center.y);
                ticks.push((
                    at + egui::vec2(0., -4.),
                    at + egui::vec2(0., 4.),
                    selected_x,
                ));
                let x_label_multiple = if circle_defined {
                    2 * x_divisions
                } else if x_unit >= 160. {
                    x_divisions / 2
                } else if x_unit >= 80. {
                    x_divisions
                } else {
                    2 * x_divisions
                };
                if x_label_multiple > 0 && i % x_label_multiple == 0 && x_unit >= 30. {
                    let label_at = at + egui::vec2(0., 8.);
                    if vp.rect.contains(label_at) {
                        let value = i as f32 / x_divisions as f32;
                        labels.push((
                            label_at,
                            if i % x_divisions == 0 {
                                format!("{}", i / x_divisions)
                            } else {
                                format!("{value:.1}")
                            },
                            false,
                        ));
                    }
                }
            }
        }
        if doc.is_displayed(y_axis) && vp.rect.min.x <= center.x && center.x <= vp.rect.max.x {
            for i in first_y..=last_y {
                if i == 0 {
                    continue;
                }
                let at = egui::pos2(center.x, center.y - i as f32 * y_spacing);
                ticks.push((
                    at + egui::vec2(-4., 0.),
                    at + egui::vec2(4., 0.),
                    selected_y,
                ));
                if y_divisions % 2 == 0 && i % (y_divisions / 2) == 0 && y_unit >= 80. {
                    let label_at = at + egui::vec2(-8., 0.);
                    if vp.rect.contains(label_at) {
                        let value = i as f32 / y_divisions as f32;
                        labels.push((
                            label_at,
                            if i % y_divisions == 0 {
                                format!("{}", i / y_divisions)
                            } else {
                                format!("{value:.1}")
                            },
                            true,
                        ));
                    }
                }
            }
        }
    }
    (ticks, labels)
}

pub fn defined_axis_labels(doc: &Document, vp: &Viewport) -> Vec<(Pos2, String, bool)> {
    defined_axis_annotations(doc, vp).1
}

#[cfg(test)]
fn grid_points(doc: &Document, vp: &Viewport) -> Vec<Pos2> {
    let mut grid = grid_geometry(doc, vp);
    grid.dots.append(&mut grid.selected_dots);
    grid.dots
}

pub fn trigonometric_axis_labels(doc: &Document, vp: &Viewport) -> Vec<(Pos2, String)> {
    if !doc.axis_trigonometric() {
        return Vec::new();
    }
    let Some(frame_id) = doc.active_coordinate_system() else {
        return Vec::new();
    };
    let Kind::CoordinateSystem(x_axis, _) = doc.objects[frame_id - 1].kind else {
        return Vec::new();
    };
    if !doc.is_displayed(x_axis) {
        return Vec::new();
    }
    let Some(frame) = doc.graph_frame(frame_id) else {
        return Vec::new();
    };
    let origin = vp.screen(Point {
        x: frame.origin.x,
        y: frame.origin.y,
    });
    let Some(next) = frame.world(std::f64::consts::PI, 0.) else {
        return Vec::new();
    };
    let spacing = vp
        .screen(Point {
            x: next.x,
            y: next.y,
        })
        .distance(origin);
    if !spacing.is_finite() || spacing <= 0. {
        return Vec::new();
    }
    let interval = (45. / spacing).ceil().max(1.) as i32;
    let mut labels = Vec::new();
    for multiple in -64_i32..=64 {
        if multiple == 0 || multiple % interval != 0 {
            continue;
        }
        let Some(at) = frame.world(multiple as f64 * std::f64::consts::PI, 0.) else {
            continue;
        };
        let at = vp.screen(Point { x: at.x, y: at.y }) + egui::vec2(0., 8.);
        if !vp.rect.contains(at) {
            continue;
        }
        let magnitude = multiple.abs();
        let label = format!(
            "{}{}π",
            if multiple < 0 { "−" } else { "" },
            if magnitude == 1 {
                String::new()
            } else {
                magnitude.to_string()
            }
        );
        labels.push((at, label));
    }
    labels
}

pub fn svg_page(doc: &Document, vp: &Viewport) -> Result<String, String> {
    svg_page_with_button_offset(doc, vp, egui::Vec2::ZERO)
}

pub fn svg_page_with_button_offset(
    doc: &Document,
    vp: &Viewport,
    button_offset: egui::Vec2,
) -> Result<String, String> {
    if doc.partial_view {
        return Err("This document contains objects that cannot be exported yet".into());
    }
    let r = vp.rect;
    if !r.is_finite() || r.width() <= 0. || r.height() <= 0. {
        return Err("No visible sketch viewport".into());
    }
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"{} {} {} {}\">\n<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"white\"/>\n",
        r.width(),
        r.height(),
        r.min.x,
        r.min.y,
        r.width(),
        r.height(),
        r.min.x,
        r.min.y,
        r.width(),
        r.height()
    );
    for object in doc
        .objects
        .iter()
        .filter(|object| doc.is_displayed(object.id))
    {
        if let Kind::Image { png, transform } = &object.kind {
            let (width, height) = crate::image_records::ImageFile::png_size(png)?;
            let origin = vp.screen(Point {
                x: transform[2],
                y: transform[5],
            });
            writeln!(svg,
                "<image width=\"{width}\" height=\"{height}\" transform=\"matrix({} {} {} {} {} {})\" href=\"data:image/png;base64,{}\"/>",
                transform[0] * vp.scale as f64, transform[3] * vp.scale as f64,
                transform[1] * vp.scale as f64, transform[4] * vp.scale as f64,
                origin.x, origin.y, base64(png)).unwrap();
        }
    }
    svg.push_str("<g fill=\"none\" stroke=\"#d4dbe1\" stroke-width=\"1\">\n");
    let mut grid = grid_geometry(doc, vp);
    grid.dots.append(&mut grid.selected_dots);
    grid.lines.append(&mut grid.selected_lines);
    grid.polar_paths.append(&mut grid.selected_polar_paths);
    for point in grid.dots {
        writeln!(
            svg,
            "<circle cx=\"{}\" cy=\"{}\" r=\"1.2\" fill=\"#d4dbe1\" stroke=\"none\"/>",
            point.x, point.y
        )
        .unwrap();
    }
    for (a, b) in grid.lines {
        writeln!(
            svg,
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
            a.x, a.y, b.x, b.y
        )
        .unwrap();
    }
    for path in grid.polar_paths {
        let points = path
            .iter()
            .map(|point| format!("{},{}", point.x, point.y))
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(svg, "<polyline points=\"{points}\"/>").unwrap();
    }
    svg.push_str("</g>\n<g fill=\"none\" stroke=\"#24303d\" stroke-width=\"2\">\n");
    for (a, b, _) in defined_axis_annotations(doc, vp).0 {
        writeln!(
            svg,
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke-width=\"1\"/>",
            a.x, a.y, b.x, b.y
        )
        .unwrap();
    }
    for o in doc.objects.iter().filter(|o| doc.is_displayed(o.id)) {
        if let Some((a, b, mode)) = doc.line_points(o.id)
            && let Some((a, b)) = clipped_line(vp.screen(a), vp.screen(b), mode, r)
        {
            writeln!(
                svg,
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"{}/>",
                a.x,
                a.y,
                b.x,
                b.y,
                svg_stroke(o.visual_style)
            )
            .unwrap();
        }
        if let Some(polygons) = doc.locus_polygon_surfaces(o.id, 384) {
            for points in polygons {
                let vertices = points
                    .into_iter()
                    .map(|point| {
                        let at = vp.screen(point);
                        format!("{},{}", at.x, at.y)
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                writeln!(
                    svg,
                    "<polygon points=\"{}\" fill=\"#ffb165\" stroke=\"none\"/>",
                    vertices
                )
                .unwrap();
            }
        } else if let Some(segments) = doc
            .function_plot_segments(o.id, 384)
            .or_else(|| doc.parametric_plot_segments(o.id, 384))
            .map(|segments| {
                segments
                    .into_iter()
                    .map(|(start, end)| (start, end, LineMode::Segment))
                    .collect::<Vec<_>>()
            })
            .or_else(|| doc.locus_strokes(o.id, 384))
        {
            for (start, end, mode) in segments {
                if let Some((a, b)) = clipped_line(vp.screen(start), vp.screen(end), mode, r) {
                    writeln!(
                        svg,
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#b43a9b\"/>",
                        a.x, a.y, b.x, b.y
                    )
                    .unwrap();
                }
            }
        }
        if let Some((center, radius)) = doc.circle_points(o.id) {
            let at = vp.screen(center);
            writeln!(
                svg,
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\"{}/>",
                at.x,
                at.y,
                radius as f32 * vp.scale,
                svg_stroke(o.visual_style)
            )
            .unwrap();
        }
        if let Some(points) = doc.arc_points(o.id, 128) {
            let mut points = points.into_iter();
            if let Some(first) = points.next() {
                let at = vp.screen(first);
                write!(
                    svg,
                    "<path{} d=\"M {} {}",
                    svg_stroke(o.visual_style),
                    at.x,
                    at.y
                )
                .unwrap();
                for p in points {
                    let at = vp.screen(p);
                    write!(svg, " L {} {}", at.x, at.y).unwrap();
                }
                svg.push_str("\"/>\n");
            }
        }
        if let Some(points) = doc.polygon_points(o.id) {
            let vertices = points
                .into_iter()
                .map(|point| {
                    let at = vp.screen(point);
                    format!("{},{}", at.x, at.y)
                })
                .collect::<Vec<_>>()
                .join(" ");
            writeln!(
                svg,
                "<polygon points=\"{}\" fill=\"{}\" fill-opacity=\"0.35\" stroke=\"none\"/>",
                vertices,
                match o.visual_style {
                    Some(VisualStyle::Polygon { color }) => css_color(color),
                    _ => "#f6ad60".into(),
                }
            )
            .unwrap();
        }
        if let Some(segments) = doc.segment_iteration_images(o.id) {
            for (start, end) in segments {
                let start = vp.screen(start);
                let end = vp.screen(end);
                writeln!(svg, "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#176bd6\" stroke-width=\"2\"/>", start.x, start.y, end.x, end.y).unwrap();
            }
        }
    }
    svg.push_str("</g>\n<g fill=\"#176bd6\">\n");
    for o in doc.objects.iter().filter(|o| doc.is_displayed(o.id)) {
        if let Some(p) = doc.position(o.id) {
            let at = vp.screen(p);
            let (radius, color) = match o.visual_style {
                Some(VisualStyle::Point { shape, color }) => {
                    (point_radius(shape), css_color(color))
                }
                _ => (4., "#176bd6".into()),
            };
            writeln!(
                svg,
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"/>",
                at.x, at.y, radius, color
            )
            .unwrap();
        }
    }
    for object in doc
        .objects
        .iter()
        .filter(|object| doc.is_displayed(object.id))
    {
        if let Some(images) = doc.point_iteration_images(object.id) {
            for point in images {
                let at = vp.screen(point);
                writeln!(
                    svg,
                    "<circle cx=\"{}\" cy=\"{}\" r=\"2\" fill=\"#176bd6\"/>",
                    at.x, at.y
                )
                .unwrap();
            }
        }
    }
    svg.push_str("</g>\n<g fill=\"#232323\" font-family=\"sans-serif\" font-size=\"16\">\n");
    for object in doc.objects.iter().filter(|o| doc.is_displayed(o.id)) {
        if let Kind::PlainText { text, rect } = &object.kind {
            let at = vp.screen(Point {
                x: rect.left.into(),
                y: rect.top.into(),
            });
            let style = doc.text_font_state(object.id);
            let size = style.as_ref().map_or(16, |style| style.size_code);
            let family = style
                .as_ref()
                .map_or("sans-serif", |style| style.family.as_str())
                .replace('&', "&amp;")
                .replace('"', "&quot;")
                .replace('<', "&lt;");
            let weight = if style.as_ref().is_some_and(|style| style.bold) {
                "bold"
            } else {
                "normal"
            };
            let italic = if style.as_ref().is_some_and(|style| style.italic) {
                "italic"
            } else {
                "normal"
            };
            let decoration = if style.as_ref().is_some_and(|style| style.underline) {
                "underline"
            } else {
                "none"
            };
            let runs = doc.text_rich_runs(object.id);
            let families = doc.text_font_families().unwrap_or_default();
            let has_mixed_runs = runs.as_ref().is_some_and(|rows| {
                rows.iter()
                    .flatten()
                    .any(|run| run.style != crate::text_object::RichRunStyle::default())
            });
            if has_mixed_runs {
                let mut line_y = at.y;
                for row in runs.unwrap().iter() {
                    let line_size = row
                        .iter()
                        .map(|run| run.style.size_code.unwrap_or(size) as f32)
                        .fold(size as f32, f32::max);
                    write!(
                        svg,
                        "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\">",
                        at.x,
                        line_y + line_size * 0.85,
                        family,
                        size
                    )
                    .unwrap();
                    for run in row {
                        let run_family = run
                            .style
                            .family_index
                            .and_then(|index| families.get(usize::from(index)))
                            .map(|name| {
                                format!(
                                    " font-family=\"{}\"",
                                    name.replace('&', "&amp;")
                                        .replace('"', "&quot;")
                                        .replace('<', "&lt;")
                                )
                            })
                            .unwrap_or_default();
                        let bold = run
                            .style
                            .bold
                            .unwrap_or_else(|| style.as_ref().is_some_and(|style| style.bold));
                        let slanted = run
                            .style
                            .italic
                            .unwrap_or_else(|| style.as_ref().is_some_and(|style| style.italic));
                        let underlined = run
                            .style
                            .underline
                            .unwrap_or_else(|| style.as_ref().is_some_and(|style| style.underline));
                        write!(
                            svg,
                            "<tspan{} font-size=\"{}\" font-weight=\"{}\" font-style=\"{}\" text-decoration=\"{}\">{}</tspan>",
                            run_family,
                            run.style.size_code.unwrap_or(size),
                            if bold { "bold" } else { "normal" },
                            if slanted { "italic" } else { "normal" },
                            if underlined { "underline" } else { "none" },
                            xml_text(&run.text)
                        )
                        .unwrap();
                    }
                    svg.push_str("</text>\n");
                    line_y += (line_size * 1.1).max(19.);
                }
            } else {
                for (line, content) in text.split('\n').enumerate() {
                    writeln!(
                    svg,
                    "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\" font-weight=\"{}\" font-style=\"{}\" text-decoration=\"{}\">{}</text>",
                    at.x,
                    at.y + size as f32 * 0.85 + line as f32 * 19.,
                    family,
                    size,
                    weight,
                    italic,
                    decoration,
                    xml_text(content)
                )
                .unwrap();
                }
            }
        }
        if let Some((anchor, lines)) = number_lines_for_object(doc, object.id) {
            let at = vp.rect.min + egui::vec2(anchor.0 as f32, anchor.1 as f32);
            for (line, content) in lines.iter().enumerate() {
                writeln!(
                    svg,
                    "<text x=\"{}\" y=\"{}\">{}</text>",
                    at.x + 4.,
                    at.y + 18. + line as f32 * 21.,
                    xml_text(content)
                )
                .unwrap();
            }
        }
        if let Some(label) = &object.label
            && label.visible()
            && let Ok(name) = label.name_utf8()
            && let Some(point) = doc.position(object.id)
        {
            let at = vp.screen(point);
            let offset = label.offset();
            let style = doc.text_font_state(object.id);
            let size = style.as_ref().map_or(13, |style| style.size_code);
            let family = style
                .as_ref()
                .map_or("sans-serif", |style| style.family.as_str())
                .replace('&', "&amp;")
                .replace('"', "&quot;")
                .replace('<', "&lt;");
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\" font-weight=\"{}\" font-style=\"{}\" text-decoration=\"{}\">{}</text>",
                at.x + offset.0 as f32,
                at.y + offset.1 as f32 + size as f32 * 0.85,
                family,
                size,
                if style.as_ref().is_some_and(|style| style.bold) { "bold" } else { "normal" },
                if style.as_ref().is_some_and(|style| style.italic) { "italic" } else { "normal" },
                if style.as_ref().is_some_and(|style| style.underline) { "underline" } else { "none" },
                xml_text(name)
            )
            .unwrap();
        }
    }
    let mut row = 0;
    for object in doc
        .objects
        .iter()
        .filter(|object| doc.is_displayed(object.id))
    {
        if let Some(length) = doc.segment_length_cm(object.id) {
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">Length = {:.2} cm</text>",
                r.min.x + 12.,
                r.min.y + 28. + row as f32 * 20.,
                length
            )
            .unwrap();
            row += 1;
        }
        if let Some(distance) = doc.point_distance_cm(object.id) {
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">Distance = {:.2} cm</text>",
                r.min.x + 12.,
                r.min.y + 28. + row as f32 * 20.,
                distance
            )
            .unwrap();
            row += 1;
        }
        if let Some(ratio) = doc.ratio_value(object.id) {
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">Ratio = {:.2}</text>",
                r.min.x + 12.,
                r.min.y + 28. + row as f32 * 20.,
                ratio
            )
            .unwrap();
            row += 1;
        }
        if let Some(perimeter) = doc.perimeter_cm(object.id) {
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">Perimeter = {:.2} cm</text>",
                r.min.x + 12.,
                r.min.y + 28. + row as f32 * 20.,
                perimeter
            )
            .unwrap();
            row += 1;
        }
        if let Some(circumference) = doc.circle_circumference_cm(object.id) {
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">Circumference = {:.2} cm</text>",
                r.min.x + 12.,
                r.min.y + 28. + row as f32 * 20.,
                circumference
            )
            .unwrap();
            row += 1;
        }
        if let Some(radius) = doc.circle_radius_cm(object.id) {
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">Radius = {:.2} cm</text>",
                r.min.x + 12.,
                r.min.y + 28. + row as f32 * 20.,
                radius
            )
            .unwrap();
            row += 1;
        }
        if let Some(area) = doc.circle_area_cm2(object.id) {
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">Area = {:.2} cm²</text>",
                r.min.x + 12.,
                r.min.y + 28. + row as f32 * 20.,
                area
            )
            .unwrap();
            row += 1;
        }
        if let Some(area) = doc.polygon_area_cm2(object.id) {
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">Area = {:.2} cm²</text>",
                r.min.x + 12.,
                r.min.y + 28. + row as f32 * 20.,
                area
            )
            .unwrap();
            row += 1;
        }
        if let Some(angle) = doc.arc_angle_degrees(object.id) {
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">Arc Angle = {:.2}°</text>",
                r.min.x + 12.,
                r.min.y + 28. + row as f32 * 20.,
                angle
            )
            .unwrap();
            row += 1;
        }
        if let Some(length) = doc.arc_length_cm(object.id) {
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">Arc Length = {:.2} cm</text>",
                r.min.x + 12.,
                r.min.y + 28. + row as f32 * 20.,
                length
            )
            .unwrap();
            row += 1;
        }
        if let Some(angle) = doc.angle_degrees(object.id) {
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">Angle = {:.2}°</text>",
                r.min.x + 12.,
                r.min.y + 28. + row as f32 * 20.,
                angle
            )
            .unwrap();
            row += 1;
        }
        if matches!(object.kind, Kind::SlopeMeasurement(_, _)) {
            let value = doc
                .slope_value(object.id)
                .map(|value| format!("{value:.2}"))
                .unwrap_or_else(|| "undefined".into());
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">Slope = {}</text>",
                r.min.x + 12.,
                r.min.y + 28. + row as f32 * 20.,
                value
            )
            .unwrap();
            row += 1;
        }
        if let Some(value) = doc.coordinate_measure_text(object.id) {
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">{}</text>",
                r.min.x + 12.,
                r.min.y + 28. + row as f32 * 20.,
                xml_text(&value)
            )
            .unwrap();
            row += 1;
        }
        if let Some(value) = doc.value_equation_text(object.id) {
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">{}</text>",
                r.min.x + 12.,
                r.min.y + 28. + row as f32 * 20.,
                xml_text(&value)
            )
            .unwrap();
            row += 1;
        }
        if let Kind::FunctionDefinition(expression) = object.kind {
            let (x, y) = object
                .function_anchor
                .map(|(x, y)| (r.min.x + x as f32 + 2., r.min.y + y as f32 + 23.))
                .unwrap_or((r.min.x + 12., r.min.y + 28. + row as f32 * 20.));
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">{}</text>",
                x,
                y,
                doc.function_display_text(object.id)
                    .unwrap_or_else(|| expression.display())
            )
            .unwrap();
            row += 1;
        }
        if let Kind::DerivativeFunction(_, expression) = object.kind {
            let (x, y) = object
                .function_anchor
                .map(|(x, y)| (r.min.x + x as f32 + 2., r.min.y + y as f32 + 23.))
                .unwrap_or((r.min.x + 12., r.min.y + 28. + row as f32 * 20.));
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">{}</text>",
                x,
                y,
                doc.function_display_text(object.id)
                    .unwrap_or_else(|| { expression.display().replacen("f(x)", "f′(x)", 1) })
            )
            .unwrap();
            row += 1;
        }
    }
    svg.push_str("</g>\n");
    for object in doc
        .objects
        .iter()
        .filter(|object| doc.is_displayed(object.id))
    {
        if let Kind::HideShowButton { label, anchor, .. }
        | Kind::AnimationButton { label, anchor, .. }
        | Kind::MovementButton { label, anchor, .. }
        | Kind::ScrollButton { label, anchor, .. }
        | Kind::PresentationButton { label, anchor, .. }
        | Kind::LinkUrlButton { label, anchor, .. }
        | Kind::LinkPageButton { label, anchor, .. } = &object.kind
        {
            let box_rect = action_button_rect(vp, *anchor, label).translate(button_offset);
            writeln!(svg, "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"3\" fill=\"#e7e9ed\" stroke=\"#5e6672\"/>", box_rect.min.x, box_rect.min.y, box_rect.width(), box_rect.height()).unwrap();
            writeln!(svg, "<text x=\"{}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"14\" fill=\"#20242a\">{}</text>", box_rect.min.x + 8., box_rect.min.y + 17., xml_text(label)).unwrap();
        }
    }
    for (at, label) in trigonometric_axis_labels(doc, vp) {
        writeln!(svg, "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"11\" fill=\"#24303d\">{label}</text>", at.x, at.y + 11.).unwrap();
    }
    for (at, label, vertical) in defined_axis_labels(doc, vp) {
        writeln!(svg, "<text x=\"{}\" y=\"{}\" text-anchor=\"{}\" dominant-baseline=\"{}\" font-family=\"sans-serif\" font-size=\"11\" fill=\"#24303d\">{label}</text>", at.x, at.y, if vertical { "end" } else { "middle" }, if vertical { "middle" } else { "hanging" }).unwrap();
    }
    svg.push_str("</svg>\n");
    Ok(svg)
}
struct Mesh {
    vertices: Vec<Vertex>,
    width: f32,
    height: f32,
    ppp: f32,
}
impl Mesh {
    fn v(&self, p: Pos2, c: [f32; 4]) -> Vertex {
        Vertex {
            pos: [
                2. * p.x * self.ppp / self.width - 1.,
                1. - 2. * p.y * self.ppp / self.height,
            ],
            color: c,
        }
    }
    fn tri(&mut self, a: Pos2, b: Pos2, c: Pos2, col: [f32; 4]) {
        self.vertices
            .extend_from_slice(&[self.v(a, col), self.v(b, col), self.v(c, col)]);
    }
    fn line(&mut self, a: Pos2, b: Pos2, w: f32, col: [f32; 4]) {
        let d = b - a;
        let n = d.length();
        if n < 0.01 {
            return;
        }
        let off = egui::vec2(-d.y / n, d.x / n) * w / 2.;
        self.tri(a + off, b + off, b - off, col);
        self.tri(a + off, b - off, a - off, col);
    }
    fn styled_path(
        &mut self,
        points: &[Pos2],
        closed: bool,
        style: SegmentLineStyle,
        color: [f32; 4],
    ) {
        if points.len() < 2 {
            return;
        }
        let width = line_width(style.width);
        let mut edges = points
            .windows(2)
            .map(|pair| (pair[0], pair[1]))
            .collect::<Vec<_>>();
        if closed {
            edges.push((points[points.len() - 1], points[0]));
        }
        if style.pattern == LinePattern::Solid {
            for (a, b) in edges {
                self.line(a, b, width, color);
            }
            return;
        }
        let total: f32 = edges.iter().map(|(a, b)| a.distance(*b)).sum();
        let scale = (total / 20_000.).max(1.);
        let (on, off) = match style.pattern {
            LinePattern::Dashed => (9. * scale, 6. * scale),
            LinePattern::Dotted => (2. * scale, 5. * scale),
            LinePattern::Solid => unreachable!(),
        };
        let cycle = on + off;
        let mut traveled = 0.;
        for (a, b) in edges {
            let delta = b - a;
            let length = delta.length();
            if length < 0.01 {
                continue;
            }
            let mut along = 0.;
            while along < length - 0.001 {
                let phase = (traveled + along) % cycle;
                let step = (if phase < on {
                    on - phase
                } else {
                    cycle - phase
                })
                .min(length - along)
                .max(0.001);
                if phase < on {
                    self.line(
                        a + delta * (along / length),
                        a + delta * ((along + step) / length),
                        width,
                        color,
                    );
                }
                along += step;
            }
            traveled += length;
        }
    }
    fn disk(&mut self, p: Pos2, r: f32, col: [f32; 4]) {
        for i in 0..20 {
            let a = i as f32 * std::f32::consts::TAU / 20.;
            let b = (i + 1) as f32 * std::f32::consts::TAU / 20.;
            self.tri(
                p,
                p + egui::vec2(a.cos(), a.sin()) * r,
                p + egui::vec2(b.cos(), b.sin()) * r,
                col,
            );
        }
    }
    fn polygon(&mut self, points: &[Pos2], fill: [f32; 4]) {
        for [a, b, c] in polygon_triangles(points) {
            self.tri(points[a], points[b], points[c], fill);
        }
    }
}

pub fn polygon_triangles(points: &[Pos2]) -> Vec<[usize; 3]> {
    if points.len() < 3 {
        return Vec::new();
    }
    let cross = |a: Pos2, b: Pos2, c: Pos2| (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    let signed_area: f32 = points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(a, b)| a.x * b.y - b.x * a.y)
        .sum();
    let winding = signed_area.signum();
    let mut remaining: Vec<usize> = (0..points.len()).collect();
    let mut triangles = Vec::with_capacity(points.len() - 2);
    while remaining.len() >= 3 {
        let mut ear = None;
        for i in 0..remaining.len() {
            let a = points[remaining[(i + remaining.len() - 1) % remaining.len()]];
            let b = points[remaining[i]];
            let c = points[remaining[(i + 1) % remaining.len()]];
            if cross(a, b, c) * winding <= 1e-4 {
                continue;
            }
            let has_inner = remaining.iter().enumerate().any(|(j, &vertex)| {
                if j == i
                    || j == (i + 1) % remaining.len()
                    || j == (i + remaining.len() - 1) % remaining.len()
                {
                    return false;
                }
                let p = points[vertex];
                cross(a, b, p) * winding >= -1e-4
                    && cross(b, c, p) * winding >= -1e-4
                    && cross(c, a, p) * winding >= -1e-4
            });
            if !has_inner {
                ear = Some(i);
                triangles.push([
                    remaining[(i + remaining.len() - 1) % remaining.len()],
                    remaining[i],
                    remaining[(i + 1) % remaining.len()],
                ]);
                break;
            }
        }
        let Some(index) = ear else {
            break;
        };
        remaining.remove(index);
    }
    triangles
}

#[derive(Clone)]
pub enum TraceMark {
    Point(Point, RgbColor),
    Straight(Point, Point, LineMode, RgbColor),
    Circle(Point, f64, RgbColor),
    Arc(Vec<Point>, RgbColor),
    Polygon(Vec<Point>, RgbColor),
}

pub struct GeometryRenderer {
    pipeline: wgpu::RenderPipeline,
}
impl GeometryRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sketch geometry"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sketch geometry"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self { pipeline }
    }
    pub fn render(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        doc: &Document,
        vp: &Viewport,
        size: [u32; 2],
        ppp: f32,
        traces: &[TraceMark],
    ) {
        let mut mesh = Mesh {
            vertices: Vec::new(),
            width: size[0] as f32,
            height: size[1] as f32,
            ppp,
        };
        let dark = [0.14, 0.18, 0.24, 1.];
        let blue = [0.09, 0.42, 0.84, 1.];
        let selected = [0.94, 0.32, 0.10, 1.];
        let grid = grid_geometry(doc, vp);
        for point in grid.dots {
            mesh.line(
                point + egui::vec2(-1., 0.),
                point + egui::vec2(1., 0.),
                2.,
                [0.78, 0.83, 0.88, 1.],
            );
        }
        for (a, b) in grid.lines {
            mesh.line(a, b, 1., [0.78, 0.83, 0.88, 1.]);
        }
        for path in grid.polar_paths {
            for pair in path.windows(2) {
                mesh.line(pair[0], pair[1], 1., [0.78, 0.83, 0.88, 1.]);
            }
        }
        let selected_grid = [0.94, 0.12, 0.62, 1.];
        for point in grid.selected_dots {
            mesh.line(
                point + egui::vec2(-1.5, 0.),
                point + egui::vec2(1.5, 0.),
                2.,
                selected_grid,
            );
        }
        for (a, b) in grid.selected_lines {
            mesh.line(a, b, 1.5, selected_grid);
        }
        for path in grid.selected_polar_paths {
            for pair in path.windows(2) {
                mesh.line(pair[0], pair[1], 1.5, selected_grid);
            }
        }
        for (a, b, is_selected) in defined_axis_annotations(doc, vp).0 {
            mesh.line(a, b, 1., if is_selected { selected } else { dark });
        }
        for mark in traces {
            match mark {
                TraceMark::Point(point, color) => {
                    mesh.disk(vp.screen(*point), 2., rgba(*color, 1.));
                }
                TraceMark::Straight(a, b, mode, color) => {
                    if let Some((a, b)) = clipped_line(vp.screen(*a), vp.screen(*b), *mode, vp.rect)
                    {
                        mesh.line(a, b, 1., rgba(*color, 1.));
                    }
                }
                TraceMark::Circle(center, radius, color) => {
                    let center = vp.screen(*center);
                    let radius = (*radius as f32 * vp.scale).abs();
                    let points = (0..64)
                        .map(|step| {
                            let angle = step as f32 * std::f32::consts::TAU / 64.;
                            center + egui::vec2(angle.cos(), angle.sin()) * radius
                        })
                        .collect::<Vec<_>>();
                    mesh.styled_path(
                        &points,
                        true,
                        SegmentLineStyle {
                            width: LineWidth::Hairline,
                            pattern: LinePattern::Solid,
                        },
                        rgba(*color, 1.),
                    );
                }
                TraceMark::Arc(points, color) => {
                    let screen = points
                        .iter()
                        .copied()
                        .map(|point| vp.screen(point))
                        .collect::<Vec<_>>();
                    mesh.styled_path(
                        &screen,
                        false,
                        SegmentLineStyle {
                            width: LineWidth::Hairline,
                            pattern: LinePattern::Solid,
                        },
                        rgba(*color, 1.),
                    );
                }
                TraceMark::Polygon(points, color) => {
                    let screen = points
                        .iter()
                        .copied()
                        .map(|point| vp.screen(point))
                        .collect::<Vec<_>>();
                    mesh.polygon(&screen, rgba(*color, 0.025));
                }
            }
        }
        // Draw paths first and points last.
        for o in &doc.objects {
            if !doc.is_displayed(o.id) {
                continue;
            }
            let col = if doc.selected.contains(&o.id) {
                selected
            } else {
                match o.visual_style {
                    Some(VisualStyle::Stroke { color, .. }) => rgba(color, 1.),
                    _ => dark,
                }
            };
            let stroke = match o.visual_style {
                Some(VisualStyle::Stroke { line, .. }) => line,
                _ => SegmentLineStyle {
                    width: LineWidth::Medium,
                    pattern: LinePattern::Solid,
                },
            };
            if let Some((a, b, mode)) = doc.line_points(o.id) {
                if let Some((a, b)) = clipped_line(vp.screen(a), vp.screen(b), mode, vp.rect) {
                    mesh.styled_path(&[a, b], false, stroke, col);
                }
            }
            if let Some(segments) = doc.segment_iteration_images(o.id) {
                for (start, end) in segments {
                    if let Some((a, b)) =
                        clipped_line(vp.screen(start), vp.screen(end), LineMode::Segment, vp.rect)
                    {
                        mesh.styled_path(&[a, b], false, stroke, col);
                    }
                }
            }
            if let Some(polygons) = doc.locus_polygon_surfaces(o.id, 384) {
                let fill = if doc.selected.contains(&o.id) {
                    [1., 0., 1., 0.18]
                } else {
                    [1., 177. / 255., 101. / 255., 1.]
                };
                for points in polygons {
                    let screen = points
                        .into_iter()
                        .map(|point| vp.screen(point))
                        .collect::<Vec<_>>();
                    mesh.polygon(&screen, fill);
                }
            } else if let Some(segments) = doc
                .function_plot_segments(o.id, 384)
                .or_else(|| doc.parametric_plot_segments(o.id, 384))
                .map(|segments| {
                    segments
                        .into_iter()
                        .map(|(start, end)| (start, end, LineMode::Segment))
                        .collect::<Vec<_>>()
                })
                .or_else(|| doc.locus_strokes(o.id, 384))
            {
                for (start, end, mode) in segments {
                    if let Some((a, b)) =
                        clipped_line(vp.screen(start), vp.screen(end), mode, vp.rect)
                    {
                        mesh.line(
                            a,
                            b,
                            2.,
                            if doc.selected.contains(&o.id) {
                                selected
                            } else {
                                [0.70, 0.23, 0.60, 1.]
                            },
                        );
                    }
                }
            }
            match o.kind {
                Kind::Circle(_, _)
                | Kind::CircleBySegment(_, _)
                | Kind::PolarTranslatedCircle(_, _, _, _)
                | Kind::FixedRectangularTranslatedCircle(_, _, _, _, _)
                | Kind::VectorTranslatedCircle(_, _, _)
                | Kind::MarkedDistanceTranslatedCircle(_, _, _)
                | Kind::AngleRotatedCircle(_, _, _, _, _)
                | Kind::RotatedCircle(_, _, _)
                | Kind::ReflectedCircle(_, _)
                | Kind::ScaledCircle(_, _, _)
                | Kind::RatioScaledCircle(_, _, _, _, _) => {
                    if let Some((a, radius)) = doc.circle_points(o.id) {
                        let c = vp.screen(a);
                        let r = (radius as f32 * vp.scale).abs();
                        if r > 0.5 && r < 100000. {
                            let n = ((r * 0.45) as usize).clamp(32, 256);
                            let points = (0..n)
                                .map(|i| {
                                    let t = i as f32 * std::f32::consts::TAU / n as f32;
                                    c + egui::vec2(t.cos(), t.sin()) * r
                                })
                                .collect::<Vec<_>>();
                            mesh.styled_path(&points, true, stroke, col);
                        }
                    }
                }
                Kind::ThreePointArc(_, _, _) | Kind::ArcOnCircle(_, _, _) => {
                    if let Some(points) = doc.arc_points(o.id, 128) {
                        let screen = points
                            .into_iter()
                            .map(|point| vp.screen(point))
                            .collect::<Vec<_>>();
                        mesh.styled_path(&screen, false, stroke, col);
                    }
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
                    if let Some(points) = doc.polygon_points(o.id) {
                        let screen = points
                            .into_iter()
                            .map(|point| vp.screen(point))
                            .collect::<Vec<_>>();
                        let fill = match o.visual_style {
                            Some(VisualStyle::Polygon { color }) => rgba(color, 0.35),
                            _ => [0.96, 0.58, 0.22, 0.35],
                        };
                        mesh.polygon(&screen, fill);
                    }
                }
                _ => {}
            }
        }
        for o in &doc.objects {
            if !doc.is_displayed(o.id) {
                continue;
            }
            if let Some(p) = doc.position(o.id) {
                let (radius, color) = match o.visual_style {
                    Some(VisualStyle::Point { shape, color }) => {
                        (point_radius(shape), rgba(color, 1.))
                    }
                    _ => (4., blue),
                };
                mesh.disk(
                    vp.screen(p),
                    if doc.selected.contains(&o.id) {
                        radius + 1.5
                    } else {
                        radius
                    },
                    if doc.selected.contains(&o.id) {
                        selected
                    } else {
                        color
                    },
                );
            }
            if let Some(images) = doc.point_iteration_images(o.id) {
                for point in images {
                    mesh.disk(vp.screen(point), 2., blue);
                }
            }
        }
        let buffer = (!mesh.vertices.is_empty()).then(|| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("sketch vertices"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("sketch"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.96,
                        g: 0.97,
                        b: 0.985,
                        a: 1.,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if let Some(buffer) = buffer {
            let x = (vp.rect.min.x * ppp).max(0.) as u32;
            let y = (vp.rect.min.y * ppp).max(0.) as u32;
            let right = (vp.rect.max.x * ppp).min(size[0] as f32) as u32;
            let bottom = (vp.rect.max.y * ppp).min(size[1] as f32) as u32;
            if right > x && bottom > y {
                pass.set_scissor_rect(x, y, right - x, bottom - y);
                pass.set_pipeline(&self.pipeline);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..mesh.vertices.len() as u32, 0..1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn styled_free_text_uses_each_objects_saved_font_in_svg() {
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 199., y: 125. },
        };
        for (source, first_bold) in [
            (
                include_bytes!("../tests/fixtures/text_two_objects_first_bold_off_original.gsp")
                    .as_slice(),
                false,
            ),
            (
                include_bytes!("../tests/fixtures/text_two_objects_second_bold_off_original.gsp")
                    .as_slice(),
                true,
            ),
        ] {
            let doc = Document::from_bytes(source, None).unwrap();
            let svg = svg_page(&doc, &vp).unwrap();
            let hello = svg
                .lines()
                .find(|line| line.contains(">Hello</text>"))
                .unwrap();
            let world = svg
                .lines()
                .find(|line| line.contains(">World</text>"))
                .unwrap();
            assert!(hello.contains(if first_bold {
                "font-weight=\"bold\""
            } else {
                "font-weight=\"normal\""
            }));
            assert!(world.contains(if first_bold {
                "font-weight=\"normal\""
            } else {
                "font-weight=\"bold\""
            }));
        }
        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/text_two_objects_generated.gsp"),
            None,
        )
        .unwrap();
        doc.selected.insert(2);
        doc.toggle_selected_text_style(crate::document::TextStyleCommand::Italic)
            .unwrap();
        doc.toggle_selected_text_style(crate::document::TextStyleCommand::Underline)
            .unwrap();
        let svg = svg_page(&doc, &vp).unwrap();
        let world = svg
            .lines()
            .find(|line| line.contains(">World</text>"))
            .unwrap();
        assert!(world.contains("font-style=\"italic\""));
        assert!(world.contains("text-decoration=\"underline\""));
        doc.selected.clear();
        doc.selected.insert(1);
        doc.set_selected_text_family("Arial").unwrap();
        let svg = svg_page(&doc, &vp).unwrap();
        let hello = svg
            .lines()
            .find(|line| line.contains(">Hello</text>"))
            .unwrap();
        assert!(hello.contains("font-family=\"Arial\""));
    }
    #[test]
    fn point_labels_use_independent_saved_text_styles_in_svg() {
        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/text_two_point_labels_first_bold_off_original.gsp"),
            None,
        )
        .unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 199., y: 125. },
        };
        let svg = svg_page(&doc, &vp).unwrap();
        let first = svg.lines().find(|line| line.contains(">A</text>")).unwrap();
        let second = svg.lines().find(|line| line.contains(">B</text>")).unwrap();
        assert!(first.contains("font-weight=\"normal\""));
        assert!(second.contains("font-weight=\"bold\""));
        doc.selected.insert(1);
        doc.toggle_selected_text_style(crate::document::TextStyleCommand::IncreaseSize)
            .unwrap();
        let svg = svg_page(&doc, &vp).unwrap();
        let first = svg.lines().find(|line| line.contains(">A</text>")).unwrap();
        let second = svg.lines().find(|line| line.contains(">B</text>")).unwrap();
        assert!(first.contains("font-size=\"36\""));
        assert!(second.contains("font-size=\"24\""));
    }
    #[test]
    fn mixed_format_caption_exports_distinct_svg_spans() {
        let doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/text_mixed_bold_italic_he_original.gsp"),
            None,
        )
        .unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 199., y: 125. },
        };
        let svg = svg_page(&doc, &vp).unwrap();
        let line = svg
            .lines()
            .find(|line| line.contains("He</tspan>"))
            .unwrap();
        assert!(line.contains("font-weight=\"normal\" font-style=\"italic\""));
        assert!(line.contains(
            "<tspan font-size=\"24\" font-weight=\"bold\" font-style=\"normal\" text-decoration=\"none\">llo</tspan>"
        ));
    }
    #[test]
    fn mixed_size_caption_exports_distinct_svg_sizes() {
        let doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/text_mixed_size36_he_original.gsp"),
            None,
        )
        .unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 199., y: 125. },
        };
        let svg = svg_page(&doc, &vp).unwrap();
        let line = svg
            .lines()
            .find(|line| line.contains("He</tspan>"))
            .unwrap();
        assert!(line.contains("<tspan font-size=\"36\""));
        assert!(line.contains("<tspan font-size=\"24\""));
    }
    #[test]
    fn mixed_family_caption_exports_native_font_index() {
        let doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/text_mixed_adventor_he_original.gsp"),
            None,
        )
        .unwrap();
        assert!(!doc.partial_view);
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 199., y: 125. },
        };
        let svg = svg_page(&doc, &vp).unwrap();
        assert!(svg.contains("<tspan font-family=\"TeX Gyre Adventor\""));
    }
    #[test]
    fn dotted_grid_exports_intersection_points() {
        let doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/graph_dotted_grid_original.gsp"),
            None,
        )
        .unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 199., y: 125. },
        };
        assert!(grid_points(&doc, &vp).len() > 40);
        let svg = svg_page(&doc, &vp).unwrap();
        assert!(svg.matches("r=\"1.2\"").count() > 40);
        assert_eq!(svg.matches("<line x1=").count(), 2);
    }
    #[test]
    fn mixed_coordinate_systems_render_square_lines_and_polar_dots_independently() {
        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/graph_two_frames_mixed_grid_generated.gsp"),
            None,
        )
        .unwrap();
        assert_eq!(doc.active_coordinate_system(), Some(9));
        assert_eq!(doc.frame_styles.get(&9), Some(&(GridForm::Square, false)));
        assert_eq!(doc.frame_styles.get(&14), Some(&(GridForm::Polar, true)));
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 199., y: 125. },
        };
        let grid = grid_geometry(&doc, &vp);
        assert!(grid.lines.len() > 10);
        assert!(grid.dots.len() > 30);
        assert!(grid.polar_paths.is_empty());
        let svg = svg_page(&doc, &vp).unwrap();
        assert!(svg.matches("r=\"1.2\"").count() > 30);
        assert!(svg.matches("<line x1=").count() > 10);

        doc.selected.insert(9);
        let selected = grid_geometry(&doc, &vp);
        assert!(selected.selected_lines.len() > 10);
        assert!(selected.lines.is_empty());
        assert!(selected.dots.len() > 30);
        doc.selected.clear();

        doc.set_grid_visible(false);
        let hidden = grid_geometry(&doc, &vp);
        assert!(hidden.lines.is_empty());
        assert!(hidden.dots.len() > 30);
    }
    #[test]
    fn snap_points_keeps_the_visible_grid() {
        let doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/graph_snap_points_original.gsp"),
            None,
        )
        .unwrap();
        assert_eq!(doc.snap_points, vec![true]);
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 199., y: 125. },
        };
        assert!(grid_geometry(&doc, &vp).lines.len() > 10);
        assert!(svg_page(&doc, &vp).unwrap().matches("<line x1=").count() > 10);
    }
    #[test]
    fn circle_frame_axis_ticks_and_labels_follow_its_radius() {
        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/graph_unit_circle_generated.gsp"),
            None,
        )
        .unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(640., 480.)),
            scale: 1.,
            center: Point { x: 323., y: 217. },
        };
        let (ticks, labels) = defined_axis_annotations(&doc, &vp);
        assert!(ticks.len() > 70);
        assert!(
            labels
                .iter()
                .any(|(_, label, _)| label == "−2" || label == "-2")
        );
        assert!(labels.iter().any(|(_, label, _)| label == "0.5"));
        assert!(svg_page(&doc, &vp).unwrap().contains(">0.5</text>"));
        doc.move_point(2, Point { x: 383., y: 217. });
        let moved = defined_axis_annotations(&doc, &vp);
        assert!(moved.0.len() > ticks.len());
        doc.set_grid_visible(false);
        assert!(defined_axis_annotations(&doc, &vp).0.is_empty());
    }
    #[test]
    fn rectangular_distance_frame_uses_independent_axis_tick_spacing() {
        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/graph_two_segments_generated.gsp"),
            None,
        )
        .unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(640., 480.)),
            scale: 1.,
            center: Point { x: 199., y: 125. },
        };
        let (ticks, labels) = defined_axis_annotations(&doc, &vp);
        assert!(ticks.iter().any(|(a, b, _)| a.x == 330. && b.x == 330.));
        assert!(ticks.iter().any(|(a, b, _)| a.y == 225. && b.y == 225.));
        assert!(labels.iter().any(|(_, label, _)| label == "1"));
        assert!(labels.iter().any(|(_, label, _)| label == "0.5"));
        assert!(svg_page(&doc, &vp).unwrap().contains(">0.5</text>"));
        doc.set_grid_visible(false);
        assert!(defined_axis_annotations(&doc, &vp).0.is_empty());
    }
    #[test]
    fn hidden_coordinate_system_does_not_draw_its_axes_or_origin() {
        let mut doc = Document::new();
        doc.add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        doc.set_grid_visible(false);
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 199., y: 125. },
        };
        let svg = svg_page(&doc, &vp).unwrap();
        assert!(!svg.contains("<line "));
        assert!(!svg.contains("<circle "));
    }
    #[test]
    fn polar_grid_draws_rings_and_rectangular_unit_is_visible() {
        let polar = Document::from_bytes(
            include_bytes!("../tests/fixtures/graph_polar_grid_original.gsp"),
            None,
        )
        .unwrap();
        let rectangular = Document::from_bytes(
            include_bytes!("../tests/fixtures/graph_rectangular_grid_original.gsp"),
            None,
        )
        .unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 199., y: 125. },
        };
        let svg = svg_page(&polar, &vp).unwrap();
        assert!(svg.matches("<polyline ").count() > 24);
        assert!(!polar.is_displayed(4));
        assert!(rectangular.is_displayed(4));
        let svg = svg_page(&rectangular, &vp).unwrap();
        assert!(svg.matches("<circle ").count() >= 3);
    }
    #[test]
    fn trigonometric_axis_labels_are_placed_at_pi_multiples() {
        let doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/graph_trigonometric_axis_original.gsp"),
            None,
        )
        .unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 199., y: 125. },
        };
        let labels = trigonometric_axis_labels(&doc, &vp);
        assert_eq!(labels.len(), 2);
        assert!(labels.iter().any(|(_, text)| text == "−π"));
        assert!(labels.iter().any(|(_, text)| text == "π"));
        let svg = svg_page(&doc, &vp).unwrap();
        assert!(svg.contains(">−π</text>"));
        assert!(svg.contains(">π</text>"));
    }
    #[test]
    fn svg_uses_saved_point_stroke_and_polygon_styles() {
        let doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/style_integrated_saved_by_wine.gsp"),
            None,
        )
        .unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(egui::pos2(0., 0.), egui::vec2(800., 600.)),
            scale: 1.,
            center: Point { x: 200., y: 200. },
        };
        let svg = svg_page(&doc, &vp).unwrap();
        assert!(svg.contains("stroke=\"#008000\" stroke-width=\"4\" stroke-dasharray=\"9 6\""));
        assert!(svg.contains("stroke=\"#0000ff\" stroke-width=\"1.5\" stroke-dasharray=\"2 5\""));
        assert!(svg.contains("fill=\"#800080\" fill-opacity=\"0.35\" stroke=\"none\""));
        assert!(svg.contains("r=\"6\" fill=\"#0000ff\""));
    }
    #[test]
    fn picture_hit_and_svg_use_the_same_affine_viewport() {
        let mut doc = Document::new();
        let png = include_bytes!("../tests/fixtures/image_tiny.png");
        doc.import_png(png, Point { x: 120., y: 130. }).unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(egui::pos2(10., 20.), egui::vec2(300., 200.)),
            scale: 2.,
            center: Point { x: 150., y: 150. },
        };
        let Kind::Image { png, transform } = &doc.objects[0].kind else {
            unreachable!()
        };
        let corners = picture_screen_quad(&vp, png, *transform).unwrap();
        assert!(picture_hit(
            &vp,
            png,
            *transform,
            corners[0] + egui::vec2(8., 8.)
        ));
        assert!(!picture_hit(
            &vp,
            png,
            *transform,
            corners[0] - egui::vec2(8., 8.)
        ));
        let svg = svg_page(&doc, &vp).unwrap();
        assert!(svg.contains("data:image/png;base64,iVBORw0KGgo"));
        assert!(svg.contains("<image width=\"16\" height=\"16\""));
    }
    #[test]
    fn svg_export_uses_visible_geometry_and_refuses_partial_documents() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 10., y: 20. }));
        let b = doc.add(Kind::Free(Point { x: 80., y: 20. }));
        doc.add(Kind::Segment(a, b));
        let circle = doc.add(Kind::Circle(a, b));
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(100., 100.)),
            scale: 1.,
            center: Point { x: 50., y: 50. },
        };
        let svg = svg_page(&doc, &vp).unwrap();
        assert!(svg.contains("<line"));
        assert!(svg.contains("<circle"));
        doc.selected.insert(circle);
        doc.set_selected_visibility(false);
        assert!(!svg_page(&doc, &vp).unwrap().contains("r=\"70\""));
        doc.partial_view = true;
        assert!(svg_page(&doc, &vp).is_err());
    }
    #[test]
    fn clips_line_ray_and_segment() {
        let r = Rect::from_min_max(Pos2::new(0., 0.), Pos2::new(100., 100.));
        let a = Pos2::new(50., 50.);
        let b = Pos2::new(60., 50.);
        assert_eq!(
            clipped_line(a, b, LineMode::Line, r),
            Some((Pos2::new(0., 50.), Pos2::new(100., 50.)))
        );
        assert_eq!(
            clipped_line(a, b, LineMode::Ray, r),
            Some((a, Pos2::new(100., 50.)))
        );
        assert_eq!(clipped_line(a, b, LineMode::Segment, r), Some((a, b)));
        assert_eq!(clipped_line(a, a, LineMode::Line, r), None);
    }
    #[test]
    fn action_button_uses_same_rectangle_for_export_and_hit_testing() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 10., y: 20. }));
        let b = doc.add(Kind::Free(Point { x: 100., y: 20. }));
        let segment = doc.add(Kind::Segment(a, b));
        let button = doc.add_hide_show_button(segment).unwrap();
        let viewport = Viewport {
            rect: Rect::from_min_size(Pos2::new(50., 70.), egui::vec2(300., 200.)),
            scale: 1.,
            center: Point { x: 150., y: 100. },
        };
        let Kind::HideShowButton { label, anchor, .. } = &doc.objects[button - 1].kind else {
            unreachable!()
        };
        let bounds = action_button_rect(&viewport, *anchor, label);
        assert_eq!(bounds.min, Pos2::new(60., 75.));
        assert!(bounds.contains(Pos2::new(105., 85.)));
        assert!(
            svg_page(&doc, &viewport)
                .unwrap()
                .contains("Hide Segment</text>")
        );
        assert!(
            svg_page_with_button_offset(&doc, &viewport, egui::vec2(40., 10.))
                .unwrap()
                .contains("<rect x=\"100\" y=\"85\"")
        );
        doc.activate_hide_show_button(button).unwrap();
        assert!(
            svg_page(&doc, &viewport)
                .unwrap()
                .contains("Show Segment</text>")
        );
    }
    #[test]
    fn svg_length_tracks_geometry_and_visibility() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = doc.add(Kind::Free(Point { x: 100., y: 0. }));
        let segment = doc.add(Kind::Segment(a, b));
        let measurement = doc.add(Kind::SegmentLength(segment));
        let distance = doc.add(Kind::PointDistance(a, b));
        let circle = doc.add(Kind::Circle(a, b));
        let circumference = doc.add(Kind::CircleCircumference(circle));
        let radius = doc.add(Kind::CircleRadius(circle));
        let viewport = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(200., 100.)),
            scale: 1.,
            center: Point { x: 100., y: 50. },
        };
        assert!(
            svg_page(&doc, &viewport)
                .unwrap()
                .contains("Length = 2.65 cm")
        );
        assert!(
            svg_page(&doc, &viewport)
                .unwrap()
                .contains("Distance = 2.65 cm")
        );
        assert!(
            svg_page(&doc, &viewport)
                .unwrap()
                .contains("Circumference = 16.62 cm")
        );
        assert!(
            svg_page(&doc, &viewport)
                .unwrap()
                .contains("Radius = 2.65 cm")
        );
        doc.checkpoint();
        doc.move_point(b, Point { x: 200., y: 0. });
        assert!(
            svg_page(&doc, &viewport)
                .unwrap()
                .contains("Length = 5.29 cm")
        );
        assert!(
            svg_page(&doc, &viewport)
                .unwrap()
                .contains("Distance = 5.29 cm")
        );
        assert!(
            svg_page(&doc, &viewport)
                .unwrap()
                .contains("Circumference = 33.25 cm")
        );
        assert!(
            svg_page(&doc, &viewport)
                .unwrap()
                .contains("Radius = 5.29 cm")
        );
        doc.selected.insert(measurement);
        doc.selected.insert(distance);
        doc.selected.insert(circumference);
        doc.selected.insert(radius);
        doc.set_selected_visibility(false);
        assert!(!svg_page(&doc, &viewport).unwrap().contains("Length ="));
        assert!(!svg_page(&doc, &viewport).unwrap().contains("Distance ="));
        assert!(
            !svg_page(&doc, &viewport)
                .unwrap()
                .contains("Circumference =")
        );
        assert!(!svg_page(&doc, &viewport).unwrap().contains("Radius ="));
    }
}
