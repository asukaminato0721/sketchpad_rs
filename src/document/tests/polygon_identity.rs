use super::*;

fn control(state: &str) -> Vec<u8> {
    fs::read(format!(
        "tests/fixtures/polygon_identity58_{state}_native_saved.gsp"
    ))
    .unwrap()
}

#[test]
fn native_polygon_identity_preserves_normal_and_repeated_walk_geometry() {
    for (walk, point, midpoint, distance) in [
        (
            "normal",
            Point { x: 360., y: 300. },
            Point { x: 220., y: 350. },
            7.866628045456043,
        ),
        (
            "repeated",
            Point { x: 380., y: 205. },
            Point { x: 230., y: 302.5 },
            9.466945475739521,
        ),
    ] {
        for identity in ["canonical", "generic15"] {
            let bytes = control(&format!("{walk}_{identity}"));
            let doc = Document::from_bytes(&bytes, None).unwrap();
            assert!(!doc.partial_view, "{walk}_{identity}");
            assert_eq!(doc.objects[10].kind, Kind::PointOnPolygon(10, 4, 0.25));
            assert_eq!(doc.position(11), Some(point));
            assert_eq!(doc.position(14), Some(midpoint));
            assert!((doc.point_distance_cm(15).unwrap() - distance).abs() < 1e-12);
            assert_eq!(doc.encode().unwrap(), bytes);
            doc.verify_roundtrip().unwrap();
        }
    }
}

#[test]
fn inherited_polygon_identity_authors_new_constraints_and_preserves_edits_on_pages() {
    for multi in [false, true] {
        let mut doc = Document::from_bytes(&control("normal_generic15"), None).unwrap();
        if multi {
            doc.add_blank_page().unwrap();
            doc.active_page = 0;
        }
        let original = doc.encode().unwrap();
        let mut doc = Document::from_bytes(&original, None).unwrap();
        doc.checkpoint();
        let id = doc.add(Kind::PointOnPolygon(10, 4, 0.5));
        assert_eq!(doc.position(id), Some(Point { x: 340., y: 320. }));
        assert!(doc.move_point(5, Point { x: 420., y: 280. }));
        assert_eq!(doc.position(11), Some(Point { x: 390., y: 300. }));
        assert_eq!(doc.position(id), Some(Point { x: 360., y: 320. }));
        let bytes = doc.encode().unwrap();
        let saved = Document::from_bytes(&bytes, None).unwrap();
        if !multi {
            assert!(!saved.partial_view);
        }
        assert!(saved
            .objects
            .iter()
            .all(|o| !matches!(o.kind, Kind::Unsupported(_))));
        assert_eq!(saved.objects[id - 1].kind, Kind::PointOnPolygon(10, 4, 0.5));
        assert_eq!(saved.position(id), Some(Point { x: 360., y: 320. }));
        let at = saved.objects[id - 1].constraint_offset.unwrap();
        assert_eq!(&bytes[at..at + 4], &[15, 0, 15, 0]);
        let source = saved.objects[9].header_offset.unwrap();
        assert_eq!(&bytes[source + 6..source + 10], &[4, 0, 15, 0]);
        let existing = saved.objects[10].constraint_offset.unwrap();
        let before = Document::from_bytes(&original, None).unwrap().objects[10]
            .constraint_offset
            .unwrap();
        assert_eq!(
            &bytes[existing + 14..existing + 20],
            &original[before + 14..before + 20]
        );
        doc.verify_roundtrip().unwrap();
        doc.undo();
        assert_eq!(doc.encode().unwrap(), original);
        doc.redo();
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn polygon_identity_keeps_constraint_role_and_range_guards() {
    let original = control("normal_generic15");
    let doc = Document::from_bytes(&original, None).unwrap();
    let at = doc.objects[10].constraint_offset.unwrap();
    let mut mutations = Vec::new();
    for t in [-0.1, 1.1, f64::NAN] {
        let mut b = original.clone();
        b[at + 4..at + 12].copy_from_slice(&t.to_le_bytes());
        mutations.push(b);
    }
    for edge in [9u16, u16::MAX] {
        let mut b = original.clone();
        b[at + 12..at + 14].copy_from_slice(&edge.to_le_bytes());
        mutations.push(b);
    }
    for subtype in [14u16, 58, 59] {
        let mut b = original.clone();
        b[at + 2..at + 4].copy_from_slice(&subtype.to_le_bytes());
        mutations.push(b);
    }
    let rs = records(&original).unwrap();
    let start = rs
        .iter()
        .position(|r| r.data.start == doc.objects[10].header_offset.unwrap())
        .unwrap();
    let refs = rs[start..]
        .iter()
        .find(|r| r.kind == 0x7d2)
        .unwrap()
        .data
        .start;
    for host in [0u32, 11, 12] {
        let mut b = original.clone();
        b[refs + 4..refs + 8].copy_from_slice(&host.to_le_bytes());
        mutations.push(b);
    }
    let constraint_record = rs.iter().find(|r| r.data.start == at).unwrap();
    let mut duplicate = original.clone();
    let extra = original[constraint_record.start..constraint_record.data.end].to_vec();
    duplicate.splice(constraint_record.start..constraint_record.start, extra);
    mutations.push(duplicate);
    let mut short_record = Vec::new();
    put_record(&mut short_record, 0x7d3, &original[at..at + 19]);
    let mut short = original.clone();
    short.splice(
        constraint_record.start..constraint_record.data.end,
        short_record,
    );
    mutations.push(short);
    let mut wrong_header = original.clone();
    let header = doc.objects[10].header_offset.unwrap();
    wrong_header[header + 6..header + 8].copy_from_slice(&2u16.to_le_bytes());
    mutations.push(wrong_header);
    for bytes in mutations {
        let guarded = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(guarded.objects[10].kind, Kind::Unsupported(15));
        assert_eq!(guarded.encode().unwrap(), bytes);
    }
}

#[test]
fn generic_polygon_identity_is_bounded_to_proved_nine_reference_source() {
    let mut doc = Document::new();
    for i in 0..12 {
        let a = (i as f64) * std::f64::consts::TAU / 12.;
        doc.add(Kind::Free(Point {
            x: 200. + 80. * a.cos(),
            y: 200. + 80. * a.sin(),
        }));
    }
    doc.add(Kind::Polygon((1..=12).collect()));
    doc.add(Kind::PointOnPolygon(13, 4, 0.25));
    doc.verify_roundtrip().unwrap();
    let mut bytes = doc.encode().unwrap();
    let d = Document::from_bytes(&bytes, None).unwrap();
    let source = d.objects[12].header_offset.unwrap();
    let constraint = d.objects[13].constraint_offset.unwrap();
    bytes[source + 8..source + 10].copy_from_slice(&15u16.to_le_bytes());
    bytes[constraint + 2..constraint + 4].copy_from_slice(&15u16.to_le_bytes());
    let unsupported = Document::from_bytes(&bytes, None).unwrap();
    assert!(matches!(unsupported.objects[12].kind, Kind::Polygon(_)));
    assert_eq!(unsupported.objects[13].kind, Kind::Unsupported(15));
    assert_eq!(unsupported.encode().unwrap(), bytes);
}
