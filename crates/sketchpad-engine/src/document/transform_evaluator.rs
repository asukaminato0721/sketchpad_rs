//! Shared geometry evaluation of transform definitions, independent of their
//! native wire flavor.
use super::geometry::{marked_rotation_theta, ratio_of_points};
use super::kind::{is_arc_kind, is_point_kind};
use super::transform_model::{
    DisplacementExpr, ElementType, NativeTransformFlavor, NumericExpr, Transform,
};
use super::{Document, Kind, LineMode, Point};
use crate::table_records::NumericUnit;
use std::collections::BTreeSet;

use sketchpad_ir::affine::MatrixEvaluation;
pub use sketchpad_ir::affine::{
    AffineMatrix, TransformEvaluationError, TransformGeometry, TransformSequence,
};

use TransformEvaluationError as Error;

impl Document {
    /// Evaluate a transform through the shared model. This does not
    /// change authoring capability guards or native serialization.
    pub fn evaluate_transform(&self, id: usize) -> Result<TransformGeometry, Error> {
        self.evaluate_transform_with_matrix(id)
            .map(|(geometry, _)| geometry)
    }

    /// Resolve each live operation and reject undefined intermediate geometry
    /// before exposing a matrix sequence. A later transform cannot recover an
    /// undefined circle, ray, line, or dependency.
    pub fn transform_sequence(&self, id: usize) -> Result<TransformSequence, Error> {
        let (_, matrix) = self.evaluate_transform_with_matrix(id)?;
        let definition = self.objects[id - 1]
            .kind
            .transform_definition()
            .ok_or(Error::NotTransform)?;
        let element = definition.element();
        let mut source = definition.source();
        let mut matrices = vec![matrix];
        while let Some(parent) = self.objects[source - 1].kind.transform_definition() {
            if parent.element() != element {
                return Err(Error::WrongElement(source));
            }
            let (_, matrix) = self.evaluate_transform_with_matrix(source)?;
            matrices.push(matrix);
            source = parent.source();
        }
        matrices.reverse();
        Ok(TransformSequence {
            source,
            element,
            matrices,
        })
    }

    fn evaluate_transform_with_matrix(
        &self,
        id: usize,
    ) -> Result<(TransformGeometry, AffineMatrix), Error> {
        let object = id
            .checked_sub(1)
            .and_then(|i| self.objects.get(i))
            .filter(|o| !o.deleted)
            .ok_or(Error::InvalidReference(id))?;
        let definition = object
            .kind
            .transform_definition()
            .ok_or(Error::NotTransform)?;
        self.validate_transform_dependencies(id)?;
        let source = definition.source();
        let kind = &self.objects[source - 1].kind;
        // Resolve source geometry first: finite target inputs cannot restore an
        // undefined or zero-length transformed ray/line ancestor.
        let mut geometry = match definition.element() {
            ElementType::Point if is_point_kind(kind) => {
                TransformGeometry::Point(self.position(source).ok_or(Error::Undefined)?)
            }
            ElementType::Segment if kind.is_segment() => {
                let (start, end, mode) = self.line_points(source).ok_or(Error::Undefined)?;
                if mode != LineMode::Segment {
                    return Err(Error::WrongElement(source));
                }
                TransformGeometry::Straight { start, end, mode }
            }
            ElementType::Ray | ElementType::Line => {
                let expected = if definition.element() == ElementType::Ray {
                    LineMode::Ray
                } else {
                    LineMode::Line
                };
                let actual = if matches!(
                    definition.wire_flavor(),
                    NativeTransformFlavor::FixedRotation | NativeTransformFlavor::NumericRotation
                ) {
                    kind.rotation_transform_straight_mode()
                } else {
                    kind.fixed_transform_straight_mode()
                };
                if actual != Some(expected) {
                    return Err(Error::WrongElement(source));
                }
                let (start, end, mode) = self.line_points(source).ok_or(Error::Undefined)?;
                if mode != expected {
                    return Err(Error::WrongElement(source));
                }
                if start == end {
                    return Err(Error::Undefined);
                }
                TransformGeometry::Straight { start, end, mode }
            }
            ElementType::Circle if kind.is_circle() => {
                let (center, radius) = self.circle_points(source).ok_or(Error::Undefined)?;
                TransformGeometry::Circle { center, radius }
            }
            ElementType::Disk if kind.is_circle_interior() => {
                let (center, radius) = self
                    .circle_interior_geometry(source)
                    .ok_or(Error::Undefined)?;
                TransformGeometry::Circle { center, radius }
            }
            ElementType::Arc if is_arc_kind(kind) => {
                if let Some((start, end)) = self.linear_three_point_arc(source) {
                    TransformGeometry::Straight {
                        start,
                        end,
                        mode: LineMode::Segment,
                    }
                } else {
                    let (center, radius, start, sweep, clockwise_y) =
                        self.arc_geometry(source).ok_or(Error::Undefined)?;
                    TransformGeometry::Arc {
                        center,
                        radius,
                        start,
                        sweep,
                        clockwise_y,
                    }
                }
            }
            ElementType::Polygon if kind.is_polygon() => {
                TransformGeometry::Polygon(self.polygon_points(source).ok_or(Error::Undefined)?)
            }
            ElementType::Sector if kind.is_sector() => {
                let (center, radius, start, sweep, clockwise_y) =
                    self.sector_arc_geometry(source).ok_or(Error::Undefined)?;
                TransformGeometry::Sector {
                    center,
                    radius,
                    start,
                    sweep,
                    clockwise_y,
                }
            }
            ElementType::CircularSegment if kind.is_circular_segment() => {
                let (center, radius, start, sweep, clockwise_y) = self
                    .circular_segment_geometry(source)
                    .ok_or(Error::Undefined)?;
                TransformGeometry::Sector {
                    center,
                    radius,
                    start,
                    sweep,
                    clockwise_y,
                }
            }
            _ => return Err(Error::WrongElement(source)),
        };
        let (matrix, radius_factor) = self.transform_operation_matrix(&definition)?;
        let apply = |p| -> Result<Point, Error> {
            matrix
                .apply(p)
                .filter(|p| p.x.is_finite() && p.y.is_finite())
                .ok_or(Error::Undefined)
        };
        match &mut geometry {
            TransformGeometry::Point(p) => *p = apply(*p)?,
            TransformGeometry::Straight { start, end, mode } => {
                *start = apply(*start)?;
                *end = apply(*end)?;
                if *mode != LineMode::Segment && start == end {
                    return Err(Error::Undefined);
                }
            }
            TransformGeometry::Circle { center, radius } => {
                *center = apply(*center)?;
                *radius *= radius_factor;
                if !radius.is_finite() || *radius <= 0. {
                    return Err(Error::Undefined);
                }
            }
            TransformGeometry::Arc {
                center,
                radius,
                start,
                sweep,
                clockwise_y,
            }
            | TransformGeometry::Sector {
                center,
                radius,
                start,
                sweep,
                clockwise_y,
            } => {
                let first = Point {
                    x: center.x + *radius * start.cos(),
                    y: center.y
                        + if *clockwise_y {
                            -*radius * start.sin()
                        } else {
                            *radius * start.sin()
                        },
                };
                let first = apply(first)?;
                *center = apply(*center)?;
                *radius *= radius_factor;
                if !radius.is_finite() || *radius <= 0. {
                    return Err(Error::Undefined);
                }
                *start = (if *clockwise_y {
                    center.y - first.y
                } else {
                    first.y - center.y
                })
                .atan2(first.x - center.x);
                if matches!(definition.transform(), Transform::Reflect { .. }) {
                    *sweep = -*sweep;
                }
            }
            TransformGeometry::Polygon(vertices) => {
                for p in vertices {
                    *p = apply(*p)?;
                }
            }
        }
        Ok((geometry, matrix))
    }

    pub(super) fn validate_transform_dependencies(&self, id: usize) -> Result<(), Error> {
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or(Error::InvalidReference(id))?)
            .ok_or(Error::InvalidReference(id))?;
        fn validate(
            document: &Document,
            id: usize,
            page: usize,
            visiting: &mut BTreeSet<usize>,
            completed: &mut BTreeSet<usize>,
        ) -> Result<(), Error> {
            let o = id
                .checked_sub(1)
                .and_then(|i| document.objects.get(i))
                .filter(|o| !o.deleted && o.page == page && !matches!(o.kind, Kind::Unsupported(_)))
                .ok_or(Error::InvalidReference(id))?;
            if completed.contains(&id) {
                return Ok(());
            }
            if !visiting.insert(id) {
                return Err(Error::InvalidReference(id));
            }
            for parent in o.kind.dependencies() {
                if parent == 0 || parent >= id {
                    return Err(Error::InvalidReference(parent));
                }
                validate(document, parent, page, visiting, completed)?;
            }
            visiting.remove(&id);
            completed.insert(id);
            Ok(())
        }
        self.with_validated_dependencies(|completed| {
            validate(self, id, object.page, &mut BTreeSet::new(), completed)
        })
    }

    /// Evaluate one live affine operation independently of its source geometry.
    /// Pictures use this to compose the operation with their source placement.
    pub(super) fn transform_operation_matrix(
        &self,
        definition: &super::transform_model::TransformDefinition,
    ) -> Result<(AffineMatrix, f64), Error> {
        let point = |id| self.position(id).ok_or(Error::Undefined);
        let numeric = |expr: &NumericExpr| -> Result<f64, Error> {
            let value = match *expr {
                NumericExpr::Literal {
                    magnitude, scale, ..
                } => Some(magnitude * scale),
                NumericExpr::ObjectValue { id, unit } => {
                    let parent = &self
                        .objects
                        .get(id.checked_sub(1).ok_or(Error::InvalidReference(id))?)
                        .ok_or(Error::InvalidReference(id))?
                        .kind;
                    if !parent.is_transform_operand(unit) {
                        return Err(Error::WrongUnit { id, expected: unit });
                    }
                    match unit {
                        NumericUnit::Scalar | NumericUnit::Degrees => self.numeric_value(id),
                        NumericUnit::Length => self.marked_distance_value_pixels(id),
                        _ => None,
                    }
                }
                NumericExpr::DirectedAngle {
                    first,
                    vertex,
                    last,
                } => marked_rotation_theta(point(first)?, point(vertex)?, point(last)?)
                    .map(f64::to_degrees),
                NumericExpr::DistanceRatio {
                    origin,
                    base,
                    target,
                } => ratio_of_points(point(origin)?, point(base)?, point(target)?),
                NumericExpr::SegmentRatio {
                    numerator,
                    denominator,
                } => self.ratio_for_segments(numerator, denominator),
            };
            value.filter(|v| v.is_finite()).ok_or(Error::Undefined)
        };
        let mut radius_factor = 1.;
        let matrix = match definition.transform() {
            Transform::Translate(displacement) => {
                let (dx, dy) = match displacement {
                    DisplacementExpr::Cartesian {
                        horizontal,
                        vertical,
                    } => (numeric(horizontal)?, -numeric(vertical)?),
                    DisplacementExpr::Polar { angle, distance } => {
                        let theta = numeric(angle)?.to_radians();
                        let step = numeric(distance)?;
                        (step * theta.cos(), -step * theta.sin())
                    }
                    DisplacementExpr::Vector { start, end } => {
                        let a = point(*start)?;
                        let b = point(*end)?;
                        (b.x - a.x, b.y - a.y)
                    }
                };
                AffineMatrix::from_evaluation(
                    [[1., 0., dx], [0., 1., dy]],
                    MatrixEvaluation::Translation(dx, dy),
                )
            }
            Transform::Rotate { center, angle } => {
                let center = point(*center)?;
                let theta = numeric(angle)?.to_radians();
                let (sin, cos) = theta.sin_cos();
                AffineMatrix::from_evaluation(
                    [
                        [cos, sin, center.x - cos * center.x - sin * center.y],
                        [-sin, cos, center.y + sin * center.x - cos * center.y],
                    ],
                    MatrixEvaluation::Rotation(center, theta),
                )
            }
            Transform::Dilate { center, ratio } => {
                let center = point(*center)?;
                let factor = numeric(ratio)?;
                if factor == 0.
                    && matches!(
                        definition.element(),
                        ElementType::Ray
                            | ElementType::Line
                            | ElementType::Circle
                            | ElementType::Arc
                    )
                {
                    return Err(Error::Undefined);
                }
                radius_factor = factor.abs();
                AffineMatrix::from_evaluation(
                    [
                        [factor, 0., center.x * (1. - factor)],
                        [0., factor, center.y * (1. - factor)],
                    ],
                    MatrixEvaluation::Dilation(center, factor),
                )
            }
            Transform::Reflect { mirror } => {
                let (a, b, _) = self.line_points(*mirror).ok_or(Error::Undefined)?;
                let (dx, dy) = (b.x - a.x, b.y - a.y);
                let length_sq = dx * dx + dy * dy;
                if !length_sq.is_finite() || length_sq <= 1e-18 {
                    return Err(Error::Undefined);
                }
                let xx = 2. * dx * dx / length_sq - 1.;
                let xy = 2. * dx * dy / length_sq;
                let yy = 2. * dy * dy / length_sq - 1.;
                AffineMatrix::from_evaluation(
                    [
                        [xx, xy, a.x - xx * a.x - xy * a.y],
                        [xy, yy, a.y - xy * a.x - yy * a.y],
                    ],
                    MatrixEvaluation::Reflection(a, b),
                )
            }
        };
        Ok((matrix, radius_factor))
    }
}
