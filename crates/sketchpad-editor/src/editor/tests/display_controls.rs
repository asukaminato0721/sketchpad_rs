use super::*;

#[test]
fn text_palette_formats_selected_text_without_changing_document_visibility_state() {
    let mut editor = Editor::new(Document::new());
    assert!(!editor.show_text_palette);

    editor.show_text_palette = true;
    let id = editor
        .doc
        .add_text("Palette", Point { x: 40., y: 50. })
        .unwrap();
    editor.select_only(id);
    let initial_bold = editor.doc.text_font_state(id).unwrap().bold;
    editor.apply_selected_text_style(sketchpad_engine::document::TextStyleCommand::Bold);
    assert_ne!(editor.doc.text_font_state(id).unwrap().bold, initial_bold);

    editor.text_font_input = "Arial".into();
    editor.apply_selected_text_family();
    assert_eq!(editor.doc.text_font_state(id).unwrap().family, "Arial");
    assert!(editor.show_text_palette);

    editor.preferences.point_color = [1, 2, 3];
    editor.preferences.straight_color = [4, 5, 6];
    editor.preferences.round_color = [7, 8, 9];
    editor.preferences.interior_color = [10, 11, 12];
    editor.sync_document_preferences();
    let a = editor.doc.add(Kind::Free(Point { x: 10., y: 10. }));
    let b = editor.doc.add(Kind::Free(Point { x: 30., y: 10. }));
    let c = editor.doc.add(Kind::Free(Point { x: 20., y: 30. }));
    let segment = editor.doc.add(Kind::Segment(a, b));
    let circle = editor.doc.add(Kind::Circle(a, b));
    let polygon = editor.doc.add(Kind::Polygon(vec![a, b, c]));
    for (id, expected) in [
        (a, [1, 2, 3]),
        (segment, [4, 5, 6]),
        (circle, [7, 8, 9]),
        (polygon, [10, 11, 12]),
    ] {
        let color = match editor.doc.objects()[id - 1].visual_style.unwrap() {
            sketchpad_engine::style_records::VisualStyle::Point { color, .. }
            | sketchpad_engine::style_records::VisualStyle::Stroke { color, .. }
            | sketchpad_engine::style_records::VisualStyle::Polygon { color, .. } => color,
        };
        assert_eq!([color.red, color.green, color.blue], expected);
    }
    let reopened = Document::from_bytes(&editor.doc.encode().unwrap(), None).unwrap();
    for id in [a, segment, circle, polygon] {
        assert_eq!(
            reopened.objects()[id - 1].visual_style,
            editor.doc.objects()[id - 1].visual_style
        );
    }

    editor.doc.selected = [segment, circle].into_iter().collect();
    editor.apply_selected_color(
        sketchpad_engine::style_records::RgbColor {
            red: 21,
            green: 22,
            blue: 23,
        },
        true,
    );
    assert_eq!(editor.preferences.straight_color, [21, 22, 23]);
    assert_eq!(editor.preferences.round_color, [21, 22, 23]);
    let next_segment = editor.doc.add(Kind::Segment(b, c));
    let next_circle = editor.doc.add(Kind::Circle(b, c));
    for id in [next_segment, next_circle] {
        let color = match editor.doc.objects()[id - 1].visual_style.unwrap() {
            sketchpad_engine::style_records::VisualStyle::Stroke { color, .. } => color,
            _ => panic!("expected stroke style"),
        };
        assert_eq!([color.red, color.green, color.blue], [21, 22, 23]);
    }

    editor.select_only(a);
    editor.apply_selected_color(
        sketchpad_engine::style_records::RgbColor {
            red: 31,
            green: 32,
            blue: 33,
        },
        false,
    );
    assert_eq!(editor.preferences.point_color, [1, 2, 3]);
    editor.preferences.update_colors_automatically = false;
    editor.apply_selected_color(
        sketchpad_engine::style_records::RgbColor {
            red: 41,
            green: 42,
            blue: 43,
        },
        true,
    );
    assert_eq!(editor.preferences.point_color, [1, 2, 3]);
    let next_point = editor.doc.add(Kind::Free(Point { x: 50., y: 50. }));
    assert!(matches!(
        editor.doc.objects()[next_point - 1].visual_style,
        Some(sketchpad_engine::style_records::VisualStyle::Point {
            color: sketchpad_engine::style_records::RgbColor {
                red: 1,
                green: 2,
                blue: 3
            },
            ..
        })
    ));

    editor.preferences.fade_traces = true;
    editor.preferences.trace_fade_seconds = 0.5;
    let trace = crate::render::TraceMark::Point(
        Point { x: 0., y: 0. },
        sketchpad_engine::style_records::RgbColor {
            red: 1,
            green: 2,
            blue: 3,
        },
    );
    editor.trace_marks.push(crate::render::TraceSample {
        mark: trace.clone(),
        created_at: std::time::Instant::now() - std::time::Duration::from_secs(1),
    });
    editor
        .trace_marks
        .push(crate::render::TraceSample::new(trace));
    editor.prune_faded_traces();
    assert_eq!(editor.trace_marks.len(), 1);
    editor.preferences.fade_traces = false;
    editor.trace_marks[0].created_at =
        std::time::Instant::now() - std::time::Duration::from_secs(1);
    editor.prune_faded_traces();
    assert_eq!(editor.trace_marks.len(), 1);

    editor.preferences.randomize_interior_colors = true;
    editor.sync_document_preferences();
    let random_a = editor.doc.add(Kind::Polygon(vec![a, b, c]));
    let random_b = editor.doc.add(Kind::Polygon(vec![a, b, c]));
    let polygon_color = |id: usize| match editor.doc.objects()[id - 1].visual_style.unwrap() {
        sketchpad_engine::style_records::VisualStyle::Polygon { color, .. } => color,
        _ => panic!("expected polygon style"),
    };
    assert_ne!(polygon_color(random_a), polygon_color(random_b));

    editor.doc.selected = [segment, circle].into_iter().collect();
    editor.apply_selected_line_width(sketchpad_engine::style_records::LineWidth::Thick, true);
    editor.select_only(segment);
    editor.apply_selected_line_pattern(sketchpad_engine::style_records::LinePattern::Dashed, true);
    editor.select_only(a);
    editor.apply_selected_point_style(sketchpad_engine::style_records::PointStyle::Large, true);
    editor.apply_selected_point_style(sketchpad_engine::style_records::PointStyle::Dot, false);
    editor.preferences.angle_precision = 1;
    editor.preferences.distance_precision = 3;
    editor.preferences.other_precision = 5;
    editor.preferences.new_polygon_opacity = 49151;
    editor.sync_document_preferences();

    editor.replace_document(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 10., y: 10. }));
    let b = editor.doc.add(Kind::Free(Point { x: 30., y: 10. }));
    let segment = editor.doc.add(Kind::Segment(a, b));
    let circle = editor.doc.add(Kind::Circle(a, b));
    assert!(matches!(
        editor.doc.objects()[a - 1].visual_style,
        Some(sketchpad_engine::style_records::VisualStyle::Point {
            shape: sketchpad_engine::style_records::PointStyle::Large,
            ..
        })
    ));
    assert!(matches!(
        editor.doc.objects()[segment - 1].visual_style,
        Some(sketchpad_engine::style_records::VisualStyle::Stroke {
            line: sketchpad_engine::style_records::SegmentLineStyle {
                width: sketchpad_engine::style_records::LineWidth::Thick,
                pattern: sketchpad_engine::style_records::LinePattern::Dashed,
            },
            ..
        })
    ));
    assert!(matches!(
        editor.doc.objects()[circle - 1].visual_style,
        Some(sketchpad_engine::style_records::VisualStyle::Stroke {
            line: sketchpad_engine::style_records::SegmentLineStyle {
                width: sketchpad_engine::style_records::LineWidth::Thick,
                pattern: sketchpad_engine::style_records::LinePattern::Solid,
            },
            ..
        })
    ));

    let c = editor.doc.add(Kind::Free(Point { x: 20., y: 30. }));
    let length = editor.doc.add(Kind::SegmentLength(segment));
    let polygon = editor.doc.add(Kind::Polygon(vec![a, b, c]));
    let circle_interior = editor.doc.add_circle_interior(circle).unwrap();
    let area = editor.doc.add(Kind::PolygonArea(polygon));
    let angle = editor.doc.add(Kind::AngleMeasurement(a, b, c));
    let ratio = editor.doc.add(Kind::PointRatio(a, b, c));
    let calculation = editor
        .doc
        .add_calculation_expression("2+3", (10, 40))
        .unwrap();
    assert_eq!(
        editor.doc.objects()[length - 1].measurement_precision,
        Some(3)
    );
    assert_eq!(
        editor.doc.objects()[area - 1].measurement_precision,
        Some(3)
    );
    assert!(matches!(
        editor.doc.objects()[polygon - 1].visual_style,
        Some(sketchpad_engine::style_records::VisualStyle::Polygon { opacity: 49151, .. })
    ));
    assert!(matches!(
        editor.doc.objects()[circle_interior - 1].visual_style,
        Some(sketchpad_engine::style_records::VisualStyle::Polygon { opacity: 32767, .. })
    ));
    assert_eq!(
        editor.doc.objects()[angle - 1].measurement_precision,
        Some(1)
    );
    assert_eq!(
        editor.doc.objects()[ratio - 1].measurement_precision,
        Some(5)
    );
    let Kind::Calculation(group) = &editor.doc.objects()[calculation - 1].kind else {
        panic!("expected calculation");
    };
    assert_eq!(group.precision(), 5);

    editor.preferences.angle_precision = 4;
    editor.sync_document_preferences();
    let next_angle = editor.doc.add(Kind::AngleMeasurement(a, b, c));
    assert_eq!(
        editor.doc.objects()[angle - 1].measurement_precision,
        Some(1)
    );
    assert_eq!(
        editor.doc.objects()[next_angle - 1].measurement_precision,
        Some(4)
    );
}
