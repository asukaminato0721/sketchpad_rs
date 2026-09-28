use super::*;
use sketchpad_engine::action_records::{VisibilityAction, VisibilityOptions};

fn viewport() -> Viewport {
    Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(500., 400.)),
        scale: 1.,
        center: Point { x: 250., y: 200. },
    }
}

fn click_presentation(editor: &mut Editor, button: usize) {
    let Kind::PresentationButton { label, anchor, .. } = &editor.doc.objects[button - 1].kind
    else {
        panic!("Expected Presentation button");
    };
    let v = viewport();
    let click = editor.action_button_rect(&v, *anchor, label).center();
    editor.click(&v, click, false);
}

#[test]
fn hidden_visibility_child_materializes_caption_and_matches_native_two_clicks() {
    let input =
        include_bytes!("../../../../../tests/fixtures/visibility48_a0_o4_missing_child_input.gsp");
    let mut editor = Editor::new(Document::from_bytes(input, None).unwrap());
    let child = editor
        .doc
        .objects
        .iter()
        .find(|object| matches!(object.kind, Kind::HideShowButton { .. }))
        .unwrap()
        .id;
    let parent = editor
        .doc
        .objects
        .iter()
        .find(|object| matches!(object.kind, Kind::PresentationButton { .. }))
        .unwrap()
        .id;
    assert!(!editor.doc.objects[child - 1].visible);
    assert!(editor.doc.activate_hide_show_button(child).is_err());
    for expected in [
        include_bytes!(
            "../../../../../tests/fixtures/visibility48_a0_o4_missing_child_step1_native_verified.gsp"
        )
        .as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/visibility48_a0_o4_missing_child_step2_native_verified.gsp"
        )
        .as_slice(),
    ] {
        let before = editor.doc.encode().unwrap();
        click_presentation(&mut editor, parent);
        let native = Document::from_bytes(expected, None).unwrap();
        assert_eq!(
            editor.doc.objects[child - 1].kind,
            native.objects[child - 1].kind
        );
        for (actual, expected) in editor.doc.objects.iter().zip(&native.objects) {
            assert_eq!(actual.visible, expected.visible, "object {}", actual.id);
        }
        assert!(!editor.doc.objects[child - 1].visible);
        editor.doc.verify_roundtrip().unwrap();
        let after = editor.doc.encode().unwrap();
        editor.doc.undo();
        assert_eq!(editor.doc.encode().unwrap(), before);
        editor.doc.redo();
        assert_eq!(editor.doc.encode().unwrap(), after);
    }
}

#[test]
fn presentations_apply_shared_fixed_and_alternating_visibility_to_mixed_targets() {
    for mode in [PresentationMode::Simultaneous, PresentationMode::Sequential] {
        for word in 0..8 {
            for initial_action in [VisibilityAction::Hide, VisibilityAction::Show] {
                let mut doc = Document::new();
                let first = doc.add(Kind::Free(Point { x: 160., y: 160. }));
                let second = doc.add(Kind::Free(Point { x: 260., y: 160. }));
                doc.objects[second - 1].visible = false;
                let options = VisibilityOptions::from_native(word).unwrap();
                let child = doc
                    .add_visibility_button_for_targets(&[first, second], initial_action, options)
                    .unwrap();
                let parent = doc
                    .add_presentation_button_for_targets(&[child], mode)
                    .unwrap();
                let mut editor = Editor::new(doc);
                let mut action = initial_action;
                for _ in 0..2 {
                    click_presentation(&mut editor, parent);
                    let visible = action == VisibilityAction::Show;
                    assert_eq!(editor.doc.objects[first - 1].visible, visible);
                    assert_eq!(editor.doc.objects[second - 1].visible, visible);
                    if word >= 4 {
                        action = if visible {
                            VisibilityAction::Hide
                        } else {
                            VisibilityAction::Show
                        };
                    }
                    assert!(matches!(editor.doc.objects[child - 1].kind,
                        Kind::HideShowButton { action: next, options: retained, .. }
                        if next == action && retained == options));
                    editor.doc.verify_roundtrip().unwrap();
                }
            }
        }
    }
}
