use super::*;
use crate::document::transform_model::NativeTransformFlavor;

fn original() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../gsp-rs/tests/Samples/个人专栏/侯仰顺作品/正n边形的渐开线(蚂蚁).gsp"
    ))
    .unwrap()
}

#[test]
fn native_standalone_free_source_point_uses_live_angle_and_length() {
    let bytes = original();
    let mut document = Document::from_bytes(&bytes, None).unwrap();
    assert!(
        matches!(&document.objects[16].kind, Kind::Transformed(definition)
        if definition.wire_flavor() == NativeTransformFlavor::DualMarkedPolar
            && definition.native_references() == [1, 11, 16])
    );
    let original_point = document.position(17).unwrap();
    assert!((original_point.x - 265.420).abs() < 0.01);
    assert!((original_point.y - 493.916).abs() < 0.01);
    assert_eq!(document.encode().unwrap(), bytes);
    assert!(document.move_point(1, Point { x: 301., y: 460. }));
    let moved = document.position(17).unwrap();
    assert!((moved.x - original_point.x - 40.).abs() < 1e-9);
    assert!((moved.y - original_point.y + 40.).abs() < 1e-9);
    document.verify_roundtrip().unwrap();
}

#[test]
fn standalone_point_rejects_wrong_numeric_roles_and_keeps_original_bytes() {
    let bytes = original();
    let document = Document::from_bytes(&bytes, None).unwrap();
    let header = document.objects[16].header_offset.unwrap();
    let records = records(&bytes).unwrap();
    let start = records
        .iter()
        .position(|record| record.kind == 0x07d0 && record.data.start == header)
        .unwrap();
    let path = records[start + 1..]
        .iter()
        .take_while(|record| record.kind != 0x07d7)
        .find(|record| record.kind == 0x07d2)
        .unwrap();
    for (offset, value) in [(8, 16u32), (12, 11u32)] {
        let mut malformed = bytes.clone();
        malformed[path.data.start + offset..path.data.start + offset + 4]
            .copy_from_slice(&value.to_le_bytes());
        let decoded = Document::from_bytes(&malformed, None).unwrap();
        assert_eq!(decoded.objects[16].kind, Kind::Unsupported(26));
        assert_eq!(decoded.encode().unwrap(), malformed);
    }
}

#[test]
fn native_saved_standalone_controls_match_live_positions_in_two_originals() {
    for (bytes, expected) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/code26_standalone73_visible_native_saved.gsp"
            )
            .as_slice(),
            Point {
                x: 265.420,
                y: 493.916,
            },
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/code26_standalone73_source_edited_native_saved.gsp"
            )
            .as_slice(),
            Point {
                x: 305.420,
                y: 453.916,
            },
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/code26_standalone73_angle_edited_native_saved.gsp"
            )
            .as_slice(),
            Point {
                x: 267.512,
                y: 496.240,
            },
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/code26_standalone73_distance_edited_native_saved.gsp"
            )
            .as_slice(),
            Point {
                x: 320.500,
                y: 418.105,
            },
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        let actual = document.position(17).unwrap();
        assert!((actual.x - expected.x).abs() < 0.01);
        assert!((actual.y - expected.y).abs() < 0.01);
        assert_eq!(document.encode().unwrap(), bytes);
    }
    for (bytes, visible) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/code26_second73_moved_hidden_native_saved.gsp"
            )
            .as_slice(),
            false,
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/code26_second73_moved_visible_native_saved.gsp"
            )
            .as_slice(),
            true,
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        let actual = document.position(9).unwrap();
        assert!((actual.x - 922.677).abs() < 0.01);
        assert!((actual.y - 700.).abs() < 0.01);
        assert_eq!(document.is_displayed(9), visible);
        assert_eq!(document.encode().unwrap(), bytes);
    }
    for (bytes, expected, visible) in [
        (
            include_bytes!("../../../../../tests/fixtures/code26_standalone73_zero_hidden_native_saved.gsp")
                .as_slice(),
            Point { x: 261., y: 500. },
            false,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/code26_standalone73_zero_distance_native_saved.gsp")
                .as_slice(),
            Point { x: 261., y: 500. },
            true,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/code26_standalone73_negative_distance_native_saved.gsp")
                .as_slice(),
            Point {
                x: 255.804001333,
                y: 507.151678624,
            },
            true,
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        let actual = document.position(17).unwrap();
        assert!((actual.x - expected.x).abs() < 0.01);
        assert!((actual.y - expected.y).abs() < 0.01);
        assert_eq!(document.is_displayed(17), visible);
        assert_eq!(document.encode().unwrap(), bytes);
    }
}
