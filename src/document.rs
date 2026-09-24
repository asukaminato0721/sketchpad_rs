use crate::action_records::{HideShowButtonGroup, VisibilityAction};
use crate::coordinate_measure_records::{
    self, CoordinateBasis, CoordinateMeasureKind, CoordinateMeasureValue, ScreenPoint,
};
use crate::function_records::{self, SimpleFunction};
use crate::graph::{self, CoordinateFrame, GraphObject, GraphPoint, GridForm};
use crate::image_records::ImageFile;
use crate::plot_axis_records;
use crate::plot_points_records::{self, PlotPointsMode};
use crate::style_records::{
    self, LinePattern, LineWidth, PointStyle, RgbColor, SegmentLineStyle, VisualStyle,
};
use crate::table_records::{CalculationGroup, ParameterGroup, TableGroup};
use crate::text_object::{self, TextRect};
use crate::text_records::LabelAux;
use crate::value_equation_records::{self, CartesianFrame, EquationValue, ValueEquationKind};
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
    fn is_measurement(&self) -> bool {
        matches!(
            self,
            Kind::SegmentLength(_)
                | Kind::PointDistance(_, _)
                | Kind::PointRatio(_, _, _)
                | Kind::PolygonPerimeter(_)
                | Kind::CircleCircumference(_)
                | Kind::CircleRadius(_)
                | Kind::CircleArea(_)
                | Kind::AngleMeasurement(_, _, _)
                | Kind::SlopeMeasurement(_, _)
                | Kind::CoordinateMeasurement { .. }
                | Kind::ValueEquation { .. }
                | Kind::ArcAngle(_)
                | Kind::ArcLength(_)
        )
    }
    pub fn dependencies(&self) -> Vec<usize> {
        if let Kind::HideShowButton { target, .. } = self {
            return vec![*target];
        }
        if let Kind::Polygon(vertices) = self {
            return vertices.clone();
        }
        if let Kind::CoordinateMeasurement {
            first,
            second,
            frame,
            ..
        } = self
        {
            let mut ids = vec![*first];
            if let Some(second) = second {
                ids.push(*second);
            }
            ids.push(*frame);
            return ids;
        }
        if let Kind::ValueEquation { first, second, .. } = self {
            return vec![*first, *second];
        }
        if let Kind::ValueTablePair(first, second, _) = self {
            return vec![*first, *second];
        }
        match *self {
            Kind::Free(_)
            | Kind::Image { .. }
            | Kind::PlainText { .. }
            | Kind::Parameter(_)
            | Kind::Calculation(_)
            | Kind::FunctionDefinition(_)
            | Kind::Unsupported(_) => vec![],
            Kind::PointOnSegment(host, _)
            | Kind::ValueTable(host, _)
            | Kind::PointOnCircle(host, _, _)
            | Kind::PointOnArc(host, _)
            | Kind::PolarTranslatedPoint(host, _, _, _)
            | Kind::PathLiteralPoint(host, _)
            | Kind::SegmentLength(host)
            | Kind::PolygonPerimeter(host)
            | Kind::CircleCircumference(host)
            | Kind::CircleRadius(host)
            | Kind::CircleArea(host)
            | Kind::ArcAngle(host)
            | Kind::ArcLength(host) => vec![host],
            Kind::GraphHorizontalUnit(origin, _) | Kind::GraphVerticalUnit(origin, _) => {
                vec![origin]
            }
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
            | Kind::ReflectedCircle(a, b)
            | Kind::PointDistance(a, b) => vec![a, b],
            Kind::GraphAxis(a, b, _)
            | Kind::CoordinateSystem(a, b)
            | Kind::SlopeMeasurement(a, b)
            | Kind::FunctionPlot(a, b, _, _, _)
            | Kind::PathValuePoint(a, b)
            | Kind::PlottedPoint(a, b, _, _) => vec![a, b],
            Kind::ScaledPoint(source, center, _)
            | Kind::ScaledCircle(source, center, _)
            | Kind::RotatedPoint(source, center, _) => {
                vec![source, center]
            }
            Kind::Bisector(a, b, c)
            | Kind::ThreePointArc(a, b, c)
            | Kind::ArcOnCircle(a, b, c)
            | Kind::AngleMeasurement(a, b, c) => {
                vec![a, b, c]
            }
            Kind::PointRatio(a, b, c) => vec![a, b, c],
            Kind::Polygon(_) => unreachable!(),
            Kind::HideShowButton { .. } => unreachable!(),
            Kind::CoordinateMeasurement { .. } => unreachable!(),
            Kind::ValueEquation { .. } => unreachable!(),
            Kind::ValueTablePair(_, _, _) => unreachable!(),
        }
    }
    fn remap(&self, ids: &[usize]) -> Self {
        let m = |id: usize| ids[id];
        if let Kind::HideShowButton {
            target,
            action,
            label,
            anchor,
        } = self
        {
            return Kind::HideShowButton {
                target: m(*target),
                action: *action,
                label: label.clone(),
                anchor: *anchor,
            };
        }
        if let Kind::Polygon(vertices) = self {
            return Kind::Polygon(vertices.iter().map(|&id| m(id)).collect());
        }
        if let Kind::CoordinateMeasurement {
            kind,
            first,
            second,
            frame,
        } = self
        {
            return Kind::CoordinateMeasurement {
                kind: *kind,
                first: m(*first),
                second: second.map(m),
                frame: m(*frame),
            };
        }
        if let Kind::ValueEquation {
            kind,
            first,
            second,
        } = self
        {
            return Kind::ValueEquation {
                kind: *kind,
                first: m(*first),
                second: m(*second),
            };
        }
        match *self {
            Kind::Free(p) => Kind::Free(p),
            Kind::Image { ref png, transform } => Kind::Image {
                png: png.clone(),
                transform,
            },
            Kind::PlainText { ref text, rect } => Kind::PlainText {
                text: text.clone(),
                rect,
            },
            Kind::Parameter(ref group) => Kind::Parameter(group.clone()),
            Kind::Calculation(ref group) => Kind::Calculation(group.clone()),
            Kind::ValueTable(source, ref group) => Kind::ValueTable(m(source), group.clone()),
            Kind::ValueTablePair(first, second, ref group) => {
                Kind::ValueTablePair(m(first), m(second), group.clone())
            }
            Kind::PointOnSegment(a, t) => Kind::PointOnSegment(m(a), t),
            Kind::PointOnCircle(a, x, y) => Kind::PointOnCircle(m(a), x, y),
            Kind::PointOnArc(a, t) => Kind::PointOnArc(m(a), t),
            Kind::GraphHorizontalUnit(a, step) => Kind::GraphHorizontalUnit(m(a), step),
            Kind::GraphVerticalUnit(a, step) => Kind::GraphVerticalUnit(m(a), step),
            Kind::GraphAxis(a, b, axis) => Kind::GraphAxis(m(a), m(b), axis),
            Kind::CoordinateSystem(a, b) => Kind::CoordinateSystem(m(a), m(b)),
            Kind::SlopeMeasurement(a, b) => Kind::SlopeMeasurement(m(a), m(b)),
            Kind::FunctionDefinition(expr) => Kind::FunctionDefinition(expr),
            Kind::FunctionPlot(a, b, min, max, samples) => {
                Kind::FunctionPlot(m(a), m(b), min, max, samples)
            }
            Kind::PathLiteralPoint(axis, value) => Kind::PathLiteralPoint(m(axis), value),
            Kind::PathValuePoint(a, b) => Kind::PathValuePoint(m(a), m(b)),
            Kind::PlottedPoint(a, b, mode, stored) => Kind::PlottedPoint(m(a), m(b), mode, stored),
            Kind::SegmentLength(a) => Kind::SegmentLength(m(a)),
            Kind::PolygonPerimeter(a) => Kind::PolygonPerimeter(m(a)),
            Kind::CircleCircumference(a) => Kind::CircleCircumference(m(a)),
            Kind::CircleRadius(a) => Kind::CircleRadius(m(a)),
            Kind::CircleArea(a) => Kind::CircleArea(m(a)),
            Kind::ArcAngle(a) => Kind::ArcAngle(m(a)),
            Kind::ArcLength(a) => Kind::ArcLength(m(a)),
            Kind::AngleMeasurement(a, b, c) => Kind::AngleMeasurement(m(a), m(b), m(c)),
            Kind::PointDistance(a, b) => Kind::PointDistance(m(a), m(b)),
            Kind::PointRatio(a, b, c) => Kind::PointRatio(m(a), m(b), m(c)),
            Kind::PolarTranslatedPoint(a, angle, units, distance) => {
                Kind::PolarTranslatedPoint(m(a), angle, units, distance)
            }
            Kind::ReflectedPoint(a, b) => Kind::ReflectedPoint(m(a), m(b)),
            Kind::ReflectedCircle(a, b) => Kind::ReflectedCircle(m(a), m(b)),
            Kind::ScaledPoint(a, b, factor) => Kind::ScaledPoint(m(a), m(b), factor),
            Kind::ScaledCircle(a, b, factor) => Kind::ScaledCircle(m(a), m(b), factor),
            Kind::RotatedPoint(a, b, angle) => Kind::RotatedPoint(m(a), m(b), angle),
            Kind::Midpoint(a, b) => Kind::Midpoint(m(a), m(b)),
            Kind::Segment(a, b) => Kind::Segment(m(a), m(b)),
            Kind::Line(a, b) => Kind::Line(m(a), m(b)),
            Kind::Ray(a, b) => Kind::Ray(m(a), m(b)),
            Kind::Parallel(a, b) => Kind::Parallel(m(a), m(b)),
            Kind::Perpendicular(a, b) => Kind::Perpendicular(m(a), m(b)),
            Kind::Circle(a, b) => Kind::Circle(m(a), m(b)),
            Kind::CircleBySegment(a, b) => Kind::CircleBySegment(m(a), m(b)),
            Kind::Bisector(a, b, c) => Kind::Bisector(m(a), m(b), m(c)),
            Kind::ThreePointArc(a, b, c) => Kind::ThreePointArc(m(a), m(b), m(c)),
            Kind::ArcOnCircle(a, b, c) => Kind::ArcOnCircle(m(a), m(b), m(c)),
            Kind::Intersection(a, b) => Kind::Intersection(m(a), m(b)),
            Kind::LineCircleIntersection(a, b, root) => {
                Kind::LineCircleIntersection(m(a), m(b), root)
            }
            Kind::CircleCircleIntersection(a, b, root) => {
                Kind::CircleCircleIntersection(m(a), m(b), root)
            }
            Kind::Unsupported(code) => Kind::Unsupported(code),
            Kind::Polygon(_) => unreachable!(),
            Kind::HideShowButton { .. } => unreachable!(),
            Kind::CoordinateMeasurement { .. } => unreachable!(),
            Kind::ValueEquation { .. } => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    HideShowButton {
        target: usize,
        action: VisibilityAction,
        label: String,
        anchor: (i16, i16),
    },
    Free(Point),
    Image {
        png: Vec<u8>,
        transform: [f64; 6],
    },
    PlainText {
        text: String,
        rect: TextRect,
    },
    Parameter(ParameterGroup),
    Calculation(CalculationGroup),
    ValueTable(usize, TableGroup),
    ValueTablePair(usize, usize, TableGroup),
    PointOnSegment(usize, f64),
    PointOnCircle(usize, f64, f64),
    PointOnArc(usize, f64),
    GraphHorizontalUnit(usize, f64),
    GraphVerticalUnit(usize, f64),
    GraphAxis(usize, usize, u32),
    CoordinateSystem(usize, usize),
    SlopeMeasurement(usize, usize),
    CoordinateMeasurement {
        kind: CoordinateMeasureKind,
        first: usize,
        second: Option<usize>,
        frame: usize,
    },
    ValueEquation {
        kind: ValueEquationKind,
        first: usize,
        second: usize,
    },
    FunctionDefinition(SimpleFunction),
    FunctionPlot(usize, usize, f64, f64, u32),
    PathLiteralPoint(usize, f64),
    PathValuePoint(usize, usize),
    PlottedPoint(usize, usize, PlotPointsMode, [f64; 2]),
    SegmentLength(usize),
    PolygonPerimeter(usize),
    CircleCircumference(usize),
    CircleRadius(usize),
    CircleArea(usize),
    ArcAngle(usize),
    ArcLength(usize),
    AngleMeasurement(usize, usize, usize),
    PointDistance(usize, usize),
    PointRatio(usize, usize, usize),
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
    Polygon(Vec<usize>),
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
    pub tracing: bool,
    pub deleted: bool,
    pub midpoint_host: Option<usize>,
    pub point_offset: Option<usize>,
    pub constraint_offset: Option<usize>,
    pub header_offset: Option<usize>,
    pub label: Option<LabelAux>,
    pub visual_style: Option<VisualStyle>,
}
#[derive(Clone, Debug)]
pub struct Clipboard {
    objects: Vec<(usize, Kind, bool, bool, Option<usize>, Option<VisualStyle>)>,
}
impl Clipboard {
    pub fn len(&self) -> usize {
        self.objects.len()
    }
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
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
        8 => [
            8, 0, 0, 0, 0, 0, 4, 0, 53, 0, 0x21, 1, 0xff, 0x80, 0, 0, 0xff, 0x7f, 0x39, 0x20, 0, 0,
            0, 0, 0, 0, 0, 0,
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
            | Kind::GraphAxis(_, _, _)
    )
}

fn is_point_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Free(_)
            | Kind::PointOnSegment(_, _)
            | Kind::PointOnCircle(_, _, _)
            | Kind::PointOnArc(_, _)
            | Kind::GraphHorizontalUnit(_, _)
            | Kind::GraphVerticalUnit(_, _)
            | Kind::PathLiteralPoint(_, _)
            | Kind::PathValuePoint(_, _)
            | Kind::PlottedPoint(_, _, _, _)
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

fn is_plot_path_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Segment(_, _)
            | Kind::Ray(_, _)
            | Kind::Line(_, _)
            | Kind::GraphAxis(_, _, 0 | 1 | 0x1_0001)
    ) || is_circle_kind(kind)
}

fn plot_path_accepts(kind: &Kind, value: f64) -> bool {
    value.is_finite()
        && is_plot_path_kind(kind)
        && match kind {
            Kind::Segment(_, _) => (0.0..=1.0).contains(&value),
            Kind::Ray(_, _) => value >= 0.0,
            _ => true,
        }
}

fn default_visual_style(kind: &Kind) -> Option<VisualStyle> {
    let red = RgbColor {
        red: 255,
        green: 0,
        blue: 0,
    };
    let navy = RgbColor {
        red: 0,
        green: 0,
        blue: 128,
    };
    let green = RgbColor {
        red: 0,
        green: 128,
        blue: 0,
    };
    let line = SegmentLineStyle {
        width: LineWidth::Medium,
        pattern: LinePattern::Solid,
    };
    match kind {
        Kind::Free(_)
        | Kind::PointOnSegment(_, _)
        | Kind::PointOnCircle(_, _, _)
        | Kind::PointOnArc(_, _)
        | Kind::PathLiteralPoint(_, _)
        | Kind::PathValuePoint(_, _)
        | Kind::PolarTranslatedPoint(_, _, _, _)
        | Kind::ReflectedPoint(_, _)
        | Kind::ScaledPoint(_, _, _)
        | Kind::RotatedPoint(_, _, _)
        | Kind::Midpoint(_, _)
        | Kind::Intersection(_, _)
        | Kind::LineCircleIntersection(_, _, _)
        | Kind::CircleCircleIntersection(_, _, _) => Some(VisualStyle::Point {
            shape: PointStyle::Medium,
            color: red,
        }),
        Kind::Segment(_, _)
        | Kind::Line(_, _)
        | Kind::Ray(_, _)
        | Kind::Parallel(_, _)
        | Kind::Perpendicular(_, _)
        | Kind::Bisector(_, _, _) => Some(VisualStyle::Stroke { line, color: navy }),
        Kind::Circle(_, _)
        | Kind::CircleBySegment(_, _)
        | Kind::ReflectedCircle(_, _)
        | Kind::ScaledCircle(_, _, _)
        | Kind::ThreePointArc(_, _, _)
        | Kind::ArcOnCircle(_, _, _) => Some(VisualStyle::Stroke { line, color: green }),
        Kind::Polygon(_) => Some(VisualStyle::Polygon {
            color: RgbColor {
                red: 255,
                green: 128,
                blue: 0,
            },
        }),
        _ => None,
    }
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
    pub snap_points: Vec<bool>,
    pub grid_dotted: Vec<bool>,
    pub grid_form: Vec<GridForm>,
    original: Vec<u8>,
    source_count: usize,
    safe_structure: bool,
    image_read_only: bool,
    pub undo: Vec<(Vec<Object>, Vec<bool>, Vec<bool>, Vec<GridForm>)>,
    pub redo: Vec<(Vec<Object>, Vec<bool>, Vec<bool>, Vec<GridForm>)>,
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
        let has_picture_resource = rs.iter().any(|record| record.kind == 0x0514);
        let image_file = has_picture_resource
            .then(|| ImageFile::parse(b))
            .transpose()
            .ok()
            .flatten();
        let image_read_only =
            has_picture_resource && !image_file.as_ref().is_some_and(ImageFile::can_edit);
        let baseline_records = records(include_bytes!("../assets/point_template.gsp"))?;
        let safe_outside: BTreeSet<u32> = baseline_records
            .iter()
            .map(|record| record.kind)
            .chain([0x2328, 0x232d])
            .collect();
        let mut objects: Vec<Object> = Vec::new();
        let mut i = 0;
        let mut safe = true;
        let mut snap_points = Vec::new();
        let mut grid_dotted = Vec::new();
        let mut grid_form = Vec::new();
        let (mut page, mut page_base, mut page_count) = (0, 0, 0);
        while i < rs.len() {
            if rs[i].kind == 0x3e8 {
                if page_count > 0 {
                    page += 1;
                    page_base = objects.len();
                }
                page_count += 1;
                snap_points.push(graph::snap_points(&b[rs[i].data.clone()]).unwrap_or(false));
                grid_dotted.push(false);
                grid_form.push(GridForm::Square);
                i += 1;
                continue;
            }
            if rs[i].kind != 0x7d0 {
                if !safe_outside.contains(&rs[i].kind)
                    && !(rs[i].kind == 0x0514 && !image_read_only)
                {
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
            let group_start = rs[i].start;
            let is_image = image_file
                .as_ref()
                .is_some_and(|picture| picture.picture_group_offset() == group_start);
            let raw = u32_at(h, 0);
            let code = raw as u16;
            let mut point = None;
            let mut point_offset = None;
            let mut constraint_payload: Option<(Vec<u8>, usize)> = None;
            let mut refs = Vec::new();
            let mut measurement_format = false;
            let mut measurement_style = false;
            let mut graph_records: Vec<(u32, Vec<u8>)> = Vec::new();
            let mut function_expression: Option<Vec<u8>> = None;
            let mut function_range: Option<Vec<u8>> = None;
            let mut label: Option<LabelAux> = None;
            let mut rich_text: Option<Vec<u8>> = None;
            let mut text_box: Option<Vec<u8>> = None;
            let mut text_style = false;
            i += 1;
            while i < rs.len() && rs[i].kind != 0x7d7 {
                let r = &rs[i];
                let d = &b[r.data.clone()];
                if matches!(code, 52 | 54 | 55 | 58 | 61 | 87) {
                    graph_records.push((r.kind, d.to_vec()));
                }
                if code == 0 && r.kind == 0x0907 {
                    function_expression = Some(d.to_vec());
                }
                if code == 72 && r.kind == 0x0902 {
                    function_range = Some(d.to_vec());
                }
                match r.kind {
                    0x07d5 if code == 0 => match LabelAux::parse(d) {
                        Ok(value) => label = Some(value),
                        Err(_) => safe = false,
                    },
                    0x08fc if code == 0 => rich_text = Some(d.to_vec()),
                    0x0898 if code == 0 => text_box = Some(d.to_vec()),
                    0x07d8 if code == 0 => text_style = d == [0, 0],
                    0x899 if d.len() == 16 => {
                        point = Some(Point {
                            x: f64_at(d, 0),
                            y: f64_at(d, 8),
                        });
                        point_offset = Some(r.data.start);
                    }
                    0x0903 if matches!(code, 36 | 37 | 39 | 40 | 41 | 42 | 43 | 44 | 46 | 47) => {
                        let sample: &[u8] = match code {
                            36 => &include_bytes!("../assets/length_measurement_group.bin")[60..92],
                            37 => {
                                &include_bytes!("../assets/distance_measurement_group.bin")[64..96]
                            }
                            39 => {
                                &include_bytes!("../assets/perimeter_measurement_group.bin")[60..92]
                            }
                            40 => &include_bytes!("../assets/circumference_measurement_group.bin")
                                [60..92],
                            41 => &include_bytes!("../assets/angle_measurement_group.bin")[68..100],
                            42 => &include_bytes!("../assets/area_measurement_group.bin")[60..92],
                            43 => {
                                &include_bytes!("../assets/arc_angle_measurement_group.bin")[60..92]
                            }
                            44 => &include_bytes!("../assets/arc_length_measurement_group.bin")
                                [60..92],
                            47 => &include_bytes!("../assets/ratio_measurement_group.bin")[68..100],
                            _ => &include_bytes!("../assets/radius_measurement_group.bin")[60..92],
                        };
                        measurement_format = d.len() == sample.len()
                            && d.iter()
                                .zip(sample)
                                .enumerate()
                                .all(|(i, (actual, expected))| {
                                    // Label x/y and presentation bytes vary after edits.
                                    matches!(i, 12..=15 | 22 | 30 | 31) || actual == expected
                                });
                    }
                    0x07d8 if matches!(code, 36 | 37 | 39 | 40 | 41 | 42 | 43 | 44 | 46 | 47) => {
                        let sample: &[u8] = match code {
                            36 => {
                                &include_bytes!("../assets/length_measurement_group.bin")[100..102]
                            }
                            37 => &include_bytes!("../assets/distance_measurement_group.bin")
                                [104..106],
                            39 => &include_bytes!("../assets/perimeter_measurement_group.bin")
                                [100..102],
                            40 => &include_bytes!("../assets/circumference_measurement_group.bin")
                                [100..102],
                            41 => {
                                &include_bytes!("../assets/angle_measurement_group.bin")[108..110]
                            }
                            42 => &include_bytes!("../assets/area_measurement_group.bin")[100..102],
                            43 => &include_bytes!("../assets/arc_angle_measurement_group.bin")
                                [100..102],
                            44 => &include_bytes!("../assets/arc_length_measurement_group.bin")
                                [100..102],
                            47 => {
                                &include_bytes!("../assets/ratio_measurement_group.bin")[108..110]
                            }
                            _ => {
                                &include_bytes!("../assets/radius_measurement_group.bin")[100..102]
                            }
                        };
                        measurement_style = d == sample;
                    }
                    0x7d3 if matches!(code, 15 | 21 | 27 | 30 | 54 | 55) => {
                        constraint_payload = Some((d.to_vec(), r.data.start));
                    }
                    0x7d2 if matches!(code, 67 | 68) && d.len() == 8 => {
                        refs = [u32_at(d, 0), u32_at(d, 4)]
                            .into_iter()
                            .map(|raw| {
                                if raw == 0 {
                                    0
                                } else {
                                    page_base + raw as usize
                                }
                            })
                            .collect();
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
                if !matches!(r.kind, 0x899 | 0x7d2 | 0x7d3)
                    && !(code == 0 && matches!(r.kind, 0x07d5 | 0x07d8 | 0x08fc | 0x0898))
                    && !(matches!(code, 36 | 37 | 39 | 40 | 41 | 42 | 43 | 44 | 46 | 47)
                        && matches!(r.kind, 0x0903 | 0x07d8))
                    && !(matches!(code, 52 | 54 | 55 | 58 | 61 | 87)
                        && matches!(r.kind, 0x0905 | 0x0904 | 0x0903 | 0x07d5 | 0x07d8))
                    && !(matches!(code, 49 | 65 | 66 | 86)
                        && matches!(r.kind, 0x08fc | 0x0903 | 0x07d8))
                    && !(matches!(code, 50 | 51 | 94) && matches!(r.kind, 0x08fc | 0x0903 | 0x07d8))
                    && !(code == 62 && matches!(r.kind, 0x0906 | 0x07d5 | 0x07d8))
                    && !(code == 0 && matches!(r.kind, 0x0907 | 0x08a3))
                    && !(code == 91 && matches!(r.kind, 0x090d | 0x07d8))
                    && !(is_image && matches!(r.kind, 0x090c | 0x08a8 | 0x1f44))
                    && !(code == 72 && r.kind == 0x0902)
                {
                    safe = false;
                }
                i += 1;
            }
            if i == rs.len() {
                return Err("Unterminated object group".into());
            }
            let action_group = (code == 62)
                .then(|| HideShowButtonGroup::parse(&b[group_start..rs[i].data.end]))
                .transpose();
            if action_group.is_err() {
                safe = false;
            }
            let action_group = action_group.ok().flatten();
            let coordinate_group = matches!(code, 49 | 65 | 66 | 86)
                .then(|| coordinate_measure_records::decode_group(&b[group_start..rs[i].data.end]))
                .transpose();
            if coordinate_group.is_err() {
                safe = false;
            }
            let coordinate_group = coordinate_group.ok().flatten();
            let value_equation_group = matches!(code, 50 | 51 | 94)
                .then(|| value_equation_records::decode_group(&b[group_start..rs[i].data.end]))
                .transpose();
            if value_equation_group.is_err() {
                safe = false;
            }
            let value_equation_group = value_equation_group.ok().flatten();
            let plotted_point = matches!(code, 67 | 68)
                .then(|| plot_points_records::decode_group(&b[group_start..rs[i].data.end]))
                .transpose();
            if plotted_point.is_err() {
                safe = false;
            }
            let plotted_point = plotted_point.ok().flatten();
            let axis_value_refs = (code == 95)
                .then(|| plot_axis_records::decode_group(&b[group_start..rs[i].data.end]))
                .transpose();
            if axis_value_refs.is_err() {
                safe = false;
            }
            let axis_value_refs = axis_value_refs.ok().flatten();
            let literal_axis_value = (code == 123)
                .then(|| plot_axis_records::decode_literal_group(&b[group_start..rs[i].data.end]))
                .transpose();
            if literal_axis_value.is_err() {
                safe = false;
            }
            let literal_axis_value = literal_axis_value.ok().flatten();
            let graph_borrowed = graph_records
                .iter()
                .map(|(kind, payload)| (*kind, payload.as_slice()))
                .collect::<Vec<_>>();
            let graph_kind = graph::decode_graph_object(code, &refs, &graph_borrowed);
            let kind = if refs
                .iter()
                .any(|&reference| reference == 0 || reference > objects.len())
            {
                Kind::Unsupported(code)
            } else if let Some(graph_kind) = graph_kind {
                match graph_kind {
                    GraphObject::HorizontalUnit { origin, step }
                        if objects
                            .get(origin - 1)
                            .is_some_and(|o| is_point_kind(&o.kind)) =>
                    {
                        Kind::GraphHorizontalUnit(origin, step)
                    }
                    GraphObject::VerticalUnit {
                        horizontal_unit,
                        step,
                    } if objects
                        .get(horizontal_unit - 1)
                        .is_some_and(|o| matches!(o.kind, Kind::GraphHorizontalUnit(_, _))) =>
                    {
                        Kind::GraphVerticalUnit(horizontal_unit, step)
                    }
                    GraphObject::Axis { start, end, axis }
                        if objects
                            .get(start - 1)
                            .is_some_and(|o| is_point_kind(&o.kind))
                            && objects.get(end - 1).is_some_and(|o| is_point_kind(&o.kind)) =>
                    {
                        Kind::GraphAxis(start, end, axis)
                    }
                    GraphObject::CoordinateSystem {
                        x_axis,
                        y_axis,
                        form,
                        dotted,
                    } if objects
                        .get(x_axis - 1)
                        .is_some_and(|o| matches!(o.kind, Kind::GraphAxis(_, _, 1 | 0x1_0001)))
                        && objects
                            .get(y_axis - 1)
                            .is_some_and(|o| matches!(o.kind, Kind::GraphAxis(_, _, 0))) =>
                    {
                        grid_dotted[page] = dotted;
                        grid_form[page] = form;
                        Kind::CoordinateSystem(x_axis, y_axis)
                    }
                    GraphObject::Slope { line, coordinates }
                        if objects.get(line - 1).is_some_and(|o| {
                            matches!(
                                o.kind,
                                Kind::Segment(_, _) | Kind::Line(_, _) | Kind::Ray(_, _)
                            )
                        }) && objects
                            .get(coordinates - 1)
                            .is_some_and(|o| matches!(o.kind, Kind::CoordinateSystem(_, _))) =>
                    {
                        Kind::SlopeMeasurement(line, coordinates)
                    }
                    _ => Kind::Unsupported(code),
                }
            } else {
                match (code, point, refs.as_slice()) {
                    (123, _, [path])
                        if literal_axis_value
                            .is_some_and(|(local_path, _)| local_path as usize + page_base == *path)
                            && objects.get(path - 1).is_some_and(|o| is_plot_path_kind(&o.kind)) => {
                        Kind::PathLiteralPoint(*path, literal_axis_value.unwrap().1)
                    }
                    (95, _, [source, path])
                        if axis_value_refs == Some([(*source - page_base) as u32, (*path - page_base) as u32])
                            && objects.get(source - 1).is_some_and(|o| matches!(o.kind, Kind::Parameter(_)))
                            && objects.get(path - 1).is_some_and(|o| is_plot_path_kind(&o.kind)) => {
                        Kind::PathValuePoint(*source, *path)
                    }
                    (67 | 68, _, [source, frame])
                        if plotted_point.as_ref().is_some_and(|plot| {
                            plot.references == [(*source - page_base) as u32, (*frame - page_base) as u32]
                        })
                        && objects.get(source - 1).is_some_and(|o| matches!(o.kind, Kind::Parameter(_)))
                        && objects.get(frame - 1).is_some_and(|o| matches!(o.kind, Kind::CoordinateSystem(_, _))) => {
                        let plot = plotted_point.unwrap();
                        Kind::PlottedPoint(*source, *frame, plot.mode, plot.stored)
                    }
                    (94, _, [first, second]) if value_equation_group.as_ref().is_some_and(|group| group.kind == ValueEquationKind::PointValue && group.references == [(*first - page_base) as u32, (*second - page_base) as u32])
                        && objects.get(first - 1).is_some_and(|o| matches!(o.kind, Kind::PointOnSegment(host, _) | Kind::PointOnCircle(host, _, _) if host == *second)) => {
                        Kind::ValueEquation { kind: ValueEquationKind::PointValue, first: *first, second: *second }
                    }
                    (50, _, [first, second]) if value_equation_group.as_ref().is_some_and(|group| group.kind == ValueEquationKind::LineEquation && group.references == [(*first - page_base) as u32, (*second - page_base) as u32])
                        && objects.get(first - 1).is_some_and(|o| matches!(o.kind, Kind::Line(_, _)))
                        && objects.get(second - 1).is_some_and(|o| matches!(o.kind, Kind::CoordinateSystem(_, _))) => {
                        Kind::ValueEquation { kind: ValueEquationKind::LineEquation, first: *first, second: *second }
                    }
                    (51, _, [first, second]) if value_equation_group.as_ref().is_some_and(|group| group.kind == ValueEquationKind::CircleEquation && group.references == [(*first - page_base) as u32, (*second - page_base) as u32])
                        && objects.get(first - 1).is_some_and(|o| matches!(o.kind, Kind::Circle(_, _)))
                        && objects.get(second - 1).is_some_and(|o| matches!(o.kind, Kind::CoordinateSystem(_, _))) => {
                        Kind::ValueEquation { kind: ValueEquationKind::CircleEquation, first: *first, second: *second }
                    }
                    (49 | 65 | 66, _, [first, frame]) if coordinate_group.as_ref().is_some_and(|g| {
                        g.references == vec![(*first - page_base) as u32, (*frame - page_base) as u32]
                    }) && objects.get(first - 1).is_some_and(|o| is_point_kind(&o.kind))
                        && objects.get(frame - 1).is_some_and(|o| matches!(o.kind, Kind::CoordinateSystem(_, _))) => {
                        Kind::CoordinateMeasurement { kind: coordinate_group.as_ref().unwrap().kind, first: *first, second: None, frame: *frame }
                    }
                    (86, _, [first, second, frame]) if coordinate_group.as_ref().is_some_and(|g| {
                        g.references == vec![(*first - page_base) as u32, (*second - page_base) as u32, (*frame - page_base) as u32]
                    }) && [first, second].iter().all(|&&id| objects.get(id - 1).is_some_and(|o| is_point_kind(&o.kind)))
                        && objects.get(frame - 1).is_some_and(|o| matches!(o.kind, Kind::CoordinateSystem(_, _))) => {
                        Kind::CoordinateMeasurement { kind: CoordinateMeasureKind::CoordinateDistance, first: *first, second: Some(*second), frame: *frame }
                    }
                    (0, None, []) if is_image => {
                        let picture = image_file.as_ref().unwrap();
                        Kind::Image { png: picture.png().to_vec(), transform: picture.transform() }
                    }
                    (0, None, []) if function_expression.is_some() && rich_text.is_none() && text_box.is_none() => {
                        let group = &b[group_start..rs[i].data.end];
                        if let Ok(parameter) = ParameterGroup::parse(group) {
                            Kind::Parameter(parameter)
                        } else if let Ok(calculation) = CalculationGroup::parse(group) {
                            Kind::Calculation(calculation)
                        } else {
                            match function_records::decode_simple_expression(function_expression.as_ref().unwrap()) {
                                Ok(expr) => Kind::FunctionDefinition(expr),
                                Err(_) => Kind::Unsupported(code),
                            }
                        }
                    }
                    (91, None, [source]) if *source == page_base + 1
                        && objects.get(*source - 1).is_some_and(|o| matches!(o.kind, Kind::Parameter(_))) => {
                        match TableGroup::parse(&b[group_start..rs[i].data.end]) {
                            Ok(table) if table.source_id() as usize == *source - page_base => Kind::ValueTable(*source, table),
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (91, None, [first, second]) if *first == page_base + 2 && *second == page_base + 1
                        && [first, second].iter().all(|&&id| objects.get(id - 1).is_some_and(|o| matches!(o.kind, Kind::Parameter(_)))) => {
                        match TableGroup::parse_pair(&b[group_start..rs[i].data.end]) {
                            Ok(table) if table.pair_source_ids() == Some([2, 1]) => Kind::ValueTablePair(*first, *second, table),
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (72, None, [definition, frame]) if function_range.is_some()
                        && matches!(objects.get(definition - 1).map(|o| &o.kind), Some(Kind::FunctionDefinition(_)))
                        && matches!(objects.get(frame - 1).map(|o| &o.kind), Some(Kind::CoordinateSystem(_, _))) => {
                        match function_records::decode_plot_range(function_range.as_ref().unwrap()) {
                            Ok((min,max,samples)) => Kind::FunctionPlot(*definition,*frame,min,max,samples),
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (62, _, [target]) if action_group.as_ref().is_some_and(|group| {
                        group.target_ids() == vec![(*target - page_base) as u32]
                    }) => {
                        let group = action_group.as_ref().unwrap();
                        match group.label_utf8() {
                            Ok(label) => Kind::HideShowButton {
                                target: *target,
                                action: group.action(),
                                label: label.to_owned(),
                                anchor: group.screen_anchor(),
                            },
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (0, Some(p), _) if rich_text.is_none() && text_box.is_none() => Kind::Free(p),
                    (0, None, []) if rich_text.is_some() && text_box.is_some()
                        && text_style && label.is_none() => {
                        match text_object::decode_plain_text(rich_text.as_ref().unwrap(), text_box.as_ref().unwrap()) {
                            Ok((text, rect)) => Kind::PlainText { text, rect },
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (36, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host - 1).is_some_and(|o| matches!(o.kind, Kind::Segment(_, _))) =>
                    {
                        Kind::SegmentLength(*host)
                    }
                    (37, _, [a, b])
                        if measurement_format && measurement_style
                            && objects.get(a - 1).is_some_and(|o| is_point_kind(&o.kind))
                            && objects.get(b - 1).is_some_and(|o| is_point_kind(&o.kind)) =>
                    {
                        Kind::PointDistance(*a, *b)
                    }
                    (47, _, [a, b, c])
                        if measurement_format && measurement_style
                            && [a, b, c].iter().all(|&&id| objects.get(id - 1).is_some_and(|o| is_point_kind(&o.kind))) =>
                    {
                        Kind::PointRatio(*a, *b, *c)
                    }
                    (39, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host - 1).is_some_and(|o| matches!(o.kind, Kind::Polygon(_))) =>
                    {
                        Kind::PolygonPerimeter(*host)
                    }
                    (40, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host - 1).is_some_and(|o| matches!(o.kind, Kind::Circle(_, _) | Kind::CircleBySegment(_, _))) =>
                    {
                        Kind::CircleCircumference(*host)
                    }
                    (41, _, [a, b, c])
                        if measurement_format && measurement_style
                            && [a, b, c].iter().all(|&&id| objects.get(id - 1).is_some_and(|o| is_point_kind(&o.kind))) =>
                    {
                        Kind::AngleMeasurement(*a, *b, *c)
                    }
                    (42, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host - 1).is_some_and(|o| matches!(o.kind, Kind::Circle(_, _) | Kind::CircleBySegment(_, _))) =>
                    {
                        Kind::CircleArea(*host)
                    }
                    (43, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host - 1).is_some_and(|o| matches!(o.kind, Kind::ThreePointArc(_, _, _) | Kind::ArcOnCircle(_, _, _))) =>
                    {
                        Kind::ArcAngle(*host)
                    }
                    (44, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host - 1).is_some_and(|o| matches!(o.kind, Kind::ThreePointArc(_, _, _) | Kind::ArcOnCircle(_, _, _))) =>
                    {
                        Kind::ArcLength(*host)
                    }
                    (46, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host - 1).is_some_and(|o| matches!(o.kind, Kind::Circle(_, _) | Kind::CircleBySegment(_, _))) =>
                    {
                        Kind::CircleRadius(*host)
                    }
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
                    (8, _, vertices)
                        if vertices.len() >= 3
                            && vertices.iter().all(|&id| objects.get(id - 1).is_some_and(|o| is_point_kind(&o.kind))) =>
                    {
                        Kind::Polygon(vertices.to_vec())
                    }
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
            if matches!(kind, Kind::FunctionDefinition(_) | Kind::Parameter(_)) {
                label = None;
            }
            if label.is_some() && !is_point_kind(&kind) {
                safe = false;
                label = None;
            }
            objects.push(Object {
                id: objects.len() + 1,
                page,
                midpoint_host: matches!(kind, Kind::Midpoint(_, _)).then(|| refs[0]),
                kind,
                visible: raw & 0x10000 == 0,
                tracing: h[4] & 1 != 0,
                deleted: false,
                point_offset,
                constraint_offset: constraint_payload.map(|(_, offset)| offset),
                header_offset: Some(header_offset),
                label,
                visual_style: style_records::visual_style(h),
            });
            if matches!(objects.last().map(|o| &o.kind), Some(Kind::Unsupported(_))) {
                safe = false;
            }
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
        if rs
            .iter()
            .filter(|record| record.kind == 0x232d)
            .any(|record| {
                record.data.len() != 4
                    || !objects
                        .get((u32_at(b, record.data.start) as usize).saturating_sub(1))
                        .is_some_and(|object| matches!(object.kind, Kind::CoordinateSystem(_, _)))
            })
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
            snap_points: if snap_points.is_empty() {
                vec![false]
            } else {
                snap_points
            },
            grid_dotted: if grid_dotted.is_empty() {
                vec![false]
            } else {
                grid_dotted
            },
            grid_form: if grid_form.is_empty() {
                vec![GridForm::Square]
            } else {
                grid_form
            },
            original: b.to_vec(),
            source_count,
            safe_structure: safe,
            image_read_only,
            undo: Vec::new(),
            redo: Vec::new(),
        })
    }
    pub fn checkpoint(&mut self) {
        self.undo.push((
            self.objects.clone(),
            self.snap_points.clone(),
            self.grid_dotted.clone(),
            self.grid_form.clone(),
        ));
        self.redo.clear();
        self.dirty = true;
    }
    pub fn undo(&mut self) {
        if let Some((objects, snap_points, grid_dotted, grid_form)) = self.undo.pop() {
            self.redo.push((
                std::mem::replace(&mut self.objects, objects),
                std::mem::replace(&mut self.snap_points, snap_points),
                std::mem::replace(&mut self.grid_dotted, grid_dotted),
                std::mem::replace(&mut self.grid_form, grid_form),
            ));
            self.dirty = true;
        }
    }
    pub fn redo(&mut self) {
        if let Some((objects, snap_points, grid_dotted, grid_form)) = self.redo.pop() {
            self.undo.push((
                std::mem::replace(&mut self.objects, objects),
                std::mem::replace(&mut self.snap_points, snap_points),
                std::mem::replace(&mut self.grid_dotted, grid_dotted),
                std::mem::replace(&mut self.grid_form, grid_form),
            ));
            self.dirty = true;
        }
    }
    pub fn add(&mut self, kind: Kind) -> usize {
        self.checkpoint();
        self.append_object(kind)
    }
    pub fn add_text(&mut self, text: &str, at: Point) -> Result<usize, String> {
        let x = i16::try_from(at.x.round() as i64).map_err(|_| "Text x is out of range")?;
        let y = i16::try_from(at.y.round() as i64).map_err(|_| "Text y is out of range")?;
        let rect = TextRect::roomy_at(x, y, text)?;
        text_object::encode_plain_text_group(text, rect)?;
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot add text to this sketch safely".into());
        }
        Ok(self.add(Kind::PlainText {
            text: text.to_owned(),
            rect,
        }))
    }
    pub fn import_png(&mut self, png: &[u8], at: Point) -> Result<usize, String> {
        if self.partial_view
            || self.page_count != 1
            || self.image_read_only
            || self
                .objects
                .iter()
                .any(|object| matches!(object.kind, Kind::Image { .. }))
        {
            return Err(
                "Only one independent picture can be edited in a single-page sketch".into(),
            );
        }
        if !at.x.is_finite() || !at.y.is_finite() {
            return Err("Picture position must be finite".into());
        }
        image::load_from_memory_with_format(png, image::ImageFormat::Png)
            .map_err(|error| format!("PNG pixels cannot be decoded: {error}"))?;
        ImageFile::independent_from_png(png, [1., 0., at.x, 0., 1., at.y])?;
        Ok(self.add(Kind::Image {
            png: png.to_vec(),
            transform: [1., 0., at.x, 0., 1., at.y],
        }))
    }
    pub fn move_image(&mut self, id: usize, at: Point) -> Result<(), String> {
        self.move_image_inner(id, at, true)
    }
    pub fn drag_image(&mut self, id: usize, at: Point) -> Result<(), String> {
        self.move_image_inner(id, at, false)
    }
    fn move_image_inner(&mut self, id: usize, at: Point, checkpoint: bool) -> Result<(), String> {
        if self.image_read_only || !at.x.is_finite() || !at.y.is_finite() {
            return Err("Picture cannot be moved safely".into());
        }
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid picture ID")?)
            .ok_or("Invalid picture ID")?;
        let Kind::Image { png, mut transform } = object.kind.clone() else {
            return Err("Object is not a picture".into());
        };
        if checkpoint {
            self.checkpoint();
        }
        transform[2] = at.x;
        transform[5] = at.y;
        self.objects[id - 1].kind = Kind::Image { png, transform };
        self.dirty = true;
        Ok(())
    }
    pub fn edit_text(&mut self, id: usize, text: &str) -> Result<(), String> {
        if self.partial_view {
            return Err("Cannot edit text in a sketch with unsupported records safely".into());
        }
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid text ID")?)
            .ok_or("Invalid text ID")?;
        let Kind::PlainText { rect, .. } = object.kind else {
            return Err("Object is not free text".into());
        };
        let rect = TextRect::roomy_at(rect.left, rect.top, text)?;
        text_object::encode_plain_text_group(text, rect)?;
        self.checkpoint();
        self.objects[id - 1].kind = Kind::PlainText {
            text: text.to_owned(),
            rect,
        };
        Ok(())
    }
    pub fn move_text(&mut self, id: usize, at: Point) -> Result<(), String> {
        self.move_text_inner(id, at, true)
    }
    pub fn drag_text(&mut self, id: usize, at: Point) -> Result<(), String> {
        self.move_text_inner(id, at, false)
    }
    fn move_text_inner(&mut self, id: usize, at: Point, checkpoint: bool) -> Result<(), String> {
        let x = i16::try_from(at.x.round() as i64).map_err(|_| "Text x is out of range")?;
        let y = i16::try_from(at.y.round() as i64).map_err(|_| "Text y is out of range")?;
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid text ID")?)
            .ok_or("Invalid text ID")?;
        let Kind::PlainText { ref text, rect } = object.kind else {
            return Err("Object is not free text".into());
        };
        let dx = i32::from(x) - i32::from(rect.left);
        let dy = i32::from(y) - i32::from(rect.top);
        let right =
            i16::try_from(i32::from(rect.right) + dx).map_err(|_| "Text x is out of range")?;
        let bottom =
            i16::try_from(i32::from(rect.bottom) + dy).map_err(|_| "Text y is out of range")?;
        let text = text.clone();
        if checkpoint {
            self.checkpoint();
        }
        self.objects[id - 1].kind = Kind::PlainText {
            text,
            rect: TextRect {
                left: x,
                top: y,
                right,
                bottom,
            },
        };
        self.dirty = true;
        Ok(())
    }
    pub fn set_point_label(&mut self, id: usize, name: &str, visible: bool) -> Result<(), String> {
        if self.partial_view {
            return Err("Cannot edit labels in a sketch with unsupported records safely".into());
        }
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid point ID")?)
            .ok_or("Invalid point ID")?;
        if !is_point_kind(&object.kind) || object.deleted {
            return Err("Object is not a point".into());
        }
        let mut label = if let Some(existing) = &object.label {
            existing.clone()
        } else {
            LabelAux::new_point(name, visible, (8, 5))?
        };
        label.set_name(name)?;
        label.set_visible(visible);
        self.checkpoint();
        self.objects[id - 1].label = Some(label);
        Ok(())
    }
    fn append_object(&mut self, kind: Kind) -> usize {
        let id = self.objects.len() + 1;
        let midpoint_host = if let Kind::Midpoint(a, b) = kind {
            self.objects
                .iter()
                .find(|object| {
                    !object.deleted
                        && matches!(object.kind, Kind::Segment(x, y) if (x == a && y == b) || (x == b && y == a))
                })
                .map(|object| object.id)
        } else {
            None
        };
        self.objects.push(Object {
            id,
            page: self.active_page,
            visual_style: default_visual_style(&kind),
            kind,
            visible: true,
            tracing: false,
            deleted: false,
            midpoint_host,
            point_offset: None,
            constraint_offset: None,
            header_offset: None,
            label: None,
        });
        id
    }
    pub fn add_default_parameter(&mut self, anchor: (i16, i16)) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot add a parameter in this sketch".into());
        }
        let mut group = match self.objects.as_slice() {
            [] => ParameterGroup::default_value(),
            [first] if matches!(&first.kind, Kind::Parameter(group) if group.name() == "t[1]" && group.value() == 1.0) => {
                ParameterGroup::second_default_value()
            }
            _ => return Err("The verified parameter sequence is t[1], then t[2]".into()),
        };
        group.set_anchor(anchor)?;
        self.checkpoint();
        Ok(self.append_object(Kind::Parameter(group)))
    }
    pub fn set_second_parameter_value(&mut self, id: usize, value: u16) -> Result<(), String> {
        if id != 2 {
            return Err("Select t[2]".into());
        }
        self.set_parameter_value(id, value as f64)
    }
    pub fn set_parameter_value(&mut self, id: usize, value: f64) -> Result<(), String> {
        if self.partial_view || self.page_count != 1 || !matches!(id, 1 | 2) {
            return Err("Only verified parameters can be edited".into());
        }
        let Some(Kind::Parameter(group)) = self.objects.get(id - 1).map(|o| &o.kind) else {
            return Err("Select a parameter".into());
        };
        if id == 1
            && self.objects.iter().any(|object| {
                matches!(
                    object.kind,
                    Kind::ValueTable(_, _) | Kind::ValueTablePair(_, _, _)
                )
            })
        {
            return Err("Editing a tabulated t[1] value is not yet supported".into());
        }
        let mut group = group.clone();
        group.set_value(value)?;
        let table = (id == 2)
            .then(|| self.objects.get(2))
            .flatten()
            .and_then(|object| {
                if let Kind::ValueTablePair(_, _, table) = &object.kind {
                    Some(table.clone())
                } else {
                    None
                }
            });
        let table = table
            .map(|mut table| {
                table.set_last_pair_value(value)?;
                Ok::<_, String>(table)
            })
            .transpose()?;
        self.checkpoint();
        self.objects[id - 1].kind = Kind::Parameter(group);
        if let Some(table) = table {
            self.objects[2].kind = Kind::ValueTablePair(2, 1, table);
        }
        Ok(())
    }
    pub fn add_literal_calculation(&mut self, anchor: (i16, i16)) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || !self.objects.is_empty() {
            return Err("The verified calculation can only be added to a blank sketch".into());
        }
        let mut group = CalculationGroup::literal_two_plus_three();
        group.set_anchor(anchor)?;
        self.checkpoint();
        Ok(self.append_object(Kind::Calculation(group)))
    }
    pub fn tabulate_default_parameter(
        &mut self,
        source: usize,
        anchor: (i16, i16),
    ) -> Result<usize, String> {
        if self.partial_view
            || self.page_count != 1
            || source != 1
            || self.objects.len() != 1
            || !matches!(
                self.objects.first().map(|o| &o.kind),
                Some(Kind::Parameter(_))
            )
        {
            return Err("Only the verified t[1] parameter can be tabulated".into());
        }
        let group = TableGroup::for_source(1, anchor)?;
        self.checkpoint();
        Ok(self.append_object(Kind::ValueTable(source, group)))
    }
    pub fn tabulate_default_pair(&mut self, anchor: (i16, i16)) -> Result<usize, String> {
        if self.partial_view
            || self.page_count != 1
            || self.objects.len() != 2
            || !matches!(self.objects.first().map(|o| &o.kind), Some(Kind::Parameter(group)) if group.name() == "t[1]")
            || !matches!(self.objects.get(1).map(|o| &o.kind), Some(Kind::Parameter(group)) if group.name() == "t[2]")
        {
            return Err("Only the verified t[1], t[2] pair can be tabulated".into());
        }
        let mut group = TableGroup::pair_snapshot();
        group.set_anchor(anchor);
        self.checkpoint();
        Ok(self.append_object(Kind::ValueTablePair(2, 1, group)))
    }
    pub fn move_number(&mut self, id: usize, anchor: (i16, i16)) -> Result<(), String> {
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid object ID")?)
            .ok_or("Invalid object ID")?;
        let mut kind = object.kind.clone();
        match &mut kind {
            Kind::Parameter(group) => group.set_anchor(anchor)?,
            Kind::Calculation(group) => group.set_anchor(anchor)?,
            Kind::ValueTable(_, group) | Kind::ValueTablePair(_, _, group) => {
                group.set_anchor(anchor)
            }
            _ => return Err("Object is not a number or table".into()),
        }
        self.checkpoint();
        self.objects[id - 1].kind = kind;
        Ok(())
    }
    pub fn change_table_rows(&mut self, id: usize, add: bool) -> Result<(), String> {
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid table ID")?)
            .ok_or("Invalid table ID")?;
        let kind = match &object.kind {
            Kind::ValueTable(_, group) => {
                let mut group = group.clone();
                if add {
                    group.add_default_snapshot()?;
                } else {
                    group.remove_last_snapshot()?;
                }
                Kind::ValueTable(1, group)
            }
            Kind::ValueTablePair(_, _, group) => {
                let values = group.pair_values().ok_or("Invalid pair table")?;
                let rows = if add {
                    group.rows() + 1
                } else {
                    group.rows().checked_sub(1).ok_or("Table has no rows")?
                };
                let mut changed = TableGroup::pair_snapshot_rows(rows, group.anchor())?;
                if add {
                    for (row, value) in values.iter().enumerate() {
                        changed.set_pair_row_value(row, value[0])?;
                    }
                    changed.set_last_pair_value(values.last().unwrap()[0])?;
                } else {
                    changed.set_last_pair_value(values.last().unwrap()[0])?;
                }
                Kind::ValueTablePair(2, 1, changed)
            }
            _ => return Err("Object is not a table".into()),
        };
        self.checkpoint();
        self.objects[id - 1].kind = kind;
        Ok(())
    }
    pub fn active_coordinate_system(&self) -> Option<usize> {
        self.objects
            .iter()
            .rev()
            .find(|o| {
                o.page == self.active_page
                    && !o.deleted
                    && matches!(o.kind, Kind::CoordinateSystem(_, _))
            })
            .map(|o| o.id)
    }
    fn active_vertical_unit(&self, page: usize) -> Option<usize> {
        let frame = self.objects.iter().rev().find(|o| {
            o.page == page && !o.deleted && matches!(o.kind, Kind::CoordinateSystem(_, _))
        })?;
        let Kind::CoordinateSystem(_, y_axis) = frame.kind else {
            return None;
        };
        let Kind::GraphAxis(_, y_unit, 0) = self.objects.get(y_axis.checked_sub(1)?)?.kind else {
            return None;
        };
        Some(y_unit)
    }
    fn active_x_axis(&self, page: usize) -> Option<usize> {
        let frame = self.objects.iter().rev().find(|o| {
            o.page == page && !o.deleted && matches!(o.kind, Kind::CoordinateSystem(_, _))
        })?;
        let Kind::CoordinateSystem(x_axis, _) = frame.kind else {
            return None;
        };
        Some(x_axis)
    }
    pub fn add_default_coordinate_system(&mut self, origin: Point) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || self.active_coordinate_system().is_some() {
            return Err("Cannot define another coordinate system in this sketch".into());
        }
        if !origin.x.is_finite() || !origin.y.is_finite() {
            return Err("Coordinate-system origin must be finite".into());
        }
        let step: f64 = 37.79527559055118;
        self.checkpoint();
        let center = self.append_object(Kind::Free(origin));
        let x_unit = self.append_object(Kind::GraphHorizontalUnit(center, step));
        let x_axis = self.append_object(Kind::GraphAxis(center, x_unit, 1));
        let y_unit = self.append_object(Kind::GraphVerticalUnit(x_unit, step));
        let y_axis = self.append_object(Kind::GraphAxis(center, y_unit, 0));
        let frame = self.append_object(Kind::CoordinateSystem(x_axis, y_axis));
        self.snap_points[self.active_page] = false;
        self.grid_dotted[self.active_page] = false;
        self.grid_form[self.active_page] = GridForm::Square;
        Ok(frame)
    }
    pub fn add_function_definition(&mut self, expression: SimpleFunction) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot add a function to this sketch".into());
        }
        self.checkpoint();
        Ok(self.append_object(Kind::FunctionDefinition(expression)))
    }
    pub fn plot_new_function(
        &mut self,
        expression: SimpleFunction,
        origin: Point,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot plot a function in this sketch".into());
        }
        let frame = match self.active_coordinate_system() {
            Some(id) => id,
            None => self.add_default_coordinate_system(origin)?,
        };
        self.checkpoint();
        let definition = self.append_object(Kind::FunctionDefinition(expression));
        Ok(self.append_object(Kind::FunctionPlot(
            definition,
            frame,
            -5.265208333333335,
            5.265208333333335,
            750,
        )))
    }
    pub fn function_plot_points(&self, id: usize, count: usize) -> Option<Vec<Point>> {
        let Kind::FunctionPlot(definition, frame, min, max, _) =
            self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let Kind::FunctionDefinition(expression) =
            self.objects.get(definition.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let frame = self.graph_frame(frame)?;
        let count = count.clamp(2, 4096);
        (0..count)
            .map(|i| {
                let x = min + (max - min) * (i as f64 / (count - 1) as f64);
                let p = frame.world(x, expression.evaluate(x))?;
                Some(Point { x: p.x, y: p.y })
            })
            .collect()
    }
    pub fn plot_default_parameter(&mut self, mode: PlotPointsMode) -> Result<usize, String> {
        if self.partial_view
            || self.page_count != 1
            || self.objects.len() != 8
            || !matches!(self.objects.first().map(|o| &o.kind), Some(Kind::Parameter(group)) if group.name() == "t[1]" && group.value() == 1.0)
            || !matches!(self.objects.get(1).map(|o| &o.kind), Some(Kind::ValueTable(1, group)) if group.rows() > 0)
            || self.active_coordinate_system() != Some(8)
        {
            return Err(
                "Plot Points currently requires the t[1] table and its default Cartesian frame"
                    .into(),
            );
        }
        let group = plot_points_records::encode_t1_group(mode, [1, 8])?;
        let plot = plot_points_records::decode_group(&group)?;
        let style = style_records::visual_style(&group[8..36]);
        self.checkpoint();
        let id = self.append_object(Kind::PlottedPoint(1, 8, mode, plot.stored));
        self.objects[id - 1].visual_style = style;
        Ok(id)
    }
    pub fn plot_default_pair_table(&mut self, table: usize) -> Result<Vec<usize>, String> {
        if self.partial_view
            || self.page_count != 1
            || table != 3
            || self.objects.len() != 9
            || !matches!(self.objects.first().map(|o| &o.kind), Some(Kind::Parameter(group)) if group.name() == "t[1]" && group.value() == 1.0)
            || !matches!(self.objects.get(1).map(|o| &o.kind), Some(Kind::Parameter(group)) if group.name() == "t[2]" && (0.0..10.0).contains(&group.value()))
            || !matches!(self.objects.get(2).map(|o| &o.kind), Some(Kind::ValueTablePair(2, 1, group)) if (1..=2).contains(&group.rows()))
            || self.active_coordinate_system() != Some(9)
        {
            return Err(
                "Plot Table Data requires the verified t[2], t[1] table and coordinate system"
                    .into(),
            );
        }
        let Some(Kind::ValueTablePair(_, _, table_group)) = self.objects.get(2).map(|o| &o.kind)
        else {
            return Err("Select the verified pair table".into());
        };
        let rows = table_group
            .pair_values()
            .ok_or("Invalid pair table values")?;
        let mut unique = Vec::new();
        for value in rows {
            if !unique.contains(&value) {
                unique.push(value);
            }
        }
        self.checkpoint();
        let mut ids = Vec::new();
        for stored in unique {
            let group =
                plot_points_records::encode_group(PlotPointsMode::Cartesian, [1, 9], stored)?;
            let style = style_records::visual_style(&group[8..36]);
            let id =
                self.append_object(Kind::PlottedPoint(1, 9, PlotPointsMode::Cartesian, stored));
            self.objects[id - 1].visual_style = style;
            ids.push(id);
        }
        Ok(ids)
    }
    fn plotted_path_position(&self, path: usize, value: f64) -> Result<Point, String> {
        let object = self
            .objects
            .get(path.checked_sub(1).ok_or("Invalid path")?)
            .ok_or("Invalid path")?;
        if object.page != self.active_page || object.deleted {
            return Err("Select a path on this page".into());
        }
        if !plot_path_accepts(&object.kind, value) {
            return Err(match object.kind {
                Kind::Segment(_, _) => "Segment values must be between 0 and 1",
                Kind::Ray(_, _) => "Ray values must be zero or greater",
                _ => "Select a supported path and finite value",
            }
            .into());
        }
        if is_circle_kind(&object.kind) {
            let (center, radius) = self.circle_points(path).ok_or("Invalid circle")?;
            let angle = value * std::f64::consts::TAU;
            let point = Point {
                x: center.x + radius * angle.cos(),
                y: center.y - radius * angle.sin(),
            };
            return (point.x.is_finite() && point.y.is_finite())
                .then_some(point)
                .ok_or("Plotted value is outside the supported range".into());
        }
        let (start, end) = match object.kind {
            Kind::GraphAxis(start, end, 0 | 1 | 0x1_0001)
            | Kind::Segment(start, end)
            | Kind::Line(start, end)
            | Kind::Ray(start, end) => (start, end),
            _ => return Err("Select a supported path".into()),
        };
        let start = self.position(start).ok_or("Invalid path start")?;
        let end = self.position(end).ok_or("Invalid path end")?;
        let point = Point {
            x: start.x + (end.x - start.x) * value,
            y: start.y + (end.y - start.y) * value,
        };
        if !point.x.is_finite() || !point.y.is_finite() {
            return Err("Plotted value is outside the supported range".into());
        }
        Ok(point)
    }
    pub fn plot_parameter_on_path(
        &mut self,
        source: usize,
        path: Option<usize>,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot plot a value in this sketch".into());
        }
        let path = path
            .or_else(|| self.active_x_axis(self.active_page))
            .ok_or("No coordinate axis or selected path")?;
        let parameter = self
            .objects
            .get(source.checked_sub(1).ok_or("Invalid value")?)
            .ok_or("Invalid value")?;
        if parameter.page != self.active_page || parameter.deleted {
            return Err("Select a supported parameter".into());
        }
        let Kind::Parameter(ref group) = parameter.kind else {
            return Err("Select a supported parameter".into());
        };
        self.plotted_path_position(path, group.value())?;
        let group = plot_axis_records::encode_group([source as u32, path as u32])?;
        let style = style_records::visual_style(&group[8..36]);
        self.checkpoint();
        let id = self.append_object(Kind::PathValuePoint(source, path));
        self.objects[id - 1].visual_style = style;
        Ok(id)
    }
    pub fn plot_parameter_on_axis(
        &mut self,
        source: usize,
        axis: Option<usize>,
    ) -> Result<usize, String> {
        self.plot_parameter_on_path(source, axis)
    }
    pub fn plot_literal_on_path(
        &mut self,
        value: f64,
        path: Option<usize>,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || !value.is_finite() {
            return Err("Cannot plot that value in this sketch".into());
        }
        let path = path
            .or_else(|| self.active_x_axis(self.active_page))
            .ok_or("No coordinate axis or selected path")?;
        self.plotted_path_position(path, value)?;
        let group = plot_axis_records::encode_literal_group(path as u32, value)?;
        let style = style_records::visual_style(&group[8..36]);
        self.checkpoint();
        let id = self.append_object(Kind::PathLiteralPoint(path, value));
        self.objects[id - 1].visual_style = style;
        Ok(id)
    }
    pub fn plot_literal_on_axis(
        &mut self,
        value: f64,
        axis: Option<usize>,
    ) -> Result<usize, String> {
        self.plot_literal_on_path(value, axis)
    }
    pub fn set_grid_visible(&mut self, visible: bool) {
        if let Some(id) = self.active_coordinate_system()
            && self.objects[id - 1].visible != visible
        {
            self.checkpoint();
            self.objects[id - 1].visible = visible;
        }
    }
    pub fn grid_visible(&self) -> bool {
        self.grid_enabled() && !self.snap_points[self.active_page]
    }
    pub fn grid_enabled(&self) -> bool {
        self.active_coordinate_system()
            .is_some_and(|id| self.objects[id - 1].visible)
    }
    pub fn is_displayed(&self, id: usize) -> bool {
        let Some(object) = id.checked_sub(1).and_then(|at| self.objects.get(at)) else {
            return false;
        };
        if !object.visible || object.deleted || object.page != self.active_page {
            return false;
        }
        if matches!(object.kind, Kind::GraphVerticalUnit(_, _))
            && self.grid_form[object.page] != GridForm::Rectangular
        {
            return false;
        }
        !self.objects.iter().any(|frame| {
            let Kind::CoordinateSystem(x_axis, y_axis) = frame.kind else {
                return false;
            };
            if frame.visible || frame.deleted || frame.page != object.page {
                return false;
            }
            let Some(Kind::GraphAxis(origin, x_unit, 1 | 0x1_0001)) = x_axis
                .checked_sub(1)
                .and_then(|at| self.objects.get(at))
                .map(|o| &o.kind)
            else {
                return false;
            };
            let Some(Kind::GraphAxis(_, y_unit, 0)) = y_axis
                .checked_sub(1)
                .and_then(|at| self.objects.get(at))
                .map(|o| &o.kind)
            else {
                return false;
            };
            [frame.id, x_axis, y_axis, *origin, *x_unit, *y_unit].contains(&id)
        })
    }
    pub fn set_snap_points(&mut self, enabled: bool) {
        if self.active_coordinate_system().is_some()
            && self.snap_points[self.active_page] != enabled
        {
            self.checkpoint();
            self.snap_points[self.active_page] = enabled;
        }
    }
    pub fn snap_point(&self, point: Point) -> Point {
        if !self.snap_points[self.active_page] {
            return point;
        }
        let Some(frame) = self
            .active_coordinate_system()
            .and_then(|id| self.graph_frame(id))
        else {
            return point;
        };
        let Some(graph_point) = frame.coordinates(GraphPoint {
            x: point.x,
            y: point.y,
        }) else {
            return point;
        };
        let (x, y) = if self.grid_form[self.active_page] == GridForm::Polar {
            let radius = graph_point.x.hypot(graph_point.y).round();
            let sector =
                (graph_point.y.atan2(graph_point.x) / (std::f64::consts::TAU / 24.)).round();
            let angle = sector * std::f64::consts::TAU / 24.;
            (radius * angle.cos(), radius * angle.sin())
        } else {
            (graph_point.x.round(), graph_point.y.round())
        };
        frame.world(x, y).map_or(point, |point| Point {
            x: point.x,
            y: point.y,
        })
    }
    pub fn set_grid_dotted(&mut self, dotted: bool) {
        if self.active_coordinate_system().is_some() && self.grid_dotted[self.active_page] != dotted
        {
            self.checkpoint();
            self.grid_dotted[self.active_page] = dotted;
        }
    }
    pub fn set_grid_form(&mut self, form: GridForm) -> Result<(), String> {
        let frame = self
            .active_coordinate_system()
            .ok_or("No coordinate system")?;
        if self.grid_form[self.active_page] == form {
            return Ok(());
        }
        let Kind::CoordinateSystem(_, y_axis) = self.objects[frame - 1].kind else {
            return Err("Invalid coordinate system".into());
        };
        let Some(Kind::GraphAxis(_, y_unit, 0)) = y_axis
            .checked_sub(1)
            .and_then(|at| self.objects.get(at))
            .map(|object| &object.kind)
        else {
            return Err("Invalid vertical axis".into());
        };
        let Some(Kind::GraphVerticalUnit(x_unit, _)) = y_unit
            .checked_sub(1)
            .and_then(|at| self.objects.get(at))
            .map(|object| &object.kind)
        else {
            return Err("Unsupported vertical unit".into());
        };
        let Some(Kind::GraphHorizontalUnit(_, x_step)) = x_unit
            .checked_sub(1)
            .and_then(|at| self.objects.get(at))
            .map(|object| &object.kind)
        else {
            return Err("Unsupported horizontal unit".into());
        };
        let (y_unit, x_unit, x_step) = (*y_unit, *x_unit, *x_step);
        let y_step = if form == GridForm::Rectangular {
            f64::from_bits(x_step.to_bits().saturating_sub(1))
        } else {
            x_step
        };
        if !y_step.is_finite() || y_step <= 0.0 {
            return Err("Invalid grid spacing".into());
        }
        self.checkpoint();
        self.objects[y_unit - 1].kind = Kind::GraphVerticalUnit(x_unit, y_step);
        self.grid_form[self.active_page] = form;
        Ok(())
    }
    pub fn axis_trigonometric(&self) -> bool {
        self.active_x_axis(self.active_page)
            .and_then(|id| self.objects.get(id.checked_sub(1)?))
            .is_some_and(|object| matches!(object.kind, Kind::GraphAxis(_, _, 0x1_0001)))
    }
    pub fn set_axis_trigonometric(&mut self, enabled: bool) -> Result<(), String> {
        let x_axis = self
            .active_x_axis(self.active_page)
            .ok_or("No coordinate system")?;
        let axis_index = x_axis.checked_sub(1).ok_or("Invalid horizontal axis")?;
        let Kind::GraphAxis(origin, unit, axis) = self
            .objects
            .get(axis_index)
            .ok_or("Invalid horizontal axis")?
            .kind
        else {
            return Err("Invalid horizontal axis".into());
        };
        if !matches!(axis, 1 | 0x1_0001) {
            return Err("Unsupported horizontal axis".into());
        }
        if self.axis_trigonometric() != enabled {
            self.checkpoint();
            self.objects[axis_index].kind =
                Kind::GraphAxis(origin, unit, if enabled { 0x1_0001 } else { 1 });
        }
        Ok(())
    }
    pub fn graph_frame(&self, id: usize) -> Option<CoordinateFrame> {
        let Kind::CoordinateSystem(x_axis, y_axis) = self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let Kind::GraphAxis(origin, x_unit, 1 | 0x1_0001) =
            self.objects.get(x_axis.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let Kind::GraphAxis(y_origin, y_unit, 0) = self.objects.get(y_axis.checked_sub(1)?)?.kind
        else {
            return None;
        };
        if origin != y_origin {
            return None;
        }
        let map = |p: Point| GraphPoint { x: p.x, y: p.y };
        Some(CoordinateFrame {
            origin: map(self.position(origin)?),
            x_unit: map(self.position(x_unit)?),
            y_unit: map(self.position(y_unit)?),
        })
    }
    pub fn slope_value(&self, id: usize) -> Option<f64> {
        let Kind::SlopeMeasurement(line, frame) = self.objects.get(id.checked_sub(1)?)?.kind else {
            return None;
        };
        let (a, b, _) = self.line_points(line)?;
        self.graph_frame(frame)?
            .slope(GraphPoint { x: a.x, y: a.y }, GraphPoint { x: b.x, y: b.y })
    }
    pub fn coordinate_measure_value(&self, id: usize) -> Option<CoordinateMeasureValue> {
        let Kind::CoordinateMeasurement {
            kind,
            first,
            second,
            frame,
        } = self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let graph_frame = self.graph_frame(frame)?;
        let to_screen = |point: Point| ScreenPoint {
            x: point.x,
            y: point.y,
        };
        let basis = CoordinateBasis {
            origin: to_screen(Point {
                x: graph_frame.origin.x,
                y: graph_frame.origin.y,
            }),
            x_unit: to_screen(Point {
                x: graph_frame.x_unit.x,
                y: graph_frame.x_unit.y,
            }),
            y_unit: to_screen(Point {
                x: graph_frame.y_unit.x,
                y: graph_frame.y_unit.y,
            }),
        };
        coordinate_measure_records::evaluate(
            kind,
            basis,
            to_screen(self.position(first)?),
            second.and_then(|id| self.position(id)).map(to_screen),
        )
    }
    pub fn coordinate_measure_text(&self, id: usize) -> Option<String> {
        let Kind::CoordinateMeasurement { kind, first, .. } =
            self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let name = self.coordinate_point_name(first)?;
        let value = self.coordinate_measure_value(id)?;
        match (kind, value) {
            (CoordinateMeasureKind::Coordinates, CoordinateMeasureValue::Pair(p)) => {
                Some(format!("{name}: ({:.2}, {:.2})", p.x, p.y))
            }
            (CoordinateMeasureKind::Abscissa, CoordinateMeasureValue::Scalar(x)) => {
                Some(format!("x_{name} = {x:.2}"))
            }
            (CoordinateMeasureKind::Ordinate, CoordinateMeasureValue::Scalar(y)) => {
                Some(format!("y_{name} = {y:.2}"))
            }
            (CoordinateMeasureKind::CoordinateDistance, CoordinateMeasureValue::Scalar(d)) => {
                Some(format!("Coordinate Distance = {d:.2}"))
            }
            _ => None,
        }
    }
    pub fn value_equation_value(&self, id: usize) -> Option<EquationValue> {
        let Kind::ValueEquation {
            kind,
            first,
            second,
        } = self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let screen = |point: Point| value_equation_records::ScreenPoint {
            x: point.x,
            y: point.y,
        };
        match kind {
            ValueEquationKind::PointValue => {
                let point = screen(self.position(first)?);
                match self.objects.get(second - 1)?.kind {
                    Kind::Segment(_, _) => {
                        let (a, b, _) = self.line_points(second)?;
                        value_equation_records::segment_point_value(point, screen(a), screen(b))
                    }
                    Kind::Circle(_, _) => {
                        let Kind::PointOnCircle(host, x, y) = self.objects.get(first - 1)?.kind
                        else {
                            return None;
                        };
                        (host == second)
                            .then(|| value_equation_records::circle_point_value(x, y))
                            .flatten()
                    }
                    _ => None,
                }
            }
            ValueEquationKind::LineEquation | ValueEquationKind::CircleEquation => {
                let graph = self.graph_frame(second)?;
                let frame = CartesianFrame {
                    origin: value_equation_records::ScreenPoint {
                        x: graph.origin.x,
                        y: graph.origin.y,
                    },
                    x_unit: value_equation_records::ScreenPoint {
                        x: graph.x_unit.x,
                        y: graph.x_unit.y,
                    },
                    y_unit: value_equation_records::ScreenPoint {
                        x: graph.y_unit.x,
                        y: graph.y_unit.y,
                    },
                };
                match kind {
                    ValueEquationKind::LineEquation => {
                        let (a, b, _) = self.line_points(first)?;
                        value_equation_records::line_equation(frame, screen(a), screen(b))
                    }
                    ValueEquationKind::CircleEquation => {
                        let (center, radius) = self.circle_points(first)?;
                        value_equation_records::circle_equation(
                            frame,
                            screen(center),
                            screen(Point {
                                x: center.x + radius,
                                y: center.y,
                            }),
                        )
                    }
                    ValueEquationKind::PointValue => unreachable!(),
                }
            }
        }
    }
    pub fn value_equation_text(&self, id: usize) -> Option<String> {
        let value = self.value_equation_value(id)?;
        let signed = |number: f64| {
            if number < 0. {
                format!("−{:.2}", -number)
            } else {
                format!("{number:.2}")
            }
        };
        let shifted = |number: f64| {
            if number < 0. {
                format!("+ {:.2}", -number)
            } else {
                format!("− {number:.2}")
            }
        };
        Some(match value {
            EquationValue::PointValue(t) => format!("Value of Point = {t:.2}"),
            EquationValue::Line { slope, intercept } => {
                let sign = if intercept < 0. { "−" } else { "+" };
                format!("y = {}x {sign} {:.2}", signed(slope), intercept.abs())
            }
            EquationValue::VerticalLine { x } => format!("x = {}", signed(x)),
            EquationValue::Circle { center, radius } => format!(
                "(x {})² + (y {})² = {radius:.2}²",
                shifted(center.x),
                shifted(center.y)
            ),
        })
    }
    fn coordinate_point_name(&self, first: usize) -> Option<String> {
        Some(
            self.objects
                .get(first - 1)?
                .label
                .as_ref()
                .and_then(|label| label.name_utf8().ok())
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    let ordinal = self
                        .objects
                        .iter()
                        .take(first)
                        .filter(|object| is_point_kind(&object.kind))
                        .count();
                    if (1..=26).contains(&ordinal) {
                        ((b'A' + ordinal as u8 - 1) as char).to_string()
                    } else {
                        format!("Point {first}")
                    }
                }),
        )
    }
    pub fn dependencies_of(&self, id: usize) -> Option<Vec<usize>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let mut dependencies = object.kind.dependencies();
        if let Some(host) = object.midpoint_host {
            dependencies.push(host);
        }
        Some(dependencies)
    }
    pub fn copy_selected(&self) -> Result<Clipboard, String> {
        let mut included = BTreeSet::new();
        let mut pending: Vec<usize> = self.selected.iter().copied().collect();
        if pending.is_empty() {
            return Err("Select objects to copy".into());
        }
        while let Some(id) = pending.pop() {
            if !included.insert(id) {
                continue;
            }
            let object = self
                .objects
                .get(id.checked_sub(1).ok_or("Invalid selection")?)
                .ok_or("Invalid selection")?;
            if object.deleted
                || object.page != self.active_page
                || matches!(object.kind, Kind::Unsupported(_))
            {
                return Err(format!("Object {id} cannot be copied safely"));
            }
            pending.extend(self.dependencies_of(id).ok_or("Invalid selection")?);
        }
        Ok(Clipboard {
            objects: included
                .into_iter()
                .map(|id| {
                    let object = &self.objects[id - 1];
                    (
                        id,
                        object.kind.clone(),
                        object.visible,
                        object.tracing,
                        object.midpoint_host,
                        object.visual_style,
                    )
                })
                .collect(),
        })
    }
    pub fn paste(&mut self, clipboard: &Clipboard) -> Result<Vec<usize>, String> {
        if clipboard.is_empty() {
            return Err("Clipboard is empty".into());
        }
        if !self.safe_structure || self.page_count != 1 {
            return Err("Pasting objects in this document cannot be saved safely".into());
        }
        let mut candidate = self.clone();
        candidate.checkpoint();
        candidate.selected.clear();
        let max_source = clipboard.objects.last().ok_or("Clipboard is empty")?.0;
        let mut ids = vec![0; max_source + 1];
        let mut created = Vec::with_capacity(clipboard.len());
        for (old_id, kind, visible, tracing, midpoint_host, visual_style) in &clipboard.objects {
            if kind
                .dependencies()
                .into_iter()
                .chain(*midpoint_host)
                .any(|id| id >= ids.len() || ids[id] == 0)
            {
                return Err("Clipboard has an unresolved dependency".into());
            }
            let mut remapped = kind.remap(&ids);
            if let Kind::Free(ref mut point) = remapped {
                point.x += 20.;
                point.y += 20.;
            }
            let id = candidate.objects.len() + 1;
            candidate.objects.push(Object {
                id,
                page: candidate.active_page,
                kind: remapped,
                visible: *visible,
                tracing: *tracing,
                deleted: false,
                midpoint_host: midpoint_host.map(|host| ids[host]),
                point_offset: None,
                constraint_offset: None,
                header_offset: None,
                label: None,
                visual_style: *visual_style,
            });
            ids[*old_id] = id;
            created.push(id);
        }
        candidate.selected.extend(created.iter().copied());
        candidate.encode()?;
        *self = candidate;
        Ok(created)
    }
    pub fn clear_selected(&mut self) -> Result<Vec<usize>, String> {
        let mut removed: BTreeSet<usize> = self
            .objects
            .iter()
            .filter(|o| !o.deleted && self.selected.contains(&o.id))
            .map(|o| o.id)
            .collect();
        if removed.is_empty() {
            return Ok(vec![]);
        }
        if !self.safe_structure || self.page_count != 1 {
            return Err("Deleting objects in this document cannot be saved safely".into());
        }
        if removed
            .iter()
            .any(|id| matches!(self.objects[id - 1].kind, Kind::Image { .. }))
        {
            return Err("Removing a picture resource is not yet supported".into());
        }
        loop {
            let before = removed.len();
            for object in self.objects.iter().filter(|o| !o.deleted) {
                if self
                    .dependencies_of(object.id)
                    .unwrap_or_default()
                    .iter()
                    .any(|id| removed.contains(id))
                {
                    removed.insert(object.id);
                }
            }
            if removed.len() == before {
                break;
            }
        }
        self.checkpoint();
        for object in &mut self.objects {
            if removed.contains(&object.id) {
                object.deleted = true;
            }
        }
        if !self.objects.iter().any(|object| {
            object.page == self.active_page
                && !object.deleted
                && matches!(object.kind, Kind::CoordinateSystem(_, _))
        }) {
            self.grid_dotted[self.active_page] = false;
            self.snap_points[self.active_page] = false;
            self.grid_form[self.active_page] = GridForm::Square;
        }
        self.selected.clear();
        Ok(removed.into_iter().collect())
    }
    pub fn set_selected_visibility(&mut self, visible: bool) {
        if !self
            .objects
            .iter()
            .any(|o| !o.deleted && self.selected.contains(&o.id) && o.visible != visible)
        {
            return;
        }
        self.checkpoint();
        for o in &mut self.objects {
            if !o.deleted && self.selected.contains(&o.id) {
                o.visible = visible;
            }
        }
        if !visible {
            self.selected.clear();
        }
    }
    pub fn set_selected_tracing(&mut self, tracing: bool) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot edit tracing in this sketch safely".into());
        }
        let ids: Vec<usize> = self
            .selected
            .iter()
            .copied()
            .filter(|&id| {
                self.objects.get(id - 1).is_some_and(|o| {
                    !o.deleted
                        && o.page == self.active_page
                        && (is_point_kind(&o.kind) || matches!(o.kind, Kind::Segment(_, _)))
                        && o.tracing != tracing
                })
            })
            .collect();
        if !ids.is_empty() {
            self.checkpoint();
            for id in &ids {
                self.objects[*id - 1].tracing = tracing;
            }
        }
        Ok(ids.len())
    }
    fn update_selected_visual_style(
        &mut self,
        update: impl Fn(VisualStyle) -> Option<VisualStyle>,
    ) -> Result<usize, String> {
        if self.partial_view {
            return Err("Cannot edit styles in a sketch with unsupported records safely".into());
        }
        let changes: Vec<_> = self
            .objects
            .iter()
            .filter_map(|object| {
                if object.deleted
                    || object.page != self.active_page
                    || !self.selected.contains(&object.id)
                {
                    return None;
                }
                let old = object.visual_style?;
                let new = update(old)?;
                (new != old).then_some((object.id, new))
            })
            .collect();
        if !changes.is_empty() {
            self.checkpoint();
            for (id, style) in &changes {
                self.objects[*id - 1].visual_style = Some(*style);
            }
        }
        Ok(changes.len())
    }
    pub fn set_selected_point_style(&mut self, shape: PointStyle) -> Result<usize, String> {
        self.update_selected_visual_style(|style| match style {
            VisualStyle::Point { color, .. } => Some(VisualStyle::Point { shape, color }),
            _ => None,
        })
    }
    pub fn set_selected_line_width(&mut self, width: LineWidth) -> Result<usize, String> {
        self.update_selected_visual_style(|style| match style {
            VisualStyle::Stroke { line, color } => Some(VisualStyle::Stroke {
                line: SegmentLineStyle { width, ..line },
                color,
            }),
            _ => None,
        })
    }
    pub fn set_selected_line_pattern(&mut self, pattern: LinePattern) -> Result<usize, String> {
        self.update_selected_visual_style(|style| match style {
            VisualStyle::Stroke { line, color } => Some(VisualStyle::Stroke {
                line: SegmentLineStyle { pattern, ..line },
                color,
            }),
            _ => None,
        })
    }
    pub fn set_selected_color(&mut self, color: RgbColor) -> Result<usize, String> {
        self.update_selected_visual_style(|style| {
            Some(match style {
                VisualStyle::Point { shape, .. } => VisualStyle::Point { shape, color },
                VisualStyle::Stroke { line, .. } => VisualStyle::Stroke { line, color },
                VisualStyle::Polygon { .. } => VisualStyle::Polygon { color },
            })
        })
    }
    pub fn show_all_hidden(&mut self) {
        if !self
            .objects
            .iter()
            .any(|o| !o.deleted && o.page == self.active_page && !o.visible)
        {
            return;
        }
        self.checkpoint();
        for o in &mut self.objects {
            if !o.deleted && o.page == self.active_page {
                o.visible = true;
            }
        }
    }
    pub fn add_hide_show_button(&mut self, target: usize) -> Result<usize, String> {
        let object = self
            .objects
            .get(target.checked_sub(1).ok_or("Invalid button target")?)
            .ok_or("Invalid button target")?;
        if self.partial_view
            || object.deleted
            || !object.visible
            || object.page != self.active_page
            || matches!(
                object.kind,
                Kind::HideShowButton { .. } | Kind::Unsupported(_)
            )
        {
            return Err("A Hide/Show button needs a visible supported object on this page".into());
        }
        let row = self
            .objects
            .iter()
            .filter(|object| {
                object.page == self.active_page
                    && !object.deleted
                    && matches!(object.kind, Kind::HideShowButton { .. })
            })
            .count();
        let y = (5usize + row * 30).min(i16::MAX as usize) as i16;
        let label = if matches!(object.kind, Kind::Segment(_, _)) {
            "Hide Segment"
        } else {
            "Hide Objects"
        };
        Ok(self.add(Kind::HideShowButton {
            target,
            action: VisibilityAction::Hide,
            label: label.into(),
            anchor: (10, y),
        }))
    }
    pub fn activate_hide_show_button(&mut self, id: usize) -> Result<(), String> {
        let button = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid button ID")?)
            .ok_or("Invalid button ID")?;
        if button.deleted || !button.visible || button.page != self.active_page {
            return Err("Button is not visible on this page".into());
        }
        let Kind::HideShowButton { target, action, .. } = &button.kind else {
            return Err("Object is not a Hide/Show button".into());
        };
        let (target, action) = (*target, *action);
        if self
            .objects
            .get(target - 1)
            .is_none_or(|object| object.deleted || object.page != self.active_page)
        {
            return Err("Button target is unavailable".into());
        }
        self.checkpoint();
        self.objects[target - 1].visible = action == VisibilityAction::Show;
        if let Kind::HideShowButton {
            action: current,
            label,
            ..
        } = &mut self.objects[id - 1].kind
        {
            *current = match action {
                VisibilityAction::Hide => VisibilityAction::Show,
                VisibilityAction::Show => VisibilityAction::Hide,
            };
            *label = match label.as_str() {
                "Hide Segment" => "Show Segment".into(),
                "Show Segment" => "Hide Segment".into(),
                "Hide Objects" => "Show Objects".into(),
                "Show Objects" => "Hide Objects".into(),
                "隐藏线段" => "显示线段".into(),
                "显示线段" => "隐藏线段".into(),
                _ => label.clone(),
            };
        }
        self.selected.clear();
        Ok(())
    }
    pub fn position(&self, id: usize) -> Option<Point> {
        fn go(d: &Document, id: usize, seen: &mut BTreeSet<usize>) -> Option<Point> {
            fn on_path(
                d: &Document,
                path: usize,
                t: f64,
                seen: &mut BTreeSet<usize>,
            ) -> Option<Point> {
                let kind = &d.objects.get(path.checked_sub(1)?)?.kind;
                if !plot_path_accepts(kind, t) {
                    return None;
                }
                if is_circle_kind(kind) {
                    let (center, radius) = d.circle_points(path)?;
                    let angle = t * std::f64::consts::TAU;
                    let point = Point {
                        x: center.x + radius * angle.cos(),
                        y: center.y - radius * angle.sin(),
                    };
                    return (point.x.is_finite() && point.y.is_finite()).then_some(point);
                }
                let (start, end) = match kind {
                    Kind::GraphAxis(a, b, 0 | 1 | 0x1_0001)
                    | Kind::Segment(a, b)
                    | Kind::Line(a, b)
                    | Kind::Ray(a, b) => (*a, *b),
                    _ => return None,
                };
                let start = go(d, start, seen)?;
                let end = go(d, end, seen)?;
                let point = Point {
                    x: start.x + (end.x - start.x) * t,
                    y: start.y + (end.y - start.y) * t,
                };
                (point.x.is_finite() && point.y.is_finite()).then_some(point)
            }
            if !seen.insert(id) {
                return None;
            }
            let object = d.objects.get(id.checked_sub(1)?)?;
            if object.deleted {
                return None;
            }
            let p = match &object.kind {
                Kind::Free(p) => Some(*p),
                Kind::GraphHorizontalUnit(origin, step) => {
                    let origin = go(d, *origin, seen)?;
                    Some(Point {
                        x: origin.x + step,
                        y: origin.y,
                    })
                }
                Kind::GraphVerticalUnit(horizontal_unit, step) => {
                    let Kind::GraphHorizontalUnit(origin, _) =
                        d.objects.get(horizontal_unit.checked_sub(1)?)?.kind
                    else {
                        return None;
                    };
                    let origin = go(d, origin, seen)?;
                    Some(Point {
                        x: origin.x,
                        y: origin.y - step,
                    })
                }
                Kind::PathLiteralPoint(path, t) => on_path(d, *path, *t, seen),
                Kind::PathValuePoint(source, path) => {
                    let Kind::Parameter(ref value) = d.objects.get(source.checked_sub(1)?)?.kind
                    else {
                        return None;
                    };
                    on_path(d, *path, value.value(), seen)
                }
                Kind::PlottedPoint(_, frame, mode, stored) => {
                    let frame = d.graph_frame(*frame)?;
                    let [x, y] = match mode {
                        PlotPointsMode::Cartesian => *stored,
                        PlotPointsMode::Polar => {
                            [stored[0] * stored[1].cos(), stored[0] * stored[1].sin()]
                        }
                    };
                    let point = frame.world(x, y)?;
                    Some(Point {
                        x: point.x,
                        y: point.y,
                    })
                }
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
    pub fn segment_length_cm(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::SegmentLength(segment) = object.kind else {
            return None;
        };
        let (a, b, LineMode::Segment) = self.line_points(segment)? else {
            return None;
        };
        Some(a.distance(b) / 37.79527559055118)
    }
    pub fn point_distance_cm(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::PointDistance(a, b) = object.kind else {
            return None;
        };
        Some(self.position(a)?.distance(self.position(b)?) / 37.79527559055118)
    }
    pub fn ratio_for_points(&self, a: usize, b: usize, c: usize) -> Option<f64> {
        let (a, b, c) = (self.position(a)?, self.position(b)?, self.position(c)?);
        let (bx, by) = (b.x - a.x, b.y - a.y);
        let denominator = bx.mul_add(bx, by * by);
        if !denominator.is_finite() || denominator <= 0.0 {
            return None;
        }
        let value = ((c.x - a.x) * bx + (c.y - a.y) * by) / denominator;
        value.is_finite().then_some(value)
    }
    pub fn ratio_value(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::PointRatio(a, b, c) = object.kind else {
            return None;
        };
        self.ratio_for_points(a, b, c)
    }
    pub fn polygon_points(&self, id: usize) -> Option<Vec<Point>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::Polygon(vertices) = &object.kind else {
            return None;
        };
        vertices
            .iter()
            .map(|&vertex| self.position(vertex))
            .collect()
    }
    pub fn polygon_perimeter_cm(&self, id: usize) -> Option<f64> {
        let points = self.polygon_points(id)?;
        if points.len() < 3 {
            return None;
        }
        let perimeter = points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .take(points.len())
            .map(|(a, b)| a.distance(*b))
            .sum::<f64>();
        perimeter
            .is_finite()
            .then_some(perimeter / 37.79527559055118)
    }
    pub fn perimeter_cm(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::PolygonPerimeter(polygon) = object.kind else {
            return None;
        };
        self.polygon_perimeter_cm(polygon)
    }
    pub fn circle_circumference_cm(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::CircleCircumference(circle) = object.kind else {
            return None;
        };
        let (_, radius) = self.circle_points(circle)?;
        Some(std::f64::consts::TAU * radius / 37.79527559055118)
    }
    pub fn circle_radius_cm(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::CircleRadius(circle) = object.kind else {
            return None;
        };
        let (_, radius) = self.circle_points(circle)?;
        Some(radius / 37.79527559055118)
    }
    pub fn circle_area_cm2(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::CircleArea(circle) = object.kind else {
            return None;
        };
        let (_, radius) = self.circle_points(circle)?;
        let cm = radius / 37.79527559055118;
        Some(std::f64::consts::PI * cm * cm)
    }
    pub fn arc_angle_degrees(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::ArcAngle(arc) = object.kind else {
            return None;
        };
        Some(self.arc_geometry(arc)?.3.abs().to_degrees())
    }
    pub fn arc_length_cm(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::ArcLength(arc) = object.kind else {
            return None;
        };
        let (_, radius, _, sweep, _) = self.arc_geometry(arc)?;
        Some(radius * sweep.abs() / 37.79527559055118)
    }
    pub fn angle_degrees(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::AngleMeasurement(a, vertex, c) = object.kind else {
            return None;
        };
        self.angle_for_points(a, vertex, c)
    }
    pub fn angle_for_points(&self, a: usize, vertex: usize, c: usize) -> Option<f64> {
        let a = self.position(a)?;
        let vertex = self.position(vertex)?;
        let c = self.position(c)?;
        let (ax, ay) = (a.x - vertex.x, a.y - vertex.y);
        let (cx, cy) = (c.x - vertex.x, c.y - vertex.y);
        let (al, cl) = (ax.hypot(ay), cx.hypot(cy));
        if !al.is_finite() || !cl.is_finite() || al == 0. || cl == 0. {
            return None;
        }
        let (ax, ay, cx, cy) = (ax / al, ay / al, cx / cl, cy / cl);
        Some(
            (ax * cy - ay * cx)
                .abs()
                .atan2(ax * cx + ay * cy)
                .to_degrees(),
        )
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
            let object = d.objects.get(id.checked_sub(1)?)?;
            if object.deleted {
                return None;
            }
            let result = match object.kind {
                Kind::Segment(a, b) => Some((d.position(a)?, d.position(b)?, LineMode::Segment)),
                Kind::GraphAxis(a, b, _) => Some((d.position(a)?, d.position(b)?, LineMode::Line)),
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
        if self.objects.get(id.checked_sub(1)?)?.deleted {
            return None;
        }
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
            Some(Kind::Free(_)) => Kind::Free(self.snap_point(p)),
            Some(Kind::GraphVerticalUnit(horizontal_unit, _))
                if self.grid_form[self.active_page] == GridForm::Rectangular =>
            {
                let horizontal_unit = *horizontal_unit;
                let Some(Kind::GraphHorizontalUnit(origin, _)) = horizontal_unit
                    .checked_sub(1)
                    .and_then(|at| self.objects.get(at))
                    .map(|object| &object.kind)
                else {
                    return false;
                };
                let Some(origin) = self.position(*origin) else {
                    return false;
                };
                let step = origin.y - p.y;
                if !step.is_finite() || step <= 0.0 {
                    return false;
                }
                Kind::GraphVerticalUnit(horizontal_unit, step)
            }
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
    fn compact_deleted(&self, bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        if !self.objects.iter().any(|o| o.deleted) {
            return Ok(bytes);
        }
        let mut ids = vec![0; self.objects.len() + 1];
        let mut kept = 0;
        for object in &self.objects {
            if !object.deleted {
                kept += 1;
                ids[object.id] = kept;
            }
        }
        let rs = records(&bytes)?;
        let mut out = bytes[..4].to_vec();
        let mut group_id = 0;
        let mut dropping = false;
        for (index, record) in rs.iter().enumerate() {
            if record.kind == 0x7d0 {
                group_id += 1;
                dropping = self
                    .objects
                    .get(group_id - 1)
                    .ok_or("Object group count mismatch")?
                    .deleted;
            }
            if !dropping && record.kind != 0x2328 {
                let end = rs.get(index + 1).map_or(bytes.len(), |next| next.start);
                let mut raw = bytes[record.start..end].to_vec();
                if record.kind == 0x232d {
                    if record.data.len() != 4 {
                        return Err("Invalid coordinate-system page link".into());
                    }
                    let old = u32_at(&bytes, record.data.start) as usize;
                    let new = *ids.get(old).ok_or("Invalid coordinate-system page link")?;
                    if new == 0 {
                        continue;
                    }
                    raw[8..12].copy_from_slice(&(new as u32).to_le_bytes());
                }
                if record.kind == 0x3e8 {
                    if record.data.len() < 28 {
                        return Err("Short page preamble".into());
                    }
                    raw[32..36].copy_from_slice(&(kept as u32).to_le_bytes());
                }
                if record.kind == 0x7d2 {
                    let payload = &bytes[record.data.clone()];
                    if payload.len() < 4
                        || (payload.len() - 4) % 4 != 0
                        || u32_at(payload, 0) as usize != (payload.len() - 4) / 4
                    {
                        return Err("Invalid object references during deletion".into());
                    }
                    for j in 0..(payload.len() - 4) / 4 {
                        let old = u32_at(payload, 4 + j * 4) as usize;
                        let new = *ids
                            .get(old)
                            .filter(|&&id| id > 0)
                            .ok_or("Surviving object refers to a deleted object")?;
                        raw[12 + j * 4..16 + j * 4].copy_from_slice(&(new as u32).to_le_bytes());
                    }
                }
                out.extend_from_slice(&raw);
            }
            if record.kind == 0x7d7 {
                dropping = false;
            }
        }
        if group_id != self.objects.len() {
            return Err("Object group count mismatch".into());
        }
        records(&out)?;
        Ok(out)
    }
    pub fn can_save(&self) -> Result<(), String> {
        // An untouched source document can be copied byte for byte even when
        // its geometry cannot yet be evaluated by this port. Revalidate once
        // anything has changed before serializing an edited document.
        if !self.dirty {
            return Ok(());
        }
        if self.image_read_only && self.dirty {
            return Err("Cropped or multi-resource picture content is read-only".into());
        }
        if self.snap_points.len() != self.page_count
            || self.grid_dotted.len() != self.page_count
            || self.grid_form.len() != self.page_count
        {
            return Err("Invalid page grid state".into());
        }
        if self.objects.len() < self.source_count {
            return Err("Removing saved objects is not yet supported by the GSP writer".into());
        }
        if self.objects.iter().any(|o| o.deleted) && (!self.safe_structure || self.page_count != 1)
        {
            return Err("Deleting objects in this document cannot be saved safely".into());
        }
        if self.objects.iter().any(|o| o.deleted)
            && self.objects.iter().any(|o| {
                matches!(
                    o.kind,
                    Kind::FunctionDefinition(_)
                        | Kind::FunctionPlot(_, _, _, _, _)
                        | Kind::PathLiteralPoint(_, _)
                        | Kind::PathValuePoint(_, _)
                        | Kind::PlottedPoint(_, _, _, _)
                )
            })
        {
            return Err(
                "Deleting objects in a plot or function document cannot be saved safely".into(),
            );
        }
        if self
            .objects
            .iter()
            .any(|o| o.deleted && matches!(o.kind, Kind::Image { .. }))
        {
            return Err("Removing a picture resource is not yet supported".into());
        }
        if self
            .objects
            .iter()
            .filter(|o| matches!(o.kind, Kind::Image { .. }))
            .count()
            > 1
        {
            return Err("Only one independent picture can be written".into());
        }
        for (i, o) in self.objects.iter().enumerate() {
            if o.id != i + 1 {
                return Err("Invalid object ID".into());
            }
            if o.deleted {
                continue;
            }
            if let Kind::Free(point) = o.kind
                && (!point.x.is_finite() || !point.y.is_finite())
            {
                return Err(format!("Non-finite coordinates in object {}", o.id));
            }
            if let Kind::PlainText { text, rect } = &o.kind {
                text_object::encode_plain_text_group(text, *rect)?;
            }
            match &o.kind {
                Kind::Parameter(group) => {
                    ParameterGroup::parse(group.as_bytes())?;
                }
                Kind::Calculation(group) => {
                    CalculationGroup::parse(group.as_bytes())?;
                }
                Kind::ValueTable(source, group) => {
                    TableGroup::parse(group.as_bytes())?;
                    if *source != 1
                        || group.source_id() != 1
                        || !matches!(
                            self.objects.get(source - 1).map(|o| &o.kind),
                            Some(Kind::Parameter(_))
                        )
                    {
                        return Err("Unsupported table source".into());
                    }
                }
                Kind::ValueTablePair(first, second, group) => {
                    TableGroup::parse_pair(group.as_bytes())?;
                    if (*first, *second) != (2, 1)
                        || group.pair_source_ids() != Some([2, 1])
                        || !matches!(self.objects.first().map(|o| &o.kind), Some(Kind::Parameter(parameter)) if parameter.name() == "t[1]")
                        || !matches!(self.objects.get(1).map(|o| &o.kind), Some(Kind::Parameter(parameter)) if parameter.name() == "t[2]")
                    {
                        return Err("Unsupported two-column table source".into());
                    }
                }
                _ => {}
            }
            if let Kind::Image { png, transform } = &o.kind {
                ImageFile::png_size(png)?;
                if transform.iter().any(|value| !value.is_finite()) {
                    return Err(format!("Non-finite picture transform in object {}", o.id));
                }
                if o.id <= self.source_count {
                    let source = ImageFile::parse(&self.original)?;
                    if source.png() != png {
                        return Err("Replacing a saved picture's PNG is not supported".into());
                    }
                }
            }
            if o.label.is_some() && !is_point_kind(&o.kind) {
                return Err(format!("Invalid point label in object {}", o.id));
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
            if refs
                .iter()
                .any(|&r| r == 0 || r >= o.id || self.objects[r - 1].deleted)
            {
                return Err(format!("Invalid dependency in object {}", o.id));
            }
            if let Kind::HideShowButton { target, label, .. } = &o.kind {
                if self.page_count != 1
                    || self.objects[*target - 1].page != o.page
                    || label.len() > u16::MAX as usize
                {
                    return Err(format!("Invalid Hide/Show button {}", o.id));
                }
            }
            if let Kind::Midpoint(a, b) = o.kind {
                let Some(host) = o.midpoint_host else {
                    return Err(format!("Midpoint {} has no segment host", o.id));
                };
                if host == 0
                    || host >= o.id
                    || self.objects[host - 1].deleted
                    || !matches!(self.objects[host - 1].kind, Kind::Segment(x, y) if (x == a && y == b) || (x == b && y == a))
                {
                    return Err(format!("Invalid segment host in midpoint {}", o.id));
                }
            }
            if i >= self.source_count
                && let Kind::Polygon(vertices) = &o.kind
            {
                if vertices.len() < 3
                    || vertices.iter().collect::<BTreeSet<_>>().len() != vertices.len()
                    || self.polygon_points(o.id).is_none()
                {
                    return Err(format!("Invalid polygon {}", o.id));
                }
            }
            if matches!(o.kind, Kind::SegmentLength(_))
                && self
                    .segment_length_cm(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid segment in length measurement {}", o.id));
            }
            if matches!(o.kind, Kind::PointDistance(_, _))
                && self
                    .point_distance_cm(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid points in distance measurement {}", o.id));
            }
            if matches!(o.kind, Kind::PointRatio(_, _, _)) && self.ratio_value(o.id).is_none() {
                return Err(format!("Invalid points in ratio measurement {}", o.id));
            }
            if matches!(o.kind, Kind::GraphHorizontalUnit(_, step) | Kind::GraphVerticalUnit(_, step) if !step.is_finite() || step <= 0.0)
                || matches!(o.kind, Kind::GraphAxis(_, _, axis) if !matches!(axis, 0 | 1 | 0x1_0001))
            {
                return Err(format!("Invalid graph object {}", o.id));
            }
            if matches!(
                o.kind,
                Kind::GraphHorizontalUnit(_, _) | Kind::GraphVerticalUnit(_, _)
            ) && self
                .position(o.id)
                .is_none_or(|p| !p.x.is_finite() || !p.y.is_finite())
            {
                return Err(format!("Undefined graph point {}", o.id));
            }
            if matches!(o.kind, Kind::GraphAxis(_, _, _)) && self.line_points(o.id).is_none() {
                return Err(format!("Undefined graph axis {}", o.id));
            }
            if matches!(o.kind, Kind::CoordinateSystem(_, _)) && self.graph_frame(o.id).is_none() {
                return Err(format!("Invalid coordinate system {}", o.id));
            }
            if let Kind::FunctionPlot(definition, frame, min, max, samples) = o.kind {
                if !matches!(
                    self.objects
                        .get(definition.saturating_sub(1))
                        .map(|o| &o.kind),
                    Some(Kind::FunctionDefinition(_))
                ) || self.graph_frame(frame).is_none()
                    || !min.is_finite()
                    || !max.is_finite()
                    || min >= max
                    || !(2..=4096).contains(&samples)
                {
                    return Err(format!("Invalid function plot {}", o.id));
                }
                if o.id > self.source_count
                    && (min != -5.265208333333335 || max != 5.265208333333335 || samples != 750)
                {
                    return Err(format!(
                        "Cannot encode modified function range in object {}",
                        o.id
                    ));
                }
            }
            if let Kind::PlottedPoint(source, frame, mode, stored) = o.kind {
                if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Parameter(group)) if group.name() == "t[1]" && group.value() == 1.0)
                    || self.graph_frame(frame).is_none()
                    || stored.iter().any(|value| !value.is_finite())
                    || self.position(o.id).is_none()
                {
                    return Err(format!("Invalid plotted point {}", o.id));
                }
                if o.id > self.source_count {
                    let group = plot_points_records::encode_group(
                        mode,
                        [source as u32, frame as u32],
                        stored,
                    )?;
                    if plot_points_records::decode_group(&group)?.stored != stored {
                        return Err(
                            "Only the observed t[1] Plot Points values can be written".into()
                        );
                    }
                }
            }
            if let Kind::PathValuePoint(source, path) = o.kind {
                if !matches!(
                    self.objects.get(source.saturating_sub(1)).map(|o| &o.kind),
                    Some(Kind::Parameter(_))
                ) || !self
                    .objects
                    .get(path.saturating_sub(1))
                    .is_some_and(|host| {
                        let Kind::Parameter(ref group) = self.objects[source - 1].kind else {
                            return false;
                        };
                        plot_path_accepts(&host.kind, group.value())
                    })
                    || self.position(o.id).is_none()
                {
                    return Err(format!("Invalid path value point {}", o.id));
                }
            }
            if let Kind::PathLiteralPoint(path, value) = o.kind {
                if !self
                    .objects
                    .get(path.saturating_sub(1))
                    .is_some_and(|host| plot_path_accepts(&host.kind, value))
                    || self.position(o.id).is_none()
                {
                    return Err(format!("Invalid literal path point {}", o.id));
                }
            }
            if let Kind::SlopeMeasurement(line, frame) = o.kind {
                if !matches!(
                    self.objects.get(line.saturating_sub(1)).map(|o| &o.kind),
                    Some(Kind::Segment(_, _) | Kind::Line(_, _) | Kind::Ray(_, _))
                ) || self.line_points(line).is_none()
                    || self.graph_frame(frame).is_none()
                {
                    return Err(format!("Invalid slope measurement {}", o.id));
                }
            }
            if let Kind::CoordinateMeasurement {
                kind,
                first,
                second,
                frame,
            } = o.kind
            {
                let point_ok = |id: usize| {
                    self.objects
                        .get(id.saturating_sub(1))
                        .is_some_and(|object| {
                            !object.deleted && object.page == o.page && is_point_kind(&object.kind)
                        })
                };
                if !point_ok(first)
                    || second.is_some_and(|id| !point_ok(id))
                    || (kind == CoordinateMeasureKind::CoordinateDistance) != second.is_some()
                    || !matches!(
                        self.objects
                            .get(frame.saturating_sub(1))
                            .map(|object| &object.kind),
                        Some(Kind::CoordinateSystem(_, _))
                    )
                    || self.coordinate_measure_value(o.id).is_none()
                {
                    return Err(format!("Invalid coordinate measurement {}", o.id));
                }
            }
            if let Kind::ValueEquation {
                kind,
                first,
                second,
            } = o.kind
            {
                let hosts_match = match kind {
                    ValueEquationKind::PointValue => {
                        matches!(self.objects.get(first.saturating_sub(1)).map(|object| &object.kind), Some(Kind::PointOnSegment(host, _) | Kind::PointOnCircle(host, _, _)) if *host == second)
                    }
                    ValueEquationKind::LineEquation => {
                        matches!(
                            self.objects
                                .get(first.saturating_sub(1))
                                .map(|object| &object.kind),
                            Some(Kind::Line(_, _))
                        ) && matches!(
                            self.objects
                                .get(second.saturating_sub(1))
                                .map(|object| &object.kind),
                            Some(Kind::CoordinateSystem(_, _))
                        )
                    }
                    ValueEquationKind::CircleEquation => {
                        matches!(
                            self.objects
                                .get(first.saturating_sub(1))
                                .map(|object| &object.kind),
                            Some(Kind::Circle(_, _))
                        ) && matches!(
                            self.objects
                                .get(second.saturating_sub(1))
                                .map(|object| &object.kind),
                            Some(Kind::CoordinateSystem(_, _))
                        )
                    }
                };
                if !hosts_match || self.value_equation_value(o.id).is_none() {
                    return Err(format!("Invalid Value/Equation measurement {}", o.id));
                }
            }
            if matches!(o.kind, Kind::PolygonPerimeter(_))
                && self
                    .perimeter_cm(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid polygon in perimeter measurement {}", o.id));
            }
            if matches!(o.kind, Kind::CircleCircumference(_))
                && self
                    .circle_circumference_cm(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!(
                    "Invalid circle in circumference measurement {}",
                    o.id
                ));
            }
            if matches!(o.kind, Kind::CircleRadius(_))
                && self
                    .circle_radius_cm(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid circle in radius measurement {}", o.id));
            }
            if matches!(o.kind, Kind::CircleArea(_))
                && self
                    .circle_area_cm2(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid circle in area measurement {}", o.id));
            }
            if matches!(o.kind, Kind::ArcAngle(_))
                && self
                    .arc_angle_degrees(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid arc in angle measurement {}", o.id));
            }
            if matches!(o.kind, Kind::ArcLength(_))
                && self
                    .arc_length_cm(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid arc in length measurement {}", o.id));
            }
            if matches!(o.kind, Kind::AngleMeasurement(_, _, _))
                && self
                    .angle_degrees(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid points in angle measurement {}", o.id));
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
    fn place_measurement(
        group: &mut [u8],
        format_offset: usize,
        row: &mut usize,
    ) -> Result<(), String> {
        let y = row
            .checked_mul(28)
            .and_then(|v| v.checked_add(5))
            .ok_or("Too many measurement labels")?;
        let y = u16::try_from(y).map_err(|_| "Too many measurement labels")?;
        group[format_offset + 14..format_offset + 16].copy_from_slice(&y.to_le_bytes());
        *row += 1;
        Ok(())
    }
    fn graph_template_group(&self, object: &Object) -> Result<Option<Vec<u8>>, String> {
        let first = match object.kind {
            Kind::Free(_) if self.objects.get(object.id).is_some_and(|next| matches!(next.kind, Kind::GraphHorizontalUnit(host, _) if host == object.id)) => Some(object.id),
            Kind::GraphHorizontalUnit(_, _) => object.id.checked_sub(1),
            Kind::GraphAxis(_, _, 1 | 0x1_0001) => object.id.checked_sub(2),
            Kind::GraphVerticalUnit(_, _) => object.id.checked_sub(3),
            Kind::GraphAxis(_, _, 0) => object.id.checked_sub(4),
            Kind::CoordinateSystem(_, _) => object.id.checked_sub(5),
            _ => None,
        };
        let Some(first) = first else {
            return Ok(None);
        };
        if first == 0 {
            return Err("Invalid coordinate-system object ID".into());
        }
        let [
            Some(origin),
            Some(x_unit),
            Some(x_axis),
            Some(y_unit),
            Some(y_axis),
            Some(frame),
        ] = self
            .objects
            .get(first - 1..first + 5)
            .map(|slice| {
                [
                    slice.get(0),
                    slice.get(1),
                    slice.get(2),
                    slice.get(3),
                    slice.get(4),
                    slice.get(5),
                ]
            })
            .unwrap_or([None; 6])
        else {
            return Err("Incomplete coordinate-system group".into());
        };
        let step: f64 = 37.79527559055118;
        let form = self.grid_form[object.page];
        let y_step = match y_unit.kind {
            Kind::GraphVerticalUnit(host, value)
                if host == first + 1
                    && value.is_finite()
                    && value > 0.0
                    && (form == GridForm::Rectangular || value == step) =>
            {
                value
            }
            _ => return Err("Unsupported vertical unit".into()),
        };
        if !matches!(origin.kind, Kind::Free(_))
            || !matches!(x_unit.kind, Kind::GraphHorizontalUnit(host, value) if host == first && value == step)
            || !matches!(x_axis.kind, Kind::GraphAxis(a,b,1 | 0x1_0001) if a == first && b == first+1)
            || !matches!(y_axis.kind, Kind::GraphAxis(a,b,0) if a == first && b == first+3)
            || !matches!(frame.kind, Kind::CoordinateSystem(a,b) if a == first+2 && b == first+4)
        {
            return Err("Unsupported coordinate-system construction".into());
        }
        let point = self
            .position(first)
            .ok_or("Undefined coordinate-system origin")?;
        let bytes = graph::coordinate_system_groups(
            first,
            GraphPoint {
                x: point.x,
                y: point.y,
            },
        )?;
        let offsets = [0, 68, 148, 268, 348, 468, 548];
        let index = object.id - first;
        let mut group = bytes[offsets[index]..offsets[index + 1]].to_vec();
        if object.visible {
            group[10] &= !1;
        } else {
            group[10] |= 1;
        }
        if matches!(object.kind, Kind::CoordinateSystem(_, _)) {
            graph::set_coordinate_system_style(&mut group, form, self.grid_dotted[object.page])?;
        }
        if matches!(object.kind, Kind::GraphVerticalUnit(_, _)) {
            graph::set_vertical_unit_form(&mut group, form, y_step)?;
        }
        if let Kind::GraphAxis(_, _, axis @ (1 | 0x1_0001)) = object.kind {
            graph::set_x_axis_trigonometric(&mut group, axis == 0x1_0001)?;
        }
        Ok(Some(group))
    }
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        if !self.dirty {
            return Ok(self.original.clone());
        }
        self.can_save()?;
        let rs = records(&self.original)?;
        let mut out = self.original.clone();
        for o in self.objects.iter().take(self.source_count) {
            if let Some(i) = o.header_offset {
                let mut class = u32_at(&out, i);
                if matches!(o.kind, Kind::GraphVerticalUnit(_, _))
                    && self.active_vertical_unit(o.page) == Some(o.id)
                {
                    let code = if self.grid_form[o.page] == GridForm::Rectangular {
                        55
                    } else {
                        54
                    };
                    class = (class & !0xffff) | code;
                }
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
                    Kind::GraphVerticalUnit(_, step)
                        if self.active_vertical_unit(o.page) == Some(o.id)
                            && matches!(out.get(i..i + 4), Some([55, 0, 0, 0] | [54, 0, 0, 0])) =>
                    {
                        let code: u16 = if self.grid_form[o.page] == GridForm::Rectangular {
                            55
                        } else {
                            54
                        };
                        out[i..i + 2].copy_from_slice(&code.to_le_bytes());
                        out[i + 4..i + 12].copy_from_slice(&step.to_le_bytes());
                    }
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
            if let (
                Kind::HideShowButton {
                    action,
                    label,
                    anchor,
                    ..
                },
                Some(i),
            ) = (&o.kind, o.header_offset)
            {
                let start_record = rs
                    .iter()
                    .position(|record| record.kind == 0x7d0 && record.data.start == i)
                    .ok_or("Missing saved action button header")?;
                let end_record = rs[start_record..]
                    .iter()
                    .position(|record| record.kind == 0x7d7)
                    .map(|index| start_record + index)
                    .ok_or("Missing saved action button terminator")?;
                let start = rs[start_record].start;
                let end = rs[end_record].data.end;
                let mut group = HideShowButtonGroup::parse(&out[start..end])?;
                group.set_action(*action);
                group.set_label(label)?;
                group.set_screen_anchor(*anchor);
                group.set_hidden(!o.visible);
                let encoded = group.encode();
                if encoded.len() != end - start {
                    return Err("Editing a saved button label cannot change its byte length".into());
                }
                out[start..end].copy_from_slice(&encoded);
            }
        }
        if !self.image_read_only
            && let Some(Kind::Image { transform, .. }) = self
                .objects
                .iter()
                .take(self.source_count)
                .find_map(|object| {
                    matches!(object.kind, Kind::Image { .. }).then_some(&object.kind)
                })
        {
            let mut picture = ImageFile::parse(&out)?;
            picture.set_transform(*transform)?;
            out = picture.encode().to_vec();
        }
        let mut page_index = 0;
        for record in rs.iter().filter(|r| r.kind == 0x3e8) {
            if self.objects.iter().any(|o| {
                o.page == page_index && !o.deleted && matches!(o.kind, Kind::CoordinateSystem(_, _))
            }) {
                out[record.data.start + 10] = u8::from(self.snap_points[page_index]);
            }
            page_index += 1;
        }
        for object in self
            .objects
            .iter()
            .take(self.source_count)
            .filter(|object| {
                matches!(object.kind, Kind::CoordinateSystem(_, _))
                    && !object.deleted
                    && !self.objects.iter().any(|later| {
                        later.id > object.id
                            && later.page == object.page
                            && !later.deleted
                            && matches!(later.kind, Kind::CoordinateSystem(_, _))
                    })
            })
        {
            let header = object
                .header_offset
                .ok_or("Missing coordinate-system header")?;
            let start = rs
                .iter()
                .position(|record| record.kind == 0x7d0 && record.data.start == header)
                .ok_or("Missing coordinate-system group")?;
            let flag = rs[start + 1..]
                .iter()
                .take_while(|record| record.kind != 0x7d7)
                .find(|record| record.kind == 0x0904 && record.data.len() == 8)
                .ok_or("Missing coordinate-system grid record")?;
            out[flag.data.start] = self.grid_form[object.page].code();
            out[flag.data.start + 2] = u8::from(self.grid_dotted[object.page]);
        }
        for object in self
            .objects
            .iter()
            .take(self.source_count)
            .filter(|object| {
                !object.deleted
                    && self.active_x_axis(object.page) == Some(object.id)
                    && matches!(object.kind, Kind::GraphAxis(_, _, 1 | 0x1_0001))
            })
        {
            let header = object.header_offset.ok_or("Missing X-axis header")?;
            let start = rs
                .iter()
                .position(|record| record.kind == 0x7d0 && record.data.start == header)
                .ok_or("Missing X-axis group")?;
            let format = rs[start + 1..]
                .iter()
                .take_while(|record| record.kind != 0x7d7)
                .find(|record| record.kind == 0x0905 && record.data.len() == 4)
                .ok_or("Missing X-axis format")?;
            out[format.data.start + 2] =
                u8::from(matches!(object.kind, Kind::GraphAxis(_, _, 0x1_0001)));
        }
        for object in self.objects.iter().take(self.source_count) {
            let Kind::PathLiteralPoint(_, value) = object.kind else {
                continue;
            };
            let header = object
                .header_offset
                .ok_or("Missing literal axis point header")?;
            let start = rs
                .iter()
                .position(|record| record.kind == 0x7d0 && record.data.start == header)
                .ok_or("Missing literal axis point group")?;
            let constraint = rs[start + 1..]
                .iter()
                .take_while(|record| record.kind != 0x7d7)
                .find(|record| {
                    record.kind == 0x07d3
                        && record.data.len() == 12
                        && u32_at(&out, record.data.start) == 123
                })
                .ok_or("Missing literal axis point value")?;
            out[constraint.data.start + 4..constraint.data.start + 12]
                .copy_from_slice(&value.to_le_bytes());
        }
        if self.objects.len() == self.source_count {
            return self.patch_visual_styles(
                self.patch_trace_flags(self.patch_text_and_labels(self.compact_deleted(out)?)?)?,
            );
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
            .find(|r| {
                matches!(r.kind, 0x2328 | 0x232d) && r.start >= last_group_end && r.start < trailer
            })
            .map(|r| r.start)
            .unwrap_or(trailer);
        let mut added = Vec::new();
        let mut picture_resource: Option<Vec<u8>> = None;
        let mut measurement_row = self
            .objects
            .iter()
            .take(self.source_count)
            .filter(|o| !o.deleted && o.kind.is_measurement())
            .count();
        for o in self.objects.iter().skip(self.source_count) {
            if let Kind::Image { png, transform } = &o.kind {
                let picture = ImageFile::independent_from_png(png, *transform)?;
                let mut group = picture.picture_group().to_vec();
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                picture_resource = Some(picture.resource_record().to_vec());
                continue;
            }
            if let Kind::PlainText { text, rect } = &o.kind {
                let mut group = text_object::encode_plain_text_group(text, *rect)?;
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::Parameter(group) = &o.kind {
                let mut bytes = group.as_bytes().to_vec();
                if !o.visible {
                    bytes[10] |= 1;
                }
                added.extend_from_slice(&bytes);
                continue;
            }
            if let Kind::Calculation(group) = &o.kind {
                let mut bytes = group.as_bytes().to_vec();
                if !o.visible {
                    bytes[10] |= 1;
                }
                added.extend_from_slice(&bytes);
                continue;
            }
            if let Kind::ValueTable(_, group) | Kind::ValueTablePair(_, _, group) = &o.kind {
                let mut bytes = group.as_bytes().to_vec();
                if !o.visible {
                    bytes[10] |= 1;
                }
                added.extend_from_slice(&bytes);
                continue;
            }
            if let Kind::HideShowButton {
                target,
                action,
                label,
                anchor,
            } = &o.kind
            {
                let mut group =
                    HideShowButtonGroup::from_original_template(*target as u32, label, *anchor)?;
                group.set_action(*action);
                group.set_hidden(!o.visible);
                added.extend_from_slice(&group.encode());
                continue;
            }
            if let Some(group) = self.graph_template_group(o)? {
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::FunctionDefinition(expression) = o.kind {
                let mut group = function_records::encode_definition_group(expression)?;
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::FunctionPlot(definition, frame, _, _, _) = o.kind {
                let Kind::FunctionDefinition(expression) = self.objects[definition - 1].kind else {
                    return Err("Invalid function definition".into());
                };
                let mut group = function_records::encode_plot_group(expression, definition, frame)?;
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::PlottedPoint(source, frame, mode, stored) = o.kind {
                let mut group =
                    plot_points_records::encode_group(mode, [source as u32, frame as u32], stored)?;
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::PathValuePoint(source, axis) = o.kind {
                let mut group = plot_axis_records::encode_group([source as u32, axis as u32])?;
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::PathLiteralPoint(axis, value) = o.kind {
                let mut group = plot_axis_records::encode_literal_group(axis as u32, value)?;
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::SlopeMeasurement(line, frame) = o.kind {
                let mut group = graph::slope_measurement_group(line, frame)?;
                Self::place_measurement(&mut group, 64, &mut measurement_row)?;
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::CoordinateMeasurement {
                kind,
                first,
                second,
                frame,
            } = o.kind
            {
                let mut refs = vec![u32::try_from(first).map_err(|_| "Too many objects")?];
                if let Some(second) = second {
                    refs.push(u32::try_from(second).map_err(|_| "Too many objects")?);
                }
                refs.push(u32::try_from(frame).map_err(|_| "Too many objects")?);
                let mut group = coordinate_measure_records::encode_group(kind, &refs)?;
                if kind == CoordinateMeasureKind::Coordinates {
                    let name = self
                        .coordinate_point_name(first)
                        .ok_or("Missing coordinate point")?;
                    coordinate_measure_records::set_coordinates_name(&mut group, &name)?;
                }
                let format_offset = if second.is_some() { 68 } else { 64 };
                Self::place_measurement(&mut group, format_offset, &mut measurement_row)?;
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::ValueEquation {
                kind,
                first,
                second,
            } = o.kind
            {
                let references = [
                    u32::try_from(first).map_err(|_| "Too many objects")?,
                    u32::try_from(second).map_err(|_| "Too many objects")?,
                ];
                let mut group = value_equation_records::encode_group(kind, references)?;
                if kind == ValueEquationKind::PointValue {
                    Self::place_measurement(&mut group, 64, &mut measurement_row)?;
                }
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::SegmentLength(segment) = o.kind {
                let mut group = *include_bytes!("../assets/length_measurement_group.bin");
                group[48..52].copy_from_slice(&(segment as u32).to_le_bytes());
                Self::place_measurement(&mut group, 60, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::PointDistance(a, b) = o.kind {
                let mut group = *include_bytes!("../assets/distance_measurement_group.bin");
                group[48..52].copy_from_slice(&(a as u32).to_le_bytes());
                group[52..56].copy_from_slice(&(b as u32).to_le_bytes());
                Self::place_measurement(&mut group, 64, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::PointRatio(a, b, c) = o.kind {
                let mut group = *include_bytes!("../assets/ratio_measurement_group.bin");
                group[48..52].copy_from_slice(&(a as u32).to_le_bytes());
                group[52..56].copy_from_slice(&(b as u32).to_le_bytes());
                group[56..60].copy_from_slice(&(c as u32).to_le_bytes());
                Self::place_measurement(&mut group, 68, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::PolygonPerimeter(polygon) = o.kind {
                let mut group = *include_bytes!("../assets/perimeter_measurement_group.bin");
                group[48..52].copy_from_slice(&(polygon as u32).to_le_bytes());
                Self::place_measurement(&mut group, 60, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::CircleCircumference(circle) = o.kind {
                let mut group = *include_bytes!("../assets/circumference_measurement_group.bin");
                group[48..52].copy_from_slice(&(circle as u32).to_le_bytes());
                Self::place_measurement(&mut group, 60, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::CircleRadius(circle) = o.kind {
                let mut group = *include_bytes!("../assets/radius_measurement_group.bin");
                group[48..52].copy_from_slice(&(circle as u32).to_le_bytes());
                Self::place_measurement(&mut group, 60, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::CircleArea(circle) = o.kind {
                let mut group = *include_bytes!("../assets/area_measurement_group.bin");
                group[48..52].copy_from_slice(&(circle as u32).to_le_bytes());
                Self::place_measurement(&mut group, 60, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::ArcAngle(arc) = o.kind {
                let mut group = *include_bytes!("../assets/arc_angle_measurement_group.bin");
                group[48..52].copy_from_slice(&(arc as u32).to_le_bytes());
                Self::place_measurement(&mut group, 60, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::ArcLength(arc) = o.kind {
                let mut group = *include_bytes!("../assets/arc_length_measurement_group.bin");
                group[48..52].copy_from_slice(&(arc as u32).to_le_bytes());
                Self::place_measurement(&mut group, 60, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::AngleMeasurement(a, b, c) = o.kind {
                let mut group = *include_bytes!("../assets/angle_measurement_group.bin");
                group[48..52].copy_from_slice(&(a as u32).to_le_bytes());
                group[52..56].copy_from_slice(&(b as u32).to_le_bytes());
                group[56..60].copy_from_slice(&(c as u32).to_le_bytes());
                Self::place_measurement(&mut group, 68, &mut measurement_row)?;
                if !o.visible {
                    group[10] = 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
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
                Kind::Midpoint(_, _) => (
                    1,
                    vec![
                        o.midpoint_host
                            .ok_or("Midpoint requires an existing segment")?,
                    ],
                ),
                Kind::Segment(a, b) => (2, vec![a, b]),
                Kind::Circle(a, b) => (3, vec![a, b]),
                Kind::CircleBySegment(a, b) => (4, vec![a, b]),
                Kind::Polygon(ref vertices) => (8, vertices.clone()),
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
            if let Some(label) = &o.label {
                added.extend_from_slice(&crate::text_records::encode_point_label_records(label));
            }
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
        let new_frame = self
            .objects
            .iter()
            .skip(self.source_count)
            .find(|object| matches!(object.kind, Kind::CoordinateSystem(_, _)))
            .map(|object| object.id);
        let added_len = added.len();
        out.splice(insert_at..insert_at, added);
        let last_function = self
            .objects
            .iter()
            .rev()
            .skip_while(|o| o.deleted)
            .find(|o| {
                matches!(
                    o.kind,
                    Kind::FunctionDefinition(_)
                        | Kind::FunctionPlot(_, _, _, _, _)
                        | Kind::PlottedPoint(_, _, _, _)
                        | Kind::CoordinateMeasurement { .. }
                )
            })
            .map(|o| o.id);
        let mut footer_added = 0;
        if let Some(id) = last_function {
            if let Some(marker) = rs
                .iter()
                .find(|r| r.kind == 0x2328 && r.start >= insert_at && r.start < trailer)
            {
                if marker.data.len() != 4 {
                    return Err("Invalid function marker".into());
                }
                let offset = marker.data.start + added_len;
                out[offset..offset + 4].copy_from_slice(&(id as u32).to_le_bytes());
            } else {
                let mut marker = Vec::new();
                put_record(&mut marker, 0x2328, &(id as u32).to_le_bytes());
                footer_added = marker.len();
                out.splice(insert_at + added_len..insert_at + added_len, marker);
            }
        }
        if let Some(frame) = new_frame {
            let frame = u32::try_from(frame).map_err(|_| "Too many objects")?;
            let mut page_link = Vec::new();
            put_record(&mut page_link, 0x232d, &frame.to_le_bytes());
            out.splice(
                trailer + added_len + footer_added..trailer + added_len + footer_added,
                page_link,
            );
        }
        records(&out)?;
        if let Some(resource) = picture_resource {
            let resource_at = records(&out)?
                .iter()
                .find(|record| record.kind == 0x03ef)
                .ok_or("Missing picture resource section")?
                .start;
            out.splice(resource_at..resource_at, resource);
        }
        self.patch_visual_styles(
            self.patch_trace_flags(self.patch_text_and_labels(self.compact_deleted(out)?)?)?,
        )
    }
    fn patch_trace_flags(&self, mut bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        let entries = records(&bytes)?;
        let mut objects = self.objects.iter().filter(|o| !o.deleted);
        for record in entries.iter().filter(|r| r.kind == 0x07d0) {
            let object = objects.next().ok_or("Object group count mismatch")?;
            if record.data.len() < 5 {
                return Err("Short object header".into());
            }
            let flag = &mut bytes[record.data.start + 4];
            *flag = (*flag & !1) | u8::from(object.tracing);
        }
        if objects.next().is_some() {
            return Err("Object group count mismatch".into());
        }
        Ok(bytes)
    }
    fn patch_visual_styles(&self, mut bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        let rs = records(&bytes)?;
        let survivors: Vec<_> = self
            .objects
            .iter()
            .filter(|object| !object.deleted)
            .collect();
        let mut ordinal = 0;
        for record in rs.iter().filter(|record| record.kind == 0x07d0) {
            let object = survivors
                .get(ordinal)
                .ok_or("Object group count mismatch")?;
            ordinal += 1;
            if let Some(style) = object.visual_style {
                style_records::set_visual_style(&mut bytes[record.data.clone()], style)
                    .map_err(|error| format!("Invalid style in object {}: {error}", object.id))?;
            }
        }
        if ordinal != survivors.len() {
            return Err("Object group count mismatch".into());
        }
        Ok(bytes)
    }
    fn patch_text_and_labels(&self, mut bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        let rs = records(&bytes)?;
        let mut edits: Vec<(usize, usize, Vec<u8>)> = Vec::new();
        let mut ordinal = 0usize;
        let mut current_group: Option<usize> = None;
        let survivors: Vec<&Object> = self.objects.iter().filter(|o| !o.deleted).collect();
        for (index, record) in rs.iter().enumerate() {
            if record.kind == 0x07d0 {
                ordinal += 1;
                current_group = Some(index);
            }
            if record.kind != 0x07d7 {
                continue;
            }
            let start_index = current_group
                .take()
                .ok_or("Object terminator without header")?;
            let object = survivors
                .get(ordinal - 1)
                .ok_or("Object group count mismatch")?;
            if object.id > self.source_count {
                continue;
            }
            let group = &rs[start_index..=index];
            if let Some(replacement) = match &object.kind {
                Kind::Parameter(value) => Some(value.as_bytes()),
                Kind::Calculation(value) => Some(value.as_bytes()),
                Kind::ValueTable(_, value) | Kind::ValueTablePair(_, _, value) => {
                    Some(value.as_bytes())
                }
                _ => None,
            } {
                if bytes[group[0].start..record.data.end] != *replacement {
                    let mut replacement = replacement.to_vec();
                    if !object.visible {
                        replacement[10] |= 1;
                    }
                    edits.push((group[0].start, record.data.end, replacement));
                }
                continue;
            }
            if let Kind::PlainText { text, rect } = &object.kind {
                let rich = group
                    .iter()
                    .find(|r| r.kind == 0x08fc)
                    .ok_or("Missing free-text payload")?;
                let bbox = group
                    .iter()
                    .find(|r| r.kind == 0x0898)
                    .ok_or("Missing free-text box")?;
                let (old_text, old_rect) = text_object::decode_plain_text(
                    &bytes[rich.data.clone()],
                    &bytes[bbox.data.clone()],
                )?;
                if old_text != *text || old_rect != *rect {
                    let mut replacement = text_object::encode_plain_text_group(text, *rect)?;
                    if !object.visible {
                        replacement[10] |= 1;
                    }
                    edits.push((group[0].start, record.data.end, replacement));
                }
            }
            if let Some(label) = &object.label {
                if let Some(old) = group.iter().find(|r| r.kind == 0x07d5) {
                    if bytes[old.data.clone()] != *label.payload() {
                        let mut replacement = Vec::new();
                        put_record(&mut replacement, 0x07d5, label.payload());
                        edits.push((old.start, old.data.end + old.data.len() % 2, replacement));
                    }
                } else {
                    let insertion = group[0].data.end + group[0].data.len() % 2;
                    edits.push((
                        insertion,
                        insertion,
                        crate::text_records::encode_point_label_records(label),
                    ));
                }
            }
        }
        if ordinal != survivors.len() {
            return Err("Object group count mismatch".into());
        }
        edits.sort_by(|a, b| b.0.cmp(&a.0));
        for (start, end, replacement) in edits {
            bytes.splice(start..end, replacement);
        }
        records(&bytes)?;
        Ok(bytes)
    }
    pub fn save_to(&mut self, path: &Path) -> Result<(), String> {
        let data = self.encode()?;
        let reparsed = Self::from_bytes(&data, Some(path.to_path_buf()))?;
        let had_deletion = self.objects.iter().any(|o| o.deleted);
        let mut ids = vec![0; self.objects.len() + 1];
        let mut survivors = Vec::new();
        for object in self.objects.iter().filter(|o| !o.deleted) {
            ids[object.id] = survivors.len() + 1;
            survivors.push(object);
        }
        if reparsed.snap_points != self.snap_points
            || reparsed.grid_dotted != self.grid_dotted
            || reparsed.grid_form != self.grid_form
            || reparsed.objects.len() != survivors.len()
            || reparsed
                .objects
                .iter()
                .zip(survivors)
                .any(|(saved, current)| {
                    saved.id != ids[current.id]
                        || saved.page != current.page
                        || saved.kind != current.kind.remap(&ids)
                        || saved.visible != current.visible
                        || saved.label != current.label
                        || saved.visual_style != current.visual_style
                        || saved.midpoint_host != current.midpoint_host.map(|host| ids[host])
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
        self.objects = reparsed.objects.clone();
        if had_deletion {
            self.selected.clear();
        }
        self.original = data;
        self.source_count = self.objects.len();
        self.path = Some(path.to_path_buf());
        self.safe_structure = reparsed.safe_structure;
        self.image_read_only = reparsed.image_read_only;
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
    fn source_point_style_undoes_and_survives_save() {
        let mut document = Document::from_bytes(
            include_bytes!("../tests/fixtures/style_blue_point.gsp"),
            None,
        )
        .unwrap();
        document.selected.insert(1);
        assert_eq!(
            document
                .set_selected_point_style(PointStyle::Large)
                .unwrap(),
            1
        );
        assert_eq!(
            document.objects[0].visual_style,
            Some(VisualStyle::Point {
                shape: PointStyle::Large,
                color: RgbColor {
                    red: 0,
                    green: 0,
                    blue: 255
                },
            })
        );
        document.undo();
        assert!(matches!(
            document.objects[0].visual_style,
            Some(VisualStyle::Point {
                shape: PointStyle::Medium,
                ..
            })
        ));
        document.redo();
        let check = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
        assert_eq!(
            check.objects[0].visual_style,
            document.objects[0].visual_style
        );
        assert_eq!(check.objects[0].kind, document.objects[0].kind);
        let path =
            std::env::temp_dir().join(format!("sketchpad-style-point-{}.gsp", std::process::id()));
        document.save_to(&path).unwrap();
        let reopened = Document::open(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            reopened.objects[0].visual_style,
            document.objects[0].visual_style
        );
        assert!(!reopened.partial_view);
    }

    #[test]
    fn new_styled_segment_and_copied_points_reopen() {
        let mut document = Document::new();
        let a = document.add(Kind::Free(Point { x: 100., y: 120. }));
        let b = document.add(Kind::Free(Point { x: 200., y: 120. }));
        let segment = document.add(Kind::Segment(a, b));
        document.selected.insert(segment);
        assert_eq!(
            document.set_selected_line_width(LineWidth::Thick).unwrap(),
            1
        );
        assert_eq!(
            document
                .set_selected_line_pattern(LinePattern::Dashed)
                .unwrap(),
            1
        );
        assert_eq!(
            document
                .set_selected_color(RgbColor {
                    red: 0,
                    green: 0,
                    blue: 255
                })
                .unwrap(),
            1
        );
        document.selected.clear();
        document.selected.insert(a);
        document
            .set_selected_point_style(PointStyle::Small)
            .unwrap();
        let copied = document.copy_selected().unwrap();
        let pasted = document.paste(&copied).unwrap();
        assert_eq!(
            document.objects[pasted[0] - 1].visual_style,
            document.objects[a - 1].visual_style
        );
        let path = std::env::temp_dir().join(format!(
            "sketchpad-style-segment-{}.gsp",
            std::process::id()
        ));
        document.save_to(&path).unwrap();
        let reopened = Document::open(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            reopened.objects[segment - 1].visual_style,
            Some(VisualStyle::Stroke {
                line: SegmentLineStyle {
                    width: LineWidth::Thick,
                    pattern: LinePattern::Dashed
                },
                color: RgbColor {
                    red: 0,
                    green: 0,
                    blue: 255
                },
            })
        );
        assert_eq!(
            reopened.objects[pasted[0] - 1].visual_style,
            document.objects[a - 1].visual_style
        );
    }
    #[test]
    fn imported_picture_moves_undoes_and_survives_save() {
        let png = include_bytes!("../tests/fixtures/image_tiny.png");
        let mut document = Document::new();
        document.add(Kind::Free(Point { x: 80., y: 90. }));
        let image = document
            .import_png(png, Point { x: 120., y: 130. })
            .unwrap();
        assert_eq!(image, 2);
        document
            .move_image(image, Point { x: 160., y: 155. })
            .unwrap();
        assert!(
            matches!(&document.objects[1].kind, Kind::Image { transform, .. } if transform[2] == 160.)
        );
        document.undo();
        assert!(
            matches!(&document.objects[1].kind, Kind::Image { transform, .. } if transform[2] == 120.)
        );
        document.redo();
        let path = std::env::temp_dir().join(format!("sketchpad-image-{}.gsp", std::process::id()));
        document.save_to(&path).unwrap();
        let reopened = Document::open(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects.len(), 2);
        assert_eq!(reopened.objects[1].kind, document.objects[1].kind);
    }

    #[test]
    fn picture_import_rejects_bad_png_and_read_only_sources() {
        let mut document = Document::new();
        assert!(
            document
                .import_png(b"broken", Point { x: 0., y: 0. })
                .is_err()
        );
        assert!(document.objects.is_empty());
        let png = include_bytes!("../tests/fixtures/image_tiny.png");
        document.import_png(png, Point { x: 0., y: 0. }).unwrap();
        assert!(document.import_png(png, Point { x: 20., y: 20. }).is_err());
        assert_eq!(document.objects.len(), 1);
        for source in [
            include_bytes!("../tests/fixtures/image_original_cropped.gsp").as_slice(),
            include_bytes!("../tests/fixtures/image_original_pasted.gsp").as_slice(),
        ] {
            let mut read_only = Document::from_bytes(source, None).unwrap();
            assert_eq!(read_only.encode().unwrap(), source);
            read_only.checkpoint();
            assert!(read_only.can_save().unwrap_err().contains("read-only"));
        }
    }

    #[test]
    fn original_drag_of_port_picture_reopens_and_remains_editable() {
        let source = include_bytes!("../tests/fixtures/image_port_imported_dragged_wine.gsp");
        let mut document = Document::from_bytes(source, None).unwrap();
        assert!(!document.partial_view);
        assert!(
            matches!(&document.objects[1].kind, Kind::Image { transform, .. }
            if transform[2] == 184. && transform[5] == 170.)
        );
        document.move_image(2, Point { x: 204., y: 180. }).unwrap();
        let written = document.encode().unwrap();
        let reopened = Document::from_bytes(&written, None).unwrap();
        assert!(
            matches!(&reopened.objects[1].kind, Kind::Image { transform, .. }
            if transform[2] == 204. && transform[5] == 180.)
        );
    }
    #[test]
    fn plain_text_edit_move_and_label_survive_save() {
        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/text_object_original_Hello.gsp"),
            None,
        )
        .unwrap();
        assert!(!doc.partial_view);
        assert_eq!(
            doc.encode().unwrap(),
            include_bytes!("../tests/fixtures/text_object_original_Hello.gsp")
        );
        doc.edit_text(1, "猫\nA<B>").unwrap();
        doc.move_text(1, Point { x: 180., y: 130. }).unwrap();
        let encoded = doc.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert_eq!(reopened.objects[0].kind, doc.objects[0].kind);
        assert!(!reopened.partial_view);

        let mut label_doc = Document::new();
        let point = label_doc.add(Kind::Free(Point { x: 100., y: 120. }));
        label_doc.set_point_label(point, "P", true).unwrap();
        let bytes = label_doc.encode().unwrap();
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(
            reopened.objects[0]
                .label
                .as_ref()
                .unwrap()
                .name_utf8()
                .unwrap(),
            "P"
        );
        assert!(!reopened.partial_view);
    }
    #[test]
    fn source_point_label_rename_and_hide_survive_save() {
        let original = include_bytes!("../tests/fixtures/text_point_label.gsp");
        let mut doc = Document::from_bytes(original, None).unwrap();
        assert_eq!(doc.encode().unwrap(), original);
        doc.set_point_label(1, "Cat", false).unwrap();
        let encoded = doc.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        let label = reopened.objects[0].label.as_ref().unwrap();
        assert_eq!(label.name_utf8().unwrap(), "Cat");
        assert!(!label.visible());
        assert!(!reopened.partial_view);
    }
    #[test]
    fn wine_dragged_text_preserves_opaque_padding_until_edited() {
        let original = include_bytes!("../tests/fixtures/text_object_generated_Hello_dragged.gsp");
        let mut doc = Document::from_bytes(original, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), original);
        doc.edit_text(1, "Hello again").unwrap();
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(
            matches!(&reopened.objects[0].kind, Kind::PlainText { text, .. } if text == "Hello again")
        );
    }
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
            "wine_moved_length",
            "reflection",
            "wine_moved_reflection",
            "wine_moved_reflected_circle",
            "scale",
            "wine_moved_dilation",
            "rotation",
            "wine_moved_rotation",
            "wine_deleted_source",
            "wine_cut_segment",
            "wine_pasted_segment",
            "wine_segment_length",
            "wine_point_distance",
            "wine_moved_distance",
            "wine_circle_circumference",
            "wine_moved_circumference",
            "wine_circle_radius",
            "wine_moved_radius",
            "wine_point_angle",
            "wine_moved_angle",
            "wine_moved_generated_angle",
            "wine_circle_area",
            "wine_circle_area_radius",
            "wine_moved_area",
            "wine_moved_generated_area",
            "wine_arc_angle",
            "wine_moved_arc_angle",
            "wine_arc_length",
            "wine_moved_arc_length",
            "wine_arc_measurements_on_circle",
            "wine_segment_slope",
            "wine_point_ratio",
            "wine_moved_ratio",
            "wine_opposite_ratio",
            "wine_offline_ratio",
            "wine_polygon_perimeter",
            "wine_moved_generated_polygon_perimeter",
            "action_hide_show",
            "action_hide_show_clicked",
            "action_hide_show_reclicked",
            "action_generated_english",
            "action_generated_english_clicked",
            "action_port_generated",
            "action_port_clicked",
            "action_port_wine_clicked",
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
    fn hide_show_button_survives_original_click_and_port_edit() {
        let mut doc = Document::open(Path::new("tests/fixtures/action_hide_show.gsp")).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects[3].kind.dependencies(), vec![3]);
        doc.activate_hide_show_button(4).unwrap();
        assert!(!doc.objects[2].visible);
        assert!(matches!(
            doc.objects[3].kind,
            Kind::HideShowButton {
                action: VisibilityAction::Show,
                ..
            }
        ));
        let encoded = doc.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert_eq!(reopened.objects[2].visible, doc.objects[2].visible);
        assert_eq!(reopened.objects[3].kind, doc.objects[3].kind);
        doc.undo();
        assert!(doc.objects[2].visible);
        doc.redo();
        assert!(!doc.objects[2].visible);
        let mut wine =
            Document::open(Path::new("tests/fixtures/action_port_wine_clicked.gsp")).unwrap();
        assert!(!wine.partial_view);
        assert!(!wine.objects[2].visible);
        assert!(matches!(
            wine.objects[3].kind,
            Kind::HideShowButton {
                action: VisibilityAction::Show,
                ..
            }
        ));
        wine.activate_hide_show_button(4).unwrap();
        let path =
            std::env::temp_dir().join(format!("sketchpad_action_{}.gsp", std::process::id()));
        wine.save_to(&path).unwrap();
        let port_saved = Document::open(&path).unwrap();
        assert!(port_saved.objects[2].visible);
        assert!(matches!(
            port_saved.objects[3].kind,
            Kind::HideShowButton {
                action: VisibilityAction::Hide,
                ..
            }
        ));
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn polygon_perimeter_matches_original_and_updates_after_drag() {
        let original =
            Document::open(Path::new("tests/fixtures/wine_polygon_perimeter.gsp")).unwrap();
        assert_eq!(original.objects[3].kind, Kind::Polygon(vec![1, 2, 3]));
        assert_eq!(original.objects[4].kind, Kind::PolygonPerimeter(4));
        assert!((original.perimeter_cm(5).unwrap() - 9.525).abs() < 0.001);
        let edited = Document::open(Path::new(
            "tests/fixtures/wine_moved_generated_polygon_perimeter.gsp",
        ))
        .unwrap();
        assert_eq!(edited.objects[4].kind, Kind::PolygonPerimeter(4));
        assert!((edited.perimeter_cm(5).unwrap() - 10.64).abs() < 0.01);

        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let b = doc.add(Kind::Free(Point { x: 270., y: 120. }));
        let c = doc.add(Kind::Free(Point { x: 270., y: 210. }));
        let polygon = doc.add(Kind::Polygon(vec![a, b, c]));
        let perimeter = doc.add(Kind::PolygonPerimeter(polygon));
        assert_eq!(doc.objects[polygon - 1].kind.dependencies(), vec![a, b, c]);
        assert_eq!(
            doc.objects[perimeter - 1].kind.dependencies(),
            vec![polygon]
        );
        let path =
            std::env::temp_dir().join(format!("sketchpad_polygon_{}.gsp", std::process::id()));
        doc.save_to(&path).unwrap();
        let mut reopened = Document::open(&path).unwrap();
        assert_eq!(
            reopened.objects[polygon - 1].kind,
            Kind::Polygon(vec![a, b, c])
        );
        assert!((reopened.perimeter_cm(perimeter).unwrap() - 9.525).abs() < 0.001);
        reopened.checkpoint();
        assert!(reopened.move_point(c, Point { x: 310., y: 210. }));
        let changed = reopened.perimeter_cm(perimeter).unwrap();
        assert!((changed - 10.64).abs() < 0.01);
        reopened.undo();
        assert!((reopened.perimeter_cm(perimeter).unwrap() - 9.525).abs() < 0.001);
        reopened.redo();
        assert!((reopened.perimeter_cm(perimeter).unwrap() - changed).abs() < 1e-9);
        reopened.save_to(&path).unwrap();
        let saved = Document::open(&path).unwrap();
        assert!((saved.perimeter_cm(perimeter).unwrap() - changed).abs() < 1e-9);
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn invalid_new_polygon_does_not_serialize() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = doc.add(Kind::Free(Point { x: 20., y: 0. }));
        doc.add(Kind::Polygon(vec![a, b, a]));
        assert!(doc.encode().is_err());
    }
    #[test]
    fn ratio_matches_signed_projection_after_drag_and_reopen() {
        for (path, expected) in [
            ("wine_point_ratio", 1.2),
            ("wine_moved_ratio", 1.4),
            ("wine_opposite_ratio", -1.0),
            ("wine_offline_ratio", 1.2),
        ] {
            let path = format!("tests/fixtures/{path}.gsp");
            let doc = Document::open(Path::new(&path)).unwrap();
            assert_eq!(doc.objects[3].kind, Kind::PointRatio(1, 2, 3));
            assert!((doc.ratio_value(4).unwrap() - expected).abs() < 1e-9);
        }
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 150., y: 100. }));
        let b = doc.add(Kind::Free(Point { x: 250., y: 100. }));
        let c = doc.add(Kind::Free(Point { x: 270., y: 100. }));
        let ratio = doc.add(Kind::PointRatio(a, b, c));
        assert!((doc.ratio_value(ratio).unwrap() - 1.2).abs() < 1e-9);
        let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects[3].kind, Kind::PointRatio(a, b, c));
        reopened.checkpoint();
        assert!(reopened.move_point(c, Point { x: 50., y: 150. }));
        assert!((reopened.ratio_value(ratio).unwrap() + 1.0).abs() < 1e-9);
        reopened.undo();
        assert!((reopened.ratio_value(ratio).unwrap() - 1.2).abs() < 1e-9);
        reopened.redo();
        assert!((reopened.ratio_value(ratio).unwrap() + 1.0).abs() < 1e-9);
        let saved = Document::from_bytes(&reopened.encode().unwrap(), None).unwrap();
        assert!((saved.ratio_value(ratio).unwrap() + 1.0).abs() < 1e-9);
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
    fn segment_length_matches_original_and_updates_after_drag() {
        let original = Document::open(Path::new("tests/fixtures/wine_segment_length.gsp")).unwrap();
        assert_eq!(original.objects[3].kind, Kind::SegmentLength(3));
        let (a, b, _) = original.line_points(3).unwrap();
        assert!(
            (original.segment_length_cm(4).unwrap() - a.distance(b) / 37.79527559055118).abs()
                < 1e-10
        );
        let moved = Document::open(Path::new("tests/fixtures/wine_moved_length.gsp")).unwrap();
        assert_eq!(moved.objects[3].kind, Kind::SegmentLength(3));
        assert!((moved.segment_length_cm(4).unwrap() - 5.291666666666667).abs() < 1e-10);

        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let b = doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let segment = doc.add(Kind::Segment(a, b));
        let length = doc.add(Kind::SegmentLength(segment));
        assert!((doc.segment_length_cm(length).unwrap() - 2.6458333333333335).abs() < 1e-10);
        let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects[3].kind, Kind::SegmentLength(3));
        reopened.checkpoint();
        assert!(reopened.move_point(b, Point { x: 350., y: 120. }));
        assert!((reopened.segment_length_cm(length).unwrap() - 5.291666666666667).abs() < 1e-10);
        assert_eq!(
            Document::from_bytes(&reopened.encode().unwrap(), None)
                .unwrap()
                .segment_length_cm(length),
            reopened.segment_length_cm(length)
        );
        reopened.selected.insert(segment);
        assert_eq!(reopened.clear_selected().unwrap(), vec![segment, length]);
        assert_eq!(
            Document::from_bytes(&reopened.encode().unwrap(), None)
                .unwrap()
                .objects
                .len(),
            2
        );
    }
    #[test]
    fn point_distance_matches_original_and_updates_after_edit() {
        let original = Document::open(Path::new("tests/fixtures/wine_point_distance.gsp")).unwrap();
        assert_eq!(original.objects[5].kind, Kind::PointDistance(1, 2));
        assert!((original.point_distance_cm(6).unwrap() - 2.6458333333333335).abs() < 1e-10);
        let moved = Document::open(Path::new("tests/fixtures/wine_moved_distance.gsp")).unwrap();
        assert_eq!(moved.objects[2].kind, Kind::PointDistance(1, 2));
        assert!((moved.point_distance_cm(3).unwrap() - 5.291666666666667).abs() < 1e-10);

        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let b = doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let distance = doc.add(Kind::PointDistance(a, b));
        let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects[2].kind, Kind::PointDistance(a, b));
        reopened.checkpoint();
        assert!(reopened.move_point(b, Point { x: 350., y: 120. }));
        assert!((reopened.point_distance_cm(distance).unwrap() - 5.291666666666667).abs() < 1e-10);
        let bytes = reopened.encode().unwrap();
        assert_eq!(
            Document::from_bytes(&bytes, None)
                .unwrap()
                .point_distance_cm(distance),
            reopened.point_distance_cm(distance)
        );
        reopened.undo();
        assert!((reopened.point_distance_cm(distance).unwrap() - 2.6458333333333335).abs() < 1e-10);
        reopened.selected.insert(a);
        assert_eq!(reopened.clear_selected().unwrap(), vec![a, distance]);
    }
    #[test]
    fn circumference_matches_original_and_updates_after_resize() {
        let original =
            Document::open(Path::new("tests/fixtures/wine_circle_circumference.gsp")).unwrap();
        assert_eq!(original.objects[3].kind, Kind::CircleCircumference(3));
        assert!((original.circle_circumference_cm(4).unwrap() - 16.62426112524599).abs() < 1e-10);
        let moved =
            Document::open(Path::new("tests/fixtures/wine_moved_circumference.gsp")).unwrap();
        assert_eq!(moved.objects[3].kind, Kind::CircleCircumference(3));
        assert!((moved.circle_circumference_cm(4).unwrap() - 24.936391687868986).abs() < 1e-10);

        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let b = doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let circle = doc.add(Kind::Circle(a, b));
        let circumference = doc.add(Kind::CircleCircumference(circle));
        let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects[3].kind, Kind::CircleCircumference(circle));
        reopened.checkpoint();
        assert!(reopened.move_point(b, Point { x: 350., y: 120. }));
        assert!(
            (reopened.circle_circumference_cm(circumference).unwrap() - 33.24852225049198).abs()
                < 1e-10
        );
        let bytes = reopened.encode().unwrap();
        assert_eq!(
            Document::from_bytes(&bytes, None)
                .unwrap()
                .circle_circumference_cm(circumference),
            reopened.circle_circumference_cm(circumference)
        );
        reopened.selected.insert(circle);
        assert_eq!(
            reopened.clear_selected().unwrap(),
            vec![circle, circumference]
        );
    }
    #[test]
    fn radius_matches_original_and_updates_after_resize() {
        let original = Document::open(Path::new("tests/fixtures/wine_circle_radius.gsp")).unwrap();
        assert_eq!(original.objects[3].kind, Kind::CircleRadius(3));
        assert!((original.circle_radius_cm(4).unwrap() - 2.6458333333333335).abs() < 1e-10);
        let moved = Document::open(Path::new("tests/fixtures/wine_moved_radius.gsp")).unwrap();
        assert_eq!(moved.objects[3].kind, Kind::CircleRadius(3));
        assert!((moved.circle_radius_cm(4).unwrap() - 3.96875).abs() < 1e-10);

        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let b = doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let circle = doc.add(Kind::Circle(a, b));
        let radius = doc.add(Kind::CircleRadius(circle));
        let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects[3].kind, Kind::CircleRadius(circle));
        reopened.checkpoint();
        assert!(reopened.move_point(b, Point { x: 350., y: 120. }));
        assert!((reopened.circle_radius_cm(radius).unwrap() - 5.291666666666667).abs() < 1e-10);
        assert_eq!(
            Document::from_bytes(&reopened.encode().unwrap(), None)
                .unwrap()
                .circle_radius_cm(radius),
            reopened.circle_radius_cm(radius)
        );
        reopened.selected.insert(circle);
        assert_eq!(reopened.clear_selected().unwrap(), vec![circle, radius]);
    }
    #[test]
    fn angle_matches_original_and_updates_after_drag() {
        let original = Document::open(Path::new("tests/fixtures/wine_point_angle.gsp")).unwrap();
        assert_eq!(original.objects[3].kind, Kind::AngleMeasurement(1, 2, 3));
        assert!((original.angle_degrees(4).unwrap() - 90.).abs() < 1e-10);
        let moved = Document::open(Path::new("tests/fixtures/wine_moved_angle.gsp")).unwrap();
        assert_eq!(moved.objects[3].kind, Kind::AngleMeasurement(1, 2, 3));
        assert!((moved.angle_degrees(4).unwrap() - 135.).abs() < 1e-10);
        let edited_generated =
            Document::open(Path::new("tests/fixtures/wine_moved_generated_angle.gsp")).unwrap();
        assert_eq!(
            edited_generated.objects[3].kind,
            Kind::AngleMeasurement(1, 2, 3)
        );
        assert!((edited_generated.angle_degrees(4).unwrap() - 135.).abs() < 1e-10);

        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let b = doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let c = doc.add(Kind::Free(Point { x: 250., y: 220. }));
        let angle = doc.add(Kind::AngleMeasurement(a, b, c));
        let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects[3].kind, Kind::AngleMeasurement(a, b, c));
        reopened.checkpoint();
        assert!(reopened.move_point(c, Point { x: 350., y: 220. }));
        assert!((reopened.angle_degrees(angle).unwrap() - 135.).abs() < 1e-10);
        assert_eq!(
            Document::from_bytes(&reopened.encode().unwrap(), None)
                .unwrap()
                .angle_degrees(angle),
            reopened.angle_degrees(angle)
        );
        reopened.undo();
        assert!((reopened.angle_degrees(angle).unwrap() - 90.).abs() < 1e-10);
        reopened.selected.insert(b);
        assert_eq!(reopened.clear_selected().unwrap(), vec![b, angle]);
    }
    #[test]
    fn angle_rejects_zero_rays_and_evaluates_large_coordinates() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let c = doc.add(Kind::Free(Point { x: 1., y: 0. }));
        assert_eq!(doc.angle_for_points(a, b, c), None);
        assert!(doc.move_point(a, Point { x: -1e150, y: 0. }));
        assert!(doc.move_point(c, Point { x: 0., y: 1e150 }));
        assert!((doc.angle_for_points(a, b, c).unwrap() - 90.).abs() < 1e-10);
    }
    #[test]
    fn circle_area_matches_original_and_updates_after_resize() {
        let original = Document::open(Path::new("tests/fixtures/wine_circle_area.gsp")).unwrap();
        assert_eq!(original.objects[3].kind, Kind::CircleArea(3));
        assert!((original.circle_area_cm2(4).unwrap() - 21.992512113606676).abs() < 1e-10);
        let moved = Document::open(Path::new("tests/fixtures/wine_moved_area.gsp")).unwrap();
        assert_eq!(moved.objects[3].kind, Kind::CircleArea(3));
        assert!((moved.circle_area_cm2(4).unwrap() - 49.483152255615025).abs() < 1e-10);
        let generated_moved =
            Document::open(Path::new("tests/fixtures/wine_moved_generated_area.gsp")).unwrap();
        assert_eq!(generated_moved.objects[3].kind, Kind::CircleArea(3));
        assert!((generated_moved.circle_area_cm2(4).unwrap() - 49.483152255615025).abs() < 1e-10);

        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let b = doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let circle = doc.add(Kind::Circle(a, b));
        let area = doc.add(Kind::CircleArea(circle));
        let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects[3].kind, Kind::CircleArea(circle));
        reopened.checkpoint();
        assert!(reopened.move_point(b, Point { x: 300., y: 120. }));
        assert!((reopened.circle_area_cm2(area).unwrap() - 49.483152255615025).abs() < 1e-10);
        assert_eq!(
            Document::from_bytes(&reopened.encode().unwrap(), None)
                .unwrap()
                .circle_area_cm2(area),
            reopened.circle_area_cm2(area)
        );
        reopened.selected.insert(circle);
        assert_eq!(reopened.clear_selected().unwrap(), vec![circle, area]);
    }
    #[test]
    fn multiple_measurements_use_original_label_spacing() {
        let original =
            Document::open(Path::new("tests/fixtures/wine_circle_area_radius.gsp")).unwrap();
        assert_eq!(original.objects[3].kind, Kind::CircleArea(3));
        assert_eq!(original.objects[4].kind, Kind::CircleRadius(3));

        let mut doc = Document::new();
        let center = doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let edge = doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let circle = doc.add(Kind::Circle(center, edge));
        doc.add(Kind::CircleArea(circle));
        doc.add(Kind::CircleRadius(circle));
        let bytes = doc.encode().unwrap();
        let ys: Vec<u16> = records(&bytes)
            .unwrap()
            .iter()
            .filter(|record| record.kind == 0x0903)
            .map(|record| {
                u16::from_le_bytes([bytes[record.data.start + 14], bytes[record.data.start + 15]])
            })
            .collect();
        assert_eq!(ys, [5, 33]);
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(reopened.objects[3].kind, Kind::CircleArea(circle));
        assert_eq!(reopened.objects[4].kind, Kind::CircleRadius(circle));
    }
    #[test]
    fn arc_angle_matches_original_and_updates_after_drag() {
        let original = Document::open(Path::new("tests/fixtures/wine_arc_angle.gsp")).unwrap();
        assert_eq!(original.objects[4].kind, Kind::ArcAngle(4));
        assert!((original.arc_angle_degrees(5).unwrap() - 159.22).abs() < 0.01);
        let moved = Document::open(Path::new("tests/fixtures/wine_moved_arc_angle.gsp")).unwrap();
        assert_eq!(moved.objects[4].kind, Kind::ArcAngle(4));
        assert!((moved.arc_angle_degrees(5).unwrap() - 225.24).abs() < 0.01);

        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 200., y: 180. }));
        let b = doc.add(Kind::Free(Point { x: 260., y: 130. }));
        let c = doc.add(Kind::Free(Point { x: 320., y: 180. }));
        let arc = doc.add(Kind::ThreePointArc(a, b, c));
        let measurement = doc.add(Kind::ArcAngle(arc));
        let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects[4].kind, Kind::ArcAngle(arc));
        reopened.checkpoint();
        assert!(reopened.move_point(b, Point { x: 260., y: 90. }));
        assert!((reopened.arc_angle_degrees(measurement).unwrap() - 225.24).abs() < 0.01);
        assert_eq!(
            Document::from_bytes(&reopened.encode().unwrap(), None)
                .unwrap()
                .arc_angle_degrees(measurement),
            reopened.arc_angle_degrees(measurement)
        );
        reopened.selected.insert(arc);
        assert_eq!(reopened.clear_selected().unwrap(), vec![arc, measurement]);
    }
    #[test]
    fn arc_length_matches_original_and_updates_after_drag() {
        let original = Document::open(Path::new("tests/fixtures/wine_arc_length.gsp")).unwrap();
        assert_eq!(original.objects[4].kind, Kind::ArcLength(4));
        assert!((original.arc_length_cm(5).unwrap() - 4.49).abs() < 0.01);
        let moved = Document::open(Path::new("tests/fixtures/wine_moved_arc_length.gsp")).unwrap();
        assert_eq!(moved.objects[4].kind, Kind::ArcLength(4));
        assert!((moved.arc_length_cm(5).unwrap() - 6.76).abs() < 0.01);

        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 200., y: 180. }));
        let b = doc.add(Kind::Free(Point { x: 260., y: 130. }));
        let c = doc.add(Kind::Free(Point { x: 320., y: 180. }));
        let arc = doc.add(Kind::ThreePointArc(a, b, c));
        let measurement = doc.add(Kind::ArcLength(arc));
        let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects[4].kind, Kind::ArcLength(arc));
        reopened.checkpoint();
        assert!(reopened.move_point(b, Point { x: 260., y: 90. }));
        assert!((reopened.arc_length_cm(measurement).unwrap() - 6.76).abs() < 0.01);
        assert_eq!(
            Document::from_bytes(&reopened.encode().unwrap(), None)
                .unwrap()
                .arc_length_cm(measurement),
            reopened.arc_length_cm(measurement)
        );
        reopened.selected.insert(arc);
        assert_eq!(reopened.clear_selected().unwrap(), vec![arc, measurement]);
    }
    #[test]
    fn arc_on_circle_measurements_reopen_with_separate_labels() {
        let doc = Document::open(Path::new(
            "tests/fixtures/wine_arc_measurements_on_circle.gsp",
        ))
        .unwrap();
        assert_eq!(doc.objects[5].kind, Kind::ArcOnCircle(3, 4, 5));
        assert_eq!(doc.objects[6].kind, Kind::ArcAngle(6));
        assert_eq!(doc.objects[7].kind, Kind::ArcLength(6));
        assert!((doc.arc_angle_degrees(7).unwrap() - 90.).abs() < 1e-10);
        assert!((doc.arc_length_cm(8).unwrap() - 2.91).abs() < 0.01);
    }
    #[test]
    fn copy_segment_includes_parents_but_not_children_like_original() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let b = doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let segment = doc.add(Kind::Segment(a, b));
        doc.add(Kind::Midpoint(a, b));
        doc.add(Kind::Free(Point { x: 330., y: 200. }));
        doc.selected.insert(segment);
        let clipboard = doc.copy_selected().unwrap();
        assert_eq!(clipboard.len(), 3);
        assert_eq!(doc.paste(&clipboard).unwrap(), vec![6, 7, 8]);
        assert_eq!(doc.objects[5].kind, Kind::Free(Point { x: 170., y: 140. }));
        assert_eq!(doc.objects[6].kind, Kind::Free(Point { x: 270., y: 140. }));
        assert_eq!(doc.objects[7].kind, Kind::Segment(6, 7));
        let original = Document::open(Path::new("tests/fixtures/wine_pasted_segment.gsp")).unwrap();
        assert_eq!(
            doc.objects.iter().map(|o| &o.kind).collect::<Vec<_>>(),
            original.objects.iter().map(|o| &o.kind).collect::<Vec<_>>()
        );
        doc.undo();
        assert_eq!(doc.objects.len(), 5);
        doc.redo();
        assert_eq!(doc.objects.len(), 8);
        let path =
            std::env::temp_dir().join(format!("sketchpad_rs_paste_{}.gsp", std::process::id()));
        doc.save_to(&path).unwrap();
        assert_eq!(Document::open(&path).unwrap().objects, doc.objects);
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn paste_midpoint_across_documents_preserves_dependencies() {
        let mut source = Document::new();
        let a = source.add(Kind::Free(Point { x: 1., y: 2. }));
        let b = source.add(Kind::Free(Point { x: 5., y: 8. }));
        source.add(Kind::Segment(a, b));
        let midpoint = source.add(Kind::Midpoint(a, b));
        source.selected.insert(midpoint);
        let clipboard = source.copy_selected().unwrap();
        assert_eq!(clipboard.len(), 4);
        let mut dest = Document::new();
        assert_eq!(dest.paste(&clipboard).unwrap(), vec![1, 2, 3, 4]);
        assert_eq!(dest.position(4), Some(Point { x: 23., y: 25. }));
        assert_eq!(
            Document::from_bytes(&dest.encode().unwrap(), None)
                .unwrap()
                .position(4),
            dest.position(4)
        );
    }
    #[test]
    fn clearing_segment_also_clears_its_midpoint() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 10., y: 20. }));
        let b = doc.add(Kind::Free(Point { x: 30., y: 20. }));
        let segment = doc.add(Kind::Segment(a, b));
        let midpoint = doc.add(Kind::Midpoint(a, b));
        doc.selected.insert(segment);
        assert_eq!(doc.clear_selected().unwrap(), vec![segment, midpoint]);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects.len(), 2);
        assert_eq!(reopened.position(1), Some(Point { x: 10., y: 20. }));

        let mut original_shape = Document::new();
        let first = original_shape.add(Kind::Free(Point { x: 150., y: 120. }));
        let second = original_shape.add(Kind::Free(Point { x: 250., y: 120. }));
        let line = original_shape.add(Kind::Segment(first, second));
        original_shape.add(Kind::Midpoint(first, second));
        original_shape.add(Kind::Free(Point { x: 330., y: 200. }));
        original_shape.selected.insert(line);
        original_shape.clear_selected().unwrap();
        let parsed = Document::from_bytes(&original_shape.encode().unwrap(), None).unwrap();
        let wine = Document::open(Path::new("tests/fixtures/wine_cut_segment.gsp")).unwrap();
        assert_eq!(
            parsed.objects.iter().map(|o| &o.kind).collect::<Vec<_>>(),
            wine.objects.iter().map(|o| &o.kind).collect::<Vec<_>>()
        );
    }
    #[test]
    fn midpoint_tracks_its_specific_segment_when_segments_share_endpoints() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 10., y: 20. }));
        let b = doc.add(Kind::Free(Point { x: 30., y: 20. }));
        let first = doc.add(Kind::Segment(a, b));
        let second = doc.add(Kind::Segment(a, b));
        let midpoint = doc.add(Kind::Midpoint(a, b));
        doc.objects[midpoint - 1].midpoint_host = Some(second);
        let mut loaded = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(loaded.objects[midpoint - 1].midpoint_host, Some(second));
        loaded.selected.insert(first);
        assert_eq!(loaded.clear_selected().unwrap(), vec![first]);
        let after = Document::from_bytes(&loaded.encode().unwrap(), None).unwrap();
        assert_eq!(after.objects[3].midpoint_host, Some(3));
        assert_eq!(after.position(4), Some(Point { x: 20., y: 20. }));
    }
    #[test]
    fn paste_refusal_leaves_unsupported_document_unchanged() {
        let mut source = Document::new();
        let point = source.add(Kind::Free(Point { x: 10., y: 20. }));
        source.selected.insert(point);
        let clipboard = source.copy_selected().unwrap();
        let mut dest = Document::open(Path::new("tests/fixtures/example_tools.gsp")).unwrap();
        let before = dest.encode().unwrap();
        assert!(dest.paste(&clipboard).is_err());
        assert_eq!(dest.encode().unwrap(), before);
        assert!(!dest.dirty);
    }
    #[test]
    fn clearing_source_removes_dependents_and_survives_save() {
        let mut doc = Document::new();
        let first = doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let second = doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let segment = doc.add(Kind::Segment(first, second));
        let midpoint = doc.add(Kind::Midpoint(first, second));
        let unrelated = doc.add(Kind::Free(Point { x: 330., y: 200. }));
        doc.selected.insert(first);
        assert_eq!(
            doc.clear_selected().unwrap(),
            vec![first, segment, midpoint]
        );
        assert_eq!(doc.position(first), None);
        assert_eq!(doc.position(second), Some(Point { x: 250., y: 120. }));
        doc.undo();
        assert_eq!(doc.position(midpoint), Some(Point { x: 200., y: 120. }));
        doc.redo();
        assert_eq!(doc.position(midpoint), None);
        let bytes = doc.encode().unwrap();
        let parsed = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(parsed.objects.len(), 2);
        assert_eq!(parsed.position(1), doc.position(second));
        assert_eq!(parsed.position(2), doc.position(unrelated));
        assert!(!parsed.partial_view);
        let wine = Document::open(Path::new("tests/fixtures/wine_deleted_source.gsp")).unwrap();
        assert_eq!(
            wine.objects.iter().map(|o| &o.kind).collect::<Vec<_>>(),
            parsed.objects.iter().map(|o| &o.kind).collect::<Vec<_>>()
        );
        let path =
            std::env::temp_dir().join(format!("sketchpad_rs_clear_{}.gsp", std::process::id()));
        doc.save_to(&path).unwrap();
        assert_eq!(doc.objects.len(), 2);
        assert_eq!(Document::open(&path).unwrap().objects, doc.objects);
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn clearing_independent_object_remaps_surviving_references() {
        let mut doc = Document::new();
        let removed = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let a = doc.add(Kind::Free(Point { x: 10., y: 10. }));
        let b = doc.add(Kind::Free(Point { x: 20., y: 10. }));
        let segment = doc.add(Kind::Segment(a, b));
        let middle = doc.add(Kind::Midpoint(a, b));
        doc.selected.insert(removed);
        assert_eq!(doc.clear_selected().unwrap(), vec![removed]);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects.len(), 4);
        assert_eq!(reopened.objects[2].kind, Kind::Segment(1, 2));
        assert_eq!(reopened.objects[3].kind, Kind::Midpoint(1, 2));
        assert_eq!(reopened.position(4), doc.position(middle));
        assert_eq!(reopened.line_points(3), doc.line_points(segment));
    }
    #[test]
    fn clear_then_construct_and_unsupported_document_guard() {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let b = doc.add(Kind::Free(Point { x: 10., y: 0. }));
        doc.add(Kind::Segment(a, b));
        doc.selected.insert(a);
        assert_eq!(doc.clear_selected().unwrap(), vec![1, 3]);
        let c = doc.add(Kind::Free(Point { x: 20., y: 0. }));
        doc.add(Kind::Segment(b, c));
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects.len(), 3);
        assert_eq!(reopened.objects[2].kind, Kind::Segment(1, 2));

        let mut source = Document::open(Path::new("tests/fixtures/segment.gsp")).unwrap();
        source.selected.insert(1);
        assert_eq!(source.clear_selected().unwrap(), vec![1, 3]);
        let reopened = Document::from_bytes(&source.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects.len(), 1);
        assert_eq!(reopened.objects[0].kind, source.objects[1].kind);

        let mut unsupported =
            Document::open(Path::new("tests/fixtures/example_tools.gsp")).unwrap();
        unsupported.selected.insert(1);
        assert!(unsupported.clear_selected().is_err());
        assert_eq!(
            unsupported.encode().unwrap(),
            fs::read("tests/fixtures/example_tools.gsp").unwrap()
        );
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
    #[test]
    fn graph_frame_and_slope_reopen_after_edit() {
        let original = Document::open(Path::new("tests/fixtures/wine_segment_slope.gsp")).unwrap();
        assert_eq!(
            original.objects[4].kind,
            Kind::GraphHorizontalUnit(4, 37.79527559055118)
        );
        assert_eq!(original.objects[8].kind, Kind::CoordinateSystem(6, 8));
        assert_eq!(original.objects[9].kind, Kind::SlopeMeasurement(3, 9));
        assert_eq!(original.slope_value(10), Some(0.0));

        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
        let b = doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let line = doc.add(Kind::Segment(a, b));
        let frame = doc
            .add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(frame, 9);
        let slope = doc.add(Kind::SlopeMeasurement(line, frame));
        assert_eq!(doc.slope_value(slope), Some(0.0));
        doc.set_grid_visible(false);
        assert!(!doc.grid_visible());
        doc.undo();
        assert!(doc.grid_visible());
        doc.redo();
        assert!(!doc.grid_visible());
        let bytes = doc.encode().unwrap();
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(reopened.objects.len(), 10);
        assert!(!reopened.partial_view);
        assert!(!reopened.grid_visible());
        assert_eq!(reopened.snap_points, vec![false]);
        assert_eq!(reopened.slope_value(slope), Some(0.0));
        let mut reopened = reopened;
        assert!(reopened.move_point(b, Point { x: 300., y: 170. }));
        assert!((reopened.slope_value(slope).unwrap() + 1. / 3.).abs() < 1e-12);
        let edited = Document::from_bytes(&reopened.encode().unwrap(), None).unwrap();
        assert!((edited.slope_value(slope).unwrap() + 1. / 3.).abs() < 1e-12);
    }

    #[test]
    fn blank_graph_frame_has_page_link_and_persistent_grid_state() {
        let mut doc = Document::new();
        let frame = doc
            .add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(frame, 6);
        doc.set_grid_visible(false);
        let bytes = doc.encode().unwrap();
        let page_link = records(&bytes)
            .unwrap()
            .into_iter()
            .find(|record| record.kind == 0x232d)
            .unwrap();
        assert_eq!(u32_at(&bytes, page_link.data.start), 6);
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.active_coordinate_system(), Some(frame));
        assert!(!reopened.grid_visible());
        assert_eq!(reopened.snap_points, vec![false]);
    }

    #[test]
    fn dotted_grid_matches_original_and_survives_undo_and_save() {
        let original = fs::read("tests/fixtures/graph_dotted_grid_original.gsp").unwrap();
        let parsed = Document::from_bytes(&original, None).unwrap();
        assert!(!parsed.partial_view);
        assert_eq!(parsed.grid_dotted, vec![true]);
        assert_eq!(parsed.encode().unwrap(), original);

        let mut doc = Document::new();
        doc.add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        doc.set_grid_dotted(true);
        assert_eq!(doc.grid_dotted, vec![true]);
        doc.undo();
        assert_eq!(doc.grid_dotted, vec![false]);
        doc.redo();
        assert_eq!(doc.grid_dotted, vec![true]);
        let encoded = doc.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.grid_dotted, vec![true]);
        assert_eq!(reopened.encode().unwrap(), encoded);

        let mut edited_original = parsed;
        edited_original.set_grid_dotted(false);
        let encoded = edited_original.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert_eq!(reopened.grid_dotted, vec![false]);
    }

    #[test]
    fn show_grid_and_snap_points_use_distinct_original_records() {
        let hidden = fs::read("tests/fixtures/graph_hidden_by_menu_original.gsp").unwrap();
        let hidden = Document::from_bytes(&hidden, None).unwrap();
        assert!(!hidden.partial_view);
        assert!(!hidden.grid_enabled());
        assert_eq!(hidden.snap_points, vec![false]);
        assert!(hidden.is_displayed(1));
        assert!((2..=7).all(|id| !hidden.is_displayed(id)));
        assert_eq!(
            hidden.encode().unwrap(),
            fs::read("tests/fixtures/graph_hidden_by_menu_original.gsp").unwrap()
        );

        let snapped = fs::read("tests/fixtures/graph_snap_points_original.gsp").unwrap();
        let mut snapped = Document::from_bytes(&snapped, None).unwrap();
        assert!(!snapped.partial_view);
        assert!(snapped.grid_enabled());
        assert!(!snapped.grid_visible());
        assert_eq!(snapped.snap_points, vec![true]);
        assert_eq!(
            snapped.position(1),
            Some(Point {
                x: 236.79527559055117,
                y: 162.79527559055117,
            })
        );
        snapped.checkpoint();
        assert!(snapped.move_point(1, Point { x: 270., y: 175. }));
        assert_eq!(
            snapped.position(1),
            Some(Point {
                x: 274.59055118110234,
                y: 162.79527559055117,
            })
        );
        let reopened = Document::from_bytes(&snapped.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.snap_points, vec![true]);
        assert_eq!(reopened.position(1), snapped.position(1));

        snapped.set_snap_points(false);
        assert_eq!(snapped.snap_points, vec![false]);
        snapped.undo();
        assert_eq!(snapped.snap_points, vec![true]);
        snapped.redo();
        assert_eq!(snapped.snap_points, vec![false]);
    }

    #[test]
    fn polar_and_rectangular_grid_forms_reopen_and_edit() {
        for (path, form) in [
            (
                "tests/fixtures/graph_polar_grid_original.gsp",
                GridForm::Polar,
            ),
            (
                "tests/fixtures/graph_rectangular_grid_original.gsp",
                GridForm::Rectangular,
            ),
            (
                "tests/fixtures/graph_rectangular_grid_dragged_original.gsp",
                GridForm::Rectangular,
            ),
        ] {
            let bytes = fs::read(path).unwrap();
            let doc = Document::from_bytes(&bytes, None).unwrap();
            assert!(!doc.partial_view, "{path}");
            assert_eq!(doc.grid_form, vec![form]);
            assert_eq!(doc.encode().unwrap(), bytes);
        }

        let mut polar = Document::new();
        polar
            .add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        polar.set_grid_form(GridForm::Polar).unwrap();
        assert_eq!(polar.grid_form, vec![GridForm::Polar]);
        polar.undo();
        assert_eq!(polar.grid_form, vec![GridForm::Square]);
        polar.redo();
        assert_eq!(polar.grid_form, vec![GridForm::Polar]);
        assert_eq!(
            polar.encode().unwrap(),
            fs::read("tests/fixtures/graph_polar_grid_generated.gsp").unwrap()
        );
        polar.set_grid_dotted(true);
        assert_eq!(
            polar.encode().unwrap(),
            fs::read("tests/fixtures/graph_polar_dotted_generated.gsp").unwrap()
        );

        let mut rectangular = Document::new();
        rectangular
            .add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        rectangular.set_grid_form(GridForm::Rectangular).unwrap();
        assert!(rectangular.is_displayed(4));
        assert_eq!(
            rectangular.encode().unwrap(),
            fs::read("tests/fixtures/graph_rectangular_grid_generated.gsp").unwrap()
        );
        assert!(rectangular.move_point(4, Point { x: 199., y: 60. }));
        assert_eq!(rectangular.position(4), Some(Point { x: 199., y: 60. }));
        let reopened = Document::from_bytes(&rectangular.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.grid_form, vec![GridForm::Rectangular]);
        assert_eq!(reopened.position(4), Some(Point { x: 199., y: 60. }));

        let mut original = Document::open(Path::new(
            "tests/fixtures/graph_rectangular_grid_original.gsp",
        ))
        .unwrap();
        assert!(original.move_point(4, Point { x: 199., y: 60. }));
        let reopened = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.position(4), Some(Point { x: 199., y: 60. }));

        let mut changed = Document::open(Path::new(
            "tests/fixtures/graph_rectangular_grid_original.gsp",
        ))
        .unwrap();
        changed.set_grid_form(GridForm::Polar).unwrap();
        let reopened = Document::from_bytes(&changed.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.grid_form, vec![GridForm::Polar]);
        assert_eq!(
            reopened.position(4),
            Some(Point {
                x: 199.,
                y: 125. - 37.79527559055118
            })
        );
    }

    #[test]
    fn polar_snap_matches_original_grid_intersection() {
        let original =
            Document::open(Path::new("tests/fixtures/graph_polar_snap_original.gsp")).unwrap();
        assert!(!original.partial_view);
        assert_eq!(original.grid_form, vec![GridForm::Polar]);
        assert_eq!(original.snap_points, vec![true]);
        let expected = original.position(1).unwrap();

        let mut doc = Document::new();
        let point = doc.add(Kind::Free(Point { x: 210., y: 140. }));
        doc.add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        doc.set_grid_form(GridForm::Polar).unwrap();
        doc.set_snap_points(true);
        assert!(doc.move_point(point, Point { x: 250., y: 170. }));
        let actual = doc.position(point).unwrap();
        assert!(actual.distance(expected) < 1e-9);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(reopened.position(point).unwrap().distance(expected) < 1e-9);
    }

    #[test]
    fn trigonometric_axis_matches_original_and_roundtrips() {
        let bytes = fs::read("tests/fixtures/graph_trigonometric_axis_original.gsp").unwrap();
        let mut original = Document::from_bytes(&bytes, None).unwrap();
        assert!(!original.partial_view);
        assert!(original.axis_trigonometric());
        assert_eq!(original.objects[2].kind, Kind::GraphAxis(1, 2, 0x1_0001));
        assert_eq!(original.encode().unwrap(), bytes);
        original.set_axis_trigonometric(false).unwrap();
        assert!(!original.axis_trigonometric());
        original.undo();
        assert!(original.axis_trigonometric());
        original.redo();
        assert!(!original.axis_trigonometric());
        let reopened = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects[2].kind, Kind::GraphAxis(1, 2, 1));

        let mut generated = Document::new();
        generated
            .add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        generated.set_axis_trigonometric(true).unwrap();
        assert_eq!(
            generated.encode().unwrap(),
            fs::read("tests/fixtures/graph_trigonometric_axis_generated.gsp").unwrap()
        );
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(reopened.axis_trigonometric());
    }

    #[test]
    fn original_function_files_decode_as_editable_documents() {
        for (source, expression) in [
            (
                include_bytes!("../tests/fixtures/function_plot_identity.gsp").as_slice(),
                SimpleFunction::Identity,
            ),
            (
                include_bytes!("../tests/fixtures/function_plot_square.gsp").as_slice(),
                SimpleFunction::Square,
            ),
        ] {
            let mut doc = Document::from_bytes(source, None).unwrap();
            assert!(!doc.partial_view);
            assert_eq!(doc.objects[6].kind, Kind::FunctionDefinition(expression));
            assert!(matches!(
                doc.objects[7].kind,
                Kind::FunctionPlot(7, 6, _, _, 750)
            ));
            assert_eq!(doc.encode().unwrap(), source);
            let before = doc.function_plot_points(8, 3).unwrap();
            assert!(doc.move_point(1, Point { x: 228., y: 140. }));
            let after = doc.function_plot_points(8, 3).unwrap();
            assert!((after[1].x - before[1].x - 29.).abs() < 1e-9);
            assert!((after[1].y - before[1].y - 15.).abs() < 1e-9);
            let saved = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
            assert!(!saved.partial_view);
            assert_eq!(saved.objects[7].kind, doc.objects[7].kind);
        }
    }

    #[test]
    fn new_function_plot_reopens_and_tracks_drag_with_undo() {
        let mut doc = Document::new();
        let plot = doc
            .plot_new_function(SimpleFunction::Square, Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(plot, 8);
        let frame = doc.graph_frame(6).unwrap();
        let vertex = doc.function_plot_points(plot, 3).unwrap()[1];
        assert_eq!(
            vertex,
            Point {
                x: frame.origin.x,
                y: frame.origin.y
            }
        );
        let saved = doc.encode().unwrap();
        let mut reopened = Document::from_bytes(&saved, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[7].kind, doc.objects[7].kind);
        reopened.checkpoint();
        assert!(reopened.move_point(1, Point { x: 228., y: 140. }));
        assert_eq!(
            reopened.function_plot_points(plot, 3).unwrap()[1],
            Point { x: 228., y: 140. }
        );
        reopened.undo();
        assert_eq!(reopened.function_plot_points(plot, 3).unwrap()[1], vertex);
        reopened.redo();
        assert_eq!(
            reopened.function_plot_points(plot, 3).unwrap()[1],
            Point { x: 228., y: 140. }
        );
        let edited = Document::from_bytes(&reopened.encode().unwrap(), None).unwrap();
        assert_eq!(
            edited.function_plot_points(plot, 3).unwrap()[1],
            Point { x: 228., y: 140. }
        );
    }
    #[test]
    fn plotted_points_open_move_and_reopen() {
        for (bytes, mode) in [
            (
                include_bytes!("../tests/fixtures/plot_points_table_single_original.gsp")
                    .as_slice(),
                PlotPointsMode::Cartesian,
            ),
            (
                include_bytes!("../tests/fixtures/plot_points_table_polar_original.gsp").as_slice(),
                PlotPointsMode::Polar,
            ),
        ] {
            let mut doc = Document::from_bytes(bytes, None).unwrap();
            assert!(!doc.partial_view);
            assert!(matches!(doc.objects[8].kind, Kind::PlottedPoint(1, 8, m, _) if m == mode));
            assert_eq!(
                doc.objects[8].visual_style,
                Some(VisualStyle::Point {
                    shape: PointStyle::Medium,
                    color: RgbColor {
                        red: 255,
                        green: 0,
                        blue: 0
                    },
                })
            );
            let before = doc.position(9).unwrap();
            doc.checkpoint();
            assert!(doc.move_point(3, Point { x: 228., y: 140. }));
            let after = doc.position(9).unwrap();
            assert!((after.x - before.x - 29.).abs() < 1e-9);
            assert!((after.y - before.y - 15.).abs() < 1e-9);
            doc.undo();
            assert_eq!(doc.position(9), Some(before));
            doc.redo();
            let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
            assert!(!reopened.partial_view);
            assert_eq!(reopened.position(9), Some(after));
        }
    }

    #[test]
    fn new_plotted_points_save_and_reopen() {
        fn last_object_group(bytes: &[u8]) -> &[u8] {
            let entries = records(bytes).unwrap();
            let start = entries.iter().rfind(|r| r.kind == 0x07d0).unwrap().start;
            let end = entries.iter().rfind(|r| r.kind == 0x07d7).unwrap().data.end;
            &bytes[start..end]
        }
        for (mode, rows) in [
            (PlotPointsMode::Cartesian, 1),
            (PlotPointsMode::Polar, 1),
            (PlotPointsMode::Cartesian, 2),
        ] {
            let mut doc = Document::new();
            assert_eq!(doc.add_default_parameter((10, 5)).unwrap(), 1);
            assert_eq!(doc.tabulate_default_parameter(1, (10, 38)).unwrap(), 2);
            if rows == 2 {
                doc.change_table_rows(2, true).unwrap();
            }
            assert_eq!(
                doc.add_default_coordinate_system(Point { x: 199., y: 125. })
                    .unwrap(),
                8
            );
            assert_eq!(doc.plot_default_parameter(mode).unwrap(), 9);
            assert_eq!(
                doc.objects[8].visual_style,
                Some(VisualStyle::Point {
                    shape: PointStyle::Medium,
                    color: RgbColor {
                        red: 255,
                        green: 0,
                        blue: 0
                    },
                })
            );
            let expected = doc.position(9).unwrap();
            let saved = doc.encode().unwrap();
            let original = match mode {
                PlotPointsMode::Cartesian => {
                    include_bytes!("../tests/fixtures/plot_points_table_single_original.gsp")
                        .as_slice()
                }
                PlotPointsMode::Polar => {
                    include_bytes!("../tests/fixtures/plot_points_table_polar_original.gsp")
                        .as_slice()
                }
            };
            assert_eq!(last_object_group(&saved), last_object_group(original));
            let reopened = Document::from_bytes(&saved, None).unwrap();
            assert!(!reopened.partial_view);
            assert_eq!(reopened.objects.len(), 9);
            assert_eq!(reopened.position(9), Some(expected));
        }
    }

    #[test]
    fn original_two_row_plotted_point_is_editable() {
        let bytes = include_bytes!("../tests/fixtures/plot_points_table_two_rows_original.gsp");
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert!(matches!(doc.objects[1].kind, Kind::ValueTable(1, ref table) if table.rows() == 2));
        assert_eq!(doc.encode().unwrap(), bytes);
        let before = doc.position(9).unwrap();
        doc.checkpoint();
        assert!(doc.move_point(3, Point { x: 228., y: 140. }));
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        let after = reopened.position(9).unwrap();
        assert!((after.x - before.x - 29.).abs() < 1e-9);
        assert!((after.y - before.y - 15.).abs() < 1e-9);
    }

    #[test]
    fn wine_saved_integrated_plots_preserve_all_bytes() {
        for (bytes, mode) in [
            (
                include_bytes!(
                    "../tests/fixtures/plot_points_integrated_cartesian_saved_by_wine.gsp"
                )
                .as_slice(),
                PlotPointsMode::Cartesian,
            ),
            (
                include_bytes!("../tests/fixtures/plot_points_integrated_polar_saved_by_wine.gsp")
                    .as_slice(),
                PlotPointsMode::Polar,
            ),
        ] {
            let doc = Document::from_bytes(bytes, None).unwrap();
            assert!(!doc.partial_view);
            assert!(matches!(doc.objects[8].kind, Kind::PlottedPoint(1, 8, m, _) if m == mode));
            assert!(doc.position(9).is_some());
            assert_eq!(doc.encode().unwrap(), bytes);
        }
    }

    #[test]
    fn plot_value_on_axis_tracks_its_axis_and_cross_opens() {
        let original_bytes = include_bytes!("../tests/fixtures/graph_plot_value_axis_original.gsp");
        let mut original = Document::from_bytes(original_bytes, None).unwrap();
        assert!(!original.partial_view);
        assert_eq!(original.objects[8].kind, Kind::PathValuePoint(1, 5));
        assert_eq!(
            original.objects[8].visual_style,
            Some(VisualStyle::Point {
                shape: PointStyle::Medium,
                color: RgbColor {
                    red: 255,
                    green: 0,
                    blue: 0,
                },
            })
        );
        assert_eq!(original.encode().unwrap(), original_bytes);
        let initial = original.position(9).unwrap();
        assert!((initial.x - 236.79527559055118).abs() < 1e-9);
        assert!((initial.y - 125.).abs() < 1e-9);
        original.checkpoint();
        assert!(original.move_point(3, Point { x: 228., y: 140. }));
        let shifted = original.position(9).unwrap();
        assert!((shifted.x - initial.x - 29.).abs() < 1e-9);
        assert!((shifted.y - initial.y - 15.).abs() < 1e-9);
        original.undo();
        assert_eq!(original.position(9), Some(initial));
        original.redo();
        let edited = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
        assert!(!edited.partial_view);
        assert_eq!(edited.position(9), Some(shifted));

        let mut generated = Document::new();
        generated.add_default_parameter((10, 5)).unwrap();
        generated.tabulate_default_parameter(1, (10, 38)).unwrap();
        generated
            .add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(generated.plot_parameter_on_path(1, None).unwrap(), 9);
        assert_eq!(
            generated.encode().unwrap(),
            include_bytes!("../tests/fixtures/graph_plot_value_axis_generated.gsp")
        );
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[8].kind, Kind::PathValuePoint(1, 5));

        let mut no_table = Document::new();
        no_table.add_default_parameter((10, 5)).unwrap();
        no_table
            .add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(no_table.plot_parameter_on_path(1, None).unwrap(), 8);
        assert_eq!(
            no_table.encode().unwrap(),
            include_bytes!("../tests/fixtures/graph_plot_value_axis_no_table_generated.gsp")
        );
        no_table.undo();
        assert_eq!(no_table.objects.len(), 7);
        assert_eq!(no_table.plot_parameter_on_path(1, Some(6)).unwrap(), 8);
        assert_eq!(no_table.position(8).unwrap().x, 199.);
        assert!((no_table.position(8).unwrap().y - 87.20472440944882).abs() < 1e-9);
        assert_eq!(
            no_table.encode().unwrap(),
            include_bytes!("../tests/fixtures/graph_plot_value_y_generated.gsp")
        );
    }

    #[test]
    fn literal_axis_value_roundtrips_and_follows_axis_movement() {
        let original_bytes =
            include_bytes!("../tests/fixtures/graph_plot_value_literal_original.gsp");
        let mut original = Document::from_bytes(original_bytes, None).unwrap();
        assert!(!original.partial_view);
        assert_eq!(original.objects[6].kind, Kind::PathLiteralPoint(3, 2.5));
        assert_eq!(original.encode().unwrap(), original_bytes);
        assert_eq!(
            original.objects[6].visual_style,
            Some(VisualStyle::Point {
                shape: PointStyle::Medium,
                color: RgbColor {
                    red: 255,
                    green: 0,
                    blue: 0,
                },
            })
        );
        let before = original.position(7).unwrap();
        assert!((before.x - (199. + 2.5 * 37.79527559055118)).abs() < 1e-9);
        assert_eq!(before.y, 125.);
        original.checkpoint();
        assert!(original.move_point(1, Point { x: 228., y: 140. }));
        let after = original.position(7).unwrap();
        assert!((after.x - before.x - 29.).abs() < 1e-9);
        assert!((after.y - before.y - 15.).abs() < 1e-9);
        original.undo();
        assert_eq!(original.position(7), Some(before));
        original.redo();
        let edited = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
        assert_eq!(edited.position(7), Some(after));
        original.checkpoint();
        original.objects[6].kind = Kind::PathLiteralPoint(3, 3.);
        let changed_value = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
        assert_eq!(changed_value.objects[6].kind, Kind::PathLiteralPoint(3, 3.));

        let mut x = Document::new();
        x.add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(x.plot_literal_on_path(2.5, None).unwrap(), 7);
        assert_eq!(
            x.encode().unwrap(),
            include_bytes!("../tests/fixtures/graph_plot_value_literal_generated.gsp")
        );
        let mut y = Document::new();
        y.add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(y.plot_literal_on_path(-1.25, Some(5)).unwrap(), 7);
        assert_eq!(y.position(7).unwrap().x, 199.);
        assert!((y.position(7).unwrap().y - 172.24409448818898).abs() < 1e-9);
        assert_eq!(
            y.encode().unwrap(),
            include_bytes!("../tests/fixtures/graph_plot_value_literal_y_generated.gsp")
        );
        assert!(y.plot_literal_on_path(f64::NAN, None).is_err());
    }

    #[test]
    fn plotted_values_on_segments_track_their_endpoints() {
        let literal_bytes =
            include_bytes!("../tests/fixtures/graph_plot_segment_literal_original.gsp");
        let mut literal = Document::from_bytes(literal_bytes, None).unwrap();
        assert!(!literal.partial_view);
        assert_eq!(literal.objects[9].kind, Kind::PathLiteralPoint(3, 0.4));
        assert_eq!(literal.position(10), Some(Point { x: 190., y: 140. }));
        assert_eq!(literal.encode().unwrap(), literal_bytes);
        literal.checkpoint();
        assert!(literal.move_point(2, Point { x: 300., y: 200. }));
        assert_eq!(literal.position(10), Some(Point { x: 210., y: 152. }));
        literal.undo();
        assert_eq!(literal.position(10), Some(Point { x: 190., y: 140. }));
        literal.redo();
        let reopened = Document::from_bytes(&literal.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(10), Some(Point { x: 210., y: 152. }));

        let parameter_bytes =
            include_bytes!("../tests/fixtures/graph_plot_segment_parameter_original.gsp");
        let mut parameter = Document::from_bytes(parameter_bytes, None).unwrap();
        assert!(!parameter.partial_view);
        assert_eq!(parameter.objects[4].kind, Kind::PathValuePoint(1, 4));
        assert_eq!(parameter.position(5), Some(Point { x: 250., y: 170. }));
        assert_eq!(parameter.encode().unwrap(), parameter_bytes);
        parameter.checkpoint();
        assert!(parameter.move_point(3, Point { x: 300., y: 200. }));
        let reopened = Document::from_bytes(&parameter.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(5), Some(Point { x: 300., y: 200. }));

        let mut generated_literal = Document::new();
        generated_literal.add(Kind::Free(Point { x: 150., y: 120. }));
        generated_literal.add(Kind::Free(Point { x: 250., y: 170. }));
        generated_literal.add(Kind::Segment(1, 2));
        assert_eq!(
            generated_literal
                .plot_literal_on_path(0.4, Some(3))
                .unwrap(),
            4
        );
        assert!(
            generated_literal
                .plot_literal_on_path(1.2, Some(3))
                .is_err()
        );
        assert_eq!(
            generated_literal.encode().unwrap(),
            include_bytes!("../tests/fixtures/graph_plot_segment_literal_generated.gsp")
        );
        let mut generated_parameter = Document::new();
        generated_parameter.add_default_parameter((10, 5)).unwrap();
        generated_parameter.add(Kind::Free(Point { x: 150., y: 120. }));
        generated_parameter.add(Kind::Free(Point { x: 250., y: 170. }));
        generated_parameter.add(Kind::Segment(2, 3));
        assert_eq!(
            generated_parameter
                .plot_parameter_on_path(1, Some(4))
                .unwrap(),
            5
        );
        assert_eq!(
            generated_parameter.encode().unwrap(),
            include_bytes!("../tests/fixtures/graph_plot_segment_parameter_generated.gsp")
        );
    }

    #[test]
    fn plotted_values_on_circles_lines_and_rays_roundtrip_and_follow_paths() {
        for (bytes, value, expected) in [
            (
                include_bytes!("../tests/fixtures/graph_plot_circle_literal_original.gsp")
                    .as_slice(),
                0.25,
                Point {
                    x: 323.,
                    y: 217. - (27_f64.powi(2) + 115_f64.powi(2)).sqrt(),
                },
            ),
            (
                include_bytes!("../tests/fixtures/graph_plot_line_literal_original.gsp").as_slice(),
                -0.5,
                Point { x: 100., y: 95. },
            ),
            (
                include_bytes!("../tests/fixtures/graph_plot_ray_literal_original.gsp").as_slice(),
                2.,
                Point { x: 350., y: 220. },
            ),
        ] {
            let mut document = Document::from_bytes(bytes, None).unwrap();
            assert!(!document.partial_view);
            assert_eq!(document.objects[3].kind, Kind::PathLiteralPoint(3, value));
            assert_eq!(document.encode().unwrap(), bytes);
            let position = document.position(4).unwrap();
            assert!((position.x - expected.x).abs() < 1e-9);
            assert!((position.y - expected.y).abs() < 1e-9);
            let start = document.position(1).unwrap();
            document.checkpoint();
            assert!(document.move_point(
                1,
                Point {
                    x: start.x + 10.,
                    y: start.y + 20.
                }
            ));
            let moved = document.position(4).unwrap();
            assert!(moved.x.is_finite() && moved.y.is_finite());
            let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
            assert!(!reopened.partial_view);
            assert_eq!(reopened.position(4), Some(moved));
            document.undo();
            assert_eq!(document.position(4), Some(position));
        }

        let mut circle = Document::new();
        circle.add(Kind::Free(Point { x: 199., y: 125. }));
        circle.add(Kind::Free(Point { x: 236., y: 125. }));
        circle.add(Kind::Circle(1, 2));
        assert_eq!(circle.plot_literal_on_path(0.25, Some(3)).unwrap(), 4);
        assert_eq!(circle.position(4), Some(Point { x: 199., y: 88. }));
        assert_eq!(
            circle.encode().unwrap(),
            include_bytes!("../tests/fixtures/graph_plot_circle_literal_generated.gsp")
        );

        for (kind, value, expected, file) in [
            (
                Kind::Line(1, 2),
                -0.5,
                Point { x: 100., y: 95. },
                include_bytes!("../tests/fixtures/graph_plot_line_literal_generated.gsp")
                    .as_slice(),
            ),
            (
                Kind::Ray(1, 2),
                2.,
                Point { x: 350., y: 220. },
                include_bytes!("../tests/fixtures/graph_plot_ray_literal_generated.gsp").as_slice(),
            ),
        ] {
            let mut document = Document::new();
            document.add(Kind::Free(Point { x: 150., y: 120. }));
            document.add(Kind::Free(Point { x: 250., y: 170. }));
            document.add(kind);
            document.plot_literal_on_path(value, Some(3)).unwrap();
            assert_eq!(document.position(4), Some(expected));
            assert_eq!(document.encode().unwrap(), file);
        }
        let mut ray = Document::new();
        ray.add(Kind::Free(Point { x: 150., y: 120. }));
        ray.add(Kind::Free(Point { x: 250., y: 170. }));
        ray.add(Kind::Ray(1, 2));
        assert!(ray.plot_literal_on_path(-0.5, Some(3)).is_err());

        let mut live_circle = Document::new();
        live_circle.add_default_parameter((10, 5)).unwrap();
        live_circle.add(Kind::Free(Point { x: 199., y: 125. }));
        live_circle.add(Kind::Free(Point { x: 236., y: 125. }));
        live_circle.add(Kind::Circle(2, 3));
        assert_eq!(live_circle.plot_parameter_on_path(1, Some(4)).unwrap(), 5);
        assert!((live_circle.position(5).unwrap().x - 236.).abs() < 1e-9);
        assert_eq!(
            live_circle.encode().unwrap(),
            include_bytes!("../tests/fixtures/graph_plot_circle_parameter_generated.gsp")
        );
        live_circle.checkpoint();
        assert!(live_circle.move_point(3, Point { x: 249., y: 125. }));
        assert!((live_circle.position(5).unwrap().x - 249.).abs() < 1e-9);
    }

    #[test]
    fn two_parameter_table_data_roundtrips_and_tracks_frame() {
        for bytes in [
            include_bytes!("../tests/fixtures/table_two_parameters_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/table_two_columns_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/plot_table_data_base_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/plot_table_data_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/table_two_columns_two_rows_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/plot_table_data_two_rows_base_original.gsp")
                .as_slice(),
            include_bytes!("../tests/fixtures/plot_table_data_two_rows_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/table_t2_value2_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/table_two_columns_distinct_rows_original.gsp")
                .as_slice(),
            include_bytes!("../tests/fixtures/plot_table_distinct_base_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/plot_table_distinct_original.gsp").as_slice(),
        ] {
            let document = Document::from_bytes(bytes, None).unwrap();
            assert!(!document.partial_view);
            assert_eq!(document.encode().unwrap(), bytes);
            assert!(
                matches!(&document.objects[0].kind, Kind::Parameter(group) if group.name() == "t[1]")
            );
            assert!(
                matches!(&document.objects[1].kind, Kind::Parameter(group) if group.name() == "t[2]")
            );
        }
        let mut original = Document::from_bytes(
            include_bytes!("../tests/fixtures/plot_table_data_original.gsp"),
            None,
        )
        .unwrap();
        assert!(
            matches!(&original.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.rows() == 1)
        );
        assert_eq!(
            original.objects[9].kind,
            Kind::PlottedPoint(1, 9, PlotPointsMode::Cartesian, [1., 1.])
        );
        let before = original.position(10).unwrap();
        assert!((before.x - 236.79527559055118).abs() < 1e-9);
        assert!((before.y - 87.20472440944882).abs() < 1e-9);
        original.move_number(3, (25, 80)).unwrap();
        assert!(
            matches!(&original.objects[2].kind, Kind::ValueTablePair(_, _, group) if group.anchor() == (25, 80))
        );
        let moved = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
        assert!(!moved.partial_view);
        assert_eq!(moved.position(10), Some(before));
        original.checkpoint();
        assert!(original.move_point(4, Point { x: 219., y: 135. }));
        let shifted = original.position(10).unwrap();
        assert!((shifted.x - before.x - 20.).abs() < 1e-9);
        assert!((shifted.y - before.y - 10.).abs() < 1e-9);
        original.undo();
        assert_eq!(original.position(10), Some(before));

        let mut generated = Document::new();
        generated.add_default_parameter((10, 5)).unwrap();
        generated.add_default_parameter((10, 38)).unwrap();
        generated.tabulate_default_pair((10, 71)).unwrap();
        generated
            .add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(generated.plot_default_pair_table(3).unwrap(), vec![10]);
        assert_eq!(
            generated.encode().unwrap(),
            include_bytes!("../tests/fixtures/plot_table_data_generated.gsp")
        );
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(10), Some(before));

        let mut two_rows = Document::new();
        two_rows.add_default_parameter((10, 5)).unwrap();
        two_rows.add_default_parameter((10, 38)).unwrap();
        two_rows.tabulate_default_pair((10, 71)).unwrap();
        two_rows.change_table_rows(3, true).unwrap();
        assert!(
            matches!(&two_rows.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.rows() == 2)
        );
        two_rows
            .add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(two_rows.plot_default_pair_table(3).unwrap(), vec![10]);
        assert_eq!(
            two_rows.encode().unwrap(),
            include_bytes!("../tests/fixtures/plot_table_data_two_rows_generated.gsp")
        );
        assert_eq!(two_rows.position(10), Some(before));
        let restored = Document::from_bytes(&two_rows.encode().unwrap(), None).unwrap();
        assert!(!restored.partial_view);
        assert!(
            matches!(&restored.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.rows() == 2)
        );

        let distinct_original = Document::from_bytes(
            include_bytes!("../tests/fixtures/plot_table_distinct_original.gsp"),
            None,
        )
        .unwrap();
        assert_eq!(distinct_original.objects.len(), 11);
        assert!(
            matches!(&distinct_original.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.pair_values() == Some(vec![[1., 1.], [2., 1.]]))
        );
        assert_eq!(
            distinct_original.objects[9].kind,
            Kind::PlottedPoint(1, 9, PlotPointsMode::Cartesian, [1., 1.])
        );
        assert_eq!(
            distinct_original.objects[10].kind,
            Kind::PlottedPoint(1, 9, PlotPointsMode::Cartesian, [2., 1.])
        );
        assert!(
            (distinct_original.position(11).unwrap().x - (199. + 2. * 37.79527559055118)).abs()
                < 1e-9
        );

        let mut distinct = Document::new();
        distinct.add_default_parameter((10, 5)).unwrap();
        distinct.add_default_parameter((10, 38)).unwrap();
        distinct.tabulate_default_pair((10, 71)).unwrap();
        distinct.change_table_rows(3, true).unwrap();
        distinct.set_second_parameter_value(2, 2).unwrap();
        assert!(matches!(&distinct.objects[1].kind, Kind::Parameter(group) if group.value() == 2.));
        assert!(
            matches!(&distinct.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.pair_values() == Some(vec![[1., 1.], [2., 1.]]))
        );
        distinct
            .add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(distinct.plot_default_pair_table(3).unwrap(), vec![10, 11]);
        assert_eq!(
            distinct.encode().unwrap(),
            include_bytes!("../tests/fixtures/plot_table_distinct_generated.gsp")
        );
        let reopened = Document::from_bytes(&distinct.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(11), distinct.position(11));
    }

    #[test]
    fn decimal_parameters_roundtrip_and_drive_path_points() {
        for (bytes, id, value) in [
            (
                include_bytes!("../tests/fixtures/table_t1_fraction_original.gsp").as_slice(),
                1,
                3.25,
            ),
            (
                include_bytes!("../tests/fixtures/table_t2_fraction_original.gsp").as_slice(),
                2,
                2.5,
            ),
            (
                include_bytes!("../tests/fixtures/table_t2_negative_original.gsp").as_slice(),
                2,
                -1.,
            ),
            (
                include_bytes!("../tests/fixtures/table_t2_negative_fraction_original.gsp")
                    .as_slice(),
                2,
                -2.5,
            ),
            (
                include_bytes!("../tests/fixtures/table_t2_multidigit_original.gsp").as_slice(),
                2,
                12.34,
            ),
        ] {
            let document = Document::from_bytes(bytes, None).unwrap();
            assert!(!document.partial_view);
            let Kind::Parameter(group) = &document.objects[id - 1].kind else {
                panic!("expected parameter")
            };
            assert!((group.value() - value).abs() < 1e-9);
            assert_eq!(document.encode().unwrap(), bytes);
        }
        let mut first = Document::new();
        first.add_default_parameter((10, 5)).unwrap();
        first.set_parameter_value(1, 3.25).unwrap();
        assert_eq!(
            first.encode().unwrap(),
            include_bytes!("../tests/fixtures/table_t1_fraction_generated.gsp")
        );
        let mut second = Document::new();
        second.add_default_parameter((10, 5)).unwrap();
        second.add_default_parameter((10, 38)).unwrap();
        second.set_parameter_value(2, -2.5).unwrap();
        assert_eq!(
            second.encode().unwrap(),
            include_bytes!("../tests/fixtures/table_t2_negative_fraction_generated.gsp")
        );
        second.set_parameter_value(2, 12.34).unwrap();
        assert_eq!(
            second.encode().unwrap(),
            include_bytes!("../tests/fixtures/table_t2_multidigit_generated.gsp")
        );
        assert!(second.set_parameter_value(2, 100.).is_err());
        assert!(second.set_parameter_value(2, 2.555).is_err());
        assert!(second.set_parameter_value(2, f64::NAN).is_err());

        let mut line = Document::new();
        line.add_default_parameter((10, 5)).unwrap();
        line.set_parameter_value(1, 3.25).unwrap();
        line.add(Kind::Free(Point { x: 150., y: 120. }));
        line.add(Kind::Free(Point { x: 180., y: 130. }));
        line.add(Kind::Line(2, 3));
        assert_eq!(line.plot_parameter_on_path(1, Some(4)).unwrap(), 5);
        assert_eq!(line.position(5), Some(Point { x: 247.5, y: 152.5 }));
        assert_eq!(
            line.encode().unwrap(),
            include_bytes!("../tests/fixtures/graph_plot_line_fraction_parameter_generated.gsp")
        );
        let reopened = Document::from_bytes(&line.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(5), line.position(5));

        for bytes in [
            include_bytes!("../tests/fixtures/table_pair_fraction_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/plot_table_fraction_base_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/plot_table_fraction_original.gsp").as_slice(),
        ] {
            let document = Document::from_bytes(bytes, None).unwrap();
            assert!(!document.partial_view);
            assert_eq!(document.encode().unwrap(), bytes);
            assert!(
                matches!(&document.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.pair_values() == Some(vec![[2.5, 1.]]))
            );
        }
        let mut table = Document::new();
        table.add_default_parameter((10, 5)).unwrap();
        table.add_default_parameter((10, 38)).unwrap();
        table.tabulate_default_pair((10, 71)).unwrap();
        table.set_parameter_value(2, 2.5).unwrap();
        table
            .add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(table.plot_default_pair_table(3).unwrap(), vec![10]);
        assert_eq!(
            table.objects[9].kind,
            Kind::PlottedPoint(1, 9, PlotPointsMode::Cartesian, [2.5, 1.])
        );
        assert!((table.position(10).unwrap().x - (199. + 2.5 * 37.79527559055118)).abs() < 1e-9);
        assert_eq!(
            table.encode().unwrap(),
            include_bytes!("../tests/fixtures/plot_table_fraction_generated.gsp")
        );
        let reopened = Document::from_bytes(&table.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(10), table.position(10));
        let plotted = table.position(10);
        table.set_parameter_value(2, 3.25).unwrap();
        assert_eq!(table.position(10), plotted);
        assert!(
            matches!(&table.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.pair_values() == Some(vec![[3.25, 1.]]))
        );
        assert_eq!(
            table.encode().unwrap(),
            include_bytes!("../tests/fixtures/plot_table_fraction_changed_generated.gsp")
        );
        let reopened = Document::from_bytes(&table.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(10), plotted);
        let original_changed =
            include_bytes!("../tests/fixtures/plot_table_fraction_after_value_change_original.gsp");
        let original = Document::from_bytes(original_changed, None).unwrap();
        assert!(!original.partial_view);
        assert_eq!(original.encode().unwrap(), original_changed);
        assert_eq!(original.position(10), plotted);
        assert!(
            matches!(&original.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.pair_values() == Some(vec![[3.25, 1.]]))
        );
    }

    #[test]
    fn trace_flags_follow_undo_and_round_trip_for_points_and_segments() {
        for (bytes, traced_id) in [
            (
                include_bytes!("../tests/fixtures/trace_point_original.gsp").as_slice(),
                1,
            ),
            (
                include_bytes!("../tests/fixtures/trace_segment_original.gsp").as_slice(),
                3,
            ),
            (
                include_bytes!("../tests/fixtures/trace_port_point_saved_by_wine.gsp").as_slice(),
                1,
            ),
            (
                include_bytes!("../tests/fixtures/trace_port_segment_saved_by_wine.gsp").as_slice(),
                3,
            ),
            (
                include_bytes!("../tests/fixtures/trace_port_point_dragged_by_wine.gsp").as_slice(),
                1,
            ),
        ] {
            let mut doc = Document::from_bytes(bytes, None).unwrap();
            assert!(!doc.partial_view);
            assert!(doc.objects[traced_id - 1].tracing);
            assert_eq!(doc.encode().unwrap(), bytes);
            doc.selected.insert(traced_id);
            assert_eq!(doc.set_selected_tracing(false).unwrap(), 1);
            assert!(!doc.objects[traced_id - 1].tracing);
            doc.undo();
            assert!(doc.objects[traced_id - 1].tracing);
            doc.redo();
            let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
            assert!(!reopened.objects[traced_id - 1].tracing);
        }

        let mut doc = Document::new();
        let id = doc.add(Kind::Free(Point { x: 200., y: 150. }));
        doc.selected.insert(id);
        assert_eq!(doc.set_selected_tracing(true).unwrap(), 1);
        let encoded = doc.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(reopened.objects[0].tracing);
        assert!(!reopened.partial_view);
        let clipboard = doc.copy_selected().unwrap();
        let pasted = doc.paste(&clipboard).unwrap();
        assert!(doc.objects[pasted[0] - 1].tracing);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(reopened.objects[pasted[0] - 1].tracing);
    }

    #[test]
    fn coordinate_measurements_decode_track_drag_and_reopen() {
        for (source, kind, expected) in [
            (
                include_bytes!("../tests/fixtures/coordinate_measure_menu257.gsp").as_slice(),
                CoordinateMeasureKind::Coordinates,
                -1.296458333333333,
            ),
            (
                include_bytes!("../tests/fixtures/coordinate_measure_menu274.gsp").as_slice(),
                CoordinateMeasureKind::Abscissa,
                -1.296458333333333,
            ),
            (
                include_bytes!("../tests/fixtures/coordinate_measure_menu291.gsp").as_slice(),
                CoordinateMeasureKind::Ordinate,
                0.132291666666667,
            ),
            (
                include_bytes!("../tests/fixtures/coordinate_measure_menu308.gsp").as_slice(),
                CoordinateMeasureKind::CoordinateDistance,
                1.3032,
            ),
        ] {
            let mut doc = Document::from_bytes(source, None).unwrap();
            assert!(!doc.partial_view);
            assert_eq!(doc.encode().unwrap(), source);
            let Kind::CoordinateMeasurement {
                kind: actual,
                first: 1,
                frame: 9,
                ..
            } = doc.objects[9].kind
            else {
                panic!("measurement not decoded")
            };
            assert_eq!(actual, kind);
            let before = doc.coordinate_measure_value(10).unwrap();
            let scalar = match before {
                CoordinateMeasureValue::Pair(p) => p.x,
                CoordinateMeasureValue::Scalar(v) => v,
            };
            assert!((scalar - expected).abs() < 0.01);
            doc.checkpoint();
            assert!(doc.move_point(1, Point { x: 170., y: 100. }));
            assert_ne!(doc.coordinate_measure_value(10), Some(before));
            doc.undo();
            assert_eq!(doc.coordinate_measure_value(10), Some(before));
            doc.redo();
            let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
            assert!(!reopened.partial_view);
            assert_eq!(
                reopened.coordinate_measure_value(10),
                doc.coordinate_measure_value(10)
            );
        }
    }

    #[test]
    fn new_coordinate_measurements_serialize_with_valid_marker_and_references() {
        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/coordinate_measure_base.gsp"),
            None,
        )
        .unwrap();
        let frame = doc.active_coordinate_system().unwrap();
        for (kind, first, second) in [
            (CoordinateMeasureKind::Coordinates, 1, None),
            (CoordinateMeasureKind::Abscissa, 2, None),
            (CoordinateMeasureKind::Ordinate, 2, None),
            (CoordinateMeasureKind::CoordinateDistance, 1, Some(4)),
        ] {
            doc.add(Kind::CoordinateMeasurement {
                kind,
                first,
                second,
                frame,
            });
        }
        let bytes = doc.encode().unwrap();
        let marker = records(&bytes)
            .unwrap()
            .into_iter()
            .find(|r| r.kind == 0x2328)
            .unwrap();
        assert_eq!(u32_at(&bytes, marker.data.start), 13);
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects.len(), 13);
        assert_eq!(
            reopened.coordinate_measure_text(11).as_deref(),
            Some("x_B = 1.35")
        );
        assert!(
            matches!(reopened.coordinate_measure_value(13), Some(CoordinateMeasureValue::Scalar(v)) if (v-1.30).abs()<0.01)
        );
    }
}
