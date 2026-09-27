use super::*;

fn native_control(state: &str) -> Vec<u8> {
    fs::read(format!(
        "tests/fixtures/bisector53_{state}_native_saved.gsp"
    ))
    .unwrap()
}

#[test]
fn native_bisector_parameters_follow_the_unit_arm_sum_and_both_wire_subtypes() {
    for (state, t, distance_cm, value) in [
        ("equal", 1., 1.77488, Some(1.05409)),
        ("first_double", 1., 1.77488, Some(1.05409)),
        ("second_double", 1., 1.77488, Some(1.05409)),
        ("oblique", 1., 2.29576, None),
        ("t0", 0., 0., Some(1.05409)),
        ("t100", 100., 177.48790, Some(1.05409)),
    ] {
        let bytes = native_control(state);
        let doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view, "{state}");
        for id in [5, 6] {
            assert!(
                matches!(doc.objects[id - 1].kind, Kind::PointOnRay(4, fraction) if fraction == t)
            );
        }
        assert_eq!(doc.position(5), doc.position(6));
        assert!(
            (doc.numeric_value(7).unwrap() - distance_cm).abs() <= 0.000005,
            "{state}"
        );
        if let Some(value) = value {
            assert!(
                (doc.numeric_value(9).unwrap() - value).abs() <= 0.000005,
                "{state}"
            );
        }
        assert_eq!(doc.numeric_value(10), Some(0.));
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn bisector_drag_and_existing_page_writers_preserve_native_subtypes_and_undo() {
    for multi_page in [false, true] {
        let mut doc = Document::from_bytes(&native_control("equal"), None).unwrap();
        if multi_page {
            doc.add_blank_page().unwrap();
            doc.active_page = 0;
        }
        let original = doc.encode().unwrap();
        let mut doc = Document::from_bytes(&original, None).unwrap();
        doc.checkpoint();
        for id in [5, 6] {
            assert!(doc.move_point(id, Point { x: 200., y: 200. }));
            assert!(
                doc.position(id)
                    .unwrap()
                    .distance(Point { x: 200., y: 200. })
                    < 1e-10
            );
        }
        let bytes = doc.encode().unwrap();
        let saved = Document::from_bytes(&bytes, None).unwrap();
        for (id, subtype) in [(5, 44), (6, 13)] {
            let at = saved.objects[id - 1].constraint_offset.unwrap();
            assert_eq!(&bytes[at..at + 4], &[15, 0, subtype, 0]);
            assert!(
                saved
                    .position(id)
                    .unwrap()
                    .distance(Point { x: 200., y: 200. })
                    < 1e-10
            );
        }
        doc.verify_roundtrip().unwrap();
        doc.undo();
        assert_eq!(doc.encode().unwrap(), original);
        doc.redo();
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn bisector_source_edits_recover_from_undefined_geometry_without_replacing_definitions() {
    let original = native_control("equal");
    let mut doc = Document::from_bytes(&original, None).unwrap();
    doc.checkpoint();
    assert!(doc.move_point(3, Point { x: 400., y: 300. }));
    assert_eq!(doc.line_points(4), None);
    assert_eq!(doc.position(5), None);
    assert_eq!(doc.numeric_value(7), None);
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
    assert!(doc.position(5).is_some());
}

#[test]
fn native_bisector_zero_and_opposite_arms_leave_the_path_and_values_undefined() {
    for state in ["opposite", "zero_first", "zero_second"] {
        let bytes = fs::read(format!(
            "tests/fixtures/bisector53_{state}_native_saved.gsp"
        ))
        .unwrap();
        let doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(matches!(doc.objects[3].kind, Kind::Bisector(1, 2, 3)));
        assert_eq!(doc.line_points(4), None, "{state}");
        assert!(matches!(doc.objects[5].kind, Kind::PointOnRay(4, 1.)));
        assert_eq!(doc.position(6), None, "{state}");
        assert_eq!(doc.numeric_value(9), None, "{state}");
        assert_eq!(doc.constrained_point(4, Point { x: 300., y: 300. }), None);
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn native_bisector_coefficient_matches_high_precision_independent_markers() {
    for state in ["candidate_scaled_exact", "candidate_scaled_oblique"] {
        let bytes = native_control(state);
        let doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view, "{state}");
        assert!((doc.numeric_value(9).unwrap() - 1.).abs() < 1e-12);
        assert!(doc.numeric_value(11).unwrap().abs() < 1e-12);
        assert!(doc.numeric_value(13).unwrap().abs() < 0.000005);
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn bisector_point_and_undefined_measurement_guards_reject_invalid_roles() {
    let bytes = native_control("equal");
    let doc = Document::from_bytes(&bytes, None).unwrap();
    let at = doc.objects[4].constraint_offset.unwrap();
    for subtype in [12u16, 46, 47] {
        let mut malformed = bytes.clone();
        malformed[at + 2..at + 4].copy_from_slice(&subtype.to_le_bytes());
        let reopened = Document::from_bytes(&malformed, None).unwrap();
        assert_eq!(reopened.objects[4].kind, Kind::Unsupported(15));
        assert_eq!(reopened.encode().unwrap(), malformed);
    }
    for fraction in [f64::NAN, f64::INFINITY] {
        let mut malformed = bytes.clone();
        malformed[at + 4..at + 12].copy_from_slice(&fraction.to_le_bytes());
        let reopened = Document::from_bytes(&malformed, None).unwrap();
        assert_eq!(reopened.objects[4].kind, Kind::Unsupported(15));
        assert_eq!(reopened.encode().unwrap(), malformed);
    }
    let mut invalid = Document::from_bytes(&native_control("opposite"), None).unwrap();
    invalid.objects[6].kind = Kind::PointDistance(2, 4);
    invalid.dirty = true;
    assert!(invalid.encode().is_err());
    let mut invalid = Document::from_bytes(&native_control("opposite"), None).unwrap();
    invalid.objects[8].kind = Kind::ValueEquation {
        kind: ValueEquationKind::PointValue,
        first: 4,
        second: 4,
    };
    invalid.dirty = true;
    assert!(invalid.encode().is_err());
}
