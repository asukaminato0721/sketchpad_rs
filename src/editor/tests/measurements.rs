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
        let id = editor.doc.objects.last().unwrap().id;
        assert!(
            matches!(editor.doc.objects[id-1].kind, Kind::ValueEquation { kind: ValueEquationKind::LineEquation, first, .. } if first == host)
        );
        assert!(editor.doc.value_equation_value(id).is_some());
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
    let frame = editor.doc.active_coordinate_system().unwrap();
    let Kind::CoordinateSystem(x_axis, _) = editor.doc.objects[frame - 1].kind else {
        panic!("Expected frame");
    };
    editor.selection_order = vec![x_axis];
    editor.measure_value_equation(true);
    assert!(
        editor
            .doc
            .value_equation_value(editor.doc.objects.last().unwrap().id)
            .is_some()
    );
    let bisector = editor.doc.add(Kind::Bisector(a, b, c));
    editor.selection_order = vec![bisector];
    let count = editor.doc.objects.len();
    editor.measure_value_equation(true);
    assert_eq!(editor.doc.objects.len(), count);
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
        let id = editor.doc.objects.last().unwrap().id;
        assert!(
            matches!(editor.doc.objects[id-1].kind, Kind::ValueEquation { kind: ValueEquationKind::CircleEquation, first, .. } if first == host)
        );
        assert!(editor.doc.value_equation_value(id).is_some());
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
    editor.selection_order = vec![segment];
    let count = editor.doc.objects.len();
    editor.measure_value_equation(true);
    assert_eq!(editor.doc.objects.len(), count);
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
        let measurement = editor.doc.objects.last().unwrap().id;
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
        let id = editor.doc.objects.last().unwrap().id;
        assert!(
            matches!(editor.doc.objects[id - 1].kind, Kind::SlopeMeasurement(parent, _) if parent == host)
        );
        assert!(editor.doc.slope_value(id).is_some());
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
    editor.selection_order = vec![a];
    let count = editor.doc.objects.len();
    editor.measure_slope();
    assert_eq!(editor.doc.objects.len(), count);
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
            editor.doc.objects.last().unwrap().kind,
            Kind::PointLineDistance(point, host)
        );
        assert_eq!(
            editor.doc.measurement_text(5).unwrap(),
            "Distance = 1.59 cm"
        );
    }
}
