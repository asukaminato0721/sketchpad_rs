//! Legacy Kind construction compatibility over the shared semantic IR.
use super::Kind;
use crate::table_records::NumericUnit;
pub use sketchpad_ir::transform_model::*;

fn literal(magnitude: f64, unit: NumericUnit, scale: f64) -> NumericExpr {
    NumericExpr::Literal {
        magnitude,
        unit,
        scale,
    }
}
fn value(id: usize, unit: NumericUnit) -> NumericExpr {
    NumericExpr::ObjectValue { id, unit }
}
// Compatibility construction names create the single generic storage variant.
// Each native flavor defines its expression once for all supported elements.
macro_rules! native_transform_constructor {
    ($name:ident, $element:ident, $element_binding:ident, $parameters:tt, $definition:expr) => {
        impl Kind {
            #[allow(non_snake_case)]
            pub fn $name $parameters -> Self {
                let $element_binding = ElementType::$element;
                let (source, element, transform, flavor) = $definition;
                Self::Transformed(TransformDefinition::from_compatibility_parts(source, element, transform, flavor))
            }
        }
    };
}
macro_rules! native_transform_constructors {
    ($element_binding:ident; $parameters:tt; $definition:expr; $( $name:ident => $element:ident ),+ $(,)?) => {
        $(native_transform_constructor!($name, $element, $element_binding, $parameters, $definition);)+
    };
}
impl Kind {
    pub fn transform_definition(&self) -> Option<TransformDefinition> {
        match self {
            Self::Transformed(definition) => Some(definition.clone()),
            _ => None,
        }
    }
}

native_transform_constructors!(element_type; (source: usize, a: f64, u: f64, d: f64);
(
            source,
            element_type,
            Transform::Translate(DisplacementExpr::Polar {
                angle: literal(a, NumericUnit::Degrees, 1.),
                distance: literal(d, NumericUnit::Length, u),
            }),
            NativeTransformFlavor::FixedPolar,
        );
    PolarTranslatedPoint => Point,
    PolarTranslatedSegment => Segment,
    PolarTranslatedCircle => Circle,
    PolarTranslatedPolygon => Polygon
);

native_transform_constructors!(element_type; (source: usize, xu: f64, x: f64, yu: f64, y: f64,);
(
            source,
            element_type,
            Transform::Translate(DisplacementExpr::Cartesian {
                horizontal: literal(x, NumericUnit::Length, xu),
                vertical: literal(y, NumericUnit::Length, yu),
            }),
            NativeTransformFlavor::FixedCartesian,
        );
    FixedRectangularTranslatedPoint => Point,
    FixedRectangularTranslatedSegment => Segment,
    FixedRectangularTranslatedCircle => Circle,
    FixedRectangularTranslatedPolygon => Polygon
);

native_transform_constructors!(element_type; (source: usize, d: usize, a: f64);
(
            source,
            element_type,
            Transform::Translate(DisplacementExpr::Polar {
                angle: literal(a, NumericUnit::Degrees, 1.),
                distance: value(d, NumericUnit::Length),
            }),
            NativeTransformFlavor::MarkedPolar,
        );
    MarkedDistanceTranslatedPoint => Point,
    MarkedDistanceTranslatedSegment => Segment,
    MarkedDistanceTranslatedCircle => Circle,
    MarkedDistanceTranslatedPolygon => Polygon
);

native_transform_constructors!(element_type; (source: usize, d: usize, f: f64, h: bool);
(
            source,
            element_type,
            Transform::Translate(DisplacementExpr::Cartesian {
                horizontal: if h {
                    literal(f, NumericUnit::Length, 37.79527559055118)
                } else {
                    value(d, NumericUnit::Length)
                },
                vertical: if h {
                    value(d, NumericUnit::Length)
                } else {
                    literal(f, NumericUnit::Length, 37.79527559055118)
                },
            }),
            if h {
                NativeTransformFlavor::FixedHorizontal
            } else {
                NativeTransformFlavor::FixedVertical
            },
        );
    MixedRectangularTranslatedPoint => Point,
    MixedRectangularTranslatedSegment => Segment,
    MixedRectangularTranslatedCircle => Circle,
    MixedRectangularTranslatedPolygon => Polygon
);

native_transform_constructors!(element_type; (source: usize, h: usize, v: usize);
(
            source,
            element_type,
            Transform::Translate(DisplacementExpr::Cartesian {
                horizontal: value(h, NumericUnit::Length),
                vertical: value(v, NumericUnit::Length),
            }),
            NativeTransformFlavor::MarkedCartesian,
        );
    RectangularMarkedTranslatedPoint => Point,
    RectangularMarkedTranslatedSegment => Segment,
    RectangularMarkedTranslatedCircle => Circle,
    RectangularMarkedTranslatedPolygon => Polygon
);

native_transform_constructors!(element_type; (source: usize, start: usize, end: usize);
(
            source,
            element_type,
            Transform::Translate(DisplacementExpr::Vector { start, end }),
            NativeTransformFlavor::Vector,
        );
    VectorTranslatedPoint => Point,
    VectorTranslatedSegment => Segment,
    VectorTranslatedCircle => Circle,
    VectorTranslatedPolygon => Polygon
);

native_transform_constructors!(element_type; (source: usize, center: usize, first: usize, vertex: usize, last: usize,);
(
            source,
            element_type,
            Transform::Rotate {
                center,
                angle: NumericExpr::DirectedAngle {
                    first,
                    vertex,
                    last,
                },
            },
            NativeTransformFlavor::PointAngleRotation,
        );
    AngleRotatedPoint => Point,
    AngleRotatedSegment => Segment,
    AngleRotatedCircle => Circle,
    AngleRotatedPolygon => Polygon
);

native_transform_constructors!(element_type; (source: usize, center: usize, origin: usize, base: usize, target: usize,);
(
            source,
            element_type,
            Transform::Dilate {
                center,
                ratio: NumericExpr::DistanceRatio {
                    origin,
                    base,
                    target,
                },
            },
            NativeTransformFlavor::PointRatioDilation,
        );
    RatioScaledPoint => Point,
    RatioScaledSegment => Segment,
    RatioScaledCircle => Circle,
    RatioScaledPolygon => Polygon
);

native_transform_constructors!(element_type; (source: usize, mirror: usize);
(
            source,
            element_type,
            Transform::Reflect { mirror },
            NativeTransformFlavor::Reflection,
        );
    ReflectedPoint => Point,
    ReflectedSegment => Segment,
    ReflectedCircle => Circle,
    ReflectedPolygon => Polygon
);

native_transform_constructors!(element_type; (source: usize, center: usize, factor: f64);
(
            source,
            element_type,
            Transform::Dilate {
                center,
                ratio: literal(factor, NumericUnit::Scalar, 1.),
            },
            NativeTransformFlavor::FixedDilation,
        );
    ScaledPoint => Point,
    ScaledSegment => Segment,
    ScaledRay => Ray,
    ScaledLine => Line,
    ScaledCircle => Circle,
    ScaledPolygon => Polygon
);

native_transform_constructors!(element_type; (source: usize, center: usize, angle: f64);
(
            source,
            element_type,
            Transform::Rotate {
                center,
                angle: literal(angle, NumericUnit::Degrees, 1.),
            },
            NativeTransformFlavor::FixedRotation,
        );
    RotatedPoint => Point,
    RotatedSegment => Segment,
    RotatedRay => Ray,
    RotatedLine => Line,
    RotatedCircle => Circle,
    RotatedPolygon => Polygon
);

native_transform_constructors!(element_type; (source: usize, center: usize, ratio: usize);
(
            source,
            element_type,
            Transform::Dilate {
                center,
                ratio: value(ratio, NumericUnit::Scalar),
            },
            NativeTransformFlavor::NumericDilation,
        );
    CalculatedRatioScaledPoint => Point,
    CalculatedRatioScaledSegment => Segment,
    CalculatedRatioScaledRay => Ray,
    CalculatedRatioScaledLine => Line,
    CalculatedRatioScaledCircle => Circle,
    CalculatedRatioScaledPolygon => Polygon
);

native_transform_constructors!(element_type; (source: usize, center: usize, angle: usize);
(
            source,
            element_type,
            Transform::Rotate {
                center,
                angle: value(angle, NumericUnit::Degrees),
            },
            NativeTransformFlavor::NumericRotation,
        );
    CalculatedAngleRotatedPoint => Point,
    CalculatedAngleRotatedSegment => Segment,
    CalculatedAngleRotatedRay => Ray,
    CalculatedAngleRotatedLine => Line,
    CalculatedAngleRotatedCircle => Circle,
    CalculatedAngleRotatedPolygon => Polygon
);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_sixty_construction_helpers_preserve_dependencies_and_remapping() {
        let kinds = [
            Kind::PolarTranslatedPoint(1, -0., 17., 2.),
            Kind::PolarTranslatedSegment(1, -0., 17., 2.),
            Kind::PolarTranslatedCircle(1, -0., 17., 2.),
            Kind::PolarTranslatedPolygon(1, -0., 17., 2.),
            Kind::FixedRectangularTranslatedPoint(1, 17., -0., 23., 2.),
            Kind::FixedRectangularTranslatedSegment(1, 17., -0., 23., 2.),
            Kind::FixedRectangularTranslatedCircle(1, 17., -0., 23., 2.),
            Kind::FixedRectangularTranslatedPolygon(1, 17., -0., 23., 2.),
            Kind::MarkedDistanceTranslatedPoint(1, 3, -0.),
            Kind::MarkedDistanceTranslatedSegment(1, 3, -0.),
            Kind::MarkedDistanceTranslatedCircle(1, 3, -0.),
            Kind::MarkedDistanceTranslatedPolygon(1, 3, -0.),
            Kind::MixedRectangularTranslatedPoint(1, 3, -0., true),
            Kind::MixedRectangularTranslatedSegment(1, 3, -0., true),
            Kind::MixedRectangularTranslatedCircle(1, 3, -0., true),
            Kind::MixedRectangularTranslatedPolygon(1, 3, -0., true),
            Kind::RectangularMarkedTranslatedPoint(1, 3, 4),
            Kind::RectangularMarkedTranslatedSegment(1, 3, 4),
            Kind::RectangularMarkedTranslatedCircle(1, 3, 4),
            Kind::RectangularMarkedTranslatedPolygon(1, 3, 4),
            Kind::VectorTranslatedPoint(1, 3, 4),
            Kind::VectorTranslatedSegment(1, 3, 4),
            Kind::VectorTranslatedCircle(1, 3, 4),
            Kind::VectorTranslatedPolygon(1, 3, 4),
            Kind::AngleRotatedPoint(1, 2, 3, 4, 5),
            Kind::AngleRotatedSegment(1, 2, 3, 4, 5),
            Kind::AngleRotatedCircle(1, 2, 3, 4, 5),
            Kind::AngleRotatedPolygon(1, 2, 3, 4, 5),
            Kind::RatioScaledPoint(1, 2, 3, 4, 5),
            Kind::RatioScaledSegment(1, 2, 3, 4, 5),
            Kind::RatioScaledCircle(1, 2, 3, 4, 5),
            Kind::RatioScaledPolygon(1, 2, 3, 4, 5),
            Kind::ReflectedPoint(1, 2),
            Kind::ReflectedSegment(1, 2),
            Kind::ReflectedCircle(1, 2),
            Kind::ReflectedPolygon(1, 2),
            Kind::ScaledPoint(1, 2, -0.),
            Kind::ScaledSegment(1, 2, -0.),
            Kind::ScaledRay(1, 2, -0.),
            Kind::ScaledLine(1, 2, -0.),
            Kind::ScaledCircle(1, 2, -0.),
            Kind::ScaledPolygon(1, 2, -0.),
            Kind::RotatedPoint(1, 2, -0.),
            Kind::RotatedSegment(1, 2, -0.),
            Kind::RotatedRay(1, 2, -0.),
            Kind::RotatedLine(1, 2, -0.),
            Kind::RotatedCircle(1, 2, -0.),
            Kind::RotatedPolygon(1, 2, -0.),
            Kind::CalculatedRatioScaledPoint(1, 2, 3),
            Kind::CalculatedRatioScaledSegment(1, 2, 3),
            Kind::CalculatedRatioScaledRay(1, 2, 3),
            Kind::CalculatedRatioScaledLine(1, 2, 3),
            Kind::CalculatedRatioScaledCircle(1, 2, 3),
            Kind::CalculatedRatioScaledPolygon(1, 2, 3),
            Kind::CalculatedAngleRotatedPoint(1, 2, 3),
            Kind::CalculatedAngleRotatedSegment(1, 2, 3),
            Kind::CalculatedAngleRotatedRay(1, 2, 3),
            Kind::CalculatedAngleRotatedLine(1, 2, 3),
            Kind::CalculatedAngleRotatedCircle(1, 2, 3),
            Kind::CalculatedAngleRotatedPolygon(1, 2, 3),
        ];
        assert_eq!(kinds.len(), 60);
        let ids = [0, 11, 12, 13, 14, 15];
        for kind in kinds {
            let definition = kind.transform_definition().unwrap();
            assert_eq!(definition.dependencies(), kind.dependencies(), "{kind:?}");
            let remapped = definition.remap_checked(&ids).unwrap();
            assert_eq!(Some(remapped), kind.remap(&ids).transform_definition());
            assert!(definition.remap_checked(&[0, 0]).is_err());
        }
    }
    #[test]
    fn malformed_references_retain_transform_identity_and_dependencies() {
        let kind = Kind::CalculatedAngleRotatedRay(0, 2, 0);
        let definition = kind.transform_definition().unwrap();
        assert_eq!(definition.element(), ElementType::Ray);
        assert_eq!(definition.dependencies(), [0, 2, 0]);
        assert!(
            TransformDefinition::new(
                0,
                definition.element(),
                definition.transform().clone(),
                definition.wire_flavor()
            )
            .is_err()
        );
    }

    #[test]
    fn wire_flavors_keep_literal_fields_and_repeated_references() {
        let d = Kind::FixedRectangularTranslatedPoint(1, 17., -0., 23., 2.)
            .transform_definition()
            .unwrap();
        assert_eq!(d.wire_flavor().code(), 17);
        let values = d.native_literals();
        assert_eq!(
            values.iter().map(|v| v.value.to_bits()).collect::<Vec<_>>(),
            [
                17f64.to_bits(),
                (-0f64).to_bits(),
                23f64.to_bits(),
                2f64.to_bits()
            ]
        );
        let d = Kind::PolarTranslatedCircle(1, -0., 17., 2.)
            .transform_definition()
            .unwrap();
        assert_eq!(
            d.native_literals()
                .iter()
                .map(|v| v.value.to_bits())
                .collect::<Vec<_>>(),
            [(-0f64).to_bits(), 17f64.to_bits(), 2f64.to_bits()]
        );
        let d = Kind::AngleRotatedPoint(1, 1, 2, 1, 2)
            .transform_definition()
            .unwrap();
        assert_eq!(d.native_references(), [1, 1, 2, 1, 2]);
        assert_eq!(d.wire_flavor().code(), 28);
        assert!(
            Kind::Free(super::super::Point { x: 0., y: 0. })
                .transform_definition()
                .is_none()
        );
    }
    #[test]
    fn checked_constructor_does_not_enable_unproved_element_flavor_or_units() {
        let angle = NumericExpr::ObjectValue {
            id: 3,
            unit: NumericUnit::Scalar,
        };
        assert!(
            TransformDefinition::new(
                1,
                ElementType::Ray,
                Transform::Rotate { center: 2, angle },
                NativeTransformFlavor::NumericRotation
            )
            .is_err()
        );
        assert!(
            TransformDefinition::new(
                1,
                ElementType::Line,
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: NumericExpr::Literal {
                        magnitude: 1.,
                        unit: NumericUnit::Length,
                        scale: 37.79527559055118
                    },
                    vertical: NumericExpr::Literal {
                        magnitude: 1.,
                        unit: NumericUnit::Length,
                        scale: 37.79527559055118
                    },
                }),
                NativeTransformFlavor::FixedCartesian
            )
            .is_err()
        );
        assert!(
            TransformDefinition::new(
                1,
                ElementType::Ray,
                Transform::Rotate {
                    center: 2,
                    angle: NumericExpr::DirectedAngle {
                        first: 2,
                        vertex: 3,
                        last: 4
                    }
                },
                NativeTransformFlavor::PointAngleRotation
            )
            .is_err()
        );
        assert!(
            TransformDefinition::new(
                0,
                ElementType::Point,
                Transform::Reflect { mirror: 2 },
                NativeTransformFlavor::Reflection
            )
            .is_err()
        );
    }
}
