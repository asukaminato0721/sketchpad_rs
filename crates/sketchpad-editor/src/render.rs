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

// Keep each connected curve together so dash phase survives sample boundaries.
fn curve_screen_paths(vp: &Viewport, segments: &[(Point, Point, LineMode)]) -> Vec<Vec<Pos2>> {
    let mut paths = Vec::new();
    let mut current: Vec<Pos2> = Vec::new();
    for &(start, end, mode) in segments {
        if let Some((a, b)) = clipped_line(vp.screen(start), vp.screen(end), mode, vp.rect) {
            if current.last().copied() == Some(a) && mode == LineMode::Segment {
                current.push(b);
            } else {
                if !current.is_empty() {
                    paths.push(std::mem::take(&mut current));
                }
                current.extend([a, b]);
            }
            if mode != LineMode::Segment {
                paths.push(std::mem::take(&mut current));
            }
        } else if !current.is_empty() {
            paths.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        paths.push(current);
    }
    paths
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
fn display_parameter_indices(text: &str) -> String {
    let mut output = String::new();
    let mut remaining = text;
    while let Some(start) = remaining.find("t[") {
        output.push_str(&remaining[..start]);
        let suffix = &remaining[start + 2..];
        if let Some(end) = suffix.find(']')
            && !suffix[..end].is_empty()
            && suffix[..end].bytes().all(|digit| digit.is_ascii_digit())
        {
            output.push('t');
            for digit in suffix[..end].bytes() {
                output.push(
                    ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'][usize::from(digit - b'0')],
                );
            }
            remaining = &suffix[end + 1..];
        } else {
            output.push_str("t[");
            remaining = suffix;
        }
    }
    output.push_str(remaining);
    output
}

pub fn number_lines(kind: &Kind) -> Option<((i16, i16), Vec<String>)> {
    match kind {
        Kind::Parameter(group) => Some((
            group.anchor(),
            vec![format!(
                "{} = {}",
                display_parameter_indices(&if group.display_mode() == 0 {
                    group.literal_definition()
                } else {
                    group.name().to_owned()
                }),
                group.value_text()
            )],
        )),
        Kind::Calculation(group) => Some((
            group.anchor(),
            vec![format!("{} = {}", group.expression(), group.value_text())],
        )),
        Kind::ValueTable(_, group) => {
            if let Some((name, values)) = group.named_label_values() {
                let mut lines = vec![display_parameter_indices(&name)];
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
    if let Kind::Parameter(group) = &object.kind {
        let name = doc.parameter_display_name(id)?;
        let name = display_parameter_indices(&name);
        return Some((
            group.anchor(),
            vec![format!("{name} = {}", group.value_text())],
        ));
    }
    if let Kind::Calculation(group) = &object.kind
        && group.is_native_radian_angle()
        && group.display_mode() == 3
        && let Some(name) = object
            .label
            .as_ref()
            .and_then(|label| label.name_utf8().ok())
            .filter(|name| !name.is_empty())
    {
        return Some((
            group.anchor(),
            vec![format!("{name} = {}", group.value_text())],
        ));
    }
    if let Kind::LengthPairCalculation(_, _, group) = &object.kind {
        let value = doc.length_pair_calculation_value(id).map_or_else(
            || "undefined".into(),
            |value| crate::table_records::format_numeric_value(value, group.precision()),
        );
        let unit = match group.length_power() {
            1 => " cm",
            2 => " cm²",
            _ => "",
        };
        return Some((
            group.anchor(),
            vec![format!(
                "{} = {value}{unit}",
                doc.length_measurement_name(id)?
            )],
        ));
    }
    if let Kind::MeasuredCalculation(source, group) = &object.kind {
        let value = crate::table_records::format_numeric_value(
            doc.measured_calculation_cm(id)?,
            group.precision(),
        );
        let name = doc
            .length_measurement_name(*source)
            .unwrap_or_else(|| "Length".into());
        return Some((
            group.anchor(),
            vec![format!(
                "{name} {} {} = {value} cm",
                if group.divides() { "÷" } else { "·" },
                group.factor()
            )],
        ));
    }
    if let Kind::ParameterCalculation(_, group) = &object.kind {
        if group.caption_suppressed() {
            return None;
        }
        let value = doc
            .parameter_calculation_display_value(id)
            .map_or_else(|| "undefined".into(), |value| group.value_text(value));
        let name = object
            .label
            .as_ref()
            .and_then(|label| label.name_utf8().ok())
            .filter(|name| !name.is_empty());
        let caption = match (group.display_mode(), name) {
            (1 | 3, Some(name)) => format!("{name} = {value}"),
            (1, None) => value,
            _ => {
                let expression = doc.parameter_calculation_expression(id)?;
                let expression = display_parameter_indices(&expression);
                format!("{expression} = {value}")
            }
        };
        return Some((group.anchor(), vec![caption]));
    } else if let Kind::ValueTable(source, group) = &object.kind
        && let Some((name, mut values)) = group.named_label_values()
    {
        if let Some(Kind::Parameter(parameter)) = doc
            .objects
            .get(source.checked_sub(1)?)
            .map(|object| &object.kind)
            && let Some(last) = values.last_mut()
        {
            *last = crate::table_records::named_table_value_text(
                parameter.value(),
                parameter.precision(),
            )
            .ok()?;
        }
        let mut lines = vec![display_parameter_indices(&name)];
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

fn numeric_label_rect(vp: &Viewport, point: Point, text: &str) -> Rect {
    Rect::from_center_size(
        vp.screen(point),
        egui::vec2(text.chars().count() as f32 * 10. + 8., 22.),
    )
}

pub fn point_parameter_iteration_label_hit(
    vp: &Viewport,
    doc: &Document,
    id: usize,
    at: Pos2,
) -> bool {
    doc.point_parameter_iteration_labels(id)
        .is_some_and(|labels| {
            labels
                .into_iter()
                .any(|(point, text)| numeric_label_rect(vp, point, &text).contains(at))
        })
}

pub fn number_screen_rect_for_object(vp: &Viewport, doc: &Document, id: usize) -> Option<Rect> {
    if let Some((point, text)) = doc.point_numeric_label(id) {
        return Some(numeric_label_rect(vp, point, &text));
    }
    let (anchor, lines) = number_lines_for_object(doc, id)?;
    Some(number_rect(vp, anchor, &lines))
}

pub fn mapped_numeric_label_hit(vp: &Viewport, doc: &Document, id: usize, at: Pos2) -> bool {
    let size = doc.numeric_label_font_state(id).map_or(16., |style| f32::from(style.size_code));
    doc.mapped_numeric_label_images(id).is_some_and(|labels| labels.into_iter().any(|(point, text)| {
        Rect::from_center_size(vp.screen(point), egui::vec2(text.chars().count() as f32 * size * 0.65, size * 1.1)).contains(at)
    }))
}
pub fn repeated_polygon_opacity(opacity: u16, paths: f64) -> f32 {
    (1. - (1. - f64::from(opacity) / 65535.).powf(paths)) as f32
}

pub fn circle_iteration_hit(vp: &Viewport, doc: &Document, id: usize, at: Pos2) -> bool {
    doc.circle_iteration_images(id).is_some_and(|circles| {
        circles.into_iter().any(|(center, radius)| {
            (at.distance(vp.screen(center)) - (radius as f32 * vp.scale).abs()).abs() < 7.
        })
    })
}

// Reorder filled region slots; preserve all other drawing and picking order.
fn ordered_region_fills(doc: &Document) -> Vec<&crate::document::Object> {
    let mut polygons: Vec<_> = doc
        .objects
        .iter()
        .filter(|o| doc.is_displayed(o.id) && o.kind.is_filled_region())
        .collect();
    polygons.sort_by_key(|o| o.interior_order.unwrap_or(0));
    polygons
}

pub fn point_iteration_hit(vp: &Viewport, doc: &Document, id: usize, at: Pos2) -> bool {
    doc.mapped_point_binding_images(id).is_some_and(|images| {
        images
            .into_iter()
            .any(|point| vp.screen(point).distance(at) < 11.)
    })
}

pub fn branch_iteration_hit(vp: &Viewport, doc: &Document, id: usize, at: Pos2) -> bool {
    if let Some(disks) = doc.colorized_disk_iteration_images(id) {
        return disks.into_iter().any(|(center, radius, _)|
            vp.screen(center).distance(at) <= (radius as f32 * vp.scale).abs());
    }
    if let Some(polygons) = doc.polygon_iteration_images(id) {
        return polygons.into_iter().any(|(points, _)| {
            let points = points.into_iter().map(|p| vp.screen(p)).collect::<Vec<_>>();
            let mut inside = false;
            for i in 0..points.len() {
                let (a, b) = (points[i], points[(i + 1) % points.len()]);
                if (a.y > at.y) != (b.y > at.y)
                    && at.x < (b.x - a.x) * (at.y - a.y) / (b.y - a.y) + a.x
                {
                    inside = !inside;
                }
            }
            inside
        });
    }
    let near = |a: Pos2, b: Pos2| {
        let delta = b - a;
        let t = if delta.length_sq() > 0. {
            ((at - a).dot(delta) / delta.length_sq()).clamp(0., 1.)
        } else {
            0.
        };
        at.distance(a + delta * t) < 7.
    };
    if doc.mapped_collection_strokes(id, 384).is_some_and(|strokes| {
        strokes.into_iter().any(|stroke| {
            clipped_line(vp.screen(stroke.start), vp.screen(stroke.end), stroke.mode, vp.rect)
                .is_some_and(|(a, b)| near(a, b))
        })
    }) {
        return true;
    }
    if doc
        .branch_segment_iteration_images(id)
        .is_some_and(|images| {
            images
                .into_iter()
                .any(|(a, b)| near(vp.screen(a), vp.screen(b)))
        })
    {
        return true;
    }
    for (images, mode) in [
        (doc.ray_iteration_images(id), LineMode::Ray),
        (doc.line_iteration_images(id), LineMode::Line),
    ] {
        if images.is_some_and(|images| {
            images.into_iter().any(|(a, b)| {
                clipped_line(vp.screen(a), vp.screen(b), mode, vp.rect)
                    .is_some_and(|(a, b)| near(a, b))
            })
        }) {
            return true;
        }
    }
    doc.arc_iteration_images(id, 96).is_some_and(|images| {
        images.into_iter().any(|points| {
            points
                .windows(2)
                .any(|pair| near(vp.screen(pair[0]), vp.screen(pair[1])))
        })
    })
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
        if let Kind::Image { png, .. } = &object.kind {
            let Some(transform) = doc.image_transform(object.id) else { continue };
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
    let mut polygon_fills = ordered_region_fills(doc).into_iter();
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
                svg_stroke(doc.effective_visual_style(o.id))
            )
            .unwrap();
        }
        if let Some(strokes) = doc.locus_styled_strokes(o.id, 384)
            .or_else(|| doc.mapped_collection_strokes(o.id, 384)) {
            for sample in strokes {
                if let Some((a, b)) = clipped_line(vp.screen(sample.start), vp.screen(sample.end), sample.mode, r) {
                    writeln!(svg, "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"{}/>", a.x, a.y, b.x, b.y, svg_stroke(Some(sample.style))).unwrap();
                }
            }
        } else if let Some(polygons) = doc.locus_polygon_surfaces(o.id, 384) {
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
                .or_else(|| doc.sampled_mapped_strokes(o.id, 384))
        {
            if matches!(doc.effective_visual_style(o.id), Some(VisualStyle::Stroke { .. })) {
                for path in curve_screen_paths(vp, &segments) {
                    let points = path
                        .iter()
                        .map(|p| format!("{},{}", p.x, p.y))
                        .collect::<Vec<_>>()
                        .join(" ");
                    writeln!(
                        svg,
                        "<polyline points=\"{}\"{}/>",
                        points,
                        svg_stroke(doc.effective_visual_style(o.id))
                    )
                    .unwrap();
                }
            } else {
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
        }
        if let Some((center, radius)) = doc.circle_points(o.id) {
            let at = vp.screen(center);
            writeln!(
                svg,
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\"{}/>",
                at.x,
                at.y,
                radius as f32 * vp.scale,
                svg_stroke(doc.effective_visual_style(o.id))
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
                    svg_stroke(doc.effective_visual_style(o.id)),
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
        if let Some(paths) = doc.angle_marker_strokes(o.id) {
            for points in paths {
                let values = points.into_iter().map(|point| {
                    let at = vp.screen(point);
                    format!("{},{}", at.x, at.y)
                }).collect::<Vec<_>>().join(" ");
                writeln!(svg, "<polyline points=\"{}\"{}/>", values,
                    svg_stroke(doc.effective_visual_style(o.id))).unwrap();
            }
        }
        let polygon_object = if o.kind.is_filled_region() {
            polygon_fills.next().unwrap()
        } else {
            o
        };
        if let Some(points) = doc.filled_region_points(polygon_object.id) {
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
                "<polygon points=\"{}\" fill=\"{}\" fill-opacity=\"{}\" stroke=\"none\"/>",
                vertices,
                match doc.effective_visual_style(polygon_object.id) {
                    Some(VisualStyle::Polygon { color, .. }) => css_color(color),
                    _ => "#f6ad60".into(),
                },
                match doc.effective_visual_style(polygon_object.id) {
                    Some(VisualStyle::Polygon { opacity, .. }) => f64::from(opacity) / 65535.,
                    _ => 0.5,
                }
            )
            .unwrap();
        }
        if let Some(polygons) = doc.colorized_polygon_iteration_images(o.id) {
            let opacity = match doc.effective_visual_style(o.id) {
                Some(VisualStyle::Polygon { opacity, .. }) => opacity,
                _ => 32767,
            };
            for (points, color) in polygons {
                let vertices = points.into_iter().map(|point| {
                    let at = vp.screen(point);
                    format!("{},{}", at.x, at.y)
                }).collect::<Vec<_>>().join(" ");
                writeln!(svg,
                    "<polygon points=\"{}\" fill=\"{}\" fill-opacity=\"{}\" stroke=\"none\"/>",
                    vertices, css_color(color), f64::from(opacity) / 65535.
                ).unwrap();
            }
        } else if let Some(polygons) = doc.polygon_iteration_images(o.id) {
            let (color, opacity) = match doc.effective_visual_style(o.id) {
                Some(VisualStyle::Polygon { color, opacity }) => (css_color(color), opacity),
                _ => ("#f6ad60".into(), 32767),
            };
            for (points, paths) in polygons {
                let vertices = points
                    .into_iter()
                    .map(|p| {
                        let at = vp.screen(p);
                        format!("{},{}", at.x, at.y)
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                writeln!(
                    svg,
                    "<polygon points=\"{}\" fill=\"{}\" fill-opacity=\"{}\" stroke=\"none\"/>",
                    vertices,
                    color,
                    repeated_polygon_opacity(opacity, paths)
                )
                .unwrap();
            }
        }
        if let Some(disks) = doc.colorized_disk_iteration_images(o.id) {
            let opacity = match doc.effective_visual_style(o.id) {
                Some(VisualStyle::Polygon { opacity, .. }) => opacity,
                _ => 32767,
            };
            for (center, radius, color) in disks {
                let at = vp.screen(center);
                writeln!(svg,
                    "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\" fill-opacity=\"{}\" stroke=\"none\"/>",
                    at.x, at.y, radius as f32 * vp.scale, css_color(color), f64::from(opacity) / 65535.
                ).unwrap();
            }
        }
        if let Some(segments) = doc.branch_segment_iteration_images(o.id) {
            for (a, b) in segments {
                let (a, b) = (vp.screen(a), vp.screen(b));
                writeln!(
                    svg,
                    "<line{} x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
                    svg_stroke(doc.effective_visual_style(o.id)),
                    a.x,
                    a.y,
                    b.x,
                    b.y
                )
                .unwrap();
            }
        }
        if let Some(circles) = doc.circle_iteration_images(o.id) {
            for (center, radius) in circles {
                let center = vp.screen(center);
                writeln!(
                    svg,
                    "<circle{} cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"none\"/>",
                    svg_stroke(doc.effective_visual_style(o.id)),
                    center.x,
                    center.y,
                    (radius as f32 * vp.scale).abs()
                )
                .unwrap();
            }
        }
        if let Some(segments) = doc.segment_iteration_images(o.id) {
            for (start, end) in segments {
                let start = vp.screen(start);
                let end = vp.screen(end);
                writeln!(svg, "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#176bd6\" stroke-width=\"2\"/>", start.x, start.y, end.x, end.y).unwrap();
            }
        }
        if let Some(lines) = doc.line_iteration_images(o.id) {
            for (start, end) in lines {
                if let Some((a, b)) =
                    clipped_line(vp.screen(start), vp.screen(end), LineMode::Line, r)
                {
                    writeln!(
                        svg,
                        "<line{} x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
                        svg_stroke(doc.effective_visual_style(o.id)),
                        a.x,
                        a.y,
                        b.x,
                        b.y
                    )
                    .unwrap();
                }
            }
        }
        if let Some(arcs) = doc.arc_iteration_images(o.id, 96) {
            for points in arcs {
                let vertices = points
                    .into_iter()
                    .map(|p| {
                        let at = vp.screen(p);
                        format!("{},{}", at.x, at.y)
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                writeln!(
                    svg,
                    "<polyline{} points=\"{}\" fill=\"none\"/>",
                    svg_stroke(doc.effective_visual_style(o.id)),
                    vertices
                )
                .unwrap();
            }
        }
        if let Some(rays) = doc.ray_iteration_images(o.id) {
            for (start, through) in rays {
                if let Some((a, b)) =
                    clipped_line(vp.screen(start), vp.screen(through), LineMode::Ray, r)
                {
                    writeln!(
                        svg,
                        "<line{} x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
                        svg_stroke(doc.effective_visual_style(o.id)),
                        a.x,
                        a.y,
                        b.x,
                        b.y
                    )
                    .unwrap();
                }
            }
        }
    }
    svg.push_str("</g>\n<g fill=\"#176bd6\">\n");
    for o in doc.objects.iter().filter(|o| doc.is_displayed(o.id)) {
        if let Some(p) = doc.position(o.id) {
            let at = vp.screen(p);
            let (radius, color) = match doc.effective_visual_style(o.id) {
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
            let (radius, color) = match doc.effective_visual_style(object.id) {
                Some(VisualStyle::Point { shape, color }) => {
                    (point_radius(shape), css_color(color))
                }
                _ => (2., "#176bd6".into()),
            };
            for point in images {
                let at = vp.screen(point);
                writeln!(
                    svg,
                    "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"/>",
                    at.x, at.y, radius, color
                )
                .unwrap();
            }
        }
    }
    svg.push_str("</g>\n<g fill=\"#232323\" font-family=\"sans-serif\" font-size=\"16\">\n");
    for object in doc.objects.iter().filter(|o| doc.is_displayed(o.id)) {
        if let Some((text, rect)) = doc.text_content(object.id) {
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
                        let run_color = run.style.rgb_color().map(|[red, green, blue]| {
                            format!(" fill=\"#{red:02x}{green:02x}{blue:02x}\"")
                        }).unwrap_or_default();
                        write!(
                            svg,
                            "<tspan{}{} font-size=\"{}\" font-weight=\"{}\" font-style=\"{}\" text-decoration=\"{}\">{}</tspan>",
                            run_family,
                            run_color,
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
        if let Some(labels) = doc.point_parameter_iteration_labels(object.id) {
            for (point, text) in labels {
                let at = vp.screen(point);
                writeln!(svg,"<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"central\">{}</text>",at.x,at.y,xml_text(&text)).unwrap();
            }
        }
        for (point, text) in doc.point_numeric_label(object.id).map(|label| vec![label]).or_else(|| doc.mapped_numeric_label_images(object.id)).unwrap_or_default() {
            let at = vp.screen(point);
            let style = doc.numeric_label_font_state(object.id);
            let size = style.as_ref().map_or(16, |style| style.size_code);
            let family = style.as_ref().map_or("sans-serif", |style| style.family.as_str())
                .replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;");
            writeln!(svg,"<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"central\" font-family=\"{}\" font-size=\"{}\" font-weight=\"{}\" font-style=\"{}\" text-decoration=\"{}\">{}</text>",
                at.x,at.y,family,size,
                if style.as_ref().is_some_and(|style| style.bold) { "bold" } else { "normal" },
                if style.as_ref().is_some_and(|style| style.italic) { "italic" } else { "normal" },
                if style.as_ref().is_some_and(|style| style.underline) { "underline" } else { "none" },
                xml_text(&text)).unwrap();
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
            && let Some(point) = doc.position(object.id).or_else(||
                doc.angle_marker_strokes(object.id)?.into_iter().next()?.get(12).copied())
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
        let caption = doc
            .measurement_text(object.id)
            .or_else(|| {
                matches!(object.kind, Kind::SlopeMeasurement(_, _)).then(|| {
                    format!(
                        "Slope = {}",
                        doc.measurement_value_text(object.id)
                            .unwrap_or_else(|| "undefined".into())
                    )
                })
            })
            .or_else(|| doc.coordinate_measure_text(object.id))
            .or_else(|| doc.value_equation_text(object.id));
        if let Some(text) = caption {
            let style = doc.text_font_state(object.id);
            let size = style.as_ref().map_or(16, |style| style.size_code);
            let (x, y) = object
                .measurement_anchor
                .map(|(x, y)| {
                    (
                        r.min.x + x as f32 + 2.,
                        r.min.y + y as f32 + 7. + size as f32 * 0.85,
                    )
                })
                .unwrap_or((
                    r.min.x + 12.,
                    r.min.y + 12. + row as f32 * 20. + size as f32 * 0.85,
                ));
            writeln!(svg, "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\" font-weight=\"{}\" font-style=\"{}\" text-decoration=\"{}\">{}</text>",
                x, y, xml_text(style.as_ref().map_or("sans-serif", |style| &style.family)).replace('"', "&quot;"), size,
                if style.as_ref().is_some_and(|style| style.bold) { "bold" } else { "normal" },
                if style.as_ref().is_some_and(|style| style.italic) { "italic" } else { "normal" },
                if style.as_ref().is_some_and(|style| style.underline) { "underline" } else { "none" }, xml_text(&text)).unwrap();
            row += 1;
        }
        if let Kind::FunctionDefinition(expression) | Kind::ReferencedFunction(_, expression) =
            object.kind
        {
            let (x, y) = object
                .function_anchor
                .map(|(x, y)| (r.min.x + x as f32 + 2., r.min.y + y as f32 + 23.))
                .unwrap_or((r.min.x + 12., r.min.y + 28. + row as f32 * 20.));
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">{}</text>",
                x,
                y,
                xml_text(
                    &doc.function_display_text(object.id)
                        .unwrap_or_else(|| expression.display())
                )
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
                xml_text(
                    &doc.function_display_text(object.id)
                        .unwrap_or_else(|| { expression.display().replacen("f(x)", "f′(x)", 1) })
                )
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
    fn locus_strokes(
        &mut self,
        vp: &Viewport,
        samples: &[crate::document::LocusStroke],
        selection_color: Option<[f32; 4]>,
    ) {
        for sample in samples {
            let VisualStyle::Stroke { line, color } = sample.style else {
                continue;
            };
            if let Some((a, b)) = clipped_line(
                vp.screen(sample.start),
                vp.screen(sample.end),
                sample.mode,
                vp.rect,
            ) {
                self.styled_path(&[a, b], false, line, selection_color.unwrap_or_else(|| rgba(color, 1.)));
            }
        }
    }
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
        let mut polygon_fills = ordered_region_fills(doc).into_iter();
        for o in &doc.objects {
            if !doc.is_displayed(o.id) {
                continue;
            }
            let is_selected = doc.selected.contains(&o.id)
                || matches!(o.kind,Kind::IterationBinding(_,parent) if doc.selected.contains(&parent));
            let col = if is_selected {
                selected
            } else {
                match doc.effective_visual_style(o.id) {
                    Some(VisualStyle::Stroke { color, .. }) => rgba(color, 1.),
                    _ => dark,
                }
            };
            let stroke = match doc.effective_visual_style(o.id) {
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
            if let Some(polygons) = doc.colorized_polygon_iteration_images(o.id) {
                let opacity = match doc.effective_visual_style(o.id) {
                    Some(VisualStyle::Polygon { opacity, .. }) => opacity,
                    _ => 32767,
                };
                for (points, color) in polygons {
                    let screen = points.into_iter().map(|p| vp.screen(p)).collect::<Vec<_>>();
                    mesh.polygon(&screen,
                        if is_selected { [selected[0], selected[1], selected[2], f32::from(opacity) / 65535.] }
                        else { rgba(color, f32::from(opacity) / 65535.) });
                }
            } else if let Some(polygons) = doc.polygon_iteration_images(o.id) {
                let (color, opacity) = match doc.effective_visual_style(o.id) {
                    Some(VisualStyle::Polygon { color, opacity }) => (color, opacity),
                    _ => (
                        RgbColor {
                            red: 255,
                            green: 128,
                            blue: 0,
                        },
                        32767,
                    ),
                };
                for (points, paths) in polygons {
                    let screen = points.into_iter().map(|p| vp.screen(p)).collect::<Vec<_>>();
                    mesh.polygon(
                        &screen,
                        if is_selected {
                            [
                                selected[0],
                                selected[1],
                                selected[2],
                                repeated_polygon_opacity(opacity, paths),
                            ]
                        } else {
                            rgba(color, repeated_polygon_opacity(opacity, paths))
                        },
                    );
                }
            }
            if let Some(disks) = doc.colorized_disk_iteration_images(o.id) {
                let opacity = match doc.effective_visual_style(o.id) {
                    Some(VisualStyle::Polygon { opacity, .. }) => opacity,
                    _ => 32767,
                };
                for (center, radius, color) in disks {
                    let r = (radius as f32 * vp.scale).abs();
                    if r > 0.5 && r < 100000. {
                        mesh.disk(vp.screen(center), r,
                            if is_selected { [selected[0], selected[1], selected[2], f32::from(opacity) / 65535.] }
                            else { rgba(color, f32::from(opacity) / 65535.) });
                    }
                }
            }
            if let Some(segments) = doc.branch_segment_iteration_images(o.id) {
                for (a, b) in segments {
                    if let Some((a, b)) =
                        clipped_line(vp.screen(a), vp.screen(b), LineMode::Segment, vp.rect)
                    {
                        mesh.styled_path(&[a, b], false, stroke, col);
                    }
                }
            }
            if let Some(circles) = doc.circle_iteration_images(o.id) {
                for (center, radius) in circles {
                    let c = vp.screen(center);
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
            if let Some(segments) = doc.segment_iteration_images(o.id) {
                for (start, end) in segments {
                    if let Some((a, b)) =
                        clipped_line(vp.screen(start), vp.screen(end), LineMode::Segment, vp.rect)
                    {
                        mesh.styled_path(&[a, b], false, stroke, col);
                    }
                }
            }
            if let Some(lines) = doc.line_iteration_images(o.id) {
                for (start, end) in lines {
                    if let Some((a, b)) =
                        clipped_line(vp.screen(start), vp.screen(end), LineMode::Line, vp.rect)
                    {
                        mesh.styled_path(&[a, b], false, stroke, col);
                    }
                }
            }
            if let Some(arcs) = doc.arc_iteration_images(o.id, 96) {
                for points in arcs {
                    let points = points.into_iter().map(|p| vp.screen(p)).collect::<Vec<_>>();
                    mesh.styled_path(&points, false, stroke, col);
                }
            }
            if let Some(rays) = doc.ray_iteration_images(o.id) {
                for (start, through) in rays {
                    if let Some((a, b)) =
                        clipped_line(vp.screen(start), vp.screen(through), LineMode::Ray, vp.rect)
                    {
                        mesh.styled_path(&[a, b], false, stroke, col);
                    }
                }
            }
            if let Some(strokes) = doc.locus_styled_strokes(o.id, 384)
                .or_else(|| doc.mapped_collection_strokes(o.id, 384)) {
                mesh.locus_strokes(vp, &strokes, is_selected.then_some(selected));
            } else if let Some(polygons) = doc.locus_polygon_surfaces(o.id, 384) {
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
                .or_else(|| doc.sampled_mapped_strokes(o.id, 384))
            {
                if matches!(doc.effective_visual_style(o.id), Some(VisualStyle::Stroke { .. })) {
                    for path in curve_screen_paths(vp, &segments) {
                        mesh.styled_path(&path, false, stroke, col);
                    }
                } else {
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
            }
            match o.kind {
                ref kind if kind.is_circle() => {
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
                ref kind if kind.is_arc() => {
                    if let Some(points) = doc.arc_points(o.id, 128) {
                        let screen = points
                            .into_iter()
                            .map(|point| vp.screen(point))
                            .collect::<Vec<_>>();
                        mesh.styled_path(&screen, false, stroke, col);
                    }
                }
                Kind::AngleMarker(..) => {
                    if let Some(paths) = doc.angle_marker_strokes(o.id) {
                        for path in paths {
                            let screen = path.into_iter().map(|point| vp.screen(point)).collect::<Vec<_>>();
                            mesh.styled_path(&screen, false, stroke, col);
                        }
                    }
                }
                ref kind if kind.is_filled_region() => {
                    let polygon_object = polygon_fills.next().unwrap();
                    if let Some(points) = doc.filled_region_points(polygon_object.id) {
                        let screen = points
                            .into_iter()
                            .map(|point| vp.screen(point))
                            .collect::<Vec<_>>();
                        let fill = match doc.effective_visual_style(polygon_object.id) {
                            Some(VisualStyle::Polygon { color, opacity }) => {
                                rgba(color, f32::from(opacity) / 65535.)
                            }
                            _ => [0.96, 0.58, 0.22, 0.5],
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
                let (radius, color) = match doc.effective_visual_style(o.id) {
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
                let (radius, color) = match doc.effective_visual_style(o.id) {
                    Some(VisualStyle::Point { shape, color }) => {
                        (point_radius(shape), rgba(color, 1.))
                    }
                    _ => (2., blue),
                };
                let highlighted = doc.selected.contains(&o.id)
                    || matches!(o.kind, Kind::IterationBinding(_, parent) if doc.selected.contains(&parent));
                for point in images {
                    mesh.disk(
                        vp.screen(point),
                        if highlighted { radius + 1.5 } else { radius },
                        if highlighted { selected } else { color },
                    );
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
    fn native_angle_markers_export_one_and_two_live_svg_arcs() {
        let viewport = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(1000., 800.)),
            scale: 1.,
            center: Point { x: 500., y: 400. },
        };
        let mut hidden = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/angle_marker87_hide_both_native_saved.gsp"),
            None,
        ).unwrap();
        hidden.partial_view = false;
        let baseline = svg_page(&hidden, &viewport).unwrap().matches("<polyline").count();
        for (bytes, expected_arcs) in [
            (include_bytes!("../../../tests/fixtures/angle_marker87_original_native_saved.gsp").as_slice(), 3),
            (include_bytes!("../../../tests/fixtures/angle_marker87_hide13_native_saved.gsp").as_slice(), 2),
            (include_bytes!("../../../tests/fixtures/angle_marker87_hide14_native_saved.gsp").as_slice(), 1),
        ] {
            let mut doc = Document::from_bytes(bytes, None).unwrap();
            // This fixture contains unrelated opaque objects; isolate the
            // already-decoded marker strokes for renderer verification.
            doc.partial_view = false;
            let svg = svg_page(&doc, &viewport).unwrap();
            assert_eq!(svg.matches("<polyline").count(), baseline + expected_arcs);
        }
    }

    #[test]
    fn mapped_colorized_disks_export_each_generation_with_its_live_fill() {
        let viewport = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(1000., 800.)),
            scale: 1.,
            center: Point { x: 500., y: 400. },
        };
        let mut original = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/colorized_iteration88_original_native_saved.gsp"),
            None,
        ).unwrap();
        let mut hidden = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/colorized_iteration88_hide_fill_binding20_native_saved.gsp"),
            None,
        ).unwrap();
        let generations = original.colorized_disk_iteration_images(20).unwrap().len();
        original.partial_view = false;
        hidden.partial_view = false;
        let svg = svg_page(&original, &viewport).unwrap();
        let baseline = svg_page(&hidden, &viewport).unwrap();
        assert_eq!(svg.matches("fill-opacity=").count(), baseline.matches("fill-opacity=").count() + generations);
        assert!(svg.contains("fill=\"#ff0000\""));
        assert!(svg.contains("fill=\"#1f42f5\""));
    }

    #[test]
    fn mapped_colorized_polygons_export_each_generation_with_its_live_fill() {
        let viewport = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(1000., 800.)),
            scale: 1.,
            center: Point { x: 500., y: 400. },
        };
        let mut doc = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/colorized_polygon89_depth3_native_saved.gsp"),
            None,
        ).unwrap();
        let images = doc.colorized_polygon_iteration_images(23).unwrap();
        let distinct = images.iter().map(|(_, color)| css_color(*color)).collect::<std::collections::HashSet<_>>();
        assert!(distinct.len() >= 3);
        for object in &mut doc.objects {
            object.visible = object.id == 23;
        }
        doc.partial_view = false;
        let svg = svg_page(&doc, &viewport).unwrap();
        assert_eq!(svg.matches("<polygon ").count(), images.len());
        for color in distinct {
            assert!(svg.contains(&format!("fill=\"{color}\"")), "missing {color}");
        }
    }

    #[test]
    fn direct_colorized_polygon_svg_recolors_without_changing_its_outline() {
        let viewport = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(1000., 800.)),
            scale: 1.,
            center: Point { x: 500., y: 400. },
        };
        let mut svg_results = Vec::new();
        for bytes in [
            include_bytes!("../../../tests/fixtures/colorized_polygon89_hide_binding23_native_saved.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/colorized_polygon89_range_hi2_hide_binding23_native_saved.gsp").as_slice(),
        ] {
            let mut doc = Document::from_bytes(bytes, None).unwrap();
            let color = css_color(doc.colorized_color(14).unwrap());
            for object in &mut doc.objects {
                object.visible = object.id == 14;
            }
            doc.partial_view = false;
            let svg = svg_page(&doc, &viewport).unwrap();
            assert_eq!(svg.matches("<polygon ").count(), 1);
            assert!(svg.contains(&format!("fill=\"{color}\"")));
            svg_results.push((svg, color));
        }
        assert_ne!(svg_results[0].1, svg_results[1].1);
        let without_fill = |svg: &str, color: &str| svg.replace(color, "#fill");
        assert_eq!(
            without_fill(&svg_results[0].0, &svg_results[0].1),
            without_fill(&svg_results[1].0, &svg_results[1].1)
        );
    }

    #[test]
    fn generic_straight_locus_svg_keeps_independent_sample_colors_and_pen_styles() {
        let mut doc = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/locus_generic_straights63_chained_colorized_native_saved.gsp"),
            None,
        ).unwrap();
        // The independently verified middle row is Grayscale(Vector(Ray), q).
        let outer = doc.objects.iter().filter(|o| matches!(o.kind, Kind::Locus(..))).nth(1).unwrap().id;
        for object in &mut doc.objects {
            object.visible = object.id == outer;
        }
        doc.selected.insert(outer);
        doc.set_selected_line_width(LineWidth::Thick).unwrap();
        doc.set_selected_line_pattern(LinePattern::Dashed).unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(1000., 800.)),
            scale: 1.,
            center: Point { x: 500., y: 400. },
        };
        let svg = svg_page(&doc, &vp).unwrap();
        // Field3 / explicit query budget gives q = 0, 1/3, 2/3, 1.
        for color in ["#000000", "#555555", "#aaaaaa", "#ffffff"] {
            assert_eq!(svg.matches(&format!("stroke=\"{color}\" stroke-width=\"4\" stroke-dasharray=\"9 6\"")).count(), 1, "{svg}");
        }
        assert!(!svg.contains("<polygon"));
        assert!(!svg.contains("<polyline"));
        assert!(!svg.contains("stroke=\"#00b400\""));
    }

    #[test]
    fn generic_straight_locus_canvas_mesh_preserves_sample_color_and_selection() {
        let doc = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/locus_generic_straights63_chained_colorized_native_saved.gsp"),
            None,
        ).unwrap();
        let outer = doc.objects.iter().filter(|o| matches!(o.kind, Kind::Locus(..))).nth(1).unwrap().id;
        let strokes = doc.locus_styled_strokes(outer, 4).unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(1000., 800.)),
            scale: 1.,
            center: Point { x: 500., y: 400. },
        };
        let mut mesh = Mesh { vertices: Vec::new(), width: 1000., height: 800., ppp: 1. };
        mesh.locus_strokes(&vp, &strokes, None);
        assert_eq!(mesh.vertices.len(), 24);
        for (vertices, gray) in mesh.vertices.chunks_exact(6).zip([0., 85./255., 170./255., 1.]) {
            assert!(vertices.iter().all(|v| v.color == [gray, gray, gray, 1.]));
            assert!(vertices.iter().all(|v| v.pos[1].abs() <= 1.01));
        }
        mesh.vertices.clear();
        let selected = [1., 0., 1., 1.];
        mesh.locus_strokes(&vp, &strokes, Some(selected));
        assert_eq!(mesh.vertices.len(), 24);
        assert!(mesh.vertices.iter().all(|v| v.color == selected));
    }

    #[test]
    fn native_calculation_caption_flag_is_independent_of_geometry() {
        let suppressed = Document::from_bytes(
            include_bytes!(
                "../../../tests/fixtures/numeric_calculation_trailing_decimal_cube84_native_verified.gsp"
            ),
            None,
        )
        .unwrap();
        assert_eq!(suppressed.numeric_value(5), Some(2.));
        assert!(number_lines_for_object(&suppressed, 5).is_none());
        assert!(suppressed.position(14).is_some());

        let visible = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/numeric_calculation_trailing_decimal_cube_formula_visible_native_verified.gsp"),
            None,
        ).unwrap();
        assert_eq!(visible.numeric_value(5), Some(2.));
        let caption = &number_lines_for_object(&visible, 5).unwrap().1[0];
        assert!(caption.ends_with("2.00"), "{caption}");
        assert!(visible.position(14).is_some());
    }

    #[test]
    fn native_infinite_angle_caption_keeps_geometry_undefined() {
        let mut doc = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/numeric_angle_ray_undefined_native_verified.gsp"),
            None,
        )
        .unwrap();
        assert_eq!(doc.numeric_value(6), None);
        assert_eq!(
            doc.parameter_calculation_display_value(6),
            Some(f64::INFINITY)
        );
        assert!(number_lines_for_object(&doc, 6).unwrap().1[0].ends_with("∞°"));
        assert!(doc.line_points(8).is_none());
        doc.set_parameter_value(5, 4.).unwrap();
        assert_eq!(doc.numeric_value(6), Some(90.));
        assert!(number_lines_for_object(&doc, 6).unwrap().1[0].ends_with("90.00°"));
        assert!(doc.line_points(8).is_some());
    }

    #[test]
    fn native_radian_literal_calculations_render_named_and_formula_modes() {
        for (bytes, value, text) in [
            (
                include_bytes!(
                    "../../../tests/fixtures/numeric_scalar_fraction_precision_native_verified.gsp"
                )
                .as_slice(),
                0.075,
                "0.07",
            ),
            (
                include_bytes!(
                    "../../../tests/fixtures/numeric_scalar_true_half_precision_native_verified.gsp"
                )
                .as_slice(),
                0.125,
                "0.13",
            ),
            (
                include_bytes!(
                    "../../../tests/fixtures/numeric_scalar_negative_half_precision_native_verified.gsp"
                )
                .as_slice(),
                -0.125,
                "-0.13",
            ),
        ] {
            let doc = Document::from_bytes(bytes, None).unwrap();
            assert_eq!(doc.numeric_value(17), Some(value));
            let caption = &number_lines_for_object(&doc, 17).unwrap().1[0];
            assert!(caption.ends_with(text), "{caption}");
        }
        let mut named = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/numeric_sourcefree_radian_named_native_verified.gsp"),
            None,
        )
        .unwrap();
        assert_eq!(
            number_lines_for_object(&named, 5).unwrap().1,
            vec!["x轴首刻度 = 15.00°"]
        );
        assert!(number_lines_for_object(&named, 15).unwrap().1[0].contains("x轴首刻度"));
        named.objects[4]
            .label
            .as_mut()
            .unwrap()
            .set_name("θ")
            .unwrap();
        assert_eq!(
            number_lines_for_object(&named, 5).unwrap().1,
            vec!["θ = 15.00°"]
        );
        assert!(number_lines_for_object(&named, 15).unwrap().1[0].contains("θ"));
        let unnamed = Document::from_bytes(
            include_bytes!(
                "../../../tests/fixtures/numeric_sourcefree_radian_unnamed_native_verified.gsp"
            ),
            None,
        )
        .unwrap();
        let caption = &number_lines_for_object(&unnamed, 5).unwrap().1[0];
        assert!(caption.contains("180°"), "{caption}");
        assert!(caption.contains("14.28571°"), "{caption}");
    }

    #[test]
    fn native_generated_parameter_names_render_in_captions_and_formulas() {
        let doc = Document::from_bytes(
            include_bytes!(
                "../../../tests/fixtures/numeric_parameter_autoname_all_unnamed_native_verified.gsp"
            ),
            None,
        )
        .unwrap();
        for (id, prefix) in [(1, "t₄ = "), (2, "t₅ = "), (3, "t₆ = ")] {
            let (_, lines) = number_lines_for_object(&doc, id).unwrap();
            assert!(lines[0].starts_with(prefix), "{:?}", lines);
        }
        let doc = Document::from_bytes(
            include_bytes!(
                "../../../tests/fixtures/numeric_parameter_autoname_counter_seven_native_verified.gsp"
            ),
            None,
        )
        .unwrap();
        let (_, lines) = number_lines_for_object(&doc, 2).unwrap();
        assert!(lines[0].starts_with("t₇ = "));
        let (_, lines) = number_lines_for_object(&doc, 5).unwrap();
        assert!(lines[0].contains("t₇"), "{:?}", lines);
        let doc = Document::from_bytes(
            include_bytes!(
                "../../../tests/fixtures/numeric_parameter_presentation_mode_zero_native_verified.gsp"
            ),
            None,
        )
        .unwrap();
        let (_, lines) = number_lines_for_object(&doc, 2).unwrap();
        assert!(lines[0].starts_with("0.5 cm = "), "{:?}", lines);
        let (_, lines) = number_lines_for_object(&doc, 5).unwrap();
        assert!(lines[0].contains("0.5 cm"), "{:?}", lines);
    }

    #[test]
    fn legacy_polygon_fill_color_edit_and_modern_clipboard_keep_native_opacity() {
        let original = include_bytes!("../../../tests/fixtures/legacy_polygon_rank_original.gsp");
        let blue = include_bytes!("../../../tests/fixtures/legacy_polygon_blue_native_verified.gsp");
        let mut doc = Document::from_bytes(original, None).unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(500., 400.)),
            scale: 1.,
            center: Point { x: 200., y: 150. },
        };
        let alpha = f64::from(49151) / 65535.;
        assert!(
            svg_page(&doc, &vp)
                .unwrap()
                .contains(&format!("fill=\"#ff0000\" fill-opacity=\"{alpha}\""))
        );
        doc.selected.insert(5);
        let color = RgbColor {
            red: 0,
            green: 128,
            blue: 255,
        };
        assert_eq!(doc.set_selected_color(color).unwrap(), 1);
        assert_eq!(doc.encode().unwrap(), blue);
        assert_eq!(doc.objects[4].interior_order, None);
        let header = doc.objects[4].header_offset.unwrap();
        assert_eq!(
            crate::document::records(blue)
                .unwrap()
                .into_iter()
                .find(|r| r.data.start == header)
                .unwrap()
                .data
                .len(),
            16
        );
        assert!(
            svg_page(&doc, &vp)
                .unwrap()
                .contains(&format!("fill=\"#0080ff\" fill-opacity=\"{alpha}\""))
        );
        doc.undo();
        assert_eq!(doc.encode().unwrap(), original);
        doc.redo();
        assert_eq!(doc.encode().unwrap(), blue);
        let clip = doc.copy_selected().unwrap();
        let mut modern = Document::new();
        let pasted = modern.paste(&clip).unwrap();
        let polygon = pasted
            .into_iter()
            .find(|id| modern.objects[id - 1].kind.is_polygon())
            .unwrap();
        assert_eq!(
            modern.objects[polygon - 1].visual_style,
            Some(VisualStyle::Polygon {
                color,
                opacity: 49151
            })
        );
        assert_eq!(modern.objects[polygon - 1].interior_order, Some(0));
        modern.verify_roundtrip().unwrap();
        let saved = modern.encode().unwrap();
        if std::env::var_os("SKETCHPAD_WRITE_LEGACY_STYLE_FIXTURE").is_some() {
            std::fs::write("/tmp/legacy_polygon_modern_copy_writer.gsp", &saved).unwrap();
        }
        assert_eq!(
            saved,
            include_bytes!("../../../tests/fixtures/legacy_polygon_modern_copy_native_verified.gsp")
        );
        let reopened = Document::from_bytes(&saved, None).unwrap();
        assert!(
            svg_page(&reopened, &vp)
                .unwrap()
                .contains(&format!("fill=\"#0080ff\" fill-opacity=\"{alpha}\""))
        );
    }

    #[test]
    fn polygon_fills_follow_signed_native_rank_and_keep_other_draw_slots() {
        let mut doc = Document::new();
        for p in [
            Point { x: 100., y: 100. },
            Point { x: 300., y: 100. },
            Point { x: 200., y: 300. },
        ] {
            doc.add(Kind::Free(p));
        }
        let orange = doc.add(Kind::Polygon(vec![1, 2, 3]));
        let line = doc.add(Kind::Segment(1, 2));
        let green = doc.add(Kind::Polygon(vec![1, 2, 3]));
        let blue = doc.add(Kind::Polygon(vec![1, 2, 3]));
        for (id, rank, color) in [
            (
                orange,
                0,
                RgbColor {
                    red: 255,
                    green: 128,
                    blue: 0,
                },
            ),
            (
                green,
                -1,
                RgbColor {
                    red: 0,
                    green: 255,
                    blue: 0,
                },
            ),
            (
                blue,
                0,
                RgbColor {
                    red: 0,
                    green: 0,
                    blue: 255,
                },
            ),
        ] {
            doc.objects[id - 1].interior_order = Some(rank);
            doc.objects[id - 1].visual_style = Some(VisualStyle::Polygon {
                color,
                opacity: 32767,
            });
        }
        assert_eq!(
            ordered_region_fills(&doc)
                .iter()
                .map(|o| o.id)
                .collect::<Vec<_>>(),
            [green, orange, blue]
        );
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(600., 400.)),
            scale: 1.,
            center: Point { x: 200., y: 200. },
        };
        let svg = svg_page(&doc, &vp).unwrap();
        let green_at = svg.find("fill=\"#00ff00\"").unwrap();
        let orange_at = svg.find("fill=\"#ff8000\"").unwrap();
        let blue_at = svg.find("fill=\"#0000ff\"").unwrap();
        let line_at = svg.find("<line x1=").unwrap();
        assert!(green_at < line_at && line_at < orange_at && orange_at < blue_at);
        assert_eq!(doc.objects[line - 1].kind, Kind::Segment(1, 2));
        doc.objects[green - 1].visible = false;
        assert_eq!(
            ordered_region_fills(&doc)
                .iter()
                .map(|o| o.id)
                .collect::<Vec<_>>(),
            [orange, blue]
        );
        doc.objects[green - 1].visible = true;
        doc.objects[orange - 1].interior_order = Some(i32::MAX);
        doc.objects[green - 1].interior_order = Some(i32::MIN);
        assert_eq!(
            ordered_region_fills(&doc)
                .iter()
                .map(|o| o.id)
                .collect::<Vec<_>>(),
            [green, blue, orange]
        );
    }
    #[test]
    fn styled_point_curve_samples_keep_dash_phase_and_undefined_breaks() {
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(300., 300.)),
            scale: 1.,
            center: Point { x: 0., y: 0. },
        };
        let samples = (0..30)
            .map(|x| {
                (
                    Point { x: x as f64, y: 0. },
                    Point {
                        x: x as f64 + 1.,
                        y: 0.,
                    },
                    LineMode::Segment,
                )
            })
            .collect::<Vec<_>>();
        let paths = curve_screen_paths(&vp, &samples);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].len(), 31);
        let mut mesh = Mesh {
            vertices: Vec::new(),
            width: 300.,
            height: 300.,
            ppp: 1.,
        };
        let color = [0.1, 0.6, 0.8, 1.];
        mesh.styled_path(
            &paths[0],
            false,
            SegmentLineStyle {
                width: LineWidth::Thick,
                pattern: LinePattern::Dashed,
            },
            color,
        );
        assert_eq!(mesh.vertices.len(), 108); // 18 painted 1px samples, 12 gap samples.
        assert!(mesh.vertices.iter().all(|v| v.color == color));
        assert!(!mesh.vertices.iter().any(|v| {
            let x = (v.pos[0] + 1.) * 150.;
            x > 159.1 && x < 164.9
        }));
        let broken = [
            (
                Point { x: 0., y: 0. },
                Point { x: 1., y: 0. },
                LineMode::Segment,
            ),
            (
                Point { x: 1., y: 0. },
                Point { x: 2., y: 0. },
                LineMode::Segment,
            ),
            (
                Point { x: 10., y: 0. },
                Point { x: 11., y: 0. },
                LineMode::Segment,
            ),
        ];
        assert_eq!(
            curve_screen_paths(&vp, &broken)
                .iter()
                .map(Vec::len)
                .collect::<Vec<_>>(),
            vec![3, 2]
        );
    }

    #[test]
    fn carried_circle_export_and_hit_respect_binding_visibility_and_style() {
        let mut doc = Document::from_bytes(
            include_bytes!(
                "../../../tests/fixtures/point_translation_carried_circle_native_verified.gsp"
            ),
            None,
        )
        .unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(500., 400.)),
            scale: 2.,
            center: Point { x: 200., y: 160. },
        };
        let (center, radius) = doc.circle_iteration_images(6).unwrap()[2];
        let at = vp.screen(Point {
            x: center.x,
            y: center.y + radius,
        });
        assert!(circle_iteration_hit(&vp, &doc, 6, at));
        assert!(!circle_iteration_hit(
            &vp,
            &doc,
            6,
            vp.screen(Point {
                x: center.x,
                y: center.y + radius * 3.
            })
        ));
        let svg = svg_page(&doc, &vp).unwrap();
        assert_eq!(
            svg.lines()
                .filter(|line| line.starts_with("<circle") && line.contains("fill=\"none\""))
                .count(),
            3
        );
        let stroke = svg_stroke(doc.objects[5].visual_style);
        assert!(!stroke.is_empty());
        assert_eq!(svg.matches(&format!("<circle{stroke}")).count(), 3);
        doc.selected.insert(6);
        doc.set_selected_visibility(false);
        let svg = svg_page(&doc, &vp).unwrap();
        assert_eq!(
            svg.lines()
                .filter(|line| line.starts_with("<circle") && line.contains("fill=\"none\""))
                .count(),
            0
        );
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.is_displayed(6));
    }

    #[test]
    fn branching_polygon_svg_uses_path_multiplicity_and_carried_bindings() {
        let doc = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/branch_polygon_iteration_native_edited_port.gsp"),
            None,
        )
        .unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(egui::pos2(0., 0.), egui::vec2(500., 400.)),
            scale: 1.,
            center: Point { x: 200., y: 180. },
        };
        let svg = svg_page(&doc, &vp).unwrap();
        assert_eq!(svg.matches("<polygon ").count(), 6);
        assert_eq!(svg.matches("<line").count(), 18);
        assert_eq!(svg.matches("<circle").count(), 3);
        let opacity = repeated_polygon_opacity(32767, 2.);
        assert!((opacity - 0.75).abs() < 0.0001);
        assert!(svg.contains(&format!("fill-opacity=\"{opacity}\"")));
    }

    #[test]
    fn styled_free_text_uses_each_objects_saved_font_in_svg() {
        let vp = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(400., 300.)),
            scale: 1.,
            center: Point { x: 199., y: 125. },
        };
        for (source, first_bold) in [
            (
                include_bytes!("../../../tests/fixtures/text_two_objects_first_bold_off_original.gsp")
                    .as_slice(),
                false,
            ),
            (
                include_bytes!("../../../tests/fixtures/text_two_objects_second_bold_off_original.gsp")
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
            include_bytes!("../../../tests/fixtures/text_two_objects_generated.gsp"),
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
            include_bytes!("../../../tests/fixtures/text_two_point_labels_first_bold_off_original.gsp"),
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
            include_bytes!("../../../tests/fixtures/text_mixed_bold_italic_he_original.gsp"),
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
            include_bytes!("../../../tests/fixtures/text_mixed_size36_he_original.gsp"),
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
            include_bytes!("../../../tests/fixtures/text_mixed_adventor_he_original.gsp"),
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
            include_bytes!("../../../tests/fixtures/graph_dotted_grid_original.gsp"),
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
            include_bytes!("../../../tests/fixtures/graph_two_frames_mixed_grid_generated.gsp"),
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
            include_bytes!("../../../tests/fixtures/graph_snap_points_original.gsp"),
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
            include_bytes!("../../../tests/fixtures/graph_unit_circle_generated.gsp"),
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
            include_bytes!("../../../tests/fixtures/graph_two_segments_generated.gsp"),
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
            include_bytes!("../../../tests/fixtures/graph_polar_grid_original.gsp"),
            None,
        )
        .unwrap();
        let rectangular = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/graph_rectangular_grid_original.gsp"),
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
            include_bytes!("../../../tests/fixtures/graph_trigonometric_axis_original.gsp"),
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
    fn numeric_ratio_shapes_export_and_zero_circle_is_omitted() {
        for (bytes, image, primitive, count) in [
            (
                include_bytes!("../../../tests/fixtures/numeric_ratio_point_port.gsp").as_slice(),
                6,
                "<circle ",
                1,
            ),
            (
                include_bytes!("../../../tests/fixtures/numeric_ratio_segment_port.gsp").as_slice(),
                7,
                "<line ",
                1,
            ),
            (
                include_bytes!("../../../tests/fixtures/numeric_ratio_circle_port.gsp").as_slice(),
                7,
                "<circle ",
                1,
            ),
            (
                include_bytes!("../../../tests/fixtures/numeric_ratio_polygon_port.gsp").as_slice(),
                7,
                "<polygon ",
                1,
            ),
            (
                include_bytes!("../../../tests/fixtures/numeric_ratio_circle_zero_port.gsp").as_slice(),
                7,
                "<circle ",
                0,
            ),
        ] {
            let mut doc = Document::from_bytes(bytes, None).unwrap();
            for object in &mut doc.objects {
                object.visible = object.id == image;
            }
            let vp = Viewport {
                rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(400., 300.)),
                scale: 1.,
                center: Point { x: 200., y: 150. },
            };
            assert_eq!(
                svg_page(&doc, &vp).unwrap().matches(primitive).count(),
                count
            );
        }
    }

    #[test]
    fn numeric_angle_shapes_export_with_saved_colors() {
        for (bytes, primitive) in [
            (
                include_bytes!("../../../tests/fixtures/numeric_angle_segment_edited_port.gsp")
                    .as_slice(),
                "<line ",
            ),
            (
                include_bytes!("../../../tests/fixtures/numeric_angle_circle_edited_port.gsp").as_slice(),
                "<circle ",
            ),
            (
                include_bytes!("../../../tests/fixtures/numeric_angle_polygon_edited_port.gsp")
                    .as_slice(),
                "<polygon ",
            ),
        ] {
            let mut doc = Document::from_bytes(bytes, None).unwrap();
            for object in &mut doc.objects {
                object.visible = object.id == 10;
            }
            let vp = Viewport {
                rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(400., 300.)),
                scale: 1.,
                center: Point { x: 200., y: 150. },
            };
            let svg = svg_page(&doc, &vp).unwrap();
            assert_eq!(svg.matches(primitive).count(), 1);
            assert!(svg.contains("#2850c8"));
        }
    }

    #[test]
    fn svg_uses_saved_point_stroke_and_polygon_styles() {
        let doc = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/style_integrated_saved_by_wine.gsp"),
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
        assert!(svg.contains(&format!(
            "fill=\"#800080\" fill-opacity=\"{}\" stroke=\"none\"",
            f64::from(32767) / 65535.
        )));
        assert!(svg.contains("r=\"6\" fill=\"#0000ff\""));
    }
    #[test]
    fn referenced_captions_export_current_source_values_and_names() {
        let mut doc = Document::from_bytes(include_bytes!("../../../tests/fixtures/referenced_rich51_api_baseline_native_saved.gsp"), None).unwrap();
        let vp = Viewport { rect: Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.,600.)), scale: 1., center: Point{x:400.,y:300.} };
        let before=svg_page(&doc,&vp).unwrap();
        assert!(before.contains("POINT Alpha"));
        assert!(before.contains("VALUE 2.0"));
        doc.set_parameter_value(2,4.).unwrap();
        doc.set_point_label(1,"Beta",false).unwrap();
        let after=svg_page(&doc,&vp).unwrap();
        assert!(after.contains("POINT Beta"));
        assert!(after.contains("VALUE 4.0"));
        assert!(!after.contains("CACHE"));
        doc.verify_roundtrip().unwrap();
    }
    #[test]
    fn live_color_wrappers_update_svg_geometry_and_color_after_reopen() {
        use crate::document::colorized::{ColorBinding, ColorRangeBehavior};
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 100., y: 100. }));
        let b = doc.add(Kind::Free(Point { x: 200., y: 100. }));
        let c = doc.add(Kind::Free(Point { x: 100., y: 200. }));
        let segment = doc.add(Kind::Segment(a, b));
        let circle = doc.add(Kind::Circle(a, b));
        let polygon = doc.add(Kind::Polygon(vec![a, b, c]));
        let arc = doc.add(Kind::CenterArc(a, b, c));
        let value = doc.add_named_parameter("q", 0.5, (10, 10)).unwrap();
        for source in [a, segment, circle, polygon, arc] {
            doc.add_colorized(source, ColorBinding::Grayscale { value }, 0., 1., ColorRangeBehavior::Clamp).unwrap();
            doc.objects[source - 1].visible = false;
        }
        let vp = Viewport {
            rect: Rect::from_min_size(egui::pos2(0., 0.), egui::vec2(800., 600.)),
            scale: 1., center: Point { x: 200., y: 200. },
        };
        let before = svg_page(&doc, &vp).unwrap();
        assert!(before.matches("#7f7f7f").count() >= 5);
        doc.set_parameter_value(value, 0.75).unwrap();
        assert!(doc.move_point(b, Point { x: 250., y: 100. }));
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        let after = svg_page(&reopened, &vp).unwrap();
        assert!(after.matches("#bfbfbf").count() >= 5);
        assert!(!after.contains("#7f7f7f"));
        assert_ne!(before, after);
    }
    #[test]
    fn picture_hit_and_svg_use_the_same_affine_viewport() {
        let mut doc = Document::new();
        let png = include_bytes!("../../../tests/fixtures/image_tiny.png");
        doc.import_png(png, Point { x: 120., y: 130. }).unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(egui::pos2(10., 20.), egui::vec2(300., 200.)),
            scale: 2.,
            center: Point { x: 150., y: 150. },
        };
        let Kind::Image { png, transform, .. } = &doc.objects[0].kind else {
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
    fn imported_point_anchored_picture_moves_in_svg_and_picking() {
        let original = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/anchored_image94_coin_original_input.gsp"),
            None,
        )
        .unwrap();
        let moved = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/anchored_image94_coin_move_input.gsp"),
            None,
        )
        .unwrap();
        let vp = Viewport {
            rect: Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000., 800.)),
            scale: 1.,
            center: Point { x: 500., y: 400. },
        };
        let Kind::Image { png, .. } = &original.objects[28].kind else {
            panic!("native picture is not decoded")
        };
        let original_quad = picture_screen_quad(&vp, png, original.image_transform(29).unwrap()).unwrap();
        let moved_quad = picture_screen_quad(&vp, png, moved.image_transform(29).unwrap()).unwrap();
        for (before, after) in original_quad.into_iter().zip(moved_quad) {
            assert_eq!(after, before + egui::vec2(40., 20.));
        }
        assert!(picture_hit(&vp, png, moved.image_transform(29).unwrap(), moved_quad[0] + egui::vec2(10., 10.)));
        let Kind::Image { transform, .. } = &original.objects[28].kind else {
            unreachable!()
        };
        let isolated = |point: Point| {
            let mut document = Document::new();
            let anchor = document.add(Kind::Free(point));
            document.add(Kind::Image {
                png: png.clone(),
                transform: *transform,
                anchor: Some(anchor),
            });
            document
        };
        let before = svg_page(&isolated(Point { x: 686., y: 341. }), &vp).unwrap();
        let after = svg_page(&isolated(Point { x: 726., y: 361. }), &vp).unwrap();
        assert_eq!(before.matches("<image ").count(), after.matches("<image ").count());
        assert_ne!(before, after);
    }
    #[test]
    fn imported_multi_resource_pictures_export_their_own_pngs() {
        let viewport = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(1000., 800.)),
            scale: 1.,
            center: Point { x: 500., y: 400. },
        };
        for (bytes, first, second) in [
            (include_bytes!("../../../tests/fixtures/image_objects91_original_native_saved.gsp").as_slice(), true, true),
            (include_bytes!("../../../tests/fixtures/image_objects91_hide20_native_saved.gsp").as_slice(), false, true),
            (include_bytes!("../../../tests/fixtures/image_objects91_hide21_native_saved.gsp").as_slice(), true, false),
            (include_bytes!("../../../tests/fixtures/image_objects91_hide_both_native_saved.gsp").as_slice(), false, false),
        ] {
            let mut doc = Document::from_bytes(bytes, None).unwrap();
            doc.partial_view = false;
            let svg = svg_page(&doc, &viewport).unwrap();
            assert_eq!(svg.matches("<image ").count(), usize::from(first) + usize::from(second));
            assert_eq!(svg.contains("<image width=\"253\" height=\"35\""), first);
            assert_eq!(svg.contains("<image width=\"321\" height=\"35\""), second);
        }
    }
    #[test]
    fn imported_free_text_rgb_runs_export_live_run_colors() {
        let viewport = Viewport {
            rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(1000., 800.)),
            scale: 1.,
            center: Point { x: 500., y: 400. },
        };
        for (bytes, color) in [
            (include_bytes!("../../../tests/fixtures/rich_style92_original_native_saved.gsp").as_slice(), "#0000ff"),
            (include_bytes!("../../../tests/fixtures/rich_style92_g2_native_saved.gsp").as_slice(), "#0001ff"),
            (include_bytes!("../../../tests/fixtures/rich_style92_l050_native_saved.gsp").as_slice(), "#00004f"),
        ] {
            let mut doc = Document::from_bytes(bytes, None).unwrap();
            for object in &mut doc.objects {
                object.visible = object.id == 14;
            }
            doc.partial_view = false;
            let svg = svg_page(&doc, &viewport).unwrap();
            assert!(svg.contains(&format!("fill=\"{color}\"")), "missing {color}");
        }
    }
    #[test]
    fn collapsed_iteration_circles_are_omitted_from_svg() {
        let doc = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/vector_collapsed_circle_iteration_port.gsp"),
            None,
        )
        .unwrap();
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(480., 360.)),
            scale: 1.,
            center: Point { x: 200., y: 180. },
        };
        let svg = svg_page(&doc, &viewport).unwrap();
        assert!(!svg.contains("r=\"0\""));
        assert!(svg.contains("r=\"40\""));
    }

    #[test]
    fn scientific_captions_reach_svg_without_changing_numeric_values() {
        let mut doc = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/scientific_rounding_native_verified.gsp"),
            None,
        )
        .unwrap();
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(480., 360.)),
            scale: 1.,
            center: Point { x: 200., y: 180. },
        };
        let svg = svg_page(&doc, &viewport).unwrap();
        assert!(svg.contains("1.00 · 10⁷"));
        assert!(svg.contains("-2.86 · 10¹⁴"));
        assert_eq!(doc.numeric_value(1), Some(9999999.));
        doc.set_numeric_precision(&[1], 5).unwrap();
        assert!(svg_page(&doc, &viewport).unwrap().contains("1.00000 · 10⁷"));
        assert_eq!(doc.numeric_value(1), Some(9999999.));
    }

    #[test]
    fn calculation_precision_survives_svg_export() {
        let mut doc = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/calculation_precision_native_verified.gsp"),
            None,
        )
        .unwrap();
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(480., 360.)),
            scale: 1.,
            center: Point { x: 200., y: 180. },
        };
        let svg = svg_page(&doc, &viewport).unwrap();
        for caption in ["5.292 cm", "2.0000", "0.24690", "5</text>"] {
            assert!(svg.contains(caption), "missing {caption}");
        }
        doc.set_numeric_precision(&[8, 9, 11, 12], 5).unwrap();
        assert!(svg_page(&doc, &viewport).unwrap().contains("5.12500"));
        doc.set_numeric_precision(&[11, 12], 0).unwrap();
        let svg = svg_page(&doc, &viewport).unwrap();
        assert!(!svg.contains("0.24690"));
        assert!(!svg.contains("5.12500"));
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
                .contains("显示线段</text>")
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
