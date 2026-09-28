use super::*;

#[test]
fn native_code26_point_follows_a_dual_marked_polar_source() {
    let states: [(&[u8], bool); 4] = [
        (
            include_bytes!("../../../../../tests/fixtures/point26_dual94_original_input.gsp"),
            true,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/point26_dual94_hide_input.gsp"),
            false,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/point26_dual94_move_free_input.gsp"),
            true,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/point26_dual94_change_length_input.gsp"),
            true,
        ),
    ];
    let mut positions = Vec::new();
    for (bytes, visible) in states {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(matches!(&doc.objects[10].kind, Kind::Transformed(definition)
            if definition.wire_flavor() == NativeTransformFlavor::DualMarkedPolar));
        assert!(matches!(&doc.objects[12].kind, Kind::Transformed(definition)
            if definition.wire_flavor() == NativeTransformFlavor::DualMarkedPolar));
        assert_eq!(doc.objects[12].kind.dependencies(), [11, 7, 12]);
        assert_eq!(doc.is_displayed(13), visible);
        positions.push(doc.position(13).unwrap());
        assert_eq!(doc.encode().unwrap(), bytes);
    }
    assert_eq!(positions[0], positions[1]);
    assert!((positions[2].x - positions[0].x - 40.).abs() < 1e-9);
    assert!((positions[2].y - positions[0].y - 20.).abs() < 1e-9);
    assert!((positions[3].x - positions[0].x).abs() < 1e-9);
    assert!((positions[3].y - positions[0].y).abs() < 1e-9);

    for (name, visible) in [
        (
            include_bytes!("../../../../../tests/fixtures/point26_dual94_rebind_distance_input.gsp")
                .as_slice(),
            true,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/point26_dual94_rebind_distance_hide_input.gsp")
                .as_slice(),
            false,
        ),
    ] {
        let doc = Document::from_bytes(name, None).unwrap();
        assert_eq!(doc.objects[12].kind.dependencies(), [11, 7, 10]);
        assert_eq!(doc.is_displayed(13), visible);
        let point = doc.position(13).unwrap();
        assert!(point.x - positions[0].x > 22. && point.x - positions[0].x < 23.);
        assert!((point.y - positions[0].y).abs() < 1e-9);
        assert_eq!(doc.encode().unwrap(), name);
    }

    for (bytes, visible) in [
        (
            include_bytes!("../../../../../tests/fixtures/point26_dual94_rebind_angle_input.gsp")
                .as_slice(),
            true,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/point26_dual94_rebind_angle_hide_input.gsp")
                .as_slice(),
            false,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(doc.objects[12].kind.dependencies(), [11, 7, 10]);
        assert_eq!(doc.is_displayed(13), visible);
        let point = doc.position(13).unwrap();
        assert!((point.x - positions[0].x).abs() < 1e-9);
        assert!(positions[0].y - point.y > 22. && positions[0].y - point.y < 23.);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}
