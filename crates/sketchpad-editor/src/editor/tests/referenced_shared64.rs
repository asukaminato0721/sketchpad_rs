use super::*;
use sketchpad_engine::document::TextStyleCommand;

fn shared_reference_editor(edited: bool) -> Editor {
    let bytes: &[u8] = if edited {
        include_bytes!(
            "../../../../../tests/fixtures/referenced_roles52_anchored_edited_native_saved.gsp"
        )
    } else {
        include_bytes!(
            "../../../../../tests/fixtures/referenced_roles52_anchored_baseline_native_saved.gsp"
        )
    };
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    for object in &mut doc.objects {
        object.visible = (9..=12).contains(&object.id);
    }
    for id in 9..=12 {
        assert!(matches!(
            doc.objects[id - 1].kind,
            Kind::PointReferencedText { .. }
        ));
    }
    Editor::new(doc)
}

fn shared_reference_viewport() -> Viewport {
    Viewport {
        rect: egui::Rect::from_min_size(egui::pos2(20., 30.), egui::vec2(1000., 800.)),
        scale: 1.,
        center: Point { x: 500., y: 400. },
    }
}

#[test]
fn anchored_rich_native_controls_render_and_hit_their_live_anchors() {
    for edited in [false, true] {
        let mut editor = shared_reference_editor(edited);
        let viewport = shared_reference_viewport();
        for id in 9..=12 {
            let (anchor, content) = editor.doc.point_numeric_label(id).unwrap();
            let hit = viewport.screen(anchor) + egui::vec2(17., 0.);
            assert_eq!(
                editor.hit(&viewport, hit),
                Some(id),
                "caption{id}: {content}"
            );
            editor.click(&viewport, hit, false);
            assert!(editor.doc.selected.contains(&id));
        }
        let svg = crate::render::svg_page(&editor.doc, &viewport).unwrap();
        assert!(svg.contains(if edited { "EDIT_TEXT" } else { "SOURCE_TEXT" }));
        assert!(svg.contains(if edited { "VALUE 4.0" } else { "VALUE 2.0" }));
        assert!(svg.contains("FUNCTION"));
        assert!(svg.contains(if edited { "f(x) = x" } else { "x²" }));
        assert!(svg.contains("MATH"));
        assert!(svg.contains("(1)/(2)"));
        editor.doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn anchored_rich_source_edits_update_content_without_moving_points_and_undo() {
    let mut editor = shared_reference_editor(false);
    let anchors = (9..=12)
        .map(|id| editor.doc.point_numeric_label(id).unwrap().0)
        .collect::<Vec<_>>();
    let before = editor.doc.point_numeric_label(10).unwrap().1;
    editor.doc.set_parameter_value(5, 4.).unwrap();
    assert_ne!(editor.doc.point_numeric_label(10).unwrap().1, before);
    assert!(
        editor
            .doc
            .point_numeric_label(10)
            .unwrap()
            .1
            .contains("4.0")
    );
    assert!(
        editor
            .doc
            .point_numeric_label(12)
            .unwrap()
            .1
            .contains("4.0")
    );
    assert_eq!(
        (9..=12)
            .map(|id| editor.doc.point_numeric_label(id).unwrap().0)
            .collect::<Vec<_>>(),
        anchors
    );
    editor.doc.verify_roundtrip().unwrap();
    editor.doc.undo();
    assert_eq!(editor.doc.point_numeric_label(10).unwrap().1, before);
    editor.doc.edit_text(6, "EDIT_TEXT").unwrap();
    assert_eq!(
        editor.doc.point_numeric_label(9).unwrap().1,
        "TEXT EDIT_TEXT"
    );
    assert_eq!(editor.doc.point_numeric_label(9).unwrap().0, anchors[0]);
    editor.doc.verify_roundtrip().unwrap();
    editor.doc.undo();
    assert_eq!(
        editor.doc.point_numeric_label(9).unwrap().1,
        "TEXT SOURCE_TEXT"
    );
    editor
        .doc
        .set_function_expression(8, SimpleFunction::Identity)
        .unwrap();
    assert!(
        editor
            .doc
            .point_numeric_label(11)
            .unwrap()
            .1
            .ends_with("f(x) = x")
    );
    assert_eq!(editor.doc.point_numeric_label(11).unwrap().0, anchors[2]);
    editor.doc.undo();
    assert!(
        editor
            .doc
            .point_numeric_label(11)
            .unwrap()
            .1
            .ends_with("x²")
    );
}

#[test]
fn anchor_point_drag_and_label_bold_are_independent_editable_properties() {
    let mut editor = shared_reference_editor(false);
    let viewport = shared_reference_viewport();
    let original = editor.doc.point_numeric_label(12).unwrap();
    let other = editor.doc.point_numeric_label(9).unwrap();
    let moved = Point {
        x: original.0.x + 120.,
        y: original.0.y + 40.,
    };
    // The canvas point drag uses this same public move/checkpoint operation.
    editor.doc.checkpoint();
    assert!(editor.doc.move_point(4, moved));
    let changed = editor.doc.point_numeric_label(12).unwrap();
    assert_eq!(changed, (moved, original.1.clone()));
    assert_eq!(editor.doc.point_numeric_label(9).unwrap(), other);
    assert_eq!(
        editor.hit(&viewport, viewport.screen(moved) + egui::vec2(17., 0.)),
        Some(12)
    );
    assert_ne!(
        editor.hit(&viewport, viewport.screen(original.0) + egui::vec2(17., 0.)),
        Some(12)
    );
    editor.doc.verify_roundtrip().unwrap();
    editor.doc.undo();
    assert_eq!(editor.doc.point_numeric_label(12).unwrap(), original);

    editor.select_only(9);
    let before = editor.doc.text_font_state(9).unwrap();
    assert_eq!(
        editor
            .doc
            .toggle_selected_text_style(TextStyleCommand::Bold)
            .unwrap(),
        1
    );
    let changed = editor.doc.text_font_state(9).unwrap();
    assert_eq!(changed.bold, !before.bold);
    assert_eq!(editor.doc.point_numeric_label(9).unwrap(), other);
    let bytes = editor.doc.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.text_font_state(9).unwrap(), changed);
    reopened.verify_roundtrip().unwrap();
    editor.doc.undo();
    assert_eq!(editor.doc.text_font_state(9).unwrap(), before);
}
