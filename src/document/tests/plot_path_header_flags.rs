use super::*;

const FLAGS: &[u8] =
    include_bytes!("../../../tests/fixtures/plot_path_flags_original_native_verified.gsp");
const MARKERS: &[u8] =
    include_bytes!("../../../tests/fixtures/plot_path_flags_dependency_edit_native_verified.gsp");

#[test]
fn native_plot_header_flags_preserve_defined_suppression_and_hide_show() {
    let mut doc = Document::from_bytes(FLAGS, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.objects.len(), 20);
    assert!(
        doc.objects
            .iter()
            .all(|object| !matches!(object.kind, Kind::Unsupported(_)))
    );
    for id in [13, 14, 15, 16, 17, 18, 19, 20] {
        assert!(doc.position(id).is_some());
        assert_eq!(doc.is_displayed(id), matches!(id, 13 | 17));
    }
    assert!(doc.objects[14].visible);
    assert!(!doc.objects[15].visible);
    assert_eq!(doc.encode().unwrap(), FLAGS);
    doc.show_all_hidden();
    assert!(doc.is_displayed(14) && doc.is_displayed(18));
    assert!(!doc.is_displayed(15) && !doc.is_displayed(16));
    assert!(!doc.objects[15].visible);
    doc.selected = [15, 16, 19, 20].into_iter().collect();
    let before = doc.encode().unwrap();
    let history = doc.undo.len();
    doc.set_selected_visibility(false);
    doc.set_selected_visibility(true);
    assert_eq!(doc.undo.len(), history);
    assert_eq!(doc.encode().unwrap(), before);
    let reopened = Document::from_bytes(&before, None).unwrap();
    assert!(!reopened.is_displayed(15) && !reopened.is_displayed(16));
    doc.undo();
    assert!(!doc.is_displayed(14));
    doc.redo();
    assert!(doc.is_displayed(14));
    doc.verify_roundtrip().unwrap();
}

#[test]
fn suppressed_plot_dependencies_copy_as_ordinary_hidden_and_recover() {
    let mut doc = Document::from_bytes(MARKERS, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.position(15), Some(Point { x: 205., y: 190. }));
    assert_eq!(doc.position(19), Some(Point { x: 305., y: 190. }));
    assert!(doc.line_points(21).is_some() && doc.line_points(22).is_some());
    assert!(doc.move_point(6, Point { x: 350., y: 210. }));
    doc.set_parameter_value(11, 0.5).unwrap();
    assert_eq!(doc.position(15), Some(Point { x: 240., y: 200. }));
    assert!(!doc.is_displayed(15) && !doc.is_displayed(19));
    let edited = doc.encode().unwrap();
    assert_eq!(
        edited,
        include_bytes!(
            "../../../tests/fixtures/plot_path_flags_authored_edited_native_verified.gsp"
        )
    );
    let reopened = Document::from_bytes(&edited, None).unwrap();
    assert!(!reopened.is_displayed(15) && !reopened.is_displayed(19));
    doc.selected = [21, 22].into_iter().collect();
    let clipboard = doc.copy_selected().unwrap();
    let copied_hidden: Vec<_> = clipboard
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| matches!(object.old_id, 15 | 19))
        .map(|(index, object)| {
            assert!(!object.visible);
            index
        })
        .collect();
    assert_eq!(copied_hidden.len(), 2);
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = doc.encode().unwrap();
    assert_eq!(
        copied,
        include_bytes!(
            "../../../tests/fixtures/plot_path_flags_authored_copied_native_verified.gsp"
        )
    );
    for &index in &copied_hidden {
        let id = pasted[index];
        assert!(!doc.is_displayed(id));
        assert!(doc.position(id).is_some());
        let offset = Document::from_bytes(&copied, None).unwrap().objects[id - 1]
            .header_offset
            .unwrap();
        assert_eq!(u32_at(&copied, offset) >> 16, 1);
    }
    doc.show_all_hidden();
    for &index in &copied_hidden {
        assert!(doc.is_displayed(pasted[index]));
    }
    assert!(!doc.is_displayed(15) && !doc.is_displayed(19));
    let shown = doc.encode().unwrap();
    assert_eq!(
        shown,
        include_bytes!(
            "../../../tests/fixtures/plot_path_flags_authored_shown_native_verified.gsp"
        )
    );
    doc.verify_roundtrip().unwrap();
    if let Ok(directory) = std::env::var("SKETCHPAD_FLAGS_GOLDEN_DIR") {
        fs::write(
            Path::new(&directory).join("plot_path_flags_authored_edited.gsp"),
            &edited,
        )
        .unwrap();
        fs::write(
            Path::new(&directory).join("plot_path_flags_authored_copied.gsp"),
            &copied,
        )
        .unwrap();
        fs::write(
            Path::new(&directory).join("plot_path_flags_authored_shown.gsp"),
            &shown,
        )
        .unwrap();
    }
}
