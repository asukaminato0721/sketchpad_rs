use super::*;

#[test]
#[ignore = "Set SKETCHPAD_MATH_SAMPLE to the junior mathematics teaching sample"]
fn external_bisector_animation_updates_table_values() {
    let bytes = std::fs::read(std::env::var("SKETCHPAD_MATH_SAMPLE").unwrap()).unwrap();
    let mut doc = Document::from_bytes(&bytes, None).unwrap();
    doc.active_page = 9;
    let mut editor = Editor::new(doc);
    let before =
        crate::render::number_lines_for_object(&editor.evaluated_document(), 1496).unwrap();
    let initial = editor.doc.numeric_value(1488).unwrap();
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1100., 750.)),
        scale: 1.,
        center: editor.center,
    };
    let Kind::AnimationButton { anchor, label, .. } = &editor.doc.objects[1494].kind else {
        panic!("Expected animation button")
    };
    let click = crate::render::action_button_rect(&viewport, *anchor, label).center();
    editor.click(&viewport, click, false);
    assert_eq!(editor.animations.len(), 1);
    for _ in 0..30 {
        editor.tick_animation(0.1);
    }
    assert!((editor.doc.numeric_value(1488).unwrap() - initial).abs() > 0.01);
    let evaluated = editor.evaluated_document();
    let after = crate::render::number_lines_for_object(&evaluated, 1496).unwrap();
    assert_ne!(
        before.1, after.1,
        "The table must follow the animated point"
    );
    for id in [1488, 1489, 1473, 1474] {
        assert!(
            after
                .1
                .last()
                .unwrap()
                .contains(&evaluated.numeric_value_text(id).unwrap())
        );
    }
    let svg = crate::render::svg_page(&evaluated, &viewport).unwrap();
    assert!(svg.contains(after.1.last().unwrap()));
    editor.click(&viewport, click, false);
    assert!(editor.animations.is_empty());
    editor.tick_animation(0.5);
    assert_eq!(
        crate::render::number_lines_for_object(&editor.evaluated_document(), 1496).unwrap(),
        after
    );
    let reopened = Document::from_bytes(&editor.doc.encode().unwrap(), None).unwrap();
    assert_eq!(
        crate::render::number_lines_for_object(&reopened, 1496).unwrap(),
        after
    );

    // Dragging the independent point P must also update the PA and PB columns.
    assert!(editor.doc.move_point(1469, Point { x: 450., y: 120. }));
    let moved = editor.evaluated_document();
    let lines = crate::render::number_lines_for_object(&moved, 1496)
        .unwrap()
        .1;
    assert_ne!(lines, after.1);
    for id in [1473, 1474] {
        assert!(
            lines
                .last()
                .unwrap()
                .contains(&moved.numeric_value_text(id).unwrap())
        );
    }
}

#[test]
#[ignore = "Set SKETCHPAD_MATH_SAMPLE to the junior mathematics teaching sample"]
fn external_math_measurements_follow_dragged_points() {
    let bytes = std::fs::read(std::env::var("SKETCHPAD_MATH_SAMPLE").unwrap()).unwrap();
    let mut doc = Document::from_bytes(&bytes, None).unwrap();
    doc.active_page = 16;
    let mut editor = Editor::new(doc);
    let before = editor.evaluated_document();
    let angle_before = before.numeric_value(2524).unwrap();
    let _ = before.numeric_value(2521);
    let _ = before.numeric_value(2525);
    let p = editor.doc.position(2517).unwrap();
    for dx in [100., -100., 50.] {
        assert!(editor.doc.move_point(
            2517,
            Point {
                x: p.x + dx,
                y: p.y + 50.
            }
        ));
        let expected = editor.doc.numeric_value(2524).unwrap();
        assert!((expected - angle_before).abs() > 1.);
        let after = editor.evaluated_document();
        for id in [2524, 2521, 2525] {
            assert_eq!(
                after.numeric_value(id),
                editor.doc.numeric_value(id),
                "measurement {id}"
            );
            assert_eq!(
                after.measurement_text(id),
                editor.doc.measurement_text(id),
                "caption {id}"
            );
        }
    }
}

#[test]
#[ignore = "Set SKETCHPAD_MATH_SAMPLE to the junior mathematics teaching sample"]
fn external_math_measurement_captions_match_native() {
    let bytes = std::fs::read(std::env::var("SKETCHPAD_MATH_SAMPLE").unwrap()).unwrap();
    let mut doc = Document::from_bytes(&bytes, None).unwrap();
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1100., 750.)),
        scale: 1.,
        center: Point { x: 450., y: 320. },
    };
    for (page, captions) in [
        (
            16,
            vec![(2524, "∠ABC = 68.2°"), (2521, "弧a的度数 = 136.4°")],
        ),
        (
            29,
            vec![
                (4576, "重叠部分面积 = 0.60 cm²"),
                (4578, "道路总面积 = 9.03 cm²"),
            ],
        ),
        (21, vec![(3322, "PA = 8.58 cm"), (3323, "PB = 3.63 cm")]),
    ] {
        doc.active_page = page;
        let rows = Editor::overlay_rows_for(&doc);
        let svg = crate::render::svg_page(&doc, &viewport).unwrap();
        for (id, caption) in captions {
            assert!(rows.contains(&(id, caption.into())), "{rows:?}");
            assert!(svg.contains(&format!(">{caption}</text>")));
        }
    }
    let (_, lines) = crate::render::number_lines_for_object(&doc, 2525).unwrap();
    assert_eq!(lines, vec!["(∠ABC ÷ 弧a的度数) = 0.50"]);
    let (_, lines) = crate::render::number_lines_for_object(&doc, 3324).unwrap();
    assert_eq!(lines, vec!["(PA ÷ PB) = 2.36"]);
    let mut named = 0;
    for object in doc.objects.iter() {
        if object.kind.is_measurement()
            && let Some(label) = &object.label
            && let Ok(name) = label.name_utf8()
            && !name.is_empty()
            && let Some(caption) = doc.measurement_text(object.id)
        {
            assert!(
                caption.starts_with(&format!("{name} = ")),
                "{}: {caption}",
                object.id
            );
            named += 1;
        }
    }
    assert_eq!(named, 80);
    doc.verify_roundtrip().unwrap();
}

#[test]
fn named_scalar_captions_reach_desktop_svg_and_undefined_slope() {
    use sketchpad_engine::text_records::LabelAux;
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/scalar_precision_coordinates_port.gsp"),
        None,
    )
    .unwrap();
    let viewport = Viewport {
        rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(480., 360.)),
        scale: 1.,
        center: Point { x: 200., y: 180. },
    };
    for (id, name, value) in [
        (11, "横坐标", "1"),
        (13, "距离", "1.30319"),
        (14, "斜率", "0.0000"),
    ] {
        doc.objects.edit(id - 1).label = Some(LabelAux::new_point(name, false, (0, 0)).unwrap());
        let caption = format!("{name} = {value}");
        assert!(Editor::overlay_rows_for(&doc).contains(&(id, caption.clone())));
        assert!(
            crate::render::svg_page(&doc, &viewport)
                .unwrap()
                .contains(&format!(">{caption}</text>"))
        );
    }
    doc.move_point(2, Point { x: 150., y: 175. });
    let rows = Editor::overlay_rows_for(&doc);
    assert_eq!(rows.iter().filter(|(id, _)| *id == 14).count(), 1);
    assert!(rows.contains(&(14, "斜率 = undefined".into())));
}

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
                "../../../../../tests/fixtures/scalar_precision_point_value_iteration_port.gsp"
            )
            .as_slice(),
            "Value of Point = 0.25000",
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/scalar_precision_point_value_iteration_edited_port.gsp"
            )
            .as_slice(),
            "Value of Point = 1",
        ),
    ] {
        let editor = Editor::new(Document::from_bytes(bytes, None).unwrap());
        assert!(editor.overlay_rows().contains(&(5, expected.into())));
        let svg = crate::render::svg_page(&editor.doc, &viewport).unwrap();
        assert!(svg.contains(&format!(">{expected}</text>")));
    }
    let mut editor = Editor::new(
        Document::from_bytes(
            include_bytes!("../../../../../tests/fixtures/scalar_precision_coordinates_port.gsp"),
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
    let svg = crate::render::svg_page(&editor.doc, &viewport).unwrap();
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
        include_bytes!("../../../../../tests/fixtures/measurement_precision_basic_port.gsp"),
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
    let svg = crate::render::svg_page(&editor.doc, &viewport).unwrap();
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
    let svg = crate::render::svg_page(&editor.doc, &viewport).unwrap();
    assert!(svg.contains("Length = 1.323 cm</text>"));
    assert!(svg.contains("Radius = 1.323 cm</text>"));
}

#[test]
fn one_host_measurements_work_on_multi_page_sketch() {
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
    let polygon = editor.doc.add(Kind::Polygon(vec![a, b, c]));
    let segment = editor.doc.add(Kind::Segment(a, c));
    let circle = editor.doc.add(Kind::Circle(a, c));
    let arc = editor.doc.add(Kind::ThreePointArc(a, b, c));

    editor.selection_order = vec![segment];
    editor.measure_length();
    editor.selection_order = vec![polygon];
    editor.measure_perimeter();
    editor.selection_order = vec![circle];
    editor.measure_shape(ShapeMeasure::Circumference);
    editor.selection_order = vec![circle];
    editor.measure_shape(ShapeMeasure::Radius);
    editor.selection_order = vec![circle];
    editor.measure_shape(ShapeMeasure::Area);
    editor.selection_order = vec![polygon];
    editor.measure_shape(ShapeMeasure::Area);
    editor.selection_order = vec![arc];
    editor.measure_arc_angle();
    editor.selection_order = vec![arc];
    editor.measure_arc_length();
    assert_eq!(
        editor.doc.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/two_page_one_host_measurements_port.gsp")
    );
    editor.selection_order = vec![arc];
    editor.measure_shape(ShapeMeasure::Radius);
    let radius = *editor.doc.selected.iter().next().unwrap();
    assert_eq!(editor.doc.objects[radius - 1].kind, Kind::CircleRadius(arc));
    assert!(editor.doc.circle_radius_cm(radius).unwrap() > 0.);
    let reopened = Document::from_bytes(&editor.doc.encode().unwrap(), None).unwrap();
    assert!(reopened.circle_radius_cm(radius).unwrap() > 0.);

    let circular_segment = editor.doc.add(Kind::CircularSegmentBoundary(arc));
    editor.selection_order = vec![circular_segment];
    editor.measure_perimeter();
    let perimeter = *editor.doc.selected.iter().next().unwrap();
    editor.selection_order = vec![circular_segment];
    editor.measure_shape(ShapeMeasure::Area);
    let area = *editor.doc.selected.iter().next().unwrap();
    let reopened = Document::from_bytes(&editor.doc.encode().unwrap(), None).unwrap();
    assert!(reopened.perimeter_cm(perimeter).unwrap() > 0.);
    assert!(reopened.polygon_area_cm2(area).unwrap() > 0.);

    editor.selection_order = vec![circle, b, c];
    editor.measure_arc_length();
    let circle_arc_length = *editor.doc.selected.iter().next().unwrap();
    assert_eq!(
        editor.doc.objects[circle_arc_length - 1].kind,
        Kind::CircleArcLength(circle, b, c)
    );
    let reopened = Document::from_bytes(&editor.doc.encode().unwrap(), None).unwrap();
    assert!(reopened.arc_length_cm(circle_arc_length).unwrap() > 0.);
}

#[test]
fn point_measurements_work_on_multi_page_sketch() {
    let document = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/two_page_one_host_measurements_port.gsp"),
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
        include_bytes!("../../../../../tests/fixtures/two_page_point_measurements_port.gsp")
    );
    let first = editor.doc.add(Kind::Segment(2, 4));
    let second = editor.doc.add(Kind::Segment(2, target));
    editor.selection_order = vec![first, second];
    editor.measure_ratio();
    let ratio = *editor.doc.selected.iter().next().unwrap();
    assert_eq!(
        editor.doc.objects[ratio - 1].kind,
        Kind::SegmentRatio(first, second)
    );
    let reopened = Document::from_bytes(&editor.doc.encode().unwrap(), None).unwrap();
    assert_eq!(
        reopened.objects[ratio - 1].kind,
        Kind::SegmentRatio(first, second)
    );
}

#[test]
fn copy_and_paste_segment_to_first_page() {
    let document = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/two_page_segment_native.gsp"),
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
        let mut expected = include_bytes!(
            "../../../../../tests/fixtures/two_page_pasted_segment_first_page_port.gsp"
        )
        .to_vec();
        let record = sketchpad_engine::document::records(&expected)
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
    let base = include_bytes!("../../../../../tests/fixtures/two_page_segment_native.gsp");
    let mut editor = Editor::new(Document::from_bytes(base, None).unwrap());
    editor.doc.active_page = 1;
    editor.doc.selected.insert(2);
    editor.clear_selected();
    assert_eq!(
        editor.doc.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/two_page_point_and_segment_cleared_port.gsp")
    );

    let mut editor = Editor::new(Document::from_bytes(base, None).unwrap());
    editor.doc.active_page = 1;
    editor.doc.selected.insert(4);
    editor.cut_selected();
    editor.paste();
    assert_eq!(
        editor.doc.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/two_page_cut_pasted_segment_port.gsp")
    );
}

#[test]
fn imported_measurement_caption_position_and_font_reach_picking_and_svg() {
    let native = include_bytes!("../../../../../tests/fixtures/measurement_copy_native.gsp");
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
    let svg = crate::render::svg_page(&editor.doc, &viewport).unwrap();
    let caption_line = svg
        .lines()
        .find(|line| line.contains(&format!(">{caption}</text>")))
        .unwrap();
    assert!(caption_line.contains("x=\"142\""));
    assert!(caption_line.contains("font-size=\"18\""));
    editor.doc.verify_roundtrip().unwrap();
}
