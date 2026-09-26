//! Shared evaluation of projected transform definitions. Existing geometry and
//! native writing callers are deliberately unchanged during migration.
use super::geometry::{
    marked_rotation_theta, ratio_of_points, reflect_point, rotate_point_about, scale_point_about,
};
use super::kind::is_point_kind;
use super::transform_model::{DisplacementExpr, ElementType, NumericExpr, Transform};
use super::{Document, Kind, LineMode, Point};
use crate::table_records::NumericUnit;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq)]
pub enum TransformGeometry {
    Point(Point),
    Straight {
        start: Point,
        end: Point,
        mode: LineMode,
    },
    Circle {
        center: Point,
        radius: f64,
    },
    Polygon(Vec<Point>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransformEvaluationError {
    NotTransform,
    InvalidReference(usize),
    WrongElement(usize),
    WrongUnit { id: usize, expected: NumericUnit },
    Undefined,
}
use TransformEvaluationError as Error;

impl Document {
    /// Evaluate a legacy transform through the shared model. This does not
    /// change authoring capability guards or native serialization.
    pub fn evaluate_transform(&self, id: usize) -> Result<TransformGeometry, Error> {
        let object = id
            .checked_sub(1)
            .and_then(|i| self.objects.get(i))
            .filter(|o| !o.deleted)
            .ok_or(Error::InvalidReference(id))?;
        let definition = object
            .kind
            .transform_definition()
            .ok_or(Error::NotTransform)?;
        fn validate(
            document: &Document,
            id: usize,
            page: usize,
            visiting: &mut BTreeSet<usize>,
        ) -> Result<(), Error> {
            let o = id
                .checked_sub(1)
                .and_then(|i| document.objects.get(i))
                .filter(|o| !o.deleted && o.page == page && !matches!(o.kind, Kind::Unsupported(_)))
                .ok_or(Error::InvalidReference(id))?;
            if !visiting.insert(id) {
                return Err(Error::InvalidReference(id));
            }
            for parent in o.kind.dependencies() {
                if parent == 0 || parent >= id {
                    return Err(Error::InvalidReference(parent));
                }
                validate(document, parent, page, visiting)?;
            }
            visiting.remove(&id);
            Ok(())
        }
        validate(self, id, object.page, &mut BTreeSet::new())?;
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
                if kind.fixed_transform_straight_mode() != Some(expected) {
                    return Err(Error::WrongElement(source));
                }
                let (start, end, mode) = self.line_points(source).ok_or(Error::Undefined)?;
                if mode != expected {
                    return Err(Error::WrongElement(source));
                }
                TransformGeometry::Straight { start, end, mode }
            }
            ElementType::Circle if kind.is_circle() => {
                let (center, radius) = self.circle_points(source).ok_or(Error::Undefined)?;
                TransformGeometry::Circle { center, radius }
            }
            ElementType::Polygon if kind.is_polygon() => {
                TransformGeometry::Polygon(self.polygon_points(source).ok_or(Error::Undefined)?)
            }
            _ => return Err(Error::WrongElement(source)),
        };
        let point = |id| self.position(id).ok_or(Error::Undefined);
        let numeric = |expr: &NumericExpr| -> Result<f64, Error> {
            let value = match *expr {
                NumericExpr::Literal {
                    magnitude, scale, ..
                } => Some(magnitude * scale),
                NumericExpr::ObjectValue { id, unit } => {
                    if self.objects.get(id - 1).and_then(|o| o.kind.numeric_unit()) != Some(unit) {
                        return Err(Error::WrongUnit { id, expected: unit });
                    }
                    match unit {
                        NumericUnit::Scalar => self.unitless_numeric_value(id),
                        NumericUnit::Degrees => self.numeric_value(id),
                        NumericUnit::Length => self.distance_value_pixels(id),
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
            };
            value.filter(|v| v.is_finite()).ok_or(Error::Undefined)
        };
        let mut radius_factor = 1.;
        let map: Box<dyn Fn(Point) -> Option<Point>> = match definition.transform() {
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
                Box::new(move |p| {
                    Some(Point {
                        x: p.x + dx,
                        y: p.y + dy,
                    })
                })
            }
            Transform::Rotate { center, angle } => {
                let center = point(*center)?;
                let theta = numeric(angle)?.to_radians();
                Box::new(move |p| rotate_point_about(p, center, theta))
            }
            Transform::Dilate { center, ratio } => {
                let center = point(*center)?;
                let factor = numeric(ratio)?;
                if factor == 0.
                    && matches!(
                        definition.element(),
                        ElementType::Ray | ElementType::Line | ElementType::Circle
                    )
                {
                    return Err(Error::Undefined);
                }
                radius_factor = factor.abs();
                Box::new(move |p| scale_point_about(p, center, factor))
            }
            Transform::Reflect { mirror } => {
                let (a, b, _) = self.line_points(*mirror).ok_or(Error::Undefined)?;
                Box::new(move |p| reflect_point(p, a, b))
            }
        };
        let apply = |p| -> Result<Point, Error> {
            map(p)
                .filter(|p| p.x.is_finite() && p.y.is_finite())
                .ok_or(Error::Undefined)
        };
        match &mut geometry {
            TransformGeometry::Point(p) => *p = apply(*p)?,
            TransformGeometry::Straight { start, end, .. } => {
                *start = apply(*start)?;
                *end = apply(*end)?;
            }
            TransformGeometry::Circle { center, radius } => {
                *center = apply(*center)?;
                *radius *= radius_factor;
                if !radius.is_finite() || *radius <= 0. {
                    return Err(Error::Undefined);
                }
            }
            TransformGeometry::Polygon(vertices) => {
                for p in vertices {
                    *p = apply(*p)?;
                }
            }
        }
        Ok(geometry)
    }
}
