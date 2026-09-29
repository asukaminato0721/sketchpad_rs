//! Shared native transform schemas with independently validated output typing.
//! Appearance, caches and unknown auxiliary records remain in the preserving serializer.
use super::decode::{cartesian_distance_scale, polar_distance_scale, transform_angle_degrees};
use super::kind::{
    is_arc_kind, is_point_kind, is_segment_kind, is_straight_kind, polygon_vertex_count,
};
use super::records::{f64_at, polygon_subtype, u32_at};
use super::transform_model::{
    DisplacementExpr, ElementType, NativeTransformFlavor, NumericExpr, Transform,
    TransformDefinition,
};
use super::{LineMode, Object};
use crate::table_records::NumericUnit;

pub(super) struct DecodeContext {
    pub custom_polar_translation: bool,
    pub source_is_generic_polygon: bool,
    pub legacy_angle_rotation_layout: bool,
}

pub(super) fn decode_native_transform(
    objects: &[Object],
    page: usize,
    object_id: usize,
    header: &[u8],
    references: &[usize],
    constraint: Option<&[u8]>,
    context: DecodeContext,
) -> Option<TransformDefinition> {
    if header.len() != 28 {
        return None;
    }
    let code = u16::from_le_bytes(header[..2].try_into().ok()?);
    let element = match header[6..10] {
        [1, 0, 4 | 89, 0] => ElementType::Point,
        [2, 0, 14, 0] => ElementType::Segment,
        [3, 0, 28, 0] => ElementType::Circle,
        [5, 0, 16, 0] => ElementType::Disk,
        [4, 0, _, _] => ElementType::Polygon,
        [12, 0, 67, 0] => ElementType::Arc,
        [13, 0, 68, 0] => ElementType::Sector,
        [13, 0, 69, 0] => ElementType::CircularSegment,
        [16, 0, 78, 0] => ElementType::Image,
        [2, 0, 13, 0] => ElementType::Ray,
        [2, 0, 12, 0] => ElementType::Line,
        _ => return None,
    };
    // Keep this region type bounded to independently observed native flavors.
    if element == ElementType::CircularSegment && !matches!(code, 19 | 27 | 30 | 34) {
        return None;
    }
    let get = |id: usize| {
        objects
            .get(id.checked_sub(1)?)
            .filter(|o| id < object_id && o.id == id && o.page == page && !o.deleted)
    };
    if references.iter().any(|id| get(*id).is_none()) {
        return None;
    }
    let source = *references.first()?;
    let source_kind = &get(source)?.kind;
    let source_matches = match element {
        ElementType::Point => is_point_kind(source_kind),
        ElementType::Arc => is_arc_kind(source_kind),
        ElementType::Segment => is_segment_kind(source_kind),
        ElementType::Circle => source_kind.is_circle(),
        ElementType::Disk => source_kind.is_circle_interior(),
        ElementType::Polygon => {
            source_kind.is_polygon()
                && (polygon_vertex_count(objects, source).and_then(polygon_subtype)
                    == Some(u16::from_le_bytes([header[8], header[9]]))
                    || (header[8..10] == [15, 0] && context.source_is_generic_polygon))
        }
        ElementType::Sector => source_kind.is_sector(),
        ElementType::CircularSegment => source_kind.is_circular_segment(),
        ElementType::Image => matches!(source_kind, super::Kind::Image { .. }),
        ElementType::Ray => source_kind.fixed_transform_straight_mode() == Some(LineMode::Ray),
        ElementType::Line => {
            if matches!(code, 27 | 29) {
                source_kind.rotation_transform_straight_mode() == Some(LineMode::Line)
            } else {
                source_kind.fixed_transform_straight_mode() == Some(LineMode::Line)
            }
        }
    };
    if !source_matches {
        return None;
    }
    let point = |id: usize| get(id).is_some_and(|o| is_point_kind(&o.kind));
    let numeric = |id: usize, unit| get(id).is_some_and(|o| o.kind.numeric_unit() == Some(unit));
    let quantity = |id: usize| get(id).is_some_and(|o| o.kind.numeric_unit().is_some());
    let distance = |id: usize| get(id).is_some_and(|o| o.kind.is_marked_distance_value());
    let literal = |magnitude, unit, scale| NumericExpr::Literal {
        magnitude,
        unit,
        scale,
    };
    let value = |id, unit| NumericExpr::ObjectValue { id, unit };
    let cartesian = |horizontal, vertical| {
        Transform::Translate(DisplacementExpr::Cartesian {
            horizontal,
            vertical,
        })
    };
    let payload = |length: usize| {
        constraint.filter(|p| p.len() == length && p[..4] == (code as u32).to_le_bytes())
    };
    use NativeTransformFlavor as F;
    let (transform, flavor) = match (code, references) {
        (26, [_, angle, distance])
            if context.custom_polar_translation
                && constraint.is_none()
                && numeric(*angle, NumericUnit::Degrees)
                && numeric(*distance, NumericUnit::Length) =>
        {
            (
                Transform::Translate(DisplacementExpr::Polar {
                    angle: value(*angle, NumericUnit::Degrees),
                    distance: value(*distance, NumericUnit::Length),
                }),
                F::DualMarkedPolar,
            )
        }
        (16, [_, start, end]) if constraint.is_none() && point(*start) && point(*end) => (
            Transform::Translate(DisplacementExpr::Vector {
                start: *start,
                end: *end,
            }),
            F::Vector,
        ),
        (17, [_]) => {
            let p = payload(52)?;
            let horizontal_scale = cartesian_distance_scale(p, 4, 12, 20)?;
            let vertical_scale = cartesian_distance_scale(p, 24, 32, 40)?;
            (
                cartesian(
                    literal(f64_at(p, 12), NumericUnit::Length, horizontal_scale),
                    literal(f64_at(p, 32), NumericUnit::Length, vertical_scale),
                ),
                F::FixedCartesian,
            )
        }
        (18 | 19, [_, measured]) if distance(*measured) => {
            let p = payload(28)?;
            if u32_at(p, 20) != 1
                || p[24..28] != [0; 4]
                || !f64_at(p, 4).is_finite()
                || !f64_at(p, 12).is_finite()
                || (f64_at(p, 4) - f64_at(p, 12) * 37.79527559055118).abs() >= 1e-5
            {
                return None;
            }
            let fixed = literal(f64_at(p, 12), NumericUnit::Length, 37.79527559055118);
            let marked = value(*measured, NumericUnit::Length);
            if code == 18 {
                (cartesian(fixed, marked), F::FixedHorizontal)
            } else {
                (cartesian(marked, fixed), F::FixedVertical)
            }
        }
        (20, [_, horizontal, vertical])
            if constraint.is_none() && distance(*horizontal) && distance(*vertical) =>
        {
            (
                cartesian(
                    value(*horizontal, NumericUnit::Length),
                    value(*vertical, NumericUnit::Length),
                ),
                F::MarkedCartesian,
            )
        }
        (21, [_]) => {
            let p = payload(60)?;
            let angle = transform_angle_degrees(p)?;
            let scale = polar_distance_scale(p)?;
            (
                Transform::Translate(DisplacementExpr::Polar {
                    angle: literal(angle, NumericUnit::Degrees, 1.),
                    distance: literal(f64_at(p, 40), NumericUnit::Length, scale),
                }),
                F::FixedPolar,
            )
        }
        (24, [_, measured]) if distance(*measured) => {
            let p = payload(36)?;
            let angle = transform_angle_degrees(p)?;
            (
                Transform::Translate(DisplacementExpr::Polar {
                    angle: literal(angle, NumericUnit::Degrees, 1.),
                    distance: value(*measured, NumericUnit::Length),
                }),
                F::MarkedPolar,
            )
        }
        (27, [_, center]) if point(*center) => {
            let p = payload(36)?;
            (
                Transform::Rotate {
                    center: *center,
                    angle: literal(transform_angle_degrees(p)?, NumericUnit::Degrees, 1.),
                },
                F::FixedRotation,
            )
        }
        (28, [_, center, first, vertex, last])
            if constraint.is_none() && [*center, *first, *vertex, *last].into_iter().all(point) =>
        {
            (
                Transform::Rotate {
                    center: *center,
                    angle: NumericExpr::DirectedAngle {
                        first: *first,
                        vertex: *vertex,
                        last: *last,
                    },
                },
                F::PointAngleRotation,
            )
        }
        (29, [_, center, angle]) if constraint.is_none() && point(*center) && quantity(*angle) => (
            Transform::Rotate {
                center: *center,
                angle: value(*angle, NumericUnit::Degrees),
            },
            F::NumericRotation,
        ),
        (119, [_, center, angle])
            if context.legacy_angle_rotation_layout
                && constraint.is_none()
                && point(*center)
                && numeric(*angle, NumericUnit::Degrees) =>
        {
            (
                Transform::Rotate {
                    center: *center,
                    angle: value(*angle, NumericUnit::Degrees),
                },
                F::NumericRotation,
            )
        }
        (30, [_, center]) if point(*center) => {
            let p = payload(12)?;
            let ratio = f64_at(p, 4);
            if !ratio.is_finite() {
                return None;
            }
            (
                Transform::Dilate {
                    center: *center,
                    ratio: literal(ratio, NumericUnit::Scalar, 1.),
                },
                F::FixedDilation,
            )
        }
        (32, [_, center, origin, base, target])
            if constraint.is_none()
                && [*center, *origin, *base, *target].into_iter().all(point) =>
        {
            (
                Transform::Dilate {
                    center: *center,
                    ratio: NumericExpr::DistanceRatio {
                        origin: *origin,
                        base: *base,
                        target: *target,
                    },
                },
                F::PointRatioDilation,
            )
        }
        (33, [_, center, ratio]) if constraint.is_none() && point(*center) && quantity(*ratio) => (
            Transform::Dilate {
                center: *center,
                ratio: value(*ratio, NumericUnit::Scalar),
            },
            F::NumericDilation,
        ),
        (34, [_, mirror])
            if constraint.is_none() && get(*mirror).is_some_and(|o| is_straight_kind(&o.kind)) =>
        {
            (Transform::Reflect { mirror: *mirror }, F::Reflection)
        }
        _ => return None,
    };
    TransformDefinition::new(source, element, transform, flavor).ok()
}
