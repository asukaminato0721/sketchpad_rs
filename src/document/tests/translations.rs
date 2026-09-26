use super::*;

#[test]
fn vector_translation_tracks_marked_points_and_round_trips() {
    let native =
        Document::open(Path::new("tests/fixtures/vector_translation_original.gsp")).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.objects[4].kind, Kind::VectorTranslatedPoint(4, 1, 2));
    assert_eq!(native.position(5), Some(Point { x: 170., y: 210. }));

    let mut created = Document::new();
    let start = created.add(Kind::Free(Point { x: 150., y: 120. }));
    let end = created.add(Kind::Free(Point { x: 220., y: 130. }));
    created.add(Kind::Segment(start, end));
    let source = created.add(Kind::Free(Point { x: 100., y: 200. }));
    let image = created.add(Kind::VectorTranslatedPoint(source, start, end));
    assert_eq!(created.position(image), Some(Point { x: 170., y: 210. }));
    let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.objects[4].kind,
        Kind::VectorTranslatedPoint(4, 1, 2)
    );
    assert!(reopened.move_point(end, Point { x: 250., y: 130. }));
    assert_eq!(reopened.position(image), Some(Point { x: 200., y: 210. }));
    let moved = Document::from_bytes(&reopened.encode().unwrap(), None).unwrap();
    assert_eq!(moved.position(image), reopened.position(image));
}
#[test]
fn marked_distance_translation_imports_and_tracks_measurement() {
    let bytes = fs::read("tests/fixtures/marked_distance_translation_original.gsp").unwrap();
    let native = Document::from_bytes(&bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(
        native.objects[4].kind,
        Kind::MarkedDistanceTranslatedPoint(3, 4, 90.)
    );
    assert_eq!(native.position(5), Some(Point { x: 100., y: 120. }));
    assert_eq!(native.encode().unwrap(), bytes);

    let mut created = Document::new();
    let start = created.add(Kind::Free(Point { x: 150., y: 120. }));
    let end = created.add(Kind::Free(Point { x: 250., y: 120. }));
    let source = created.add(Kind::Free(Point { x: 100., y: 220. }));
    let distance = created.add(Kind::PointDistance(start, end));
    let image = created.add(Kind::MarkedDistanceTranslatedPoint(source, distance, 90.));
    let horizontal = created.add(Kind::MarkedDistanceTranslatedPoint(source, distance, 0.));
    assert_eq!(created.position(image), Some(Point { x: 100., y: 120. }));
    assert_eq!(
        created.position(horizontal),
        Some(Point { x: 200., y: 220. })
    );
    let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[4].kind, created.objects[4].kind);
    assert!(reopened.move_point(end, Point { x: 300., y: 120. }));
    let moved = reopened.position(image).unwrap();
    assert!((moved.x - 100.).abs() < 1e-9 && (moved.y - 70.).abs() < 1e-9);
    assert_eq!(
        reopened.position(horizontal),
        Some(Point { x: 250., y: 220. })
    );
}
#[test]
fn marked_distance_shape_translations_import_and_track_measurement() {
    for (path, expected_kind) in [
        (
            "tests/fixtures/marked_distance_segment_translation_original.gsp",
            Kind::MarkedDistanceTranslatedSegment(6, 5, 90.),
        ),
        (
            "tests/fixtures/marked_distance_circle_translation_original.gsp",
            Kind::MarkedDistanceTranslatedCircle(6, 5, 90.),
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view, "{path}");
        assert_eq!(native.objects[6].kind, expected_kind, "{path}");
        assert_eq!(native.encode().unwrap(), bytes, "{path}");
    }

    let mut created = Document::new();
    let measure_start = created.add(Kind::Free(Point { x: 150., y: 120. }));
    let measure_end = created.add(Kind::Free(Point { x: 250., y: 120. }));
    let source_start = created.add(Kind::Free(Point { x: 100., y: 220. }));
    let source_end = created.add(Kind::Free(Point { x: 200., y: 220. }));
    let distance = created.add(Kind::PointDistance(measure_start, measure_end));
    let segment = created.add(Kind::Segment(source_start, source_end));
    let circle = created.add(Kind::Circle(source_start, source_end));
    let moved_segment = created.add(Kind::MarkedDistanceTranslatedSegment(
        segment, distance, 90.,
    ));
    let moved_circle = created.add(Kind::MarkedDistanceTranslatedCircle(circle, distance, 90.));
    let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(reopened.move_point(measure_end, Point { x: 300., y: 120. }));
    let (a, b, mode) = reopened.line_points(moved_segment).unwrap();
    assert_eq!(mode, LineMode::Segment);
    assert!((a.x - 100.).abs() < 1e-9 && (a.y - 70.).abs() < 1e-9);
    assert!((b.x - 200.).abs() < 1e-9 && (b.y - 70.).abs() < 1e-9);
    let (center, radius) = reopened.circle_points(moved_circle).unwrap();
    assert!((center.x - 100.).abs() < 1e-9 && (center.y - 70.).abs() < 1e-9);
    assert_eq!(radius, 100.);
}
#[test]
fn rectangular_marked_translation_imports_and_tracks_both_measurements() {
    let bytes = fs::read("tests/fixtures/rectangular_marked_translation_original.gsp").unwrap();
    let native = Document::from_bytes(&bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(
        native.objects[7].kind,
        Kind::RectangularMarkedTranslatedPoint(5, 6, 7)
    );
    assert_eq!(native.position(8), Some(Point { x: 200., y: 160. }));
    assert_eq!(native.encode().unwrap(), bytes);

    let mut created = Document::new();
    let h_start = created.add(Kind::Free(Point { x: 150., y: 120. }));
    let h_end = created.add(Kind::Free(Point { x: 250., y: 120. }));
    let v_start = created.add(Kind::Free(Point { x: 150., y: 180. }));
    let v_end = created.add(Kind::Free(Point { x: 150., y: 230. }));
    let source = created.add(Kind::Free(Point { x: 100., y: 210. }));
    let horizontal = created.add(Kind::PointDistance(h_start, h_end));
    let vertical = created.add(Kind::PointDistance(v_start, v_end));
    let image = created.add(Kind::RectangularMarkedTranslatedPoint(
        source, horizontal, vertical,
    ));
    assert_eq!(created.position(image), Some(Point { x: 200., y: 160. }));
    let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[7].kind, created.objects[7].kind);
    assert!(reopened.move_point(h_end, Point { x: 300., y: 120. }));
    assert!(reopened.move_point(v_end, Point { x: 150., y: 250. }));
    assert_eq!(reopened.position(image), Some(Point { x: 250., y: 140. }));
}
#[test]
fn fixed_rectangular_translation_imports_and_tracks_source() {
    let bytes = fs::read("tests/fixtures/rectangular_fixed_translation_original.gsp").unwrap();
    let native = Document::from_bytes(&bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(
        native.objects[7].kind,
        Kind::FixedRectangularTranslatedPoint(5, 37.79527559055118, 2., 37.79527559055118, -0.5)
    );
    let expected = Point {
        x: 100. + 2. * 37.79527559055118,
        y: 210. + 0.5 * 37.79527559055118,
    };
    let position = native.position(8).unwrap();
    assert!((position.x - expected.x).abs() < 1e-9);
    assert!((position.y - expected.y).abs() < 1e-9);
    assert_eq!(native.encode().unwrap(), bytes);

    let mut created = Document::new();
    let source = created.add(Kind::Free(Point { x: 100., y: 210. }));
    let image = created.add(Kind::FixedRectangularTranslatedPoint(
        source,
        37.79527559055118,
        2.,
        37.79527559055118,
        -0.5,
    ));
    let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[1].kind, created.objects[1].kind);
    assert!(reopened.move_point(source, Point { x: 120., y: 200. }));
    let moved = reopened.position(image).unwrap();
    assert!((moved.x - expected.x - 20.).abs() < 1e-9);
    assert!((moved.y - expected.y + 10.).abs() < 1e-9);
}
#[test]
fn mixed_rectangular_translation_imports_and_tracks_marked_axis() {
    for (path, expected_kind, expected_position) in [
        (
            "tests/fixtures/rectangular_mixed_horizontal_original.gsp",
            Kind::MixedRectangularTranslatedPoint(5, 7, 2., true),
            Point {
                x: 100. + 2. * 37.79527559055118,
                y: 160.,
            },
        ),
        (
            "tests/fixtures/rectangular_mixed_vertical_original.gsp",
            Kind::MixedRectangularTranslatedPoint(5, 6, 2., false),
            Point {
                x: 200.,
                y: 210. - 2. * 37.79527559055118,
            },
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view, "{path}");
        assert_eq!(native.objects[7].kind, expected_kind, "{path}");
        let position = native.position(8).unwrap();
        assert!((position.x - expected_position.x).abs() < 1e-9, "{path}");
        assert!((position.y - expected_position.y).abs() < 1e-9, "{path}");
        assert_eq!(native.encode().unwrap(), bytes, "{path}");
    }

    let mut created = Document::new();
    let h_start = created.add(Kind::Free(Point { x: 150., y: 120. }));
    let h_end = created.add(Kind::Free(Point { x: 250., y: 120. }));
    let v_start = created.add(Kind::Free(Point { x: 150., y: 180. }));
    let v_end = created.add(Kind::Free(Point { x: 150., y: 230. }));
    let source = created.add(Kind::Free(Point { x: 100., y: 210. }));
    let horizontal = created.add(Kind::PointDistance(h_start, h_end));
    let vertical = created.add(Kind::PointDistance(v_start, v_end));
    let fixed_x = created.add(Kind::MixedRectangularTranslatedPoint(
        source, vertical, 2., true,
    ));
    let fixed_y = created.add(Kind::MixedRectangularTranslatedPoint(
        source, horizontal, 2., false,
    ));
    let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[7].kind, created.objects[7].kind);
    assert_eq!(reopened.objects[8].kind, created.objects[8].kind);
    assert!(reopened.move_point(h_end, Point { x: 300., y: 120. }));
    assert!(reopened.move_point(v_end, Point { x: 150., y: 250. }));
    let x = reopened.position(fixed_x).unwrap();
    let y = reopened.position(fixed_y).unwrap();
    assert!((x.x - (100. + 2. * 37.79527559055118)).abs() < 1e-9);
    assert_eq!(x.y, 140.);
    assert_eq!(y.x, 250.);
    assert!((y.y - (210. - 2. * 37.79527559055118)).abs() < 1e-9);
}
#[test]
fn polar_segment_translation_imports_and_tracks_endpoints() {
    let bytes = fs::read("tests/fixtures/polar_segment_translation_original.gsp").unwrap();
    let native = Document::from_bytes(&bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(
        native.objects[3].kind,
        Kind::PolarTranslatedSegment(3, 90., 37.79527559055118, 1.)
    );
    let (a, b, mode) = native.line_points(4).unwrap();
    assert_eq!(mode, LineMode::Segment);
    assert!((a.x - 100.).abs() < 1e-9 && (a.y - (200. - 37.79527559055118)).abs() < 1e-9);
    assert!((b.x - 200.).abs() < 1e-9 && (b.y - (200. - 37.79527559055118)).abs() < 1e-9);
    assert_eq!(native.encode().unwrap(), bytes);

    let mut created = Document::new();
    let start = created.add(Kind::Free(Point { x: 100., y: 200. }));
    let end = created.add(Kind::Free(Point { x: 200., y: 200. }));
    let segment = created.add(Kind::Segment(start, end));
    let translated = created.add(Kind::PolarTranslatedSegment(
        segment,
        90.,
        37.79527559055118,
        1.,
    ));
    let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[3].kind, created.objects[3].kind);
    assert!(reopened.move_point(end, Point { x: 230., y: 220. }));
    let (a, b, mode) = reopened.line_points(translated).unwrap();
    assert_eq!(mode, LineMode::Segment);
    assert!((a.x - 100.).abs() < 1e-9 && (a.y - (200. - 37.79527559055118)).abs() < 1e-9);
    assert!((b.x - 230.).abs() < 1e-9 && (b.y - (220. - 37.79527559055118)).abs() < 1e-9);
}
#[test]
fn polar_circle_translation_imports_and_tracks_center_and_radius() {
    let bytes = fs::read("tests/fixtures/polar_circle_translation_original.gsp").unwrap();
    let native = Document::from_bytes(&bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(
        native.objects[3].kind,
        Kind::PolarTranslatedCircle(3, 90., 37.79527559055118, 1.)
    );
    let (center, radius) = native.circle_points(4).unwrap();
    assert!((center.x - 100.).abs() < 1e-9);
    assert!((center.y - (200. - 37.79527559055118)).abs() < 1e-9);
    assert_eq!(radius, 50.);
    assert_eq!(native.encode().unwrap(), bytes);

    let mut created = Document::new();
    let center = created.add(Kind::Free(Point { x: 100., y: 200. }));
    let radius_point = created.add(Kind::Free(Point { x: 150., y: 200. }));
    let circle = created.add(Kind::Circle(center, radius_point));
    let moved = created.add(Kind::PolarTranslatedCircle(
        circle,
        90.,
        37.79527559055118,
        1.,
    ));
    let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[3].kind, created.objects[3].kind);
    assert!(reopened.move_point(radius_point, Point { x: 160., y: 200. }));
    assert!(reopened.move_point(center, Point { x: 120., y: 210. }));
    let (new_center, new_radius) = reopened.circle_points(moved).unwrap();
    assert!((new_center.x - 120.).abs() < 1e-9);
    assert!((new_center.y - (210. - 37.79527559055118)).abs() < 1e-9);
    assert!((new_radius - 40_f64.hypot(10.)).abs() < 1e-9);
}

#[test]
fn rectangular_segment_translation_imports_and_tracks_endpoints() {
    let bytes = fs::read("tests/fixtures/rectangular_segment_translation_original.gsp").unwrap();
    let native = Document::from_bytes(&bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(
        native.objects[3].kind,
        Kind::FixedRectangularTranslatedSegment(3, 37.79527559055118, 1., 37.79527559055118, 1.)
    );
    let (a, b, mode) = native.line_points(4).unwrap();
    assert_eq!(mode, LineMode::Segment);
    assert!((a.x - (100. + 37.79527559055118)).abs() < 1e-9);
    assert!((a.y - (200. - 37.79527559055118)).abs() < 1e-9);
    assert!((b.x - (200. + 37.79527559055118)).abs() < 1e-9);
    assert!((b.y - (200. - 37.79527559055118)).abs() < 1e-9);
    assert_eq!(native.encode().unwrap(), bytes);

    let mut created = Document::new();
    let start = created.add(Kind::Free(Point { x: 100., y: 200. }));
    let end = created.add(Kind::Free(Point { x: 200., y: 200. }));
    let segment = created.add(Kind::Segment(start, end));
    let translated = created.add(Kind::FixedRectangularTranslatedSegment(
        segment,
        37.79527559055118,
        1.,
        37.79527559055118,
        1.,
    ));
    let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(reopened.move_point(end, Point { x: 230., y: 220. }));
    let (_, end, mode) = reopened.line_points(translated).unwrap();
    assert_eq!(mode, LineMode::Segment);
    assert!((end.x - (230. + 37.79527559055118)).abs() < 1e-9);
    assert!((end.y - (220. - 37.79527559055118)).abs() < 1e-9);
}
#[test]
fn rectangular_circle_and_vector_shapes_import_and_track_sources() {
    for (path, expected_kind) in [
        (
            "tests/fixtures/rectangular_circle_translation_original.gsp",
            Kind::FixedRectangularTranslatedCircle(3, 37.79527559055118, 1., 37.79527559055118, 1.),
        ),
        (
            "tests/fixtures/vector_segment_translation_original.gsp",
            Kind::VectorTranslatedSegment(3, 1, 2),
        ),
        (
            "tests/fixtures/vector_circle_translation_original.gsp",
            Kind::VectorTranslatedCircle(6, 1, 2),
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view, "{path}");
        assert_eq!(native.objects.last().unwrap().kind, expected_kind, "{path}");
        assert_eq!(native.encode().unwrap(), bytes, "{path}");
    }

    let mut created = Document::new();
    let start = created.add(Kind::Free(Point { x: 150., y: 120. }));
    let end = created.add(Kind::Free(Point { x: 220., y: 130. }));
    let segment = created.add(Kind::Segment(start, end));
    let center = created.add(Kind::Free(Point { x: 100., y: 200. }));
    let radius_point = created.add(Kind::Free(Point { x: 150., y: 200. }));
    let circle = created.add(Kind::Circle(center, radius_point));
    let vector_segment = created.add(Kind::VectorTranslatedSegment(segment, start, end));
    let vector_circle = created.add(Kind::VectorTranslatedCircle(circle, start, end));
    let rectangular_circle = created.add(Kind::FixedRectangularTranslatedCircle(
        circle,
        37.79527559055118,
        1.,
        37.79527559055118,
        1.,
    ));
    let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(reopened.move_point(end, Point { x: 250., y: 140. }));
    assert!(reopened.move_point(radius_point, Point { x: 160., y: 200. }));
    let (segment_a, segment_b, mode) = reopened.line_points(vector_segment).unwrap();
    assert_eq!(mode, LineMode::Segment);
    assert_eq!(segment_a, Point { x: 250., y: 140. });
    assert_eq!(segment_b, Point { x: 350., y: 160. });
    let (vector_center, vector_radius) = reopened.circle_points(vector_circle).unwrap();
    assert_eq!(vector_center, Point { x: 200., y: 220. });
    assert_eq!(vector_radius, 60.);
    let (rect_center, rect_radius) = reopened.circle_points(rectangular_circle).unwrap();
    assert!((rect_center.x - (100. + 37.79527559055118)).abs() < 1e-9);
    assert!((rect_center.y - (200. - 37.79527559055118)).abs() < 1e-9);
    assert_eq!(rect_radius, 60.);
}

#[test]
fn marked_and_mixed_rectangular_shapes_track_distances_and_round_trip() {
    let mut doc = Document::new();
    let origin = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let h = doc.add(Kind::Free(Point { x: 100., y: 0. }));
    let v = doc.add(Kind::Free(Point { x: 0., y: 50. }));
    let horizontal = doc.add(Kind::PointDistance(origin, h));
    let vertical = doc.add(Kind::PointDistance(origin, v));
    let a = doc.add(Kind::Free(Point { x: 200., y: 300. }));
    let b = doc.add(Kind::Free(Point { x: 250., y: 300. }));
    let segment = doc.add(Kind::Segment(a, b));
    let circle = doc.add(Kind::Circle(a, b));
    let kinds = [
        Kind::RectangularMarkedTranslatedSegment(segment, horizontal, vertical),
        Kind::RectangularMarkedTranslatedCircle(circle, horizontal, vertical),
        Kind::MixedRectangularTranslatedSegment(segment, vertical, -2., true),
        Kind::MixedRectangularTranslatedCircle(circle, vertical, -2., true),
        Kind::MixedRectangularTranslatedSegment(segment, horizontal, -0.5, false),
        Kind::MixedRectangularTranslatedCircle(circle, horizontal, -0.5, false),
    ];
    let ids = kinds
        .iter()
        .map(|kind| doc.add(kind.clone()))
        .collect::<Vec<_>>();
    let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    for (&id, kind) in ids.iter().zip(&kinds) {
        assert_eq!(&reopened.objects[id - 1].kind, kind);
    }
    assert!(reopened.move_point(h, Point { x: 150., y: 0. }));
    assert!(reopened.move_point(v, Point { x: 0., y: 80. }));
    let offsets = [
        (150., -80.),
        (-2. * 37.79527559055118, -80.),
        (150., 0.5 * 37.79527559055118),
    ];
    for (pair, (dx, dy)) in ids.chunks(2).zip(offsets) {
        let (start, end, mode) = reopened.line_points(pair[0]).unwrap();
        let (center, radius) = reopened.circle_points(pair[1]).unwrap();
        assert_eq!(mode, LineMode::Segment);
        assert!((start.x - 200. - dx).abs() < 1e-9);
        assert!((start.y - 300. - dy).abs() < 1e-9);
        assert_eq!(center, start);
        assert!((end.x - start.x - 50.).abs() < 1e-9);
        assert_eq!(end.y, start.y);
        assert_eq!(radius, 50.);
    }
    let moved = Document::from_bytes(&reopened.encode().unwrap(), None).unwrap();
    for &id in &ids {
        assert_eq!(moved.line_points(id), reopened.line_points(id));
        assert_eq!(moved.circle_points(id), reopened.circle_points(id));
    }
    // Clipboard remaps both shape sources and marked values to the copied parents.
    for &id in &ids {
        reopened.selected.clear();
        reopened.selected.insert(id);
        let clipboard = reopened.copy_selected().unwrap();
        let mut pasted = Document::new();
        let result = pasted.paste(&clipboard).unwrap();
        let image = *result.last().unwrap();
        assert!(pasted.line_points(image).is_some() || pasted.circle_points(image).is_some());
        assert!(
            !Document::from_bytes(&pasted.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
    reopened.selected.clear();
    reopened.selected.insert(vertical);
    let removed = reopened.clear_selected().unwrap();
    assert!(ids[..4].iter().all(|id| removed.contains(id)));
    assert!(ids[4..].iter().all(|id| !removed.contains(id)));
}

#[test]
fn unavailable_marked_lengths_suppress_shapes_without_blocking_save() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/quantity_functions_native.gsp"),
        None,
    )
    .unwrap();
    doc.set_parameter_value(6, 1.).unwrap();
    let length = doc
        .add_multi_calculation(&[5, 6], "v1*v2", (10, 5))
        .unwrap();
    let a = doc.add(Kind::Free(Point { x: 200., y: 300. }));
    let b = doc.add(Kind::Free(Point { x: 250., y: 300. }));
    let segment = doc.add(Kind::Segment(a, b));
    let circle = doc.add(Kind::Circle(a, b));
    let kinds = [
        Kind::MarkedDistanceTranslatedSegment(segment, length, 0.),
        Kind::MarkedDistanceTranslatedCircle(circle, length, 0.),
        Kind::RectangularMarkedTranslatedSegment(segment, length, length),
        Kind::RectangularMarkedTranslatedCircle(circle, length, length),
        Kind::MixedRectangularTranslatedSegment(segment, length, 1., true),
        Kind::MixedRectangularTranslatedCircle(circle, length, 1., false),
    ];
    let ids = kinds
        .into_iter()
        .map(|kind| doc.add(kind))
        .collect::<Vec<_>>();
    doc.set_parameter_value(6, -1.).unwrap();
    for &id in &ids {
        assert!(doc.line_points(id).is_none() && doc.circle_points(id).is_none());
    }
    let mut reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    reopened.set_parameter_value(6, 2.).unwrap();
    for &id in &ids {
        assert!(reopened.line_points(id).is_some() || reopened.circle_points(id).is_some());
    }
}

#[test]
fn native_verified_rectangular_shapes_preserve_records_and_offsets() {
    let bytes =
        include_bytes!("../../../tests/fixtures/rectangular_shapes_translation_verified.gsp");
    let native = Document::from_bytes(bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), bytes);
    assert_eq!(
        native.objects[9].kind,
        Kind::RectangularMarkedTranslatedSegment(8, 4, 5)
    );
    assert_eq!(
        native.objects[10].kind,
        Kind::RectangularMarkedTranslatedCircle(9, 4, 5)
    );
    assert_eq!(
        native.objects[11].kind,
        Kind::MixedRectangularTranslatedSegment(8, 5, -2., true)
    );
    assert_eq!(
        native.objects[12].kind,
        Kind::MixedRectangularTranslatedCircle(9, 5, -2., true)
    );
    assert_eq!(
        native.objects[13].kind,
        Kind::MixedRectangularTranslatedSegment(8, 4, -0.5, false)
    );
    assert_eq!(
        native.objects[14].kind,
        Kind::MixedRectangularTranslatedCircle(9, 4, -0.5, false)
    );
    let (a, b, mode) = native.line_points(10).unwrap();
    assert_eq!(mode, LineMode::Segment);
    assert_eq!(a, Point { x: 450., y: 220. });
    assert_eq!(b, Point { x: 500., y: 220. });
    assert_eq!(native.circle_points(11), Some((a, 50.)));
}

#[test]
fn corpus_vector_translated_points_retain_names_and_follow_live_vector() {
    let source = include_bytes!("../../../tests/fixtures/labeled_vector_points_corpus_original.gsp");
    let imported = Document::from_bytes(source, None).unwrap();
    for (id, kind, name, position) in [
        (8, Kind::VectorTranslatedPoint(2, 1, 7), "B'", Point { x: 467., y: 250. }),
        (10, Kind::VectorTranslatedPoint(4, 1, 7), "C'", Point { x: 404., y: 161. }),
    ] {
        assert_eq!(imported.objects[id - 1].kind, kind);
        assert_eq!(imported.objects[id - 1].label.as_ref().unwrap().name_utf8().unwrap(), name);
        assert_eq!(imported.position(id), Some(position));
    }
    assert_eq!(imported.encode().unwrap(), source);

    let bytes = include_bytes!("../../../tests/fixtures/labeled_vector_points_native.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert!(doc.move_point(7, Point { x: 318., y: 227. }));
    assert_eq!(doc.position(8), Some(Point { x: 487., y: 240. }));
    assert_eq!(doc.position(10), Some(Point { x: 424., y: 151. }));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[7].label, doc.objects[7].label);
    assert_eq!(reopened.position(8), doc.position(8));

    doc.selected.insert(8);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let image = *pasted.last().unwrap();
    assert_eq!(doc.objects[image - 1].label.as_ref().unwrap().name_utf8().unwrap(), "B'");
    assert_eq!(doc.position(image), Some(Point { x: 507., y: 260. }));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[image - 1].label, doc.objects[image - 1].label);
    assert_eq!(reopened.position(image), doc.position(image));
    doc.selected.clear();
    doc.selected.insert(7);
    assert!(doc.clear_selected().unwrap().contains(&8));
}

#[test]
fn vector_translations_of_rotated_shapes_follow_edits_constraints_and_copy() {
    for (bytes, image, constrained) in [
        (include_bytes!("../../../tests/fixtures/vector_rotated_segment_native.gsp").as_slice(),
            Kind::VectorTranslatedSegment(5, 1, 2), Kind::PointOnSegment(6, 0.25)),
        (include_bytes!("../../../tests/fixtures/vector_rotated_circle_native.gsp").as_slice(),
            Kind::VectorTranslatedCircle(5, 1, 2), Kind::PointOnCircle(6, 0., 1.)),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects[5].kind, image);
        assert_eq!(doc.encode().unwrap(), bytes);
        if matches!(image, Kind::VectorTranslatedSegment(..)) {
            let (a,b,mode) = doc.line_points(6).unwrap();
            assert!(a.distance(Point { x: 210., y: 150. }) < 1e-12);
            assert!(b.distance(Point { x: 210., y: 110. }) < 1e-12);
            assert_eq!(mode, LineMode::Segment);
        } else {
            let (center,radius) = doc.circle_points(6).unwrap();
            assert!(center.distance(Point { x: 210., y: 150. }) < 1e-12);
            assert!((radius - 40.).abs() < 1e-12);
        }
        let point = doc.add(constrained);
        let value = doc.add(Kind::ValueEquation {
            kind: ValueEquationKind::PointValue, first: point, second: 6,
        });
        assert!(matches!(doc.value_equation_value(value), Some(EquationValue::PointValue(v)) if (v-0.25).abs()<1e-12));
        let before = doc.position(point).unwrap();
        assert!(doc.move_point(3, Point { x: 250., y: 190. }));
        assert_ne!(doc.position(point).unwrap(), before);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[5].kind, image);
        assert_eq!(reopened.position(point), doc.position(point));
        doc.selected.insert(6);
        let clipboard = doc.copy_selected().unwrap();
        let pasted = doc.paste(&clipboard).unwrap();
        let copied = *pasted.last().unwrap();
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[copied - 1].kind, doc.objects[copied - 1].kind);
        assert_eq!(reopened.line_points(copied), doc.line_points(copied));
        assert_eq!(reopened.circle_points(copied), doc.circle_points(copied));
        doc.selected.clear();
        doc.selected.insert(5);
        let deleted = doc.clear_selected().unwrap();
        assert!(deleted.contains(&6) && deleted.contains(&point) && deleted.contains(&value));
    }
}
