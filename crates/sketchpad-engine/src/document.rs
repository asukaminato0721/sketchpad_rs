//! Document model and editing API. Implementations are grouped by responsibility
//! in the private `document/` modules.

mod actions;
mod boundary_intersection;
mod button_encoding;
mod calculations;
mod circle_interior;
pub mod colorized;
pub mod coordinate_points;
mod coordinates;
mod curve_intersection;
mod curve_points;
mod custom_transform_metadata;
mod data_function;
mod decode;
mod embedded_tools;
mod encode;
mod encode_pages;
mod encode_patches;
mod encode_templates;
mod evaluation_context;
pub use evaluation_context::EvaluatedDocument;
mod functions;
mod geometry;
mod history;
mod images;
mod iteration;
mod iteration_endpoint;
mod movement;
mod nested_iteration_scope;
mod point_curve;
mod referenced_captions;
pub mod referenced_text;
mod sector_boundary;
pub use movement::MovementValue;
mod kind;
mod legacy;
mod locus;
pub use locus::LocusStroke;
pub mod mapped_element;
mod sampled_mapped;
pub use sampled_mapped::SampledMappedDefinition;
mod sampled_image;
pub use sampled_image::{SampledImageDefinition, SampledImageDistortion, SampledImageMesh};
mod angle_marker;
mod measurements;
pub use angle_marker::{AngleMarkerGeometry, AngleMarkerStyle};
mod measurement_names;
mod pages;
mod parameter_names;
mod plotting;
mod point_numeric_labels;
mod records;
mod save;
mod segment_marker;
mod selection;
mod style;
mod tables;
mod text;
mod tool_creation;
mod tool_inputs;
mod transform_decode;
pub mod transform_evaluator;
pub mod transform_model;
mod transform_validation;
mod transform_wire;
mod transforms;
mod validation;

use crate::action_records::{
    AnimationSettings, MovementSpeed, PresentationMode, PresentationOptions, ScrollPlacement,
    VisibilityAction, VisibilityOptions,
};
use crate::coordinate_measure_records::CoordinateMeasureKind;
use crate::function_records::{FunctionPlotMode, PlotPresentation, SimpleFunction};
use crate::graph::GridForm;
use crate::plot_points_records::PlotPointsMode;
use crate::style_records::{
    LinePattern, LineWidth, PointStyle, RgbColor, SegmentLineStyle, VisualStyle,
};
use crate::table_records::{
    CalculationGroup, LengthPairCalculationGroup, MeasuredCalculationGroup, NumericUnit,
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

pub use custom_transform_metadata::CustomTransformDefinition;
pub use embedded_tools::EmbeddedToolSummary;
pub use records::{Record, records};
pub use tool_inputs::{EmbeddedToolInput, EmbeddedToolInputKind};

pub use sketchpad_ir::{DependencyIndex, LineMode, ObjectNode, Point};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IterationDepth {
    Fixed(u32),
    Value(usize),
}

/// Session-local colors applied to newly constructed objects. These are editor
/// preferences, not part of the saved sketch, so decoding a document starts
/// with the native defaults and leaves every imported object unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectColorDefaults {
    pub point: RgbColor,
    pub straight: RgbColor,
    pub round: RgbColor,
    pub interior: RgbColor,
    pub point_locus: RgbColor,
    pub plot: RgbColor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectClass {
    Point,
    Straight,
    Round,
    Interior,
    PointLocus,
    Plot,
}

impl ObjectColorDefaults {
    pub fn color(self, class: ObjectClass) -> RgbColor {
        match class {
            ObjectClass::Point => self.point,
            ObjectClass::Straight => self.straight,
            ObjectClass::Round => self.round,
            ObjectClass::Interior => self.interior,
            ObjectClass::PointLocus => self.point_locus,
            ObjectClass::Plot => self.plot,
        }
    }

    pub fn set_color(&mut self, class: ObjectClass, color: RgbColor) {
        match class {
            ObjectClass::Point => self.point = color,
            ObjectClass::Straight => self.straight = color,
            ObjectClass::Round => self.round = color,
            ObjectClass::Interior => self.interior = color,
            ObjectClass::PointLocus => self.point_locus = color,
            ObjectClass::Plot => self.plot = color,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct LineStyleOverrides {
    width: Option<LineWidth>,
    pattern: Option<LinePattern>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObjectStyleDefaults {
    point: Option<PointStyle>,
    straight: LineStyleOverrides,
    round: LineStyleOverrides,
    point_locus: LineStyleOverrides,
    plot: LineStyleOverrides,
}

impl ObjectStyleDefaults {
    fn line_mut(&mut self, class: ObjectClass) -> Option<&mut LineStyleOverrides> {
        match class {
            ObjectClass::Straight => Some(&mut self.straight),
            ObjectClass::Round => Some(&mut self.round),
            ObjectClass::PointLocus => Some(&mut self.point_locus),
            ObjectClass::Plot => Some(&mut self.plot),
            ObjectClass::Point | ObjectClass::Interior => None,
        }
    }

    pub fn set_point_style(&mut self, style: PointStyle) {
        self.point = Some(style);
    }

    pub fn set_line_width(&mut self, class: ObjectClass, width: LineWidth) -> bool {
        let Some(line) = self.line_mut(class) else {
            return false;
        };
        line.width = Some(width);
        true
    }

    pub fn set_line_pattern(&mut self, class: ObjectClass, pattern: LinePattern) -> bool {
        let Some(line) = self.line_mut(class) else {
            return false;
        };
        line.pattern = Some(pattern);
        true
    }

    fn apply(self, class: ObjectClass, style: VisualStyle) -> VisualStyle {
        match (class, style) {
            (ObjectClass::Point, VisualStyle::Point { shape, color }) => VisualStyle::Point {
                shape: self.point.unwrap_or(shape),
                color,
            },
            (class, VisualStyle::Stroke { line, color }) => {
                let overrides = match class {
                    ObjectClass::Straight => self.straight,
                    ObjectClass::Round => self.round,
                    ObjectClass::PointLocus => self.point_locus,
                    ObjectClass::Plot => self.plot,
                    ObjectClass::Point | ObjectClass::Interior => {
                        return VisualStyle::Stroke { line, color };
                    }
                };
                VisualStyle::Stroke {
                    line: SegmentLineStyle {
                        width: overrides.width.unwrap_or(line.width),
                        pattern: overrides.pattern.unwrap_or(line.pattern),
                    },
                    color,
                }
            }
            (_, style) => style,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeasurementPrecisionDefaults {
    angle: u16,
    distance: u16,
    other: u16,
}

impl MeasurementPrecisionDefaults {
    pub fn new(angle: u16, distance: u16, other: u16) -> Result<Self, String> {
        if [angle, distance, other]
            .into_iter()
            .any(|precision| precision > 5)
        {
            return Err("Precision must be between 0 and 5 decimal places".into());
        }
        Ok(Self {
            angle,
            distance,
            other,
        })
    }

    fn for_kind(self, kind: &Kind) -> u16 {
        match kind.numeric_unit() {
            Some(NumericUnit::Degrees) => self.angle,
            Some(NumericUnit::Length | NumericUnit::Area) => self.distance,
            _ => self.other,
        }
    }
}

impl Default for MeasurementPrecisionDefaults {
    fn default() -> Self {
        Self {
            angle: 2,
            distance: 2,
            other: 2,
        }
    }
}

impl Default for ObjectColorDefaults {
    fn default() -> Self {
        Self {
            point: RgbColor {
                red: 255,
                green: 0,
                blue: 0,
            },
            straight: RgbColor {
                red: 0,
                green: 0,
                blue: 128,
            },
            round: RgbColor {
                red: 0,
                green: 128,
                blue: 0,
            },
            interior: RgbColor {
                red: 255,
                green: 128,
                blue: 0,
            },
            point_locus: RgbColor {
                red: 128,
                green: 0,
                blue: 0,
            },
            plot: RgbColor {
                red: 128,
                green: 0,
                blue: 128,
            },
        }
    }
}

fn object_class(kind: &Kind) -> Option<ObjectClass> {
    match kind {
        Kind::FunctionPlot(..) | Kind::ParametricPlot(..) => Some(ObjectClass::Plot),
        Kind::ParameterLocus(..) => Some(ObjectClass::PointLocus),
        Kind::CircleInterior(..)
        | Kind::SectorBoundary(..)
        | Kind::CircularSegmentBoundary(..)
        | Kind::Polygon(..) => Some(ObjectClass::Interior),
        _ => match kind
            .transform_definition()
            .map(|definition| definition.element())
            .or_else(|| match kind {
                Kind::Colorized(definition) => Some(definition.element()),
                _ => None,
            }) {
            Some(transform_model::ElementType::Point) => Some(ObjectClass::Point),
            Some(
                transform_model::ElementType::Segment
                | transform_model::ElementType::Ray
                | transform_model::ElementType::Line,
            ) => Some(ObjectClass::Straight),
            Some(transform_model::ElementType::Circle | transform_model::ElementType::Arc) => {
                Some(ObjectClass::Round)
            }
            Some(
                transform_model::ElementType::Polygon
                | transform_model::ElementType::Sector
                | transform_model::ElementType::CircularSegment
                | transform_model::ElementType::Disk,
            ) => Some(ObjectClass::Interior),
            Some(transform_model::ElementType::Image) => None,
            None if kind::is_point_kind(kind) => Some(ObjectClass::Point),
            None if kind::is_straight_kind(kind) => Some(ObjectClass::Straight),
            None if kind.is_circle() || kind::is_arc_kind(kind) => Some(ObjectClass::Round),
            None => None,
        },
    }
}

const RANDOM_INTERIOR_COLORS: [RgbColor; 12] = [
    RgbColor {
        red: 255,
        green: 0,
        blue: 0,
    },
    RgbColor {
        red: 255,
        green: 128,
        blue: 0,
    },
    RgbColor {
        red: 220,
        green: 180,
        blue: 0,
    },
    RgbColor {
        red: 0,
        green: 160,
        blue: 0,
    },
    RgbColor {
        red: 0,
        green: 160,
        blue: 160,
    },
    RgbColor {
        red: 0,
        green: 96,
        blue: 255,
    },
    RgbColor {
        red: 0,
        green: 0,
        blue: 128,
    },
    RgbColor {
        red: 128,
        green: 0,
        blue: 128,
    },
    RgbColor {
        red: 255,
        green: 0,
        blue: 255,
    },
    RgbColor {
        red: 128,
        green: 0,
        blue: 0,
    },
    RgbColor {
        red: 138,
        green: 102,
        blue: 217,
    },
    RgbColor {
        red: 0,
        green: 192,
        blue: 255,
    },
];

fn initial_interior_color_cursor() -> usize {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| {
            duration.subsec_nanos() as usize % RANDOM_INTERIOR_COLORS.len()
        })
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImagePlacement {
    Independent([f64; 6]),
    Point {
        anchor: usize,
        relative: [f64; 6],
    },
    Rectangle {
        bottom_right: usize,
        top_left: usize,
    },
    Parallelogram {
        top_left: usize,
        bottom_right: usize,
        bottom_left: usize,
    },
    Transformed(transform_model::TransformDefinition),
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
    SampledImage(SampledImageDefinition),
    Colorized(colorized::ColorizedDefinition),
    PresentationButton {
        targets: Vec<usize>,
        mode: PresentationMode,
        options: PresentationOptions,
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
    SoundButton {
        functions: Vec<usize>,
        label: String,
        anchor: (i16, i16),
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
        placement: ImagePlacement,
    },
    /// A live view of an imported picture clipped to a polygon.
    CroppedImage {
        source: usize,
        polygon: usize,
    },
    /// A native picture transform whose source remains clipped to its live
    /// polygon before the shared affine operation is applied.
    TransformedCroppedImage(transform_model::TransformDefinition),
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
    /// A native snapshot table with arbitrary numeric or referenced-text columns.
    DataTable(Vec<usize>, TableGroup),
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
    /// Normalized physical perimeter parameter on an arc-and-chord boundary.
    PointOnCircularSegment(usize, f64),
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
    /// The first dependency is the differentiated function. Native cached
    /// derivatives may retain additional live numeric/function dependencies.
    DerivativeFunction(Vec<usize>, SimpleFunction),
    /// Code 122: a scalar function sampled from a drawing or picture's top edge.
    DataFunction {
        source: usize,
        frame: usize,
        smoothing: u32,
    },
    FunctionPlot(usize, usize, f64, f64, u32, FunctionPlotMode),
    ParametricPlot(usize, usize, usize, f64, f64, u32, PlotPointsMode),
    PathLiteralPoint(usize, f64),
    PathValuePoint(usize, usize),
    PlottedPoint(usize, PlotPointsMode, [f64; 2]),
    NumericCoordinatePoint(usize, usize, usize, PlotPointsMode),
    /// A live numeric magnitude plotted at `(value, 0)` in a coordinate frame.
    GraphValuePoint(usize, usize),
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
    /// Native code 44: minor-arc length on a circle between two point directions.
    CircleArcLength(usize, usize, usize),
    AngleMeasurement(usize, usize, usize),
    /// Positive Cartesian turn from vertex→first to vertex→last, in [0,360).
    PolarAngleValue(usize, usize, usize),
    /// Reverse Cartesian turn from vertex→first to vertex→last, in [0,360).
    VertexAngleValue(usize, usize, usize),
    /// Native code 114: signed shortest turn from vertex→first to vertex→last.
    SignedAngleValue(usize, usize, usize),
    /// A displayed three-point angle mark with a native arc count.
    AngleMarker(usize, usize, usize, u16),
    /// Native code 121: one to three congruence ticks on a finite segment.
    SegmentMarker {
        host: usize,
        parameter: f64,
        count: u16,
    },
    /// Native code 121 with flag `0x0001`: an arrow parallel to a segment.
    SegmentArrow {
        host: usize,
        parameter: f64,
        forward: bool,
    },
    /// A named numeric display whose value remains bound to its source.
    NumericAlias {
        source: usize,
        unit: NumericUnit,
    },
    PointDistance(usize, usize),
    PointLineDistance(usize, usize),
    PointNumericLabel(usize, usize),
    PointReferencedText {
        point: usize,
        source: usize,
        text: referenced_text::ReferencedText,
    },
    ReferencedText {
        parents: Vec<usize>,
        text: referenced_text::ReferencedText,
    },
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
    /// A native code-92 table containing the values carried by an iteration.
    IterationValues(usize, Vec<usize>),
    /// Native code 47: the length of the first segment divided by the second.
    SegmentRatio(usize, usize),
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
    /// Native omits Arrow-tool hit testing when an empty `0x232e` marker is present.
    pub arrow_selectable: bool,
    /// Native flag 2 suppresses presentation independently of ordinary hide/show.
    pub presentation_suppressed: bool,
    /// Native `0x090a` display mode for an iteration controller.
    pub iteration_final_only: bool,
    /// Native Plot-tab choices stored in a plot's `0x0902` descriptor.
    pub plot_presentation: Option<PlotPresentation>,
    pub tracing: bool,
    pub deleted: bool,
    pub midpoint_host: Option<usize>,
    /// Saved geometric-locus driver interval, in the host's native parameter units.
    pub locus_interval: Option<[f64; 2]>,
    pub point_offset: Option<usize>,
    pub constraint_offset: Option<usize>,
    pub header_offset: Option<usize>,
    pub label: Option<LabelAux>,
    pub visual_style: Option<VisualStyle>,
    pub angle_marker_style: Option<AngleMarkerStyle>,
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
    arrow_selectable: bool,
    iteration_final_only: bool,
    plot_presentation: Option<PlotPresentation>,
    tracing: bool,
    midpoint_host: Option<usize>,
    locus_interval: Option<[f64; 2]>,
    visual_style: Option<VisualStyle>,
    angle_marker_style: Option<AngleMarkerStyle>,
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
    fn deref(&self) -> &Self::Target {
        &self.node
    }
}
impl std::ops::DerefMut for Object {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}
impl sketchpad_ir::DependencyNode for Object {
    fn id(&self) -> usize {
        self.id
    }
    fn page(&self) -> usize {
        self.page
    }
    fn dependencies(&self) -> Vec<usize> {
        self.construction_dependencies()
    }
}
impl Document {
    pub fn edit_restriction(&self) -> Option<EditRestriction> {
        if !self.partial_view {
            return None;
        }
        if self
            .objects
            .iter()
            .any(|object| matches!(object.kind, Kind::Unsupported(_)))
        {
            Some(EditRestriction::UnsupportedObjects)
        } else if self.image_read_only {
            Some(EditRestriction::ReadOnlyImages)
        } else if self.page_count > 1 {
            Some(EditRestriction::MultiPageStructure)
        } else {
            Some(EditRestriction::UnrecognizedRecords)
        }
    }

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EditRestriction {
    UnsupportedObjects,
    ReadOnlyImages,
    MultiPageStructure,
    UnrecognizedRecords,
}

#[derive(Clone, Debug)]
pub struct DocumentSnapshot {
    objects: Vec<Object>,
    merge_aliases: BTreeMap<usize, usize>,
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
    object_color_defaults: ObjectColorDefaults,
    object_style_defaults: ObjectStyleDefaults,
    measurement_precision_defaults: MeasurementPrecisionDefaults,
    new_polygon_opacity: u16,
    randomize_interior_colors: bool,
    interior_color_cursor: usize,
    merge_aliases: BTreeMap<usize, usize>,
    measurement_names: BTreeMap<usize, u32>,
    evaluation_context: Option<evaluation_context::EvaluationContext>,
    original: Vec<u8>,
    run_font_families: Vec<String>,
    source_count: usize,
    last_moved_object: Option<usize>,
    custom_transform_definitions: Vec<CustomTransformDefinition>,
    safe_structure: bool,
    image_read_only: bool,
    pub undo: Vec<DocumentSnapshot>,
    pub redo: Vec<DocumentSnapshot>,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
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
    pub fn custom_transform_definitions(&self) -> &[CustomTransformDefinition] {
        &self.custom_transform_definitions
    }
    pub fn set_object_color_defaults(&mut self, defaults: ObjectColorDefaults) {
        self.object_color_defaults = defaults;
    }
    pub fn object_color_defaults(&self) -> ObjectColorDefaults {
        self.object_color_defaults
    }
    pub fn set_object_style_defaults(&mut self, defaults: ObjectStyleDefaults) {
        self.object_style_defaults = defaults;
    }
    pub fn set_measurement_precision_defaults(&mut self, defaults: MeasurementPrecisionDefaults) {
        self.measurement_precision_defaults = defaults;
    }
    pub fn set_new_polygon_opacity(&mut self, opacity: u16) {
        self.new_polygon_opacity = opacity;
    }
    pub fn set_randomize_interior_colors(&mut self, randomize: bool) {
        self.randomize_interior_colors = randomize;
    }
    pub fn object_class(&self, id: usize) -> Option<ObjectClass> {
        let mut current = id;
        for _ in 0..self.objects.len() {
            let object = self.objects.get(current.checked_sub(1)?)?;
            if object.deleted {
                return None;
            }
            match &object.kind {
                Kind::Locus(refs, _)
                    if refs
                        .first()
                        .and_then(|target| self.objects.get(target.saturating_sub(1)))
                        .is_some_and(|target| kind::is_point_kind(&target.kind)) =>
                {
                    return Some(ObjectClass::PointLocus);
                }
                Kind::IterationBinding(source, _) if *source != current => current = *source,
                kind => return object_class(kind),
            }
        }
        None
    }
    fn apply_default_calculation_precision(&self, kind: &mut Kind) {
        let precision = self.measurement_precision_defaults.for_kind(kind);
        match kind {
            Kind::Calculation(group) => group.set_precision(precision),
            Kind::MeasuredCalculation(_, group) => group.set_precision(precision),
            Kind::LengthPairCalculation(_, _, group) => group.set_precision(precision),
            Kind::ParameterCalculation(_, group) => group.set_precision(precision),
            _ => return,
        }
        .expect("validated default precision must fit native calculation groups");
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
        let precision = self.measurement_precision_defaults.for_kind(&kind);
        self.last_moved_object = None;
        let id = self.objects.len() + 1;
        let inherited_transform_source = kind
            .transform_definition()
            .filter(|definition| {
                matches!(
                    definition.element(),
                    transform_model::ElementType::Polygon
                        | transform_model::ElementType::Disk
                        | transform_model::ElementType::Sector
                        | transform_model::ElementType::CircularSegment
                        | transform_model::ElementType::Arc
                        | transform_model::ElementType::Ray
                        | transform_model::ElementType::Line
                )
            })
            .map(|definition| definition.source());
        let recolor = |style: VisualStyle, color: RgbColor| match style {
            VisualStyle::Point { shape, .. } => VisualStyle::Point { shape, color },
            VisualStyle::Stroke { line, .. } => VisualStyle::Stroke { line, color },
            VisualStyle::Polygon { opacity, .. } => VisualStyle::Polygon { opacity, color },
        };
        let object_color_defaults = self.object_color_defaults;
        let object_style_defaults = self.object_style_defaults;
        let new_polygon_opacity = self.new_polygon_opacity;
        let randomize_interior_colors = self.randomize_interior_colors;
        let mut interior_color_cursor = self.interior_color_cursor;
        let mut default_style = |kind: &Kind| {
            let style = default_visual_style(kind)?;
            Some(if let Some(class) = object_class(kind) {
                let color = if class == ObjectClass::Interior && randomize_interior_colors {
                    let color = RANDOM_INTERIOR_COLORS[interior_color_cursor];
                    interior_color_cursor =
                        (interior_color_cursor + 5) % RANDOM_INTERIOR_COLORS.len();
                    color
                } else {
                    object_color_defaults.color(class)
                };
                let style = object_style_defaults.apply(class, recolor(style, color));
                if matches!(kind, Kind::Polygon(_)) {
                    match style {
                        VisualStyle::Polygon { color, .. } => VisualStyle::Polygon {
                            color,
                            opacity: new_polygon_opacity,
                        },
                        style => style,
                    }
                } else {
                    style
                }
            } else {
                style
            })
        };
        let visual_style =
            if let Kind::IterationBinding(source, parent) = &kind {
                let header = &self.iteration_binding_template(*source, *parent)[8..36];
                if self
                    .objects
                    .get(source.saturating_sub(1))
                    .is_some_and(|o| kind::is_point_kind(&o.kind))
                {
                    crate::style_records::iteration_point_binding_visual_style(header)
                } else if self.objects.get(source.saturating_sub(1)).is_some_and(|o| {
                    o.kind.is_polygon() || matches!(o.kind, Kind::CircleInterior(_))
                }) {
                    crate::style_records::iteration_binding_visual_style(header, true)
                } else if self.objects.get(source.saturating_sub(1)).is_some_and(|o| {
                    kind::is_straight_kind(&o.kind)
                        || o.kind.is_circle()
                        || kind::is_arc_kind(&o.kind)
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
                    .or_else(|| default_style(&kind))
            } else if let Kind::Locus(refs, _) = &kind {
                refs.first()
                    .and_then(|id| id.checked_sub(1))
                    .and_then(|at| self.objects.get(at))
                    .and_then(|target| {
                        if kind::is_point_kind(&target.kind) {
                            Some(self.object_style_defaults.apply(
                                ObjectClass::PointLocus,
                                recolor(
                                    kind::point_locus_visual_style(),
                                    self.object_color_defaults.point_locus,
                                ),
                            ))
                        } else if kind::is_locus_straight_kind(&target.kind) {
                            Some(VisualStyle::Stroke {
                                line: crate::style_records::SegmentLineStyle {
                                    width: crate::style_records::LineWidth::Thin,
                                    pattern: crate::style_records::LinePattern::Solid,
                                },
                                color: crate::style_records::RgbColor {
                                    red: 102,
                                    green: 102,
                                    blue: 178,
                                },
                            })
                        } else {
                            None
                        }
                    })
            } else {
                default_style(&kind)
            };
        self.interior_color_cursor = interior_color_cursor;
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
        let measurement_precision = kind.has_measurement_precision().then_some(precision);
        let iteration_final_only = matches!(
            kind,
            Kind::MappedPointIteration {
                final_only: true,
                ..
            }
        );
        let plot_presentation = matches!(kind, Kind::FunctionPlot(..) | Kind::ParametricPlot(..))
            .then_some(PlotPresentation::default());
        let interior_order = if matches!(
            kind,
            Kind::Polygon(_) | Kind::SectorBoundary(_) | Kind::CircularSegmentBoundary(_)
        ) || kind.is_circle_interior()
        {
            Some(0)
        } else if kind.is_filled_region() {
            Some(1)
        } else {
            None
        };
        self.objects.push(Object {
            angle_marker_style: AngleMarkerStyle::for_kind(&kind),
            node: ObjectNode::new(id, self.active_page, kind),
            visual_style,
            interior_order,
            visible: true,
            arrow_selectable: true,
            presentation_suppressed: false,
            iteration_final_only,
            plot_presentation,
            tracing: false,
            deleted: false,
            midpoint_host,
            locus_interval: None,
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
