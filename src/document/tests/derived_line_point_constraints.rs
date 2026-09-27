use super::*;

fn native_control(state: &str) -> Vec<u8> {
    fs::read(format!(
        "tests/fixtures/referenced_roles52_path_{state}_native_saved.gsp"
    ))
    .unwrap()
}

#[test]
fn derived_line_point_constraints_follow_native_host_subtypes_and_centered_parameters() {
    for (state, t) in [("probe", 0.5), ("quarter", 0.25), ("three_quarter", 0.75)] {
        let bytes = native_control(state);
        let doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view);
        for (id, host, subtype, expected) in [
            (
                37,
                33,
                46,
                Point {
                    x: 220. + 200. * (t - 0.5),
                    y: 480.,
                },
            ),
            (
                47,
                43,
                47,
                Point {
                    x: 620.,
                    y: 480. + 200. * (t - 0.5),
                },
            ),
        ] {
            assert!(
                matches!(doc.objects[id - 1].kind, Kind::PointOnLine(h, fraction) if h == host && fraction == t)
            );
            assert_eq!(doc.position(id), Some(expected));
            let at = doc.objects[id - 1].constraint_offset.unwrap();
            assert_eq!(&bytes[at..at + 4], &[15, 0, subtype, 0]);
        }
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn derived_line_point_constraints_drag_save_and_undo_without_changing_host_identity() {
    let original = native_control("probe");
    let mut doc = Document::from_bytes(&original, None).unwrap();
    for (id, host, subtype, target, expected) in [
        (
            37,
            33,
            46,
            Point { x: 270., y: 350. },
            Point { x: 270., y: 480. },
        ),
        (
            47,
            43,
            47,
            Point { x: 700., y: 530. },
            Point { x: 620., y: 530. },
        ),
    ] {
        doc.checkpoint();
        assert!(doc.move_point(id, target));
        assert_eq!(doc.position(id), Some(expected));
        let value = doc.add(Kind::ValueEquation {
            kind: ValueEquationKind::PointValue,
            first: id,
            second: host,
        });
        assert_eq!(
            doc.value_equation_value(value),
            Some(EquationValue::PointValue(0.75))
        );
        let bytes = doc.encode().unwrap();
        let saved = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(saved.position(id), Some(expected));
        let at = saved.objects[id - 1].constraint_offset.unwrap();
        assert_eq!(&bytes[at..at + 4], &[15, 0, subtype, 0]);
        doc.verify_roundtrip().unwrap();
        doc.undo(); // Added measurement.
        doc.undo(); // Drag.
        assert_eq!(doc.encode().unwrap(), original);
        doc.redo();
        doc.verify_roundtrip().unwrap();
        doc.undo();
    }
}

#[test]
fn derived_line_point_constraints_keep_live_host_dependencies() {
    let mut doc = Document::from_bytes(&native_control("quarter"), None).unwrap();
    assert!(doc.move_point(30, Point { x: 260., y: 510. }));
    assert!(doc.move_point(40, Point { x: 660., y: 510. }));
    assert_eq!(doc.position(37), Some(Point { x: 210., y: 510. }));
    assert_eq!(doc.position(47), Some(Point { x: 660., y: 460. }));
    doc.verify_roundtrip().unwrap();
}

#[test]
fn derived_line_point_constraints_patch_existing_pages_without_replacing_subtypes() {
    let mut doc = Document::from_bytes(&native_control("probe"), None).unwrap();
    doc.add_blank_page().unwrap();
    doc.active_page = 0;
    let original = doc.encode().unwrap();
    let mut doc = Document::from_bytes(&original, None).unwrap();
    doc.active_page = 0;
    doc.checkpoint();
    assert!(doc.move_point(37, Point { x: 270., y: 350. }));
    assert!(doc.move_point(47, Point { x: 700., y: 530. }));
    doc.verify_roundtrip().unwrap();
    let bytes = doc.encode().unwrap();
    let saved = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(saved.page_count, 2);
    for (id, subtype, expected) in [
        (37, 46, Point { x: 270., y: 480. }),
        (47, 47, Point { x: 620., y: 530. }),
    ] {
        assert_eq!(saved.position(id), Some(expected));
        let at = saved.objects[id - 1].constraint_offset.unwrap();
        assert_eq!(&bytes[at..at + 4], &[15, 0, subtype, 0]);
    }
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
}

#[test]
fn derived_line_point_constraints_reject_wrong_host_subtypes_and_nonfinite_parameters() {
    let original = native_control("probe");
    let doc = Document::from_bytes(&original, None).unwrap();
    for id in [37, 47] {
        let at = doc.objects[id - 1].constraint_offset.unwrap();
        for subtype in [38u16, if id == 37 { 47 } else { 46 }, 0x012e] {
            let mut bytes = original.clone();
            bytes[at + 2..at + 4].copy_from_slice(&subtype.to_le_bytes());
            let guarded = Document::from_bytes(&bytes, None).unwrap();
            assert!(matches!(
                guarded.objects[id - 1].kind,
                Kind::Unsupported(15)
            ));
            assert_eq!(guarded.encode().unwrap(), bytes);
        }
        let mut bytes = original.clone();
        bytes[at + 4..at + 12].copy_from_slice(&f64::NAN.to_le_bytes());
        let guarded = Document::from_bytes(&bytes, None).unwrap();
        assert!(matches!(
            guarded.objects[id - 1].kind,
            Kind::Unsupported(15)
        ));
        assert_eq!(guarded.encode().unwrap(), bytes);
    }
}
