use super::*;
use crate::document::transform_model::NativeTransformFlavor;

#[test]
fn native_fixed_polar_source_custom_point_follows_its_live_parent() {
    for (bytes, expected, visible) in [
        (
            include_bytes!("../../../../../tests/fixtures/code26_fixedpolar76_original_native_saved.gsp")
                .as_slice(),
            Point { x: 216.624913, y: 244.071914 },
            false,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/code26_fixedpolar76_show833_synthetic_native_saved.gsp")
                .as_slice(),
            Point { x: 216.624913, y: 244.071914 },
            true,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/code26_fixedpolar76_source95_edited_synthetic_native_saved.gsp")
                .as_slice(),
            Point { x: 256.624913, y: 264.071914 },
            false,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/code26_fixedpolar76_source95_edited_show833_synthetic_native_saved.gsp")
                .as_slice(),
            Point { x: 256.624913, y: 264.071914 },
            true,
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(matches!(&document.objects[95].kind, Kind::Transformed(definition)
            if definition.wire_flavor() == NativeTransformFlavor::FixedPolar));
        assert!(matches!(&document.objects[832].kind, Kind::Transformed(definition)
            if definition.wire_flavor() == NativeTransformFlavor::DualMarkedPolar
                && definition.native_references() == [96, 832, 830]));
        let actual = document.position(833).unwrap();
        assert!((actual.x - expected.x).abs() < 0.01);
        assert!((actual.y - expected.y).abs() < 0.01);
        assert_eq!(document.is_displayed(833), visible);
        assert_eq!(document.encode().unwrap(), bytes);
    }
    assert_eq!(
        include_bytes!(
            "../../../../../tests/fixtures/code26_fixedpolar76_original_native_saved.gsp"
        ),
        include_bytes!("../../../../../tests/fixtures/code26_fixedpolar76_original_input.gsp"),
    );

    let mut edited = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/code26_fixedpolar76_original_input.gsp"),
        None,
    )
    .unwrap();
    assert!(edited.move_point(95, Point { x: 317., y: 323. }));
    let output = edited.position(833).unwrap();
    assert!((output.x - 256.624913).abs() < 0.01);
    assert!((output.y - 264.071914).abs() < 0.01);
}
