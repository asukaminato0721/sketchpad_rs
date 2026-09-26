use super::*;

#[test]
fn marked_translation_iterations_pin_measurements_and_follow_source_edits() {
    let bytes = include_bytes!("../../../tests/fixtures/marked_distance_iteration_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(
        doc.point_iteration_images(5).unwrap(),
        vec![
            Point { x: 160., y: 180. },
            Point { x: 180., y: 180. },
            Point { x: 200., y: 180. }
        ]
    );
    doc.move_point(2, Point { x: 90., y: 180. });
    assert_eq!(
        doc.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/marked_distance_iteration_edited_port.gsp")
    );
    assert_eq!(
        doc.point_iteration_images(5).unwrap()[2],
        Point { x: 240., y: 180. }
    );
    let calculation = doc.add_parameter_calculation(3, "v/2", (10, 60)).unwrap();
    let image = doc.add(Kind::MarkedDistanceTranslatedPoint(1, calculation, 0.));
    let iteration = doc
        .add_point_iteration(1, image, IterationDepth::Fixed(3))
        .unwrap();
    assert_eq!(
        doc.point_iteration_images(iteration).unwrap()[2],
        Point { x: 180., y: 180. }
    );
    let denominator = doc.add_named_parameter("n", 2., (10, 100)).unwrap();
    let quotient = doc
        .add_multi_calculation(&[3, denominator], "v1/v2", (10, 130))
        .unwrap();
    let quotient_image = doc.add(Kind::MarkedDistanceTranslatedPoint(1, quotient, 0.));
    let quotient_iteration = doc
        .add_point_iteration(1, quotient_image, IterationDepth::Fixed(2))
        .unwrap();
    doc.set_parameter_value(denominator, 0.).unwrap();
    assert!(doc.point_iteration_images(quotient_iteration).is_none());
    doc.set_parameter_value(denominator, 2.).unwrap();
    assert_eq!(
        doc.point_iteration_images(quotient_iteration).unwrap()[1],
        Point { x: 165., y: 180. }
    );
    let mut numeric = Document::from_bytes(
        include_bytes!(
            "../../../tests/fixtures/marked_calculated_distance_numeric_iteration_port.gsp"
        ),
        None,
    )
    .unwrap();
    assert!(!numeric.partial_view);
    assert_eq!(
        numeric.point_parameter_iteration_labels(10).unwrap(),
        vec![
            (Point { x: 160., y: 220. }, "4".into()),
            (Point { x: 200., y: 220. }, "8".into()),
            (Point { x: 240., y: 220. }, "16".into())
        ]
    );
    numeric.move_point(3, Point { x: 90., y: 260. });
    assert_eq!(
        numeric.encode().unwrap(),
        include_bytes!(
            "../../../tests/fixtures/marked_calculated_distance_numeric_iteration_edited_port.gsp"
        )
    );
    assert_eq!(
        numeric.point_parameter_iteration_labels(10).unwrap()[2].0,
        Point { x: 320., y: 220. }
    );
    doc.selected.insert(iteration);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = *pasted
        .iter()
        .find(|id| matches!(doc.objects[**id - 1].kind, Kind::PointIteration(..)))
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.point_iteration_images(copied),
        doc.point_iteration_images(copied)
    );
    doc.selected.clear();
    doc.selected.insert(2);
    let deleted = doc.clear_selected().unwrap();
    assert!(deleted.contains(&iteration));
    assert!(deleted.contains(&calculation));
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn marked_rectangular_and_mixed_iterations_carry_geometry() {
    let doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/marked_rectangular_segment_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert!(!doc.partial_view);
    assert_eq!(
        doc.segment_iteration_images(9).unwrap(),
        vec![
            (Point { x: 120., y: 210. }, Point { x: 140., y: 200. }),
            (Point { x: 140., y: 200. }, Point { x: 160., y: 190. }),
            (Point { x: 160., y: 190. }, Point { x: 180., y: 180. })
        ]
    );
    let doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/mixed_horizontal_circle_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert!(!doc.partial_view);
    let circles = doc.circle_iteration_images(9).unwrap();
    let dx = 0.5 * 37.79527559055118;
    for (i, (center, radius)) in circles.iter().enumerate() {
        assert!((center.x - (100. + dx * (i + 1) as f64)).abs() < 1e-10);
        assert_eq!(center.y, 220. - 10. * (i + 1) as f64);
        assert!((radius - dx.hypot(10.)).abs() < 1e-10);
    }
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/mixed_vertical_ray_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert!(!doc.partial_view);
    let rays = doc.ray_iteration_images(8).unwrap();
    assert_eq!(rays.len(), 3);
    assert_eq!(rays[0].0.x, 120.);
    assert_eq!(rays[0].1.x, 140.);
    assert!((rays[0].1.y - (220. - 0.5 * 37.79527559055118)).abs() < 1e-10);
    doc.set_parameter_value(7, 4.8).unwrap();
    assert_eq!(doc.ray_iteration_images(8).unwrap().len(), 4);
}

#[test]
fn marked_angle_and_ratio_iterations_substitute_every_direct_seed_input() {
    let mut first = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/marked_angle_seed_first_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert!(!first.partial_view);
    let image = first.position(4).unwrap();
    for point in first.point_iteration_images(5).unwrap() {
        assert!(point.distance(image) < 1e-10);
    }
    first.move_point(3, first.position(2).unwrap());
    assert!(first.point_iteration_images(5).is_none());
    first.move_point(3, Point { x: 190., y: 150. });
    assert_eq!(first.point_iteration_images(5).unwrap().len(), 3);
    let last = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/marked_angle_seed_last_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert!(!last.partial_view);
    let points = last.point_iteration_images(6).unwrap();
    for (actual, expected) in points.iter().zip([
        Point {
            x: 160. - 3200_f64.sqrt(),
            y: 180.,
        },
        Point {
            x: 160. + 3200_f64.sqrt(),
            y: 180.,
        },
        Point {
            x: 160. + 3200_f64.sqrt(),
            y: 180.,
        },
    ]) {
        assert!(actual.distance(expected) < 1e-10);
    }
    let carried = last.segment_iteration_images(6).unwrap();
    for i in 0..2 {
        assert_eq!(carried[i].1, carried[i + 1].0);
    }
    let mut ratio = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/marked_ratio_seed_target_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert!(!ratio.partial_view);
    let points = ratio.point_iteration_images(6).unwrap();
    for (point, x) in points.iter().zip([140.96, 116.777216, 102.81474976710656]) {
        assert!((point.x - x).abs() < 1e-10);
    }
    ratio.selected.insert(6);
    let clipboard = ratio.copy_selected().unwrap();
    let pasted = ratio.paste(&clipboard).unwrap();
    let copied = *pasted
        .iter()
        .find(|id| matches!(ratio.objects[**id - 1].kind, Kind::PointIteration(..)))
        .unwrap();
    for (actual, original) in ratio
        .point_iteration_images(copied)
        .unwrap()
        .iter()
        .zip(points)
    {
        assert!(
            actual.distance(Point {
                x: original.x + 20.,
                y: original.y + 20.
            }) < 1e-10
        );
    }
    assert!(
        !Document::from_bytes(&ratio.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
    ratio.move_point(4, ratio.position(3).unwrap());
    assert!(ratio.point_iteration_images(6).is_none());
    ratio.move_point(4, Point { x: 200., y: 180. });
    assert_eq!(ratio.point_iteration_images(6).unwrap().len(), 3);
}

#[test]
fn iteration_centers_from_external_constructions_are_pinned() {
    for (bytes, expected) in [
        (
            include_bytes!("../../../tests/fixtures/rotation_derived_center_iteration_port.gsp")
                .as_slice(),
            vec![
                Point { x: 100., y: 180. },
                Point { x: 160., y: 240. },
                Point { x: 220., y: 180. },
            ],
        ),
        (
            include_bytes!("../../../tests/fixtures/dilation_derived_center_iteration_port.gsp")
                .as_slice(),
            vec![
                Point { x: 175., y: 180. },
                Point { x: 167.5, y: 180. },
                Point { x: 163.75, y: 180. },
            ],
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        for (point, expected) in doc.point_iteration_images(6).unwrap().iter().zip(expected) {
            assert!(point.distance(expected) < 1e-10);
        }
    }
}

#[test]
fn vector_iteration_substitutes_repeated_seeds_and_pins_external_inputs() {
    for (bytes, iteration, xs) in [
        (
            include_bytes!("../../../tests/fixtures/vector_point_iteration_port.gsp").as_slice(),
            5,
            vec![180., 220., 260.],
        ),
        (
            include_bytes!("../../../tests/fixtures/vector_seed_start_iteration_port.gsp")
                .as_slice(),
            4,
            vec![140., 140., 140.],
        ),
        (
            include_bytes!("../../../tests/fixtures/vector_seed_end_iteration_port.gsp").as_slice(),
            4,
            vec![180., 260.],
        ),
        (
            include_bytes!("../../../tests/fixtures/vector_external_dependency_iteration_port.gsp")
                .as_slice(),
            7,
            vec![160., 180., 200.],
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        assert_eq!(
            doc.point_iteration_images(iteration)
                .unwrap()
                .iter()
                .map(|point| point.x)
                .collect::<Vec<_>>(),
            xs
        );
    }
    let mut stationary = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/vector_seed_start_iteration_port.gsp"),
        None,
    )
    .unwrap();
    stationary.selected.insert(4);
    let clipboard = stationary.copy_selected().unwrap();
    let pasted = stationary.paste(&clipboard).unwrap();
    let copied = *pasted
        .iter()
        .find(|id| matches!(stationary.objects[**id - 1].kind, Kind::PointIteration(..)))
        .unwrap();
    let Kind::PointIteration(_, image, _) = stationary.objects[copied - 1].kind else {
        panic!("iteration")
    };
    let target = stationary.position(image).unwrap();
    assert_eq!(
        stationary.point_iteration_images(copied).unwrap(),
        vec![target; 3]
    );
    let reopened = Document::from_bytes(&stationary.encode().unwrap(), None).unwrap();
    assert_eq!(
        reopened.point_iteration_images(copied),
        stationary.point_iteration_images(copied)
    );
    stationary.move_point(2, Point { x: 160., y: 170. });
    assert_eq!(
        stationary.point_iteration_images(4).unwrap(),
        vec![Point { x: 160., y: 170. }; 3]
    );
}

#[test]
fn reflected_iterations_alternate_and_keep_external_mirrors_pinned() {
    for (bytes, iteration, image) in [
        (
            include_bytes!("../../../tests/fixtures/reflection_point_iteration_port.gsp")
                .as_slice(),
            6,
            5,
        ),
        (
            include_bytes!(
                "../../../tests/fixtures/reflection_external_dependency_iteration_port.gsp"
            )
            .as_slice(),
            8,
            7,
        ),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        let seed = doc.position(1).unwrap();
        let first = doc.position(image).unwrap();
        assert_eq!(
            doc.point_iteration_images(iteration).unwrap(),
            vec![seed, first, seed]
        );
        doc.move_point(1, Point { x: 240., y: 190. });
        let positions = doc.point_iteration_images(iteration).unwrap();
        assert_eq!(positions[0], doc.position(1).unwrap());
        assert!(positions[1].distance(doc.position(image).unwrap()) < 1e-10);
        doc.selected.insert(iteration);
        let clipboard = doc.copy_selected().unwrap();
        let pasted = doc.paste(&clipboard).unwrap();
        let copied = *pasted
            .iter()
            .find(|id| matches!(doc.objects[**id - 1].kind, Kind::PointIteration(..)))
            .unwrap();
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.point_iteration_images(copied),
            doc.point_iteration_images(copied)
        );
    }
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/reflection_segment_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(
        doc.segment_iteration_images(7).unwrap(),
        vec![
            (Point { x: 80., y: 180. }, Point { x: 220., y: 180. }),
            (Point { x: 220., y: 180. }, Point { x: 80., y: 180. }),
            (Point { x: 80., y: 180. }, Point { x: 220., y: 180. }),
        ]
    );
    doc.move_point(2, Point { x: 120., y: 80. });
    assert_eq!(
        doc.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/reflection_segment_iteration_edited_port.gsp")
    );
    let lines = doc.segment_iteration_images(7).unwrap();
    assert!(lines[0].0.distance(doc.position(5).unwrap()) < 1e-10);
    assert!(lines[0].1.distance(doc.position(1).unwrap()) < 1e-10);
    doc.move_point(2, Point { x: 150., y: 260. });
    assert!(doc.segment_iteration_images(7).is_none());
    doc.move_point(2, Point { x: 150., y: 80. });
    assert_eq!(doc.segment_iteration_images(7).unwrap().len(), 3);
    let numeric = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/reflection_numeric_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert!(!numeric.partial_view);
    assert_eq!(
        numeric.point_parameter_iteration_labels(9).unwrap(),
        vec![(Point { x: 220., y: 180. }, "5".into())]
    );
    assert_eq!(
        numeric.point_numeric_label(8).unwrap(),
        (Point { x: 80., y: 180. }, "3".into())
    );
}

#[test]
fn reflected_and_vector_carried_circles_rays_and_degeneracy_match_native() {
    let reflection = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/reflection_circle_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(
        reflection.circle_iteration_images(10).unwrap(),
        vec![
            (Point { x: 30., y: 180. }, 140.),
            (Point { x: 170., y: 180. }, 140.),
            (Point { x: 30., y: 180. }, 140.),
        ]
    );
    let expanding = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/vector_expanding_circle_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(
        expanding.circle_iteration_images(7).unwrap(),
        vec![
            (Point { x: 140., y: 180. }, 40.),
            (Point { x: 180., y: 180. }, 80.),
        ]
    );
    let collapsed = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/vector_collapsed_circle_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert!(collapsed.circle_iteration_images(7).unwrap().is_empty());
    assert_eq!(
        collapsed.point_iteration_images(5).unwrap(),
        vec![Point { x: 140., y: 180. }; 3]
    );
    let mut rays = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/vector_ray_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(
        rays.ray_iteration_images(7).unwrap()[0],
        (Point { x: 140., y: 180. }, Point { x: 180., y: 180. })
    );
    rays.move_point(3, Point { x: 110., y: 220. });
    rays.set_parameter_value(6, 4.).unwrap();
    assert_eq!(
        rays.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/vector_ray_iteration_edited_port.gsp")
    );
    assert_eq!(
        rays.ray_iteration_images(7).unwrap()[0],
        (Point { x: 150., y: 160. }, Point { x: 200., y: 140. })
    );
    rays.selected.insert(3);
    assert_eq!(rays.clear_selected().unwrap(), vec![3, 4, 5, 7, 8, 9]);
    assert!(
        !Document::from_bytes(&rays.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn dilated_point_and_numeric_iterations_preserve_geometric_and_numeric_steps() {
    let bytes = include_bytes!("../../../tests/fixtures/dilation_point_iteration_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(
        doc.point_iteration_images(4).unwrap(),
        vec![
            Point { x: 130., y: 180. },
            Point { x: 115., y: 180. },
            Point { x: 107.5, y: 180. },
            Point { x: 103.75, y: 180. },
        ]
    );
    doc.move_point(2, Point { x: 120., y: 160. });
    let images = doc.point_iteration_images(4).unwrap();
    assert_eq!(images[0], Point { x: 145., y: 165. });
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.point_iteration_images(4), Some(images));
    let mut numeric = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/dilation_numeric_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert!(!numeric.partial_view);
    let labels = numeric.point_parameter_iteration_labels(7).unwrap();
    assert_eq!(
        labels
            .iter()
            .map(|(_, text)| text.as_str())
            .collect::<Vec<_>>(),
        vec!["4.00", "8.00", "16.00", "32.00"]
    );
    for (index, (point, _)) in labels.iter().enumerate() {
        let expected = 100. + 50. * 1.3_f64.powi(index as i32 + 2);
        assert!((point.x - expected).abs() < 1e-10);
    }
    numeric.set_parameter_value(4, 2.).unwrap();
    assert_eq!(
        numeric.point_parameter_iteration_labels(7).unwrap()[0].1,
        "8.00"
    );
    let reopened = Document::from_bytes(&numeric.encode().unwrap(), None).unwrap();
    assert_eq!(
        reopened.point_parameter_iteration_labels(7),
        numeric.point_parameter_iteration_labels(7)
    );
}

#[test]
fn dilated_carried_geometry_scales_radii_and_tracks_ratio_dependencies() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/dilation_segment_iteration_port.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/dilation_segment_iteration_edited_port.gsp")
            .as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        let seed = doc.position(1).unwrap();
        let center = doc.position(2).unwrap();
        let lines = doc.segment_iteration_images(5).unwrap();
        for (index, (a, b)) in lines.iter().enumerate() {
            let scale = (-0.5_f64).powi(index as i32 + 1);
            assert!((a.x - (center.x + scale * (seed.x - center.x))).abs() < 1e-10);
            assert!((a.y - (center.y + scale * (seed.y - center.y))).abs() < 1e-10);
            assert!((b.distance(center) - a.distance(center) * 0.5).abs() < 1e-10);
        }
    }
    let circle = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/dilation_circle_iteration_port.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(
        circle.circle_iteration_images(8).unwrap(),
        vec![
            (Point { x: 160., y: 180. }, 30.),
            (Point { x: 130., y: 180. }, 15.),
            (Point { x: 115., y: 180. }, 7.5),
        ]
    );
    let bytes = include_bytes!("../../../tests/fixtures/dilation_ratio_ray_iteration_port.gsp");
    let mut ratio = Document::from_bytes(bytes, None).unwrap();
    assert!(!ratio.partial_view);
    assert_eq!(
        ratio.ray_iteration_images(9).unwrap()[0],
        (Point { x: 160., y: 180. }, Point { x: 130., y: 180. })
    );
    ratio.move_point(5, Point { x: 175., y: 260. });
    ratio.set_parameter_value(8, 4.).unwrap();
    assert_eq!(
        ratio.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/dilation_ratio_ray_iteration_edited_port.gsp")
    );
    assert_eq!(
        ratio.ray_iteration_images(9).unwrap()[0],
        (Point { x: 190., y: 180. }, Point { x: 167.5, y: 180. })
    );
    ratio.selected.insert(9);
    let clipboard = ratio.copy_selected().unwrap();
    let pasted = ratio.paste(&clipboard).unwrap();
    let copied = *pasted
        .iter()
        .find(|id| matches!(ratio.objects[**id - 1].kind, Kind::RayIteration(..)))
        .unwrap();
    let reopened = Document::from_bytes(&ratio.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.ray_iteration_images(copied).unwrap().len(), 4);
    ratio.move_point(4, Point { x: 100., y: 260. });
    assert!(ratio.ray_iteration_images(9).is_none());
    ratio.move_point(4, Point { x: 200., y: 260. });
    assert_eq!(ratio.ray_iteration_images(9).unwrap().len(), 4);
    ratio.selected.clear();
    ratio.selected.insert(5);
    assert_eq!(ratio.clear_selected().unwrap(), vec![5, 6, 7, 9, 10, 11]);
    assert!(
        !Document::from_bytes(&ratio.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
