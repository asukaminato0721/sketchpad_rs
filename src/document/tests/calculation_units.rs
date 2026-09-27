use super::*;

#[test]
fn dimensional_length_expressions_update_units_and_marked_distance() {
    let bytes = include_bytes!("../../../tests/fixtures/calculation_dimensions_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    let length = 100. / 37.79527559055118;
    for (id, value, unit) in [
        (5, 2. * length, NumericUnit::Length),
        (6, length * length, NumericUnit::Area),
        (7, length, NumericUnit::Length),
        (8, length, NumericUnit::Length),
        (9, 1., NumericUnit::Scalar),
    ] {
        assert_eq!(doc.objects[id - 1].kind.numeric_unit(), Some(unit));
        assert!((doc.numeric_value(id).unwrap() - value).abs() < 1e-10);
    }
    let threshold = doc.add_parameter_calculation(4, "v+1", (10, 250)).unwrap();
    assert_eq!(doc.objects[threshold - 1].kind.numeric_unit(), Some(NumericUnit::Scalar));
    assert_eq!(doc.numeric_value(threshold), Some(length + 1.));
    let mut mixed=doc.clone();
    let result=mixed.add_parameter_calculation(4, "v+1°", (10, 250)).unwrap();
    assert_eq!(mixed.objects[result-1].kind.numeric_unit(),Some(NumericUnit::Scalar));
    assert_eq!(mixed.numeric_value(result),Some(length+1.));
    let cube = doc.add_parameter_calculation(4, "v^3", (10, 250)).unwrap();
    assert_eq!(
        doc.objects[cube - 1].kind.numeric_unit(),
        Some(NumericUnit::Compound {
            length: 3,
            degrees: 0
        })
    );
    let mut typed=doc.clone();
    let result=typed.add_parameter_calculation(4,"v^2°",(10,250)).unwrap();
    assert_eq!(typed.objects[result-1].kind.numeric_unit(),Some(NumericUnit::Scalar));
    assert_eq!(typed.numeric_value(result),Some(length.powi(2)));
    let identity = doc.add_parameter_calculation(4, "v^1", (10, 250)).unwrap();
    assert_eq!(
        doc.objects[identity - 1].kind.numeric_unit(),
        Some(NumericUnit::Length)
    );
    let zero_power = doc.add_parameter_calculation(4, "v^0", (10, 283)).unwrap();
    assert_eq!(doc.unitless_numeric_value(zero_power), Some(1.));
    assert_eq!(doc.position(10), Some(Point { x: 300., y: 180. }));
    doc.move_point(2, Point { x: 250., y: 180. });
    assert_eq!(doc.position(10), Some(Point { x: 400., y: 180. }));
    let chain = doc
        .add_measured_length_calculation(5, 2, (10, 250))
        .unwrap();
    assert!((doc.measured_calculation_cm(chain).unwrap() - 600. / 37.79527559055118).abs() < 1e-10);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(10), doc.position(10));
    assert_eq!(
        reopened.measured_calculation_cm(chain),
        doc.measured_calculation_cm(chain)
    );
    doc.selected.insert(10);
    let pasted = doc.paste(&doc.copy_selected().unwrap()).unwrap();
    let copied = pasted
        .iter()
        .copied()
        .find(|id| {
            transform_is(&doc.objects[*id - 1].kind, NativeTransformFlavor::MarkedPolar, &[ElementType::Point])
        })
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(copied), Some(Point { x: 420., y: 200. }));
    let mut cleared = Document::from_bytes(bytes, None).unwrap();
    cleared.selected.insert(4);
    assert_eq!(
        cleared.clear_selected().unwrap(),
        vec![4, 5, 6, 7, 8, 9, 10]
    );
    assert!(
        !Document::from_bytes(&cleared.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn area_expressions_chain_and_reject_invalid_dimensions() {
    let bytes = include_bytes!("../../../tests/fixtures/calculation_areas_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    for (source, id, factor, unit) in [
        (5, 8, 0.5, NumericUnit::Area),
        (7, 9, 1.25, NumericUnit::Area),
        (8, 10, 1.5, NumericUnit::Area),
    ] {
        assert_eq!(doc.objects[id - 1].kind.numeric_unit(), Some(unit));
        assert!(
            (doc.numeric_value(id).unwrap() - doc.numeric_value(source).unwrap() * factor).abs()
                < 1e-10
        );
    }
    assert_eq!(
        doc.objects[10].kind.numeric_unit(),
        Some(NumericUnit::Scalar)
    );
    assert_eq!(doc.numeric_value(11), Some(1.));
    let fractional = doc
        .add_parameter_calculation(7, "v^0.5", (10, 230))
        .unwrap();
    assert_eq!(
        doc.objects[fractional - 1].kind.numeric_unit(),
        Some(NumericUnit::Scalar)
    );
    let squared = doc.add_parameter_calculation(7, "v*v", (10, 230)).unwrap();
    assert_eq!(
        doc.objects[squared - 1].kind.numeric_unit(),
        Some(NumericUnit::Compound {
            length: 4,
            degrees: 0
        })
    );
    doc.move_point(2, Point { x: 275., y: 160. });
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    for id in [8, 9, 10, 11] {
        assert_eq!(reopened.numeric_value(id), doc.numeric_value(id));
    }
}

#[test]
fn scalar_coordinate_slope_and_literal_calculations_roundtrip_and_update() {
    let bytes = include_bytes!("../../../tests/fixtures/calculation_coordinates_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    for (source, calculation, factor, add) in [
        (11, 15, 1., 1.),
        (12, 16, 2., 0.),
        (13, 17, 0.5, 0.),
        (14, 18, 1., 1.),
        (19, 20, 2., 0.),
    ] {
        assert!(
            (doc.parameter_calculation_value(calculation).unwrap()
                - (doc.numeric_value(source).unwrap() * factor + add))
                .abs()
                < 1e-10
        );
    }
    assert!(doc.add_parameter_calculation(10, "v+1", (10, 300)).is_err());
    assert_eq!(doc.parameter_calculation_value(20), Some(10.));
    doc.move_point(2, Point { x: 300., y: 180. });
    assert!((doc.parameter_calculation_value(18).unwrap() - 0.6).abs() < 1e-10);
    assert!(
        (doc.parameter_calculation_value(15).unwrap() - (101. / 37.79527559055118 + 1.)).abs()
            < 1e-10
    );
    assert!(
        (doc.parameter_calculation_value(16).unwrap() - (-110. / 37.79527559055118)).abs() < 1e-10
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    for id in [15, 16, 17, 18, 20] {
        assert_eq!(
            reopened.parameter_calculation_value(id),
            doc.parameter_calculation_value(id)
        );
    }
    doc.selected.insert(18);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let calc = pasted
        .iter()
        .copied()
        .find(|id| matches!(doc.objects[*id - 1].kind, Kind::ParameterCalculation(..)))
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.parameter_calculation_value(calc),
        doc.parameter_calculation_value(18)
    );
    let mut undefined = Document::from_bytes(bytes, None).unwrap();
    undefined.move_point(2, Point { x: 150., y: 180. });
    assert!(undefined.numeric_value(14).is_none());
    assert!(undefined.parameter_calculation_value(18).is_none());
}

#[test]
fn scalar_measurements_feed_calculations_and_live_iteration_depth() {
    let bytes = include_bytes!("../../../tests/fixtures/calculation_scalars_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(doc.parameter_calculation_value(8), Some(4.));
    assert_eq!(doc.parameter_calculation_value(9), Some(1.));
    assert_eq!(doc.parameter_calculation_value(14), Some(1.));
    assert_eq!(doc.segment_iteration_images(17).unwrap().len(), 4);
    doc.move_point(3, Point { x: 500., y: 180. });
    doc.move_point(6, Point { x: 150., y: 180. });
    assert_eq!(doc.parameter_calculation_value(8), Some(5.));
    assert_eq!(doc.parameter_calculation_value(9), Some(2.));
    assert_eq!(doc.parameter_calculation_value(14), Some(0.75));
    assert_eq!(doc.segment_iteration_images(17).unwrap().len(), 5);
    doc.selected.insert(17);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    assert_eq!(
        pasted
            .iter()
            .filter(|id| matches!(doc.objects[**id - 1].kind, Kind::IterationBinding(..)))
            .count(),
        2
    );
    let iteration = pasted
        .iter()
        .copied()
        .find(|id| matches!(doc.objects[*id - 1].kind, Kind::SegmentIteration(..)))
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.segment_iteration_images(iteration).unwrap().len(),
        5
    );
    let mut fractional = Document::from_bytes(bytes, None).unwrap();
    fractional.move_point(3, Point { x: 450., y: 180. });
    assert_eq!(fractional.parameter_calculation_value(8), Some(4.5));
    assert_eq!(fractional.segment_iteration_images(17).unwrap().len(), 4);
    let mut cleared = Document::from_bytes(bytes, None).unwrap();
    cleared.selected.insert(4);
    assert_eq!(cleared.clear_selected().unwrap(), vec![4, 8, 17, 18, 19]);
    assert!(
        !Document::from_bytes(&cleared.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn measured_angle_calculations_follow_geometry_and_remap_dependencies() {
    let bytes = include_bytes!("../../../tests/fixtures/calculation_angles_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    let anchors = [7, 8, 9, 10].map(|id| match &doc.objects[id - 1].kind {
        Kind::ParameterCalculation(_, g) => g.anchor().1,
        _ => panic!("expected calculation"),
    });
    assert!(anchors[0] >= 61);
    assert!(anchors[1] - anchors[0] >= 64);
    assert!(anchors[2] - anchors[1] >= 33);
    assert!(anchors[3] - anchors[2] >= 64);
    assert_eq!(doc.parameter_calculation_value(7), Some(45.));
    assert_eq!(doc.parameter_calculation_value(8), Some(100.));
    assert_eq!(doc.unitless_numeric_value(9), Some(1.));
    assert_eq!(
        doc.parameter_calculation_value(10),
        doc.arc_angle_degrees(6).map(|v| v / 2.)
    );
    let angle_magnitude = doc.add_parameter_calculation(4, "v+10", (10, 160)).unwrap();
    assert_eq!(doc.objects[angle_magnitude - 1].kind.numeric_unit(), Some(NumericUnit::Scalar));
    assert_eq!(doc.numeric_value(angle_magnitude), doc.numeric_value(4).map(|value| value + 10.));
    let squared = doc.add_parameter_calculation(6, "v*v", (10, 160)).unwrap();
    assert_eq!(
        doc.objects[squared - 1].kind.numeric_unit(),
        Some(NumericUnit::Compound {
            length: 0,
            degrees: 2
        })
    );
    doc.move_point(3, Point { x: 150., y: 110. });
    assert!((doc.parameter_calculation_value(7).unwrap() - 67.5).abs() < 1e-9);
    assert!((doc.parameter_calculation_value(8).unwrap() - 145.).abs() < 1e-9);
    assert!((doc.unitless_numeric_value(9).unwrap() - 1.5).abs() < 1e-9);
    let source = doc.position(1).unwrap();
    let center = doc.position(2).unwrap();
    assert!(
        doc.position(11)
            .unwrap()
            .distance(rotate_point_about(source, center, 67.5_f64.to_radians()).unwrap())
            < 1e-9
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.parameter_calculation_value(10),
        doc.parameter_calculation_value(10)
    );
    doc.selected.insert(11);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = pasted
        .iter()
        .copied()
        .find(|id| {
            transform_is(&doc.objects[*id - 1].kind, NativeTransformFlavor::NumericRotation, &[ElementType::Point])
        })
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.position(copied),
        doc.position(11).map(|p| Point {
            x: p.x + 20.,
            y: p.y + 20.
        })
    );
    let mut cleared = Document::from_bytes(bytes, None).unwrap();
    cleared.selected.insert(4);
    assert_eq!(cleared.clear_selected().unwrap(), vec![4, 7, 8, 9, 11]);
    assert!(
        !Document::from_bytes(&cleared.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
    let mut undefined = Document::from_bytes(bytes, None).unwrap();
    undefined.move_point(3, undefined.position(2).unwrap());
    assert!(undefined.parameter_calculation_value(7).is_none());
    assert!(undefined.position(11).is_none());
}

#[test]
fn degree_calculations_track_units_and_undefined_angles() {
    let bytes = include_bytes!("../../../tests/fixtures/regular_polygon_iteration_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(doc.unitless_numeric_value(5).is_none());
    let mut mixed=doc.clone();
    let result=mixed.add_parameter_calculation(1, "n+1°", (10, 100)).unwrap();
    assert_eq!(mixed.objects[result-1].kind.numeric_unit(),Some(NumericUnit::Scalar));
    assert_eq!(mixed.numeric_value(result),Some(doc.numeric_value(1).unwrap()+1.));
    let squared = doc
        .add_parameter_calculation(1, "360°*90°/n", (10, 100))
        .unwrap();
    assert_eq!(
        doc.objects[squared - 1].kind.numeric_unit(),
        Some(NumericUnit::Compound {
            length: 0,
            degrees: 2
        })
    );
    let half = doc.add_parameter_calculation(5, "v/2", (10, 100)).unwrap();
    assert_eq!(doc.parameter_calculation_value(half), Some(36.));
    assert!(doc.unitless_numeric_value(half).is_none());
    let ratio = doc
        .add_parameter_calculation(5, "v/90°", (10, 130))
        .unwrap();
    assert_eq!(doc.unitless_numeric_value(ratio), Some(0.8));
    assert!(
        doc.add_segment_iteration(4, 6, 7, IterationDepth::Value(5))
            .is_err()
    );
    doc.set_parameter_value(1, 0.).unwrap();
    assert!(doc.position(6).is_none());
    assert!(doc.segment_iteration_images(8).is_none());
}
