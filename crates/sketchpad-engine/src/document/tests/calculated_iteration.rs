use super::*;

#[test]
fn numeric_iteration_multiple_sources_map_all_seed_occurrences_and_remap() {
    for (bytes, iteration, expected) in [
        (
            include_bytes!("../../../../../tests/fixtures/numeric_iteration_multi_port.gsp")
                .as_slice(),
            7,
            vec!["5.00", "7.00", "9.00", "11.00"],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/numeric_iteration_multi_edited_port.gsp")
                .as_slice(),
            7,
            vec!["7.00", "10.00", "13.00", "16.00"],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/numeric_iteration_multi_chain_port.gsp")
                .as_slice(),
            7,
            vec!["5.00", "7.00", "9.00", "11.00"],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/numeric_iteration_multi_length_port.gsp")
                .as_slice(),
            11,
            vec!["3.00", "4.00", "5.00"],
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/numeric_iteration_multi_length_edited_port.gsp"
            )
            .as_slice(),
            11,
            vec!["3.00", "4.00", "5.00", "6.00"],
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        assert_eq!(
            doc.point_parameter_iteration_labels(iteration)
                .unwrap()
                .iter()
                .map(|(_, text)| text.as_str())
                .collect::<Vec<_>>(),
            expected
        );
    }
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/numeric_iteration_multi_port.gsp"),
        None,
    )
    .unwrap();
    assert!(!doc.partial_view);
    assert_eq!(
        doc.point_parameter_iteration_labels(7)
            .unwrap()
            .iter()
            .map(|(_, text)| text.as_str())
            .collect::<Vec<_>>(),
        ["5.00", "7.00", "9.00", "11.00"]
    );
    doc.set_parameter_value(4, 3.).unwrap();
    assert_eq!(
        doc.point_parameter_iteration_labels(7)
            .unwrap()
            .iter()
            .map(|(_, text)| text.as_str())
            .collect::<Vec<_>>(),
        ["7.00", "10.00", "13.00", "16.00"]
    );
    doc.selected.insert(7);
    let copied = doc.copy_selected().unwrap();
    let pasted = doc.paste(&copied).unwrap();
    let iteration = *pasted
        .iter()
        .find(|id| {
            matches!(
                doc.objects[**id - 1].kind,
                Kind::PointParameterIteration(..)
            )
        })
        .unwrap();
    assert_eq!(
        doc.point_parameter_iteration_labels(iteration)
            .unwrap()
            .iter()
            .map(|(_, text)| text.clone())
            .collect::<Vec<_>>(),
        ["7.00", "10.00", "13.00", "16.00"]
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        doc.point_parameter_iteration_labels(iteration),
        reopened.point_parameter_iteration_labels(iteration)
    );
    doc.selected.clear();
    doc.selected.insert(4);
    doc.clear_selected().unwrap();
    assert!(doc.objects[6].deleted);
    assert!(doc.point_parameter_iteration_labels(7).is_none());

    let mut repeated = Document::new();
    let seed = repeated.add(Kind::Free(Point { x: 100., y: 200. }));
    let image = repeated.add(Kind::PolarTranslatedPoint(seed, 0., 37.79527559055118, 1.));
    let parameter = repeated.add_named_parameter("x", 1., (10, 30)).unwrap();
    let formula = repeated
        .add_multi_calculation(&[parameter, parameter], "v1+v2", (10, 65))
        .unwrap();
    let label = repeated.add_point_numeric_label(image, formula).unwrap();
    let iteration = repeated
        .add_point_parameter_iteration(
            seed,
            parameter,
            image,
            formula,
            label,
            IterationDepth::Fixed(3),
        )
        .unwrap();
    assert_eq!(
        repeated
            .point_parameter_iteration_labels(iteration)
            .unwrap()
            .iter()
            .map(|(_, text)| text.as_str())
            .collect::<Vec<_>>(),
        ["4.00", "8.00", "16.00"]
    );
    repeated.set_parameter_value(parameter, 2.).unwrap();
    assert_eq!(
        repeated
            .point_parameter_iteration_labels(iteration)
            .unwrap()
            .iter()
            .map(|(_, text)| text.as_str())
            .collect::<Vec<_>>(),
        ["8.00", "16.00", "32.00"]
    );
    let mut undefined = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/numeric_iteration_multi_length_port.gsp"),
        None,
    )
    .unwrap();
    undefined.move_point(5, undefined.position(4).unwrap());
    assert!(undefined.point_parameter_iteration_labels(11).is_none());
    undefined.move_point(
        5,
        Point {
            x: 175.59055118110236,
            y: 160.,
        },
    );
    assert_eq!(
        undefined
            .point_parameter_iteration_labels(11)
            .unwrap()
            .len(),
        3
    );
}

#[test]
fn multiple_source_native_iterations_keep_external_angle_and_update_depth() {
    for (bytes, depth, angle) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/calculation_multi_angle_iteration_port.gsp"
            )
            .as_slice(),
            2,
            45_f64,
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/calculation_multi_angle_iteration_edited_port.gsp"
            )
            .as_slice(),
            3,
            30_f64,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        assert!((doc.numeric_value(6).unwrap() - angle).abs() < 1e-12);
        let lines = doc.segment_iteration_images(10).unwrap();
        assert_eq!(lines.len(), depth);
        let center = doc.position(2).unwrap();
        let seed = doc.position(3).unwrap();
        // The measured angle includes the seed, but native Sketchpad holds
        // that external formula fixed for all steps in this iteration.
        for (step, (start, end)) in lines.iter().enumerate() {
            let expected_start =
                rotate_point_about(seed, center, angle.to_radians() * (step + 1) as f64).unwrap();
            let expected_end =
                rotate_point_about(seed, center, angle.to_radians() * (step + 2) as f64).unwrap();
            assert!(start.distance(expected_start) < 1e-10);
            assert!(end.distance(expected_end) < 1e-10);
        }
    }
    for (bytes, depth) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/calculation_multi_lengths_iteration_port.gsp"
            )
            .as_slice(),
            2,
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/calculation_multi_lengths_iteration_edited_port.gsp"
            )
            .as_slice(),
            3,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        assert_eq!(doc.segment_iteration_images(15).unwrap().len(), depth);
    }
}

#[test]
fn source_free_calculations_native_import_rotation_depth_and_copy() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/calculation_constants_only_native_verified.gsp"
    );
    let original = Document::from_bytes(bytes, None).unwrap();
    assert!(!original.partial_view);
    assert_eq!(original.encode().unwrap(), bytes);
    let mut doc = Document::new();
    let angle = doc
        .add_calculation_expression("asin(.5)", (10, 30))
        .unwrap();
    let depth = doc
        .add_calculation_expression("sqrt(25)", (10, 65))
        .unwrap();
    let center = doc.add(Kind::Free(Point { x: 250., y: 180. }));
    let seed = doc.add(Kind::Free(Point { x: 310., y: 180. }));
    let image = doc
        .add_calculated_angle_rotation(seed, center, angle)
        .unwrap();
    let segment = doc.add(Kind::Segment(seed, image));
    assert!(
        doc.add_segment_iteration(seed, image, segment, IterationDepth::Value(angle))
            .is_err()
    );
    let iteration = doc
        .add_segment_iteration(seed, image, segment, IterationDepth::Value(depth))
        .unwrap();
    assert_eq!(doc.segment_iteration_images(iteration).unwrap().len(), 5);
    let label = doc.add_point_numeric_label(seed, angle).unwrap();
    assert!(doc.point_numeric_label(label).unwrap().1.ends_with('°'));
    let chain = doc
        .add_parameter_calculation(angle, "v/2", (10, 100))
        .unwrap();
    assert!((doc.numeric_value(chain).unwrap() - 15.).abs() < 1e-12);
    doc.selected.insert(iteration);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = *pasted
        .iter()
        .find(|id| matches!(doc.objects[**id - 1].kind, Kind::SegmentIteration(..)))
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.segment_iteration_images(copied).unwrap().len(), 5);
    let undefined = doc
        .add_calculation_expression("sqrt(-1)", (10, 135))
        .unwrap();
    assert!(doc.numeric_value(undefined).is_none());
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn inverse_angle_calculation_drives_rotation_and_iteration() {
    let mut doc = Document::new();
    let n = doc.add_named_parameter("n", 0.5, (10, 30)).unwrap();
    let angle = doc
        .add_parameter_calculation(n, "asin(n)", (10, 60))
        .unwrap();
    let depth = doc
        .add_parameter_calculation(n, "round(abs(n)*10)", (10, 90))
        .unwrap();
    let center = doc.add(Kind::Free(Point { x: 250., y: 180. }));
    let seed = doc.add(Kind::Free(Point { x: 310., y: 180. }));
    let image = doc
        .add_calculated_angle_rotation(seed, center, angle)
        .unwrap();
    let segment = doc.add(Kind::Segment(seed, image));
    let iteration = doc
        .add_segment_iteration(seed, image, segment, IterationDepth::Value(depth))
        .unwrap();
    for value in [0.5, -0.5, 0.8] {
        doc.set_parameter_value(n, value).unwrap();
        let degrees = value.asin() / CALCULATOR_PI * 180.;
        assert!((doc.numeric_value(angle).unwrap() - degrees).abs() < 1e-12);
        let expected = rotate_point_about(
            doc.position(seed).unwrap(),
            doc.position(center).unwrap(),
            degrees.to_radians(),
        )
        .unwrap();
        assert!(doc.position(image).unwrap().distance(expected) < 1e-10);
        assert_eq!(
            doc.segment_iteration_images(iteration).unwrap().len(),
            (value.abs() * 10.).round() as usize
        );
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.segment_iteration_images(iteration),
            doc.segment_iteration_images(iteration)
        );
    }
    doc = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    doc.set_parameter_value(n, 1.2).unwrap();
    assert!(doc.numeric_value(angle).is_none());
    assert!(doc.position(image).is_none());
    assert!(doc.segment_iteration_images(iteration).is_none());
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
    doc.set_parameter_value(n, 0.5).unwrap();
    doc.selected.insert(iteration);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = *pasted
        .iter()
        .find(|id| matches!(doc.objects[**id - 1].kind, Kind::SegmentIteration(..)))
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.segment_iteration_images(copied).unwrap().len(), 5);
}

#[test]
fn built_in_calculations_native_import_edit_and_dependency_remapping() {
    let native =
        include_bytes!("../../../../../tests/fixtures/calculation_sin_scalar_native_verified.gsp");
    let mut doc = Document::from_bytes(native, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), native);
    assert!((doc.numeric_value(2).unwrap() - 0.5).abs() < 1e-12);
    doc.set_parameter_value(1, 60.).unwrap();
    assert!((doc.numeric_value(2).unwrap() - 3_f64.sqrt() / 2.).abs() < 1e-12);
    let nested = doc
        .add_parameter_calculation(1, "sqrt(abs(n-64))", (10, 100))
        .unwrap();
    assert_eq!(doc.numeric_value(nested), Some(2.));
    assert!(
        doc.add_parameter_calculation(1, "ln(n-60)", (10, 130))
            .is_err()
    );
    let source = doc.add(Kind::Free(Point { x: 100., y: 160. }));
    let image = doc.add(Kind::FixedRectangularTranslatedPoint(
        source,
        37.79527559055118,
        0.5,
        37.79527559055118,
        0.,
    ));
    let iteration = doc
        .add_point_iteration(source, image, IterationDepth::Value(nested))
        .unwrap();
    assert_eq!(doc.point_iteration_images(iteration).unwrap().len(), 2);
    doc.selected.insert(iteration);
    let clipboard = doc.copy_selected().unwrap();
    let copied = doc.paste(&clipboard).unwrap();
    assert!(
        copied
            .iter()
            .any(|id| matches!(doc.objects[*id - 1].kind, Kind::ParameterCalculation(..)))
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.numeric_value(nested), Some(2.));
    doc.set_parameter_value(1, 62.).unwrap();
    assert_eq!(doc.point_iteration_images(iteration).unwrap().len(), 1);
    doc.set_parameter_value(1, 64.).unwrap();
    assert_eq!(doc.numeric_value(nested), Some(0.));
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn measured_angle_and_length_function_calculations_follow_geometry() {
    let mut doc = Document::new();
    let vertex = doc.add(Kind::Free(Point { x: 100., y: 160. }));
    let right = doc.add(Kind::Free(Point { x: 140., y: 160. }));
    let top = doc.add(Kind::Free(Point { x: 100., y: 120. }));
    let angle = doc.add(Kind::AngleMeasurement(right, vertex, top));
    let sine = doc
        .add_parameter_calculation(angle, "sin(v)", (10, 30))
        .unwrap();
    assert!((doc.numeric_value(sine).unwrap() - 1.).abs() < 1e-12);
    doc.move_point(top, Point { x: 140., y: 120. });
    assert!((doc.numeric_value(sine).unwrap() - 0.5_f64.sqrt()).abs() < 1e-12);
    let segment = doc.add(Kind::Segment(vertex, right));
    let length = doc.add(Kind::SegmentLength(segment));
    let absolute = doc
        .add_parameter_calculation(length, "abs(v-v*2)", (10, 60))
        .unwrap();
    assert_eq!(
        doc.objects[absolute - 1].kind.numeric_unit(),
        Some(NumericUnit::Length)
    );
    assert_eq!(doc.numeric_value(absolute), doc.numeric_value(length));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.numeric_value(absolute),
        doc.numeric_value(absolute)
    );
    doc.selected.insert(top);
    assert!(doc.clear_selected().unwrap().contains(&sine));
}
