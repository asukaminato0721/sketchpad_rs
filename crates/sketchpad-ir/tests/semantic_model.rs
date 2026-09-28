use sketchpad_ir::{
    NumericUnit,
    transform_model::{
        DisplacementExpr, ElementType, NativeLiteralRole, NativeTransformFlavor, NumericExpr,
        Transform, TransformDefinition,
    },
};

#[test]
fn quantity_dimensions_round_trip_without_native_subtypes() {
    for (length, degrees) in [(0, 0), (1, 0), (2, 0), (0, 1), (-3, 2), (3, -2)] {
        let unit = NumericUnit::from_dimensions(length, degrees);
        assert_eq!(unit.dimensions(), (length, degrees));
    }
    assert_eq!(NumericUnit::from_dimensions(2, 0), NumericUnit::Area);
    assert_eq!(
        NumericUnit::Compound {
            length: 3,
            degrees: -2
        }
        .suffix(),
        " cm³°⁻²"
    );
    assert_eq!(NumericUnit::Scalar.suffix(), "");
}

#[test]
fn checked_transform_retains_signed_zero_and_ordered_references() {
    let rotation = TransformDefinition::new(
        4,
        ElementType::Point,
        Transform::Rotate {
            center: 2,
            angle: NumericExpr::Literal {
                magnitude: -0.0,
                unit: NumericUnit::Degrees,
                scale: 1.,
            },
        },
        NativeTransformFlavor::FixedRotation,
    )
    .unwrap();
    let literals = rotation.native_literals();
    assert_eq!(literals[0].role, NativeLiteralRole::AngleDegrees);
    assert_eq!(literals[0].value.to_bits(), (-0.0_f64).to_bits());
    assert_eq!(rotation.native_references(), [4, 2]);
    assert_eq!(
        rotation
            .remap_checked(&[0, 10, 20, 30, 40])
            .unwrap()
            .native_references(),
        [40, 20]
    );
    assert!(rotation.remap_checked(&[0, 10, 0, 30, 40]).is_err());
}

#[test]
fn compatibility_adapter_preserves_unresolved_and_repeated_references() {
    let definition = TransformDefinition::from_compatibility_parts(
        0,
        ElementType::Point,
        Transform::Translate(DisplacementExpr::Vector { start: 1, end: 1 }),
        NativeTransformFlavor::Vector,
    );
    assert_eq!(definition.native_references(), [0, 1, 1]);
    assert!(
        TransformDefinition::new(
            0,
            ElementType::Point,
            definition.transform().clone(),
            NativeTransformFlavor::Vector
        )
        .is_err()
    );
    assert_eq!(
        definition.remap_unchecked(&[0, 7]).native_references(),
        [0, 7, 7]
    );
}
