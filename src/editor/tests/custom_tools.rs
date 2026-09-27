use super::*;

fn square_editor() -> Editor {
    Editor::new(
        Document::from_bytes(
            include_bytes!(
                "../../../tests/fixtures/embedded_tool_square_registered_native_verified.gsp"
            ),
            None,
        )
        .unwrap(),
    )
}

fn viewport() -> Viewport {
    Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(620., 450.)),
        scale: 1.,
        center: Point { x: 310., y: 225. },
    }
}

#[test]
fn actual_desktop_tool_saves_preserve_ordered_geometry_and_hidden_helpers() {
    for (bytes, count) in [
        (
            include_bytes!("../../../tests/fixtures/embedded_tool_square_desktop_clicked.gsp")
                .as_slice(),
            14,
        ),
        (
            include_bytes!("../../../tests/fixtures/embedded_tool_square_desktop_selected.gsp")
                .as_slice(),
            25,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects.len(), count);
        let a = Point { x: 373., y: 241.5 };
        let b = Point {
            x: 406.3333333333333,
            y: 241.5,
        };
        assert_eq!(doc.position(2), Some(a));
        assert_eq!(doc.position(3), Some(b));
        assert_eq!(doc.objects[13].kind, Kind::Polygon(vec![2, 3, 10, 8]));
        assert!((doc.position(8).unwrap().y - 208.16666666666669).abs() < 1e-8);
        for id in [5, 6, 7, 9] {
            assert!(!doc.objects[id - 1].visible);
        }
        if count == 25 {
            assert_eq!(doc.objects[14].kind, Kind::Segment(3, 2));
            assert_eq!(doc.objects[24].kind, Kind::Polygon(vec![3, 2, 21, 19]));
            assert!((doc.position(19).unwrap().y - 274.8333333333333).abs() < 1e-8);
            for id in [16, 17, 18, 20] {
                assert!(!doc.objects[id - 1].visible);
            }
        }
        assert_eq!(
            doc.embedded_tools().unwrap()[0].input_ids.as_deref(),
            Some(&[2, 1][..])
        );
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn selected_tool_inputs_keep_selection_order_and_select_visible_outputs() {
    let mut editor = square_editor();
    let a = editor.doc.add(Kind::Free(Point { x: 200., y: 180. }));
    let b = editor.doc.add(Kind::Free(Point { x: 260., y: 180. }));
    editor.selection_order = vec![b, a];
    editor.doc.selected.extend([a, b]);
    let before = editor.doc.encode().unwrap();
    let history = editor.doc.undo.len();
    assert!(editor.invoke_selected_tool(0), "{}", editor.status);
    assert_eq!(editor.doc.undo.len(), history + 1);
    let polygon = editor
        .doc
        .objects
        .iter()
        .find(|object| object.kind.is_polygon())
        .unwrap()
        .id;
    let points = editor.doc.polygon_points(polygon).unwrap();
    assert_eq!(
        points,
        vec![
            Point { x: 260., y: 180. },
            Point { x: 200., y: 180. },
            Point { x: 200., y: 240. },
            Point { x: 260., y: 240. },
        ]
    );
    assert!(!editor.selection_order.is_empty());
    assert!(
        editor
            .selection_order
            .iter()
            .all(|id| editor.doc.selected.contains(id)
                && editor.doc.objects[*id - 1].visible
                && *id != a
                && *id != b)
    );
    let after = editor.doc.encode().unwrap();
    editor.doc.undo();
    assert_eq!(editor.doc.encode().unwrap(), before);
    editor.doc.redo();
    assert_eq!(editor.doc.encode().unwrap(), after);
}

#[test]
fn tool_blank_clicks_create_declared_point_inputs_and_allow_repeating() {
    let mut editor = square_editor();
    let view = viewport();
    editor.begin_custom_tool(0);
    assert!(editor.tool == Tool::Custom(0));
    let before_count = editor.doc.objects.len();
    editor.click(&view, view.screen(Point { x: 200., y: 180. }), false);
    assert_eq!(editor.doc.objects.len(), before_count + 1);
    assert_eq!(editor.custom_tool_inputs, vec![before_count + 1]);
    editor.click(&view, view.screen(Point { x: 260., y: 180. }), false);
    assert!(
        editor.status.starts_with("Applied custom tool"),
        "{}",
        editor.status
    );
    assert!(editor.custom_tool_inputs.is_empty());
    assert!(!editor.selection_order.is_empty());
    let polygon = editor
        .doc
        .objects
        .iter()
        .find(|object| object.kind.is_polygon())
        .unwrap()
        .id;
    assert_eq!(
        editor.doc.polygon_points(polygon).unwrap(),
        vec![
            Point { x: 200., y: 180. },
            Point { x: 260., y: 180. },
            Point { x: 260., y: 120. },
            Point { x: 200., y: 120. },
        ]
    );
    editor.click(&view, view.screen(Point { x: 400., y: 280. }), false);
    assert_eq!(editor.custom_tool_inputs.len(), 1);
    assert_eq!(editor.selection_order, editor.custom_tool_inputs);
    let with_input = editor.doc.encode().unwrap();
    editor.cancel_custom_tool();
    assert!(editor.tool == Tool::Select);
    assert!(editor.custom_tool_inputs.is_empty());
    assert_eq!(editor.doc.encode().unwrap(), with_input);
    editor.doc.verify_roundtrip().unwrap();
}

#[test]
fn tool_point_input_on_segment_creates_a_live_constrained_point() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 100., y: 100. }));
    let b = editor.doc.add(Kind::Free(Point { x: 180., y: 100. }));
    let host = editor.doc.add(Kind::Segment(a, b));
    editor.select_only(host);
    editor.doc.selected.extend([a, b]);
    editor.tool_name_draft = "Join points".into();
    assert!(editor.create_selected_tool(), "{}", editor.status);
    editor.begin_custom_tool(0);
    let view = viewport();
    editor.click(&view, view.screen(Point { x: 140., y: 100. }), false);
    let input = *editor.custom_tool_inputs.first().unwrap();
    assert_ne!(input, host);
    assert_eq!(
        editor.doc.objects[input - 1].kind,
        Kind::PointOnSegment(host, 0.5)
    );
    editor.click(&view, view.screen(Point { x: 250., y: 200. }), false);
    assert!(
        editor.status.starts_with("Applied custom tool"),
        "{}",
        editor.status
    );
    let output = *editor.selection_order.last().unwrap();
    let Kind::Segment(first, second) = editor.doc.objects[output - 1].kind else {
        panic!("Expected the tool's segment output");
    };
    assert_eq!(first, input);
    assert_eq!(
        editor.doc.position(second),
        Some(Point { x: 250., y: 200. })
    );
    assert!(editor.doc.move_point(b, Point { x: 220., y: 140. }));
    assert_eq!(editor.doc.position(input), Some(Point { x: 160., y: 120. }));
    assert_eq!(
        editor.doc.line_points(output).unwrap().0,
        Point { x: 160., y: 120. }
    );
    let encoded = editor.doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.position(input), Some(Point { x: 160., y: 120. }));
    assert_eq!(
        reopened.objects[output - 1].kind,
        Kind::Segment(input, second)
    );
    editor.doc.verify_roundtrip().unwrap();
}

#[test]
fn tool_input_errors_preserve_document_selection_and_history() {
    let mut editor = square_editor();
    editor.select_only(1);
    let before = editor.doc.encode().unwrap();
    let history = editor.doc.undo.len();
    let selected = editor.doc.selected.clone();
    assert!(!editor.invoke_selected_tool(0));
    assert!(!editor.status.is_empty());
    assert_eq!(editor.doc.encode().unwrap(), before);
    assert_eq!(editor.doc.undo.len(), history);
    assert_eq!(editor.doc.selected, selected);
    assert_eq!(editor.selection_order, vec![1]);
    editor.begin_custom_tool(usize::MAX);
    assert!(editor.tool == Tool::Select);
    assert_eq!(editor.doc.selected, selected);
}

#[test]
fn unsupported_tool_binding_does_not_create_input_points() {
    let mut editor = Editor::new(
        Document::from_bytes(
            include_bytes!("../../../tests/fixtures/embedded_tool_12t_original.gsp"),
            None,
        )
        .unwrap(),
    );
    let before = editor.doc.encode().unwrap();
    let history = editor.doc.undo.len();
    editor.begin_custom_tool(0);
    assert!(editor.tool == Tool::Select);
    assert!(!editor.status.is_empty());
    assert_eq!(editor.doc.encode().unwrap(), before);
    assert_eq!(editor.doc.undo.len(), history);
}

#[test]
fn create_tool_dialog_uses_selected_construction_and_is_undoable() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 100., y: 100. }));
    let b = editor.doc.add(Kind::Free(Point { x: 180., y: 100. }));
    let segment = editor.doc.add(Kind::Segment(a, b));
    editor.select_only(segment);
    editor.doc.selected.extend([a, b]);
    editor.create_tool_dialog = true;
    let before = editor.doc.encode().unwrap();
    let history = editor.doc.undo.len();
    editor.tool_name_draft = " ".into();
    assert!(!editor.create_selected_tool());
    assert!(editor.create_tool_dialog);
    assert_eq!(editor.doc.encode().unwrap(), before);
    assert_eq!(editor.doc.undo.len(), history);
    editor.tool_name_draft = "Join points".into();
    assert!(editor.create_selected_tool(), "{}", editor.status);
    assert!(!editor.create_tool_dialog);
    let tools = editor.doc.embedded_tools().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name.as_deref(), Some("Join points"));
    assert_eq!(tools[0].input_ids.as_ref().unwrap().len(), 2);
    assert_eq!(editor.doc.undo.len(), history + 1);
    let created = editor.doc.encode().unwrap();
    editor.doc.undo();
    assert_eq!(editor.doc.encode().unwrap(), before);
    editor.doc.redo();
    assert_eq!(editor.doc.encode().unwrap(), created);
    editor.selection_order = vec![a, b];
    editor.doc.selected = [a, b].into_iter().collect();
    assert!(editor.invoke_selected_tool(0), "{}", editor.status);
    let output = *editor.selection_order.last().unwrap();
    assert_eq!(editor.doc.objects[output - 1].kind, Kind::Segment(a, b));
    editor.doc.verify_roundtrip().unwrap();
}

fn path_editor() -> Editor {
    Editor::new(
        Document::from_bytes(
            include_bytes!(
                "../../../tests/fixtures/embedded_tool_path_circle_created_native_verified.gsp"
            ),
            None,
        )
        .unwrap(),
    )
}

#[test]
fn path_tool_picks_arc_stroke_and_polygon_edge_or_interior_over_point_overlays() {
    for case in 0..3 {
        let mut editor = path_editor();
        let a = editor.doc.add(Kind::Free(Point { x: 300., y: 280. }));
        let b = editor.doc.add(Kind::Free(Point { x: 420., y: 280. }));
        let c = editor.doc.add(Kind::Free(Point { x: 360., y: 180. }));
        let host = if case == 0 {
            editor.doc.add(Kind::ThreePointArc(a, c, b))
        } else {
            editor.doc.add(Kind::Polygon(vec![a, b, c]))
        };
        let at = match case {
            0 => editor.doc.point_on_arc(host, 0.5).unwrap(),
            1 => Point { x: 360., y: 280. },
            _ => Point { x: 360., y: 240. },
        };
        let overlay = editor.doc.add(Kind::Free(at));
        let history = editor.doc.undo.len();
        editor.begin_custom_tool(0);
        editor.click(&viewport(), viewport().screen(at), false);
        assert!(
            editor.status.starts_with("Applied custom tool"),
            "{}",
            editor.status
        );
        assert!(editor.custom_tool_path_start.is_none());
        let output = editor.selection_order[0];
        assert_ne!(output, overlay);
        if case == 0 {
            assert!(
                matches!(editor.doc.objects[output - 1].kind, Kind::PointOnArc(id, _) if id == host)
            );
        } else {
            assert!(
                matches!(editor.doc.objects[output - 1].kind, Kind::PointOnPolygon(id, _, _) if id == host)
            );
        }
        assert_eq!(editor.doc.undo.len(), history + 1);
        let before = editor.doc.position(output).unwrap();
        for vertex in [a, b, c] {
            let position = editor.doc.position(vertex).unwrap();
            assert!(editor.doc.move_point(
                vertex,
                Point {
                    x: position.x + 20.,
                    y: position.y + 30.
                }
            ));
        }
        let after = editor.doc.position(output).unwrap();
        assert!((after.x - before.x - 20.).abs() < 1e-8);
        assert!((after.y - before.y - 30.).abs() < 1e-8);
        editor.doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn path_tool_picks_an_existing_path_through_a_point_overlay() {
    let mut editor = path_editor();
    let a = editor.doc.add(Kind::Free(Point { x: 300., y: 280. }));
    let b = editor.doc.add(Kind::Free(Point { x: 420., y: 280. }));
    let host = editor.doc.add(Kind::Segment(a, b));
    let overlay = editor.doc.add(Kind::Free(Point { x: 360., y: 280. }));
    editor.begin_custom_tool(0);
    editor.click(
        &viewport(),
        viewport().screen(editor.doc.position(overlay).unwrap()),
        false,
    );
    let output = editor.selection_order[0];
    assert!(
        matches!(editor.doc.objects[output - 1].kind, Kind::PointOnSegment(id, _) if id == host)
    );
    assert!(editor.custom_tool_path_start.is_none());
}

#[test]
fn path_tool_two_click_segment_defers_creation_and_supports_history_repeat_and_cancel() {
    let mut editor = path_editor();
    let end = editor.doc.add(Kind::Free(Point { x: 500., y: 350. }));
    let initial = editor.doc.encode().unwrap();
    let count = editor.doc.objects.len();
    let history = editor.doc.undo.len();
    let view = viewport();
    editor.begin_custom_tool(0);
    editor.click(&view, view.screen(Point { x: 420., y: 380. }), false);
    assert!(editor.custom_tool_path_start.is_some());
    assert_eq!(editor.doc.objects.len(), count);
    assert_eq!(editor.doc.undo.len(), history);
    assert_eq!(editor.doc.encode().unwrap(), initial);
    editor.click(&view, view.screen(Point { x: 500., y: 350. }), false);
    assert!(
        editor.status.starts_with("Applied custom tool"),
        "{}",
        editor.status
    );
    let first = count + 1;
    let host = count + 2;
    let output = editor.selection_order[0];
    assert_eq!(
        editor.doc.objects[first - 1].kind,
        Kind::Free(Point { x: 420., y: 380. })
    );
    assert_eq!(editor.doc.objects[host - 1].kind, Kind::Segment(first, end));
    assert!(
        matches!(editor.doc.objects[output - 1].kind, Kind::PointOnSegment(id, _) if id == host)
    );
    assert_eq!(editor.doc.undo.len(), history + 2);
    let completed = editor.doc.encode().unwrap();
    editor.doc.undo();
    assert_eq!(editor.doc.objects.len(), count + 2);
    editor.doc.undo();
    assert_eq!(editor.doc.encode().unwrap(), initial);
    editor.doc.redo();
    editor.doc.redo();
    assert_eq!(editor.doc.encode().unwrap(), completed);
    let position = editor.doc.position(output).unwrap();
    assert!(editor.doc.move_point(end, Point { x: 520., y: 370. }));
    assert_ne!(editor.doc.position(output).unwrap(), position);
    let moved = editor.doc.encode().unwrap();
    let count = editor.doc.objects.len();
    editor.click(&view, view.screen(Point { x: 300., y: 400. }), false);
    assert!(editor.custom_tool_path_start.is_some());
    assert_eq!(editor.doc.objects.len(), count);
    editor.cancel_custom_tool();
    assert!(editor.custom_tool_path_start.is_none());
    assert_eq!(editor.doc.encode().unwrap(), moved);
    editor.begin_custom_tool(0);
    editor.click(&view, view.screen(Point { x: 100., y: 380. }), false);
    assert_eq!(editor.doc.objects.len(), count);
    editor.click(&view, view.screen(Point { x: 200., y: 380. }), false);
    assert!(
        editor.status.starts_with("Applied custom tool"),
        "{}",
        editor.status
    );
    assert_eq!(
        editor.doc.objects[count + 2].kind,
        Kind::Segment(count + 1, count + 2)
    );
    assert_eq!(editor.doc.objects.len(), count + 4);
    assert!(editor.custom_tool_path_start.is_none());
    editor.doc.verify_roundtrip().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_TOOL_PATH_UI_FIXTURE").is_some() {
        std::fs::write(
            "/tmp/embedded_tool_path_two_click_editor_port.gsp",
            editor.doc.encode().unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn path_tool_selected_host_and_same_endpoint_failure_preserve_order_and_state() {
    let mut editor = path_editor();
    let a = editor.doc.add(Kind::Free(Point { x: 300., y: 350. }));
    let b = editor.doc.add(Kind::Free(Point { x: 400., y: 350. }));
    let host = editor.doc.add(Kind::Segment(a, b));
    editor.select_only(host);
    assert!(editor.invoke_selected_tool(0), "{}", editor.status);
    assert!(
        matches!(editor.doc.objects[editor.selection_order[0] - 1].kind, Kind::PointOnSegment(id, _) if id == host)
    );
    let isolated = editor.doc.add(Kind::Free(Point { x: 500., y: 400. }));
    editor.begin_custom_tool(0);
    let at = viewport().screen(editor.doc.position(isolated).unwrap());
    editor.click(&viewport(), at, false);
    let before = editor.doc.encode().unwrap();
    let history = editor.doc.undo.len();
    editor.click(&viewport(), at, false);
    assert!(editor.status.contains("different second endpoint"));
    assert_eq!(editor.doc.encode().unwrap(), before);
    assert_eq!(editor.doc.undo.len(), history);
    assert!(editor.custom_tool_path_start.is_some());
    editor.cancel_custom_tool();
    assert!(editor.custom_tool_path_start.is_none());
}

#[test]
fn native_verified_editor_path_segments_keep_ordered_inputs_and_live_output_placement() {
    let bytes = include_bytes!(
        "../../../tests/fixtures/embedded_tool_path_two_click_editor_native_verified.gsp"
    );
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.objects.len(), 12);
    assert_eq!(doc.objects[6].kind, Kind::Segment(6, 5));
    assert_eq!(doc.objects[10].kind, Kind::Segment(9, 10));
    for (point, host, first, second) in [(8, 7, 6, 5), (12, 11, 9, 10)] {
        let Kind::PointOnSegment(bound, fraction) = doc.objects[point - 1].kind else {
            panic!("Expected a point on the constructed input segment");
        };
        assert_eq!(bound, host);
        assert!((0.0..1.0).contains(&fraction));
        let a = doc.position(first).unwrap();
        let b = doc.position(second).unwrap();
        assert_eq!(
            doc.position(point),
            Some(Point {
                x: a.x + (b.x - a.x) * fraction,
                y: a.y + (b.y - a.y) * fraction
            })
        );
    }
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(doc.embedded_tools().unwrap().len(), 1);
    assert!(doc.move_point(5, Point { x: 540., y: 410. }));
    let Kind::PointOnSegment(_, fraction) = doc.objects[7].kind else {
        unreachable!()
    };
    assert_eq!(
        doc.position(8),
        Some(Point {
            x: 420. + 120. * fraction,
            y: 380. + 30. * fraction
        })
    );
    doc.verify_roundtrip().unwrap();
}

#[test]
fn finite_segment_given_picks_only_available_segment_and_preserves_invalid_click_state() {
    let mut editor = Editor::new(Document::from_bytes(include_bytes!(
        "../../../tests/fixtures/embedded_tool_segment_authored_midpoint_writer_native_verified.gsp"
    ), None).unwrap());
    let a = editor.doc.add(Kind::Free(Point { x: 100., y: 350. }));
    let b = editor.doc.add(Kind::Free(Point { x: 200., y: 350. }));
    let ray = editor.doc.add(Kind::Ray(a, b));
    let start = editor.doc.add(Kind::Free(Point { x: 400., y: 350. }));
    let end = editor.doc.add(Kind::Free(Point { x: 500., y: 350. }));
    let segment = editor.doc.add(Kind::Segment(start, end));
    editor.begin_custom_tool(0);
    assert!(editor.tool == Tool::Custom(0));
    let before = editor.doc.encode().unwrap();
    let history = editor.doc.undo.len();
    editor.click(
        &viewport(),
        viewport().screen(Point { x: 150., y: 350. }),
        false,
    );
    assert!(editor.status.contains("existing finite segment"));
    assert!(editor.custom_tool_path_start.is_none());
    assert_eq!(editor.doc.encode().unwrap(), before);
    assert_eq!(editor.doc.undo.len(), history);
    editor.click(
        &viewport(),
        viewport().screen(Point { x: 450., y: 350. }),
        false,
    );
    assert!(
        editor.status.starts_with("Applied custom tool"),
        "{}",
        editor.status
    );
    let result = editor.selection_order[0];
    assert_eq!(
        editor.doc.position(result),
        Some(Point { x: 450., y: 350. })
    );
    assert_eq!(editor.doc.objects[result - 1].midpoint_host, Some(segment));
    editor.select_only(ray);
    let before = editor.doc.encode().unwrap();
    assert!(!editor.invoke_selected_tool(0));
    assert_eq!(editor.doc.encode().unwrap(), before);
    editor.doc.verify_roundtrip().unwrap();
}

#[test]
fn numeric_given_clicks_bind_existing_parameters_calculations_and_measurements_atomically() {
    let scalar =
        include_bytes!("../../../tests/fixtures/embedded_tool_scalar_created_native_verified.gsp")
            .as_slice();
    let degree = include_bytes!(
        "../../../tests/fixtures/embedded_tool_numeric_regular_degree_native_verified.gsp"
    )
    .as_slice();
    let measured = include_bytes!(
        "../../../tests/fixtures/embedded_tool_numeric_regular_measure_native_verified.gsp"
    )
    .as_slice();
    for (bytes, input) in [(scalar, 1), (scalar, 2), (degree, 3), (measured, 6)] {
        let mut editor = Editor::new(Document::from_bytes(bytes, None).unwrap());
        let value = editor.doc.numeric_value(input).unwrap();
        editor.begin_custom_tool(0);
        assert!(editor.tool == Tool::Custom(0), "{}", editor.status);
        let before = editor.doc.encode().unwrap();
        let history = editor.doc.undo.len();
        let count = editor.doc.objects.len();
        editor.click(
            &viewport(),
            viewport().screen(Point { x: 500., y: 400. }),
            false,
        );
        assert!(editor.status.contains("available numeric caption"));
        assert_eq!(editor.doc.encode().unwrap(), before);
        assert_eq!(editor.doc.undo.len(), history);
        assert_eq!(editor.doc.objects.len(), count);
        let at =
            sketchpad_rs::render::number_screen_rect_for_object(&viewport(), &editor.doc, input)
                .or_else(|| {
                    editor
                        .overlay_rows()
                        .iter()
                        .enumerate()
                        .find(|(_, (id, _))| *id == input)
                        .map(|(row, (id, text))| {
                            editor.overlay_row_rect(&viewport(), row, *id, text)
                        })
                })
                .expect("Visible numeric caption")
                .center();
        editor.click(&viewport(), at, false);
        assert!(
            editor.status.starts_with("Applied custom tool"),
            "{}",
            editor.status
        );
        let output = editor.selection_order[0];
        assert!(editor.doc.dependencies_of(output).unwrap().contains(&input));
        assert!((editor.doc.numeric_value(output).unwrap() - value - 1.).abs() < 1e-10);
        assert_eq!(
            editor.doc.objects[output - 1].kind.numeric_unit(),
            Some(sketchpad_rs::table_records::NumericUnit::Scalar)
        );
        assert_eq!(editor.doc.undo.len(), history + 1);
        if input == 1 {
            editor.doc.set_parameter_value(input, value + 2.).unwrap();
            assert_eq!(editor.doc.numeric_value(output), Some(value + 3.));
        }
        editor.doc.verify_roundtrip().unwrap();
    }
    let mut editor = Editor::new(Document::from_bytes(scalar, None).unwrap());
    editor.select_only(1);
    assert!(editor.invoke_selected_tool(0), "{}", editor.status);
    assert_eq!(
        editor.doc.numeric_value(editor.selection_order[0]),
        Some(3.)
    );
}
