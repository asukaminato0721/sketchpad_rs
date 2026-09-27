use super::*;

#[test]
fn parameter_locus_construction_accepts_both_selection_orders() {
    for order in [vec![1, 9], vec![9, 1]] {
        let doc = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/numeric_cood_trace_native.gsp"),
            None,
        )
        .unwrap();
        let mut editor = Editor::new(doc);
        editor.selection_order = order.clone();
        editor.doc.selected.extend(order);
        editor.construct_locus();
        assert!(matches!(
            editor.doc.objects.last().unwrap().kind,
            Kind::ParameterLocus(_, 0., 1., 500)
        ));
        assert_eq!(editor.doc.objects.len(), 11);
        assert_eq!(editor.doc.selected.len(), 1);
        editor.doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn referenced_function_dialog_captures_sources_and_keeps_alias_order() {
    // This imported document has fewer font styles than the native reference
    // function template, so authoring must resolve styles in this document.
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/source_exponents_base_port.gsp"),
        None,
    )
    .unwrap();
    let parameter = 6;
    doc.set_parameter_value(parameter, 3.).unwrap();
    let primitive = doc.add_function_definition(SimpleFunction::Square).unwrap();
    let point = doc.add(Kind::Free(Point { x: 10., y: 20. }));
    let mut editor = Editor::new(doc);
    editor.doc.selected.extend([parameter, primitive, point]);
    // Selection updates can omit older IDs from the recorded click order.
    editor.selection_order = vec![primitive, point];
    editor.begin_function_dialog(false, None);
    assert_eq!(editor.function_draft_sources, vec![primitive, parameter]);
    editor.doc.selected.clear();
    let expression = SimpleFunction::parse_with_sources("f1(x)+v2", 2).unwrap();
    let definition = editor
        .apply_function_expression(expression, false, None, None)
        .unwrap();
    assert_eq!(
        editor.doc.function_sources(definition),
        Some([primitive, parameter].as_slice())
    );
    assert_eq!(editor.doc.function_value(definition, 2.), Some(7.));
    editor.doc.set_parameter_value(parameter, 5.).unwrap();
    assert_eq!(editor.doc.function_value(definition, 2.), Some(9.));
    editor.doc.verify_roundtrip().unwrap();
}

#[test]
fn referenced_function_edit_preserves_sources_and_rejects_wrong_alias_roles() {
    let mut doc = Document::new();
    let parameter = doc.add_named_parameter("a", 3., (10, 10)).unwrap();
    let primitive = doc.add_function_definition(SimpleFunction::Square).unwrap();
    let definition = doc
        .add_referenced_function_definition(
            &[parameter, primitive],
            SimpleFunction::parse_with_sources("v1*f2(x)", 2).unwrap(),
        )
        .unwrap();
    let mut editor = Editor::new(doc);
    editor.begin_function_dialog(false, Some(definition));
    assert_eq!(editor.function_draft_sources, [parameter, primitive]);
    assert_eq!(editor.function_edit_id, Some(definition));
    assert_eq!(
        SimpleFunction::parse_with_sources(&editor.function_draft, 2).unwrap(),
        SimpleFunction::parse_with_sources("v1*f2(x)", 2).unwrap()
    );
    let before = editor.doc.encode().unwrap();
    let wrong = SimpleFunction::parse_with_sources("f1(x)+v2", 2).unwrap();
    assert!(
        editor
            .apply_function_expression(wrong, false, Some(definition), None)
            .is_err()
    );
    assert_eq!(editor.doc.encode().unwrap(), before);
    let changed = SimpleFunction::parse_with_sources("v1+f2(x+1)", 2).unwrap();
    editor
        .apply_function_expression(changed, false, Some(definition), None)
        .unwrap();
    assert_eq!(editor.doc.function_value(definition, 2.), Some(12.));
    assert_eq!(
        editor.doc.function_sources(definition),
        Some([parameter, primitive].as_slice())
    );
    editor.doc.undo();
    assert_eq!(editor.doc.encode().unwrap(), before);
}

#[test]
fn referenced_function_plot_angle_choice_keeps_sources_and_undoes_together() {
    let mut doc = Document::new();
    let parameter = doc.add_named_parameter("a", 2., (10, 10)).unwrap();
    let mut editor = Editor::new(doc);
    editor.doc.selected.insert(parameter);
    editor.begin_function_dialog(true, None);
    let before = editor.doc.encode().unwrap();
    let expression = SimpleFunction::parse_with_sources("v1*sin(x)", 1).unwrap();
    let plot = editor
        .apply_function_expression(expression, true, None, Some(false))
        .unwrap();
    let definition = match editor.doc.objects[plot - 1].kind {
        Kind::FunctionPlot(definition, ..) => definition,
        _ => panic!("Expected a function plot"),
    };
    assert_eq!(
        editor.doc.function_sources(definition),
        Some([parameter].as_slice())
    );
    assert_eq!(editor.doc.function_value(definition, 90.), Some(2.));
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: editor.center,
    };
    let caption = editor.doc.function_display_text(definition).unwrap();
    assert!(
        editor
            .overlay_rows()
            .iter()
            .any(|(id, text)| *id == definition && *text == caption)
    );
    assert!(
        sketchpad_rs::render::svg_page(&editor.doc, &viewport)
            .unwrap()
            .contains(&caption)
    );
    editor.doc.verify_roundtrip().unwrap();
    editor.doc.undo();
    assert_eq!(editor.doc.encode().unwrap(), before);
    // A subsequent independent creation does not inherit the previous references.
    editor.doc.selected.clear();
    editor.begin_function_dialog(false, None);
    assert!(editor.function_draft_sources.is_empty());
    assert_eq!(editor.function_draft, "x");
}
