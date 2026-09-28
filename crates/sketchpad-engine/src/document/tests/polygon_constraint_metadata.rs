use super::*;

fn native_control(state: &str) -> Vec<u8> {
    fs::read(format!(
        "../../tests/fixtures/polygon_constraint52_{state}_native_verified.gsp"
    ))
    .unwrap()
}

#[test]
fn native_polygon_constraint_has_a_u16_edge_and_six_opaque_bytes() {
    for (state, edge, t, expected) in [
        (
            "original",
            1,
            0.7013458662678914,
            Point {
                x: 300.,
                y: 170.13458662678914,
            },
        ),
        ("quarter", 1, 0.25, Point { x: 300., y: 125. }),
        ("edge2", 2, 0.25, Point { x: 250., y: 200. }),
    ] {
        let bytes = native_control(state);
        let doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert!(
            matches!(doc.objects[5].kind, Kind::PointOnPolygon(5, e, fraction) if e == edge && fraction == t)
        );
        assert!(doc.position(6).unwrap().distance(expected) < 1e-10);
        assert_eq!(doc.distance_value_pixels(8), Some(0.));
        let at = doc.objects[5].constraint_offset.unwrap();
        assert_eq!(&bytes[at + 14..at + 20], &[255, 0, 255, 255, 0, 0]);
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn polygon_constraint_edits_preserve_opaque_metadata_on_existing_pages() {
    for multi_page in [false, true] {
        let mut doc = Document::from_bytes(&native_control("quarter"), None).unwrap();
        if multi_page {
            doc.add_blank_page().unwrap();
            doc.active_page = 0;
        }
        let original = doc.encode().unwrap();
        let mut doc = Document::from_bytes(&original, None).unwrap();
        doc.checkpoint();
        assert!(doc.move_point(6, Point { x: 250., y: 200. }));
        assert!(
            matches!(doc.objects[5].kind, Kind::PointOnPolygon(5, 2, fraction) if fraction == 0.25)
        );
        let bytes = doc.encode().unwrap();
        let saved = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(saved.position(6), Some(Point { x: 250., y: 200. }));
        let at = saved.objects[5].constraint_offset.unwrap();
        assert_eq!(&bytes[at + 12..at + 14], &[2, 0]);
        assert_eq!(&bytes[at + 14..at + 20], &[255, 0, 255, 255, 0, 0]);
        doc.verify_roundtrip().unwrap();
        doc.undo();
        assert_eq!(doc.encode().unwrap(), original);
        doc.redo();
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn polygon_constraint_metadata_does_not_replace_live_geometry_or_range_guards() {
    let original = native_control("quarter");
    let mut doc = Document::from_bytes(&original, None).unwrap();
    assert!(doc.move_point(3, Point { x: 340., y: 200. }));
    assert_eq!(doc.position(6), Some(Point { x: 310., y: 125. }));
    doc.verify_roundtrip().unwrap();
    let at = doc.objects[5].constraint_offset.unwrap();
    for edge in [4u16, u16::MAX] {
        let mut bytes = original.clone();
        bytes[at + 12..at + 14].copy_from_slice(&edge.to_le_bytes());
        let guarded = Document::from_bytes(&bytes, None).unwrap();
        assert!(matches!(guarded.objects[5].kind, Kind::Unsupported(15)));
        assert_eq!(guarded.encode().unwrap(), bytes);
    }
    for t in [-0.1f64, 1.1, f64::NAN] {
        let mut bytes = original.clone();
        bytes[at + 4..at + 12].copy_from_slice(&t.to_le_bytes());
        let guarded = Document::from_bytes(&bytes, None).unwrap();
        assert!(matches!(guarded.objects[5].kind, Kind::Unsupported(15)));
        assert_eq!(guarded.encode().unwrap(), bytes);
    }
}
