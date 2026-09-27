use super::*;

const NUMERIC_VISIBLE: &[u8] =
    include_bytes!("../../../tests/fixtures/plot_value_flags58_flags0_native_saved.gsp");
const NUMERIC_ALIAS_HIDDEN: &[u8] =
    include_bytes!("../../../tests/fixtures/plot_value_flags58_flags4_native_saved.gsp");

fn numeric_path_header_flags(bytes: &[u8], doc: &Document, id: usize) -> u32 {
    u32_at(bytes, doc.objects[id - 1].header_offset.unwrap()) >> 16
}

#[test]
fn native_numeric_path_alias_hidden_flag_keeps_dependency_geometry_and_exact_bytes() {
    for (bytes, displayed, flags) in [(NUMERIC_VISIBLE, true, 0), (NUMERIC_ALIAS_HIDDEN, false, 4)]
    {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects[4].kind, Kind::PathValuePoint(1, 4));
        assert_eq!(doc.position(5), Some(Point { x: 150., y: 180. }));
        assert_eq!(doc.position(8), Some(Point { x: 135., y: 230. }));
        assert_eq!(
            doc.line_points(7),
            Some((
                Point { x: 120., y: 280. },
                Point { x: 150., y: 180. },
                LineMode::Segment
            ))
        );
        assert_eq!(doc.objects[4].visible, displayed);
        assert_eq!(doc.is_displayed(5), displayed);
        assert!(doc.is_displayed(7) && doc.is_displayed(8));
        assert_eq!(numeric_path_header_flags(bytes, &doc, 5), flags);
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn showing_numeric_path_alias_clears_both_hidden_bits_and_new_hide_is_canonical() {
    let mut doc = Document::from_bytes(NUMERIC_ALIAS_HIDDEN, None).unwrap();
    doc.selected.insert(5);
    doc.set_selected_visibility(true);
    let shown = doc.encode().unwrap();
    let mut reopened = Document::from_bytes(&shown, None).unwrap();
    assert!(reopened.is_displayed(5));
    assert_eq!(numeric_path_header_flags(&shown, &reopened, 5), 0);
    assert_eq!(shown, NUMERIC_VISIBLE);
    doc.undo();
    assert!(!doc.is_displayed(5));
    assert_eq!(doc.encode().unwrap(), NUMERIC_ALIAS_HIDDEN);
    doc.redo();
    assert!(doc.is_displayed(5));
    reopened.selected.insert(5);
    reopened.set_selected_visibility(false);
    let hidden = reopened.encode().unwrap();
    let reloaded = Document::from_bytes(&hidden, None).unwrap();
    assert_eq!(numeric_path_header_flags(&hidden, &reloaded, 5), 1);
    assert!(!reloaded.is_displayed(5));
    assert_eq!(reloaded.position(8), Some(Point { x: 135., y: 230. }));
    assert!(reloaded.line_points(7).is_some());
    reloaded.verify_roundtrip().unwrap();
}

#[test]
fn unproved_numeric_path_flag_combinations_and_literal_alias_flag_stay_opaque() {
    let decoded = Document::from_bytes(NUMERIC_VISIBLE, None).unwrap();
    let at = decoded.objects[4].header_offset.unwrap();
    for flags in [5u16, 6, 7] {
        let mut bytes = NUMERIC_VISIBLE.to_vec();
        bytes[at + 2..at + 4].copy_from_slice(&flags.to_le_bytes());
        let doc = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(doc.objects[4].kind, Kind::Unsupported(95));
        assert_eq!(doc.encode().unwrap(), bytes);
    }
    let original =
        include_bytes!("../../../tests/fixtures/labeled_literal_path_control_native_verified.gsp");
    let source = Document::from_bytes(original, None).unwrap();
    assert_eq!(source.objects[7].kind, Kind::PathLiteralPoint(7, 0.5));
    let at = source.objects[7].header_offset.unwrap();
    let mut bytes = original.to_vec();
    bytes[at + 2..at + 4].copy_from_slice(&4u16.to_le_bytes());
    let doc = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(doc.objects[7].kind, Kind::Unsupported(123));
    assert_eq!(doc.encode().unwrap(), bytes);
}

#[test]
fn numeric_path_alias_hidden_flag_survives_second_page_local_reference_writing() {
    let mut doc = Document::from_bytes(NUMERIC_ALIAS_HIDDEN, None).unwrap();
    doc.add_blank_page().unwrap();
    doc.move_page(0, 1).unwrap();
    doc.active_page = 0;
    doc.add(Kind::Free(Point { x: 20., y: 40. }));
    doc.active_page = 1;
    let bytes = doc.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(
        reopened
            .objects
            .iter()
            .all(|o| !matches!(o.kind, Kind::Unsupported(_)))
    );
    let point = reopened
        .objects
        .iter()
        .find(|o| matches!(o.kind, Kind::PathValuePoint(..)))
        .unwrap();
    let Kind::PathValuePoint(source, host) = point.kind else {
        unreachable!()
    };
    assert_eq!(point.page, 1);
    assert_eq!(
        reopened.position(point.id),
        Some(Point { x: 150., y: 180. })
    );
    assert!(!reopened.is_displayed(point.id));
    assert_eq!(numeric_path_header_flags(&bytes, &reopened, point.id), 4);
    let rs = records(&bytes).unwrap();
    let at = point.header_offset.unwrap();
    let i = rs
        .iter()
        .position(|r| r.kind == 0x7d0 && r.data.start == at)
        .unwrap();
    let refs = rs[i..].iter().find(|r| r.kind == 0x7d2).unwrap();
    assert_eq!(
        &bytes[refs.data.start + 4..refs.data.start + 12],
        &[1, 0, 0, 0, 4, 0, 0, 0]
    );
    assert_eq!(reopened.objects[source - 1].page, 1);
    assert_eq!(reopened.objects[host - 1].page, 1);
    assert_eq!(reopened.encode().unwrap(), bytes);
    reopened.verify_roundtrip().unwrap();
}
