use super::*;

#[test]
fn define_coordinate_system_uses_selected_distances_in_order() {
    let doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/graph_two_segments_base.gsp"),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(doc);
    editor.selection_order = vec![1, 3, 6];
    editor.doc.selected.extend([1, 3, 6]);
    assert_eq!(
        editor.define_coordinate_system_label(),
        "Define Unit Distances"
    );
    editor.define_coordinate_system();
    assert_eq!(editor.doc.active_coordinate_system(), Some(9));
    assert_eq!(editor.doc.objects[6].kind, Kind::DistanceAxis(1, 3, 1));
    assert_eq!(editor.doc.objects[7].kind, Kind::DistanceAxis(1, 6, 0));
    assert_eq!(editor.doc.grid_form[0], GridForm::Rectangular);
}

#[test]
fn clicking_a_grid_intersection_selects_its_coordinate_system() {
    let doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/graph_two_coordinate_systems_original.gsp"),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(doc);
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: Point { x: 199., y: 125. },
    };
    for (frame_id, x, y) in [(9, 2., 2.), (14, 2., 3.)] {
        let at = editor
            .doc
            .graph_frame(frame_id)
            .unwrap()
            .world(x, y)
            .unwrap();
        let click = viewport.screen(Point { x: at.x, y: at.y });
        assert_eq!(editor.hit(&viewport, click), Some(frame_id));
    }
    editor.select_only(9);
    editor.mark_coordinate_system();
    assert_eq!(editor.doc.active_coordinate_system(), Some(9));
    editor.doc.mark_coordinate_system(1).unwrap();
    editor.doc.set_grid_visible(false);
    let at = editor.doc.graph_frame(14).unwrap().world(2., 3.).unwrap();
    assert_ne!(
        editor.hit_grid(&viewport, viewport.screen(Point { x: at.x, y: at.y })),
        Some(14)
    );

    let mixed = Editor::new(
        Document::from_bytes(
            include_bytes!("../../../tests/fixtures/graph_two_frames_mixed_grid_generated.gsp"),
            None,
        )
        .unwrap(),
    );
    let radius = 3. / std::f64::consts::SQRT_2;
    let at = mixed
        .doc
        .graph_frame(14)
        .unwrap()
        .world(radius, radius)
        .unwrap();
    assert_eq!(
        mixed.hit_grid(&viewport, viewport.screen(Point { x: at.x, y: at.y })),
        Some(14)
    );
}

#[test]
fn clicking_function_label_selects_definition_for_plotting() {
    let doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/function_definition_identity.gsp"),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(doc);
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.)),
        scale: 1.,
        center: Point { x: 199., y: 125. },
    };
    let label = viewport.rect.min + egui::vec2(20., 20.);
    editor.click(&viewport, label, false);
    assert_eq!(
        editor.doc.selected.iter().copied().collect::<Vec<_>>(),
        vec![7]
    );
    let plot = editor
        .doc
        .plot_function(7, Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(plot, 8);
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );

    let mut mixed = Document::new();
    let a = mixed.add(Kind::Free(Point { x: 100., y: 100. }));
    let b = mixed.add(Kind::Free(Point { x: 150., y: 100. }));
    let segment = mixed.add(Kind::Segment(a, b));
    mixed.add(Kind::SegmentLength(segment));
    let definition = mixed
        .add_function_definition(SimpleFunction::Square)
        .unwrap();
    let mut editor = Editor::new(mixed);
    assert_eq!(editor.overlay_rows().len(), 2);
    editor.click(&viewport, viewport.rect.min + egui::vec2(20., 40.), false);
    assert_eq!(
        editor.doc.selected.iter().copied().collect::<Vec<_>>(),
        vec![definition]
    );
}

#[test]
fn traced_point_and_segment_capture_motion_until_erased() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 20., y: 30. }));
    let b = editor.doc.add(Kind::Free(Point { x: 80., y: 30. }));
    let segment = editor.doc.add(Kind::Segment(a, b));
    editor.doc.selected.extend([a, segment]);
    assert_eq!(editor.doc.set_selected_tracing(true).unwrap(), 2);
    editor.capture_traces();
    assert_eq!(editor.trace_marks.len(), 2);
    assert!(editor.doc.move_point(a, Point { x: 35., y: 45. }));
    editor.capture_traces();
    assert_eq!(editor.trace_marks.len(), 4);
    assert!(matches!(
        editor.trace_marks[2],
        TraceMark::Point(Point { x: 35., y: 45. }, _)
    ));
    assert!(matches!(
        editor.trace_marks[3],
        TraceMark::Straight(Point { x: 35., y: 45. }, _, LineMode::Segment, _)
    ));
    editor.trace_marks.clear();
    assert!(editor.trace_marks.is_empty());
    editor.capture_traces();
    assert_eq!(editor.trace_marks.len(), 2);
    editor.trace_marks.clear();
    editor.doc.set_selected_tracing(false).unwrap();
    editor.capture_traces();
    assert!(editor.trace_marks.is_empty());
}

#[test]
fn traced_circle_arc_and_polygon_capture_geometry_changes() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 20., y: 30. }));
    let b = editor.doc.add(Kind::Free(Point { x: 80., y: 30. }));
    let c = editor.doc.add(Kind::Free(Point { x: 45., y: 70. }));
    let circle = editor.doc.add(Kind::Circle(a, b));
    let arc = editor.doc.add(Kind::ThreePointArc(a, c, b));
    let polygon = editor.doc.add(Kind::Polygon(vec![a, b, c]));
    editor.doc.selected.extend([circle, arc, polygon]);
    assert_eq!(editor.doc.set_selected_tracing(true).unwrap(), 3);
    editor.capture_traces();
    assert_eq!(editor.trace_marks.len(), 3);
    assert!(matches!(editor.trace_marks[0], TraceMark::Circle(_, _, _)));
    assert!(matches!(editor.trace_marks[1], TraceMark::Arc(_, _)));
    assert!(matches!(editor.trace_marks[2], TraceMark::Polygon(_, _)));
    assert!(editor.doc.move_point(b, Point { x: 90., y: 40. }));
    editor.capture_traces();
    assert_eq!(editor.trace_marks.len(), 6);
    match (&editor.trace_marks[0], &editor.trace_marks[3]) {
        (TraceMark::Circle(_, first, _), TraceMark::Circle(_, second, _)) => {
            assert_ne!(first, second);
        }
        _ => panic!("Expected two circle trace samples"),
    }
    match (&editor.trace_marks[1], &editor.trace_marks[4]) {
        (TraceMark::Arc(first, _), TraceMark::Arc(second, _)) => {
            assert_ne!(first, second);
        }
        _ => panic!("Expected two arc trace samples"),
    }
    match (&editor.trace_marks[2], &editor.trace_marks[5]) {
        (TraceMark::Polygon(first, _), TraceMark::Polygon(second, _)) => {
            assert_ne!(first, second);
        }
        _ => panic!("Expected two polygon trace samples"),
    }
}

#[test]
fn traced_ray_and_line_capture_modes_and_motion() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 20., y: 30. }));
    let b = editor.doc.add(Kind::Free(Point { x: 80., y: 30. }));
    let ray = editor.doc.add(Kind::Ray(a, b));
    let line = editor.doc.add(Kind::Line(a, b));
    editor.doc.selected.extend([ray, line]);
    assert_eq!(editor.doc.set_selected_tracing(true).unwrap(), 2);
    editor.capture_traces();
    assert_eq!(editor.trace_marks.len(), 2);
    assert!(matches!(
        editor.trace_marks[0],
        TraceMark::Straight(_, _, LineMode::Ray, _)
    ));
    assert!(matches!(
        editor.trace_marks[1],
        TraceMark::Straight(_, _, LineMode::Line, _)
    ));
    assert!(editor.doc.move_point(a, Point { x: 35., y: 45. }));
    editor.capture_traces();
    assert!(matches!(
        editor.trace_marks[2],
        TraceMark::Straight(Point { x: 35., y: 45. }, _, LineMode::Ray, _)
    ));
    assert!(matches!(
        editor.trace_marks[3],
        TraceMark::Straight(Point { x: 35., y: 45. }, _, LineMode::Line, _)
    ));
}

#[test]
fn native_linear_three_point_arc_is_pickable_on_its_segment() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/arc_three_point_collapsed_plot_native_verified.gsp"), None).unwrap();
    for object in &mut doc.objects { object.visible = object.id == 5; }
    let editor = Editor::new(doc);
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(300., 300.)),
        scale: 1., center: Point { x: 150., y: 150. },
    };
    assert_eq!(editor.hit(&viewport, viewport.screen(Point { x: 225., y: 180. })), Some(5));
    assert_eq!(editor.hit(&viewport, viewport.screen(Point { x: 190., y: 180. })), None);
    assert_eq!(editor.hit(&viewport, viewport.screen(Point { x: 225., y: 190. })), None);
}
