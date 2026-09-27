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
fn marked_distance_translation_chains_preserve_shapes_and_downstream_measurements() {
    let mut d = Document::new();
    let measure_start = d.add(Kind::Free(Point { x: 150., y: 120. }));
    let measure_end = d.add(Kind::Free(Point { x: 250., y: 120. }));
    let distance = d.add(Kind::PointDistance(measure_start, measure_end));
    let a = d.add(Kind::Free(Point { x: 100., y: 220. }));
    let b = d.add(Kind::Free(Point { x: 200., y: 220. }));
    let c = d.add(Kind::Free(Point { x: 100., y: 320. }));
    let segment = d.add(Kind::Segment(a, b));
    let circle = d.add(Kind::Circle(a, b));
    let polygon = d.add(Kind::Polygon(vec![a, b, c]));
    let segment1 = d.add(Kind::MarkedDistanceTranslatedSegment(segment, distance, 0.));
    let segment2 = d.add(Kind::MarkedDistanceTranslatedSegment(
        segment1, distance, 90.,
    ));
    let circle1 = d.add(Kind::MarkedDistanceTranslatedCircle(circle, distance, 0.));
    let circle2 = d.add(Kind::MarkedDistanceTranslatedCircle(circle1, distance, 90.));
    let polygon1 = d.add(Kind::MarkedDistanceTranslatedPolygon(polygon, distance, 0.));
    let polygon2 = d.add(Kind::MarkedDistanceTranslatedPolygon(
        polygon1, distance, 90.,
    ));
    let length = d.add(Kind::SegmentLength(segment2));
    let area = d.add(Kind::CircleArea(circle2));
    let polygon_area = d.add(Kind::PolygonArea(polygon2));
    let bytes = d.encode().unwrap();
    let mut reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.encode().unwrap(), bytes);
    for (id, original) in [(segment2, segment), (circle2, circle), (polygon2, polygon)] {
        let sequence = reopened.transform_sequence(id).unwrap();
        assert_eq!(sequence.source, original);
        assert_eq!(sequence.matrices.len(), 2);
    }
    let scale = 37.79527559055118_f64;
    for (id, expected) in [
        (length, 100. / scale),
        (area, std::f64::consts::PI * 10000. / scale.powi(2)),
        (polygon_area, 5000. / scale.powi(2)),
    ] {
        assert!((reopened.numeric_value(id).unwrap() - expected).abs() < 1e-9);
    }
    assert_eq!(reopened.numeric_value(length), d.numeric_value(length));
    assert_eq!(reopened.numeric_value(area), d.numeric_value(area));
    assert_eq!(
        reopened.numeric_value(polygon_area),
        d.numeric_value(polygon_area)
    );
    reopened.checkpoint();
    assert!(reopened.move_point(measure_end, Point { x: 300., y: 120. }));
    let expected = Point { x: 250., y: 70. };
    let (first, _, mode) = reopened.line_points(segment2).unwrap();
    assert_eq!(mode, LineMode::Segment);
    assert!(first.distance(expected) < 1e-9);
    assert!(
        reopened
            .circle_points(circle2)
            .unwrap()
            .0
            .distance(expected)
            < 1e-9
    );
    assert!(reopened.polygon_points(polygon2).unwrap()[0].distance(expected) < 1e-9);
    reopened.verify_roundtrip().unwrap();
    let edited = reopened.encode().unwrap();
    reopened.undo();
    assert_eq!(reopened.encode().unwrap(), bytes);
    reopened.redo();
    assert_eq!(reopened.encode().unwrap(), edited);
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
        .add_multi_calculation(&[5, 6], "v1/v2", (10, 5))
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
    doc.set_parameter_value(6, 0.).unwrap();
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
    let source =
        include_bytes!("../../../tests/fixtures/labeled_vector_points_corpus_original.gsp");
    let imported = Document::from_bytes(source, None).unwrap();
    for (id, kind, name, position) in [
        (
            8,
            Kind::VectorTranslatedPoint(2, 1, 7),
            "B'",
            Point { x: 467., y: 250. },
        ),
        (
            10,
            Kind::VectorTranslatedPoint(4, 1, 7),
            "C'",
            Point { x: 404., y: 161. },
        ),
    ] {
        assert_eq!(imported.objects[id - 1].kind, kind);
        assert_eq!(
            imported.objects[id - 1]
                .label
                .as_ref()
                .unwrap()
                .name_utf8()
                .unwrap(),
            name
        );
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
    assert_eq!(
        doc.objects[image - 1]
            .label
            .as_ref()
            .unwrap()
            .name_utf8()
            .unwrap(),
        "B'"
    );
    assert_eq!(doc.position(image), Some(Point { x: 507., y: 260. }));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.objects[image - 1].label,
        doc.objects[image - 1].label
    );
    assert_eq!(reopened.position(image), doc.position(image));
    doc.selected.clear();
    doc.selected.insert(7);
    assert!(doc.clear_selected().unwrap().contains(&8));
}

#[test]
fn vector_translations_of_rotated_shapes_follow_edits_constraints_and_copy() {
    for (bytes, image, constrained) in [
        (
            include_bytes!("../../../tests/fixtures/vector_rotated_segment_native.gsp").as_slice(),
            Kind::VectorTranslatedSegment(5, 1, 2),
            Kind::PointOnSegment(6, 0.25),
        ),
        (
            include_bytes!("../../../tests/fixtures/vector_rotated_circle_native.gsp").as_slice(),
            Kind::VectorTranslatedCircle(5, 1, 2),
            Kind::PointOnCircle(6, 0., 1.),
        ),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects[5].kind, image);
        assert_eq!(doc.encode().unwrap(), bytes);
        if transform_is(
            &image,
            NativeTransformFlavor::Vector,
            &[ElementType::Segment],
        ) {
            let (a, b, mode) = doc.line_points(6).unwrap();
            assert!(a.distance(Point { x: 210., y: 150. }) < 1e-12);
            assert!(b.distance(Point { x: 210., y: 110. }) < 1e-12);
            assert_eq!(mode, LineMode::Segment);
        } else {
            let (center, radius) = doc.circle_points(6).unwrap();
            assert!(center.distance(Point { x: 210., y: 150. }) < 1e-12);
            assert!((radius - 40.).abs() < 1e-12);
        }
        let point = doc.add(constrained);
        let value = doc.add(Kind::ValueEquation {
            kind: ValueEquationKind::PointValue,
            first: point,
            second: 6,
        });
        assert!(
            matches!(doc.value_equation_value(value), Some(EquationValue::PointValue(v)) if (v-0.25).abs()<1e-12)
        );
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
        assert_eq!(
            reopened.objects[copied - 1].kind,
            doc.objects[copied - 1].kind
        );
        assert_eq!(reopened.line_points(copied), doc.line_points(copied));
        assert_eq!(reopened.circle_points(copied), doc.circle_points(copied));
        doc.selected.clear();
        doc.selected.insert(5);
        let deleted = doc.clear_selected().unwrap();
        assert!(deleted.contains(&6) && deleted.contains(&point) && deleted.contains(&value));
    }
}

#[test]
fn fixed_polar_shape_chains_reuse_matrices_and_preserve_live_sources() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 100., y: 220. }));
    let b = doc.add(Kind::Free(Point { x: 200., y: 220. }));
    let c = doc.add(Kind::Free(Point { x: 100., y: 320. }));
    let segment = doc.add(Kind::Segment(a, b));
    let circle = doc.add(Kind::Circle(a, b));
    let polygon = doc.add(Kind::Polygon(vec![a, b, c]));
    let s1 = doc.add(Kind::PolarTranslatedSegment(
        segment,
        0.,
        37.79527559055118,
        1.,
    ));
    let s2 = doc.add(Kind::PolarTranslatedSegment(s1, 90., 37.79527559055118, 2.));
    let c1 = doc.add(Kind::PolarTranslatedCircle(
        circle,
        0.,
        37.79527559055118,
        1.,
    ));
    let c2 = doc.add(Kind::PolarTranslatedCircle(c1, 90., 37.79527559055118, 2.));
    let p1 = doc.add(Kind::PolarTranslatedPolygon(
        polygon,
        0.,
        37.79527559055118,
        1.,
    ));
    let p2 = doc.add(Kind::PolarTranslatedPolygon(p1, 90., 37.79527559055118, 2.));
    let length = doc.add(Kind::SegmentLength(s2));
    let area = doc.add(Kind::CircleArea(c2));
    let polygon_area = doc.add(Kind::PolygonArea(p2));
    let bytes = doc.encode().unwrap();
    let mut reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.encode().unwrap(), bytes);
    for (id, base) in [(s2, segment), (c2, circle), (p2, polygon)] {
        let sequence = reopened.transform_sequence(id).unwrap();
        assert_eq!(sequence.source, base);
        assert_eq!(sequence.matrices.len(), 2);
        assert!(
            sequence
                .apply(Point { x: 100., y: 220. })
                .unwrap()
                .distance(Point {
                    x: 137.79527559055118,
                    y: 144.40944881889763
                })
                < 1e-9
        );
    }
    reopened.checkpoint();
    assert!(reopened.move_point(b, Point { x: 300., y: 220. }));
    let scale = 37.79527559055118_f64;
    assert!((reopened.numeric_value(length).unwrap() - 200. / scale).abs() < 1e-9);
    assert!(
        (reopened.numeric_value(area).unwrap() - std::f64::consts::PI * 40000. / scale.powi(2))
            .abs()
            < 1e-9
    );
    assert!((reopened.numeric_value(polygon_area).unwrap() - 10000. / scale.powi(2)).abs() < 1e-9);
    reopened.verify_roundtrip().unwrap();
    let saved = reopened.encode().unwrap();
    reopened.undo();
    assert_eq!(reopened.encode().unwrap(), bytes);
    reopened.redo();
    assert_eq!(reopened.encode().unwrap(), saved);
}

#[test]
fn corpus_polar_distance_caches_and_radian_angles_match_exported_geometry() {
    for (bytes, id, expected_distance) in [
        (
            include_bytes!("../../../tests/fixtures/polar_radian_fractional_distance_corpus.gsp")
                .as_slice(),
            45,
            3.779527559055118,
        ),
        (
            include_bytes!("../../../tests/fixtures/polar_fractional_distance_corpus.gsp")
                .as_slice(),
            35,
            11.338582677165356,
        ),
        (
            include_bytes!("../../../tests/fixtures/polar_inch_distance_corpus.gsp").as_slice(),
            17,
            19.2,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        let object = &doc.objects[id - 1];
        let definition = object.kind.transform_definition().unwrap();
        // Some corpus pages still have undefined ancestors. Evaluate the parsed
        // literal on a free source as well, so cache semantics are tested there.
        let mut isolated = Document::new();
        let source = Point { x: 150., y: 200. };
        let source_id = isolated.add(Kind::Free(source));
        let imported = crate::document::transform_model::TransformDefinition::new(
            source_id,
            definition.element(),
            definition.transform().clone(),
            definition.wire_flavor(),
        )
        .unwrap();
        let image = isolated.add(Kind::Transformed(imported));
        let offset = object.constraint_offset.unwrap();
        let pixels = f64_at(bytes, offset + 32);
        assert!((pixels - expected_distance).abs() < 1e-12);
        let expected = Point {
            x: source.x + pixels * f64_at(bytes, offset + 12),
            y: source.y + pixels * f64_at(bytes, offset + 4),
        };
        assert!(isolated.position(image).unwrap().distance(expected) < 1e-9);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
    let rotation = include_bytes!("../../../tests/fixtures/rotation_radian_corpus.gsp");
    let doc = Document::from_bytes(rotation, None).unwrap();
    let definition = doc.objects[8].kind.transform_definition().unwrap();
    assert_eq!(definition.source(), 2);
    let source = doc.position(2).unwrap();
    let center = doc.position(1).unwrap();
    assert!(
        doc.position(9).unwrap().distance(Point {
            x: center.x,
            y: center.y - (source.x - center.x),
        }) < 1e-9
    );
    assert_eq!(doc.encode().unwrap(), rotation);
    let marked = include_bytes!("../../../tests/fixtures/marked_polar_radian_corpus.gsp");
    let doc = Document::from_bytes(marked, None).unwrap();
    let base = doc.position(1).unwrap();
    let distance = doc.distance_value_pixels(19).unwrap();
    assert!(
        doc.position(20).unwrap().distance(Point {
            x: base.x,
            y: base.y - distance
        }) < 1e-9
    );
    assert_eq!(doc.encode().unwrap(), marked);
}

#[test]
fn polar_writer_stores_total_pixels_for_fractional_negative_and_zero_distances() {
    for units in [37.79527559055118_f64, 96.] {
        for magnitude in [0.1, 0.3, 2.65, -0.3, 0., -0.] {
            let mut doc = Document::new();
            let source = doc.add(Kind::Free(Point { x: 150., y: 200. }));
            let image = doc.add(Kind::PolarTranslatedPoint(source, 23., units, magnitude));
            let bytes = doc.encode().unwrap();
            let parsed = Document::from_bytes(&bytes, None).unwrap();
            assert!(!parsed.partial_view);
            let offset = parsed.objects[image - 1].constraint_offset.unwrap();
            assert_eq!(
                f64_at(&bytes, offset + 32).to_bits(),
                (units * magnitude).to_bits()
            );
            assert_eq!(
                u32_at(&bytes, offset + 48),
                if units == 96. { 2 } else { 1 }
            );
            assert_eq!(parsed.objects[image - 1].kind, doc.objects[image - 1].kind);
            assert_eq!(parsed.position(image), doc.position(image));
            let mut parsed = parsed;
            parsed.checkpoint();
            assert!(parsed.move_point(source, Point { x: 170., y: 180. }));
            parsed.verify_roundtrip().unwrap();
            let saved = parsed.encode().unwrap();
            parsed.undo();
            assert_eq!(parsed.encode().unwrap(), bytes);
            parsed.redo();
            assert_eq!(parsed.encode().unwrap(), saved);
        }
    }
}

#[test]
fn fractional_polar_shape_chain_writer_matches_native_verified_file() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 100., y: 220. }));
    let b = doc.add(Kind::Free(Point { x: 200., y: 220. }));
    let c = doc.add(Kind::Free(Point { x: 100., y: 300. }));
    let segment = doc.add(Kind::Segment(a, b));
    let circle = doc.add(Kind::Circle(a, b));
    let polygon = doc.add(Kind::Polygon(vec![a, b, c]));
    let s = doc.add(Kind::PolarTranslatedSegment(
        segment,
        23.,
        37.79527559055118,
        0.3,
    ));
    let s2 = doc.add(Kind::PolarTranslatedSegment(
        s,
        -31.,
        37.79527559055118,
        2.65,
    ));
    let ci = doc.add(Kind::PolarTranslatedCircle(
        circle,
        23.,
        37.79527559055118,
        0.3,
    ));
    let ci2 = doc.add(Kind::PolarTranslatedCircle(
        ci,
        -31.,
        37.79527559055118,
        2.65,
    ));
    let po = doc.add(Kind::PolarTranslatedPolygon(
        polygon,
        23.,
        37.79527559055118,
        0.3,
    ));
    let po2 = doc.add(Kind::PolarTranslatedPolygon(
        po,
        -31.,
        37.79527559055118,
        2.65,
    ));
    doc.add(Kind::SegmentLength(s2));
    doc.add(Kind::CircleArea(ci2));
    doc.add(Kind::PolygonArea(po2));
    let native = include_bytes!(
        "../../../tests/fixtures/polar_translation_cache_and_chains_writer_native_verified.gsp"
    );
    assert_eq!(doc.encode().unwrap(), native);
    doc.verify_roundtrip().unwrap();
    let mut reopened = Document::from_bytes(native, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.encode().unwrap(), native);
    reopened.checkpoint();
    assert!(reopened.move_point(b, Point { x: 250., y: 220. }));
    reopened.verify_roundtrip().unwrap();
    let saved = reopened.encode().unwrap();
    reopened.undo();
    assert_eq!(reopened.encode().unwrap(), native);
    reopened.redo();
    assert_eq!(reopened.encode().unwrap(), saved);
}

#[test]
fn typed_length_parameters_and_calculations_drive_marked_transforms_live() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/numeric_typed_parameters_native_verified.gsp"),
        None,
    )
    .unwrap();
    assert!(!doc.partial_view);
    let parameter = doc.objects.iter().find(|object|
        matches!(&object.kind, Kind::Parameter(group) if group.unit() == NumericUnit::Length)
    ).unwrap().id;
    doc.set_parameter_value(parameter, 1.).unwrap();
    let literal = doc
        .add_multi_calculation(&[parameter, parameter], "0.3 cm+0*v1+0*v2", (10, 80))
        .unwrap();
    assert_eq!(
        doc.objects[literal - 1].kind.numeric_unit(),
        Some(NumericUnit::Length)
    );
    let name = doc.parameter_display_name(parameter).unwrap();
    let doubled = doc
        .add_parameter_calculation(parameter, &format!("2*{name}"), (10, 110))
        .unwrap();
    let a = doc.add(Kind::Free(Point { x: 150., y: 220. }));
    let b = doc.add(Kind::Free(Point { x: 250., y: 220. }));
    let c = doc.add(Kind::Free(Point { x: 150., y: 320. }));
    let segment = doc.add(Kind::Segment(a, b));
    let circle = doc.add(Kind::Circle(a, b));
    let polygon = doc.add(Kind::Polygon(vec![a, b, c]));
    let point = doc.add(Kind::MarkedDistanceTranslatedPoint(a, parameter, 0.));
    let s = doc.add(Kind::MarkedDistanceTranslatedSegment(
        segment, parameter, 0.,
    ));
    let ci = doc.add(Kind::MarkedDistanceTranslatedCircle(circle, parameter, 0.));
    let po = doc.add(Kind::MarkedDistanceTranslatedPolygon(
        polygon, parameter, 0.,
    ));
    let rectangle = doc.add(Kind::RectangularMarkedTranslatedPoint(
        a, parameter, literal,
    ));
    let mixed = doc.add(Kind::MixedRectangularTranslatedPoint(a, doubled, 0.5, true));
    let baseline = doc.encode().unwrap();
    let mut reopened = Document::from_bytes(&baseline, None).unwrap();
    assert!(!reopened.partial_view);
    reopened.verify_roundtrip().unwrap();
    for value in [2., 0., -1., 1.5] {
        reopened.set_parameter_value(parameter, value).unwrap();
        let expected = Point {
            x: 150. + value * 37.79527559055118,
            y: 220.,
        };
        assert!(reopened.position(point).unwrap().distance(expected) < 1e-9);
        assert!(reopened.line_points(s).unwrap().0.distance(expected) < 1e-9);
        assert!(reopened.circle_points(ci).unwrap().0.distance(expected) < 1e-9);
        assert!(reopened.polygon_points(po).unwrap()[0].distance(expected) < 1e-9);
        assert!(
            reopened.position(rectangle).unwrap().distance(Point {
                x: expected.x,
                y: 220. - 0.3 * 37.79527559055118,
            }) < 1e-9
        );
        assert!(
            reopened.position(mixed).unwrap().distance(Point {
                x: 150. + 0.5 * 37.79527559055118,
                y: 220. - 2. * value * 37.79527559055118,
            }) < 1e-9
        );
        reopened.verify_roundtrip().unwrap();
        let saved = reopened.encode().unwrap();
        assert_eq!(
            Document::from_bytes(&saved, None)
                .unwrap()
                .encode()
                .unwrap(),
            saved
        );
    }
    let edited = reopened.encode().unwrap();
    for _ in 0..4 {
        reopened.undo();
    }
    assert_eq!(reopened.encode().unwrap(), baseline);
    for _ in 0..4 {
        reopened.redo();
    }
    assert_eq!(reopened.encode().unwrap(), edited);
}

#[test]
fn native_verified_length_parameter_transforms_reopen_and_follow_edits() {
    let bytes = include_bytes!(
        "../../../tests/fixtures/typed_length_marked_transforms_writer_native_verified.gsp"
    );
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(doc.numeric_value(2), Some(2.));
    assert_eq!(
        doc.objects[1].kind.numeric_unit(),
        Some(NumericUnit::Length)
    );
    assert_eq!(
        doc.objects[3].kind.numeric_unit(),
        Some(NumericUnit::Length)
    );
    assert_eq!(
        doc.objects[4].kind.numeric_unit(),
        Some(NumericUnit::Length)
    );
    let scale = 37.79527559055118;
    let original = Point {
        x: 150. + 2. * scale,
        y: 220.,
    };
    assert!(doc.position(12).unwrap().distance(original) < 1e-9);
    assert!(doc.line_points(13).unwrap().0.distance(original) < 1e-9);
    assert!(doc.circle_points(14).unwrap().0.distance(original) < 1e-9);
    assert!(doc.polygon_points(15).unwrap()[0].distance(original) < 1e-9);
    doc.set_parameter_value(2, 1.5).unwrap();
    let expected = Point {
        x: 150. + 1.5 * scale,
        y: 220.,
    };
    assert!(doc.position(12).unwrap().distance(expected) < 1e-9);
    assert!(doc.line_points(13).unwrap().0.distance(expected) < 1e-9);
    assert!(doc.circle_points(14).unwrap().0.distance(expected) < 1e-9);
    assert!(doc.polygon_points(15).unwrap()[0].distance(expected) < 1e-9);
    assert!(
        doc.position(16).unwrap().distance(Point {
            x: expected.x,
            y: 220. - 0.3 * scale
        }) < 1e-9
    );
    assert!(
        doc.position(17).unwrap().distance(Point {
            x: 150. + 0.5 * scale,
            y: 220. - 3. * scale
        }) < 1e-9
    );
    doc.verify_roundtrip().unwrap();
    let saved = doc.encode().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), bytes);
    doc.redo();
    assert_eq!(doc.encode().unwrap(), saved);
}

#[test]
fn marked_scalar_distance_native_controls_keep_units_and_signed_domains() {
    for (bytes, value, point, image, length, expected) in [
        (include_bytes!("../../../tests/fixtures/scalar_distance_positive_native_verified.gsp").as_slice(),3.,2,3,5,Some(3.)),
        (include_bytes!("../../../tests/fixtures/scalar_distance_negative_native_verified.gsp").as_slice(),-3.,2,3,5,Some(-3.)),
        (include_bytes!("../../../tests/fixtures/scalar_distance_zero_native_verified.gsp").as_slice(),0.,2,3,5,Some(0.)),
        (include_bytes!("../../../tests/fixtures/scalar_distance_typed_isolated_negative_native_verified.gsp").as_slice(),-3.,2,3,5,Some(-3.)),
        (include_bytes!("../../../tests/fixtures/scalar_distance_typed_isolated_positive_native_verified.gsp").as_slice(),3.,2,3,5,Some(3.)),
        (include_bytes!("../../../tests/fixtures/scalar_distance_typed_negative_native_verified.gsp").as_slice(),-3.,4,5,7,Some(-3.)),
    ] {
        let d=Document::from_bytes(bytes,None).unwrap();
        assert!(!d.partial_view);
        let source=if point==2 {1} else {2};
        assert_eq!(d.numeric_value(source),Some(value));
        if d.objects[source-1].kind.numeric_unit()==Some(crate::table_records::NumericUnit::Scalar) || value<0. {
            assert_eq!(d.distance_value_pixels(source),None);
        } else {
            assert!((d.distance_value_pixels(source).unwrap()-value*37.79527559055118).abs()<1e-10);
        }
        if let Some(distance)=expected {
            assert!((d.marked_distance_value_pixels(source).unwrap()-distance*37.79527559055118).abs()<1e-10);
            assert!(d.position(image).unwrap().distance(Point{x:350.+distance*37.79527559055118,y:220.})<1e-10);
            assert!((d.numeric_value(length).unwrap()-distance.abs()).abs()<1e-10);
        } else {
            assert_eq!(d.marked_distance_value_pixels(source),None);
            assert_eq!(d.position(image),None);
        }
        d.verify_roundtrip().unwrap();
        assert_eq!(d.encode().unwrap(),bytes);
    }
}

fn scalar_distance_transform_document() -> (Document, usize, Vec<usize>) {
    let mut d = Document::new();
    let scalar = d.add_named_parameter("distance", 3., (10, 20)).unwrap();
    let vertical = d.add_calculation_expression("0.5", (10, 60)).unwrap();
    let a = d.add(Kind::Free(Point { x: 350., y: 220. }));
    let b = d.add(Kind::Free(Point { x: 450., y: 220. }));
    let c = d.add(Kind::Free(Point { x: 350., y: 300. }));
    let segment = d.add(Kind::Segment(a, b));
    let circle = d.add(Kind::Circle(a, b));
    let polygon = d.add(Kind::Polygon(vec![a, b, c]));
    let images = vec![
        d.add(Kind::MarkedDistanceTranslatedPoint(a, scalar, 0.)),
        d.add(Kind::MarkedDistanceTranslatedSegment(segment, scalar, 0.)),
        d.add(Kind::MarkedDistanceTranslatedCircle(circle, scalar, 0.)),
        d.add(Kind::MarkedDistanceTranslatedPolygon(polygon, scalar, 0.)),
        d.add(Kind::RectangularMarkedTranslatedPoint(a, scalar, vertical)),
        d.add(Kind::MixedRectangularTranslatedPoint(a, scalar, 0.5, false)),
    ];
    (d, scalar, images)
}

#[test]
fn scalar_distance_transforms_update_all_geometry_and_preserve_history() {
    let (mut d, scalar, images) = scalar_distance_transform_document();
    let before = d.encode().unwrap();
    for value in [-3., 0., 0.5] {
        d.set_parameter_value(scalar, value).unwrap();
        let expected = Point {
            x: 350. + value * 37.79527559055118,
            y: 220.,
        };
        assert!(d.position(images[0]).unwrap().distance(expected) < 1e-10);
        assert!(d.line_points(images[1]).unwrap().0.distance(expected) < 1e-10);
        assert!(d.circle_points(images[2]).unwrap().0.distance(expected) < 1e-10);
        assert!(d.polygon_points(images[3]).unwrap()[0].distance(expected) < 1e-10);
        let cartesian = Point {
            x: expected.x,
            y: 220. - 0.5 * 37.79527559055118,
        };
        assert!(d.position(images[4]).unwrap().distance(cartesian) < 1e-10);
        assert!(d.position(images[5]).unwrap().distance(cartesian) < 1e-10);
        d.verify_roundtrip().unwrap();
        let saved = d.encode().unwrap();
        d.undo();
        d.redo();
        assert_eq!(d.encode().unwrap(), saved);
    }
    d.undo();
    d.undo();
    d.undo();
    assert_eq!(d.encode().unwrap(), before);
    let undefined = d
        .add_parameter_calculation(scalar, "1/distance", (10, 100))
        .unwrap();
    let image = d.add(Kind::MarkedDistanceTranslatedPoint(3, undefined, 0.));
    d.set_parameter_value(scalar, 0.).unwrap();
    assert_eq!(d.numeric_value(undefined), None);
    assert_eq!(d.position(image), None);
    d.verify_roundtrip().unwrap();
}

#[test]
fn original_scalar_marked_distance_calculation_unblocks_cube_geometry() {
    let bytes = include_bytes!("../../../tests/fixtures/scalar_marked_distance_cube_corpus.gsp");
    let d = Document::from_bytes(bytes, None).unwrap();
    assert_eq!(
        d.objects[16].kind.numeric_unit(),
        Some(crate::table_records::NumericUnit::Scalar)
    );
    assert_eq!(d.numeric_value(17), Some(14.));
    assert_eq!(
        d.objects[22].kind,
        Kind::MarkedDistanceTranslatedPoint(14, 17, 0.)
    );
    let source = d.position(14).unwrap();
    assert!(
        d.position(23).unwrap().distance(Point {
            x: source.x + 14. * 37.79527559055118,
            y: source.y
        }) < 1e-9
    );
    assert!(
        d.position(24).unwrap().distance(Point {
            x: source.x,
            y: source.y - 14. * 37.79527559055118
        }) < 1e-8
    );
    assert!(d.numeric_value(28).is_some());
    assert!(d.position(30).is_some());
    assert_eq!(d.encode().unwrap(), bytes);
}

#[test]
fn scalar_distance_transform_writer_matches_native_saved_golden() {
    let (mut d, scalar, _) = scalar_distance_transform_document();
    d.set_parameter_value(scalar, -3.).unwrap();
    d.verify_roundtrip().unwrap();
    assert_eq!(
        d.encode().unwrap(),
        include_bytes!(
            "../../../tests/fixtures/scalar_distance_marked_transforms_writer_native_verified.gsp"
        )
    );
}

#[test]
fn scalar_distance_iteration_pins_numeric_value_and_updates_signed_steps() {
    let (mut d, scalar, images) = scalar_distance_transform_document();
    let iteration = d
        .add_point_iteration(3, images[0], crate::document::IterationDepth::Fixed(3))
        .unwrap();
    for value in [3., -3., 0.] {
        d.set_parameter_value(scalar, value).unwrap();
        let points = d.point_iteration_images(iteration).unwrap();
        assert_eq!(points.len(), 3);
        for (index, point) in points.into_iter().enumerate() {
            assert!(
                point.distance(Point {
                    x: 350. + (index + 2) as f64 * value * 37.79527559055118,
                    y: 220.
                }) < 1e-9
            );
        }
        d.verify_roundtrip().unwrap();
    }
}
