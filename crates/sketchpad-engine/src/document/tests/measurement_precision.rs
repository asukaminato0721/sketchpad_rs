use super::*;

#[test]
fn scalar_measurement_precision_native_values_copy_and_undefined_slope() {
    for bytes in [
        include_bytes!(
            "../../../../../tests/fixtures/scalar_precision_coordinates_native_verified.gsp"
        )
        .as_slice(),
        include_bytes!("../../../../../tests/fixtures/scalar_precision_coordinates_port.gsp")
            .as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/scalar_precision_coordinates_edited_port.gsp"
        )
        .as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        assert_eq!(doc.objects[10].measurement_precision, Some(0));
        assert_eq!(doc.objects[11].measurement_precision, Some(3));
        assert_eq!(doc.objects[12].measurement_precision, Some(5));
        assert_eq!(doc.objects[13].measurement_precision, Some(4));
    }
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/scalar_precision_coordinates_port.gsp"),
        None,
    )
    .unwrap();
    assert!(doc.coordinate_measure_text(11).unwrap().ends_with(" = 1"));
    assert!(
        doc.coordinate_measure_text(12)
            .unwrap()
            .ends_with(" = 0.132")
    );
    assert_eq!(
        doc.coordinate_measure_text(13).as_deref(),
        Some("Coordinate Distance = 1.30319")
    );
    assert_eq!(doc.measurement_value_text(14).as_deref(), Some("0.0000"));
    let value = doc.numeric_value(11).unwrap();
    assert!((doc.numeric_value(15).unwrap() - value * 2.).abs() < 1e-12);
    doc.set_measurement_precision(&[11, 12, 13, 14], 2).unwrap();
    assert_eq!(doc.numeric_value(11), Some(value));
    doc.undo();
    assert_eq!(doc.objects[10].measurement_precision, Some(0));
    doc.selected.insert(11);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = *pasted
        .iter()
        .find(|id| {
            matches!(
                doc.objects[**id - 1].kind,
                Kind::CoordinateMeasurement {
                    kind: CoordinateMeasureKind::Abscissa,
                    ..
                }
            )
        })
        .unwrap();
    assert_eq!(doc.objects[copied - 1].measurement_precision, Some(0));
    assert_eq!(doc.objects[copied - 1].measurement_anchor, Some((30, 53)));
    let numeric_value = doc.numeric_value(copied);
    doc.move_measurement_caption(copied, (-120, 245)).unwrap();
    assert_eq!(doc.numeric_value(copied), numeric_value);
    assert_eq!(
        doc.objects[copied - 1].measurement_anchor,
        Some((-120, 245))
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[copied - 1].measurement_precision, Some(0));
    assert_eq!(
        reopened.objects[copied - 1].measurement_anchor,
        Some((-120, 245))
    );
    assert_eq!(reopened.numeric_value(copied), numeric_value);
    assert!(doc.set_measurement_precision(&[10], 3).is_err());
    doc.move_point(2, Point { x: 150., y: 175. });
    assert!(doc.measurement_value_text(14).is_none());
}

#[test]
fn point_value_precision_retains_full_calculation_and_iteration_depth() {
    for (bytes, value, precision, depth) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/scalar_precision_point_value_iteration_port.gsp"
            )
            .as_slice(),
            0.25,
            5,
            1,
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/scalar_precision_point_value_iteration_edited_port.gsp"
            )
            .as_slice(),
            0.675,
            0,
            3,
        ),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        assert_eq!(doc.objects[4].measurement_precision, Some(precision));
        assert!((doc.numeric_value(5).unwrap() - value).abs() < 1e-12);
        assert!((doc.numeric_value(6).unwrap() - value * 4.).abs() < 1e-12);
        assert_eq!(doc.point_iteration_images(10).unwrap().len(), depth);
        doc.set_measurement_precision(&[5], 2).unwrap();
        assert!((doc.numeric_value(5).unwrap() - value).abs() < 1e-12);
        assert_eq!(doc.point_iteration_images(10).unwrap().len(), depth);
        doc.undo();
        assert_eq!(doc.objects[4].measurement_precision, Some(precision));
        doc.selected.insert(5);
        let clipboard = doc.copy_selected().unwrap();
        let pasted = doc.paste(&clipboard).unwrap();
        let copied = *pasted
            .iter()
            .find(|id| {
                matches!(
                    doc.objects[**id - 1].kind,
                    Kind::ValueEquation {
                        kind: ValueEquationKind::PointValue,
                        ..
                    }
                )
            })
            .unwrap();
        assert_eq!(
            doc.objects[copied - 1].measurement_precision,
            Some(precision)
        );
        assert!(doc.value_equation_text(copied).is_some());
        assert!(
            !Document::from_bytes(&doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}

#[test]
fn measurement_precision_preserves_values_undo_clipboard_and_all_native_kinds() {
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/measurement_precision_basic_port.gsp")
            .as_slice(),
        include_bytes!("../../../../../tests/fixtures/measurement_precision_basic_edited_port.gsp")
            .as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/measurement_precision_polygon_ratio_port.gsp"
        )
        .as_slice(),
        include_bytes!("../../../../../tests/fixtures/measurement_precision_arc_port.gsp")
            .as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        for object in doc
            .objects
            .iter()
            .filter(|object| object.kind.has_measurement_precision())
        {
            assert!(doc.measurement_text(object.id).is_some());
            assert!(
                object
                    .measurement_precision
                    .is_some_and(|precision| precision <= 5)
            );
        }
    }
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/measurement_precision_basic_port.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(doc.measurement_text(4).as_deref(), Some("Length = 1 cm"));
    assert_eq!(doc.measurement_text(6).as_deref(), Some("Area = 5.498 cm²"));
    assert_eq!(
        doc.measurement_text(7).as_deref(),
        Some("Radius = 1.32292 cm")
    );
    assert_eq!(doc.measurement_text(9).as_deref(), Some("Angle = 45.0000°"));
    let length = doc.numeric_value(4).unwrap();
    let calculation = doc.numeric_value(10).unwrap();
    assert!((calculation - length * 2.).abs() < 1e-12);
    let before = doc.encode().unwrap();
    assert_eq!(doc.set_measurement_precision(&[4, 6, 7, 9], 2).unwrap(), 4);
    assert_eq!(doc.numeric_value(4), Some(length));
    assert_eq!(doc.numeric_value(10), Some(calculation));
    assert_eq!(doc.measurement_text(4).as_deref(), Some("Length = 1.32 cm"));
    assert_eq!(doc.set_measurement_precision(&[4], 2).unwrap(), 0);
    doc.undo();
    assert_eq!(doc.encode().unwrap(), before);
    doc.redo();
    assert_eq!(doc.measurement_text(4).as_deref(), Some("Length = 1.32 cm"));
    assert!(doc.set_measurement_precision(&[4, 1], 3).is_err());
    assert_eq!(doc.objects[3].measurement_precision, Some(2));
    assert!(doc.set_measurement_precision(&[4], 6).is_err());
    doc.set_measurement_precision(&[4], 5).unwrap();
    doc.selected.insert(4);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = *pasted
        .iter()
        .find(|id| matches!(doc.objects[**id - 1].kind, Kind::SegmentLength(_)))
        .unwrap();
    assert_eq!(doc.objects[copied - 1].measurement_precision, Some(5));
    assert_eq!(doc.measurement_text(copied), doc.measurement_text(4));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.measurement_text(copied),
        doc.measurement_text(copied)
    );
    let mut deleted = reopened;
    deleted.selected.insert(4);
    deleted.clear_selected().unwrap();
    assert!(
        !Document::from_bytes(&deleted.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn measurement_precision_multi_page_creation_and_existing_state() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/two_page_blank_native.gsp"),
        None,
    )
    .unwrap();
    for page in [1, 0] {
        doc.active_page = page;
        let a = doc.add(Kind::Free(Point { x: 100., y: 180. }));
        let b = doc.add(Kind::Free(Point { x: 150., y: 180. }));
        let segment = doc.add(Kind::Segment(a, b));
        let length = doc.add(Kind::SegmentLength(segment));
        doc.set_measurement_precision(&[length], if page == 1 { 5 } else { 0 })
            .unwrap();
    }
    let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(
        reopened
            .objects
            .iter()
            .all(|object| !matches!(object.kind, Kind::Unsupported(_)))
    );
    let measurements = reopened
        .objects
        .iter()
        .filter(|o| o.kind.has_measurement_precision())
        .map(|o| (o.id, o.page, o.measurement_precision))
        .collect::<Vec<_>>();
    assert_eq!(
        measurements
            .iter()
            .map(|(_, page, precision)| (*page, *precision))
            .collect::<Vec<_>>(),
        [(0, Some(0)), (1, Some(5))]
    );
    reopened.active_page = 1;
    assert!(
        reopened
            .set_measurement_precision(&[measurements[0].0], 2)
            .is_err()
    );
    reopened
        .set_measurement_precision(&[measurements[1].0], 3)
        .unwrap();
    let edited = Document::from_bytes(&reopened.encode().unwrap(), None).unwrap();
    assert_eq!(
        edited.objects[measurements[1].0 - 1].measurement_precision,
        Some(3)
    );
    assert_eq!(
        edited.objects[measurements[0].0 - 1].measurement_precision,
        Some(0)
    );
}
