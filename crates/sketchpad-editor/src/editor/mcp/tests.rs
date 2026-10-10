use super::*;
use sketchpad_engine::document::{Document, Kind};

fn call(editor: &mut Editor, name: &str, arguments: Value) -> Result<Value, String> {
    editor.mcp_call(serde_json::from_value(json!({"name":name,"arguments":arguments})).unwrap())
}

fn editor() -> Editor {
    Editor::new(sketchpad_engine::demo::geometry_document(5).unwrap())
}

#[test]
fn live_edits_update_dependents_and_share_history() {
    let mut editor = editor();
    let original = editor.doc.position(1).unwrap();
    let b = editor.doc.position(2).unwrap();
    let history = editor.doc.undo.len();
    let result = call(&mut editor, "move_point", json!({"id":1,"x":110,"y":120})).unwrap();
    assert_eq!(result["position"], json!({"x":110.0,"y":120.0}));
    assert_eq!(editor.doc.undo.len(), history + 1);
    let midpoint = call(&mut editor, "inspect_object", json!({"id":5})).unwrap();
    assert_eq!(midpoint["parents"], json!([1, 2, 3]));
    assert_eq!(
        midpoint["position"],
        json!({"x":(110. + b.x)/2.,"y":(120. + b.y)/2.})
    );
    call(&mut editor, "undo", json!({})).unwrap();
    assert_eq!(editor.doc.position(1), Some(original));
    call(&mut editor, "redo", json!({})).unwrap();
    assert_eq!(editor.doc.position(1), Some(Point { x: 110., y: 120. }));
    // A subsequent GUI-side edit is visible through the same MCP interface.
    editor.doc.move_point(1, Point { x: 130., y: 140. });
    assert_eq!(
        call(&mut editor, "inspect_object", json!({"id":1})).unwrap()["position"],
        json!({"x":130.,"y":140.})
    );
}

#[test]
fn rejected_moves_preserve_document_dirty_flag_and_redo() {
    let mut editor = editor();
    call(&mut editor, "move_point", json!({"id":1,"x":120,"y":130})).unwrap();
    call(&mut editor, "undo", json!({})).unwrap();
    editor.doc.dirty = false;
    let before = editor.doc.encode().unwrap();
    let history = (editor.doc.undo.len(), editor.doc.redo.len());
    for id in [0, 3, 5, 999] {
        assert!(call(&mut editor, "move_point", json!({"id":id,"x":120,"y":130})).is_err());
    }
    assert!(
        !editor
            .doc
            .move_point_with_history(0, Point { x: 1., y: 2. })
    );
    assert!(
        !editor
            .doc
            .move_point_with_history(1, Point { x: f64::NAN, y: 2. })
    );
    assert_eq!((editor.doc.undo.len(), editor.doc.redo.len()), history);
    assert!(!editor.doc.dirty);
    assert_eq!(editor.doc.encode().unwrap(), before);
}

#[test]
fn list_pagination_and_selection_validation() {
    let mut editor = editor();
    let first = call(&mut editor, "list_objects", json!({"limit":2})).unwrap();
    assert_eq!(first["total"], 5);
    assert_eq!(first["next_offset"], 2);
    assert_eq!(first["objects"][0]["id"], 1);
    let last = call(&mut editor, "list_objects", json!({"offset":4,"limit":2})).unwrap();
    assert_eq!(last["objects"].as_array().unwrap().len(), 1);
    assert!(last["next_offset"].is_null());
    call(&mut editor, "select_objects", json!({"ids":[2,1]})).unwrap();
    assert_eq!(editor.selection_order, vec![2, 1]);
    for ids in [json!([1, 999]), json!([0]), json!([1, 1])] {
        assert!(call(&mut editor, "select_objects", json!({"ids":ids})).is_err());
        assert_eq!(editor.selection_order, vec![2, 1]);
    }
    assert!(call(&mut editor, "list_objects", json!({"limit":501})).is_err());
    assert!(call(&mut editor, "list_objects", json!({"page":0})).is_err());
    call(&mut editor, "select_objects", json!({"ids":[]})).unwrap();
    assert!(editor.doc.selected.is_empty());
}

#[test]
fn parameter_edits_share_history_and_svg_uses_live_viewport() {
    let mut document = Document::new();
    let id = document.add_default_parameter((10, 10)).unwrap();
    let mut editor = Editor::new(document);
    let original = editor.doc.numeric_value(id).unwrap();
    let result = call(&mut editor, "set_parameter", json!({"id":id,"value":4})).unwrap();
    assert_eq!(result["numeric_value"], 4.);
    call(&mut editor, "undo", json!({})).unwrap();
    assert_eq!(editor.doc.numeric_value(id), Some(original));
    assert!(call(&mut editor, "export_svg", json!({})).is_err());
    editor.viewport_rect = Some(egui::Rect::from_min_size(
        egui::pos2(210., 50.),
        egui::vec2(640., 480.),
    ));
    let result = call(&mut editor, "export_svg", json!({})).unwrap();
    assert_eq!(result["mime_type"], "image/svg+xml");
    assert!(result["svg"].as_str().unwrap().contains("<svg"));
}

#[test]
fn edits_reject_inactive_deleted_objects_and_active_drag() {
    let mut editor = editor();
    editor.doc.objects.edit(0).page = 1;
    assert!(call(&mut editor, "move_point", json!({"id":1,"x":1,"y":2})).is_err());
    editor.doc.objects.edit(0).page = 0;
    editor.doc.objects.edit(0).deleted = true;
    assert!(call(&mut editor, "inspect_object", json!({"id":1})).is_err());
    editor.dragging = Some(2);
    assert!(call(&mut editor, "undo", json!({})).is_err());
    assert!(call(&mut editor, "get_state", json!({})).is_ok());
}

#[test]
fn failed_constraint_projection_does_not_create_a_checkpoint() {
    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: 1., y: 1. }));
    let b = document.add(Kind::Free(Point { x: 1., y: 1. }));
    let segment = document.add(Kind::Segment(a, b));
    let point = document.add(Kind::PointOnSegment(segment, 0.5));
    let mut editor = Editor::new(document);
    let history = editor.doc.undo.len();
    assert!(call(&mut editor, "move_point", json!({"id":point,"x":2,"y":2})).is_err());
    assert_eq!(editor.doc.undo.len(), history);
}
