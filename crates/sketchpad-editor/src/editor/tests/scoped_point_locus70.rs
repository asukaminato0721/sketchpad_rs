use super::*;

fn native_bytes(state: &str) -> Vec<u8> {
    std::fs::read(format!(
        "../../tests/fixtures/scoped_point_locus70_{state}_native_saved.gsp"
    ))
    .unwrap()
}

fn viewport() -> Viewport {
    Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(3200., 2400.)),
        scale: 2.,
        center: Point { x: 630., y: 300. },
    }
}

fn isolated_binding(doc: &Document, binding: usize) -> Editor {
    let mut isolated = doc.clone();
    for object in &mut isolated.objects {
        object.visible &= object.id == binding;
    }
    Editor::new(isolated)
}

fn binding_midpoints(doc: &Document, binding: usize, generations: usize) -> Vec<Point> {
    // Render and hit-testing both sample 384 pieces per generation. These
    // points are deliberately an internal editor integration witness; Native
    // pixel geometry is asserted separately only after mask calibration.
    let strokes = doc.mapped_collection_strokes(binding, 384).unwrap();
    assert_eq!(strokes.len() % generations, 0);
    let per_generation = strokes.len() / generations;
    assert!(per_generation >= 64);
    (0..generations)
        .map(|generation| {
            let stroke = &strokes[generation * per_generation + per_generation / 2];
            Point {
                x: (stroke.start.x + stroke.end.x) / 2.,
                y: (stroke.start.y + stroke.end.y) / 2.,
            }
        })
        .collect()
}

fn assert_hit_each_generation(doc: &Document, binding: usize, generations: usize) {
    let editor = isolated_binding(doc, binding);
    let view = viewport();
    for (generation, at) in binding_midpoints(doc, binding, generations)
        .into_iter()
        .enumerate()
    {
        assert_eq!(
            editor.hit(&view, view.screen(at)),
            Some(binding),
            "generation {generation}: {at:?}"
        );
    }
    assert_eq!(
        editor.hit(&view, view.screen(Point { x: 100., y: 800. })),
        None
    );
}

#[test]
fn native_scoped_point_locus_binds_three_pickable_svg_traces_and_respects_hide() {
    let original = native_bytes("original");
    let hidden = native_bytes("hide_binding");
    let doc = Document::from_bytes(&original, None).unwrap();
    assert!(!doc.partial_view);
    assert!(matches!(doc.objects[16].kind, Kind::Locus(..)));
    assert!(matches!(
        doc.objects[17].kind,
        Kind::MappedPointIteration { .. }
    ));
    assert!(matches!(
        doc.objects[18].kind,
        Kind::IterationBinding(17, 18)
    ));
    assert_eq!(doc.encode().unwrap(), original);
    assert_hit_each_generation(&doc, 19, 3);

    let isolated = isolated_binding(&doc, 19);
    let svg = crate::render::svg_page(&isolated.doc, &viewport()).unwrap();
    let style = doc.effective_visual_style(19).unwrap();
    let sketchpad_engine::style_records::VisualStyle::Stroke { color, .. } = style else {
        panic!("Native binding19 is a stroke");
    };
    let color = format!(
        "stroke=\"#{:02x}{:02x}{:02x}\"",
        color.red, color.green, color.blue
    );
    assert_eq!(color, "stroke=\"#800000\"", "Native dark-red binding19");
    assert_eq!(
        svg.lines()
            .filter(|line| line.starts_with("<line ") && line.contains(&color))
            .count(),
        3 * 210,
        "three independently sampled traces in the stored binding style"
    );

    let mut editor = Editor::new(doc);
    editor.select_only(19);
    editor.doc.set_selected_visibility(false);
    assert_eq!(editor.doc.encode().unwrap(), hidden);
    assert!(!editor.doc.is_displayed(19));
    let hidden_editor = isolated_binding(&editor.doc, 19);
    for point in binding_midpoints(&editor.doc, 19, 3) {
        assert_ne!(
            hidden_editor.hit(&viewport(), viewport().screen(point)),
            Some(19)
        );
    }
    let hidden_svg = crate::render::svg_page(&hidden_editor.doc, &viewport()).unwrap();
    assert!(!hidden_svg.contains(&color));
    editor.doc.verify_roundtrip().unwrap();
    editor.doc.undo();
    assert_eq!(editor.doc.encode().unwrap(), original);
}

#[test]
fn parameter_edit_reopens_three_live_trace_hits() {
    let original = native_bytes("original");
    let edited = native_bytes("edited");
    let mut editor = Editor::new(Document::from_bytes(&original, None).unwrap());
    editor.doc.set_parameter_value(13, 0.5).unwrap();
    assert_eq!(editor.doc.encode().unwrap(), edited);
    assert_hit_each_generation(&editor.doc, 19, 3);
    let reopened = Document::from_bytes(&edited, None).unwrap();
    assert_hit_each_generation(&reopened, 19, 3);
    assert_eq!(reopened.encode().unwrap(), edited);
    // Native's additional edited dark-red triangle is the directly visible
    // source Locus17, not a fourth mapped binding19 generation.
    let source_strokes = reopened.locus_strokes(17, 384).unwrap();
    assert!(!source_strokes.is_empty());
    let (a, b, _) = source_strokes[source_strokes.len() / 2];
    let midpoint = Point {
        x: (a.x + b.x) / 2.,
        y: (a.y + b.y) / 2.,
    };
    let source_only = isolated_binding(&reopened, 17);
    assert_eq!(
        source_only.hit(&viewport(), viewport().screen(midpoint)),
        Some(17)
    );
}

#[test]
fn clean_public_locus_iteration_renders_three_separate_rows() {
    let mut doc = Document::new();
    let start = doc.add(Kind::Free(Point { x: 100., y: 100. }));
    let end = doc.add(Kind::Free(Point { x: 200., y: 100. }));
    let path = doc.add(Kind::Segment(start, end));
    let driver = doc.add(Kind::PointOnSegment(path, 0.25));
    let zero = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let plus = doc.add(Kind::Free(Point { x: 0., y: 40. }));
    let seed = doc.add(Kind::Free(Point { x: 0., y: 100. }));
    let target = doc.add(Kind::VectorTranslatedPoint(driver, zero, seed));
    let locus = doc.construct_locus(driver, target).unwrap();
    let next = doc.add(Kind::VectorTranslatedPoint(seed, zero, plus));
    let root = doc
        .add_mapped_point_iteration_with_bindings(
            &[seed],
            &[vec![next]],
            &[target, locus, next],
            &[locus],
            sketchpad_engine::document::IterationDepth::Fixed(3),
        )
        .unwrap();
    let binding = root + 1;
    assert!(!doc.partial_view);
    let editor = isolated_binding(&doc, binding);
    let view = viewport();
    for y in [240., 280., 320.] {
        assert_eq!(
            editor.hit(&view, view.screen(Point { x: 150., y })),
            Some(binding)
        );
    }
    let svg = crate::render::svg_page(&editor.doc, &view).unwrap();
    assert_eq!(
        svg.lines()
            .filter(|line| line.starts_with("<line "))
            .count(),
        3 * 384
    );
    let bytes = doc.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert_hit_each_generation(&reopened, binding, 3);
}
