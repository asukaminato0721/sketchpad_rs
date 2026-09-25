use crate::action_records::{
    AnimationButtonGroup, AnimationSettings, HideShowButtonGroup, LinkPageButtonGroup,
    LinkUrlButtonGroup, MovementButtonGroup, MovementSpeed, PresentationButtonGroup,
    PresentationMode, ScrollButtonGroup, ScrollPlacement, VisibilityAction,
};
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
use crate::text_style_records::{self, FontState};
use crate::value_equation_records::{self, CartesianFrame, EquationValue, ValueEquationKind};
use std::{
    collections::{BTreeMap, BTreeSet},
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
    pub fn is_polygon(&self) -> bool {
        matches!(
            self,
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
                | Kind::RatioScaledPolygon(_, _, _, _, _)
        )
    }
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
                | Kind::PolygonArea(_)
                | Kind::AngleMeasurement(_, _, _)
                | Kind::SlopeMeasurement(_, _)
                | Kind::CoordinateMeasurement { .. }
                | Kind::ValueEquation { .. }
                | Kind::ArcAngle(_)
                | Kind::ArcLength(_)
        )
    }
    pub fn is_action_button(&self) -> bool {
        matches!(
            self,
            Kind::HideShowButton { .. }
                | Kind::AnimationButton { .. }
                | Kind::MovementButton { .. }
                | Kind::ScrollButton { .. }
                | Kind::PresentationButton { .. }
                | Kind::LinkUrlButton { .. }
                | Kind::LinkPageButton { .. }
        )
    }
    pub fn dependencies(&self) -> Vec<usize> {
        if let Kind::HideShowButton { targets, .. } = self {
            return targets.clone();
        }
        if let Kind::AnimationButton { targets, .. } = self {
            return targets.clone();
        }
        if let Kind::MovementButton {
            source,
            destination,
            ..
        } = self
        {
            return vec![*source, *destination];
        }
        if let Kind::ScrollButton { target, .. } = self {
            return vec![*target];
        }
        if let Kind::PresentationButton { targets, .. } = self {
            return targets.clone();
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
        if let Kind::Locus(refs, _) = self {
            return refs.clone();
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
            | Kind::PolarTranslatedSegment(host, _, _, _)
            | Kind::PolarTranslatedCircle(host, _, _, _)
            | Kind::PolarTranslatedPolygon(host, _, _, _)
            | Kind::FixedRectangularTranslatedPoint(host, _, _, _, _)
            | Kind::FixedRectangularTranslatedSegment(host, _, _, _, _)
            | Kind::FixedRectangularTranslatedCircle(host, _, _, _, _)
            | Kind::FixedRectangularTranslatedPolygon(host, _, _, _, _)
            | Kind::PathLiteralPoint(host, _)
            | Kind::SegmentLength(host)
            | Kind::PolygonPerimeter(host)
            | Kind::CircleCircumference(host)
            | Kind::CircleRadius(host)
            | Kind::CircleArea(host)
            | Kind::PolygonArea(host)
            | Kind::ArcAngle(host)
            | Kind::ArcLength(host)
            | Kind::CircleAxis(host, _) => vec![host],
            Kind::DistanceVerticalUnit(host, _, _) => vec![host],
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
            | Kind::ReflectedSegment(a, b)
            | Kind::ReflectedPolygon(a, b)
            | Kind::PointDistance(a, b) => vec![a, b],
            Kind::GraphAxis(a, b, _)
            | Kind::DistanceAxis(a, b, _)
            | Kind::CoordinateSystem(a, b)
            | Kind::SlopeMeasurement(a, b)
            | Kind::FunctionPlot(a, b, _, _, _)
            | Kind::PathValuePoint(a, b)
            | Kind::PlottedPoint(a, b, _, _) => vec![a, b],
            Kind::ParametricPlot(x, y, frame, _, _, _, _) => vec![x, y, frame],
            Kind::DerivativeFunction(source, _) => vec![source],
            Kind::ScaledPoint(source, center, _)
            | Kind::ScaledCircle(source, center, _)
            | Kind::ScaledSegment(source, center, _)
            | Kind::ScaledPolygon(source, center, _)
            | Kind::RotatedPoint(source, center, _)
            | Kind::RotatedSegment(source, center, _)
            | Kind::RotatedCircle(source, center, _)
            | Kind::RotatedPolygon(source, center, _) => {
                vec![source, center]
            }
            Kind::Bisector(a, b, c)
            | Kind::ThreePointArc(a, b, c)
            | Kind::ArcOnCircle(a, b, c)
            | Kind::AngleMeasurement(a, b, c) => {
                vec![a, b, c]
            }
            Kind::PointRatio(a, b, c) => vec![a, b, c],
            Kind::MarkedDistanceTranslatedPoint(source, distance, _) => vec![source, distance],
            Kind::MarkedDistanceTranslatedSegment(source, distance, _)
            | Kind::MarkedDistanceTranslatedCircle(source, distance, _)
            | Kind::MarkedDistanceTranslatedPolygon(source, distance, _) => vec![source, distance],
            Kind::MixedRectangularTranslatedPoint(source, distance, _, _) => vec![source, distance],
            Kind::MixedRectangularTranslatedPolygon(source, distance, _, _) => {
                vec![source, distance]
            }
            Kind::RectangularMarkedTranslatedPoint(source, horizontal, vertical) => {
                vec![source, horizontal, vertical]
            }
            Kind::RectangularMarkedTranslatedPolygon(source, horizontal, vertical) => {
                vec![source, horizontal, vertical]
            }
            Kind::VectorTranslatedPoint(source, start, end) => vec![source, start, end],
            Kind::VectorTranslatedSegment(source, start, end)
            | Kind::VectorTranslatedCircle(source, start, end)
            | Kind::VectorTranslatedPolygon(source, start, end) => vec![source, start, end],
            Kind::AngleRotatedPoint(source, center, first, vertex, last)
            | Kind::AngleRotatedSegment(source, center, first, vertex, last)
            | Kind::AngleRotatedCircle(source, center, first, vertex, last)
            | Kind::AngleRotatedPolygon(source, center, first, vertex, last) => {
                vec![source, center, first, vertex, last]
            }
            Kind::RatioScaledPoint(source, center, origin, base, target)
            | Kind::RatioScaledCircle(source, center, origin, base, target)
            | Kind::RatioScaledSegment(source, center, origin, base, target)
            | Kind::RatioScaledPolygon(source, center, origin, base, target) => {
                vec![source, center, origin, base, target]
            }
            Kind::Polygon(_) => unreachable!(),
            Kind::HideShowButton { .. }
            | Kind::AnimationButton { .. }
            | Kind::MovementButton { .. }
            | Kind::ScrollButton { .. }
            | Kind::PresentationButton { .. } => unreachable!(),
            Kind::LinkUrlButton { .. } => vec![],
            Kind::LinkPageButton { .. } => vec![],
            Kind::CoordinateMeasurement { .. } => unreachable!(),
            Kind::ValueEquation { .. } => unreachable!(),
            Kind::ValueTablePair(_, _, _) => unreachable!(),
            Kind::Locus(_, _) => unreachable!(),
        }
    }
    fn remap(&self, ids: &[usize]) -> Self {
        let m = |id: usize| ids[id];
        if let Kind::HideShowButton {
            targets,
            action,
            label,
            anchor,
        } = self
        {
            return Kind::HideShowButton {
                targets: targets.iter().map(|&target| m(target)).collect(),
                action: *action,
                label: label.clone(),
                anchor: *anchor,
            };
        }
        if let Kind::AnimationButton {
            targets,
            label,
            anchor,
            settings,
        } = self
        {
            return Kind::AnimationButton {
                targets: targets.iter().map(|&target| m(target)).collect(),
                label: label.clone(),
                anchor: *anchor,
                settings: settings.clone(),
            };
        }
        if let Kind::MovementButton {
            source,
            destination,
            speed,
            follow,
            label,
            anchor,
        } = self
        {
            return Kind::MovementButton {
                source: m(*source),
                destination: m(*destination),
                speed: *speed,
                follow: *follow,
                label: label.clone(),
                anchor: *anchor,
            };
        }
        if let Kind::ScrollButton {
            target,
            placement,
            label,
            anchor,
        } = self
        {
            return Kind::ScrollButton {
                target: m(*target),
                placement: *placement,
                label: label.clone(),
                anchor: *anchor,
            };
        }
        if let Kind::PresentationButton {
            targets,
            mode,
            label,
            anchor,
        } = self
        {
            return Kind::PresentationButton {
                targets: targets.iter().map(|&target| m(target)).collect(),
                mode: *mode,
                label: label.clone(),
                anchor: *anchor,
            };
        }
        if let Kind::LinkUrlButton { url, label, anchor } = self {
            return Kind::LinkUrlButton {
                url: url.clone(),
                label: label.clone(),
                anchor: *anchor,
            };
        }
        if let Kind::LinkPageButton {
            target_page,
            target_button_ordinal,
            label,
            anchor,
        } = self
        {
            return Kind::LinkPageButton {
                target_page: *target_page,
                target_button_ordinal: *target_button_ordinal,
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
            Kind::Locus(ref refs, samples) => {
                Kind::Locus(refs.iter().map(|id| m(*id)).collect(), samples)
            }
            Kind::GraphHorizontalUnit(a, step) => Kind::GraphHorizontalUnit(m(a), step),
            Kind::GraphVerticalUnit(a, step) => Kind::GraphVerticalUnit(m(a), step),
            Kind::GraphAxis(a, b, axis) => Kind::GraphAxis(m(a), m(b), axis),
            Kind::CircleAxis(circle, axis) => Kind::CircleAxis(m(circle), axis),
            Kind::DistanceAxis(origin, distance, axis) => {
                Kind::DistanceAxis(m(origin), m(distance), axis)
            }
            Kind::DistanceVerticalUnit(axis, step, independent) => {
                Kind::DistanceVerticalUnit(m(axis), step, independent)
            }
            Kind::CoordinateSystem(a, b) => Kind::CoordinateSystem(m(a), m(b)),
            Kind::SlopeMeasurement(a, b) => Kind::SlopeMeasurement(m(a), m(b)),
            Kind::FunctionDefinition(expr) => Kind::FunctionDefinition(expr),
            Kind::DerivativeFunction(source, expr) => Kind::DerivativeFunction(m(source), expr),
            Kind::FunctionPlot(a, b, min, max, samples) => {
                Kind::FunctionPlot(m(a), m(b), min, max, samples)
            }
            Kind::ParametricPlot(x, y, frame, min, max, samples, mode) => {
                Kind::ParametricPlot(m(x), m(y), m(frame), min, max, samples, mode)
            }
            Kind::PathLiteralPoint(axis, value) => Kind::PathLiteralPoint(m(axis), value),
            Kind::PathValuePoint(a, b) => Kind::PathValuePoint(m(a), m(b)),
            Kind::PlottedPoint(a, b, mode, stored) => Kind::PlottedPoint(m(a), m(b), mode, stored),
            Kind::SegmentLength(a) => Kind::SegmentLength(m(a)),
            Kind::PolygonPerimeter(a) => Kind::PolygonPerimeter(m(a)),
            Kind::CircleCircumference(a) => Kind::CircleCircumference(m(a)),
            Kind::CircleRadius(a) => Kind::CircleRadius(m(a)),
            Kind::CircleArea(a) => Kind::CircleArea(m(a)),
            Kind::PolygonArea(a) => Kind::PolygonArea(m(a)),
            Kind::ArcAngle(a) => Kind::ArcAngle(m(a)),
            Kind::ArcLength(a) => Kind::ArcLength(m(a)),
            Kind::AngleMeasurement(a, b, c) => Kind::AngleMeasurement(m(a), m(b), m(c)),
            Kind::PointDistance(a, b) => Kind::PointDistance(m(a), m(b)),
            Kind::PointRatio(a, b, c) => Kind::PointRatio(m(a), m(b), m(c)),
            Kind::PolarTranslatedPoint(a, angle, units, distance) => {
                Kind::PolarTranslatedPoint(m(a), angle, units, distance)
            }
            Kind::PolarTranslatedSegment(a, angle, units, distance) => {
                Kind::PolarTranslatedSegment(m(a), angle, units, distance)
            }
            Kind::PolarTranslatedCircle(a, angle, units, distance) => {
                Kind::PolarTranslatedCircle(m(a), angle, units, distance)
            }
            Kind::PolarTranslatedPolygon(a, angle, units, distance) => {
                Kind::PolarTranslatedPolygon(m(a), angle, units, distance)
            }
            Kind::FixedRectangularTranslatedPoint(
                source,
                x_units,
                x_distance,
                y_units,
                y_distance,
            ) => Kind::FixedRectangularTranslatedPoint(
                m(source),
                x_units,
                x_distance,
                y_units,
                y_distance,
            ),
            Kind::FixedRectangularTranslatedSegment(
                source,
                x_units,
                x_distance,
                y_units,
                y_distance,
            ) => Kind::FixedRectangularTranslatedSegment(
                m(source),
                x_units,
                x_distance,
                y_units,
                y_distance,
            ),
            Kind::FixedRectangularTranslatedCircle(
                source,
                x_units,
                x_distance,
                y_units,
                y_distance,
            ) => Kind::FixedRectangularTranslatedCircle(
                m(source),
                x_units,
                x_distance,
                y_units,
                y_distance,
            ),
            Kind::FixedRectangularTranslatedPolygon(source, xu, xd, yu, yd) => {
                Kind::FixedRectangularTranslatedPolygon(m(source), xu, xd, yu, yd)
            }
            Kind::MarkedDistanceTranslatedPoint(source, distance, angle) => {
                Kind::MarkedDistanceTranslatedPoint(m(source), m(distance), angle)
            }
            Kind::MarkedDistanceTranslatedSegment(source, distance, angle) => {
                Kind::MarkedDistanceTranslatedSegment(m(source), m(distance), angle)
            }
            Kind::MarkedDistanceTranslatedCircle(source, distance, angle) => {
                Kind::MarkedDistanceTranslatedCircle(m(source), m(distance), angle)
            }
            Kind::MarkedDistanceTranslatedPolygon(source, distance, angle) => {
                Kind::MarkedDistanceTranslatedPolygon(m(source), m(distance), angle)
            }
            Kind::MixedRectangularTranslatedPoint(source, distance, fixed_cm, fixed_horizontal) => {
                Kind::MixedRectangularTranslatedPoint(
                    m(source),
                    m(distance),
                    fixed_cm,
                    fixed_horizontal,
                )
            }
            Kind::MixedRectangularTranslatedPolygon(
                source,
                distance,
                fixed_cm,
                fixed_horizontal,
            ) => Kind::MixedRectangularTranslatedPolygon(
                m(source),
                m(distance),
                fixed_cm,
                fixed_horizontal,
            ),
            Kind::RectangularMarkedTranslatedPoint(source, horizontal, vertical) => {
                Kind::RectangularMarkedTranslatedPoint(m(source), m(horizontal), m(vertical))
            }
            Kind::RectangularMarkedTranslatedPolygon(source, horizontal, vertical) => {
                Kind::RectangularMarkedTranslatedPolygon(m(source), m(horizontal), m(vertical))
            }
            Kind::VectorTranslatedPoint(source, start, end) => {
                Kind::VectorTranslatedPoint(m(source), m(start), m(end))
            }
            Kind::VectorTranslatedSegment(source, start, end) => {
                Kind::VectorTranslatedSegment(m(source), m(start), m(end))
            }
            Kind::VectorTranslatedCircle(source, start, end) => {
                Kind::VectorTranslatedCircle(m(source), m(start), m(end))
            }
            Kind::VectorTranslatedPolygon(source, start, end) => {
                Kind::VectorTranslatedPolygon(m(source), m(start), m(end))
            }
            Kind::AngleRotatedPoint(source, center, first, vertex, last) => {
                Kind::AngleRotatedPoint(m(source), m(center), m(first), m(vertex), m(last))
            }
            Kind::AngleRotatedSegment(source, center, first, vertex, last) => {
                Kind::AngleRotatedSegment(m(source), m(center), m(first), m(vertex), m(last))
            }
            Kind::AngleRotatedCircle(source, center, first, vertex, last) => {
                Kind::AngleRotatedCircle(m(source), m(center), m(first), m(vertex), m(last))
            }
            Kind::AngleRotatedPolygon(source, center, first, vertex, last) => {
                Kind::AngleRotatedPolygon(m(source), m(center), m(first), m(vertex), m(last))
            }
            Kind::RatioScaledPoint(source, center, origin, base, target) => {
                Kind::RatioScaledPoint(m(source), m(center), m(origin), m(base), m(target))
            }
            Kind::RatioScaledCircle(source, center, origin, base, target) => {
                Kind::RatioScaledCircle(m(source), m(center), m(origin), m(base), m(target))
            }
            Kind::RatioScaledSegment(source, center, origin, base, target) => {
                Kind::RatioScaledSegment(m(source), m(center), m(origin), m(base), m(target))
            }
            Kind::RatioScaledPolygon(source, center, origin, base, target) => {
                Kind::RatioScaledPolygon(m(source), m(center), m(origin), m(base), m(target))
            }
            Kind::ReflectedPoint(a, b) => Kind::ReflectedPoint(m(a), m(b)),
            Kind::ReflectedCircle(a, b) => Kind::ReflectedCircle(m(a), m(b)),
            Kind::ReflectedSegment(a, b) => Kind::ReflectedSegment(m(a), m(b)),
            Kind::ReflectedPolygon(a, b) => Kind::ReflectedPolygon(m(a), m(b)),
            Kind::ScaledPoint(a, b, factor) => Kind::ScaledPoint(m(a), m(b), factor),
            Kind::ScaledCircle(a, b, factor) => Kind::ScaledCircle(m(a), m(b), factor),
            Kind::ScaledSegment(a, b, factor) => Kind::ScaledSegment(m(a), m(b), factor),
            Kind::ScaledPolygon(a, b, factor) => Kind::ScaledPolygon(m(a), m(b), factor),
            Kind::RotatedPoint(a, b, angle) => Kind::RotatedPoint(m(a), m(b), angle),
            Kind::RotatedSegment(a, b, angle) => Kind::RotatedSegment(m(a), m(b), angle),
            Kind::RotatedCircle(a, b, angle) => Kind::RotatedCircle(m(a), m(b), angle),
            Kind::RotatedPolygon(a, b, angle) => Kind::RotatedPolygon(m(a), m(b), angle),
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
            Kind::HideShowButton { .. }
            | Kind::AnimationButton { .. }
            | Kind::MovementButton { .. }
            | Kind::ScrollButton { .. }
            | Kind::PresentationButton { .. } => unreachable!(),
            Kind::LinkUrlButton { .. } => unreachable!(),
            Kind::LinkPageButton { .. } => unreachable!(),
            Kind::CoordinateMeasurement { .. } => unreachable!(),
            Kind::ValueEquation { .. } => unreachable!(),
        }
    }
    fn remap_checked(&self, ids: &[usize]) -> Result<Self, String> {
        let mut remapped = self.remap(ids);
        match (&mut remapped, self) {
            (Kind::ValueTable(_, group), Kind::ValueTable(_, original))
            | (Kind::ValueTablePair(_, _, group), Kind::ValueTablePair(_, _, original)) => {
                *group = original.remap_sources(ids)?;
            }
            _ => {}
        }
        Ok(remapped)
    }
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
    ValueTable(usize, TableGroup),
    ValueTablePair(usize, usize, TableGroup),
    PointOnSegment(usize, f64),
    PointOnCircle(usize, f64, f64),
    PointOnArc(usize, f64),
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
    MixedRectangularTranslatedPolygon(usize, usize, f64, bool),
    RectangularMarkedTranslatedPoint(usize, usize, usize),
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
    ScaledPolygon(usize, usize, f64),
    RotatedPoint(usize, usize, f64),
    RotatedSegment(usize, usize, f64),
    RotatedCircle(usize, usize, f64),
    RotatedPolygon(usize, usize, f64),
    Midpoint(usize, usize),
    /// Native code 35. The reference list records the target, the driver's
    /// path, the driver, and the construction chain leading to the target.
    Locus(Vec<usize>, u32),
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
    pub text_style_override: Option<FontState>,
    pub rich_text_override: Option<text_object::RichTextTemplate>,
    pub function_anchor: Option<(i16, i16)>,
    pub function_name: Option<String>,
}
#[derive(Clone, Debug)]
pub struct Clipboard {
    hide_on_paste: BTreeSet<usize>,
    rename_on_paste: Option<usize>,
    objects: Vec<(
        usize,
        Kind,
        bool,
        bool,
        Option<usize>,
        Option<VisualStyle>,
        Option<FontState>,
        Option<LabelAux>,
        Option<text_object::RichTextTemplate>,
        Option<(GridForm, bool)>,
        Option<(i16, i16)>,
        Option<String>,
    )>,
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
fn is_multi_page_addition_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Free(_)
            | Kind::PointOnSegment(_, _)
            | Kind::PointOnCircle(_, _, _)
            | Kind::PointOnArc(_, _)
            | Kind::Segment(_, _)
            | Kind::Circle(_, _)
            | Kind::Line(_, _)
            | Kind::Ray(_, _)
            | Kind::Midpoint(_, _)
            | Kind::CircleBySegment(_, _)
            | Kind::Parallel(_, _)
            | Kind::Perpendicular(_, _)
            | Kind::Intersection(_, _)
            | Kind::LineCircleIntersection(_, _, _)
            | Kind::CircleCircleIntersection(_, _, _)
            | Kind::LinkPageButton { .. }
            | Kind::LinkUrlButton { .. }
    )
}
fn polygon_subtype(vertex_count: usize) -> Option<u16> {
    49u16.checked_add(u16::try_from(vertex_count).ok()?)
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
        16 => [
            16, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x51, 0x5c, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        21 => [
            21, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x39, 0x20, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        17 | 18 | 19 | 20 | 24 => [
            kind as u8, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x51, 0x5c,
            0xc1, 0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        30 => [
            30, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x51, 0x5c, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        32 => [
            32, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x51, 0x5c, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        27 => [
            27, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0xdf, 0x3f, 0xc1,
            0xbd, 0xf0, 0xff, 0, 0, 0, 0,
        ],
        28 => [
            28, 0, 0, 0, 0, 0, 1, 0, 4, 0, 0x21, 2, 0xff, 0, 0, 0, 0xff, 0xff, 0x51, 0x5c, 0xc1,
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
            | Kind::PolarTranslatedSegment(_, _, _, _)
            | Kind::FixedRectangularTranslatedSegment(_, _, _, _, _)
            | Kind::VectorTranslatedSegment(_, _, _)
            | Kind::MarkedDistanceTranslatedSegment(_, _, _)
            | Kind::AngleRotatedSegment(_, _, _, _, _)
            | Kind::RotatedSegment(_, _, _)
            | Kind::ScaledSegment(_, _, _)
            | Kind::RatioScaledSegment(_, _, _, _, _)
            | Kind::ReflectedSegment(_, _)
            | Kind::Line(_, _)
            | Kind::Ray(_, _)
            | Kind::Bisector(_, _, _)
            | Kind::Parallel(_, _)
            | Kind::Perpendicular(_, _)
            | Kind::GraphAxis(_, _, _)
            | Kind::CircleAxis(_, _)
            | Kind::DistanceAxis(_, _, _)
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
            | Kind::DistanceVerticalUnit(_, _, _)
            | Kind::PathLiteralPoint(_, _)
            | Kind::PathValuePoint(_, _)
            | Kind::PlottedPoint(_, _, _, _)
            | Kind::PolarTranslatedPoint(_, _, _, _)
            | Kind::FixedRectangularTranslatedPoint(_, _, _, _, _)
            | Kind::MarkedDistanceTranslatedPoint(_, _, _)
            | Kind::MixedRectangularTranslatedPoint(_, _, _, _)
            | Kind::RectangularMarkedTranslatedPoint(_, _, _)
            | Kind::VectorTranslatedPoint(_, _, _)
            | Kind::ReflectedPoint(_, _)
            | Kind::ScaledPoint(_, _, _)
            | Kind::RotatedPoint(_, _, _)
            | Kind::AngleRotatedPoint(_, _, _, _, _)
            | Kind::RatioScaledPoint(_, _, _, _, _)
            | Kind::Midpoint(_, _)
            | Kind::Intersection(_, _)
            | Kind::LineCircleIntersection(_, _, _)
            | Kind::CircleCircleIntersection(_, _, _)
    )
}

fn expected_locus_refs(objects: &[Object], driver: usize, target: usize) -> Option<Vec<usize>> {
    fn ancestry(
        objects: &[Object],
        driver: usize,
        current: usize,
        page: usize,
    ) -> Option<BTreeSet<usize>> {
        if current == driver {
            return Some(BTreeSet::from([driver]));
        }
        if current <= driver {
            return None;
        }
        let object = objects.get(current.checked_sub(1)?)?;
        if object.deleted || object.page != page {
            return None;
        }
        let parents = if let Kind::Midpoint(a, b) = object.kind {
            let host = object.midpoint_host?;
            if !matches!(objects.get(host.checked_sub(1)?)?.kind, Kind::Segment(x, y) if (x == a && y == b) || (x == b && y == a))
            {
                return None;
            }
            vec![host]
        } else if is_locus_straight_kind(&object.kind)
            || is_point_kind(&object.kind)
            || is_circle_kind(&object.kind)
            || is_arc_kind(&object.kind)
            || is_locus_polygon_kind(&object.kind)
        {
            object.kind.dependencies()
        } else {
            return None;
        };
        let mut descendants = BTreeSet::new();
        for parent in parents {
            if parent >= current {
                return None;
            }
            if let Some(found) = ancestry(objects, driver, parent, page) {
                descendants.extend(found);
            }
        }
        if descendants.is_empty() {
            return None;
        }
        descendants.insert(current);
        Some(descendants)
    }
    if driver == 0 || target <= driver {
        return None;
    }
    let moving = objects.get(driver - 1)?;
    let traced = objects.get(target - 1)?;
    if moving.deleted
        || traced.deleted
        || moving.page != traced.page
        || !(is_point_kind(&traced.kind)
            || is_locus_straight_kind(&traced.kind)
            || is_circle_kind(&traced.kind)
            || is_arc_kind(&traced.kind)
            || is_locus_polygon_kind(&traced.kind))
    {
        return None;
    }
    let path = match moving.kind {
        Kind::PointOnSegment(path, _)
            if matches!(objects.get(path.checked_sub(1)?)?.kind, Kind::Segment(_, _)) =>
        {
            path
        }
        Kind::PointOnCircle(path, _, _)
            if is_circle_kind(&objects.get(path.checked_sub(1)?)?.kind) =>
        {
            path
        }
        Kind::PointOnArc(path, _)
            if matches!(
                objects.get(path.checked_sub(1)?)?.kind,
                Kind::ThreePointArc(_, _, _) | Kind::ArcOnCircle(_, _, _)
            ) =>
        {
            path
        }
        _ => return None,
    };
    let ancestors = ancestry(objects, driver, target, moving.page)?;
    let mut refs = vec![target, path];
    refs.extend(ancestors);
    Some(refs)
}

fn is_distance_value_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::SegmentLength(_)
            | Kind::PointDistance(_, _)
            | Kind::PolygonPerimeter(_)
            | Kind::CircleCircumference(_)
            | Kind::CircleRadius(_)
            | Kind::ArcLength(_)
    )
}

fn is_circle_kind(kind: &Kind) -> bool {
    matches!(
        kind,
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
            | Kind::RatioScaledCircle(_, _, _, _, _)
    )
}

fn is_arc_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::ThreePointArc(_, _, _) | Kind::ArcOnCircle(_, _, _)
    )
}

fn is_locus_polygon_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Polygon(_)
            | Kind::VectorTranslatedPolygon(_, _, _)
            | Kind::PolarTranslatedPolygon(_, _, _, _)
            | Kind::FixedRectangularTranslatedPolygon(_, _, _, _, _)
    )
}

fn is_locus_straight_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Segment(_, _)
            | Kind::Ray(_, _)
            | Kind::Line(_, _)
            | Kind::Parallel(_, _)
            | Kind::Perpendicular(_, _)
            | Kind::Bisector(_, _, _)
    )
}

fn is_defining_distance_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Segment(_, _) | Kind::SegmentLength(_) | Kind::PointDistance(_, _)
    )
}

fn is_plot_path_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Segment(_, _)
            | Kind::Ray(_, _)
            | Kind::Line(_, _)
            | Kind::GraphAxis(_, _, 0 | 1 | 0x1_0001)
            | Kind::CircleAxis(_, 0 | 1)
            | Kind::DistanceAxis(_, _, 0 | 1)
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
        | Kind::FixedRectangularTranslatedPoint(_, _, _, _, _)
        | Kind::MarkedDistanceTranslatedPoint(_, _, _)
        | Kind::MixedRectangularTranslatedPoint(_, _, _, _)
        | Kind::RectangularMarkedTranslatedPoint(_, _, _)
        | Kind::VectorTranslatedPoint(_, _, _)
        | Kind::ReflectedPoint(_, _)
        | Kind::ScaledPoint(_, _, _)
        | Kind::RotatedPoint(_, _, _)
        | Kind::AngleRotatedPoint(_, _, _, _, _)
        | Kind::RatioScaledPoint(_, _, _, _, _)
        | Kind::Midpoint(_, _)
        | Kind::Intersection(_, _)
        | Kind::LineCircleIntersection(_, _, _)
        | Kind::CircleCircleIntersection(_, _, _) => Some(VisualStyle::Point {
            shape: PointStyle::Medium,
            color: red,
        }),
        Kind::Segment(_, _)
        | Kind::PolarTranslatedSegment(_, _, _, _)
        | Kind::FixedRectangularTranslatedSegment(_, _, _, _, _)
        | Kind::VectorTranslatedSegment(_, _, _)
        | Kind::MarkedDistanceTranslatedSegment(_, _, _)
        | Kind::AngleRotatedSegment(_, _, _, _, _)
        | Kind::RotatedSegment(_, _, _)
        | Kind::ScaledSegment(_, _, _)
        | Kind::RatioScaledSegment(_, _, _, _, _)
        | Kind::ReflectedSegment(_, _)
        | Kind::Line(_, _)
        | Kind::Ray(_, _)
        | Kind::Parallel(_, _)
        | Kind::Perpendicular(_, _)
        | Kind::Bisector(_, _, _) => Some(VisualStyle::Stroke { line, color: navy }),
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
        | Kind::RatioScaledCircle(_, _, _, _, _)
        | Kind::ThreePointArc(_, _, _)
        | Kind::ArcOnCircle(_, _, _) => Some(VisualStyle::Stroke { line, color: green }),
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
        | Kind::RatioScaledPolygon(_, _, _, _, _) => Some(VisualStyle::Polygon {
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

fn marked_rotation_theta(first: Point, vertex: Point, last: Point) -> Option<f64> {
    let (ax, ay) = (first.x - vertex.x, first.y - vertex.y);
    let (cx, cy) = (last.x - vertex.x, last.y - vertex.y);
    if ax.hypot(ay) == 0. || cx.hypot(cy) == 0. {
        return None;
    }
    Some((-cy).atan2(cx) - (-ay).atan2(ax))
}

fn rotate_point_about(source: Point, center: Point, theta: f64) -> Option<Point> {
    let dx = source.x - center.x;
    let dy = source.y - center.y;
    let rotated = Point {
        x: center.x + dx * theta.cos() + dy * theta.sin(),
        y: center.y - dx * theta.sin() + dy * theta.cos(),
    };
    (rotated.x.is_finite() && rotated.y.is_finite()).then_some(rotated)
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
    pub angle_unit_radians: bool,
    pub marked_coordinate_system: Vec<Option<usize>>,
    pub frame_styles: BTreeMap<usize, (GridForm, bool)>,
    original: Vec<u8>,
    run_font_families: Vec<String>,
    source_count: usize,
    last_moved_object: Option<usize>,
    safe_structure: bool,
    image_read_only: bool,
    pub undo: Vec<(
        Vec<Object>,
        Vec<bool>,
        Vec<bool>,
        Vec<GridForm>,
        bool,
        Vec<Option<usize>>,
        BTreeMap<usize, (GridForm, bool)>,
        Vec<String>,
        Option<usize>,
    )>,
    pub redo: Vec<(
        Vec<Object>,
        Vec<bool>,
        Vec<bool>,
        Vec<GridForm>,
        bool,
        Vec<Option<usize>>,
        BTreeMap<usize, (GridForm, bool)>,
        Vec<String>,
        Option<usize>,
    )>,
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
        let angle_unit_radians = rs
            .iter()
            .find(|record| record.kind == 0x273a && record.data.len() == 2)
            .is_some_and(|record| b[record.data.clone()] == [0, 0]);
        let safe_outside: BTreeSet<u32> = baseline_records
            .iter()
            .map(|record| record.kind)
            .chain([0x2328, 0x232d])
            .collect();
        let text_style_count = rs.iter().filter(|record| record.kind == 0x04b3).count();
        let font_family_count = rs.iter().filter(|record| record.kind == 0x04b1).count();
        let mut objects: Vec<Object> = Vec::new();
        let total_pages = rs.iter().filter(|record| record.kind == 0x3e8).count();
        let mut i = 0;
        let mut safe = true;
        let mut snap_points = Vec::new();
        let mut grid_dotted = Vec::new();
        let mut grid_form = Vec::new();
        let mut frame_styles = BTreeMap::new();
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
            let mut locus_range: Option<Vec<u8>> = None;
            let mut label: Option<LabelAux> = None;
            let labeled_polygon = h[6..8] == [4, 0]
                && matches!(
                    code,
                    8 | 16 | 17 | 18 | 19 | 20 | 21 | 24 | 27 | 28 | 30 | 32 | 34
                );
            let mut rich_text: Option<Vec<u8>> = None;
            let mut text_box: Option<Vec<u8>> = None;
            let mut text_style = false;
            i += 1;
            while i < rs.len() && rs[i].kind != 0x7d7 {
                let r = &rs[i];
                let d = &b[r.data.clone()];
                if matches!(code, 52 | 54 | 55 | 56 | 57 | 58 | 59 | 60 | 61 | 87) {
                    graph_records.push((r.kind, d.to_vec()));
                }
                if matches!(code, 0 | 78) && r.kind == 0x0907 {
                    function_expression = Some(d.to_vec());
                }
                if matches!(code, 72 | 126 | 127) && r.kind == 0x0902 {
                    function_range = Some(d.to_vec());
                }
                if code == 35 && r.kind == 0x0902 {
                    locus_range = Some(d.to_vec());
                }
                match r.kind {
                    0x07d5 if code == 0 || labeled_polygon => match LabelAux::parse(d) {
                        Ok(value) if code == 0 => label = Some(value),
                        Ok(_) => {}
                        Err(_) => safe = false,
                    },
                    0x08fc if code == 0 => rich_text = Some(d.to_vec()),
                    0x0898 if code == 0 => text_box = Some(d.to_vec()),
                    0x07d8 if code == 0 || labeled_polygon => {
                        text_style = d.len() == 2
                            && (u16::from_le_bytes([d[0], d[1]]) as usize) < text_style_count;
                        if labeled_polygon && !text_style {
                            safe = false;
                        }
                    }
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
                                    matches!(i, 12..=15 | 22 | 23 | 30 | 31) || actual == expected
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
                    0x7d3
                        if matches!(
                            code,
                            15 | 17 | 18 | 19 | 21 | 24 | 27 | 30 | 54 | 55 | 56 | 57
                        ) =>
                    {
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
                    && !(code == 0
                        && matches!(r.kind, 0x07d5 | 0x07d8 | 0x08fc | 0x0898 | 0x0906 | 0x08a2))
                    && !(labeled_polygon && matches!(r.kind, 0x07d5 | 0x07d8))
                    && !(matches!(code, 36 | 37 | 39 | 40 | 41 | 42 | 43 | 44 | 46 | 47)
                        && matches!(r.kind, 0x0903 | 0x07d8))
                    && !(matches!(code, 52 | 54 | 55 | 56 | 57 | 58 | 59 | 60 | 61 | 87)
                        && matches!(r.kind, 0x0905 | 0x0904 | 0x0903 | 0x07d5 | 0x07d8))
                    && !(matches!(code, 49 | 65 | 66 | 86)
                        && matches!(r.kind, 0x08fc | 0x0903 | 0x07d8))
                    && !(matches!(code, 50 | 51 | 94) && matches!(r.kind, 0x08fc | 0x0903 | 0x07d8))
                    && !(code == 62 && matches!(r.kind, 0x0906 | 0x07d5 | 0x07d8))
                    && !(code == 0 && matches!(r.kind, 0x0907 | 0x08a3))
                    && !(code == 78 && matches!(r.kind, 0x0907 | 0x07d8))
                    && !(code == 91 && matches!(r.kind, 0x090d | 0x07d8))
                    && !(is_image && matches!(r.kind, 0x090c | 0x08a8 | 0x1f44))
                    && !(matches!(code, 72 | 126 | 127) && r.kind == 0x0902)
                    && !(code == 35 && r.kind == 0x0902)
                {
                    safe = false;
                }
                i += 1;
            }
            if i == rs.len() {
                return Err("Unterminated object group".into());
            }
            let action_bytes = (code == 62).then_some(&b[group_start..rs[i].data.end]);
            let action_group =
                action_bytes.and_then(|bytes| HideShowButtonGroup::parse(bytes).ok());
            let animation_group =
                action_bytes.and_then(|bytes| AnimationButtonGroup::parse(bytes).ok());
            let movement_group =
                action_bytes.and_then(|bytes| MovementButtonGroup::parse(bytes).ok());
            let scroll_group = action_bytes.and_then(|bytes| ScrollButtonGroup::parse(bytes).ok());
            let presentation_group =
                action_bytes.and_then(|bytes| PresentationButtonGroup::parse(bytes).ok());
            let link_url_group = (code == 0)
                .then_some(&b[group_start..rs[i].data.end])
                .and_then(|bytes| LinkUrlButtonGroup::parse(bytes).ok());
            let link_page_group = (code == 0)
                .then_some(&b[group_start..rs[i].data.end])
                .and_then(|bytes| LinkPageButtonGroup::parse(bytes).ok());
            if code == 0
                && rs.iter().any(|record| {
                    record.start >= group_start
                        && record.start < rs[i].start
                        && record.kind == 0x0906
                })
                && link_url_group.is_none()
                && link_page_group.is_none()
            {
                safe = false;
            }
            if code == 62
                && action_group.is_none()
                && animation_group.is_none()
                && movement_group.is_none()
                && scroll_group.is_none()
                && presentation_group.is_none()
            {
                safe = false;
            }
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
                    GraphObject::CircleAxis { circle, axis }
                        if objects
                            .get(circle - 1)
                            .is_some_and(|o| is_circle_kind(&o.kind)) =>
                    {
                        Kind::CircleAxis(circle, axis)
                    }
                    GraphObject::DistanceAxis {
                        origin,
                        distance,
                        axis,
                    } if objects
                        .get(origin - 1)
                        .is_some_and(|o| is_point_kind(&o.kind))
                        && objects
                            .get(distance - 1)
                            .is_some_and(|o| is_defining_distance_kind(&o.kind)) =>
                    {
                        Kind::DistanceAxis(origin, distance, axis)
                    }
                    GraphObject::DistanceVerticalUnit {
                        x_axis,
                        step,
                        independent,
                    } if objects
                        .get(x_axis - 1)
                        .is_some_and(|o| matches!(o.kind, Kind::DistanceAxis(_, _, 1))) =>
                    {
                        Kind::DistanceVerticalUnit(x_axis, step, independent)
                    }
                    GraphObject::CoordinateSystem {
                        x_axis,
                        y_axis,
                        form,
                        dotted,
                    } if (objects.get(x_axis - 1).is_some_and(|o| {
                        matches!(o.kind, Kind::GraphAxis(_, _, 1 | 0x1_0001))
                    }) && objects
                        .get(y_axis - 1)
                        .is_some_and(|o| matches!(o.kind, Kind::GraphAxis(_, _, 0))))
                        || matches!(
                            (objects.get(x_axis - 1), objects.get(y_axis - 1)),
                            (Some(Object { kind: Kind::CircleAxis(a, 1), .. }),
                             Some(Object { kind: Kind::CircleAxis(b, 0), .. })) if a == b
                        )
                        || matches!(
                            (objects.get(x_axis - 1), objects.get(y_axis - 1)),
                            (Some(Object { kind: Kind::DistanceAxis(a, _, 1), .. }),
                             Some(Object { kind: Kind::DistanceAxis(b, _, 0), .. })) if a == b
                        )
                        || matches!(
                            (objects.get(x_axis - 1), objects.get(y_axis - 1)),
                            (Some(Object { kind: Kind::DistanceAxis(origin, _, 1), .. }),
                             Some(Object { kind: Kind::GraphAxis(y_origin, unit, 0), .. }))
                                if origin == y_origin && objects.get(unit - 1)
                                    .is_some_and(|o| matches!(o.kind, Kind::DistanceVerticalUnit(host, _, _) if host == x_axis))
                        ) =>
                    {
                        grid_dotted[page] = dotted;
                        grid_form[page] = form;
                        frame_styles.insert(objects.len() + 1, (form, dotted));
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
                    (35, None, _)
                        if refs.first().and_then(|id| objects.get(id.saturating_sub(1))).is_some_and(|o| {
                            if is_locus_straight_kind(&o.kind) || is_circle_kind(&o.kind) || is_arc_kind(&o.kind) || is_locus_polygon_kind(&o.kind) {
                                h[6..10] == [6, 0, 43, 0]
                            } else {
                                h[6..10] == [6, 0, 42, 0]
                            }
                        })
                            && refs.first().zip(refs.get(2)).and_then(|(&target, &driver)| {
                                expected_locus_refs(&objects, driver, target)
                            }).as_ref() == Some(&refs)
                            && locus_range.as_ref().is_some_and(|data| {
                                data.len() == 40
                                    && f64_at(data, 0) == 0.0
                                    && (2..=10_000).contains(&u32_at(data, 16))
                                    && refs.get(2).and_then(|id| objects.get(id.saturating_sub(1))).is_some_and(|o| match o.kind {
                                        Kind::PointOnSegment(_, _) | Kind::PointOnArc(_, _) => f64_at(data, 8) == 1.0 && data[20..24] == [4, 0, 1, 0],
                                        Kind::PointOnCircle(_, _, _) => (f64_at(data, 8) - std::f64::consts::TAU).abs() < 1e-12 && data[20..24] == [4, 0, 0, 0],
                                        _ => false,
                                    })
                                    && f64_at(data, 24).is_finite()
                                    && refs.first().and_then(|id| objects.get(id.saturating_sub(1))).is_some_and(|o| {
                                        data[32..] == if is_locus_straight_kind(&o.kind) || is_circle_kind(&o.kind) || is_arc_kind(&o.kind) || is_locus_polygon_kind(&o.kind) {
                                            [0, 0, 0, 0, 0, 0, 0, 0]
                                        } else {
                                            [1, 0, 0, 0, 0, 0, 0, 0]
                                        }
                                    })
                            }) => Kind::Locus(refs.clone(), u32_at(locus_range.as_ref().unwrap(), 16)),
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
                        && objects.get(source - 1).is_some_and(|o| o.page == page)
                        && objects.get(frame - 1).is_some_and(|o| o.page == page && matches!(o.kind, Kind::CoordinateSystem(_, _))) => {
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
                    (91, None, [source]) if source.checked_sub(1).and_then(|index| objects.get(index)).is_some_and(|o| matches!(o.kind, Kind::Parameter(_))) => {
                        match TableGroup::parse(&b[group_start..rs[i].data.end]) {
                            Ok(table) if source.checked_sub(page_base) == Some(table.source_id() as usize)
                                && source.checked_sub(1).and_then(|index| objects.get(index)).is_some_and(|o| match &o.kind {
                                    Kind::Parameter(parameter) => match table.named_label_value() {
                                        Some((name, _)) => name == parameter.name(),
                                        None => parameter.name() == "t[1]",
                                    },
                                    _ => false,
                                }) => Kind::ValueTable(*source, table),
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (91, None, [first, second]) if objects.get(first - 1).is_some_and(|o| {
                        o.page == page && matches!(&o.kind, Kind::Parameter(group) if group.name() == "t[2]")
                    }) && objects.get(second - 1).is_some_and(|o| {
                        o.page == page && matches!(&o.kind, Kind::Parameter(group) if group.name() == "t[1]")
                    }) => {
                        match TableGroup::parse_pair(&b[group_start..rs[i].data.end]) {
                            Ok(table) if table.pair_source_ids() == Some([
                                (*first - page_base) as u32,
                                (*second - page_base) as u32,
                            ]) => Kind::ValueTablePair(*first, *second, table),
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (72, None, [definition, frame]) if function_range.is_some()
                        && matches!(objects.get(definition - 1).map(|o| &o.kind), Some(Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _)))
                        && matches!(objects.get(frame - 1).map(|o| &o.kind), Some(Kind::CoordinateSystem(_, _))) => {
                        match function_records::decode_plot_range(function_range.as_ref().unwrap()) {
                            Ok((min,max,samples)) => Kind::FunctionPlot(*definition,*frame,min,max,samples),
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (78, None, [source]) if *source > 0 && function_expression.is_some()
                        && matches!(objects.get(source - 1).map(|o| &o.kind), Some(Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _))) => {
                        match function_records::decode_simple_expression(function_expression.as_ref().unwrap()) {
                            Ok(expression) => Kind::DerivativeFunction(*source, expression),
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (126 | 127, None, [x, y, frame]) if function_range.is_some()
                        && *x > 0 && *y > 0 && *frame > 0 && x != y
                        && matches!(objects.get(x - 1).map(|o| &o.kind), Some(Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _)))
                        && matches!(objects.get(y - 1).map(|o| &o.kind), Some(Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _)))
                        && matches!(objects.get(frame - 1).map(|o| &o.kind), Some(Kind::CoordinateSystem(_, _))) => {
                        match function_records::decode_parametric_range(function_range.as_ref().unwrap(), code == 127) {
                            Ok((min,max,samples)) => Kind::ParametricPlot(*x,*y,*frame,min,max,samples,if code == 127 { PlotPointsMode::Polar } else { PlotPointsMode::Cartesian }),
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (62, _, targets) if !targets.is_empty() && action_group.as_ref().is_some_and(|group| {
                        group.target_ids() == targets.iter().map(|target| (*target - page_base) as u32).collect::<Vec<_>>()
                    }) => {
                        let group = action_group.as_ref().unwrap();
                        match group.label_utf8() {
                            Ok(label) => Kind::HideShowButton {
                                targets: targets.to_vec(),
                                action: group.action(),
                                label: label.to_owned(),
                                anchor: group.screen_anchor(),
                            },
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (62, _, targets) if !targets.is_empty() && animation_group.as_ref().is_some_and(|group| {
                        group.target_ids() == targets.iter().map(|target| (*target - page_base) as u32).collect::<Vec<_>>()
                    }) => {
                        let group = animation_group.as_ref().unwrap();
                        match group.label_utf8() {
                            Ok(label) => Kind::AnimationButton {
                                targets: targets.to_vec(),
                                label: label.to_owned(),
                                anchor: group.screen_anchor(),
                                settings: group.settings_list().unwrap(),
                            },
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (62, _, [source, destination]) if movement_group.as_ref().is_some_and(|group| {
                        group.target_ids() == [(*source - page_base) as u32, (*destination - page_base) as u32]
                    }) => {
                        let group = movement_group.as_ref().unwrap();
                        match group.label_utf8() {
                            Ok(label) => Kind::MovementButton {
                                source: *source,
                                destination: *destination,
                                speed: group.speed(),
                                follow: group.follows_destination(),
                                label: label.to_owned(),
                                anchor: group.screen_anchor(),
                            },
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (62, _, [target]) if scroll_group.as_ref().is_some_and(|group| {
                        group.target_id() == (*target - page_base) as u32
                    }) => {
                        let group = scroll_group.as_ref().unwrap();
                        match group.label_utf8() {
                            Ok(label) => Kind::ScrollButton {
                                target: *target,
                                placement: group.placement(),
                                label: label.to_owned(),
                                anchor: group.screen_anchor(),
                            },
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (62, _, targets) if !targets.is_empty() && presentation_group.as_ref().is_some_and(|group| {
                        group.target_ids() == targets.iter().map(|target| (*target - page_base) as u32).collect::<Vec<_>>()
                    }) => {
                        let group = presentation_group.as_ref().unwrap();
                        match group.label_utf8() {
                            Ok(label) => Kind::PresentationButton {
                                targets: targets.to_vec(),
                                mode: group.mode(),
                                label: label.to_owned(),
                                anchor: group.screen_anchor(),
                            },
                            Err(_) => Kind::Unsupported(code),
                        }
                    }
                    (0, None, []) if link_url_group.is_some() => {
                        let group = link_url_group.as_ref().unwrap();
                        match (group.url_utf8(), group.label_utf8()) {
                            (Ok(url), Ok(label)) => Kind::LinkUrlButton {
                                url: url.to_owned(),
                                label: label.to_owned(),
                                anchor: group.screen_anchor(),
                            },
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (0, None, []) if link_page_group.is_some() => {
                        let group = link_page_group.as_ref().unwrap();
                        match group.label_utf8() {
                            Ok(label) if group.target_page() < total_pages.max(1) => {
                                Kind::LinkPageButton {
                                    target_page: group.target_page(),
                                    target_button_ordinal: group.target_button_ordinal(),
                                    label: label.to_owned(),
                                    anchor: group.screen_anchor(),
                                }
                            }
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (0, Some(p), _) if rich_text.is_none() && text_box.is_none() => Kind::Free(p),
                    (0, None, []) if rich_text.is_some() && text_box.is_some()
                        && text_style && label.is_none() => {
                        match text_object::decode_rich_text(rich_text.as_ref().unwrap(), text_box.as_ref().unwrap()) {
                            Ok((text, rect, runs)) if runs.iter().flatten().all(|run| run.style.family_index.is_none_or(|index| usize::from(index) < font_family_count)) => Kind::PlainText { text, rect },
                            _ => Kind::Unsupported(code),
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
                            && objects.get(host.saturating_sub(1)).is_some_and(|o| o.kind.is_polygon()) =>
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
                    (42, _, [host])
                        if measurement_format && measurement_style
                            && objects.get(host.saturating_sub(1)).is_some_and(|o| o.kind.is_polygon()) =>
                    {
                        Kind::PolygonArea(*host)
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
                                Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _) | Kind::ReflectedCircle(_, _) | Kind::ScaledCircle(_, _, _) | Kind::RatioScaledCircle(_, _, _, _, _)),
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
                                && (f64_at(payload, 4) + f64_at(payload, 20).to_radians().sin()).abs() < 1e-6
                                && (f64_at(payload, 12) - f64_at(payload, 20).to_radians().cos()).abs() < 1e-6
                                && *origin > 0
                                && objects.get(origin - 1).is_some_and(|o| is_point_kind(&o.kind) || matches!(o.kind, Kind::Segment(_, _) | Kind::Circle(_, _) | Kind::CircleBySegment(_, _) | Kind::Polygon(_))) =>
                        {
                            if h[8..10] == [14, 0] && matches!(objects[origin - 1].kind, Kind::Segment(_, _)) {
                                Kind::PolarTranslatedSegment(*origin, f64_at(payload, 20), f64_at(payload, 32), f64_at(payload, 40))
                            } else if h[8..10] == [28, 0] && matches!(objects[origin - 1].kind, Kind::Circle(_, _) | Kind::CircleBySegment(_, _)) {
                                Kind::PolarTranslatedCircle(*origin, f64_at(payload, 20), f64_at(payload, 32), f64_at(payload, 40))
                            } else if h[8..10] == [4, 0] && is_point_kind(&objects[origin - 1].kind) {
                                Kind::PolarTranslatedPoint(*origin, f64_at(payload, 20), f64_at(payload, 32), f64_at(payload, 40))
                            } else if matches!(&objects[origin - 1].kind, Kind::Polygon(vertices) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]])))
                                && h[6..8] == [4, 0] && h[20..24] == [1, 0, 0, 0] {
                                let distance = f64_at(payload, 40);
                                // Sketchpad's native polygon-locus save replaces the unit
                                // scale with the translated pixel distance in this cache form.
                                let units = if payload[52..60] == [0x64, 0xe4, 0x83, 0x01, 0, 0, 0, 0]
                                    && distance != 0. {
                                    f64_at(payload, 32) / distance
                                } else {
                                    f64_at(payload, 32)
                                };
                                Kind::PolarTranslatedPolygon(*origin, f64_at(payload, 20), units, distance)
                            } else {
                                Kind::Unsupported(code)
                            }
                        }
                        _ => Kind::Unsupported(code),
                    },
                    (17, _, [source]) => match &constraint_payload {
                        Some((payload, _))
                            if payload.len() == 52
                                && payload[..4] == [17, 0, 0, 0]
                                && u32_at(payload, 20) == 1
                                && u32_at(payload, 40) == 1
                                && [4, 12, 24, 32].iter().all(|&offset| f64_at(payload, offset).is_finite())
                                && (f64_at(payload, 4) - f64_at(payload, 12) * 37.79527559055118).abs() < 1e-5
                                && (f64_at(payload, 24) - f64_at(payload, 32) * 37.79527559055118).abs() < 1e-5
                                && *source > 0
                                && objects.get(source - 1).is_some_and(|o| is_point_kind(&o.kind) || matches!(o.kind, Kind::Segment(_, _) | Kind::Circle(_, _) | Kind::CircleBySegment(_, _) | Kind::Polygon(_))) =>
                        {
                            if h[8..10] == [14, 0] && matches!(objects[source - 1].kind, Kind::Segment(_, _)) {
                                Kind::FixedRectangularTranslatedSegment(*source, 37.79527559055118, f64_at(payload, 12), 37.79527559055118, f64_at(payload, 32))
                            } else if h[8..10] == [28, 0] && matches!(objects[source - 1].kind, Kind::Circle(_, _) | Kind::CircleBySegment(_, _)) {
                                Kind::FixedRectangularTranslatedCircle(*source, 37.79527559055118, f64_at(payload, 12), 37.79527559055118, f64_at(payload, 32))
                            } else if h[8..10] == [4, 0] && is_point_kind(&objects[source - 1].kind) {
                                Kind::FixedRectangularTranslatedPoint(*source, 37.79527559055118, f64_at(payload, 12), 37.79527559055118, f64_at(payload, 32))
                            } else if matches!(&objects[source - 1].kind, Kind::Polygon(vertices) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]])))
                                && h[6..8] == [4, 0] && h[20..24] == [1, 0, 0, 0] {
                                Kind::FixedRectangularTranslatedPolygon(*source, 37.79527559055118, f64_at(payload, 12), 37.79527559055118, f64_at(payload, 32))
                            } else {
                                Kind::Unsupported(code)
                            }
                        }
                        _ => Kind::Unsupported(code),
                    },
                    (18 | 19, _, [source, distance])
                        if objects.get(source.saturating_sub(1)).is_some_and(|o| is_point_kind(&o.kind)) => match &constraint_payload {
                        Some((payload, _))
                            if payload.len() == 28
                                && payload[..4] == [code as u8, 0, 0, 0]
                                && u32_at(payload, 20) == 1
                                && payload[24..28] == [0, 0, 0, 0]
                                && f64_at(payload, 4).is_finite()
                                && f64_at(payload, 12).is_finite()
                                && (f64_at(payload, 4) - f64_at(payload, 12) * 37.79527559055118).abs() < 1e-5
                                && *source > 0 && *distance > 0
                                && objects.get(source - 1).is_some_and(|o| is_point_kind(&o.kind))
                                && objects.get(distance - 1).is_some_and(|o| is_distance_value_kind(&o.kind)) =>
                        {
                            Kind::MixedRectangularTranslatedPoint(*source, *distance, f64_at(payload, 12), code == 18)
                        }
                        _ => Kind::Unsupported(code),
                    },
                    (18 | 19, _, [source, distance])
                        if objects.get(source.saturating_sub(1)).is_some_and(|o| matches!(&o.kind, Kind::Polygon(vertices) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]]))))
                            && objects.get(distance.saturating_sub(1)).is_some_and(|o| is_distance_value_kind(&o.kind))
                            && h[6..8] == [4, 0] && h[20..24] == [1, 0, 0, 0] => match &constraint_payload {
                        Some((payload, _)) if payload.len() == 28
                            && payload[..4] == [code as u8, 0, 0, 0]
                            && u32_at(payload, 20) == 1
                            && payload[24..28] == [0, 0, 0, 0]
                            && f64_at(payload, 4).is_finite()
                            && f64_at(payload, 12).is_finite()
                            && (f64_at(payload, 4) - f64_at(payload, 12) * 37.79527559055118).abs() < 1e-5 =>
                            Kind::MixedRectangularTranslatedPolygon(*source, *distance, f64_at(payload, 12), code == 18),
                        _ => Kind::Unsupported(code),
                    },
                    (24, _, [source, distance])
                        if *source > 0 && *distance > 0
                            && objects.get(source - 1).is_some_and(|o| is_point_kind(&o.kind) || matches!(o.kind, Kind::Segment(_, _) | Kind::Circle(_, _) | Kind::CircleBySegment(_, _) | Kind::Polygon(_)))
                            && objects.get(distance - 1).is_some_and(|o| is_distance_value_kind(&o.kind)) =>
                    {
                        match &constraint_payload {
                            Some((payload, _))
                                if payload.len() == 36
                                    && payload[..4] == [24, 0, 0, 0]
                                    && payload[28..36] == [1, 0, 0, 0, 0, 0, 0, 0]
                                    && f64_at(payload, 20).is_finite()
                                    && f64_at(payload, 4).is_finite()
                                    && f64_at(payload, 12).is_finite()
                                    && (f64_at(payload, 4)
                                        + f64_at(payload, 20).to_radians().sin()).abs() < 1e-6
                                    && (f64_at(payload, 12)
                                        - f64_at(payload, 20).to_radians().cos()).abs() < 1e-6 =>
                            {
                                match (&objects[source - 1].kind, &h[8..10]) {
                                    (kind, [4, 0]) if is_point_kind(kind) => Kind::MarkedDistanceTranslatedPoint(*source, *distance, f64_at(payload, 20)),
                                    (Kind::Segment(_, _), [14, 0]) => Kind::MarkedDistanceTranslatedSegment(*source, *distance, f64_at(payload, 20)),
                                    (Kind::Circle(_, _) | Kind::CircleBySegment(_, _), [28, 0]) => Kind::MarkedDistanceTranslatedCircle(*source, *distance, f64_at(payload, 20)),
                                    (Kind::Polygon(vertices), _) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]]))
                                        && h[6..8] == [4, 0] && h[20..24] == [1, 0, 0, 0] => Kind::MarkedDistanceTranslatedPolygon(*source, *distance, f64_at(payload, 20)),
                                    _ => Kind::Unsupported(code),
                                }
                            }
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (20, None, [source, horizontal, vertical])
                        if *source > 0 && objects.get(source - 1).is_some_and(|o| is_point_kind(&o.kind))
                            && [horizontal, vertical].iter().all(|&&id| {
                                id > 0 && objects.get(id - 1).is_some_and(|o| is_distance_value_kind(&o.kind))
                            }) =>
                    {
                        Kind::RectangularMarkedTranslatedPoint(*source, *horizontal, *vertical)
                    }
                    (20, None, [source, horizontal, vertical])
                        if objects.get(source.saturating_sub(1)).is_some_and(|o| matches!(&o.kind, Kind::Polygon(vertices) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]]))))
                            && [horizontal, vertical].iter().all(|&&id| objects.get(id.saturating_sub(1)).is_some_and(|o| is_distance_value_kind(&o.kind)))
                            && h[6..8] == [4, 0] && h[20..24] == [1, 0, 0, 0] => {
                        Kind::RectangularMarkedTranslatedPolygon(*source, *horizontal, *vertical)
                    }
                    (16, _, [source, start, end])
                        if [source, start, end].iter().all(|&&id| id > 0 && objects.get(id - 1).is_some())
                            && [start, end].iter().all(|&&id| is_point_kind(&objects[id - 1].kind)) => {
                        match (&objects[source - 1].kind, &h[8..10]) {
                            (kind, [4, 0]) if is_point_kind(kind) => Kind::VectorTranslatedPoint(*source, *start, *end),
                            (Kind::Segment(_, _), [14, 0]) => Kind::VectorTranslatedSegment(*source, *start, *end),
                            (Kind::Circle(_, _) | Kind::CircleBySegment(_, _), [28, 0]) => Kind::VectorTranslatedCircle(*source, *start, *end),
                            (Kind::Polygon(vertices), _) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]]))
                                && h[6..8] == [4, 0] && h[20..24] == [1, 0, 0, 0] => Kind::VectorTranslatedPolygon(*source, *start, *end),
                            _ => Kind::Unsupported(code),
                        }
                    },
                    (28, _, [source, center, first, vertex, last])
                        if [center, first, vertex, last].iter().all(|&&id| {
                            objects.get(id - 1).is_some_and(|object| is_point_kind(&object.kind))
                        }) => match (objects.get(source.saturating_sub(1)).map(|o| &o.kind), &h[8..10]) {
                            (Some(kind), [4, 0]) if is_point_kind(kind) => Kind::AngleRotatedPoint(*source, *center, *first, *vertex, *last),
                            (Some(Kind::Segment(_, _)), [14, 0]) => Kind::AngleRotatedSegment(*source, *center, *first, *vertex, *last),
                            (Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)), [28, 0]) => Kind::AngleRotatedCircle(*source, *center, *first, *vertex, *last),
                            (Some(Kind::Polygon(vertices)), _) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]]))
                                && h[6..8] == [4, 0] && h[20..24] == [1, 0, 0, 0] => Kind::AngleRotatedPolygon(*source, *center, *first, *vertex, *last),
                            _ => Kind::Unsupported(code),
                        },
                    (32, _, [source, center, origin, base, target])
                        if [source, center, origin, base, target].iter().all(|&&id| {
                            objects.get(id - 1).is_some_and(|object| is_point_kind(&object.kind))
                        }) => Kind::RatioScaledPoint(*source, *center, *origin, *base, *target),
                    (32, _, [source, center, origin, base, target])
                        if objects.get(source - 1).is_some_and(|object| is_circle_kind(&object.kind))
                            && [center, origin, base, target].iter().all(|&&id| {
                                objects.get(id - 1).is_some_and(|object| is_point_kind(&object.kind))
                            }) => Kind::RatioScaledCircle(*source, *center, *origin, *base, *target),
                    (32, _, [source, center, origin, base, target])
                        if objects.get(source.saturating_sub(1)).is_some_and(|object| matches!(object.kind, Kind::Segment(_, _)))
                            && [center, origin, base, target].iter().all(|&&id| {
                                objects.get(id.saturating_sub(1)).is_some_and(|object| is_point_kind(&object.kind))
                            }) && h[8..10] == [14, 0] => Kind::RatioScaledSegment(*source, *center, *origin, *base, *target),
                    (32, _, [source, center, origin, base, target])
                        if objects.get(source.saturating_sub(1)).is_some_and(|object| matches!(&object.kind, Kind::Polygon(vertices) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]]))))
                            && [center, origin, base, target].iter().all(|&&id| objects.get(id.saturating_sub(1)).is_some_and(|object| is_point_kind(&object.kind)))
                            && h[6..8] == [4, 0] && h[20..24] == [1, 0, 0, 0] => Kind::RatioScaledPolygon(*source, *center, *origin, *base, *target),
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
                    (34, _, [source, mirror])
                        if objects.get(source.saturating_sub(1)).is_some_and(|o| matches!(o.kind, Kind::Segment(_, _)))
                            && objects.get(mirror.saturating_sub(1)).is_some_and(|o| is_straight_kind(&o.kind))
                            && h[8..10] == [14, 0] => Kind::ReflectedSegment(*source, *mirror),
                    (34, _, [source, mirror])
                        if objects.get(source.saturating_sub(1)).is_some_and(|o| matches!(&o.kind, Kind::Polygon(vertices) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]]))))
                            && objects.get(mirror.saturating_sub(1)).is_some_and(|o| is_straight_kind(&o.kind))
                            && h[20..24] == [1, 0, 0, 0] => Kind::ReflectedPolygon(*source, *mirror),
                    (30, _, [source, center])
                        if objects.get(center - 1).is_some_and(|o| is_point_kind(&o.kind)) =>
                    {
                        match &constraint_payload {
                            Some((payload, _)) if payload.len() == 12 && payload[..4] == [30, 0, 0, 0] && f64_at(payload, 4).is_finite() => {
                                let factor = f64_at(payload, 4);
                                match objects.get(source - 1).map(|o| &o.kind) {
                                    Some(kind) if is_point_kind(kind) => Kind::ScaledPoint(*source, *center, factor),
                                    Some(kind) if is_circle_kind(kind) => Kind::ScaledCircle(*source, *center, factor),
                                    Some(Kind::Segment(_, _)) if h[8..10] == [14, 0] => Kind::ScaledSegment(*source, *center, factor),
                                    Some(Kind::Polygon(vertices)) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]]))
                                        && h[6..8] == [4, 0] && h[20..24] == [1, 0, 0, 0] => Kind::ScaledPolygon(*source, *center, factor),
                                    _ => Kind::Unsupported(code),
                                }
                            }
                            _ => Kind::Unsupported(code),
                        }
                    }
                    (27, _, [source, center])
                        if objects.get(center.saturating_sub(1)).is_some_and(|o| is_point_kind(&o.kind)) =>
                    {
                        match &constraint_payload {
                            Some((payload, _)) if payload.len() == 36 && payload[..4] == [27, 0, 0, 0] && f64_at(payload, 20).is_finite()
                                && f64_at(payload, 4).is_finite() && f64_at(payload, 12).is_finite()
                                && (f64_at(payload, 4) + f64_at(payload, 20).to_radians().sin()).abs() < 1e-6
                                && (f64_at(payload, 12) - f64_at(payload, 20).to_radians().cos()).abs() < 1e-6 => {
                                match (objects.get(source.saturating_sub(1)).map(|o| &o.kind), &h[8..10]) {
                                    (Some(kind), [4, 0]) if is_point_kind(kind) => Kind::RotatedPoint(*source, *center, f64_at(payload, 20)),
                                    (Some(Kind::Segment(_, _)), [14, 0]) => Kind::RotatedSegment(*source, *center, f64_at(payload, 20)),
                                    (Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)), [28, 0]) => Kind::RotatedCircle(*source, *center, f64_at(payload, 20)),
                                    (Some(Kind::Polygon(vertices)), _) if polygon_subtype(vertices.len()) == Some(u16::from_le_bytes([h[8], h[9]]))
                                        && h[6..8] == [4, 0] && h[20..24] == [1, 0, 0, 0] => Kind::RotatedPolygon(*source, *center, f64_at(payload, 20)),
                                    _ => Kind::Unsupported(code),
                                }
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
            let function_name = if matches!(kind, Kind::FunctionDefinition(_)) {
                label
                    .as_ref()
                    .and_then(|value| value.name_utf8().ok())
                    .map(str::to_owned)
            } else {
                None
            };
            if matches!(
                kind,
                Kind::FunctionDefinition(_)
                    | Kind::Parameter(_)
                    | Kind::LinkUrlButton { .. }
                    | Kind::LinkPageButton { .. }
            ) {
                label = None;
            }
            if label.is_some() && !is_point_kind(&kind) {
                safe = false;
                label = None;
            }
            let function_anchor = if matches!(
                kind,
                Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _)
            ) {
                function_expression
                    .as_deref()
                    .and_then(function_records::expression_anchor)
            } else {
                None
            };
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
                text_style_override: None,
                rich_text_override: None,
                function_anchor,
                function_name,
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
        let mut marked_coordinate_system = vec![None; page_count.max(1)];
        let mut link_page = 0usize;
        let mut seen_page = false;
        for record in &rs {
            if record.kind == 0x3e8 {
                if seen_page {
                    link_page += 1;
                }
                seen_page = true;
            } else if record.kind == 0x232d && record.data.len() == 4 {
                let id = u32_at(b, record.data.start) as usize;
                if objects.get(id.saturating_sub(1)).is_some_and(|object| {
                    object.page == link_page && matches!(object.kind, Kind::CoordinateSystem(_, _))
                }) {
                    marked_coordinate_system[link_page] = Some(id);
                } else {
                    safe = false;
                }
            }
        }
        for (page, marked) in marked_coordinate_system.iter().enumerate() {
            if let Some((form, dotted)) = marked.and_then(|id| frame_styles.get(&id).copied()) {
                grid_form[page] = form;
                grid_dotted[page] = dotted;
            }
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
            angle_unit_radians,
            marked_coordinate_system,
            frame_styles,
            original: b.to_vec(),
            run_font_families: Vec::new(),
            source_count,
            last_moved_object: None,
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
            self.angle_unit_radians,
            self.marked_coordinate_system.clone(),
            self.frame_styles.clone(),
            self.run_font_families.clone(),
            self.last_moved_object,
        ));
        self.redo.clear();
        self.dirty = true;
    }
    pub fn undo(&mut self) {
        if let Some((
            objects,
            snap_points,
            grid_dotted,
            grid_form,
            radians,
            marked,
            styles,
            families,
            last_moved_object,
        )) = self.undo.pop()
        {
            self.redo.push((
                std::mem::replace(&mut self.objects, objects),
                std::mem::replace(&mut self.snap_points, snap_points),
                std::mem::replace(&mut self.grid_dotted, grid_dotted),
                std::mem::replace(&mut self.grid_form, grid_form),
                std::mem::replace(&mut self.angle_unit_radians, radians),
                std::mem::replace(&mut self.marked_coordinate_system, marked),
                std::mem::replace(&mut self.frame_styles, styles),
                std::mem::replace(&mut self.run_font_families, families),
                std::mem::replace(&mut self.last_moved_object, last_moved_object),
            ));
            self.dirty = true;
        }
    }
    pub fn redo(&mut self) {
        if let Some((
            objects,
            snap_points,
            grid_dotted,
            grid_form,
            radians,
            marked,
            styles,
            families,
            last_moved_object,
        )) = self.redo.pop()
        {
            self.undo.push((
                std::mem::replace(&mut self.objects, objects),
                std::mem::replace(&mut self.snap_points, snap_points),
                std::mem::replace(&mut self.grid_dotted, grid_dotted),
                std::mem::replace(&mut self.grid_form, grid_form),
                std::mem::replace(&mut self.angle_unit_radians, radians),
                std::mem::replace(&mut self.marked_coordinate_system, marked),
                std::mem::replace(&mut self.frame_styles, styles),
                std::mem::replace(&mut self.run_font_families, families),
                std::mem::replace(&mut self.last_moved_object, last_moved_object),
            ));
            self.dirty = true;
        }
    }
    pub fn add(&mut self, kind: Kind) -> usize {
        self.checkpoint();
        self.append_object(kind)
    }
    pub fn construct_locus(&mut self, driver: usize, target: usize) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot construct a locus in this sketch safely".into());
        }
        let refs = expected_locus_refs(&self.objects, driver, target)
            .ok_or("Select a constrained point and a dependent object")?;
        let samples = if is_locus_polygon_kind(&self.objects[target - 1].kind) {
            125
        } else if is_locus_straight_kind(&self.objects[target - 1].kind)
            || is_circle_kind(&self.objects[target - 1].kind)
            || is_arc_kind(&self.objects[target - 1].kind)
        {
            250
        } else {
            500
        };
        Ok(self.add(Kind::Locus(refs, samples)))
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
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        let text = normalized.as_str();
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid text ID")?)
            .ok_or("Invalid text ID")?;
        let Kind::PlainText {
            rect,
            text: ref current,
        } = object.kind
        else {
            return Err("Object is not free text".into());
        };
        let rect = TextRect::roomy_at(rect.left, rect.top, text)?;
        text_object::encode_plain_text_group(text, rect)?;
        let rich = if current != text {
            let runs = self.text_rich_runs(id).ok_or("Unsupported text markup")?;
            if runs
                .iter()
                .flatten()
                .any(|run| run.style != text_object::RichRunStyle::default())
            {
                Some(text_object::remap_rich_text(&runs, text, rect)?)
            } else {
                None
            }
        } else {
            object.rich_text_override.clone()
        };
        self.checkpoint();
        self.objects[id - 1].kind = Kind::PlainText {
            text: text.to_owned(),
            rect,
        };
        self.objects[id - 1].rich_text_override = rich;
        Ok(())
    }
    fn text_rich_template(&self, id: usize) -> Option<text_object::RichTextTemplate> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::PlainText { ref text, .. } = object.kind else {
            return None;
        };
        if let Some(template) = &object.rich_text_override {
            return (template.decode().ok()?.0 == *text).then(|| template.clone());
        }
        let header = object.header_offset?;
        let rs = records(&self.original).ok()?;
        let start = rs
            .iter()
            .position(|record| record.kind == 0x07d0 && record.data.start == header)?;
        let group = rs[start + 1..]
            .iter()
            .take_while(|record| record.kind != 0x07d7)
            .collect::<Vec<_>>();
        let rich = group.iter().find(|record| record.kind == 0x08fc)?;
        let bbox = group.iter().find(|record| record.kind == 0x0898)?;
        let template = text_object::RichTextTemplate::from_parts(
            &self.original[rich.data.clone()],
            &self.original[bbox.data.clone()],
        )
        .ok()?;
        (template.decode().ok()?.0 == *text).then_some(template)
    }
    pub fn text_rich_runs(&self, id: usize) -> Option<Vec<Vec<text_object::RichTextRun>>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::PlainText { ref text, .. } = object.kind else {
            return None;
        };
        if let Some(template) = self.text_rich_template(id) {
            return Some(template.decode().ok()?.2);
        }
        Some(
            text.split('\n')
                .map(|line| {
                    vec![text_object::RichTextRun {
                        text: line.to_owned(),
                        style: text_object::RichRunStyle::default(),
                    }]
                })
                .collect(),
        )
    }
    pub fn text_font_state(&self, id: usize) -> Option<FontState> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if !matches!(object.kind, Kind::PlainText { .. })
            && !(is_point_kind(&object.kind) && object.label.is_some())
        {
            return None;
        }
        if let Some(style) = &object.text_style_override {
            return Some(style.clone());
        }
        let style = if let Some(header) = object.header_offset {
            let rs = records(&self.original).ok()?;
            let start = rs
                .iter()
                .position(|record| record.kind == 0x07d0 && record.data.start == header)?;
            let record = rs[start + 1..]
                .iter()
                .take_while(|record| record.kind != 0x07d7)
                .find(|record| record.kind == 0x07d8 && record.data.len() == 2);
            record.map_or(0, |record| {
                u16::from_le_bytes(self.original[record.data.clone()].try_into().unwrap()) as usize
            })
        } else {
            0
        };
        text_style_records::decode_font_state_at(&self.original, style).ok()
    }
    pub fn text_font_families(&self) -> Option<Vec<String>> {
        let mut families = text_style_records::font_families(&self.original).ok()?;
        families.extend(self.run_font_families.iter().cloned());
        Some(families)
    }
    pub fn toggle_selected_text_style(
        &mut self,
        command: TextStyleCommand,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot format text in this sketch safely".into());
        }
        let updates = self
            .selected
            .iter()
            .copied()
            .filter(|&id| {
                self.objects.get(id - 1).is_some_and(|object| {
                    !object.deleted
                        && object.page == self.active_page
                        && (matches!(object.kind, Kind::PlainText { .. })
                            || (is_point_kind(&object.kind) && object.label.is_some()))
                })
            })
            .map(|id| {
                let mut style = self.text_font_state(id).ok_or("Unsupported text font")?;
                match command {
                    TextStyleCommand::Bold => style.bold = !style.bold,
                    TextStyleCommand::Italic => style.italic = !style.italic,
                    TextStyleCommand::Underline => style.underline = !style.underline,
                    TextStyleCommand::IncreaseSize => {
                        style.size_code = match style.size_code {
                            14 => 16,
                            16 => 18,
                            18 => 24,
                            24 => 36,
                            36 => 48,
                            48 => 64,
                            _ => return Err("This text size has no verified larger step".into()),
                        }
                    }
                    TextStyleCommand::DecreaseSize => {
                        style.size_code = match style.size_code {
                            16 => 14,
                            18 => 16,
                            24 => 18,
                            36 => 24,
                            48 => 36,
                            64 => 48,
                            _ => return Err("This text size has no verified smaller step".into()),
                        }
                    }
                }
                Ok((id, style))
            })
            .collect::<Result<Vec<_>, String>>()?;
        if updates.is_empty() {
            return Err("Select free text to format".into());
        }
        let count = updates.len();
        self.checkpoint();
        for (id, style) in updates {
            self.objects[id - 1].text_style_override = Some(style);
        }
        Ok(count)
    }
    pub fn set_selected_text_family(&mut self, family: &str) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot format text in this sketch safely".into());
        }
        text_style_records::encode_font_family_payload(family)?;
        let updates = self
            .selected
            .iter()
            .copied()
            .filter(|&id| {
                self.objects.get(id - 1).is_some_and(|object| {
                    !object.deleted
                        && object.page == self.active_page
                        && (matches!(object.kind, Kind::PlainText { .. })
                            || (is_point_kind(&object.kind) && object.label.is_some()))
                })
            })
            .map(|id| {
                let mut style = self.text_font_state(id).ok_or("Unsupported text font")?;
                style.family = family.to_owned();
                Ok((id, style))
            })
            .collect::<Result<Vec<_>, String>>()?;
        if updates.is_empty() {
            return Err("Select free text or labeled points to format".into());
        }
        let count = updates.len();
        self.checkpoint();
        for (id, style) in updates {
            self.objects[id - 1].text_style_override = Some(style);
        }
        Ok(count)
    }
    pub fn format_text_range(
        &mut self,
        id: usize,
        start: usize,
        end: usize,
        command: TextStyleCommand,
    ) -> Result<(), String> {
        self.format_text_range_with(id, start, end, |style, base| {
            match command {
                TextStyleCommand::Bold | TextStyleCommand::Italic | TextStyleCommand::Underline => {
                    let (override_flag, baseline) = match command {
                        TextStyleCommand::Bold => (&mut style.bold, base.bold),
                        TextStyleCommand::Italic => (&mut style.italic, base.italic),
                        TextStyleCommand::Underline => (&mut style.underline, base.underline),
                        _ => unreachable!(),
                    };
                    let next = !override_flag.unwrap_or(baseline);
                    *override_flag = (next != baseline).then_some(next);
                }
                TextStyleCommand::IncreaseSize | TextStyleCommand::DecreaseSize => {
                    let current = style.size_code.unwrap_or(base.size_code);
                    let next = match (command, current) {
                        (TextStyleCommand::IncreaseSize, 14) => 16,
                        (TextStyleCommand::IncreaseSize, 16) => 18,
                        (TextStyleCommand::IncreaseSize, 18) => 24,
                        (TextStyleCommand::IncreaseSize, 24) => 36,
                        (TextStyleCommand::IncreaseSize, 36) => 48,
                        (TextStyleCommand::IncreaseSize, 48) => 64,
                        (TextStyleCommand::DecreaseSize, 16) => 14,
                        (TextStyleCommand::DecreaseSize, 18) => 16,
                        (TextStyleCommand::DecreaseSize, 24) => 18,
                        (TextStyleCommand::DecreaseSize, 36) => 24,
                        (TextStyleCommand::DecreaseSize, 48) => 36,
                        (TextStyleCommand::DecreaseSize, 64) => 48,
                        (TextStyleCommand::IncreaseSize, _) => {
                            return Err("This text size has no verified larger step".into());
                        }
                        _ => return Err("This text size has no verified smaller step".into()),
                    };
                    style.size_code = (next != base.size_code).then_some(next);
                }
            }
            Ok(())
        })
    }
    pub fn format_text_range_family(
        &mut self,
        id: usize,
        start: usize,
        end: usize,
        family: &str,
    ) -> Result<(), String> {
        text_style_records::encode_font_family_payload(family)?;
        let families = self
            .text_font_families()
            .ok_or("Unsupported font family table")?;
        let index = families.iter().position(|name| name == family);
        let next_index = index.unwrap_or(families.len());
        let next_index = u16::try_from(next_index).map_err(|_| "Too many font families")?;
        self.format_text_range_with(id, start, end, |style, base| {
            style.family_index = (family != base.family).then_some(next_index);
            Ok(())
        })?;
        if index.is_none() {
            self.run_font_families.push(family.to_owned());
        }
        Ok(())
    }
    fn format_text_range_with(
        &mut self,
        id: usize,
        start: usize,
        end: usize,
        mut modify: impl FnMut(&mut text_object::RichRunStyle, &FontState) -> Result<(), String>,
    ) -> Result<(), String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot format text in this sketch safely".into());
        }
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid text ID")?)
            .ok_or("Invalid text ID")?;
        let Kind::PlainText { ref text, rect } = object.kind else {
            return Err("Object is not free text".into());
        };
        if object.deleted || object.page != self.active_page {
            return Err("Text is not on the active page".into());
        }
        if start >= end || end > text.chars().count() {
            return Err("Select a valid text range".into());
        }
        let base = self.text_font_state(id).ok_or("Unsupported text font")?;
        let rows = self.text_rich_runs(id).ok_or("Unsupported text markup")?;
        let mut position = 0;
        let mut changed = 0;
        let mut styled_rows = Vec::new();
        let row_count = rows.len();
        for (line_index, row) in rows.into_iter().enumerate() {
            let mut styled = Vec::<text_object::RichTextRun>::new();
            for run in row {
                for ch in run.text.chars() {
                    let mut style = run.style;
                    if (start..end).contains(&position) {
                        modify(&mut style, &base)?;
                        changed += 1;
                    }
                    if let Some(last) = styled.last_mut().filter(|last| last.style == style) {
                        last.text.push(ch);
                    } else {
                        styled.push(text_object::RichTextRun {
                            text: ch.to_string(),
                            style,
                        });
                    }
                    position += 1;
                }
            }
            if styled.is_empty() {
                styled.push(text_object::RichTextRun {
                    text: String::new(),
                    style: text_object::RichRunStyle::default(),
                });
            }
            styled_rows.push(styled);
            if line_index + 1 < row_count {
                position += 1;
            }
        }
        if changed == 0 {
            return Err("Select text characters to format".into());
        }
        let template = text_object::encode_rich_runs(&styled_rows, rect)?;
        if template.decode()?.0 != *text {
            return Err("Styled text did not round-trip".into());
        }
        self.checkpoint();
        self.objects[id - 1].rich_text_override = Some(template);
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
        | Kind::AngleRotatedPolygon(source, _, _, _, _)
        | Kind::ScaledPolygon(source, _, _)
        | Kind::RatioScaledPolygon(source, _, _, _, _) = kind
        {
            self.objects
                .get(source.saturating_sub(1))
                .and_then(|object| object.visual_style)
                .or_else(|| default_visual_style(&kind))
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
        self.objects.push(Object {
            id,
            page: self.active_page,
            visual_style,
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
        });
        id
    }
    pub fn add_default_parameter(&mut self, anchor: (i16, i16)) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot add a parameter in this sketch".into());
        }
        let first = self.objects.iter().find(|object| {
            !object.deleted
                && matches!(&object.kind, Kind::Parameter(group) if group.name() == "t[1]")
        });
        let second = self.objects.iter().find(|object| {
            !object.deleted
                && matches!(&object.kind, Kind::Parameter(group) if group.name() == "t[2]")
        });
        let mut group = match (first, second) {
            (None, None) => ParameterGroup::default_value(),
            (Some(_), None) => ParameterGroup::second_default_value(),
            _ => return Err("The supported parameter sequence is t[1], then t[2]".into()),
        };
        group.set_anchor(anchor)?;
        self.checkpoint();
        Ok(self.append_object(Kind::Parameter(group)))
    }
    pub fn add_named_parameter(
        &mut self,
        name: &str,
        value: f64,
        anchor: (i16, i16),
    ) -> Result<usize, String> {
        if self.partial_view || !self.safe_structure || self.page_count != 1 {
            return Err("Cannot add a parameter in this sketch".into());
        }
        if self.objects.iter().any(|object| {
            !object.deleted
                && matches!(&object.kind, Kind::Parameter(group) if group.name() == name)
        }) {
            return Err("A parameter with this name already exists".into());
        }
        let mut group = ParameterGroup::named(name, value)?;
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
        if self.partial_view || self.page_count != 1 || id == 0 {
            return Err("Cannot edit a parameter in this sketch".into());
        }
        let Some(Kind::Parameter(group)) = self.objects.get(id - 1).map(|o| &o.kind) else {
            return Err("Select a parameter".into());
        };
        let mut group = group.clone();
        group.set_value(value)?;
        let mut tables = Vec::new();
        let mut single_tables = Vec::new();
        if group.name() == "t[1]" {
            for (index, object) in self.objects.iter().enumerate() {
                if let Kind::ValueTable(source, table) = &object.kind
                    && *source == id
                    && table.default_values().is_some()
                {
                    let mut changed = table.clone();
                    changed.set_last_default_value(value)?;
                    single_tables.push((index, *source, changed));
                }
            }
        }
        if matches!(group.name(), "t[1]" | "t[2]") {
            for (index, object) in self.objects.iter().enumerate() {
                if let Kind::ValueTablePair(first, second, table) = &object.kind {
                    if *first == id || *second == id {
                        let mut changed = table.clone();
                        changed.set_last_pair_axis_value(usize::from(*second == id), value)?;
                        tables.push((index, *first, *second, changed));
                    }
                }
            }
        }
        self.checkpoint();
        self.objects[id - 1].kind = Kind::Parameter(group);
        for (index, source, table) in single_tables {
            self.objects[index].kind = Kind::ValueTable(source, table);
        }
        for (index, first, second, table) in tables {
            self.objects[index].kind = Kind::ValueTablePair(first, second, table);
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
        if self.partial_view || self.page_count != 1 || source == 0 {
            return Err("Cannot tabulate a parameter in this sketch".into());
        }
        let Some(Kind::Parameter(parameter)) = self.objects.get(source - 1).map(|o| &o.kind) else {
            return Err("Select a supported parameter".into());
        };
        let group = if parameter.name() == "t[1]" {
            TableGroup::for_source_value(
                u32::try_from(source).map_err(|_| "Invalid parameter ID")?,
                anchor,
                parameter.value(),
            )?
        } else if !matches!(parameter.name(), "t[1]" | "t[2]") {
            TableGroup::named_snapshot(
                u32::try_from(source).map_err(|_| "Invalid parameter ID")?,
                parameter,
                anchor,
            )?
        } else {
            return Err("This default parameter table is not yet supported".into());
        };
        self.checkpoint();
        Ok(self.append_object(Kind::ValueTable(source, group)))
    }
    pub fn tabulate_default_pair(&mut self, anchor: (i16, i16)) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || !self.safe_structure {
            return Err("Cannot tabulate this parameter pair".into());
        }
        let pair = self
            .objects
            .iter()
            .filter_map(|object| match &object.kind {
                Kind::Parameter(group) if !object.deleted && group.name() == "t[1]" => {
                    Some((object.id, 1, group.value()))
                }
                Kind::Parameter(group) if !object.deleted && group.name() == "t[2]" => {
                    Some((object.id, 2, group.value()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let [first, second] = pair.as_slice() else {
            return Err("Select supported t[1] and t[2] parameters".into());
        };
        if first.1 != 1 || second.1 != 2 {
            return Err("Select supported t[1] and t[2] parameters".into());
        }
        let sources = [
            u32::try_from(second.0).map_err(|_| "Invalid parameter ID")?,
            u32::try_from(first.0).map_err(|_| "Invalid parameter ID")?,
        ];
        let mut group = TableGroup::pair_snapshot_rows_for_sources(1, anchor, sources)?;
        group.set_pair_row_values(0, [second.2, first.2])?;
        self.checkpoint();
        Ok(self.append_object(Kind::ValueTablePair(second.0, first.0, group)))
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
            Kind::ValueTable(source, group) => {
                let mut group = group.clone();
                if add {
                    if group.named_label_values().is_some() {
                        let Some(Kind::Parameter(parameter)) = self
                            .objects
                            .get(source.saturating_sub(1))
                            .map(|object| &object.kind)
                        else {
                            return Err("Invalid named table source".into());
                        };
                        let value = format!("{:.*}", parameter.precision(), parameter.value());
                        group.add_named_snapshot(&value)?;
                    } else {
                        group.add_default_snapshot()?;
                    }
                } else {
                    group.remove_last_snapshot()?;
                }
                Kind::ValueTable(*source, group)
            }
            Kind::ValueTablePair(first, second, group) => {
                let values = group.pair_values().ok_or("Invalid pair table")?;
                let rows = if add {
                    group.rows() + 1
                } else {
                    group.rows().checked_sub(1).ok_or("Table has no rows")?
                };
                let mut changed = TableGroup::pair_snapshot_rows_for_sources(
                    rows,
                    group.anchor(),
                    group
                        .pair_source_ids()
                        .ok_or("Invalid pair table sources")?,
                )?;
                for row in 0..rows {
                    let value = if row + 1 == rows {
                        values.last().unwrap()
                    } else {
                        &values[row]
                    };
                    changed.set_pair_row_values(row, *value)?;
                }
                Kind::ValueTablePair(*first, *second, changed)
            }
            _ => return Err("Object is not a table".into()),
        };
        self.checkpoint();
        self.objects[id - 1].kind = kind;
        Ok(())
    }
    pub fn active_coordinate_system(&self) -> Option<usize> {
        self.active_coordinate_system_for_page(self.active_page)
    }
    fn active_coordinate_system_for_page(&self, page: usize) -> Option<usize> {
        if let Some(id) = self.marked_coordinate_system.get(page).copied().flatten()
            && self.objects.get(id.checked_sub(1)?).is_some_and(|object| {
                object.page == page
                    && !object.deleted
                    && matches!(object.kind, Kind::CoordinateSystem(_, _))
            })
        {
            return Some(id);
        }
        self.objects
            .iter()
            .rev()
            .find(|o| {
                o.page == page && !o.deleted && matches!(o.kind, Kind::CoordinateSystem(_, _))
            })
            .map(|o| o.id)
    }
    pub fn coordinate_system_for_object(&self, id: usize) -> Option<usize> {
        self.objects.iter().rev().find_map(|frame| {
            if frame.page != self.active_page || frame.deleted {
                return None;
            }
            let Kind::CoordinateSystem(x_axis, y_axis) = frame.kind else {
                return None;
            };
            match (
                self.objects.get(x_axis.checked_sub(1)?)?.kind.clone(),
                self.objects.get(y_axis.checked_sub(1)?)?.kind.clone(),
            ) {
                (Kind::GraphAxis(origin, x_unit, 1 | 0x1_0001), Kind::GraphAxis(_, y_unit, 0)) => {
                    [frame.id, x_axis, y_axis, origin, x_unit, y_unit]
                        .contains(&id)
                        .then_some(frame.id)
                }
                (Kind::CircleAxis(circle, 1), Kind::CircleAxis(other, 0)) if circle == other => {
                    [frame.id, x_axis, y_axis, circle]
                        .contains(&id)
                        .then_some(frame.id)
                }
                (Kind::DistanceAxis(origin, distance, 1), Kind::DistanceAxis(other, second, 0))
                    if origin == other =>
                {
                    [frame.id, x_axis, y_axis, origin, distance, second]
                        .contains(&id)
                        .then_some(frame.id)
                }
                (Kind::DistanceAxis(origin, distance, 1), Kind::GraphAxis(other, unit, 0))
                    if origin == other
                        && matches!(
                            self.objects.get(unit.checked_sub(1)?)?.kind,
                            Kind::DistanceVerticalUnit(host, _, _) if host == x_axis
                        ) =>
                {
                    [frame.id, x_axis, y_axis, origin, distance, unit]
                        .contains(&id)
                        .then_some(frame.id)
                }
                _ => None,
            }
        })
    }
    pub fn mark_coordinate_system(&mut self, selected: usize) -> Result<usize, String> {
        let frame = self
            .coordinate_system_for_object(selected)
            .ok_or("Select a coordinate system axis, origin, or unit point")?;
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot mark a coordinate system in this sketch".into());
        }
        if self.active_coordinate_system() != Some(frame) {
            self.checkpoint();
            self.marked_coordinate_system[self.active_page] = Some(frame);
            if let Some((form, dotted)) = self.frame_styles.get(&frame).copied() {
                self.grid_form[self.active_page] = form;
                self.grid_dotted[self.active_page] = dotted;
            }
        }
        Ok(frame)
    }
    fn active_x_axis(&self, page: usize) -> Option<usize> {
        let frame = self.objects.get(
            self.active_coordinate_system_for_page(page)?
                .checked_sub(1)?,
        )?;
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
        self.marked_coordinate_system[self.active_page] = Some(frame);
        self.frame_styles.insert(frame, (GridForm::Square, false));
        self.snap_points[self.active_page] = false;
        self.grid_dotted[self.active_page] = false;
        self.grid_form[self.active_page] = GridForm::Square;
        Ok(frame)
    }
    pub fn add_coordinate_system_at_point(&mut self, origin: usize) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || !self.safe_structure {
            return Err("Cannot define a coordinate system in this sketch".into());
        }
        let point = self.position(origin).ok_or("Select one origin point")?;
        if !point.x.is_finite() || !point.y.is_finite() {
            return Err("Coordinate-system origin must be finite".into());
        }
        let step: f64 = 37.79527559055118;
        self.checkpoint();
        let x_unit = self.append_object(Kind::GraphHorizontalUnit(origin, step));
        let x_axis = self.append_object(Kind::GraphAxis(origin, x_unit, 1));
        let y_unit = self.append_object(Kind::GraphVerticalUnit(x_unit, step));
        let y_axis = self.append_object(Kind::GraphAxis(origin, y_unit, 0));
        let frame = self.append_object(Kind::CoordinateSystem(x_axis, y_axis));
        self.marked_coordinate_system[self.active_page] = Some(frame);
        self.frame_styles.insert(frame, (GridForm::Square, false));
        self.grid_form[self.active_page] = GridForm::Square;
        self.grid_dotted[self.active_page] = false;
        Ok(frame)
    }
    pub fn add_coordinate_system_from_circle(&mut self, circle: usize) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || !self.safe_structure {
            return Err("Cannot define a coordinate system in this sketch".into());
        }
        if !self
            .objects
            .get(circle.saturating_sub(1))
            .is_some_and(|object| {
                object.page == self.active_page && !object.deleted && is_circle_kind(&object.kind)
            })
        {
            return Err("Select one circle".into());
        }
        let (_, radius) = self.circle_points(circle).ok_or("Invalid unit circle")?;
        if !radius.is_finite() || radius <= 0.0 {
            return Err("Invalid unit circle".into());
        }
        self.checkpoint();
        let x_axis = self.append_object(Kind::CircleAxis(circle, 1));
        let y_axis = self.append_object(Kind::CircleAxis(circle, 0));
        let frame = self.append_object(Kind::CoordinateSystem(x_axis, y_axis));
        self.marked_coordinate_system[self.active_page] = Some(frame);
        self.frame_styles.insert(frame, (GridForm::Square, false));
        self.grid_form[self.active_page] = GridForm::Square;
        self.grid_dotted[self.active_page] = false;
        Ok(frame)
    }
    pub fn add_coordinate_system_from_distances(
        &mut self,
        origin: Option<usize>,
        distances: &[usize],
        default_origin: Point,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || !self.safe_structure {
            return Err("Cannot define a coordinate system in this sketch".into());
        }
        if !matches!(distances.len(), 1 | 2)
            || distances[0] == *distances.last().unwrap() && distances.len() == 2
            || distances.iter().any(|&id| {
                self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                    id == 0
                        || object.deleted
                        || object.page != self.active_page
                        || !is_defining_distance_kind(&object.kind)
                }) || self.defining_distance_pixels(id).is_none()
            })
        {
            return Err("Select one or two nonzero defining distances".into());
        }
        if let Some(id) = origin {
            if self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                id == 0
                    || object.deleted
                    || object.page != self.active_page
                    || !is_point_kind(&object.kind)
            }) {
                return Err("Select one origin point".into());
            }
        } else if !default_origin.x.is_finite() || !default_origin.y.is_finite() {
            return Err("Coordinate-system origin must be finite".into());
        }
        self.checkpoint();
        let origin = origin.unwrap_or_else(|| self.append_object(Kind::Free(default_origin)));
        let x_axis = self.append_object(Kind::DistanceAxis(origin, distances[0], 1));
        let (y_axis, form) = if distances.len() == 1 {
            let unit =
                self.append_object(Kind::DistanceVerticalUnit(x_axis, 37.79527559055118, false));
            (
                self.append_object(Kind::GraphAxis(origin, unit, 0)),
                GridForm::Square,
            )
        } else {
            (
                self.append_object(Kind::DistanceAxis(origin, distances[1], 0)),
                GridForm::Rectangular,
            )
        };
        let frame = self.append_object(Kind::CoordinateSystem(x_axis, y_axis));
        self.marked_coordinate_system[self.active_page] = Some(frame);
        self.frame_styles.insert(frame, (form, false));
        self.grid_form[self.active_page] = form;
        self.grid_dotted[self.active_page] = false;
        Ok(frame)
    }
    pub fn add_function_definition(&mut self, expression: SimpleFunction) -> Result<usize, String> {
        self.add_function_definition_with_angle_unit(expression, true)
    }
    fn function_expression(&self, id: usize) -> Option<SimpleFunction> {
        match self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::FunctionDefinition(expression) | Kind::DerivativeFunction(_, expression) => {
                Some(expression)
            }
            _ => None,
        }
    }
    fn function_symbol(&self, id: usize) -> Option<String> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        match object.kind {
            Kind::FunctionDefinition(_) => Some(
                object
                    .function_name
                    .as_deref()
                    .filter(|name| !name.is_empty())
                    .unwrap_or("f")
                    .to_owned(),
            ),
            Kind::DerivativeFunction(source, _) => {
                Some(format!("{}′", self.function_symbol(source)?))
            }
            _ => None,
        }
    }
    pub fn function_display_text(&self, id: usize) -> Option<String> {
        let expression = self.function_expression(id)?;
        let symbol = self.function_symbol(id)?;
        Some(
            expression
                .display()
                .replacen("f(x)", &format!("{symbol}(x)"), 1),
        )
    }
    pub fn add_derivative_function(&mut self, source: usize) -> Result<usize, String> {
        self.add_derivative_function_with_angle_unit(source, self.angle_unit_radians)
    }
    pub fn add_derivative_function_with_angle_unit(
        &mut self,
        source: usize,
        radians: bool,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot define a derivative in this sketch".into());
        }
        let object = self
            .objects
            .get(source.saturating_sub(1))
            .ok_or("Select one function")?;
        if object.id != source || object.deleted || object.page != self.active_page {
            return Err("Select one function".into());
        }
        let expression = self
            .function_expression(source)
            .ok_or("Select one function")?;
        let derivative = expression.symbolic_derivative()?;
        self.checkpoint();
        if expression.uses_trigonometry() {
            self.angle_unit_radians = radians;
        }
        Ok(self.append_object(Kind::DerivativeFunction(source, derivative)))
    }
    pub fn add_function_definition_with_angle_unit(
        &mut self,
        expression: SimpleFunction,
        radians: bool,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot add a function to this sketch".into());
        }
        self.checkpoint();
        if expression.uses_trigonometry() {
            self.angle_unit_radians = radians;
        }
        Ok(self.append_object(Kind::FunctionDefinition(expression)))
    }
    pub fn set_function_expression(
        &mut self,
        definition: usize,
        expression: SimpleFunction,
    ) -> Result<(), String> {
        self.set_function_expression_with_angle_unit(
            definition,
            expression,
            self.angle_unit_radians,
        )
    }
    pub fn set_function_expression_with_angle_unit(
        &mut self,
        definition: usize,
        expression: SimpleFunction,
        radians: bool,
    ) -> Result<(), String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot edit a function in this sketch".into());
        }
        let object = self
            .objects
            .get(definition.saturating_sub(1))
            .ok_or("Select one function definition")?;
        if object.id != definition
            || object.page != self.active_page
            || object.deleted
            || !matches!(object.kind, Kind::FunctionDefinition(_))
        {
            return Err("Select one function definition".into());
        }
        if object.kind == Kind::FunctionDefinition(expression)
            && (!expression.uses_trigonometry() || self.angle_unit_radians == radians)
        {
            return Ok(());
        }
        let mut updates = BTreeMap::new();
        updates.insert(definition, expression);
        for object in &self.objects {
            if let Kind::DerivativeFunction(source, _) = object.kind
                && let Some(source_expression) = updates.get(&source).copied()
            {
                updates.insert(object.id, source_expression.symbolic_derivative()?);
            }
        }
        self.checkpoint();
        self.objects[definition - 1].kind = Kind::FunctionDefinition(expression);
        for (id, expression) in updates {
            if id != definition
                && let Kind::DerivativeFunction(source, _) = self.objects[id - 1].kind
            {
                self.objects[id - 1].kind = Kind::DerivativeFunction(source, expression);
            }
        }
        if expression.uses_trigonometry() {
            self.angle_unit_radians = radians;
        }
        Ok(())
    }
    pub fn plot_new_function(
        &mut self,
        expression: SimpleFunction,
        origin: Point,
    ) -> Result<usize, String> {
        self.plot_new_function_with_angle_unit(expression, origin, true)
    }
    pub fn plot_new_function_with_angle_unit(
        &mut self,
        expression: SimpleFunction,
        origin: Point,
        radians: bool,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot plot a function in this sketch".into());
        }
        let frame = match self.active_coordinate_system() {
            Some(id) => id,
            None => self.add_default_coordinate_system(origin)?,
        };
        self.checkpoint();
        if expression.uses_trigonometry() {
            self.angle_unit_radians = radians;
        }
        let definition = self.append_object(Kind::FunctionDefinition(expression));
        Ok(self.append_object(Kind::FunctionPlot(
            definition,
            frame,
            -5.265208333333335,
            5.265208333333335,
            750,
        )))
    }
    pub fn plot_function(&mut self, definition: usize, origin: Point) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot plot a function in this sketch".into());
        }
        if !self
            .objects
            .get(definition.saturating_sub(1))
            .is_some_and(|object| {
                object.id == definition
                    && object.page == self.active_page
                    && !object.deleted
                    && matches!(
                        object.kind,
                        Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _)
                    )
            })
        {
            return Err("Select one function definition".into());
        }
        let frame = match self.active_coordinate_system() {
            Some(id) => id,
            None => self.add_default_coordinate_system(origin)?,
        };
        self.checkpoint();
        Ok(self.append_object(Kind::FunctionPlot(
            definition,
            frame,
            -5.265208333333335,
            5.265208333333335,
            750,
        )))
    }
    pub fn set_plot_range(
        &mut self,
        plot: usize,
        min: f64,
        max: f64,
        samples: u32,
    ) -> Result<(), String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot edit a plot in this sketch".into());
        }
        if !min.is_finite() || !max.is_finite() || min >= max || !(2..=4096).contains(&samples) {
            return Err("Invalid plot range or sample count".into());
        }
        let object = self
            .objects
            .get(plot.saturating_sub(1))
            .ok_or("Select one plot")?;
        if object.id != plot || object.page != self.active_page || object.deleted {
            return Err("Select one plot".into());
        }
        let updated = match object.kind {
            Kind::FunctionPlot(definition, frame, _, _, _) => {
                Kind::FunctionPlot(definition, frame, min, max, samples)
            }
            Kind::ParametricPlot(x, y, frame, _, _, _, mode) => {
                Kind::ParametricPlot(x, y, frame, min, max, samples, mode)
            }
            _ => return Err("Select one function or parametric plot".into()),
        };
        if object.kind == updated {
            return Ok(());
        }
        self.checkpoint();
        self.objects[plot - 1].kind = updated;
        Ok(())
    }
    pub fn function_plot_points(&self, id: usize, count: usize) -> Option<Vec<Point>> {
        let Kind::FunctionPlot(definition, frame, min, max, samples) =
            self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let expression = self.function_expression(definition)?;
        let frame = self.graph_frame(frame)?;
        let count = count.min(samples as usize).clamp(2, 4096);
        (0..count)
            .map(|i| {
                let x = min + (max - min) * (i as f64 / (count - 1) as f64);
                let p = frame.world(
                    x,
                    expression.evaluate_with_angle_unit(x, self.angle_unit_radians),
                )?;
                Some(Point { x: p.x, y: p.y })
            })
            .collect()
    }
    pub fn function_plot_segments(&self, id: usize, count: usize) -> Option<Vec<(Point, Point)>> {
        let Kind::FunctionPlot(definition, frame, min, max, samples) =
            self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let expression = self.function_expression(definition)?;
        let frame = self.graph_frame(frame)?;
        let count = count.min(samples as usize).clamp(2, 4096);
        let sample = |index: usize| {
            let x = min + (max - min) * (index as f64 / (count - 1) as f64);
            (
                x,
                expression.evaluate_with_angle_unit(x, self.angle_unit_radians),
            )
        };
        let mut segments = Vec::with_capacity(count - 1);
        let mut left = sample(0);
        for index in 1..count {
            let right = sample(index);
            let (x0, y0) = left;
            let (x1, y1) = right;
            left = right;
            if !y0.is_finite() || !y1.is_finite() {
                continue;
            }
            let low = y0.min(y1);
            let high = y0.max(y1);
            let tolerance = 1e-3 * high.abs().max(low.abs()).max(1.0);
            let crosses_gap = expression.may_have_poles()
                && [0.25, 0.5, 0.75].into_iter().any(|fraction| {
                    let x = x0 + (x1 - x0) * fraction;
                    let y = expression.evaluate_with_angle_unit(x, self.angle_unit_radians);
                    !y.is_finite() || y < low - tolerance || y > high + tolerance
                });
            if crosses_gap {
                continue;
            }
            if let (Some(a), Some(b)) = (frame.world(x0, y0), frame.world(x1, y1)) {
                segments.push((Point { x: a.x, y: a.y }, Point { x: b.x, y: b.y }));
            }
        }
        Some(segments)
    }
    pub fn plot_parametric_curve(
        &mut self,
        x_definition: usize,
        y_definition: usize,
        mode: PlotPointsMode,
        min: f64,
        max: f64,
        origin: Point,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || x_definition == y_definition {
            return Err("Select two distinct functions".into());
        }
        for id in [x_definition, y_definition] {
            if !self
                .objects
                .get(id.saturating_sub(1))
                .is_some_and(|object| {
                    object.id == id
                        && object.page == self.active_page
                        && !object.deleted
                        && matches!(
                            object.kind,
                            Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _)
                        )
                })
            {
                return Err("Select two distinct functions".into());
            }
        }
        if !min.is_finite() || !max.is_finite() || min >= max {
            return Err("Invalid parametric plot domain".into());
        }
        let existing_frame = self.active_coordinate_system();
        let frame = match existing_frame {
            Some(id) => id,
            None => self.add_default_coordinate_system(origin)?,
        };
        if existing_frame.is_none() && mode == PlotPointsMode::Polar {
            self.frame_styles.insert(frame, (GridForm::Polar, false));
            self.grid_form[self.active_page] = GridForm::Polar;
        }
        self.checkpoint();
        Ok(self.append_object(Kind::ParametricPlot(
            x_definition,
            y_definition,
            frame,
            min,
            max,
            750,
            mode,
        )))
    }
    pub fn parametric_plot_segments(&self, id: usize, count: usize) -> Option<Vec<(Point, Point)>> {
        let Kind::ParametricPlot(x_id, y_id, frame_id, min, max, samples, mode) =
            self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let x_function = self.function_expression(x_id)?;
        let y_function = self.function_expression(y_id)?;
        let frame = self.graph_frame(frame_id)?;
        let sample = |t: f64| {
            let x = x_function.evaluate_with_angle_unit(t, self.angle_unit_radians);
            let y = y_function.evaluate_with_angle_unit(t, self.angle_unit_radians);
            match mode {
                PlotPointsMode::Cartesian => (x, y),
                PlotPointsMode::Polar => (x * y.cos(), x * y.sin()),
            }
        };
        let count = count.min(samples as usize).clamp(2, 4096);
        let mut segments = Vec::with_capacity(count - 1);
        let mut left = (min, sample(min));
        for index in 1..count {
            let t = min + (max - min) * (index as f64 / (count - 1) as f64);
            let right = (t, sample(t));
            let ((t0, (x0, y0)), (t1, (x1, y1))) = (left, right);
            left = right;
            if ![x0, y0, x1, y1].into_iter().all(f64::is_finite) {
                continue;
            }
            let tolerance = 1e-3
                * [x0, y0, x1, y1]
                    .into_iter()
                    .map(f64::abs)
                    .fold(1.0, f64::max);
            let crosses_gap = (x_function.may_have_poles() || y_function.may_have_poles())
                && [0.25, 0.5, 0.75].into_iter().any(|fraction| {
                    let (x, y) = sample(t0 + (t1 - t0) * fraction);
                    !x.is_finite()
                        || !y.is_finite()
                        || x < x0.min(x1) - tolerance
                        || x > x0.max(x1) + tolerance
                        || y < y0.min(y1) - tolerance
                        || y > y0.max(y1) + tolerance
                });
            if crosses_gap {
                continue;
            }
            if let (Some(a), Some(b)) = (frame.world(x0, y0), frame.world(x1, y1)) {
                segments.push((Point { x: a.x, y: a.y }, Point { x: b.x, y: b.y }));
            }
        }
        Some(segments)
    }
    pub fn plot_default_parameter(&mut self, mode: PlotPointsMode) -> Result<usize, String> {
        self.plot_points_at(mode, [1.0, 1.0])
    }
    pub fn plot_points_at(
        &mut self,
        mode: PlotPointsMode,
        coordinates: [f64; 2],
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Plot Points requires a supported coordinate frame".into());
        }
        let frame = self
            .active_coordinate_system()
            .ok_or("Define a coordinate system before plotting points")?;
        let source = self
            .objects
            .first()
            .filter(|object| {
                object.page == self.active_page && !object.deleted && object.id != frame
            })
            .ok_or("Plot Points requires a source object on this page")?
            .id;
        if coordinates.iter().any(|value| !value.is_finite()) {
            return Err("Plot coordinates must be finite".into());
        }
        let stored = match mode {
            PlotPointsMode::Cartesian => coordinates,
            PlotPointsMode::Polar => [
                coordinates[0],
                coordinates[1] * plot_points_records::DEGREES_TO_RADIANS,
            ],
        };
        let [x, y] = match mode {
            PlotPointsMode::Cartesian => stored,
            PlotPointsMode::Polar => [stored[0] * stored[1].cos(), stored[0] * stored[1].sin()],
        };
        self.graph_frame(frame)
            .and_then(|frame| frame.world(x, y))
            .ok_or("Plot coordinates are outside the supported range")?;
        let group = plot_points_records::encode_group(mode, [source as u32, frame as u32], stored)?;
        let plot = plot_points_records::decode_group(&group)?;
        let style = style_records::visual_style(&group[8..36]);
        self.checkpoint();
        let id = self.append_object(Kind::PlottedPoint(source, frame, mode, plot.stored));
        self.objects[id - 1].visual_style = style;
        Ok(id)
    }
    pub fn plot_default_pair_table(&mut self, table: usize) -> Result<Vec<usize>, String> {
        self.plot_pair_table_data(table, PlotPointsMode::Cartesian)
    }
    pub fn plot_pair_table_data(
        &mut self,
        table: usize,
        mode: PlotPointsMode,
    ) -> Result<Vec<usize>, String> {
        if self.partial_view || self.page_count != 1 {
            return Err(
                "Plot Table Data requires the verified t[2], t[1] table and coordinate system"
                    .into(),
            );
        }
        let frame = self
            .active_coordinate_system()
            .ok_or("Define a coordinate system before plotting table data")?;
        let source = self
            .objects
            .first()
            .filter(|object| !object.deleted && object.page == self.active_page)
            .ok_or("Plot Table Data requires a source object")?
            .id;
        let Some(object) = table
            .checked_sub(1)
            .and_then(|index| self.objects.get(index))
        else {
            return Err("Select the verified pair table".into());
        };
        let Kind::ValueTablePair(first, second, table_group) = &object.kind else {
            return Err("Select the verified pair table".into());
        };
        if object.deleted
            || object.page != self.active_page
            || !(1..=256).contains(&table_group.rows())
            || !matches!(self.objects.get(first - 1).map(|o| &o.kind), Some(Kind::Parameter(group)) if group.name() == "t[2]")
            || !matches!(self.objects.get(second - 1).map(|o| &o.kind), Some(Kind::Parameter(group)) if group.name() == "t[1]")
        {
            return Err("Select the supported t[2], t[1] table".into());
        }
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
            let stored = match mode {
                PlotPointsMode::Cartesian => stored,
                PlotPointsMode::Polar => [
                    stored[0],
                    stored[1] * plot_points_records::DEGREES_TO_RADIANS,
                ],
            };
            let group =
                plot_points_records::encode_group(mode, [source as u32, frame as u32], stored)?;
            let style = style_records::visual_style(&group[8..36]);
            let id = self.append_object(Kind::PlottedPoint(source, frame, mode, stored));
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
        self.grid_enabled()
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
        if matches!(object.kind, Kind::DistanceVerticalUnit(_, _, false)) {
            return false;
        }
        if matches!(object.kind, Kind::GraphVerticalUnit(_, _)) {
            let form = self
                .coordinate_system_for_object(id)
                .and_then(|frame| self.frame_styles.get(&frame))
                .map_or(self.grid_form[object.page], |style| style.0);
            if form != GridForm::Rectangular {
                return false;
            }
        }
        !self.objects.iter().any(|frame| {
            let Kind::CoordinateSystem(x_axis, y_axis) = frame.kind else {
                return false;
            };
            if frame.visible || frame.deleted || frame.page != object.page {
                return false;
            }
            if let (Some(Kind::CircleAxis(circle, 1)), Some(Kind::CircleAxis(other, 0))) = (
                x_axis
                    .checked_sub(1)
                    .and_then(|at| self.objects.get(at))
                    .map(|o| &o.kind),
                y_axis
                    .checked_sub(1)
                    .and_then(|at| self.objects.get(at))
                    .map(|o| &o.kind),
            ) && circle == other
            {
                return [frame.id, x_axis, y_axis].contains(&id);
            }
            if let Some(Kind::DistanceAxis(origin, _, 1)) = x_axis
                .checked_sub(1)
                .and_then(|at| self.objects.get(at))
                .map(|o| &o.kind)
            {
                let unit = match y_axis
                    .checked_sub(1)
                    .and_then(|at| self.objects.get(at))
                    .map(|o| &o.kind)
                {
                    Some(Kind::GraphAxis(_, unit, 0)) => Some(*unit),
                    _ => None,
                };
                return [
                    Some(frame.id),
                    Some(x_axis),
                    Some(y_axis),
                    Some(*origin),
                    unit,
                ]
                .contains(&Some(id));
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
        if let Some(frame) = self.active_coordinate_system()
            && self.grid_dotted[self.active_page] != dotted
        {
            self.checkpoint();
            self.grid_dotted[self.active_page] = dotted;
            self.frame_styles
                .insert(frame, (self.grid_form[self.active_page], dotted));
        }
    }
    pub fn set_grid_form(&mut self, form: GridForm) -> Result<(), String> {
        let frame = self
            .active_coordinate_system()
            .ok_or("No coordinate system")?;
        if self.grid_form[self.active_page] == form {
            return Ok(());
        }
        let Kind::CoordinateSystem(x_axis, y_axis) = self.objects[frame - 1].kind else {
            return Err("Invalid coordinate system".into());
        };
        if matches!(
            self.objects.get(x_axis - 1).map(|object| &object.kind),
            Some(Kind::CircleAxis(_, 1))
        ) {
            if form == GridForm::Rectangular {
                return Err("A unit circle cannot define a rectangular grid".into());
            }
            self.checkpoint();
            self.grid_form[self.active_page] = form;
            self.frame_styles
                .insert(frame, (form, self.grid_dotted[self.active_page]));
            return Ok(());
        }
        if let Some(Kind::DistanceAxis(_, distance, 1)) =
            self.objects.get(x_axis - 1).map(|object| &object.kind)
        {
            let unit = match self.objects.get(y_axis - 1).map(|object| &object.kind) {
                Some(Kind::DistanceAxis(_, _, 0)) if form != GridForm::Square => None,
                Some(Kind::GraphAxis(_, unit, 0))
                    if matches!(
                        self.objects.get(unit.saturating_sub(1)).map(|object| &object.kind),
                        Some(Kind::DistanceVerticalUnit(host, _, _)) if *host == x_axis
                    ) =>
                {
                    Some(*unit)
                }
                _ => return Err("This defining-distance frame cannot use that grid form".into()),
            };
            let unit_change = unit.and_then(|unit| {
                let Kind::DistanceVerticalUnit(_, step, independent) = self.objects[unit - 1].kind
                else {
                    return None;
                };
                let next = match form {
                    GridForm::Square => (step, false),
                    GridForm::Rectangular if !independent => {
                        (self.defining_distance_pixels(*distance)?, true)
                    }
                    GridForm::Rectangular | GridForm::Polar => (step, independent),
                };
                Some((unit, next))
            });
            if unit.is_some() && unit_change.is_none() {
                return Err("Invalid defining-distance unit".into());
            }
            self.checkpoint();
            if let Some((unit, (step, independent))) = unit_change {
                self.objects[unit - 1].kind = Kind::DistanceVerticalUnit(x_axis, step, independent);
            }
            self.grid_form[self.active_page] = form;
            self.frame_styles
                .insert(frame, (form, self.grid_dotted[self.active_page]));
            return Ok(());
        }
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
        self.frame_styles
            .insert(frame, (form, self.grid_dotted[self.active_page]));
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
        let map = |p: Point| GraphPoint { x: p.x, y: p.y };
        match (
            self.objects.get(x_axis.checked_sub(1)?)?.kind.clone(),
            self.objects.get(y_axis.checked_sub(1)?)?.kind.clone(),
        ) {
            (
                Kind::GraphAxis(origin, x_unit, 1 | 0x1_0001),
                Kind::GraphAxis(y_origin, y_unit, 0),
            ) if origin == y_origin => Some(CoordinateFrame {
                origin: map(self.position(origin)?),
                x_unit: map(self.position(x_unit)?),
                y_unit: map(self.position(y_unit)?),
            }),
            (Kind::CircleAxis(circle, 1), Kind::CircleAxis(other, 0)) if circle == other => {
                let (origin, radius) = self.circle_points(circle)?;
                (radius.is_finite() && radius > 0.0).then_some(CoordinateFrame {
                    origin: map(origin),
                    x_unit: GraphPoint {
                        x: origin.x + radius,
                        y: origin.y,
                    },
                    y_unit: GraphPoint {
                        x: origin.x,
                        y: origin.y - radius,
                    },
                })
            }
            (Kind::DistanceAxis(origin, first, 1), Kind::DistanceAxis(other, second, 0))
                if origin == other =>
            {
                let origin = self.position(origin)?;
                let x = self.defining_distance_pixels(first)?;
                let y = self.defining_distance_pixels(second)?;
                Some(CoordinateFrame {
                    origin: map(origin),
                    x_unit: GraphPoint {
                        x: origin.x + x,
                        y: origin.y,
                    },
                    y_unit: GraphPoint {
                        x: origin.x,
                        y: origin.y - y,
                    },
                })
            }
            (Kind::DistanceAxis(origin, _, 1), Kind::GraphAxis(other, unit, 0))
                if origin == other
                    && matches!(
                        self.objects.get(unit.checked_sub(1)?)?.kind,
                        Kind::DistanceVerticalUnit(host, _, _) if host == x_axis
                    ) =>
            {
                let origin = self.position(origin)?;
                let x = self.line_points(x_axis)?.1;
                Some(CoordinateFrame {
                    origin: map(origin),
                    x_unit: map(x),
                    y_unit: map(self.position(unit)?),
                })
            }
            _ => None,
        }
    }
    fn defining_distance_pixels(&self, id: usize) -> Option<f64> {
        let kind = &self.objects.get(id.checked_sub(1)?)?.kind;
        let value = match *kind {
            Kind::Segment(a, b) => self.position(a)?.distance(self.position(b)?),
            Kind::SegmentLength(_) => self.segment_length_cm(id)? * 37.79527559055118,
            Kind::PointDistance(_, _) => self.point_distance_cm(id)? * 37.79527559055118,
            _ => return None,
        };
        (value.is_finite() && value > 0.).then_some(value)
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
        for object in self
            .objects
            .iter()
            .filter(|object| !object.deleted && object.page == self.active_page)
        {
            match object.kind {
                Kind::FunctionPlot(definition, _, _, _, _)
                    if self.selected.contains(&definition) =>
                {
                    pending.push(object.id)
                }
                Kind::ParametricPlot(x, y, _, _, _, _, _)
                    if self.selected.contains(&x) || self.selected.contains(&y) =>
                {
                    pending.push(object.id)
                }
                _ => {}
            }
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
        let mut hide_on_paste = BTreeSet::new();
        let rename_on_paste = self.objects.iter().find_map(|object| {
            let Kind::ParametricPlot(x, y, _, _, _, _, _) = object.kind else {
                return None;
            };
            if !included.contains(&object.id)
                || self.selected.contains(&x) == self.selected.contains(&y)
            {
                return None;
            }
            Some(if self.selected.contains(&x) { x } else { y })
        });
        for object in self
            .objects
            .iter()
            .filter(|object| included.contains(&object.id))
        {
            let frame = match object.kind {
                Kind::FunctionPlot(_, frame, _, _, _)
                | Kind::ParametricPlot(_, _, frame, _, _, _, _) => frame,
                _ => continue,
            };
            if self.selected.contains(&frame) {
                continue;
            }
            let mut stack = vec![frame];
            while let Some(id) = stack.pop() {
                let kind = &self.objects[id - 1].kind;
                if !matches!(
                    kind,
                    Kind::GraphHorizontalUnit(_, _)
                        | Kind::GraphVerticalUnit(_, _)
                        | Kind::GraphAxis(_, _, _)
                        | Kind::CircleAxis(_, _)
                        | Kind::DistanceAxis(_, _, _)
                        | Kind::DistanceVerticalUnit(_, _, _)
                        | Kind::CoordinateSystem(_, _)
                ) || !hide_on_paste.insert(id)
                {
                    continue;
                }
                stack.extend(kind.dependencies());
            }
        }
        Ok(Clipboard {
            hide_on_paste,
            rename_on_paste,
            objects: included
                .into_iter()
                .map(|id| -> Result<_, String> {
                    let object = &self.objects[id - 1];
                    let function_anchor = match object.kind {
                        Kind::FunctionDefinition(expression) => {
                            Some(if let Some(anchor) = object.function_anchor {
                                anchor
                            } else {
                                function_records::group_expression_anchor(
                                    &function_records::encode_definition_group(expression)?,
                                )?
                            })
                        }
                        Kind::DerivativeFunction(source, expression) => {
                            Some(if let Some(anchor) = object.function_anchor {
                                anchor
                            } else {
                                function_records::group_expression_anchor(
                                    &function_records::encode_derivative_group(source, expression)?,
                                )?
                            })
                        }
                        _ => None,
                    };
                    Ok((
                        id,
                        object.kind.clone(),
                        object.visible,
                        object.tracing,
                        object.midpoint_host,
                        object.visual_style,
                        self.text_font_state(id),
                        object.label.clone(),
                        self.text_rich_template(id),
                        self.frame_styles.get(&id).copied(),
                        function_anchor,
                        matches!(object.kind, Kind::FunctionDefinition(_))
                            .then(|| object.function_name.clone().unwrap_or_else(|| "f".into())),
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?,
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
        for (
            old_id,
            kind,
            visible,
            tracing,
            midpoint_host,
            visual_style,
            text_style,
            label,
            rich_text,
            frame_style,
            function_anchor,
            function_name,
        ) in &clipboard.objects
        {
            if kind
                .dependencies()
                .into_iter()
                .chain(*midpoint_host)
                .any(|id| id >= ids.len() || ids[id] == 0)
            {
                return Err("Clipboard has an unresolved dependency".into());
            }
            let mut remapped = kind.remap_checked(&ids)?;
            if let Kind::Free(ref mut point) = remapped {
                point.x += 20.;
                point.y += 20.;
            }
            if let Kind::PlainText { ref mut rect, .. } = remapped {
                rect.left = rect.left.checked_add(20).ok_or("Pasted text is off page")?;
                rect.top = rect.top.checked_add(20).ok_or("Pasted text is off page")?;
                rect.right = rect
                    .right
                    .checked_add(20)
                    .ok_or("Pasted text is off page")?;
                rect.bottom = rect
                    .bottom
                    .checked_add(20)
                    .ok_or("Pasted text is off page")?;
            }
            let shifted_anchor = |(x, y): (i16, i16)| -> Result<(i16, i16), String> {
                Ok((
                    x.checked_add(20).ok_or("Pasted number is off page")?,
                    y.checked_add(20).ok_or("Pasted number is off page")?,
                ))
            };
            match &mut remapped {
                Kind::Parameter(group) => group.set_anchor(shifted_anchor(group.anchor())?)?,
                Kind::Calculation(group) => group.set_anchor(shifted_anchor(group.anchor())?)?,
                Kind::ValueTable(_, group) | Kind::ValueTablePair(_, _, group) => {
                    group.set_anchor(shifted_anchor(group.anchor())?);
                }
                _ => {}
            }
            let function_anchor = (*function_anchor).map(shifted_anchor).transpose()?;
            let mut function_name = function_name.clone();
            if clipboard.rename_on_paste == Some(*old_id) {
                let used = candidate
                    .objects
                    .iter()
                    .filter(|object| matches!(object.kind, Kind::FunctionDefinition(_)))
                    .map(|object| object.function_name.as_deref().unwrap_or("f"))
                    .collect::<BTreeSet<_>>();
                let next = (b'g'..=b'z')
                    .map(|letter| (letter as char).to_string())
                    .find(|name| !used.contains(name.as_str()))
                    .ok_or("No available function name")?;
                function_name = Some(next);
            }
            let id = candidate.objects.len() + 1;
            candidate.objects.push(Object {
                id,
                page: candidate.active_page,
                kind: remapped,
                visible: *visible && !clipboard.hide_on_paste.contains(old_id),
                tracing: *tracing,
                deleted: false,
                midpoint_host: midpoint_host.map(|host| ids[host]),
                point_offset: None,
                constraint_offset: None,
                header_offset: None,
                label: label.clone(),
                visual_style: *visual_style,
                text_style_override: text_style.clone(),
                rich_text_override: rich_text.clone(),
                function_anchor,
                function_name,
            });
            if let Some(style) = frame_style {
                candidate.frame_styles.insert(id, *style);
            }
            ids[*old_id] = id;
            created.push(id);
        }
        candidate.selected.extend(created.iter().copied());
        let encoded = candidate.encode()?;
        let reparsed = Self::from_bytes(&encoded, None)?;
        candidate.validate_reparsed(&reparsed)?;
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
        let mut candidate = self.clone();
        candidate.checkpoint();
        for object in &mut candidate.objects {
            if removed.contains(&object.id) {
                object.deleted = true;
            }
        }
        if !candidate.objects.iter().any(|object| {
            object.page == candidate.active_page
                && !object.deleted
                && matches!(object.kind, Kind::CoordinateSystem(_, _))
        }) {
            candidate.grid_dotted[candidate.active_page] = false;
            candidate.snap_points[candidate.active_page] = false;
            candidate.grid_form[candidate.active_page] = GridForm::Square;
        }
        candidate.selected.clear();
        let encoded = candidate.encode()?;
        let reparsed = Self::from_bytes(&encoded, None)?;
        candidate.validate_reparsed(&reparsed)?;
        *self = candidate;
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
        if self.partial_view && self.page_count == 1 {
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
                        && (is_point_kind(&o.kind)
                            || self.line_points(id).is_some()
                            || is_circle_kind(&o.kind)
                            || is_arc_kind(&o.kind)
                            || o.kind.is_polygon())
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
        if self.partial_view && self.page_count == 1 {
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
        self.add_hide_show_button_for_targets(&[target])
    }
    pub fn add_hide_show_button_for_targets(&mut self, targets: &[usize]) -> Result<usize, String> {
        if targets.is_empty() || self.partial_view || self.page_count != 1 {
            return Err("A Hide/Show button needs visible supported objects on this page".into());
        }
        for (index, &target) in targets.iter().enumerate() {
            let object = self
                .objects
                .get(target.checked_sub(1).ok_or("Invalid button target")?)
                .ok_or("Invalid button target")?;
            if targets[..index].contains(&target)
                || object.deleted
                || !object.visible
                || object.page != self.active_page
                || matches!(
                    object.kind,
                    Kind::HideShowButton { .. }
                        | Kind::AnimationButton { .. }
                        | Kind::MovementButton { .. }
                        | Kind::ScrollButton { .. }
                        | Kind::PresentationButton { .. }
                        | Kind::Unsupported(_)
                )
            {
                return Err(
                    "A Hide/Show button needs distinct visible supported objects on this page"
                        .into(),
                );
            }
        }
        let row = self
            .objects
            .iter()
            .filter(|object| {
                object.page == self.active_page
                    && !object.deleted
                    && matches!(
                        object.kind,
                        Kind::HideShowButton { .. }
                            | Kind::AnimationButton { .. }
                            | Kind::MovementButton { .. }
                            | Kind::ScrollButton { .. }
                            | Kind::PresentationButton { .. }
                    )
            })
            .count();
        let y = (5usize + row * 30).min(i16::MAX as usize) as i16;
        let label = if targets.len() == 1
            && matches!(self.objects[targets[0] - 1].kind, Kind::Segment(_, _))
        {
            "Hide Segment"
        } else {
            "Hide Objects"
        };
        Ok(self.add(Kind::HideShowButton {
            targets: targets.to_vec(),
            action: VisibilityAction::Hide,
            label: label.into(),
            anchor: (10, y),
        }))
    }
    pub fn add_animation_button(&mut self, target: usize) -> Result<usize, String> {
        let settings = self
            .animation_button_default_settings(target)
            .ok_or("An Animation button needs an animatable point on this page")?;
        self.add_animation_button_with_settings(target, settings)
    }
    pub fn add_movement_button(
        &mut self,
        source: usize,
        destination: usize,
        speed: MovementSpeed,
        follow: bool,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 || source == destination {
            return Err("A Movement button needs two distinct points on this page".into());
        }
        let moving = self
            .objects
            .get(source.checked_sub(1).ok_or("Invalid source point")?)
            .ok_or("Invalid source point")?;
        let target = self
            .objects
            .get(
                destination
                    .checked_sub(1)
                    .ok_or("Invalid destination point")?,
            )
            .ok_or("Invalid destination point")?;
        if moving.deleted
            || !moving.visible
            || moving.page != self.active_page
            || !matches!(moving.kind, Kind::Free(_))
            || target.deleted
            || !target.visible
            || target.page != self.active_page
            || self.position(destination).is_none()
        {
            return Err(
                "A Movement button needs a free source point and a destination point".into(),
            );
        }
        let row = self
            .objects
            .iter()
            .filter(|object| {
                object.page == self.active_page
                    && !object.deleted
                    && matches!(
                        object.kind,
                        Kind::HideShowButton { .. }
                            | Kind::AnimationButton { .. }
                            | Kind::MovementButton { .. }
                            | Kind::ScrollButton { .. }
                            | Kind::PresentationButton { .. }
                    )
            })
            .count();
        let y = (5usize + row * 30).min(i16::MAX as usize) as i16;
        Ok(self.add(Kind::MovementButton {
            source,
            destination,
            speed,
            follow,
            label: "Move to Point".into(),
            anchor: (10, y),
        }))
    }
    pub fn add_scroll_button(
        &mut self,
        target: usize,
        placement: ScrollPlacement,
    ) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("A Scroll button needs a point on this page".into());
        }
        let object = self
            .objects
            .get(target.checked_sub(1).ok_or("Invalid Scroll target")?)
            .ok_or("Invalid Scroll target")?;
        if object.deleted
            || !object.visible
            || object.page != self.active_page
            || !is_point_kind(&object.kind)
            || self.position(target).is_none()
        {
            return Err("A Scroll button needs a visible point on this page".into());
        }
        let row = self
            .objects
            .iter()
            .filter(|object| {
                object.page == self.active_page
                    && !object.deleted
                    && matches!(
                        object.kind,
                        Kind::HideShowButton { .. }
                            | Kind::AnimationButton { .. }
                            | Kind::MovementButton { .. }
                            | Kind::ScrollButton { .. }
                            | Kind::PresentationButton { .. }
                    )
            })
            .count();
        let y = (5usize + row * 30).min(i16::MAX as usize) as i16;
        Ok(self.add(Kind::ScrollButton {
            target,
            placement,
            label: "Scroll".into(),
            anchor: (10, y),
        }))
    }
    pub fn add_presentation_button_for_targets(
        &mut self,
        targets: &[usize],
        mode: PresentationMode,
    ) -> Result<usize, String> {
        if targets.is_empty() || self.partial_view || self.page_count != 1 {
            return Err("A Presentation button needs action buttons on this page".into());
        }
        for (index, &target) in targets.iter().enumerate() {
            let object = self
                .objects
                .get(target.checked_sub(1).ok_or("Invalid button target")?)
                .ok_or("Invalid button target")?;
            if targets[..index].contains(&target)
                || object.deleted
                || !object.visible
                || object.page != self.active_page
                || !object.kind.is_action_button()
            {
                return Err("A Presentation button needs distinct visible action buttons".into());
            }
        }
        let row = self
            .objects
            .iter()
            .filter(|object| {
                object.page == self.active_page && !object.deleted && object.kind.is_action_button()
            })
            .count();
        let y = (5usize + row * 30).min(i16::MAX as usize) as i16;
        Ok(self.add(Kind::PresentationButton {
            targets: targets.to_vec(),
            mode,
            label: "Run Presentation".into(),
            anchor: (10, y),
        }))
    }
    pub fn add_link_url_button(&mut self, url: &str) -> Result<usize, String> {
        let multi_page_append = self.page_count > 1
            && !self
                .objects
                .iter()
                .any(|object| matches!(object.kind, Kind::Unsupported(_)))
            && self
                .objects
                .iter()
                .skip(self.source_count)
                .all(|object| is_multi_page_addition_kind(&object.kind));
        if !multi_page_append && (self.partial_view || self.page_count != 1) {
            return Err("A Link URL button needs an editable page".into());
        }
        if url.is_empty() || url.len() > u16::MAX as usize || url.as_bytes().contains(&0) {
            return Err("Link URL must be nonempty text of at most 65535 bytes".into());
        }
        let row = self
            .objects
            .iter()
            .filter(|object| {
                object.page == self.active_page && !object.deleted && object.kind.is_action_button()
            })
            .count();
        let y = (5usize + row * 30).min(i16::MAX as usize) as i16;
        Ok(self.add(Kind::LinkUrlButton {
            url: url.into(),
            label: url.into(),
            anchor: (10, y),
        }))
    }
    pub fn has_pending_multi_page_additions(&self) -> bool {
        self.page_count > 1
            && self.objects.len() > self.source_count
            && self.objects[self.source_count..]
                .iter()
                .all(|object| is_multi_page_addition_kind(&object.kind))
    }
    pub fn add_link_page_button(&mut self, target_page: usize) -> Result<usize, String> {
        self.add_link_page_button_with_target(target_page, None)
    }
    pub fn link_destination_buttons(&self, page: usize) -> Vec<usize> {
        self.objects
            .iter()
            .filter(|object| {
                object.page == page
                    && !object.deleted
                    && object.visible
                    && matches!(
                        object.kind,
                        Kind::HideShowButton { .. }
                            | Kind::AnimationButton { .. }
                            | Kind::MovementButton { .. }
                            | Kind::ScrollButton { .. }
                            | Kind::PresentationButton { .. }
                    )
            })
            .map(|object| object.id)
            .collect()
    }
    pub fn add_link_page_button_with_target(
        &mut self,
        target_page: usize,
        target_button_ordinal: Option<usize>,
    ) -> Result<usize, String> {
        if self.page_count < 2
            || target_page >= self.page_count
            || target_page == self.active_page
            || self
                .objects
                .iter()
                .any(|object| matches!(object.kind, Kind::Unsupported(_)))
            || self
                .objects
                .iter()
                .skip(self.source_count)
                .any(|object| !is_multi_page_addition_kind(&object.kind))
        {
            return Err("A Page Link needs a supported page and a different destination".into());
        }
        let destination_buttons = self.link_destination_buttons(target_page);
        if target_button_ordinal.is_some_and(|ordinal| {
            ordinal == 0
                || destination_buttons.get(ordinal - 1).is_none_or(|id| {
                    !matches!(self.objects[id - 1].kind, Kind::HideShowButton { .. })
                })
        }) {
            return Err("Page Link destination button is unavailable".into());
        }
        let row = self
            .objects
            .iter()
            .filter(|object| {
                object.page == self.active_page && !object.deleted && object.kind.is_action_button()
            })
            .count();
        let y = (5usize + row * 30).min(i16::MAX as usize) as i16;
        Ok(self.add(Kind::LinkPageButton {
            target_page,
            target_button_ordinal,
            label: format!("Go to Page {}", target_page + 1),
            anchor: (10, y),
        }))
    }
    pub fn animation_button_default_settings(&self, target: usize) -> Option<AnimationSettings> {
        let object = self.objects.get(target.checked_sub(1)?)?;
        let direction = match object.kind {
            Kind::Free(_) | Kind::PointOnCircle(_, _, _) => {
                crate::action_records::AnimationDirection::Forward
            }
            Kind::PointOnSegment(_, _) | Kind::PointOnArc(_, _) => {
                crate::action_records::AnimationDirection::Bidirectional
            }
            _ => return None,
        };
        Some(AnimationSettings {
            direction,
            ..AnimationSettings::default()
        })
    }
    pub fn add_animation_button_with_settings(
        &mut self,
        target: usize,
        settings: AnimationSettings,
    ) -> Result<usize, String> {
        self.add_animation_button_for_targets_with_settings(&[(target, settings)])
    }
    pub fn add_animation_button_for_targets_with_settings(
        &mut self,
        targets_and_settings: &[(usize, AnimationSettings)],
    ) -> Result<usize, String> {
        if targets_and_settings.is_empty() || self.partial_view || self.page_count != 1 {
            return Err("An Animation button needs animatable points on this page".into());
        }
        for (index, &(target, settings)) in targets_and_settings.iter().enumerate() {
            settings.validate()?;
            let object = self
                .objects
                .get(target.checked_sub(1).ok_or("Invalid button target")?)
                .ok_or("Invalid button target")?;
            if targets_and_settings[..index]
                .iter()
                .any(|(id, _)| *id == target)
                || object.deleted
                || !object.visible
                || object.page != self.active_page
                || !matches!(
                    object.kind,
                    Kind::Free(_)
                        | Kind::PointOnSegment(_, _)
                        | Kind::PointOnCircle(_, _, _)
                        | Kind::PointOnArc(_, _)
                )
            {
                return Err(
                    "An Animation button needs distinct animatable points on this page".into(),
                );
            }
            if matches!(object.kind, Kind::Free(_))
                && settings.direction != crate::action_records::AnimationDirection::Forward
            {
                return Err("A free-point Animation button has no direction setting".into());
            }
        }
        let row = self
            .objects
            .iter()
            .filter(|object| {
                object.page == self.active_page
                    && !object.deleted
                    && matches!(
                        object.kind,
                        Kind::HideShowButton { .. }
                            | Kind::AnimationButton { .. }
                            | Kind::MovementButton { .. }
                            | Kind::ScrollButton { .. }
                            | Kind::PresentationButton { .. }
                    )
            })
            .count();
        let y = (5usize + row * 30).min(i16::MAX as usize) as i16;
        Ok(self.add(Kind::AnimationButton {
            targets: targets_and_settings
                .iter()
                .map(|(target, _)| *target)
                .collect(),
            label: "Animate Point".into(),
            anchor: (10, y),
            settings: targets_and_settings
                .iter()
                .map(|(_, settings)| *settings)
                .collect(),
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
        let Kind::HideShowButton {
            targets, action, ..
        } = &button.kind
        else {
            return Err("Object is not a Hide/Show button".into());
        };
        let (targets, action) = (targets.clone(), *action);
        if targets.is_empty()
            || targets.iter().any(|&target| {
                target == 0
                    || self
                        .objects
                        .get(target - 1)
                        .is_none_or(|object| object.deleted || object.page != self.active_page)
            })
        {
            return Err("Button target is unavailable".into());
        }
        self.checkpoint();
        for target in targets {
            self.objects[target - 1].visible = action == VisibilityAction::Show;
        }
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
                "隐藏点" => "显示点".into(),
                "显示点" => "隐藏点".into(),
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
                if matches!(
                    kind,
                    Kind::CircleAxis(_, 0 | 1) | Kind::DistanceAxis(_, _, 0 | 1)
                ) {
                    let (start, end, _) = d.line_points(path)?;
                    let point = Point {
                        x: start.x + (end.x - start.x) * t,
                        y: start.y + (end.y - start.y) * t,
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
                Kind::DistanceVerticalUnit(x_axis, step, independent) => {
                    let Kind::DistanceAxis(origin, distance, 1) =
                        d.objects.get(x_axis.checked_sub(1)?)?.kind
                    else {
                        return None;
                    };
                    let origin = go(d, origin, seen)?;
                    let length = if *independent {
                        *step
                    } else {
                        d.defining_distance_pixels(distance)?
                    };
                    Some(Point {
                        x: origin.x,
                        y: origin.y - length,
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
                Kind::FixedRectangularTranslatedPoint(
                    source,
                    x_units,
                    x_distance,
                    y_units,
                    y_distance,
                ) => {
                    let source = go(d, *source, seen)?;
                    Some(Point {
                        x: source.x + x_units * x_distance,
                        y: source.y - y_units * y_distance,
                    })
                }
                Kind::MarkedDistanceTranslatedPoint(source, distance, angle) => {
                    let source = go(d, *source, seen)?;
                    let step = d.distance_value_pixels(*distance)?;
                    let radians = angle.to_radians();
                    Some(Point {
                        x: source.x + step * radians.cos(),
                        y: source.y - step * radians.sin(),
                    })
                }
                Kind::MixedRectangularTranslatedPoint(
                    source,
                    distance,
                    fixed_cm,
                    fixed_horizontal,
                ) => {
                    let source = go(d, *source, seen)?;
                    let fixed_pixels = fixed_cm * 37.79527559055118;
                    let marked_pixels = d.distance_value_pixels(*distance)?;
                    Some(Point {
                        x: source.x
                            + if *fixed_horizontal {
                                fixed_pixels
                            } else {
                                marked_pixels
                            },
                        y: source.y
                            - if *fixed_horizontal {
                                marked_pixels
                            } else {
                                fixed_pixels
                            },
                    })
                }
                Kind::RectangularMarkedTranslatedPoint(source, horizontal, vertical) => {
                    let source = go(d, *source, seen)?;
                    Some(Point {
                        x: source.x + d.distance_value_pixels(*horizontal)?,
                        y: source.y - d.distance_value_pixels(*vertical)?,
                    })
                }
                Kind::VectorTranslatedPoint(source, start, end) => {
                    let source = go(d, *source, seen)?;
                    let start = go(d, *start, seen)?;
                    let end = go(d, *end, seen)?;
                    Some(Point {
                        x: source.x + end.x - start.x,
                        y: source.y + end.y - start.y,
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
                Kind::RatioScaledPoint(source, center, origin, base, target) => {
                    let source = go(d, *source, seen)?;
                    let center = go(d, *center, seen)?;
                    let factor = d.ratio_for_points(*origin, *base, *target)?;
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
                Kind::AngleRotatedPoint(source, center, first, vertex, last) => {
                    let source = go(d, *source, seen)?;
                    let center = go(d, *center, seen)?;
                    let first = go(d, *first, seen)?;
                    let vertex = go(d, *vertex, seen)?;
                    let last = go(d, *last, seen)?;
                    rotate_point_about(source, center, marked_rotation_theta(first, vertex, last)?)
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
    pub fn distance_value_pixels(&self, id: usize) -> Option<f64> {
        let kind = &self.objects.get(id.checked_sub(1)?)?.kind;
        let cm = match kind {
            Kind::SegmentLength(_) => self.segment_length_cm(id)?,
            Kind::PointDistance(_, _) => self.point_distance_cm(id)?,
            Kind::PolygonPerimeter(_) => self.perimeter_cm(id)?,
            Kind::CircleCircumference(_) => self.circle_circumference_cm(id)?,
            Kind::CircleRadius(_) => self.circle_radius_cm(id)?,
            Kind::ArcLength(_) => self.arc_length_cm(id)?,
            _ => return None,
        };
        let pixels = cm * 37.79527559055118;
        (pixels.is_finite() && pixels >= 0.).then_some(pixels)
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
        match &object.kind {
            Kind::Polygon(vertices) => vertices
                .iter()
                .map(|&vertex| self.position(vertex))
                .collect(),
            Kind::ReflectedPolygon(source, mirror) => {
                let points = self.polygon_points(*source)?;
                let (a, b, _) = self.line_points(*mirror)?;
                points
                    .into_iter()
                    .map(|point| reflect_point(point, a, b))
                    .collect()
            }
            Kind::VectorTranslatedPolygon(source, start, end) => {
                let start = self.position(*start)?;
                let end = self.position(*end)?;
                let dx = end.x - start.x;
                let dy = end.y - start.y;
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| {
                        let x = point.x + dx;
                        let y = point.y + dy;
                        (x.is_finite() && y.is_finite()).then_some(Point { x, y })
                    })
                    .collect()
            }
            Kind::PolarTranslatedPolygon(source, angle, units, distance) => {
                let radians = angle.to_radians();
                let dx = units * distance * radians.cos();
                let dy = -units * distance * radians.sin();
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| {
                        let x = point.x + dx;
                        let y = point.y + dy;
                        (x.is_finite() && y.is_finite()).then_some(Point { x, y })
                    })
                    .collect()
            }
            Kind::FixedRectangularTranslatedPolygon(source, xu, xd, yu, yd) => {
                let dx = xu * xd;
                let dy = -yu * yd;
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| {
                        let x = point.x + dx;
                        let y = point.y + dy;
                        (x.is_finite() && y.is_finite()).then_some(Point { x, y })
                    })
                    .collect()
            }
            Kind::MarkedDistanceTranslatedPolygon(source, distance, angle) => {
                let step = self.distance_value_pixels(*distance)?;
                let radians = angle.to_radians();
                let dx = step * radians.cos();
                let dy = -step * radians.sin();
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| {
                        let x = point.x + dx;
                        let y = point.y + dy;
                        (x.is_finite() && y.is_finite()).then_some(Point { x, y })
                    })
                    .collect()
            }
            Kind::RectangularMarkedTranslatedPolygon(source, horizontal, vertical) => {
                let dx = self.distance_value_pixels(*horizontal)?;
                let dy = -self.distance_value_pixels(*vertical)?;
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| {
                        let x = point.x + dx;
                        let y = point.y + dy;
                        (x.is_finite() && y.is_finite()).then_some(Point { x, y })
                    })
                    .collect()
            }
            Kind::MixedRectangularTranslatedPolygon(
                source,
                distance,
                fixed_cm,
                fixed_horizontal,
            ) => {
                let marked = self.distance_value_pixels(*distance)?;
                let fixed = fixed_cm * 37.79527559055118;
                let (dx, dy) = if *fixed_horizontal {
                    (fixed, -marked)
                } else {
                    (marked, -fixed)
                };
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| {
                        let x = point.x + dx;
                        let y = point.y + dy;
                        (x.is_finite() && y.is_finite()).then_some(Point { x, y })
                    })
                    .collect()
            }
            Kind::RotatedPolygon(source, center, angle) => {
                let center = self.position(*center)?;
                let theta = angle.to_radians();
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| rotate_point_about(point, center, theta))
                    .collect()
            }
            Kind::AngleRotatedPolygon(source, center, first, vertex, last) => {
                let center = self.position(*center)?;
                let theta = marked_rotation_theta(
                    self.position(*first)?,
                    self.position(*vertex)?,
                    self.position(*last)?,
                )?;
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| rotate_point_about(point, center, theta))
                    .collect()
            }
            Kind::ScaledPolygon(source, center, factor) => {
                let center = self.position(*center)?;
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| {
                        let x = center.x + factor * (point.x - center.x);
                        let y = center.y + factor * (point.y - center.y);
                        (x.is_finite() && y.is_finite()).then_some(Point { x, y })
                    })
                    .collect()
            }
            Kind::RatioScaledPolygon(source, center, origin, base, target) => {
                let center = self.position(*center)?;
                let factor = self.ratio_for_points(*origin, *base, *target)?;
                self.polygon_points(*source)?
                    .into_iter()
                    .map(|point| {
                        let x = center.x + factor * (point.x - center.x);
                        let y = center.y + factor * (point.y - center.y);
                        (x.is_finite() && y.is_finite()).then_some(Point { x, y })
                    })
                    .collect()
            }
            _ => None,
        }
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
    pub fn polygon_area_cm2(&self, id: usize) -> Option<f64> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::PolygonArea(polygon) = object.kind else {
            return None;
        };
        let points = self.polygon_points(polygon)?;
        if points.len() < 3 {
            return None;
        }
        let origin = points[0];
        let twice_area = points
            .iter()
            .skip(1)
            .zip(points.iter().skip(2))
            .map(|(a, b)| (a.x - origin.x) * (b.y - origin.y) - (a.y - origin.y) * (b.x - origin.x))
            .sum::<f64>();
        let value = twice_area.abs() / (2. * 37.79527559055118_f64.powi(2));
        value.is_finite().then_some(value)
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
                Kind::PolarTranslatedSegment(source, angle, units, distance) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let step = units * distance;
                    let radians = angle.to_radians();
                    let dx = step * radians.cos();
                    let dy = -step * radians.sin();
                    Some((
                        Point {
                            x: a.x + dx,
                            y: a.y + dy,
                        },
                        Point {
                            x: b.x + dx,
                            y: b.y + dy,
                        },
                        mode,
                    ))
                }
                Kind::FixedRectangularTranslatedSegment(
                    source,
                    x_units,
                    x_distance,
                    y_units,
                    y_distance,
                ) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let dx = x_units * x_distance;
                    let dy = -y_units * y_distance;
                    Some((
                        Point {
                            x: a.x + dx,
                            y: a.y + dy,
                        },
                        Point {
                            x: b.x + dx,
                            y: b.y + dy,
                        },
                        mode,
                    ))
                }
                Kind::VectorTranslatedSegment(source, start, end) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let start = d.position(start)?;
                    let end = d.position(end)?;
                    let dx = end.x - start.x;
                    let dy = end.y - start.y;
                    Some((
                        Point {
                            x: a.x + dx,
                            y: a.y + dy,
                        },
                        Point {
                            x: b.x + dx,
                            y: b.y + dy,
                        },
                        mode,
                    ))
                }
                Kind::MarkedDistanceTranslatedSegment(source, distance, angle) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let step = d.distance_value_pixels(distance)?;
                    let radians = angle.to_radians();
                    let dx = step * radians.cos();
                    let dy = -step * radians.sin();
                    Some((
                        Point {
                            x: a.x + dx,
                            y: a.y + dy,
                        },
                        Point {
                            x: b.x + dx,
                            y: b.y + dy,
                        },
                        mode,
                    ))
                }
                Kind::AngleRotatedSegment(source, center, first, vertex, last) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let center = d.position(center)?;
                    let theta = marked_rotation_theta(
                        d.position(first)?,
                        d.position(vertex)?,
                        d.position(last)?,
                    )?;
                    Some((
                        rotate_point_about(a, center, theta)?,
                        rotate_point_about(b, center, theta)?,
                        mode,
                    ))
                }
                Kind::RotatedSegment(source, center, angle) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let center = d.position(center)?;
                    let theta = angle.to_radians();
                    Some((
                        rotate_point_about(a, center, theta)?,
                        rotate_point_about(b, center, theta)?,
                        mode,
                    ))
                }
                Kind::ScaledSegment(source, center, factor) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let center = d.position(center)?;
                    let scale = |point: Point| Point {
                        x: center.x + factor * (point.x - center.x),
                        y: center.y + factor * (point.y - center.y),
                    };
                    let (a, b) = (scale(a), scale(b));
                    (a.x.is_finite() && a.y.is_finite() && b.x.is_finite() && b.y.is_finite())
                        .then_some((a, b, mode))
                }
                Kind::RatioScaledSegment(source, center, origin, base, target) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let center = d.position(center)?;
                    let factor = d.ratio_for_points(origin, base, target)?;
                    let scale = |point: Point| Point {
                        x: center.x + factor * (point.x - center.x),
                        y: center.y + factor * (point.y - center.y),
                    };
                    let (a, b) = (scale(a), scale(b));
                    (a.x.is_finite() && a.y.is_finite() && b.x.is_finite() && b.y.is_finite())
                        .then_some((a, b, mode))
                }
                Kind::ReflectedSegment(source, mirror) => {
                    let (a, b, mode) = go(d, source, seen)?;
                    let (mirror_a, mirror_b, _) = go(d, mirror, seen)?;
                    Some((
                        reflect_point(a, mirror_a, mirror_b)?,
                        reflect_point(b, mirror_a, mirror_b)?,
                        mode,
                    ))
                }
                Kind::GraphAxis(a, b, _) => Some((d.position(a)?, d.position(b)?, LineMode::Line)),
                Kind::CircleAxis(circle, axis @ (0 | 1)) => {
                    let (center, radius) = d.circle_points(circle)?;
                    let unit = if axis == 1 {
                        Point {
                            x: center.x + radius,
                            y: center.y,
                        }
                    } else {
                        Point {
                            x: center.x,
                            y: center.y - radius,
                        }
                    };
                    Some((center, unit, LineMode::Line))
                }
                Kind::DistanceAxis(origin, distance, axis @ (0 | 1)) => {
                    let center = d.position(origin)?;
                    let length = d.defining_distance_pixels(distance)?;
                    let unit = if axis == 1 {
                        Point {
                            x: center.x + length,
                            y: center.y,
                        }
                    } else {
                        Point {
                            x: center.x,
                            y: center.y - length,
                        }
                    };
                    Some((center, unit, LineMode::Line))
                }
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
        if let Kind::PolarTranslatedCircle(source, angle, units, distance) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (center, radius) = self.circle_points(source)?;
            let step = units * distance;
            let radians = angle.to_radians();
            let moved = Point {
                x: center.x + step * radians.cos(),
                y: center.y - step * radians.sin(),
            };
            return (moved.x.is_finite() && moved.y.is_finite()).then_some((moved, radius));
        }
        if let Kind::FixedRectangularTranslatedCircle(
            source,
            x_units,
            x_distance,
            y_units,
            y_distance,
        ) = self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (center, radius) = self.circle_points(source)?;
            let moved = Point {
                x: center.x + x_units * x_distance,
                y: center.y - y_units * y_distance,
            };
            return (moved.x.is_finite() && moved.y.is_finite()).then_some((moved, radius));
        }
        if let Kind::VectorTranslatedCircle(source, start, end) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (center, radius) = self.circle_points(source)?;
            let start = self.position(start)?;
            let end = self.position(end)?;
            let moved = Point {
                x: center.x + end.x - start.x,
                y: center.y + end.y - start.y,
            };
            return (moved.x.is_finite() && moved.y.is_finite()).then_some((moved, radius));
        }
        if let Kind::MarkedDistanceTranslatedCircle(source, distance, angle) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (center, radius) = self.circle_points(source)?;
            let step = self.distance_value_pixels(distance)?;
            let radians = angle.to_radians();
            let moved = Point {
                x: center.x + step * radians.cos(),
                y: center.y - step * radians.sin(),
            };
            return (moved.x.is_finite() && moved.y.is_finite()).then_some((moved, radius));
        }
        if let Kind::AngleRotatedCircle(source, center, first, vertex, last) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (source_center, radius) = self.circle_points(source)?;
            let center = self.position(center)?;
            let theta = marked_rotation_theta(
                self.position(first)?,
                self.position(vertex)?,
                self.position(last)?,
            )?;
            return Some((rotate_point_about(source_center, center, theta)?, radius));
        }
        if let Kind::RotatedCircle(source, center, angle) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (source_center, radius) = self.circle_points(source)?;
            let center = self.position(center)?;
            return Some((
                rotate_point_about(source_center, center, angle.to_radians())?,
                radius,
            ));
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
        if let Kind::RatioScaledCircle(source, center, origin, base, target) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            let (source_center, radius) = self.circle_points(source)?;
            let center = self.position(center)?;
            let factor = self.ratio_for_points(origin, base, target)?;
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
    pub fn locus_strokes(
        &self,
        id: usize,
        subdivisions: usize,
    ) -> Option<Vec<(Point, Point, LineMode)>> {
        let Kind::Locus(ref refs, samples) = self.objects.get(id.checked_sub(1)?)?.kind else {
            return None;
        };
        let (&target, &driver) = (refs.first()?, refs.get(2)?);
        let driver_kind = &self.objects.get(driver.checked_sub(1)?)?.kind;
        let straight_target =
            is_locus_straight_kind(&self.objects.get(target.checked_sub(1)?)?.kind);
        let circle_target = is_circle_kind(&self.objects.get(target - 1)?.kind);
        let arc_target = is_arc_kind(&self.objects.get(target - 1)?.kind);
        let polygon_target = is_locus_polygon_kind(&self.objects.get(target - 1)?.kind);
        let mut sample = self.clone();
        let steps = subdivisions.clamp(2, 1000).min(samples as usize);
        let mut segments = Vec::with_capacity(
            (steps + 1)
                * if circle_target || arc_target {
                    64
                } else if polygon_target {
                    3
                } else {
                    1
                },
        );
        let mut previous = None;
        let circle_units = circle_target.then(|| {
            (0..=64)
                .map(|index| {
                    let angle = index as f64 * std::f64::consts::TAU / 64.;
                    (angle.cos(), angle.sin())
                })
                .collect::<Vec<_>>()
        });
        for step in 0..=steps {
            let fraction = step as f64 / steps as f64;
            sample.objects.get_mut(driver - 1)?.kind = match driver_kind {
                Kind::PointOnSegment(_, _) => Kind::PointOnSegment(refs[1], fraction),
                Kind::PointOnCircle(_, _, _) => {
                    let angle = fraction * std::f64::consts::TAU;
                    Kind::PointOnCircle(refs[1], angle.cos(), angle.sin())
                }
                Kind::PointOnArc(_, _) => Kind::PointOnArc(refs[1], fraction),
                _ => return None,
            };
            if straight_target {
                let (start, end, mode) = sample.line_points(target)?;
                segments.push((start, end, mode));
            } else if let Some(units) = &circle_units {
                let (center, radius) = sample.circle_points(target)?;
                for pair in units.windows(2) {
                    let start = Point {
                        x: center.x + radius * pair[0].0,
                        y: center.y + radius * pair[0].1,
                    };
                    let end = Point {
                        x: center.x + radius * pair[1].0,
                        y: center.y + radius * pair[1].1,
                    };
                    segments.push((start, end, LineMode::Segment));
                }
            } else if arc_target {
                for pair in sample.arc_points(target, 64)?.windows(2) {
                    segments.push((pair[0], pair[1], LineMode::Segment));
                }
            } else if polygon_target {
                let points = sample.polygon_points(target)?;
                for index in 0..points.len() {
                    segments.push((
                        points[index],
                        points[(index + 1) % points.len()],
                        LineMode::Segment,
                    ));
                }
            } else {
                let current = sample.position(target)?;
                if let Some(start) = previous {
                    segments.push((start, current, LineMode::Segment));
                }
                previous = Some(current);
            }
        }
        Some(segments)
    }

    pub fn locus_polygons(&self, id: usize, subdivisions: usize) -> Option<Vec<Vec<Point>>> {
        let Kind::Locus(ref refs, samples) = self.objects.get(id.checked_sub(1)?)?.kind else {
            return None;
        };
        let (&target, &driver) = (refs.first()?, refs.get(2)?);
        if !is_locus_polygon_kind(&self.objects.get(target.checked_sub(1)?)?.kind) {
            return None;
        }
        let driver_kind = &self.objects.get(driver.checked_sub(1)?)?.kind;
        let steps = subdivisions.clamp(2, 1000).min(samples as usize);
        let mut sample = self.clone();
        let mut polygons = Vec::with_capacity(steps + 1);
        for step in 0..=steps {
            let fraction = step as f64 / steps as f64;
            sample.objects.get_mut(driver - 1)?.kind = match driver_kind {
                Kind::PointOnSegment(_, _) => Kind::PointOnSegment(refs[1], fraction),
                Kind::PointOnCircle(_, _, _) => {
                    let angle = fraction * std::f64::consts::TAU;
                    Kind::PointOnCircle(refs[1], angle.cos(), angle.sin())
                }
                Kind::PointOnArc(_, _) => Kind::PointOnArc(refs[1], fraction),
                _ => return None,
            };
            polygons.push(sample.polygon_points(target)?);
        }
        Some(polygons)
    }

    pub fn locus_polygon_surfaces(
        &self,
        id: usize,
        subdivisions: usize,
    ) -> Option<Vec<Vec<Point>>> {
        let polygons = self.locus_polygons(id, subdivisions)?;
        let mut surfaces = polygons.clone();
        for pair in polygons.windows(2) {
            let (before, after) = (&pair[0], &pair[1]);
            if before.len() != after.len() {
                return None;
            }
            for index in 0..before.len() {
                let next = (index + 1) % before.len();
                surfaces.push(vec![before[index], before[next], after[next], after[index]]);
            }
        }
        Some(surfaces)
    }

    pub fn locus_segments(&self, id: usize, subdivisions: usize) -> Option<Vec<(Point, Point)>> {
        Some(
            self.locus_strokes(id, subdivisions)?
                .into_iter()
                .map(|(start, end, _)| (start, end))
                .collect(),
        )
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
            Some(Kind::DistanceVerticalUnit(x_axis, _, true)) => {
                let x_axis = *x_axis;
                let Some(Kind::DistanceAxis(origin, _, 1)) = x_axis
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
                Kind::DistanceVerticalUnit(x_axis, step, true)
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
        self.last_moved_object = Some(id);
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
            | Kind::ScaledCircle(_, _, _)
            | Kind::RatioScaledCircle(_, _, _, _, _) => {
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
        if self.page_count > 1 {
            if self.has_pending_multi_page_additions() {
                self.encode_multi_page_additions()?;
            } else {
                self.encode_multi_page_state()?;
            }
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
        let first_deleted = self.objects.iter().position(|object| object.deleted);
        let last_plot_related = self.objects.iter().rposition(|o| {
            !o.deleted
                && matches!(
                    o.kind,
                    Kind::FunctionDefinition(_)
                        | Kind::DerivativeFunction(_, _)
                        | Kind::FunctionPlot(_, _, _, _, _)
                        | Kind::ParametricPlot(_, _, _, _, _, _, _)
                        | Kind::PathLiteralPoint(_, _)
                        | Kind::PathValuePoint(_, _)
                        | Kind::PlottedPoint(_, _, _, _)
                )
        });
        let only_independent_objects_deleted = self
            .objects
            .iter()
            .filter(|object| object.deleted)
            .all(|object| {
                matches!(object.kind, Kind::Free(_) | Kind::PlainText { .. })
                    && object.label.is_none()
            });
        if first_deleted.is_some_and(|first| last_plot_related.is_some_and(|last| first <= last))
            && self.objects[first_deleted.unwrap()..]
                .iter()
                .any(|object| !object.deleted)
            && !only_independent_objects_deleted
        {
            return Err(
                "Deleting an object before a surviving plot or function is not supported".into(),
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
                if let Some(template) = &o.rich_text_override {
                    template.encode_group(text, *rect)?;
                } else {
                    text_object::encode_plain_text_group(text, *rect)?;
                }
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
                    if group.source_id() as usize != *source
                        || !self.objects.get(source.saturating_sub(1)).is_some_and(|o| {
                            match &o.kind {
                                Kind::Parameter(parameter) => match group.named_label_value() {
                                    Some((name, _)) => name == parameter.name(),
                                    None => parameter.name() == "t[1]",
                                },
                                _ => false,
                            }
                        })
                    {
                        return Err("Unsupported table source".into());
                    }
                }
                Kind::ValueTablePair(first, second, group) => {
                    TableGroup::parse_pair(group.as_bytes())?;
                    if group.pair_source_ids() != Some([*first as u32, *second as u32])
                        || !matches!(self.objects.get(second - 1).map(|o| &o.kind), Some(Kind::Parameter(parameter)) if parameter.name() == "t[1]")
                        || !matches!(self.objects.get(first - 1).map(|o| &o.kind), Some(Kind::Parameter(parameter)) if parameter.name() == "t[2]")
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
                || matches!(o.kind, Kind::PolarTranslatedSegment(_, a, u, d) if !a.is_finite() || !u.is_finite() || !d.is_finite() || u <= 0.)
                || matches!(o.kind, Kind::PolarTranslatedCircle(_, a, u, d) if !a.is_finite() || !u.is_finite() || !d.is_finite() || u <= 0.)
                || matches!(o.kind, Kind::PolarTranslatedPolygon(_, a, u, d) if !a.is_finite() || !u.is_finite() || !d.is_finite() || u <= 0.)
                || matches!(o.kind, Kind::FixedRectangularTranslatedPoint(_, xu, xd, yu, yd)
                    if !xu.is_finite() || !xd.is_finite() || !yu.is_finite() || !yd.is_finite()
                    || (xu - 37.79527559055118).abs() > 1e-9
                    || (yu - 37.79527559055118).abs() > 1e-9)
                || matches!(o.kind, Kind::FixedRectangularTranslatedSegment(_, xu, xd, yu, yd)
                    if !xu.is_finite() || !xd.is_finite() || !yu.is_finite() || !yd.is_finite()
                    || (xu - 37.79527559055118).abs() > 1e-9
                    || (yu - 37.79527559055118).abs() > 1e-9)
                || matches!(o.kind, Kind::FixedRectangularTranslatedCircle(_, xu, xd, yu, yd)
                    if !xu.is_finite() || !xd.is_finite() || !yu.is_finite() || !yd.is_finite()
                    || (xu - 37.79527559055118).abs() > 1e-9
                    || (yu - 37.79527559055118).abs() > 1e-9)
                || matches!(o.kind, Kind::FixedRectangularTranslatedPolygon(_, xu, xd, yu, yd)
                    if !xu.is_finite() || !xd.is_finite() || !yu.is_finite() || !yd.is_finite()
                    || (xu - 37.79527559055118).abs() > 1e-9
                    || (yu - 37.79527559055118).abs() > 1e-9)
                || matches!(o.kind, Kind::MarkedDistanceTranslatedPoint(_, _, angle) if !angle.is_finite())
                || matches!(o.kind, Kind::MarkedDistanceTranslatedSegment(_, _, angle) | Kind::MarkedDistanceTranslatedCircle(_, _, angle) | Kind::MarkedDistanceTranslatedPolygon(_, _, angle) if !angle.is_finite())
                || matches!(o.kind, Kind::MixedRectangularTranslatedPoint(_, _, fixed_cm, _) | Kind::MixedRectangularTranslatedPolygon(_, _, fixed_cm, _) if !fixed_cm.is_finite())
                || matches!(o.kind, Kind::ScaledPoint(_, _, factor) | Kind::ScaledCircle(_, _, factor) | Kind::ScaledSegment(_, _, factor) | Kind::ScaledPolygon(_, _, factor) if !factor.is_finite() || factor == 0.)
                || matches!(o.kind, Kind::RotatedPoint(_, _, angle) | Kind::RotatedSegment(_, _, angle) | Kind::RotatedCircle(_, _, angle) | Kind::RotatedPolygon(_, _, angle) if !angle.is_finite())
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
            if let Kind::HideShowButton { targets, label, .. } = &o.kind {
                if self.page_count != 1
                    || targets.is_empty()
                    || targets
                        .iter()
                        .any(|&target| self.objects[target - 1].page != o.page)
                    || label.len() > u16::MAX as usize
                {
                    return Err(format!("Invalid Hide/Show button {}", o.id));
                }
            }
            if let Kind::AnimationButton {
                targets,
                settings,
                label,
                ..
            } = &o.kind
            {
                if self.page_count != 1
                    || targets.is_empty()
                    || targets.len() != settings.len()
                    || targets
                        .iter()
                        .any(|&target| self.objects[target - 1].page != o.page)
                    || label.len() > u16::MAX as usize
                {
                    return Err(format!("Invalid Animation button {}", o.id));
                }
            }
            if let Kind::AnimationButton { settings, .. } = &o.kind {
                for setting in settings {
                    setting
                        .validate()
                        .map_err(|error| format!("Invalid Animation button {}: {error}", o.id))?;
                }
            }
            if let Kind::MovementButton {
                source,
                destination,
                label,
                ..
            } = &o.kind
            {
                if self.page_count != 1
                    || *source == *destination
                    || *source == 0
                    || *destination == 0
                    || *source > self.objects.len()
                    || *destination > self.objects.len()
                    || self.objects[*source - 1].page != o.page
                    || self.objects[*destination - 1].page != o.page
                    || !matches!(self.objects[*source - 1].kind, Kind::Free(_))
                    || self.position(*destination).is_none()
                    || label.len() > u16::MAX as usize
                {
                    return Err(format!("Invalid Movement button {}", o.id));
                }
            }
            if let Kind::ScrollButton { target, label, .. } = &o.kind {
                if self.page_count != 1
                    || *target == 0
                    || *target > self.objects.len()
                    || self.objects[*target - 1].page != o.page
                    || !is_point_kind(&self.objects[*target - 1].kind)
                    || self.position(*target).is_none()
                    || label.len() > u16::MAX as usize
                {
                    return Err(format!("Invalid Scroll button {}", o.id));
                }
            }
            if let Kind::PresentationButton { targets, label, .. } = &o.kind {
                if self.page_count != 1
                    || targets.is_empty()
                    || targets.iter().enumerate().any(|(index, &target)| {
                        target == 0
                            || target >= o.id
                            || targets[..index].contains(&target)
                            || self.objects[target - 1].page != o.page
                            || !self.objects[target - 1].kind.is_action_button()
                    })
                    || label.len() > u16::MAX as usize
                {
                    return Err(format!("Invalid Presentation button {}", o.id));
                }
            }
            if let Kind::LinkUrlButton { url, label, .. } = &o.kind {
                if self.page_count != 1
                    || url.is_empty()
                    || url.len() > u16::MAX as usize
                    || label.len() > u16::MAX as usize
                    || url.as_bytes().contains(&0)
                    || label.as_bytes().contains(&0)
                {
                    return Err(format!("Invalid Link URL button {}", o.id));
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
            if let Kind::Locus(ref refs, samples) = o.kind {
                if !(2..=10_000).contains(&samples)
                    || refs
                        .first()
                        .zip(refs.get(2))
                        .and_then(|(&target, &driver)| {
                            expected_locus_refs(&self.objects, driver, target)
                        })
                        .as_ref()
                        != Some(refs)
                    || self.locus_segments(o.id, 2).is_none()
                {
                    return Err(format!("Invalid locus {}", o.id));
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
                || matches!(o.kind, Kind::DistanceVerticalUnit(_, step, _) if !step.is_finite() || step <= 0.0)
                || matches!(o.kind, Kind::GraphAxis(_, _, axis) if !matches!(axis, 0 | 1 | 0x1_0001))
                || matches!(o.kind, Kind::CircleAxis(_, axis) if !matches!(axis, 0 | 1))
                || matches!(o.kind, Kind::DistanceAxis(_, _, axis) if !matches!(axis, 0 | 1))
            {
                return Err(format!("Invalid graph object {}", o.id));
            }
            if matches!(
                o.kind,
                Kind::GraphHorizontalUnit(_, _)
                    | Kind::GraphVerticalUnit(_, _)
                    | Kind::DistanceVerticalUnit(_, _, _)
            ) && self
                .position(o.id)
                .is_none_or(|p| !p.x.is_finite() || !p.y.is_finite())
            {
                return Err(format!("Undefined graph point {}", o.id));
            }
            if matches!(
                o.kind,
                Kind::GraphAxis(_, _, _) | Kind::CircleAxis(_, _) | Kind::DistanceAxis(_, _, _)
            ) && self.line_points(o.id).is_none()
            {
                return Err(format!("Undefined graph axis {}", o.id));
            }
            if matches!(o.kind, Kind::CoordinateSystem(_, _)) && self.graph_frame(o.id).is_none() {
                return Err(format!("Invalid coordinate system {}", o.id));
            }
            if let Kind::DerivativeFunction(source, _) = o.kind
                && (source >= o.id || self.function_expression(source).is_none())
            {
                return Err(format!("Invalid derivative source in object {}", o.id));
            }
            if let Kind::FunctionPlot(definition, frame, min, max, samples) = o.kind {
                if !matches!(
                    self.objects
                        .get(definition.saturating_sub(1))
                        .map(|o| &o.kind),
                    Some(Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _))
                ) || self.graph_frame(frame).is_none()
                    || !min.is_finite()
                    || !max.is_finite()
                    || min >= max
                    || !(2..=4096).contains(&samples)
                {
                    return Err(format!("Invalid function plot {}", o.id));
                }
            }
            if let Kind::ParametricPlot(x, y, frame, min, max, samples, _) = o.kind {
                if x == y
                    || [x, y].into_iter().any(|id| {
                        !matches!(
                            self.objects.get(id.saturating_sub(1)).map(|o| &o.kind),
                            Some(Kind::FunctionDefinition(_) | Kind::DerivativeFunction(_, _))
                        )
                    })
                    || self.graph_frame(frame).is_none()
                    || !min.is_finite()
                    || !max.is_finite()
                    || min >= max
                    || !(2..=4096).contains(&samples)
                {
                    return Err(format!("Invalid parametric plot {}", o.id));
                }
            }
            if let Kind::PlottedPoint(source, frame, mode, stored) = o.kind {
                if self
                    .objects
                    .get(source.saturating_sub(1))
                    .is_none_or(|anchor| anchor.page != o.page)
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
                        return Err("Cannot encode this Plot Points value".into());
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
            if matches!(o.kind, Kind::PolygonArea(_))
                && self
                    .polygon_area_cm2(o.id)
                    .is_none_or(|value| !value.is_finite())
            {
                return Err(format!("Invalid polygon in area measurement {}", o.id));
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
                && matches!(o.kind, Kind::PolarTranslatedSegment(source, _, _, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Segment(_, _)))
                    || self.line_points(o.id).is_none())
            {
                return Err(format!("Invalid source segment in translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::PolarTranslatedCircle(source, _, _, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)))
                    || self.circle_points(o.id).is_none())
            {
                return Err(format!("Invalid source circle in translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::FixedRectangularTranslatedPoint(source, _, _, _, _)
                    if self.position(source).is_none()
                    || !self.objects.get(source.saturating_sub(1)).is_some_and(|object| is_point_kind(&object.kind)))
            {
                return Err(format!(
                    "Invalid source point in rectangular translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::FixedRectangularTranslatedSegment(source, _, _, _, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Segment(_, _)))
                    || self.line_points(o.id).is_none())
            {
                return Err(format!(
                    "Invalid source segment in rectangular translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::FixedRectangularTranslatedCircle(source, _, _, _, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)))
                    || self.circle_points(o.id).is_none())
            {
                return Err(format!(
                    "Invalid source circle in rectangular translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::MarkedDistanceTranslatedPoint(source, distance, _)
                    if self.position(source).is_none()
                    || !self.objects.get(source.saturating_sub(1)).is_some_and(|object| is_point_kind(&object.kind))
                    || !self.objects.get(distance.saturating_sub(1)).is_some_and(|object| is_distance_value_kind(&object.kind))
                    || self.distance_value_pixels(distance).is_none())
            {
                return Err(format!("Invalid marked distance translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::MarkedDistanceTranslatedSegment(source, distance, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Segment(_, _)))
                    || !self.objects.get(distance.saturating_sub(1)).is_some_and(|object| is_distance_value_kind(&object.kind))
                    || self.distance_value_pixels(distance).is_none()
                    || self.line_points(o.id).is_none())
            {
                return Err(format!(
                    "Invalid marked-distance segment translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::MarkedDistanceTranslatedCircle(source, distance, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)))
                    || !self.objects.get(distance.saturating_sub(1)).is_some_and(|object| is_distance_value_kind(&object.kind))
                    || self.distance_value_pixels(distance).is_none()
                    || self.circle_points(o.id).is_none())
            {
                return Err(format!(
                    "Invalid marked-distance circle translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::MixedRectangularTranslatedPoint(source, distance, _, _)
                    if self.position(source).is_none()
                    || !self.objects.get(source.saturating_sub(1)).is_some_and(|object| is_point_kind(&object.kind))
                    || !self.objects.get(distance.saturating_sub(1)).is_some_and(|object| is_distance_value_kind(&object.kind))
                    || self.distance_value_pixels(distance).is_none())
            {
                return Err(format!("Invalid mixed rectangular translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RectangularMarkedTranslatedPoint(source, horizontal, vertical)
                if self.position(source).is_none()
                || !self.objects.get(source.saturating_sub(1)).is_some_and(|object| is_point_kind(&object.kind))
                || [horizontal, vertical].iter().any(|&id| {
                    !self.objects.get(id.saturating_sub(1)).is_some_and(|object| is_distance_value_kind(&object.kind))
                    || self.distance_value_pixels(id).is_none()
                }))
            {
                return Err(format!("Invalid rectangular marked translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::VectorTranslatedPoint(source, start, end)
                if [source, start, end].iter().any(|&id| {
                    self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                        object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()
                    })
                }))
            {
                return Err(format!("Invalid point in vector translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::VectorTranslatedSegment(source, start, end)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Segment(_, _)))
                    || [start, end].iter().any(|&id| self.objects.get(id.saturating_sub(1)).is_none_or(|object| object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()))
                    || self.line_points(o.id).is_none())
            {
                return Err(format!("Invalid segment in vector translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::VectorTranslatedCircle(source, start, end)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)))
                    || [start, end].iter().any(|&id| self.objects.get(id.saturating_sub(1)).is_none_or(|object| object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()))
                    || self.circle_points(o.id).is_none())
            {
                return Err(format!("Invalid circle in vector translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::PolarTranslatedPolygon(source, _, _, _)
                    | Kind::FixedRectangularTranslatedPolygon(source, _, _, _, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || self.polygon_points(o.id).is_none())
            {
                return Err(format!("Invalid interior in translation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::MarkedDistanceTranslatedPolygon(source, distance, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || self.distance_value_pixels(distance).is_none()
                    || self.polygon_points(o.id).is_none())
            {
                return Err(format!(
                    "Invalid interior or distance in translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RectangularMarkedTranslatedPolygon(source, horizontal, vertical)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || self.distance_value_pixels(horizontal).is_none()
                    || self.distance_value_pixels(vertical).is_none()
                    || self.polygon_points(o.id).is_none())
            {
                return Err(format!(
                    "Invalid interior or distances in translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::MixedRectangularTranslatedPolygon(source, distance, _, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || self.distance_value_pixels(distance).is_none()
                    || self.polygon_points(o.id).is_none())
            {
                return Err(format!(
                    "Invalid interior or distance in mixed translation {}",
                    o.id
                ));
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
                && matches!(o.kind, Kind::ReflectedSegment(source, mirror)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Segment(_, _)))
                    || self.line_points(mirror).is_none() || self.line_points(o.id).is_none())
            {
                return Err(format!("Invalid segment or mirror in reflection {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::ReflectedPolygon(source, mirror)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || self.line_points(mirror).is_none() || self.polygon_points(o.id).is_none())
            {
                return Err(format!("Invalid interior or mirror in reflection {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::VectorTranslatedPolygon(source, start, end)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || self.position(start).is_none() || self.position(end).is_none()
                    || self.polygon_points(o.id).is_none())
            {
                return Err(format!(
                    "Invalid interior or vector in translation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RotatedPolygon(source, center, _) | Kind::ScaledPolygon(source, center, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || self.position(center).is_none() || self.polygon_points(o.id).is_none())
            {
                return Err(format!("Invalid interior or center in transform {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::AngleRotatedPolygon(source, center, first, vertex, last)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || [center, first, vertex, last].iter().any(|&id| self.position(id).is_none())
                    || self.polygon_points(o.id).is_none())
            {
                return Err(format!(
                    "Invalid interior in marked-angle rotation {}",
                    o.id
                ));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RatioScaledPolygon(source, center, origin, base, target)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Polygon(_)))
                    || [center, origin, base, target].iter().any(|&id| self.position(id).is_none())
                    || self.polygon_points(o.id).is_none())
            {
                return Err(format!(
                    "Invalid interior in marked-ratio dilation {}",
                    o.id
                ));
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
                && matches!(o.kind, Kind::RotatedSegment(source, center, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Segment(_, _)))
                    || self.line_points(o.id).is_none()
                    || self.objects.get(center.saturating_sub(1)).is_none_or(|object| object.deleted || !is_point_kind(&object.kind) || self.position(center).is_none()))
            {
                return Err(format!("Invalid segment or center in rotation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RotatedCircle(source, center, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)))
                    || self.circle_points(o.id).is_none()
                    || self.objects.get(center.saturating_sub(1)).is_none_or(|object| object.deleted || !is_point_kind(&object.kind) || self.position(center).is_none()))
            {
                return Err(format!("Invalid circle or center in rotation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::AngleRotatedPoint(source, center, first, vertex, last)
                    if [source, center, first, vertex, last].iter().any(|&id| {
                        self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                            object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()
                    })
                }))
            {
                return Err(format!("Invalid point in marked-angle rotation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::AngleRotatedSegment(source, center, first, vertex, last)
                if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Segment(_, _)))
                || self.line_points(o.id).is_none()
                || [center, first, vertex, last].iter().any(|&id| {
                    self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                        object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()
                    })
                }))
            {
                return Err(format!("Invalid segment in marked-angle rotation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::AngleRotatedCircle(source, center, first, vertex, last)
                if !matches!(self.objects.get(source.saturating_sub(1)).map(|object| &object.kind), Some(Kind::Circle(_, _) | Kind::CircleBySegment(_, _)))
                || self.circle_points(o.id).is_none()
                || [center, first, vertex, last].iter().any(|&id| {
                    self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                        object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()
                    })
                }))
            {
                return Err(format!("Invalid circle in marked-angle rotation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RatioScaledPoint(source, center, origin, base, target)
                    if [source, center, origin, base, target].iter().any(|&id| {
                        self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                            object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()
                        })
                    }) || self.ratio_for_points(origin, base, target).is_none())
            {
                return Err(format!("Invalid point in marked-ratio dilation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::ScaledCircle(source, center, _)
                    if self.circle_points(source).is_none() || self.position(center).is_none() || self.circle_points(o.id).is_none())
            {
                return Err(format!("Invalid circle or center in dilation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::ScaledSegment(source, center, _)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Segment(_, _)))
                    || self.objects.get(center.saturating_sub(1)).is_none_or(|o| o.deleted || !is_point_kind(&o.kind) || self.position(center).is_none())
                    || self.line_points(o.id).is_none())
            {
                return Err(format!("Invalid segment or center in dilation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RatioScaledCircle(source, center, origin, base, target)
                    if self.circle_points(source).is_none()
                    || self.position(center).is_none()
                    || [center, origin, base, target].iter().any(|&id| {
                        self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                            object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()
                        })
                    })
                    || self.ratio_for_points(origin, base, target).is_none()
                    || self.circle_points(o.id).is_none())
            {
                return Err(format!("Invalid circle in marked-ratio dilation {}", o.id));
            }
            if i >= self.source_count
                && matches!(o.kind, Kind::RatioScaledSegment(source, center, origin, base, target)
                    if !matches!(self.objects.get(source.saturating_sub(1)).map(|o| &o.kind), Some(Kind::Segment(_, _)))
                    || [center, origin, base, target].iter().any(|&id| {
                        self.objects.get(id.saturating_sub(1)).is_none_or(|object| {
                            object.deleted || !is_point_kind(&object.kind) || self.position(id).is_none()
                        })
                    }) || self.ratio_for_points(origin, base, target).is_none()
                    || self.line_points(o.id).is_none())
            {
                return Err(format!("Invalid segment in marked-ratio dilation {}", o.id));
            }
            if i >= self.source_count
                && matches!(
                    o.kind,
                    Kind::PolarTranslatedPoint(_, _, _, _)
                        | Kind::FixedRectangularTranslatedPoint(_, _, _, _, _)
                        | Kind::MarkedDistanceTranslatedPoint(_, _, _)
                        | Kind::MixedRectangularTranslatedPoint(_, _, _, _)
                        | Kind::RectangularMarkedTranslatedPoint(_, _, _)
                        | Kind::VectorTranslatedPoint(_, _, _)
                        | Kind::ReflectedPoint(_, _)
                        | Kind::ScaledPoint(_, _, _)
                        | Kind::RotatedPoint(_, _, _)
                        | Kind::AngleRotatedPoint(_, _, _, _, _)
                        | Kind::RatioScaledPoint(_, _, _, _, _)
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
        let distance_x_axis = match object.kind {
            Kind::Free(_)
                if matches!(
                    self.objects.get(object.id).map(|next| &next.kind),
                    Some(Kind::DistanceAxis(origin, _, 1)) if *origin == object.id
                ) =>
            {
                object.id.checked_add(1)
            }
            Kind::DistanceAxis(_, _, 1) => Some(object.id),
            Kind::DistanceAxis(_, _, 0) => object.id.checked_sub(1),
            Kind::DistanceVerticalUnit(axis, _, _) => Some(axis),
            Kind::GraphAxis(_, unit, 0) => match self
                .objects
                .get(unit.saturating_sub(1))
                .map(|other| &other.kind)
            {
                Some(Kind::DistanceVerticalUnit(axis, _, _)) => Some(*axis),
                _ => None,
            },
            Kind::CoordinateSystem(axis, _)
                if matches!(
                    self.objects
                        .get(axis.saturating_sub(1))
                        .map(|other| &other.kind),
                    Some(Kind::DistanceAxis(_, _, 1))
                ) =>
            {
                Some(axis)
            }
            _ => None,
        };
        if let Some(x_axis_id) = distance_x_axis {
            let Kind::DistanceAxis(origin_id, first_distance, 1) = self
                .objects
                .get(x_axis_id.saturating_sub(1))
                .ok_or("Missing defining-distance X axis")?
                .kind
            else {
                return Err("Invalid defining-distance X axis".into());
            };
            let new_origin =
                if origin_id.checked_add(1) == Some(x_axis_id) && origin_id > self.source_count {
                    let point = self
                        .objects
                        .get(origin_id - 1)
                        .ok_or("Missing graph origin")?;
                    let Kind::Free(position) = point.kind else {
                        return Err("Invalid defining-distance origin".into());
                    };
                    Some(graph::GraphPoint {
                        x: position.x,
                        y: position.y,
                    })
                } else {
                    None
                };
            let y = self
                .objects
                .get(x_axis_id)
                .ok_or("Missing defining-distance Y axis")?;
            let (distances, frame_id) = match y.kind {
                Kind::DistanceAxis(other, second_distance, 0) if other == origin_id => {
                    (vec![first_distance, second_distance], x_axis_id + 2)
                }
                Kind::DistanceVerticalUnit(host, step, _)
                    if host == x_axis_id && step.is_finite() && step > 0. =>
                {
                    let y_axis = self
                        .objects
                        .get(x_axis_id + 1)
                        .ok_or("Missing defining-distance Y axis")?;
                    if !matches!(y_axis.kind, Kind::GraphAxis(origin, unit, 0) if origin == origin_id && unit == x_axis_id + 1)
                    {
                        return Err("Invalid defining-distance Y axis".into());
                    }
                    (vec![first_distance], x_axis_id + 3)
                }
                _ => return Err("Unsupported defining-distance construction".into()),
            };
            let frame = self
                .objects
                .get(frame_id - 1)
                .ok_or("Missing defining-distance frame")?;
            if !matches!(frame.kind, Kind::CoordinateSystem(a,b) if a == x_axis_id && b == frame_id - 1)
            {
                return Err("Invalid defining-distance frame".into());
            }
            let (bytes, offsets) =
                graph::distance_coordinate_groups(x_axis_id, origin_id, &distances, new_origin)?;
            let first_id = if new_origin.is_some() {
                origin_id
            } else {
                x_axis_id
            };
            let index = object
                .id
                .checked_sub(first_id)
                .ok_or("Invalid graph object ID")?;
            let end = *offsets.get(index + 1).ok_or("Invalid graph object ID")?;
            let mut group = bytes[offsets[index]..end].to_vec();
            if !object.visible {
                group[10] |= 1;
            }
            if matches!(object.kind, Kind::CoordinateSystem(_, _)) {
                let (form, dotted) = self
                    .frame_styles
                    .get(&object.id)
                    .copied()
                    .unwrap_or((self.grid_form[object.page], self.grid_dotted[object.page]));
                graph::set_coordinate_system_style(&mut group, form, dotted)?;
            }
            if let Kind::DistanceVerticalUnit(_, step, independent) = object.kind {
                graph::set_distance_vertical_unit_form(&mut group, independent, step)?;
            }
            return Ok(Some(group));
        }
        let circle_axis_start = match object.kind {
            Kind::CircleAxis(_, 1) => Some(object.id),
            Kind::CircleAxis(_, 0) => object.id.checked_sub(1),
            Kind::CoordinateSystem(x_axis, _)
                if matches!(
                    self.objects.get(x_axis.saturating_sub(1)).map(|o| &o.kind),
                    Some(Kind::CircleAxis(_, 1))
                ) =>
            {
                Some(x_axis)
            }
            _ => None,
        };
        if let Some(first) = circle_axis_start {
            if first == 0 {
                return Err("Invalid unit-circle graph object ID".into());
            }
            let [Some(x_axis), Some(y_axis), Some(frame)] = self
                .objects
                .get(first - 1..first + 2)
                .map(|slice| [slice.get(0), slice.get(1), slice.get(2)])
                .unwrap_or([None; 3])
            else {
                return Err("Incomplete unit-circle coordinate system".into());
            };
            let Kind::CircleAxis(circle, 1) = x_axis.kind else {
                return Err("Invalid unit-circle X axis".into());
            };
            if !matches!(y_axis.kind, Kind::CircleAxis(other, 0) if other == circle)
                || !matches!(frame.kind, Kind::CoordinateSystem(a, b) if a == first && b == first + 1)
            {
                return Err("Invalid unit-circle coordinate system".into());
            }
            let bytes = graph::unit_circle_groups(first, circle)?;
            let offsets = [0, 116, 232, 312];
            let index = object
                .id
                .checked_sub(first)
                .ok_or("Invalid graph object ID")?;
            if index >= 3 {
                return Err("Invalid graph object ID".into());
            }
            let mut group = bytes[offsets[index]..offsets[index + 1]].to_vec();
            if !object.visible {
                group[10] |= 1;
            }
            if matches!(object.kind, Kind::CoordinateSystem(_, _)) {
                let (form, dotted) = self
                    .frame_styles
                    .get(&object.id)
                    .copied()
                    .unwrap_or((self.grid_form[object.page], self.grid_dotted[object.page]));
                graph::set_coordinate_system_style(&mut group, form, dotted)?;
            }
            return Ok(Some(group));
        }
        let shared_x_unit = match object.kind {
            Kind::GraphHorizontalUnit(_, _) => Some(object.id),
            Kind::GraphAxis(_, unit, 1 | 0x1_0001) => Some(unit),
            Kind::GraphVerticalUnit(unit, _) => Some(unit),
            Kind::GraphAxis(_, y_unit, 0) => {
                match y_unit
                    .checked_sub(1)
                    .and_then(|at| self.objects.get(at))
                    .map(|o| &o.kind)
                {
                    Some(Kind::GraphVerticalUnit(unit, _)) => Some(*unit),
                    _ => None,
                }
            }
            Kind::CoordinateSystem(x_axis, _) => {
                match x_axis
                    .checked_sub(1)
                    .and_then(|at| self.objects.get(at))
                    .map(|o| &o.kind)
                {
                    Some(Kind::GraphAxis(_, unit, 1 | 0x1_0001)) => Some(*unit),
                    _ => None,
                }
            }
            _ => None,
        };
        if let Some(x_unit_id) = shared_x_unit
            && x_unit_id > self.source_count
            && let Some(Kind::GraphHorizontalUnit(origin_id, _)) =
                self.objects.get(x_unit_id - 1).map(|o| &o.kind)
            && !(*origin_id > self.source_count && origin_id.checked_add(1) == Some(x_unit_id))
        {
            let step: f64 = 37.79527559055118;
            let [
                Some(x_unit),
                Some(x_axis),
                Some(y_unit),
                Some(y_axis),
                Some(frame),
            ] = self
                .objects
                .get(x_unit_id - 1..x_unit_id + 4)
                .map(|slice| {
                    [
                        slice.get(0),
                        slice.get(1),
                        slice.get(2),
                        slice.get(3),
                        slice.get(4),
                    ]
                })
                .unwrap_or([None; 5])
            else {
                return Err("Incomplete coordinate-system group".into());
            };
            let origin_id = *origin_id;
            let y_step = match y_unit.kind {
                Kind::GraphVerticalUnit(host, value)
                    if host == x_unit_id && value.is_finite() && value > 0.0 =>
                {
                    value
                }
                _ => return Err("Unsupported vertical unit".into()),
            };
            let form = self
                .frame_styles
                .get(&frame.id)
                .map_or(self.grid_form[object.page], |style| style.0);
            if !matches!(x_unit.kind, Kind::GraphHorizontalUnit(host, value) if host == origin_id && value == step)
                || !matches!(x_axis.kind, Kind::GraphAxis(a,b,1 | 0x1_0001) if a == origin_id && b == x_unit_id)
                || !matches!(y_axis.kind, Kind::GraphAxis(a,b,0) if a == origin_id && b == x_unit_id+2)
                || !matches!(frame.kind, Kind::CoordinateSystem(a,b) if a == x_unit_id+1 && b == x_unit_id+3)
            {
                return Err("Unsupported coordinate-system construction".into());
            }
            let point = self
                .position(origin_id)
                .ok_or("Undefined coordinate-system origin")?;
            let bytes = graph::coordinate_system_groups_at_point(
                x_unit_id,
                origin_id,
                GraphPoint {
                    x: point.x,
                    y: point.y,
                },
            )?;
            let offsets = [0, 80, 200, 280, 400, 480];
            let index = object
                .id
                .checked_sub(x_unit_id)
                .ok_or("Invalid graph object ID")?;
            if index >= 5 {
                return Err("Invalid graph object ID".into());
            }
            let mut group = bytes[offsets[index]..offsets[index + 1]].to_vec();
            if !object.visible {
                group[10] |= 1;
            }
            if matches!(object.kind, Kind::CoordinateSystem(_, _)) {
                let (form, dotted) = self
                    .frame_styles
                    .get(&object.id)
                    .copied()
                    .unwrap_or((self.grid_form[object.page], self.grid_dotted[object.page]));
                graph::set_coordinate_system_style(&mut group, form, dotted)?;
            }
            if matches!(object.kind, Kind::GraphVerticalUnit(_, _)) {
                graph::set_vertical_unit_form(&mut group, form, y_step)?;
            }
            if let Kind::GraphAxis(_, _, axis @ (1 | 0x1_0001)) = object.kind {
                graph::set_x_axis_trigonometric(&mut group, axis == 0x1_0001)?;
            }
            return Ok(Some(group));
        }
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
        let form = self
            .frame_styles
            .get(&frame.id)
            .map_or(self.grid_form[object.page], |style| style.0);
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
        } else if matches!(object.kind, Kind::GraphVerticalUnit(_, _)) {
            // The native hidden vertical-unit group drops its 0x40000
            // presentation flag instead of combining it with the hide bit.
            group[10] = 1;
        } else {
            group[10] |= 1;
        }
        if matches!(object.kind, Kind::CoordinateSystem(_, _)) {
            let (form, dotted) = self
                .frame_styles
                .get(&object.id)
                .copied()
                .unwrap_or((form, self.grid_dotted[object.page]));
            graph::set_coordinate_system_style(&mut group, form, dotted)?;
        }
        if matches!(object.kind, Kind::GraphVerticalUnit(_, _)) {
            graph::set_vertical_unit_form(&mut group, form, y_step)?;
        }
        if let Kind::GraphAxis(_, _, axis @ (1 | 0x1_0001)) = object.kind {
            graph::set_x_axis_trigonometric(&mut group, axis == 0x1_0001)?;
        }
        Ok(Some(group))
    }
    fn patch_marked_coordinate_link(&self, mut out: Vec<u8>) -> Result<Vec<u8>, String> {
        let Some(frame) = self.active_coordinate_system() else {
            return Ok(out);
        };
        let frame = u32::try_from(frame).map_err(|_| "Too many objects")?;
        let rs = records(&out)?;
        if let Some(link) = rs.iter().find(|record| record.kind == 0x232d) {
            if link.data.len() != 4 {
                return Err("Invalid coordinate-system page link".into());
            }
            out[link.data.clone()].copy_from_slice(&frame.to_le_bytes());
        } else {
            let trailer = rs
                .iter()
                .find(|record| record.kind == 0x03e9)
                .ok_or("Missing document trailer")?
                .start;
            let mut link = Vec::new();
            put_record(&mut link, 0x232d, &frame.to_le_bytes());
            out.splice(trailer..trailer, link);
        }
        Ok(out)
    }
    fn patch_defined_coordinate_grid_style(&self, mut out: Vec<u8>) -> Result<Vec<u8>, String> {
        if !self.objects.iter().skip(self.source_count).any(|object| {
            !object.deleted
                && matches!(
                    object.kind,
                    Kind::CircleAxis(_, _) | Kind::DistanceAxis(_, _, _)
                )
        }) {
            return Ok(out);
        }
        let rs = records(&out)?;
        let styles: Vec<_> = rs.iter().filter(|record| record.kind == 0x04b3).collect();
        if styles.iter().any(|record| {
            record.data.len() >= 4 && out[record.data.start + 2..record.data.start + 4] == [14, 0]
        }) {
            return Ok(out);
        }
        // Sketchpad adds this page style for circle- and distance-defined frames.
        // Without it the native app displays the axes but omits the grid lines.
        let mut style = Vec::new();
        put_record(
            &mut style,
            0x04b3,
            &[0, 0, 14, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
        );
        let anchor = styles
            .last()
            .copied()
            .or_else(|| rs.iter().find(|record| record.kind == 0x04b1))
            .ok_or("Missing page style records")?;
        let at = anchor.data.end + anchor.data.len() % 2;
        out.splice(at..at, style);
        Ok(out)
    }
    fn multi_page_local_reference(
        &self,
        page: usize,
        current_id: usize,
        reference: usize,
    ) -> Result<u32, String> {
        let target = self
            .objects
            .get(reference.saturating_sub(1))
            .ok_or("Missing multi-page construction parent")?;
        if reference == 0 || reference >= current_id || target.page != page || target.deleted {
            return Err(
                "Multi-page construction parent must precede its child on the same page".into(),
            );
        }
        let preceding = self
            .objects
            .iter()
            .take(self.source_count)
            .filter(|object| object.page == page)
            .chain(
                self.objects
                    .iter()
                    .skip(self.source_count)
                    .take_while(|object| object.id < current_id)
                    .filter(|object| object.page == page),
            );
        let ordinal = preceding
            .enumerate()
            .find_map(|(index, object)| (object.id == reference).then_some(index + 1))
            .ok_or("Missing page-local construction parent")?;
        u32::try_from(ordinal).map_err(|_| "Too many objects on page".into())
    }
    fn append_multi_page_reference_group(
        groups: &mut Vec<u8>,
        object: &Object,
        code: u16,
        refs: &[u32],
    ) -> Result<(), String> {
        let mut object_header = header(code);
        if !object.visible {
            object_header[2] = 1;
        }
        object_header[4] = u8::from(object.tracing);
        if let Some(style) = object.visual_style {
            style_records::set_visual_style(&mut object_header, style)?;
        }
        put_record(groups, 0x07d0, &object_header);
        if is_point_kind(&object.kind) {
            if let Some(label) = &object.label {
                groups.extend_from_slice(&crate::text_records::encode_point_label_records(label));
            }
        }
        let mut references = Vec::with_capacity(4 + refs.len() * 4);
        references.extend_from_slice(
            &u32::try_from(refs.len())
                .map_err(|_| "Too many construction parents")?
                .to_le_bytes(),
        );
        for reference in refs {
            references.extend_from_slice(&reference.to_le_bytes());
        }
        put_record(groups, 0x07d2, &references);
        put_record(groups, 0x07d7, &[]);
        Ok(())
    }
    fn encode_multi_page_additions(&self) -> Result<Vec<u8>, String> {
        let original = Self::from_bytes(&self.original, None)?;
        if self.page_count < 2
            || original.page_count != self.page_count
            || self.source_count > self.objects.len()
            || original.snap_points != self.snap_points
            || original.grid_dotted != self.grid_dotted
            || original.grid_form != self.grid_form
            || original.angle_unit_radians != self.angle_unit_radians
            || original.marked_coordinate_system != self.marked_coordinate_system
            || original.frame_styles != self.frame_styles
            || self.objects[self.source_count..].iter().any(|object| {
                object.deleted
                    || object.page >= self.page_count
                    || !is_multi_page_addition_kind(&object.kind)
            })
        {
            return Err("Multi-page edit contains unsupported additions".into());
        }
        let rs = records(&self.original)?;
        let mut out = self.encode_multi_page_source_state()?;
        let preambles: Vec<_> = rs.iter().filter(|record| record.kind == 0x3e8).collect();
        if preambles.len() != self.page_count {
            return Err("Page preamble count mismatch".into());
        }
        let mut insertions = Vec::new();
        for (page, preamble) in preambles.iter().enumerate() {
            if preamble.data.len() < 28 {
                return Err("Short page preamble".into());
            }
            let next_page = preambles.get(page + 1).map_or(out.len(), |next| next.start);
            let trailer = rs
                .iter()
                .find(|record| {
                    record.kind == 0x3e9
                        && record.start > preamble.start
                        && record.start < next_page
                })
                .ok_or("Missing page trailer")?
                .start;
            let existing = original
                .objects
                .iter()
                .filter(|object| object.page == page)
                .count();
            if u32_at(&self.original, preamble.data.start + 24) as usize != existing {
                return Err("Page object count differs from its preamble".into());
            }
            let new_objects: Vec<_> = self.objects[self.source_count..]
                .iter()
                .filter(|object| object.page == page)
                .collect();
            if new_objects.is_empty() {
                continue;
            }
            let last_group_end = rs
                .iter()
                .filter(|record| {
                    record.kind == 0x7d7 && record.start > preamble.start && record.start < trailer
                })
                .map(|record| record.data.end)
                .max()
                .unwrap_or(preamble.data.end);
            let insert_at = rs
                .iter()
                .find(|record| {
                    matches!(record.kind, 0x2328 | 0x232d)
                        && record.start >= last_group_end
                        && record.start < trailer
                })
                .map(|record| record.start)
                .unwrap_or(trailer);
            let mut groups = Vec::new();
            for (added_index, object) in new_objects.into_iter().enumerate() {
                match &object.kind {
                    Kind::Free(point) => {
                        if !point.x.is_finite() || !point.y.is_finite() {
                            return Err("Multi-page point coordinates must be finite".into());
                        }
                        let mut object_header = header(0);
                        if !object.visible {
                            object_header[2] = 1;
                        }
                        object_header[4] = u8::from(object.tracing);
                        if let Some(style) = object.visual_style {
                            style_records::set_visual_style(&mut object_header, style)?;
                        }
                        put_record(&mut groups, 0x07d0, &object_header);
                        if let Some(label) = &object.label {
                            groups.extend_from_slice(
                                &crate::text_records::encode_point_label_records(label),
                            );
                        }
                        let mut coordinates = Vec::with_capacity(16);
                        coordinates.extend_from_slice(&point.x.to_le_bytes());
                        coordinates.extend_from_slice(&point.y.to_le_bytes());
                        put_record(&mut groups, 0x0899, &coordinates);
                        put_record(&mut groups, 0x07d7, &[]);
                    }
                    Kind::PointOnSegment(host, t) | Kind::PointOnArc(host, t) => {
                        if !t.is_finite() || !(0.0..=1.0).contains(t) {
                            return Err("Invalid path-point parameter".into());
                        }
                        let host_object = self
                            .objects
                            .get(host.saturating_sub(1))
                            .ok_or("Missing point host")?;
                        if host_object.page != page
                            || matches!(object.kind, Kind::PointOnSegment(_, _))
                                && !matches!(host_object.kind, Kind::Segment(_, _))
                            || matches!(object.kind, Kind::PointOnArc(_, _))
                                && !is_arc_kind(&host_object.kind)
                        {
                            return Err("Invalid point host on this page".into());
                        }
                        let local_host = self.multi_page_local_reference(page, object.id, *host)?;
                        let mut object_header = header(15);
                        if !object.visible {
                            object_header[2] = 1;
                        }
                        object_header[4] = u8::from(object.tracing);
                        if let Some(style) = object.visual_style {
                            style_records::set_visual_style(&mut object_header, style)?;
                        }
                        put_record(&mut groups, 0x07d0, &object_header);
                        if let Some(label) = &object.label {
                            groups.extend_from_slice(
                                &crate::text_records::encode_point_label_records(label),
                            );
                        }
                        let mut references = Vec::with_capacity(8);
                        references.extend_from_slice(&1u32.to_le_bytes());
                        references.extend_from_slice(&local_host.to_le_bytes());
                        put_record(&mut groups, 0x07d2, &references);
                        let code = if matches!(object.kind, Kind::PointOnArc(_, _)) {
                            67
                        } else {
                            14
                        };
                        let mut constraint = vec![15, 0, code, 0];
                        constraint.extend_from_slice(&t.to_le_bytes());
                        put_record(&mut groups, 0x07d3, &constraint);
                        put_record(&mut groups, 0x07d7, &[]);
                    }
                    Kind::PointOnCircle(host, x, y) => {
                        if !x.is_finite() || !y.is_finite() || (x * x + y * y - 1.).abs() > 1e-6 {
                            return Err("Invalid circle-point parameter".into());
                        }
                        let host_object = self
                            .objects
                            .get(host.saturating_sub(1))
                            .ok_or("Missing point host")?;
                        if host_object.page != page || !is_circle_kind(&host_object.kind) {
                            return Err("Invalid circle host on this page".into());
                        }
                        let local_host = self.multi_page_local_reference(page, object.id, *host)?;
                        let mut object_header = header(15);
                        if !object.visible {
                            object_header[2] = 1;
                        }
                        object_header[4] = u8::from(object.tracing);
                        if let Some(style) = object.visual_style {
                            style_records::set_visual_style(&mut object_header, style)?;
                        }
                        put_record(&mut groups, 0x07d0, &object_header);
                        if let Some(label) = &object.label {
                            groups.extend_from_slice(
                                &crate::text_records::encode_point_label_records(label),
                            );
                        }
                        let mut references = Vec::with_capacity(8);
                        references.extend_from_slice(&1u32.to_le_bytes());
                        references.extend_from_slice(&local_host.to_le_bytes());
                        put_record(&mut groups, 0x07d2, &references);
                        let mut constraint = vec![15, 0, 28, 0];
                        constraint.extend_from_slice(&x.to_le_bytes());
                        constraint.extend_from_slice(&y.to_le_bytes());
                        put_record(&mut groups, 0x07d3, &constraint);
                        put_record(&mut groups, 0x07d7, &[]);
                    }
                    Kind::Segment(a, b)
                    | Kind::Circle(a, b)
                    | Kind::Line(a, b)
                    | Kind::Ray(a, b) => {
                        if a == b
                            || !self
                                .objects
                                .get(a.saturating_sub(1))
                                .is_some_and(|parent| is_point_kind(&parent.kind))
                            || !self
                                .objects
                                .get(b.saturating_sub(1))
                                .is_some_and(|parent| is_point_kind(&parent.kind))
                            || self
                                .position(*a)
                                .zip(self.position(*b))
                                .is_none_or(|(first, second)| first.distance(second) <= 1e-9)
                        {
                            return Err(
                                "Straight object or circle needs two distinct points".into()
                            );
                        }
                        let first = self.multi_page_local_reference(page, object.id, *a)?;
                        let second = self.multi_page_local_reference(page, object.id, *b)?;
                        let code = match object.kind {
                            Kind::Segment(_, _) => 2,
                            Kind::Circle(_, _) => 3,
                            Kind::Line(_, _) => 63,
                            Kind::Ray(_, _) => 64,
                            _ => unreachable!(),
                        };
                        let mut object_header = header(code);
                        if !object.visible {
                            object_header[2] = 1;
                        }
                        object_header[4] = u8::from(object.tracing);
                        if let Some(style) = object.visual_style {
                            style_records::set_visual_style(&mut object_header, style)?;
                        }
                        put_record(&mut groups, 0x07d0, &object_header);
                        let mut references = Vec::with_capacity(12);
                        references.extend_from_slice(&2u32.to_le_bytes());
                        references.extend_from_slice(&first.to_le_bytes());
                        references.extend_from_slice(&second.to_le_bytes());
                        put_record(&mut groups, 0x07d2, &references);
                        put_record(&mut groups, 0x07d7, &[]);
                    }
                    Kind::Midpoint(a, b) => {
                        let host = object
                            .midpoint_host
                            .ok_or("Midpoint needs a segment host")?;
                        if !matches!(self.objects.get(host.saturating_sub(1)).map(|parent| &parent.kind), Some(Kind::Segment(first, second)) if (*first == *a && *second == *b) || (*first == *b && *second == *a))
                        {
                            return Err("Midpoint host has different endpoints".into());
                        }
                        let local_host = self.multi_page_local_reference(page, object.id, host)?;
                        let mut object_header = header(1);
                        if !object.visible {
                            object_header[2] = 1;
                        }
                        object_header[4] = u8::from(object.tracing);
                        if let Some(style) = object.visual_style {
                            style_records::set_visual_style(&mut object_header, style)?;
                        }
                        put_record(&mut groups, 0x07d0, &object_header);
                        if let Some(label) = &object.label {
                            groups.extend_from_slice(
                                &crate::text_records::encode_point_label_records(label),
                            );
                        }
                        let mut references = Vec::with_capacity(8);
                        references.extend_from_slice(&1u32.to_le_bytes());
                        references.extend_from_slice(&local_host.to_le_bytes());
                        put_record(&mut groups, 0x07d2, &references);
                        put_record(&mut groups, 0x07d7, &[]);
                    }
                    Kind::CircleBySegment(center, radius) => {
                        if !self
                            .objects
                            .get(center.saturating_sub(1))
                            .is_some_and(|parent| is_point_kind(&parent.kind))
                            || !matches!(
                                self.objects
                                    .get(radius.saturating_sub(1))
                                    .map(|parent| &parent.kind),
                                Some(Kind::Segment(_, _))
                            )
                            || self
                                .line_points(*radius)
                                .is_none_or(|(first, second, _)| first.distance(second) <= 1e-9)
                        {
                            return Err(
                                "Circle by segment needs a point and nonzero segment".into()
                            );
                        }
                        let local_center =
                            self.multi_page_local_reference(page, object.id, *center)?;
                        let local_radius =
                            self.multi_page_local_reference(page, object.id, *radius)?;
                        let mut object_header = header(4);
                        if !object.visible {
                            object_header[2] = 1;
                        }
                        object_header[4] = u8::from(object.tracing);
                        if let Some(style) = object.visual_style {
                            style_records::set_visual_style(&mut object_header, style)?;
                        }
                        put_record(&mut groups, 0x07d0, &object_header);
                        let mut references = Vec::with_capacity(12);
                        references.extend_from_slice(&2u32.to_le_bytes());
                        references.extend_from_slice(&local_center.to_le_bytes());
                        references.extend_from_slice(&local_radius.to_le_bytes());
                        put_record(&mut groups, 0x07d2, &references);
                        put_record(&mut groups, 0x07d7, &[]);
                    }
                    Kind::Parallel(point, line) | Kind::Perpendicular(point, line) => {
                        if !self
                            .objects
                            .get(point.saturating_sub(1))
                            .is_some_and(|parent| is_point_kind(&parent.kind))
                            || self
                                .line_points(*line)
                                .is_none_or(|(first, second, _)| first.distance(second) <= 1e-9)
                        {
                            return Err(
                                "Derived line needs a point and nonzero straight object".into()
                            );
                        }
                        let local_point =
                            self.multi_page_local_reference(page, object.id, *point)?;
                        let local_line = self.multi_page_local_reference(page, object.id, *line)?;
                        let code = if matches!(object.kind, Kind::Parallel(_, _)) {
                            6
                        } else {
                            5
                        };
                        Self::append_multi_page_reference_group(
                            &mut groups,
                            object,
                            code,
                            &[local_point, local_line],
                        )?;
                    }
                    Kind::Intersection(a, b)
                    | Kind::LineCircleIntersection(a, b, _)
                    | Kind::CircleCircleIntersection(a, b, _) => {
                        let code = match object.kind {
                            Kind::Intersection(_, _)
                                if self.line_points(*a).is_some()
                                    && self.line_points(*b).is_some() =>
                            {
                                9
                            }
                            Kind::LineCircleIntersection(_, _, variant)
                                if variant <= 1
                                    && self.line_points(*a).is_some()
                                    && self.circle_points(*b).is_some() =>
                            {
                                if variant == 0 {
                                    12
                                } else {
                                    11
                                }
                            }
                            Kind::CircleCircleIntersection(_, _, variant)
                                if variant <= 1
                                    && self.circle_points(*a).is_some()
                                    && self.circle_points(*b).is_some() =>
                            {
                                if variant == 0 {
                                    14
                                } else {
                                    13
                                }
                            }
                            _ => return Err("Intersection needs compatible paths".into()),
                        };
                        if self.position(object.id).is_none() {
                            return Err("Intersection has no finite point".into());
                        }
                        let local_a = self.multi_page_local_reference(page, object.id, *a)?;
                        let local_b = self.multi_page_local_reference(page, object.id, *b)?;
                        Self::append_multi_page_reference_group(
                            &mut groups,
                            object,
                            code,
                            &[local_a, local_b],
                        )?;
                    }
                    Kind::LinkPageButton {
                        target_page,
                        target_button_ordinal,
                        label,
                        anchor,
                    } => {
                        if *target_page >= self.page_count || *target_page == page {
                            return Err("Invalid Page Link destination".into());
                        }
                        let destination_buttons = self.link_destination_buttons(*target_page);
                        if target_button_ordinal.is_some_and(|ordinal| {
                            ordinal == 0
                                || destination_buttons.get(ordinal - 1).is_none_or(|id| {
                                    !matches!(
                                        self.objects[id - 1].kind,
                                        Kind::HideShowButton { .. }
                                    )
                                })
                        }) {
                            return Err("Page Link destination button is unavailable".into());
                        }
                        let mut group = LinkPageButtonGroup::from_original_template(
                            *target_page,
                            label,
                            *anchor,
                        )?;
                        group.set_target_button_ordinal(*target_button_ordinal)?;
                        group.set_page_object_index(existing + added_index)?;
                        group.set_hidden(!object.visible);
                        groups.extend_from_slice(&group.encode());
                        if let Some(ordinal) = target_button_ordinal {
                            let destination = &self.objects[destination_buttons[ordinal - 1] - 1];
                            let header = destination
                                .header_offset
                                .ok_or("Missing destination button header")?;
                            let header_index = rs
                                .iter()
                                .position(|record| {
                                    record.kind == 0x07d0 && record.data.start == header
                                })
                                .ok_or("Missing destination button group")?;
                            let action = rs[header_index + 1..]
                                .iter()
                                .take_while(|record| record.kind != 0x07d7)
                                .find(|record| record.kind == 0x0906 && record.data.len() >= 20)
                                .ok_or("Missing destination button action")?;
                            out[action.data.start + 18..action.data.start + 20]
                                .copy_from_slice(&1u16.to_le_bytes());
                        }
                    }
                    Kind::LinkUrlButton { url, label, anchor } => {
                        let mut group =
                            LinkUrlButtonGroup::from_original_template(url, label, *anchor)?;
                        group.set_page_object_index(existing + added_index)?;
                        group.set_hidden(!object.visible);
                        groups.extend_from_slice(&group.encode());
                    }
                    _ => return Err("Expected supported multi-page addition".into()),
                }
            }
            let count = u32::try_from(
                existing
                    + self.objects[self.source_count..]
                        .iter()
                        .filter(|object| object.page == page)
                        .count(),
            )
            .map_err(|_| "Too many objects on page")?;
            out[preamble.data.start + 24..preamble.data.start + 28]
                .copy_from_slice(&count.to_le_bytes());
            insertions.push((insert_at, groups));
        }
        for (insert_at, groups) in insertions.into_iter().rev() {
            out.splice(insert_at..insert_at, groups);
        }
        records(&out)?;
        Ok(out)
    }
    fn encode_multi_page_state(&self) -> Result<Vec<u8>, String> {
        if self.source_count != self.objects.len() {
            return Err("Multi-page state edit cannot add objects".into());
        }
        self.encode_multi_page_source_state()
    }
    fn encode_multi_page_source_state(&self) -> Result<Vec<u8>, String> {
        let original = Self::from_bytes(&self.original, None)?;
        if self.page_count < 2
            || self.image_read_only
            || self.source_count > self.objects.len()
            || original.objects.len() != self.source_count
            || original.page_count != self.page_count
            || original.snap_points != self.snap_points
            || original.grid_dotted != self.grid_dotted
            || original.grid_form != self.grid_form
            || original.angle_unit_radians != self.angle_unit_radians
            || original.marked_coordinate_system != self.marked_coordinate_system
            || original.frame_styles != self.frame_styles
        {
            return Err("Multi-page state edit changes the document structure".into());
        }
        let rs = records(&self.original)?;
        let mut out = self.original.clone();
        for (saved, current) in original
            .objects
            .iter()
            .zip(&self.objects[..self.source_count])
        {
            let mut expected = saved.clone();
            expected.visible = current.visible;
            if saved.visual_style.is_some() {
                expected.tracing = current.tracing;
            }
            if saved.visual_style.is_some() && current.visual_style.is_some() {
                expected.visual_style = current.visual_style;
            }
            if let (Kind::Free(_), Kind::Free(point)) = (&saved.kind, &current.kind) {
                if !point.x.is_finite() || !point.y.is_finite() {
                    return Err("Multi-page point coordinates must be finite".into());
                }
                expected.kind = current.kind.clone();
            }
            match (&saved.kind, &current.kind) {
                (Kind::PointOnSegment(old_host, _), Kind::PointOnSegment(host, t))
                    if old_host == host && t.is_finite() && (0.0..=1.0).contains(t) =>
                {
                    expected.kind = current.kind.clone();
                }
                (Kind::PointOnArc(old_host, _), Kind::PointOnArc(host, t))
                    if old_host == host && t.is_finite() && (0.0..=1.0).contains(t) =>
                {
                    expected.kind = current.kind.clone();
                }
                (Kind::PointOnCircle(old_host, _, _), Kind::PointOnCircle(host, x, y))
                    if old_host == host
                        && x.is_finite()
                        && y.is_finite()
                        && (x * x + y * y - 1.0).abs() <= 1e-6 =>
                {
                    expected.kind = current.kind.clone();
                }
                _ => {}
            }
            if let (
                Kind::HideShowButton {
                    targets: old_targets,
                    anchor: old_anchor,
                    ..
                },
                Kind::HideShowButton {
                    targets, anchor, ..
                },
            ) = (&saved.kind, &current.kind)
                && old_targets == targets
                && old_anchor == anchor
            {
                expected.kind = current.kind.clone();
            }
            if expected != *current
                || matches!(current.kind, Kind::Unsupported(_)) && saved.visible != current.visible
            {
                return Err("Multi-page state edit changes unsupported object data".into());
            }
            let header = current.header_offset.ok_or("Missing object header")?;
            if saved.tracing != current.tracing {
                let record = rs
                    .iter()
                    .find(|record| record.kind == 0x07d0 && record.data.start == header)
                    .ok_or("Missing traced object header")?;
                if record.data.len() < 5 {
                    return Err("Short traced object header".into());
                }
                out[header + 4] = (out[header + 4] & !1) | u8::from(current.tracing);
            }
            if saved.visual_style != current.visual_style {
                let record = rs
                    .iter()
                    .find(|record| record.kind == 0x07d0 && record.data.start == header)
                    .ok_or("Missing styled object header")?;
                let style = current.visual_style.ok_or("Missing object style")?;
                style_records::set_visual_style(&mut out[record.data.clone()], style)
                    .map_err(|error| format!("Invalid style in object {}: {error}", current.id))?;
            }
            if let (Kind::Free(before), Kind::Free(point)) = (&saved.kind, &current.kind)
                && before != point
            {
                let at = current
                    .point_offset
                    .ok_or("Missing free-point coordinates")?;
                out[at..at + 8].copy_from_slice(&point.x.to_le_bytes());
                out[at + 8..at + 16].copy_from_slice(&point.y.to_le_bytes());
            }
            if saved.kind != current.kind
                && matches!(
                    current.kind,
                    Kind::PointOnSegment(_, _)
                        | Kind::PointOnArc(_, _)
                        | Kind::PointOnCircle(_, _, _)
                )
            {
                let at = current
                    .constraint_offset
                    .ok_or("Missing point constraint")?;
                match &current.kind {
                    Kind::PointOnSegment(_, t) => {
                        if out.get(at..at + 4) != Some(&[15, 0, 14, 0]) {
                            return Err("Invalid segment-point constraint".into());
                        }
                        out.get_mut(at + 4..at + 12)
                            .ok_or("Short segment-point constraint")?
                            .copy_from_slice(&t.to_le_bytes());
                    }
                    Kind::PointOnArc(_, t) => {
                        if out.get(at..at + 4) != Some(&[15, 0, 67, 0]) {
                            return Err("Invalid arc-point constraint".into());
                        }
                        out.get_mut(at + 4..at + 12)
                            .ok_or("Short arc-point constraint")?
                            .copy_from_slice(&t.to_le_bytes());
                    }
                    Kind::PointOnCircle(_, x, y) => {
                        if out.get(at..at + 4) != Some(&[15, 0, 28, 0]) {
                            return Err("Invalid circle-point constraint".into());
                        }
                        out.get_mut(at + 4..at + 12)
                            .ok_or("Short circle-point constraint")?
                            .copy_from_slice(&x.to_le_bytes());
                        out.get_mut(at + 12..at + 20)
                            .ok_or("Short circle-point constraint")?
                            .copy_from_slice(&y.to_le_bytes());
                    }
                    _ => {}
                }
            }
            if let Kind::HideShowButton { action, label, .. } = &current.kind {
                let start_record = rs
                    .iter()
                    .position(|record| record.kind == 0x07d0 && record.data.start == header)
                    .ok_or("Missing saved Hide/Show button header")?;
                let end_record = rs[start_record..]
                    .iter()
                    .position(|record| record.kind == 0x07d7)
                    .map(|index| start_record + index)
                    .ok_or("Missing saved Hide/Show button terminator")?;
                let start = rs[start_record].start;
                let end = rs[end_record].data.end;
                let mut group = HideShowButtonGroup::parse(&out[start..end])?;
                group.set_action(*action);
                group.set_label(label)?;
                group.set_hidden(!current.visible);
                let encoded = group.encode();
                if encoded.len() != end - start {
                    return Err("Editing a saved button label cannot change its byte length".into());
                }
                out[start..end].copy_from_slice(&encoded);
            } else if saved.visible != current.visible {
                let mut class = u32_at(&out, header);
                if current.visible {
                    class &= !0x1_0000;
                } else {
                    class |= 0x1_0000;
                }
                out[header..header + 4].copy_from_slice(&class.to_le_bytes());
            }
        }
        records(&out)?;
        Ok(out)
    }
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        if !self.dirty {
            return Ok(self.original.clone());
        }
        if self.has_pending_multi_page_additions() {
            return self.encode_multi_page_additions();
        }
        if self.page_count > 1 {
            return self.encode_multi_page_state();
        }
        self.can_save()?;
        let rs = records(&self.original)?;
        let mut out = self.original.clone();
        if let Some(record) = rs.iter().find(|record| record.kind == 0x273a) {
            if record.data.len() != 2 {
                return Err("Invalid angle-unit record".into());
            }
            out[record.data.start..record.data.end]
                .copy_from_slice(&[u8::from(!self.angle_unit_radians), 0]);
        } else if self.angle_unit_radians {
            return Err("Missing angle-unit record".into());
        }
        for o in self.objects.iter().take(self.source_count) {
            if let Some(i) = o.header_offset {
                let mut class = u32_at(&out, i);
                if matches!(o.kind, Kind::GraphVerticalUnit(_, _)) {
                    let form = self
                        .coordinate_system_for_object(o.id)
                        .and_then(|frame| self.frame_styles.get(&frame).copied())
                        .map_or(self.grid_form[o.page], |style| style.0);
                    let code = if form == GridForm::Rectangular {
                        55
                    } else {
                        54
                    };
                    class = (class & !0xffff) | code;
                }
                if let Kind::DistanceVerticalUnit(_, _, independent) = o.kind {
                    let code = if independent { 57 } else { 56 };
                    class =
                        (class & !0x0004_ffff) | code | if independent { 0 } else { 0x0004_0000 };
                }
                if o.visible {
                    class &= !0x1_0000;
                } else {
                    class |= 0x1_0000;
                }
                out[i..i + 4].copy_from_slice(&class.to_le_bytes());
                if matches!(o.kind, Kind::VectorTranslatedPolygon(_, _, _))
                    && self.objects.iter().any(|candidate| {
                        !candidate.deleted
                            && matches!(&candidate.kind, Kind::Locus(refs, _) if refs.first() == Some(&o.id))
                    })
                {
                    out[i + 18..i + 20].copy_from_slice(&[0x39, 0x20]);
                }
            }
            if let (Kind::Free(p), Some(i)) = (&o.kind, o.point_offset) {
                out[i..i + 8].copy_from_slice(&p.x.to_le_bytes());
                out[i + 8..i + 16].copy_from_slice(&p.y.to_le_bytes());
            }
            if let Some(i) = o.constraint_offset {
                match o.kind {
                    Kind::GraphVerticalUnit(_, step)
                        if matches!(out.get(i..i + 4), Some([55, 0, 0, 0] | [54, 0, 0, 0])) =>
                    {
                        let form = self
                            .coordinate_system_for_object(o.id)
                            .and_then(|frame| self.frame_styles.get(&frame).copied())
                            .map_or(self.grid_form[o.page], |style| style.0);
                        let code: u16 = if form == GridForm::Rectangular {
                            55
                        } else {
                            54
                        };
                        out[i..i + 2].copy_from_slice(&code.to_le_bytes());
                        out[i + 4..i + 12].copy_from_slice(&step.to_le_bytes());
                    }
                    Kind::DistanceVerticalUnit(_, step, independent)
                        if matches!(out.get(i..i + 4), Some([56, 0, 0, 0] | [57, 0, 0, 0])) =>
                    {
                        let code: u16 = if independent { 57 } else { 56 };
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
                    targets,
                    action,
                    label,
                    anchor,
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
                group.set_targets(
                    &targets
                        .iter()
                        .map(|&target| target as u32)
                        .collect::<Vec<_>>(),
                )?;
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
            if let (
                Kind::AnimationButton {
                    targets,
                    label,
                    anchor,
                    settings,
                },
                Some(i),
            ) = (&o.kind, o.header_offset)
            {
                let start_record = rs
                    .iter()
                    .position(|record| record.kind == 0x7d0 && record.data.start == i)
                    .ok_or("Missing saved animation button header")?;
                let end_record = rs[start_record..]
                    .iter()
                    .position(|record| record.kind == 0x7d7)
                    .map(|index| start_record + index)
                    .ok_or("Missing saved animation button terminator")?;
                let start = rs[start_record].start;
                let end = rs[end_record].data.end;
                let mut group = AnimationButtonGroup::parse(&out[start..end])?;
                group.set_targets(
                    &targets
                        .iter()
                        .map(|&target| target as u32)
                        .collect::<Vec<_>>(),
                )?;
                group.set_label(label)?;
                group.set_screen_anchor(*anchor);
                group.set_settings_list(settings)?;
                group.set_hidden(!o.visible);
                let encoded = group.encode();
                if encoded.len() != end - start {
                    return Err(
                        "Editing a saved animation button label cannot change its byte length"
                            .into(),
                    );
                }
                out[start..end].copy_from_slice(&encoded);
            }
            if let (
                Kind::MovementButton {
                    source,
                    destination,
                    speed,
                    follow,
                    label,
                    anchor,
                },
                Some(i),
            ) = (&o.kind, o.header_offset)
            {
                let start_record = rs
                    .iter()
                    .position(|record| record.kind == 0x7d0 && record.data.start == i)
                    .ok_or("Missing saved Movement button header")?;
                let end_record = rs[start_record..]
                    .iter()
                    .position(|record| record.kind == 0x7d7)
                    .map(|index| start_record + index)
                    .ok_or("Missing saved Movement button terminator")?;
                let start = rs[start_record].start;
                let end = rs[end_record].data.end;
                let mut group = MovementButtonGroup::parse(&out[start..end])?;
                group.set_targets(*source as u32, *destination as u32)?;
                group.set_speed(*speed);
                group.set_follows_destination(*follow);
                group.set_label(label)?;
                group.set_screen_anchor(*anchor);
                group.set_hidden(!o.visible);
                let encoded = group.encode();
                if encoded.len() != end - start {
                    return Err(
                        "Editing a saved Movement button label cannot change its byte length"
                            .into(),
                    );
                }
                out[start..end].copy_from_slice(&encoded);
            }
            if let (
                Kind::ScrollButton {
                    target,
                    placement,
                    label,
                    anchor,
                },
                Some(i),
            ) = (&o.kind, o.header_offset)
            {
                let start_record = rs
                    .iter()
                    .position(|record| record.kind == 0x7d0 && record.data.start == i)
                    .ok_or("Missing saved Scroll button header")?;
                let end_record = rs[start_record..]
                    .iter()
                    .position(|record| record.kind == 0x7d7)
                    .map(|index| start_record + index)
                    .ok_or("Missing saved Scroll button terminator")?;
                let start = rs[start_record].start;
                let end = rs[end_record].data.end;
                let mut group = ScrollButtonGroup::parse(&out[start..end])?;
                group.set_target(*target as u32)?;
                group.set_placement(*placement);
                group.set_label(label)?;
                group.set_screen_anchor(*anchor);
                group.set_hidden(!o.visible);
                let encoded = group.encode();
                if encoded.len() != end - start {
                    return Err(
                        "Editing a saved Scroll button label cannot change its byte length".into(),
                    );
                }
                out[start..end].copy_from_slice(&encoded);
            }
            if let (
                Kind::PresentationButton {
                    targets,
                    mode,
                    label,
                    anchor,
                },
                Some(i),
            ) = (&o.kind, o.header_offset)
            {
                let start_record = rs
                    .iter()
                    .position(|record| record.kind == 0x7d0 && record.data.start == i)
                    .ok_or("Missing saved Presentation button header")?;
                let end_record = rs[start_record..]
                    .iter()
                    .position(|record| record.kind == 0x7d7)
                    .map(|index| start_record + index)
                    .ok_or("Missing saved Presentation button terminator")?;
                let start = rs[start_record].start;
                let end = rs[end_record].data.end;
                let mut group = PresentationButtonGroup::parse(&out[start..end])?;
                group.set_targets(
                    &targets
                        .iter()
                        .map(|&target| target as u32)
                        .collect::<Vec<_>>(),
                )?;
                group.set_mode(*mode);
                group.set_label(label)?;
                group.set_screen_anchor(*anchor);
                group.set_hidden(!o.visible);
                let encoded = group.encode();
                if encoded.len() != end - start {
                    return Err(
                        "Editing a saved Presentation button label cannot change its byte length"
                            .into(),
                    );
                }
                out[start..end].copy_from_slice(&encoded);
            }
            if let (Kind::LinkUrlButton { url, label, anchor }, Some(i)) =
                (&o.kind, o.header_offset)
            {
                let start_record = rs
                    .iter()
                    .position(|record| record.kind == 0x7d0 && record.data.start == i)
                    .ok_or("Missing saved Link URL button header")?;
                let end_record = rs[start_record..]
                    .iter()
                    .position(|record| record.kind == 0x7d7)
                    .map(|index| start_record + index)
                    .ok_or("Missing saved Link URL button terminator")?;
                let start = rs[start_record].start;
                let end = rs[end_record].data.end;
                let mut group = LinkUrlButtonGroup::parse(&out[start..end])?;
                group.set_url(url)?;
                group.set_label(label)?;
                group.set_screen_anchor(*anchor);
                group.set_hidden(!o.visible);
                let encoded = group.encode();
                if encoded.len() != end - start {
                    return Err(
                        "Editing a saved Link URL button cannot change its byte length".into(),
                    );
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
            .filter(|object| matches!(object.kind, Kind::CoordinateSystem(_, _)) && !object.deleted)
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
            let (form, dotted) = self
                .frame_styles
                .get(&object.id)
                .copied()
                .unwrap_or((self.grid_form[object.page], self.grid_dotted[object.page]));
            out[flag.data.start] = form.code();
            out[flag.data.start + 2] = u8::from(dotted);
        }
        for object in self
            .objects
            .iter()
            .take(self.source_count)
            .filter(|object| {
                !object.deleted && matches!(object.kind, Kind::GraphAxis(_, _, 1 | 0x1_0001))
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
            let out = self.patch_marked_coordinate_link(out)?;
            let out = self.patch_text_font_overrides(self.patch_visual_styles(
                self.patch_trace_flags(self.patch_text_and_labels(self.compact_deleted(out)?)?)?,
            )?)?;
            return self.patch_last_moved_marker(out);
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
                let mut group = if let Some(template) = &o.rich_text_override {
                    template.encode_group(text, *rect)?
                } else {
                    text_object::encode_plain_text_group(text, *rect)?
                };
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
                targets,
                action,
                label,
                anchor,
            } = &o.kind
            {
                let ids = targets
                    .iter()
                    .map(|&target| target as u32)
                    .collect::<Vec<_>>();
                let mut group =
                    HideShowButtonGroup::from_original_template_targets(&ids, label, *anchor)?;
                group.set_action(*action);
                group.set_hidden(!o.visible);
                added.extend_from_slice(&group.encode());
                continue;
            }
            if let Kind::AnimationButton {
                targets,
                label,
                anchor,
                settings,
            } = &o.kind
            {
                let ids = targets
                    .iter()
                    .map(|&target| target as u32)
                    .collect::<Vec<_>>();
                let mut group =
                    AnimationButtonGroup::from_original_template_targets(&ids, label, *anchor)?;
                group.set_hidden(!o.visible);
                group.set_settings_list(settings)?;
                added.extend_from_slice(&group.encode());
                continue;
            }
            if let Kind::MovementButton {
                source,
                destination,
                speed,
                follow,
                label,
                anchor,
            } = &o.kind
            {
                let mut group = MovementButtonGroup::from_original_template(
                    *source as u32,
                    *destination as u32,
                    label,
                    *anchor,
                )?;
                group.set_speed(*speed);
                group.set_follows_destination(*follow);
                group.set_hidden(!o.visible);
                added.extend_from_slice(&group.encode());
                continue;
            }
            if let Kind::ScrollButton {
                target,
                placement,
                label,
                anchor,
            } = &o.kind
            {
                let mut group =
                    ScrollButtonGroup::from_original_template(*target as u32, label, *anchor)?;
                group.set_placement(*placement);
                group.set_hidden(!o.visible);
                added.extend_from_slice(&group.encode());
                continue;
            }
            if let Kind::PresentationButton {
                targets,
                mode,
                label,
                anchor,
            } = &o.kind
            {
                let ids = targets
                    .iter()
                    .map(|&target| target as u32)
                    .collect::<Vec<_>>();
                let mut group =
                    PresentationButtonGroup::from_original_template_targets(&ids, label, *anchor)?;
                group.set_mode(*mode);
                group.set_hidden(!o.visible);
                added.extend_from_slice(&group.encode());
                continue;
            }
            if let Kind::LinkUrlButton { url, label, anchor } = &o.kind {
                let mut group = LinkUrlButtonGroup::from_original_template(url, label, *anchor)?;
                let index = self.objects[..o.id - 1]
                    .iter()
                    .filter(|object| !object.deleted)
                    .count();
                group.set_page_object_index(index)?;
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
                if let Some(anchor) = o.function_anchor {
                    function_records::set_group_expression_anchor(&mut group, anchor)?;
                }
                if let Some(name) = &o.function_name {
                    function_records::set_group_definition_name(&mut group, name)?;
                }
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::DerivativeFunction(source, expression) = o.kind {
                let mut group = function_records::encode_derivative_group(source, expression)?;
                if let Some(anchor) = o.function_anchor {
                    function_records::set_group_expression_anchor(&mut group, anchor)?;
                }
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::FunctionPlot(definition, frame, min, max, samples) = o.kind {
                let expression = self
                    .function_expression(definition)
                    .ok_or("Invalid function definition")?;
                let mut group = function_records::encode_plot_group_with_range(
                    expression, definition, frame, min, max, samples,
                )?;
                if !o.visible {
                    group[10] |= 1;
                }
                added.extend_from_slice(&group);
                continue;
            }
            if let Kind::ParametricPlot(x, y, frame, min, max, samples, mode) = o.kind {
                let mut group = function_records::encode_parametric_plot_group(
                    x,
                    y,
                    frame,
                    min,
                    max,
                    samples,
                    mode == PlotPointsMode::Polar,
                )?;
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
            if let Kind::Locus(ref refs, samples) = o.kind {
                let mut h = [
                    35, 0, 0, 0, 0, 0, 6, 0, 42, 0, 0x22, 1, 0x80, 0, 0, 0, 0xff, 0xff, 0x39, 0x20,
                    0xc1, 0xbd, 0xf0, 0xff, 0, 0, 0, 0,
                ];
                let target = *refs.first().ok_or("Locus has no traced object")?;
                let target_kind = &self
                    .objects
                    .get(target.saturating_sub(1))
                    .ok_or("Missing locus target")?
                    .kind;
                let straight_target = is_locus_straight_kind(target_kind);
                let curved_target = is_circle_kind(target_kind) || is_arc_kind(target_kind);
                let polygon_target = is_locus_polygon_kind(target_kind);
                if polygon_target {
                    h[8] = 43;
                    h[10] = 0x21;
                    h[12..16].copy_from_slice(&[0xff, 0xb2, 0x66, 0]);
                    h[17] = 0x7f;
                    if matches!(target_kind, Kind::PolarTranslatedPolygon(_, _, _, _)) {
                        h[18..20].copy_from_slice(&[0xf0, 0x3f]);
                    } else if matches!(
                        target_kind,
                        Kind::FixedRectangularTranslatedPolygon(_, _, _, _, _)
                    ) {
                        h[18..20].copy_from_slice(&[0, 0x40]);
                    }
                    h[20..24].copy_from_slice(
                        &if matches!(
                            target_kind,
                            Kind::VectorTranslatedPolygon(_, _, _)
                                | Kind::PolarTranslatedPolygon(_, _, _, _)
                                | Kind::FixedRectangularTranslatedPolygon(_, _, _, _, _)
                        ) {
                            2u32
                        } else {
                            1u32
                        }
                        .to_le_bytes(),
                    );
                } else if straight_target || curved_target {
                    h[8] = 43;
                    h[12..16].copy_from_slice(if curved_target {
                        &[0x66, 0xb2, 0x66, 0]
                    } else {
                        &[0x66, 0x66, 0xb2, 0]
                    });
                }
                if !o.visible {
                    h[2] = 1;
                }
                put_record(&mut added, 0x07d0, &h);
                let mut references = Vec::with_capacity(4 + refs.len() * 4);
                references.extend_from_slice(&(refs.len() as u32).to_le_bytes());
                for &id in refs {
                    references.extend_from_slice(&(id as u32).to_le_bytes());
                }
                put_record(&mut added, 0x07d2, &references);
                let driver = *refs.get(2).ok_or("Locus has no moving point")?;
                let circle_driver = matches!(
                    self.objects
                        .get(driver.saturating_sub(1))
                        .map(|object| &object.kind),
                    Some(Kind::PointOnCircle(_, _, _))
                );
                let mut range = Vec::with_capacity(40);
                range.extend_from_slice(&0f64.to_le_bytes());
                range.extend_from_slice(
                    &if circle_driver {
                        std::f64::consts::TAU
                    } else {
                        1.0
                    }
                    .to_le_bytes(),
                );
                range.extend_from_slice(&samples.to_le_bytes());
                range.extend_from_slice(&[
                    4,
                    0,
                    if circle_driver { 0 } else { 1 },
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    if straight_target || curved_target || polygon_target {
                        0
                    } else {
                        1
                    },
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ]);
                put_record(&mut added, 0x0902, &range);
                put_record(&mut added, 0x07d7, &[]);
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
            if let Kind::PolygonArea(polygon) = o.kind {
                let mut group = *include_bytes!("../assets/area_measurement_group.bin");
                group[48..52].copy_from_slice(&(polygon as u32).to_le_bytes());
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
                Kind::PolarTranslatedSegment(source, _, _, _) => (21, vec![source]),
                Kind::PolarTranslatedCircle(source, _, _, _) => (21, vec![source]),
                Kind::PolarTranslatedPolygon(source, _, _, _) => (21, vec![source]),
                Kind::FixedRectangularTranslatedPoint(source, _, _, _, _) => (17, vec![source]),
                Kind::FixedRectangularTranslatedSegment(source, _, _, _, _) => (17, vec![source]),
                Kind::FixedRectangularTranslatedCircle(source, _, _, _, _) => (17, vec![source]),
                Kind::FixedRectangularTranslatedPolygon(source, _, _, _, _) => (17, vec![source]),
                Kind::MarkedDistanceTranslatedPoint(source, distance, _) => {
                    (24, vec![source, distance])
                }
                Kind::MarkedDistanceTranslatedSegment(source, distance, _)
                | Kind::MarkedDistanceTranslatedCircle(source, distance, _)
                | Kind::MarkedDistanceTranslatedPolygon(source, distance, _) => {
                    (24, vec![source, distance])
                }
                Kind::MixedRectangularTranslatedPoint(source, distance, _, fixed_horizontal) => (
                    if fixed_horizontal { 18 } else { 19 },
                    vec![source, distance],
                ),
                Kind::MixedRectangularTranslatedPolygon(source, distance, _, fixed_horizontal) => (
                    if fixed_horizontal { 18 } else { 19 },
                    vec![source, distance],
                ),
                Kind::RectangularMarkedTranslatedPoint(source, horizontal, vertical) => {
                    (20, vec![source, horizontal, vertical])
                }
                Kind::RectangularMarkedTranslatedPolygon(source, horizontal, vertical) => {
                    (20, vec![source, horizontal, vertical])
                }
                Kind::VectorTranslatedPoint(source, start, end) => (16, vec![source, start, end]),
                Kind::VectorTranslatedSegment(source, start, end)
                | Kind::VectorTranslatedCircle(source, start, end)
                | Kind::VectorTranslatedPolygon(source, start, end) => {
                    (16, vec![source, start, end])
                }
                Kind::ReflectedPoint(source, mirror) => (34, vec![source, mirror]),
                Kind::ReflectedCircle(source, mirror)
                | Kind::ReflectedSegment(source, mirror)
                | Kind::ReflectedPolygon(source, mirror) => (34, vec![source, mirror]),
                Kind::ScaledPoint(source, center, _)
                | Kind::ScaledCircle(source, center, _)
                | Kind::ScaledSegment(source, center, _)
                | Kind::ScaledPolygon(source, center, _) => (30, vec![source, center]),
                Kind::RotatedPoint(source, center, _)
                | Kind::RotatedSegment(source, center, _)
                | Kind::RotatedCircle(source, center, _)
                | Kind::RotatedPolygon(source, center, _) => (27, vec![source, center]),
                Kind::AngleRotatedPoint(source, center, first, vertex, last)
                | Kind::AngleRotatedSegment(source, center, first, vertex, last)
                | Kind::AngleRotatedCircle(source, center, first, vertex, last)
                | Kind::AngleRotatedPolygon(source, center, first, vertex, last) => {
                    (28, vec![source, center, first, vertex, last])
                }
                Kind::RatioScaledPoint(source, center, origin, base, target) => {
                    (32, vec![source, center, origin, base, target])
                }
                Kind::RatioScaledCircle(source, center, origin, base, target)
                | Kind::RatioScaledSegment(source, center, origin, base, target)
                | Kind::RatioScaledPolygon(source, center, origin, base, target) => {
                    (32, vec![source, center, origin, base, target])
                }
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
            if let Kind::Polygon(ref vertices) = o.kind {
                h[8..10].copy_from_slice(
                    &polygon_subtype(vertices.len())
                        .ok_or("Polygon has too many vertices")?
                        .to_le_bytes(),
                );
            }
            if matches!(
                o.kind,
                Kind::ReflectedCircle(_, _) | Kind::ScaledCircle(_, _, _)
            ) {
                h = header(3);
                h[0] = code as u8;
                h[18] = 0;
                h[19] = 0;
            }
            if matches!(
                o.kind,
                Kind::PolarTranslatedSegment(_, _, _, _)
                    | Kind::FixedRectangularTranslatedSegment(_, _, _, _, _)
                    | Kind::VectorTranslatedSegment(_, _, _)
                    | Kind::MarkedDistanceTranslatedSegment(_, _, _)
                    | Kind::AngleRotatedSegment(_, _, _, _, _)
                    | Kind::RotatedSegment(_, _, _)
                    | Kind::ScaledSegment(_, _, _)
                    | Kind::RatioScaledSegment(_, _, _, _, _)
                    | Kind::ReflectedSegment(_, _)
            ) {
                h = header(2);
                h[0] = code as u8;
                if matches!(
                    o.kind,
                    Kind::VectorTranslatedSegment(_, _, _)
                        | Kind::MarkedDistanceTranslatedSegment(_, _, _)
                        | Kind::AngleRotatedSegment(_, _, _, _, _)
                        | Kind::RotatedSegment(_, _, _)
                        | Kind::ScaledSegment(_, _, _)
                        | Kind::RatioScaledSegment(_, _, _, _, _)
                ) {
                    h[18] = 0x51;
                    h[19] = 0x5c;
                }
            }
            if matches!(
                o.kind,
                Kind::PolarTranslatedCircle(_, _, _, _)
                    | Kind::FixedRectangularTranslatedCircle(_, _, _, _, _)
                    | Kind::VectorTranslatedCircle(_, _, _)
                    | Kind::MarkedDistanceTranslatedCircle(_, _, _)
                    | Kind::AngleRotatedCircle(_, _, _, _, _)
                    | Kind::RotatedCircle(_, _, _)
            ) {
                h = header(3);
                h[0] = code as u8;
                if matches!(
                    o.kind,
                    Kind::VectorTranslatedCircle(_, _, _)
                        | Kind::MarkedDistanceTranslatedCircle(_, _, _)
                        | Kind::AngleRotatedCircle(_, _, _, _, _)
                        | Kind::RotatedCircle(_, _, _)
                ) {
                    h[18] = 0x51;
                    h[19] = 0x5c;
                }
            }
            if matches!(o.kind, Kind::RatioScaledCircle(_, _, _, _, _)) {
                h = header(3);
                h[0] = code as u8;
                h[18] = 0x51;
                h[19] = 0x5c;
            }
            if matches!(
                o.kind,
                Kind::ReflectedPolygon(_, _)
                    | Kind::VectorTranslatedPolygon(_, _, _)
                    | Kind::PolarTranslatedPolygon(_, _, _, _)
                    | Kind::FixedRectangularTranslatedPolygon(_, _, _, _, _)
                    | Kind::MarkedDistanceTranslatedPolygon(_, _, _)
                    | Kind::RectangularMarkedTranslatedPolygon(_, _, _)
                    | Kind::MixedRectangularTranslatedPolygon(_, _, _, _)
                    | Kind::RotatedPolygon(_, _, _)
                    | Kind::AngleRotatedPolygon(_, _, _, _, _)
                    | Kind::ScaledPolygon(_, _, _)
                    | Kind::RatioScaledPolygon(_, _, _, _, _)
            ) {
                h = header(8);
                h[0] = code as u8;
                if let Kind::ReflectedPolygon(source, _)
                | Kind::VectorTranslatedPolygon(source, _, _)
                | Kind::PolarTranslatedPolygon(source, _, _, _)
                | Kind::FixedRectangularTranslatedPolygon(source, _, _, _, _)
                | Kind::MarkedDistanceTranslatedPolygon(source, _, _)
                | Kind::RectangularMarkedTranslatedPolygon(source, _, _)
                | Kind::MixedRectangularTranslatedPolygon(source, _, _, _)
                | Kind::RotatedPolygon(source, _, _)
                | Kind::AngleRotatedPolygon(source, _, _, _, _)
                | Kind::ScaledPolygon(source, _, _)
                | Kind::RatioScaledPolygon(source, _, _, _, _) = o.kind
                {
                    let vertex_count = match &self.objects[source - 1].kind {
                        Kind::Polygon(vertices) => vertices.len(),
                        _ => return Err("Reflected interior requires a polygon".into()),
                    };
                    h[8..10].copy_from_slice(
                        &polygon_subtype(vertex_count)
                            .ok_or("Polygon has too many vertices")?
                            .to_le_bytes(),
                    );
                }
                h[20..24].copy_from_slice(&1u32.to_le_bytes());
                if !matches!(
                    o.kind,
                    Kind::ReflectedPolygon(_, _)
                        | Kind::PolarTranslatedPolygon(_, _, _, _)
                        | Kind::FixedRectangularTranslatedPolygon(_, _, _, _, _)
                ) {
                    h[18] = 0x51;
                    h[19] = 0x5c;
                }
            }
            if matches!(o.kind, Kind::VectorTranslatedPolygon(_, _, _))
                && self.objects.iter().any(|candidate| {
                    !candidate.deleted
                        && matches!(&candidate.kind, Kind::Locus(refs, _) if refs.first() == Some(&o.id))
                })
            {
                h[18..20].copy_from_slice(&[0x39, 0x20]);
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
                    Kind::PolarTranslatedPoint(_, angle, units, distance)
                    | Kind::PolarTranslatedSegment(_, angle, units, distance)
                    | Kind::PolarTranslatedCircle(_, angle, units, distance)
                    | Kind::PolarTranslatedPolygon(_, angle, units, distance) => {
                        let mut payload =
                            *include_bytes!("../assets/polar_translation_payload.bin");
                        if angle != 90. {
                            payload[4..12]
                                .copy_from_slice(&(-angle.to_radians().sin()).to_le_bytes());
                            payload[12..20]
                                .copy_from_slice(&angle.to_radians().cos().to_le_bytes());
                        }
                        payload[20..28].copy_from_slice(&angle.to_le_bytes());
                        payload[32..40].copy_from_slice(&units.to_le_bytes());
                        payload[40..48].copy_from_slice(&distance.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::FixedRectangularTranslatedPoint(
                        _,
                        x_units,
                        x_distance,
                        y_units,
                        y_distance,
                    )
                    | Kind::FixedRectangularTranslatedSegment(
                        _,
                        x_units,
                        x_distance,
                        y_units,
                        y_distance,
                    )
                    | Kind::FixedRectangularTranslatedCircle(
                        _,
                        x_units,
                        x_distance,
                        y_units,
                        y_distance,
                    )
                    | Kind::FixedRectangularTranslatedPolygon(
                        _,
                        x_units,
                        x_distance,
                        y_units,
                        y_distance,
                    ) => {
                        let mut payload =
                            *include_bytes!("../assets/rectangular_fixed_translation_payload.bin");
                        payload[4..12].copy_from_slice(&(x_units * x_distance).to_le_bytes());
                        payload[12..20].copy_from_slice(&x_distance.to_le_bytes());
                        payload[24..32].copy_from_slice(&(y_units * y_distance).to_le_bytes());
                        payload[32..40].copy_from_slice(&y_distance.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::MarkedDistanceTranslatedPoint(_, _, angle)
                    | Kind::MarkedDistanceTranslatedSegment(_, _, angle)
                    | Kind::MarkedDistanceTranslatedCircle(_, _, angle)
                    | Kind::MarkedDistanceTranslatedPolygon(_, _, angle) => {
                        let mut payload =
                            *include_bytes!("../assets/marked_distance_translation_payload.bin");
                        if angle != 90. {
                            payload[4..12]
                                .copy_from_slice(&(-angle.to_radians().sin()).to_le_bytes());
                            payload[12..20]
                                .copy_from_slice(&angle.to_radians().cos().to_le_bytes());
                        }
                        payload[20..28].copy_from_slice(&angle.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::MixedRectangularTranslatedPoint(_, _, fixed_cm, fixed_horizontal)
                    | Kind::MixedRectangularTranslatedPolygon(_, _, fixed_cm, fixed_horizontal) => {
                        let mut payload =
                            *include_bytes!("../assets/mixed_rectangular_translation_payload.bin");
                        payload[0] = if fixed_horizontal { 18 } else { 19 };
                        payload[4..12]
                            .copy_from_slice(&(fixed_cm * 37.79527559055118).to_le_bytes());
                        payload[12..20].copy_from_slice(&fixed_cm.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::ScaledPoint(_, _, factor)
                    | Kind::ScaledCircle(_, _, factor)
                    | Kind::ScaledSegment(_, _, factor)
                    | Kind::ScaledPolygon(_, _, factor) => {
                        let mut payload = vec![30, 0, 0, 0];
                        payload.extend_from_slice(&factor.to_le_bytes());
                        put_record(&mut added, 0x7d3, &payload);
                    }
                    Kind::RotatedPoint(_, _, angle)
                    | Kind::RotatedSegment(_, _, angle)
                    | Kind::RotatedCircle(_, _, angle)
                    | Kind::RotatedPolygon(_, _, angle) => {
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
        let added_len = added.len();
        out.splice(insert_at..insert_at, added);
        let last_marker_object = self
            .objects
            .iter()
            .rev()
            .skip_while(|o| o.deleted)
            .find(|o| {
                matches!(
                    o.kind,
                    Kind::FunctionDefinition(_)
                        | Kind::DerivativeFunction(_, _)
                        | Kind::FunctionPlot(_, _, _, _, _)
                        | Kind::ParametricPlot(_, _, _, _, _, _, _)
                        | Kind::Locus(_, _)
                        | Kind::PlottedPoint(_, _, _, _)
                        | Kind::CoordinateMeasurement { .. }
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
                        | Kind::RatioScaledPolygon(_, _, _, _, _)
                )
            })
            .map(|o| o.id);
        if let Some(id) = last_marker_object {
            if let Some(marker) = rs
                .iter()
                .find(|r| r.kind == 0x2328 && r.start >= insert_at && r.start < trailer)
            {
                if marker.data.len() == 4 {
                    let offset = marker.data.start + added_len;
                    out[offset..offset + 4].copy_from_slice(&(id as u32).to_le_bytes());
                }
            } else {
                let mut marker = Vec::new();
                put_record(&mut marker, 0x2328, &(id as u32).to_le_bytes());
                out.splice(insert_at + added_len..insert_at + added_len, marker);
            }
        }
        let mut out = self.patch_marked_coordinate_link(out)?;
        records(&out)?;
        if let Some(resource) = picture_resource {
            let resource_at = records(&out)?
                .iter()
                .find(|record| record.kind == 0x03ef)
                .ok_or("Missing picture resource section")?
                .start;
            out.splice(resource_at..resource_at, resource);
        }
        let out = self.patch_text_font_overrides(self.patch_defined_coordinate_grid_style(
            self.patch_visual_styles(
                self.patch_trace_flags(self.patch_text_and_labels(self.compact_deleted(out)?)?)?,
            )?,
        )?)?;
        self.patch_last_moved_marker(out)
    }
    fn patch_last_moved_marker(&self, mut bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        let Some(id) = self.last_moved_object else {
            return Ok(bytes);
        };
        if !self.objects.iter().any(|object| {
            matches!(
                object.kind,
                Kind::ReflectedPolygon(_, _)
                    | Kind::VectorTranslatedPolygon(_, _, _)
                    | Kind::PolarTranslatedPolygon(_, _, _, _)
                    | Kind::FixedRectangularTranslatedPolygon(_, _, _, _, _)
                    | Kind::MarkedDistanceTranslatedPolygon(_, _, _)
                    | Kind::RectangularMarkedTranslatedPolygon(_, _, _)
                    | Kind::MixedRectangularTranslatedPolygon(_, _, _, _)
                    | Kind::RotatedPolygon(_, _, _)
                    | Kind::AngleRotatedPolygon(_, _, _, _, _)
                    | Kind::ScaledPolygon(_, _, _)
                    | Kind::RatioScaledPolygon(_, _, _, _, _)
            )
        }) {
            return Ok(bytes);
        }
        if let Some(marker) = records(&bytes)?
            .into_iter()
            .find(|record| record.kind == 0x2328 && record.data.len() == 4)
        {
            bytes[marker.data].copy_from_slice(&(id as u32).to_le_bytes());
        }
        Ok(bytes)
    }
    fn patch_text_font_overrides(&self, mut bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        let overrides = self
            .objects
            .iter()
            .filter(|object| !object.deleted)
            .enumerate()
            .filter_map(|(ordinal, object)| {
                object
                    .text_style_override
                    .as_ref()
                    .map(|style| (ordinal, object, style))
            })
            .collect::<Vec<_>>();
        if overrides.is_empty() && self.run_font_families.is_empty() {
            return Ok(bytes);
        }
        if overrides.iter().any(|(_, object, _)| {
            !matches!(object.kind, Kind::PlainText { .. })
                && !(is_point_kind(&object.kind) && object.label.is_some())
        }) {
            return Err("Only free text and labeled points can have font styles".into());
        }
        let rs = records(&bytes)?;
        let last_family = rs
            .iter()
            .filter(|record| record.kind == 0x04b1)
            .last()
            .ok_or("Missing font family table")?;
        let style_records = rs
            .iter()
            .filter(|record| record.kind == 0x04b3)
            .collect::<Vec<_>>();
        style_records.last().ok_or("Missing font style table")?;
        let mut known = (0..style_records.len())
            .map(|index| text_style_records::decode_font_state_at(&bytes, index))
            .collect::<Result<Vec<_>, _>>()?;
        let mut families = text_style_records::font_families(&bytes)?;
        let mut appended_families = Vec::new();
        let mut appended = Vec::new();
        let mut targets = Vec::new();
        for family in &self.run_font_families {
            if families.contains(family) {
                return Err("Duplicate pending text font family".into());
            }
            if families.len() > u16::MAX as usize {
                return Err("Too many font families".into());
            }
            let payload = text_style_records::encode_font_family_payload(family)?;
            put_record(&mut appended_families, 0x04b1, &payload);
            families.push(family.clone());
        }
        for (ordinal, _, style) in overrides {
            let family_index =
                if let Some(index) = families.iter().position(|name| name == &style.family) {
                    index
                } else {
                    let index = families.len();
                    if index > u16::MAX as usize {
                        return Err("Too many font families".into());
                    }
                    let payload = text_style_records::encode_font_family_payload(&style.family)?;
                    put_record(&mut appended_families, 0x04b1, &payload);
                    families.push(style.family.clone());
                    index
                };
            let index = if let Some(index) = known.iter().position(|known| known == style) {
                index
            } else {
                let family_template = known
                    .iter()
                    .take(style_records.len())
                    .position(|known| known.family == style.family)
                    .unwrap_or(0);
                let index = known.len();
                if index > u16::MAX as usize {
                    return Err("Too many text styles".into());
                }
                let template = &bytes[style_records[family_template].data.clone()];
                let mut payload = text_style_records::encode_style_payload(template, style)?;
                payload[..2].copy_from_slice(&(family_index as u16).to_le_bytes());
                put_record(&mut appended, 0x04b3, &payload);
                known.push(style.clone());
                index
            };
            targets.push((ordinal, index as u16));
        }
        if !appended_families.is_empty() {
            let insert_at = last_family.data.end + last_family.data.len() % 2;
            bytes.splice(insert_at..insert_at, appended_families);
        }
        if !appended.is_empty() {
            let rs = records(&bytes)?;
            let last = rs
                .iter()
                .filter(|record| record.kind == 0x04b3)
                .last()
                .ok_or("Missing font style table")?;
            let insert_at = last.data.end + last.data.len() % 2;
            bytes.splice(insert_at..insert_at, appended);
        }
        let rs = records(&bytes)?;
        let starts = rs
            .iter()
            .enumerate()
            .filter_map(|(index, record)| (record.kind == 0x07d0).then_some(index))
            .collect::<Vec<_>>();
        for (ordinal, index) in targets {
            let start = *starts.get(ordinal).ok_or("Missing styled text object")?;
            let group = rs[start + 1..]
                .iter()
                .take_while(|record| record.kind != 0x07d7)
                .collect::<Vec<_>>();
            if !group
                .iter()
                .any(|record| matches!(record.kind, 0x08fc | 0x07d5))
            {
                return Err("Font style target has no text".into());
            }
            let reference = group
                .iter()
                .find(|record| record.kind == 0x07d8 && record.data.len() == 2)
                .ok_or("Missing free-text font reference")?;
            bytes[reference.data.clone()].copy_from_slice(&index.to_le_bytes());
        }
        Ok(bytes)
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
        let mut ids = vec![0; self.objects.len() + 1];
        for (index, object) in survivors.iter().enumerate() {
            ids[object.id] = index + 1;
        }
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
            if let Some((min, max, samples, polar)) = match object.kind {
                Kind::FunctionPlot(_, _, min, max, samples) => Some((min, max, samples, None)),
                Kind::ParametricPlot(_, _, _, min, max, samples, mode) => {
                    Some((min, max, samples, Some(mode == PlotPointsMode::Polar)))
                }
                _ => None,
            } {
                let range = group
                    .iter()
                    .find(|record| record.kind == 0x0902)
                    .ok_or("Missing saved plot range")?;
                let original = &bytes[range.data.clone()];
                let previous = match polar {
                    Some(polar) => function_records::decode_parametric_range(original, polar)?,
                    None => function_records::decode_plot_range(original)?,
                };
                if previous != (min, max, samples) {
                    let mut payload = original.to_vec();
                    payload[..8].copy_from_slice(&min.to_le_bytes());
                    payload[8..16].copy_from_slice(&max.to_le_bytes());
                    payload[16..20].copy_from_slice(&samples.to_le_bytes());
                    let mut replacement = Vec::new();
                    put_record(&mut replacement, 0x0902, &payload);
                    edits.push((
                        range.start,
                        range.data.end + original.len() % 2,
                        replacement,
                    ));
                }
                continue;
            }
            if let Kind::FunctionDefinition(expression) | Kind::DerivativeFunction(_, expression) =
                object.kind
            {
                let record = group
                    .iter()
                    .find(|record| record.kind == 0x0907)
                    .ok_or("Missing saved function expression")?;
                let original = &bytes[record.data.clone()];
                if function_records::decode_simple_expression(original)? != expression {
                    edits.push((
                        record.start,
                        record.data.end + original.len() % 2,
                        function_records::encode_expression_record(original, expression)?,
                    ));
                }
                continue;
            }
            if let Some(replacement) = match &object.kind {
                Kind::Parameter(value) => Some(value.as_bytes().to_vec()),
                Kind::Calculation(value) => Some(value.as_bytes().to_vec()),
                Kind::ValueTable(_, value) | Kind::ValueTablePair(_, _, value) => {
                    Some(value.remap_sources(&ids)?.as_bytes().to_vec())
                }
                _ => None,
            } {
                if bytes[group[0].start..record.data.end] != replacement {
                    let mut replacement = replacement;
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
                if old_text != *text || old_rect != *rect || object.rich_text_override.is_some() {
                    let (new_rich, new_bbox) = if let Some(template) = &object.rich_text_override {
                        template.encode_parts(text, *rect)?
                    } else {
                        text_object::encode_from_template(
                            &bytes[rich.data.clone()],
                            &bytes[bbox.data.clone()],
                            text,
                            *rect,
                        )?
                    };
                    if bytes[rich.data.clone()] != new_rich {
                        let mut rich_record = Vec::new();
                        put_record(&mut rich_record, 0x08fc, &new_rich);
                        edits.push((rich.start, rich.data.end + rich.data.len() % 2, rich_record));
                    }
                    if bytes[bbox.data.clone()] != new_bbox {
                        let mut box_record = Vec::new();
                        put_record(&mut box_record, 0x0898, &new_bbox);
                        edits.push((bbox.start, bbox.data.end + bbox.data.len() % 2, box_record));
                    }
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
    fn validate_reparsed(&self, reparsed: &Self) -> Result<(), String> {
        let survivors = self
            .objects
            .iter()
            .filter(|object| !object.deleted)
            .collect();
        self.validate_reparsed_in_order(reparsed, survivors, false)
    }
    fn validate_reparsed_in_order(
        &self,
        reparsed: &Self,
        survivors: Vec<&Object>,
        ignore_active_page_frame: bool,
    ) -> Result<(), String> {
        let mut ids = vec![0; self.objects.len() + 1];
        for (index, object) in survivors.iter().enumerate() {
            ids[object.id] = index + 1;
        }
        let expected_kinds = survivors
            .iter()
            .map(|object| object.kind.remap_checked(&ids))
            .collect::<Result<Vec<_>, String>>()?;
        if reparsed.partial_view && self.safe_structure
            || reparsed.snap_points != self.snap_points
            || reparsed.grid_dotted != self.grid_dotted
            || reparsed.grid_form != self.grid_form
            || reparsed.angle_unit_radians != self.angle_unit_radians
            || reparsed.page_count != self.page_count
            || reparsed.marked_coordinate_system
                != self
                    .marked_coordinate_system
                    .iter()
                    .map(|id| id.and_then(|id| (ids[id] != 0).then_some(ids[id])))
                    .collect::<Vec<_>>()
            || reparsed.frame_styles
                != self
                    .frame_styles
                    .iter()
                    .filter_map(|(id, style)| {
                        let remapped = ids.get(*id).copied().unwrap_or(0);
                        (remapped != 0).then_some((remapped, *style))
                    })
                    .collect()
            || !ignore_active_page_frame
                && reparsed.active_coordinate_system()
                    != self.active_coordinate_system().map(|id| ids[id])
            || reparsed.objects.len() != survivors.len()
            || reparsed
                .objects
                .iter()
                .zip(survivors.iter().zip(&expected_kinds))
                .any(|(saved, (current, expected_kind))| {
                    saved.id != ids[current.id]
                        || saved.page != current.page
                        || saved.kind != *expected_kind
                        || saved.visible != current.visible
                        || saved.label != current.label
                        || saved.visual_style != current.visual_style
                        || saved.midpoint_host != current.midpoint_host.map(|host| ids[host])
                        || current.function_anchor.is_some()
                            && saved.function_anchor != current.function_anchor
                        || current.function_name.is_some()
                            && saved.function_name != current.function_name
                })
        {
            return Err("Serialized sketch does not preserve the edited objects".into());
        }
        Ok(())
    }
    pub fn save_to(&mut self, path: &Path) -> Result<(), String> {
        let reordered_additions = self.has_pending_multi_page_additions();
        let data = self.encode()?;
        let reparsed = Self::from_bytes(&data, Some(path.to_path_buf()))?;
        let had_deletion = self.objects.iter().any(|o| o.deleted);
        if reordered_additions {
            let mut survivors: Vec<_> = self
                .objects
                .iter()
                .filter(|object| !object.deleted)
                .collect();
            survivors.sort_by_key(|object| (object.page, object.id > self.source_count, object.id));
            self.validate_reparsed_in_order(&reparsed, survivors, true)?;
        } else if self.page_count > 1 {
            let survivors = self
                .objects
                .iter()
                .filter(|object| !object.deleted)
                .collect();
            self.validate_reparsed_in_order(&reparsed, survivors, true)?;
        } else {
            self.validate_reparsed(&reparsed)?;
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
        self.marked_coordinate_system = reparsed.marked_coordinate_system;
        self.frame_styles = reparsed.frame_styles;
        if had_deletion || reordered_additions {
            self.selected.clear();
        }
        self.original = data;
        self.run_font_families.clear();
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
    fn styled_text_imports_and_edits_without_changing_other_runs() {
        for source in [
            include_bytes!("../tests/fixtures/text_style_bold_off.gsp").as_slice(),
            include_bytes!("../tests/fixtures/text_style_italic_on.gsp").as_slice(),
            include_bytes!("../tests/fixtures/text_style_underline_on.gsp").as_slice(),
            include_bytes!("../tests/fixtures/text_style_larger.gsp").as_slice(),
            include_bytes!("../tests/fixtures/text_two_objects_first_bold_off_original.gsp")
                .as_slice(),
            include_bytes!("../tests/fixtures/text_two_objects_second_bold_off_original.gsp")
                .as_slice(),
        ] {
            let doc = Document::from_bytes(source, None).unwrap();
            assert!(!doc.partial_view);
            assert_eq!(doc.encode().unwrap(), source);
        }

        let original =
            include_bytes!("../tests/fixtures/text_two_objects_first_bold_off_original.gsp");
        let mut doc = Document::from_bytes(original, None).unwrap();
        let other = doc.objects[1].kind.clone();
        doc.edit_text(1, "Greetings").unwrap();
        let edited = doc.encode().unwrap();
        let reopened = Document::from_bytes(&edited, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[1].kind, other);
        assert!(
            matches!(&reopened.objects[0].kind, Kind::PlainText { text, .. } if text == "Greetings")
        );
        for kind in [0x04b1, 0x04b3, 0x273c] {
            let payloads = |bytes: &[u8]| {
                records(bytes)
                    .unwrap()
                    .into_iter()
                    .filter(|record| record.kind == kind)
                    .map(|record| bytes[record.data].to_vec())
                    .collect::<Vec<_>>()
            };
            assert_eq!(payloads(&edited), payloads(original));
        }
        doc.move_text(1, Point { x: 110., y: 100. }).unwrap();
        let moved = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!moved.partial_view);
        assert_eq!(moved.objects[1].kind, other);
        doc.undo();
        assert_eq!(doc.encode().unwrap(), edited);
    }
    #[test]
    fn text_formatting_keeps_independent_styles_through_save_undo_and_copy() {
        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/text_two_objects_generated.gsp"),
            None,
        )
        .unwrap();
        assert!(doc.text_font_state(1).unwrap().bold);
        doc.selected.insert(1);
        assert_eq!(
            doc.toggle_selected_text_style(TextStyleCommand::Bold)
                .unwrap(),
            1
        );
        assert!(!doc.text_font_state(1).unwrap().bold);
        assert!(doc.text_font_state(2).unwrap().bold);
        let one_formatted = doc.encode().unwrap();
        let reopened = Document::from_bytes(&one_formatted, None).unwrap();
        assert!(!reopened.partial_view);
        assert!(!reopened.text_font_state(1).unwrap().bold);
        assert!(reopened.text_font_state(2).unwrap().bold);
        doc.undo();
        assert!(doc.text_font_state(1).unwrap().bold);
        doc.redo();
        assert!(!doc.text_font_state(1).unwrap().bold);

        doc.selected.clear();
        doc.selected.insert(2);
        doc.toggle_selected_text_style(TextStyleCommand::Italic)
            .unwrap();
        doc.toggle_selected_text_style(TextStyleCommand::Underline)
            .unwrap();
        let font = doc.text_font_state(2).unwrap();
        assert!(font.bold && font.italic && font.underline);
        let clipboard = doc.copy_selected().unwrap();
        let pasted = doc.paste(&clipboard).unwrap();
        assert_eq!(pasted, vec![3]);
        assert_eq!(doc.text_font_state(3).unwrap(), font);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.text_font_state(3).unwrap(), font);
        assert_eq!(reopened.text_font_state(2).unwrap(), font);
        assert!(!reopened.text_font_state(1).unwrap().bold);
    }
    #[test]
    fn mixed_font_families_survive_style_edit_and_copy() {
        let source = include_bytes!("../tests/fixtures/text_two_font_cjk_first.gsp");
        let mut doc = Document::from_bytes(source, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), source);
        assert_eq!(doc.text_font_state(1).unwrap().family, "@Noto Sans CJK JP");
        assert_eq!(doc.text_font_state(2).unwrap().family, "Tahoma");
        doc.selected.insert(1);
        doc.toggle_selected_text_style(TextStyleCommand::Italic)
            .unwrap();
        let copied = doc.copy_selected().unwrap();
        assert_eq!(doc.paste(&copied).unwrap(), vec![3]);
        doc.selected.clear();
        doc.selected.insert(2);
        doc.toggle_selected_text_style(TextStyleCommand::Bold)
            .unwrap();
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(
            reopened.text_font_state(1).unwrap().family,
            "@Noto Sans CJK JP"
        );
        assert!(reopened.text_font_state(1).unwrap().italic);
        assert_eq!(reopened.text_font_state(2).unwrap().family, "Tahoma");
        assert!(!reopened.text_font_state(2).unwrap().bold);
        assert_eq!(reopened.text_font_state(3), reopened.text_font_state(1));
    }
    #[test]
    fn native_mixed_format_text_moves_without_losing_runs() {
        let source = include_bytes!("../tests/fixtures/text_mixed_bold_italic_he_original.gsp");
        let mut doc = Document::from_bytes(source, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), source);
        let runs = doc.text_rich_runs(1).unwrap();
        assert_eq!(runs[0][0].text, "He");
        assert_eq!(runs[0][0].style.bold, Some(false));
        assert_eq!(runs[0][0].style.italic, Some(true));
        assert_eq!(runs[0][1].text, "llo");
        doc.edit_text(1, "Hexllo").unwrap();
        let edited = doc.text_rich_runs(1).unwrap();
        assert_eq!(edited[0][0].text, "Hex");
        assert_eq!(edited[0][0].style, runs[0][0].style);
        assert_eq!(edited[0][1].text, "llo");
        let edited_file = doc.encode().unwrap();
        let reopened_edit = Document::from_bytes(&edited_file, None).unwrap();
        assert_eq!(reopened_edit.text_rich_runs(1).unwrap(), edited);
        doc.undo();
        assert_eq!(doc.text_rich_runs(1).unwrap(), runs);
        doc.move_text(1, Point { x: 140., y: 110. }).unwrap();
        let encoded = doc.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.text_rich_runs(1).unwrap(), runs);
        assert_eq!(reopened.encode().unwrap(), encoded);

        doc.selected.insert(1);
        let clipboard = doc.copy_selected().unwrap();
        assert_eq!(doc.paste(&clipboard).unwrap(), vec![2]);
        assert_eq!(doc.text_rich_runs(2).unwrap(), runs);
        let Kind::PlainText { rect: first, .. } = &doc.objects[0].kind else {
            unreachable!()
        };
        let Kind::PlainText { rect: second, .. } = &doc.objects[1].kind else {
            unreachable!()
        };
        assert_eq!(second.left, first.left + 20);
        assert_eq!(second.top, first.top + 20);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.text_rich_runs(2).unwrap(), runs);
    }
    #[test]
    fn substring_size_steps_preserve_native_rich_text() {
        let source = include_bytes!("../tests/fixtures/text_mixed_size36_he_original.gsp");
        let mut doc = Document::from_bytes(source, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), source);
        let original = doc.text_rich_runs(1).unwrap();
        assert_eq!(original[0][0].style.size_code, Some(36));
        doc.format_text_range(1, 0, 2, TextStyleCommand::DecreaseSize)
            .unwrap();
        assert_eq!(doc.text_rich_runs(1).unwrap()[0][0].style.size_code, None);
        doc.format_text_range(1, 0, 2, TextStyleCommand::IncreaseSize)
            .unwrap();
        assert_eq!(doc.text_rich_runs(1).unwrap(), original);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.text_rich_runs(1).unwrap(), original);
    }
    #[test]
    fn native_mixed_family_caption_edits_and_reopens() {
        let source = include_bytes!("../tests/fixtures/text_mixed_adventor_he_original.gsp");
        let mut doc = Document::from_bytes(source, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), source);
        assert_eq!(doc.text_font_families().unwrap()[1], "TeX Gyre Adventor");
        let runs = doc.text_rich_runs(1).unwrap();
        assert_eq!(runs[0][0].style.family_index, Some(1));
        doc.edit_text(1, "Hexllo").unwrap();
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.text_rich_runs(1).unwrap()[0][0].text, "Hex");
        assert_eq!(
            reopened.text_rich_runs(1).unwrap()[0][0].style.family_index,
            Some(1)
        );
    }
    #[test]
    fn selected_range_can_add_a_font_family_and_undo_it() {
        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/text_object_generated_Hello.gsp"),
            None,
        )
        .unwrap();
        doc.format_text_range_family(1, 0, 2, "TeX Gyre Adventor")
            .unwrap();
        assert_eq!(doc.text_font_families().unwrap()[1], "TeX Gyre Adventor");
        assert_eq!(
            doc.text_rich_runs(1).unwrap()[0][0].style.family_index,
            Some(1)
        );
        let encoded = doc.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.text_rich_runs(1).unwrap(),
            doc.text_rich_runs(1).unwrap()
        );
        doc.undo();
        assert_eq!(
            doc.text_rich_runs(1).unwrap()[0][0].style.family_index,
            None
        );
        assert_eq!(doc.text_font_families().unwrap().len(), 1);
        doc.redo();
        assert_eq!(
            doc.text_rich_runs(1).unwrap()[0][0].style.family_index,
            Some(1)
        );
        let path =
            std::env::temp_dir().join(format!("sketchpad_run_family_{}.gsp", std::process::id()));
        doc.save_to(&path).unwrap();
        let first = fs::read(&path).unwrap();
        doc.save_to(&path).unwrap();
        assert_eq!(fs::read(&path).unwrap(), first);
        assert_eq!(doc.text_font_families().unwrap()[1], "TeX Gyre Adventor");
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn selected_caption_range_writes_native_style_markup() {
        let source = include_bytes!("../tests/fixtures/text_object_generated_Hello.gsp");
        let mut doc = Document::from_bytes(source, None).unwrap();
        doc.format_text_range(1, 0, 2, TextStyleCommand::Bold)
            .unwrap();
        let runs = doc.text_rich_runs(1).unwrap();
        assert_eq!(runs[0][0].text, "He");
        assert_eq!(runs[0][0].style.bold, Some(false));
        assert_eq!(runs[0][1].text, "llo");
        let encoded = doc.encode().unwrap();
        assert!(
            encoded
                .windows(b"<VL<H<SW1<T2xHe>><T3xllo>>>".len())
                .any(|part| part == b"<VL<H<SW1<T2xHe>><T3xllo>>>")
        );
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.text_rich_runs(1).unwrap(), runs);
        doc.format_text_range(1, 0, 2, TextStyleCommand::Italic)
            .unwrap();
        assert_eq!(
            doc.text_rich_runs(1).unwrap()[0][0].style.italic,
            Some(true)
        );
        doc.undo();
        assert_eq!(doc.text_rich_runs(1).unwrap(), runs);
        doc.edit_text(1, "Hexllo").unwrap();
        let edited = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        let edited_runs = edited.text_rich_runs(1).unwrap();
        assert_eq!(edited_runs[0][0].text, "Hex");
        assert_eq!(edited_runs[0][0].style.bold, Some(false));
        assert_eq!(edited_runs[0][1].text, "llo");
    }
    #[test]
    fn selected_text_can_use_new_font_families_independently() {
        let source = include_bytes!("../tests/fixtures/text_two_objects_generated.gsp");
        let mut doc = Document::from_bytes(source, None).unwrap();
        doc.selected.insert(1);
        assert_eq!(doc.set_selected_text_family("Arial").unwrap(), 1);
        doc.toggle_selected_text_style(TextStyleCommand::Italic)
            .unwrap();
        assert_eq!(doc.text_font_state(1).unwrap().family, "Arial");
        assert_eq!(doc.text_font_state(2).unwrap().family, "Tahoma");
        doc.selected.clear();
        doc.selected.insert(2);
        doc.set_selected_text_family("@Noto Sans CJK JP").unwrap();
        let encoded = doc.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.text_font_state(1).unwrap().family, "Arial");
        assert!(reopened.text_font_state(1).unwrap().italic);
        assert_eq!(
            reopened.text_font_state(2).unwrap().family,
            "@Noto Sans CJK JP"
        );
        assert_eq!(reopened.encode().unwrap(), encoded);
        assert!(doc.set_selected_text_family("").is_err());
        assert!(doc.set_selected_text_family("Noto 日本語").is_err());
        doc.undo();
        assert_eq!(doc.text_font_state(2).unwrap().family, "Tahoma");
    }
    #[test]
    fn text_size_commands_follow_verified_native_steps() {
        for (source, size) in [
            (
                include_bytes!("../tests/fixtures/text_style_smaller_18_original.gsp").as_slice(),
                18,
            ),
            (
                include_bytes!("../tests/fixtures/text_style_larger_48_original.gsp").as_slice(),
                48,
            ),
        ] {
            let doc = Document::from_bytes(source, None).unwrap();
            assert!(!doc.partial_view);
            assert_eq!(doc.text_font_state(1).unwrap().size_code, size);
            assert_eq!(doc.encode().unwrap(), source);
        }
        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/text_object_generated_Hello.gsp"),
            None,
        )
        .unwrap();
        doc.selected.insert(1);
        for size in [36, 48, 64] {
            doc.toggle_selected_text_style(TextStyleCommand::IncreaseSize)
                .unwrap();
            assert_eq!(doc.text_font_state(1).unwrap().size_code, size);
        }
        assert!(
            doc.toggle_selected_text_style(TextStyleCommand::IncreaseSize)
                .is_err()
        );
        for size in [48, 36, 24, 18, 16, 14] {
            doc.toggle_selected_text_style(TextStyleCommand::DecreaseSize)
                .unwrap();
            assert_eq!(doc.text_font_state(1).unwrap().size_code, size);
        }
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.text_font_state(1).unwrap().size_code, 14);
        doc.undo();
        assert_eq!(doc.text_font_state(1).unwrap().size_code, 16);
    }
    #[test]
    fn point_label_styles_are_independent_and_survive_copy() {
        let source =
            include_bytes!("../tests/fixtures/text_two_point_labels_first_bold_off_original.gsp");
        let mut doc = Document::from_bytes(source, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), source);
        assert!(!doc.text_font_state(1).unwrap().bold);
        assert!(doc.text_font_state(2).unwrap().bold);

        doc.selected.insert(2);
        doc.toggle_selected_text_style(TextStyleCommand::Italic)
            .unwrap();
        assert!(doc.text_font_state(2).unwrap().italic);
        assert!(!doc.text_font_state(1).unwrap().bold);
        let copied = doc.copy_selected().unwrap();
        let pasted = doc.paste(&copied).unwrap();
        assert_eq!(pasted, vec![3]);
        assert_eq!(doc.objects[2].label, doc.objects[1].label);
        assert_eq!(doc.text_font_state(3), doc.text_font_state(2));
        doc.selected.clear();
        doc.selected.insert(1);
        doc.toggle_selected_text_style(TextStyleCommand::IncreaseSize)
            .unwrap();
        assert_eq!(doc.text_font_state(1).unwrap().size_code, 36);
        assert_eq!(doc.text_font_state(2).unwrap().size_code, 24);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(!reopened.text_font_state(1).unwrap().bold);
        assert_eq!(reopened.text_font_state(1).unwrap().size_code, 36);
        assert!(reopened.text_font_state(2).unwrap().italic);
        assert_eq!(reopened.text_font_state(3), reopened.text_font_state(2));
        assert_eq!(reopened.objects[2].label, reopened.objects[1].label);
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
    fn two_target_hide_show_button_import_create_and_toggle() {
        for bytes in [
            include_bytes!("../tests/fixtures/action_hide_show_two_points_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/action_hide_show_two_points_clicked.gsp").as_slice(),
            include_bytes!(
                "../tests/fixtures/action_hide_show_two_points_port_clicked_by_wine.gsp"
            )
            .as_slice(),
        ] {
            let doc = Document::from_bytes(bytes, None).unwrap();
            assert!(!doc.partial_view);
            assert_eq!(doc.objects[3].kind.dependencies(), vec![1, 2]);
            assert_eq!(doc.encode().unwrap(), bytes);
        }
        let mut native = Document::open(Path::new(
            "tests/fixtures/action_hide_show_two_points_original.gsp",
        ))
        .unwrap();
        native.activate_hide_show_button(4).unwrap();
        assert!(!native.objects[0].visible);
        assert!(!native.objects[1].visible);
        assert!(native.objects[2].visible);
        let hidden = Document::from_bytes(&native.encode().unwrap(), None).unwrap();
        assert_eq!(hidden.objects[3].kind.dependencies(), vec![1, 2]);
        assert!(matches!(
            hidden.objects[3].kind,
            Kind::HideShowButton {
                action: VisibilityAction::Show,
                ..
            }
        ));
        native.undo();
        assert!(native.objects[0].visible && native.objects[1].visible);

        let mut generated = Document::new();
        let first = generated.add(Kind::Free(Point { x: 150., y: 120. }));
        let second = generated.add(Kind::Free(Point { x: 270., y: 120. }));
        generated.add(Kind::Segment(first, second));
        let button = generated
            .add_hide_show_button_for_targets(&[first, second])
            .unwrap();
        assert!(generated.add_hide_show_button_for_targets(&[]).is_err());
        assert!(
            generated
                .add_hide_show_button_for_targets(&[first, first])
                .is_err()
        );
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert_eq!(
            reopened.objects[button - 1].kind.dependencies(),
            vec![first, second]
        );

        let mut source = Document::open(Path::new(
            "tests/fixtures/action_hide_show_two_points_original.gsp",
        ))
        .unwrap();
        source.selected.insert(4);
        let clipboard = source.copy_selected().unwrap();
        let mut destination = Document::new();
        let pasted = destination.paste(&clipboard).unwrap();
        let pasted_button = *pasted.last().unwrap();
        assert_eq!(
            destination.objects[pasted_button - 1].kind.dependencies(),
            vec![1, 2]
        );
        assert!(
            !Document::from_bytes(&destination.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
    #[test]
    fn animation_button_imports_click_state_and_generated_file_round_trips() {
        for bytes in [
            include_bytes!("../tests/fixtures/action_animation_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/action_animation_clicked.gsp").as_slice(),
        ] {
            let doc = Document::from_bytes(bytes, None).unwrap();
            assert!(!doc.partial_view);
            assert!(matches!(
                &doc.objects[4].kind,
                Kind::AnimationButton { targets, label, anchor: (10, 5), .. }
                    if targets == &[4] && label == "动画点"
            ));
            assert_eq!(doc.objects[4].kind.dependencies(), vec![4]);
            assert_eq!(doc.encode().unwrap(), bytes);
        }
        let base = include_bytes!("../tests/fixtures/animation_button_base.gsp");
        let mut doc = Document::from_bytes(base, None).unwrap();
        let button = doc.add_animation_button(4).unwrap();
        assert_eq!(button, 5);
        let generated = doc.encode().unwrap();
        let reopened = Document::from_bytes(&generated, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.encode().unwrap(), generated);
        assert!(matches!(
            &reopened.objects[4].kind,
            Kind::AnimationButton { targets, label, .. } if targets == &[4] && label == "Animate Point"
        ));
        let clicked = include_bytes!("../tests/fixtures/action_animation_port_clicked_by_wine.gsp");
        let clicked_doc = Document::from_bytes(clicked, None).unwrap();
        assert!(!clicked_doc.partial_view);
        assert_ne!(clicked_doc.position(4), doc.position(4));
        assert_eq!(clicked_doc.encode().unwrap(), clicked);
        doc.selected.insert(button);
        let clipboard = doc.copy_selected().unwrap();
        let pasted = doc.paste(&clipboard).unwrap();
        let pasted_button = *pasted.last().unwrap();
        assert!(matches!(
            doc.objects[pasted_button - 1].kind,
            Kind::AnimationButton { ref targets, .. } if targets.len() == 1 && targets[0] != 4 && targets[0] < pasted_button
        ));
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(matches!(
            reopened.objects[pasted_button - 1].kind,
            Kind::AnimationButton { .. }
        ));
    }

    #[test]
    fn movement_button_imports_and_generates_editable_file() {
        for bytes in [
            include_bytes!("../tests/fixtures/action_movement_default_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/action_movement_slow_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/action_movement_fast_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/action_movement_instant_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/action_movement_initial_original.gsp").as_slice(),
        ] {
            let document = Document::from_bytes(bytes, None).unwrap();
            assert!(!document.partial_view);
            assert!(matches!(
                document.objects[3].kind,
                Kind::MovementButton {
                    source: 1,
                    destination: 2,
                    ..
                }
            ));
            assert_eq!(document.encode().unwrap(), bytes);
        }
        let mut document = Document::new();
        let source = document.add(Kind::Free(Point { x: 150., y: 120. }));
        let destination = document.add(Kind::Free(Point { x: 270., y: 120. }));
        let id = document
            .add_movement_button(source, destination, MovementSpeed::Fast, false)
            .unwrap();
        let bytes = document.encode().unwrap();
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert!(matches!(
            reopened.objects[id - 1].kind,
            Kind::MovementButton {
                source: 1,
                destination: 2,
                speed: MovementSpeed::Fast,
                follow: false,
                ..
            }
        ));
        assert_eq!(reopened.encode().unwrap(), bytes);
    }

    #[test]
    fn scroll_button_imports_and_generates_editable_file() {
        for (bytes, expected) in [
            (
                include_bytes!("../tests/fixtures/action_scroll_top_left_original.gsp").as_slice(),
                ScrollPlacement::UpperLeft,
            ),
            (
                include_bytes!("../tests/fixtures/action_scroll_center_original.gsp").as_slice(),
                ScrollPlacement::Center,
            ),
        ] {
            let document = Document::from_bytes(bytes, None).unwrap();
            assert!(!document.partial_view);
            assert!(
                matches!(document.objects[3].kind, Kind::ScrollButton { target: 1, placement, .. } if placement == expected)
            );
            assert_eq!(document.encode().unwrap(), bytes);
        }
        let mut document = Document::new();
        let target = document.add(Kind::Free(Point { x: 150., y: 120. }));
        let button = document
            .add_scroll_button(target, ScrollPlacement::Center)
            .unwrap();
        let bytes = document.encode().unwrap();
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert!(matches!(
            reopened.objects[button - 1].kind,
            Kind::ScrollButton {
                target: 1,
                placement: ScrollPlacement::Center,
                ..
            }
        ));
        assert_eq!(reopened.encode().unwrap(), bytes);
    }

    #[test]
    fn presentation_button_imports_and_generates_editable_file() {
        for (bytes, expected, targets) in [
            (
                include_bytes!("../tests/fixtures/action_presentation_default_original.gsp")
                    .as_slice(),
                PresentationMode::Simultaneous,
                vec![3, 4],
            ),
            (
                include_bytes!(
                    "../tests/fixtures/action_presentation_sequential_scroll_original.gsp"
                )
                .as_slice(),
                PresentationMode::Sequential,
                vec![4, 5],
            ),
        ] {
            let document = Document::from_bytes(bytes, None).unwrap();
            assert!(!document.partial_view);
            let last = document.objects.last().unwrap();
            assert!(
                matches!(&last.kind, Kind::PresentationButton { targets: actual, mode, .. }
                if actual == &targets && *mode == expected)
            );
            assert_eq!(document.encode().unwrap(), bytes);
        }
        let base = include_bytes!("../tests/fixtures/presentation_two_hide_show_base.gsp");
        let mut document = Document::from_bytes(base, None).unwrap();
        let button = document
            .add_presentation_button_for_targets(&[3, 4], PresentationMode::Sequential)
            .unwrap();
        let bytes = document.encode().unwrap();
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert!(
            matches!(&reopened.objects[button - 1].kind, Kind::PresentationButton { targets, mode: PresentationMode::Sequential, .. }
            if targets == &[3, 4])
        );
        assert_eq!(reopened.encode().unwrap(), bytes);
    }

    #[test]
    fn presentation_button_clicks_in_native_and_desktop_match() {
        for (bytes, mode) in [
            (
                include_bytes!(
                    "../tests/fixtures/action_presentation_port_simultaneous_clicked_by_wine.gsp"
                )
                .as_slice(),
                PresentationMode::Simultaneous,
            ),
            (
                include_bytes!(
                    "../tests/fixtures/action_presentation_port_sequential_clicked_by_wine.gsp"
                )
                .as_slice(),
                PresentationMode::Sequential,
            ),
            (
                include_bytes!("../tests/fixtures/action_presentation_port_desktop_clicked.gsp")
                    .as_slice(),
                PresentationMode::Simultaneous,
            ),
        ] {
            let document = Document::from_bytes(bytes, None).unwrap();
            assert!(!document.partial_view);
            assert!(!document.objects[0].visible);
            assert!(!document.objects[1].visible);
            for button in &document.objects[2..4] {
                assert!(matches!(
                    button.kind,
                    Kind::HideShowButton {
                        action: VisibilityAction::Show,
                        ..
                    }
                ));
            }
            assert!(matches!(
                document.objects[4].kind,
                Kind::PresentationButton { mode: actual, .. } if actual == mode
            ));
            assert_eq!(document.encode().unwrap(), bytes);
        }
    }

    #[test]
    fn link_url_buttons_import_and_generate_editable_files() {
        for (bytes, url) in [
            (
                include_bytes!("../tests/fixtures/action_link_url_default_original.gsp").as_slice(),
                "http://kejianyuan.com",
            ),
            (
                include_bytes!("../tests/fixtures/action_link_url_custom_original.gsp").as_slice(),
                "https://example.invalid/sketchpad",
            ),
            (
                include_bytes!("../tests/fixtures/action_link_url_even_original.gsp").as_slice(),
                "https://example.org/",
            ),
        ] {
            let document = Document::from_bytes(bytes, None).unwrap();
            assert!(!document.partial_view);
            assert!(matches!(&document.objects[1].kind,
                Kind::LinkUrlButton { url: actual, label, anchor }
                if actual == url && label == url && *anchor == (10, 5)
            ));
            assert_eq!(document.encode().unwrap(), bytes);
        }
        let base = include_bytes!("../tests/fixtures/point.gsp");
        let mut document = Document::from_bytes(base, None).unwrap();
        let button = document
            .add_link_url_button("https://example.org/")
            .unwrap();
        let bytes = document.encode().unwrap();
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert!(matches!(&reopened.objects[button - 1].kind,
            Kind::LinkUrlButton { url, label, .. }
            if url == "https://example.org/" && label == url
        ));
        assert_eq!(reopened.encode().unwrap(), bytes);
    }

    #[test]
    fn native_page_links_import_with_destinations_and_preserve_file() {
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
        ] {
            let document = Document::from_bytes(bytes, None).unwrap();
            assert_eq!(document.page_count, 2);
            assert_eq!(document.objects[1].page, source_page);
            assert!(matches!(&document.objects[1].kind,
                Kind::LinkPageButton { target_page, label, anchor, .. }
                if *target_page == destination_page && label == "链接" && *anchor == (10, 5)
            ));
            assert_eq!(document.encode().unwrap(), bytes);
        }
    }

    #[test]
    fn page_link_appends_to_last_page_of_native_document() {
        let bytes = include_bytes!("../tests/fixtures/two_page_blank_native.gsp");
        let mut document = Document::from_bytes(bytes, None).unwrap();
        document.active_page = 1;
        let button = document.add_link_page_button(0).unwrap();
        let url_button = document
            .add_link_url_button("https://example.org/")
            .unwrap();
        let encoded = document.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert_eq!(reopened.page_count, 2);
        assert_eq!(reopened.objects[button - 1].page, 1);
        assert!(matches!(&reopened.objects[button - 1].kind,
            Kind::LinkPageButton { target_page: 0, label, .. } if label == "Go to Page 1"
        ));
        assert!(matches!(&reopened.objects[url_button - 1].kind,
            Kind::LinkUrlButton { url, .. } if url == "https://example.org/"
        ));
        assert_eq!(reopened.encode().unwrap(), encoded);
    }
    #[test]
    fn links_on_earlier_pages_preserve_later_objects_and_ids() {
        let bytes = include_bytes!("../tests/fixtures/two_page_two_points_native.gsp");
        let mut document = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(document.objects.len(), 2);
        document.active_page = 0;
        document.add_link_page_button(1).unwrap();
        document
            .add_link_url_button("https://example.org/")
            .unwrap();
        document.active_page = 1;
        document.add_link_page_button(0).unwrap();
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_page_links_{}.gsp",
            std::process::id()
        ));
        document.save_to(&path).unwrap();
        assert_eq!(document.objects.len(), 5);
        assert!(matches!(document.objects[0].kind, Kind::Free(_)));
        assert!(matches!(
            &document.objects[1].kind,
            Kind::LinkPageButton { target_page: 1, .. }
        ));
        assert!(matches!(&document.objects[2].kind,
            Kind::LinkUrlButton { url, .. } if url == "https://example.org/"
        ));
        assert!(matches!(document.objects[3].kind, Kind::Free(_)));
        assert_eq!(document.objects[3].page, 1);
        assert!(matches!(
            &document.objects[4].kind,
            Kind::LinkPageButton { target_page: 0, .. }
        ));
        let saved = std::fs::read(&path).unwrap();
        assert_eq!(document.encode().unwrap(), saved);
        document.active_page = 0;
        document.add_link_page_button(1).unwrap();
        document.save_to(&path).unwrap();
        assert_eq!(document.objects.len(), 6);
        assert!(matches!(
            document.objects[3].kind,
            Kind::LinkPageButton { .. }
        ));
        assert!(matches!(document.objects[4].kind, Kind::Free(_)));
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn page_link_destination_button_imports_and_writes_native_activation_flag() {
        let native = include_bytes!(
            "../tests/fixtures/action_link_page_activate_hide_show_native_created.gsp"
        );
        let imported = Document::from_bytes(native, None).unwrap();
        assert!(matches!(
            imported.objects[3].kind,
            Kind::LinkPageButton {
                target_page: 0,
                target_button_ordinal: Some(1),
                ..
            }
        ));
        assert_eq!(imported.encode().unwrap(), native);

        let base = include_bytes!("../tests/fixtures/two_page_hide_show_first_native.gsp");
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = 1;
        document
            .add_link_page_button_with_target(0, Some(1))
            .unwrap();
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_link_destination_{}.gsp",
            std::process::id()
        ));
        document.save_to(&path).unwrap();
        let saved = std::fs::read(&path).unwrap();
        assert_eq!(
            saved,
            include_bytes!(
                "../tests/fixtures/action_link_page_activate_hide_show_port_generated.gsp"
            )
        );
        assert!(matches!(
            document.objects[3].kind,
            Kind::LinkPageButton {
                target_button_ordinal: Some(1),
                ..
            }
        ));
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn multi_page_hide_show_state_saves_without_changing_other_pages() {
        for (index, base) in [
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
            let mut document = Document::from_bytes(base, None).unwrap();
            document.active_page = 0;
            document.activate_hide_show_button(2).unwrap();
            assert!(document.can_save().is_ok());
            let path = std::env::temp_dir().join(format!(
                "sketchpad_rs_multi_page_hide_show_{}_{}.gsp",
                std::process::id(),
                index
            ));
            document.save_to(&path).unwrap();
            let saved = std::fs::read(&path).unwrap();
            assert!(!document.objects[0].visible);
            assert!(matches!(
                document.objects[1].kind,
                Kind::HideShowButton {
                    action: VisibilityAction::Show,
                    ..
                }
            ));
            let original_records = records(base).unwrap();
            let saved_records = records(&saved).unwrap();
            let original_page_2 = original_records
                .iter()
                .filter(|record| record.kind == 0x3e8)
                .nth(1)
                .unwrap()
                .start;
            let saved_page_2 = saved_records
                .iter()
                .filter(|record| record.kind == 0x3e8)
                .nth(1)
                .unwrap()
                .start;
            assert_eq!(&saved[saved_page_2..], &base[original_page_2..]);

            document.activate_hide_show_button(2).unwrap();
            document.save_to(&path).unwrap();
            assert_eq!(std::fs::read(&path).unwrap(), base);
            std::fs::remove_file(path).unwrap();
        }
    }
    #[test]
    fn multi_page_link_and_destination_state_save_together() {
        let base = include_bytes!("../tests/fixtures/two_page_hide_show_first_native.gsp");
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = 1;
        document
            .add_link_page_button_with_target(0, Some(1))
            .unwrap();
        document.active_page = 0;
        document.activate_hide_show_button(2).unwrap();
        assert!(document.can_save().is_ok());
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_link_with_state_{}.gsp",
            std::process::id()
        ));
        document.save_to(&path).unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            include_bytes!(
                "../tests/fixtures/action_link_page_activate_hide_show_port_clicked.gsp"
            )
        );
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn free_points_move_and_save_on_both_pages() {
        let base = include_bytes!("../tests/fixtures/two_page_two_points_native.gsp");
        for (index, page, id, position, expected) in [
            (
                0,
                1,
                2,
                Point { x: 261., y: 171. },
                include_bytes!("../tests/fixtures/two_page_point_moved_port.gsp").as_slice(),
            ),
            (
                1,
                0,
                1,
                Point { x: 363., y: 247. },
                include_bytes!("../tests/fixtures/two_page_point_moved_link_port.gsp").as_slice(),
            ),
        ] {
            let mut document = Document::from_bytes(base, None).unwrap();
            document.active_page = page;
            assert!(document.move_point(id, position));
            if index == 1 {
                document.add_link_page_button(1).unwrap();
            }
            assert!(document.can_save().is_ok());
            let path = std::env::temp_dir().join(format!(
                "sketchpad_rs_multi_page_move_{}_{}.gsp",
                std::process::id(),
                index
            ));
            document.save_to(&path).unwrap();
            assert_eq!(std::fs::read(&path).unwrap(), expected);
            assert_eq!(document.position(id), Some(position));
            assert_eq!(document.objects[id - 1].page, page);
            std::fs::remove_file(path).unwrap();
        }
    }
    #[test]
    fn constrained_points_move_and_save_on_second_page() {
        let cases: &[(&[u8], usize, Point, &[u8])] = &[
            (
                include_bytes!("../tests/fixtures/two_page_segment_constrained_native.gsp"),
                5,
                Point { x: 246., y: 111. },
                include_bytes!("../tests/fixtures/two_page_segment_constrained_moved_port.gsp"),
            ),
            (
                include_bytes!("../tests/fixtures/two_page_circle_constrained_native.gsp"),
                5,
                Point { x: 111., y: 111. },
                include_bytes!("../tests/fixtures/two_page_circle_constrained_moved_port.gsp"),
            ),
            (
                include_bytes!("../tests/fixtures/two_page_arc_constrained_native.gsp"),
                6,
                Point { x: 241., y: 64. },
                include_bytes!("../tests/fixtures/two_page_arc_constrained_moved_port.gsp"),
            ),
        ];
        for (index, &(base, id, target, expected)) in cases.iter().enumerate() {
            let mut document = Document::from_bytes(base, None).unwrap();
            document.active_page = 1;
            assert!(document.move_point(id, target));
            assert!(document.can_save().is_ok());
            let path = std::env::temp_dir().join(format!(
                "sketchpad_rs_multi_page_constrained_{}_{}.gsp",
                std::process::id(),
                index
            ));
            document.save_to(&path).unwrap();
            let saved = std::fs::read(&path).unwrap();
            assert_eq!(saved, expected);
            let reopened = Document::from_bytes(&saved, None).unwrap();
            assert_eq!(reopened.page_count, 2);
            assert_eq!(reopened.objects[id - 1].page, 1);
            match (index, &reopened.objects[id - 1].kind) {
                (0, Kind::PointOnSegment(4, t)) => assert!((*t - 0.75).abs() < 1e-12),
                (1, Kind::PointOnCircle(4, x, y)) => {
                    assert!((*x + 1.).abs() < 1e-12);
                    assert!(y.abs() < 1e-12);
                }
                (2, Kind::PointOnArc(5, t)) => assert!((*t - 0.6888239941915089).abs() < 1e-4),
                _ => panic!("Unexpected constrained point after multi-page save"),
            }
            if index == 0 {
                assert_eq!(
                    saved,
                    include_bytes!(
                        "../tests/fixtures/two_page_segment_constrained_dragged_native.gsp"
                    )
                );
            }
            std::fs::remove_file(path).unwrap();
        }
    }
    #[test]
    fn multi_page_constraint_writer_rejects_invalid_edits() {
        for (base, index, replacement) in [
            (
                include_bytes!("../tests/fixtures/two_page_segment_constrained_native.gsp")
                    .as_slice(),
                4,
                Kind::PointOnSegment(2, 0.5),
            ),
            (
                include_bytes!("../tests/fixtures/two_page_arc_constrained_native.gsp").as_slice(),
                5,
                Kind::PointOnArc(5, 1.1),
            ),
            (
                include_bytes!("../tests/fixtures/two_page_circle_constrained_native.gsp")
                    .as_slice(),
                4,
                Kind::PointOnCircle(4, 0.5, 0.5),
            ),
        ] {
            let mut document = Document::from_bytes(base, None).unwrap();
            document.checkpoint();
            document.objects[index].kind = replacement;
            assert!(document.can_save().is_err());
            assert!(document.encode().is_err());
        }
    }
    #[test]
    fn multi_page_visual_styles_save_on_both_pages() {
        let base = include_bytes!("../tests/fixtures/two_page_segment_constrained_native.gsp");
        let mut document = Document::from_bytes(base, None).unwrap();
        assert!(document.partial_view);
        let blue = RgbColor {
            red: 0,
            green: 0,
            blue: 255,
        };
        let purple = RgbColor {
            red: 128,
            green: 0,
            blue: 128,
        };
        let green = RgbColor {
            red: 0,
            green: 128,
            blue: 0,
        };
        document.active_page = 0;
        document.selected.insert(1);
        assert_eq!(
            document
                .set_selected_point_style(PointStyle::Large)
                .unwrap(),
            1
        );
        assert_eq!(document.set_selected_color(blue).unwrap(), 1);
        document.active_page = 1;
        document.selected.clear();
        document.selected.insert(5);
        assert_eq!(
            document
                .set_selected_point_style(PointStyle::Small)
                .unwrap(),
            1
        );
        assert_eq!(document.set_selected_color(purple).unwrap(), 1);
        document.selected.clear();
        document.selected.insert(4);
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
        assert_eq!(document.set_selected_color(green).unwrap(), 1);
        assert!(document.can_save().is_ok());
        let saved = document.encode().unwrap();
        assert_eq!(
            saved,
            include_bytes!("../tests/fixtures/two_page_styled_port.gsp")
        );
        let reopened = Document::from_bytes(&saved, None).unwrap();
        assert_eq!(
            reopened.objects[0].visual_style,
            Some(VisualStyle::Point {
                shape: PointStyle::Large,
                color: blue
            })
        );
        assert_eq!(
            reopened.objects[3].visual_style,
            Some(VisualStyle::Stroke {
                line: SegmentLineStyle {
                    width: LineWidth::Thick,
                    pattern: LinePattern::Dashed
                },
                color: green
            })
        );
        assert_eq!(
            reopened.objects[4].visual_style,
            Some(VisualStyle::Point {
                shape: PointStyle::Small,
                color: purple
            })
        );
        assert!(document.move_point(5, Point { x: 246., y: 111. }));
        document.add_link_page_button(0).unwrap();
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_multi_page_style_link_{}.gsp",
            std::process::id()
        ));
        document.save_to(&path).unwrap();
        let combined = Document::open(&path).unwrap();
        assert_eq!(
            combined.objects[0].visual_style,
            reopened.objects[0].visual_style
        );
        assert_eq!(
            combined.objects[3].visual_style,
            reopened.objects[3].visual_style
        );
        assert_eq!(
            combined.objects[4].visual_style,
            reopened.objects[4].visual_style
        );
        assert!(
            matches!(combined.objects[4].kind, Kind::PointOnSegment(4, t) if (t - 0.75).abs() < 1e-12)
        );
        assert!(matches!(
            combined.objects[5].kind,
            Kind::LinkPageButton { target_page: 0, .. }
        ));
        std::fs::remove_file(path).unwrap();

        let mut invalid = Document::from_bytes(base, None).unwrap();
        invalid.checkpoint();
        invalid.objects[0].visual_style = Some(VisualStyle::Stroke {
            line: SegmentLineStyle {
                width: LineWidth::Thick,
                pattern: LinePattern::Dashed,
            },
            color: green,
        });
        assert!(invalid.can_save().is_err());
    }
    #[test]
    fn multi_page_trace_flags_save_and_toggle_back() {
        let base = include_bytes!("../tests/fixtures/two_page_segment_constrained_native.gsp");
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = 1;
        document.selected.extend([4, 5]);
        assert_eq!(document.set_selected_tracing(true).unwrap(), 2);
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_multi_page_trace_{}.gsp",
            std::process::id()
        ));
        document.save_to(&path).unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            include_bytes!("../tests/fixtures/two_page_traced_port.gsp")
        );
        assert!(document.objects[3].tracing);
        assert!(document.objects[4].tracing);
        assert_eq!(document.set_selected_tracing(false).unwrap(), 2);
        document.save_to(&path).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), base);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn multi_page_points_and_link_append_with_page_local_references() {
        let base = include_bytes!("../tests/fixtures/two_page_segment_constrained_native.gsp");
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = 0;
        let first = document.add(Kind::Free(Point { x: 150., y: 200. }));
        document.active_page = 1;
        let constrained = document.add(Kind::PointOnSegment(4, 0.75));
        let link = document.add_link_page_button(0).unwrap();
        assert_eq!((first, constrained, link), (6, 7, 8));
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_multi_page_additions_{}.gsp",
            std::process::id()
        ));
        document.save_to(&path).unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            include_bytes!("../tests/fixtures/two_page_mixed_additions_port.gsp")
        );
        assert_eq!(document.objects[1].page, 0);
        assert!(
            matches!(document.objects[6].kind, Kind::PointOnSegment(5, t) if (t - 0.75).abs() < 1e-12)
        );
        assert!(matches!(
            document.objects[7].kind,
            Kind::LinkPageButton { target_page: 0, .. }
        ));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn multi_page_point_additions_refuse_cross_page_hosts() {
        let base = include_bytes!("../tests/fixtures/two_page_segment_constrained_native.gsp");
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = 0;
        document.add(Kind::PointOnSegment(4, 0.5));
        assert!(document.can_save().is_err());
    }

    #[test]
    fn multi_page_constrained_point_additions_reopen() {
        for (index, base, kind, expected) in [
            (
                0,
                include_bytes!("../tests/fixtures/two_page_segment_constrained_native.gsp")
                    .as_slice(),
                Kind::PointOnSegment(4, 0.75),
                include_bytes!("../tests/fixtures/two_page_segment_extra_point_port.gsp")
                    .as_slice(),
            ),
            (
                1,
                include_bytes!("../tests/fixtures/two_page_circle_constrained_native.gsp")
                    .as_slice(),
                Kind::PointOnCircle(4, -1., -0.),
                include_bytes!("../tests/fixtures/two_page_circle_extra_point_port.gsp").as_slice(),
            ),
            (
                2,
                include_bytes!("../tests/fixtures/two_page_arc_constrained_native.gsp").as_slice(),
                Kind::PointOnArc(5, 0.6887639439801854),
                include_bytes!("../tests/fixtures/two_page_arc_extra_point_port.gsp").as_slice(),
            ),
        ] {
            let mut document = Document::from_bytes(base, None).unwrap();
            document.active_page = 1;
            let id = document.add(kind);
            let path = std::env::temp_dir().join(format!(
                "sketchpad_rs_multi_page_add_constraint_{}_{}.gsp",
                std::process::id(),
                index
            ));
            document.save_to(&path).unwrap();
            assert_eq!(std::fs::read(&path).unwrap(), expected);
            assert_eq!(document.objects[id - 1].page, 1);
            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn multi_page_straight_and_circle_use_page_local_parents() {
        fn last_object_group(bytes: &[u8]) -> &[u8] {
            let entries = records(bytes).unwrap();
            let start = entries
                .iter()
                .rposition(|entry| entry.kind == 0x07d0)
                .unwrap();
            let end = start
                + entries[start..]
                    .iter()
                    .position(|entry| entry.kind == 0x07d7)
                    .unwrap();
            &bytes[entries[start].start..entries[end].data.end]
        }
        let base = include_bytes!("../tests/fixtures/two_page_blank_native.gsp");
        for (index, first, second, expected, native) in [
            (
                0,
                Point { x: 111., y: 111. },
                Point { x: 291., y: 111. },
                include_bytes!("../tests/fixtures/two_page_segment_port.gsp").as_slice(),
                include_bytes!("../tests/fixtures/two_page_segment_native.gsp").as_slice(),
            ),
            (
                1,
                Point { x: 201., y: 111. },
                Point { x: 291., y: 111. },
                include_bytes!("../tests/fixtures/two_page_circle_port.gsp").as_slice(),
                include_bytes!("../tests/fixtures/two_page_circle_native.gsp").as_slice(),
            ),
            (
                2,
                Point { x: 111., y: 111. },
                Point { x: 291., y: 111. },
                include_bytes!("../tests/fixtures/two_page_line_port.gsp").as_slice(),
                include_bytes!("../tests/fixtures/two_page_line_native.gsp").as_slice(),
            ),
            (
                3,
                Point { x: 111., y: 111. },
                Point { x: 291., y: 111. },
                include_bytes!("../tests/fixtures/two_page_ray_port.gsp").as_slice(),
                include_bytes!("../tests/fixtures/two_page_ray_native.gsp").as_slice(),
            ),
        ] {
            let mut document = Document::from_bytes(base, None).unwrap();
            document.active_page = 1;
            let a = document.add(Kind::Free(first));
            let b = document.add(Kind::Free(second));
            let path_id = document.add(match index {
                0 => Kind::Segment(a, b),
                1 => Kind::Circle(a, b),
                2 => Kind::Line(a, b),
                3 => Kind::Ray(a, b),
                _ => unreachable!(),
            });
            let path = std::env::temp_dir().join(format!(
                "sketchpad_rs_multi_page_path_{}_{}.gsp",
                std::process::id(),
                index
            ));
            document.save_to(&path).unwrap();
            assert_eq!(std::fs::read(&path).unwrap(), expected);
            assert_eq!(last_object_group(expected), last_object_group(native));
            assert_eq!(document.objects[path_id - 1].page, 1);
            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn multi_page_constructed_paths_and_points_remap_across_pages() {
        let base = include_bytes!("../tests/fixtures/two_page_blank_native.gsp");
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = 0;
        let first = document.add(Kind::Free(Point { x: 150., y: 200. }));
        let second = document.add(Kind::Free(Point { x: 250., y: 200. }));
        document.add(Kind::Segment(first, second));
        document.active_page = 1;
        let center = document.add(Kind::Free(Point { x: 111., y: 111. }));
        let radius = document.add(Kind::Free(Point { x: 291., y: 111. }));
        let segment = document.add(Kind::Segment(center, radius));
        document.add(Kind::PointOnSegment(segment, 0.5));
        let circle = document.add(Kind::Circle(center, radius));
        document.add(Kind::PointOnCircle(circle, 0., -1.));
        document.add_link_page_button(0).unwrap();
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_multi_page_construct_mixed_{}.gsp",
            std::process::id()
        ));
        document.save_to(&path).unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            include_bytes!("../tests/fixtures/two_page_construct_mixed_port.gsp")
        );
        assert_eq!(document.objects[6].kind, Kind::Segment(5, 6));
        assert_eq!(document.objects[8].kind, Kind::Circle(5, 6));
        assert_eq!(document.objects[9].kind, Kind::PointOnCircle(9, 0., -1.));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn multi_page_path_addition_rejects_cross_page_parent() {
        let base = include_bytes!("../tests/fixtures/two_page_blank_native.gsp");
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = 1;
        let new_point = document.add(Kind::Free(Point { x: 200., y: 200. }));
        document.add(Kind::Segment(1, new_point));
        assert!(document.can_save().is_err());
    }
    #[test]
    fn multi_page_midpoint_and_circle_by_segment_match_native_groups() {
        fn last_group(bytes: &[u8]) -> &[u8] {
            let entries = records(bytes).unwrap();
            let start = entries
                .iter()
                .rposition(|entry| entry.kind == 0x07d0)
                .unwrap();
            let end = start
                + entries[start..]
                    .iter()
                    .position(|entry| entry.kind == 0x07d7)
                    .unwrap();
            &bytes[entries[start].start..entries[end].data.end]
        }
        let base = include_bytes!("../tests/fixtures/two_page_segment_native.gsp");
        for (index, kind, expected, native) in [
            (
                0,
                Kind::Midpoint(2, 3),
                include_bytes!("../tests/fixtures/two_page_midpoint_port.gsp").as_slice(),
                include_bytes!("../tests/fixtures/two_page_midpoint_native.gsp").as_slice(),
            ),
            (
                1,
                Kind::CircleBySegment(2, 4),
                include_bytes!("../tests/fixtures/two_page_circle_segment_port.gsp").as_slice(),
                include_bytes!("../tests/fixtures/two_page_circle_segment_native.gsp").as_slice(),
            ),
        ] {
            let mut document = Document::from_bytes(base, None).unwrap();
            document.active_page = 1;
            let id = document.add(kind.clone());
            let path = std::env::temp_dir().join(format!(
                "sketchpad_rs_multi_page_mid_circle_{}_{}.gsp",
                std::process::id(),
                index
            ));
            document.save_to(&path).unwrap();
            assert_eq!(std::fs::read(&path).unwrap(), expected);
            assert_eq!(last_group(expected), last_group(native));
            assert_eq!(document.objects[id - 1].kind, kind);
            if index == 0 {
                assert_eq!(document.objects[id - 1].midpoint_host, Some(4));
            }
            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn multi_page_midpoint_and_circle_by_new_segment_reopen() {
        let base = include_bytes!("../tests/fixtures/two_page_blank_native.gsp");
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = 1;
        let a = document.add(Kind::Free(Point { x: 111., y: 111. }));
        let b = document.add(Kind::Free(Point { x: 291., y: 111. }));
        let segment = document.add(Kind::Segment(a, b));
        let midpoint = document.add(Kind::Midpoint(a, b));
        let circle = document.add(Kind::CircleBySegment(a, segment));
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_multi_page_new_mid_circle_{}.gsp",
            std::process::id()
        ));
        document.save_to(&path).unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            include_bytes!("../tests/fixtures/two_page_midpoint_circle_segment_mixed_port.gsp")
        );
        assert_eq!(document.objects[midpoint - 1].midpoint_host, Some(segment));
        assert_eq!(
            document.objects[circle - 1].kind,
            Kind::CircleBySegment(a, segment)
        );
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn multi_page_derived_lines_and_intersections_save() {
        fn object_group_with_code(bytes: &[u8], code: u16) -> &[u8] {
            let entries = records(bytes).unwrap();
            let start = entries
                .iter()
                .position(|entry| {
                    entry.kind == 0x07d0
                        && entry.data.len() >= 2
                        && u16::from_le_bytes(
                            bytes[entry.data.start..entry.data.start + 2]
                                .try_into()
                                .unwrap(),
                        ) == code
                })
                .unwrap();
            let end = start
                + entries[start..]
                    .iter()
                    .position(|entry| entry.kind == 0x07d7)
                    .unwrap();
            &bytes[entries[start].start..entries[end].data.end]
        }
        fn saved(mut document: Document, expected: &[u8], name: &str) -> Document {
            let path = std::env::temp_dir().join(format!(
                "sketchpad_rs_multi_page_{}_{}.gsp",
                name,
                std::process::id()
            ));
            document.save_to(&path).unwrap();
            assert_eq!(std::fs::read(&path).unwrap(), expected);
            std::fs::remove_file(path).unwrap();
            document
        }
        let segment_base = include_bytes!("../tests/fixtures/two_page_segment_native.gsp");
        let mut document = Document::from_bytes(segment_base, None).unwrap();
        document.active_page = 1;
        let point = document.add(Kind::Free(Point { x: 201., y: 211. }));
        document.add(Kind::Parallel(point, 4));
        document.add(Kind::Perpendicular(point, 4));
        let reopened = saved(
            document,
            include_bytes!("../tests/fixtures/two_page_parallel_perpendicular_port.gsp"),
            "derived",
        );
        assert!(reopened.line_points(6).is_some());
        assert!(reopened.line_points(7).is_some());
        let generated =
            include_bytes!("../tests/fixtures/two_page_parallel_perpendicular_port.gsp");
        assert_eq!(
            object_group_with_code(generated, 6),
            object_group_with_code(
                include_bytes!("../tests/fixtures/two_page_parallel_native.gsp"),
                6,
            )
        );
        assert_eq!(
            object_group_with_code(generated, 5),
            object_group_with_code(
                include_bytes!("../tests/fixtures/two_page_perpendicular_native.gsp"),
                5,
            )
        );

        let mut document = Document::from_bytes(segment_base, None).unwrap();
        document.active_page = 1;
        let first = document.add(Kind::Free(Point { x: 201., y: 51. }));
        let second = document.add(Kind::Free(Point { x: 201., y: 211. }));
        let line = document.add(Kind::Line(first, second));
        let crossing = document.add(Kind::Intersection(4, line));
        let reopened = saved(
            document,
            include_bytes!("../tests/fixtures/two_page_straight_intersection_port.gsp"),
            "straight_crossing",
        );
        assert_eq!(
            reopened.position(crossing),
            Some(Point { x: 201., y: 111. })
        );
        assert_eq!(
            object_group_with_code(
                include_bytes!("../tests/fixtures/two_page_straight_intersection_port.gsp"),
                9,
            ),
            object_group_with_code(
                include_bytes!("../tests/fixtures/two_page_straight_intersection_native.gsp"),
                9,
            )
        );

        let mut document = Document::from_bytes(segment_base, None).unwrap();
        document.active_page = 1;
        let center = document.add(Kind::Free(Point { x: 201., y: 161. }));
        let radius = document.add(Kind::Free(Point { x: 281., y: 161. }));
        let circle = document.add(Kind::Circle(center, radius));
        let first = document.add(Kind::LineCircleIntersection(4, circle, 0));
        let second = document.add(Kind::LineCircleIntersection(4, circle, 1));
        let reopened = saved(
            document,
            include_bytes!("../tests/fixtures/two_page_line_circle_intersections_port.gsp"),
            "line_circle_crossings",
        );
        assert_ne!(reopened.position(first), reopened.position(second));
        assert!(reopened.position(first).is_some());
        assert!(reopened.position(second).is_some());

        let mut document = Document::from_bytes(
            include_bytes!("../tests/fixtures/two_page_blank_native.gsp"),
            None,
        )
        .unwrap();
        document.active_page = 1;
        let a = document.add(Kind::Free(Point { x: 160., y: 160. }));
        let b = document.add(Kind::Free(Point { x: 260., y: 160. }));
        let first_circle = document.add(Kind::Circle(a, b));
        let c = document.add(Kind::Free(Point { x: 240., y: 160. }));
        let d = document.add(Kind::Free(Point { x: 340., y: 160. }));
        let second_circle = document.add(Kind::Circle(c, d));
        let first = document.add(Kind::CircleCircleIntersection(
            first_circle,
            second_circle,
            0,
        ));
        let second = document.add(Kind::CircleCircleIntersection(
            first_circle,
            second_circle,
            1,
        ));
        let reopened = saved(
            document,
            include_bytes!("../tests/fixtures/two_page_circle_circle_intersections_port.gsp"),
            "circle_crossings",
        );
        assert_ne!(reopened.position(first), reopened.position(second));
        assert!(reopened.position(first).is_some());
        assert!(reopened.position(second).is_some());
    }

    #[test]
    fn multi_page_intersection_requires_finite_crossing() {
        let base = include_bytes!("../tests/fixtures/two_page_segment_native.gsp");
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = 1;
        let point = document.add(Kind::Free(Point { x: 201., y: 211. }));
        let parallel = document.add(Kind::Parallel(point, 4));
        document.add(Kind::Intersection(4, parallel));
        assert!(document.can_save().is_err());
    }
    #[test]
    fn multi_page_state_writer_refuses_geometry_changes() {
        let base = include_bytes!(
            "../tests/fixtures/action_link_page_activate_hide_show_port_generated.gsp"
        );
        let mut document = Document::from_bytes(base, None).unwrap();
        document.checkpoint();
        document.objects[0].kind = Kind::Segment(1, 2);
        assert!(document.can_save().is_err());
        assert!(document.encode().is_err());
    }
    #[test]
    fn animation_button_variants_import_and_preserve_settings() {
        use crate::action_records::AnimationDirection;
        for (bytes, direction, speed, once) in [
            (
                include_bytes!("../tests/fixtures/action_animation_forward_original.gsp")
                    .as_slice(),
                AnimationDirection::Forward,
                24.,
                false,
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_backward_original.gsp")
                    .as_slice(),
                AnimationDirection::Backward,
                24.,
                false,
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_random_original.gsp").as_slice(),
                AnimationDirection::Random,
                24.,
                false,
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_slow_original.gsp").as_slice(),
                AnimationDirection::Bidirectional,
                4.,
                false,
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_fast_original.gsp").as_slice(),
                AnimationDirection::Bidirectional,
                80.,
                false,
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_forward_once_original.gsp")
                    .as_slice(),
                AnimationDirection::Forward,
                24.,
                true,
            ),
        ] {
            let doc = Document::from_bytes(bytes, None).unwrap();
            assert!(!doc.partial_view);
            assert!(matches!(
                doc.objects[4].kind,
                Kind::AnimationButton { ref settings, .. }
                    if settings.len() == 1 && settings[0] == AnimationSettings { direction, speed, once }
            ));
            assert_eq!(doc.encode().unwrap(), bytes);
        }
        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/animation_button_base.gsp"),
            None,
        )
        .unwrap();
        let settings = AnimationSettings {
            direction: AnimationDirection::Backward,
            speed: 80.,
            once: true,
        };
        let id = doc.add_animation_button_with_settings(4, settings).unwrap();
        let encoded = doc.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert!(
            matches!(reopened.objects[id-1].kind, Kind::AnimationButton { settings: ref actual, .. } if actual == &[settings])
        );
        assert_eq!(reopened.encode().unwrap(), encoded);
        for (bytes, target, button) in [
            (
                include_bytes!("../tests/fixtures/action_animation_circle_original.gsp").as_slice(),
                4,
                5,
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_free_original.gsp").as_slice(),
                1,
                2,
            ),
        ] {
            let native = Document::from_bytes(bytes, None).unwrap();
            assert!(!native.partial_view);
            assert!(
                matches!(native.objects[button-1].kind, Kind::AnimationButton { ref targets, ref settings, .. } if targets == &[target] && settings == &[AnimationSettings { direction: AnimationDirection::Forward, speed: 24., once: false }])
            );
            assert_eq!(native.encode().unwrap(), bytes);
        }
        let mut circle_doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/animation_button_circle_base.gsp"),
            None,
        )
        .unwrap();
        let circle_id = circle_doc.add_animation_button(4).unwrap();
        assert!(matches!(
            circle_doc.objects[circle_id - 1].kind,
            Kind::AnimationButton { ref settings, .. }
                if settings.len() == 1 && settings[0].direction == AnimationDirection::Forward
        ));
        let mut free_doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/animation_free_point_base.gsp"),
            None,
        )
        .unwrap();
        let free_id = free_doc.add_animation_button(1).unwrap();
        assert!(matches!(
            free_doc.objects[free_id - 1].kind,
            Kind::AnimationButton { ref settings, .. }
                if settings.len() == 1 && settings[0].direction == AnimationDirection::Forward
        ));
    }

    #[test]
    fn two_point_animation_button_imports_creates_and_copies() {
        use crate::action_records::AnimationDirection;
        let mut native = Document::open(Path::new(
            "tests/fixtures/action_animation_two_points_second_backward.gsp",
        ))
        .unwrap();
        let expected_settings = vec![
            AnimationSettings::default(),
            AnimationSettings {
                direction: AnimationDirection::Backward,
                ..AnimationSettings::default()
            },
        ];
        for bytes in [
            include_bytes!("../tests/fixtures/action_animation_two_points_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/action_animation_two_points_second_backward.gsp")
                .as_slice(),
            include_bytes!("../tests/fixtures/action_animation_two_points_clicked.gsp").as_slice(),
            include_bytes!(
                "../tests/fixtures/action_animation_two_points_port_clicked_by_wine.gsp"
            )
            .as_slice(),
        ] {
            let doc = Document::from_bytes(bytes, None).unwrap();
            assert!(!doc.partial_view);
            assert_eq!(doc.objects[5].kind.dependencies(), vec![4, 5]);
            assert_eq!(doc.encode().unwrap(), bytes);
        }
        assert!(
            matches!(&native.objects[5].kind, Kind::AnimationButton { settings, .. } if settings == &expected_settings)
        );
        native.selected.insert(6);
        let clipboard = native.copy_selected().unwrap();
        let mut destination = Document::new();
        let pasted = destination.paste(&clipboard).unwrap();
        assert_eq!(
            destination.objects[pasted.last().unwrap() - 1]
                .kind
                .dependencies(),
            vec![4, 5]
        );
        assert!(
            !Document::from_bytes(&destination.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );

        let mut generated = Document::open(Path::new(
            "tests/fixtures/animation_button_two_points_base.gsp",
        ))
        .unwrap();
        let button = generated
            .add_animation_button_for_targets_with_settings(&[
                (4, expected_settings[0]),
                (5, expected_settings[1]),
            ])
            .unwrap();
        assert_eq!(button, 6);
        assert!(
            generated
                .add_animation_button_for_targets_with_settings(&[
                    (4, expected_settings[0]),
                    (4, expected_settings[1])
                ])
                .is_err()
        );
        let encoded = generated.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(
            matches!(&reopened.objects[5].kind, Kind::AnimationButton { settings, .. } if settings == &expected_settings)
        );
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
    fn clearing_trailing_function_objects_keeps_savable_state() {
        let source = include_bytes!("../tests/fixtures/function_plot_square.gsp");
        let mut plot = Document::from_bytes(source, None).unwrap();
        plot.selected.insert(8);
        assert_eq!(plot.clear_selected().unwrap(), vec![8]);
        let reopened = Document::from_bytes(&plot.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects.len(), 7);
        assert!(matches!(
            reopened.objects[6].kind,
            Kind::FunctionDefinition(_)
        ));

        let mut definition = Document::from_bytes(source, None).unwrap();
        definition.selected.insert(7);
        assert_eq!(definition.clear_selected().unwrap(), vec![7, 8]);
        let reopened = Document::from_bytes(&definition.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects.len(), 6);

        let mut unsafe_edit = Document::from_bytes(source, None).unwrap();
        unsafe_edit.selected.insert(1);
        assert!(unsafe_edit.clear_selected().is_err());
        assert!(!unsafe_edit.dirty);
        assert_eq!(unsafe_edit.encode().unwrap(), source);

        let mut with_points = Document::from_bytes(source, None).unwrap();
        let first = with_points.add(Kind::Free(Point { x: 310., y: 190. }));
        let second = with_points.add(Kind::Free(Point { x: 340., y: 210. }));
        let mut saved = Document::from_bytes(&with_points.encode().unwrap(), None).unwrap();
        saved.selected.insert(first);
        assert_eq!(saved.clear_selected().unwrap(), vec![first]);
        let reopened = Document::from_bytes(&saved.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(matches!(
            reopened.objects[7].kind,
            Kind::FunctionPlot(_, _, _, _, _)
        ));
        assert_eq!(
            reopened.position(second - 1),
            Some(Point { x: 340., y: 210. })
        );
    }
    #[test]
    fn clearing_independent_point_before_saved_plot_remaps_its_dependencies() {
        let mut original = Document::new();
        let extra = original.add(Kind::Free(Point { x: 100., y: 100. }));
        let plot = original
            .plot_new_function(SimpleFunction::Square, Point { x: 199., y: 125. })
            .unwrap();
        let mut saved = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
        let before = saved.function_plot_points(plot, 8).unwrap();
        saved.selected.insert(extra);
        assert_eq!(saved.clear_selected().unwrap(), vec![extra]);
        let reopened = Document::from_bytes(&saved.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.function_plot_points(plot - 1, 8).unwrap(), before);
        assert!(matches!(
            reopened.objects[plot - 2].kind,
            Kind::FunctionPlot(7, 6, _, _, _)
        ));
    }
    #[test]
    fn clearing_independent_text_before_saved_plot_remaps_its_dependencies() {
        let mut original = Document::new();
        let text = original.add_text("hello", Point { x: 10., y: 5. }).unwrap();
        let plot = original
            .plot_new_function(SimpleFunction::Square, Point { x: 199., y: 125. })
            .unwrap();
        let mut saved = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
        let before = saved.function_plot_points(plot, 8).unwrap();
        saved.selected.insert(text);
        assert_eq!(saved.clear_selected().unwrap(), vec![text]);
        let reopened = Document::from_bytes(&saved.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.function_plot_points(plot - 1, 8).unwrap(), before);
    }
    #[test]
    fn clearing_earlier_point_remaps_table_and_live_path_sources() {
        let mut original = Document::new();
        let extra = original.add(Kind::Free(Point { x: 100., y: 100. }));
        let parameter = original.add_default_parameter((10, 5)).unwrap();
        original
            .tabulate_default_parameter(parameter, (10, 38))
            .unwrap();
        original
            .add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        original.plot_parameter_on_path(parameter, None).unwrap();
        let mut saved = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
        saved.selected.insert(extra);
        assert_eq!(saved.clear_selected().unwrap(), vec![extra]);
        let reopened = Document::from_bytes(&saved.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(
            matches!(&reopened.objects[1].kind, Kind::ValueTable(1, group) if group.source_id() == 1)
        );
        assert!(matches!(
            reopened.objects[8].kind,
            Kind::PathValuePoint(1, _)
        ));

        let mut pair = Document::new();
        let extra = pair.add(Kind::Free(Point { x: 100., y: 100. }));
        pair.add_default_parameter((10, 5)).unwrap();
        pair.add_default_parameter((10, 38)).unwrap();
        pair.tabulate_default_pair((10, 71)).unwrap();
        let mut saved = Document::from_bytes(&pair.encode().unwrap(), None).unwrap();
        saved.selected.insert(extra);
        assert_eq!(saved.clear_selected().unwrap(), vec![extra]);
        let reopened = Document::from_bytes(&saved.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(
            matches!(&reopened.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.pair_source_ids() == Some([2, 1]))
        );
    }
    #[test]
    fn clearing_parameter_removes_its_table_and_path_point_before_save() {
        let mut original = Document::new();
        original.add(Kind::Free(Point { x: 100., y: 100. }));
        let parameter = original.add_default_parameter((10, 5)).unwrap();
        original
            .tabulate_default_parameter(parameter, (10, 38))
            .unwrap();
        original
            .add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        original.plot_parameter_on_path(parameter, None).unwrap();
        let mut saved = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
        saved.selected.insert(parameter);
        let removed = saved.clear_selected().unwrap();
        assert_eq!(removed.len(), 3);
        let reopened = Document::from_bytes(&saved.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects.len(), 7);
        assert!(
            reopened
                .objects
                .iter()
                .any(|object| matches!(object.kind, Kind::CoordinateSystem(_, _)))
        );
        assert!(reopened.objects.iter().all(|object| !matches!(
            object.kind,
            Kind::Parameter(_) | Kind::ValueTable(_, _) | Kind::PathValuePoint(_, _)
        )));
    }
    #[test]
    fn copying_tables_remaps_embedded_source_ids() {
        let mut one = Document::new();
        let source = one.add_default_parameter((10, 5)).unwrap();
        let table = one.tabulate_default_parameter(source, (10, 38)).unwrap();
        one.selected.insert(table);
        let clipboard = one.copy_selected().unwrap();
        let pasted = one.paste(&clipboard).unwrap();
        assert_eq!(pasted.len(), 2);
        let reopened = Document::from_bytes(&one.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(
            matches!(&reopened.objects[pasted[1] - 1].kind, Kind::ValueTable(id, group) if *id == pasted[0] && group.source_id() == pasted[0] as u32)
        );
        assert!(
            matches!(&reopened.objects[pasted[0] - 1].kind, Kind::Parameter(group) if group.anchor() == (30, 25))
        );
        assert!(
            matches!(&reopened.objects[pasted[1] - 1].kind, Kind::ValueTable(_, group) if group.anchor() == (30, 58))
        );

        let mut pair = Document::new();
        pair.add_default_parameter((10, 5)).unwrap();
        pair.add_default_parameter((10, 38)).unwrap();
        let table = pair.tabulate_default_pair((10, 71)).unwrap();
        pair.selected.insert(table);
        let clipboard = pair.copy_selected().unwrap();
        let pasted = pair.paste(&clipboard).unwrap();
        assert_eq!(pasted.len(), 3);
        let reopened = Document::from_bytes(&pair.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(
            matches!(&reopened.objects[pasted[2] - 1].kind, Kind::ValueTablePair(first, second, group) if *first == pasted[1] && *second == pasted[0] && group.pair_source_ids() == Some([pasted[1] as u32, pasted[0] as u32]))
        );
        assert!(
            matches!(&reopened.objects[pasted[2] - 1].kind, Kind::ValueTablePair(_, _, group) if group.anchor() == (30, 91))
        );
    }
    #[test]
    fn copying_function_plot_keeps_its_frame_style_and_curve() {
        let mut original = Document::new();
        let plot = original
            .plot_new_function(SimpleFunction::Square, Point { x: 199., y: 125. })
            .unwrap();
        let mut saved = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
        let before = saved.function_plot_points(plot, 8).unwrap();
        saved.selected.insert(plot);
        let clipboard = saved.copy_selected().unwrap();
        let pasted = saved.paste(&clipboard).unwrap();
        assert_eq!(pasted.len(), 8);
        let reopened = Document::from_bytes(&saved.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.frame_styles.get(&pasted[5]),
            Some(&(GridForm::Square, false))
        );
        assert!(reopened.objects[pasted[0] - 1].visible);
        assert!(
            pasted[1..=5]
                .iter()
                .all(|id| !reopened.objects[*id - 1].visible)
        );
        assert_eq!(
            reopened.objects[pasted[6] - 1].function_anchor,
            Some((30, 25))
        );
        assert!(
            matches!(reopened.objects[pasted[7] - 1].kind, Kind::FunctionPlot(definition, frame, _, _, _) if definition == pasted[6] && frame == pasted[5])
        );
        let after = reopened.function_plot_points(pasted[7], 8).unwrap();
        assert!(
            before
                .iter()
                .zip(after)
                .all(|(a, b)| (a.x + 20. - b.x).abs() < 1e-8 && (a.y + 20. - b.y).abs() < 1e-8)
        );
    }
    #[test]
    fn copying_selected_function_includes_its_plot_and_hides_frame_parts() {
        let mut original = Document::new();
        let plot = original
            .plot_new_function(SimpleFunction::Square, Point { x: 199., y: 125. })
            .unwrap();
        let mut saved = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
        saved.selected.insert(plot - 1);
        let clipboard = saved.copy_selected().unwrap();
        assert_eq!(clipboard.len(), 8);
        let pasted = saved.paste(&clipboard).unwrap();
        let reopened = Document::from_bytes(&saved.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(reopened.objects[pasted[0] - 1].visible);
        assert!(
            pasted[1..=5]
                .iter()
                .all(|id| !reopened.objects[*id - 1].visible)
        );
        assert!(reopened.objects[pasted[6] - 1].visible);
        assert!(reopened.objects[pasted[7] - 1].visible);
    }
    #[test]
    fn copying_selected_parametric_source_includes_curve_and_hides_frame_parts() {
        let mut native = Document::open(Path::new(
            "tests/fixtures/parametric_rectangular_original.gsp",
        ))
        .unwrap();
        let before = native.parametric_plot_segments(9, 8).unwrap();
        native.selected.insert(1);
        let clipboard = native.copy_selected().unwrap();
        assert_eq!(clipboard.len(), 9);
        let pasted = native.paste(&clipboard).unwrap();
        let reopened = Document::from_bytes(&native.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.objects[pasted[0] - 1].function_anchor,
            Some((30, 25))
        );
        assert_eq!(
            reopened.objects[pasted[1] - 1].function_anchor,
            Some((30, 69))
        );
        assert_eq!(
            reopened.function_display_text(pasted[0]).as_deref(),
            Some("g(x) = x")
        );
        assert_eq!(
            reopened.function_display_text(pasted[1]).as_deref(),
            Some("f(x) = x²")
        );
        assert!(reopened.objects[pasted[2] - 1].visible);
        assert!(
            pasted[3..=7]
                .iter()
                .all(|id| !reopened.objects[*id - 1].visible)
        );
        assert!(
            matches!(reopened.objects[pasted[8] - 1].kind, Kind::ParametricPlot(x, y, frame, _, _, _, _) if x == pasted[0] && y == pasted[1] && frame == pasted[7])
        );
        let after = reopened.parametric_plot_segments(pasted[8], 8).unwrap();
        assert!(before.iter().zip(after).all(|((a, b), (c, d))| {
            (a.x + 20. - c.x).abs() < 1e-8
                && (a.y + 20. - c.y).abs() < 1e-8
                && (b.x + 20. - d.x).abs() < 1e-8
                && (b.y + 20. - d.y).abs() < 1e-8
        }));

        let mut second = Document::open(Path::new(
            "tests/fixtures/parametric_rectangular_original.gsp",
        ))
        .unwrap();
        second.selected.insert(2);
        let clipboard = second.copy_selected().unwrap();
        let pasted = second.paste(&clipboard).unwrap();
        let reopened = Document::from_bytes(&second.encode().unwrap(), None).unwrap();
        assert_eq!(
            reopened.function_display_text(pasted[0]).as_deref(),
            Some("f(x) = x")
        );
        assert_eq!(
            reopened.function_display_text(pasted[1]).as_deref(),
            Some("g(x) = x²")
        );
    }
    #[test]
    fn native_copied_parametric_functions_keep_their_saved_names() {
        let native = Document::open(Path::new(
            "tests/fixtures/parametric_function_copied_native.gsp",
        ))
        .unwrap();
        assert!(!native.partial_view);
        assert_eq!(
            native.function_display_text(10).as_deref(),
            Some("f(x) = x²")
        );
        assert_eq!(
            native.function_display_text(12).as_deref(),
            Some("g(x) = x")
        );
        let second = Document::open(Path::new(
            "tests/fixtures/parametric_second_function_copied_native.gsp",
        ))
        .unwrap();
        assert!(!second.partial_view);
        assert_eq!(
            second.function_display_text(10).as_deref(),
            Some("f(x) = x")
        );
        assert_eq!(
            second.function_display_text(12).as_deref(),
            Some("g(x) = x²")
        );
    }
    #[test]
    fn native_copied_plot_imports_and_moves_as_an_editable_curve() {
        let mut native =
            Document::open(Path::new("tests/fixtures/function_plot_copied_native.gsp")).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.objects[9].function_anchor, Some((30, 25)));
        let before = native.function_plot_points(16, 8).unwrap();
        assert!(native.move_point(9, Point { x: 239., y: 145. }));
        let reopened = Document::from_bytes(&native.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        let after = reopened.function_plot_points(16, 8).unwrap();
        assert!(
            before
                .iter()
                .zip(after)
                .all(|(a, b)| (a.x + 20. - b.x).abs() < 1e-8 && (a.y - b.y).abs() < 1e-8)
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
    fn vector_translation_tracks_marked_points_and_round_trips() {
        let native =
            Document::open(Path::new("tests/fixtures/vector_translation_original.gsp")).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.objects[4].kind, Kind::VectorTranslatedPoint(4, 1, 2));
        assert_eq!(native.position(5), Some(Point { x: 170., y: 210. }));

        let mut created = Document::new();
        let start = created.add(Kind::Free(Point { x: 150., y: 120. }));
        let end = created.add(Kind::Free(Point { x: 220., y: 130. }));
        created.add(Kind::Segment(start, end));
        let source = created.add(Kind::Free(Point { x: 100., y: 200. }));
        let image = created.add(Kind::VectorTranslatedPoint(source, start, end));
        assert_eq!(created.position(image), Some(Point { x: 170., y: 210. }));
        let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.objects[4].kind,
            Kind::VectorTranslatedPoint(4, 1, 2)
        );
        assert!(reopened.move_point(end, Point { x: 250., y: 130. }));
        assert_eq!(reopened.position(image), Some(Point { x: 200., y: 210. }));
        let moved = Document::from_bytes(&reopened.encode().unwrap(), None).unwrap();
        assert_eq!(moved.position(image), reopened.position(image));
    }
    #[test]
    fn marked_distance_translation_imports_and_tracks_measurement() {
        let bytes = fs::read("tests/fixtures/marked_distance_translation_original.gsp").unwrap();
        let native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(
            native.objects[4].kind,
            Kind::MarkedDistanceTranslatedPoint(3, 4, 90.)
        );
        assert_eq!(native.position(5), Some(Point { x: 100., y: 120. }));
        assert_eq!(native.encode().unwrap(), bytes);

        let mut created = Document::new();
        let start = created.add(Kind::Free(Point { x: 150., y: 120. }));
        let end = created.add(Kind::Free(Point { x: 250., y: 120. }));
        let source = created.add(Kind::Free(Point { x: 100., y: 220. }));
        let distance = created.add(Kind::PointDistance(start, end));
        let image = created.add(Kind::MarkedDistanceTranslatedPoint(source, distance, 90.));
        let horizontal = created.add(Kind::MarkedDistanceTranslatedPoint(source, distance, 0.));
        assert_eq!(created.position(image), Some(Point { x: 100., y: 120. }));
        assert_eq!(
            created.position(horizontal),
            Some(Point { x: 200., y: 220. })
        );
        let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[4].kind, created.objects[4].kind);
        assert!(reopened.move_point(end, Point { x: 300., y: 120. }));
        let moved = reopened.position(image).unwrap();
        assert!((moved.x - 100.).abs() < 1e-9 && (moved.y - 70.).abs() < 1e-9);
        assert_eq!(
            reopened.position(horizontal),
            Some(Point { x: 250., y: 220. })
        );
    }
    #[test]
    fn marked_distance_shape_translations_import_and_track_measurement() {
        for (path, expected_kind) in [
            (
                "tests/fixtures/marked_distance_segment_translation_original.gsp",
                Kind::MarkedDistanceTranslatedSegment(6, 5, 90.),
            ),
            (
                "tests/fixtures/marked_distance_circle_translation_original.gsp",
                Kind::MarkedDistanceTranslatedCircle(6, 5, 90.),
            ),
        ] {
            let bytes = fs::read(path).unwrap();
            let native = Document::from_bytes(&bytes, None).unwrap();
            assert!(!native.partial_view, "{path}");
            assert_eq!(native.objects[6].kind, expected_kind, "{path}");
            assert_eq!(native.encode().unwrap(), bytes, "{path}");
        }

        let mut created = Document::new();
        let measure_start = created.add(Kind::Free(Point { x: 150., y: 120. }));
        let measure_end = created.add(Kind::Free(Point { x: 250., y: 120. }));
        let source_start = created.add(Kind::Free(Point { x: 100., y: 220. }));
        let source_end = created.add(Kind::Free(Point { x: 200., y: 220. }));
        let distance = created.add(Kind::PointDistance(measure_start, measure_end));
        let segment = created.add(Kind::Segment(source_start, source_end));
        let circle = created.add(Kind::Circle(source_start, source_end));
        let moved_segment = created.add(Kind::MarkedDistanceTranslatedSegment(
            segment, distance, 90.,
        ));
        let moved_circle = created.add(Kind::MarkedDistanceTranslatedCircle(circle, distance, 90.));
        let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(reopened.move_point(measure_end, Point { x: 300., y: 120. }));
        let (a, b, mode) = reopened.line_points(moved_segment).unwrap();
        assert_eq!(mode, LineMode::Segment);
        assert!((a.x - 100.).abs() < 1e-9 && (a.y - 70.).abs() < 1e-9);
        assert!((b.x - 200.).abs() < 1e-9 && (b.y - 70.).abs() < 1e-9);
        let (center, radius) = reopened.circle_points(moved_circle).unwrap();
        assert!((center.x - 100.).abs() < 1e-9 && (center.y - 70.).abs() < 1e-9);
        assert_eq!(radius, 100.);
    }
    #[test]
    fn rectangular_marked_translation_imports_and_tracks_both_measurements() {
        let bytes = fs::read("tests/fixtures/rectangular_marked_translation_original.gsp").unwrap();
        let native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(
            native.objects[7].kind,
            Kind::RectangularMarkedTranslatedPoint(5, 6, 7)
        );
        assert_eq!(native.position(8), Some(Point { x: 200., y: 160. }));
        assert_eq!(native.encode().unwrap(), bytes);

        let mut created = Document::new();
        let h_start = created.add(Kind::Free(Point { x: 150., y: 120. }));
        let h_end = created.add(Kind::Free(Point { x: 250., y: 120. }));
        let v_start = created.add(Kind::Free(Point { x: 150., y: 180. }));
        let v_end = created.add(Kind::Free(Point { x: 150., y: 230. }));
        let source = created.add(Kind::Free(Point { x: 100., y: 210. }));
        let horizontal = created.add(Kind::PointDistance(h_start, h_end));
        let vertical = created.add(Kind::PointDistance(v_start, v_end));
        let image = created.add(Kind::RectangularMarkedTranslatedPoint(
            source, horizontal, vertical,
        ));
        assert_eq!(created.position(image), Some(Point { x: 200., y: 160. }));
        let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[7].kind, created.objects[7].kind);
        assert!(reopened.move_point(h_end, Point { x: 300., y: 120. }));
        assert!(reopened.move_point(v_end, Point { x: 150., y: 250. }));
        assert_eq!(reopened.position(image), Some(Point { x: 250., y: 140. }));
    }
    #[test]
    fn fixed_rectangular_translation_imports_and_tracks_source() {
        let bytes = fs::read("tests/fixtures/rectangular_fixed_translation_original.gsp").unwrap();
        let native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(
            native.objects[7].kind,
            Kind::FixedRectangularTranslatedPoint(
                5,
                37.79527559055118,
                2.,
                37.79527559055118,
                -0.5
            )
        );
        let expected = Point {
            x: 100. + 2. * 37.79527559055118,
            y: 210. + 0.5 * 37.79527559055118,
        };
        let position = native.position(8).unwrap();
        assert!((position.x - expected.x).abs() < 1e-9);
        assert!((position.y - expected.y).abs() < 1e-9);
        assert_eq!(native.encode().unwrap(), bytes);

        let mut created = Document::new();
        let source = created.add(Kind::Free(Point { x: 100., y: 210. }));
        let image = created.add(Kind::FixedRectangularTranslatedPoint(
            source,
            37.79527559055118,
            2.,
            37.79527559055118,
            -0.5,
        ));
        let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[1].kind, created.objects[1].kind);
        assert!(reopened.move_point(source, Point { x: 120., y: 200. }));
        let moved = reopened.position(image).unwrap();
        assert!((moved.x - expected.x - 20.).abs() < 1e-9);
        assert!((moved.y - expected.y + 10.).abs() < 1e-9);
    }
    #[test]
    fn mixed_rectangular_translation_imports_and_tracks_marked_axis() {
        for (path, expected_kind, expected_position) in [
            (
                "tests/fixtures/rectangular_mixed_horizontal_original.gsp",
                Kind::MixedRectangularTranslatedPoint(5, 7, 2., true),
                Point {
                    x: 100. + 2. * 37.79527559055118,
                    y: 160.,
                },
            ),
            (
                "tests/fixtures/rectangular_mixed_vertical_original.gsp",
                Kind::MixedRectangularTranslatedPoint(5, 6, 2., false),
                Point {
                    x: 200.,
                    y: 210. - 2. * 37.79527559055118,
                },
            ),
        ] {
            let bytes = fs::read(path).unwrap();
            let native = Document::from_bytes(&bytes, None).unwrap();
            assert!(!native.partial_view, "{path}");
            assert_eq!(native.objects[7].kind, expected_kind, "{path}");
            let position = native.position(8).unwrap();
            assert!((position.x - expected_position.x).abs() < 1e-9, "{path}");
            assert!((position.y - expected_position.y).abs() < 1e-9, "{path}");
            assert_eq!(native.encode().unwrap(), bytes, "{path}");
        }

        let mut created = Document::new();
        let h_start = created.add(Kind::Free(Point { x: 150., y: 120. }));
        let h_end = created.add(Kind::Free(Point { x: 250., y: 120. }));
        let v_start = created.add(Kind::Free(Point { x: 150., y: 180. }));
        let v_end = created.add(Kind::Free(Point { x: 150., y: 230. }));
        let source = created.add(Kind::Free(Point { x: 100., y: 210. }));
        let horizontal = created.add(Kind::PointDistance(h_start, h_end));
        let vertical = created.add(Kind::PointDistance(v_start, v_end));
        let fixed_x = created.add(Kind::MixedRectangularTranslatedPoint(
            source, vertical, 2., true,
        ));
        let fixed_y = created.add(Kind::MixedRectangularTranslatedPoint(
            source, horizontal, 2., false,
        ));
        let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[7].kind, created.objects[7].kind);
        assert_eq!(reopened.objects[8].kind, created.objects[8].kind);
        assert!(reopened.move_point(h_end, Point { x: 300., y: 120. }));
        assert!(reopened.move_point(v_end, Point { x: 150., y: 250. }));
        let x = reopened.position(fixed_x).unwrap();
        let y = reopened.position(fixed_y).unwrap();
        assert!((x.x - (100. + 2. * 37.79527559055118)).abs() < 1e-9);
        assert_eq!(x.y, 140.);
        assert_eq!(y.x, 250.);
        assert!((y.y - (210. - 2. * 37.79527559055118)).abs() < 1e-9);
    }
    #[test]
    fn polar_segment_translation_imports_and_tracks_endpoints() {
        let bytes = fs::read("tests/fixtures/polar_segment_translation_original.gsp").unwrap();
        let native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(
            native.objects[3].kind,
            Kind::PolarTranslatedSegment(3, 90., 37.79527559055118, 1.)
        );
        let (a, b, mode) = native.line_points(4).unwrap();
        assert_eq!(mode, LineMode::Segment);
        assert!((a.x - 100.).abs() < 1e-9 && (a.y - (200. - 37.79527559055118)).abs() < 1e-9);
        assert!((b.x - 200.).abs() < 1e-9 && (b.y - (200. - 37.79527559055118)).abs() < 1e-9);
        assert_eq!(native.encode().unwrap(), bytes);

        let mut created = Document::new();
        let start = created.add(Kind::Free(Point { x: 100., y: 200. }));
        let end = created.add(Kind::Free(Point { x: 200., y: 200. }));
        let segment = created.add(Kind::Segment(start, end));
        let translated = created.add(Kind::PolarTranslatedSegment(
            segment,
            90.,
            37.79527559055118,
            1.,
        ));
        let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[3].kind, created.objects[3].kind);
        assert!(reopened.move_point(end, Point { x: 230., y: 220. }));
        let (a, b, mode) = reopened.line_points(translated).unwrap();
        assert_eq!(mode, LineMode::Segment);
        assert!((a.x - 100.).abs() < 1e-9 && (a.y - (200. - 37.79527559055118)).abs() < 1e-9);
        assert!((b.x - 230.).abs() < 1e-9 && (b.y - (220. - 37.79527559055118)).abs() < 1e-9);
    }
    #[test]
    fn polar_circle_translation_imports_and_tracks_center_and_radius() {
        let bytes = fs::read("tests/fixtures/polar_circle_translation_original.gsp").unwrap();
        let native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(
            native.objects[3].kind,
            Kind::PolarTranslatedCircle(3, 90., 37.79527559055118, 1.)
        );
        let (center, radius) = native.circle_points(4).unwrap();
        assert!((center.x - 100.).abs() < 1e-9);
        assert!((center.y - (200. - 37.79527559055118)).abs() < 1e-9);
        assert_eq!(radius, 50.);
        assert_eq!(native.encode().unwrap(), bytes);

        let mut created = Document::new();
        let center = created.add(Kind::Free(Point { x: 100., y: 200. }));
        let radius_point = created.add(Kind::Free(Point { x: 150., y: 200. }));
        let circle = created.add(Kind::Circle(center, radius_point));
        let moved = created.add(Kind::PolarTranslatedCircle(
            circle,
            90.,
            37.79527559055118,
            1.,
        ));
        let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[3].kind, created.objects[3].kind);
        assert!(reopened.move_point(radius_point, Point { x: 160., y: 200. }));
        assert!(reopened.move_point(center, Point { x: 120., y: 210. }));
        let (new_center, new_radius) = reopened.circle_points(moved).unwrap();
        assert!((new_center.x - 120.).abs() < 1e-9);
        assert!((new_center.y - (210. - 37.79527559055118)).abs() < 1e-9);
        assert!((new_radius - 40_f64.hypot(10.)).abs() < 1e-9);
    }

    #[test]
    fn rectangular_segment_translation_imports_and_tracks_endpoints() {
        let bytes =
            fs::read("tests/fixtures/rectangular_segment_translation_original.gsp").unwrap();
        let native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(
            native.objects[3].kind,
            Kind::FixedRectangularTranslatedSegment(
                3,
                37.79527559055118,
                1.,
                37.79527559055118,
                1.
            )
        );
        let (a, b, mode) = native.line_points(4).unwrap();
        assert_eq!(mode, LineMode::Segment);
        assert!((a.x - (100. + 37.79527559055118)).abs() < 1e-9);
        assert!((a.y - (200. - 37.79527559055118)).abs() < 1e-9);
        assert!((b.x - (200. + 37.79527559055118)).abs() < 1e-9);
        assert!((b.y - (200. - 37.79527559055118)).abs() < 1e-9);
        assert_eq!(native.encode().unwrap(), bytes);

        let mut created = Document::new();
        let start = created.add(Kind::Free(Point { x: 100., y: 200. }));
        let end = created.add(Kind::Free(Point { x: 200., y: 200. }));
        let segment = created.add(Kind::Segment(start, end));
        let translated = created.add(Kind::FixedRectangularTranslatedSegment(
            segment,
            37.79527559055118,
            1.,
            37.79527559055118,
            1.,
        ));
        let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(reopened.move_point(end, Point { x: 230., y: 220. }));
        let (_, end, mode) = reopened.line_points(translated).unwrap();
        assert_eq!(mode, LineMode::Segment);
        assert!((end.x - (230. + 37.79527559055118)).abs() < 1e-9);
        assert!((end.y - (220. - 37.79527559055118)).abs() < 1e-9);
    }
    #[test]
    fn rectangular_circle_and_vector_shapes_import_and_track_sources() {
        for (path, expected_kind) in [
            (
                "tests/fixtures/rectangular_circle_translation_original.gsp",
                Kind::FixedRectangularTranslatedCircle(
                    3,
                    37.79527559055118,
                    1.,
                    37.79527559055118,
                    1.,
                ),
            ),
            (
                "tests/fixtures/vector_segment_translation_original.gsp",
                Kind::VectorTranslatedSegment(3, 1, 2),
            ),
            (
                "tests/fixtures/vector_circle_translation_original.gsp",
                Kind::VectorTranslatedCircle(6, 1, 2),
            ),
        ] {
            let bytes = fs::read(path).unwrap();
            let native = Document::from_bytes(&bytes, None).unwrap();
            assert!(!native.partial_view, "{path}");
            assert_eq!(native.objects.last().unwrap().kind, expected_kind, "{path}");
            assert_eq!(native.encode().unwrap(), bytes, "{path}");
        }

        let mut created = Document::new();
        let start = created.add(Kind::Free(Point { x: 150., y: 120. }));
        let end = created.add(Kind::Free(Point { x: 220., y: 130. }));
        let segment = created.add(Kind::Segment(start, end));
        let center = created.add(Kind::Free(Point { x: 100., y: 200. }));
        let radius_point = created.add(Kind::Free(Point { x: 150., y: 200. }));
        let circle = created.add(Kind::Circle(center, radius_point));
        let vector_segment = created.add(Kind::VectorTranslatedSegment(segment, start, end));
        let vector_circle = created.add(Kind::VectorTranslatedCircle(circle, start, end));
        let rectangular_circle = created.add(Kind::FixedRectangularTranslatedCircle(
            circle,
            37.79527559055118,
            1.,
            37.79527559055118,
            1.,
        ));
        let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(reopened.move_point(end, Point { x: 250., y: 140. }));
        assert!(reopened.move_point(radius_point, Point { x: 160., y: 200. }));
        let (segment_a, segment_b, mode) = reopened.line_points(vector_segment).unwrap();
        assert_eq!(mode, LineMode::Segment);
        assert_eq!(segment_a, Point { x: 250., y: 140. });
        assert_eq!(segment_b, Point { x: 350., y: 160. });
        let (vector_center, vector_radius) = reopened.circle_points(vector_circle).unwrap();
        assert_eq!(vector_center, Point { x: 200., y: 220. });
        assert_eq!(vector_radius, 60.);
        let (rect_center, rect_radius) = reopened.circle_points(rectangular_circle).unwrap();
        assert!((rect_center.x - (100. + 37.79527559055118)).abs() < 1e-9);
        assert!((rect_center.y - (200. - 37.79527559055118)).abs() < 1e-9);
        assert_eq!(rect_radius, 60.);
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
    fn marked_angle_rotation_imports_and_tracks_angle_points() {
        let bytes = fs::read("tests/fixtures/angle_rotation_original.gsp").unwrap();
        let mut native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(
            native.objects[5].kind,
            Kind::AngleRotatedPoint(5, 1, 2, 3, 4)
        );
        assert_eq!(native.position(6), Some(Point { x: 130., y: 190. }));
        assert_eq!(native.encode().unwrap(), bytes);
        assert!(native.move_point(4, Point { x: 290., y: 190. }));
        assert_ne!(native.position(6), Some(Point { x: 130., y: 190. }));

        let mut created = Document::new();
        let center = created.add(Kind::Free(Point { x: 130., y: 150. }));
        let first = created.add(Kind::Free(Point { x: 230., y: 160. }));
        let vertex = created.add(Kind::Free(Point { x: 230., y: 220. }));
        let last = created.add(Kind::Free(Point { x: 290., y: 220. }));
        let source = created.add(Kind::Free(Point { x: 170., y: 150. }));
        let image = created.add(Kind::AngleRotatedPoint(source, center, first, vertex, last));
        assert_eq!(created.position(image), Some(Point { x: 130., y: 190. }));
        let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[5].kind, created.objects[5].kind);
        assert!(reopened.move_point(last, Point { x: 290., y: 190. }));
        assert_ne!(reopened.position(image), created.position(image));
        assert!(
            !Document::from_bytes(&reopened.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
    #[test]
    fn marked_angle_rotation_of_shapes_imports_and_tracks_angle_points() {
        for (path, expected) in [
            (
                "tests/fixtures/angle_rotation_segment_original.gsp",
                Kind::AngleRotatedSegment(7, 1, 2, 3, 4),
            ),
            (
                "tests/fixtures/angle_rotation_circle_original.gsp",
                Kind::AngleRotatedCircle(7, 1, 2, 3, 4),
            ),
        ] {
            let bytes = fs::read(path).unwrap();
            let mut native = Document::from_bytes(&bytes, None).unwrap();
            assert!(!native.partial_view, "{path}");
            assert_eq!(native.objects[7].kind, expected);
            assert_eq!(native.encode().unwrap(), bytes);
            let before = if matches!(&expected, Kind::AngleRotatedSegment(..)) {
                let (a, b, _) = native.line_points(8).unwrap();
                assert!((a.x - 90.).abs() < 1e-8 && (a.y - 190.).abs() < 1e-8);
                assert!((b.x - 90.).abs() < 1e-8 && (b.y - 230.).abs() < 1e-8);
                a
            } else {
                let (center, radius) = native.circle_points(8).unwrap();
                assert_eq!(radius, 40.);
                assert!((center.x - 90.).abs() < 1e-8 && (center.y - 190.).abs() < 1e-8);
                center
            };
            assert!(native.move_point(4, Point { x: 290., y: 190. }));
            let after = if matches!(&expected, Kind::AngleRotatedSegment(..)) {
                native.line_points(8).unwrap().0
            } else {
                native.circle_points(8).unwrap().0
            };
            assert_ne!(after, before);
        }
    }
    #[test]
    fn fixed_rotation_of_shapes_imports_and_tracks_center() {
        for (path, expected) in [
            (
                "tests/fixtures/fixed_rotation_segment_original.gsp",
                Kind::RotatedSegment(4, 1, 90.),
            ),
            (
                "tests/fixtures/fixed_rotation_circle_original.gsp",
                Kind::RotatedCircle(4, 1, 90.),
            ),
        ] {
            let bytes = fs::read(path).unwrap();
            let mut native = Document::from_bytes(&bytes, None).unwrap();
            assert!(!native.partial_view, "{path}");
            assert_eq!(native.objects[4].kind, expected);
            assert_eq!(native.encode().unwrap(), bytes);
            let rotated_center = if matches!(&expected, Kind::RotatedSegment(..)) {
                let (a, b, _) = native.line_points(5).unwrap();
                assert!((a.x - 170.).abs() < 1e-8 && (a.y - 110.).abs() < 1e-8);
                assert!((b.x - 170.).abs() < 1e-8 && (b.y - 70.).abs() < 1e-8);
                a
            } else {
                let (center, radius) = native.circle_points(5).unwrap();
                assert_eq!(radius, 40.);
                assert!((center.x - 170.).abs() < 1e-8 && (center.y - 110.).abs() < 1e-8);
                center
            };
            assert!(native.move_point(1, Point { x: 140., y: 150. }));
            let after = if matches!(&expected, Kind::RotatedSegment(..)) {
                native.line_points(5).unwrap().0
            } else {
                native.circle_points(5).unwrap().0
            };
            assert_ne!(rotated_center, after);
        }
    }
    #[test]
    fn segment_dilations_and_reflection_import_and_track_dependencies() {
        for (path, expected) in [
            (
                "tests/fixtures/fixed_dilation_segment_original.gsp",
                Kind::ScaledSegment(4, 1, 0.5),
            ),
            (
                "tests/fixtures/ratio_dilation_segment_original.gsp",
                Kind::RatioScaledSegment(7, 1, 2, 3, 4),
            ),
            (
                "tests/fixtures/reflection_segment_original.gsp",
                Kind::ReflectedSegment(6, 3),
            ),
        ] {
            let bytes = fs::read(path).unwrap();
            let mut native = Document::from_bytes(&bytes, None).unwrap();
            assert!(!native.partial_view, "{path}");
            let image = native.objects.len();
            assert_eq!(native.objects[image - 1].kind, expected);
            assert_eq!(native.encode().unwrap(), bytes);
            let (a, b, mode) = native.line_points(image).unwrap();
            assert_eq!(mode, LineMode::Segment);
            if matches!(&expected, Kind::ReflectedSegment(..)) {
                assert!((a.x - 40.).abs() < 1e-8 && (b.x - 0.).abs() < 1e-8);
                assert!(native.move_point(2, Point { x: 120., y: 220. }));
            } else {
                assert_eq!(a, Point { x: 120., y: 120. });
                assert_eq!(b, Point { x: 130., y: 120. });
                let moved = if matches!(&expected, Kind::ScaledSegment(..)) {
                    native.move_point(1, Point { x: 110., y: 120. })
                } else {
                    native.move_point(4, Point { x: 255., y: 100. })
                };
                assert!(moved);
            }
            assert_ne!(native.line_points(image).unwrap().0, a);
        }
    }
    #[test]
    fn reflected_polygon_imports_and_tracks_mirror() {
        for (path, polygon, mirror, image) in [
            ("tests/fixtures/reflection_polygon_original.gsp", 7, 3, 8),
            ("tests/fixtures/reflection.gsp", 9, 4, 13),
        ] {
            let bytes = fs::read(path).unwrap();
            let native = Document::from_bytes(&bytes, None).unwrap();
            assert!(!native.partial_view, "{path}");
            assert_eq!(
                native.objects[image - 1].kind,
                Kind::ReflectedPolygon(polygon, mirror)
            );
            assert_eq!(native.encode().unwrap(), bytes);
            assert_eq!(
                native.objects[image - 1].visual_style,
                native.objects[polygon - 1].visual_style
            );
        }

        let mut native =
            Document::open(Path::new("tests/fixtures/reflection_polygon_original.gsp")).unwrap();
        let before = native.polygon_points(8).unwrap();
        assert_eq!(
            before,
            vec![
                Point { x: 100., y: 130. },
                Point { x: 60., y: 180. },
                Point { x: 100., y: 230. },
            ]
        );
        assert!(native.move_point(2, Point { x: 170., y: 230. }));
        assert_ne!(native.polygon_points(8).unwrap(), before);
        assert_eq!(
            native.encode().unwrap(),
            fs::read("tests/fixtures/reflection_polygon_mirror_dragged_by_wine.gsp").unwrap()
        );

        let mut created = Document::new();
        let mirror_a = created.add(Kind::Free(Point { x: 150., y: 100. }));
        let mirror_b = created.add(Kind::Free(Point { x: 150., y: 230. }));
        let mirror = created.add(Kind::Segment(mirror_a, mirror_b));
        let a = created.add(Kind::Free(Point { x: 200., y: 130. }));
        let b = created.add(Kind::Free(Point { x: 240., y: 180. }));
        let c = created.add(Kind::Free(Point { x: 200., y: 230. }));
        let polygon = created.add(Kind::Polygon(vec![a, b, c]));
        let image = created.add(Kind::ReflectedPolygon(polygon, mirror));
        assert_eq!(
            created.objects[image - 1].visual_style,
            created.objects[polygon - 1].visual_style
        );
        let encoded = created.encode().unwrap();
        let marker = records(&encoded)
            .unwrap()
            .into_iter()
            .find(|record| record.kind == 0x2328)
            .unwrap();
        assert_eq!(u32_at(&encoded, marker.data.start), image as u32);
        assert!(created.move_point(mirror_b, Point { x: 170., y: 230. }));
        let moved = created.encode().unwrap();
        let marker = records(&moved)
            .unwrap()
            .into_iter()
            .find(|record| record.kind == 0x2328)
            .unwrap();
        assert_eq!(u32_at(&moved, marker.data.start), mirror_b as u32);
        let reopened = Document::from_bytes(&moved, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.polygon_points(image),
            created.polygon_points(image)
        );
    }
    #[test]
    fn transformed_polygon_fixtures_import_and_track_sources() {
        for (path, image, expected) in [
            (
                "tests/fixtures/fixed_rotation_polygon_original.gsp",
                6,
                Kind::RotatedPolygon(5, 1, 90.),
            ),
            (
                "tests/fixtures/angle_rotation_polygon_original.gsp",
                9,
                Kind::AngleRotatedPolygon(8, 1, 2, 3, 4),
            ),
            (
                "tests/fixtures/fixed_dilation_polygon_original.gsp",
                6,
                Kind::ScaledPolygon(5, 1, 0.5),
            ),
            (
                "tests/fixtures/ratio_dilation_polygon_original.gsp",
                9,
                Kind::RatioScaledPolygon(8, 1, 2, 3, 4),
            ),
        ] {
            let bytes = fs::read(path).unwrap();
            let mut document = Document::from_bytes(&bytes, None).unwrap();
            assert!(!document.partial_view, "{path}");
            assert_eq!(document.objects[image - 1].kind, expected);
            assert_eq!(document.encode().unwrap(), bytes, "{path}");
            assert_eq!(
                document.objects[image - 1].visual_style,
                document.objects[image - 2].visual_style
            );
            let before = document.polygon_points(image).unwrap();
            assert!(
                document.move_point(if image == 6 { 2 } else { 4 }, Point { x: 175., y: 170. })
            );
            assert_ne!(document.polygon_points(image).unwrap(), before);
            assert!(
                !Document::from_bytes(&document.encode().unwrap(), None)
                    .unwrap()
                    .partial_view
            );
        }
    }
    #[test]
    fn vector_translated_polygon_imports_and_tracks_vector() {
        let bytes = fs::read("tests/fixtures/vector_translation_polygon_original.gsp").unwrap();
        let mut document = Document::from_bytes(&bytes, None).unwrap();
        assert!(!document.partial_view);
        assert_eq!(
            document.objects[7].kind,
            Kind::VectorTranslatedPolygon(7, 1, 2)
        );
        assert_eq!(document.encode().unwrap(), bytes);
        assert_eq!(
            document.polygon_points(8),
            Some(vec![
                Point { x: 170., y: 210. },
                Point { x: 210., y: 210. },
                Point { x: 190., y: 250. },
            ])
        );
        assert!(document.move_point(2, Point { x: 250., y: 130. }));
        assert_eq!(
            document.polygon_points(8).unwrap()[0],
            Point { x: 200., y: 210. }
        );
        assert!(
            !Document::from_bytes(&document.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
    #[test]
    fn fixed_translated_polygon_fixtures_import_and_track_source() {
        for (path, expected) in [
            (
                "tests/fixtures/polar_translation_polygon_original.gsp",
                Kind::PolarTranslatedPolygon(7, 90., 37.79527559055118, 1.),
            ),
            (
                "tests/fixtures/rectangular_translation_polygon_original.gsp",
                Kind::FixedRectangularTranslatedPolygon(
                    7,
                    37.79527559055118,
                    2.,
                    37.79527559055118,
                    -0.5,
                ),
            ),
        ] {
            let bytes = fs::read(path).unwrap();
            let mut document = Document::from_bytes(&bytes, None).unwrap();
            assert!(!document.partial_view, "{path}");
            assert_eq!(document.objects[7].kind, expected, "{path}");
            assert_eq!(document.encode().unwrap(), bytes, "{path}");
            let before = document.polygon_points(8).unwrap();
            assert!(document.move_point(5, Point { x: 160., y: 200. }));
            assert_ne!(document.polygon_points(8).unwrap(), before);
            assert!(
                !Document::from_bytes(&document.encode().unwrap(), None)
                    .unwrap()
                    .partial_view
            );
        }
    }
    #[test]
    fn marked_distance_polygon_imports_and_tracks_measurement() {
        let bytes =
            fs::read("tests/fixtures/marked_distance_polygon_translation_original.gsp").unwrap();
        let mut document = Document::from_bytes(&bytes, None).unwrap();
        assert!(!document.partial_view);
        assert_eq!(
            document.objects[7].kind,
            Kind::MarkedDistanceTranslatedPolygon(7, 6, 90.)
        );
        assert_eq!(document.encode().unwrap(), bytes);
        let before = document.polygon_points(8).unwrap();
        assert_eq!(before[0], Point { x: 100., y: 90. });
        assert!(document.move_point(2, Point { x: 300., y: 120. }));
        let moved = document.polygon_points(8).unwrap()[0];
        assert!((moved.x - 100.).abs() < 1e-9 && (moved.y - 40.).abs() < 1e-9);
        assert!(
            !Document::from_bytes(&document.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
    #[test]
    fn polygon_area_imports_and_tracks_vertices() {
        let bytes = fs::read("tests/fixtures/polygon_area_original.gsp").unwrap();
        let mut document = Document::from_bytes(&bytes, None).unwrap();
        assert!(!document.partial_view);
        assert_eq!(document.objects[5].kind, Kind::PolygonArea(5));
        assert_eq!(document.encode().unwrap(), bytes);
        assert!(
            (document.polygon_area_cm2(6).unwrap() - 800. / 37.79527559055118_f64.powi(2)).abs()
                < 1e-10
        );
        assert!(document.move_point(4, Point { x: 190., y: 250. }));
        assert!(
            (document.polygon_area_cm2(6).unwrap() - 1200. / 37.79527559055118_f64.powi(2)).abs()
                < 1e-10
        );
        assert!(
            !Document::from_bytes(&document.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
    #[test]
    fn transformed_polygon_measurements_import_and_track_source() {
        for (path, expected) in [
            (
                "tests/fixtures/transformed_polygon_perimeter_original.gsp",
                Kind::PolygonPerimeter(6),
            ),
            (
                "tests/fixtures/transformed_polygon_area_original.gsp",
                Kind::PolygonArea(6),
            ),
        ] {
            let bytes = fs::read(path).unwrap();
            let mut document = Document::from_bytes(&bytes, None).unwrap();
            assert!(!document.partial_view, "{path}");
            assert_eq!(document.objects[6].kind, expected, "{path}");
            assert_eq!(document.encode().unwrap(), bytes, "{path}");
            let before = if matches!(expected, Kind::PolygonArea(_)) {
                document.polygon_area_cm2(7).unwrap()
            } else {
                document.perimeter_cm(7).unwrap()
            };
            assert!(document.move_point(4, Point { x: 190., y: 250. }));
            let after = if matches!(expected, Kind::PolygonArea(_)) {
                document.polygon_area_cm2(7).unwrap()
            } else {
                document.perimeter_cm(7).unwrap()
            };
            assert_ne!(before, after);
            assert!(
                !Document::from_bytes(&document.encode().unwrap(), None)
                    .unwrap()
                    .partial_view
            );
        }
    }
    #[test]
    fn rectangular_marked_polygon_imports_and_tracks_both_distances() {
        let bytes = fs::read("tests/fixtures/rectangular_marked_polygon_original.gsp").unwrap();
        let mut document = Document::from_bytes(&bytes, None).unwrap();
        assert!(!document.partial_view);
        assert_eq!(
            document.objects[11].kind,
            Kind::RectangularMarkedTranslatedPolygon(11, 6, 7)
        );
        assert_eq!(document.encode().unwrap(), bytes);
        assert_eq!(
            document.polygon_points(12).unwrap()[0],
            Point { x: 200., y: 140. }
        );
        assert!(document.move_point(2, Point { x: 300., y: 120. }));
        assert!(document.move_point(4, Point { x: 150., y: 250. }));
        assert_eq!(
            document.polygon_points(12).unwrap()[0],
            Point { x: 250., y: 120. }
        );
        assert!(
            !Document::from_bytes(&document.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
    #[test]
    fn mixed_rectangular_polygon_fixtures_import_and_track_distance() {
        for (path, expected) in [
            (
                "tests/fixtures/rectangular_mixed_polygon_horizontal_original.gsp",
                Kind::MixedRectangularTranslatedPolygon(11, 7, 2., true),
            ),
            (
                "tests/fixtures/rectangular_mixed_polygon_vertical_original.gsp",
                Kind::MixedRectangularTranslatedPolygon(11, 6, 2., false),
            ),
        ] {
            let bytes = fs::read(path).unwrap();
            let mut document = Document::from_bytes(&bytes, None).unwrap();
            assert!(!document.partial_view, "{path}");
            assert_eq!(document.objects[11].kind, expected, "{path}");
            assert_eq!(document.encode().unwrap(), bytes, "{path}");
            let before = document.polygon_points(12).unwrap();
            let id = if matches!(
                expected,
                Kind::MixedRectangularTranslatedPolygon(_, _, _, true)
            ) {
                4
            } else {
                2
            };
            assert!(document.move_point(id, Point { x: 300., y: 250. }));
            assert_ne!(document.polygon_points(12).unwrap(), before);
            assert!(
                !Document::from_bytes(&document.encode().unwrap(), None)
                    .unwrap()
                    .partial_view
            );
        }
    }
    #[test]
    fn marked_ratio_dilation_imports_and_tracks_ratio_points() {
        let bytes = fs::read("tests/fixtures/ratio_dilation_original.gsp").unwrap();
        let native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(
            native.objects[5].kind,
            Kind::RatioScaledPoint(5, 1, 2, 3, 4)
        );
        assert_eq!(native.position(6), Some(Point { x: 120., y: 120. }));
        assert_eq!(native.encode().unwrap(), bytes);

        let mut created = Document::new();
        let center = created.add(Kind::Free(Point { x: 100., y: 120. }));
        let origin = created.add(Kind::Free(Point { x: 180., y: 100. }));
        let base = created.add(Kind::Free(Point { x: 230., y: 100. }));
        let target = created.add(Kind::Free(Point { x: 205., y: 100. }));
        let source = created.add(Kind::Free(Point { x: 140., y: 120. }));
        let image = created.add(Kind::RatioScaledPoint(source, center, origin, base, target));
        assert_eq!(created.position(image), Some(Point { x: 120., y: 120. }));
        let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[5].kind, created.objects[5].kind);
        assert!(reopened.move_point(target, Point { x: 255., y: 100. }));
        assert_eq!(reopened.position(image), Some(Point { x: 160., y: 120. }));
        assert!(reopened.move_point(target, Point { x: 155., y: 100. }));
        assert_eq!(reopened.position(image), Some(Point { x: 80., y: 120. }));
        assert!(
            !Document::from_bytes(&reopened.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
    #[test]
    fn marked_ratio_circle_dilation_imports_and_tracks_ratio_points() {
        let bytes = fs::read("tests/fixtures/ratio_dilation_circle_original.gsp").unwrap();
        let native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(
            native.objects[7].kind,
            Kind::RatioScaledCircle(7, 1, 2, 3, 4)
        );
        assert_eq!(
            native.circle_points(8),
            Some((Point { x: 120., y: 120. }, 10.))
        );
        assert_eq!(native.encode().unwrap(), bytes);

        let mut created = Document::new();
        let center = created.add(Kind::Free(Point { x: 100., y: 120. }));
        let origin = created.add(Kind::Free(Point { x: 180., y: 100. }));
        let base = created.add(Kind::Free(Point { x: 230., y: 100. }));
        let target = created.add(Kind::Free(Point { x: 205., y: 100. }));
        let source_center = created.add(Kind::Free(Point { x: 140., y: 120. }));
        let radius_point = created.add(Kind::Free(Point { x: 160., y: 120. }));
        let source = created.add(Kind::Circle(source_center, radius_point));
        let image = created.add(Kind::RatioScaledCircle(
            source, center, origin, base, target,
        ));
        assert_eq!(
            created.circle_points(image),
            Some((Point { x: 120., y: 120. }, 10.))
        );
        let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[7].kind, created.objects[7].kind);
        assert!(reopened.move_point(target, Point { x: 255., y: 100. }));
        assert_eq!(
            reopened.circle_points(image),
            Some((Point { x: 160., y: 120. }, 30.))
        );
        assert!(reopened.move_point(target, Point { x: 155., y: 100. }));
        assert_eq!(
            reopened.circle_points(image),
            Some((Point { x: 80., y: 120. }, 10.))
        );
        assert!(
            !Document::from_bytes(&reopened.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
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
    fn marking_an_older_coordinate_system_updates_the_page_link_and_new_plot_reference() {
        let source = include_bytes!("../tests/fixtures/graph_two_coordinate_systems_original.gsp");
        let native_marked =
            include_bytes!("../tests/fixtures/graph_mark_coordinate_system_original.gsp");
        let native = Document::from_bytes(native_marked, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.active_coordinate_system(), Some(9));
        assert_eq!(native.encode().unwrap(), native_marked);
        let mut doc = Document::from_bytes(source, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.active_coordinate_system(), Some(14));
        assert_eq!(doc.coordinate_system_for_object(4), Some(9));
        assert_eq!(doc.coordinate_system_for_object(1), Some(14));
        assert_eq!(doc.encode().unwrap(), source);

        assert_eq!(doc.mark_coordinate_system(4).unwrap(), 9);
        assert_eq!(doc.active_coordinate_system(), Some(9));
        doc.undo();
        assert_eq!(doc.active_coordinate_system(), Some(14));
        doc.redo();
        assert_eq!(doc.active_coordinate_system(), Some(9));
        let marked = doc.encode().unwrap();
        let mut expected = source.to_vec();
        let link = records(source)
            .unwrap()
            .into_iter()
            .find(|record| record.kind == 0x232d)
            .unwrap();
        expected[link.data].copy_from_slice(&9_u32.to_le_bytes());
        assert_eq!(marked, expected);
        let reopened = Document::from_bytes(&marked, None).unwrap();
        assert_eq!(reopened.active_coordinate_system(), Some(9));

        let point = doc
            .plot_points_at(PlotPointsMode::Cartesian, [1.0, 2.0])
            .unwrap();
        assert!(matches!(
            doc.objects[point - 1].kind,
            Kind::PlottedPoint(_, 9, _, _)
        ));
        let plotted = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(plotted.active_coordinate_system(), Some(9));
        assert!(matches!(
            plotted.objects[point - 1].kind,
            Kind::PlottedPoint(_, 9, _, _)
        ));
    }

    #[test]
    fn selected_point_can_define_a_second_coordinate_system() {
        let source = include_bytes!("../tests/fixtures/graph_defined_coordinate_system.gsp");
        let mut doc = Document::from_bytes(source, None).unwrap();
        let frame = doc.add_coordinate_system_at_point(1).unwrap();
        assert_eq!(frame, 14);
        assert_eq!(doc.active_coordinate_system(), Some(14));
        assert_eq!(doc.objects.len(), 14);
        let encoded = doc.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.active_coordinate_system(), Some(14));
        assert_eq!(
            reopened.objects[9].kind,
            Kind::GraphHorizontalUnit(1, 37.79527559055118)
        );
        assert_eq!(reopened.objects[13].kind, Kind::CoordinateSystem(11, 13));
        assert_eq!(reopened.graph_frame(14).unwrap().origin.x, 150.0);
    }

    #[test]
    fn circle_defined_coordinate_system_tracks_its_radius_and_round_trips() {
        let native = include_bytes!("../tests/fixtures/graph_unit_circle_original.gsp");
        let mut original = Document::from_bytes(native, None).unwrap();
        assert!(!original.partial_view);
        assert_eq!(original.encode().unwrap(), native);
        assert_eq!(original.active_coordinate_system(), Some(6));
        let radius = original.circle_points(3).unwrap().1;
        let frame = original.graph_frame(6).unwrap();
        assert!((frame.x_unit.x - frame.origin.x - radius).abs() < 1e-12);
        assert!((frame.origin.y - frame.y_unit.y - radius).abs() < 1e-12);
        assert!(original.move_point(2, Point { x: 383., y: 217. }));
        let moved = original.graph_frame(6).unwrap();
        assert!((moved.x_unit.x - moved.origin.x - 60.).abs() < 1e-12);

        let mut generated =
            Document::from_bytes(include_bytes!("../tests/fixtures/circle.gsp"), None).unwrap();
        assert_eq!(generated.add_coordinate_system_from_circle(3).unwrap(), 6);
        let bytes = generated.encode().unwrap();
        let native_styles: Vec<_> = records(native)
            .unwrap()
            .into_iter()
            .filter(|record| record.kind == 0x04b3)
            .collect();
        let generated_styles: Vec<_> = records(&bytes)
            .unwrap()
            .into_iter()
            .filter(|record| record.kind == 0x04b3)
            .collect();
        assert_eq!(generated_styles.len(), 2);
        assert_eq!(
            &bytes[generated_styles[1].data.clone()],
            &native[native_styles[1].data.clone()]
        );
        let mut reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[3].kind, Kind::CircleAxis(3, 1));
        assert_eq!(reopened.objects[4].kind, Kind::CircleAxis(3, 0));
        assert_eq!(reopened.objects[5].kind, Kind::CoordinateSystem(4, 5));
        assert_eq!(reopened.active_coordinate_system(), Some(6));
        assert_eq!(reopened.graph_frame(6), generated.graph_frame(6));
        assert!(reopened.set_grid_form(GridForm::Rectangular).is_err());
        reopened.set_grid_form(GridForm::Polar).unwrap();
        let polar = Document::from_bytes(&reopened.encode().unwrap(), None).unwrap();
        assert_eq!(polar.grid_form[0], GridForm::Polar);
    }

    #[test]
    fn distance_defined_frames_decode_and_follow_their_sources() {
        for bytes in [
            include_bytes!("../tests/fixtures/graph_unit_segment_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/graph_unit_segment_point_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/graph_two_segments_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/graph_two_segments_point_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/graph_unit_distance_value_original.gsp").as_slice(),
            include_bytes!("../tests/fixtures/graph_unit_segment_dragged_by_wine.gsp").as_slice(),
        ] {
            let doc = Document::from_bytes(bytes, None).unwrap();
            assert!(!doc.partial_view);
            assert_eq!(doc.encode().unwrap(), bytes);
            assert!(
                doc.graph_frame(doc.active_coordinate_system().unwrap())
                    .is_some()
            );
        }
        let mut square = Document::from_bytes(
            include_bytes!("../tests/fixtures/graph_unit_segment_original.gsp"),
            None,
        )
        .unwrap();
        let original = square.graph_frame(8).unwrap();
        assert!((original.x_unit.x - original.origin.x - 188.73261509341728).abs() < 1e-6);
        assert!((original.origin.y - original.y_unit.y - 188.73261509341728).abs() < 1e-6);
        square.move_point(2, Point { x: 168., y: 204. });
        let moved = square.graph_frame(8).unwrap();
        assert!((moved.x_unit.x - moved.origin.x - 100.).abs() < 1e-9);
        assert!((moved.origin.y - moved.y_unit.y - 100.).abs() < 1e-9);
        let saved_square = Document::from_bytes(&square.encode().unwrap(), None).unwrap();
        assert_eq!(saved_square.graph_frame(8), Some(moved));
        square.set_grid_visible(false);
        assert!(!square.is_displayed(4));
        assert!(square.is_displayed(3));
        let native_dragged = Document::from_bytes(
            include_bytes!("../tests/fixtures/graph_unit_segment_dragged_by_wine.gsp"),
            None,
        )
        .unwrap();
        let dragged_frame = native_dragged.graph_frame(8).unwrap();
        assert!(
            (dragged_frame.x_unit.x - dragged_frame.origin.x - 192f64.hypot(103.)).abs() < 1e-9
        );
        assert!(
            (dragged_frame.origin.y - dragged_frame.y_unit.y - 192f64.hypot(103.)).abs() < 1e-9
        );

        let mut rectangular = Document::from_bytes(
            include_bytes!("../tests/fixtures/graph_two_segments_original.gsp"),
            None,
        )
        .unwrap();
        let frame = rectangular.graph_frame(10).unwrap();
        assert_eq!(frame.x_unit.x - frame.origin.x, 100.);
        assert_eq!(frame.origin.y - frame.y_unit.y, 150.);
        rectangular.move_point(5, Point { x: 350., y: 200. });
        let changed = rectangular.graph_frame(10).unwrap();
        assert_eq!(changed.x_unit.x - changed.origin.x, 100.);
        assert_eq!(changed.origin.y - changed.y_unit.y, 200.);
    }

    #[test]
    fn generated_distance_defined_frames_reopen_with_their_grid_style() {
        let sources: [(&[u8], Option<usize>, &[usize], usize, GridForm); 6] = [
            (
                include_bytes!("../tests/fixtures/segment.gsp"),
                None,
                &[3],
                8,
                GridForm::Square,
            ),
            (
                include_bytes!("../tests/fixtures/segment.gsp"),
                Some(2),
                &[3],
                7,
                GridForm::Square,
            ),
            (
                include_bytes!("../tests/fixtures/graph_two_segments_base.gsp"),
                None,
                &[3, 6],
                10,
                GridForm::Rectangular,
            ),
            (
                include_bytes!("../tests/fixtures/graph_two_segments_base.gsp"),
                Some(1),
                &[3, 6],
                9,
                GridForm::Rectangular,
            ),
            (
                include_bytes!("../tests/fixtures/wine_point_distance.gsp"),
                None,
                &[6],
                11,
                GridForm::Square,
            ),
            (
                include_bytes!("../tests/fixtures/wine_segment_length.gsp"),
                None,
                &[4],
                9,
                GridForm::Square,
            ),
        ];
        for (source, origin, distances, expected_id, form) in sources {
            let mut doc = Document::from_bytes(source, None).unwrap();
            assert_eq!(
                doc.add_coordinate_system_from_distances(
                    origin,
                    distances,
                    Point { x: 199., y: 125. },
                )
                .unwrap(),
                expected_id
            );
            let bytes = doc.encode().unwrap();
            let reopened = Document::from_bytes(&bytes, None).unwrap();
            assert!(!reopened.partial_view);
            assert_eq!(reopened.active_coordinate_system(), Some(expected_id));
            assert_eq!(reopened.grid_form[0], form);
            assert_eq!(
                reopened.graph_frame(expected_id),
                doc.graph_frame(expected_id)
            );
            assert!(records(&bytes).unwrap().iter().any(|record| {
                record.kind == 0x04b3
                    && bytes[record.data.start + 2..record.data.start + 4] == [14, 0]
            }));
        }
        let polar = Document::from_bytes(
            include_bytes!("../tests/fixtures/graph_unit_segment_polar_generated.gsp"),
            None,
        )
        .unwrap();
        assert_eq!(polar.grid_form[0], GridForm::Polar);
    }

    #[test]
    fn one_distance_rectangular_unit_edits_and_grid_form_conversions_round_trip() {
        for bytes in [
            include_bytes!("../tests/fixtures/graph_unit_segment_rectangular_original.gsp")
                .as_slice(),
            include_bytes!(
                "../tests/fixtures/graph_unit_segment_rectangular_to_square_original.gsp"
            )
            .as_slice(),
            include_bytes!(
                "../tests/fixtures/graph_unit_segment_rectangular_to_polar_original.gsp"
            )
            .as_slice(),
        ] {
            let doc = Document::from_bytes(bytes, None).unwrap();
            assert!(!doc.partial_view);
            assert_eq!(doc.encode().unwrap(), bytes);
        }
        let mut generated = Document::from_bytes(
            include_bytes!("../tests/fixtures/graph_unit_segment_generated.gsp"),
            None,
        )
        .unwrap();
        generated.set_grid_form(GridForm::Rectangular).unwrap();
        assert!(generated.is_displayed(6));
        let rectangular = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert_eq!(rectangular.grid_form[0], GridForm::Rectangular);
        assert_eq!(
            rectangular.objects[5].kind,
            Kind::DistanceVerticalUnit(5, 168f64.hypot(86.), true)
        );
        assert_eq!(rectangular.graph_frame(8), generated.graph_frame(8));

        let mut edited = rectangular;
        edited.move_point(2, Point { x: 168., y: 204. });
        let frame = edited.graph_frame(8).unwrap();
        assert_eq!(frame.x_unit.x - frame.origin.x, 100.);
        assert!((frame.origin.y - frame.y_unit.y - 168f64.hypot(86.)).abs() < 1e-9);
        assert!(edited.move_point(6, Point { x: 199., y: 45. }));
        let moved = edited.graph_frame(8).unwrap();
        assert_eq!(moved.origin.y - moved.y_unit.y, 80.);
        let saved = Document::from_bytes(&edited.encode().unwrap(), None).unwrap();
        assert_eq!(saved.graph_frame(8), Some(moved));

        let mut polar_edited = saved;
        polar_edited.set_grid_form(GridForm::Polar).unwrap();
        let polar = Document::from_bytes(&polar_edited.encode().unwrap(), None).unwrap();
        assert_eq!(polar.grid_form[0], GridForm::Polar);
        assert_eq!(
            polar.objects[5].kind,
            Kind::DistanceVerticalUnit(5, 80., true)
        );
        assert_eq!(
            polar.graph_frame(8).unwrap().origin.y - polar.graph_frame(8).unwrap().y_unit.y,
            80.
        );

        edited.set_grid_form(GridForm::Square).unwrap();
        let mut square = Document::from_bytes(&edited.encode().unwrap(), None).unwrap();
        assert_eq!(square.grid_form[0], GridForm::Square);
        assert_eq!(
            square.objects[5].kind,
            Kind::DistanceVerticalUnit(5, 80., false)
        );
        assert_eq!(
            square.graph_frame(8).unwrap().origin.y - square.graph_frame(8).unwrap().y_unit.y,
            100.
        );
        square.set_grid_form(GridForm::Polar).unwrap();
        let dependent_polar = Document::from_bytes(&square.encode().unwrap(), None).unwrap();
        assert_eq!(dependent_polar.grid_form[0], GridForm::Polar);
        assert_eq!(
            dependent_polar.objects[5].kind,
            Kind::DistanceVerticalUnit(5, 80., false)
        );
        assert_eq!(
            dependent_polar.graph_frame(8).unwrap().origin.y
                - dependent_polar.graph_frame(8).unwrap().y_unit.y,
            100.
        );
    }

    #[test]
    fn two_distance_frame_can_switch_between_rectangular_and_polar() {
        let native = include_bytes!("../tests/fixtures/graph_two_segments_polar_original.gsp");
        let parsed = Document::from_bytes(native, None).unwrap();
        assert!(!parsed.partial_view);
        assert_eq!(parsed.grid_form[0], GridForm::Polar);
        assert_eq!(parsed.encode().unwrap(), native);

        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/graph_two_segments_generated.gsp"),
            None,
        )
        .unwrap();
        assert!(doc.set_grid_form(GridForm::Square).is_err());
        doc.set_grid_form(GridForm::Polar).unwrap();
        let saved = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(saved.grid_form[0], GridForm::Polar);
        assert_eq!(saved.graph_frame(10), doc.graph_frame(10));
        doc.set_grid_form(GridForm::Rectangular).unwrap();
        assert_eq!(doc.grid_form[0], GridForm::Rectangular);
    }

    #[test]
    fn marking_another_frame_preserves_each_frames_grid_form() {
        let source = include_bytes!("../tests/fixtures/graph_two_coordinate_systems_original.gsp");
        let mut doc = Document::from_bytes(source, None).unwrap();
        doc.set_grid_form(GridForm::Rectangular).unwrap();
        assert_eq!(
            doc.frame_styles.get(&14),
            Some(&(GridForm::Rectangular, false))
        );
        doc.mark_coordinate_system(4).unwrap();
        assert_eq!(doc.grid_form[0], GridForm::Square);
        assert_eq!(doc.frame_styles.get(&9), Some(&(GridForm::Square, false)));
        assert!(!doc.is_displayed(7));
        assert!(doc.is_displayed(12));
        let encoded = doc.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.active_coordinate_system(), Some(9));
        assert_eq!(
            reopened.frame_styles.get(&9),
            Some(&(GridForm::Square, false))
        );
        assert_eq!(
            reopened.frame_styles.get(&14),
            Some(&(GridForm::Rectangular, false))
        );
        assert!(!reopened.is_displayed(7));
        assert!(reopened.is_displayed(12));
        assert_eq!(reopened.objects[11].kind, doc.objects[11].kind);
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
        assert!(snapped.grid_visible());
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
            (
                include_bytes!("../tests/fixtures/function_plot_x_plus_1.gsp").as_slice(),
                SimpleFunction::parse("x+1").unwrap(),
            ),
            (
                include_bytes!("../tests/fixtures/function_plot_two_times_x_plus_1.gsp").as_slice(),
                SimpleFunction::parse("2*(x+1)").unwrap(),
            ),
            (
                include_bytes!("../tests/fixtures/function_plot_x_plus_half.gsp").as_slice(),
                SimpleFunction::parse("x+0.5").unwrap(),
            ),
            (
                include_bytes!("../tests/fixtures/function_plot_sin_original.gsp").as_slice(),
                SimpleFunction::parse("sin(x)").unwrap(),
            ),
            (
                include_bytes!("../tests/fixtures/function_plot_cos_original.gsp").as_slice(),
                SimpleFunction::parse("cos(x)").unwrap(),
            ),
            (
                include_bytes!("../tests/fixtures/function_plot_tan_original.gsp").as_slice(),
                SimpleFunction::parse("tan(x)").unwrap(),
            ),
        ] {
            let mut doc = Document::from_bytes(source, None).unwrap();
            assert!(!doc.partial_view);
            assert_eq!(doc.objects[6].kind, Kind::FunctionDefinition(expression));
            assert_eq!(doc.angle_unit_radians, expression.uses_trigonometry());
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
    fn native_derivatives_roundtrip_and_source_edit_refreshes_cached_formula() {
        for (source, expected_source, derivative) in [
            (
                include_bytes!("../tests/fixtures/function_derivative_identity_original.gsp")
                    .as_slice(),
                1,
                SimpleFunction::parse("1").unwrap(),
            ),
            (
                include_bytes!("../tests/fixtures/function_derivative_square_original.gsp")
                    .as_slice(),
                2,
                SimpleFunction::parse("2*x").unwrap(),
            ),
            (
                include_bytes!("../tests/fixtures/function_derivative_cubic_original.gsp")
                    .as_slice(),
                1,
                SimpleFunction::parse("3*x^2").unwrap(),
            ),
            (
                include_bytes!("../tests/fixtures/function_derivative_sine_original.gsp")
                    .as_slice(),
                7,
                SimpleFunction::parse("cos(x)").unwrap(),
            ),
            (
                include_bytes!("../tests/fixtures/function_derivative_sine_degrees_original.gsp")
                    .as_slice(),
                7,
                SimpleFunction::parse("cos(x)").unwrap(),
            ),
        ] {
            let doc = Document::from_bytes(source, None).unwrap();
            assert!(!doc.partial_view);
            assert_eq!(
                doc.objects.last().unwrap().kind,
                Kind::DerivativeFunction(expected_source, derivative)
            );
            assert_eq!(doc.encode().unwrap(), source);
        }

        let native = include_bytes!("../tests/fixtures/function_derivative_square_original.gsp");
        let mut doc = Document::from_bytes(native, None).unwrap();
        doc.set_function_expression(2, SimpleFunction::parse("x^3").unwrap())
            .unwrap();
        assert_eq!(
            doc.objects[2].kind,
            Kind::DerivativeFunction(2, SimpleFunction::parse("3*x^2").unwrap())
        );
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[2].kind, doc.objects[2].kind);
        doc.undo();
        assert_eq!(doc.encode().unwrap(), native);
    }

    #[test]
    fn generated_derivative_can_be_plotted_and_reopened() {
        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/parametric_two_functions_moved_original.gsp"),
            None,
        )
        .unwrap();
        let derivative = doc.add_derivative_function(2).unwrap();
        assert_eq!(derivative, 3);
        assert_eq!(
            doc.objects[2].kind,
            Kind::DerivativeFunction(2, SimpleFunction::parse("2*x").unwrap())
        );
        let plot = doc
            .plot_function(derivative, Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(plot, 10);
        let before = doc.function_plot_points(plot, 5).unwrap();
        doc.set_function_expression(2, SimpleFunction::parse("x^3").unwrap())
            .unwrap();
        let after = doc.function_plot_points(plot, 5).unwrap();
        assert_ne!(after, before);
        assert_eq!(
            doc.objects[2].kind,
            Kind::DerivativeFunction(2, SimpleFunction::parse("3*x^2").unwrap())
        );
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[2].kind, doc.objects[2].kind);
        assert_eq!(reopened.objects[plot - 1].kind, doc.objects[plot - 1].kind);
        assert_eq!(reopened.function_plot_points(plot, 5).unwrap(), after);

        let mut parametric = Document::from_bytes(
            include_bytes!("../tests/fixtures/function_derivative_square_original.gsp"),
            None,
        )
        .unwrap();
        let curve = parametric
            .plot_parametric_curve(
                1,
                3,
                PlotPointsMode::Cartesian,
                -2.,
                2.,
                Point { x: 199., y: 125. },
            )
            .unwrap();
        assert_eq!(curve, 10);
        let reopened = Document::from_bytes(&parametric.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.objects[curve - 1].kind,
            parametric.objects[curve - 1].kind
        );

        let mut degrees = Document::from_bytes(
            include_bytes!("../tests/fixtures/function_plot_sin_degrees_original.gsp"),
            None,
        )
        .unwrap();
        let derivative = degrees
            .add_derivative_function_with_angle_unit(7, false)
            .unwrap();
        assert!(!degrees.angle_unit_radians);
        let reopened = Document::from_bytes(&degrees.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(!reopened.angle_unit_radians);
        assert_eq!(
            reopened.objects[derivative - 1].kind,
            degrees.objects[derivative - 1].kind
        );
    }

    #[test]
    fn adding_trigonometric_function_sets_radians_and_undo_restores_units() {
        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/function_base_frame.gsp"),
            None,
        )
        .unwrap();
        assert!(!doc.angle_unit_radians);
        let sine = SimpleFunction::parse("sin(x)").unwrap();
        doc.plot_new_function(sine, Point { x: 199., y: 125. })
            .unwrap();
        assert!(doc.angle_unit_radians);
        assert!(
            (doc.function_plot_points(8, 3).unwrap()[1].y - doc.graph_frame(6).unwrap().origin.y)
                .abs()
                < 1e-9
        );
        let saved = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!saved.partial_view);
        assert!(saved.angle_unit_radians);
        assert_eq!(saved.objects[6].kind, Kind::FunctionDefinition(sine));
        doc.undo();
        assert!(!doc.angle_unit_radians);
        doc.redo();
        assert!(doc.angle_unit_radians);
    }

    #[test]
    fn degree_based_trigonometric_plot_imports_and_reopens() {
        let native = include_bytes!("../tests/fixtures/function_plot_sin_degrees_original.gsp");
        let imported = Document::from_bytes(native, None).unwrap();
        assert!(!imported.partial_view);
        assert!(!imported.angle_unit_radians);
        assert_eq!(imported.encode().unwrap(), native);
        let sine = SimpleFunction::parse("sin(x)").unwrap();
        assert_eq!(imported.objects[6].kind, Kind::FunctionDefinition(sine));

        let mut generated = Document::from_bytes(
            include_bytes!("../tests/fixtures/function_base_frame.gsp"),
            None,
        )
        .unwrap();
        generated
            .plot_new_function_with_angle_unit(sine, Point { x: 199., y: 125. }, false)
            .unwrap();
        assert!(!generated.angle_unit_radians);
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(!reopened.angle_unit_radians);
        assert_eq!(reopened.objects[6].kind, imported.objects[6].kind);
        assert_eq!(
            reopened.function_plot_points(8, 3),
            imported.function_plot_points(8, 3)
        );
    }

    #[test]
    fn discontinuous_function_plots_leave_asymptote_gaps() {
        for (source, poles) in [
            (
                include_bytes!("../tests/fixtures/function_plot_reciprocal_original.gsp")
                    .as_slice(),
                vec![0.0],
            ),
            (
                include_bytes!("../tests/fixtures/function_plot_tan_original.gsp").as_slice(),
                vec![
                    -3.0 * std::f64::consts::FRAC_PI_2,
                    -std::f64::consts::FRAC_PI_2,
                    std::f64::consts::FRAC_PI_2,
                    3.0 * std::f64::consts::FRAC_PI_2,
                ],
            ),
        ] {
            let doc = Document::from_bytes(source, None).unwrap();
            assert!(!doc.partial_view);
            let frame = doc.graph_frame(6).unwrap();
            let segments = doc.function_plot_segments(8, 384).unwrap();
            assert!(segments.len() < 383);
            for (a, b) in segments {
                let x0 = frame.coordinates(GraphPoint { x: a.x, y: a.y }).unwrap().x;
                let x1 = frame.coordinates(GraphPoint { x: b.x, y: b.y }).unwrap().x;
                for pole in &poles {
                    assert!(!(x0 < *pole && x1 > *pole));
                }
            }
        }
        let smooth = Document::from_bytes(
            include_bytes!("../tests/fixtures/function_plot_square.gsp"),
            None,
        )
        .unwrap();
        assert_eq!(smooth.function_plot_segments(8, 384).unwrap().len(), 383);
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
    fn plot_existing_function_reuses_definition_and_reopens() {
        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/function_definition_identity.gsp"),
            None,
        )
        .unwrap();
        assert!(doc.plot_function(6, Point { x: 199., y: 125. }).is_err());
        let plot = doc.plot_function(7, Point { x: 199., y: 125. }).unwrap();
        assert_eq!(plot, 8);
        assert_eq!(doc.objects.len(), 8);
        assert!(matches!(
            doc.objects[7].kind,
            Kind::FunctionPlot(7, 6, _, _, 750)
        ));
        let encoded = doc.encode().unwrap();
        assert_eq!(
            encoded,
            include_bytes!("../tests/fixtures/function_plot_existing_generated.gsp")
        );
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.objects[6].kind,
            Kind::FunctionDefinition(SimpleFunction::Identity)
        );
        assert_eq!(reopened.objects[7].kind, doc.objects[7].kind);
        doc.undo();
        assert_eq!(doc.objects.len(), 7);
        doc.redo();
        assert_eq!(doc.objects.len(), 8);

        let mut fresh = Document::new();
        let definition = fresh
            .add_function_definition(SimpleFunction::Square)
            .unwrap();
        assert_eq!(definition, 1);
        let plot = fresh
            .plot_function(definition, Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(plot, 8);
        assert!(matches!(
            fresh.objects[7].kind,
            Kind::FunctionPlot(1, 7, _, _, 750)
        ));
        let reopened = Document::from_bytes(&fresh.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[7].kind, fresh.objects[7].kind);
    }

    #[test]
    fn editing_saved_function_updates_plot_and_survives_reopen() {
        let mut doc = Document::from_bytes(
            include_bytes!("../tests/fixtures/function_plot_square.gsp"),
            None,
        )
        .unwrap();
        let before = doc.function_plot_points(8, 3).unwrap()[1];
        let expression = SimpleFunction::parse("x+1").unwrap();
        assert!(doc.set_function_expression(8, expression).is_err());
        doc.set_function_expression(7, expression).unwrap();
        let after = doc.function_plot_points(8, 3).unwrap()[1];
        assert!((after.x - before.x).abs() < 1e-9);
        assert!((after.y - before.y + 37.79527559055118).abs() < 1e-9);
        let saved = doc.encode().unwrap();
        let reopened = Document::from_bytes(&saved, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.objects[6].kind,
            Kind::FunctionDefinition(expression)
        );
        assert_eq!(reopened.function_plot_points(8, 3).unwrap()[1], after);
        doc.undo();
        assert_eq!(doc.function_plot_points(8, 3).unwrap()[1], before);
        doc.redo();
        assert_eq!(doc.function_plot_points(8, 3).unwrap()[1], after);

        let compound = SimpleFunction::parse("2*(x+1)").unwrap();
        doc.set_function_expression(7, compound).unwrap();
        assert_eq!(
            doc.encode().unwrap(),
            include_bytes!("../tests/fixtures/function_plot_edited_compound_generated.gsp")
        );
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects[6].kind, Kind::FunctionDefinition(compound));
        assert!(!reopened.partial_view);

        let sine = SimpleFunction::parse("sin(x)").unwrap();
        doc.set_function_expression_with_angle_unit(7, sine, true)
            .unwrap();
        assert!(doc.angle_unit_radians);
        assert_eq!(
            doc.encode().unwrap(),
            include_bytes!("../tests/fixtures/function_plot_edited_sine_radians_generated.gsp")
        );
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects[6].kind, Kind::FunctionDefinition(sine));
        assert!(reopened.angle_unit_radians);
        doc.undo();
        assert_eq!(doc.objects[6].kind, Kind::FunctionDefinition(compound));
        assert!(!doc.angle_unit_radians);
    }

    #[test]
    fn parametric_plots_import_render_and_cross_open() {
        for (mode, native, generated) in [
            (
                PlotPointsMode::Cartesian,
                include_bytes!("../tests/fixtures/parametric_rectangular_original.gsp").as_slice(),
                include_bytes!("../tests/fixtures/parametric_rectangular_generated.gsp").as_slice(),
            ),
            (
                PlotPointsMode::Polar,
                include_bytes!("../tests/fixtures/parametric_polar_original.gsp").as_slice(),
                include_bytes!("../tests/fixtures/parametric_polar_generated.gsp").as_slice(),
            ),
        ] {
            let original = Document::from_bytes(native, None).unwrap();
            assert!(!original.partial_view);
            assert_eq!(original.encode().unwrap(), native);
            assert!(
                matches!(original.objects[8].kind, Kind::ParametricPlot(1, 2, 8, 0., 1., 750, m) if m == mode)
            );

            let mut doc = Document::from_bytes(
                include_bytes!("../tests/fixtures/parametric_two_functions_moved_original.gsp"),
                None,
            )
            .unwrap();
            assert!(!doc.partial_view);
            let plot = doc
                .plot_parametric_curve(1, 2, mode, 0., 1., Point { x: 199., y: 125. })
                .unwrap();
            assert_eq!(plot, 9);
            assert_eq!(doc.encode().unwrap(), generated);
            assert_eq!(
                doc.grid_form[0] == GridForm::Polar,
                mode == PlotPointsMode::Polar
            );
            let frame = doc.graph_frame(8).unwrap();
            let segments = doc.parametric_plot_segments(plot, 101).unwrap();
            assert!(!segments.is_empty());
            assert_eq!(
                segments[0].0,
                Point {
                    x: frame.origin.x,
                    y: frame.origin.y
                }
            );
            let (x, y) = match mode {
                PlotPointsMode::Cartesian => (1., 1.),
                PlotPointsMode::Polar => (1_f64.cos(), 1_f64.sin()),
            };
            let endpoint = frame.world(x, y).unwrap();
            let actual = segments.last().unwrap().1;
            assert!((actual.x - endpoint.x).abs() < 1e-9);
            assert!((actual.y - endpoint.y).abs() < 1e-9);
            let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
            assert!(!reopened.partial_view);
            assert_eq!(reopened.objects[8].kind, doc.objects[8].kind);
        }

        let mut custom = Document::from_bytes(
            include_bytes!("../tests/fixtures/parametric_two_functions_moved_original.gsp"),
            None,
        )
        .unwrap();
        custom
            .plot_parametric_curve(
                1,
                2,
                PlotPointsMode::Cartesian,
                -1.,
                2.,
                Point { x: 199., y: 125. },
            )
            .unwrap();
        assert_eq!(
            custom.encode().unwrap(),
            include_bytes!("../tests/fixtures/parametric_custom_domain_generated.gsp")
        );

        let mut native = Document::from_bytes(
            include_bytes!("../tests/fixtures/parametric_rectangular_original.gsp"),
            None,
        )
        .unwrap();
        let before = native
            .parametric_plot_segments(9, 101)
            .unwrap()
            .last()
            .unwrap()
            .1;
        native
            .set_function_expression(2, SimpleFunction::parse("x+1").unwrap())
            .unwrap();
        let after = native
            .parametric_plot_segments(9, 101)
            .unwrap()
            .last()
            .unwrap()
            .1;
        assert!((after.x - before.x).abs() < 1e-9);
        assert!((after.y - before.y + 37.79527559055118).abs() < 1e-9);
        let reopened = Document::from_bytes(&native.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened
                .parametric_plot_segments(9, 101)
                .unwrap()
                .last()
                .unwrap()
                .1,
            after
        );
    }

    #[test]
    fn plot_property_edits_reopen_and_limit_render_samples() {
        let mut parametric = Document::from_bytes(
            include_bytes!("../tests/fixtures/parametric_rectangular_original.gsp"),
            None,
        )
        .unwrap();
        assert!(parametric.set_plot_range(1, -1., 2., 64).is_err());
        parametric.set_plot_range(9, -1., 2., 64).unwrap();
        assert_eq!(
            parametric.parametric_plot_segments(9, 384).unwrap().len(),
            63
        );
        assert_eq!(
            parametric.encode().unwrap(),
            include_bytes!("../tests/fixtures/parametric_range_edited_generated.gsp")
        );
        let reopened = Document::from_bytes(&parametric.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(matches!(
            reopened.objects[8].kind,
            Kind::ParametricPlot(1, 2, 8, -1., 2., 64, PlotPointsMode::Cartesian)
        ));
        parametric.undo();
        assert_eq!(
            parametric.encode().unwrap(),
            include_bytes!("../tests/fixtures/parametric_rectangular_original.gsp")
        );
        parametric.redo();
        assert_eq!(
            parametric.parametric_plot_segments(9, 384).unwrap().len(),
            63
        );

        let mut function = Document::from_bytes(
            include_bytes!("../tests/fixtures/function_plot_square.gsp"),
            None,
        )
        .unwrap();
        function.set_plot_range(8, -2., 2., 64).unwrap();
        assert_eq!(function.function_plot_segments(8, 384).unwrap().len(), 63);
        assert_eq!(
            function.encode().unwrap(),
            include_bytes!("../tests/fixtures/function_range_edited_generated.gsp")
        );
        let reopened = Document::from_bytes(&function.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!(matches!(
            reopened.objects[7].kind,
            Kind::FunctionPlot(7, 6, -2., 2., 64)
        ));

        let mut fresh_function = Document::new();
        fresh_function
            .plot_new_function(SimpleFunction::Square, Point { x: 199., y: 125. })
            .unwrap();
        fresh_function.set_plot_range(8, -2., 2., 64).unwrap();
        assert_eq!(
            fresh_function.encode().unwrap(),
            include_bytes!("../tests/fixtures/function_new_range_generated.gsp")
        );

        let mut fresh_parametric = Document::from_bytes(
            include_bytes!("../tests/fixtures/parametric_two_functions_moved_original.gsp"),
            None,
        )
        .unwrap();
        fresh_parametric
            .plot_parametric_curve(
                1,
                2,
                PlotPointsMode::Polar,
                0.,
                1.,
                Point { x: 199., y: 125. },
            )
            .unwrap();
        fresh_parametric.set_plot_range(9, -1., 2., 64).unwrap();
        assert_eq!(
            fresh_parametric
                .parametric_plot_segments(9, 384)
                .unwrap()
                .len(),
            63
        );
        assert_eq!(
            fresh_parametric.encode().unwrap(),
            include_bytes!("../tests/fixtures/parametric_new_range_generated.gsp")
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
    fn named_parameter_table_plot_matches_original_point_group() {
        let original = include_bytes!("../tests/fixtures/plot_points_named_speed_original.gsp");
        let native = Document::from_bytes(original, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), original);
        assert!(matches!(
            native.objects[8].kind,
            Kind::PlottedPoint(1, 8, PlotPointsMode::Cartesian, [1.0, 1.0])
        ));
        let mut doc = Document::new();
        doc.add_named_parameter("speed", 2.5, (10, 5)).unwrap();
        doc.tabulate_default_parameter(1, (10, 45)).unwrap();
        doc.add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(doc.plot_default_parameter(PlotPointsMode::Cartesian), Ok(9));
        let saved = doc.encode().unwrap();
        let last = |bytes: &[u8]| {
            let entries = records(bytes).unwrap();
            let start = entries.iter().rfind(|r| r.kind == 0x07d0).unwrap().start;
            let end = entries.iter().rfind(|r| r.kind == 0x07d7).unwrap().data.end;
            bytes[start..end].to_vec()
        };
        assert_eq!(last(&saved), last(original));
        let reopened = Document::from_bytes(&saved, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(9), native.position(9));
    }

    #[test]
    fn custom_named_plot_coordinates_match_original_groups() {
        let original_cases: [(PlotPointsMode, [f64; 2], &[u8]); 2] = [
            (
                PlotPointsMode::Cartesian,
                [2.5, -1.25],
                include_bytes!(
                    "../tests/fixtures/plot_points_named_speed_custom_cartesian_original.gsp"
                ),
            ),
            (
                PlotPointsMode::Polar,
                [2.5, 30.0],
                include_bytes!(
                    "../tests/fixtures/plot_points_named_speed_custom_polar_original.gsp"
                ),
            ),
        ];
        for (mode, coordinates, original) in original_cases {
            let native = Document::from_bytes(original, None).unwrap();
            assert!(!native.partial_view);
            assert_eq!(native.encode().unwrap(), original);
            let mut doc = Document::new();
            doc.add_named_parameter("speed", 2.5, (10, 5)).unwrap();
            doc.tabulate_default_parameter(1, (10, 45)).unwrap();
            doc.add_default_coordinate_system(Point { x: 199., y: 125. })
                .unwrap();
            assert_eq!(doc.plot_points_at(mode, coordinates), Ok(9));
            let saved = doc.encode().unwrap();
            let last = |bytes: &[u8]| {
                let entries = records(bytes).unwrap();
                let start = entries.iter().rfind(|r| r.kind == 0x07d0).unwrap().start;
                let end = entries.iter().rfind(|r| r.kind == 0x07d7).unwrap().data.end;
                bytes[start..end].to_vec()
            };
            assert_eq!(last(&saved), last(original));
            let reopened = Document::from_bytes(&saved, None).unwrap();
            assert_eq!(reopened.position(9), native.position(9));
        }
    }

    #[test]
    fn repeated_plot_points_round_trip_and_reject_invalid_coordinates() {
        let mut doc = Document::new();
        doc.add_named_parameter("speed", 2.5, (10, 5)).unwrap();
        doc.tabulate_default_parameter(1, (10, 45)).unwrap();
        doc.add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(
            doc.plot_points_at(PlotPointsMode::Cartesian, [2.5, -1.25]),
            Ok(9)
        );
        assert_eq!(
            doc.plot_points_at(PlotPointsMode::Polar, [2.5, 30.0]),
            Ok(10)
        );
        let before_invalid = doc.encode().unwrap();
        assert!(
            doc.plot_points_at(PlotPointsMode::Cartesian, [f64::NAN, 1.0])
                .is_err()
        );
        assert_eq!(doc.encode().unwrap(), before_invalid);
        doc.undo();
        assert_eq!(doc.objects.len(), 9);
        doc.redo();
        assert_eq!(doc.objects.len(), 10);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(9), doc.position(9));
        assert_eq!(reopened.position(10), doc.position(10));
    }

    #[test]
    fn frame_only_plot_point_matches_original() {
        let original = include_bytes!("../tests/fixtures/plot_points_frame_only_original.gsp");
        let native = Document::from_bytes(original, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), original);
        assert!(matches!(
            native.objects[6].kind,
            Kind::PlottedPoint(1, 6, PlotPointsMode::Cartesian, [1.0, 1.0])
        ));
        let mut doc = Document::new();
        doc.add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(
            doc.plot_points_at(PlotPointsMode::Cartesian, [1.0, 1.0]),
            Ok(7)
        );
        let generated = doc.encode().unwrap();
        let last = |bytes: &[u8]| {
            let entries = records(bytes).unwrap();
            let start = entries.iter().rfind(|r| r.kind == 0x07d0).unwrap().start;
            let end = entries.iter().rfind(|r| r.kind == 0x07d7).unwrap().data.end;
            bytes[start..end].to_vec()
        };
        assert_eq!(last(&generated), last(original));
        let reopened = Document::from_bytes(&generated, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(7), native.position(7));
    }

    #[test]
    fn plot_points_anchor_to_first_object_in_mixed_sketches() {
        for (original, text_first) in [
            (
                include_bytes!("../tests/fixtures/plot_points_prepoint_original.gsp").as_slice(),
                false,
            ),
            (
                include_bytes!("../tests/fixtures/plot_points_pretext_original.gsp").as_slice(),
                true,
            ),
        ] {
            let native = Document::from_bytes(original, None).unwrap();
            assert!(!native.partial_view);
            assert_eq!(native.encode().unwrap(), original);
            assert!(matches!(
                native.objects[7].kind,
                Kind::PlottedPoint(1, 7, PlotPointsMode::Cartesian, [1.0, 1.0])
            ));
            let mut doc = Document::new();
            if text_first {
                doc.add_text("hello", Point { x: 10., y: 5. }).unwrap();
            } else {
                doc.add(Kind::Free(Point { x: 100., y: 100. }));
            }
            doc.add_default_coordinate_system(Point { x: 199., y: 125. })
                .unwrap();
            assert_eq!(
                doc.plot_points_at(PlotPointsMode::Cartesian, [1., 1.]),
                Ok(8)
            );
            let generated = doc.encode().unwrap();
            let last = |bytes: &[u8]| {
                let entries = records(bytes).unwrap();
                let start = entries.iter().rfind(|r| r.kind == 0x07d0).unwrap().start;
                let end = entries.iter().rfind(|r| r.kind == 0x07d7).unwrap().data.end;
                bytes[start..end].to_vec()
            };
            assert_eq!(last(&generated), last(original));
            let reopened = Document::from_bytes(&generated, None).unwrap();
            assert!(!reopened.partial_view);
            assert_eq!(reopened.position(8), native.position(8));
        }
    }

    #[test]
    fn polar_table_data_matches_original_point_group() {
        let original = include_bytes!("../tests/fixtures/plot_table_fraction_polar_original.gsp");
        let native = Document::from_bytes(original, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), original);
        assert!(matches!(
            native.objects[9].kind,
            Kind::PlottedPoint(1, 9, PlotPointsMode::Polar, [2.5, angle])
                if angle == plot_points_records::DEGREES_TO_RADIANS
        ));
        let base = include_bytes!("../tests/fixtures/plot_table_fraction_base_original.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(
            generated.plot_pair_table_data(3, PlotPointsMode::Polar),
            Ok(vec![10])
        );
        let saved = generated.encode().unwrap();
        let last = |bytes: &[u8]| {
            let entries = records(bytes).unwrap();
            let start = entries.iter().rfind(|r| r.kind == 0x07d0).unwrap().start;
            let end = entries.iter().rfind(|r| r.kind == 0x07d7).unwrap().data.end;
            bytes[start..end].to_vec()
        };
        assert_eq!(last(&saved), last(original));
        let reopened = Document::from_bytes(&saved, None).unwrap();
        assert_eq!(reopened.position(10), native.position(10));
    }

    #[test]
    fn distinct_polar_table_rows_plot_and_round_trip() {
        let original = include_bytes!("../tests/fixtures/plot_table_distinct_polar_original.gsp");
        let native = Document::from_bytes(original, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), original);
        for (id, radius) in [(10, 1.0), (11, 2.0)] {
            assert!(matches!(
                native.objects[id - 1].kind,
                Kind::PlottedPoint(1, 9, PlotPointsMode::Polar, [r, angle])
                    if r == radius && angle == plot_points_records::DEGREES_TO_RADIANS
            ));
        }
        let base = include_bytes!("../tests/fixtures/plot_table_distinct_base_original.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(
            generated.plot_pair_table_data(3, PlotPointsMode::Polar),
            Ok(vec![10, 11])
        );
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        for id in [10, 11] {
            assert_eq!(reopened.objects[id - 1].kind, native.objects[id - 1].kind);
            assert_eq!(reopened.position(id), native.position(id));
        }
    }

    #[test]
    fn three_row_pair_table_edits_live_row_and_plots_distinct_values() {
        let original =
            include_bytes!("../tests/fixtures/table_pair_three_rows_distinct_original.gsp");
        let native = Document::from_bytes(original, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), original);
        assert!(matches!(
            &native.objects[2].kind,
            Kind::ValueTablePair(2, 1, group)
                if group.pair_values() == Some(vec![[1., 1.], [2., 1.], [2., 1.]])
        ));

        let base = include_bytes!("../tests/fixtures/plot_table_distinct_base_original.gsp");
        let mut doc = Document::from_bytes(base, None).unwrap();
        doc.change_table_rows(3, true).unwrap();
        assert!(matches!(
            &doc.objects[2].kind,
            Kind::ValueTablePair(2, 1, group)
                if group.pair_values() == Some(vec![[1., 1.], [2., 1.], [2., 1.]])
        ));
        doc.set_parameter_value(2, 3.25).unwrap();
        assert!(matches!(
            &doc.objects[2].kind,
            Kind::ValueTablePair(2, 1, group)
                if group.pair_values() == Some(vec![[1., 1.], [2., 1.], [3.25, 1.]])
        ));
        assert_eq!(
            doc.plot_pair_table_data(3, PlotPointsMode::Polar),
            Ok(vec![10, 11, 12])
        );
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[11].kind, doc.objects[11].kind);
        assert_eq!(reopened.position(12), doc.position(12));
    }

    #[test]
    fn five_distinct_pair_rows_plot_and_stay_fixed_after_parameter_edit() {
        let native_four = include_bytes!("../tests/fixtures/table_pair_four_rows_original.gsp");
        let imported = Document::from_bytes(native_four, None).unwrap();
        assert!(!imported.partial_view);
        assert_eq!(imported.encode().unwrap(), native_four);
        let cross_opened =
            include_bytes!("../tests/fixtures/table_pair_five_distinct_polar_generated.gsp");
        let imported = Document::from_bytes(cross_opened, None).unwrap();
        assert!(!imported.partial_view);
        assert_eq!(imported.encode().unwrap(), cross_opened);

        let base = include_bytes!("../tests/fixtures/plot_table_distinct_base_original.gsp");
        let mut doc = Document::from_bytes(base, None).unwrap();
        for next in [3.25, 4.5, 5.75] {
            doc.change_table_rows(3, true).unwrap();
            doc.set_parameter_value(2, next).unwrap();
        }
        assert!(matches!(
            &doc.objects[2].kind,
            Kind::ValueTablePair(2, 1, group)
                if group.pair_values() == Some(vec![[1., 1.], [2., 1.], [3.25, 1.], [4.5, 1.], [5.75, 1.]])
        ));
        assert_eq!(
            doc.plot_pair_table_data(3, PlotPointsMode::Polar),
            Ok(vec![10, 11, 12, 13, 14])
        );
        let plotted = (10..=14)
            .map(|id| doc.position(id).unwrap())
            .collect::<Vec<_>>();
        doc.set_parameter_value(2, 6.0).unwrap();
        assert!(matches!(
            &doc.objects[2].kind,
            Kind::ValueTablePair(2, 1, group)
                if group.pair_values().unwrap().last() == Some(&[6., 1.])
        ));
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            (10..=14)
                .map(|id| reopened.position(id).unwrap())
                .collect::<Vec<_>>(),
            plotted
        );
    }

    #[test]
    fn removing_pair_rows_preserves_earlier_cached_values() {
        let five = include_bytes!("../tests/fixtures/table_pair_five_distinct_polar_generated.gsp");
        let mut plotted_doc = Document::from_bytes(five, None).unwrap();
        let positions = (10..=14)
            .map(|id| plotted_doc.position(id).unwrap())
            .collect::<Vec<_>>();
        plotted_doc.change_table_rows(3, false).unwrap();
        assert_eq!(
            plotted_doc.encode().unwrap(),
            include_bytes!("../tests/fixtures/table_pair_four_rows_after_remove_generated.gsp")
        );
        assert_eq!(
            (10..=14)
                .map(|id| plotted_doc.position(id).unwrap())
                .collect::<Vec<_>>(),
            positions
        );

        let base = include_bytes!("../tests/fixtures/plot_table_distinct_base_original.gsp");
        let mut doc = Document::from_bytes(base, None).unwrap();
        for next in [3.25, 4.5, 5.75] {
            doc.change_table_rows(3, true).unwrap();
            doc.set_parameter_value(2, next).unwrap();
        }
        doc.change_table_rows(3, false).unwrap();
        assert!(matches!(
            &doc.objects[2].kind,
            Kind::ValueTablePair(2, 1, group)
                if group.pair_values() == Some(vec![[1., 1.], [2., 1.], [3.25, 1.], [5.75, 1.]])
        ));
        doc.change_table_rows(3, false).unwrap();
        assert!(matches!(
            &doc.objects[2].kind,
            Kind::ValueTablePair(2, 1, group)
                if group.pair_values() == Some(vec![[1., 1.], [2., 1.], [5.75, 1.]])
        ));
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[2].kind, doc.objects[2].kind);

        doc.change_table_rows(3, false).unwrap();
        doc.change_table_rows(3, false).unwrap();
        assert!(doc.change_table_rows(3, false).is_err());
        assert!(matches!(
            &doc.objects[2].kind,
            Kind::ValueTablePair(2, 1, group)
                if group.pair_values() == Some(vec![[5.75, 1.]])
        ));
    }

    #[test]
    fn pair_table_can_be_plotted_again_after_parameter_edit() {
        let five = include_bytes!("../tests/fixtures/table_pair_five_distinct_polar_generated.gsp");
        let mut doc = Document::from_bytes(five, None).unwrap();
        let first_positions = (10..=14)
            .map(|id| doc.position(id).unwrap())
            .collect::<Vec<_>>();
        doc.set_parameter_value(2, 6.0).unwrap();
        assert_eq!(
            doc.plot_pair_table_data(3, PlotPointsMode::Polar),
            Ok(vec![15, 16, 17, 18, 19])
        );
        assert_eq!(
            (10..=14)
                .map(|id| doc.position(id).unwrap())
                .collect::<Vec<_>>(),
            first_positions
        );
        let bytes = doc.encode().unwrap();
        assert_eq!(
            bytes,
            include_bytes!("../tests/fixtures/table_pair_replotted_polar_generated.gsp")
        );
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(19), doc.position(19));
    }

    #[test]
    fn pair_table_uses_later_parameter_and_frame_ids() {
        let mut doc = Document::new();
        assert_eq!(doc.add(Kind::Free(Point { x: 300., y: 300. })), 1);
        assert_eq!(doc.add_default_parameter((10, 5)).unwrap(), 2);
        assert_eq!(doc.add_default_parameter((10, 38)).unwrap(), 3);
        doc.set_parameter_value(3, 2.0).unwrap();
        assert_eq!(doc.tabulate_default_pair((10, 71)).unwrap(), 4);
        doc.change_table_rows(4, true).unwrap();
        doc.set_parameter_value(3, 3.25).unwrap();
        assert!(matches!(
            &doc.objects[3].kind,
            Kind::ValueTablePair(3, 2, group)
                if group.pair_source_ids() == Some([3, 2])
                    && group.pair_values() == Some(vec![[2., 1.], [3.25, 1.]])
        ));
        assert_eq!(
            doc.add_default_coordinate_system(Point { x: 199., y: 125. })
                .unwrap(),
            10
        );
        assert_eq!(
            doc.plot_pair_table_data(4, PlotPointsMode::Polar),
            Ok(vec![11, 12])
        );
        let positions = [doc.position(11).unwrap(), doc.position(12).unwrap()];
        let bytes = doc.encode().unwrap();
        assert_eq!(
            bytes,
            include_bytes!("../tests/fixtures/table_pair_late_sources_polar_generated.gsp")
        );
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(11), Some(positions[0]));
        assert_eq!(reopened.position(12), Some(positions[1]));
    }

    #[test]
    fn pair_table_caches_and_plots_both_parameter_values() {
        let mut doc = Document::new();
        doc.add_default_parameter((10, 5)).unwrap();
        doc.add_default_parameter((10, 38)).unwrap();
        doc.set_parameter_value(1, 2.5).unwrap();
        doc.set_parameter_value(2, 3.25).unwrap();
        doc.tabulate_default_pair((10, 71)).unwrap();
        doc.change_table_rows(3, true).unwrap();
        doc.set_parameter_value(1, 4.0).unwrap();
        doc.set_parameter_value(2, 5.75).unwrap();
        assert!(matches!(
            &doc.objects[2].kind,
            Kind::ValueTablePair(2, 1, group)
                if group.pair_values() == Some(vec![[3.25, 2.5], [5.75, 4.0]])
        ));
        doc.add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(doc.plot_default_pair_table(3), Ok(vec![10, 11]));
        let positions = [doc.position(10).unwrap(), doc.position(11).unwrap()];
        let bytes = doc.encode().unwrap();
        assert_eq!(
            bytes,
            include_bytes!("../tests/fixtures/table_pair_both_columns_cartesian_generated.gsp")
        );
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(10), Some(positions[0]));
        assert_eq!(reopened.position(11), Some(positions[1]));

        doc.set_parameter_value(1, 4.5).unwrap();
        assert!(matches!(
            &doc.objects[2].kind,
            Kind::ValueTablePair(2, 1, group)
                if group.pair_values() == Some(vec![[3.25, 2.5], [5.75, 4.5]])
        ));
        assert_eq!(doc.position(10), Some(positions[0]));
        assert_eq!(doc.position(11), Some(positions[1]));
        doc.change_table_rows(3, false).unwrap();
        assert!(matches!(
            &doc.objects[2].kind,
            Kind::ValueTablePair(2, 1, group)
                if group.pair_values() == Some(vec![[5.75, 4.5]])
        ));
    }

    #[test]
    fn pair_table_preserves_native_wide_cells_and_plots_mixed_rows() {
        for (bytes, expected) in [
            (
                include_bytes!("../tests/fixtures/table_pair_negative_x_original.gsp").as_slice(),
                [-2.5, 1.0],
            ),
            (
                include_bytes!("../tests/fixtures/table_pair_two_digit_y_original.gsp").as_slice(),
                [1.0, 12.34],
            ),
        ] {
            let imported = Document::from_bytes(bytes, None).unwrap();
            assert!(!imported.partial_view);
            assert!(matches!(
                &imported.objects[2].kind,
                Kind::ValueTablePair(2, 1, group)
                    if group.pair_values() == Some(vec![expected])
            ));
            assert_eq!(imported.encode().unwrap(), bytes);
        }

        let mut doc = Document::new();
        doc.add_default_parameter((10, 5)).unwrap();
        doc.add_default_parameter((10, 38)).unwrap();
        doc.set_parameter_value(1, 12.34).unwrap();
        doc.set_parameter_value(2, -2.5).unwrap();
        doc.tabulate_default_pair((10, 71)).unwrap();
        doc.change_table_rows(3, true).unwrap();
        doc.set_parameter_value(1, -3.25).unwrap();
        doc.set_parameter_value(2, 12.34).unwrap();
        doc.change_table_rows(3, true).unwrap();
        doc.set_parameter_value(1, 2.5).unwrap();
        doc.set_parameter_value(2, 3.25).unwrap();
        assert!(matches!(
            &doc.objects[2].kind,
            Kind::ValueTablePair(2, 1, group)
                if group.pair_values()
                    == Some(vec![[-2.5, 12.34], [12.34, -3.25], [3.25, 2.5]])
        ));
        doc.add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(doc.plot_default_pair_table(3), Ok(vec![10, 11, 12]));
        let plotted = (10..=12)
            .map(|id| doc.position(id).unwrap())
            .collect::<Vec<_>>();
        let bytes = doc.encode().unwrap();
        assert_eq!(
            bytes,
            include_bytes!("../tests/fixtures/table_pair_mixed_width_plot_generated.gsp")
        );
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            (10..=12)
                .map(|id| reopened.position(id).unwrap())
                .collect::<Vec<_>>(),
            plotted
        );

        doc.set_parameter_value(2, -99.99).unwrap();
        assert!(matches!(
            &doc.objects[2].kind,
            Kind::ValueTablePair(2, 1, group)
                if group.pair_values()
                    == Some(vec![[-2.5, 12.34], [12.34, -3.25], [-99.99, 2.5]])
        ));
        assert_eq!(
            (10..=12)
                .map(|id| doc.position(id).unwrap())
                .collect::<Vec<_>>(),
            plotted
        );
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
    fn default_table_keeps_fractional_snapshots_and_native_edit() {
        let original = include_bytes!("../tests/fixtures/table_t1_tabulated_edited_original.gsp");
        let imported = Document::from_bytes(original, None).unwrap();
        assert!(!imported.partial_view);
        assert!(matches!(
            &imported.objects[1].kind,
            Kind::ValueTable(1, group) if group.default_values() == Some(vec![2.5])
        ));
        assert_eq!(imported.encode().unwrap(), original);

        let mut doc = Document::new();
        doc.add_default_parameter((10, 5)).unwrap();
        doc.set_parameter_value(1, 2.5).unwrap();
        doc.tabulate_default_parameter(1, (10, 38)).unwrap();
        doc.change_table_rows(2, true).unwrap();
        doc.set_parameter_value(1, 3.25).unwrap();
        assert!(matches!(
            &doc.objects[1].kind,
            Kind::ValueTable(1, group) if group.default_values() == Some(vec![2.5, 3.25])
        ));
        let bytes = doc.encode().unwrap();
        assert_eq!(
            bytes,
            include_bytes!("../tests/fixtures/table_t1_snapshots_generated.gsp")
        );
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[1].kind, doc.objects[1].kind);

        doc.set_parameter_value(1, 4.5).unwrap();
        assert!(matches!(
            &doc.objects[1].kind,
            Kind::ValueTable(1, group) if group.default_values() == Some(vec![2.5, 4.5])
        ));
        doc.change_table_rows(2, false).unwrap();
        assert!(matches!(
            &doc.objects[1].kind,
            Kind::ValueTable(1, group) if group.default_values() == Some(vec![4.5])
        ));
    }

    #[test]
    fn default_table_accepts_later_parameter_source() {
        let mut doc = Document::new();
        assert_eq!(doc.add(Kind::Free(Point { x: 300., y: 300. })), 1);
        assert_eq!(doc.add_default_parameter((10, 5)).unwrap(), 2);
        doc.set_parameter_value(2, 2.5).unwrap();
        assert_eq!(doc.tabulate_default_parameter(2, (10, 38)).unwrap(), 3);
        doc.change_table_rows(3, true).unwrap();
        doc.set_parameter_value(2, 3.25).unwrap();
        assert!(matches!(
            &doc.objects[2].kind,
            Kind::ValueTable(2, group)
                if group.source_id() == 2 && group.default_values() == Some(vec![2.5, 3.25])
        ));
        let bytes = doc.encode().unwrap();
        assert_eq!(
            bytes,
            include_bytes!("../tests/fixtures/table_t1_late_source_snapshots_generated.gsp")
        );
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[2].kind, doc.objects[2].kind);

        doc.set_parameter_value(2, 4.5).unwrap();
        assert!(matches!(
            &doc.objects[2].kind,
            Kind::ValueTable(2, group) if group.default_values() == Some(vec![2.5, 4.5])
        ));
        doc.change_table_rows(3, false).unwrap();
        assert!(matches!(
            &doc.objects[2].kind,
            Kind::ValueTable(2, group) if group.default_values() == Some(vec![4.5])
        ));
    }

    #[test]
    fn default_table_preserves_native_wide_values_and_mixed_snapshots() {
        for (bytes, value) in [
            (
                include_bytes!("../tests/fixtures/table_t1_tabulated_negative_original.gsp")
                    .as_slice(),
                -2.5,
            ),
            (
                include_bytes!("../tests/fixtures/table_t1_tabulated_two_digit_original.gsp")
                    .as_slice(),
                12.34,
            ),
        ] {
            let imported = Document::from_bytes(bytes, None).unwrap();
            assert!(!imported.partial_view);
            assert!(matches!(
                &imported.objects[1].kind,
                Kind::ValueTable(1, group) if group.default_values() == Some(vec![value])
            ));
            assert_eq!(imported.encode().unwrap(), bytes);
        }

        let mut doc = Document::new();
        doc.add_default_parameter((10, 5)).unwrap();
        doc.set_parameter_value(1, -2.5).unwrap();
        doc.tabulate_default_parameter(1, (10, 38)).unwrap();
        doc.change_table_rows(2, true).unwrap();
        doc.set_parameter_value(1, 12.34).unwrap();
        doc.change_table_rows(2, true).unwrap();
        doc.set_parameter_value(1, -99.99).unwrap();
        assert!(matches!(
            &doc.objects[1].kind,
            Kind::ValueTable(1, group)
                if group.default_values() == Some(vec![-2.5, 12.34, -99.99])
        ));
        let bytes = doc.encode().unwrap();
        assert_eq!(
            bytes,
            include_bytes!("../tests/fixtures/table_t1_mixed_width_snapshots_generated.gsp")
        );
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[1].kind, doc.objects[1].kind);

        doc.set_parameter_value(1, 99.99).unwrap();
        assert!(matches!(
            &doc.objects[1].kind,
            Kind::ValueTable(1, group)
                if group.default_values() == Some(vec![-2.5, 12.34, 99.99])
        ));
        doc.change_table_rows(2, false).unwrap();
        assert!(matches!(
            &doc.objects[1].kind,
            Kind::ValueTable(1, group)
                if group.default_values() == Some(vec![-2.5, 99.99])
        ));
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
    fn named_parameters_decode_create_and_edit() {
        for (bytes, name, precision) in [
            (
                include_bytes!("../tests/fixtures/table_parameter_custom.gsp").as_slice(),
                "a",
                2,
            ),
            (
                include_bytes!("../tests/fixtures/table_parameter_speed_original.gsp").as_slice(),
                "speed",
                1,
            ),
            (
                include_bytes!("../tests/fixtures/table_parameter_speed_precision2_original.gsp")
                    .as_slice(),
                "speed",
                2,
            ),
            (
                include_bytes!("../tests/fixtures/table_parameter_test_original.gsp").as_slice(),
                "test",
                1,
            ),
        ] {
            let document = Document::from_bytes(bytes, None).unwrap();
            assert!(!document.partial_view);
            let Kind::Parameter(group) = &document.objects[0].kind else {
                panic!("expected parameter")
            };
            assert_eq!(group.name(), name);
            assert_eq!(group.precision(), precision);
            assert_eq!(document.encode().unwrap(), bytes);
        }
        let mut document = Document::new();
        document.add_named_parameter("a", 2.5, (10, 5)).unwrap();
        let generated = document.encode().unwrap();
        let reopened = Document::from_bytes(&generated, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.encode().unwrap(), generated);
        document.set_parameter_value(1, 3.25).unwrap();
        assert_eq!(
            Document::from_bytes(&document.encode().unwrap(), None)
                .unwrap()
                .objects[0]
                .kind,
            document.objects[0].kind
        );
    }

    #[test]
    fn multiple_named_parameters_can_follow_geometry_and_drive_a_plot() {
        let mut document = Document::new();
        assert_eq!(
            document.add_named_parameter("speed", 2.5, (10, 5)).unwrap(),
            1
        );
        document.add(Kind::Free(Point { x: 150., y: 120. }));
        document.add(Kind::Free(Point { x: 180., y: 130. }));
        document.add(Kind::Line(2, 3));
        assert_eq!(
            document
                .add_named_parameter("test", 0.25, (10, 38))
                .unwrap(),
            5
        );
        let before_duplicate = document.encode().unwrap();
        assert!(document.add_named_parameter("speed", 3., (10, 71)).is_err());
        assert_eq!(document.encode().unwrap(), before_duplicate);
        assert_eq!(document.plot_parameter_on_path(5, Some(4)).unwrap(), 6);
        assert_eq!(document.position(6), Some(Point { x: 157.5, y: 122.5 }));
        document.set_parameter_value(5, 0.5).unwrap();
        assert_eq!(document.position(6), Some(Point { x: 165., y: 125. }));
        let encoded = document.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(6), document.position(6));
        assert_eq!(reopened.encode().unwrap(), encoded);
    }

    #[test]
    fn named_parameter_tables_decode_and_save() {
        for (bytes, source, table_id, name, value) in [
            (
                include_bytes!("../tests/fixtures/table_named_speed_snapshot_original.gsp")
                    .as_slice(),
                1,
                2,
                "speed",
                "2.5",
            ),
            (
                include_bytes!(
                    "../tests/fixtures/table_named_speed_precision2_snapshot_original.gsp"
                )
                .as_slice(),
                1,
                2,
                "speed",
                "3.25",
            ),
            (
                include_bytes!("../tests/fixtures/table_named_test_late_source_original.gsp")
                    .as_slice(),
                5,
                7,
                "test",
                "0.25",
            ),
        ] {
            let document = Document::from_bytes(bytes, None).unwrap();
            assert!(!document.partial_view);
            assert!(
                matches!(&document.objects[table_id - 1].kind, Kind::ValueTable(id, group)
                if *id == source && group.named_label_value() == Some((name.into(), value.into())))
            );
            assert_eq!(document.encode().unwrap(), bytes);
        }
        let mut generated = Document::new();
        generated
            .add_named_parameter("speed", 2.5, (10, 5))
            .unwrap();
        let table_id = generated.tabulate_default_parameter(1, (10, 45)).unwrap();
        assert_eq!(table_id, 2);
        generated.change_table_rows(table_id, true).unwrap();
        assert!(
            matches!(&generated.objects[1].kind, Kind::ValueTable(1, table)
            if table.named_label_values() == Some(("speed".into(), vec!["2.5".into(), "2.5".into()])))
        );
        let encoded = generated.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.encode().unwrap(), encoded);
        generated.set_parameter_value(1, 3.25).unwrap();
        assert_eq!(
            crate::render::number_lines_for_object(&generated, table_id)
                .unwrap()
                .1,
            vec!["speed", "2.5", "3.25"]
        );
        generated.change_table_rows(table_id, true).unwrap();
        assert!(
            matches!(&generated.objects[1].kind, Kind::ValueTable(1, table)
            if table.named_label_values() == Some(("speed".into(), vec!["2.5".into(), "3.25".into(), "3.25".into()])))
        );
        generated.change_table_rows(table_id, false).unwrap();
        assert_eq!(
            crate::render::number_lines_for_object(&generated, table_id)
                .unwrap()
                .1,
            vec!["speed", "2.5", "3.25"]
        );
        let changed = generated.encode().unwrap();
        let reopened = Document::from_bytes(&changed, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.encode().unwrap(), changed);
        let original_two_rows =
            include_bytes!("../tests/fixtures/table_named_speed_two_rows_original.gsp");
        let original = Document::from_bytes(original_two_rows, None).unwrap();
        assert!(!original.partial_view);
        assert_eq!(original.encode().unwrap(), original_two_rows);
        assert!(
            matches!(&original.objects[1].kind, Kind::ValueTable(1, table)
            if table.named_label_values() == Some(("speed".into(), vec!["2.5".into(), "2.5".into()])))
        );
        let original_live_add =
            include_bytes!("../tests/fixtures/table_named_speed_live_add_original.gsp");
        let original = Document::from_bytes(original_live_add, None).unwrap();
        assert!(!original.partial_view);
        assert_eq!(original.encode().unwrap(), original_live_add);
        assert!(
            matches!(&original.objects[1].kind, Kind::ValueTable(1, table)
            if table.named_label_values() == Some(("speed".into(), vec!["3.25".into(), "3.25".into()])))
        );
    }

    #[test]
    fn trace_flags_follow_undo_and_round_trip_for_supported_shapes() {
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
            (
                include_bytes!("../tests/fixtures/trace_circle_original.gsp").as_slice(),
                3,
            ),
            (
                include_bytes!("../tests/fixtures/trace_polygon_original.gsp").as_slice(),
                7,
            ),
            (
                include_bytes!("../tests/fixtures/trace_ray_original.gsp").as_slice(),
                3,
            ),
            (
                include_bytes!("../tests/fixtures/trace_line_original.gsp").as_slice(),
                3,
            ),
            (
                include_bytes!("../tests/fixtures/trace_arc_original.gsp").as_slice(),
                4,
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

        let a = doc.add(Kind::Free(Point { x: 120., y: 150. }));
        let b = doc.add(Kind::Free(Point { x: 170., y: 150. }));
        let c = doc.add(Kind::Free(Point { x: 145., y: 100. }));
        let circle = doc.add(Kind::Circle(a, b));
        let arc = doc.add(Kind::ThreePointArc(a, c, b));
        let polygon = doc.add(Kind::Polygon(vec![a, b, c]));
        let ray = doc.add(Kind::Ray(a, b));
        let line = doc.add(Kind::Line(a, b));
        doc.selected.clear();
        doc.selected.extend([circle, arc, polygon, ray, line]);
        assert_eq!(doc.set_selected_tracing(true).unwrap(), 5);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        for id in [circle, arc, polygon, ray, line] {
            assert!(reopened.objects[id - 1].tracing);
        }
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
    #[test]
    fn native_midpoint_locus_import_generation_and_parent_edit() {
        let native_bytes = include_bytes!("../tests/fixtures/locus_midpoint_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(
            native.objects[7].kind,
            Kind::Locus(vec![7, 3, 4, 6, 7], 500)
        );
        let points = native.locus_segments(8, 8).unwrap();
        assert_eq!(points.first().unwrap().0, Point { x: 160., y: 140. });
        assert_eq!(points.last().unwrap().1, Point { x: 250., y: 140. });

        let base = include_bytes!("../tests/fixtures/locus_midpoint_base.gsp");
        let mut created = Document::from_bytes(base, None).unwrap();
        assert_eq!(created.construct_locus(4, 7), Ok(8));
        assert!(created.construct_locus(1, 7).is_err());
        let bytes = created.encode().unwrap();
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[7].kind, native.objects[7].kind);
        assert_eq!(reopened.locus_segments(8, 8), Some(points));

        assert!(created.move_point(1, Point { x: 100., y: 220. }));
        let edited = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
        assert!(!edited.partial_view);
        let moved = edited.locus_segments(8, 8).unwrap();
        assert_eq!(moved.first().unwrap().0, Point { x: 150., y: 160. });
        assert_eq!(moved.last().unwrap().1, Point { x: 250., y: 140. });
    }
    #[test]
    fn translated_point_locus_import_generation_and_parent_edit() {
        let native_bytes = include_bytes!("../tests/fixtures/locus_translated_point_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(native.objects[5].kind, Kind::Locus(vec![5, 3, 4, 5], 500));
        let native_segments = native.locus_segments(6, 8).unwrap();
        assert!((native_segments[0].0.x - 120.).abs() < 1e-9);
        assert!((native_segments[0].0.y - (180. - 37.79527559055118)).abs() < 1e-9);
        assert!((native_segments.last().unwrap().1.x - 300.).abs() < 1e-9);

        let base = include_bytes!("../tests/fixtures/locus_translated_point_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(4, 5), Ok(6));
        assert!(generated.construct_locus(4, 2).is_err());
        let saved = generated.encode().unwrap();
        let reopened = Document::from_bytes(&saved, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[5].kind, native.objects[5].kind);
        assert_eq!(reopened.locus_segments(6, 8), Some(native_segments));

        assert!(generated.move_point(1, Point { x: 100., y: 220. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let segments = moved.locus_segments(6, 8).unwrap();
        assert!((segments[0].0.x - 100.).abs() < 1e-9);
        assert!((segments[0].0.y - (220. - 37.79527559055118)).abs() < 1e-9);
        assert!((segments.last().unwrap().1.x - 300.).abs() < 1e-9);
    }
    #[test]
    fn circle_driver_locus_import_generation_and_parent_edit() {
        let native_bytes = include_bytes!("../tests/fixtures/locus_circle_driver_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(native.objects[5].kind, Kind::Locus(vec![5, 3, 4, 5], 500));
        let segments = native.locus_segments(6, 4).unwrap();
        let translated_y = 180. - 37.79527559055118;
        assert!((segments[0].0.x - 260.).abs() < 1e-9);
        assert!((segments[0].0.y - translated_y).abs() < 1e-9);
        assert!((segments[0].1.x - 200.).abs() < 1e-9);
        assert!((segments[0].1.y - (translated_y - 60.)).abs() < 1e-9);
        assert!((segments.last().unwrap().1.x - 260.).abs() < 1e-9);

        let base = include_bytes!("../tests/fixtures/locus_circle_driver_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(4, 5), Ok(6));
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[5].kind, native.objects[5].kind);
        assert_eq!(reopened.locus_segments(6, 4), Some(segments));

        assert!(generated.move_point(1, Point { x: 220., y: 190. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let first = moved.locus_segments(6, 4).unwrap()[0].0;
        assert!((first.x - (220. + 1700f64.sqrt())).abs() < 1e-9);
        assert!((first.y - (190. - 37.79527559055118)).abs() < 1e-9);
    }
    #[test]
    fn arc_driver_locus_import_generation_and_parent_edit() {
        let native_bytes = include_bytes!("../tests/fixtures/locus_arc_driver_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(native.objects[6].kind, Kind::Locus(vec![6, 4, 5, 6], 500));
        let segments = native.locus_segments(7, 4).unwrap();
        let offset = 37.79527559055118;
        assert!((segments[0].0.x - 100.).abs() < 1e-9);
        assert!((segments[0].0.y - (200. - offset)).abs() < 1e-9);
        assert!((segments.last().unwrap().1.x - 200.).abs() < 1e-9);
        assert!((segments[1].1.y - (150. - offset)).abs() < 1e-9);

        let base = include_bytes!("../tests/fixtures/locus_arc_driver_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(5, 6), Ok(7));
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[6].kind, native.objects[6].kind);
        assert_eq!(reopened.locus_segments(7, 4), Some(segments));

        assert!(generated.move_point(2, Point { x: 150., y: 130. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let changed = moved.locus_segments(7, 4).unwrap();
        assert!((changed[1].1.y - (130. - offset)).abs() < 1e-9);
    }
    #[test]
    fn two_step_locus_import_generation_and_parent_edit() {
        let native_bytes = include_bytes!("../tests/fixtures/locus_two_translations_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(
            native.objects[6].kind,
            Kind::Locus(vec![6, 3, 4, 5, 6], 500)
        );
        let original = native.locus_segments(7, 4).unwrap();
        let offset = 2. * 37.79527559055118;
        assert!((original[0].0.x - 120.).abs() < 1e-9);
        assert!((original[0].0.y - (180. - offset)).abs() < 1e-9);
        assert!((original.last().unwrap().1.x - 300.).abs() < 1e-9);

        let base = include_bytes!("../tests/fixtures/locus_two_translations_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(4, 6), Ok(7));
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[6].kind, native.objects[6].kind);
        assert_eq!(reopened.locus_segments(7, 4), Some(original));

        assert!(generated.move_point(1, Point { x: 100., y: 220. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let changed = moved.locus_segments(7, 4).unwrap();
        assert!((changed[0].0.x - 100.).abs() < 1e-9);
        assert!((changed[0].0.y - (220. - offset)).abs() < 1e-9);
    }
    #[test]
    fn branched_locus_and_non_right_angle_polar_cache() {
        let native_bytes = include_bytes!("../tests/fixtures/locus_branch_midpoint_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(
            native.objects[8].kind,
            Kind::Locus(vec![8, 3, 4, 5, 6, 7, 8], 500)
        );
        let original = native.locus_segments(9, 4).unwrap();
        let half_unit = 37.79527559055118 / 2.;
        assert!((original[0].0.x - (120. - half_unit)).abs() < 1e-9);
        assert!((original[0].0.y - (180. - half_unit)).abs() < 1e-9);
        assert!((original.last().unwrap().1.x - (300. - half_unit)).abs() < 1e-9);

        let base = include_bytes!("../tests/fixtures/locus_branch_midpoint_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        let offset = generated.objects[5].constraint_offset.unwrap();
        assert!(f64_at(base, offset + 4).abs() < 1e-12);
        assert_eq!(f64_at(base, offset + 12), -1.);
        let mut stale_cache = base.to_vec();
        stale_cache[offset + 4..offset + 20]
            .copy_from_slice(&include_bytes!("../assets/polar_translation_payload.bin")[4..20]);
        assert!(
            Document::from_bytes(&stale_cache, None)
                .unwrap()
                .partial_view
        );
        assert_eq!(generated.construct_locus(4, 8), Ok(9));
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[8].kind, native.objects[8].kind);
        assert_eq!(reopened.locus_segments(9, 4), Some(original));

        assert!(generated.move_point(1, Point { x: 100., y: 220. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let changed = moved.locus_segments(9, 4).unwrap();
        assert!((changed[0].0.x - (100. - half_unit)).abs() < 1e-9);
        assert!((changed[0].0.y - (220. - half_unit)).abs() < 1e-9);
    }
    #[test]
    fn segment_target_locus_import_generation_and_parent_edit() {
        let native_bytes = include_bytes!("../tests/fixtures/locus_segment_target_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(native.objects[7].kind, Kind::Locus(vec![6, 3, 4, 6], 250));
        let family = native.locus_segments(8, 384).unwrap();
        assert_eq!(family.len(), 251);
        assert_eq!(
            family.first().unwrap(),
            &(Point { x: 120., y: 180. }, Point { x: 200., y: 100. })
        );
        assert_eq!(
            family.last().unwrap(),
            &(Point { x: 300., y: 180. }, Point { x: 200., y: 100. })
        );

        let base = include_bytes!("../tests/fixtures/locus_midpoint_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(4, 6), Ok(8));
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[7].kind, native.objects[7].kind);
        assert_eq!(reopened.locus_segments(8, 384), Some(family));

        assert!(generated.move_point(1, Point { x: 100., y: 220. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let changed = moved.locus_segments(8, 384).unwrap();
        assert_eq!(changed.first().unwrap().0, Point { x: 100., y: 220. });
        assert_eq!(changed.last().unwrap().0, Point { x: 300., y: 180. });
    }
    #[test]
    fn circle_target_locus_import_generation_and_parent_edit() {
        let native_bytes = include_bytes!("../tests/fixtures/locus_circle_target_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(native.objects[8].kind, Kind::Locus(vec![8, 3, 4, 8], 250));
        let family = native.locus_segments(9, 384).unwrap();
        assert_eq!(family.len(), 251 * 64);
        assert_eq!(family[0].0, Point { x: 150., y: 180. });
        assert_eq!(family[250 * 64].0, Point { x: 330., y: 180. });

        let base = include_bytes!("../tests/fixtures/locus_circle_target_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(4, 8), Ok(9));
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[8].kind, native.objects[8].kind);
        assert_eq!(reopened.locus_segments(9, 384), Some(family));

        assert!(generated.move_point(1, Point { x: 100., y: 220. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let changed = moved.locus_segments(9, 384).unwrap();
        assert_eq!(changed[0].0, Point { x: 130., y: 220. });
        assert_eq!(changed[250 * 64].0, Point { x: 330., y: 180. });
    }
    #[test]
    fn line_target_locus_import_generation_and_parent_edit() {
        let native_bytes = include_bytes!("../tests/fixtures/locus_line_target_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(native.objects[6].kind, Kind::Locus(vec![6, 3, 4, 6], 250));
        let family = native.locus_strokes(7, 384).unwrap();
        assert_eq!(family.len(), 251);
        assert_eq!(
            family.first().unwrap(),
            &(
                Point { x: 120., y: 180. },
                Point { x: 200., y: 100. },
                LineMode::Line
            )
        );
        assert_eq!(
            family.last().unwrap(),
            &(
                Point { x: 300., y: 180. },
                Point { x: 200., y: 100. },
                LineMode::Line
            )
        );

        let base = include_bytes!("../tests/fixtures/locus_line_target_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(4, 6), Ok(7));
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[6].kind, native.objects[6].kind);
        assert_eq!(reopened.locus_strokes(7, 384), Some(family));

        assert!(generated.move_point(1, Point { x: 100., y: 220. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let changed = moved.locus_strokes(7, 384).unwrap();
        assert_eq!(changed.first().unwrap().0, Point { x: 100., y: 220. });
        assert_eq!(changed.last().unwrap().0, Point { x: 300., y: 180. });
        assert!(changed.iter().all(|stroke| stroke.2 == LineMode::Line));
    }
    #[test]
    fn ray_target_locus_import_generation_and_parent_edit() {
        let native_bytes = include_bytes!("../tests/fixtures/locus_ray_target_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(native.objects[6].kind, Kind::Locus(vec![6, 3, 4, 6], 250));
        let family = native.locus_strokes(7, 384).unwrap();
        assert_eq!(family.len(), 251);
        assert_eq!(
            family.first().unwrap(),
            &(
                Point { x: 120., y: 180. },
                Point { x: 200., y: 100. },
                LineMode::Ray
            )
        );
        assert_eq!(family.last().unwrap().0, Point { x: 300., y: 180. });
        assert!(family.iter().all(|stroke| stroke.2 == LineMode::Ray));

        let base = include_bytes!("../tests/fixtures/locus_ray_target_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(4, 6), Ok(7));
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[6].kind, native.objects[6].kind);
        assert_eq!(reopened.locus_strokes(7, 384), Some(family));

        assert!(generated.move_point(1, Point { x: 100., y: 220. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let changed = moved.locus_strokes(7, 384).unwrap();
        assert_eq!(changed.first().unwrap().0, Point { x: 100., y: 220. });
        assert_eq!(changed.last().unwrap().0, Point { x: 300., y: 180. });
    }
    #[test]
    fn arc_target_locus_import_generation_and_parent_edit() {
        let native_bytes = include_bytes!("../tests/fixtures/locus_arc_target_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(native.objects[7].kind, Kind::Locus(vec![7, 3, 4, 7], 250));
        let family = native.locus_strokes(8, 384).unwrap();
        assert_eq!(family.len(), 251 * 64);
        assert!(family[0].0.distance(Point { x: 120., y: 180. }) < 1e-9);
        assert!(family[250 * 64].0.distance(Point { x: 300., y: 180. }) < 1e-9);
        assert!(family.iter().all(|stroke| stroke.2 == LineMode::Segment));

        let base = include_bytes!("../tests/fixtures/locus_arc_target_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(4, 7), Ok(8));
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[7].kind, native.objects[7].kind);
        assert_eq!(reopened.locus_strokes(8, 384), Some(family));

        assert!(generated.move_point(1, Point { x: 100., y: 220. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let changed = moved.locus_strokes(8, 384).unwrap();
        assert!(changed[0].0.distance(Point { x: 100., y: 220. }) < 1e-9);
        assert!(changed[250 * 64].0.distance(Point { x: 300., y: 180. }) < 1e-9);
    }
    #[test]
    fn arc_on_circle_target_locus_import_generation_and_parent_edit() {
        let native_bytes =
            include_bytes!("../tests/fixtures/locus_arc_on_circle_target_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(native.objects[6].kind, Kind::Locus(vec![6, 3, 4, 6], 250));
        let family = native.locus_strokes(7, 384).unwrap();
        assert_eq!(family.len(), 251 * 64);
        assert!(family[0].0.distance(Point { x: 330., y: 180. }) < 1e-9);
        assert!(family[250 * 64].0.distance(Point { x: 330., y: 180. }) < 1e-9);
        let first = family[0].0;

        let base = include_bytes!("../tests/fixtures/locus_arc_on_circle_target_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(4, 6), Ok(7));
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[6].kind, native.objects[6].kind);
        assert_eq!(reopened.locus_strokes(7, 384), Some(family));

        assert!(generated.move_point(1, Point { x: 240., y: 190. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let changed = moved.locus_strokes(7, 384).unwrap();
        assert!(changed[0].0.distance(moved.position(4).unwrap()) < 1e-9);
        assert!(changed[0].0.distance(first) > 1.);
        assert!(changed[250 * 64].0.distance(changed[0].0) < 1e-9);
    }
    #[test]
    fn parallel_line_target_locus_import_generation_and_parent_edit() {
        let native_bytes = include_bytes!("../tests/fixtures/locus_parallel_target_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(native.objects[8].kind, Kind::Locus(vec![8, 3, 4, 8], 250));
        let family = native.locus_strokes(9, 384).unwrap();
        assert_eq!(family.len(), 251);
        assert_eq!(
            family[0],
            (
                Point { x: 120., y: 180. },
                Point { x: 220., y: 210. },
                LineMode::Line
            )
        );
        assert_eq!(family[250].0, Point { x: 300., y: 180. });
        assert_eq!(family[250].1, Point { x: 400., y: 210. });

        let base = include_bytes!("../tests/fixtures/locus_parallel_target_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(4, 8), Ok(9));
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[8].kind, native.objects[8].kind);
        assert_eq!(reopened.locus_strokes(9, 384), Some(family));

        assert!(generated.move_point(1, Point { x: 100., y: 220. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let changed = moved.locus_strokes(9, 384).unwrap();
        assert_eq!(changed[0].0, Point { x: 100., y: 220. });
        assert_eq!(changed[0].1, Point { x: 200., y: 250. });
        assert_eq!(changed[250].0, Point { x: 300., y: 180. });
    }
    #[test]
    fn perpendicular_line_target_locus_import_generation_and_parent_edit() {
        let native_bytes =
            include_bytes!("../tests/fixtures/locus_perpendicular_target_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(native.objects[8].kind, Kind::Locus(vec![8, 3, 4, 8], 250));
        let family = native.locus_strokes(9, 384).unwrap();
        assert_eq!(family.len(), 251);
        assert_eq!(
            family[0],
            (
                Point { x: 120., y: 180. },
                Point { x: 90., y: 280. },
                LineMode::Line
            )
        );
        assert_eq!(family[250].0, Point { x: 300., y: 180. });
        assert_eq!(family[250].1, Point { x: 270., y: 280. });

        let base = include_bytes!("../tests/fixtures/locus_perpendicular_target_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(4, 8), Ok(9));
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[8].kind, native.objects[8].kind);
        assert_eq!(reopened.locus_strokes(9, 384), Some(family));

        assert!(generated.move_point(1, Point { x: 100., y: 220. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let changed = moved.locus_strokes(9, 384).unwrap();
        assert_eq!(changed[0].0, Point { x: 100., y: 220. });
        assert_eq!(changed[0].1, Point { x: 70., y: 320. });
        assert_eq!(changed[250].0, Point { x: 300., y: 180. });
    }
    #[test]
    fn bisector_target_locus_import_generation_and_parent_edit() {
        let native_bytes = include_bytes!("../tests/fixtures/locus_bisector_target_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(native.objects[7].kind, Kind::Locus(vec![7, 3, 4, 7], 250));
        let family = native.locus_strokes(8, 384).unwrap();
        assert_eq!(family.len(), 251);
        assert_eq!(family[0].0, Point { x: 120., y: 180. });
        assert_eq!(family[250].0, Point { x: 300., y: 180. });
        assert!(family.iter().all(|stroke| stroke.2 == LineMode::Ray));
        assert!(family[0].1.y < family[0].0.y);

        let base = include_bytes!("../tests/fixtures/locus_bisector_target_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(4, 7), Ok(8));
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[7].kind, native.objects[7].kind);
        assert_eq!(reopened.locus_strokes(8, 384), Some(family));

        assert!(generated.move_point(1, Point { x: 100., y: 220. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let changed = moved.locus_strokes(8, 384).unwrap();
        assert_eq!(changed[0].0, Point { x: 100., y: 220. });
        assert_eq!(changed[250].0, Point { x: 300., y: 180. });
        assert!(changed.iter().all(|stroke| stroke.2 == LineMode::Ray));
    }
    #[test]
    fn polygon_target_locus_import_generation_and_parent_edit() {
        let native_bytes = include_bytes!("../tests/fixtures/locus_polygon_target_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(native.objects[7].kind, Kind::Locus(vec![7, 3, 4, 7], 125));
        let family = native.locus_polygons(8, 384).unwrap();
        assert_eq!(family.len(), 126);
        assert_eq!(
            family[0],
            vec![
                Point { x: 120., y: 180. },
                Point { x: 150., y: 100. },
                Point { x: 250., y: 100. }
            ]
        );
        assert_eq!(family[125][0], Point { x: 300., y: 180. });
        assert_eq!(native.locus_strokes(8, 384).unwrap().len(), 126 * 3);

        let base = include_bytes!("../tests/fixtures/locus_polygon_target_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(4, 7), Ok(8));
        let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[7].kind, native.objects[7].kind);
        assert_eq!(reopened.locus_polygons(8, 384), Some(family));

        assert!(generated.move_point(1, Point { x: 100., y: 220. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let changed = moved.locus_polygons(8, 384).unwrap();
        assert_eq!(changed[0][0], Point { x: 100., y: 220. });
        assert_eq!(changed[125][0], Point { x: 300., y: 180. });
    }
    #[test]
    fn new_triangle_uses_native_polygon_subtype() {
        let mut document = Document::new();
        for point in [
            Point { x: 120., y: 180. },
            Point { x: 150., y: 100. },
            Point { x: 250., y: 100. },
        ] {
            document.add(Kind::Free(point));
        }
        document.add(Kind::Polygon(vec![1, 2, 3]));
        let bytes = document.encode().unwrap();
        let polygon_header = records(&bytes)
            .unwrap()
            .into_iter()
            .find(|record| record.kind == 0x07d0 && bytes[record.data.start] == 8)
            .unwrap();
        assert_eq!(
            u16::from_le_bytes(
                bytes[polygon_header.data.start + 8..polygon_header.data.start + 10]
                    .try_into()
                    .unwrap()
            ),
            52
        );
    }
    #[test]
    fn vector_translated_polygon_locus_import_generation_and_parent_edit() {
        let native_bytes =
            include_bytes!("../tests/fixtures/locus_vector_polygon_target_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(
            native.objects[10].kind,
            Kind::Locus(vec![10, 3, 4, 7, 10], 125)
        );
        let family = native.locus_polygons(11, 384).unwrap();
        assert_eq!(family.len(), 126);
        assert_eq!(family[0][0], Point { x: 220., y: 180. });
        assert_eq!(family[125][0], Point { x: 400., y: 180. });
        assert_eq!(family[0][1], Point { x: 250., y: 100. });

        let base = include_bytes!("../tests/fixtures/locus_vector_polygon_target_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(4, 10), Ok(11));
        let encoded = generated.encode().unwrap();
        let target_header = records(&encoded)
            .unwrap()
            .into_iter()
            .find(|record| record.kind == 0x07d0 && encoded[record.data.start] == 16)
            .unwrap();
        assert_eq!(
            &encoded[target_header.data.start + 18..target_header.data.start + 20],
            &[0x39, 0x20]
        );
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[10].kind, native.objects[10].kind);
        assert_eq!(reopened.locus_polygons(11, 384), Some(family));

        assert!(generated.move_point(1, Point { x: 100., y: 220. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let changed = moved.locus_polygons(11, 384).unwrap();
        assert_eq!(changed[0][0], Point { x: 200., y: 220. });
        assert_eq!(changed[125][0], Point { x: 400., y: 180. });
    }
    #[test]
    fn polar_translated_polygon_locus_import_generation_and_parent_edit() {
        let native_bytes =
            include_bytes!("../tests/fixtures/locus_polar_polygon_target_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(
            native.objects[8].kind,
            Kind::Locus(vec![8, 3, 4, 7, 8], 125)
        );
        let offset = 37.79527559055118 * 2.65;
        let family = native.locus_polygons(9, 384).unwrap();
        assert_eq!(family.len(), 126);
        assert!((family[0][0].x - (120. + offset)).abs() < 1e-9);
        assert_eq!(family[0][0].y, 180.);
        assert!((family[125][0].x - (300. + offset)).abs() < 1e-9);
        assert!((family[0][1].x - (150. + offset)).abs() < 1e-9);

        let base = include_bytes!("../tests/fixtures/locus_polar_polygon_target_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(4, 8), Ok(9));
        let encoded = generated.encode().unwrap();
        let locus_header = records(&encoded)
            .unwrap()
            .into_iter()
            .filter(|record| record.kind == 0x07d0)
            .last()
            .unwrap();
        assert_eq!(
            &encoded[locus_header.data.start + 18..locus_header.data.start + 20],
            &[0xf0, 0x3f]
        );
        assert_eq!(
            &encoded[locus_header.data.start + 20..locus_header.data.start + 24],
            &2u32.to_le_bytes()
        );
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[8].kind, native.objects[8].kind);
        assert_eq!(reopened.locus_polygons(9, 384), Some(family));

        assert!(generated.move_point(1, Point { x: 100., y: 220. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let changed = moved.locus_polygons(9, 384).unwrap();
        assert!((changed[0][0].x - (100. + offset)).abs() < 1e-9);
        assert_eq!(changed[0][0].y, 220.);
        assert!((changed[125][0].x - (300. + offset)).abs() < 1e-9);
    }
    #[test]
    fn rectangular_translated_polygon_locus_import_generation_and_parent_edit() {
        let native_bytes =
            include_bytes!("../tests/fixtures/locus_rectangular_polygon_target_original.gsp");
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), native_bytes);
        assert_eq!(
            native.objects[8].kind,
            Kind::Locus(vec![8, 3, 4, 7, 8], 125)
        );
        let dx = 37.79527559055118 * 2.;
        let dy = 37.79527559055118 * 0.5;
        let family = native.locus_polygons(9, 384).unwrap();
        assert_eq!(family.len(), 126);
        assert!((family[0][0].x - (120. + dx)).abs() < 1e-9);
        assert!((family[0][0].y - (180. + dy)).abs() < 1e-9);
        assert!((family[125][0].x - (300. + dx)).abs() < 1e-9);

        let base = include_bytes!("../tests/fixtures/locus_rectangular_polygon_target_base.gsp");
        let mut generated = Document::from_bytes(base, None).unwrap();
        assert_eq!(generated.construct_locus(4, 8), Ok(9));
        let encoded = generated.encode().unwrap();
        let locus_header = records(&encoded)
            .unwrap()
            .into_iter()
            .filter(|record| record.kind == 0x07d0)
            .last()
            .unwrap();
        assert_eq!(
            &encoded[locus_header.data.start + 18..locus_header.data.start + 20],
            &[0, 0x40]
        );
        assert_eq!(
            &encoded[locus_header.data.start + 20..locus_header.data.start + 24],
            &2u32.to_le_bytes()
        );
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[8].kind, native.objects[8].kind);
        assert_eq!(reopened.locus_polygons(9, 384), Some(family));

        assert!(generated.move_point(1, Point { x: 100., y: 220. }));
        let moved = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
        let changed = moved.locus_polygons(9, 384).unwrap();
        assert!((changed[0][0].x - (100. + dx)).abs() < 1e-9);
        assert!((changed[0][0].y - (220. + dy)).abs() < 1e-9);
        assert!((changed[125][0].x - (300. + dx)).abs() < 1e-9);
    }
}
