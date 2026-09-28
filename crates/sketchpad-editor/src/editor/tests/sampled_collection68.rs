use super::*;

// Native hide-one-object controls identify the role of each exact RGB pixel.
// Canvas world→screen offset is (55,21), independently checked by Free9/10.
// Hiding 23 removes the right-green family. Hiding 22 removes this left-green
// point while leaving all right-green pixels intact. Right navy is another
// object's paint; it is deliberately absent from these binding assertions.
const ORIGINAL_SAMPLED: Point = Point { x: 599., y: 353. }; // Native (654,374), hide23 removes it
const EDITED_SAMPLED: Point = Point { x: 616., y: 308. }; // Native (671,329), edited hide23 removes it
const ORIGINAL_CIRCLE: Point = Point { x: 502., y: 347. }; // Native (557,368), hide22 removes it

fn native_editor(state: &str, visible: &[usize]) -> Editor {
    let bytes = std::fs::read(format!(
        "../../tests/fixtures/sampled_locus68_{state}_native_saved.gsp"
    ))
    .unwrap();
    let mut doc = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(
        doc.encode().unwrap(),
        bytes,
        "{state}: original Native bytes"
    );
    assert!(matches!(doc.objects[18].kind, Kind::SampledMapped(..)));
    assert!(matches!(
        doc.objects[20].kind,
        Kind::MappedPointIteration { .. }
    ));
    assert!(matches!(
        doc.objects[21].kind,
        Kind::IterationBinding(17, 21)
    ));
    assert!(matches!(
        doc.objects[22].kind,
        Kind::IterationBinding(19, 21)
    ));
    assert!(!doc.objects[18].visible, "source19 is an unpainted recipe");
    for object in &mut doc.objects {
        object.visible = object.id != 19 && visible.contains(&object.id);
    }
    Editor::new(doc)
}

fn viewport() -> Viewport {
    Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(3600., 2400.)),
        scale: 2.,
        center: Point { x: 650., y: 330. },
    }
}

#[test]
fn native_sampled_collection_and_circle_binding_have_separate_green_geometry() {
    let view = viewport();
    for (state, sampled) in [("original", ORIGINAL_SAMPLED), ("edited", EDITED_SAMPLED)] {
        let sampled_only = native_editor(state, &[23]);
        assert_eq!(
            sampled_only.hit(&view, view.screen(sampled)),
            Some(23),
            "{state}: Native right-green sampled collection"
        );
        let members = sampled_only.doc.mapped_collection_strokes(23, 64).unwrap();
        assert!(
            members.len() > 100,
            "{state}: green sampled members reach canvas geometry"
        );
        assert!(
            members
                .iter()
                .all(|s| s.style == sampled_only.doc.effective_visual_style(23).unwrap())
        );
        assert!(
            crate::render::svg_page(&sampled_only.doc, &view).is_err(),
            "partial original cannot export SVG"
        );

        // A hidden sampled source produces no painted collection by itself.
        let source_only = native_editor(state, &[19]);
        assert!(!source_only.doc.is_displayed(19));
        assert!(crate::render::svg_page(&source_only.doc, &view).is_err());
    }

    let circle_only = native_editor("original", &[22]);
    assert_eq!(
        circle_only.hit(&view, view.screen(ORIGINAL_CIRCLE)),
        Some(22),
        "Native hide22-only left-green witness"
    );
    assert!(circle_only.doc.circle_iteration_images(22).unwrap().len() >= 3);
    assert_eq!(
        circle_only.doc.effective_visual_style(22),
        circle_only.doc.effective_visual_style(23)
    );
}

#[test]
fn sampled_collection_binding_visibility_and_undo_preserve_native_roles() {
    let view = viewport();
    let mut editor = native_editor("original", &[22, 23]);
    assert_eq!(editor.hit(&view, view.screen(ORIGINAL_SAMPLED)), Some(23));
    assert!(editor.doc.is_displayed(23) && editor.doc.is_displayed(22));
    assert!(editor.doc.mapped_collection_strokes(23, 64).unwrap().len() > 100);
    assert!(editor.doc.circle_iteration_images(22).unwrap().len() >= 3);

    editor.select_only(23);
    editor.doc.set_selected_visibility(false);
    assert!(!editor.doc.is_displayed(23));
    assert!(editor.doc.is_displayed(22));
    assert_ne!(editor.hit(&view, view.screen(ORIGINAL_SAMPLED)), Some(23));
    editor.doc.verify_roundtrip().unwrap();
    editor.doc.undo();
    assert!(editor.doc.is_displayed(23));
    assert_eq!(editor.hit(&view, view.screen(ORIGINAL_SAMPLED)), Some(23));
}

#[test]
fn clean_public_sampled_collection_exports_styled_svg_lines() {
    // The original Native file has an unrelated partial-view export guard.
    // Author the same 102→89→77 relationship in a clean document to exercise
    // the SVG path without weakening that guard.
    let mut doc = Document::new();
    let center = doc.add(Kind::Free(Point { x: 300., y: 300. }));
    let radius = doc.add(Kind::Free(Point { x: 340., y: 300. }));
    let circle = doc.add(Kind::Circle(center, radius));
    let seed = doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let zero = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let plus20 = doc.add(Kind::Free(Point { x: 20., y: 0. }));
    let output = doc.add(Kind::VectorTranslatedPoint(seed, zero, plus20));
    let source = doc
        .add_sampled_mapped(output, circle, seed, &[output], 40)
        .unwrap();
    let plus100 = doc.add(Kind::Free(Point { x: 100., y: 0. }));
    let next = doc.add(Kind::VectorTranslatedPoint(center, zero, plus100));
    let root = doc
        .add_mapped_point_iteration_with_bindings(
            &[center],
            &[vec![next]],
            &[circle, output, source, next],
            &[source],
            sketchpad_engine::document::IterationDepth::Fixed(3),
        )
        .unwrap();
    let binding = root + 1;
    let mut editor = Editor::new(doc);
    editor.select_only(binding);
    assert_eq!(
        editor
            .doc
            .set_selected_color(sketchpad_engine::style_records::RgbColor {
                red: 0,
                green: 128,
                blue: 0,
            })
            .unwrap(),
        1
    );
    for object in &mut editor.doc.objects {
        object.visible = object.id == binding;
    }
    let svg = crate::render::svg_page(&editor.doc, &viewport()).unwrap();
    assert!(
        svg.lines()
            .filter(|line| line.starts_with("<line ") && line.contains("stroke=\"#008000\""))
            .count()
            >= 100
    );
    assert!(!svg.contains("NaN"));
}
