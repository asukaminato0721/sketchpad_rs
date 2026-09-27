use super::*;

#[test]
fn numeric_ratio_shapes_match_native_and_track_signed_parameter_edits() {
    for (native_bytes, bytes, edited, image, expected) in [
        (
            include_bytes!("../../../tests/fixtures/numeric_ratio_point_native.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/numeric_ratio_point_port.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/numeric_ratio_point_edited_port.gsp")
                .as_slice(),
            6,
            Kind::CalculatedRatioScaledPoint(3, 2, 1),
        ),
        (
            include_bytes!("../../../tests/fixtures/numeric_ratio_segment_native.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/numeric_ratio_segment_port.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/numeric_ratio_segment_edited_port.gsp")
                .as_slice(),
            7,
            Kind::CalculatedRatioScaledSegment(6, 2, 1),
        ),
        (
            include_bytes!("../../../tests/fixtures/numeric_ratio_circle_native.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/numeric_ratio_circle_port.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/numeric_ratio_circle_edited_port.gsp")
                .as_slice(),
            7,
            Kind::CalculatedRatioScaledCircle(6, 2, 1),
        ),
        (
            include_bytes!("../../../tests/fixtures/numeric_ratio_polygon_native.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/numeric_ratio_polygon_port.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/numeric_ratio_polygon_edited_port.gsp")
                .as_slice(),
            7,
            Kind::CalculatedRatioScaledPolygon(6, 2, 1),
        ),
    ] {
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.objects[image - 1].kind, expected);
        assert_eq!(native.encode().unwrap(), native_bytes);
        let group = |bytes: &[u8]| {
            let records = records(bytes).unwrap();
            let start = records
                .iter()
                .filter(|r| r.kind == 0x07d0)
                .nth(image - 1)
                .unwrap()
                .start;
            let end = records
                .iter()
                .find(|r| r.start > start && r.kind == 0x07d7)
                .unwrap()
                .data
                .end;
            bytes[start..end].to_vec()
        };
        assert_eq!(group(native_bytes), group(bytes));
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        assert!(transform_is(
            &expected,
            NativeTransformFlavor::NumericDilation,
            &[
                ElementType::Point,
                ElementType::Segment,
                ElementType::Circle,
                ElementType::Polygon
            ]
        ));
        match expected.transform_definition().unwrap().element() {
            ElementType::Point => {
                assert_eq!(doc.position(image), Some(Point { x: 160., y: 180. }));
                assert_eq!(
                    doc.point_iteration_images(7).unwrap(),
                    vec![
                        Point { x: 130., y: 180. },
                        Point { x: 115., y: 180. },
                        Point { x: 107.5, y: 180. }
                    ]
                );
            }
            ElementType::Segment => assert_eq!(
                doc.line_points(image),
                Some((
                    Point { x: 160., y: 180. },
                    Point { x: 160., y: 200. },
                    LineMode::Segment
                ))
            ),
            ElementType::Circle => assert_eq!(
                doc.circle_points(image),
                Some((Point { x: 160., y: 180. }, 20.))
            ),
            ElementType::Polygon => assert_eq!(
                doc.polygon_points(image).unwrap(),
                vec![
                    Point { x: 160., y: 180. },
                    Point { x: 160., y: 200. },
                    Point { x: 180., y: 200. }
                ]
            ),
            element => panic!("Unexpected numeric dilation element: {element:?}"),
        }
        doc.set_parameter_value(1, -0.5).unwrap();
        assert_eq!(doc.encode().unwrap(), edited);
        if image == 6 {
            assert_eq!(doc.position(image), Some(Point { x: 40., y: 180. }));
            assert_eq!(
                doc.point_iteration_images(7).unwrap()[1],
                Point { x: 85., y: 180. }
            );
        }
        doc.selected.insert(image);
        let clipboard = doc.copy_selected().unwrap();
        let pasted = doc.paste(&clipboard).unwrap();
        let copied = *pasted
            .iter()
            .find(|id| {
                transform_is(
                    &doc.objects[**id - 1].kind,
                    NativeTransformFlavor::NumericDilation,
                    &[
                        ElementType::Point,
                        ElementType::Segment,
                        ElementType::Circle,
                        ElementType::Polygon,
                    ],
                )
            })
            .unwrap();
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(copied), doc.position(copied));
        assert_eq!(reopened.line_points(copied), doc.line_points(copied));
        assert_eq!(reopened.circle_points(copied), doc.circle_points(copied));
        assert_eq!(reopened.polygon_points(copied), doc.polygon_points(copied));
        doc.selected.clear();
        doc.selected.insert(1);
        assert!(doc.clear_selected().unwrap().contains(&image));
        assert!(
            !Document::from_bytes(&doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}

#[test]
fn numeric_ratio_calculations_drive_carried_circles_and_numeric_labels() {
    let bytes = include_bytes!(
        "../../../tests/fixtures/numeric_ratio_calculated_circle_iteration_port.gsp"
    );
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(
        doc.circle_iteration_images(10).unwrap(),
        vec![
            (Point { x: 190., y: 180. }, 22.5),
            (Point { x: 167.5, y: 180. }, 16.875),
            (
                Point {
                    x: 150.625,
                    y: 180.
                },
                12.65625
            )
        ]
    );
    doc.set_parameter_value(1, 0.25).unwrap();
    doc.set_parameter_value(7, 4.8).unwrap();
    assert_eq!(
        doc.encode().unwrap(),
        include_bytes!(
            "../../../tests/fixtures/numeric_ratio_calculated_circle_iteration_edited_port.gsp"
        )
    );
    assert_eq!(doc.circle_iteration_images(10).unwrap().len(), 4);
    assert_eq!(
        doc.circle_iteration_images(10).unwrap()[3],
        (Point { x: 107.5, y: 180. }, 3.75)
    );
    let numeric = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/numeric_ratio_numeric_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert!(!numeric.partial_view);
    assert_eq!(
        numeric.point_parameter_iteration_labels(9).unwrap(),
        vec![
            (Point { x: 130., y: 180. }, "4".into()),
            (Point { x: 175., y: 180. }, "8".into()),
            (Point { x: 242.5, y: 180. }, "16".into())
        ]
    );
    let zero = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/numeric_ratio_circle_zero_port.gsp"),
        None,
    )
    .unwrap();
    assert!(!zero.partial_view);
    assert!(zero.circle_points(7).is_none());
    let mut zero_new = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/numeric_ratio_circle_base_port.gsp"),
        None,
    )
    .unwrap();
    zero_new.set_parameter_value(1, 0.).unwrap();
    zero_new.add_calculated_ratio_dilation(6, 2, 1).unwrap();
    assert!(
        !Document::from_bytes(&zero_new.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn measured_numeric_ratio_is_pinned_and_undefined_calculations_recover() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/numeric_ratio_measured_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert!(!doc.partial_view);
    assert_eq!(
        doc.point_iteration_images(6).unwrap(),
        vec![
            Point { x: 130., y: 180. },
            Point { x: 115., y: 180. },
            Point { x: 107.5, y: 180. }
        ]
    );
    doc.move_point(3, Point { x: 100., y: 180. });
    assert!(doc.point_iteration_images(6).is_none());
    doc.move_point(3, Point { x: 340., y: 180. });
    assert_eq!(doc.point_iteration_images(6).unwrap().len(), 3);
    let n = doc.add_named_parameter("n", 2., (200, 30)).unwrap();
    let ratio = doc.add_parameter_calculation(n, "1/n", (200, 60)).unwrap();
    let image = doc.add_calculated_ratio_dilation(1, 2, ratio).unwrap();
    let iteration = doc
        .add_point_iteration(1, image, IterationDepth::Fixed(3))
        .unwrap();
    doc.set_parameter_value(n, 0.).unwrap();
    assert!(doc.position(image).is_none());
    assert!(doc.point_iteration_images(iteration).is_none());
    doc.set_parameter_value(n, 2.).unwrap();
    assert_eq!(doc.point_iteration_images(iteration).unwrap().len(), 3);
    let degrees = doc.add_calculation_expression("30°", (10, 130)).unwrap();
    let count = doc.objects.len();
    assert!(doc.add_calculated_ratio_dilation(1, 2, degrees).is_err());
    assert_eq!(doc.objects.len(), count);
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn numeric_angle_shapes_match_native_records_and_follow_edits() {
    for (native_bytes, bytes, edited, expected) in [
        (
            include_bytes!("../../../tests/fixtures/numeric_angle_segment_native.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/numeric_angle_segment_port.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/numeric_angle_segment_edited_port.gsp")
                .as_slice(),
            Kind::CalculatedAngleRotatedSegment(9, 1, 5),
        ),
        (
            include_bytes!("../../../tests/fixtures/numeric_angle_circle_native.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/numeric_angle_circle_port.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/numeric_angle_circle_edited_port.gsp")
                .as_slice(),
            Kind::CalculatedAngleRotatedCircle(9, 1, 5),
        ),
        (
            include_bytes!("../../../tests/fixtures/numeric_angle_polygon_native.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/numeric_angle_polygon_port.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/numeric_angle_polygon_edited_port.gsp")
                .as_slice(),
            Kind::CalculatedAngleRotatedPolygon(9, 1, 5),
        ),
    ] {
        let native = Document::from_bytes(native_bytes, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.objects[9].kind, expected);
        assert_eq!(native.encode().unwrap(), native_bytes);
        let group = |bytes: &[u8]| {
            let records = records(bytes).unwrap();
            let start = records
                .iter()
                .filter(|r| r.kind == 0x07d0)
                .nth(9)
                .unwrap()
                .start;
            let end = records
                .iter()
                .find(|r| r.start > start && r.kind == 0x07d7)
                .unwrap()
                .data
                .end;
            bytes[start..end].to_vec()
        };
        assert_eq!(group(native_bytes), group(bytes));
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects[9].kind, expected);
        assert_eq!(doc.encode().unwrap(), bytes);
        assert!(transform_is(
            &expected,
            NativeTransformFlavor::NumericRotation,
            &[
                ElementType::Segment,
                ElementType::Circle,
                ElementType::Polygon
            ]
        ));
        match expected.transform_definition().unwrap().element() {
            ElementType::Segment => {
                let (a, b, mode) = doc.line_points(10).unwrap();
                assert_eq!(mode, LineMode::Segment);
                assert!(a.distance(Point { x: 160., y: 120. }) < 1e-10);
                assert!(b.distance(Point { x: 200., y: 120. }) < 1e-10);
            }
            ElementType::Circle => {
                let (center, radius) = doc.circle_points(10).unwrap();
                assert!(center.distance(Point { x: 160., y: 120. }) < 1e-10);
                assert_eq!(radius, 40.);
            }
            ElementType::Polygon => {
                for (actual, expected) in doc.polygon_points(10).unwrap().iter().zip([
                    Point { x: 160., y: 120. },
                    Point { x: 200., y: 120. },
                    Point { x: 200., y: 80. },
                ]) {
                    assert!(actual.distance(expected) < 1e-10);
                }
            }
            element => panic!("Unexpected numeric rotation element: {element:?}"),
        }
        doc.move_point(4, Point { x: 150., y: 110. });
        doc.selected.insert(10);
        doc.set_selected_color(RgbColor {
            red: 40,
            green: 80,
            blue: 200,
        })
        .unwrap();
        assert_eq!(doc.encode().unwrap(), edited);
        let clipboard = doc.copy_selected().unwrap();
        let pasted = doc.paste(&clipboard).unwrap();
        let copied = *pasted
            .iter()
            .find(|id| {
                transform_is(
                    &doc.objects[**id - 1].kind,
                    NativeTransformFlavor::NumericRotation,
                    &[
                        ElementType::Segment,
                        ElementType::Circle,
                        ElementType::Polygon,
                    ],
                )
            })
            .unwrap();
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.objects[copied - 1].visual_style,
            doc.objects[9].visual_style
        );
        assert_eq!(reopened.line_points(copied), doc.line_points(copied));
        assert_eq!(reopened.circle_points(copied), doc.circle_points(copied));
        assert_eq!(reopened.polygon_points(copied), doc.polygon_points(copied));
        doc.selected.clear();
        doc.selected.insert(5);
        assert!(doc.clear_selected().unwrap().contains(&10));
        assert!(
            !Document::from_bytes(&doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}

#[test]
fn calculated_angles_rotate_shapes_and_undefined_sources_recover() {
    let bytes = include_bytes!(
        "../../../tests/fixtures/numeric_angle_calculated_segment_measurement_port.gsp"
    );
    let mut measured = Document::from_bytes(bytes, None).unwrap();
    assert!(!measured.partial_view);
    assert_eq!(measured.encode().unwrap(), bytes);
    assert!((measured.numeric_value(13).unwrap() - 40. / 37.79527559055118).abs() < 1e-10);
    measured.set_parameter_value(10, 8.).unwrap();
    measured.move_point(7, Point { x: 220., y: 240. });
    assert_eq!(
        measured.encode().unwrap(),
        include_bytes!(
            "../../../tests/fixtures/numeric_angle_calculated_segment_measurement_edited_port.gsp"
        )
    );
    assert!((measured.numeric_value(13).unwrap() - 60. / 37.79527559055118).abs() < 1e-10);
    for family in 0..3 {
        let mut doc = Document::from_bytes(
            include_bytes!("../../../tests/fixtures/numeric_angle_polygon_port.gsp"),
            None,
        )
        .unwrap();
        let source = match family {
            0 => doc.add(Kind::Segment(6, 7)),
            1 => doc.add(Kind::Circle(6, 7)),
            _ => 9,
        };
        let parameter = doc.add_named_parameter("n", 4., (200, 30)).unwrap();
        let angle = doc
            .add_parameter_calculation(parameter, "360°/n", (200, 60))
            .unwrap();
        let transformed = doc.add_calculated_angle_rotation(source, 1, angle).unwrap();
        let measurement = doc.add(match family {
            0 => Kind::SegmentLength(transformed),
            1 => Kind::CircleRadius(transformed),
            _ => Kind::PolygonArea(transformed),
        });
        let value = doc.numeric_value(measurement).unwrap();
        assert!(value > 0.);
        let encoded = doc.encode().unwrap();
        let mut doc = Document::from_bytes(&encoded, None).unwrap();
        doc.set_parameter_value(parameter, 0.).unwrap();
        assert!(doc.numeric_value(measurement).is_none());
        assert!(doc.line_points(transformed).is_none());
        assert!(doc.circle_points(transformed).is_none());
        assert!(doc.polygon_points(transformed).is_none());
        doc.set_parameter_value(parameter, 8.).unwrap();
        assert!((doc.numeric_value(measurement).unwrap() - value).abs() < 1e-10);
        assert!(
            !Document::from_bytes(&doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
        let scalar = doc.add_calculation_expression("2", (10, 120)).unwrap();
        let count = doc.objects.len();
        assert!(
            doc.add_calculated_angle_rotation(source, 1, scalar)
                .is_err()
        );
        assert_eq!(doc.objects.len(), count);
    }
}

#[test]
fn measured_angles_drive_rotation_and_iteration_without_intermediate_calculation() {
    let native_bytes = include_bytes!("../../../tests/fixtures/measured_angle_rotation_native.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(
        native.objects[6].kind,
        Kind::CalculatedAngleRotatedPoint(1, 2, 6)
    );
    let bytes = include_bytes!("../../../tests/fixtures/measured_angle_segment_iteration_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    let group = |bytes: &[u8]| {
        let records = records(bytes).unwrap();
        let start = records
            .iter()
            .filter(|record| record.kind == 0x07d0)
            .nth(6)
            .unwrap()
            .start;
        let end = records
            .iter()
            .find(|record| record.start > start && record.kind == 0x07d7)
            .unwrap()
            .data
            .end;
        bytes[start..end].to_vec()
    };
    assert_eq!(group(native_bytes), group(bytes));
    assert_eq!(doc.numeric_value(6), Some(90.));
    assert!(
        doc.position(7)
            .unwrap()
            .distance(Point { x: 160., y: 120. })
            < 1e-10
    );
    for (actual, expected) in doc.point_iteration_images(9).unwrap().iter().zip([
        Point { x: 100., y: 180. },
        Point { x: 160., y: 240. },
        Point { x: 220., y: 180. },
    ]) {
        assert!(actual.distance(expected) < 1e-10);
    }
    doc.move_point(5, Point { x: 150., y: 110. });
    assert_eq!(
        doc.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/measured_angle_segment_iteration_edited_port.gsp")
    );
    assert_eq!(doc.numeric_value(6), Some(135.));
    assert!(doc.point_iteration_images(9).unwrap()[0].distance(Point { x: 160., y: 240. }) < 1e-10);
    doc.selected.insert(9);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = *pasted
        .iter()
        .find(|id| matches!(doc.objects[**id - 1].kind, Kind::SegmentIteration(..)))
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.segment_iteration_images(copied),
        doc.segment_iteration_images(copied)
    );
    doc.move_point(5, doc.position(4).unwrap());
    assert!(doc.position(7).is_none());
    assert!(doc.segment_iteration_images(9).is_none());
    doc.move_point(5, Point { x: 200., y: 110. });
    assert_eq!(doc.segment_iteration_images(9).unwrap().len(), 3);
    doc.selected.clear();
    doc.selected.insert(6);
    let removed = doc.clear_selected().unwrap();
    assert!(removed.contains(&7) && removed.contains(&8) && removed.contains(&9));
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn measured_arc_angle_is_pinned_during_circle_iteration_and_tracks_edits() {
    let bytes =
        include_bytes!("../../../tests/fixtures/measured_arc_angle_circle_iteration_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert!((doc.numeric_value(6).unwrap() - 90.).abs() < 1e-10);
    for (actual, expected) in doc.point_iteration_images(10).unwrap().iter().zip([
        Point { x: 100., y: 180. },
        Point { x: 160., y: 240. },
        Point { x: 220., y: 180. },
    ]) {
        assert!(actual.distance(expected) < 1e-10);
    }
    for (_, radius) in doc.circle_iteration_images(12).unwrap() {
        assert!((radius - 7200_f64.sqrt()).abs() < 1e-10);
    }
    doc.move_point(4, Point { x: 112., y: 144. });
    assert_eq!(
        doc.encode().unwrap(),
        include_bytes!(
            "../../../tests/fixtures/measured_arc_angle_circle_iteration_edited_port.gsp"
        )
    );
    assert!(
        doc.position(7)
            .unwrap()
            .distance(Point { x: 112., y: 144. })
            < 1e-10
    );
    doc.set_parameter_value(9, 5.8).unwrap();
    assert_eq!(doc.circle_iteration_images(12).unwrap().len(), 5);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(
        reopened.circle_iteration_images(12),
        doc.circle_iteration_images(12)
    );
}

#[test]
fn calculated_transforms_accept_live_transformed_segment_and_circle_sources() {
    for (name, bytes) in [
        (
            "segment",
            include_bytes!("../../../tests/fixtures/vector_rotated_segment_native.gsp").as_slice(),
        ),
        (
            "circle",
            include_bytes!("../../../tests/fixtures/vector_rotated_circle_native.gsp").as_slice(),
        ),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        let source = doc
            .objects
            .iter()
            .find(|o| {
                transform_is(
                    &o.kind,
                    NativeTransformFlavor::Vector,
                    &[ElementType::Segment, ElementType::Circle],
                )
            })
            .unwrap()
            .id;
        let center = 1;
        let ratio = doc.add_named_parameter("r", 2., (350, 30)).unwrap();
        let scaled = doc
            .add_calculated_ratio_dilation(source, center, ratio)
            .unwrap();
        let angle = doc.add_calculation_expression("90°", (350, 60)).unwrap();
        let rotated = doc
            .add_calculated_angle_rotation(scaled, center, angle)
            .unwrap();
        let measurement = doc.add(if name == "segment" {
            Kind::SegmentLength(rotated)
        } else {
            Kind::CircleRadius(rotated)
        });
        let expected = if name == "segment" {
            let (a, b, _) = doc.line_points(source).unwrap();
            2. * ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt() / 37.79527559055118
        } else {
            2. * doc.circle_points(source).unwrap().1 / 37.79527559055118
        };
        assert!((doc.numeric_value(measurement).unwrap() - expected).abs() < 1e-10);
        doc.verify_roundtrip().unwrap();
        let saved = doc.encode().unwrap();
        assert_eq!(
            saved,
            if name == "segment" {
                include_bytes!(
                    "../../../tests/fixtures/calculated_transform_chain_segment_native_verified.gsp"
                )
                .as_slice()
            } else {
                include_bytes!(
                    "../../../tests/fixtures/calculated_transform_chain_circle_native_verified.gsp"
                )
                .as_slice()
            }
        );
        if std::env::var_os("SKETCHPAD_WRITE_TRANSFORM_CHAIN_FIXTURE").is_some() {
            fs::write(
                format!("/tmp/calculated_transform_chain_{name}_port.gsp"),
                &saved,
            )
            .unwrap();
        }
        let mut reopened = Document::from_bytes(&saved, None).unwrap();
        assert!(!reopened.partial_view);
        reopened.set_parameter_value(ratio, -3.).unwrap();
        assert!((reopened.numeric_value(measurement).unwrap() - expected * 1.5).abs() < 1e-10);
        reopened.verify_roundtrip().unwrap();
        assert_eq!(
            reopened.encode().unwrap(),
            if name == "segment" {
                include_bytes!("../../../tests/fixtures/calculated_transform_chain_segment_edited_native_verified.gsp").as_slice()
            } else {
                include_bytes!("../../../tests/fixtures/calculated_transform_chain_circle_edited_native_verified.gsp").as_slice()
            }
        );
        if std::env::var_os("SKETCHPAD_WRITE_TRANSFORM_CHAIN_FIXTURE").is_some() {
            fs::write(
                format!("/tmp/calculated_transform_chain_{name}_edited_port.gsp"),
                reopened.encode().unwrap(),
            )
            .unwrap();
        }
        reopened.selected.insert(rotated);
        let pasted = reopened.paste(&reopened.copy_selected().unwrap()).unwrap();
        let copy = pasted
            .iter()
            .find(|&&id| {
                transform_is(
                    &reopened.objects[id - 1].kind,
                    NativeTransformFlavor::NumericRotation,
                    &[ElementType::Segment, ElementType::Circle],
                )
            })
            .copied()
            .unwrap();
        let copied_value = if name == "segment" {
            let (a, b, _) = reopened.line_points(copy).unwrap();
            ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt() / 37.79527559055118
        } else {
            reopened.circle_points(copy).unwrap().1 / 37.79527559055118
        };
        assert!((copied_value - expected * 1.5).abs() < 1e-10);
        reopened.verify_roundtrip().unwrap();
        reopened.selected.clear();
        reopened.selected.insert(ratio);
        reopened.clear_selected().unwrap();
        assert!(reopened.objects[rotated - 1].deleted);
        assert!(!reopened.objects[copy - 1].deleted);
        reopened.verify_roundtrip().unwrap();
    }
}
