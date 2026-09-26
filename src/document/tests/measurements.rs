use super::*;

#[test]
fn point_line_distance_matches_native_supporting_lines_and_dependency_edits() {
    for shape in ["segment", "ray", "line"] {
        let bytes = std::fs::read(format!(
            "tests/fixtures/point_line_distance_{shape}_native.gsp"
        ))
        .unwrap();
        let mut doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects[4].kind, Kind::PointLineDistance(3, 4));
        assert_eq!(doc.encode().unwrap(), bytes);
        let mut arithmetic = doc.clone();
        let scaled = arithmetic
            .add_measured_length_calculation(5, 2, (10, 40))
            .unwrap();
        let sum = arithmetic
            .add_length_pair_calculation(5, scaled, false, (10, 70))
            .unwrap();
        let arithmetic = Document::from_bytes(&arithmetic.encode().unwrap(), None).unwrap();
        assert!(!arithmetic.partial_view);
        assert!((arithmetic.numeric_value(scaled).unwrap() - 3.175).abs() < 1e-12);
        assert!((arithmetic.numeric_value(sum).unwrap() - 4.7625).abs() < 1e-12);
        assert!(arithmetic.length_measurement_name(sum).is_some());
        assert!((doc.numeric_value(5).unwrap() - 1.5875).abs() < 1e-12);
        assert_eq!(doc.measurement_text(5).unwrap(), "Distance = 1.59 cm");
        let calc = doc.add_parameter_calculation(5, "v*2", (10, 40)).unwrap();
        assert!((doc.numeric_value(calc).unwrap() - 3.175).abs() < 1e-12);
        doc.set_measurement_precision(&[5], 0).unwrap();
        assert_eq!(doc.measurement_text(5).unwrap(), "Distance = 2 cm");
        assert!((doc.numeric_value(calc).unwrap() - 3.175).abs() < 1e-12);
        assert!(doc.move_point(3, Point { x: 250., y: 90. }));
        assert!((doc.distance_value_pixels(5).unwrap() - 90.).abs() < 1e-12);
        let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        reopened.selected.insert(5);
        let clipboard = reopened.copy_selected().unwrap();
        assert_eq!(clipboard.len(), 5);
        let pasted = reopened.paste(&clipboard).unwrap();
        assert_eq!(
            reopened.numeric_value(*pasted.last().unwrap()),
            reopened.numeric_value(5)
        );
        assert!(reopened.move_point(2, reopened.position(1).unwrap()));
        assert_eq!(reopened.numeric_value(5), None);
        assert_eq!(reopened.numeric_value(calc), None);
        assert!(reopened.move_point(2, Point { x: 200., y: 180. }));
        assert!(reopened.numeric_value(calc).is_some());
        reopened.selected.clear();
        reopened.selected.insert(4);
        assert_eq!(reopened.clear_selected().unwrap(), vec![4, 5, calc]);
    }
}
#[test]
fn point_line_distance_feeds_calculated_iteration_and_transformed_hosts() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let b = doc.add(Kind::Free(Point { x: 200., y: 180. }));
    let seed = doc.add(Kind::Free(Point { x: 50., y: 120. }));
    let segment = doc.add(Kind::Segment(a, b));
    let transformed = doc.add(Kind::PolarTranslatedSegment(segment, 0., 20., 1.));
    let distance = doc.add(Kind::PointLineDistance(seed, transformed));
    assert!((doc.distance_value_pixels(distance).unwrap() - 60.).abs() < 1e-12);
    let calc = doc
        .add_parameter_calculation(distance, "v/2", (10, 40))
        .unwrap();
    let image = doc.add(Kind::MarkedDistanceTranslatedPoint(seed, calc, 0.));
    let iteration = doc
        .add_point_iteration(seed, image, IterationDepth::Fixed(3))
        .unwrap();
    let points = doc.point_iteration_images(iteration).unwrap();
    assert_eq!(
        points,
        vec![
            Point { x: 110., y: 120. },
            Point { x: 140., y: 120. },
            Point { x: 170., y: 120. }
        ]
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.point_iteration_images(iteration), Some(points));
    assert!(doc.move_point(seed, Point { x: 50., y: 90. }));
    assert_eq!(
        doc.point_iteration_images(iteration).unwrap().last(),
        Some(&Point { x: 230., y: 90. })
    );
}
#[test]
fn point_line_distance_preserves_multi_page_parent_references() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/two_page_blank_native.gsp"),
        None,
    )
    .unwrap();
    doc.active_page = 1;
    let a = doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let b = doc.add(Kind::Free(Point { x: 200., y: 180. }));
    let point = doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let ray = doc.add(Kind::Ray(a, b));
    let distance = doc.add(Kind::PointLineDistance(point, ray));
    doc.set_measurement_precision(&[distance], 4).unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.partial_view, doc.partial_view);
    assert_eq!(
        reopened.objects[distance - 1].kind,
        Kind::PointLineDistance(point, ray)
    );
    assert_eq!(
        reopened.measurement_text(distance).unwrap(),
        "Distance = 1.5875 cm"
    );
    assert_eq!(reopened.objects[distance - 1].page, 1);
}

#[test]
fn point_distance_matches_original_and_updates_after_edit() {
    let original = Document::open(Path::new("tests/fixtures/wine_point_distance.gsp")).unwrap();
    assert_eq!(original.objects[5].kind, Kind::PointDistance(1, 2));
    assert!((original.point_distance_cm(6).unwrap() - 2.6458333333333335).abs() < 1e-10);
    let moved = Document::open(Path::new("tests/fixtures/wine_moved_distance.gsp")).unwrap();
    assert_eq!(moved.objects[2].kind, Kind::PointDistance(1, 2));
    assert!((moved.point_distance_cm(3).unwrap() - 5.291666666666667).abs() < 1e-10);

    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let b = doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let distance = doc.add(Kind::PointDistance(a, b));
    let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects[2].kind, Kind::PointDistance(a, b));
    reopened.checkpoint();
    assert!(reopened.move_point(b, Point { x: 350., y: 120. }));
    assert!((reopened.point_distance_cm(distance).unwrap() - 5.291666666666667).abs() < 1e-10);
    let bytes = reopened.encode().unwrap();
    assert_eq!(
        Document::from_bytes(&bytes, None)
            .unwrap()
            .point_distance_cm(distance),
        reopened.point_distance_cm(distance)
    );
    reopened.undo();
    assert!((reopened.point_distance_cm(distance).unwrap() - 2.6458333333333335).abs() < 1e-10);
    reopened.selected.insert(a);
    assert_eq!(reopened.clear_selected().unwrap(), vec![a, distance]);
}
#[test]
fn circumference_matches_original_and_updates_after_resize() {
    let original =
        Document::open(Path::new("tests/fixtures/wine_circle_circumference.gsp")).unwrap();
    assert_eq!(original.objects[3].kind, Kind::CircleCircumference(3));
    assert!((original.circle_circumference_cm(4).unwrap() - 16.62426112524599).abs() < 1e-10);
    let moved = Document::open(Path::new("tests/fixtures/wine_moved_circumference.gsp")).unwrap();
    assert_eq!(moved.objects[3].kind, Kind::CircleCircumference(3));
    assert!((moved.circle_circumference_cm(4).unwrap() - 24.936391687868986).abs() < 1e-10);

    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let b = doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let circle = doc.add(Kind::Circle(a, b));
    let circumference = doc.add(Kind::CircleCircumference(circle));
    let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects[3].kind, Kind::CircleCircumference(circle));
    reopened.checkpoint();
    assert!(reopened.move_point(b, Point { x: 350., y: 120. }));
    assert!(
        (reopened.circle_circumference_cm(circumference).unwrap() - 33.24852225049198).abs()
            < 1e-10
    );
    let bytes = reopened.encode().unwrap();
    assert_eq!(
        Document::from_bytes(&bytes, None)
            .unwrap()
            .circle_circumference_cm(circumference),
        reopened.circle_circumference_cm(circumference)
    );
    reopened.selected.insert(circle);
    assert_eq!(
        reopened.clear_selected().unwrap(),
        vec![circle, circumference]
    );
}
#[test]
fn radius_matches_original_and_updates_after_resize() {
    let original = Document::open(Path::new("tests/fixtures/wine_circle_radius.gsp")).unwrap();
    assert_eq!(original.objects[3].kind, Kind::CircleRadius(3));
    assert!((original.circle_radius_cm(4).unwrap() - 2.6458333333333335).abs() < 1e-10);
    let moved = Document::open(Path::new("tests/fixtures/wine_moved_radius.gsp")).unwrap();
    assert_eq!(moved.objects[3].kind, Kind::CircleRadius(3));
    assert!((moved.circle_radius_cm(4).unwrap() - 3.96875).abs() < 1e-10);

    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let b = doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let circle = doc.add(Kind::Circle(a, b));
    let radius = doc.add(Kind::CircleRadius(circle));
    let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects[3].kind, Kind::CircleRadius(circle));
    reopened.checkpoint();
    assert!(reopened.move_point(b, Point { x: 350., y: 120. }));
    assert!((reopened.circle_radius_cm(radius).unwrap() - 5.291666666666667).abs() < 1e-10);
    assert_eq!(
        Document::from_bytes(&reopened.encode().unwrap(), None)
            .unwrap()
            .circle_radius_cm(radius),
        reopened.circle_radius_cm(radius)
    );
    reopened.selected.insert(circle);
    assert_eq!(reopened.clear_selected().unwrap(), vec![circle, radius]);
}
#[test]
fn angle_matches_original_and_updates_after_drag() {
    let original = Document::open(Path::new("tests/fixtures/wine_point_angle.gsp")).unwrap();
    assert_eq!(original.objects[3].kind, Kind::AngleMeasurement(1, 2, 3));
    assert!((original.angle_degrees(4).unwrap() - 90.).abs() < 1e-10);
    let moved = Document::open(Path::new("tests/fixtures/wine_moved_angle.gsp")).unwrap();
    assert_eq!(moved.objects[3].kind, Kind::AngleMeasurement(1, 2, 3));
    assert!((moved.angle_degrees(4).unwrap() - 135.).abs() < 1e-10);
    let edited_generated =
        Document::open(Path::new("tests/fixtures/wine_moved_generated_angle.gsp")).unwrap();
    assert_eq!(
        edited_generated.objects[3].kind,
        Kind::AngleMeasurement(1, 2, 3)
    );
    assert!((edited_generated.angle_degrees(4).unwrap() - 135.).abs() < 1e-10);

    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let b = doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let c = doc.add(Kind::Free(Point { x: 250., y: 220. }));
    let angle = doc.add(Kind::AngleMeasurement(a, b, c));
    let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects[3].kind, Kind::AngleMeasurement(a, b, c));
    reopened.checkpoint();
    assert!(reopened.move_point(c, Point { x: 350., y: 220. }));
    assert!((reopened.angle_degrees(angle).unwrap() - 135.).abs() < 1e-10);
    assert_eq!(
        Document::from_bytes(&reopened.encode().unwrap(), None)
            .unwrap()
            .angle_degrees(angle),
        reopened.angle_degrees(angle)
    );
    reopened.undo();
    assert!((reopened.angle_degrees(angle).unwrap() - 90.).abs() < 1e-10);
    reopened.selected.insert(b);
    assert_eq!(reopened.clear_selected().unwrap(), vec![b, angle]);
}
#[test]
fn angle_rejects_zero_rays_and_evaluates_large_coordinates() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let b = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let c = doc.add(Kind::Free(Point { x: 1., y: 0. }));
    assert_eq!(doc.angle_for_points(a, b, c), None);
    assert!(doc.move_point(a, Point { x: -1e150, y: 0. }));
    assert!(doc.move_point(c, Point { x: 0., y: 1e150 }));
    assert!((doc.angle_for_points(a, b, c).unwrap() - 90.).abs() < 1e-10);
}
#[test]
fn circle_area_matches_original_and_updates_after_resize() {
    let original = Document::open(Path::new("tests/fixtures/wine_circle_area.gsp")).unwrap();
    assert_eq!(original.objects[3].kind, Kind::CircleArea(3));
    assert!((original.circle_area_cm2(4).unwrap() - 21.992512113606676).abs() < 1e-10);
    let moved = Document::open(Path::new("tests/fixtures/wine_moved_area.gsp")).unwrap();
    assert_eq!(moved.objects[3].kind, Kind::CircleArea(3));
    assert!((moved.circle_area_cm2(4).unwrap() - 49.483152255615025).abs() < 1e-10);
    let generated_moved =
        Document::open(Path::new("tests/fixtures/wine_moved_generated_area.gsp")).unwrap();
    assert_eq!(generated_moved.objects[3].kind, Kind::CircleArea(3));
    assert!((generated_moved.circle_area_cm2(4).unwrap() - 49.483152255615025).abs() < 1e-10);

    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let b = doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let circle = doc.add(Kind::Circle(a, b));
    let area = doc.add(Kind::CircleArea(circle));
    let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects[3].kind, Kind::CircleArea(circle));
    reopened.checkpoint();
    assert!(reopened.move_point(b, Point { x: 300., y: 120. }));
    assert!((reopened.circle_area_cm2(area).unwrap() - 49.483152255615025).abs() < 1e-10);
    assert_eq!(
        Document::from_bytes(&reopened.encode().unwrap(), None)
            .unwrap()
            .circle_area_cm2(area),
        reopened.circle_area_cm2(area)
    );
    reopened.selected.insert(circle);
    assert_eq!(reopened.clear_selected().unwrap(), vec![circle, area]);
}
#[test]
fn transformed_circle_measurements_follow_their_sources() {
    for (base, host, expected) in [
        (
            include_bytes!(
                "../../../tests/fixtures/marked_distance_circle_translation_original.gsp"
            )
            .as_slice(),
            7,
            include_bytes!("../../../tests/fixtures/transformed_circle_measurements_port.gsp")
                .as_slice(),
        ),
        (
            include_bytes!("../../../tests/fixtures/ratio_dilation_circle_original.gsp").as_slice(),
            8,
            include_bytes!("../../../tests/fixtures/scaled_circle_measurements_port.gsp")
                .as_slice(),
        ),
    ] {
        let mut document = Document::from_bytes(base, None).unwrap();
        let area = document.add(Kind::CircleArea(host));
        let circumference = document.add(Kind::CircleCircumference(host));
        let radius = document.add(Kind::CircleRadius(host));
        let before = document.circle_radius_cm(radius).unwrap();
        if host == 8 {
            assert!(document.move_point(6, Point { x: 180., y: 120. }));
            assert!(document.circle_radius_cm(radius).unwrap() > before);
        }
        let encoded = document.encode().unwrap();
        assert_eq!(encoded, expected);
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.encode().unwrap(), encoded);
        assert!(reopened.circle_area_cm2(area).unwrap() > 0.);
        assert!(reopened.circle_circumference_cm(circumference).unwrap() > 0.);
        assert!(reopened.circle_radius_cm(radius).unwrap() > 0.);
    }
}

#[test]
fn multiple_measurements_use_original_label_spacing() {
    let original = Document::open(Path::new("tests/fixtures/wine_circle_area_radius.gsp")).unwrap();
    assert_eq!(original.objects[3].kind, Kind::CircleArea(3));
    assert_eq!(original.objects[4].kind, Kind::CircleRadius(3));

    let mut doc = Document::new();
    let center = doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let edge = doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let circle = doc.add(Kind::Circle(center, edge));
    doc.add(Kind::CircleArea(circle));
    doc.add(Kind::CircleRadius(circle));
    let bytes = doc.encode().unwrap();
    let ys: Vec<u16> = records(&bytes)
        .unwrap()
        .iter()
        .filter(|record| record.kind == 0x0903)
        .map(|record| {
            u16::from_le_bytes([bytes[record.data.start + 14], bytes[record.data.start + 15]])
        })
        .collect();
    assert_eq!(ys, [5, 33]);
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(reopened.objects[3].kind, Kind::CircleArea(circle));
    assert_eq!(reopened.objects[4].kind, Kind::CircleRadius(circle));
}
#[test]
fn arc_angle_matches_original_and_updates_after_drag() {
    let original = Document::open(Path::new("tests/fixtures/wine_arc_angle.gsp")).unwrap();
    assert_eq!(original.objects[4].kind, Kind::ArcAngle(4));
    assert!((original.arc_angle_degrees(5).unwrap() - 159.22).abs() < 0.01);
    let moved = Document::open(Path::new("tests/fixtures/wine_moved_arc_angle.gsp")).unwrap();
    assert_eq!(moved.objects[4].kind, Kind::ArcAngle(4));
    assert!((moved.arc_angle_degrees(5).unwrap() - 225.24).abs() < 0.01);

    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 200., y: 180. }));
    let b = doc.add(Kind::Free(Point { x: 260., y: 130. }));
    let c = doc.add(Kind::Free(Point { x: 320., y: 180. }));
    let arc = doc.add(Kind::ThreePointArc(a, b, c));
    let measurement = doc.add(Kind::ArcAngle(arc));
    let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects[4].kind, Kind::ArcAngle(arc));
    reopened.checkpoint();
    assert!(reopened.move_point(b, Point { x: 260., y: 90. }));
    assert!((reopened.arc_angle_degrees(measurement).unwrap() - 225.24).abs() < 0.01);
    assert_eq!(
        Document::from_bytes(&reopened.encode().unwrap(), None)
            .unwrap()
            .arc_angle_degrees(measurement),
        reopened.arc_angle_degrees(measurement)
    );
    reopened.selected.insert(arc);
    assert_eq!(reopened.clear_selected().unwrap(), vec![arc, measurement]);
}
#[test]
fn arc_length_matches_original_and_updates_after_drag() {
    let original = Document::open(Path::new("tests/fixtures/wine_arc_length.gsp")).unwrap();
    assert_eq!(original.objects[4].kind, Kind::ArcLength(4));
    assert!((original.arc_length_cm(5).unwrap() - 4.49).abs() < 0.01);
    let moved = Document::open(Path::new("tests/fixtures/wine_moved_arc_length.gsp")).unwrap();
    assert_eq!(moved.objects[4].kind, Kind::ArcLength(4));
    assert!((moved.arc_length_cm(5).unwrap() - 6.76).abs() < 0.01);

    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 200., y: 180. }));
    let b = doc.add(Kind::Free(Point { x: 260., y: 130. }));
    let c = doc.add(Kind::Free(Point { x: 320., y: 180. }));
    let arc = doc.add(Kind::ThreePointArc(a, b, c));
    let measurement = doc.add(Kind::ArcLength(arc));
    let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects[4].kind, Kind::ArcLength(arc));
    reopened.checkpoint();
    assert!(reopened.move_point(b, Point { x: 260., y: 90. }));
    assert!((reopened.arc_length_cm(measurement).unwrap() - 6.76).abs() < 0.01);
    assert_eq!(
        Document::from_bytes(&reopened.encode().unwrap(), None)
            .unwrap()
            .arc_length_cm(measurement),
        reopened.arc_length_cm(measurement)
    );
    reopened.selected.insert(arc);
    assert_eq!(reopened.clear_selected().unwrap(), vec![arc, measurement]);
}
#[test]
fn arc_on_circle_measurements_reopen_with_separate_labels() {
    let doc = Document::open(Path::new(
        "tests/fixtures/wine_arc_measurements_on_circle.gsp",
    ))
    .unwrap();
    assert_eq!(doc.objects[5].kind, Kind::ArcOnCircle(3, 4, 5));
    assert_eq!(doc.objects[6].kind, Kind::ArcAngle(6));
    assert_eq!(doc.objects[7].kind, Kind::ArcLength(6));
    assert!((doc.arc_angle_degrees(7).unwrap() - 90.).abs() < 1e-10);
    assert!((doc.arc_length_cm(8).unwrap() - 2.91).abs() < 0.01);
}
