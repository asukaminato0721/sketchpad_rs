use super::*;

#[test]
fn native_constructed_line_equations_follow_direction_and_vertical_edits() {
    for (name, slope, edit, expected) in [
        (
            "parallel",
            0.5,
            Point { x: 100., y: 100. },
            EquationValue::VerticalLine {
                x: -3.6777083333333334,
            },
        ),
        (
            "perpendicular",
            -2.,
            Point { x: 180., y: 180. },
            EquationValue::VerticalLine {
                x: -3.6777083333333334,
            },
        ),
    ] {
        let bytes =
            std::fs::read(format!("../../tests/fixtures/{name}_line_equation_native.gsp")).unwrap();
        let mut doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        assert_eq!(
            doc.objects[11].kind,
            Kind::ValueEquation {
                kind: ValueEquationKind::LineEquation,
                first: 5,
                second: 11
            }
        );
        let EquationValue::Line { slope: value, .. } = doc.value_equation_value(12).unwrap() else {
            panic!("Expected line equation");
        };
        assert!((value - slope).abs() < 1e-12);
        assert!(doc.move_point(2, edit));
        let (
            Some(EquationValue::VerticalLine { x }),
            EquationValue::VerticalLine { x: expected_x },
        ) = (doc.value_equation_value(12), expected)
        else {
            panic!("Expected vertical equation");
        };
        assert!((x - expected_x).abs() < 1e-12);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.value_equation_value(12),
            doc.value_equation_value(12)
        );
        doc.selected.insert(12);
        let clipboard = doc.copy_selected().unwrap();
        let pasted = doc.paste(&clipboard).unwrap();
        assert!(matches!(
            doc.value_equation_value(*pasted.last().unwrap()),
            Some(EquationValue::VerticalLine { .. })
        ));
        doc.selected.clear();
        doc.selected.insert(5);
        assert!(doc.clear_selected().unwrap().contains(&12));
    }
}
#[test]
fn native_axis_equations_preserve_frame_references_and_constant_captions() {
    for (name, host, expected, caption) in [
        (
            "x_axis",
            3,
            EquationValue::Line {
                slope: 0.,
                intercept: 0.,
            },
            "y = 0.00",
        ),
        (
            "y_axis",
            5,
            EquationValue::VerticalLine { x: 0. },
            "x = 0.00",
        ),
    ] {
        let bytes =
            std::fs::read(format!("../../tests/fixtures/{name}_line_equation_native.gsp")).unwrap();
        let mut doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        assert_eq!(
            doc.objects[6].kind,
            Kind::ValueEquation {
                kind: ValueEquationKind::LineEquation,
                first: host,
                second: 6
            }
        );
        assert_eq!(doc.value_equation_value(7), Some(expected));
        assert_eq!(doc.value_equation_text(7).as_deref(), Some(caption));
        assert!(doc.move_point(1, Point { x: 219., y: 145. }));
        assert_eq!(doc.value_equation_value(7), Some(expected));
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.value_equation_text(7).as_deref(), Some(caption));
    }
}
#[test]
fn line_equations_exclude_segments_rays_and_ray_bisectors() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let vertex = doc.add(Kind::Free(Point { x: 160., y: 180. }));
    let end = doc.add(Kind::Free(Point { x: 160., y: 120. }));
    let segment = doc.add(Kind::Segment(a, vertex));
    let ray = doc.add(Kind::Ray(a, vertex));
    let bisector = doc.add(Kind::Bisector(a, vertex, end));
    for host in [segment, ray, bisector] {
        assert!(!doc.objects[host - 1].kind.is_line());
    }
    assert_eq!(
        straight_kind_mode(&doc.objects[bisector - 1].kind),
        Some(LineMode::Ray)
    );
    let point = doc.add(
        doc.constrained_point(bisector, Point { x: 140., y: 160. })
            .unwrap(),
    );
    assert!(
        matches!(doc.objects[point-1].kind, Kind::PointOnRay(host, t) if host == bisector && t > 0.)
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.objects[point - 1].kind,
        doc.objects[point - 1].kind
    );
    let frame = doc
        .add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    let Kind::CoordinateSystem(x_axis, y_axis) = doc.objects[frame - 1].kind else {
        panic!("Expected frame");
    };
    for (host, expected) in [
        (
            x_axis,
            EquationValue::Line {
                slope: 0.,
                intercept: 0.,
            },
        ),
        (y_axis, EquationValue::VerticalLine { x: 0. }),
    ] {
        assert!(doc.objects[host - 1].kind.is_line());
        let equation = doc.add(Kind::ValueEquation {
            kind: ValueEquationKind::LineEquation,
            first: host,
            second: frame,
        });
        assert_eq!(doc.value_equation_value(equation), Some(expected));
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.value_equation_value(equation), Some(expected));
    }
}

#[test]
fn native_equations_of_constructed_and_transformed_circles_preserve_labels() {
    for name in ["ratio", "segment_radius"] {
        let bytes =
            std::fs::read(format!("../../tests/fixtures/{name}_circle_equation_native.gsp")).unwrap();
        let mut doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        assert_eq!(
            doc.objects[11].kind,
            Kind::ValueEquation {
                kind: ValueEquationKind::CircleEquation,
                first: 5,
                second: 11
            }
        );
        let EquationValue::Circle { center, radius } = doc.value_equation_value(12).unwrap() else {
            panic!("Expected circle equation");
        };
        assert!((center.x + 1.031875).abs() < 1e-12);
        assert!((center.y + 1.4552083333333334).abs() < 1e-12);
        let expected = if name == "ratio" { 0.79375 } else { 1.5875 };
        assert!((radius - expected).abs() < 1e-12);
        if name == "ratio" {
            doc.set_parameter_value(1, -0.75).unwrap();
        } else {
            assert!(doc.move_point(3, Point { x: 130., y: 180. }));
        }
        let expected = if name == "ratio" { 1.190625 } else { 2.38125 };
        let EquationValue::Circle { radius, .. } = doc.value_equation_value(12).unwrap() else {
            panic!("Expected circle equation");
        };
        assert!((radius - expected).abs() < 1e-12);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.value_equation_value(12),
            doc.value_equation_value(12)
        );
        doc.selected.insert(12);
        let clipboard = doc.copy_selected().unwrap();
        let pasted = doc.paste(&clipboard).unwrap();
        let equation = *pasted.last().unwrap();
        assert!(matches!(
            doc.value_equation_value(equation),
            Some(EquationValue::Circle { .. })
        ));
        doc.selected.clear();
        doc.selected.insert(5);
        assert!(doc.clear_selected().unwrap().contains(&12));
    }
}
#[test]
fn circle_equations_follow_transformed_centers_radii_and_frame_edits() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 160., y: 180. }));
    let b = doc.add(Kind::Free(Point { x: 220., y: 180. }));
    let center = doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let mirror_end = doc.add(Kind::Free(Point { x: 100., y: 100. }));
    let mirror = doc.add(Kind::Line(center, mirror_end));
    let circle = doc.add(Kind::Circle(a, b));
    let frame = doc
        .add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    let hosts = [
        Kind::ReflectedCircle(circle, mirror),
        Kind::PolarTranslatedCircle(circle, 90., 37.79527559055118, 1.),
        Kind::RotatedCircle(circle, center, 90.),
        Kind::ScaledCircle(circle, center, -0.5),
    ];
    for kind in hosts {
        let host = doc.add(kind);
        let equation = doc.add(Kind::ValueEquation {
            kind: ValueEquationKind::CircleEquation,
            first: host,
            second: frame,
        });
        let (circle_center, raw_radius) = doc.circle_points(host).unwrap();
        let coordinates = doc
            .graph_frame(frame)
            .unwrap()
            .coordinates(GraphPoint {
                x: circle_center.x,
                y: circle_center.y,
            })
            .unwrap();
        let EquationValue::Circle { center, radius } = doc.value_equation_value(equation).unwrap()
        else {
            panic!("Expected circle equation");
        };
        assert!(
            (center.x - coordinates.x).abs() < 1e-12 && (center.y - coordinates.y).abs() < 1e-12
        );
        assert!((radius - raw_radius / 37.79527559055118).abs() < 1e-12);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.value_equation_value(equation),
            doc.value_equation_value(equation)
        );
    }
    let equation = doc.objects.last().unwrap().id;
    let before = doc.value_equation_value(equation);
    assert!(doc.move_point(a, Point { x: 170., y: 170. }));
    assert_ne!(doc.value_equation_value(equation), before);
    let before = doc.value_equation_value(equation);
    let origin = frame - 5;
    assert!(doc.move_point(origin, Point { x: 209., y: 135. }));
    assert_ne!(doc.value_equation_value(equation), before);
}

#[test]
fn native_transformed_path_points_measure_and_control_iteration_depth() {
    for (shape, point, host, expression, edited, expected, depth_count) in [
        ("segment", 5, 4, "v*4", Point { x: 120., y: 120. }, 0.75, 3),
        ("circle", 6, 5, "v*8+1", Point { x: 130., y: 180. }, 0.5, 5),
    ] {
        let bytes = std::fs::read(format!(
            "../../tests/fixtures/transformed_{shape}_point_native.gsp"
        ))
        .unwrap();
        let mut doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        let position = doc.position(point).unwrap();
        assert!(doc.constrained_point(host, position).is_some());
        let measurement = doc.add(Kind::ValueEquation {
            kind: ValueEquationKind::PointValue,
            first: point,
            second: host,
        });
        let value = if shape == "segment" { 0.5 } else { 0.25 };
        assert!((doc.numeric_value(measurement).unwrap() - value).abs() < 1e-12);
        doc.set_measurement_precision(&[measurement], 4).unwrap();
        let depth = doc
            .add_parameter_calculation(measurement, expression, (10, 40))
            .unwrap();
        let seed = doc.add(Kind::Free(Point { x: 60., y: 220. }));
        let image = doc.add(Kind::PolarTranslatedPoint(seed, 0., 37.79527559055118, 0.5));
        let iteration = doc
            .add_point_iteration(seed, image, IterationDepth::Value(depth))
            .unwrap();
        assert_eq!(
            doc.point_iteration_images(iteration).unwrap().len(),
            if shape == "segment" { 2 } else { 3 }
        );
        assert!(doc.move_point(point, edited));
        assert!((doc.numeric_value(measurement).unwrap() - expected).abs() < 1e-12);
        assert_eq!(
            doc.point_iteration_images(iteration).unwrap().len(),
            depth_count
        );
        let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.numeric_value(measurement),
            doc.numeric_value(measurement)
        );
        assert_eq!(
            reopened.point_iteration_images(iteration),
            doc.point_iteration_images(iteration)
        );
        reopened.selected.insert(measurement);
        let clipboard = reopened.copy_selected().unwrap();
        let pasted = reopened.paste(&clipboard).unwrap();
        assert!(
            (reopened.numeric_value(*pasted.last().unwrap()).unwrap() - expected).abs() < 1e-12
        );
        reopened.selected.clear();
        reopened.selected.insert(host);
        let removed = reopened.clear_selected().unwrap();
        assert!(
            removed.contains(&point)
                && removed.contains(&measurement)
                && removed.contains(&depth)
                && removed.contains(&iteration)
        );
    }
}
#[test]
fn constrained_points_use_supported_path_modes_and_live_transform_geometry() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let b = doc.add(Kind::Free(Point { x: 180., y: 140. }));
    let c = doc.add(Kind::Free(Point { x: 100., y: 100. }));
    let segment = doc.add(Kind::Segment(a, b));
    let circle = doc.add(Kind::Circle(a, b));
    let hosts = [
        Kind::RotatedSegment(segment, a, 90.),
        Kind::ScaledSegment(segment, a, -0.5),
        Kind::ReflectedSegment(segment, doc.add(Kind::Line(a, c))),
        Kind::Parallel(c, segment),
        Kind::Perpendicular(c, segment),
        Kind::PolarTranslatedCircle(circle, 90., 37.79527559055118, 1.),
        Kind::RotatedCircle(circle, c, 30.),
        Kind::ScaledCircle(circle, c, -0.5),
    ];
    for kind in hosts {
        let host = doc.add(kind);
        let target = if let Some((a, b, _)) = doc.line_points(host) {
            Point {
                x: (a.x + b.x) * 0.5,
                y: (a.y + b.y) * 0.5,
            }
        } else {
            let (center, radius) = doc.circle_points(host).unwrap();
            Point {
                x: center.x,
                y: center.y - radius,
            }
        };
        let constrained = doc.constrained_point(host, target).unwrap();
        assert_eq!(
            matches!(constrained, Kind::PointOnSegment(_, _)),
            is_segment_kind(&doc.objects[host - 1].kind)
        );
        let point = doc.add(constrained);
        let measurement = doc.add(Kind::ValueEquation {
            kind: ValueEquationKind::PointValue,
            first: point,
            second: host,
        });
        let expected = if doc.line_points(host).is_some() {
            0.5
        } else {
            0.25
        };
        assert!((doc.numeric_value(measurement).unwrap() - expected).abs() < 1e-12);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert!((reopened.numeric_value(measurement).unwrap() - expected).abs() < 1e-12);
        assert!(doc.position(point).unwrap().distance(target) < 1e-10);
    }
    assert_eq!(doc.constrained_point(a, Point { x: 100., y: 180. }), None);
}

#[test]
fn constructed_and_transformed_slope_measurements_feed_live_calculated_depth() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let b = doc.add(Kind::Free(Point { x: 180., y: 140. }));
    let seed = doc.add(Kind::Free(Point { x: 60., y: 220. }));
    let line = doc.add(Kind::Segment(a, b));
    let frame = doc
        .add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    let angle = doc.add_named_parameter("angle", 30., (10, 5)).unwrap();
    let angle = doc
        .add_parameter_calculation(angle, "angle*1°", (10, 40))
        .unwrap();
    let mirror_end = doc.add(Kind::Free(Point { x: 180., y: 180. }));
    let mirror = doc.add(Kind::Line(a, mirror_end));
    let hosts = [
        (Kind::Parallel(seed, line), 0.5),
        (Kind::Perpendicular(seed, line), -2.),
        (Kind::RotatedSegment(line, a, 90.), -2.),
        (
            Kind::CalculatedAngleRotatedSegment(line, a, angle),
            ((0.5_f64).atan() + 30_f64.to_radians()).tan(),
        ),
        (Kind::ScaledSegment(line, a, -0.5), 0.5),
        (Kind::ReflectedSegment(line, mirror), -0.5),
        (
            Kind::PolarTranslatedSegment(line, 90., 37.79527559055118, 1.),
            0.5,
        ),
    ];
    for (kind, expected) in hosts {
        let host = doc.add(kind);
        let slope = doc.add(Kind::SlopeMeasurement(host, frame));
        doc.set_measurement_precision(&[slope], 5).unwrap();
        assert!((doc.slope_value(slope).unwrap() - expected).abs() < 1e-12);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.objects[slope - 1].kind,
            Kind::SlopeMeasurement(host, frame)
        );
        assert_eq!(reopened.objects[slope - 1].measurement_precision, Some(5));
    }
    let host = doc.add(Kind::Parallel(seed, line));
    let slope = doc.add(Kind::SlopeMeasurement(host, frame));
    let depth = doc
        .add_parameter_calculation(slope, "abs(v)*4", (10, 80))
        .unwrap();
    let image = doc.add(Kind::PolarTranslatedPoint(seed, 0., 37.79527559055118, 0.5));
    let iteration = doc
        .add_point_iteration(seed, image, IterationDepth::Value(depth))
        .unwrap();
    assert_eq!(doc.point_iteration_images(iteration).unwrap().len(), 2);
    assert!(doc.move_point(b, Point { x: 180., y: 100. }));
    assert_eq!(doc.numeric_value(depth), Some(4.));
    assert_eq!(doc.point_iteration_images(iteration).unwrap().len(), 4);
    assert!(doc.move_point(b, Point { x: 100., y: 100. }));
    assert_eq!(doc.numeric_value(slope), None);
    assert_eq!(doc.numeric_value(depth), None);
    assert_eq!(doc.point_iteration_images(iteration), None);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.numeric_value(slope), None);
    assert!(doc.move_point(b, Point { x: 180., y: 140. }));
    doc.selected.insert(slope);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    assert_eq!(
        doc.numeric_value(*pasted.last().unwrap()),
        doc.numeric_value(slope)
    );
    doc.selected.clear();
    doc.selected.insert(host);
    let removed = doc.clear_selected().unwrap();
    assert!(removed.contains(&slope) && removed.contains(&depth) && removed.contains(&iteration));
}

#[test]
fn native_point_values_project_independent_anchors_onto_supported_hosts() {
    for (source, point, measurement, expected, moved, after) in [
        (
            include_bytes!("../../../../../tests/fixtures/point_value_free_segment_native.gsp")
                .as_slice(),
            4,
            5,
            0.5,
            Point { x: 275., y: 150. },
            1.25,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/point_value_free_circle_native.gsp").as_slice(),
            4,
            5,
            197f64.atan2(-73.) / std::f64::consts::TAU,
            Point { x: 323., y: 17. },
            0.25,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/point_value_free_arc_native.gsp").as_slice(),
            5,
            6,
            0.077340094557163,
            Point { x: 323., y: 150. },
            0.0,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/point_value_free_arc_outside_native.gsp")
                .as_slice(),
            5,
            6,
            0.0,
            Point { x: 497., y: 311. },
            1.0,
        ),
    ] {
        let mut doc = Document::from_bytes(source, None).unwrap();
        assert!(!doc.partial_view);
        assert!(matches!(doc.objects[point - 1].kind, Kind::Free(_)));
        assert!(matches!(doc.value_equation_value(measurement),
            Some(EquationValue::PointValue(value)) if (value - expected).abs() < 1e-12));
        assert_eq!(doc.encode().unwrap(), source);
        assert!(doc.move_point(point, moved));
        assert!(matches!(doc.value_equation_value(measurement),
            Some(EquationValue::PointValue(value)) if (value - after).abs() < 1e-12));
        let edited = doc.encode().unwrap();
        let reopened = Document::from_bytes(&edited, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.encode().unwrap(), edited);
        assert_eq!(
            reopened.value_equation_value(measurement),
            doc.value_equation_value(measurement)
        );
        doc.selected.insert(measurement);
        let clipboard = doc.copy_selected().unwrap();
        let pasted = doc.paste(&clipboard).unwrap();
        let copy = *pasted.last().unwrap();
        assert_eq!(
            doc.value_equation_value(copy),
            doc.value_equation_value(measurement)
        );
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(
            reopened.value_equation_value(copy),
            doc.value_equation_value(measurement)
        );
    }
}

#[test]
fn corpus_named_point_values_import_matching_and_independent_points() {
    for (source, measurement, first, host, caption) in [
        (
            include_bytes!("../../../../../tests/fixtures/refraction_labeled_point_value_original.gsp")
                .as_slice(),
            18,
            17,
            16,
            "折射率n",
        ),
        (
            include_bytes!("../../../../../tests/fixtures/ningde_labeled_point_value_original.gsp")
                .as_slice(),
            20,
            10,
            18,
            "M",
        ),
    ] {
        let doc = Document::from_bytes(source, None).unwrap();
        assert_eq!(
            doc.objects[measurement - 1].kind,
            Kind::ValueEquation {
                kind: ValueEquationKind::PointValue,
                first,
                second: host,
            },
            "Anchor {:?}, host {:?}",
            doc.objects[first - 1].kind,
            doc.objects[host - 1].kind
        );
        assert_eq!(
            doc.objects[measurement - 1]
                .label
                .as_ref()
                .unwrap()
                .name_utf8()
                .unwrap(),
            caption
        );
        assert!(
            matches!(doc.value_equation_value(measurement), Some(EquationValue::PointValue(value)) if value.is_finite())
        );
        if measurement == 20 {
            assert!(
                matches!(doc.value_equation_value(measurement), Some(EquationValue::PointValue(value)) if (0.345..0.355).contains(&value)),
                "Native Sketchpad caption M = 0.35"
            );
        } else {
            assert!(
                matches!(doc.value_equation_value(measurement), Some(EquationValue::PointValue(value)) if (value - 1.640416666666667).abs() < 1e-12)
            );
        }
        assert_eq!(doc.encode().unwrap(), source);
    }
}

#[test]
fn corpus_point_value_empty_attribute_preserves_dependencies_and_bytes() {
    let bytes = include_bytes!("../../../../../tests/fixtures/point_value_empty_attribute_corpus.gsp");
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert_eq!(
        doc.objects[12].kind,
        Kind::ValueEquation {
            kind: ValueEquationKind::PointValue,
            first: 12,
            second: 7
        }
    );
    assert!(doc.numeric_value(13).unwrap().is_finite());
    assert!(doc.numeric_value(14).unwrap().is_finite());
    assert_eq!(doc.encode().unwrap(), bytes);
}

#[test]
fn point_value_empty_attribute_survives_live_edit_save_and_undo() {
    let mut bytes =
        include_bytes!("../../../../../tests/fixtures/point_value_free_segment_native.gsp").to_vec();
    let rs = records(&bytes).unwrap();
    let end = rs
        .iter()
        .rfind(|record| record.kind == 0x07d7)
        .unwrap()
        .start;
    bytes.splice(end..end, [0, 0, 0, 0, 0x2e, 0x23, 0, 0]);
    let mut doc = Document::from_bytes(&bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.numeric_value(5), Some(0.5));
    assert_eq!(doc.encode().unwrap(), bytes);
    doc.checkpoint();
    assert!(doc.move_point(4, Point { x: 275., y: 150. }));
    assert_eq!(doc.numeric_value(5), Some(1.25));
    doc.verify_roundtrip().unwrap();
    let saved = doc.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.numeric_value(5), Some(1.25));
    assert_eq!(reopened.encode().unwrap(), saved);
    doc.undo();
    assert_eq!(doc.encode().unwrap(), bytes);
    doc.redo();
    assert_eq!(doc.encode().unwrap(), saved);
}
