use super::*;
use crate::document::transform_model::NativeTransformFlavor;

#[test]
fn native_vector_source_custom_point_follows_its_transformed_parent() {
    for (bytes, expected, visible) in [
        (
            include_bytes!("../../../../../tests/fixtures/code26_transformed74_original_native_saved.gsp")
                .as_slice(),
            Point { x: 603.594749, y: 308.486659 },
            true,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/code26_transformed74_hide280_native_saved.gsp")
                .as_slice(),
            Point { x: 603.594749, y: 308.486659 },
            false,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/code26_transformed74_source_edited_synthetic_native_saved.gsp")
                .as_slice(),
            Point { x: 643.594749, y: 328.486659 },
            true,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/code26_transformed74_source_edited_synthetic_hide280_native_saved.gsp")
                .as_slice(),
            Point { x: 643.594749, y: 328.486659 },
            false,
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(matches!(&document.objects[279].kind, Kind::Transformed(definition)
            if definition.wire_flavor() == NativeTransformFlavor::DualMarkedPolar
                && definition.native_references() == [180, 278, 279]));
        assert!(matches!(&document.objects[179].kind, Kind::Transformed(definition)
            if definition.wire_flavor() == NativeTransformFlavor::Vector));
        let actual = document.position(280).unwrap();
        assert!((actual.x - expected.x).abs() < 0.01);
        assert!((actual.y - expected.y).abs() < 0.01);
        assert_eq!(document.is_displayed(280), visible);
        assert_eq!(document.encode().unwrap(), bytes);
    }
}
