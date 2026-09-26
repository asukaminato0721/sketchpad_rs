//! Document model and editing API. Implementations are grouped by responsibility
//! in the private `document/` modules.

mod actions;
mod calculations;
mod coordinates;
mod decode;
mod encode;
mod encode_pages;
mod encode_patches;
mod encode_templates;
mod embedded_tools;
mod functions;
mod geometry;
mod history;
mod images;
mod iteration;
mod kind;
mod legacy;
mod locus;
mod measurements;
mod pages;
mod parameter_names;
mod plotting;
mod records;
mod save;
mod selection;
mod style;
mod tables;
mod text;
mod transforms;
mod validation;

use crate::action_records::{
    AnimationSettings, MovementSpeed, PresentationMode, ScrollPlacement, VisibilityAction,
};
use crate::coordinate_measure_records::CoordinateMeasureKind;
use crate::function_records::SimpleFunction;
use crate::graph::GridForm;
use crate::plot_points_records::PlotPointsMode;
use crate::style_records::VisualStyle;
use crate::table_records::{
    CalculationGroup, LengthPairCalculationGroup, MeasuredCalculationGroup,
    ParameterCalculationGroup, ParameterGroup, TableGroup,
};
use crate::text_object;
use crate::text_object::TextRect;
use crate::text_records::LabelAux;
use crate::text_style_records::FontState;
use crate::value_equation_records::ValueEquationKind;
use kind::default_visual_style;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

pub use records::{Record, records};
pub use embedded_tools::EmbeddedToolSummary;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IterationDepth {
    Fixed(u32),
    Value(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    PresentationButton {
        targets: Vec<usize>,
        mode: PresentationMode,
        label: String,
        anchor: (i16, i16),
    },
    LinkUrlButton {
        url: String,
        label: String,
        anchor: (i16, i16),
    },
    LinkPageButton {
        target_page: usize,
        target_button_ordinal: Option<usize>,
        label: String,
        anchor: (i16, i16),
    },
    ScrollButton {
        target: usize,
        placement: ScrollPlacement,
        label: String,
        anchor: (i16, i16),
    },
    MovementButton {
        source: usize,
        destination: usize,
        speed: MovementSpeed,
        follow: bool,
        label: String,
        anchor: (i16, i16),
    },
    AnimationButton {
        targets: Vec<usize>,
        label: String,
        anchor: (i16, i16),
        settings: Vec<AnimationSettings>,
    },
    HideShowButton {
        targets: Vec<usize>,
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
    MeasuredCalculation(usize, MeasuredCalculationGroup),
    LengthPairCalculation(usize, usize, LengthPairCalculationGroup),
    ParameterCalculation(usize, ParameterCalculationGroup),
    ValueTable(usize, TableGroup),
    ValueTablePair(usize, usize, TableGroup),
    PointOnSegment(usize, f64),
    PointOnRay(usize, f64),
    PointOnLine(usize, f64),
    PointOnCircle(usize, f64, f64),
    PointOnArc(usize, f64),
    PointOnPolygon(usize, usize, f64),
    GraphHorizontalUnit(usize, f64),
    GraphVerticalUnit(usize, f64),
    GraphAxis(usize, usize, u32),
    CircleAxis(usize, u32),
    DistanceAxis(usize, usize, u32),
    DistanceVerticalUnit(usize, f64, bool),
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
    DerivativeFunction(usize, SimpleFunction),
    FunctionPlot(usize, usize, f64, f64, u32),
    ParametricPlot(usize, usize, usize, f64, f64, u32, PlotPointsMode),
    PathLiteralPoint(usize, f64),
    PathValuePoint(usize, usize),
    PlottedPoint(usize, usize, PlotPointsMode, [f64; 2]),
    NumericCoordinatePoint(usize, usize, usize),
    // Parameter ranges retain native units (radians for angles). Public range
    // access and editing use the driver's displayed numeric units.
    ParameterLocus(Vec<usize>, f64, f64, u32),
    SegmentLength(usize),
    PolygonPerimeter(usize),
    CircleCircumference(usize),
    CircleRadius(usize),
    CircleArea(usize),
    PolygonArea(usize),
    ArcAngle(usize),
    ArcLength(usize),
    AngleMeasurement(usize, usize, usize),
    PointDistance(usize, usize),
    PointLineDistance(usize, usize),
    PointNumericLabel(usize, usize),
    PointParameterIteration(usize, usize, usize, usize, usize, IterationDepth),
    BranchingPolygonIteration {
        seed: usize,
        first: usize,
        second: usize,
        polygon: usize,
        segments: Vec<usize>,
        depth: IterationDepth,
    },
    MappedPointIteration {
        seeds: Vec<usize>,
        branches: Vec<Vec<usize>>,
        carried: Vec<usize>,
        depth: IterationDepth,
        final_only: bool,
    },
    IterationValues(usize, usize),
    PointRatio(usize, usize, usize),
    PolarTranslatedPoint(usize, f64, f64, f64),
    PolarTranslatedSegment(usize, f64, f64, f64),
    PolarTranslatedCircle(usize, f64, f64, f64),
    PolarTranslatedPolygon(usize, f64, f64, f64),
    FixedRectangularTranslatedPoint(usize, f64, f64, f64, f64),
    FixedRectangularTranslatedSegment(usize, f64, f64, f64, f64),
    FixedRectangularTranslatedCircle(usize, f64, f64, f64, f64),
    FixedRectangularTranslatedPolygon(usize, f64, f64, f64, f64),
    MarkedDistanceTranslatedPoint(usize, usize, f64),
    MarkedDistanceTranslatedSegment(usize, usize, f64),
    MarkedDistanceTranslatedCircle(usize, usize, f64),
    MarkedDistanceTranslatedPolygon(usize, usize, f64),
    MixedRectangularTranslatedPoint(usize, usize, f64, bool),
    MixedRectangularTranslatedSegment(usize, usize, f64, bool),
    MixedRectangularTranslatedCircle(usize, usize, f64, bool),
    MixedRectangularTranslatedPolygon(usize, usize, f64, bool),
    RectangularMarkedTranslatedPoint(usize, usize, usize),
    RectangularMarkedTranslatedSegment(usize, usize, usize),
    RectangularMarkedTranslatedCircle(usize, usize, usize),
    RectangularMarkedTranslatedPolygon(usize, usize, usize),
    VectorTranslatedPoint(usize, usize, usize),
    VectorTranslatedSegment(usize, usize, usize),
    VectorTranslatedCircle(usize, usize, usize),
    VectorTranslatedPolygon(usize, usize, usize),
    AngleRotatedPoint(usize, usize, usize, usize, usize),
    AngleRotatedSegment(usize, usize, usize, usize, usize),
    AngleRotatedCircle(usize, usize, usize, usize, usize),
    AngleRotatedPolygon(usize, usize, usize, usize, usize),
    RatioScaledPoint(usize, usize, usize, usize, usize),
    RatioScaledCircle(usize, usize, usize, usize, usize),
    RatioScaledSegment(usize, usize, usize, usize, usize),
    RatioScaledPolygon(usize, usize, usize, usize, usize),
    ReflectedPoint(usize, usize),
    ReflectedCircle(usize, usize),
    ReflectedSegment(usize, usize),
    ReflectedPolygon(usize, usize),
    ScaledPoint(usize, usize, f64),
    ScaledCircle(usize, usize, f64),
    ScaledSegment(usize, usize, f64),
    ScaledRay(usize, usize, f64),
    ScaledLine(usize, usize, f64),
    ScaledPolygon(usize, usize, f64),
    RotatedPoint(usize, usize, f64),
    CalculatedRatioScaledPoint(usize, usize, usize),
    CalculatedRatioScaledSegment(usize, usize, usize),
    CalculatedRatioScaledRay(usize, usize, usize),
    CalculatedRatioScaledLine(usize, usize, usize),
    CalculatedRatioScaledCircle(usize, usize, usize),
    CalculatedRatioScaledPolygon(usize, usize, usize),
    CalculatedAngleRotatedPoint(usize, usize, usize),
    CalculatedAngleRotatedSegment(usize, usize, usize),
    CalculatedAngleRotatedRay(usize, usize, usize),
    CalculatedAngleRotatedLine(usize, usize, usize),
    CalculatedAngleRotatedCircle(usize, usize, usize),
    CalculatedAngleRotatedPolygon(usize, usize, usize),
    RotatedSegment(usize, usize, f64),
    RotatedRay(usize, usize, f64),
    RotatedLine(usize, usize, f64),
    RotatedCircle(usize, usize, f64),
    RotatedPolygon(usize, usize, f64),
    Midpoint(usize, usize),
    /// Native code 35. The reference list records the target, the driver's
    /// path, the driver, and the construction chain leading to the target.
    Locus(Vec<usize>, u32),
    /// A single point seed mapped to its first image, repeated at fixed or live depth.
    PointIteration(usize, usize, IterationDepth),
    SegmentIteration(usize, usize, usize, IterationDepth),
    /// Carry a circle centered on the seed through its mapped image.
    CircleIteration(usize, usize, usize, IterationDepth),
    RayIteration(usize, usize, usize, usize),
    IterationBinding(usize, usize),
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextStyleCommand {
    Bold,
    Italic,
    Underline,
    IncreaseSize,
    DecreaseSize,
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
    pub interior_order: Option<i32>,
    pub text_style_override: Option<FontState>,
    pub rich_text_override: Option<text_object::RichTextTemplate>,
    pub function_anchor: Option<(i16, i16)>,
    pub function_name: Option<String>,
    pub measurement_precision: Option<u16>,
    pub measurement_anchor: Option<(i16, i16)>,
}
#[derive(Clone, Debug)]
pub struct Clipboard {
    hide_on_paste: BTreeSet<usize>,
    rename_on_paste: Option<usize>,
    objects: Vec<ClipboardObject>,
}
#[derive(Clone, Debug)]
struct ClipboardObject {
    old_id: usize,
    kind: Kind,
    visible: bool,
    tracing: bool,
    midpoint_host: Option<usize>,
    visual_style: Option<VisualStyle>,
    interior_order: Option<i32>,
    text_style: Option<FontState>,
    label: Option<LabelAux>,
    rich_text: Option<text_object::RichTextTemplate>,
    frame_style: Option<(GridForm, bool)>,
    function_anchor: Option<(i16, i16)>,
    function_name: Option<String>,
    measurement_precision: Option<u16>,
    measurement_anchor: Option<(i16, i16)>,
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
pub struct DocumentSnapshot {
    objects: Vec<Object>,
    snap_points: Vec<bool>,
    grid_dotted: Vec<bool>,
    grid_form: Vec<GridForm>,
    page_names: Vec<Option<String>>,
    angle_unit_radians: bool,
    marked_coordinate_system: Vec<Option<usize>>,
    frame_styles: BTreeMap<usize, (GridForm, bool)>,
    run_font_families: Vec<String>,
    last_moved_object: Option<usize>,
    page_count: usize,
    active_page: usize,
    safe_structure: bool,
    partial_view: bool,
    source: Option<(Vec<u8>, usize, bool)>,
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
    pub page_names: Vec<Option<String>>,
    pub angle_unit_radians: bool,
    pub marked_coordinate_system: Vec<Option<usize>>,
    pub frame_styles: BTreeMap<usize, (GridForm, bool)>,
    original: Vec<u8>,
    run_font_families: Vec<String>,
    source_count: usize,
    last_moved_object: Option<usize>,
    safe_structure: bool,
    image_read_only: bool,
    pub undo: Vec<DocumentSnapshot>,
    pub redo: Vec<DocumentSnapshot>,
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
    pub fn add(&mut self, kind: Kind) -> usize {
        self.checkpoint();
        self.append_object(kind)
    }
    fn append_object(&mut self, kind: Kind) -> usize {
        self.last_moved_object = None;
        let id = self.objects.len() + 1;
        let visual_style = if let Kind::ReflectedPolygon(source, _)
        | Kind::VectorTranslatedPolygon(source, _, _)
        | Kind::PolarTranslatedPolygon(source, _, _, _)
        | Kind::FixedRectangularTranslatedPolygon(source, _, _, _, _)
        | Kind::MarkedDistanceTranslatedPolygon(source, _, _)
        | Kind::RectangularMarkedTranslatedPolygon(source, _, _)
        | Kind::MixedRectangularTranslatedPolygon(source, _, _, _)
        | Kind::RotatedPolygon(source, _, _)
        | Kind::CalculatedAngleRotatedPolygon(source, _, _)
        | Kind::CalculatedRatioScaledPolygon(source, _, _)
        | Kind::AngleRotatedPolygon(source, _, _, _, _)
        | Kind::ScaledPolygon(source, _, _)
        | Kind::RatioScaledPolygon(source, _, _, _, _)
        | Kind::CalculatedRatioScaledRay(source, _, _)
        | Kind::CalculatedRatioScaledLine(source, _, _)
        | Kind::CalculatedAngleRotatedRay(source, _, _)
        | Kind::CalculatedAngleRotatedLine(source, _, _)
        | Kind::RotatedRay(source, _, _)
        | Kind::RotatedLine(source, _, _)
        | Kind::ScaledRay(source, _, _)
        | Kind::ScaledLine(source, _, _) = kind
        {
            self.objects
                .get(source.saturating_sub(1))
                .and_then(|object| object.visual_style)
                .or_else(|| default_visual_style(&kind))
        } else if let Kind::Locus(refs,_) = &kind {
            refs.first().and_then(|id| id.checked_sub(1)).and_then(|at| self.objects.get(at))
                .filter(|target| kind::is_point_kind(&target.kind))
                .map(|_| kind::point_locus_visual_style())
        } else {
            default_visual_style(&kind)
        };
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
        let measurement_precision = kind.has_measurement_precision().then_some(2);
        let interior_order = if matches!(kind, Kind::Polygon(_)) {
            Some(0)
        } else if kind.is_polygon() {
            Some(1)
        } else {
            None
        };
        self.objects.push(Object {
            id,
            page: self.active_page,
            visual_style,
            interior_order,
            kind,
            visible: true,
            tracing: false,
            deleted: false,
            midpoint_host,
            point_offset: None,
            constraint_offset: None,
            header_offset: None,
            label: None,
            text_style_override: None,
            rich_text_override: None,
            function_anchor: None,
            function_name: None,
            measurement_precision,
            measurement_anchor: None,
        });
        id
    }
}

#[cfg(test)]
mod tests;
