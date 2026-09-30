#[path = "math_text.rs"]
pub(crate) mod math_text;
#[path = "editor/system_fonts.rs"]
pub(crate) mod system_fonts;

use crate::document::{Document, Kind, LineMode, Point};
use crate::graph::GridForm;
use crate::style_records::{
    LinePattern, LineWidth, PointStyle, RgbColor, SegmentLineStyle, VisualStyle,
};
use crate::text_records::display_label_name;
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
        Kind::DataTable(_, group) => {
            let rows = group.general_cells()?;
            let columns = rows.first()?.len();
            let widths = (0..columns)
                .map(|column| {
                    rows.iter()
                        .filter_map(|row| row.get(column))
                        .map(|cell| cell.chars().count())
                        .max()
                        .unwrap_or(0)
                })
                .collect::<Vec<_>>();
            let lines = rows
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .enumerate()
                        .map(|(column, cell)| {
                            if column + 1 == columns {
                                cell
                            } else {
                                format!("{cell:>width$}", width = widths[column])
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("  ")
                })
                .collect();
            Some((group.anchor(), lines))
        }
        _ => None,
    }
}

pub fn number_lines_for_object(doc: &Document, id: usize) -> Option<((i16, i16), Vec<String>)> {
    let object = doc.objects.get(id.checked_sub(1)?)?;
    if matches!(object.kind, Kind::IterationValues(..)) {
        let rows = doc.iteration_value_table(id)?;
        let columns = rows.first()?.len();
        let widths = (0..columns)
            .map(|column| {
                rows.iter()
                    .filter_map(|row| row.get(column))
                    .map(|cell| display_parameter_indices(cell).chars().count())
                    .max()
                    .unwrap_or(0)
            })
            .collect::<Vec<_>>();
        let lines = rows
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .enumerate()
                    .map(|(column, cell)| {
                        let cell = display_parameter_indices(&cell);
                        if column + 1 == columns {
                            cell
                        } else {
                            format!("{cell:>width$}", width = widths[column])
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("  ")
            })
            .collect();
        return Some((object.measurement_anchor?, lines));
    }
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
    let labels = doc.visible_mapped_numeric_label_images();
    mapped_numeric_labels_hit(vp, doc, id, labels.get(&id).map(Vec::as_slice), at)
}

pub fn mapped_numeric_labels_hit(
    vp: &Viewport,
    doc: &Document,
    id: usize,
    labels: Option<&[(Point, String)]>,
    at: Pos2,
) -> bool {
    let size = doc
        .numeric_label_font_state(id)
        .map_or(16., |style| f32::from(style.size_code));
    labels.is_some_and(|labels| {
        labels.iter().any(|(point, text)| {
            Rect::from_center_size(
                vp.screen(*point),
                egui::vec2(text.chars().count() as f32 * size * 0.65, size * 1.1),
            )
            .contains(at)
        })
    })
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

pub fn ordered_picture_ids(doc: &Document) -> Vec<usize> {
    let mut pictures = doc
        .objects
        .iter()
        .filter(|object| {
            doc.is_displayed(object.id)
                && (matches!(object.kind, Kind::SampledImage(_))
                    || doc.cropped_image(object.id).is_some()
                    || doc.image_instances(object.id).is_some())
        })
        .map(|object| object.id)
        .collect::<Vec<_>>();
    pictures.sort_by(|first, second| {
        match (doc.picture_order(*first), doc.picture_order(*second)) {
            (Some(first_order), Some(second_order)) => first_order
                .cmp(&second_order)
                .then_with(|| first.cmp(second)),
            _ => first.cmp(second),
        }
    });
    pictures
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
        return disks.into_iter().any(|(center, radius, _)| {
            vp.screen(center).distance(at) <= (radius as f32 * vp.scale).abs()
        });
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
    if doc
        .mapped_collection_surfaces(id, 125)
        .is_some_and(|surfaces| {
            surfaces.into_iter().any(|surface| {
                let points = surface
                    .points
                    .into_iter()
                    .map(|point| vp.screen(point))
                    .collect::<Vec<_>>();
                point_in_polygon(&points, at)
            })
        })
    {
        return true;
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
    if doc
        .mapped_collection_strokes(id, 384)
        .is_some_and(|strokes| {
            strokes.into_iter().any(|stroke| {
                clipped_line(
                    vp.screen(stroke.start),
                    vp.screen(stroke.end),
                    stroke.mode,
                    vp.rect,
                )
                .is_some_and(|(a, b)| near(a, b))
            })
        })
    {
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

pub fn point_in_polygon(points: &[Pos2], at: Pos2) -> bool {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .fold(false, |inside, (a, b)| {
            inside
                ^ ((a.y > at.y) != (b.y > at.y)
                    && at.x < (b.x - a.x) * (at.y - a.y) / (b.y - a.y) + a.x)
        })
}

/// Clip a live crop polygon to the source bitmap and return screen/UV vertices.
pub fn cropped_picture_vertices(
    vp: &Viewport,
    png: &[u8],
    transform: [f64; 6],
    polygon: &[Point],
) -> Option<Vec<(Pos2, Pos2)>> {
    let (width, height) = crate::image_records::ImageFile::png_size(png).ok()?;
    let width = f64::from(width);
    let height = f64::from(height);
    let det = transform[0] * transform[4] - transform[1] * transform[3];
    if det.abs() < 1e-12 {
        return None;
    }
    let mut pixels = polygon
        .iter()
        .map(|point| {
            let x = point.x - transform[2];
            let y = point.y - transform[5];
            (
                (transform[4] * x - transform[1] * y) / det,
                (-transform[3] * x + transform[0] * y) / det,
            )
        })
        .collect::<Vec<_>>();
    for (axis, bound, keep_greater) in [
        (0, 0., true),
        (0, width, false),
        (1, 0., true),
        (1, height, false),
    ] {
        if pixels.is_empty() {
            return None;
        }
        let inside = |point: (f64, f64)| {
            let value = if axis == 0 { point.0 } else { point.1 };
            if keep_greater {
                value >= bound
            } else {
                value <= bound
            }
        };
        let intersect = |a: (f64, f64), b: (f64, f64)| {
            let (av, bv) = if axis == 0 { (a.0, b.0) } else { (a.1, b.1) };
            let t = (bound - av) / (bv - av);
            (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
        };
        let mut clipped = Vec::new();
        let mut previous = *pixels.last()?;
        for current in pixels.iter().copied() {
            match (inside(previous), inside(current)) {
                (true, true) => clipped.push(current),
                (true, false) => clipped.push(intersect(previous, current)),
                (false, true) => {
                    clipped.push(intersect(previous, current));
                    clipped.push(current);
                }
                (false, false) => {}
            }
            previous = current;
        }
        pixels = clipped;
    }
    let vertices = pixels
        .into_iter()
        .map(|(u, v)| {
            let world = Point {
                x: transform[0] * u + transform[1] * v + transform[2],
                y: transform[3] * u + transform[4] * v + transform[5],
            };
            (
                vp.screen(world),
                egui::pos2((u / width) as f32, (v / height) as f32),
            )
        })
        .collect::<Vec<_>>();
    (vertices.len() >= 3).then_some(vertices)
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
    for (axis, &(lo, hi)) in bounds.iter().enumerate() {
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
            if let (Some(a), Some(b)) = pair
                && let Some(line) = clipped_line(
                    vp.screen(Point { x: a.x, y: a.y }),
                    vp.screen(Point { x: b.x, y: b.y }),
                    LineMode::Line,
                    vp.rect,
                )
            {
                lines.push(line);
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

type AxisTick = (Pos2, Pos2, bool);
type AxisLabel = (Pos2, String, bool);

fn defined_axis_annotations(doc: &Document, vp: &Viewport) -> (Vec<AxisTick>, Vec<AxisLabel>) {
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
    svg_page_with_layout(doc, vp, button_offset, vp.rect, None)
}

pub fn svg_page_on_paper(
    doc: &Document,
    vp: &Viewport,
    button_offset: egui::Vec2,
    paper_width_mm: f32,
    paper_height_mm: f32,
) -> Result<String, String> {
    const PX_PER_MM: f32 = 96. / 25.4;
    let paper = Rect::from_min_size(
        Pos2::ZERO,
        egui::vec2(paper_width_mm * PX_PER_MM, paper_height_mm * PX_PER_MM),
    );
    svg_page_with_layout(
        doc,
        vp,
        button_offset,
        paper,
        Some((paper_width_mm, paper_height_mm)),
    )
}

#[derive(Clone, Copy)]
enum SvgInterior {
    Picture(usize),
    Region(usize),
}

fn ordered_svg_interiors(doc: &Document) -> Vec<SvgInterior> {
    let mut interiors = ordered_picture_ids(doc)
        .into_iter()
        .map(SvgInterior::Picture)
        .chain(
            doc.objects
                .iter()
                .filter(|object| doc.is_displayed(object.id) && object.kind.is_filled_region())
                .map(|object| SvgInterior::Region(object.id)),
        )
        .collect::<Vec<_>>();
    let rank = |interior: SvgInterior| match interior {
        SvgInterior::Picture(id) => doc.picture_order(id),
        SvgInterior::Region(id) => doc.objects[id - 1].interior_order,
    };
    let fallback = |interior: SvgInterior| match interior {
        SvgInterior::Picture(id) => (0, id),
        SvgInterior::Region(id) => (1, id),
    };
    interiors.sort_by(|first, second| match (rank(*first), rank(*second)) {
        (Some(first_rank), Some(second_rank)) => first_rank
            .cmp(&second_rank)
            .then_with(|| fallback(*first).cmp(&fallback(*second))),
        _ => fallback(*first).cmp(&fallback(*second)),
    });
    interiors
}

fn write_svg_picture(
    svg: &mut String,
    doc: &Document,
    vp: &Viewport,
    id: usize,
) -> Result<(), String> {
    if let Some(mesh) = doc.sampled_image_mesh(id, 900) {
        let (width, height) = crate::image_records::ImageFile::png_size(mesh.png)?;
        writeln!(
            svg,
            "<defs><image id=\"sampled-picture-{id}\" width=\"{width}\" height=\"{height}\" href=\"data:image/png;base64,{}\"/></defs>",
            base64(mesh.png)
        )
        .unwrap();
        for (index, triangle) in mesh.triangles.into_iter().enumerate() {
            let [(first, first_uv), (second, second_uv), (third, third_uv)] =
                triangle.map(|vertex| mesh.vertices[vertex as usize]);
            let source = [
                (
                    f64::from(first_uv[0]) * f64::from(width),
                    f64::from(first_uv[1]) * f64::from(height),
                ),
                (
                    f64::from(second_uv[0]) * f64::from(width),
                    f64::from(second_uv[1]) * f64::from(height),
                ),
                (
                    f64::from(third_uv[0]) * f64::from(width),
                    f64::from(third_uv[1]) * f64::from(height),
                ),
            ];
            let destination = [vp.screen(first), vp.screen(second), vp.screen(third)];
            let source_u = (source[1].0 - source[0].0, source[1].1 - source[0].1);
            let source_v = (source[2].0 - source[0].0, source[2].1 - source[0].1);
            let determinant = source_u.0 * source_v.1 - source_u.1 * source_v.0;
            if determinant.abs() <= f64::EPSILON {
                continue;
            }
            let destination_u = destination[1] - destination[0];
            let destination_v = destination[2] - destination[0];
            let a = (f64::from(destination_u.x) * source_v.1
                - f64::from(destination_v.x) * source_u.1)
                / determinant;
            let c = (-f64::from(destination_u.x) * source_v.0
                + f64::from(destination_v.x) * source_u.0)
                / determinant;
            let b = (f64::from(destination_u.y) * source_v.1
                - f64::from(destination_v.y) * source_u.1)
                / determinant;
            let d = (-f64::from(destination_u.y) * source_v.0
                + f64::from(destination_v.y) * source_u.0)
                / determinant;
            let e = f64::from(destination[0].x) - a * source[0].0 - c * source[0].1;
            let f = f64::from(destination[0].y) - b * source[0].0 - d * source[0].1;
            let points = destination
                .map(|point| format!("{},{}", point.x, point.y))
                .join(" ");
            writeln!(
                svg,
                "<clipPath id=\"sampled-picture-{id}-{index}\" clipPathUnits=\"userSpaceOnUse\"><polygon points=\"{points}\"/></clipPath><g clip-path=\"url(#sampled-picture-{id}-{index})\"><use href=\"#sampled-picture-{id}\" transform=\"matrix({a} {b} {c} {d} {e} {f})\"/></g>"
            )
            .unwrap();
        }
    } else if let Some((_, png, transform, polygon)) = doc.cropped_image(id) {
        let (width, height) = crate::image_records::ImageFile::png_size(png)?;
        let origin = vp.screen(Point {
            x: transform[2],
            y: transform[5],
        });
        let points = polygon
            .into_iter()
            .map(|point| {
                let point = vp.screen(point);
                format!("{},{}", point.x, point.y)
            })
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(
            svg,
            "<clipPath id=\"picture-crop-{id}\" clipPathUnits=\"userSpaceOnUse\"><polygon points=\"{points}\"/></clipPath>"
        )
        .unwrap();
        writeln!(svg,
            "<g clip-path=\"url(#picture-crop-{id})\"><image width=\"{width}\" height=\"{height}\" transform=\"matrix({} {} {} {} {} {})\" href=\"data:image/png;base64,{}\"/></g>",
            transform[0] * vp.scale as f64, transform[3] * vp.scale as f64,
            transform[1] * vp.scale as f64, transform[4] * vp.scale as f64,
            origin.x, origin.y, base64(png)).unwrap();
    } else if let Some((_, png, transforms)) = doc.image_instances(id) {
        let (width, height) = crate::image_records::ImageFile::png_size(png)?;
        for transform in transforms {
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
    Ok(())
}

fn write_svg_region(svg: &mut String, doc: &Document, vp: &Viewport, id: usize) {
    let Some(points) = doc.filled_region_points(id) else {
        return;
    };
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
        match doc.effective_visual_style(id) {
            Some(VisualStyle::Polygon { color, .. }) => css_color(color),
            _ => "#f6ad60".into(),
        },
        match doc.effective_visual_style(id) {
            Some(VisualStyle::Polygon { opacity, .. }) => f64::from(opacity) / 65535.,
            _ => 0.5,
        }
    )
    .unwrap();
}

fn svg_page_with_layout(
    doc: &Document,
    vp: &Viewport,
    button_offset: egui::Vec2,
    page: Rect,
    physical_size_mm: Option<(f32, f32)>,
) -> Result<String, String> {
    if doc.partial_view
        && doc.objects.iter().any(|object| {
            object.page == doc.active_page
                && !object.deleted
                && matches!(object.kind, Kind::Unsupported(_))
        })
    {
        return Err("This document contains objects that cannot be exported yet".into());
    }
    let r = vp.rect;
    if !r.is_finite()
        || r.width() <= 0.
        || r.height() <= 0.
        || !page.is_finite()
        || page.width() <= 0.
        || page.height() <= 0.
    {
        return Err("No visible sketch viewport".into());
    }
    let (width, height) = physical_size_mm.map_or_else(
        || (r.width().to_string(), r.height().to_string()),
        |(width, height)| (format!("{width}mm"), format!("{height}mm")),
    );
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"{} {} {} {}\">\n<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"white\"/>\n<defs><clipPath id=\"sketch-clip\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/></clipPath></defs>\n<g clip-path=\"url(#sketch-clip)\">\n",
        width,
        height,
        page.min.x,
        page.min.y,
        page.width(),
        page.height(),
        page.min.x,
        page.min.y,
        page.width(),
        page.height(),
        r.min.x,
        r.min.y,
        r.width(),
        r.height(),
    );
    for interior in ordered_svg_interiors(doc) {
        match interior {
            SvgInterior::Picture(id) => write_svg_picture(&mut svg, doc, vp, id)?,
            SvgInterior::Region(id) => write_svg_region(&mut svg, doc, vp, id),
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
                svg_stroke(doc.effective_visual_style(o.id))
            )
            .unwrap();
        }
        if let Some(strokes) = doc
            .locus_styled_strokes(o.id, 384)
            .or_else(|| doc.mapped_collection_strokes(o.id, 384))
            .or_else(|| doc.sampled_mapped_styled_strokes(o.id, 384))
        {
            for sample in strokes {
                if let Some((a, b)) = clipped_line(
                    vp.screen(sample.start),
                    vp.screen(sample.end),
                    sample.mode,
                    r,
                ) {
                    writeln!(
                        svg,
                        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"{}/>",
                        a.x,
                        a.y,
                        b.x,
                        b.y,
                        svg_stroke(Some(sample.style))
                    )
                    .unwrap();
                }
            }
        } else if let Some(surfaces) = doc
            .mapped_collection_surfaces(o.id, 384)
            .or_else(|| doc.locus_surfaces(o.id, 384))
        {
            for surface in surfaces {
                let vertices = surface
                    .points
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
                    match surface.style {
                        VisualStyle::Polygon { color, .. } => css_color(color),
                        _ => unreachable!(),
                    },
                    match surface.style {
                        VisualStyle::Polygon { opacity, .. } => f64::from(opacity) / 65535.,
                        _ => unreachable!(),
                    }
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
            if matches!(
                doc.effective_visual_style(o.id),
                Some(VisualStyle::Stroke { .. })
            ) {
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
        if let Some(marker) = doc.angle_marker_geometry(o.id) {
            let style = doc.effective_visual_style(o.id);
            let color = match style {
                Some(VisualStyle::Stroke { color, .. }) => css_color(color),
                _ => "#404040".into(),
            };
            let vertices = |points: &[Point]| {
                points
                    .iter()
                    .map(|point| {
                        let at = vp.screen(*point);
                        format!("{},{}", at.x, at.y)
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            writeln!(
                svg,
                "<polygon points=\"{}\" fill=\"{}\" fill-opacity=\"{}\" stroke=\"none\"/>",
                vertices(&marker.sector),
                color,
                f64::from(o.angle_marker_style.unwrap().opacity) / 65535.
            )
            .unwrap();
            for path in marker.strokes {
                writeln!(
                    svg,
                    "<polyline points=\"{}\" fill=\"none\"{}/>",
                    vertices(&path),
                    svg_stroke(style)
                )
                .unwrap();
            }
            for arrow in marker.arrows {
                writeln!(
                    svg,
                    "<polygon points=\"{}\" fill=\"{}\" stroke=\"none\"/>",
                    vertices(&arrow),
                    color
                )
                .unwrap();
            }
        }
        if let Some(strokes) = doc.segment_marker_strokes(o.id) {
            for [start, end] in strokes {
                let start = vp.screen(start);
                let end = vp.screen(end);
                writeln!(
                    svg,
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"{}/>",
                    start.x,
                    start.y,
                    end.x,
                    end.y,
                    svg_stroke(doc.effective_visual_style(o.id))
                )
                .unwrap();
            }
        }
        if let Some(polygons) = doc.colorized_polygon_iteration_images(o.id) {
            let opacity = match doc.effective_visual_style(o.id) {
                Some(VisualStyle::Polygon { opacity, .. }) => opacity,
                _ => 32767,
            };
            for (points, color) in polygons {
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
                    css_color(color),
                    f64::from(opacity) / 65535.
                )
                .unwrap();
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
    let mut math_fonts = None;
    let mapped_numeric_labels = doc.visible_mapped_numeric_label_images();
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
            let math = doc.text_math_nodes(object.id).and_then(|_| {
                let ctx =
                    math_fonts.get_or_insert_with(|| crate::render::math_text::font_context(doc));
                crate::render::math_text::measured_layout(doc, object.id, ctx)
            });
            if let Some(layout) = math {
                layout.write_svg(&mut svg, at);
            } else if has_mixed_runs {
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
                        let run_color = run
                            .style
                            .rgb_color()
                            .map(|[red, green, blue]| {
                                format!(" fill=\"#{red:02x}{green:02x}{blue:02x}\"")
                            })
                            .unwrap_or_default();
                        let (run_size, baseline_shift) = run
                            .style
                            .script_metrics(f32::from(run.style.size_code.unwrap_or(size)));
                        let script = if baseline_shift == 0. {
                            String::new()
                        } else {
                            format!(" baseline-shift=\"{baseline_shift}px\"")
                        };
                        write!(
                            svg,
                            "<tspan{}{}{} font-size=\"{}\" font-weight=\"{}\" font-style=\"{}\" text-decoration=\"{}\">{}</tspan>",
                            run_family,
                            run_color,
                            script,
                            run_size,
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
        if let Some(labels) = doc.point_math_labels(object.id) {
            let ctx = math_fonts.get_or_insert_with(|| math_text::font_context(doc));
            for (point, nodes) in labels {
                let layout = math_text::measured_nodes(doc, object.id, &nodes, ctx);
                let at = vp.screen(point) - egui::vec2(layout.width, layout.height()) * 0.5;
                layout.write_svg(&mut svg, at);
            }
        } else {
            for (point, text) in doc
                .point_numeric_label(object.id)
                .map(|label| vec![label])
                .or_else(|| mapped_numeric_labels.get(&object.id).cloned())
                .unwrap_or_default()
            {
                let at = vp.screen(point);
                let style = doc.numeric_label_font_state(object.id);
                let size = style.as_ref().map_or(16, |style| style.size_code);
                let family = style
                    .as_ref()
                    .map_or("sans-serif", |style| style.family.as_str())
                    .replace('&', "&amp;")
                    .replace('"', "&quot;")
                    .replace('<', "&lt;");
                writeln!(svg,"<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"central\" font-family=\"{}\" font-size=\"{}\" font-weight=\"{}\" font-style=\"{}\" text-decoration=\"{}\">{}</text>",
                at.x,at.y,family,size,
                if style.as_ref().is_some_and(|style| style.bold) { "bold" } else { "normal" },
                if style.as_ref().is_some_and(|style| style.italic) { "italic" } else { "normal" },
                if style.as_ref().is_some_and(|style| style.underline) { "underline" } else { "none" },
                xml_text(&text)).unwrap();
            }
        }
        if let Some((anchor, lines)) = number_lines_for_object(doc, object.id) {
            let at = vp.rect.min + egui::vec2(anchor.0 as f32, anchor.1 as f32);
            for (line, content) in lines.iter().enumerate() {
                writeln!(
                    svg,
                    "<text x=\"{}\" y=\"{}\" xml:space=\"preserve\">{}</text>",
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
            && let Some(point) = doc.object_label_anchor(object.id)
        {
            let at = vp.screen(point);
            let offset = label.offset();
            let name = display_label_name(name);
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
                "<text x=\"{}\" y=\"{}\" text-anchor=\"{}\" font-family=\"{}\" font-size=\"{}\" font-weight=\"{}\" font-style=\"{}\" text-decoration=\"{}\">{}</text>",
                at.x + offset.0 as f32,
                at.y + offset.1 as f32 + size as f32 * 0.85,
                if object.angle_marker_style.is_some() { "middle" } else { "start" },
                family,
                size,
                if style.as_ref().is_some_and(|style| style.bold) { "bold" } else { "normal" },
                if style.as_ref().is_some_and(|style| style.italic) { "italic" } else { "normal" },
                if style.as_ref().is_some_and(|style| style.underline) { "underline" } else { "none" },
                xml_text(name.as_ref())
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
            &object.kind
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
        if let Kind::DerivativeFunction(_, expression) = &object.kind {
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
        | Kind::SoundButton { label, anchor, .. }
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
    svg.push_str("</g>\n</svg>\n");
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
                self.styled_path(
                    &[a, b],
                    false,
                    line,
                    selection_color.unwrap_or_else(|| rgba(color, 1.)),
                );
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

#[derive(Clone)]
pub struct TraceSample {
    pub mark: TraceMark,
    pub created_at: std::time::Instant,
}

impl TraceSample {
    pub fn new(mark: TraceMark) -> Self {
        Self {
            mark,
            created_at: std::time::Instant::now(),
        }
    }
}

pub struct GeometryRenderer {
    pipeline: wgpu::RenderPipeline,
}

pub struct RenderTarget<'a> {
    pub device: &'a wgpu::Device,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub view: &'a wgpu::TextureView,
    pub size: [u32; 2],
    pub pixels_per_point: f32,
}

pub struct RenderPreferences {
    pub background_color: [u8; 3],
    pub selection_color: [u8; 3],
    pub trace_fade_seconds: Option<f32>,
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
        target: RenderTarget<'_>,
        doc: &Document,
        vp: &Viewport,
        traces: &[TraceSample],
        preferences: RenderPreferences,
    ) {
        let RenderTarget {
            device,
            encoder,
            view,
            size,
            pixels_per_point: ppp,
        } = target;
        let mut mesh = Mesh {
            vertices: Vec::new(),
            width: size[0] as f32,
            height: size[1] as f32,
            ppp,
        };
        let dark = [0.14, 0.18, 0.24, 1.];
        let blue = [0.09, 0.42, 0.84, 1.];
        let selected = [
            f32::from(preferences.selection_color[0]) / 255.,
            f32::from(preferences.selection_color[1]) / 255.,
            f32::from(preferences.selection_color[2]) / 255.,
            1.,
        ];
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
        let selected_grid = selected;
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
        let trace_now = std::time::Instant::now();
        for sample in traces {
            let alpha = preferences.trace_fade_seconds.map_or(1., |seconds| {
                (1. - trace_now.duration_since(sample.created_at).as_secs_f32() / seconds)
                    .clamp(0., 1.)
            });
            if alpha <= 0. {
                continue;
            }
            match &sample.mark {
                TraceMark::Point(point, color) => {
                    mesh.disk(vp.screen(*point), 2., rgba(*color, alpha));
                }
                TraceMark::Straight(a, b, mode, color) => {
                    if let Some((a, b)) = clipped_line(vp.screen(*a), vp.screen(*b), *mode, vp.rect)
                    {
                        mesh.line(a, b, 1., rgba(*color, alpha));
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
                        rgba(*color, alpha),
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
                        rgba(*color, alpha),
                    );
                }
                TraceMark::Polygon(points, color) => {
                    let screen = points
                        .iter()
                        .copied()
                        .map(|point| vp.screen(point))
                        .collect::<Vec<_>>();
                    mesh.polygon(&screen, rgba(*color, 0.025 * alpha));
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
            if let Some((a, b, mode)) = doc.line_points(o.id)
                && let Some((a, b)) = clipped_line(vp.screen(a), vp.screen(b), mode, vp.rect)
            {
                mesh.styled_path(&[a, b], false, stroke, col);
            }
            if let Some(polygons) = doc.colorized_polygon_iteration_images(o.id) {
                let opacity = match doc.effective_visual_style(o.id) {
                    Some(VisualStyle::Polygon { opacity, .. }) => opacity,
                    _ => 32767,
                };
                for (points, color) in polygons {
                    let screen = points.into_iter().map(|p| vp.screen(p)).collect::<Vec<_>>();
                    mesh.polygon(
                        &screen,
                        if is_selected {
                            [
                                selected[0],
                                selected[1],
                                selected[2],
                                f32::from(opacity) / 65535.,
                            ]
                        } else {
                            rgba(color, f32::from(opacity) / 65535.)
                        },
                    );
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
                        mesh.disk(
                            vp.screen(center),
                            r,
                            if is_selected {
                                [
                                    selected[0],
                                    selected[1],
                                    selected[2],
                                    f32::from(opacity) / 65535.,
                                ]
                            } else {
                                rgba(color, f32::from(opacity) / 65535.)
                            },
                        );
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
            if let Some(strokes) = doc
                .locus_styled_strokes(o.id, 384)
                .or_else(|| doc.mapped_collection_strokes(o.id, 384))
                .or_else(|| doc.sampled_mapped_styled_strokes(o.id, 384))
            {
                mesh.locus_strokes(vp, &strokes, is_selected.then_some(selected));
            } else if let Some(surfaces) = doc
                .mapped_collection_surfaces(o.id, 384)
                .or_else(|| doc.locus_surfaces(o.id, 384))
            {
                for surface in surfaces {
                    let fill = if doc.selected.contains(&o.id) {
                        [1., 0., 1., 0.18]
                    } else {
                        match surface.style {
                            VisualStyle::Polygon { color, opacity } => {
                                rgba(color, f32::from(opacity) / 65535.)
                            }
                            _ => unreachable!(),
                        }
                    };
                    let screen = surface
                        .points
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
                if matches!(
                    doc.effective_visual_style(o.id),
                    Some(VisualStyle::Stroke { .. })
                ) {
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
                Kind::AngleMarker(..)
                | Kind::SignedAngleValue(..)
                | Kind::PolarAngleValue(..)
                | Kind::VertexAngleValue(..) => {
                    if let Some(marker) = doc.angle_marker_geometry(o.id) {
                        let screen = marker
                            .sector
                            .into_iter()
                            .map(|point| vp.screen(point))
                            .collect::<Vec<_>>();
                        mesh.polygon(
                            &screen,
                            [
                                col[0],
                                col[1],
                                col[2],
                                f32::from(o.angle_marker_style.unwrap().opacity) / 65535.,
                            ],
                        );
                        for path in marker.strokes {
                            let screen = path
                                .into_iter()
                                .map(|point| vp.screen(point))
                                .collect::<Vec<_>>();
                            mesh.styled_path(&screen, false, stroke, col);
                        }
                        for arrow in marker.arrows {
                            mesh.tri(
                                vp.screen(arrow[0]),
                                vp.screen(arrow[1]),
                                vp.screen(arrow[2]),
                                col,
                            );
                        }
                    }
                }
                Kind::SegmentMarker { .. } | Kind::SegmentArrow { .. } => {
                    if let Some(strokes) = doc.segment_marker_strokes(o.id) {
                        for [start, end] in strokes {
                            mesh.styled_path(
                                &[vp.screen(start), vp.screen(end)],
                                false,
                                stroke,
                                col,
                            );
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
                        r: f64::from(preferences.background_color[0]) / 255.,
                        g: f64::from(preferences.background_color[1]) / 255.,
                        b: f64::from(preferences.background_color[2]) / 255.,
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
