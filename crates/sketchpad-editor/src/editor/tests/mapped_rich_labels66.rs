use super::*;
use sketchpad_engine::document::TextStyleCommand;

fn native_editor(state: &str) -> (Editor, usize, usize) {
    let bytes = std::fs::read(format!(
        "../../tests/fixtures/mapped_rich_labels65_{state}_native_saved.gsp"
    ))
    .unwrap();
    let doc = Document::from_bytes(&bytes, None).unwrap();
    assert!(!doc.partial_view, "{state}");
    let (source, binding) = if state.starts_with("numeric") {
        (9, 12)
    } else {
        (10, 13)
    };
    (Editor::new(doc), source, binding)
}

fn viewport() -> Viewport {
    Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000., 600.)),
        scale: 1.,
        center: Point { x: 500., y: 300. },
    }
}

#[test]
fn native_rich_mapped_labels_share_text_picking_svg_content_and_source_font() {
    let view = viewport();
    for state in ["numeric_live", "numeric_frozen", "plain", "function"] {
        let (mut editor, source, binding) = native_editor(state);
        let labels = editor.doc.mapped_numeric_label_images(binding).unwrap();
        let last = labels.last().unwrap();
        // Use the existing shared label hit rectangle at the evaluated anchor.
        // This tests editor layout, not Native's unchanged cached box centers
        // after the separately spaced raw-source field edits.
        let hit = view.screen(last.0) + egui::vec2(17., 0.);
        assert_eq!(editor.hit(&view, hit), Some(binding), "{state}");
        assert_eq!(
            editor.hit(&view, view.screen(Point { x: 800., y: 350. })),
            None
        );
        editor.click(&view, hit, false);
        assert_eq!(
            editor.doc.selected.iter().copied().collect::<Vec<_>>(),
            vec![binding]
        );
        assert_eq!(
            editor.doc.numeric_label_font_state(binding),
            editor.doc.text_font_state(source)
        );

        let svg = crate::render::svg_page(&editor.doc, &view).unwrap();
        for (_, text) in &labels {
            assert!(svg.contains(text), "{state}: missing evaluated text {text}");
        }
        match state {
            "numeric_live" | "numeric_frozen" => {
                assert_eq!(svg.matches("(1)/(2)").count(), 3);
                assert_eq!(svg.matches("MATH").count(), 3);
            }
            "plain" => assert_eq!(svg.matches("TEXT LIVE_TEXT").count(), 3),
            "function" => {
                assert_eq!(svg.matches("FUNCTION").count(), 3);
                assert_eq!(svg.matches("x²").count(), 3);
            }
            _ => unreachable!(),
        }
        let font = editor.doc.text_font_state(source).unwrap();
        let escaped_family = font
            .family
            .replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('<', "&lt;");
        let rendered_rows: Vec<_> = svg
            .lines()
            .filter(|line| line.contains("text-anchor=\"middle\"") && line.contains(&last.1))
            .collect();
        assert!(!rendered_rows.is_empty(), "{state}");
        for row in rendered_rows {
            assert!(row.contains(&format!("font-family=\"{escaped_family}\"")));
            assert!(row.contains(&format!("font-size=\"{}\"", font.size_code)));
        }
    }
}

#[test]
fn mapped_rich_font_visibility_and_source_deletion_update_editor_and_restore_with_undo() {
    let view = viewport();
    for state in ["numeric_live", "numeric_frozen", "plain", "function"] {
        let (mut editor, source, binding) = native_editor(state);
        let original_labels = editor.doc.mapped_numeric_label_images(binding).unwrap();
        let hit = view.screen(original_labels.last().unwrap().0) + egui::vec2(17., 0.);
        editor.select_only(source);
        let before = editor.doc.text_font_state(source).unwrap();
        assert_eq!(
            editor
                .doc
                .toggle_selected_text_style(TextStyleCommand::Bold)
                .unwrap(),
            1
        );
        let inherited = editor.doc.numeric_label_font_state(binding).unwrap();
        assert_eq!(inherited.bold, !before.bold, "{state}");
        let svg = crate::render::svg_page(&editor.doc, &view).unwrap();
        let last_text = &original_labels.last().unwrap().1;
        let row = svg
            .lines()
            .find(|line| line.contains("text-anchor=\"middle\"") && line.contains(last_text))
            .unwrap();
        assert!(row.contains(if inherited.bold {
            "font-weight=\"bold\""
        } else {
            "font-weight=\"normal\""
        }));
        editor.doc.verify_roundtrip().unwrap();
        editor.doc.undo();
        assert_eq!(editor.doc.numeric_label_font_state(binding), Some(before));

        editor.select_only(binding);
        editor.doc.set_selected_visibility(false);
        assert_eq!(editor.hit(&view, hit), None, "{state}");
        assert!(
            !crate::render::svg_page(&editor.doc, &view)
                .unwrap()
                .contains(last_text)
        );
        editor.doc.verify_roundtrip().unwrap();
        editor.doc.undo();
        assert_eq!(editor.hit(&view, hit), Some(binding), "{state}");

        editor.select_only(source);
        editor.clear_selected();
        assert!(
            editor.doc.objects[source - 1].deleted,
            "{state}: {}",
            editor.status
        );
        assert!(editor.doc.objects[binding - 1].deleted, "{state}");
        assert_eq!(editor.hit(&view, hit), None, "{state}");
        assert!(
            !crate::render::svg_page(&editor.doc, &view)
                .unwrap()
                .contains(last_text)
        );
        editor.doc.verify_roundtrip().unwrap();
        editor.doc.undo();
        assert_eq!(
            editor.doc.mapped_numeric_label_images(binding).unwrap(),
            original_labels
        );
        assert_eq!(editor.hit(&view, hit), Some(binding), "{state}");
        editor.doc.redo();
        assert_eq!(editor.hit(&view, hit), None, "{state}");
        editor.doc.verify_roundtrip().unwrap();
    }
}
