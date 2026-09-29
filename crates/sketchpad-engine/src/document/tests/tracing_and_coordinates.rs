use super::*;

#[test]
fn trace_flags_follow_undo_and_round_trip_for_supported_shapes() {
    for (bytes, traced_id) in [
        (
            include_bytes!("../../../../../tests/fixtures/trace_point_original.gsp").as_slice(),
            1,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/trace_segment_original.gsp").as_slice(),
            3,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/trace_port_point_saved_by_wine.gsp")
                .as_slice(),
            1,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/trace_port_segment_saved_by_wine.gsp")
                .as_slice(),
            3,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/trace_port_point_dragged_by_wine.gsp")
                .as_slice(),
            1,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/trace_circle_original.gsp").as_slice(),
            3,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/trace_polygon_original.gsp").as_slice(),
            7,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/trace_ray_original.gsp").as_slice(),
            3,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/trace_line_original.gsp").as_slice(),
            3,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/trace_arc_original.gsp").as_slice(),
            4,
        ),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert!(doc.objects[traced_id - 1].tracing);
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.selected.insert(traced_id);
        assert_eq!(doc.set_selected_tracing(false).unwrap(), 1);
        assert!(!doc.objects[traced_id - 1].tracing);
        doc.undo();
        assert!(doc.objects[traced_id - 1].tracing);
        doc.redo();
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.objects[traced_id - 1].tracing);
    }

    let mut doc = Document::new();
    let id = doc.add(Kind::Free(Point { x: 200., y: 150. }));
    doc.selected.insert(id);
    assert_eq!(doc.set_selected_tracing(true).unwrap(), 1);
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(reopened.objects[0].tracing);
    assert!(!reopened.partial_view);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    assert!(doc.objects[pasted[0] - 1].tracing);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(reopened.objects[pasted[0] - 1].tracing);

    let a = doc.add(Kind::Free(Point { x: 120., y: 150. }));
    let b = doc.add(Kind::Free(Point { x: 170., y: 150. }));
    let c = doc.add(Kind::Free(Point { x: 145., y: 100. }));
    let circle = doc.add(Kind::Circle(a, b));
    let arc = doc.add(Kind::ThreePointArc(a, c, b));
    let polygon = doc.add(Kind::Polygon(vec![a, b, c]));
    let ray = doc.add(Kind::Ray(a, b));
    let line = doc.add(Kind::Line(a, b));
    doc.selected.clear();
    doc.selected.extend([circle, arc, polygon, ray, line]);
    assert_eq!(doc.set_selected_tracing(true).unwrap(), 5);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    for id in [circle, arc, polygon, ray, line] {
        assert!(reopened.objects[id - 1].tracing);
    }
}

#[test]
fn coordinate_measurements_decode_track_drag_and_reopen() {
    for (source, kind, expected) in [
        (
            include_bytes!("../../../../../tests/fixtures/coordinate_measure_menu257.gsp")
                .as_slice(),
            CoordinateMeasureKind::Coordinates,
            -1.296458333333333,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/coordinate_measure_menu274.gsp")
                .as_slice(),
            CoordinateMeasureKind::Abscissa,
            -1.296458333333333,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/coordinate_measure_menu291.gsp")
                .as_slice(),
            CoordinateMeasureKind::Ordinate,
            0.132291666666667,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/coordinate_measure_menu308.gsp")
                .as_slice(),
            CoordinateMeasureKind::CoordinateDistance,
            1.3032,
        ),
    ] {
        let mut doc = Document::from_bytes(source, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), source);
        let Kind::CoordinateMeasurement {
            kind: actual,
            first: 1,
            frame: 9,
            ..
        } = doc.objects[9].kind
        else {
            panic!("measurement not decoded")
        };
        assert_eq!(actual, kind);
        let before = doc.coordinate_measure_value(10).unwrap();
        let scalar = match before {
            CoordinateMeasureValue::Pair(p) => p.x,
            CoordinateMeasureValue::Scalar(v) => v,
        };
        assert!((scalar - expected).abs() < 0.01);
        doc.checkpoint();
        assert!(doc.move_point(1, Point { x: 170., y: 100. }));
        assert_ne!(doc.coordinate_measure_value(10), Some(before));
        doc.undo();
        assert_eq!(doc.coordinate_measure_value(10), Some(before));
        doc.redo();
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.coordinate_measure_value(10),
            doc.coordinate_measure_value(10)
        );
    }
}

#[test]
fn new_coordinate_measurements_serialize_with_valid_marker_and_references() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/coordinate_measure_base.gsp"),
        None,
    )
    .unwrap();
    let frame = doc.active_coordinate_system().unwrap();
    for (kind, first, second) in [
        (CoordinateMeasureKind::Coordinates, 1, None),
        (CoordinateMeasureKind::Abscissa, 2, None),
        (CoordinateMeasureKind::Ordinate, 2, None),
        (CoordinateMeasureKind::CoordinateDistance, 1, Some(4)),
    ] {
        doc.add(Kind::CoordinateMeasurement {
            kind,
            first,
            second,
            frame,
        });
    }
    let bytes = doc.encode().unwrap();
    let marker = records(&bytes)
        .unwrap()
        .into_iter()
        .find(|r| r.kind == 0x2328)
        .unwrap();
    assert_eq!(u32_at(&bytes, marker.data.start), 13);
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects.len(), 13);
    assert_eq!(
        reopened.coordinate_measure_text(11).as_deref(),
        Some("x_B = 1.35")
    );
    assert!(
        matches!(reopened.coordinate_measure_value(13), Some(CoordinateMeasureValue::Scalar(v)) if (v-1.30).abs()<0.01)
    );
}
