use super::*;
use crate::document::transform_model::NativeTransformFlavor;

#[test]
fn native_marked_polar_source_custom_point_uses_live_source_and_angle() {
    for (bytes, expected, angle, visible) in [
        (
            include_bytes!("../../../../../tests/fixtures/code26_markedpolar77_original_native_saved.gsp")
                .as_slice(),
            Point { x: 377.771654, y: 180.228346 },
            90.,
            false,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/code26_markedpolar77_forced_show25_synthetic_native_saved.gsp")
                .as_slice(),
            Point { x: 377.771654, y: 180.228346 },
            90.,
            true,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/code26_markedpolar77_source12_edited_forced_show25_synthetic_native_saved.gsp")
                .as_slice(),
            Point { x: 417.771654, y: 200.228346 },
            90.,
            true,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/code26_markedpolar77_angle_half_forced_show25_synthetic_native_saved.gsp")
                .as_slice(),
            Point { x: 604.543307, y: 407. },
            0.,
            true,
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(matches!(&document.objects[12].kind, Kind::Transformed(definition)
            if definition.wire_flavor() == NativeTransformFlavor::MarkedPolar));
        assert!(matches!(&document.objects[24].kind, Kind::Transformed(definition)
            if definition.wire_flavor() == NativeTransformFlavor::DualMarkedPolar
                && definition.native_references() == [13, 24, 9]));
        assert_eq!(document.numeric_value(24), Some(angle));
        let actual = document.position(25).unwrap();
        assert!((actual.x - expected.x).abs() < 0.01);
        assert!((actual.y - expected.y).abs() < 0.01);
        assert_eq!(document.is_displayed(25), visible);
        assert_eq!(document.encode().unwrap(), bytes);
    }
}
