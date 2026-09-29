//! Save policies for shared transform definitions. Geometry and serialization
//! share definitions, while authoring capabilities retain their native proofs.
use super::kind::{
    is_arc_kind, is_point_kind, is_segment_kind, is_straight_kind, polygon_vertex_count,
};
use super::transform_model::{
    DisplacementExpr, ElementType, NativeLiteralRole, NativeTransformFlavor, NumericExpr,
    Transform, TransformDefinition,
};
use super::{Document, LineMode, Object};

use ElementType as E;
use NativeTransformFlavor as F;

fn source_supported(document: &Document, definition: &TransformDefinition) -> bool {
    let Some(source) = definition
        .source()
        .checked_sub(1)
        .and_then(|index| document.objects.get(index))
        .filter(|source| !source.deleted)
    else {
        return false;
    };
    match definition.element() {
        E::Point => is_point_kind(&source.kind),
        E::Segment => is_segment_kind(&source.kind),
        E::Ray | E::Line => {
            source.kind.fixed_transform_straight_mode()
                == Some(if definition.element() == E::Ray {
                    LineMode::Ray
                } else {
                    LineMode::Line
                })
        }
        E::Circle => source.kind.is_circle(),
        E::Arc => is_arc_kind(&source.kind),
        E::Sector => source.kind.is_sector(),
        E::Polygon => {
            source.kind.is_polygon()
                && polygon_vertex_count(&document.objects, definition.source()).is_some()
        }
        E::Disk => source.kind.is_circle_interior(),
        E::Image => matches!(source.kind, super::Kind::Image { .. }),
    }
}

fn expression_parents_supported(document: &Document, expression: &NumericExpr) -> bool {
    let point = |id: usize| {
        id.checked_sub(1)
            .and_then(|index| document.objects.get(index))
            .is_some_and(|parent| !parent.deleted && is_point_kind(&parent.kind))
    };
    match *expression {
        NumericExpr::Literal { .. } => true,
        NumericExpr::ObjectValue { id, unit } => id
            .checked_sub(1)
            .and_then(|index| document.objects.get(index))
            .is_some_and(|parent| !parent.deleted && parent.kind.is_transform_operand(unit)),
        NumericExpr::DirectedAngle {
            first,
            vertex,
            last,
        } => [first, vertex, last].into_iter().all(point),
        NumericExpr::DistanceRatio {
            origin,
            base,
            target,
        } => [origin, base, target].into_iter().all(point),
    }
}

fn operation_parents_supported(document: &Document, definition: &TransformDefinition) -> bool {
    let point = |id: usize| {
        id.checked_sub(1)
            .and_then(|index| document.objects.get(index))
            .is_some_and(|parent| !parent.deleted && is_point_kind(&parent.kind))
    };
    match definition.transform() {
        Transform::Translate(DisplacementExpr::Cartesian {
            horizontal,
            vertical,
        }) => {
            expression_parents_supported(document, horizontal)
                && expression_parents_supported(document, vertical)
        }
        Transform::Translate(DisplacementExpr::Polar { angle, distance }) => {
            expression_parents_supported(document, angle)
                && expression_parents_supported(document, distance)
        }
        Transform::Translate(DisplacementExpr::Vector { start, end }) => {
            point(*start) && point(*end)
        }
        Transform::Rotate { center, angle } => {
            point(*center) && expression_parents_supported(document, angle)
        }
        Transform::Dilate { center, ratio } => {
            point(*center) && expression_parents_supported(document, ratio)
        }
        Transform::Reflect { mirror } => mirror
            .checked_sub(1)
            .and_then(|index| document.objects.get(index))
            .is_some_and(|parent| !parent.deleted && is_straight_kind(&parent.kind)),
    }
}

fn validate_literals(definition: &TransformDefinition) -> bool {
    let straight = matches!(definition.element(), E::Ray | E::Line);
    definition.native_literals().into_iter().all(|literal| {
        literal.value.is_finite()
            && match literal.role {
                NativeLiteralRole::PolarUnitsToPixels => literal.value > 0.,
                NativeLiteralRole::HorizontalUnitsToPixels
                | NativeLiteralRole::VerticalUnitsToPixels => {
                    (literal.value - 37.79527559055118).abs() <= 1e-9
                }
                NativeLiteralRole::ScaleFactor => straight || literal.value != 0.,
                _ => true,
            }
    })
}

pub(super) fn definition_supported(document: &Document, definition: &TransformDefinition) -> bool {
    validate_literals(definition)
        && source_supported(document, definition)
        && operation_parents_supported(document, definition)
}

/// Apply the native-proved save policies to a shared transform definition.
/// Imported nonnumeric transforms keep raw retention; numeric transforms retain
/// their stricter direct-parent checks. Generic ID/style/page save checks remain
/// in validation.rs, including its separate multi-page serialization policies.
pub(super) fn validate_transform(
    document: &Document,
    object: &Object,
    is_new: bool,
) -> Result<(), String> {
    let Some(definition) = object.kind.transform_definition() else {
        return Ok(());
    };
    let invalid = || format!("Invalid source or parameters in transform {}", object.id);
    if !validate_literals(&definition) {
        return Err(invalid());
    }
    let flavor = definition.wire_flavor();
    // Only the standalone imported point profile is proved. Authoring needs
    // an independent Native writer control for this custom-transform opcode.
    if is_new && flavor == F::DualMarkedPolar {
        return Err(invalid());
    }
    let numeric = matches!(flavor, F::NumericRotation | F::NumericDilation);
    if !is_new && !numeric {
        return Ok(());
    }
    if definition
        .native_references()
        .iter()
        .any(|id| *id == 0 || *id >= object.id)
        || !definition_supported(document, &definition)
    {
        return Err(invalid());
    }
    // Undefined live inputs retain their construction and can become defined
    // again after edits. Saving and copying require valid roles, not a current value.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::{Kind, Point};
    use super::*;
    use std::collections::BTreeSet;

    fn parents() -> Document {
        let mut document = Document::new();
        for point in [
            Point { x: 100., y: 100. },
            Point { x: 150., y: 100. },
            Point { x: 120., y: 150. },
        ] {
            document.add(Kind::Free(point));
        }
        document.add(Kind::Segment(1, 2));
        document.add(Kind::Ray(1, 2));
        document.add(Kind::Line(1, 2));
        document.add(Kind::Circle(1, 2));
        document.add(Kind::Polygon(vec![1, 2, 3]));
        document.add(Kind::SegmentLength(4));
        document.add(Kind::AngleMeasurement(1, 3, 2));
        document.add_named_parameter("n", 2., (10, 10)).unwrap();
        document
    }

    #[test]
    fn all_sixty_constructors_and_both_mixed_flavors_keep_defined_save_policies() {
        let kinds = [
            Kind::PolarTranslatedPoint(1, 15., 37.79527559055118, 2.),
            Kind::PolarTranslatedSegment(4, 15., 37.79527559055118, 2.),
            Kind::PolarTranslatedCircle(7, 15., 37.79527559055118, 2.),
            Kind::PolarTranslatedPolygon(8, 15., 37.79527559055118, 2.),
            Kind::FixedRectangularTranslatedPoint(1, 37.79527559055118, 1., 37.79527559055118, 2.),
            Kind::FixedRectangularTranslatedSegment(
                4,
                37.79527559055118,
                1.,
                37.79527559055118,
                2.,
            ),
            Kind::FixedRectangularTranslatedCircle(7, 37.79527559055118, 1., 37.79527559055118, 2.),
            Kind::FixedRectangularTranslatedPolygon(
                8,
                37.79527559055118,
                1.,
                37.79527559055118,
                2.,
            ),
            Kind::MarkedDistanceTranslatedPoint(1, 9, 15.),
            Kind::MarkedDistanceTranslatedSegment(4, 9, 15.),
            Kind::MarkedDistanceTranslatedCircle(7, 9, 15.),
            Kind::MarkedDistanceTranslatedPolygon(8, 9, 15.),
            Kind::MixedRectangularTranslatedPoint(1, 9, 1., false),
            Kind::MixedRectangularTranslatedPoint(1, 9, 1., true),
            Kind::MixedRectangularTranslatedSegment(4, 9, 1., false),
            Kind::MixedRectangularTranslatedSegment(4, 9, 1., true),
            Kind::MixedRectangularTranslatedCircle(7, 9, 1., false),
            Kind::MixedRectangularTranslatedCircle(7, 9, 1., true),
            Kind::MixedRectangularTranslatedPolygon(8, 9, 1., false),
            Kind::MixedRectangularTranslatedPolygon(8, 9, 1., true),
            Kind::RectangularMarkedTranslatedPoint(1, 9, 9),
            Kind::RectangularMarkedTranslatedSegment(4, 9, 9),
            Kind::RectangularMarkedTranslatedCircle(7, 9, 9),
            Kind::RectangularMarkedTranslatedPolygon(8, 9, 9),
            Kind::VectorTranslatedPoint(1, 1, 2),
            Kind::VectorTranslatedSegment(4, 1, 2),
            Kind::VectorTranslatedCircle(7, 1, 2),
            Kind::VectorTranslatedPolygon(8, 1, 2),
            Kind::AngleRotatedPoint(1, 3, 1, 3, 2),
            Kind::AngleRotatedSegment(4, 3, 1, 3, 2),
            Kind::AngleRotatedCircle(7, 3, 1, 3, 2),
            Kind::AngleRotatedPolygon(8, 3, 1, 3, 2),
            Kind::RatioScaledPoint(1, 3, 1, 2, 3),
            Kind::RatioScaledSegment(4, 3, 1, 2, 3),
            Kind::RatioScaledCircle(7, 3, 1, 2, 3),
            Kind::RatioScaledPolygon(8, 3, 1, 2, 3),
            Kind::ReflectedPoint(1, 4),
            Kind::ReflectedSegment(4, 4),
            Kind::ReflectedCircle(7, 4),
            Kind::ReflectedPolygon(8, 4),
            Kind::ScaledPoint(1, 3, 2.),
            Kind::ScaledSegment(4, 3, 2.),
            Kind::ScaledRay(5, 3, 2.),
            Kind::ScaledLine(6, 3, 2.),
            Kind::ScaledCircle(7, 3, 2.),
            Kind::ScaledPolygon(8, 3, 2.),
            Kind::RotatedPoint(1, 3, 15.),
            Kind::RotatedSegment(4, 3, 15.),
            Kind::RotatedRay(5, 3, 15.),
            Kind::RotatedLine(6, 3, 15.),
            Kind::RotatedCircle(7, 3, 15.),
            Kind::RotatedPolygon(8, 3, 15.),
            Kind::CalculatedRatioScaledPoint(1, 3, 11),
            Kind::CalculatedRatioScaledSegment(4, 3, 11),
            Kind::CalculatedRatioScaledRay(5, 3, 11),
            Kind::CalculatedRatioScaledLine(6, 3, 11),
            Kind::CalculatedRatioScaledCircle(7, 3, 11),
            Kind::CalculatedRatioScaledPolygon(8, 3, 11),
            Kind::CalculatedAngleRotatedPoint(1, 3, 10),
            Kind::CalculatedAngleRotatedSegment(4, 3, 10),
            Kind::CalculatedAngleRotatedRay(5, 3, 10),
            Kind::CalculatedAngleRotatedLine(6, 3, 10),
            Kind::CalculatedAngleRotatedCircle(7, 3, 10),
            Kind::CalculatedAngleRotatedPolygon(8, 3, 10),
        ];
        assert_eq!(kinds.len(), 64);
        let mut combinations = BTreeSet::new();
        let mut flavors = BTreeSet::new();
        let mut elements = BTreeSet::new();
        for kind in kinds {
            let Kind::Transformed(definition) = &kind else {
                panic!("transform constructor must use shared storage");
            };
            combinations.insert((
                definition.wire_flavor().code(),
                format!("{:?}", definition.element()),
            ));
            flavors.insert(definition.wire_flavor().code());
            elements.insert(format!("{:?}", definition.element()));
            let mut document = parents();
            let id = document.add(kind);
            let object = &document.objects[id - 1];
            validate_transform(&document, object, true).unwrap();
            validate_transform(&document, object, false).unwrap();
            document.can_save().unwrap();
        }
        assert_eq!(combinations.len(), 64);
        assert_eq!(flavors.len(), 14);
        assert_eq!(elements.len(), 6);
    }

    #[test]
    fn typed_sources_support_chained_transforms() {
        let mut document = parents();
        let translated_segment = document.add(Kind::VectorTranslatedSegment(4, 1, 2));
        let translated_circle = document.add(Kind::VectorTranslatedCircle(7, 1, 2));
        let translated_polygon = document.add(Kind::VectorTranslatedPolygon(8, 1, 2));
        for kind in [
            Kind::FixedRectangularTranslatedCircle(
                translated_circle,
                37.79527559055118,
                1.,
                37.79527559055118,
                1.,
            ),
            Kind::RotatedPolygon(translated_polygon, 3, 30.),
            Kind::AngleRotatedSegment(translated_segment, 3, 1, 3, 2),
        ] {
            let mut candidate = document.clone();
            let id = candidate.add(kind);
            let object = &candidate.objects[id - 1];
            validate_transform(&candidate, object, true).unwrap();
            assert!(validate_transform(&candidate, object, false).is_ok());
        }
        for kind in [
            Kind::PolarTranslatedSegment(translated_segment, 0., 37.79527559055118, 1.),
            Kind::VectorTranslatedSegment(translated_segment, 1, 2),
            Kind::RotatedCircle(translated_circle, 3, 30.),
            Kind::RatioScaledCircle(translated_circle, 3, 1, 2, 3),
            Kind::CalculatedAngleRotatedPolygon(translated_polygon, 3, 10),
        ] {
            let mut candidate = document.clone();
            let id = candidate.add(kind);
            validate_transform(&candidate, &candidate.objects[id - 1], true).unwrap();
        }
    }

    #[test]
    fn literal_and_numeric_parent_rejections_apply_to_imported_and_new_objects() {
        for kind in [
            Kind::PolarTranslatedPoint(1, 0., 0., 1.),
            Kind::FixedRectangularTranslatedPoint(1, 17., 1., 37.79527559055118, 1.),
            Kind::ScaledPoint(1, 3, 0.),
            Kind::RotatedRay(5, 3, f64::NAN),
            Kind::CalculatedAngleRotatedPoint(1, 3, 2),
            Kind::CalculatedRatioScaledPoint(1, 3, 2),
            Kind::CalculatedAngleRotatedRay(6, 3, 10),
            Kind::CalculatedRatioScaledPoint(0, 3, 11),
        ] {
            let mut document = parents();
            let id = document.add(kind);
            for is_new in [true, false] {
                assert!(validate_transform(&document, &document.objects[id - 1], is_new).is_err());
            }
        }
    }

    #[test]
    fn undefined_marked_values_and_numeric_zero_dilation_preserve_distinct_policies() {
        let mut document = parents();
        let collapsed = document.add(Kind::ThreePointArc(1, 1, 2));
        let distance = document.add(Kind::ArcLength(collapsed));
        assert_eq!(document.distance_value_pixels(distance), None);
        for kind in [
            Kind::MarkedDistanceTranslatedPoint(1, distance, 0.),
            Kind::MarkedDistanceTranslatedSegment(4, distance, 0.),
            Kind::MixedRectangularTranslatedCircle(7, distance, 1., true),
            Kind::RectangularMarkedTranslatedPolygon(8, distance, distance),
        ] {
            let mut candidate = document.clone();
            let id = candidate.add(kind);
            assert!(candidate.evaluate_transform(id).is_err());
            validate_transform(&candidate, &candidate.objects[id - 1], true).unwrap();
            candidate.can_save().unwrap();
        }
        document.set_parameter_value(11, 0.).unwrap();
        for kind in [
            Kind::CalculatedRatioScaledPoint(1, 3, 11),
            Kind::CalculatedRatioScaledSegment(4, 3, 11),
            Kind::CalculatedRatioScaledCircle(7, 3, 11),
            Kind::CalculatedRatioScaledPolygon(8, 3, 11),
            Kind::CalculatedRatioScaledRay(5, 3, 11),
            Kind::CalculatedRatioScaledLine(6, 3, 11),
            Kind::ScaledRay(5, 3, 0.),
            Kind::ScaledLine(6, 3, 0.),
        ] {
            let mut candidate = document.clone();
            let id = candidate.add(kind);
            validate_transform(&candidate, &candidate.objects[id - 1], true).unwrap();
            candidate.can_save().unwrap();
        }
    }
}
