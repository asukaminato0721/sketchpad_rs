use super::*;

fn fixture(state: &str) -> Vec<u8> {
    fs::read(format!(
        "tests/fixtures/locus_metadata54_{state}_native_saved.gsp"
    ))
    .unwrap()
}

fn metadata(bytes: &[u8], d: &Document) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let rs = records(bytes).unwrap();
    let first = rs
        .iter()
        .position(|r| r.data.start == d.objects[9].header_offset.unwrap())
        .unwrap();
    let group = &rs[first..=rs[first..].iter().position(|r| r.kind == 0x07d7).unwrap() + first];
    let data = |kind| bytes[group.iter().find(|r| r.kind == kind).unwrap().data.clone()].to_vec();
    (data(0x07d0), data(0x07d2), data(0x0902))
}

fn samples(d: &Document, distance: f64) {
    let lines = d.locus_segments(10, 8).unwrap();
    assert_eq!(lines.len(), 8);
    for (i, (a, b)) in lines.iter().enumerate() {
        for (point, index) in [(*a, i), (*b, i + 1)] {
            assert!((point.x - (200. + distance * (0.2 + 2. * index as f64 / 8.))).abs() < 1e-9);
            assert!((point.y - 200.).abs() < 1e-9);
        }
    }
    assert!((d.numeric_value(6).unwrap() - 0.7).abs() < 1e-12);
}

#[test]
fn native_locus_metadata_profiles_preserve_order_tail_and_live_samples() {
    for state in [
        "tail0",
        "tail0_edited",
        "unordered",
        "unordered_edited",
        "combined_edited",
    ] {
        let bytes = fixture(state);
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert!(!d.partial_view, "{state}");
        let unordered = state.starts_with("unordered") || state.starts_with("combined");
        let body = if unordered {
            vec![9, 3, 4, 6, 5, 9]
        } else {
            vec![9, 3, 4, 5, 6, 9]
        };
        assert_eq!(d.objects[9].kind, Kind::Locus(body, 500), "{state}");
        let (_, _, range) = metadata(&bytes, &d);
        assert_eq!(
            u32_at(&range, 32),
            if state.starts_with("tail0") || state.starts_with("combined") {
                0
            } else {
                1
            }
        );
        assert_eq!(u32_at(&range, 36), 0);
        samples(
            &d,
            if state.ends_with("edited") {
                150.
            } else {
                100.
            },
        );
        assert_eq!(d.encode().unwrap(), bytes, "{state}");
        d.verify_roundtrip().unwrap();
    }
}

#[test]
fn imported_locus_metadata_edits_roundtrip_undo_and_delete_dependencies() {
    let bytes = fixture("combined_edited");
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    let retained = metadata(&bytes, &d);
    d.checkpoint();
    assert!(d.move_point(8, Point { x: 300., y: 200. }));
    samples(&d, 100.);
    let edited = d.encode().unwrap();
    let reopened = Document::from_bytes(&edited, None).unwrap();
    assert_eq!(metadata(&edited, &reopened), retained);
    samples(&reopened, 100.);
    d.verify_roundtrip().unwrap();
    d.undo();
    samples(&d, 150.);
    d.redo();
    samples(&d, 100.);
    d.selected.insert(5);
    let deleted = d.clear_selected().unwrap();
    for id in [5, 6, 9, 10] {
        assert!(deleted.contains(&id));
    }
    d.verify_roundtrip().unwrap();
    d.undo();
    samples(&d, 100.);
    let restored = d.encode().unwrap();
    assert_eq!(
        metadata(&restored, &Document::from_bytes(&restored, None).unwrap()),
        retained
    );
}

#[test]
fn imported_locus_metadata_survives_existing_multi_page_edit() {
    let bytes = fixture("combined_edited");
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    let retained = metadata(&bytes, &d);
    d.add_blank_page().unwrap();
    let bytes = d.encode().unwrap();
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    assert!(d.move_point(8, Point { x: 300., y: 200. }));
    let edited = d.encode().unwrap();
    let reopened = Document::from_bytes(&edited, None).unwrap();
    assert_eq!(metadata(&edited, &reopened), retained);
    samples(&reopened, 100.);
    d.verify_roundtrip().unwrap();
}

#[test]
fn locus_metadata_relaxation_rejects_incomplete_duplicate_and_wrong_roles() {
    let original = fixture("combined_edited");
    let d = Document::from_bytes(&original, None).unwrap();
    let rs = records(&original).unwrap();
    let first = rs
        .iter()
        .position(|r| r.data.start == d.objects[9].header_offset.unwrap())
        .unwrap();
    let refs = rs[first..]
        .iter()
        .find(|r| r.kind == 0x07d2)
        .unwrap()
        .data
        .start;
    let range = rs[first..]
        .iter()
        .find(|r| r.kind == 0x0902)
        .unwrap()
        .data
        .start;
    // Prefix roles, captured ancestry, final target, list count, and range words
    // remain exact even though intermediate dependency order is preserved.
    for (offset, value) in [
        (refs + 4, 8),
        (refs + 8, 4),
        (refs + 12, 3),
        (refs + 16, 5),
        (refs + 20, 8),
        (refs + 24, 6),
        (refs, 5),
        (range + 20, 0),
        (range + 32, 2),
        (range + 36, 1),
    ] {
        let mut bytes = original.clone();
        bytes[offset..offset + 4].copy_from_slice(&u32::to_le_bytes(value));
        let invalid = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(
            invalid.objects[9].kind,
            Kind::Unsupported(35),
            "offset {offset}"
        );
        assert_eq!(invalid.encode().unwrap(), bytes);
    }
    let mut invalid = d;
    invalid.objects[9].kind = Kind::Locus(vec![9, 3, 4, 5, 5, 9], 500);
    assert!(invalid.verify_roundtrip().is_err());
}
