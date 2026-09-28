use super::*;

fn measured_iteration(length: bool, edited: bool) -> (Document, usize, usize) {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point {
        x: 160.,
        y: if edited { 160. } else { 200. },
    }));
    let p = doc.add(Kind::Free(Point { x: 200., y: 280. }));
    let origin = doc.add(Kind::Free(Point { x: 80., y: 280. }));
    let reference = doc.add(Kind::Free(Point { x: 160., y: 280. }));
    let vector_start = doc.add(Kind::Free(Point { x: 40., y: 400. }));
    let vector_end = doc.add(Kind::Free(Point { x: 120., y: 400. }));
    let host = doc.add(Kind::Segment(origin, a));
    let measurement = doc.add(if length {
        Kind::SegmentLength(host)
    } else {
        Kind::AngleMeasurement(a, origin, reference)
    });
    let ratio = if length {
        Some(
            doc.add_parameter_calculation(measurement, "v/(4 cm)", (10, 100))
                .unwrap(),
        )
    } else {
        None
    };
    let image = doc.add(Kind::VectorTranslatedPoint(a, vector_start, vector_end));
    let segment = doc.add(Kind::Segment(origin, p));
    let transformed = if let Some(ratio) = ratio {
        doc.add_calculated_ratio_dilation(segment, origin, ratio)
            .unwrap()
    } else {
        doc.add_calculated_angle_rotation(segment, origin, measurement)
            .unwrap()
    };
    let mut carried = vec![host, measurement];
    carried.extend(ratio);
    carried.extend([segment, transformed, image]);
    let parent = doc
        .add_mapped_point_iteration_with_bindings(
            &[a, p],
            &[vec![image, p]],
            &carried,
            &[transformed],
            IterationDepth::Fixed(3),
        )
        .unwrap();
    (doc, parent, measurement)
}

#[test]
fn mapped_measurements_recompute_after_each_simultaneous_state_and_reconstruct_native_writers() {
    for (length, edited, native) in [
        (
            false,
            false,
            include_bytes!(
                "../../../../../tests/fixtures/measurement_mapped44_angle_native_verified.gsp"
            )
            .as_slice(),
        ),
        (
            true,
            false,
            include_bytes!(
                "../../../../../tests/fixtures/measurement_mapped44_length_native_verified.gsp"
            )
            .as_slice(),
        ),
        (
            false,
            true,
            include_bytes!(
                "../../../../../tests/fixtures/measurement_mapped44_angle_edited_native_verified.gsp"
            )
            .as_slice(),
        ),
    ] {
        let (doc, parent, measurement) = measured_iteration(length, edited);
        assert_eq!(doc.encode().unwrap().as_slice(), native);
        let original = doc.numeric_value(measurement);
        let images = doc.branch_segment_iteration_images(parent + 1).unwrap();
        assert_eq!(images.len(), 3);
        for (generation, (a, b)) in images.iter().enumerate() {
            assert_eq!(*a, Point { x: 80., y: 280. });
            let dx = 160. + 80. * generation as f64;
            let dy = if edited { 120. } else { 80. };
            let expected = if length {
                Point {
                    x: 80. + 120. * dx.hypot(dy) / (4. * 37.79527559055118),
                    y: 280.,
                }
            } else {
                Point {
                    x: 80. + 120. * dx / dx.hypot(dy),
                    y: 280. - 120. * dy / dx.hypot(dy),
                }
            };
            assert!(
                b.distance(expected) < 1e-9,
                "{generation}: {b:?}, {expected:?}"
            );
        }
        // Sampling must retain the source measurement and its upstream graph.
        assert_eq!(doc.numeric_value(measurement), original);
        assert!(matches!(doc.objects[0].kind, Kind::Free(_)));
        doc.verify_roundtrip().unwrap();
        let reopened = Document::from_bytes(native, None).unwrap();
        assert_eq!(
            reopened.branch_segment_iteration_images(parent + 1),
            Some(images)
        );
    }
}

#[test]
fn mapped_measured_angle_source_edit_updates_all_images_and_has_structural_undo() {
    let (mut doc, parent, _) = measured_iteration(false, false);
    let original = doc.encode().unwrap();
    let images = doc.branch_segment_iteration_images(parent + 1).unwrap();
    doc.checkpoint();
    doc.move_point(1, Point { x: 160., y: 160. });
    let changed = doc.branch_segment_iteration_images(parent + 1).unwrap();
    assert_ne!(changed, images);
    assert_eq!(
        doc.encode().unwrap().as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/measurement_mapped44_angle_edited_native_verified.gsp"
        )
    );
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
    assert_eq!(
        doc.branch_segment_iteration_images(parent + 1),
        Some(images)
    );
    doc.redo();
    assert_eq!(
        doc.branch_segment_iteration_images(parent + 1),
        Some(changed)
    );
    doc.set_mapped_iteration_final_only(parent, true).unwrap();
    assert_eq!(
        doc.branch_segment_iteration_images(parent + 1)
            .unwrap()
            .len(),
        1
    );
    doc.verify_roundtrip().unwrap();
}

#[test]
fn mapped_arc_corpus_keeps_native_path_dependencies_and_visible_segment_generations() {
    let bytes = include_bytes!("../../../../../tests/fixtures/mapped_arc44_corpus_native_verified.gsp");
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert!(
        matches!(&doc.objects[48].kind,Kind::MappedPointIteration{seeds,branches,..}
        if seeds==&[2,1,12] && branches==&[vec![35,34,24]])
    );
    let images = doc.branch_segment_iteration_images(50).unwrap();
    assert_eq!(images.len(), 13);
    // Independent Native capture measurements, after subtracting its window
    // offset. Raster coordinates have a two-pixel tolerance.
    for (at, (first, last)) in [
        (0, (Point { x: 341., y: 293. }, Point { x: 404., y: 222. })),
        (5, (Point { x: 276., y: 460. }, Point { x: 276., y: 555. })),
        (12, (Point { x: 299., y: 272. }, Point { x: 323., y: 179. })),
    ] {
        assert!(images[at].0.distance(first) < 2.);
        assert!(images[at].1.distance(last) < 2.);
    }
    assert!(
        doc.objects
            .iter()
            .any(|object| matches!(object.kind, Kind::CenterArc(..)))
    );
    assert!(
        doc.objects
            .iter()
            .any(|object| matches!(object.kind, Kind::PointOnArc(..)))
    );
    assert_eq!(doc.encode().unwrap().as_slice(), bytes);
    doc.verify_roundtrip().unwrap();
}

#[test]
fn mapped_path_and_intersection_native_generations_and_external_freezing() {
    let cases: &[(&[u8], usize, &[(f64, f64)])] = &[
        (
            include_bytes!(
                "../../../../../tests/fixtures/mapped_path44_path_segment_native_verified.gsp"
            ),
            11,
            &[(165., 250.), (193.75, 250.), (225.3125, 250.)],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/mapped_path44_path_circle_native_verified.gsp"),
            12,
            &[(190., 200.), (235., 173.521849295), (277.5, 154.590330075)],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/mapped_path44_line_line_native_verified.gsp"),
            15,
            &[(190., 250.), (235., 250.), (277.5, 250.)],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/mapped_path44_line_circle_native_verified.gsp"),
            14,
            &[(190., 300.), (235., 326.478150705), (277.5, 345.409669925)],
        ),
        // The oriented branch changes its vertical direction as the centers move.
        (
            include_bytes!(
                "../../../../../tests/fixtures/mapped_path44_circle_circle_native_verified.gsp"
            ),
            13,
            &[
                (190., 336.602540378),
                (310., 371.243556530),
                (420., 319.282032303),
            ],
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/mapped_path44_path_circle_omitted_prerequisites_native_verified.gsp"
            ),
            12,
            &[(190., 200.), (190., 200.), (190., 200.)],
        ),
    ];
    for &(bytes, binding, expected) in cases {
        let doc = Document::from_bytes(bytes, None).unwrap();
        let images = doc.branch_segment_iteration_images(binding).unwrap();
        assert_eq!(images.len(), 3);
        for (i, &(x, y)) in expected.iter().enumerate() {
            assert!(
                images[i].0.distance(Point { x, y }) < 1e-8,
                "{binding}: {images:?}"
            );
            assert!(
                images[i].1.distance(Point {
                    x: 280. + 40. * i as f64,
                    y: 250.
                }) < 1e-8
            );
        }
        assert_eq!(doc.encode().unwrap().as_slice(), bytes);
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn mapped_undefined_carried_intersection_retains_valid_prefix_and_other_images() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/mapped_path44_line_circle_undefined_native_verified.gsp"
    );
    let doc = Document::from_bytes(bytes, None).unwrap();
    let segments = doc.branch_segment_iteration_images(15).unwrap();
    assert_eq!(segments.len(), 3);
    for (i, x) in [190., 215., 227.5].into_iter().enumerate() {
        assert_eq!(
            segments[i],
            (Point { x, y: 250. }, Point { x: 240., y: 250. })
        );
    }
    let points = doc.mapped_point_binding_images(16).unwrap();
    assert_eq!(points.len(), 1);
    assert!(points[0].distance(Point { x: 230., y: 280. }) < 1e-9);
    doc.verify_roundtrip().unwrap();
}

#[test]
fn labeled_point_transform_and_circle_intersection_native_roundtrips() {
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/rotation_point_label44_fixed_native_verified.gsp")
            .as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/rotation_point_label44_numeric_native_verified.gsp"
        )
        .as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/mapped_path44_circle_circle_labeled_native_verified.gsp"
        )
        .as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(doc.objects.iter().any(|o| o.label.is_some()));
        assert_eq!(doc.encode().unwrap().as_slice(), bytes);
        doc.verify_roundtrip().unwrap();
    }
    for numeric in [false, true] {
        let mut doc = Document::from_bytes(
            include_bytes!("../../../../../tests/fixtures/numeric_typed_parameters_native_verified.gsp"),
            None,
        )
        .unwrap();
        doc.set_parameter_value(1, 60.).unwrap();
        let a = doc.add(Kind::Free(Point { x: 240., y: 250. }));
        let o = doc.add(Kind::Free(Point { x: 140., y: 250. }));
        let p = if numeric {
            doc.add_calculated_angle_rotation(a, o, 1).unwrap()
        } else {
            doc.add(Kind::RotatedPoint(a, o, 90.))
        };
        doc.set_point_label(p, "T", true).unwrap();
        let golden: &[u8] = if numeric {
            include_bytes!(
                "../../../../../tests/fixtures/rotation_point_label44_numeric_native_verified.gsp"
            )
        } else {
            include_bytes!(
                "../../../../../tests/fixtures/rotation_point_label44_fixed_native_verified.gsp"
            )
        };
        assert_eq!(doc.encode().unwrap().as_slice(), golden);
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn frozen_numeric_and_function_environments_keep_seed_types() {
    let mut doc = Document::new();
    let scalar = doc.add_named_parameter("a", 2., (10, 10)).unwrap();
    let number = doc
        .add_parameter_calculation(scalar, "a*3", (10, 40))
        .unwrap();
    let function = doc
        .add_referenced_function_definition(
            &[scalar],
            SimpleFunction::parse_with_sources("v1*x", 1).unwrap(),
        )
        .unwrap();
    let origin = doc.add(Kind::Free(Point { x: 10., y: 20. }));
    doc.add_coordinate_system_at_point(origin).unwrap();
    let unit = origin + 1;
    let original = doc.clone();
    doc.freeze_evaluation_inputs(&original, &[number, function])
        .unwrap();
    doc.set_evaluation_number(scalar, 7.).unwrap();
    doc.set_evaluation_point(unit, Point { x: 90., y: 20. })
        .unwrap();
    assert_eq!(doc.numeric_value(number), Some(6.));
    assert_eq!(doc.function_value(function, 5.), Some(10.));
    assert_eq!(doc.function_value(function, 8.), Some(16.));
    assert_eq!(doc.position(unit), Some(Point { x: 90., y: 20. }));
    assert_eq!(doc.objects[unit - 1].kind, original.objects[unit - 1].kind);
    assert_eq!(doc.encode().unwrap(), original.encode().unwrap());
    doc.clear_evaluation_seeds();
    assert_eq!(doc.numeric_value(scalar), Some(2.));
    assert_eq!(doc.position(unit), original.position(unit));
}

#[test]
fn authored_labeled_circle_intersection_iteration_matches_native_bytes() {
    let mut doc = Document::new();
    for (x, y) in [
        (140., 250.),
        (240., 250.),
        (80., 80.),
        (120., 80.),
        (80., 120.),
    ] {
        doc.add(Kind::Free(Point { x, y }));
    }
    doc.add(Kind::Segment(1, 2));
    doc.add(Kind::Midpoint(1, 2));
    doc.add(Kind::VectorTranslatedPoint(2, 3, 4));
    doc.add(Kind::Circle(1, 2));
    doc.add(Kind::Circle(2, 1));
    doc.add(Kind::CircleCircleIntersection(9, 10, 0));
    for (id, label) in [(1, "A"), (2, "B"), (11, "N"), (8, "R")] {
        doc.set_point_label(id, label, true).unwrap();
    }
    doc.selected.extend([3, 4, 5, 7, 9, 10]);
    doc.set_selected_visibility(false);
    doc.selected.clear();
    doc.add_mapped_point_iteration_with_bindings(
        &[1, 2],
        &[vec![11, 8]],
        &(3..=11).collect::<Vec<_>>(),
        &[6],
        IterationDepth::Fixed(3),
    )
    .unwrap();
    assert_eq!(
        doc.encode().unwrap().as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/mapped_path44_circle_circle_labeled_native_verified.gsp"
        )
    );
    doc.verify_roundtrip().unwrap();
}
