use super::*;

#[test]
fn native_triangle_angle_marks_keep_distinct_live_arc_counts() {
    for (bytes, show_a, show_b) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/angle_marker87_original_native_saved.gsp"
            )
            .as_slice(),
            true,
            true,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/angle_marker87_hide13_native_saved.gsp")
                .as_slice(),
            false,
            true,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/angle_marker87_hide14_native_saved.gsp")
                .as_slice(),
            true,
            false,
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/angle_marker87_move_free4_native_saved.gsp"
            )
            .as_slice(),
            true,
            true,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(doc.objects[12].kind, Kind::AngleMarker(2, 1, 4, 1));
        assert_eq!(doc.objects[13].kind, Kind::AngleMarker(4, 2, 1, 2));
        assert_eq!(doc.objects[12].construction_dependencies(), vec![2, 1, 4]);
        assert_eq!(doc.objects[13].construction_dependencies(), vec![4, 2, 1]);
        assert_eq!(doc.angle_marker_strokes(13).unwrap().len(), 1);
        assert_eq!(doc.angle_marker_strokes(14).unwrap().len(), 2);
        for (id, vertex) in [(13, 1), (14, 2)] {
            let center = doc.position(vertex).unwrap();
            let geometry = doc.angle_marker_geometry(id).unwrap();
            for (ring, path) in geometry.strokes.iter().enumerate() {
                let start = path[0];
                assert!(((start.x - center.x).hypot(start.y - center.y) - (24. + ring as f64 * 6.)).abs() < 1e-9);
            }
            assert_eq!(geometry.sector[0], center);
        }
        assert_eq!(doc.is_displayed(13), show_a);
        assert_eq!(doc.is_displayed(14), show_b);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
    let before = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/angle_marker87_original_native_saved.gsp"),
        None,
    )
    .unwrap();
    let after = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/angle_marker87_move_free4_native_saved.gsp"),
        None,
    )
    .unwrap();
    assert_ne!(
        before.angle_marker_strokes(13),
        after.angle_marker_strokes(13)
    );
    assert_ne!(
        before.angle_marker_strokes(14),
        after.angle_marker_strokes(14)
    );
}

#[test]
fn angle_marker_class_is_independent_of_the_high_word_flag() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/angle_marker87_original_native_saved.gsp");
    let doc = Document::from_bytes(bytes, None).unwrap();
    let header = doc.objects[12].header_offset.unwrap();
    let marker = records(bytes)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x090e && record.start > header)
        .unwrap();
    let mut changed = bytes.to_vec();
    changed[marker.data.start + 2] = 1;
    let guarded = Document::from_bytes(&changed, None).unwrap();
    assert_eq!(guarded.objects[12].kind, doc.objects[12].kind);
    assert_eq!(guarded.angle_marker_strokes(13), doc.angle_marker_strokes(13));
    assert!(doc.angle_marker_geometry(13).unwrap().arrows.is_empty());
    assert_eq!(guarded.angle_marker_geometry(13).unwrap().arrows.len(), 1);
    assert_eq!(guarded.encode().unwrap(), changed);
}

#[test]
fn native_angle_marker_alias_uses_signed_live_angle_and_page_units() {
    for (bytes, expected) in [
        (include_bytes!("../../../../../tests/fixtures/angle_marker87_alias_small_show27_clear_native_saved.gsp").as_slice(), "∠1 = 1.08"),
        (include_bytes!("../../../../../tests/fixtures/angle_marker87_alias_small_show27_clear_q3_native_saved.gsp").as_slice(), "∠1 = 0.90"),
        (include_bytes!("../../../../../tests/fixtures/angle_marker87_alias_small_show27_clear_qnegative3_native_saved.gsp").as_slice(), "∠1 = -0.73"),
        (include_bytes!("../../../../../tests/fixtures/angle_marker87_unit_small_rad_to_deg_native_saved.gsp").as_slice(), "∠1 = 61.68°"),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(matches!(doc.objects[24].kind, Kind::AngleMarker(9, 13, 1, 1)), "{:?} parents {:?} {:?} {:?} label {:?}", doc.objects[24].kind, doc.objects[8].kind, doc.objects[12].kind, doc.objects[0].kind, doc.objects[24].label);
        assert_eq!(doc.objects[26].kind, Kind::NumericAlias { source: 25, unit: NumericUnit::Degrees });
        assert_eq!(doc.measurement_text(27).as_deref(), Some(expected));
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn native_long_named_marker_alias_uses_same_page_unit_setting() {
    for (bytes, expected) in [
        (include_bytes!("../../../../../tests/fixtures/angle_marker87_alias_show42_clear_native_saved.gsp").as_slice(), "43.11°"),
        (include_bytes!("../../../../../tests/fixtures/angle_marker87_unit_large_deg_to_rad_native_saved.gsp").as_slice(), "0.75"),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(matches!(doc.objects[40].kind, Kind::AngleMarker(..)));
        assert_eq!(doc.objects[41].kind, Kind::NumericAlias { source: 41, unit: NumericUnit::Degrees });
        assert!(doc.measurement_text(42).unwrap().ends_with(expected));
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn unnamed_marker_alias_generates_a_point_based_caption() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/angle_marker87_no_label_alias_original.gsp");
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert!(matches!(doc.objects[96].kind, Kind::AngleMarker(..)));
    assert_eq!(
        doc.objects[99].kind,
        Kind::NumericAlias {
            source: 97,
            unit: NumericUnit::Degrees
        }
    );
    assert!(doc.objects[99].label.is_none());
    let caption = doc.measurement_text(100).unwrap();
    assert!(caption.starts_with('∠'), "{caption}");
    assert!(caption.ends_with('°'), "{caption}");
    assert_eq!(doc.encode().unwrap(), bytes);
}
