use crate::document::{Document, Kind, LineMode, Point};
use egui::{Pos2, Rect};
use std::fmt::Write as _;
use wgpu::util::DeviceExt;

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
        Kind::Parameter(group) => Some((group.anchor(), vec!["t₁ = 1.00".into()])),
        Kind::Calculation(group) => Some((group.anchor(), vec!["2 + 3 = 5.00".into()])),
        Kind::ValueTable(_, group) => {
            let mut lines = vec!["t₁".into()];
            lines.extend((0..group.rows()).map(|_| "1.00".into()));
            Some((group.anchor(), lines))
        }
        _ => None,
    }
}
pub fn number_screen_rect(vp: &Viewport, kind: &Kind) -> Option<Rect> {
    let (anchor, lines) = number_lines(kind)?;
    let width = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0) as f32
        * 10.
        + 16.;
    Some(Rect::from_min_size(
        vp.rect.min + egui::vec2(anchor.0 as f32, anchor.1 as f32),
        egui::vec2(width.max(66.), lines.len() as f32 * 21. + 8.),
    ))
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

fn grid_lines(doc: &Document, vp: &Viewport) -> Vec<(Pos2, Pos2)> {
    if !doc
        .grid_visible
        .get(doc.active_page)
        .copied()
        .unwrap_or(false)
    {
        return Vec::new();
    }
    let mut lines = Vec::new();
    for object in doc.objects.iter().filter(|object| {
        object.page == doc.active_page
            && object.visible
            && !object.deleted
            && matches!(object.kind, Kind::CoordinateSystem(_, _))
    }) {
        let Some(frame) = doc.graph_frame(object.id) else {
            continue;
        };
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
            continue;
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
    }
    lines
}

pub fn svg_page(doc: &Document, vp: &Viewport) -> Result<String, String> {
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
        .filter(|object| object.page == doc.active_page && object.visible && !object.deleted)
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
    for (a, b) in grid_lines(doc, vp) {
        writeln!(
            svg,
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
            a.x, a.y, b.x, b.y
        )
        .unwrap();
    }
    svg.push_str("</g>\n<g fill=\"none\" stroke=\"#24303d\" stroke-width=\"2\">\n");
    for o in doc
        .objects
        .iter()
        .filter(|o| o.visible && !o.deleted && o.page == doc.active_page)
    {
        if let Some((a, b, mode)) = doc.line_points(o.id)
            && let Some((a, b)) = clipped_line(vp.screen(a), vp.screen(b), mode, r)
        {
            writeln!(
                svg,
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
                a.x, a.y, b.x, b.y
            )
            .unwrap();
        }
        if let Some(points) = doc.function_plot_points(o.id, 384) {
            for pair in points.windows(2) {
                if let Some((a, b)) =
                    clipped_line(vp.screen(pair[0]), vp.screen(pair[1]), LineMode::Segment, r)
                {
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
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\"/>",
                at.x,
                at.y,
                radius as f32 * vp.scale
            )
            .unwrap();
        }
        if let Some(points) = doc.arc_points(o.id, 128) {
            let mut points = points.into_iter();
            if let Some(first) = points.next() {
                let at = vp.screen(first);
                write!(svg, "<path d=\"M {} {}", at.x, at.y).unwrap();
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
                "<polygon points=\"{}\" fill=\"#f6ad60\" fill-opacity=\"0.35\"/>",
                vertices
            )
            .unwrap();
        }
    }
    svg.push_str("</g>\n<g fill=\"#176bd6\">\n");
    for o in doc
        .objects
        .iter()
        .filter(|o| o.visible && !o.deleted && o.page == doc.active_page)
    {
        if matches!(o.kind, Kind::GraphVerticalUnit(_, _)) {
            continue;
        }
        if let Some(p) = doc.position(o.id) {
            let at = vp.screen(p);
            writeln!(svg, "<circle cx=\"{}\" cy=\"{}\" r=\"4\"/>", at.x, at.y).unwrap();
        }
    }
    svg.push_str("</g>\n<g fill=\"#232323\" font-family=\"sans-serif\" font-size=\"16\">\n");
    for object in doc
        .objects
        .iter()
        .filter(|o| o.page == doc.active_page && o.visible && !o.deleted)
    {
        if let Kind::PlainText { text, rect } = &object.kind {
            let at = vp.screen(Point {
                x: rect.left.into(),
                y: rect.top.into(),
            });
            for (line, content) in text.split('\n').enumerate() {
                writeln!(
                    svg,
                    "<text x=\"{}\" y=\"{}\">{}</text>",
                    at.x,
                    at.y + 16. + line as f32 * 19.,
                    xml_text(content)
                )
                .unwrap();
            }
        }
        if let Some((anchor, lines)) = number_lines(&object.kind) {
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
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">{}</text>",
                at.x + offset.0 as f32,
                at.y + offset.1 as f32 + 13.,
                xml_text(name)
            )
            .unwrap();
        }
    }
    let mut row = 0;
    for object in doc
        .objects
        .iter()
        .filter(|object| object.page == doc.active_page && object.visible && !object.deleted)
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
        if let Kind::FunctionDefinition(expression) = object.kind {
            writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">{}</text>",
                r.min.x + 12.,
                r.min.y + 28. + row as f32 * 20.,
                expression.display()
            )
            .unwrap();
            row += 1;
        }
    }
    svg.push_str("</g>\n");
    for object in doc
        .objects
        .iter()
        .filter(|object| object.page == doc.active_page && object.visible && !object.deleted)
    {
        if let Kind::HideShowButton { label, anchor, .. } = &object.kind {
            let box_rect = action_button_rect(vp, *anchor, label);
            writeln!(svg, "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"3\" fill=\"#e7e9ed\" stroke=\"#5e6672\"/>", box_rect.min.x, box_rect.min.y, box_rect.width(), box_rect.height()).unwrap();
            writeln!(svg, "<text x=\"{}\" y=\"{}\" font-family=\"sans-serif\" font-size=\"14\" fill=\"#20242a\">{}</text>", box_rect.min.x + 8., box_rect.min.y + 17., xml_text(label)).unwrap();
        }
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
    fn polygon(&mut self, points: &[Pos2], outline: [f32; 4]) {
        if points.len() < 3 {
            return;
        }
        let cross =
            |a: Pos2, b: Pos2, c: Pos2| (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
        let signed_area: f32 = points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .take(points.len())
            .map(|(a, b)| a.x * b.y - b.x * a.y)
            .sum();
        let winding = signed_area.signum();
        let mut remaining: Vec<usize> = (0..points.len()).collect();
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
                    self.tri(a, b, c, [0.96, 0.58, 0.22, 0.35]);
                    break;
                }
            }
            let Some(index) = ear else {
                break;
            };
            remaining.remove(index);
        }
        for (a, b) in points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .take(points.len())
        {
            self.line(*a, *b, 2., outline);
        }
    }
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
        for (a, b) in grid_lines(doc, vp) {
            mesh.line(a, b, 1., [0.78, 0.83, 0.88, 1.]);
        }
        // Draw paths first and points last.
        for o in &doc.objects {
            if !o.visible || o.deleted || o.page != doc.active_page {
                continue;
            }
            let col = if doc.selected.contains(&o.id) {
                selected
            } else {
                dark
            };
            if let Some((a, b, mode)) = doc.line_points(o.id) {
                if let Some((a, b)) = clipped_line(vp.screen(a), vp.screen(b), mode, vp.rect) {
                    mesh.line(a, b, 2., col);
                }
            }
            if let Some(points) = doc.function_plot_points(o.id, 384) {
                for pair in points.windows(2) {
                    if let Some((a, b)) = clipped_line(
                        vp.screen(pair[0]),
                        vp.screen(pair[1]),
                        LineMode::Segment,
                        vp.rect,
                    ) {
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
                | Kind::ReflectedCircle(_, _)
                | Kind::ScaledCircle(_, _, _) => {
                    if let Some((a, radius)) = doc.circle_points(o.id) {
                        let c = vp.screen(a);
                        let r = (radius as f32 * vp.scale).abs();
                        if r > 0.5 && r < 100000. {
                            let n = ((r * 0.45) as usize).clamp(32, 256);
                            for i in 0..n {
                                let t = i as f32 * std::f32::consts::TAU / n as f32;
                                let u = (i + 1) as f32 * std::f32::consts::TAU / n as f32;
                                mesh.line(
                                    c + egui::vec2(t.cos(), t.sin()) * r,
                                    c + egui::vec2(u.cos(), u.sin()) * r,
                                    2.,
                                    col,
                                );
                            }
                        }
                    }
                }
                Kind::ThreePointArc(_, _, _) | Kind::ArcOnCircle(_, _, _) => {
                    if let Some(points) = doc.arc_points(o.id, 128) {
                        for pair in points.windows(2) {
                            mesh.line(vp.screen(pair[0]), vp.screen(pair[1]), 2., col);
                        }
                    }
                }
                Kind::Polygon(_) => {
                    if let Some(points) = doc.polygon_points(o.id) {
                        let screen = points
                            .into_iter()
                            .map(|point| vp.screen(point))
                            .collect::<Vec<_>>();
                        mesh.polygon(&screen, col);
                    }
                }
                _ => {}
            }
        }
        for o in &doc.objects {
            if !o.visible || o.deleted || o.page != doc.active_page {
                continue;
            }
            if matches!(o.kind, Kind::GraphVerticalUnit(_, _)) {
                continue;
            }
            if let Some(p) = doc.position(o.id) {
                mesh.disk(
                    vp.screen(p),
                    if doc.selected.contains(&o.id) {
                        5.5
                    } else {
                        4.
                    },
                    if doc.selected.contains(&o.id) {
                        selected
                    } else {
                        blue
                    },
                );
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
