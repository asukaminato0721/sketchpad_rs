use super::*;

fn native_editor(state: &str) -> Editor {
    let bytes = std::fs::read(format!(
        "../../tests/fixtures/nested_iteration65_{state}_native_saved.gsp"
    ))
    .unwrap();
    let doc = Document::from_bytes(&bytes, None).unwrap();
    assert!(!doc.partial_view, "{state}");
    Editor::new(doc)
}

fn viewport() -> Viewport {
    Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900., 600.)),
        scale: 1.,
        center: Point { x: 450., y: 300. },
    }
}

#[test]
fn nested_native_samples_pick_bindings_and_terminal_aliases_without_occlusion_inference() {
    let view = viewport();
    for (state, binding, terminal, separated_samples, terminal_x, rendered_members) in [
        ("scalar_live", 16, 17, vec![233.1, 272.8], Some(319.7), 3),
        ("scalar_frozen_isolated", 16, 17, vec![200.], Some(200.), 3),
        ("scalar_undefined", 16, 17, vec![200.], None, 2),
        ("point_live", 18, 19, vec![420., 520.], Some(620.), 3),
    ] {
        let mut editor = native_editor(state);
        for x in separated_samples {
            assert_eq!(
                editor.hit(&view, view.screen(Point { x, y: 180. })),
                Some(binding),
                "{state}: sample {x}"
            );
        }
        let svg = crate::render::svg_page(&editor.doc, &view).unwrap();
        assert_eq!(
            svg.matches("fill=\"#009600\"").count(),
            rendered_members,
            "{state}: sample family"
        );
        // The ordinary source point is also red, but has radius4. The actual
        // terminal alias has a separately saved large red point style.
        assert_eq!(
            svg.matches("r=\"6\" fill=\"#ff0000\"").count(),
            usize::from(terminal_x.is_some()),
            "{state}: terminal family"
        );
        assert!(!svg.contains("NaN"));

        // Iteration binding hit testing precedes ordinary point picking. Hide
        // it explicitly to test the coincident terminal alias independently.
        editor.select_only(binding);
        editor.doc.set_selected_visibility(false);
        if let Some(x) = terminal_x {
            let at = view.screen(Point { x, y: 180. });
            assert_eq!(
                editor.hit(&view, at),
                Some(terminal),
                "{state}: exposed terminal"
            );
            editor.click(&view, at, false);
            assert!(editor.doc.selected.contains(&terminal));
        } else {
            // q2's valid image200 is not a fallback terminal when q3 fails.
            assert_eq!(
                editor.hit(&view, view.screen(Point { x: 200., y: 180. })),
                None
            );
            assert!(
                !crate::render::svg_page(&editor.doc, &view)
                    .unwrap()
                    .contains("r=\"6\" fill=\"#ff0000\"")
            );
        }
        editor.doc.verify_roundtrip().unwrap();
        editor.doc.undo();
        assert!(editor.doc.objects[binding - 1].visible);
    }
}

#[test]
fn hiding_inner_alias_keeps_nested_samples_and_deleting_inputs_removes_editor_outputs_with_undo() {
    let view = viewport();
    for (state, binding, terminal, first_x, original_inner_x, ancestor) in [
        ("scalar_live", 16, 17, 233.1, 200., 6),
        ("point_live", 18, 19, 420., 320., 2),
    ] {
        let mut editor = native_editor(state);
        let before = editor.doc.mapped_point_binding_images(binding).unwrap();
        let sample_at = view.screen(Point {
            x: first_x,
            y: 180.,
        });
        editor.select_only(13);
        editor.doc.set_selected_visibility(false);
        assert_ne!(
            editor.hit(
                &view,
                view.screen(Point {
                    x: original_inner_x,
                    y: 180.
                })
            ),
            Some(13)
        );
        assert_eq!(editor.hit(&view, sample_at), Some(binding));
        assert_eq!(
            editor.doc.mapped_point_binding_images(binding).unwrap(),
            before
        );
        let saved = editor.doc.encode().unwrap();
        let reopened = Document::from_bytes(&saved, None).unwrap();
        assert!(!reopened.partial_view);
        let reopened_editor = Editor::new(reopened);
        assert_eq!(reopened_editor.hit(&view, sample_at), Some(binding));
        editor.doc.verify_roundtrip().unwrap();
        editor.doc.undo();
        assert!(editor.doc.objects[12].visible);

        editor.select_only(ancestor);
        editor.clear_selected();
        assert!(
            editor.doc.objects[ancestor - 1].deleted,
            "{state}: {}",
            editor.status
        );
        assert!(editor.doc.objects[binding - 1].deleted, "{state}");
        assert!(editor.doc.objects[terminal - 1].deleted, "{state}");
        assert_eq!(editor.hit(&view, sample_at), None, "{state}");
        assert!(
            !crate::render::svg_page(&editor.doc, &view)
                .unwrap()
                .contains("fill=\"#009600\"")
        );
        editor.doc.verify_roundtrip().unwrap();
        editor.doc.undo();
        assert_eq!(editor.hit(&view, sample_at), Some(binding), "{state}");
        assert_eq!(
            editor.doc.mapped_point_binding_images(binding).unwrap(),
            before
        );
        editor.doc.redo();
        assert_eq!(editor.hit(&view, sample_at), None, "{state}");
        editor.doc.verify_roundtrip().unwrap();
    }
}
