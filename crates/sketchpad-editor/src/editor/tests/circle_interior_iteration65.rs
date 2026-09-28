use super::*;
use sketchpad_engine::style_records::RgbColor;

fn native_editor(state: &str) -> Editor {
    let bytes = std::fs::read(format!(
        "../../tests/fixtures/circle_interior_iteration65_{state}_native_saved.gsp"
    ))
    .unwrap();
    let doc = Document::from_bytes(&bytes, None).unwrap();
    assert!(!doc.partial_view, "{state}");
    Editor::new(doc)
}

fn viewport() -> Viewport {
    Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800., 600.)),
        scale: 1.,
        center: Point { x: 400., y: 300. },
    }
}

#[test]
fn native_disk_generations_pick_filled_interiors_and_render_only_visible_defined_members() {
    let view = viewport();
    // Native's independent disk witnesses determine these interior locations.
    // Sources are hidden, so a circumference or source-disk hit cannot pass.
    for (state, centers, excluded, fills) in [
        ("live", vec![240., 340., 440.], vec![140., 290.], 3),
        ("edited", vec![260., 380., 500.], vec![140.], 3),
        ("frozen_circle", vec![140.], vec![240., 440.], 3),
        ("final_only", vec![440.], vec![240., 340.], 1),
        ("undefined_final", vec![240., 340.], vec![440.], 2),
        ("undefined_final_only", vec![], vec![240., 340., 440.], 0),
        ("recovered", vec![240., 340., 440.], vec![140.], 3),
    ] {
        let editor = native_editor(state);
        for x in centers {
            assert_eq!(
                editor.hit(&view, view.screen(Point { x: x + 5., y: 265. })),
                Some(15),
                "{state}: deep disk interior at {x}"
            );
        }
        for x in excluded {
            assert_eq!(
                editor.hit(&view, view.screen(Point { x, y: 260. })),
                None,
                "{state}: excluded generation or gap at {x}"
            );
        }
        assert_eq!(
            editor.hit(&view, view.screen(Point { x: 620., y: 350. })),
            None,
            "{state}: exterior"
        );
        let svg = crate::render::svg_page(&editor.doc, &view).unwrap();
        assert_eq!(svg.matches("fill=\"#008000\"").count(), fills, "{state}");
        assert!(!svg.contains("NaN"), "{state}");
    }
}

#[test]
fn disk_binding_selection_style_visibility_and_dependency_deletion_survive_undo_and_save() {
    let mut editor = native_editor("live");
    let view = viewport();
    let inside = view.screen(Point { x: 245., y: 265. });
    let source_style = editor.doc.objects[11].visual_style;
    editor.select_only(15);
    assert_eq!(
        editor.doc.selected.iter().copied().collect::<Vec<_>>(),
        vec![15]
    );
    assert_eq!(
        editor
            .doc
            .set_selected_color(RgbColor {
                red: 200,
                green: 0,
                blue: 120
            })
            .unwrap(),
        1
    );
    assert_eq!(editor.doc.objects[11].visual_style, source_style);
    let svg = crate::render::svg_page(&editor.doc, &view).unwrap();
    assert_eq!(svg.matches("fill=\"#c80078\"").count(), 3);
    let reopened = Document::from_bytes(&editor.doc.encode().unwrap(), None).unwrap();
    assert_eq!(
        reopened.objects[14].visual_style,
        editor.doc.objects[14].visual_style
    );
    editor.doc.verify_roundtrip().unwrap();
    editor.doc.undo();
    assert_eq!(
        crate::render::svg_page(&editor.doc, &view)
            .unwrap()
            .matches("fill=\"#008000\"")
            .count(),
        3
    );

    editor.select_only(15);
    editor.doc.set_selected_visibility(false);
    assert_eq!(editor.hit(&view, inside), None);
    assert!(
        !crate::render::svg_page(&editor.doc, &view)
            .unwrap()
            .contains("fill=\"#008000\"")
    );
    editor.doc.verify_roundtrip().unwrap();
    editor.doc.undo();
    assert_eq!(editor.hit(&view, inside), Some(15));

    // Removing the hidden source circle through the editor command must also
    // remove its disk, mapped definition and visible binding, rather than
    // leaving a pickable stale fill or an invalid saved dependency.
    editor.select_only(11);
    editor.clear_selected();
    assert!(editor.doc.objects[10].deleted, "{}", editor.status);
    assert!(editor.doc.objects[11].deleted);
    assert!(editor.doc.objects[13].deleted);
    assert!(editor.doc.objects[14].deleted);
    assert_eq!(editor.hit(&view, inside), None);
    assert!(
        !crate::render::svg_page(&editor.doc, &view)
            .unwrap()
            .contains("fill=\"#008000\"")
    );
    editor.doc.verify_roundtrip().unwrap();
    editor.doc.undo();
    assert_eq!(editor.hit(&view, inside), Some(15));
    editor.doc.redo();
    assert_eq!(editor.hit(&view, inside), None);
    editor.doc.verify_roundtrip().unwrap();
}
