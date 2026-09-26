use super::*;

#[test]
fn scalar_measurement_precision_reaches_desktop_and_svg_captions() {
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(480., 360.)),
        scale: 1.,
        center: Point { x: 200., y: 180. },
    };
    for (bytes, expected) in [
        (
            include_bytes!(
                "../../../tests/fixtures/scalar_precision_point_value_iteration_port.gsp"
            )
            .as_slice(),
            "Value of Point = 0.25000",
        ),
        (
            include_bytes!(
                "../../../tests/fixtures/scalar_precision_point_value_iteration_edited_port.gsp"
            )
            .as_slice(),
            "Value of Point = 1",
        ),
    ] {
        let editor = Editor::new(Document::from_bytes(bytes, None).unwrap());
        assert!(editor.overlay_rows().contains(&(5, expected.into())));
        let svg = sketchpad_rs::render::svg_page(&editor.doc, &viewport).unwrap();
        assert!(svg.contains(&format!(">{expected}</text>")));
    }
    let mut editor = Editor::new(
        Document::from_bytes(
            include_bytes!("../../../tests/fixtures/scalar_precision_coordinates_port.gsp"),
            None,
        )
        .unwrap(),
    );
    assert!(
        editor
            .overlay_rows()
            .iter()
            .any(|(id, text)| *id == 11 && text.ends_with(" = 1"))
    );
    assert!(
        editor
            .overlay_rows()
            .contains(&(14, "Slope = 0.0000".into()))
    );
    let svg = sketchpad_rs::render::svg_page(&editor.doc, &viewport).unwrap();
    assert!(svg.contains("Slope = 0.0000</text>"));
    editor.doc.set_measurement_precision(&[14], 0).unwrap();
    assert!(editor.overlay_rows().contains(&(14, "Slope = 0".into())));
    editor.doc.move_point(2, Point { x: 150., y: 175. });
    assert!(
        editor
            .overlay_rows()
            .contains(&(14, "Slope = undefined".into()))
    );
}

#[test]
fn measurement_precision_overlay_and_svg_share_saved_digits() {
    let document = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/measurement_precision_basic_port.gsp"),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(document);
    let rows = editor.overlay_rows();
    assert!(rows.contains(&(4, "Length = 1 cm".into())));
    assert!(rows.contains(&(7, "Radius = 1.32292 cm".into())));
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(480., 360.)),
        scale: 1.,
        center: Point { x: 200., y: 180. },
    };
    let svg = sketchpad_rs::render::svg_page(&editor.doc, &viewport).unwrap();
    assert!(svg.contains("Length = 1 cm</text>"));
    assert!(svg.contains("Radius = 1.32292 cm</text>"));
    editor.doc.set_measurement_precision(&[4, 7], 3).unwrap();
    assert!(
        editor
            .overlay_rows()
            .contains(&(4, "Length = 1.323 cm".into()))
    );
    assert!(
        editor
            .overlay_rows()
            .contains(&(7, "Radius = 1.323 cm".into()))
    );
    let svg = sketchpad_rs::render::svg_page(&editor.doc, &viewport).unwrap();
    assert!(svg.contains("Length = 1.323 cm</text>"));
    assert!(svg.contains("Radius = 1.323 cm</text>"));
}

#[test]
fn one_host_measurements_work_on_multi_page_sketch() {
    let document = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/two_page_blank_native.gsp"),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(document);
    editor.doc.active_page = 1;
    let a = editor.doc.add(Kind::Free(Point { x: 120., y: 180. }));
    let b = editor.doc.add(Kind::Free(Point { x: 200., y: 80. }));
    let c = editor.doc.add(Kind::Free(Point { x: 300., y: 180. }));
    let polygon = editor.doc.add(Kind::Polygon(vec![a, b, c]));
    let segment = editor.doc.add(Kind::Segment(a, c));
    let circle = editor.doc.add(Kind::Circle(a, c));
    let arc = editor.doc.add(Kind::ThreePointArc(a, b, c));

    editor.selection_order = vec![segment];
    editor.measure_length();
    editor.selection_order = vec![polygon];
    editor.measure_perimeter();
    editor.selection_order = vec![circle];
    editor.measure_circle(CircleMeasure::Circumference);
    editor.selection_order = vec![circle];
    editor.measure_circle(CircleMeasure::Radius);
    editor.selection_order = vec![circle];
    editor.measure_circle(CircleMeasure::Area);
    editor.selection_order = vec![polygon];
    editor.measure_circle(CircleMeasure::Area);
    editor.selection_order = vec![arc];
    editor.measure_arc_angle();
    editor.selection_order = vec![arc];
    editor.measure_arc_length();
    assert_eq!(
        editor.doc.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/two_page_one_host_measurements_port.gsp")
    );
}

#[test]
fn point_measurements_work_on_multi_page_sketch() {
    let document = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/two_page_one_host_measurements_port.gsp"),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(document);
    editor.doc.active_page = 1;
    let target = editor.doc.add(Kind::Free(Point { x: 210., y: 180. }));
    editor.selection_order = vec![2, 4];
    editor.measure_distance();
    editor.selection_order = vec![2, 3, 4];
    editor.measure_angle();
    editor.selection_order = vec![2, 4, target];
    editor.measure_ratio();
    assert_eq!(
        editor.doc.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/two_page_point_measurements_port.gsp")
    );
}

#[test]
fn copy_and_paste_segment_to_first_page() {
    let document = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/two_page_segment_native.gsp"),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(document);
    editor.doc.active_page = 1;
    editor.doc.selected.insert(4);
    assert!(editor.copy_selected());
    editor.doc.active_page = 0;
    editor.paste();
    assert_eq!(editor.selection_order, vec![5, 6, 7]);
    assert_eq!(editor.doc.encode().unwrap(), {
        let mut expected =
            include_bytes!("../../../tests/fixtures/two_page_pasted_segment_first_page_port.gsp")
                .to_vec();
        let record = sketchpad_rs::document::records(&expected)
            .unwrap()
            .into_iter()
            .find(|r| r.kind == 0x0384)
            .unwrap();
        expected[record.data.start + 16..record.data.start + 18]
            .copy_from_slice(&0_u16.to_le_bytes());
        expected
    });
}

#[test]
fn clear_and_cut_work_on_multi_page_sketch() {
    let base = include_bytes!("../../../tests/fixtures/two_page_segment_native.gsp");
    let mut editor = Editor::new(Document::from_bytes(base, None).unwrap());
    editor.doc.active_page = 1;
    editor.doc.selected.insert(2);
    editor.clear_selected();
    assert_eq!(
        editor.doc.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/two_page_point_and_segment_cleared_port.gsp")
    );

    let mut editor = Editor::new(Document::from_bytes(base, None).unwrap());
    editor.doc.active_page = 1;
    editor.doc.selected.insert(4);
    editor.cut_selected();
    editor.paste();
    assert_eq!(
        editor.doc.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/two_page_cut_pasted_segment_port.gsp")
    );
}

#[test]
fn imported_measurement_caption_position_and_font_reach_picking_and_svg() {
    let native = include_bytes!("../../../tests/fixtures/measurement_copy_native.gsp");
    let mut editor = Editor::new(Document::from_bytes(native, None).unwrap());
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::pos2(40., 60.), egui::vec2(480., 360.)),
        scale: 1.,
        center: Point { x: 200., y: 180. },
    };
    let caption = editor
        .overlay_rows()
        .into_iter()
        .find(|(id, _)| *id == 4)
        .unwrap()
        .1;
    let bounds = editor.overlay_row_rect(&viewport, 0, 4, &caption);
    assert_eq!(bounds.min, egui::pos2(52., 72.));
    assert_eq!(bounds.height(), 21.);
    assert_eq!(editor.hit(&viewport, bounds.center()), Some(4));
    editor.doc.drag_measurement_caption(4, (100, 110)).unwrap();
    let svg = sketchpad_rs::render::svg_page(&editor.doc, &viewport).unwrap();
    let caption_line = svg
        .lines()
        .find(|line| line.contains(&format!(">{caption}</text>")))
        .unwrap();
    assert!(caption_line.contains("x=\"142\""));
    assert!(caption_line.contains("font-size=\"18\""));
    editor.doc.verify_roundtrip().unwrap();
}
