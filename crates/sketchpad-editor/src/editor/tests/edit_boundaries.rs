use super::*;

#[test]
fn saving_changed_ids_clears_object_references_and_dialogs() {
    for multi_page in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("sketch.gsp");
        let mut doc = Document::new();
        for x in [10., 20., 30.] {
            doc.add(Kind::Free(Point { x, y: 0. }));
        }
        if multi_page {
            doc.add_blank_page().unwrap();
            doc.active_page = 0;
        }
        doc.save_to(&path).unwrap();
        let mut editor = Editor::new(doc);
        if multi_page {
            editor.doc.add(Kind::Free(Point { x: 40., y: 0. }));
            assert!(editor.doc.has_pending_multi_page_additions());
        } else {
            editor.select_only(1);
            editor.clear_selected();
        }
        editor.select_only(3);
        editor.marked_center = Some(2);
        editor.marked_vector = Some((2, 3));
        editor.pending = Some(3);
        editor.inspected = Some(2);
        editor.dialogs.properties_dialog = Some(2);
        editor.dialogs.function_edit_id = Some(3);
        editor.dialogs.function_draft_sources = vec![2, 3];
        editor.dialogs.parametric_x = 2;
        editor.custom_tool_inputs = vec![2];
        editor.create_tool_dialog = true;
        assert!(editor.save(false), "{}", editor.status);
        if !multi_page {
            assert_eq!(editor.doc.position(2).unwrap().x, 30.);
        }
        assert!(editor.selection_order.is_empty());
        assert!(editor.marked_center.is_none());
        assert!(editor.marked_vector.is_none());
        assert!(editor.pending.is_none());
        assert!(editor.inspected.is_none());
        assert!(editor.dialogs.properties_dialog.is_none());
        assert!(editor.dialogs.function_edit_id.is_none());
        assert!(editor.dialogs.function_draft_sources.is_empty());
        assert_eq!(editor.dialogs.parametric_x, 0);
        assert!(editor.custom_tool_inputs.is_empty());
        assert!(!editor.create_tool_dialog);
        assert!(!editor.doc.dirty);
    }
}

#[test]
fn failed_save_preserves_object_references() {
    let directory = tempfile::tempdir().unwrap();
    let mut doc = Document::new();
    doc.add(Kind::Free(Point { x: 10., y: 0. }));
    doc.add(Kind::Free(Point { x: 20., y: 0. }));
    doc.path = Some(directory.path().join("missing/sketch.gsp"));
    let mut editor = Editor::new(doc);
    editor.select_only(1);
    editor.clear_selected();
    editor.marked_center = Some(2);
    editor.dialogs.properties_dialog = Some(2);
    assert!(!editor.save(false));
    assert_eq!(editor.marked_center, Some(2));
    assert_eq!(editor.dialogs.properties_dialog, Some(2));
    assert_eq!(editor.doc.position(2).unwrap().x, 20.);
    assert!(editor.doc.dirty);
}

#[test]
fn saving_without_renumbering_preserves_object_references() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("sketch.gsp");
    let mut doc = Document::new();
    doc.add(Kind::Free(Point { x: 10., y: 0. }));
    assert!(!doc.save_to(&path).unwrap().object_ids_changed);
    let mut editor = Editor::new(doc);
    editor.select_only(1);
    editor.marked_center = Some(1);
    editor.dialogs.properties_dialog = Some(1);
    assert!(editor.save(false));
    assert_eq!(editor.selection_order, vec![1]);
    assert_eq!(editor.marked_center, Some(1));
    assert_eq!(editor.dialogs.properties_dialog, Some(1));
}

#[test]
fn midpoint_and_its_helper_share_one_gui_undo_step() {
    let mut doc = Document::new();
    doc.add(Kind::Free(Point { x: 10., y: 0. }));
    doc.add(Kind::Free(Point { x: 30., y: 0. }));
    let mut editor = Editor::new(doc);
    editor.selection_order = vec![1, 2];
    editor.doc.selected = [1, 2].into();
    let history = editor.doc.undo_len();
    editor.pair(2);
    assert_eq!(editor.doc.objects().len(), 4);
    assert_eq!(editor.doc.undo_len(), history + 1);
    editor.apply_history(false);
    assert_eq!(editor.doc.objects().len(), 2);
    editor.apply_history(true);
    assert_eq!(editor.doc.objects().len(), 4);
    assert_eq!(editor.doc.position(4).unwrap().x, 20.);
}

#[test]
fn both_intersections_share_one_gui_undo_step() {
    let mut doc = Document::new();
    for (x, y) in [(0., 0.), (0., 5.), (4., 0.), (4., 5.)] {
        doc.add(Kind::Free(Point { x, y }));
    }
    doc.add(Kind::Circle(1, 2));
    doc.add(Kind::Circle(3, 4));
    let mut editor = Editor::new(doc);
    editor.selection_order = vec![5, 6];
    editor.doc.selected = [5, 6].into();
    let history = editor.doc.undo_len();
    editor.intersection();
    assert_eq!(editor.selection_order.len(), 2);
    assert_eq!(editor.doc.undo_len(), history + 1);
    editor.apply_history(false);
    assert_eq!(editor.doc.objects().len(), 6);
    editor.apply_history(true);
    assert_eq!(editor.doc.objects().len(), 8);
}
