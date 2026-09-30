use super::calculations::calculation_source_ids;
use super::transform_model::ElementType;
use super::{EmbeddedToolInputKind, ImagePlacement, IterationDepth, Kind, LineMode, Object};
use crate::coordinate_measure_records::CoordinateMeasureKind;
use crate::style_records::{
    LinePattern, LineWidth, PointStyle, RgbColor, SegmentLineStyle, VisualStyle,
};
use crate::table_records::NumericUnit;
use crate::value_equation_records::ValueEquationKind;
use std::collections::BTreeSet;

impl Kind {
    pub fn is_arc(&self) -> bool {
        is_arc_kind(self)
    }
    pub fn is_function(&self) -> bool {
        matches!(
            self,
            Self::FunctionDefinition(_)
                | Self::ReferencedFunction(_, _)
                | Self::DerivativeFunction(_, _)
                | Self::DataFunction { .. }
        )
    }

    pub fn is_iteration_depth_value(&self) -> bool {
        self.numeric_unit() == Some(NumericUnit::Scalar)
    }
    pub fn numeric_unit(&self) -> Option<NumericUnit> {
        match self {
            Kind::Calculation(group) => Some(group.unit()),
            Kind::Parameter(group) => Some(group.unit()),
            Kind::SegmentRatio(_, _) | Kind::PointRatio(_, _, _) | Kind::SlopeMeasurement(_, _) => {
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
            | Kind::CircleArcLength(_, _, _)
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
                    | CoordinateMeasureKind::GraphCoordinateX
                    | CoordinateMeasureKind::CoordinateDistance,
                ..
            } => Some(NumericUnit::Scalar),
            Kind::CoordinateMeasurement {
                kind: CoordinateMeasureKind::GraphCoordinateY,
                ..
            } => Some(NumericUnit::Degrees),
            Kind::ParameterCalculation(_, group) => Some(group.unit()),
            Kind::NumericAlias { unit, .. } => Some(*unit),
            Kind::AngleMeasurement(_, _, _)
            | Kind::AngleMarker(_, _, _, _)
            | Kind::SignedAngleValue(_, _, _)
            | Kind::PolarAngleValue(_, _, _)
            | Kind::VertexAngleValue(_, _, _)
            | Kind::ArcAngle(_) => Some(NumericUnit::Degrees),
            _ => None,
        }
    }
    /// A finite segment or a length quantity can define a coordinate unit.
    pub fn is_defining_distance(&self) -> bool {
        self.is_segment() || self.is_distance_value()
    }
    pub fn is_distance_value(&self) -> bool {
        self.numeric_unit() == Some(NumericUnit::Length)
    }
    /// Marked translation interprets scalar values as signed centimeters.
    pub fn is_marked_distance_value(&self) -> bool {
        matches!(
            self.numeric_unit(),
            Some(NumericUnit::Length | NumericUnit::Scalar)
        )
    }
    /// Transform angle and ratio slots consume numeric magnitudes. Distance
    /// slots retain their signed-centimeter convention.
    pub(super) fn is_transform_operand(&self, unit: NumericUnit) -> bool {
        match unit {
            NumericUnit::Scalar | NumericUnit::Degrees => self.numeric_unit().is_some(),
            NumericUnit::Length => self.is_marked_distance_value(),
            _ => false,
        }
    }
    pub fn is_segment(&self) -> bool {
        is_segment_kind(self)
    }
    /// Straight source families supported by native transform schemas.
    pub fn fixed_transform_straight_mode(&self) -> Option<LineMode> {
        match self {
            Kind::Ray(..) | Kind::Bisector(..) => Some(LineMode::Ray),
            Kind::Line(..) | Kind::Parallel(..) | Kind::Perpendicular(..) => Some(LineMode::Line),
            _ => {
                let definition = self.transform_definition()?;
                match definition.element() {
                    ElementType::Ray => Some(LineMode::Ray),
                    ElementType::Line => Some(LineMode::Line),
                    _ => None,
                }
            }
        }
    }
    /// Straight source families proved for native rotation schemas. Native
    /// files also rotate the ordinary x-axis, but do not establish that axes
    /// support the other affine transform opcodes.
    pub fn rotation_transform_straight_mode(&self) -> Option<LineMode> {
        if matches!(self, Kind::GraphAxis(_, _, 1)) {
            Some(LineMode::Line)
        } else {
            self.fixed_transform_straight_mode()
        }
    }
    pub fn is_point(&self) -> bool {
        is_point_kind(self)
    }
    pub fn is_sector(&self) -> bool {
        matches!(self, Kind::SectorBoundary(_))
            || matches!(self, Kind::Colorized(d) if d.element() == ElementType::Sector)
            || self
                .transform_definition()
                .is_some_and(|definition| definition.element() == ElementType::Sector)
    }
    pub fn is_circular_segment(&self) -> bool {
        matches!(self, Kind::CircularSegmentBoundary(_))
            || self
                .transform_definition()
                .is_some_and(|definition| definition.element() == ElementType::CircularSegment)
    }
    pub fn is_filled_region(&self) -> bool {
        self.is_polygon()
            || self.is_sector()
            || self.is_circle_interior()
            || self.is_circular_segment()
    }
    pub fn is_circle_interior(&self) -> bool {
        matches!(self, Kind::CircleInterior(_))
            || matches!(self, Kind::Colorized(d) if d.element() == ElementType::Disk)
            || self
                .transform_definition()
                .is_some_and(|d| d.element() == ElementType::Disk)
    }
    pub fn is_circle(&self) -> bool {
        matches!(self, Kind::Circle(..) | Kind::CircleBySegment(..))
            || matches!(self, Kind::Colorized(d) if d.element() == ElementType::Circle)
            || self
                .transform_definition()
                .is_some_and(|definition| definition.element() == ElementType::Circle)
    }
    pub fn is_line(&self) -> bool {
        straight_kind_mode(self) == Some(LineMode::Line)
    }
    pub fn is_polygon(&self) -> bool {
        matches!(self, Kind::Polygon(_))
            || matches!(self, Kind::Colorized(d) if d.element() == ElementType::Polygon)
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
                | Kind::SegmentRatio(_, _)
                | Kind::PointRatio(_, _, _)
                | Kind::PolygonPerimeter(_)
                | Kind::CircleCircumference(_)
                | Kind::CircleRadius(_)
                | Kind::CircleArea(_)
                | Kind::PolygonArea(_)
                | Kind::AngleMeasurement(_, _, _)
                | Kind::NumericAlias { .. }
                | Kind::SlopeMeasurement(_, _)
                | Kind::CoordinateMeasurement { .. }
                | Kind::ValueEquation { .. }
                | Kind::ArcAngle(_)
                | Kind::ArcLength(_)
                | Kind::CircleArcLength(_, _, _)
        )
    }
    pub fn has_measurement_precision(&self) -> bool {
        matches!(
            self,
            Kind::SegmentLength(_)
                | Kind::PointDistance(_, _)
                | Kind::PointLineDistance(_, _)
                | Kind::SegmentRatio(_, _)
                | Kind::PointRatio(_, _, _)
                | Kind::PolygonPerimeter(_)
                | Kind::CircleCircumference(_)
                | Kind::CircleRadius(_)
                | Kind::CircleArea(_)
                | Kind::PolygonArea(_)
                | Kind::AngleMeasurement(_, _, _)
                | Kind::NumericAlias { .. }
                | Kind::ArcAngle(_)
                | Kind::ArcLength(_)
                | Kind::CircleArcLength(_, _, _)
                | Kind::SlopeMeasurement(_, _)
                | Kind::CoordinateMeasurement {
                    kind: CoordinateMeasureKind::Abscissa
                        | CoordinateMeasureKind::Ordinate
                        | CoordinateMeasureKind::GraphCoordinateX
                        | CoordinateMeasureKind::GraphCoordinateY
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
                | Kind::SoundButton { .. }
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
        if let Kind::CroppedImage { source, polygon } = self {
            return vec![*source, *polygon];
        }
        if let Kind::Image { placement, .. } = self {
            return match placement {
                ImagePlacement::Independent(_) => Vec::new(),
                ImagePlacement::Point { anchor, .. } => vec![*anchor],
                ImagePlacement::Rectangle {
                    bottom_right,
                    top_left,
                } => vec![*bottom_right, *top_left],
                ImagePlacement::Parallelogram {
                    top_left,
                    bottom_right,
                    bottom_left,
                } => vec![*top_left, *bottom_right, *bottom_left],
                ImagePlacement::Transformed(definition) => definition.dependencies(),
            };
        }
        if let Kind::CoordinatePoint(d) = self {
            return d.native_references();
        }
        if let Kind::ReferencedText { parents, .. } = self {
            return parents.clone();
        }
        if let Kind::Colorized(definition) = self {
            return definition.native_references();
        }
        if let Kind::MappedElement(definition) = self {
            return definition.native_references();
        }
        if let Kind::SampledMapped(definition) = self {
            return definition.native_references();
        }
        if let Kind::SampledImage(definition) = self {
            return definition.native_references();
        }
        if let Some(definition) = self.transform_definition() {
            return definition.dependencies();
        }
        if let Kind::HideShowButton { targets, .. } = self {
            return targets.clone();
        }
        if let Kind::AnimationButton { targets, .. } = self {
            return targets.clone();
        }
        if let Kind::SoundButton { functions, .. } = self {
            return functions.clone();
        }
        if let Kind::MovementButton { pairs, .. } = self {
            return pairs
                .iter()
                .flat_map(|&(source, destination)| [source, destination])
                .collect();
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
        if let Kind::DataTable(sources, _) = self {
            return sources.clone();
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
            Kind::ToolInput(_)
            | Kind::Free(_)
            | Kind::Image { .. }
            | Kind::PlainText { .. }
            | Kind::Parameter(_)
            | Kind::Calculation(_)
            | Kind::FunctionDefinition(_)
            | Kind::Unsupported(_) => vec![],
            Kind::IterationPointAlias(host)
            | Kind::ToolPointOnPath(host)
            | Kind::SegmentMidpoint(host)
            | Kind::PointOnSegment(host, _)
            | Kind::PointOnRay(host, _)
            | Kind::PointOnLine(host, _)
            | Kind::ValueTable(host, _)
            | Kind::PointOnCircle(host, _, _)
            | Kind::PointOnArc(host, _)
            | Kind::SectorBoundary(host)
            | Kind::CircularSegmentBoundary(host)
            | Kind::CircleInterior(host)
            | Kind::PointOnSector(host, _)
            | Kind::PointOnCircularSegment(host, _)
            | Kind::PointOnCurve(host, _)
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
            | Kind::SegmentMarker { host, .. }
            | Kind::SegmentArrow { host, .. }
            | Kind::NumericAlias { source: host, .. }
            | Kind::CircleAxis(host, _) => vec![host],
            Kind::CircleArcLength(circle, start, end) => vec![circle, start, end],
            Kind::DistanceVerticalUnit(host, _, _) => vec![host],
            Kind::GraphHorizontalUnit(origin, _) | Kind::GraphVerticalUnit(origin, _) => {
                vec![origin]
            }
            Kind::IterationValues(iteration, ref values) => {
                let mut parents = vec![iteration];
                parents.extend(values.iter().copied());
                parents
            }
            Kind::Midpoint(a, b)
            | Kind::PointNumericLabel(a, b)
            | Kind::IterationBinding(a, b)
            | Kind::Segment(a, b)
            | Kind::Line(a, b)
            | Kind::Ray(a, b)
            | Kind::Parallel(a, b)
            | Kind::Perpendicular(a, b)
            | Kind::Circle(a, b)
            | Kind::CircleBySegment(a, b)
            | Kind::Intersection(a, b)
            | Kind::CurveIntersection(a, b, _)
            | Kind::BoundaryIntersection(a, b)
            | Kind::LineCircleIntersection(a, b, _)
            | Kind::CircleCircleIntersection(a, b, _)
            | Kind::PointDistance(a, b)
            | Kind::PointLineDistance(a, b)
            | Kind::SegmentRatio(a, b) => vec![a, b],
            Kind::GraphAxis(a, b, _)
            | Kind::DistanceAxis(a, b, _)
            | Kind::CoordinateSystem(a, b)
            | Kind::SlopeMeasurement(a, b)
            | Kind::FunctionPlot(a, b, _, _, _, _)
            | Kind::PathValuePoint(a, b) => vec![a, b],
            Kind::DataFunction { source, frame, .. } => vec![source, frame],
            Kind::PlottedPoint(frame, _, _) => vec![frame],
            Kind::ParametricPlot(x, y, frame, _, _, _, _)
            | Kind::NumericCoordinatePoint(x, y, frame, _) => vec![x, y, frame],
            Kind::GraphValuePoint(value, frame) => vec![value, frame],
            Kind::ReferencedFunction(ref sources, _) => sources.clone(),
            Kind::DerivativeFunction(ref sources, _) => sources.clone(),
            Kind::Bisector(a, b, c)
            | Kind::ThreePointArc(a, b, c)
            | Kind::ArcOnCircle(a, b, c)
            | Kind::CenterArc(a, b, c)
            | Kind::AngleMeasurement(a, b, c)
            | Kind::SignedAngleValue(a, b, c)
            | Kind::PolarAngleValue(a, b, c)
            | Kind::VertexAngleValue(a, b, c) => {
                vec![a, b, c]
            }
            Kind::AngleMarker(a, b, c, _) => vec![a, b, c],
            Kind::PointRatio(a, b, c) => vec![a, b, c],
            Kind::PointReferencedText { point, source, .. } => vec![point, source],
            Kind::Polygon(_)
            | Kind::BranchingPolygonIteration { .. }
            | Kind::MappedPointIteration { .. } => unreachable!(),
            Kind::SegmentIteration(_, _, _, _)
            | Kind::CircleIteration(_, _, _, _)
            | Kind::PointIteration(_, _, _) => unreachable!(),
            Kind::RayIteration(_, _, _, _) => unreachable!(),
            Kind::HideShowButton { .. }
            | Kind::AnimationButton { .. }
            | Kind::SoundButton { .. }
            | Kind::MovementButton { .. }
            | Kind::ScrollButton { .. }
            | Kind::PresentationButton { .. } => unreachable!(),
            Kind::LinkUrlButton { .. } => vec![],
            Kind::LinkPageButton { .. } => vec![],
            Kind::CoordinateMeasurement { .. } => unreachable!(),
            Kind::ValueEquation { .. } => unreachable!(),
            Kind::ValueTablePair(_, _, _) | Kind::DataTable(_, _) => unreachable!(),
            Kind::Locus(_, _)
            | Kind::ParameterLocus(_, _, _, _)
            | Kind::MeasuredCalculation(_, _)
            | Kind::LengthPairCalculation(_, _, _)
            | Kind::PointParameterIteration(_, _, _, _, _, _)
            | Kind::ParameterCalculation(_, _) => unreachable!(),
            Kind::CoordinatePoint(..)
            | Kind::CroppedImage { .. }
            | Kind::TransformedCroppedImage(..)
            | Kind::Transformed(..)
            | Kind::MappedElement(..)
            | Kind::SampledMapped(..)
            | Kind::SampledImage(..)
            | Kind::Colorized(..)
            | Kind::ReferencedText { .. } => unreachable!("mapped dependencies handled above"),
        }
    }
    pub(super) fn remap(&self, ids: &[usize]) -> Self {
        let m = |id: usize| ids[id];
        if let Kind::CoordinatePoint(d) = self {
            return Kind::CoordinatePoint(d.remap(ids));
        }
        if let Kind::Colorized(definition) = self {
            return Kind::Colorized(definition.remap(ids));
        }
        if let Kind::MappedElement(definition) = self {
            return Kind::MappedElement(definition.remap(ids));
        }
        if let Kind::SampledMapped(definition) = self {
            return Kind::SampledMapped(definition.remap(ids));
        }
        if let Kind::SampledImage(definition) = self {
            return Kind::SampledImage(definition.remap(ids));
        }
        if let Kind::TransformedCroppedImage(definition) = self {
            return Kind::TransformedCroppedImage(definition.remap_unchecked(ids));
        }
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
            options,
            caption_flags,
            label,
            anchor,
        } = self
        {
            return Kind::HideShowButton {
                targets: targets.iter().map(|&target| m(target)).collect(),
                action: *action,
                options: *options,
                caption_flags: *caption_flags,
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
        if let Kind::SoundButton {
            functions,
            label,
            anchor,
        } = self
        {
            return Kind::SoundButton {
                functions: functions.iter().map(|&function| m(function)).collect(),
                label: label.clone(),
                anchor: *anchor,
            };
        }
        if let Kind::MovementButton {
            pairs,
            speed,
            follow,
            label,
            anchor,
        } = self
        {
            return Kind::MovementButton {
                pairs: pairs
                    .iter()
                    .map(|&(source, destination)| (m(source), m(destination)))
                    .collect(),
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
            options,
            label,
            anchor,
        } = self
        {
            return Kind::PresentationButton {
                targets: targets.iter().map(|&target| m(target)).collect(),
                mode: *mode,
                options: *options,
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
        if let Kind::Image { png, placement } = self {
            return Kind::Image {
                png: png.clone(),
                placement: match placement {
                    ImagePlacement::Independent(transform) => {
                        ImagePlacement::Independent(*transform)
                    }
                    ImagePlacement::Point { anchor, relative } => ImagePlacement::Point {
                        anchor: m(*anchor),
                        relative: *relative,
                    },
                    ImagePlacement::Rectangle {
                        bottom_right,
                        top_left,
                    } => ImagePlacement::Rectangle {
                        bottom_right: m(*bottom_right),
                        top_left: m(*top_left),
                    },
                    ImagePlacement::Parallelogram {
                        top_left,
                        bottom_right,
                        bottom_left,
                    } => ImagePlacement::Parallelogram {
                        top_left: m(*top_left),
                        bottom_right: m(*bottom_right),
                        bottom_left: m(*bottom_left),
                    },
                    ImagePlacement::Transformed(definition) => {
                        ImagePlacement::Transformed(definition.remap_unchecked(ids))
                    }
                },
            };
        }
        match *self {
            Kind::Free(p) => Kind::Free(p),
            Kind::CroppedImage { source, polygon } => Kind::CroppedImage {
                source: m(source),
                polygon: m(polygon),
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
            Kind::DataTable(ref sources, ref group) => {
                Kind::DataTable(sources.iter().map(|id| m(*id)).collect(), group.clone())
            }
            Kind::ToolPointOnPath(host) => Kind::ToolPointOnPath(m(host)),
            Kind::SegmentMidpoint(host) => Kind::SegmentMidpoint(m(host)),
            Kind::PointOnSegment(a, t) => Kind::PointOnSegment(m(a), t),
            Kind::PointOnRay(a, t) => Kind::PointOnRay(m(a), t),
            Kind::PointOnLine(a, t) => Kind::PointOnLine(m(a), t),
            Kind::PointOnCircle(a, x, y) => Kind::PointOnCircle(m(a), x, y),
            Kind::PointOnArc(a, t) => Kind::PointOnArc(m(a), t),
            Kind::SectorBoundary(a) => Kind::SectorBoundary(m(a)),
            Kind::CircularSegmentBoundary(a) => Kind::CircularSegmentBoundary(m(a)),
            Kind::CircleInterior(a) => Kind::CircleInterior(m(a)),
            Kind::PointOnSector(a, t) => Kind::PointOnSector(m(a), t),
            Kind::PointOnCircularSegment(a, t) => Kind::PointOnCircularSegment(m(a), t),
            Kind::PointOnCurve(a, t) => Kind::PointOnCurve(m(a), t),
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
            Kind::PointReferencedText {
                point,
                source,
                ref text,
            } => Kind::PointReferencedText {
                point: m(point),
                source: m(source),
                text: text.clone(),
            },
            Kind::ReferencedText {
                ref parents,
                ref text,
            } => Kind::ReferencedText {
                parents: parents.iter().map(|id| m(*id)).collect(),
                text: text.clone(),
            },
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
            Kind::IterationValues(iteration, ref values) => {
                Kind::IterationValues(m(iteration), values.iter().map(|id| m(*id)).collect())
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
            Kind::ReferencedFunction(ref sources, ref expr) => {
                Kind::ReferencedFunction(sources.iter().map(|&id| m(id)).collect(), expr.clone())
            }
            Kind::FunctionDefinition(ref expr) => Kind::FunctionDefinition(expr.clone()),
            Kind::DerivativeFunction(ref sources, ref expr) => {
                Kind::DerivativeFunction(sources.iter().map(|&id| m(id)).collect(), expr.clone())
            }
            Kind::DataFunction {
                source,
                frame,
                smoothing,
            } => Kind::DataFunction {
                source: m(source),
                frame: m(frame),
                smoothing,
            },
            Kind::FunctionPlot(a, b, min, max, samples, mode) => {
                Kind::FunctionPlot(m(a), m(b), min, max, samples, mode)
            }
            Kind::ParametricPlot(x, y, frame, min, max, samples, mode) => {
                Kind::ParametricPlot(m(x), m(y), m(frame), min, max, samples, mode)
            }
            Kind::PathLiteralPoint(axis, value) => Kind::PathLiteralPoint(m(axis), value),
            Kind::PathValuePoint(a, b) => Kind::PathValuePoint(m(a), m(b)),
            Kind::PlottedPoint(frame, mode, stored) => Kind::PlottedPoint(m(frame), mode, stored),
            Kind::NumericCoordinatePoint(x, y, frame, mode) => {
                Kind::NumericCoordinatePoint(m(x), m(y), m(frame), mode)
            }
            Kind::GraphValuePoint(value, frame) => Kind::GraphValuePoint(m(value), m(frame)),
            Kind::SegmentLength(a) => Kind::SegmentLength(m(a)),
            Kind::PolygonPerimeter(a) => Kind::PolygonPerimeter(m(a)),
            Kind::CircleCircumference(a) => Kind::CircleCircumference(m(a)),
            Kind::CircleRadius(a) => Kind::CircleRadius(m(a)),
            Kind::CircleArea(a) => Kind::CircleArea(m(a)),
            Kind::PolygonArea(a) => Kind::PolygonArea(m(a)),
            Kind::ArcAngle(a) => Kind::ArcAngle(m(a)),
            Kind::ArcLength(a) => Kind::ArcLength(m(a)),
            Kind::CircleArcLength(circle, start, end) => {
                Kind::CircleArcLength(m(circle), m(start), m(end))
            }
            Kind::AngleMeasurement(a, b, c) => Kind::AngleMeasurement(m(a), m(b), m(c)),
            Kind::SignedAngleValue(a, b, c) => Kind::SignedAngleValue(m(a), m(b), m(c)),
            Kind::PolarAngleValue(a, b, c) => Kind::PolarAngleValue(m(a), m(b), m(c)),
            Kind::VertexAngleValue(a, b, c) => Kind::VertexAngleValue(m(a), m(b), m(c)),
            Kind::AngleMarker(a, b, c, class) => Kind::AngleMarker(m(a), m(b), m(c), class),
            Kind::SegmentMarker {
                host,
                parameter,
                count,
            } => Kind::SegmentMarker {
                host: m(host),
                parameter,
                count,
            },
            Kind::SegmentArrow {
                host,
                parameter,
                forward,
            } => Kind::SegmentArrow {
                host: m(host),
                parameter,
                forward,
            },
            Kind::NumericAlias { source, unit } => Kind::NumericAlias {
                source: m(source),
                unit,
            },
            Kind::PointDistance(a, b) => Kind::PointDistance(m(a), m(b)),
            Kind::PointLineDistance(a, b) => Kind::PointLineDistance(m(a), m(b)),
            Kind::SegmentRatio(a, b) => Kind::SegmentRatio(m(a), m(b)),
            Kind::PointRatio(a, b, c) => Kind::PointRatio(m(a), m(b), m(c)),
            Kind::IterationPointAlias(binding) => Kind::IterationPointAlias(m(binding)),
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
            Kind::CenterArc(a, b, c) => Kind::CenterArc(m(a), m(b), m(c)),
            Kind::Intersection(a, b) => Kind::Intersection(m(a), m(b)),
            Kind::CurveIntersection(a, b, branch) => Kind::CurveIntersection(m(a), m(b), branch),
            Kind::BoundaryIntersection(a, b) => Kind::BoundaryIntersection(m(a), m(b)),
            Kind::LineCircleIntersection(a, b, root) => {
                Kind::LineCircleIntersection(m(a), m(b), root)
            }
            Kind::CircleCircleIntersection(a, b, root) => {
                Kind::CircleCircleIntersection(m(a), m(b), root)
            }
            Kind::Unsupported(code) => Kind::Unsupported(code),
            Kind::Image { .. } => unreachable!(),
            Kind::ToolInput(input) => Kind::ToolInput(input),
            Kind::Polygon(_)
            | Kind::BranchingPolygonIteration { .. }
            | Kind::MappedPointIteration { .. } => unreachable!(),
            Kind::HideShowButton { .. }
            | Kind::AnimationButton { .. }
            | Kind::SoundButton { .. }
            | Kind::MovementButton { .. }
            | Kind::ScrollButton { .. }
            | Kind::PresentationButton { .. } => unreachable!(),
            Kind::LinkUrlButton { .. } => unreachable!(),
            Kind::LinkPageButton { .. } => unreachable!(),
            Kind::CoordinateMeasurement { .. } => unreachable!(),
            Kind::ValueEquation { .. } => unreachable!(),
            Kind::Transformed(ref definition) => Kind::Transformed(definition.remap_unchecked(ids)),
            Kind::CoordinatePoint(..)
            | Kind::TransformedCroppedImage(..)
            | Kind::MappedElement(..)
            | Kind::SampledMapped(..)
            | Kind::SampledImage(..)
            | Kind::Colorized(..) => unreachable!("mapped remapping handled above"),
        }
    }
    pub(super) fn remap_checked(&self, ids: &[usize]) -> Result<Self, String> {
        let mut remapped = self.remap(ids);
        match (&mut remapped, self) {
            (Kind::ValueTable(_, group), Kind::ValueTable(_, original))
            | (Kind::ValueTablePair(_, _, group), Kind::ValueTablePair(_, _, original)) => {
                *group = original.remap_sources(ids)?;
            }
            (Kind::DataTable(_, group), Kind::DataTable(_, original)) => {
                *group = original.remap_general_sources(ids)?;
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
            | Kind::IterationPointAlias(_)
            | Kind::CoordinatePoint(_)
            | Kind::MappedElement(_)
            | Kind::SampledMapped(_)
            | Kind::SampledImage(_)
            | Kind::Colorized(_)
            | Kind::Free(_)
            | Kind::PointOnSegment(_, _)
            | Kind::PointOnRay(_, _)
            | Kind::PointOnLine(_, _)
            | Kind::PointOnCircle(_, _, _)
            | Kind::PointOnArc(_, _)
            | Kind::PointOnSector(_, _)
            | Kind::PointOnCircularSegment(_, _)
            | Kind::PointOnCurve(_, _)
            | Kind::PointOnPolygon(_, _, _)
            | Kind::Segment(_, _)
            | Kind::Circle(_, _)
            | Kind::Line(_, _)
            | Kind::Ray(_, _)
            | Kind::Midpoint(_, _)
            | Kind::SegmentMidpoint(_)
            | Kind::CircleBySegment(_, _)
            | Kind::Parallel(_, _)
            | Kind::Perpendicular(_, _)
            | Kind::Intersection(_, _)
            | Kind::CurveIntersection(_, _, _)
            | Kind::BoundaryIntersection(_, _)
            | Kind::LineCircleIntersection(_, _, _)
            | Kind::CircleCircleIntersection(_, _, _)
            | Kind::Bisector(_, _, _)
            | Kind::ThreePointArc(_, _, _)
            | Kind::ArcOnCircle(_, _, _)
            | Kind::CenterArc(_, _, _)
            | Kind::SectorBoundary(_)
            | Kind::CircularSegmentBoundary(_)
            | Kind::CircleInterior(_)
            | Kind::Polygon(_)
            | Kind::SegmentLength(_)
            | Kind::PolygonPerimeter(_)
            | Kind::CircleCircumference(_)
            | Kind::CircleRadius(_)
            | Kind::CircleArea(_)
            | Kind::PolygonArea(_)
            | Kind::ArcAngle(_)
            | Kind::ArcLength(_)
            | Kind::CircleArcLength(_, _, _)
            | Kind::PointDistance(_, _)
            | Kind::PointLineDistance(_, _)
            | Kind::SegmentRatio(_, _)
            | Kind::AngleMeasurement(_, _, _)
            | Kind::SignedAngleValue(_, _, _)
            | Kind::PolarAngleValue(_, _, _)
            | Kind::VertexAngleValue(_, _, _)
            | Kind::PointRatio(_, _, _)
            | Kind::SegmentMarker { .. }
            | Kind::SegmentArrow { .. }
            | Kind::HideShowButton { .. }
            | Kind::AnimationButton { .. }
            | Kind::SoundButton { .. }
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
    matches!(kind, Kind::Colorized(d) if d.element() == ElementType::Segment)
        || matches!(
            kind,
            Kind::Segment(..) | Kind::ToolInput(EmbeddedToolInputKind::Segment)
        )
        || kind
            .transform_definition()
            .is_some_and(|definition| definition.element() == ElementType::Segment)
}

pub(super) fn is_straight_kind(kind: &Kind) -> bool {
    straight_kind_mode(kind).is_some()
}

/// Native Point on Object uses the coordinate-axis host subtype, independently
/// of its infinite-line geometry. Keep this wire identity out of point kinds.
pub(super) fn straight_point_constraint_subtype(kind: &Kind) -> Option<u8> {
    match kind {
        Kind::GraphAxis(_, _, 0 | 1 | 0x1_0001)
        | Kind::CircleAxis(_, 0 | 1)
        | Kind::DistanceAxis(_, _, 0 | 1) => Some(38),
        _ => straight_kind_mode(kind).map(|mode| match mode {
            LineMode::Segment => 14,
            LineMode::Ray => 13,
            LineMode::Line => 12,
        }),
    }
}

/// The subtype is a host identity, not a flag word. Native also stores derived
/// lines with their construction subtype; the ordinary line packet remains a
/// valid representation of the same parameterization.
pub(super) fn straight_point_constraint_matches(kind: &Kind, payload: &[u8]) -> bool {
    let Some(prefix) = payload.get(..4) else {
        return false;
    };
    if prefix[..2] != [15, 0] {
        return false;
    }
    let subtype = u16::from_le_bytes([prefix[2], prefix[3]]);
    straight_point_constraint_subtype(kind).is_some_and(|expected| subtype == u16::from(expected))
        || matches!(
            (kind, subtype),
            (Kind::Bisector(..), 44) | (Kind::Parallel(..), 46) | (Kind::Perpendicular(..), 47)
        )
}

pub(super) fn straight_kind_mode(kind: &Kind) -> Option<LineMode> {
    if let Kind::Colorized(d) = kind {
        return match d.element() {
            ElementType::Segment => Some(LineMode::Segment),
            ElementType::Ray => Some(LineMode::Ray),
            ElementType::Line => Some(LineMode::Line),
            _ => None,
        };
    }
    if let Some(definition) = kind.transform_definition() {
        return match definition.element() {
            ElementType::Segment => Some(LineMode::Segment),
            ElementType::Ray => Some(LineMode::Ray),
            ElementType::Line => Some(LineMode::Line),
            _ => None,
        };
    }
    match kind {
        Kind::Segment(..) | Kind::ToolInput(EmbeddedToolInputKind::Segment) => {
            Some(LineMode::Segment)
        }
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
    matches!(kind, Kind::MappedElement(definition) if definition.element() == ElementType::Point)
        || matches!(kind, Kind::Colorized(d) if d.element() == ElementType::Point)
        || kind
            .transform_definition()
            .is_some_and(|definition| definition.element() == ElementType::Point)
        || matches!(
            kind,
            Kind::Free(_)
                | Kind::IterationPointAlias(_)
                | Kind::ToolPointOnPath(_)
                | Kind::SegmentMidpoint(_)
                | Kind::PointOnSegment(_, _)
                | Kind::PointOnRay(_, _)
                | Kind::PointOnLine(_, _)
                | Kind::PointOnCircle(_, _, _)
                | Kind::PointOnArc(_, _)
                | Kind::PointOnSector(_, _)
                | Kind::PointOnCircularSegment(_, _)
                | Kind::PointOnCurve(_, _)
                | Kind::PointOnPolygon(_, _, _)
                | Kind::GraphHorizontalUnit(_, _)
                | Kind::GraphVerticalUnit(_, _)
                | Kind::DistanceVerticalUnit(_, _, _)
                | Kind::PathLiteralPoint(_, _)
                | Kind::PathValuePoint(_, _)
                | Kind::PlottedPoint(_, _, _)
                | Kind::CoordinatePoint(_)
                | Kind::NumericCoordinatePoint(_, _, _, _)
                | Kind::GraphValuePoint(_, _)
                | Kind::Midpoint(_, _)
                | Kind::Intersection(_, _)
                | Kind::CurveIntersection(_, _, _)
                | Kind::BoundaryIntersection(_, _)
                | Kind::LineCircleIntersection(_, _, _)
                | Kind::CircleCircleIntersection(_, _, _)
        )
}

/// Native parameter loci fix the target and driver roles, but their capture list
/// may omit inactive branches or retain other nodes from the target closure.
pub(super) fn parameter_locus_refs_match(objects: &[Object], refs: &[usize]) -> bool {
    let Some((&target, &driver)) = refs.first().zip(refs.get(1)) else {
        return false;
    };
    let Some(expected) = expected_parameter_locus_refs(objects, driver, target) else {
        return false;
    };
    if refs.len() < 3 || refs.get(..2) != expected.get(..2) || refs.last() != Some(&target) {
        return false;
    }
    let mut allowed: BTreeSet<_> = expected[2..].iter().copied().collect();
    if let Some(page) = objects.get(target - 1).map(|object| object.page) {
        let mut closure = BTreeSet::new();
        if locus_target_dependency_closure(objects, target, page, &mut closure).is_some() {
            allowed.extend(closure);
        }
    }
    allowed.remove(&driver);
    let captured: BTreeSet<_> = refs[2..].iter().copied().collect();
    captured.len() == refs.len() - 2 && captured.is_subset(&allowed)
}

#[derive(Clone, Copy)]
pub(super) struct ParameterLocusDriverProfile {
    scale: f64,
    tag: u32,
    accepts_tag_variant: bool,
}

impl ParameterLocusDriverProfile {
    pub(super) fn scale(self) -> f64 {
        self.scale
    }

    pub(super) fn tag(self) -> u32 {
        self.tag
    }

    pub(super) fn matches_tag(self, tag: u32) -> bool {
        if self.accepts_tag_variant {
            self.tag >> 16 == tag >> 16 && tag & 0xffff <= 1
        } else {
            self.tag == tag
        }
    }
}

/// Native parameter loci may replay either a parameter or the displayed result
/// of a parameter calculation. Their ranges use radians for angular values.
pub(super) fn parameter_locus_driver_profile(
    objects: &[Object],
    driver: usize,
) -> Option<ParameterLocusDriverProfile> {
    let object = objects.get(driver.checked_sub(1)?)?;
    if object.deleted {
        return None;
    }
    let (unit, tag, accepts_tag_variant) = match &object.kind {
        Kind::Parameter(group) => (group.unit(), group.locus_unit_tag()?, true),
        Kind::ParameterCalculation(_, group) => (
            group.unit(),
            match group.unit() {
                NumericUnit::Scalar => 0,
                NumericUnit::Degrees => 0x0001_0000,
                _ => return None,
            },
            true,
        ),
        _ => return None,
    };
    let scale = match unit {
        NumericUnit::Scalar => 1.,
        NumericUnit::Degrees => crate::table_records::CALCULATOR_PI / 180.,
        _ => return None,
    };
    Some(ParameterLocusDriverProfile {
        scale,
        tag,
        accepts_tag_variant,
    })
}

pub(super) fn expected_parameter_locus_refs(
    objects: &[Object],
    driver: usize,
    target: usize,
) -> Option<Vec<usize>> {
    fn dependent_path(
        objects: &[Object],
        driver: usize,
        current: usize,
        page: usize,
        found: &mut BTreeSet<usize>,
        visited: &mut BTreeSet<usize>,
    ) -> Option<bool> {
        let object = objects.get(current.checked_sub(1)?)?;
        if object.deleted || object.page != page {
            return None;
        }
        if current == driver {
            return Some(true);
        }
        if !visited.insert(current) {
            return Some(found.contains(&current));
        }
        let mut dependent = false;
        for parent in object.kind.dependencies() {
            if parent >= current {
                return None;
            }
            dependent |= dependent_path(objects, driver, parent, page, found, visited)?;
        }
        if dependent {
            found.insert(current);
        }
        Some(dependent)
    }
    let moving = objects.get(driver.checked_sub(1)?)?;
    let traced = objects.get(target.checked_sub(1)?)?;
    if driver >= target
        || moving.deleted
        || traced.deleted
        || moving.page != traced.page
        || parameter_locus_driver_profile(objects, driver).is_none()
        || parameter_locus_subtype(objects, target).is_none()
    {
        return None;
    }
    if is_locus_straight_kind(&traced.kind) {
        locus_straight_source_mode(objects, target)?;
    }
    let mut found = BTreeSet::new();
    if !dependent_path(
        objects,
        driver,
        target,
        moving.page,
        &mut found,
        &mut BTreeSet::new(),
    )? {
        return None;
    }
    let mut refs = vec![target, driver];
    refs.extend(found);
    Some(refs)
}

/// Parameter loci use subtype 42 for a point trace and subtype 43 for a
/// sampled geometric family. Keep the latter bounded to the same evaluated
/// target categories as ordinary geometric loci.
pub(super) fn parameter_locus_subtype(objects: &[Object], target: usize) -> Option<u16> {
    let object = objects.get(target.checked_sub(1)?)?;
    if object.deleted {
        return None;
    }
    if is_point_kind(&object.kind) {
        Some(42)
    } else if is_locus_straight_kind(&object.kind)
        || is_locus_polygon_kind(&object.kind)
        || is_locus_curve_collection(objects, target)
        || super::locus::is_locus_surface_collection(objects, target)
    {
        Some(43)
    } else {
        None
    }
}

fn locus_target_dependency_closure(
    objects: &[Object],
    current: usize,
    page: usize,
    found: &mut BTreeSet<usize>,
) -> Option<()> {
    if !found.insert(current) {
        return Some(());
    }
    let object = objects.get(current.checked_sub(1)?)?;
    if object.deleted || object.page != page || matches!(object.kind, Kind::Unsupported(_)) {
        return None;
    }
    let parents = if let Kind::Midpoint(a, b) = object.kind {
        let host = object.midpoint_host?;
        if !matches!(objects.get(host.checked_sub(1)?)?.kind, Kind::Segment(x, y) if (x == a && y == b) || (x == b && y == a))
        {
            return None;
        }
        vec![host]
    } else {
        object.kind.dependencies()
    };
    for parent in parents {
        if parent >= current {
            return None;
        }
        locus_target_dependency_closure(objects, parent, page, found)?;
    }
    Some(())
}

/// Native records fix target/path/driver roles and the final target. Their
/// intermediate capture list may omit branches or retain an independent target
/// branch, but every captured node must belong to the target's live closure.
pub(super) fn locus_refs_match(objects: &[Object], refs: &[usize]) -> bool {
    let Some((&target, &path, &driver)) = refs
        .first()
        .zip(refs.get(1))
        .zip(refs.get(2))
        .map(|((target, path), driver)| (target, path, driver))
    else {
        return false;
    };
    let Some(expected) = expected_locus_refs_for_path(objects, driver, target, path) else {
        return false;
    };
    if refs.len() < 4 || refs.get(..3) != expected.get(..3) || refs.last() != Some(&target) {
        return false;
    }
    let mut allowed: BTreeSet<_> = expected[3..].iter().copied().collect();
    if let Some(page) = objects.get(target - 1).map(|object| object.page) {
        let mut closure = BTreeSet::new();
        if locus_target_dependency_closure(objects, target, page, &mut closure).is_some() {
            allowed.extend(closure);
        }
    }
    allowed.remove(&path);
    allowed.remove(&driver);
    let captured: BTreeSet<_> = refs[3..].iter().copied().collect();
    captured.len() == refs.len() - 3 && captured.is_subset(&allowed)
}

pub(super) fn expected_locus_refs(
    objects: &[Object],
    driver: usize,
    target: usize,
) -> Option<Vec<usize>> {
    let moving = objects.get(driver.checked_sub(1)?)?;
    let path = match moving.kind {
        Kind::PointOnRay(path, _)
        | Kind::PointOnLine(path, _)
        | Kind::PointOnSegment(path, _)
        | Kind::PointOnCircle(path, _, _)
        | Kind::PointOnArc(path, _)
        | Kind::PointOnCurve(path, _)
        | Kind::PointOnPolygon(path, ..)
        | Kind::PointOnSector(path, _)
        | Kind::PointOnCircularSegment(path, _) => path,
        _ => return None,
    };
    expected_locus_refs_for_path(objects, driver, target, path)
}

fn expected_locus_refs_for_path(
    objects: &[Object],
    driver: usize,
    target: usize,
    path: usize,
) -> Option<Vec<usize>> {
    fn ancestry(
        objects: &[Object],
        driver: usize,
        current: usize,
        page: usize,
        memo: &mut std::collections::BTreeMap<usize, Option<BTreeSet<usize>>>,
    ) -> Option<BTreeSet<usize>> {
        if let Some(found) = memo.get(&current) {
            return found.clone();
        }
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
        } else if !matches!(object.kind, Kind::Unsupported(_)) {
            object.kind.dependencies()
        } else {
            return None;
        };
        let mut descendants = BTreeSet::new();
        for parent in parents {
            if parent >= current {
                return None;
            }
            if let Some(found) = ancestry(objects, driver, parent, page, memo) {
                descendants.extend(found);
            }
        }
        let found = (!descendants.is_empty()).then(|| {
            descendants.insert(current);
            descendants
        });
        memo.insert(current, found.clone());
        found
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
            || is_locus_polygon_kind(&traced.kind)
            || is_locus_curve_collection(objects, target))
    {
        return None;
    }
    if is_locus_straight_kind(&traced.kind) {
        locus_straight_source_mode(objects, target)?;
    }
    super::locus::locus_driver_range_info_for_path(objects, driver, path)?;
    let host = objects.get(path.checked_sub(1)?)?;
    if path >= target || host.deleted || host.page != moving.page {
        return None;
    }
    if matches!(moving.kind, Kind::Free(_))
        && ancestry(objects, driver, path, moving.page, &mut Default::default()).is_some()
    {
        return None;
    }
    let ancestors = ancestry(
        objects,
        driver,
        target,
        moving.page,
        &mut Default::default(),
    )?;
    let mut refs = vec![target, path];
    refs.extend(ancestors);
    Some(refs)
}

pub(super) fn is_arc_kind(kind: &Kind) -> bool {
    matches!(kind, Kind::Colorized(d) if d.element() == ElementType::Arc)
        || matches!(
            kind,
            Kind::ThreePointArc(_, _, _) | Kind::ArcOnCircle(_, _, _) | Kind::CenterArc(_, _, _)
        )
        || kind
            .transform_definition()
            .is_some_and(|definition| definition.element() == ElementType::Arc)
}

pub(super) fn is_locus_polygon_kind(kind: &Kind) -> bool {
    kind.is_filled_region()
}

/// A point-output sampled object is itself a curve, so a later locus can sweep
/// that whole curve. Keep this bounded to already validated native families;
/// geometric-output loci are not recursively reclassified as curves.
pub(super) fn is_locus_curve_collection(objects: &[Object], id: usize) -> bool {
    let Some(object) = id.checked_sub(1).and_then(|index| objects.get(index)) else {
        return false;
    };
    if object.deleted {
        return false;
    }
    match &object.kind {
        Kind::SampledMapped(_) => true,
        Kind::Locus(refs, _) | Kind::ParameterLocus(refs, ..) => refs
            .first()
            .and_then(|target| target.checked_sub(1))
            .and_then(|index| objects.get(index))
            .is_some_and(|target| !target.deleted && is_point_kind(&target.kind)),
        _ => false,
    }
}

pub(super) fn is_locus_straight_kind(kind: &Kind) -> bool {
    locus_straight_mode(kind).is_some()
}

pub(super) fn is_straight_locus(objects: &[Object], kind: &Kind) -> bool {
    matches!(kind, Kind::Locus(refs, _) | Kind::ParameterLocus(refs, ..) if refs.first().and_then(|id| id.checked_sub(1))
        .and_then(|index| objects.get(index)).is_some_and(|target| is_locus_straight_kind(&target.kind)))
}

/// Geometric straight families, including their shared affine and colour
/// wrappers. Coordinate axes and abstract tool inputs have separate roles.
pub(super) fn locus_straight_mode(kind: &Kind) -> Option<LineMode> {
    if matches!(
        kind,
        Kind::Segment(_, _)
            | Kind::Ray(_, _)
            | Kind::Line(_, _)
            | Kind::Parallel(_, _)
            | Kind::Perpendicular(_, _)
            | Kind::Bisector(_, _, _)
            | Kind::Colorized(_)
    ) || kind.transform_definition().is_some()
    {
        straight_kind_mode(kind)
    } else {
        None
    }
}

/// A wrapper retains a geometric carrier's role, not an axis or abstract input
/// role. Follow only wrapper sources; ordinary derived constructions keep their
/// existing independently supported inputs.
pub(super) fn locus_straight_source_mode(objects: &[Object], id: usize) -> Option<LineMode> {
    let first = objects.get(id.checked_sub(1)?)?;
    let mode = locus_straight_mode(&first.kind)?;
    let page = first.page;
    let mut current = id;
    loop {
        let object = objects.get(current.checked_sub(1)?)?;
        if object.deleted || object.page != page || locus_straight_mode(&object.kind) != Some(mode)
        {
            return None;
        }
        let source = if let Kind::Colorized(definition) = &object.kind {
            if !definition.supported(objects, page, current) {
                return None;
            }
            definition.source()
        } else if let Some(definition) = object.kind.transform_definition() {
            definition.source()
        } else {
            return Some(mode);
        };
        if source == 0 || source >= current {
            return None;
        }
        current = source;
    }
}

pub(super) fn is_coordinate_unit_kind(kind: &Kind) -> bool {
    // Native stored axes use a numeric magnitude regardless of its units.
    kind.is_segment() || kind.numeric_unit().is_some()
}

/// Code 95/123 use the evaluated straight host's affine basis and extent.
pub(super) fn plot_straight_mode(kind: &Kind) -> Option<LineMode> {
    match kind {
        Kind::ToolInput(_) => None,
        Kind::GraphAxis(_, _, form) if !matches!(form, 0 | 1 | 0x1_0001) => None,
        Kind::CircleAxis(_, form) | Kind::DistanceAxis(_, _, form) if !matches!(form, 0 | 1) => {
            None
        }
        _ => straight_kind_mode(kind),
    }
}

pub(super) fn is_plot_path_kind(kind: &Kind) -> bool {
    plot_straight_mode(kind).is_some() || kind.is_circle() || is_arc_kind(kind)
}

pub(super) fn plot_path_accepts(kind: &Kind, value: f64) -> bool {
    value.is_finite()
        && is_plot_path_kind(kind)
        && match plot_straight_mode(kind) {
            Some(LineMode::Segment) => (0.0..=1.0).contains(&value),
            Some(LineMode::Ray) => value >= 0.0,
            Some(LineMode::Line) => true,
            None => !is_arc_kind(kind) || (0.0..=1.0).contains(&value),
        }
}

pub(super) fn is_value_plot_path_kind(kind: &Kind) -> bool {
    is_plot_path_kind(kind)
        || kind.is_polygon()
        || kind.is_sector()
        || matches!(kind, Kind::SampledMapped(_))
}

pub(super) fn value_plot_path_accepts(kind: &Kind, value: f64) -> bool {
    value.is_finite() && is_value_plot_path_kind(kind)
}

pub(super) fn is_literal_plot_path_kind(kind: &Kind) -> bool {
    is_value_plot_path_kind(kind)
}

pub(super) fn literal_plot_path_accepts(kind: &Kind, value: f64) -> bool {
    value_plot_path_accepts(kind, value)
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
            Kind::Colorized(d) if d.element() == ElementType::Polygon => d.source(),
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
    if let Some(element) = kind
        .transform_definition()
        .map(|d| d.element())
        .or_else(|| match kind {
            Kind::Colorized(d) => Some(d.element()),
            _ => None,
        })
    {
        return Some(match element {
            ElementType::Point => VisualStyle::Point {
                shape: PointStyle::Medium,
                color: red,
            },
            ElementType::Segment | ElementType::Ray | ElementType::Line => {
                VisualStyle::Stroke { line, color: navy }
            }
            ElementType::Circle | ElementType::Arc => VisualStyle::Stroke { line, color: green },
            ElementType::Polygon
            | ElementType::Sector
            | ElementType::CircularSegment
            | ElementType::Disk => VisualStyle::Polygon {
                opacity: 32767,
                color: RgbColor {
                    red: 255,
                    green: 128,
                    blue: 0,
                },
            },
            ElementType::Image => return None,
        });
    }
    match kind {
        Kind::Free(_)
        | Kind::IterationPointAlias(_)
        | Kind::MappedElement(_)
        | Kind::ToolPointOnPath(_)
        | Kind::SegmentMidpoint(_)
        | Kind::PointOnSegment(_, _)
        | Kind::PointOnRay(_, _)
        | Kind::PointOnLine(_, _)
        | Kind::PointOnCircle(_, _, _)
        | Kind::PointOnArc(_, _)
        | Kind::PointOnSector(_, _)
        | Kind::PointOnCircularSegment(_, _)
        | Kind::PointOnCurve(_, _)
        | Kind::PointOnPolygon(_, _, _)
        | Kind::PathLiteralPoint(_, _)
        | Kind::PathValuePoint(_, _)
        | Kind::CoordinatePoint(_)
        | Kind::NumericCoordinatePoint(_, _, _, _)
        | Kind::GraphValuePoint(_, _)
        | Kind::Midpoint(_, _)
        | Kind::Intersection(_, _)
        | Kind::CurveIntersection(_, _, _)
        | Kind::BoundaryIntersection(_, _)
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
        | Kind::ArcOnCircle(_, _, _)
        | Kind::CenterArc(_, _, _) => Some(VisualStyle::Stroke { line, color: green }),
        Kind::FunctionPlot(_, _, _, _, _, _) => Some(VisualStyle::Stroke {
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
        Kind::AngleMarker(..)
        | Kind::SignedAngleValue(..)
        | Kind::PolarAngleValue(..)
        | Kind::VertexAngleValue(..) => Some(VisualStyle::Stroke {
            line: SegmentLineStyle {
                width: LineWidth::Thin,
                pattern: LinePattern::Solid,
            },
            color: RgbColor {
                red: 64,
                green: 64,
                blue: 64,
            },
        }),
        Kind::SegmentMarker { .. } | Kind::SegmentArrow { .. } => Some(VisualStyle::Stroke {
            line: SegmentLineStyle {
                width: LineWidth::Thin,
                pattern: LinePattern::Solid,
            },
            color: RgbColor {
                red: 0,
                green: 0,
                blue: 0,
            },
        }),
        Kind::ParameterLocus(_, _, _, _) => Some(point_locus_visual_style()),
        Kind::SampledMapped(_) => Some(VisualStyle::Stroke {
            line: SegmentLineStyle {
                width: LineWidth::Thin,
                pattern: LinePattern::Solid,
            },
            color: RgbColor {
                red: 255,
                green: 0,
                blue: 0,
            },
        }),
        Kind::CircleInterior(_)
        | Kind::SectorBoundary(_)
        | Kind::CircularSegmentBoundary(_)
        | Kind::Polygon(_) => Some(VisualStyle::Polygon {
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
    VisualStyle::Stroke {
        line: SegmentLineStyle {
            width: LineWidth::Medium,
            pattern: LinePattern::Solid,
        },
        color: RgbColor {
            red: 128,
            green: 0,
            blue: 0,
        },
    }
}
