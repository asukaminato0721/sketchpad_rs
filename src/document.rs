use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
impl Point {
    pub fn distance(self, p: Self) -> f64 {
        (self.x - p.x).hypot(self.y - p.y)
    }
}

impl Kind {
    pub fn dependencies(&self) -> Vec<usize> {
        match *self {
            Kind::Free(_) | Kind::Unsupported(_) => vec![],
            Kind::PointOnSegment(host, _)
            | Kind::PointOnCircle(host, _, _)
            | Kind::PointOnArc(host, _)
            | Kind::PolarTranslatedPoint(host, _, _, _) => vec![host],
            Kind::Midpoint(a, b)
            | Kind::Segment(a, b)
            | Kind::Line(a, b)
            | Kind::Ray(a, b)
            | Kind::Parallel(a, b)
            | Kind::Perpendicular(a, b)
            | Kind::Circle(a, b)
            | Kind::CircleBySegment(a, b)
            | Kind::Intersection(a, b)
            | Kind::LineCircleIntersection(a, b, _)
            | Kind::CircleCircleIntersection(a, b, _)
            | Kind::ReflectedPoint(a, b)
            | Kind::ReflectedCircle(a, b) => vec![a, b],
            Kind::ScaledPoint(source, center, _)
            | Kind::ScaledCircle(source, center, _)
            | Kind::RotatedPoint(source, center, _) => {
                vec![source, center]
            }
            Kind::Bisector(a, b, c) | Kind::ThreePointArc(a, b, c) | Kind::ArcOnCircle(a, b, c) => {
                vec![a, b, c]
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Free(Point),
    PointOnSegment(usize, f64),
    PointOnCircle(usize, f64, f64),
    PointOnArc(usize, f64),
    PolarTranslatedPoint(usize, f64, f64, f64),
    ReflectedPoint(usize, usize),
    ReflectedCircle(usize, usize),
    ScaledPoint(usize, usize, f64),
    ScaledCircle(usize, usize, f64),
    RotatedPoint(usize, usize, f64),
    Midpoint(usize, usize),
    Segment(usize, usize),
    Line(usize, usize),
    Ray(usize, usize),
    Parallel(usize, usize),
    Perpendicular(usize, usize),
    Circle(usize, usize),
    CircleBySegment(usize, usize),
    Bisector(usize, usize, usize),
    ThreePointArc(usize, usize, usize),
    ArcOnCircle(usize, usize, usize),
    Intersection(usize, usize),
    LineCircleIntersection(usize, usize, usize),
    CircleCircleIntersection(usize, usize, usize),
    Unsupported(u16),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineMode {
    Segment,
    Line,
    Ray,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub id: usize,
    pub page: usize,
    pub kind: Kind,
    pub visible: bool,
    pub point_offset: Option<usize>,
    pub constraint_offset: Option<usize>,
    pub header_offset: Option<usize>,
}
#[derive(Clone, Debug)]
pub struct Record {
    pub kind: u32,
    pub start: usize,
    pub data: std::ops::Range<usize>,
}
fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}
fn f64_at(b: &[u8], i: usize) -> f64 {
    f64::from_le_bytes(b[i..i + 8].try_into().unwrap())
}
pub fn records(b: &[u8]) -> Result<Vec<Record>, String> {
    if b.len() < 12 || &b[..4] != b"GSP4" {
        return Err("Expected a GSP4 document".into());
    }
    let mut out = Vec::new();
    let mut i = 4;
    while i < b.len() {
        if i + 8 > b.len() {
            return Err(format!("Truncated record at {i:#x}"));
        }
        let n = u32_at(b, i) as usize;
        let end = (i + 8).checked_add(n).ok_or("Record length overflow")?;
        if end > b.len() {
            return Err(format!("Truncated payload at {i:#x}"));
        }
        out.push(Record {
            kind: u32_at(b, i + 4),
            start: i,
            data: i + 8..end,
        });
        i = end + n % 2;
    }
    if i != b.len() {
        return Err("Missing record padding".into());
    }
    Ok(out)
}
fn put_record(out: &mut Vec<u8>, kind: u32, data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(data);
    if data.len() % 2 != 0 {
        out.push(0);
    }
}
fn header(kind: u16) -> [u8; 28] {
    // These bytes come from the supplied original Sketchpad fixtures.
    match kind {
        0 => [
            0, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        1 => [
            1, 0, 0, 0, 0, 0, 1, 0, 11, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        2 => [
            2, 0, 0, 0, 0, 0, 2, 0, 14, 0, 0x22, 1, 0, 0, 0x80, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        3 => [
            3, 0, 0, 0, 0, 0, 3, 0, 28, 0, 0x22, 1, 0, 0x80, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        4 => [
            4, 0, 0, 0, 0, 0, 3, 0, 28, 0, 0x22, 1, 0, 0x80, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        5 => [
            5, 0, 0, 0, 0, 0, 2, 0, 47, 0, 0x22, 1, 0, 0, 0x80, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        6 => [
            6, 0, 0, 0, 0, 0, 2, 0, 46, 0, 0x22, 1, 0, 0, 0x80, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        7 => [
            7, 0, 0, 0, 0, 0, 2, 0, 44, 0, 0x22, 1, 0, 0, 0x80, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        15 => [
            15, 0, 0, 0, 0, 0, 1, 0, 10, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        21 => [
            21, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        30 => [
            30, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x51, 0x5c, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        27 => [
            27, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0xdf, 0x3f, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        34 => [
            34, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0, 0, 0xc1, 0xbd,
            0xf0, 0xff, 0, 0, 0, 0,
        ],
        9 | 11..=14 => [
            kind as u8, 0, 0, 0, 0, 0, 1, 0, 0x2d, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x39,
            0x20, 0xc1, 0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        63 => [
            63, 0, 0, 0, 0, 0, 2, 0, 12, 0, 0x22, 1, 0, 0, 0x80, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        64 => [
            64, 0, 0, 0, 0, 0, 2, 0, 13, 0, 0x22, 1, 0, 0, 0x80, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        81 => [
            81, 0, 0, 0, 0, 0, 12, 0, 67, 0, 0x22, 1, 0, 0x80, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        79 => [
            79, 0, 0, 0, 0, 0, 12, 0, 67, 0, 0x22, 1, 0, 0x80, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        _ => unreachable!(),
    }
}
fn is_straight_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Segment(_, _)
            | Kind::Line(_, _)
            | Kind::Ray(_, _)
            | Kind::Bisector(_, _, _)
            | Kind::Parallel(_, _)
            | Kind::Perpendicular(_, _)
    )
}

fn is_point_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Free(_)
            | Kind::PointOnSegment(_, _)
            | Kind::PointOnCircle(_, _, _)
            | Kind::PointOnArc(_, _)
            | Kind::PolarTranslatedPoint(_, _, _, _)
            | Kind::ReflectedPoint(_, _)
            | Kind::ScaledPoint(_, _, _)
            | Kind::RotatedPoint(_, _, _)
            | Kind::Midpoint(_, _)
            | Kind::Intersection(_, _)
            | Kind::LineCircleIntersection(_, _, _)
            | Kind::CircleCircleIntersection(_, _, _)
    )
}

fn is_circle_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Circle(_, _)
            | Kind::CircleBySegment(_, _)
            | Kind::ReflectedCircle(_, _)
            | Kind::ScaledCircle(_, _, _)
    )
}

fn reflect_point(point: Point, a: Point, b: Point) -> Option<Point> {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let length_sq = dx * dx + dy * dy;
    if !length_sq.is_finite() || length_sq <= 1e-18 {
        return None;
    }
    let t = ((point.x - a.x) * dx + (point.y - a.y) * dy) / length_sq;
    let projected = Point {
        x: a.x + t * dx,
        y: a.y + t * dy,
    };
    let reflected = Point {
        x: 2. * projected.x - point.x,
        y: 2. * projected.y - point.y,
    };
    (reflected.x.is_finite() && reflected.y.is_finite()).then_some(reflected)
}

fn intersect_straight(
    first: (Point, Point, LineMode),
    second: (Point, Point, LineMode),
) -> Option<Point> {
    let (a, b, first_mode) = first;
    let (c, d, second_mode) = second;
    let r = Point {
        x: b.x - a.x,
        y: b.y - a.y,
    };
    let s = Point {
        x: d.x - c.x,
        y: d.y - c.y,
    };
    let (r_len, s_len) = (r.x.hypot(r.y), s.x.hypot(s.y));
    if r_len < 1e-12 || s_len < 1e-12 {
        return None;
    }
    let cross = |p: Point, q: Point| p.x * q.y - p.y * q.x;
    let denominator = cross(r, s);
    if denominator.abs() <= r_len * s_len * 1e-12 {
        return None;
    }
    let delta = Point {
        x: c.x - a.x,
        y: c.y - a.y,
    };
    let t = cross(delta, s) / denominator;
    let u = cross(delta, r) / denominator;
    let contains = |mode: LineMode, parameter: f64| match mode {
        LineMode::Segment => parameter >= -1e-10 && parameter <= 1. + 1e-10,
        LineMode::Ray => parameter >= -1e-10,
        LineMode::Line => true,
    };
    if !contains(first_mode, t) || !contains(second_mode, u) {
        return None;
    }
    let result = Point {
        x: a.x + r.x * t,
        y: a.y + r.y * t,
    };
    (result.x.is_finite() && result.y.is_finite()).then_some(result)
}
fn line_circle_intersection(
    line: (Point, Point, LineMode),
    circle: (Point, f64),
    variant: usize,
) -> Option<Point> {
    let (a, b, mode) = line;
    let (center, radius) = circle;
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let length_squared = dx * dx + dy * dy;
    if length_squared <= 1e-18 || radius <= 1e-9 {
        return None;
    }
    let (fx, fy) = (a.x - center.x, a.y - center.y);
    let linear = 2. * (fx * dx + fy * dy);
    let constant = fx * fx + fy * fy - radius * radius;
    let discriminant = linear * linear - 4. * length_squared * constant;
    if discriminant < -1e-9 {
        return None;
    }
    let root = discriminant.max(0.).sqrt();
    let mut parameters = [
        (-linear - root) / (2. * length_squared),
        (-linear + root) / (2. * length_squared),
    ]
    .into_iter()
    .filter(|t| match mode {
        LineMode::Segment => *t >= -1e-10 && *t <= 1. + 1e-10,
        LineMode::Ray => *t >= -1e-10,
        LineMode::Line => true,
    })
    .collect::<Vec<_>>();
    parameters.sort_by(f64::total_cmp);
    let t = *parameters.get(variant.min(parameters.len().checked_sub(1)?))?;
    let result = Point {
        x: a.x + dx * t,
        y: a.y + dy * t,
    };
    (result.x.is_finite() && result.y.is_finite()).then_some(result)
}
fn circle_circle_intersection(
    first: (Point, f64),
    second: (Point, f64),
    variant: usize,
) -> Option<Point> {
    let (a, first_radius) = first;
    let (b, second_radius) = second;
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let distance = dx.hypot(dy);
    if first_radius <= 1e-9
        || second_radius <= 1e-9
        || distance <= 1e-9
        || distance > first_radius + second_radius + 1e-9
        || distance < (first_radius - second_radius).abs() - 1e-9
    {
        return None;
    }
    let along = (first_radius * first_radius - second_radius * second_radius + distance * distance)
        / (2. * distance);
    let height_squared = first_radius * first_radius - along * along;
    if height_squared < -1e-9 {
        return None;
    }
    let height = height_squared.max(0.).sqrt();
    let (ux, uy) = (dx / distance, dy / distance);
    let base = Point {
        x: a.x + along * ux,
        y: a.y + along * uy,
    };
    let mut results = [
        Point {
            x: base.x - height * uy,
            y: base.y + height * ux,
        },
        Point {
            x: base.x + height * uy,
            y: base.y - height * ux,
        },
    ];
    results.sort_by(|a, b| a.y.total_cmp(&b.y).then_with(|| a.x.total_cmp(&b.x)));
    results
        .get(variant)
        .copied()
        .filter(|p| p.x.is_finite() && p.y.is_finite())
}
fn three_point_arc_geometry(
    start: Point,
    mid: Point,
    end: Point,
) -> Option<(Point, f64, f64, f64)> {
    let determinant =
        2. * (start.x * (mid.y - end.y) + mid.x * (end.y - start.y) + end.x * (start.y - mid.y));
    if determinant.abs() <= 1e-9 {
        return None;
    }
    let square = |point: Point| point.x * point.x + point.y * point.y;
    let center = Point {
        x: (square(start) * (mid.y - end.y)
            + square(mid) * (end.y - start.y)
            + square(end) * (start.y - mid.y))
            / determinant,
        y: (square(start) * (end.x - mid.x)
            + square(mid) * (start.x - end.x)
            + square(end) * (mid.x - start.x))
            / determinant,
    };
    let radius = center.distance(start);
    if !radius.is_finite() || radius <= 1e-9 {
        return None;
    }
    let angle = |p: Point| (p.y - center.y).atan2(p.x - center.x);
    let (first, middle, last) = (angle(start), angle(mid), angle(end));
    let ccw = (last - first).rem_euclid(std::f64::consts::TAU);
    let mid_ccw = (middle - first).rem_euclid(std::f64::consts::TAU);
    let sweep = if mid_ccw <= ccw + 1e-9 {
        ccw
    } else {
        -(first - last).rem_euclid(std::f64::consts::TAU)
    };
    Some((center, radius, first, sweep))
}
#[derive(Clone, Debug)]
pub struct Document {
    pub objects: Vec<Object>,
    pub path: Option<PathBuf>,
    pub selected: BTreeSet<usize>,
    pub dirty: bool,
    pub partial_view: bool,
    pub active_page: usize,
    pub page_count: usize,
    original: Vec<u8>,
    source_count: usize,
    safe_structure: bool,
    pub undo: Vec<Vec<Object>>,
    pub redo: Vec<Vec<Object>>,
}
impl Document {
    pub fn new() -> Self {
        let template = include_bytes!("../assets/point_template.gsp");
        let rs = records(template).unwrap();
        let first = rs.iter().find(|r| r.kind == 0x7d0).unwrap().start;
        let trailer = rs.iter().find(|r| r.kind == 0x3e9).unwrap().start;
        let mut blank = template[..first].to_vec();
        blank.extend_from_slice(&template[trailer..]);
        let preamble = rs.iter().find(|r| r.kind == 0x3e8).unwrap();
        blank[preamble.data.start + 24..preamble.data.start + 28]
            .copy_from_slice(&0_u32.to_le_bytes());
        Self::from_bytes(&blank, None).unwrap()
    }
    pub fn open(path: &Path) -> Result<Self, String> {
        Self::from_bytes(
            &fs::read(path).map_err(|e| e.to_string())?,
            Some(path.to_path_buf()),
        )
    }
    pub fn from_bytes(b: &[u8], path: Option<PathBuf>) -> Result<Self, String> {
        let rs = records(b)?;
        let baseline_records = records(include_bytes!("../assets/point_template.gsp"))?;
        let safe_outside: BTreeSet<u32> = baseline_records
            .iter()
            .map(|record| record.kind)
            .chain([0x2328])
            .collect();
        let mut objects: Vec<Object> = Vec::new();
        let mut i = 0;
        let mut safe = true;
        let (mut page, mut page_base, mut page_count) = (0, 0, 0);
        while i < rs.len() {
            if rs[i].kind == 0x3e8 {
                if page_count > 0 {
                    page += 1;
                    page_base = objects.len();
                }
                page_count += 1;
                i += 1;
                continue;
            }
            if rs[i].kind != 0x7d0 {
                if !safe_outside.contains(&rs[i].kind) {
                    safe = false;
                }
                i += 1;
                continue;
            }
            let h = &b[rs[i].data.clone()];
            if h.len() != 16 && h.len() != 28 {
                return Err(format!("Invalid object header at {:#x}", rs[i].start));
            }
            let header_offset = rs[i].data.start;
            let raw = u32_at(h, 0);
            let code = raw as u16;
            let mut point = None;
            let mut point_offset = None;
            let mut constraint_payload: Option<(Vec<u8>, usize)> = None;
            let mut refs = Vec::new();
            i += 1;
            while i < rs.len() && rs[i].kind != 0x7d7 {
                let r = &rs[i];
                let d = &b[r.data.clone()];
                match r.kind {
                    0x899 if d.len() == 16 => {
                        point = Some(Point {
                            x: f64_at(d, 0),
                            y: f64_at(d, 8),
                        });
                        point_offset = Some(r.data.start);
                    }
                    0x7d3 if code == 15 || code == 21 || code == 27 || code == 30 => {
                        constraint_payload = Some((d.to_vec(), r.data.start));
                    }
                    0x7d2 | 0x7d3 if d.len() >= 4 && (d.len() - 4) % 4 == 0 => {
                        let n = u32_at(d, 0) as usize;
                        if n == (d.len() - 4) / 4 {
                            refs = (0..n)
                                .map(|j| {
                                    let raw = u32_at(d, 4 + j * 4) as usize;
                                    if raw == 0 { 0 } else { page_base + raw }
                                })
                                .collect();
                        }
                    }
                    _ => {}
                }
                if !matches!(r.kind, 0x899 | 0x7d2 | 0x7d3) {
                    safe = false;
                }
                i += 1;
            }
            if i == rs.len() {
                return Err("Unterminated object group".into());
            }
            let kind = if refs
                .iter()
                .any(|&reference| reference == 0 || reference > objects.len())
            {
                Kind::Unsupported(code)
            } else {
                match (code, point, refs.as_slice()) {
                    (0, Some(p), _) => Kind::Free(p),
                    (15, _, [host]) => {
                        match (objects.get(host - 1).map(|o| &o.kind), &constraint_payload) {
                            (Some(Kind::Segment(_, _)), Some((payload, _)))
                                if payload.len() == 12
                                    && payload[..4] == [15, 0, 14, 0]
                                    && f64_at(payload, 4).is_finite() =>
                            {
                                Kind::PointOnSegment(*host, f64_at(payload, 4))
                            }
                            (
                                Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _) | Kind::ReflectedCircle(_, _) | Kind::ScaledCircle(_, _, _)),
                                Some((payload, _)),
                            ) if payload.len() == 20
                                && payload[..4] == [15, 0, 28, 0]
                                && f64_at(payload, 4).is_finite()
                                && f64_at(payload, 12).is_finite() =>
                            {
                                Kind::PointOnCircle(*host, f64_at(payload, 4), f64_at(payload, 12))
                            }
                            (
                                Some(Kind::ThreePointArc(_, _, _) | Kind::ArcOnCircle(_, _, _)),
                                Some((payload, _)),
                            ) if payload.len() == 12
                                && payload[..4] == [15, 0, 67, 0]
                                && f64_at(payload, 4).is_finite() =>
                            {
                                Kind::PointOnArc(*host, f64_at(payload, 4))
                            }
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (21, _, [origin]) => match &constraint_payload {
                        Some((payload, _))
                            if payload.len() == 60
                                && payload[..4] == [21, 0, 0, 0]
                                && f64_at(payload, 20).is_finite()
                                && f64_at(payload, 32).is_finite()
                                && f64_at(payload, 40).is_finite()
                                && objects.get(origin - 1).is_some_and(|o| is_point_kind(&o.kind)) =>
                        {
                            Kind::PolarTranslatedPoint(
                                *origin,
                                f64_at(payload, 20),
                                f64_at(payload, 32),
                                f64_at(payload, 40),
                            )
                        }
                        _ => Kind::Unsupported(code),
                    },
                    (34, _, [source, mirror])
                        if objects.get(source - 1).is_some_and(|o| is_point_kind(&o.kind))
                            && objects.get(mirror - 1).is_some_and(|o| is_straight_kind(&o.kind)) =>
                    {
                        Kind::ReflectedPoint(*source, *mirror)
                    }
                    (34, _, [source, mirror])
                        if objects.get(source - 1).is_some_and(|o| is_circle_kind(&o.kind))
                            && objects.get(mirror - 1).is_some_and(|o| is_straight_kind(&o.kind)) =>
                    {
                        Kind::ReflectedCircle(*source, *mirror)
                    }
                    (30, _, [source, center])
                        if objects.get(center - 1).is_some_and(|o| is_point_kind(&o.kind)) =>
                    {
                        match &constraint_payload {
                            Some((payload, _)) if payload.len() == 12 && payload[..4] == [30, 0, 0, 0] && f64_at(payload, 4).is_finite() => {
                                let factor = f64_at(payload, 4);
                                match objects.get(source - 1).map(|o| &o.kind) {
                                    Some(kind) if is_point_kind(kind) => Kind::ScaledPoint(*source, *center, factor),
                                    Some(kind) if is_circle_kind(kind) => Kind::ScaledCircle(*source, *center, factor),
                                    _ => Kind::Unsupported(code),
                                }
                            }
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (27, _, [source, center])
                        if objects.get(source - 1).is_some_and(|o| is_point_kind(&o.kind))
                            && objects.get(center - 1).is_some_and(|o| is_point_kind(&o.kind)) =>
                    {
                        match &constraint_payload {
                            Some((payload, _)) if payload.len() == 36 && payload[..4] == [27, 0, 0, 0] && f64_at(payload, 20).is_finite()
                                && f64_at(payload, 4).is_finite() && f64_at(payload, 12).is_finite()
                                && (f64_at(payload, 4) + f64_at(payload, 20).to_radians().sin()).abs() < 1e-6
                                && (f64_at(payload, 12) - f64_at(payload, 20).to_radians().cos()).abs() < 1e-6 => {
                                Kind::RotatedPoint(*source, *center, f64_at(payload, 20))
                            }
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (1, _, [segment]) => {
                        match objects.get(segment.saturating_sub(1)).map(|o| &o.kind) {
                            Some(Kind::Segment(a, b)) => Kind::Midpoint(*a, *b),
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (2, _, [a, b]) => Kind::Segment(*a, *b),
                    (3, _, [a, b]) => Kind::Circle(*a, *b),
                    (4, _, [a, b]) => Kind::CircleBySegment(*a, *b),
                    (5, _, [a, b]) => Kind::Perpendicular(*a, *b),
                    (6, _, [a, b]) => Kind::Parallel(*a, *b),
                    (7, _, [a, b, c]) => Kind::Bisector(*a, *b, *c),
                    (9, _, [a, b]) => Kind::Intersection(*a, *b),
                    (11, _, [a, b])
                        if objects
                            .get(a.saturating_sub(1))
                            .is_some_and(|object| is_straight_kind(&object.kind))
                            && objects.get(b.saturating_sub(1)).is_some_and(|object| {
                                is_circle_kind(&object.kind)
                            }) =>
                    {
                        Kind::LineCircleIntersection(*a, *b, 1)
                    }
                    (12, _, [a, b])
                        if objects
                            .get(a.saturating_sub(1))
                            .is_some_and(|object| is_straight_kind(&object.kind))
                            && objects.get(b.saturating_sub(1)).is_some_and(|object| {
                                is_circle_kind(&object.kind)
                            }) =>
                    {
                        Kind::LineCircleIntersection(*a, *b, 0)
                    }
                    (13, _, [a, b]) => Kind::CircleCircleIntersection(*a, *b, 1),
                    (14, _, [a, b]) => Kind::CircleCircleIntersection(*a, *b, 0),
                    (63, _, [a, b]) => Kind::Line(*a, *b),
                    (64, _, [a, b]) => Kind::Ray(*a, *b),
                    (81, _, [a, b, c]) => Kind::ThreePointArc(*a, *b, *c),
                    (79, _, [circle, start, end])
                        if objects.get(circle - 1).is_some_and(|o| is_circle_kind(&o.kind))
                            && objects.get(start - 1).is_some_and(|o| matches!(o.kind, Kind::PointOnCircle(host, _, _) if host == *circle))
                            && objects.get(end - 1).is_some_and(|o| matches!(o.kind, Kind::PointOnCircle(host, _, _) if host == *circle)) =>
                    {
                        Kind::ArcOnCircle(*circle, *start, *end)
                    }
                    _ => Kind::Unsupported(code),
                }
            };
            if matches!(kind, Kind::Unsupported(_)) {
                safe = false;
            }
            objects.push(Object {
                id: objects.len() + 1,
                page,
                kind,
                visible: raw & 0x10000 == 0,
                point_offset,
                constraint_offset: constraint_payload.map(|(_, offset)| offset),
                header_offset: Some(header_offset),
            });
            i += 1;
        }
        let source_count = objects.len();
        let preambles: Vec<_> = rs
            .iter()
            .filter(|r| r.kind == 0x3e8 && r.data.len() >= 28)
            .collect();
        if preambles.len() != 1 || u32_at(b, preambles[0].data.start + 24) as usize != source_count
        {
            safe = false;
        }
        Ok(Self {
            objects,
            path,
            selected: BTreeSet::new(),
            dirty: false,
            partial_view: !safe,
            active_page: 0,
            page_count: page_count.max(1),
            original: b.to_vec(),
            source_count,
            safe_structure: safe,
            undo: Vec::new(),
            redo: Vec::new(),
        })
    }
    pub fn checkpoint(&mut self) {
        self.undo.push(self.objects.clone());
        self.redo.clear();
        self.dirty = true;
    }
    pub fn undo(&mut self) {
        if let Some(p) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.objects, p));
            self.dirty = true;
        }
    }
    pub fn redo(&mut self) {
        if let Some(p) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.objects, p));
            self.dirty = true;
        }
    }
    pub fn add(&mut self, kind: Kind) -> usize {
        self.checkpoint();
        let id = self.objects.len() + 1;
        self.objects.push(Object {
            id,
            page: self.active_page,
            kind,
            visible: true,
            point_offset: None,
            constraint_offset: None,
            header_offset: None,
        });
        id
    }
    pub fn set_selected_visibility(&mut self, visible: bool) {
        if !self
            .objects
            .iter()
            .any(|o| self.selected.contains(&o.id) && o.visible != visible)
        {
            return;
        }
        self.checkpoint();
        for o in &mut self.objects {
            if self.selected.contains(&o.id) {
                o.visible = visible;
            }
        }
        if !visible {
            self.selected.clear();
        }
    }
    pub fn show_all_hidden(&mut self) {
        if !self
            .objects
            .iter()
            .any(|o| o.page == self.active_page && !o.visible)
        {
            return;
        }
        self.checkpoint();
        for o in &mut self.objects {
            if o.page == self.active_page {
                o.visible = true;
            }
        }
    }
    pub fn position(&self, id: usize) -> Option<Point> {
        fn go(d: &Document, id: usize, seen: &mut BTreeSet<usize>) -> Option<Point> {
            if !seen.insert(id) {
                return None;
            }
            let p = match &d.objects.get(id.checked_sub(1)?)?.kind {
                Kind::Free(p) => Some(*p),
                Kind::PointOnSegment(host, t) => {
                    let (a, b, LineMode::Segment) = d.line_points(*host)? else {
                        return None;
                    };
                    Some(Point {
                        x: a.x + (b.x - a.x) * t,
                        y: a.y + (b.y - a.y) * t,
                    })
                }
                Kind::PointOnCircle(host, unit_x, unit_y) => {
                    let (center, radius) = d.circle_points(*host)?;
                    Some(Point {
                        x: center.x + radius * unit_x,
                        y: center.y - radius * unit_y,
                    })
                }
                Kind::PointOnArc(host, t) => d.point_on_arc(*host, *t),
                Kind::PolarTranslatedPoint(origin, angle, units_to_raw, distance) => {
                    let source = go(d, *origin, seen)?;
                    let radians = angle.to_radians();
                    let step = units_to_raw * distance;
                    Some(Point {
                        x: source.x + step * radians.cos(),
                        y: source.y - step * radians.sin(),
                    })
                }
                Kind::ReflectedPoint(source, mirror) => {
                    let source = go(d, *source, seen)?;
                    let (a, b, _) = d.line_points(*mirror)?;
                    reflect_point(source, a, b)
                }
                Kind::ScaledPoint(source, center, factor) => {
                    let source = go(d, *source, seen)?;
                    let center = go(d, *center, seen)?;
                    Some(Point {
                        x: center.x + factor * (source.x - center.x),
                        y: center.y + factor * (source.y - center.y),
                    })
                }
                Kind::RotatedPoint(source, center, angle) => {
                    let source = go(d, *source, seen)?;
                    let center = go(d, *center, seen)?;
                    let theta = angle.to_radians();
                    let dx = source.x - center.x;
                    let dy = source.y - center.y;
                    Some(Point {
                        x: center.x + dx * theta.cos() + dy * theta.sin(),
                        y: center.y - dx * theta.sin() + dy * theta.cos(),
                    })
                }
                Kind::Midpoint(a, b) => {
                    let a = go(d, *a, seen)?;
                    let b = go(d, *b, seen)?;
                    Some(Point {
                        x: (a.x + b.x) / 2.,
                        y: (a.y + b.y) / 2.,
                    })
                }
                Kind::Intersection(a, b) => {
                    let first = d.line_points(*a)?;
                    let second = d.line_points(*b)?;
                    intersect_straight(first, second)
                }
                Kind::LineCircleIntersection(a, b, variant) => {
                    line_circle_intersection(d.line_points(*a)?, d.circle_points(*b)?, *variant)
                }
                Kind::CircleCircleIntersection(a, b, variant) => {
                    circle_circle_intersection(d.circle_points(*a)?, d.circle_points(*b)?, *variant)
                }
                _ => None,
            };
            seen.remove(&id);
            p
        }
        go(self, id, &mut BTreeSet::new())
    }
    pub fn line_points(&self, id: usize) -> Option<(Point, Point, LineMode)> {
        fn go(
            d: &Document,
            id: usize,
            seen: &mut BTreeSet<usize>,
        ) -> Option<(Point, Point, LineMode)> {
            if !seen.insert(id) {
                return None;
            }
            let result = match d.objects.get(id.checked_sub(1)?)?.kind {
                Kind::Segment(a, b) => Some((d.position(a)?, d.position(b)?, LineMode::Segment)),
                Kind::Line(a, b) => Some((d.position(a)?, d.position(b)?, LineMode::Line)),
                Kind::Ray(a, b) => Some((d.position(a)?, d.position(b)?, LineMode::Ray)),
                Kind::Parallel(p, line) | Kind::Perpendicular(p, line) => {
                    let origin = d.position(p)?;
                    let (a, b, _) = go(d, line, seen)?;
                    let mut direction = Point {
                        x: b.x - a.x,
                        y: b.y - a.y,
                    };
                    if matches!(d.objects[id - 1].kind, Kind::Perpendicular(_, _)) {
                        direction = Point {
                            x: -direction.y,
                            y: direction.x,
                        };
                    }
                    Some((
                        origin,
                        Point {
                            x: origin.x + direction.x,
                            y: origin.y + direction.y,
                        },
                        LineMode::Line,
                    ))
                }
                Kind::Bisector(start, vertex, end) => {
                    let (a, b, c) = (d.position(start)?, d.position(vertex)?, d.position(end)?);
                    let normalize = |p: Point| {
                        let len = p.x.hypot(p.y);
                        (len > 1e-9).then_some(Point {
                            x: p.x / len,
                            y: p.y / len,
                        })
                    };
                    let first = normalize(Point {
                        x: a.x - b.x,
                        y: a.y - b.y,
                    })?;
                    let second = normalize(Point {
                        x: c.x - b.x,
                        y: c.y - b.y,
                    })?;
                    let sum = Point {
                        x: first.x + second.x,
                        y: first.y + second.y,
                    };
                    let direction = normalize(sum).unwrap_or(Point {
                        x: -first.y,
                        y: first.x,
                    });
                    Some((
                        b,
                        Point {
                            x: b.x + direction.x,
                            y: b.y + direction.y,
                        },
                        LineMode::Ray,
                    ))
                }
                _ => None,
            };
            seen.remove(&id);
            result
        }
        go(self, id, &mut BTreeSet::new())
    }
    pub fn circle_points(&self, id: usize) -> Option<(Point, f64)> {
        if let Kind::ReflectedCircle(source, mirror) = self.objects.get(id.checked_sub(1)?)?.kind {
            let (center, radius) = self.circle_points(source)?;
            let (a, b, _) = self.line_points(mirror)?;
            return Some((reflect_point(center, a, b)?, radius));
        }
        if let Kind::ScaledCircle(source, center, factor) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (source_center, radius) = self.circle_points(source)?;
            let center = self.position(center)?;
            let scaled = Point {
                x: center.x + factor * (source_center.x - center.x),
                y: center.y + factor * (source_center.y - center.y),
            };
            let radius = radius * factor.abs();
            return (scaled.x.is_finite()
                && scaled.y.is_finite()
                && radius.is_finite()
                && radius > 0.)
                .then_some((scaled, radius));
        }
        let (center_id, radius) = match self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::Circle(center, radius_point) => {
                let center_position = self.position(center)?;
                (
                    center,
                    center_position.distance(self.position(radius_point)?),
                )
            }
            Kind::CircleBySegment(center, segment) => {
                let Kind::Segment(a, b) = self.objects.get(segment.checked_sub(1)?)?.kind else {
                    return None;
                };
                (center, self.position(a)?.distance(self.position(b)?))
            }
            _ => return None,
        };
        let center = self.position(center_id)?;
        (radius.is_finite() && radius > 0.).then_some((center, radius))
    }
    fn arc_geometry(&self, id: usize) -> Option<(Point, f64, f64, f64, bool)> {
        let geometry = match self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::ThreePointArc(a, b, c) => {
                let (center, radius, start, sweep) = three_point_arc_geometry(
                    self.position(a)?,
                    self.position(b)?,
                    self.position(c)?,
                )?;
                (center, radius, start, sweep, false)
            }
            Kind::ArcOnCircle(circle, start, end) => {
                let (center, radius) = self.circle_points(circle)?;
                let start_point = self.position(start)?;
                let end_point = self.position(end)?;
                let a = (center.y - start_point.y).atan2(start_point.x - center.x);
                let b = (center.y - end_point.y).atan2(end_point.x - center.x);
                let sweep = (b - a).rem_euclid(std::f64::consts::TAU);
                (center, radius, a, sweep, true)
            }
            _ => return None,
        };
        Some(geometry)
    }
    pub fn point_on_arc(&self, id: usize, t: f64) -> Option<Point> {
        if !t.is_finite() {
            return None;
        }
        Some(Self::sample_arc(self.arc_geometry(id)?, t))
    }
    fn sample_arc(
        (center, radius, start, sweep, clockwise_y): (Point, f64, f64, f64, bool),
        t: f64,
    ) -> Point {
        let angle = start + sweep * t.clamp(0., 1.);
        Point {
            x: center.x + radius * angle.cos(),
            y: center.y
                + if clockwise_y {
                    -radius * angle.sin()
                } else {
                    radius * angle.sin()
                },
        }
    }
    fn arc_parameter(&self, id: usize, p: Point) -> Option<f64> {
        let (center, _, start, sweep, clockwise_y) = self.arc_geometry(id)?;
        if sweep.abs() < 1e-9 {
            return None;
        }
        let angle = (if clockwise_y {
            center.y - p.y
        } else {
            p.y - center.y
        })
        .atan2(p.x - center.x);
        let progress = if sweep >= 0. {
            (angle - start).rem_euclid(std::f64::consts::TAU)
        } else {
            (start - angle).rem_euclid(std::f64::consts::TAU)
        };
        if progress <= sweep.abs() {
            Some(progress / sweep.abs())
        } else {
            let first = self.point_on_arc(id, 0.)?;
            let last = self.point_on_arc(id, 1.)?;
            Some(if p.distance(first) <= p.distance(last) {
                0.
            } else {
                1.
            })
        }
    }
    pub fn arc_points(&self, id: usize, subdivisions: usize) -> Option<Vec<Point>> {
        let geometry = self.arc_geometry(id)?;
        let count = subdivisions.max(2);
        Some(
            (0..=count)
                .map(|step| Self::sample_arc(geometry, step as f64 / count as f64))
                .collect(),
        )
    }
    pub fn move_point(&mut self, id: usize, p: Point) -> bool {
        if !p.x.is_finite() || !p.y.is_finite() {
            return false;
        }
        let kind = match self.objects.get(id.saturating_sub(1)).map(|o| &o.kind) {
            Some(Kind::Free(_)) => Kind::Free(p),
            Some(Kind::PointOnSegment(host, _)) => {
                let host = *host;
                let Some((a, b, LineMode::Segment)) = self.line_points(host) else {
                    return false;
                };
                let dx = b.x - a.x;
                let dy = b.y - a.y;
                let length_sq = dx * dx + dy * dy;
                if length_sq < 1e-18 {
                    return false;
                }
                let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / length_sq;
                Kind::PointOnSegment(host, t.clamp(0., 1.))
            }
            Some(Kind::PointOnCircle(host, _, _)) => {
                let host = *host;
                let Some((center, _)) = self.circle_points(host) else {
                    return false;
                };
                let dx = p.x - center.x;
                let dy = p.y - center.y;
                let length = dx.hypot(dy);
                if length < 1e-9 {
                    return false;
                }
                Kind::PointOnCircle(host, dx / length, -dy / length)
            }
            Some(Kind::PointOnArc(host, _)) => {
                let host = *host;
                let Some(t) = self.arc_parameter(host, p) else {
                    return false;
                };
                Kind::PointOnArc(host, t)
            }
            _ => return false,
        };
        self.objects[id - 1].kind = kind;
        self.dirty = true;
        true
    }
    pub fn constrained_point(&self, host: usize, p: Point) -> Option<Kind> {
        match self.objects.get(host.checked_sub(1)?)?.kind {
            Kind::Segment(_, _) => {
                let (a, b, _) = self.line_points(host)?;
                let dx = b.x - a.x;
                let dy = b.y - a.y;
                let length_sq = dx * dx + dy * dy;
                if length_sq < 1e-18 {
                    return None;
                }
                let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / length_sq;
                Some(Kind::PointOnSegment(host, t.clamp(0., 1.)))
            }
            Kind::Circle(_, _)
            | Kind::CircleBySegment(_, _)
            | Kind::ReflectedCircle(_, _)
            | Kind::ScaledCircle(_, _, _) => {
                let (center, _) = self.circle_points(host)?;
                let dx = p.x - center.x;
                let dy = p.y - center.y;
                let length = dx.hypot(dy);
                (length > 1e-9).then_some(Kind::PointOnCircle(host, dx / length, -dy / length))
            }
            Kind::ThreePointArc(_, _, _) | Kind::ArcOnCircle(_, _, _) => self
                .arc_parameter(host, p)
                .map(|t| Kind::PointOnArc(host, t)),
            _ => None,
        }
    }
    pub fn can_save(&self) -> Result<(), String> {
        if self.objects.len() < self.source_count {
            return Err("Removing saved objects is not yet supported by the GSP writer".into());
        }
        for (i, o) in self.objects.iter().enumerate() {
            if o.id != i + 1 {
                return Err("Invalid object ID".into());
            }
            if let Kind::Free(point) = o.kind
                && (!point.x.is_finite() || !point.y.is_finite())
            {
                return Err(format!("Non-finite coordinates in object {}", o.id));
            }
            if matches!(o.kind, Kind::PointOnSegment(_, t) | Kind::PointOnArc(_, t) if !t.is_finite() || !(0.0..=1.0).contains(&t))
                || matches!(o.kind, Kind::PointOnCircle(_, x, y) if !x.is_finite() || !y.is_finite() || (x*x+y*y-1.).abs() > 1e-6)
                || matches!(o.kind, Kind::PolarTranslatedPoint(_, a, u, d) if !a.is_finite() || !u.is_finite() || !d.is_finite() || u <= 0.)
                || matches!(o.kind, Kind::ScaledPoint(_, _, factor) | Kind::ScaledCircle(_, _, factor) if !factor.is_finite() || factor == 0.)
                || matches!(o.kind, Kind::RotatedPoint(_, _, angle) if !angle.is_finite())
            {
                return Err(format!("Invalid point constraint in object {}", o.id));
            }
            let refs = o.kind.dependencies();
            if refs.iter().any(|&r| r == 0 || r >= o.id) {
                return Err(format!("Invalid dependency in object {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::PointOnSegment(host, _) if !matches!(self.objects.get(host.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Segment(_, _))))
            {
                return Err(format!("Invalid segment host in point {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::PointOnCircle(host, _, _) if self.circle_points(host).is_none())
            {
                return Err(format!("Invalid circle host in point {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::PointOnArc(host, _) if self.arc_geometry(host).is_none())
            {
                return Err(format!("Invalid arc host in point {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::PolarTranslatedPoint(origin, _, _, _) if self.position(origin).is_none())
            {
                return Err(format!("Invalid source point in translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::ReflectedPoint(source, mirror)
                    if self.position(source).is_none() || self.line_points(mirror).is_none()
                    || !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(kind) if is_point_kind(kind)))
            {
                return Err(format!("Invalid point or mirror in reflection {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::ReflectedCircle(source, mirror)
                    if self.circle_points(source).is_none() || self.line_points(mirror).is_none()
                    || self.circle_points(o.id).is_none())
            {
                return Err(format!("Invalid circle or mirror in reflection {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::ScaledPoint(source, center, _)
                    if self.position(source).is_none() || self.position(center).is_none()
                    || !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(kind) if is_point_kind(kind))
                    || !matches!(self.objects.get(center.saturating_sub(1)).map(|o| &o.kind), Some(kind) if is_point_kind(kind)))
            {
                return Err(format!("Invalid point or center in dilation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RotatedPoint(source, center, _)
                    if self.position(source).is_none() || self.position(center).is_none()
                    || !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(kind) if is_point_kind(kind))
                    || !matches!(self.objects.get(center.saturating_sub(1)).map(|o| &o.kind), Some(kind) if is_point_kind(kind)))
            {
                return Err(format!("Invalid point or center in rotation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::ScaledCircle(source, center, _)
                    if self.circle_points(source).is_none() || self.position(center).is_none() || self.circle_points(o.id).is_none())
            {
                return Err(format!("Invalid circle or center in dilation {}", o.id));
            }
            if i >= self.source_count
                && matches!(
                    o.kind,
                    Kind::PolarTranslatedPoint(_, _, _, _)
                        | Kind::ReflectedPoint(_, _)
                        | Kind::ScaledPoint(_, _, _)
                        | Kind::RotatedPoint(_, _, _)
                )
                && !self
                    .position(o.id)
                    .is_some_and(|p| p.x.is_finite() && p.y.is_finite())
            {
                return Err(format!("Undefined transformed point {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::Segment(a,b)|Kind::Line(a,b)|Kind::Ray(a,b)|Kind::Circle(a,b)|Kind::Midpoint(a,b)
                if self.position(a).is_none()||self.position(b).is_none())
            {
                return Err(format!("Invalid point dependency in object {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::Parallel(_,line)|Kind::Perpendicular(_,line) if self.line_points(line).is_none())
            {
                return Err(format!("Invalid source line in object {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::Parallel(point,_)|Kind::Perpendicular(point,_) if self.position(point).is_none())
            {
                return Err(format!("Invalid anchor point in object {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::CircleBySegment(center,segment)
                    if self.position(center).is_none()
                    || !matches!(self.objects.get(segment.saturating_sub(1)).map(|o| &o.kind),Some(Kind::Segment(_, _))))
            {
                return Err(format!(
                    "Invalid center or radius segment in circle {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::Intersection(a,b) if self.line_points(a).is_none() || self.line_points(b).is_none())
            {
                return Err(format!("Invalid straight objects in intersection {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::Bisector(a,b,c)
                    if self.position(a).is_none() || self.position(b).is_none() || self.position(c).is_none())
            {
                return Err(format!("Invalid points in angle bisector {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::ThreePointArc(a,b,c)
                    if self.position(a).is_none() || self.position(b).is_none() || self.position(c).is_none())
            {
                return Err(format!("Invalid points in three-point arc {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::ArcOnCircle(circle,start,end)
                    if self.circle_points(circle).is_none()
                    || !matches!(self.objects.get(start.saturating_sub(1)).map(|o| &o.kind), Some(Kind::PointOnCircle(host, _, _)) if *host == circle)
                    || !matches!(self.objects.get(end.saturating_sub(1)).map(|o| &o.kind), Some(Kind::PointOnCircle(host, _, _)) if *host == circle))
            {
                return Err(format!("Invalid circle or endpoints in arc {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::LineCircleIntersection(a,b,variant)
                    if variant > 1 || self.line_points(a).is_none() || self.circle_points(b).is_none())
            {
                return Err(format!("Invalid line and circle in intersection {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind,Kind::CircleCircleIntersection(a,b,variant)
                    if variant > 1 || self.circle_points(a).is_none() || self.circle_points(b).is_none())
            {
                return Err(format!("Invalid circles in intersection {}", o.id));
            }
            if matches!(o.kind, Kind::Unsupported(_)) && i >= self.source_count {
                return Err("Unsupported new object".into());
            }
        }
        if self.objects.len() > self.source_count && !self.safe_structure {
            return Err("This sketch has unsupported object records. Adding objects could change their references; save refused.".into());
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.can_save()?;
        let rs = records(&self.original)?;
        let mut out = self.original.clone();
        for o in self.objects.iter().take(self.source_count) {
            if let Some(i) = o.header_offset {
                let mut class = u32_at(&out, i);
                if o.visible {
                    class &= !0x1_0000;
                } else {
                    class |= 0x1_0000;
                }
                out[i..i + 4].copy_from_slice(&class.to_le_bytes());
            }
            if let (Kind::Free(p), Some(i)) = (&o.kind, o.point_offset) {
                out[i..i + 8].copy_from_slice(&p.x.to_le_bytes());
                out[i + 8..i + 16].copy_from_slice(&p.y.to_le_bytes());
            }
            if let Some(i) = o.constraint_offset {
                match o.kind {
                    Kind::PointOnSegment(_, t) if out.get(i..i + 4) == Some(&[15, 0, 14, 0]) => {
                        out[i + 4..i + 12].copy_from_slice(&t.to_le_bytes());
                    }
                    Kind::PointOnArc(_, t) if out.get(i..i + 4) == Some(&[15, 0, 67, 0]) => {
                        out[i + 4..i + 12].copy_from_slice(&t.to_le_bytes());
                    }
                    Kind::PointOnCircle(_, x, y) if out.get(i..i + 4) == Some(&[15, 0, 28, 0]) => {
                        out[i + 4..i + 12].copy_from_slice(&x.to_le_bytes());
                        out[i + 12..i + 20].copy_from_slice(&y.to_le_bytes());
                    }
                    _ => {}
                }
            }
        }
        if self.objects.len() == self.source_count {
            return Ok(out);
        }
        let preamble = rs
            .iter()
            .find(|r| r.kind == 0x3e8 && r.data.len() >= 28)
            .ok_or("Missing document preamble")?;
        out[preamble.data.start + 24..preamble.data.start + 28]
            .copy_from_slice(&(self.objects.len() as u32).to_le_bytes());
        let trailer = rs
            .iter()
            .find(|r| r.kind == 0x3e9)
            .ok_or("Missing document trailer")?
            .start;
        let last_group_end = rs
            .iter()
            .filter(|r| r.kind == 0x7d7 && r.start < trailer)
            .map(|r| r.data.end)
            .max()
            .unwrap_or(0);
        let insert_at = rs
            .iter()
            .find(|r| r.kind == 0x2328 && r.start >= last_group_end && r.start < trailer)
            .map(|r| r.start)
            .unwrap_or(trailer);
        let mut added = Vec::new();
        for o in self.objects.iter().skip(self.source_count) {
            let (code, refs): (u16, Vec<usize>) = match o.kind {
                Kind::Free(_) => (0, vec![]),
                Kind::PointOnSegment(host, _)
                | Kind::PointOnCircle(host, _, _)
                | Kind::PointOnArc(host, _) => (15, vec![host]),
                Kind::PolarTranslatedPoint(origin, _, _, _) => (21, vec![origin]),
                Kind::ReflectedPoint(source, mirror) => (34, vec![source, mirror]),
                Kind::ReflectedCircle(source, mirror) => (34, vec![source, mirror]),
                Kind::ScaledPoint(source, center, _) | Kind::ScaledCircle(source, center, _) => {
                    (30, vec![source, center])
                }
                Kind::RotatedPoint(source, center, _) => (27, vec![source, center]),
                Kind::Midpoint(a, b) => {
                    let s = self
                        .objects
                        .iter()
                        .find(|s| matches!(s.kind,Kind::Segment(x,y) if (x==a&&y==b)||(x==b&&y==a)))
                        .ok_or("Midpoint requires an existing segment")?;
                    (1, vec![s.id])
                }
                Kind::Segment(a, b) => (2, vec![a, b]),
                Kind::Circle(a, b) => (3, vec![a, b]),
                Kind::CircleBySegment(a, b) => (4, vec![a, b]),
                Kind::Perpendicular(a, b) => (5, vec![a, b]),
                Kind::Parallel(a, b) => (6, vec![a, b]),
                Kind::Bisector(a, b, c) => (7, vec![a, b, c]),
                Kind::Intersection(a, b) => (9, vec![a, b]),
                Kind::LineCircleIntersection(a, b, variant) => {
                    (if variant == 0 { 12 } else { 11 }, vec![a, b])
                }
                Kind::CircleCircleIntersection(a, b, variant) => {
                    (if variant == 0 { 14 } else { 13 }, vec![a, b])
                }
                Kind::Line(a, b) => (63, vec![a, b]),
                Kind::Ray(a, b) => (64, vec![a, b]),
                Kind::ThreePointArc(a, b, c) => (81, vec![a, b, c]),
                Kind::ArcOnCircle(a, b, c) => (79, vec![a, b, c]),
                _ => return Err("Unsupported new object".into()),
            };
            let mut h = header(code);
            if matches!(
                o.kind,
                Kind::ReflectedCircle(_, _) | Kind::ScaledCircle(_, _, _)
            ) {
                h = header(3);
                h[0] = code as u8;
                h[18] = 0;
                h[19] = 0;
            }
            if !o.visible {
                h[2] = 1;
            }
            put_record(&mut added, 0x7d0, &h);
            if let Kind::Free(p) = o.kind {
                let mut pair = Vec::new();
                pair.extend_from_slice(&p.x.to_le_bytes());
                pair.extend_from_slice(&p.y.to_le_bytes());
                put_record(&mut added, 0x899, &pair);
            } else {
                let mut idx = Vec::new();
                idx.extend_from_slice(&(refs.len() as u32).to_le_bytes());
                for r in refs {
                    idx.extend_from_slice(&(r as u32).to_le_bytes());
                }
                put_record(&mut added, 0x7d2, &idx);
                match o.kind {
                    Kind::PointOnSegment(_, t) => {
                        let mut payload = vec![15, 0, 14, 0];
                        payload.extend_from_slice(&t.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::PointOnArc(_, t) => {
                        let mut payload = vec![15, 0, 67, 0];
                        payload.extend_from_slice(&t.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::PointOnCircle(_, x, y) => {
                        let mut payload = vec![15, 0, 28, 0];
                        payload.extend_from_slice(&x.to_le_bytes());
                        payload.extend_from_slice(&y.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::PolarTranslatedPoint(_, angle, units, distance) => {
                        let mut payload =
                            *include_bytes!("../assets/polar_translation_payload.bin");
                        payload[20..28].copy_from_slice(&angle.to_le_bytes());
                        payload[32..40].copy_from_slice(&units.to_le_bytes());
                        payload[40..48].copy_from_slice(&distance.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::ScaledPoint(_, _, factor) | Kind::ScaledCircle(_, _, factor) => {
                        let mut payload = vec![30, 0, 0, 0];
                        payload.extend_from_slice(&factor.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::RotatedPoint(_, _, angle) => {
                        let mut payload = *include_bytes!("../assets/rotation_payload.bin");
                        payload[4..12].copy_from_slice(&(-angle.to_radians().sin()).to_le_bytes());
                        payload[12..20].copy_from_slice(&angle.to_radians().cos().to_le_bytes());
                        payload[20..28].copy_from_slice(&angle.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    _ => {}
                }
            }
            put_record(&mut added, 0x7d7, &[]);
        }
        out.splice(insert_at..insert_at, added);
        records(&out)?;
        Ok(out)
    }
    pub fn save_to(&mut self, path: &Path) -> Result<(), String> {
        let data = self.encode()?;
        let reparsed = Self::from_bytes(&data, Some(path.to_path_buf()))?;
        if reparsed.objects.len() != self.objects.len()
            || reparsed
                .objects
                .iter()
                .zip(&self.objects)
                .any(|(saved, current)| {
                    saved.id != current.id
                        || saved.page != current.page
                        || saved.kind != current.kind
                        || saved.visible != current.visible
                })
        {
            return Err("Serialized sketch does not preserve the edited objects".into());
        }
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let stem = path.file_name().ok_or("Invalid path")?.to_string_lossy();
        let mut n = 0;
        let temp = loop {
            let p = parent.join(format!(".{stem}.{}.{}.tmp", std::process::id(), n));
            if !p.exists() {
                break p;
            }
            n += 1;
        };
        let result = (|| -> Result<(), String> {
            let mut f = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temp)
                .map_err(|e| e.to_string())?;
            f.write_all(&data).map_err(|e| e.to_string())?;
            f.sync_all().map_err(|e| e.to_string())?;
            fs::rename(&temp, path).map_err(|e| e.to_string())?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result?;
        for (object, saved) in self.objects.iter_mut().zip(&reparsed.objects) {
            object.point_offset = saved.point_offset;
            object.constraint_offset = saved.constraint_offset;
            object.header_offset = saved.header_offset;
        }
        self.original = data;
        self.source_count = self.objects.len();
        self.path = Some(path.to_path_buf());
        self.safe_structure = reparsed.safe_structure;
        self.partial_view = reparsed.partial_view;
        self.undo.clear();
        self.redo.clear();
        self.dirty = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixtures_preserve_bytes() {
        for name in [
            "point",
            "point_hidden",
            "points",
            "segment",
            "circle",
            "circle_center_radius",
            "middle_point",
            "line",
            "line_intersection",
            "line_circle_intersection",
            "circle_circle_intersection",
            "bisector",
            "three_point_arc",
            "ray",
            "parallel",
            "perp",
            "example_tools",
            "wine_moved_demo",
            "reflection",
            "wine_moved_reflection",
            "wine_moved_reflected_circle",
            "scale",
            "wine_moved_dilation",
            "rotation",
            "wine_moved_rotation",
        ] {
            let p = format!("tests/fixtures/{name}.gsp");
            let b = fs::read(p).unwrap();
            let d = Document::from_bytes(&b, None).unwrap();
            assert_eq!(d.encode().unwrap(), b);
        }
    }
    #[test]
    fn new_roundtrip_and_dependencies() {
        let mut d = Document::new();
        let a = d.add(Kind::Free(Point { x: 10., y: 20. }));
        let b = d.add(Kind::Free(Point { x: 30., y: 40. }));
        d.add(Kind::Segment(a, b));
        let m = d.add(Kind::Midpoint(a, b));
        assert_eq!(d.position(m), Some(Point { x: 20., y: 30. }));
        let reopened = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.position(m), d.position(m));
    }
    #[test]
    fn undo_redo() {
        let mut d = Document::new();
        d.add(Kind::Free(Point { x: 1., y: 2. }));
        d.undo();
        assert!(d.objects.is_empty());
        d.redo();
        assert_eq!(d.objects.len(), 1);
    }
    #[test]
    fn invalid_ref() {
        let mut d = Document::new();
        d.add(Kind::Segment(1, 2));
        assert!(d.encode().is_err());
    }
    #[test]
    fn hidden_point_matches_original_sketchpad_bytes() {
        let mut d = Document::open(Path::new("tests/fixtures/point.gsp")).unwrap();
        d.selected.insert(1);
        d.set_selected_visibility(false);
        assert_eq!(
            d.encode().unwrap(),
            fs::read("tests/fixtures/point_hidden.gsp").unwrap()
        );
        d.undo();
        assert!(d.objects[0].visible);
    }
    #[test]
    fn pages_and_selection_metadata() {
        let tools = Document::open(Path::new("tests/fixtures/example_tools.gsp")).unwrap();
        assert_eq!(tools.page_count, 10);
        assert!(tools.objects.iter().any(|o| o.page == 9));
        assert!(tools.partial_view);
        let mut segment = Document::open(Path::new("tests/fixtures/segment.gsp")).unwrap();
        segment.add(Kind::Free(Point { x: 1., y: 2. }));
        let bytes = segment.encode().unwrap();
        let rs = records(&bytes).unwrap();
        let last_group = rs.iter().rposition(|r| r.kind == 0x7d7).unwrap();
        let selected = rs.iter().position(|r| r.kind == 0x2328).unwrap();
        assert!(last_group < selected);
        assert_eq!(Document::from_bytes(&bytes, None).unwrap().objects.len(), 4);
    }
    #[test]
    fn derived_lines_update() {
        let mut d = Document::new();
        let a = d.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = d.add(Kind::Free(Point { x: 10., y: 0. }));
        let c = d.add(Kind::Free(Point { x: 1., y: 5. }));
        let line = d.add(Kind::Line(a, b));
        let parallel = d.add(Kind::Parallel(c, line));
        let perpendicular = d.add(Kind::Perpendicular(c, line));
        assert_eq!(
            d.line_points(parallel),
            Some((
                Point { x: 1., y: 5. },
                Point { x: 11., y: 5. },
                LineMode::Line
            ))
        );
        assert_eq!(
            d.line_points(perpendicular),
            Some((
                Point { x: 1., y: 5. },
                Point { x: 1., y: 15. },
                LineMode::Line
            ))
        );
        d.move_point(b, Point { x: 0., y: 10. });
        assert_eq!(d.line_points(parallel).unwrap().1, Point { x: 1., y: 15. });
        let reopened = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.line_points(parallel), d.line_points(parallel));
    }
    #[test]
    fn save_reopen_edit_and_save_again() {
        let p = std::env::temp_dir().join(format!("sketchpad_rs_test_{}.gsp", std::process::id()));
        let mut d = Document::new();
        let id = d.add(Kind::Free(Point { x: 12., y: 34. }));
        d.save_to(&p).unwrap();
        assert!(d.move_point(id, Point { x: 56., y: 78. }));
        d.save_to(&p).unwrap();
        let r = Document::open(&p).unwrap();
        assert_eq!(r.position(id), Some(Point { x: 56., y: 78. }));
        fs::remove_file(p).unwrap();
    }
    #[test]
    fn constrained_point_can_move_after_first_save() {
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_constraint_save_{}.gsp",
            std::process::id()
        ));
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = doc.add(Kind::Free(Point { x: 100., y: 0. }));
        let segment = doc.add(Kind::Segment(a, b));
        let constrained = doc.add(Kind::PointOnSegment(segment, 0.25));
        doc.save_to(&path).unwrap();
        assert!(doc.move_point(constrained, Point { x: 75., y: 10. }));
        doc.save_to(&path).unwrap();
        let reopened = Document::open(&path).unwrap();
        assert_eq!(
            reopened.position(constrained),
            Some(Point { x: 75., y: 0. })
        );
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn invalid_coordinates_do_not_replace_a_file() {
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_invalid_save_{}.gsp",
            std::process::id()
        ));
        fs::write(&path, b"existing content").unwrap();
        let mut document = Document::new();
        document.add(Kind::Free(Point { x: f64::NAN, y: 1. }));
        assert!(document.save_to(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"existing content");
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn sketchpad_saved_edit_updates_dependents() {
        let document = Document::open(Path::new("tests/fixtures/wine_moved_demo.gsp")).unwrap();
        assert_eq!(document.objects.len(), 9);
        assert_eq!(document.position(1), Some(Point { x: 261., y: 181. }));
        assert_eq!(document.position(5), Some(Point { x: 370.5, y: 270.5 }));
        assert_eq!(
            document.line_points(8).unwrap().0,
            document.position(1).unwrap()
        );
    }
    #[test]
    fn unknown_page_record_blocks_structural_save() {
        let mut bytes = fs::read("tests/fixtures/point.gsp").unwrap();
        let trailer = records(&bytes)
            .unwrap()
            .iter()
            .find(|record| record.kind == 0x3e9)
            .unwrap()
            .start;
        let mut unknown = Vec::new();
        put_record(&mut unknown, 0x4444, &[1, 2, 3, 4]);
        bytes.splice(trailer..trailer, unknown);
        let mut document = Document::from_bytes(&bytes, None).unwrap();
        assert!(document.partial_view);
        assert_eq!(document.encode().unwrap(), bytes);
        document.add(Kind::Free(Point { x: 4., y: 5. }));
        assert!(document.encode().is_err());
    }
    #[test]
    fn forward_reference_is_preserved_without_evaluation() {
        let mut source = Document::new();
        let a = source.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = source.add(Kind::Free(Point { x: 10., y: 0. }));
        source.add(Kind::Segment(a, b));
        let mut bytes = source.encode().unwrap();
        let references = records(&bytes)
            .unwrap()
            .into_iter()
            .find(|record| record.kind == 0x7d2)
            .unwrap();
        bytes[references.data.start + 8..references.data.start + 12]
            .copy_from_slice(&3_u32.to_le_bytes());
        let document = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(document.objects[2].kind, Kind::Unsupported(2));
        assert!(document.partial_view);
        assert!(document.line_points(3).is_none());
        assert_eq!(document.encode().unwrap(), bytes);
    }
    #[test]
    fn straight_intersections_update_and_reopen() {
        let source = Document::open(Path::new("tests/fixtures/line_intersection.gsp")).unwrap();
        assert_eq!(source.objects[6].kind, Kind::Intersection(3, 6));
        assert!(source.position(7).is_some());
        assert!(!source.partial_view);

        let mut document = Document::new();
        let a = document.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = document.add(Kind::Free(Point { x: 10., y: 0. }));
        let c = document.add(Kind::Free(Point { x: 5., y: -5. }));
        let d = document.add(Kind::Free(Point { x: 5., y: 5. }));
        let horizontal = document.add(Kind::Segment(a, b));
        let vertical = document.add(Kind::Segment(c, d));
        let crossing = document.add(Kind::Intersection(horizontal, vertical));
        assert_eq!(document.position(crossing), Some(Point { x: 5., y: 0. }));
        let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.position(crossing), document.position(crossing));
        document.move_point(d, Point { x: 5., y: -1. });
        assert_eq!(document.position(crossing), None);
    }
    #[test]
    fn straight_intersection_handles_parallel_and_rays() {
        let a = Point { x: 0., y: 0. };
        let b = Point { x: 1., y: 0. };
        let c = Point { x: 2., y: -1. };
        let d = Point { x: 2., y: 1. };
        assert_eq!(
            intersect_straight((a, b, LineMode::Segment), (c, d, LineMode::Line)),
            None
        );
        assert_eq!(
            intersect_straight((a, b, LineMode::Ray), (c, d, LineMode::Line)),
            Some(Point { x: 2., y: 0. })
        );
        assert_eq!(
            intersect_straight(
                (a, b, LineMode::Line),
                (
                    Point { x: 0., y: 1. },
                    Point { x: 1., y: 1. },
                    LineMode::Line
                )
            ),
            None
        );
    }
    #[test]
    fn circle_intersections_update_and_reopen() {
        let original =
            Document::open(Path::new("tests/fixtures/circle_circle_intersection.gsp")).unwrap();
        assert_eq!(
            original.objects[6].kind,
            Kind::CircleCircleIntersection(6, 5, 0)
        );
        assert_eq!(
            original.objects[7].kind,
            Kind::CircleCircleIntersection(6, 5, 1)
        );
        assert!(original.position(7).is_some());
        assert!(original.position(8).is_some());
        let original_line_circle =
            Document::open(Path::new("tests/fixtures/line_circle_intersection.gsp")).unwrap();
        assert_eq!(
            original_line_circle.objects[6].kind,
            Kind::LineCircleIntersection(3, 6, 0)
        );
        assert!(original_line_circle.position(7).is_some());

        let mut document = Document::new();
        let left = document.add(Kind::Free(Point { x: -10., y: 0. }));
        let right = document.add(Kind::Free(Point { x: 10., y: 0. }));
        let origin = document.add(Kind::Free(Point { x: 0., y: 0. }));
        let radius = document.add(Kind::Free(Point { x: 5., y: 0. }));
        let other_center = document.add(Kind::Free(Point { x: 4., y: 0. }));
        let other_radius = document.add(Kind::Free(Point { x: 9., y: 0. }));
        let line = document.add(Kind::Segment(left, right));
        let first_circle = document.add(Kind::Circle(origin, radius));
        let second_circle = document.add(Kind::Circle(other_center, other_radius));
        let near = document.add(Kind::LineCircleIntersection(line, first_circle, 0));
        let far = document.add(Kind::LineCircleIntersection(line, first_circle, 1));
        let upper = document.add(Kind::CircleCircleIntersection(
            first_circle,
            second_circle,
            0,
        ));
        let lower = document.add(Kind::CircleCircleIntersection(
            first_circle,
            second_circle,
            1,
        ));
        assert_eq!(document.position(near), Some(Point { x: -5., y: 0. }));
        assert_eq!(document.position(far), Some(Point { x: 5., y: 0. }));
        assert!(document.position(upper).unwrap().y < 0.);
        assert!(document.position(lower).unwrap().y > 0.);
        let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
        for id in [near, far, upper, lower] {
            assert_eq!(reopened.position(id), document.position(id));
        }
        document.move_point(other_center, Point { x: 30., y: 0. });
        assert!(document.position(upper).is_none());
        assert!(document.position(lower).is_none());
    }
    #[test]
    fn line_circle_second_root_uses_remaining_segment_crossing() {
        let mut document = Document::new();
        let center = document.add(Kind::Free(Point { x: 0., y: 0. }));
        let end = document.add(Kind::Free(Point { x: 10., y: 0. }));
        let radius = document.add(Kind::Free(Point { x: 0., y: 5. }));
        let segment = document.add(Kind::Segment(center, end));
        let circle = document.add(Kind::Circle(center, radius));
        let second = document.add(Kind::LineCircleIntersection(segment, circle, 1));
        assert_eq!(document.position(second), Some(Point { x: 5., y: 0. }));
        let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.position(second), document.position(second));
    }
    #[test]
    fn line_circle_record_requires_straight_object_first() {
        let mut document = Document::new();
        let center = document.add(Kind::Free(Point { x: 0., y: 0. }));
        let radius = document.add(Kind::Free(Point { x: 0., y: 5. }));
        let circle = document.add(Kind::Circle(center, radius));
        let end = document.add(Kind::Free(Point { x: 10., y: 0. }));
        let segment = document.add(Kind::Segment(center, end));
        let intersection = document.add(Kind::LineCircleIntersection(segment, circle, 0));
        assert_eq!(
            document.position(intersection),
            Some(Point { x: 5., y: 0. })
        );
        let encoded = document.encode().unwrap();
        assert_eq!(
            Document::from_bytes(&encoded, None)
                .unwrap()
                .position(intersection),
            document.position(intersection)
        );
        document.objects[intersection - 1].kind = Kind::LineCircleIntersection(circle, segment, 0);
        assert!(document.encode().is_err());
    }
    #[test]
    fn angle_bisector_matches_original_record_and_updates() {
        let original = Document::open(Path::new("tests/fixtures/bisector.gsp")).unwrap();
        assert_eq!(original.objects[3].kind, Kind::Bisector(1, 2, 3));
        assert!(!original.partial_view);
        assert_eq!(original.line_points(4).unwrap().2, LineMode::Ray);
        let bytes = fs::read("tests/fixtures/bisector.gsp").unwrap();
        let object_headers: Vec<_> = records(&bytes)
            .unwrap()
            .into_iter()
            .filter(|record| record.kind == 0x7d0)
            .collect();
        assert_eq!(&bytes[object_headers[3].data.clone()], &header(7));

        let mut document = Document::new();
        let a = document.add(Kind::Free(Point { x: -1., y: 1. }));
        let vertex = document.add(Kind::Free(Point { x: 0., y: 0. }));
        let c = document.add(Kind::Free(Point { x: 1., y: 1. }));
        let id = document.add(Kind::Bisector(a, vertex, c));
        assert_eq!(
            document.line_points(id),
            Some((
                Point { x: 0., y: 0. },
                Point { x: 0., y: 1. },
                LineMode::Ray
            ))
        );
        let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.line_points(id), document.line_points(id));
        document.move_point(vertex, Point { x: 0., y: 2. });
        assert_ne!(document.line_points(id), reopened.line_points(id));
    }
    #[test]
    fn three_point_arc_matches_original_record_and_updates() {
        let original = Document::open(Path::new("tests/fixtures/three_point_arc.gsp")).unwrap();
        assert_eq!(original.objects[3].kind, Kind::ThreePointArc(1, 2, 3));
        assert!(!original.partial_view);
        assert!(original.arc_points(4, 32).is_some());
        let bytes = fs::read("tests/fixtures/three_point_arc.gsp").unwrap();
        let object_headers: Vec<_> = records(&bytes)
            .unwrap()
            .into_iter()
            .filter(|record| record.kind == 0x7d0)
            .collect();
        assert_eq!(&bytes[object_headers[3].data.clone()], &header(81));

        let mut document = Document::new();
        let a = document.add(Kind::Free(Point { x: -1., y: 0. }));
        let mid = document.add(Kind::Free(Point { x: 0., y: -1. }));
        let end = document.add(Kind::Free(Point { x: 1., y: 0. }));
        let id = document.add(Kind::ThreePointArc(a, mid, end));
        let points = document.arc_points(id, 32).unwrap();
        assert!(points.first().unwrap().distance(Point { x: -1., y: 0. }) < 1e-9);
        assert!(points[16].distance(Point { x: 0., y: -1. }) < 1e-9);
        assert!(points.last().unwrap().distance(Point { x: 1., y: 0. }) < 1e-9);
        let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.arc_points(id, 32), document.arc_points(id, 32));
        document.move_point(mid, Point { x: 0., y: -2. });
        assert_ne!(reopened.arc_points(id, 32), document.arc_points(id, 32));
        document.move_point(mid, Point { x: 0., y: 0. });
        assert!(document.arc_points(id, 32).is_none());
    }
    #[test]
    fn circle_by_segment_matches_original_and_updates() {
        let original =
            Document::open(Path::new("tests/fixtures/circle_center_radius.gsp")).unwrap();
        assert_eq!(original.objects[4].kind, Kind::CircleBySegment(4, 3));
        assert!(!original.partial_view);
        assert!(original.circle_points(5).is_some());
        let bytes = fs::read("tests/fixtures/circle_center_radius.gsp").unwrap();
        let object_headers: Vec<_> = records(&bytes)
            .unwrap()
            .into_iter()
            .filter(|record| record.kind == 0x7d0)
            .collect();
        assert_eq!(&bytes[object_headers[4].data.clone()], &header(4));

        let mut document = Document::new();
        let a = document.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = document.add(Kind::Free(Point { x: 5., y: 0. }));
        let center = document.add(Kind::Free(Point { x: 10., y: 10. }));
        let segment = document.add(Kind::Segment(a, b));
        let circle = document.add(Kind::CircleBySegment(center, segment));
        assert_eq!(
            document.circle_points(circle),
            Some((Point { x: 10., y: 10. }, 5.))
        );
        let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
        assert_eq!(
            reopened.circle_points(circle),
            document.circle_points(circle)
        );
        document.move_point(b, Point { x: 7., y: 0. });
        assert_eq!(
            document.circle_points(circle),
            Some((Point { x: 10., y: 10. }, 7.))
        );
    }
    #[test]
    fn constrained_points_match_original_and_reopen_after_drag() {
        let segment_fixture = fs::read("tests/fixtures/point_on_segment.gsp").unwrap();
        let mut segment_doc = Document::from_bytes(&segment_fixture, None).unwrap();
        assert!(matches!(
            segment_doc.objects[3].kind,
            Kind::PointOnSegment(3, _)
        ));
        assert_eq!(segment_doc.encode().unwrap(), segment_fixture);
        let before = segment_doc.position(4).unwrap();
        segment_doc.move_point(1, Point { x: 270., y: 180. });
        assert_ne!(segment_doc.position(4), Some(before));
        assert!(segment_doc.move_point(4, Point { x: 300., y: 200. }));
        let reparsed = Document::from_bytes(&segment_doc.encode().unwrap(), None).unwrap();
        assert_eq!(reparsed.position(4), segment_doc.position(4));

        let circle_fixture = fs::read("tests/fixtures/point_on_circle.gsp").unwrap();
        let mut circle_doc = Document::from_bytes(&circle_fixture, None).unwrap();
        assert!(matches!(
            circle_doc.objects[3].kind,
            Kind::PointOnCircle(3, _, _)
        ));
        assert_eq!(circle_doc.encode().unwrap(), circle_fixture);
        let (center, radius) = circle_doc.circle_points(3).unwrap();
        assert!((circle_doc.position(4).unwrap().distance(center) - radius).abs() < 1e-6);
        assert!(circle_doc.move_point(
            4,
            Point {
                x: center.x,
                y: center.y + radius
            }
        ));
        let reparsed = Document::from_bytes(&circle_doc.encode().unwrap(), None).unwrap();
        assert_eq!(reparsed.position(4), circle_doc.position(4));

        let mut created = Document::new();
        let a = created.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = created.add(Kind::Free(Point { x: 10., y: 0. }));
        let segment = created.add(Kind::Segment(a, b));
        let on_segment = created.add(
            created
                .constrained_point(segment, Point { x: 4., y: 2. })
                .unwrap(),
        );
        let circle = created.add(Kind::Circle(a, b));
        let on_circle = created.add(
            created
                .constrained_point(circle, Point { x: 0., y: 9. })
                .unwrap(),
        );
        assert_eq!(
            created.objects[on_circle - 1].kind,
            Kind::PointOnCircle(circle, 0., -1.)
        );
        assert_eq!(created.position(on_segment), Some(Point { x: 4., y: 0. }));
        assert_eq!(created.position(on_circle), Some(Point { x: 0., y: 10. }));
        let reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.position(on_segment), created.position(on_segment));
        assert_eq!(reopened.position(on_circle), created.position(on_circle));
    }
    #[test]
    fn arc_on_circle_matches_original_and_updates() {
        let bytes = fs::read("tests/fixtures/arc_on_circle.gsp").unwrap();
        let mut original = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(original.objects[5].kind, Kind::ArcOnCircle(3, 4, 5));
        assert_eq!(original.encode().unwrap(), bytes);
        let points = original.arc_points(6, 48).unwrap();
        assert!((points[0].distance(original.position(4).unwrap())) < 1e-6);
        assert!((points[48].distance(original.position(5).unwrap())) < 1e-6);
        original.move_point(4, Point { x: 300., y: 300. });
        assert_ne!(original.arc_points(6, 48).unwrap(), points);

        let mut created = Document::new();
        let center = created.add(Kind::Free(Point { x: 100., y: 100. }));
        let radius = created.add(Kind::Free(Point { x: 140., y: 100. }));
        let circle = created.add(Kind::Circle(center, radius));
        let start = created.add(Kind::PointOnCircle(circle, 1., 0.));
        let end = created.add(Kind::PointOnCircle(circle, 0., 1.));
        let arc = created.add(Kind::ArcOnCircle(circle, start, end));
        let generated = created.encode().unwrap();
        let reopened = Document::from_bytes(&generated, None).unwrap();
        assert_eq!(
            reopened.objects[arc - 1].kind,
            Kind::ArcOnCircle(circle, start, end)
        );
        assert_eq!(reopened.arc_points(arc, 48), created.arc_points(arc, 48));
        assert!(!reopened.partial_view);
    }
    #[test]
    fn point_on_arc_matches_original_and_updates() {
        let bytes = fs::read("tests/fixtures/point_on_arc.gsp").unwrap();
        let mut original = Document::from_bytes(&bytes, None).unwrap();
        assert!(matches!(original.objects[4].kind, Kind::PointOnArc(4, _)));
        assert_eq!(original.encode().unwrap(), bytes);
        assert!(original.position(5).is_some());
        original.move_point(1, Point { x: 330., y: 220. });
        let before_drag = original.position(5).unwrap();
        assert!(original.move_point(5, Point { x: 300., y: 160. }));
        assert_ne!(original.position(5), Some(before_drag));
        let reopened = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.position(5), original.position(5));

        let mut created = Document::new();
        let a = created.add(Kind::Free(Point { x: 100., y: 200. }));
        let b = created.add(Kind::Free(Point { x: 150., y: 150. }));
        let c = created.add(Kind::Free(Point { x: 200., y: 200. }));
        let arc = created.add(Kind::ThreePointArc(a, b, c));
        let p = created.add(Kind::PointOnArc(arc, 0.5));
        let reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.position(p), created.position(p));
        assert_eq!(reopened.objects[p - 1].kind, Kind::PointOnArc(arc, 0.5));
    }
    #[test]
    fn polar_translation_matches_original_and_updates() {
        let bytes = fs::read("tests/fixtures/point_translation.gsp").unwrap();
        let mut original = Document::from_bytes(&bytes, None).unwrap();
        assert!(matches!(
            original.objects[1].kind,
            Kind::PolarTranslatedPoint(1, 90., _, 1.)
        ));
        assert!(!original.partial_view);
        assert_eq!(original.encode().unwrap(), bytes);
        let source = original.position(1).unwrap();
        let translated = original.position(2).unwrap();
        assert!((translated.x - source.x).abs() < 1e-6);
        assert!((translated.y - source.y + 37.79527559055118).abs() < 1e-6);
        original.move_point(1, Point { x: 100., y: 200. });
        assert!((original.position(2).unwrap().y - 162.20472440944883).abs() < 1e-6);

        let mut created = Document::new();
        let a = created.add(Kind::Free(Point { x: 200., y: 200. }));
        let translated = created.add(Kind::PolarTranslatedPoint(a, 90., 37.79527559055118, 2.));
        let reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert_eq!(
            reopened.objects[translated - 1].kind,
            created.objects[translated - 1].kind
        );
        assert_eq!(reopened.position(translated), created.position(translated));
        assert!(!reopened.partial_view);

        let moved = Document::open(Path::new("tests/fixtures/wine_moved_translation.gsp")).unwrap();
        assert_eq!(moved.position(1), Some(Point { x: 241., y: 221. }));
        assert!(matches!(
            moved.objects[1].kind,
            Kind::PolarTranslatedPoint(1, 90., _, 1.)
        ));
        assert!((moved.position(2).unwrap().y - 183.20472440944883).abs() < 1e-6);
    }
    #[test]
    fn reflected_point_matches_original_and_updates() {
        let bytes = fs::read("tests/fixtures/reflection.gsp").unwrap();
        let mut original = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(original.objects[13].kind, Kind::ReflectedPoint(1, 4));
        assert_eq!(original.objects[14].kind, Kind::ReflectedCircle(12, 4));
        assert_eq!(original.encode().unwrap(), bytes);
        let (a, b, _) = original.line_points(4).unwrap();
        assert_eq!(
            original.position(14),
            reflect_point(original.position(1).unwrap(), a, b)
        );
        let prior = original.position(14);
        original.move_point(1, Point { x: 300., y: 150. });
        assert_ne!(original.position(14), prior);
        let reopened = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.position(14), original.position(14));
        assert_eq!(reopened.circle_points(15), original.circle_points(15));

        let mut created = Document::new();
        let source = created.add(Kind::Free(Point { x: 50., y: 50. }));
        let a = created.add(Kind::Free(Point { x: 0., y: 100. }));
        let b = created.add(Kind::Free(Point { x: 200., y: 100. }));
        let mirror = created.add(Kind::Segment(a, b));
        let result = created.add(Kind::ReflectedPoint(source, mirror));
        assert_eq!(created.position(result), Some(Point { x: 50., y: 150. }));
        let encoded = created.encode().unwrap();
        let headers: Vec<_> = records(&encoded)
            .unwrap()
            .into_iter()
            .filter(|r| r.kind == 0x7d0)
            .collect();
        assert_eq!(&encoded[headers[4].data.clone()], &header(34));
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.objects[4].kind,
            Kind::ReflectedPoint(source, mirror)
        );
        assert_eq!(reopened.position(result), created.position(result));

        let radius_point = created.add(Kind::Free(Point { x: 70., y: 50. }));
        let circle = created.add(Kind::Circle(source, radius_point));
        let reflected_circle = created.add(Kind::ReflectedCircle(circle, mirror));
        assert_eq!(
            created.circle_points(reflected_circle),
            Some((Point { x: 50., y: 150. }, 20.))
        );
        let encoded = created.encode().unwrap();
        let headers: Vec<_> = records(&encoded)
            .unwrap()
            .into_iter()
            .filter(|r| r.kind == 0x7d0)
            .collect();
        assert_eq!(
            &encoded[headers[7].data.clone()],
            &[
                34, 0, 0, 0, 0, 0, 3, 0, 28, 0, 34, 1, 0, 128, 0, 0, 255, 255, 0, 0, 193, 189, 240,
                255, 0, 0, 0, 0
            ]
        );
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.circle_points(reflected_circle),
            created.circle_points(reflected_circle)
        );

        let wine = Document::open(Path::new("tests/fixtures/wine_moved_reflection.gsp")).unwrap();
        assert_eq!(wine.objects[4].kind, Kind::ReflectedPoint(1, 4));
        assert_eq!(wine.position(1), Some(Point { x: 241., y: 130. }));
        assert_eq!(wine.position(5), Some(Point { x: 241., y: 210. }));

        let wine_circle =
            Document::open(Path::new("tests/fixtures/wine_moved_reflected_circle.gsp")).unwrap();
        assert_eq!(wine_circle.objects[8].kind, Kind::ReflectedCircle(8, 4));
        let (center, radius) = wine_circle.circle_points(9).unwrap();
        assert_eq!(center, Point { x: 271., y: 139. });
        assert!((radius - 52.55473337388365).abs() < 1e-6);
    }
    #[test]
    fn dilation_matches_original_and_updates() {
        let bytes = fs::read("tests/fixtures/scale.gsp").unwrap();
        let mut original = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(original.objects[3].kind, Kind::ScaledPoint(1, 3, 1. / 3.));
        assert_eq!(original.objects[12].kind, Kind::ScaledCircle(7, 1, 1. / 3.));
        assert_eq!(original.encode().unwrap(), bytes);
        let before = original.position(4);
        original.move_point(1, Point { x: 330., y: 237. });
        assert_ne!(original.position(4), before);
        assert_eq!(
            original.circle_points(13).unwrap().0,
            Point { x: 245., y: 317. }
        );

        let mut created = Document::new();
        let center = created.add(Kind::Free(Point { x: 100., y: 100. }));
        let source = created.add(Kind::Free(Point { x: 160., y: 100. }));
        let scaled = created.add(Kind::ScaledPoint(source, center, 0.5));
        let radius = created.add(Kind::Free(Point { x: 180., y: 100. }));
        let circle = created.add(Kind::Circle(source, radius));
        let scaled_circle = created.add(Kind::ScaledCircle(circle, center, 0.5));
        assert_eq!(created.position(scaled), Some(Point { x: 130., y: 100. }));
        assert_eq!(
            created.circle_points(scaled_circle),
            Some((Point { x: 130., y: 100. }, 10.))
        );
        let reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(scaled), created.position(scaled));
        assert_eq!(
            reopened.circle_points(scaled_circle),
            created.circle_points(scaled_circle)
        );

        let wine = Document::open(Path::new("tests/fixtures/wine_moved_dilation.gsp")).unwrap();
        assert_eq!(wine.position(1), Some(Point { x: 171., y: 131. }));
        assert_eq!(wine.position(3), Some(Point { x: 245.5, y: 115.5 }));
        assert_eq!(
            wine.circle_points(7),
            Some((Point { x: 235.5, y: 185.5 }, 15.))
        );
    }
    #[test]
    fn rotation_matches_original_and_updates() {
        let bytes = fs::read("tests/fixtures/rotation.gsp").unwrap();
        let mut original = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(original.objects[4].kind, Kind::RotatedPoint(2, 1, 60.));
        assert_eq!(original.encode().unwrap(), bytes);
        let rotated = original.position(5).unwrap();
        assert!((rotated.x - 290.5).abs() < 1e-6);
        assert!((rotated.y - (188. - 157. * 60f64.to_radians().sin())).abs() < 1e-6);
        original.move_point(2, Point { x: 300., y: 188. });
        assert_ne!(original.position(5), Some(rotated));

        let mut created = Document::new();
        let center = created.add(Kind::Free(Point { x: 200., y: 180. }));
        let source = created.add(Kind::Free(Point { x: 300., y: 180. }));
        let result = created.add(Kind::RotatedPoint(source, center, 90.));
        let reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.objects[2].kind,
            Kind::RotatedPoint(source, center, 90.)
        );
        assert!((reopened.position(result).unwrap().x - 200.).abs() < 1e-6);
        assert!((reopened.position(result).unwrap().y - 80.).abs() < 1e-6);

        let wine = Document::open(Path::new("tests/fixtures/wine_moved_rotation.gsp")).unwrap();
        assert_eq!(wine.objects[2].kind, Kind::RotatedPoint(2, 1, 90.));
        assert_eq!(wine.position(2), Some(Point { x: 271., y: 160. }));
        let rotated = wine.position(3).unwrap();
        assert!((rotated.x - 180.).abs() < 1e-6);
        assert!((rotated.y - 109.).abs() < 1e-6);
    }
}
