use super::*;

#[test]
fn equation_command_accepts_parallel_perpendicular_and_axis_lines() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let b = editor.doc.add(Kind::Free(Point { x: 180., y: 140. }));
    let c = editor.doc.add(Kind::Free(Point { x: 60., y: 180. }));
    let line = editor.doc.add(Kind::Line(a, b));
    for kind in [Kind::Parallel(c, line), Kind::Perpendicular(c, line)] {
        let host = editor.doc.add(kind);
        editor.selection_order = vec![host];
        editor.measure_value_equation(true);
        let id = editor.doc.objects().last().unwrap().id;
        assert!(
            matches!(editor.doc.objects()[id-1].kind, Kind::ValueEquation { kind: ValueEquationKind::LineEquation, first, .. } if first == host)
        );
        assert!(editor.doc.value_equation_value(id).is_some());
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
    let frame = editor.doc.active_coordinate_system().unwrap();
    let Kind::CoordinateSystem(x_axis, _) = editor.doc.objects()[frame - 1].kind else {
        panic!("Expected frame");
    };
    editor.selection_order = vec![x_axis];
    editor.measure_value_equation(true);
    assert!(
        editor
            .doc
            .value_equation_value(editor.doc.objects().last().unwrap().id)
            .is_some()
    );
    let bisector = editor.doc.add(Kind::Bisector(a, b, c));
    editor.selection_order = vec![bisector];
    let count = editor.doc.objects().len();
    editor.measure_value_equation(true);
    assert_eq!(editor.doc.objects().len(), count);
}

#[test]
fn equation_command_accepts_constructed_and_transformed_circles() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 160., y: 180. }));
    let b = editor.doc.add(Kind::Free(Point { x: 220., y: 180. }));
    let segment = editor.doc.add(Kind::Segment(a, b));
    let circle = editor.doc.add(Kind::Circle(a, b));
    for kind in [
        Kind::CircleBySegment(a, segment),
        Kind::RotatedCircle(circle, a, 30.),
        Kind::ScaledCircle(circle, a, -0.5),
    ] {
        let host = editor.doc.add(kind);
        editor.selection_order = vec![host];
        editor.measure_value_equation(true);
        let id = editor.doc.objects().last().unwrap().id;
        assert!(
            matches!(editor.doc.objects()[id-1].kind, Kind::ValueEquation { kind: ValueEquationKind::CircleEquation, first, .. } if first == host)
        );
        assert!(editor.doc.value_equation_value(id).is_some());
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
    editor.selection_order = vec![segment];
    let count = editor.doc.objects().len();
    editor.measure_value_equation(true);
    assert_eq!(editor.doc.objects().len(), count);
}

#[test]
fn value_of_point_accepts_transformed_segment_and_circle_constraints() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let b = editor.doc.add(Kind::Free(Point { x: 180., y: 180. }));
    let segment = editor.doc.add(Kind::Segment(a, b));
    let circle = editor.doc.add(Kind::Circle(a, b));
    for (kind, target, expected) in [
        (
            Kind::RotatedSegment(segment, a, 90.),
            Point { x: 100., y: 140. },
            0.5,
        ),
        (
            Kind::RotatedCircle(circle, a, 90.),
            Point { x: 100., y: 100. },
            0.25,
        ),
    ] {
        let host = editor.doc.add(kind);
        let constrained = editor.doc.constrained_point(host, target).unwrap();
        let point = editor.doc.add(constrained);
        editor.selection_order = vec![point];
        editor.measure_value_equation(false);
        let measurement = editor.doc.objects().last().unwrap().id;
        assert!((editor.doc.numeric_value(measurement).unwrap() - expected).abs() < 1e-12);
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}

#[test]
fn measure_slope_accepts_constructed_and_transformed_lines() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let b = editor.doc.add(Kind::Free(Point { x: 180., y: 140. }));
    let line = editor.doc.add(Kind::Segment(a, b));
    for kind in [Kind::Parallel(a, line), Kind::RotatedSegment(line, a, 90.)] {
        let host = editor.doc.add(kind);
        editor.selection_order = vec![host];
        editor.measure_slope();
        let id = editor.doc.objects().last().unwrap().id;
        assert!(
            matches!(editor.doc.objects()[id - 1].kind, Kind::SlopeMeasurement(parent, _) if parent == host)
        );
        assert!(editor.doc.slope_value(id).is_some());
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
    editor.selection_order = vec![a];
    let count = editor.doc.objects().len();
    editor.measure_slope();
    assert_eq!(editor.doc.objects().len(), count);
    assert!(editor.status.contains("straight object"));
}

#[test]
fn measure_distance_accepts_point_and_straight_object_in_either_order() {
    for reversed in [false, true] {
        let mut editor = Editor::new(Document::new());
        let a = editor.doc.add(Kind::Free(Point { x: 100., y: 180. }));
        let b = editor.doc.add(Kind::Free(Point { x: 200., y: 180. }));
        let point = editor.doc.add(Kind::Free(Point { x: 250., y: 120. }));
        let host = editor.doc.add(Kind::Segment(a, b));
        editor.selection_order = if reversed {
            vec![host, point]
        } else {
            vec![point, host]
        };
        editor.measure_distance();
        assert_eq!(
            editor.doc.objects().last().unwrap().kind,
            Kind::PointLineDistance(point, host)
        );
        assert_eq!(
            editor.doc.measurement_text(5).unwrap(),
            "Distance = 1.59 cm"
        );
    }
}

#[test]
fn polygon_value_command_accepts_constraints_and_independent_points_in_both_orders() {
    for selection in 0..5 {
        let mut editor = Editor::new(Document::new());
        let vertices = [
            Point { x: 100., y: 100. },
            Point { x: 300., y: 100. },
            Point { x: 300., y: 200. },
            Point { x: 100., y: 200. },
        ]
        .map(|p| editor.doc.add(Kind::Free(p)));
        let polygon = editor.doc.add(Kind::Polygon(vertices.to_vec()));
        let point = match selection {
            0 => editor.doc.add(Kind::PointOnPolygon(polygon, 0, 0.75)),
            1 => editor
                .doc
                .plot_literal_on_path(0.25, Some(polygon))
                .unwrap(),
            2 => {
                let parameter = editor.doc.add_named_parameter("t", 0.25, (10, 10)).unwrap();
                editor
                    .doc
                    .plot_parameter_on_path(parameter, Some(polygon))
                    .unwrap()
            }
            _ => editor.doc.add(Kind::Free(Point { x: 250., y: 80. })),
        };
        editor.selection_order = match selection {
            3 => vec![point, polygon],
            4 => vec![polygon, point],
            _ => vec![point],
        };
        editor.measure_value_equation(false);
        let measurement = editor.doc.objects().last().unwrap().id;
        assert!(
            matches!(editor.doc.objects()[measurement-1].kind,
            Kind::ValueEquation{kind:ValueEquationKind::PointValue,first,second} if first==point&&second==polygon),
            "selection {selection}, status {}",
            editor.status
        );
        assert!(matches!(editor.doc.value_equation_value(measurement),
            Some(sketchpad_engine::value_equation_records::EquationValue::PointValue(t)) if (t-0.25).abs()<1e-12));
        let bytes = editor.doc.encode().unwrap();
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.encode().unwrap(), bytes);
        editor.doc.undo();
        assert_eq!(editor.doc.objects().len(), measurement - 1);
        editor.doc.redo();
        assert_eq!(editor.doc.encode().unwrap(), bytes);
    }
}

#[test]
fn axis_point_command_creation_drag_and_value_use_the_same_native_parameter() {
    let mut editor = Editor::new(Document::new());
    let frame = editor
        .doc
        .add_default_coordinate_system(Point { x: 240., y: 260. })
        .unwrap();
    let Kind::CoordinateSystem(axis, _) = editor.doc.objects()[frame - 1].kind else {
        panic!()
    };
    let point = editor.doc.add(
        editor
            .doc
            .constrained_point(axis, Point { x: 315., y: 200. })
            .unwrap(),
    );
    editor.selection_order = vec![point];
    editor.measure_value_equation(false);
    let measurement = editor.doc.objects().last().unwrap().id;
    assert!(matches!(editor.doc.value_equation_value(measurement),
        Some(sketchpad_engine::value_equation_records::EquationValue::PointValue(t)) if (t-0.875).abs()<1e-12));
    editor.doc.checkpoint();
    assert!(editor.doc.move_point(point, Point { x: 280., y: 200. }));
    assert!(matches!(editor.doc.value_equation_value(measurement),
        Some(sketchpad_engine::value_equation_records::EquationValue::PointValue(t)) if (t-0.7).abs()<1e-12));
    editor.doc.verify_roundtrip().unwrap();
    editor.doc.undo();
    assert!(matches!(editor.doc.value_equation_value(measurement),
        Some(sketchpad_engine::value_equation_records::EquationValue::PointValue(t)) if (t-0.875).abs()<1e-12));
}

#[test]
fn attached_numeric_label_command_accepts_measurement_units_and_live_edits() {
    let mut editor = Editor::new(Document::new());
    let a = editor.doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let b = editor.doc.add(Kind::Free(Point { x: 180., y: 180. }));
    let distance = editor.doc.add(Kind::PointDistance(a, b));
    editor.doc.selected = [a, distance].into_iter().collect();
    editor.attach_numeric_label_selected();
    let label = editor.doc.objects().last().unwrap().id;
    assert!(
        matches!(editor.doc.objects()[label-1].kind, Kind::PointNumericLabel(point, value) if point == a && value == distance)
    );
    assert!(
        editor
            .doc
            .point_numeric_label(label)
            .unwrap()
            .1
            .ends_with(" cm")
    );
    let before = editor.doc.point_numeric_label(label).unwrap().1;
    editor.doc.checkpoint();
    assert!(editor.doc.move_point(b, Point { x: 260., y: 180. }));
    let after = editor.doc.point_numeric_label(label).unwrap().1;
    assert_ne!(before, after);
    editor.doc.verify_roundtrip().unwrap();
    editor.doc.undo();
    assert_eq!(editor.doc.point_numeric_label(label).unwrap().1, before);
    editor.doc.selected = [a, b].into_iter().collect();
    let count = editor.doc.objects().len();
    editor.attach_numeric_label_selected();
    assert_eq!(editor.doc.objects().len(), count);
}
