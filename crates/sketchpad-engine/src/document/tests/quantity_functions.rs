use super::*;

#[test]
fn native_measurement_functions_import_update_and_keep_quantity_units() {
    let bytes = include_bytes!("../../../../../tests/fixtures/quantity_functions_native.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    for (x, angle_point) in [
        (220., Point { x: 160., y: 120. }),
        (250., Point { x: 220., y: 125. }),
    ] {
        doc.move_point(2, Point { x, y: 180. });
        doc.move_point(7, angle_point);
        for (source, first, unit) in [
            (4, 9, NumericUnit::Area),
            (5, 14, NumericUnit::Length),
            (8, 19, NumericUnit::Degrees),
        ] {
            let value = doc.numeric_value(source).unwrap();
            let expected = [
                value.ln(),
                value.log10(),
                1.,
                (value + 0.5).floor(),
                value.trunc(),
            ];
            for (offset, expected) in expected.into_iter().enumerate() {
                let id = first + offset;
                assert_eq!(
                    doc.objects[id - 1].kind.numeric_unit(),
                    Some(if offset < 3 {
                        NumericUnit::Scalar
                    } else {
                        unit
                    })
                );
                assert_eq!(doc.numeric_value(id), Some(expected));
                doc.set_numeric_precision(&[id], 5).unwrap();
            }
        }
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        for id in 9..=23 {
            assert_eq!(reopened.numeric_value(id), doc.numeric_value(id));
            assert_eq!(
                reopened.objects[id - 1].kind.numeric_unit(),
                doc.objects[id - 1].kind.numeric_unit()
            );
        }
    }
}

#[test]
fn rounded_quantities_drive_marked_distance_angle_and_live_depth() {
    let bytes = include_bytes!("../../../../../tests/fixtures/quantity_functions_iteration_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    for (x, distance, depth) in [(220., 0.2, 4), (250., 0.4, 5)] {
        doc.move_point(2, Point { x, y: 150. });
        assert_eq!(doc.numeric_value(8), Some(distance));
        let points = doc.point_iteration_images(12).unwrap();
        assert_eq!(points.len(), depth);
        for (i, point) in points.iter().enumerate() {
            assert!(
                (point.x - (40. + distance * 37.79527559055118 * (i + 2) as f64)).abs() < 1e-10
            );
            assert_eq!(point.y, 230.);
        }
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.point_iteration_images(12), Some(points));
    }
    doc.selected.insert(12);
    let pasted = doc.paste(&doc.copy_selected().unwrap()).unwrap();
    let copy = pasted
        .into_iter()
        .find(|id| matches!(doc.objects[*id - 1].kind, Kind::PointIteration(..)))
        .unwrap();
    assert_eq!(doc.point_iteration_images(copy).unwrap().len(), 5);
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );

    let mut angle = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/quantity_functions_angle_port.gsp"),
        None,
    )
    .unwrap();
    assert!(!angle.partial_view);
    angle.move_point(7, Point { x: 220., y: 125. });
    assert_eq!(angle.numeric_value(22), Some(43.));
    assert_eq!(angle.numeric_value(23), Some(42.));
    let point = angle.position(24).unwrap();
    assert!((point.x - (160. + 60. * 43_f64.to_radians().cos())).abs() < 1e-10);
    assert!((point.y - (180. - 60. * 43_f64.to_radians().sin())).abs() < 1e-10);
    assert_eq!(angle.point_iteration_images(25).unwrap().len(), 1);
    let reopened = Document::from_bytes(&angle.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.point_iteration_images(25),
        angle.point_iteration_images(25)
    );
}

#[test]
fn quantity_log_domains_suppress_depth_and_recover_typed_distances() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/quantity_functions_native.gsp"),
        None,
    )
    .unwrap();
    doc.set_parameter_value(6, 1.).unwrap();
    let scaled = doc
        .add_multi_calculation(&[5, 6], "v1*v2", (10, 5))
        .unwrap();
    let rounded = doc
        .add_parameter_calculation(scaled, "round(v)", (10, 38))
        .unwrap();
    let logarithm = doc
        .add_parameter_calculation(scaled, "ln(v)", (10, 71))
        .unwrap();
    let depth = doc
        .add_parameter_calculation(logarithm, "v*8", (10, 104))
        .unwrap();
    let image = doc.add(Kind::MarkedDistanceTranslatedPoint(2, rounded, 0.));
    let iteration = doc
        .add_point_iteration(2, image, IterationDepth::Value(depth))
        .unwrap();
    assert_eq!(doc.point_iteration_images(iteration).unwrap().len(), 3);
    for parameter in [-1., 0.] {
        doc.set_parameter_value(6, parameter).unwrap();
        assert_eq!(doc.numeric_value(logarithm), None);
        assert_eq!(doc.point_iteration_images(iteration), None);
        assert_eq!(
            doc.objects[rounded - 1].kind.numeric_unit(),
            Some(NumericUnit::Length)
        );
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.numeric_value(logarithm), None);
        assert_eq!(reopened.position(image), doc.position(image));
    }
    doc.set_parameter_value(6, 2.).unwrap();
    assert_eq!(doc.point_iteration_images(iteration).unwrap().len(), 9);
    assert_eq!(doc.numeric_value(rounded), Some(3.));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.point_iteration_images(iteration),
        doc.point_iteration_images(iteration)
    );
}

#[test]
fn native_source_exponents_discard_units_and_follow_parameter_edits() {
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/source_exponent_area_native.gsp").as_slice(),
        include_bytes!("../../../../../tests/fixtures/source_exponent_quantities_native.gsp").as_slice(),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        let cases = if doc.objects.len() == 7 {
            vec![(7, 4)]
        } else {
            vec![(9, 4), (10, 5), (11, 8)]
        };
        for power in [1., 2., 0.5, 0., -1.] {
            doc.set_parameter_value(6, power).unwrap();
            for &(id, source) in &cases {
                assert_eq!(
                    doc.objects[id - 1].kind.numeric_unit(),
                    Some(NumericUnit::Scalar)
                );
                assert_eq!(
                    doc.unitless_numeric_value(id),
                    doc.numeric_value(source).map(|v| v.powf(power))
                );
            }
            if doc.objects.len() == 12 {
                let area = doc.numeric_value(4).unwrap();
                assert_eq!(doc.unitless_numeric_value(12), Some(area * area));
            }
            let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
            assert!(!reopened.partial_view);
            for &(id, _) in &cases {
                assert_eq!(
                    reopened.unitless_numeric_value(id),
                    doc.unitless_numeric_value(id)
                );
            }
        }
    }
}

#[test]
fn native_exponent_syntax_controls_units_without_changing_values() {
    let bytes = include_bytes!("../../../../../tests/fixtures/dimensional_exponent_forms_native.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    for x in [220., 250.] {
        doc.move_point(2, Point { x, y: 180. });
        let area = doc.numeric_value(4).unwrap();
        for (id, exponent, unit) in [
            (9, -2, NumericUnit::Scalar),
            (
                10,
                2,
                NumericUnit::Compound {
                    length: 4,
                    degrees: 0,
                },
            ),
            (11, 2, NumericUnit::Scalar),
            (
                12,
                2,
                NumericUnit::Compound {
                    length: 4,
                    degrees: 0,
                },
            ),
            (
                13,
                2,
                NumericUnit::Compound {
                    length: 4,
                    degrees: 0,
                },
            ),
        ] {
            assert_eq!(doc.objects[id - 1].kind.numeric_unit(), Some(unit));
            assert_eq!(doc.numeric_value(id), Some(area.powf(exponent as f64)));
        }
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        for id in 9..=13 {
            assert_eq!(
                reopened.objects[id - 1].kind.numeric_unit(),
                doc.objects[id - 1].kind.numeric_unit()
            );
            assert_eq!(reopened.numeric_value(id), doc.numeric_value(id));
        }
    }
}

#[test]
fn source_exponent_units_keep_live_depth_and_numeric_iteration_scalar() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/source_exponents_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert!(!doc.partial_view);
    for (power, count) in [(0.5, 1), (2., 3), (0., 1), (-1., 0)] {
        doc.set_parameter_value(5, power).unwrap();
        assert_eq!(
            doc.objects[6].kind.numeric_unit(),
            Some(NumericUnit::Scalar)
        );
        assert_eq!(
            doc.point_iteration_images(9).unwrap_or_default().len(),
            count
        );
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.point_iteration_images(9),
            doc.point_iteration_images(9)
        );
    }
    doc.set_parameter_value(5, 2.).unwrap();
    doc.selected.insert(9);
    let pasted = doc.paste(&doc.copy_selected().unwrap()).unwrap();
    let copy = pasted
        .into_iter()
        .find(|id| matches!(doc.objects[*id - 1].kind, Kind::PointIteration(..)))
        .unwrap();
    assert_eq!(doc.point_iteration_images(copy).unwrap().len(), 3);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.point_iteration_images(copy),
        doc.point_iteration_images(copy)
    );

    let mut numeric = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/source_exponents_numeric_iteration_port.gsp"),
        None,
    )
    .unwrap();
    for initial in [0.5, 0.7, 1.] {
        numeric.set_parameter_value(5, initial).unwrap();
        let base = numeric.numeric_value(4).unwrap();
        let mut value = base.powf(initial);
        let labels = numeric.point_parameter_iteration_labels(10).unwrap();
        assert_eq!(labels.len(), 3);
        for (_, text) in &labels {
            value = base.powf(value);
            assert_eq!(text, &format!("{value:.0}"));
        }
        let reopened = Document::from_bytes(&numeric.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.point_parameter_iteration_labels(10), Some(labels));
    }
}

#[test]
fn dimensional_source_exponent_domain_errors_recover_without_losing_units() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/source_exponents_iteration_edited_port.gsp"),
        None,
    )
    .unwrap();
    let negative = doc.add_parameter_calculation(6, "-v", (10, 100)).unwrap();
    let power = doc
        .add_multi_calculation(&[negative, 5], "v1^v2", (10, 130))
        .unwrap();
    let iteration = doc
        .add_point_iteration(1, 8, IterationDepth::Value(power))
        .unwrap();
    assert!(doc.unitless_numeric_value(power).unwrap() > 1.);
    doc.set_parameter_value(5, 0.5).unwrap();
    assert_eq!(doc.numeric_value(power), None);
    assert_eq!(doc.point_iteration_images(iteration), None);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.numeric_value(power), None);
    doc.set_parameter_value(5, 2.).unwrap();
    assert!(doc.unitless_numeric_value(power).unwrap() > 1.);
    assert!(doc.point_iteration_images(iteration).is_some());
}

#[test]
fn native_dimensional_roots_and_generic_quantity_headers_roundtrip() {
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/area_square_root_native.gsp").as_slice(),
        include_bytes!("../../../../../tests/fixtures/area_fractional_power_native.gsp").as_slice(),
        include_bytes!("../../../../../tests/fixtures/dimensional_roots_powers_native.gsp").as_slice(),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        let area = doc.numeric_value(4).unwrap();
        let cases = if doc.objects.len() == 5 {
            let unit =
                if bytes == include_bytes!("../../../../../tests/fixtures/area_square_root_native.gsp") {
                    NumericUnit::Length
                } else {
                    NumericUnit::Scalar
                };
            vec![(5, area.sqrt(), unit)]
        } else {
            vec![
                (8, area, NumericUnit::Area),
                (9, doc.numeric_value(5).unwrap().powi(2), NumericUnit::Area),
                (
                    10,
                    doc.numeric_value(5).unwrap().sqrt(),
                    NumericUnit::Scalar,
                ),
                (11, 90_f64.sqrt(), NumericUnit::Scalar),
                (
                    12,
                    8100.,
                    NumericUnit::Compound {
                        length: 0,
                        degrees: 2,
                    },
                ),
            ]
        };
        for (id, value, unit) in cases {
            assert_eq!(doc.objects[id - 1].kind.numeric_unit(), Some(unit));
            assert!((doc.numeric_value(id).unwrap() - value).abs() < 1e-10);
            doc.set_numeric_precision(&[id], 5).unwrap();
        }
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        for object in &doc.objects {
            assert_eq!(
                reopened.objects[object.id - 1].kind.numeric_unit(),
                object.kind.numeric_unit()
            );
            assert_eq!(
                reopened.numeric_value(object.id),
                doc.numeric_value(object.id)
            );
        }
    }
}

#[test]
fn dimensional_root_chains_follow_edits_and_undefined_sources() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/area_square_root_native.gsp"),
        None,
    )
    .unwrap();
    let squared = doc.add_parameter_calculation(4, "v^2", (10, 100)).unwrap();
    let root = doc
        .add_parameter_calculation(squared, "sqrt(v)", (10, 130))
        .unwrap();
    let length = doc
        .add_parameter_calculation(root, "sqrt(v)", (10, 160))
        .unwrap();
    let scalar = doc
        .add_parameter_calculation(4, "v^0.5", (10, 190))
        .unwrap();
    assert_eq!(
        doc.objects[squared - 1].kind.numeric_unit(),
        Some(NumericUnit::Compound {
            length: 4,
            degrees: 0
        })
    );
    assert_eq!(
        doc.objects[root - 1].kind.numeric_unit(),
        Some(NumericUnit::Area)
    );
    assert_eq!(
        doc.objects[length - 1].kind.numeric_unit(),
        Some(NumericUnit::Length)
    );
    assert!(doc.unitless_numeric_value(length).is_none());
    for x in [250., 220.] {
        doc.move_point(2, Point { x, y: 180. });
        let area = doc.numeric_value(4);
        assert_eq!(doc.numeric_value(squared), area.map(|v| v * v));
        assert_eq!(doc.numeric_value(root), area);
        assert_eq!(doc.numeric_value(length), area.map(f64::sqrt));
        assert_eq!(doc.unitless_numeric_value(scalar), area.map(f64::sqrt));
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.numeric_value(length), doc.numeric_value(length));
    }
    doc.selected.insert(4);
    assert_eq!(
        doc.clear_selected().unwrap(),
        vec![4, 5, squared, root, length, scalar]
    );
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn higher_unit_calculations_preserve_units_on_copy_and_page_copy() {
    for (bytes, ids) in [
        (
            include_bytes!("../../../../../tests/fixtures/dimensional_roots_chain_port.gsp").as_slice(),
            vec![8, 9, 10],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/dimensional_roots_units_port.gsp").as_slice(),
            vec![7, 8, 9],
        ),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        for id in ids {
            let unit = doc.objects[id - 1].kind.numeric_unit();
            let value = doc.numeric_value(id);
            doc.set_numeric_precision(&[id], 5).unwrap();
            doc.selected.clear();
            doc.selected.insert(id);
            let pasted = doc.paste(&doc.copy_selected().unwrap()).unwrap();
            let copy = *pasted.last().unwrap();
            assert_eq!(doc.objects[copy - 1].kind.numeric_unit(), unit);
            assert_eq!(doc.numeric_value(copy), value);
        }
        doc.copy_page(0).unwrap();
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        // Multi-page documents retain the existing partial-view guard.
        assert!(reopened.partial_view);
        for object in &doc.objects {
            assert_eq!(
                reopened.objects[object.id - 1].kind.numeric_unit(),
                object.kind.numeric_unit()
            );
            assert_eq!(
                reopened.numeric_value(object.id),
                doc.numeric_value(object.id)
            );
        }
    }
}

#[test]
fn dimensional_roots_supply_marked_distance_and_live_iteration_depth() {
    let bytes = include_bytes!("../../../../../tests/fixtures/dimensional_roots_iteration_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    for (x, count) in [(120., 1), (140., 2), (100., 0), (120., 1)] {
        doc.move_point(2, Point { x, y: 220. });
        let points = doc.point_iteration_images(8).unwrap_or_default();
        assert_eq!(points.len(), count);
        if count > 0 {
            let distance = (x - 80.) * std::f64::consts::PI.sqrt();
            for (i, point) in points.iter().enumerate() {
                assert!((point.x - (80. + distance * (i + 2) as f64)).abs() < 1e-10);
                assert_eq!(point.y, 220.);
            }
        }
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.point_iteration_images(8),
            doc.point_iteration_images(8)
        );
    }
    doc.set_numeric_precision(&[5, 6], 0).unwrap();
    assert_eq!(doc.point_iteration_images(8).unwrap().len(), 1);
    doc.selected.insert(8);
    let pasted = doc.paste(&doc.copy_selected().unwrap()).unwrap();
    let copy = pasted
        .into_iter()
        .find(|id| matches!(doc.objects[*id - 1].kind, Kind::PointIteration(..)))
        .unwrap();
    assert_eq!(doc.point_iteration_images(copy).unwrap().len(), 1);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.point_iteration_images(copy),
        doc.point_iteration_images(copy)
    );
    let mut cleared = Document::from_bytes(bytes, None).unwrap();
    cleared.selected.insert(4);
    assert_eq!(cleared.clear_selected().unwrap(), vec![4, 5, 6, 7, 8, 9]);
}
