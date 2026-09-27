use super::*;

#[test]
fn copy_segment_includes_parents_but_not_children_like_original() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let b = doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let segment = doc.add(Kind::Segment(a, b));
    doc.add(Kind::Midpoint(a, b));
    doc.add(Kind::Free(Point { x: 330., y: 200. }));
    doc.selected.insert(segment);
    let clipboard = doc.copy_selected().unwrap();
    assert_eq!(clipboard.len(), 3);
    assert_eq!(doc.paste(&clipboard).unwrap(), vec![6, 7, 8]);
    assert_eq!(doc.objects[5].kind, Kind::Free(Point { x: 170., y: 140. }));
    assert_eq!(doc.objects[6].kind, Kind::Free(Point { x: 270., y: 140. }));
    assert_eq!(doc.objects[7].kind, Kind::Segment(6, 7));
    let original = Document::open(Path::new("tests/fixtures/wine_pasted_segment.gsp")).unwrap();
    assert_eq!(
        doc.objects.iter().map(|o| &o.kind).collect::<Vec<_>>(),
        original.objects.iter().map(|o| &o.kind).collect::<Vec<_>>()
    );
    doc.undo();
    assert_eq!(doc.objects.len(), 5);
    doc.redo();
    assert_eq!(doc.objects.len(), 8);
    let path = std::env::temp_dir().join(format!("sketchpad_rs_paste_{}.gsp", std::process::id()));
    doc.save_to(&path).unwrap();
    assert_eq!(Document::open(&path).unwrap().objects, doc.objects);
    fs::remove_file(path).unwrap();
}
#[test]
fn paste_midpoint_across_documents_preserves_dependencies() {
    let mut source = Document::new();
    let a = source.add(Kind::Free(Point { x: 1., y: 2. }));
    let b = source.add(Kind::Free(Point { x: 5., y: 8. }));
    source.add(Kind::Segment(a, b));
    let midpoint = source.add(Kind::Midpoint(a, b));
    source.selected.extend(1..=midpoint);
    let clipboard = source.copy_selected().unwrap();
    assert_eq!(clipboard.len(), 4);
    let mut dest = Document::new();
    assert_eq!(dest.paste(&clipboard).unwrap(), vec![1, 2, 3, 4]);
    assert_eq!(dest.position(4), Some(Point { x: 23., y: 25. }));
    assert_eq!(
        Document::from_bytes(&dest.encode().unwrap(), None)
            .unwrap()
            .position(4),
        dest.position(4)
    );
}
#[test]
fn clearing_segment_also_clears_its_midpoint() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 10., y: 20. }));
    let b = doc.add(Kind::Free(Point { x: 30., y: 20. }));
    let segment = doc.add(Kind::Segment(a, b));
    let midpoint = doc.add(Kind::Midpoint(a, b));
    doc.selected.insert(segment);
    assert_eq!(doc.clear_selected().unwrap(), vec![segment, midpoint]);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects.len(), 2);
    assert_eq!(reopened.position(1), Some(Point { x: 10., y: 20. }));

    let mut original_shape = Document::new();
    let first = original_shape.add(Kind::Free(Point { x: 150., y: 120. }));
    let second = original_shape.add(Kind::Free(Point { x: 250., y: 120. }));
    let line = original_shape.add(Kind::Segment(first, second));
    original_shape.add(Kind::Midpoint(first, second));
    original_shape.add(Kind::Free(Point { x: 330., y: 200. }));
    original_shape.selected.insert(line);
    original_shape.clear_selected().unwrap();
    let parsed = Document::from_bytes(&original_shape.encode().unwrap(), None).unwrap();
    let wine = Document::open(Path::new("tests/fixtures/wine_cut_segment.gsp")).unwrap();
    assert_eq!(
        parsed.objects.iter().map(|o| &o.kind).collect::<Vec<_>>(),
        wine.objects.iter().map(|o| &o.kind).collect::<Vec<_>>()
    );
}
#[test]
fn clearing_trailing_function_objects_keeps_savable_state() {
    let source = include_bytes!("../../../tests/fixtures/function_plot_square.gsp");
    let mut plot = Document::from_bytes(source, None).unwrap();
    plot.selected.insert(8);
    assert_eq!(plot.clear_selected().unwrap(), vec![8]);
    let reopened = Document::from_bytes(&plot.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects.len(), 7);
    assert!(matches!(
        reopened.objects[6].kind,
        Kind::FunctionDefinition(_)
    ));

    let mut definition = Document::from_bytes(source, None).unwrap();
    definition.selected.insert(7);
    assert_eq!(definition.clear_selected().unwrap(), vec![7, 8]);
    let reopened = Document::from_bytes(&definition.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects.len(), 6);

    let mut unsafe_edit = Document::from_bytes(source, None).unwrap();
    unsafe_edit.selected.insert(1);
    assert!(unsafe_edit.clear_selected().is_err());
    assert!(!unsafe_edit.dirty);
    assert_eq!(unsafe_edit.encode().unwrap(), source);

    let mut with_points = Document::from_bytes(source, None).unwrap();
    let first = with_points.add(Kind::Free(Point { x: 310., y: 190. }));
    let second = with_points.add(Kind::Free(Point { x: 340., y: 210. }));
    let mut saved = Document::from_bytes(&with_points.encode().unwrap(), None).unwrap();
    saved.selected.insert(first);
    assert_eq!(saved.clear_selected().unwrap(), vec![first]);
    let reopened = Document::from_bytes(&saved.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(matches!(
        reopened.objects[7].kind,
        Kind::FunctionPlot(_, _, _, _, _)
    ));
    assert_eq!(
        reopened.position(second - 1),
        Some(Point { x: 340., y: 210. })
    );
}
#[test]
fn clearing_independent_point_before_saved_plot_remaps_its_dependencies() {
    let mut original = Document::new();
    let extra = original.add(Kind::Free(Point { x: 100., y: 100. }));
    let plot = original
        .plot_new_function(SimpleFunction::Square, Point { x: 199., y: 125. })
        .unwrap();
    let mut saved = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
    let before = saved.function_plot_points(plot, 8).unwrap();
    saved.selected.insert(extra);
    assert_eq!(saved.clear_selected().unwrap(), vec![extra]);
    let reopened = Document::from_bytes(&saved.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.function_plot_points(plot - 1, 8).unwrap(), before);
    assert!(matches!(
        reopened.objects[plot - 2].kind,
        Kind::FunctionPlot(7, 6, _, _, _)
    ));
}
#[test]
fn clearing_independent_text_before_saved_plot_remaps_its_dependencies() {
    let mut original = Document::new();
    let text = original.add_text("hello", Point { x: 10., y: 5. }).unwrap();
    let plot = original
        .plot_new_function(SimpleFunction::Square, Point { x: 199., y: 125. })
        .unwrap();
    let mut saved = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
    let before = saved.function_plot_points(plot, 8).unwrap();
    saved.selected.insert(text);
    assert_eq!(saved.clear_selected().unwrap(), vec![text]);
    let reopened = Document::from_bytes(&saved.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.function_plot_points(plot - 1, 8).unwrap(), before);
}
#[test]
fn clearing_earlier_point_remaps_table_and_live_path_sources() {
    let mut original = Document::new();
    let extra = original.add(Kind::Free(Point { x: 100., y: 100. }));
    let parameter = original.add_default_parameter((10, 5)).unwrap();
    original
        .tabulate_default_parameter(parameter, (10, 38))
        .unwrap();
    original
        .add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    original.plot_parameter_on_path(parameter, None).unwrap();
    let mut saved = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
    saved.selected.insert(extra);
    assert_eq!(saved.clear_selected().unwrap(), vec![extra]);
    let reopened = Document::from_bytes(&saved.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(
        matches!(&reopened.objects[1].kind, Kind::ValueTable(1, group) if group.source_id() == 1)
    );
    assert!(matches!(
        reopened.objects[8].kind,
        Kind::PathValuePoint(1, _)
    ));

    let mut pair = Document::new();
    let extra = pair.add(Kind::Free(Point { x: 100., y: 100. }));
    pair.add_default_parameter((10, 5)).unwrap();
    pair.add_default_parameter((10, 38)).unwrap();
    pair.tabulate_default_pair((10, 71)).unwrap();
    let mut saved = Document::from_bytes(&pair.encode().unwrap(), None).unwrap();
    saved.selected.insert(extra);
    assert_eq!(saved.clear_selected().unwrap(), vec![extra]);
    let reopened = Document::from_bytes(&saved.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(
        matches!(&reopened.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.pair_source_ids() == Some([2, 1]))
    );
}
#[test]
fn clearing_parameter_removes_its_table_and_path_point_before_save() {
    let mut original = Document::new();
    original.add(Kind::Free(Point { x: 100., y: 100. }));
    let parameter = original.add_default_parameter((10, 5)).unwrap();
    original
        .tabulate_default_parameter(parameter, (10, 38))
        .unwrap();
    original
        .add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    original.plot_parameter_on_path(parameter, None).unwrap();
    let mut saved = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
    saved.selected.insert(parameter);
    let removed = saved.clear_selected().unwrap();
    assert_eq!(removed.len(), 3);
    let reopened = Document::from_bytes(&saved.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects.len(), 7);
    assert!(
        reopened
            .objects
            .iter()
            .any(|object| matches!(object.kind, Kind::CoordinateSystem(_, _)))
    );
    assert!(reopened.objects.iter().all(|object| !matches!(
        object.kind,
        Kind::Parameter(_) | Kind::ValueTable(_, _) | Kind::PathValuePoint(_, _)
    )));
}
#[test]
fn copying_tables_remaps_embedded_source_ids() {
    let mut one = Document::new();
    let source = one.add_default_parameter((10, 5)).unwrap();
    let table = one.tabulate_default_parameter(source, (10, 38)).unwrap();
    one.selected.insert(table);
    let clipboard = one.copy_selected().unwrap();
    let pasted = one.paste(&clipboard).unwrap();
    assert_eq!(pasted.len(), 2);
    let reopened = Document::from_bytes(&one.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(
        matches!(&reopened.objects[pasted[1] - 1].kind, Kind::ValueTable(id, group) if *id == pasted[0] && group.source_id() == pasted[0] as u32)
    );
    assert!(
        matches!(&reopened.objects[pasted[0] - 1].kind, Kind::Parameter(group) if group.anchor() == (30, 25))
    );
    assert!(
        matches!(&reopened.objects[pasted[1] - 1].kind, Kind::ValueTable(_, group) if group.anchor() == (30, 58))
    );

    let mut pair = Document::new();
    pair.add_default_parameter((10, 5)).unwrap();
    pair.add_default_parameter((10, 38)).unwrap();
    let table = pair.tabulate_default_pair((10, 71)).unwrap();
    pair.selected.insert(table);
    let clipboard = pair.copy_selected().unwrap();
    let pasted = pair.paste(&clipboard).unwrap();
    assert_eq!(pasted.len(), 3);
    let reopened = Document::from_bytes(&pair.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(
        matches!(&reopened.objects[pasted[2] - 1].kind, Kind::ValueTablePair(first, second, group) if *first == pasted[1] && *second == pasted[0] && group.pair_source_ids() == Some([pasted[1] as u32, pasted[0] as u32]))
    );
    assert!(
        matches!(&reopened.objects[pasted[2] - 1].kind, Kind::ValueTablePair(_, _, group) if group.anchor() == (30, 91))
    );
}
#[test]
fn copying_function_plot_keeps_its_frame_style_and_curve() {
    let mut original = Document::new();
    let plot = original
        .plot_new_function(SimpleFunction::Square, Point { x: 199., y: 125. })
        .unwrap();
    let mut saved = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
    let before = saved.function_plot_points(plot, 8).unwrap();
    saved.selected.insert(plot);
    let clipboard = saved.copy_selected().unwrap();
    let pasted = saved.paste(&clipboard).unwrap();
    assert_eq!(pasted.len(), 8);
    let reopened = Document::from_bytes(&saved.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.frame_styles.get(&pasted[5]),
        Some(&(GridForm::Square, false))
    );
    assert!(reopened.objects[pasted[0] - 1].visible);
    assert!(
        pasted[1..=5]
            .iter()
            .all(|id| !reopened.objects[*id - 1].visible)
    );
    assert_eq!(
        reopened.objects[pasted[6] - 1].function_anchor,
        Some((30, 25))
    );
    assert!(
        matches!(reopened.objects[pasted[7] - 1].kind, Kind::FunctionPlot(definition, frame, _, _, _) if definition == pasted[6] && frame == pasted[5])
    );
    let after = reopened.function_plot_points(pasted[7], 8).unwrap();
    assert!(
        before
            .iter()
            .zip(after)
            .all(|(a, b)| (a.x + 20. - b.x).abs() < 1e-8 && (a.y + 20. - b.y).abs() < 1e-8)
    );
}
#[test]
fn copying_selected_function_includes_its_plot_and_hides_frame_parts() {
    let mut original = Document::new();
    let plot = original
        .plot_new_function(SimpleFunction::Square, Point { x: 199., y: 125. })
        .unwrap();
    let mut saved = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
    saved.selected.insert(plot - 1);
    let clipboard = saved.copy_selected().unwrap();
    assert_eq!(clipboard.len(), 8);
    let pasted = saved.paste(&clipboard).unwrap();
    let reopened = Document::from_bytes(&saved.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(reopened.objects[pasted[0] - 1].visible);
    assert!(
        pasted[1..=5]
            .iter()
            .all(|id| !reopened.objects[*id - 1].visible)
    );
    assert!(reopened.objects[pasted[6] - 1].visible);
    assert!(reopened.objects[pasted[7] - 1].visible);
}
#[test]
fn copying_selected_parametric_source_includes_curve_and_hides_frame_parts() {
    let mut native = Document::open(Path::new(
        "tests/fixtures/parametric_rectangular_original.gsp",
    ))
    .unwrap();
    let before = native.parametric_plot_segments(9, 8).unwrap();
    native.selected.insert(1);
    let clipboard = native.copy_selected().unwrap();
    assert_eq!(clipboard.len(), 9);
    let pasted = native.paste(&clipboard).unwrap();
    let reopened = Document::from_bytes(&native.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.objects[pasted[0] - 1].function_anchor,
        Some((30, 25))
    );
    assert_eq!(
        reopened.objects[pasted[1] - 1].function_anchor,
        Some((30, 69))
    );
    assert_eq!(
        reopened.function_display_text(pasted[0]).as_deref(),
        Some("g(x) = x")
    );
    assert_eq!(
        reopened.function_display_text(pasted[1]).as_deref(),
        Some("f(x) = x²")
    );
    assert!(reopened.objects[pasted[2] - 1].visible);
    assert!(
        pasted[3..=7]
            .iter()
            .all(|id| !reopened.objects[*id - 1].visible)
    );
    assert!(
        matches!(reopened.objects[pasted[8] - 1].kind, Kind::ParametricPlot(x, y, frame, _, _, _, _) if x == pasted[0] && y == pasted[1] && frame == pasted[7])
    );
    let after = reopened.parametric_plot_segments(pasted[8], 8).unwrap();
    assert!(before.iter().zip(after).all(|((a, b), (c, d))| {
        (a.x + 20. - c.x).abs() < 1e-8
            && (a.y + 20. - c.y).abs() < 1e-8
            && (b.x + 20. - d.x).abs() < 1e-8
            && (b.y + 20. - d.y).abs() < 1e-8
    }));

    let mut second = Document::open(Path::new(
        "tests/fixtures/parametric_rectangular_original.gsp",
    ))
    .unwrap();
    second.selected.insert(2);
    let clipboard = second.copy_selected().unwrap();
    let pasted = second.paste(&clipboard).unwrap();
    let reopened = Document::from_bytes(&second.encode().unwrap(), None).unwrap();
    assert_eq!(
        reopened.function_display_text(pasted[0]).as_deref(),
        Some("f(x) = x")
    );
    assert_eq!(
        reopened.function_display_text(pasted[1]).as_deref(),
        Some("g(x) = x²")
    );
}
#[test]
fn native_copied_parametric_functions_keep_their_saved_names() {
    let native = Document::open(Path::new(
        "tests/fixtures/parametric_function_copied_native.gsp",
    ))
    .unwrap();
    assert!(!native.partial_view);
    assert_eq!(
        native.function_display_text(10).as_deref(),
        Some("f(x) = x²")
    );
    assert_eq!(
        native.function_display_text(12).as_deref(),
        Some("g(x) = x")
    );
    let second = Document::open(Path::new(
        "tests/fixtures/parametric_second_function_copied_native.gsp",
    ))
    .unwrap();
    assert!(!second.partial_view);
    assert_eq!(
        second.function_display_text(10).as_deref(),
        Some("f(x) = x")
    );
    assert_eq!(
        second.function_display_text(12).as_deref(),
        Some("g(x) = x²")
    );
}
#[test]
fn native_copied_plot_imports_and_moves_as_an_editable_curve() {
    let mut native =
        Document::open(Path::new("tests/fixtures/function_plot_copied_native.gsp")).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.objects[9].function_anchor, Some((30, 25)));
    let before = native.function_plot_points(16, 8).unwrap();
    assert!(native.move_point(9, Point { x: 239., y: 145. }));
    let reopened = Document::from_bytes(&native.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    let after = reopened.function_plot_points(16, 8).unwrap();
    assert!(
        before
            .iter()
            .zip(after)
            .all(|(a, b)| (a.x + 20. - b.x).abs() < 1e-8 && (a.y - b.y).abs() < 1e-8)
    );
}
#[test]
fn midpoint_tracks_its_specific_segment_when_segments_share_endpoints() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 10., y: 20. }));
    let b = doc.add(Kind::Free(Point { x: 30., y: 20. }));
    let first = doc.add(Kind::Segment(a, b));
    let second = doc.add(Kind::Segment(a, b));
    let midpoint = doc.add(Kind::Midpoint(a, b));
    doc.objects[midpoint - 1].midpoint_host = Some(second);
    let mut loaded = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(loaded.objects[midpoint - 1].midpoint_host, Some(second));
    loaded.selected.insert(first);
    assert_eq!(loaded.clear_selected().unwrap(), vec![first]);
    let after = Document::from_bytes(&loaded.encode().unwrap(), None).unwrap();
    assert_eq!(after.objects[3].midpoint_host, Some(3));
    assert_eq!(after.position(4), Some(Point { x: 20., y: 20. }));
}
#[test]
fn paste_refusal_leaves_unsupported_document_unchanged() {
    let mut source = Document::new();
    let point = source.add(Kind::Free(Point { x: 10., y: 20. }));
    source.selected.insert(point);
    let clipboard = source.copy_selected().unwrap();
    let mut dest = Document::open(Path::new("tests/fixtures/example_tools.gsp")).unwrap();
    let before = dest.encode().unwrap();
    assert!(dest.paste(&clipboard).is_err());
    assert_eq!(dest.encode().unwrap(), before);
    assert!(!dest.dirty);
}
#[test]
fn clearing_source_removes_dependents_and_survives_save() {
    let mut doc = Document::new();
    let first = doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let second = doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let segment = doc.add(Kind::Segment(first, second));
    let midpoint = doc.add(Kind::Midpoint(first, second));
    let unrelated = doc.add(Kind::Free(Point { x: 330., y: 200. }));
    doc.selected.insert(first);
    assert_eq!(
        doc.clear_selected().unwrap(),
        vec![first, segment, midpoint]
    );
    assert_eq!(doc.position(first), None);
    assert_eq!(doc.position(second), Some(Point { x: 250., y: 120. }));
    doc.undo();
    assert_eq!(doc.position(midpoint), Some(Point { x: 200., y: 120. }));
    doc.redo();
    assert_eq!(doc.position(midpoint), None);
    let bytes = doc.encode().unwrap();
    let parsed = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(parsed.objects.len(), 2);
    assert_eq!(parsed.position(1), doc.position(second));
    assert_eq!(parsed.position(2), doc.position(unrelated));
    assert!(!parsed.partial_view);
    let wine = Document::open(Path::new("tests/fixtures/wine_deleted_source.gsp")).unwrap();
    assert_eq!(
        wine.objects.iter().map(|o| &o.kind).collect::<Vec<_>>(),
        parsed.objects.iter().map(|o| &o.kind).collect::<Vec<_>>()
    );
    let path = std::env::temp_dir().join(format!("sketchpad_rs_clear_{}.gsp", std::process::id()));
    doc.save_to(&path).unwrap();
    assert_eq!(doc.objects.len(), 2);
    assert_eq!(Document::open(&path).unwrap().objects, doc.objects);
    fs::remove_file(path).unwrap();
}
#[test]
fn clearing_independent_object_remaps_surviving_references() {
    let mut doc = Document::new();
    let removed = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let a = doc.add(Kind::Free(Point { x: 10., y: 10. }));
    let b = doc.add(Kind::Free(Point { x: 20., y: 10. }));
    let segment = doc.add(Kind::Segment(a, b));
    let middle = doc.add(Kind::Midpoint(a, b));
    doc.selected.insert(removed);
    assert_eq!(doc.clear_selected().unwrap(), vec![removed]);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects.len(), 4);
    assert_eq!(reopened.objects[2].kind, Kind::Segment(1, 2));
    assert_eq!(reopened.objects[3].kind, Kind::Midpoint(1, 2));
    assert_eq!(reopened.position(4), doc.position(middle));
    assert_eq!(reopened.line_points(3), doc.line_points(segment));
}
#[test]
fn clear_then_construct_and_unsupported_document_guard() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let b = doc.add(Kind::Free(Point { x: 10., y: 0. }));
    doc.add(Kind::Segment(a, b));
    doc.selected.insert(a);
    assert_eq!(doc.clear_selected().unwrap(), vec![1, 3]);
    let c = doc.add(Kind::Free(Point { x: 20., y: 0. }));
    doc.add(Kind::Segment(b, c));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects.len(), 3);
    assert_eq!(reopened.objects[2].kind, Kind::Segment(1, 2));

    let mut source = Document::open(Path::new("tests/fixtures/segment.gsp")).unwrap();
    source.selected.insert(1);
    assert_eq!(source.clear_selected().unwrap(), vec![1, 3]);
    let reopened = Document::from_bytes(&source.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects.len(), 1);
    assert_eq!(reopened.objects[0].kind, source.objects[1].kind);

    let mut unsupported = Document::open(Path::new("tests/fixtures/example_tools.gsp")).unwrap();
    unsupported.selected.insert(1);
    assert!(unsupported.clear_selected().is_err());
    assert_eq!(
        unsupported.encode().unwrap(),
        fs::read("tests/fixtures/example_tools.gsp").unwrap()
    );
}
