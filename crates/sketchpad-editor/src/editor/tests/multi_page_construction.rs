use super::*;

#[test]
fn point_tool_adds_free_and_constrained_points_on_multi_page_sketch() {
    let document = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/two_page_segment_constrained_native.gsp"),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(document);
    editor.doc.active_page = 1;
    editor.tool = Tool::Point;
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(600., 400.)),
        scale: 1.,
        center: Point { x: 250., y: 150. },
    };
    editor.click(
        &viewport,
        viewport.screen(Point { x: 246., y: 111. }),
        false,
    );
    assert!(
        matches!(editor.doc.objects[5].kind, Kind::PointOnSegment(4, t) if (t - 0.75).abs() < 1e-12)
    );
    editor.click(
        &viewport,
        viewport.screen(Point { x: 260., y: 220. }),
        false,
    );
    assert!(
        matches!(editor.doc.objects[6].kind, Kind::Free(Point { x, y }) if (x - 260.).abs() < 1e-8 && (y - 220.).abs() < 1e-8)
    );
    let saved = editor.doc.encode().unwrap();
    assert_eq!(Document::from_bytes(&saved, None).unwrap().objects.len(), 7);
}

#[test]
fn point_tool_and_construct_command_use_ray_and_line_hosts() {
    for (bytes, click, expected) in [
        (
            include_bytes!("../../../../../tests/fixtures/two_page_line_native.gsp").as_slice(),
            Point { x: -20., y: 111. },
            Kind::PointOnLine(5, -131. / 180.),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/two_page_ray_native.gsp").as_slice(),
            Point { x: 400., y: 111. },
            Kind::PointOnRay(5, 289. / 180.),
        ),
    ] {
        let mut editor = Editor::new(Document::from_bytes(bytes, None).unwrap());
        editor.doc.active_page = 1;
        editor.tool = Tool::Point;
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(600., 400.)),
            scale: 1.,
            center: Point { x: 250., y: 150. },
        };
        editor.click(&viewport, viewport.screen(click), false);
        assert_eq!(editor.doc.objects[5].kind, expected);
        editor.selection_order = vec![5];
        editor.doc.selected = [5].into();
        editor.point_on_object();
        assert_eq!(editor.doc.objects.len(), 7);
        let saved = editor.doc.encode().unwrap();
        assert_eq!(Document::from_bytes(&saved, None).unwrap().objects.len(), 7);
    }
}

#[test]
fn segment_and_circle_tools_construct_on_multi_page_sketch() {
    let document = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/two_page_blank_native.gsp"),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(document);
    editor.doc.active_page = 1;
    editor.tool = Tool::Segment;
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(600., 400.)),
        scale: 1.,
        center: Point { x: 250., y: 150. },
    };
    editor.click(
        &viewport,
        viewport.screen(Point { x: 111., y: 111. }),
        false,
    );
    editor.click(
        &viewport,
        viewport.screen(Point { x: 291., y: 111. }),
        false,
    );
    assert_eq!(editor.doc.objects[3].kind, Kind::Segment(2, 3));
    assert_eq!(
        editor.doc.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/two_page_segment_port.gsp")
    );
    editor.tool = Tool::Circle;
    editor.click(
        &viewport,
        viewport.screen(Point { x: 111., y: 111. }),
        false,
    );
    editor.click(
        &viewport,
        viewport.screen(Point { x: 291., y: 111. }),
        false,
    );
    assert_eq!(editor.doc.objects[4].kind, Kind::Circle(2, 3));
    let saved = editor.doc.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert_eq!(reopened.objects[4].kind, Kind::Circle(2, 3));
    editor.selection_order = vec![2, 3];
    editor.pair(1);
    assert_eq!(editor.doc.objects[5].kind, Kind::Circle(2, 3));
    editor.selection_order = vec![2, 3];
    editor.pair(3);
    assert_eq!(editor.doc.objects[6].kind, Kind::Line(2, 3));
    editor.selection_order = vec![2, 3];
    editor.pair(4);
    assert_eq!(editor.doc.objects[7].kind, Kind::Ray(2, 3));
    editor.selection_order = vec![2, 3];
    editor.pair(2);
    assert_eq!(editor.doc.objects[8].kind, Kind::Midpoint(2, 3));
    editor.selection_order = vec![2, 4];
    editor.circle_by_segment();
    assert_eq!(editor.doc.objects[9].kind, Kind::CircleBySegment(2, 4));
    assert!(Document::from_bytes(&editor.doc.encode().unwrap(), None).is_ok());
}

#[test]
fn derived_lines_intersection_and_point_on_object_work_on_multi_page_sketch() {
    let document = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/two_page_segment_native.gsp"),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(document);
    editor.doc.active_page = 1;
    let point = editor.doc.add(Kind::Free(Point { x: 201., y: 211. }));
    editor.selection_order = vec![point, 4];
    editor.doc.selected = editor.selection_order.iter().copied().collect();
    editor.derived_line(false);
    assert_eq!(editor.doc.objects[5].kind, Kind::Parallel(point, 4));
    editor.selection_order = vec![point, 4];
    editor.doc.selected = editor.selection_order.iter().copied().collect();
    editor.derived_line(true);
    assert_eq!(editor.doc.objects[6].kind, Kind::Perpendicular(point, 4));
    editor.selection_order = vec![4, 7];
    editor.doc.selected = editor.selection_order.iter().copied().collect();
    editor.intersection();
    assert_eq!(editor.doc.objects[7].kind, Kind::Intersection(4, 7));
    assert_eq!(editor.doc.position(8), Some(Point { x: 201., y: 111. }));
    editor.selection_order = vec![4];
    editor.doc.selected = editor.selection_order.iter().copied().collect();
    editor.point_on_object();
    assert!(
        matches!(editor.doc.objects[8].kind, Kind::PointOnSegment(4, t) if (t - 0.5).abs() < 1e-12)
    );
    let saved = editor.doc.encode().unwrap();
    assert_eq!(Document::from_bytes(&saved, None).unwrap().objects.len(), 9);
}

#[test]
fn bisector_and_arcs_work_on_multi_page_sketch() {
    let document = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/two_page_blank_native.gsp"),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(document);
    editor.doc.active_page = 1;
    let a = editor.doc.add(Kind::Free(Point { x: 160., y: 160. }));
    let b = editor.doc.add(Kind::Free(Point { x: 210., y: 100. }));
    let c = editor.doc.add(Kind::Free(Point { x: 260., y: 160. }));
    editor.selection_order = vec![a, b, c];
    editor.angle_bisector();
    assert_eq!(editor.doc.objects[4].kind, Kind::Bisector(a, b, c));
    editor.selection_order = vec![a, b, c];
    editor.three_point_arc();
    assert_eq!(editor.doc.objects[5].kind, Kind::ThreePointArc(a, b, c));
    assert!(Document::from_bytes(&editor.doc.encode().unwrap(), None).is_ok());

    let circle = editor.doc.add(Kind::Circle(a, c));
    let start = editor.doc.add(
        editor
            .doc
            .constrained_point(circle, Point { x: 260., y: 160. })
            .unwrap(),
    );
    let end = editor.doc.add(
        editor
            .doc
            .constrained_point(circle, Point { x: 160., y: 60. })
            .unwrap(),
    );
    editor.selection_order = vec![circle, start, end];
    editor.arc_on_circle();
    assert_eq!(
        editor.doc.objects[9].kind,
        Kind::ArcOnCircle(circle, start, end)
    );
    assert!(Document::from_bytes(&editor.doc.encode().unwrap(), None).is_ok());
}

#[test]
fn polygon_interior_works_on_multi_page_sketch() {
    let document = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/two_page_blank_native.gsp"),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(document);
    editor.doc.active_page = 1;
    let a = editor.doc.add(Kind::Free(Point { x: 120., y: 180. }));
    let b = editor.doc.add(Kind::Free(Point { x: 200., y: 80. }));
    let c = editor.doc.add(Kind::Free(Point { x: 300., y: 180. }));
    editor.selection_order = vec![a, b, c];
    editor.polygon_interior();
    assert_eq!(editor.doc.objects[4].kind, Kind::Polygon(vec![a, b, c]));
    assert_eq!(
        editor.doc.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/two_page_polygon_port.gsp")
    );
}

#[test]
fn text_tool_creates_editable_caption_on_supported_multi_page_sketch() {
    let mut editor = Editor::new(
        Document::from_bytes(
            include_bytes!("../../../../../tests/fixtures/two_page_text_port_saved_by_wine.gsp"),
            None,
        )
        .unwrap(),
    );
    assert!(editor.doc.partial_view);
    editor.doc.active_page = 1;
    editor.tool = Tool::Text;
    editor.text_draft = "New caption on page 2".into();
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(600., 400.)),
        scale: 1.,
        center: Point { x: 250., y: 150. },
    };
    editor.click(
        &viewport,
        viewport.screen(Point { x: 110., y: 240. }),
        false,
    );
    assert_eq!(editor.doc.objects.len(), 5);
    assert_eq!(editor.doc.objects[4].page, 1);
    assert!(matches!(&editor.doc.objects[4].kind,
        Kind::PlainText { text, .. } if text == "New caption on page 2"));
    assert_eq!(
        editor.doc.selected.iter().copied().collect::<Vec<_>>(),
        vec![5]
    );
    editor.doc.edit_text(5, "Edited caption on page 2").unwrap();
    editor.doc.verify_roundtrip().unwrap();
}
