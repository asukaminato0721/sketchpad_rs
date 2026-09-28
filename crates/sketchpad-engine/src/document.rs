//! Document model and editing API. Implementations are grouped by responsibility
//! in the private `document/` modules.

mod actions;
mod calculations;
mod button_encoding;
mod coordinates;
pub mod colorized;
pub mod coordinate_points;
pub mod referenced_text;
mod referenced_captions;
mod decode;
mod embedded_tools;
mod encode;
mod encode_pages;
mod encode_patches;
mod encode_templates;
mod evaluation_context;
mod functions;
mod geometry;
mod history;
mod images;
mod iteration;
mod iteration_endpoint;
mod nested_iteration_scope;
mod boundary_intersection;
mod sector_boundary;
mod circle_interior;
mod point_curve;
mod curve_points;
mod curve_intersection;
mod movement;
pub use movement::MovementValue;
mod kind;
mod legacy;
mod locus;
pub use locus::LocusStroke;
pub mod mapped_element;
mod sampled_mapped;
pub use sampled_mapped::SampledMappedDefinition;
mod measurements;
mod angle_marker;
mod point_numeric_labels;
mod pages;
mod parameter_names;
mod measurement_names;
mod plotting;
mod records;
mod save;
mod selection;
mod style;
mod tables;
mod text;
mod tool_creation;
mod tool_inputs;
pub mod transform_evaluator;
pub mod transform_model;
mod transform_decode;
mod transform_validation;
mod transform_wire;
mod transforms;
mod validation;

use crate::action_records::{
    AnimationSettings, MovementSpeed, PresentationMode, ScrollPlacement, VisibilityAction, VisibilityOptions,
};
use crate::coordinate_measure_records::CoordinateMeasureKind;
use crate::function_records::SimpleFunction;
use crate::graph::GridForm;
use crate::plot_points_records::PlotPointsMode;
use crate::style_records::VisualStyle;
use crate::table_records::{
    CalculationGroup, LengthPairCalculationGroup, MeasuredCalculationGroup,
    NumericUnit, ParameterCalculationGroup, ParameterGroup, TableGroup,
};
use crate::text_object;
use crate::text_object::TextRect;
use crate::text_records::LabelAux;
use crate::text_style_records::FontState;
use crate::value_equation_records::ValueEquationKind;
use kind::default_visual_style;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

pub use embedded_tools::EmbeddedToolSummary;
pub use records::{Record, records};
pub use tool_inputs::{EmbeddedToolInput, EmbeddedToolInputKind};

pub use sketchpad_ir::{Point, LineMode, ObjectNode, DependencyIndex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IterationDepth {
    Fixed(u32),
    Value(usize),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    /// Abstract input category, valid only in an isolated tool prototype.
    ToolInput(EmbeddedToolInputKind),
    /// A prototype operation whose path position is assigned at invocation.
    ToolPointOnPath(usize),
    Transformed(transform_model::TransformDefinition),
    MappedElement(mapped_element::MappedElementDefinition),
    SampledMapped(SampledMappedDefinition),
    Colorized(colorized::ColorizedDefinition),
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
        pairs: Vec<(usize, usize)>,
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
        options: VisibilityOptions,
        caption_flags: Option<u32>,
        label: String,
        anchor: (i16, i16),
    },
    Free(Point),
    Image {
        png: Vec<u8>,
        transform: [f64; 6],
        /// A live point anchor; `transform` is then relative to that point.
        anchor: Option<usize>,
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
    /// Closed sector boundary retains its live Arc dependency.
    SectorBoundary(usize),
    /// Arc and its endpoint chord form a live circular-segment boundary.
    CircularSegmentBoundary(usize),
    CircleInterior(usize),
    /// Normalized physical perimeter parameter on a closed sector boundary.
    PointOnSector(usize, f64),
    /// A draggable point constrained to an evaluated point trace.
    PointOnCurve(usize, f64),
    PointOnPolygon(usize, usize, f64),
    /// Native midpoint construction using a finite segment as its dependency.
    SegmentMidpoint(usize),
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
    ReferencedFunction(Vec<usize>, SimpleFunction),
    DerivativeFunction(usize, SimpleFunction),
    FunctionPlot(usize, usize, f64, f64, u32),
    ParametricPlot(usize, usize, usize, f64, f64, u32, PlotPointsMode),
    PathLiteralPoint(usize, f64),
    PathValuePoint(usize, usize),
    PlottedPoint(usize, PlotPointsMode, [f64; 2]),
    NumericCoordinatePoint(usize, usize, usize),
    CoordinatePoint(coordinate_points::CoordinatePointDefinition),
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
    /// Positive Cartesian turn from vertex→first to vertex→last, in [0,360).
    PolarAngleValue(usize, usize, usize),
    /// Reverse Cartesian turn from vertex→first to vertex→last, in [0,360).
    VertexAngleValue(usize, usize, usize),
    /// A displayed three-point angle mark with a native arc count.
    AngleMarker(usize, usize, usize, u16),
    /// A named numeric display whose value remains bound to its source.
    NumericAlias { source: usize, unit: NumericUnit },
    PointDistance(usize, usize),
    PointLineDistance(usize, usize),
    PointNumericLabel(usize, usize),
    PointReferencedText { point: usize, source: usize, text: referenced_text::ReferencedText },
    ReferencedText { parents: Vec<usize>, text: referenced_text::ReferencedText },
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
    /// A Native code88 point in the terminal scope of a single mapped binding.
    IterationPointAlias(usize),
    IterationValues(usize, usize),
    PointRatio(usize, usize, usize),
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
    /// Circle arc defined by center, radius/start point and end direction.
    CenterArc(usize, usize, usize),
    Intersection(usize, usize),
    /// First strictly forward polygon boundary event on an ordered straight carrier.
    BoundaryIntersection(usize, usize),
    /// A ranked, clamped crossing of an ordered carrier and a point curve.
    CurveIntersection(usize, usize, u32),
    LineCircleIntersection(usize, usize, usize),
    CircleCircleIntersection(usize, usize, usize),
    Unsupported(u16),
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
    pub node: ObjectNode<Kind>,
    pub visible: bool,
    /// Native flag 2 suppresses presentation independently of ordinary hide/show.
    pub presentation_suppressed: bool,
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

impl std::ops::Deref for Object {
    type Target = ObjectNode<Kind>;
    fn deref(&self) -> &Self::Target { &self.node }
}
impl std::ops::DerefMut for Object {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.node }
}
impl sketchpad_ir::DependencyNode for Object {
    fn id(&self) -> usize { self.id }
    fn page(&self) -> usize { self.page }
    fn dependencies(&self) -> Vec<usize> { self.construction_dependencies() }
}
impl Document {
    /// Derive dependency indexes from the authoritative semantic objects.
    pub fn dependency_index(&self) -> Result<DependencyIndex, sketchpad_ir::DependencyError> {
        DependencyIndex::build(&self.objects)
    }
}

impl Object {
    /// Native midpoint records reference their segment host, even when the
    /// convenient point-pair kind retains its endpoint IDs.
    pub(super) fn construction_dependencies(&self) -> Vec<usize> {
        if matches!(self.kind, Kind::Midpoint(..))
            && let Some(host) = self.midpoint_host
        {
            vec![host]
        } else {
            self.kind.dependencies()
        }
    }
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
    measurement_names: BTreeMap<usize, u32>,
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
    measurement_names: BTreeMap<usize, u32>,
    evaluation_context: Option<evaluation_context::EvaluationContext>,
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
        let template = include_bytes!("../../../assets/point_template.gsp");
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
        let (kind, explicit_midpoint_host) = match kind {
            Kind::SegmentMidpoint(host) => {
                match self.objects.get(host.saturating_sub(1)).map(|o| &o.kind) {
                    Some(Kind::Segment(a, b)) => (Kind::Midpoint(*a, *b), Some(host)),
                    _ => (Kind::SegmentMidpoint(host), None),
                }
            }
            other => (other, None),
        };
        self.last_moved_object = None;
        let id = self.objects.len() + 1;
        let inherited_transform_source = kind
            .transform_definition()
            .filter(|definition| {
                matches!(
                    definition.element(),
                    transform_model::ElementType::Polygon
                        | transform_model::ElementType::Arc
                        | transform_model::ElementType::Ray
                        | transform_model::ElementType::Line
                )
            })
            .map(|definition| definition.source());
        let visual_style = if let Kind::IterationBinding(source, parent) = &kind {
            let header = &self.iteration_binding_template(*source, *parent)[8..36];
            if self
                .objects
                .get(source.saturating_sub(1))
                .is_some_and(|o| kind::is_point_kind(&o.kind))
            {
                crate::style_records::iteration_point_binding_visual_style(header)
            } else if self
                .objects
                .get(source.saturating_sub(1))
                .is_some_and(|o| o.kind.is_polygon() || matches!(o.kind, Kind::CircleInterior(_)))
            {
                crate::style_records::iteration_binding_visual_style(header, true)
            } else if self.objects.get(source.saturating_sub(1)).is_some_and(|o| {
                kind::is_straight_kind(&o.kind) || o.kind.is_circle() || kind::is_arc_kind(&o.kind)
                    || kind::is_straight_locus(&self.objects, &o.kind)
                    || locus::is_point_locus(&self.objects, &o.kind)
                    || matches!(o.kind, Kind::SampledMapped(..))
            }) {
                crate::style_records::iteration_binding_visual_style(header, false)
            } else {
                None
            }
        } else if let Some(source) = inherited_transform_source {
            self.objects
                .get(source.saturating_sub(1))
                .and_then(|object| object.visual_style)
                .or_else(|| default_visual_style(&kind))
        } else if let Kind::Locus(refs, _) = &kind {
            refs.first()
                .and_then(|id| id.checked_sub(1))
                .and_then(|at| self.objects.get(at))
                .and_then(|target| {
                    if kind::is_point_kind(&target.kind) {
                        Some(kind::point_locus_visual_style())
                    } else if kind::is_locus_straight_kind(&target.kind) {
                        Some(VisualStyle::Stroke {
                            line: crate::style_records::SegmentLineStyle {
                                width: crate::style_records::LineWidth::Thin,
                                pattern: crate::style_records::LinePattern::Solid,
                            },
                            color: crate::style_records::RgbColor { red: 102, green: 102, blue: 178 },
                        })
                    } else { None }
                })
        } else {
            default_visual_style(&kind)
        };
        let midpoint_host = if explicit_midpoint_host.is_some() {
            explicit_midpoint_host
        } else if let Kind::Midpoint(a, b) = kind {
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
        let interior_order = if matches!(kind, Kind::Polygon(_) | Kind::SectorBoundary(_) | Kind::CircularSegmentBoundary(_)) || kind.is_circle_interior() {
            Some(0)
        } else if kind.is_polygon() {
            Some(1)
        } else {
            None
        };
        self.objects.push(Object {
            node: ObjectNode::new(id, self.active_page, kind),
            visual_style,
            interior_order,
            visible: true,
            presentation_suppressed: false,
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
