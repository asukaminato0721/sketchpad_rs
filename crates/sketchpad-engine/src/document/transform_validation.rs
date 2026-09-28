//! Save policies for shared transform definitions. Geometry and serialization
//! share definitions, while authoring capabilities retain their native proofs.
use super::kind::{is_arc_kind, is_point_kind, is_segment_kind, polygon_vertex_count};
use super::transform_model::{
    DisplacementExpr, ElementType, NativeLiteralRole, NativeTransformFlavor, NumericExpr,
    Transform, TransformDefinition,
};
use super::{Document, Kind, LineMode, Object};
use crate::table_records::NumericUnit;

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
    let flavor = definition.wire_flavor();
    match definition.element() {
        E::Point => is_point_kind(&source.kind),
        E::Segment => {
            if matches!(
                flavor,
                F::Vector
                    | F::MarkedPolar
                    | F::FixedPolar
                    | F::Reflection
                    | F::FixedRotation
                    | F::FixedDilation
                    | F::NumericRotation
                    | F::NumericDilation
            ) {
                is_segment_kind(&source.kind)
            } else {
                matches!(source.kind, Kind::Segment(..))
            }
        }
        E::Ray | E::Line => {
            source.kind.fixed_transform_straight_mode()
                == Some(if definition.element() == E::Ray {
                    LineMode::Ray
                } else {
                    LineMode::Line
                })
        }
        E::Circle => {
            if matches!(
                flavor,
                F::Vector
                    | F::MarkedPolar
                    | F::FixedPolar
                    | F::Reflection
                    | F::FixedRotation
                    | F::FixedDilation
                    | F::PointRatioDilation
                    | F::NumericRotation
                    | F::NumericDilation
            ) {
                source.kind.is_circle()
            } else {
                matches!(source.kind, Kind::Circle(..) | Kind::CircleBySegment(..))
            }
        }
        E::Arc => is_arc_kind(&source.kind),
        E::Sector => match flavor {
            F::Reflection => {
                matches!(source.kind, Kind::SectorBoundary(_))
                    && definition
                        .native_references()
                        .get(1)
                        .and_then(|id| document.objects.get(id.saturating_sub(1)))
                        .is_some_and(|mirror| matches!(mirror.kind, Kind::Segment(..)))
            }
            F::FixedRotation => {
                (matches!(source.kind, Kind::SectorBoundary(_))
                    || matches!(&source.kind, Kind::Transformed(parent)
                        if parent.element() == E::Sector
                            && matches!(parent.wire_flavor(), F::Reflection | F::FixedRotation)))
                    && definition
                        .native_references()
                        .get(1)
                        .and_then(|id| document.objects.get(id.saturating_sub(1)))
                        .is_some_and(|center| matches!(center.kind, Kind::Free(_)))
            }
            _ => false,
        },
        E::Polygon => {
            if matches!(
                flavor,
                F::Vector
                    | F::MarkedPolar
                    | F::FixedPolar
                    | F::Reflection
                    | F::NumericRotation
                    | F::NumericDilation
            ) {
                source.kind.is_polygon()
                    && polygon_vertex_count(&document.objects, definition.source()).is_some()
            } else {
                matches!(source.kind, Kind::Polygon(_))
            }
        }
        E::Disk => false,
    }
}

fn expression_parents_supported(
    document: &Document,
    expression: &NumericExpr,
    position_only: bool,
) -> bool {
    let point = |id: usize| {
        id.checked_sub(1)
            .and_then(|index| document.objects.get(index))
            .is_some_and(|parent| {
                !parent.deleted
                    && if position_only {
                        document.position(id).is_some()
                    } else {
                        is_point_kind(&parent.kind)
                    }
            })
    };
    match *expression {
        NumericExpr::Literal { .. } => true,
        NumericExpr::ObjectValue { id, unit } => id
            .checked_sub(1)
            .and_then(|index| document.objects.get(index))
            .is_some_and(|parent| {
                !parent.deleted
                    && if unit == NumericUnit::Length {
                        parent.kind.is_marked_distance_value()
                    } else {
                        parent.kind.numeric_unit() == Some(unit)
                    }
            }),
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
    let position_only = definition.element() == E::Polygon
        && !matches!(
            definition.wire_flavor(),
            F::NumericRotation | F::NumericDilation
        );
    let point = |id: usize| {
        id.checked_sub(1)
            .and_then(|index| document.objects.get(index))
            .is_some_and(|parent| {
                !parent.deleted
                    && if position_only {
                        document.position(id).is_some()
                    } else {
                        is_point_kind(&parent.kind)
                    }
            })
    };
    match definition.transform() {
        Transform::Translate(DisplacementExpr::Cartesian {
            horizontal,
            vertical,
        }) => {
            expression_parents_supported(document, horizontal, false)
                && expression_parents_supported(document, vertical, false)
        }
        Transform::Translate(DisplacementExpr::Polar { angle, distance }) => {
            expression_parents_supported(document, angle, false)
                && expression_parents_supported(document, distance, false)
        }
        Transform::Translate(DisplacementExpr::Vector { start, end }) => {
            point(*start) && point(*end)
        }
        Transform::Rotate { center, angle } => {
            point(*center) && expression_parents_supported(document, angle, position_only)
        }
        Transform::Dilate { center, ratio } => {
            let center_valid = if definition.element() == E::Circle
                && definition.wire_flavor() == F::FixedDilation
            {
                document.position(*center).is_some()
            } else {
                point(*center)
            };
            center_valid && expression_parents_supported(document, ratio, position_only)
        }
        // Existing reflection supports any available straight mirror, including
        // the same axis forms accepted by line_points. Do not narrow it here.
        Transform::Reflect { mirror } => document.line_points(*mirror).is_some(),
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
        || !source_supported(document, &definition)
        || !operation_parents_supported(document, &definition)
    {
        return Err(invalid());
    }
    if !is_new {
        return Ok(());
    }

    // These marked families validate categories, not availability. Undefined
    // live measurements/source geometry remain savable, as before migration.
    if matches!(
        flavor,
        F::MarkedPolar | F::MarkedCartesian | F::FixedHorizontal | F::FixedVertical
    ) {
        return Ok(());
    }
    let straight = matches!(definition.element(), E::Ray | E::Line);
    if numeric {
        match definition.transform() {
            Transform::Rotate {
                center,
                angle: NumericExpr::ObjectValue { id, .. },
            } => {
                if document.position(*center).is_none() {
                    return Err(invalid());
                }
                if straight
                    && (document.numeric_value(*id).is_none()
                        || document.straight_transform_has_suppressed_ancestor(definition.source()))
                {
                    return Ok(());
                }
            }
            Transform::Dilate {
                center,
                ratio: NumericExpr::ObjectValue { id, .. },
            } => {
                if document.position(*center).is_none() {
                    return Err(invalid());
                }
                let factor = document.unitless_numeric_value(*id);
                if straight
                    && (factor.is_none()
                        || factor == Some(0.)
                        || document.straight_transform_has_suppressed_ancestor(definition.source()))
                {
                    return Ok(());
                }
                if factor.is_none() {
                    return Err(invalid());
                }
                // Existing numeric dilation permits collapsed or undefined
                // segment/circle/interior targets; point targets remain defined.
                if matches!(
                    definition.element(),
                    E::Segment | E::Circle | E::Arc | E::Polygon
                ) {
                    return Ok(());
                }
            }
            _ => return Err(invalid()),
        }
    } else if straight {
        if let Transform::Rotate { center, .. } | Transform::Dilate { center, .. } =
            definition.transform()
        {
            if document.position(*center).is_none() {
                return Err(invalid());
            }
        }
        let zero = matches!(definition.transform(), Transform::Dilate {
            ratio: NumericExpr::Literal { magnitude, .. }, .. } if *magnitude == 0.);
        // Native retains typed straight chains when a live source loses its
        // direction, including coincident primitive endpoints and undefined
        // derived paths. Parent roles and operation parameters were checked above.
        let undefined_source = document
            .line_points(definition.source())
            .is_none_or(|(start, end, _)| start == end);
        if zero
            || undefined_source
            || document.straight_transform_has_suppressed_ancestor(definition.source())
        {
            return Ok(());
        }
    }
    document
        .evaluate_transform(object.id)
        .map(|_| ())
        .map_err(|_| invalid())
}

#[cfg(test)]
mod tests {
    use super::super::Point;
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
    fn source_capabilities_remain_specific_to_native_flavor_and_import_policy() {
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
            assert!(validate_transform(&candidate, object, true).is_err());
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
            Kind::CalculatedAngleRotatedPoint(1, 3, 11),
            Kind::CalculatedRatioScaledPoint(1, 3, 10),
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
