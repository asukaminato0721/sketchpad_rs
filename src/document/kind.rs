use super::calculations::calculation_source_ids;
use super::transform_model::{ElementType, NativeTransformFlavor};
use super::{IterationDepth, Kind, LineMode, Object};
use crate::coordinate_measure_records::CoordinateMeasureKind;
use crate::style_records::{
    LinePattern, LineWidth, PointStyle, RgbColor, SegmentLineStyle, VisualStyle,
};
use crate::table_records::NumericUnit;
use crate::value_equation_records::ValueEquationKind;
use std::collections::BTreeSet;

impl Kind {
    pub fn is_iteration_depth_value(&self) -> bool {
        matches!(
            self,
            Kind::Parameter(_)
                | Kind::Calculation(_)
                | Kind::ParameterCalculation(_, _)
                | Kind::LengthPairCalculation(_, _, _)
        ) && self.numeric_unit() == Some(NumericUnit::Scalar)
    }
    pub fn numeric_unit(&self) -> Option<NumericUnit> {
        match self {
            Kind::Calculation(group) => Some(group.unit()),
            Kind::Parameter(group) => Some(group.unit()),
            Kind::PointRatio(_, _, _) | Kind::SlopeMeasurement(_, _) => {
                Some(NumericUnit::Scalar)
            }
            Kind::LengthPairCalculation(_, _, group) => match group.length_power() {
                0 => Some(NumericUnit::Scalar),
                1 => Some(NumericUnit::Length),
                2 => Some(NumericUnit::Area),
                _ => None,
            },
            Kind::SegmentLength(_)
            | Kind::PointDistance(_, _)
            | Kind::PointLineDistance(_, _)
            | Kind::PolygonPerimeter(_)
            | Kind::CircleCircumference(_)
            | Kind::CircleRadius(_)
            | Kind::ArcLength(_)
            | Kind::MeasuredCalculation(_, _) => Some(NumericUnit::Length),
            Kind::CircleArea(_) | Kind::PolygonArea(_) => Some(NumericUnit::Area),
            Kind::ValueEquation {
                kind: ValueEquationKind::PointValue,
                ..
            } => Some(NumericUnit::Scalar),
            Kind::CoordinateMeasurement {
                kind:
                    CoordinateMeasureKind::Abscissa
                    | CoordinateMeasureKind::Ordinate
                    | CoordinateMeasureKind::CoordinateDistance,
                ..
            } => Some(NumericUnit::Scalar),
            Kind::ParameterCalculation(_, group) => Some(group.unit()),
            Kind::AngleMeasurement(_, _, _) | Kind::ArcAngle(_) => Some(NumericUnit::Degrees),
            _ => None,
        }
    }
    pub fn is_distance_value(&self) -> bool {
        if let Kind::ParameterCalculation(_, group) = self {
            return group.unit() == NumericUnit::Length;
        }
        if let Kind::LengthPairCalculation(_, _, group) = self {
            return group.length_power() == 1;
        }
        matches!(
            self,
            Kind::SegmentLength(_)
                | Kind::PointDistance(_, _)
                | Kind::PointLineDistance(_, _)
                | Kind::PolygonPerimeter(_)
                | Kind::CircleCircumference(_)
                | Kind::CircleRadius(_)
                | Kind::ArcLength(_)
                | Kind::MeasuredCalculation(_, _)
        )
    }
    pub fn is_segment(&self) -> bool {
        is_segment_kind(self)
    }
    pub fn fixed_transform_straight_mode(&self) -> Option<LineMode> {
        match self {
            Kind::Ray(..) => Some(LineMode::Ray),
            Kind::Line(..) => Some(LineMode::Line),
            _ => {
                let definition = self.transform_definition()?;
                if !matches!(
                    definition.wire_flavor(),
                    NativeTransformFlavor::FixedRotation
                        | NativeTransformFlavor::FixedDilation
                        | NativeTransformFlavor::NumericRotation
                        | NativeTransformFlavor::NumericDilation
                ) {
                    return None;
                }
                match definition.element() {
                    ElementType::Ray => Some(LineMode::Ray),
                    ElementType::Line => Some(LineMode::Line),
                    _ => None,
                }
            }
        }
    }
    pub fn is_circle(&self) -> bool {
        matches!(self, Kind::Circle(..) | Kind::CircleBySegment(..))
            || self
                .transform_definition()
                .is_some_and(|definition| definition.element() == ElementType::Circle)
    }
    pub fn is_line(&self) -> bool {
        straight_kind_mode(self) == Some(LineMode::Line)
    }
    pub fn is_polygon(&self) -> bool {
        matches!(self, Kind::Polygon(_))
            || self
                .transform_definition()
                .is_some_and(|definition| definition.element() == ElementType::Polygon)
    }
    pub fn is_measurement(&self) -> bool {
        matches!(
            self,
            Kind::SegmentLength(_)
                | Kind::PointDistance(_, _)
                | Kind::PointLineDistance(_, _)
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
    pub fn has_measurement_precision(&self) -> bool {
        matches!(
            self,
            Kind::SegmentLength(_)
                | Kind::PointDistance(_, _)
                | Kind::PointLineDistance(_, _)
                | Kind::PointRatio(_, _, _)
                | Kind::PolygonPerimeter(_)
                | Kind::CircleCircumference(_)
                | Kind::CircleRadius(_)
                | Kind::CircleArea(_)
                | Kind::PolygonArea(_)
                | Kind::AngleMeasurement(_, _, _)
                | Kind::ArcAngle(_)
                | Kind::ArcLength(_)
                | Kind::SlopeMeasurement(_, _)
                | Kind::CoordinateMeasurement {
                    kind: CoordinateMeasureKind::Abscissa
                        | CoordinateMeasureKind::Ordinate
                        | CoordinateMeasureKind::CoordinateDistance,
                    ..
                }
                | Kind::ValueEquation {
                    kind: ValueEquationKind::PointValue,
                    ..
                }
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
    pub fn animation_button_source_label(&self) -> Option<&'static str> {
        match self {
            Kind::Segment(_, _) => Some("Animate Segment"),
            Kind::Line(_, _) => Some("Animate Line"),
            Kind::Ray(_, _) => Some("Animate Ray"),
            Kind::Circle(_, _) => Some("Animate Circle"),
            Kind::Polygon(vertices) => Some(match vertices.len() {
                3 => "Animate Triangle",
                4 => "Animate Quadrilateral",
                5 => "Animate Pentagon",
                6 => "Animate Hexagon",
                7.. => "Animate Polygon",
                _ => return None,
            }),
            _ => None,
        }
    }
    pub fn dependencies(&self) -> Vec<usize> {
        if let Some(definition) = self.transform_definition() {
            return definition.dependencies();
        }
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
        if let Kind::Locus(refs, _) | Kind::ParameterLocus(refs, _, _, _) = self {
            return refs.clone();
        }
        if let Kind::MeasuredCalculation(source, _) = self {
            return vec![*source];
        }
        if let Kind::LengthPairCalculation(first, second, _) = self {
            return vec![*first, *second];
        }
        if let Kind::ParameterCalculation(source, group) = self {
            return calculation_source_ids(*source, group);
        }
        if let Kind::PointIteration(seed, image, depth) = self {
            let mut parents = vec![*seed, *image];
            if let IterationDepth::Value(id) = depth {
                parents.push(*id);
            }
            return parents;
        }
        if let Kind::SegmentIteration(seed, image, segment, depth)
        | Kind::CircleIteration(seed, image, segment, depth) = self
        {
            let mut parents = vec![*seed, *image, *segment];
            if let IterationDepth::Value(id) = depth {
                parents.push(*id);
            }
            return parents;
        }
        if let Kind::RayIteration(seed, image, ray, parameter) = self {
            return vec![*parameter, *seed, *image, *ray];
        }
        if let Kind::MappedPointIteration {
            seeds,
            branches,
            carried,
            depth,
            ..
        } = self
        {
            let mut parents = seeds.clone();
            parents.extend(branches.iter().flatten().copied());
            parents.extend(carried);
            if let IterationDepth::Value(id) = depth {
                parents.push(*id);
            }
            return parents;
        }
        if let Kind::BranchingPolygonIteration {
            seed,
            first,
            second,
            polygon,
            segments,
            depth,
        } = self
        {
            let mut parents = vec![*seed, *first, *second, *polygon];
            parents.extend(segments);
            if let IterationDepth::Value(id) = depth {
                parents.push(*id);
            }
            return parents;
        }
        if let Kind::PointParameterIteration(seed, parameter, image, calculation, label, depth) =
            self
        {
            let mut parents = vec![*seed, *parameter, *image, *calculation, *label];
            if let IterationDepth::Value(id) = depth {
                parents.push(*id);
            }
            return parents;
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
            | Kind::PointOnRay(host, _)
            | Kind::PointOnLine(host, _)
            | Kind::ValueTable(host, _)
            | Kind::PointOnCircle(host, _, _)
            | Kind::PointOnArc(host, _)
            | Kind::PointOnPolygon(host, _, _)
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
            | Kind::PointNumericLabel(a, b)
            | Kind::IterationValues(a, b)
            | Kind::IterationBinding(a, b)
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
            | Kind::PointDistance(a, b)
            | Kind::PointLineDistance(a, b) => vec![a, b],
            Kind::GraphAxis(a, b, _)
            | Kind::DistanceAxis(a, b, _)
            | Kind::CoordinateSystem(a, b)
            | Kind::SlopeMeasurement(a, b)
            | Kind::FunctionPlot(a, b, _, _, _)
            | Kind::PathValuePoint(a, b)
            | Kind::PlottedPoint(a, b, _, _) => vec![a, b],
            Kind::ParametricPlot(x, y, frame, _, _, _, _)
            | Kind::NumericCoordinatePoint(x, y, frame) => vec![x, y, frame],
            Kind::DerivativeFunction(source, _) => vec![source],
            Kind::Bisector(a, b, c)
            | Kind::ThreePointArc(a, b, c)
            | Kind::ArcOnCircle(a, b, c)
            | Kind::AngleMeasurement(a, b, c) => {
                vec![a, b, c]
            }
            Kind::PointRatio(a, b, c) => vec![a, b, c],
            Kind::Polygon(_)
            | Kind::BranchingPolygonIteration { .. }
            | Kind::MappedPointIteration { .. } => unreachable!(),
            Kind::SegmentIteration(_, _, _, _)
            | Kind::CircleIteration(_, _, _, _)
            | Kind::PointIteration(_, _, _) => unreachable!(),
            Kind::RayIteration(_, _, _, _) => unreachable!(),
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
            Kind::Locus(_, _)
            | Kind::ParameterLocus(_, _, _, _)
            | Kind::MeasuredCalculation(_, _)
            | Kind::LengthPairCalculation(_, _, _)
            | Kind::PointParameterIteration(_, _, _, _, _, _)
            | Kind::ParameterCalculation(_, _) => unreachable!(),
            Kind::Transformed(..) => unreachable!("transform dependencies handled above"),
        }
    }
    pub(super) fn remap(&self, ids: &[usize]) -> Self {
        let m = |id: usize| ids[id];
        if let Kind::MappedPointIteration {
            seeds,
            branches,
            carried,
            depth,
            final_only,
        } = self
        {
            return Kind::MappedPointIteration {
                seeds: seeds.iter().map(|id| m(*id)).collect(),
                branches: branches
                    .iter()
                    .map(|branch| branch.iter().map(|id| m(*id)).collect())
                    .collect(),
                carried: carried.iter().map(|id| m(*id)).collect(),
                depth: match depth {
                    IterationDepth::Fixed(n) => IterationDepth::Fixed(*n),
                    IterationDepth::Value(id) => IterationDepth::Value(m(*id)),
                },
                final_only: *final_only,
            };
        }
        if let Kind::BranchingPolygonIteration {
            seed,
            first,
            second,
            polygon,
            segments,
            depth,
        } = self
        {
            return Kind::BranchingPolygonIteration {
                seed: m(*seed),
                first: m(*first),
                second: m(*second),
                polygon: m(*polygon),
                segments: segments.iter().map(|id| m(*id)).collect(),
                depth: match depth {
                    IterationDepth::Fixed(value) => IterationDepth::Fixed(*value),
                    IterationDepth::Value(id) => IterationDepth::Value(m(*id)),
                },
            };
        }
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
            Kind::MeasuredCalculation(source, ref group) => {
                Kind::MeasuredCalculation(m(source), group.clone())
            }
            Kind::LengthPairCalculation(first, second, ref group) => {
                Kind::LengthPairCalculation(m(first), m(second), group.clone())
            }
            Kind::ParameterCalculation(source, ref group) => {
                Kind::ParameterCalculation(m(source), group.clone())
            }
            Kind::ValueTable(source, ref group) => Kind::ValueTable(m(source), group.clone()),
            Kind::ValueTablePair(first, second, ref group) => {
                Kind::ValueTablePair(m(first), m(second), group.clone())
            }
            Kind::PointOnSegment(a, t) => Kind::PointOnSegment(m(a), t),
            Kind::PointOnRay(a, t) => Kind::PointOnRay(m(a), t),
            Kind::PointOnLine(a, t) => Kind::PointOnLine(m(a), t),
            Kind::PointOnCircle(a, x, y) => Kind::PointOnCircle(m(a), x, y),
            Kind::PointOnArc(a, t) => Kind::PointOnArc(m(a), t),
            Kind::PointOnPolygon(a, edge, t) => Kind::PointOnPolygon(m(a), edge, t),
            Kind::Locus(ref refs, samples) => {
                Kind::Locus(refs.iter().map(|id| m(*id)).collect(), samples)
            }
            Kind::ParameterLocus(ref refs, min, max, samples) => {
                Kind::ParameterLocus(refs.iter().map(|id| m(*id)).collect(), min, max, samples)
            }
            Kind::PointIteration(seed, image, depth) => Kind::PointIteration(
                m(seed),
                m(image),
                match depth {
                    IterationDepth::Fixed(n) => IterationDepth::Fixed(n),
                    IterationDepth::Value(id) => IterationDepth::Value(m(id)),
                },
            ),
            Kind::PointNumericLabel(point, value) => Kind::PointNumericLabel(m(point), m(value)),
            Kind::PointParameterIteration(seed, parameter, image, calculation, label, depth) => {
                Kind::PointParameterIteration(
                    m(seed),
                    m(parameter),
                    m(image),
                    m(calculation),
                    m(label),
                    match depth {
                        IterationDepth::Fixed(value) => IterationDepth::Fixed(value),
                        IterationDepth::Value(id) => IterationDepth::Value(m(id)),
                    },
                )
            }
            Kind::IterationValues(iteration, calculation) => {
                Kind::IterationValues(m(iteration), m(calculation))
            }
            Kind::SegmentIteration(seed, image, segment, depth) => Kind::SegmentIteration(
                m(seed),
                m(image),
                m(segment),
                match depth {
                    IterationDepth::Fixed(value) => IterationDepth::Fixed(value),
                    IterationDepth::Value(id) => IterationDepth::Value(m(id)),
                },
            ),
            Kind::CircleIteration(seed, image, circle, depth) => Kind::CircleIteration(
                m(seed),
                m(image),
                m(circle),
                match depth {
                    IterationDepth::Fixed(n) => IterationDepth::Fixed(n),
                    IterationDepth::Value(id) => IterationDepth::Value(m(id)),
                },
            ),
            Kind::RayIteration(seed, image, ray, parameter) => {
                Kind::RayIteration(m(seed), m(image), m(ray), m(parameter))
            }
            Kind::IterationBinding(image, iteration) => {
                Kind::IterationBinding(m(image), m(iteration))
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
            Kind::NumericCoordinatePoint(x, y, frame) => {
                Kind::NumericCoordinatePoint(m(x), m(y), m(frame))
            }
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
            Kind::PointLineDistance(a, b) => Kind::PointLineDistance(m(a), m(b)),
            Kind::PointRatio(a, b, c) => Kind::PointRatio(m(a), m(b), m(c)),
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
            Kind::Polygon(_)
            | Kind::BranchingPolygonIteration { .. }
            | Kind::MappedPointIteration { .. } => unreachable!(),
            Kind::HideShowButton { .. }
            | Kind::AnimationButton { .. }
            | Kind::MovementButton { .. }
            | Kind::ScrollButton { .. }
            | Kind::PresentationButton { .. } => unreachable!(),
            Kind::LinkUrlButton { .. } => unreachable!(),
            Kind::LinkPageButton { .. } => unreachable!(),
            Kind::CoordinateMeasurement { .. } => unreachable!(),
            Kind::ValueEquation { .. } => unreachable!(),
            Kind::Transformed(ref definition) => Kind::Transformed(definition.remap_unchecked(ids)),
        }
    }
    pub(super) fn remap_checked(&self, ids: &[usize]) -> Result<Self, String> {
        let mut remapped = self.remap(ids);
        match (&mut remapped, self) {
            (Kind::ValueTable(_, group), Kind::ValueTable(_, original))
            | (Kind::ValueTablePair(_, _, group), Kind::ValueTablePair(_, _, original)) => {
                *group = original.remap_sources(ids)?;
            }
            (Kind::MeasuredCalculation(source, group), Kind::MeasuredCalculation(_, original)) => {
                *group = original
                    .remap_source(u32::try_from(*source).map_err(|_| "Invalid measurement ID")?)?;
            }
            (
                Kind::LengthPairCalculation(first, second, group),
                Kind::LengthPairCalculation(_, _, original),
            ) => {
                *group = original.remap_sources(*first as u32, *second as u32)?;
            }
            (Kind::ParameterCalculation(_, group), Kind::ParameterCalculation(first, original)) => {
                let mapped = calculation_source_ids(*first, original)
                    .into_iter()
                    .map(|id| u32::try_from(ids[id]).map_err(|_| "Invalid calculation source ID"))
                    .collect::<Result<Vec<_>, _>>()?;
                *group = original.remap_sources(&mapped)?;
            }
            _ => {}
        }
        Ok(remapped)
    }
}

pub(super) fn is_multi_page_addition_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::PlainText { .. }
            | Kind::Free(_)
            | Kind::PointOnSegment(_, _)
            | Kind::PointOnRay(_, _)
            | Kind::PointOnLine(_, _)
            | Kind::PointOnCircle(_, _, _)
            | Kind::PointOnArc(_, _)
            | Kind::PointOnPolygon(_, _, _)
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
            | Kind::Bisector(_, _, _)
            | Kind::ThreePointArc(_, _, _)
            | Kind::ArcOnCircle(_, _, _)
            | Kind::Polygon(_)
            | Kind::SegmentLength(_)
            | Kind::PolygonPerimeter(_)
            | Kind::CircleCircumference(_)
            | Kind::CircleRadius(_)
            | Kind::CircleArea(_)
            | Kind::PolygonArea(_)
            | Kind::ArcAngle(_)
            | Kind::ArcLength(_)
            | Kind::PointDistance(_, _)
            | Kind::PointLineDistance(_, _)
            | Kind::AngleMeasurement(_, _, _)
            | Kind::PointRatio(_, _, _)
            | Kind::HideShowButton { .. }
            | Kind::AnimationButton { .. }
            | Kind::MovementButton { .. }
            | Kind::ScrollButton { .. }
            | Kind::PresentationButton { .. }
            | Kind::LinkPageButton { .. }
            | Kind::LinkUrlButton { .. }
    )
}
pub(super) fn is_multi_page_deletion_kind(kind: &Kind) -> bool {
    is_multi_page_addition_kind(kind)
        && !matches!(
            kind,
            Kind::LinkPageButton { .. } | Kind::LinkUrlButton { .. }
        )
}
pub(super) fn is_segment_kind(kind: &Kind) -> bool {
    matches!(kind, Kind::Segment(..))
        || kind
            .transform_definition()
            .is_some_and(|definition| definition.element() == ElementType::Segment)
}

pub(super) fn is_straight_kind(kind: &Kind) -> bool {
    straight_kind_mode(kind).is_some()
}

pub(super) fn straight_kind_mode(kind: &Kind) -> Option<LineMode> {
    if let Some(definition) = kind.transform_definition() {
        return match definition.element() {
            ElementType::Segment => Some(LineMode::Segment),
            ElementType::Ray => Some(LineMode::Ray),
            ElementType::Line => Some(LineMode::Line),
            _ => None,
        };
    }
    match kind {
        Kind::Segment(..) => Some(LineMode::Segment),
        Kind::Ray(..) | Kind::Bisector(..) => Some(LineMode::Ray),
        Kind::Line(..)
        | Kind::Parallel(..)
        | Kind::Perpendicular(..)
        | Kind::GraphAxis(..)
        | Kind::CircleAxis(..)
        | Kind::DistanceAxis(..) => Some(LineMode::Line),
        _ => None,
    }
}

pub(super) fn is_point_kind(kind: &Kind) -> bool {
    kind.transform_definition()
        .is_some_and(|definition| definition.element() == ElementType::Point)
        || matches!(
            kind,
            Kind::Free(_)
                | Kind::PointOnSegment(_, _)
                | Kind::PointOnRay(_, _)
                | Kind::PointOnLine(_, _)
                | Kind::PointOnCircle(_, _, _)
                | Kind::PointOnArc(_, _)
                | Kind::PointOnPolygon(_, _, _)
                | Kind::GraphHorizontalUnit(_, _)
                | Kind::GraphVerticalUnit(_, _)
                | Kind::DistanceVerticalUnit(_, _, _)
                | Kind::PathLiteralPoint(_, _)
                | Kind::PathValuePoint(_, _)
                | Kind::PlottedPoint(_, _, _, _)
                | Kind::NumericCoordinatePoint(_, _, _)
                | Kind::Midpoint(_, _)
                | Kind::Intersection(_, _)
                | Kind::LineCircleIntersection(_, _, _)
                | Kind::CircleCircleIntersection(_, _, _)
        )
}

pub(super) fn expected_parameter_locus_refs(objects: &[Object], driver: usize, target: usize) -> Option<Vec<usize>> {
    fn dependent_path(objects: &[Object], driver: usize, current: usize, page: usize, found: &mut BTreeSet<usize>, visited: &mut BTreeSet<usize>) -> Option<bool> {
        let object = objects.get(current.checked_sub(1)?)?;
        if object.deleted || object.page != page { return None; }
        if current == driver { return Some(true); }
        if !visited.insert(current) { return Some(found.contains(&current)); }
        let mut dependent = false;
        for parent in object.kind.dependencies() {
            if parent >= current { return None; }
            dependent |= dependent_path(objects, driver, parent, page, found, visited)?;
        }
        if dependent { found.insert(current); }
        Some(dependent)
    }
    let moving = objects.get(driver.checked_sub(1)?)?;
    let traced = objects.get(target.checked_sub(1)?)?;
    if driver >= target || moving.deleted || traced.deleted || moving.page != traced.page
        || !matches!(&moving.kind, Kind::Parameter(group) if group.locus_unit_tag().is_some())
        || !is_point_kind(&traced.kind)
    { return None; }
    let mut found = BTreeSet::new();
    if !dependent_path(objects, driver, target, moving.page, &mut found, &mut BTreeSet::new())? { return None; }
    let mut refs = vec![target, driver];
    refs.extend(found);
    Some(refs)
}

pub(super) fn expected_locus_refs(
    objects: &[Object],
    driver: usize,
    target: usize,
) -> Option<Vec<usize>> {
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
            || object.kind.is_circle()
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
            || traced.kind.is_circle()
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
        Kind::PointOnCircle(path, _, _) if objects.get(path.checked_sub(1)?)?.kind.is_circle() => {
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

pub(super) fn is_arc_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::ThreePointArc(_, _, _) | Kind::ArcOnCircle(_, _, _)
    )
}

pub(super) fn is_locus_polygon_kind(kind: &Kind) -> bool {
    matches!(kind, Kind::Polygon(_))
        || kind.transform_definition().is_some_and(|definition| {
            definition.element() == ElementType::Polygon
                && matches!(
                    definition.wire_flavor(),
                    NativeTransformFlavor::Vector
                        | NativeTransformFlavor::FixedPolar
                        | NativeTransformFlavor::FixedCartesian
                )
        })
}

pub(super) fn is_locus_straight_kind(kind: &Kind) -> bool {
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

pub(super) fn is_defining_distance_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Segment(_, _) | Kind::SegmentLength(_) | Kind::PointDistance(_, _)
    )
}

pub(super) fn is_plot_path_kind(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Segment(_, _)
            | Kind::Ray(_, _)
            | Kind::Line(_, _)
            | Kind::GraphAxis(_, _, 0 | 1 | 0x1_0001)
            | Kind::CircleAxis(_, 0 | 1)
            | Kind::DistanceAxis(_, _, 0 | 1)
    ) || kind.is_circle() || is_arc_kind(kind) || is_transformed_plot_path_kind(kind)
}

pub(super) fn is_transformed_plot_ray_kind(kind: &Kind) -> bool {
    kind.transform_definition().is_some_and(|definition| {
        definition.element() == ElementType::Ray
            && matches!(
                definition.wire_flavor(),
                NativeTransformFlavor::FixedRotation | NativeTransformFlavor::FixedDilation
            )
    })
}

pub(super) fn is_transformed_plot_path_kind(kind: &Kind) -> bool {
    kind.transform_definition().is_some_and(|definition| {
        matches!(definition.element(), ElementType::Ray | ElementType::Line)
            && matches!(
                definition.wire_flavor(),
                NativeTransformFlavor::FixedRotation | NativeTransformFlavor::FixedDilation
            )
    })
}

pub(super) fn plot_path_accepts(kind: &Kind, value: f64) -> bool {
    value.is_finite()
        && is_plot_path_kind(kind)
        && if matches!(
            kind,
            Kind::Segment(..) | Kind::ThreePointArc(..) | Kind::ArcOnCircle(..)
        ) {
            (0.0..=1.0).contains(&value)
        } else if matches!(kind, Kind::Ray(..)) || is_transformed_plot_ray_kind(kind) {
            value >= 0.0
        } else {
            true
        }
}

/// Code 95 polygon boundaries use independently verified perimeter semantics.
pub(super) fn is_value_plot_path_kind(kind: &Kind) -> bool {
    is_plot_path_kind(kind) || kind.is_polygon()
}

pub(super) fn value_plot_path_accepts(kind: &Kind, value: f64) -> bool {
    plot_path_accepts(kind, value) || ((kind.is_polygon() || is_transformed_plot_ray_kind(kind) || is_arc_kind(kind)) && value.is_finite())
}

/// Native code 123 also accepts a literal fraction of a polygon perimeter.
pub(super) fn is_literal_plot_path_kind(kind: &Kind) -> bool {
    is_plot_path_kind(kind) || kind.is_polygon()
}

pub(super) fn literal_plot_path_accepts(kind: &Kind, value: f64) -> bool {
    plot_path_accepts(kind, value) || ((kind.is_polygon() || is_transformed_plot_ray_kind(kind) || is_arc_kind(kind)) && value.is_finite())
}

/// Supported polygon transforms retain their source's vertex count. Require
/// earlier, same-page sources so malformed chains cannot loop or cross pages.
pub(super) fn polygon_vertex_count(objects: &[Object], id: usize) -> Option<usize> {
    polygon_vertex_count_impl(objects, id, true)
}

/// Writers emit groups before deletion compaction. Recover
/// their schema even when the complete dependency chain is being removed.
pub(super) fn polygon_vertex_count_for_encoding(objects: &[Object], id: usize) -> Option<usize> {
    polygon_vertex_count_impl(objects, id, false)
}

fn polygon_vertex_count_impl(
    objects: &[Object],
    mut id: usize,
    require_live: bool,
) -> Option<usize> {
    let page = objects.get(id.checked_sub(1)?)?.page;
    loop {
        let object = objects.get(id.checked_sub(1)?)?;
        if (require_live && object.deleted) || object.id != id || object.page != page {
            return None;
        }
        let source = match &object.kind {
            Kind::Polygon(vertices) => {
                if vertices.len() < 3 {
                    return None;
                }
                super::records::polygon_subtype(vertices.len())?;
                return Some(vertices.len());
            }
            _ => object
                .kind
                .transform_definition()
                .filter(|definition| definition.element() == ElementType::Polygon)?
                .source(),
        };
        if source == 0 || source >= id {
            return None;
        }
        id = source;
    }
}

pub(super) fn default_visual_style(kind: &Kind) -> Option<VisualStyle> {
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
    if let Some(definition) = kind.transform_definition() {
        return Some(match definition.element() {
            ElementType::Point => VisualStyle::Point {
                shape: PointStyle::Medium,
                color: red,
            },
            ElementType::Segment | ElementType::Ray | ElementType::Line => {
                VisualStyle::Stroke { line, color: navy }
            }
            ElementType::Circle => VisualStyle::Stroke { line, color: green },
            ElementType::Polygon => VisualStyle::Polygon {
                opacity: 32767,
                color: RgbColor {
                    red: 255,
                    green: 128,
                    blue: 0,
                },
            },
        });
    }
    match kind {
        Kind::Free(_)
        | Kind::PointOnSegment(_, _)
        | Kind::PointOnRay(_, _)
        | Kind::PointOnLine(_, _)
        | Kind::PointOnCircle(_, _, _)
        | Kind::PointOnArc(_, _)
        | Kind::PointOnPolygon(_, _, _)
        | Kind::PathLiteralPoint(_, _)
        | Kind::PathValuePoint(_, _)
        | Kind::NumericCoordinatePoint(_, _, _)
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
        | Kind::ThreePointArc(_, _, _)
        | Kind::ArcOnCircle(_, _, _) => Some(VisualStyle::Stroke { line, color: green }),
        Kind::FunctionPlot(_, _, _, _, _) => Some(VisualStyle::Stroke {
            line: SegmentLineStyle {
                width: LineWidth::Thin,
                pattern: LinePattern::Solid,
            },
            color: RgbColor {
                red: 128,
                green: 0,
                blue: 128,
            },
        }),
        Kind::ParametricPlot(_, _, _, _, _, _, _) => Some(VisualStyle::Stroke {
            line,
            color: RgbColor {
                red: 128,
                green: 0,
                blue: 128,
            },
        }),
        Kind::ParameterLocus(_, _, _, _) => Some(point_locus_visual_style()),
        Kind::Polygon(_) => Some(VisualStyle::Polygon {
            opacity: 32767,
            color: RgbColor {
                red: 255,
                green: 128,
                blue: 0,
            },
        }),
        _ => None,
    }
}

/// Class 6/subtype 42 point-locus templates use a medium solid maroon stroke.
pub(super) fn point_locus_visual_style() -> VisualStyle {
    VisualStyle::Stroke { line: SegmentLineStyle { width:LineWidth::Medium, pattern:LinePattern::Solid }, color:RgbColor { red:128,green:0,blue:0 } }
}
