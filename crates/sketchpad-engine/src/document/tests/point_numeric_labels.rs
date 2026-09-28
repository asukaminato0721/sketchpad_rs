use super::*;

fn native_numeric_label(state: &str) -> (Vec<u8>, Document, usize) {
    let bytes = fs::read(format!(
        "../../tests/fixtures/point_numeric_labels50_{state}_native_verified.gsp"
    ))
    .unwrap();
    let doc = Document::from_bytes(&bytes, None).unwrap();
    let label = doc.objects.last().unwrap().id;
    assert!(
        matches!(doc.objects[label - 1].kind, Kind::PointNumericLabel(..)),
        "{state}"
    );
    assert!(!doc.partial_view, "{state}");
    (bytes, doc, label)
}

#[test]
fn native_point_numeric_labels_use_live_sources_instead_of_cached_markup() {
    for (state, expected) in [
        ("cached_stale", "3.00"),
        ("bare", "3.00"),
        ("precision5", "3.00000"),
        ("edited", "5.00"),
        ("bare_parameter", "2.0"),
        ("bare_named_calculation", "3.00"),
        ("cached_named_calculation", "3.00"),
        ("metadata_only", "3.00"),
        ("distance", "2.65 cm"),
        ("ratio", "1.50"),
        ("angle", "30.00°"),
    ] {
        let (bytes, doc, label) = native_numeric_label(state);
        assert_eq!(
            doc.point_numeric_label(label).unwrap().1,
            expected,
            "{state}"
        );
        assert_eq!(
            doc.encode().unwrap(),
            bytes,
            "{state}: original cache/font/padding must survive"
        );
    }
}

#[test]
fn imported_numeric_label_tracks_parameter_and_anchor_edits_with_undo_and_save() {
    let (bytes, mut doc, label) = native_numeric_label("cached_stale");
    doc.set_parameter_value(1, 4.).unwrap();
    assert_eq!(doc.point_numeric_label(label).unwrap().1, "5.00");
    doc.checkpoint();
    let moved = Point { x: 340., y: 210. };
    assert!(doc.move_point(3, moved));
    assert_eq!(doc.point_numeric_label(label), Some((moved, "5.00".into())));
    let edited = doc.encode().unwrap();
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(
        doc.point_numeric_label(label).unwrap().0,
        Point { x: 280., y: 180. }
    );
    doc.undo();
    assert_eq!(doc.encode().unwrap(), bytes);
    doc.redo();
    doc.redo();
    assert_eq!(doc.encode().unwrap(), edited);
}

#[test]
fn point_numeric_labels_reuse_mapped_evaluation_values_without_changing_stored_records() {
    for (state, source, expected) in [("bare_parameter", 1, "8.0"), ("cached_stale", 2, "8.00")] {
        let (bytes, original, label) = native_numeric_label(state);
        let mut mapped = original.clone();
        mapped.freeze_evaluation_inputs(&original, &[]).unwrap();
        mapped.set_evaluation_number(source, 8.).unwrap();
        let anchor = Point { x: 410., y: 250. };
        mapped.set_evaluation_point(3, anchor).unwrap();
        assert_eq!(
            mapped.point_numeric_label(label),
            Some((anchor, expected.into()))
        );
        assert_eq!(
            original.point_numeric_label(label).unwrap().1,
            if source == 1 { "2.0" } else { "3.00" }
        );
        assert_eq!(mapped.encode().unwrap(), bytes);
    }
}

#[test]
fn native_numeric_labels_preserve_independent_indexed_font_and_optional_metadata() {
    let (bytes, doc, label) = native_numeric_label("font_index1");
    let font = doc.text_font_state(label).unwrap();
    assert_eq!(font.size_code, 36);
    assert!(!font.bold);
    assert_eq!(doc.text_font_state(2).unwrap().size_code, 24);
    assert!(doc.text_font_state(2).unwrap().bold);
    assert_eq!(doc.encode().unwrap(), bytes);
    for state in ["offset_zero", "offset_only", "cached_offsets_font"] {
        let (bytes, doc, _) = native_numeric_label(state);
        assert_eq!(doc.encode().unwrap(), bytes, "{state}");
    }
}

#[test]
fn unknown_reference_and_cached_markup_remain_opaque_and_lossless() {
    let original =
        fs::read("../../tests/fixtures/point_numeric_labels50_cached_stale_input.gsp").unwrap();
    for (needle, replacement) in [
        (b"<?1x2".as_slice(), b"<?1x3".as_slice()),
        (b"<H<T6x".as_slice(), b"<Q<T6x".as_slice()),
    ] {
        let mut bytes = original.clone();
        let at = bytes
            .windows(needle.len())
            .position(|part| part == needle)
            .unwrap();
        bytes[at..at + needle.len()].copy_from_slice(replacement);
        let doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(matches!(
            doc.objects.last().unwrap().kind,
            Kind::Unsupported(90)
        ));
        assert!(doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn native_numeric_label_opaque_alignment_bytes_do_not_change_live_values() {
    let bytes=include_bytes!("../../../../../tests/fixtures/referenced_rich51_numeric_padding_native_saved.gsp");
    let doc=Document::from_bytes(bytes,None).unwrap();
    let label=doc.objects.last().unwrap().id;
    assert!(matches!(doc.objects[label-1].kind,Kind::PointNumericLabel(..)));
    assert_eq!(doc.point_numeric_label(label).unwrap().1,"3.00");
    assert_eq!(doc.encode().unwrap(),bytes);
}
