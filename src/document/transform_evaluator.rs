//! Shared geometry evaluation of transform definitions, independent of their
//! native wire flavor.
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
    /// Evaluate a transform through the shared model. This does not
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
            completed: &mut BTreeSet<usize>,
        ) -> Result<(), Error> {
            if completed.contains(&id) {
                return Ok(());
            }
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
                validate(document, parent, page, visiting, completed)?;
            }
            visiting.remove(&id);
            completed.insert(id);
            Ok(())
        }
        validate(
            self,
            id,
            object.page,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
        )?;
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

#[cfg(test)]
mod tests {
    use super::*;
    fn close(a: Point, b: Point) {
        assert!(
            (a.x - b.x).abs() < 1e-8 && (a.y - b.y).abs() < 1e-8,
            "{a:?} != {b:?}"
        );
    }
    fn compare(document: &Document, id: usize) {
        let definition = document.objects[id - 1]
            .kind
            .transform_definition()
            .unwrap();
        let result = document.evaluate_transform(id);
        match definition.element() {
            ElementType::Point => match document.position(id) {
                Some(expected) => match result {
                    Ok(TransformGeometry::Point(p)) => close(p, expected),
                    other => panic!("point{id}: {other:?}"),
                },
                None => assert_eq!(result, Err(Error::Undefined)),
            },
            ElementType::Segment | ElementType::Ray | ElementType::Line => {
                match document.line_points(id) {
                    Some((a, b, expected_mode)) => match result {
                        Ok(TransformGeometry::Straight { start, end, mode }) => {
                            close(start, a);
                            close(end, b);
                            assert_eq!(mode, expected_mode);
                        }
                        other => panic!("straight{id}: {other:?}"),
                    },
                    None => assert_eq!(result, Err(Error::Undefined)),
                }
            }
            ElementType::Circle => match document.circle_points(id) {
                Some((p, r)) => match result {
                    Ok(TransformGeometry::Circle { center, radius }) => {
                        close(center, p);
                        assert!((radius - r).abs() < 1e-8);
                    }
                    other => panic!("circle{id}: {other:?}"),
                },
                None => assert_eq!(result, Err(Error::Undefined)),
            },
            ElementType::Polygon => match document.polygon_points(id) {
                Some(expected) => match result {
                    Ok(TransformGeometry::Polygon(points)) => {
                        assert_eq!(points.len(), expected.len());
                        for (a, b) in points.into_iter().zip(expected) {
                            close(a, b);
                        }
                    }
                    other => panic!("polygon{id}: {other:?}"),
                },
                None => assert_eq!(result, Err(Error::Undefined)),
            },
        }
    }
    #[test]
    fn shared_evaluator_matches_native_fixture_geometry_without_mutating_records() {
        let mut flavors = BTreeSet::new();
        let mut elements = BTreeSet::new();
        let mut count = 0;
        for name in [
            "point_translation.gsp",
            "polar_segment_translation_original.gsp",
            "polar_circle_translation_original.gsp",
            "polar_translation_polygon_original.gsp",
            "rectangular_fixed_translation_original.gsp",
            "rectangular_segment_translation_original.gsp",
            "rectangular_circle_translation_original.gsp",
            "rectangular_mixed_horizontal_original.gsp",
            "rectangular_mixed_vertical_original.gsp",
            "rectangular_marked_translation_original.gsp",
            "marked_distance_translation_original.gsp",
            "marked_distance_polygon_translation_original.gsp",
            "vector_translation_polygon_generated.gsp",
            "angle_rotation_original.gsp",
            "angle_rotation_segment_original.gsp",
            "angle_rotation_circle_original.gsp",
            "ratio_dilation_segment_original.gsp",
            "ratio_dilation_circle_original.gsp",
            "reflection_polygon_original.gsp",
            "fixed_rotation_polygon_original.gsp",
            "fixed_dilation_polygon_original.gsp",
            "mixed_chain_writer_fresh_native_verified.gsp",
            "mixed_chain_writer_edited_native_verified.gsp",
            "mixed_chain_writer_negative_native_verified.gsp",
            "mixed_chain_writer_source_zero_native_verified.gsp",
            "mixed_chain_writer_source_undefined_native_verified.gsp",
            "mixed_chain_writer_child_zero_native_verified.gsp",
            "mixed_chain_writer_fixed_zero_native_verified.gsp",
            "mixed_chain_writer_copied_deleted_native_verified.gsp",
        ] {
            let bytes = std::fs::read(format!("tests/fixtures/{name}")).unwrap();
            let d = Document::from_bytes(&bytes, None).unwrap();
            for object in &d.objects {
                if let Some(definition) = object.kind.transform_definition() {
                    let Kind::Transformed(stored) = &object.kind else {
                        panic!(
                            "native transform {} in {name} must use shared storage",
                            object.id
                        );
                    };
                    assert_eq!(stored, &definition);
                    assert_eq!(object.kind.dependencies(), definition.native_references());
                    compare(&d, object.id);
                    count += 1;
                    flavors.insert(definition.wire_flavor().code());
                    elements.insert(format!("{:?}", definition.element()));
                }
            }
            assert_eq!(d.encode().unwrap(), bytes, "{name}");
        }
        assert_eq!(flavors.len(), 14);
        assert_eq!(elements.len(), 6);
        assert!(count > 200);
    }
    #[test]
    fn shared_undefined_dependency_dag_does_not_expand_repeated_parent_paths() {
        let mut d = Document::new();
        let source = d.add(Kind::Free(Point { x: 20., y: 30. }));
        let center = d.add(Kind::Free(Point { x: 0., y: 0. }));
        let parameter = d.add_named_parameter("n", 4., (10, 10)).unwrap();
        let angle = d
            .add_parameter_calculation(parameter, "360°/n", (10, 40))
            .unwrap();
        let mut image = d
            .add_calculated_angle_rotation(source, center, angle)
            .unwrap();
        // Each layer names the same previous node three times. Validation must
        // complete each node once instead of walking 3^40 equivalent paths.
        for _ in 0..40 {
            image = d.add(Kind::VectorTranslatedPoint(image, image, image));
        }
        d.set_parameter_value(parameter, 0.).unwrap();
        assert_eq!(d.evaluate_transform(image), Err(Error::Undefined));
        assert!(d.position(image).is_none());
    }

    #[test]
    fn source_first_domains_units_and_invalid_dependencies_are_distinct() {
        let bytes =
            include_bytes!("../../tests/fixtures/mixed_chain_writer_fresh_native_verified.gsp");
        let mut d = Document::from_bytes(bytes, None).unwrap();
        d.objects[50].kind = Kind::CalculatedRatioScaledLine(19, 1, 10);
        assert_eq!(
            d.evaluate_transform(51),
            Err(Error::WrongUnit {
                id: 10,
                expected: NumericUnit::Scalar
            })
        );
        d.set_parameter_value(6, 0.).unwrap();
        assert_eq!(d.evaluate_transform(51), Err(Error::Undefined));
        d.set_parameter_value(6, 4.).unwrap();
        d.objects[18].deleted = true;
        assert_eq!(d.evaluate_transform(51), Err(Error::InvalidReference(19)));
        assert_eq!(d.evaluate_transform(1), Err(Error::NotTransform));
        assert_eq!(d.evaluate_transform(0), Err(Error::InvalidReference(0)));
        let mut malformed = Document::from_bytes(bytes, None).unwrap();
        malformed.objects[35].kind = Kind::RotatedRay(0, 1, 90.);
        assert_eq!(
            malformed.evaluate_transform(36),
            Err(Error::InvalidReference(0))
        );
        let mut d = Document::from_bytes(bytes, None).unwrap();
        d.objects[35].kind = Kind::RotatedLine(16, 1, 90.);
        assert_eq!(d.evaluate_transform(36), Err(Error::WrongElement(16)));
    }
}
