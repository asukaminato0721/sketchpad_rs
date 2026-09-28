use super::*;

#[test]
fn authentic_labeled_literal_paths_import_labels_fonts_and_native_geometry() {
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/labeled_literal_path_original_native_verified.gsp")
            .as_slice(),
        include_bytes!("../../../../../tests/fixtures/labeled_literal_path_control_native_verified.gsp")
            .as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects.len(), 8);
        assert_eq!(doc.objects[3].kind, Kind::PathLiteralPoint(3, 0.));
        assert_eq!(doc.objects[7].kind, Kind::PathLiteralPoint(7, 0.5));
        for (id, name) in [(4, "C"), (8, "F")] {
            let label = doc.objects[id - 1].label.as_ref().unwrap();
            assert_eq!(label.name_utf8().unwrap(), name);
            assert!(label.visible());
            assert!(doc.text_font_state(id).is_some());
        }
        let (center, radius) = doc.circle_points(3).unwrap();
        assert_eq!(
            doc.position(4),
            Some(Point {
                x: center.x + radius,
                y: center.y
            })
        );
        let (a, b, _) = doc.line_points(7).unwrap();
        assert_eq!(
            doc.position(8),
            Some(Point {
                x: (a.x + b.x) / 2.,
                y: (a.y + b.y) / 2.
            })
        );
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn literal_label_insertion_uses_native_constraint_then_label_order() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/labeled_literal_path_control_native_verified.gsp"),
        None,
    )
    .unwrap();
    let point = doc.plot_literal_on_path(0.25, Some(7)).unwrap();
    doc.set_point_label(point, "QuarterValue", true).unwrap();
    let before_save = doc.encode().unwrap();
    let mut reopened = Document::from_bytes(&before_save, None).unwrap();
    assert!(!reopened.partial_view);
    reopened
        .set_point_label(point, "QuarterRenamed", true)
        .unwrap();
    let after_save = reopened.encode().unwrap();
    let reloaded = Document::from_bytes(&after_save, None).unwrap();
    assert_eq!(
        reloaded.objects[point - 1]
            .label
            .as_ref()
            .unwrap()
            .name_utf8()
            .unwrap(),
        "QuarterRenamed"
    );
    reloaded.verify_roundtrip().unwrap();
    for bytes in [&before_save, &after_save] {
        let rs = records(bytes).unwrap();
        let start = rs.iter().rposition(|r| r.kind == 0x07d0).unwrap();
        let kinds = rs[start..]
            .iter()
            .take_while(|r| r.kind != 0x07d7)
            .map(|r| r.kind)
            .collect::<Vec<_>>();
        assert_eq!(kinds, [0x07d0, 0x07d2, 0x07d3, 0x07d5, 0x07d8]);
    }
    let bare = doc.plot_literal_on_path(0.75, Some(7)).unwrap();
    let bare_bytes = doc.encode().unwrap();
    let mut saved_bare = Document::from_bytes(&bare_bytes, None).unwrap();
    saved_bare.set_point_label(bare, "LateLabel", true).unwrap();
    let labeled = saved_bare.encode().unwrap();
    let rs = records(&labeled).unwrap();
    let start = rs.iter().rposition(|r| r.kind == 0x07d0).unwrap();
    assert_eq!(
        rs[start..]
            .iter()
            .take_while(|r| r.kind != 0x07d7)
            .map(|r| r.kind)
            .collect::<Vec<_>>(),
        [0x07d0, 0x07d2, 0x07d3, 0x07d5, 0x07d8]
    );
    saved_bare.verify_roundtrip().unwrap();
}

#[test]
fn labeled_literal_api_edits_copy_and_both_label_writer_paths_match_native_goldens() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/labeled_literal_path_control_native_verified.gsp"),
        None,
    )
    .unwrap();
    for (id, point) in [
        (2, Point { x: 190., y: 140. }),
        (5, Point { x: 130., y: 200. }),
        (6, Point { x: 310., y: 210. }),
    ] {
        assert!(doc.move_point(id, point));
    }
    doc.set_point_label(4, "CircleValue", true).unwrap();
    doc.set_point_label(8, "HalfValue", true).unwrap();
    doc.selected = [4].into_iter().collect();
    doc.toggle_selected_text_style(TextStyleCommand::Bold)
        .unwrap();
    doc.selected = [8].into_iter().collect();
    doc.set_selected_text_family("Arial").unwrap();
    let edited = doc.encode().unwrap();
    assert_eq!(
        edited,
        include_bytes!("../../../../../tests/fixtures/labeled_literal_path_edited_native_verified.gsp")
    );
    let mut saved = Document::from_bytes(&edited, None).unwrap();
    saved.selected = [4, 8].into_iter().collect();
    let clipboard = saved.copy_selected().unwrap();
    assert_eq!(
        saved.paste(&clipboard).unwrap(),
        (9..=16).collect::<Vec<_>>()
    );
    for id in [4, 8] {
        assert_eq!(saved.objects[id - 1].label, saved.objects[id + 7].label);
        assert_eq!(saved.text_font_state(id), saved.text_font_state(id + 8));
        let point = saved.position(id).unwrap();
        assert_eq!(
            saved.position(id + 8),
            Some(Point {
                x: point.x + 20.,
                y: point.y + 20.
            })
        );
    }
    let copied = saved.encode().unwrap();
    assert_eq!(
        copied,
        include_bytes!("../../../../../tests/fixtures/labeled_literal_path_copied_native_verified.gsp")
    );
    saved.undo();
    assert_eq!(saved.encode().unwrap(), edited);
    saved.redo();
    assert_eq!(saved.encode().unwrap(), copied);
    let mut reopened = Document::from_bytes(&copied, None).unwrap();
    assert_eq!(reopened.plot_literal_on_path(0.25, Some(7)).unwrap(), 17);
    let bare = reopened.encode().unwrap();
    let mut saved_bare = Document::from_bytes(&bare, None).unwrap();
    saved_bare
        .set_point_label(17, "QuarterValue", true)
        .unwrap();
    assert_eq!(saved_bare.plot_literal_on_path(0.75, Some(7)).unwrap(), 18);
    saved_bare
        .set_point_label(18, "ThreeQuarterValue", true)
        .unwrap();
    let labeled = saved_bare.encode().unwrap();
    assert_eq!(
        labeled,
        include_bytes!(
            "../../../../../tests/fixtures/labeled_literal_path_new_label_native_verified.gsp"
        )
    );
    saved_bare.verify_roundtrip().unwrap();
}
