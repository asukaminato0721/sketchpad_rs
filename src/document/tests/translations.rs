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
