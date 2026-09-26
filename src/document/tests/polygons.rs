use super::*;

#[test]
fn reflected_polygon_imports_and_tracks_mirror() {
    for (path, polygon, mirror, image) in [
        ("tests/fixtures/reflection_polygon_original.gsp", 7, 3, 8),
        ("tests/fixtures/reflection.gsp", 9, 4, 13),
    ] {
        let bytes = fs::read(path).unwrap();
        let native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view, "{path}");
        assert_eq!(
            native.objects[image - 1].kind,
            Kind::ReflectedPolygon(polygon, mirror)
        );
        assert_eq!(native.encode().unwrap(), bytes);
        assert_eq!(
            native.objects[image - 1].visual_style,
            native.objects[polygon - 1].visual_style
        );
    }

    let mut native =
        Document::open(Path::new("tests/fixtures/reflection_polygon_original.gsp")).unwrap();
    let before = native.polygon_points(8).unwrap();
    assert_eq!(
        before,
        vec![
            Point { x: 100., y: 130. },
            Point { x: 60., y: 180. },
            Point { x: 100., y: 230. },
        ]
    );
    assert!(native.move_point(2, Point { x: 170., y: 230. }));
    assert_ne!(native.polygon_points(8).unwrap(), before);
    assert_eq!(
        native.encode().unwrap(),
        fs::read("tests/fixtures/reflection_polygon_mirror_dragged_by_wine.gsp").unwrap()
    );

    let mut created = Document::new();
    let mirror_a = created.add(Kind::Free(Point { x: 150., y: 100. }));
    let mirror_b = created.add(Kind::Free(Point { x: 150., y: 230. }));
    let mirror = created.add(Kind::Segment(mirror_a, mirror_b));
    let a = created.add(Kind::Free(Point { x: 200., y: 130. }));
    let b = created.add(Kind::Free(Point { x: 240., y: 180. }));
    let c = created.add(Kind::Free(Point { x: 200., y: 230. }));
    let polygon = created.add(Kind::Polygon(vec![a, b, c]));
    let image = created.add(Kind::ReflectedPolygon(polygon, mirror));
    assert_eq!(
        created.objects[image - 1].visual_style,
        created.objects[polygon - 1].visual_style
    );
    let encoded = created.encode().unwrap();
    let marker = records(&encoded)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x2328)
        .unwrap();
    assert_eq!(u32_at(&encoded, marker.data.start), image as u32);
    assert!(created.move_point(mirror_b, Point { x: 170., y: 230. }));
    let moved = created.encode().unwrap();
    let marker = records(&moved)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x2328)
        .unwrap();
    assert_eq!(u32_at(&moved, marker.data.start), mirror_b as u32);
    let reopened = Document::from_bytes(&moved, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.polygon_points(image),
        created.polygon_points(image)
    );
}
#[test]
fn transformed_polygon_fixtures_import_and_track_sources() {
    for (path, image, expected) in [
        (
            "tests/fixtures/fixed_rotation_polygon_original.gsp",
            6,
            Kind::RotatedPolygon(5, 1, 90.),
        ),
        (
            "tests/fixtures/angle_rotation_polygon_original.gsp",
            9,
            Kind::AngleRotatedPolygon(8, 1, 2, 3, 4),
        ),
        (
            "tests/fixtures/fixed_dilation_polygon_original.gsp",
            6,
            Kind::ScaledPolygon(5, 1, 0.5),
        ),
        (
            "tests/fixtures/ratio_dilation_polygon_original.gsp",
            9,
            Kind::RatioScaledPolygon(8, 1, 2, 3, 4),
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let mut document = Document::from_bytes(&bytes, None).unwrap();
        assert!(!document.partial_view, "{path}");
        assert_eq!(document.objects[image - 1].kind, expected);
        assert_eq!(document.encode().unwrap(), bytes, "{path}");
        assert_eq!(
            document.objects[image - 1].visual_style,
            document.objects[image - 2].visual_style
        );
        let before = document.polygon_points(image).unwrap();
        assert!(document.move_point(if image == 6 { 2 } else { 4 }, Point { x: 175., y: 170. }));
        assert_ne!(document.polygon_points(image).unwrap(), before);
        assert!(
            !Document::from_bytes(&document.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}
#[test]
fn vector_translated_polygon_imports_and_tracks_vector() {
    let bytes = fs::read("tests/fixtures/vector_translation_polygon_original.gsp").unwrap();
    let mut document = Document::from_bytes(&bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(
        document.objects[7].kind,
        Kind::VectorTranslatedPolygon(7, 1, 2)
    );
    assert_eq!(document.encode().unwrap(), bytes);
    assert_eq!(
        document.polygon_points(8),
        Some(vec![
            Point { x: 170., y: 210. },
            Point { x: 210., y: 210. },
            Point { x: 190., y: 250. },
        ])
    );
    assert!(document.move_point(2, Point { x: 250., y: 130. }));
    assert_eq!(
        document.polygon_points(8).unwrap()[0],
        Point { x: 200., y: 210. }
    );
    assert!(
        !Document::from_bytes(&document.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
#[test]
fn fixed_translated_polygon_fixtures_import_and_track_source() {
    for (path, expected) in [
        (
            "tests/fixtures/polar_translation_polygon_original.gsp",
            Kind::PolarTranslatedPolygon(7, 90., 37.79527559055118, 1.),
        ),
        (
            "tests/fixtures/rectangular_translation_polygon_original.gsp",
            Kind::FixedRectangularTranslatedPolygon(
                7,
                37.79527559055118,
                2.,
                37.79527559055118,
                -0.5,
            ),
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let mut document = Document::from_bytes(&bytes, None).unwrap();
        assert!(!document.partial_view, "{path}");
        assert_eq!(document.objects[7].kind, expected, "{path}");
        assert_eq!(document.encode().unwrap(), bytes, "{path}");
        let before = document.polygon_points(8).unwrap();
        assert!(document.move_point(5, Point { x: 160., y: 200. }));
        assert_ne!(document.polygon_points(8).unwrap(), before);
        assert!(
            !Document::from_bytes(&document.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}
#[test]
fn marked_distance_polygon_imports_and_tracks_measurement() {
    let bytes =
        fs::read("tests/fixtures/marked_distance_polygon_translation_original.gsp").unwrap();
    let mut document = Document::from_bytes(&bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(
        document.objects[7].kind,
        Kind::MarkedDistanceTranslatedPolygon(7, 6, 90.)
    );
    assert_eq!(document.encode().unwrap(), bytes);
    let before = document.polygon_points(8).unwrap();
    assert_eq!(before[0], Point { x: 100., y: 90. });
    assert!(document.move_point(2, Point { x: 300., y: 120. }));
    let moved = document.polygon_points(8).unwrap()[0];
    assert!((moved.x - 100.).abs() < 1e-9 && (moved.y - 40.).abs() < 1e-9);
    assert!(
        !Document::from_bytes(&document.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
#[test]
fn polygon_area_imports_and_tracks_vertices() {
    let bytes = fs::read("tests/fixtures/polygon_area_original.gsp").unwrap();
    let mut document = Document::from_bytes(&bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(document.objects[5].kind, Kind::PolygonArea(5));
    assert_eq!(document.encode().unwrap(), bytes);
    assert!(
        (document.polygon_area_cm2(6).unwrap() - 800. / 37.79527559055118_f64.powi(2)).abs()
            < 1e-10
    );
    assert!(document.move_point(4, Point { x: 190., y: 250. }));
    assert!(
        (document.polygon_area_cm2(6).unwrap() - 1200. / 37.79527559055118_f64.powi(2)).abs()
            < 1e-10
    );
    assert!(
        !Document::from_bytes(&document.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
#[test]
fn transformed_polygon_measurements_import_and_track_source() {
    for (path, expected) in [
        (
            "tests/fixtures/transformed_polygon_perimeter_original.gsp",
            Kind::PolygonPerimeter(6),
        ),
        (
            "tests/fixtures/transformed_polygon_area_original.gsp",
            Kind::PolygonArea(6),
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let mut document = Document::from_bytes(&bytes, None).unwrap();
        assert!(!document.partial_view, "{path}");
        assert_eq!(document.objects[6].kind, expected, "{path}");
        assert_eq!(document.encode().unwrap(), bytes, "{path}");
        let before = if matches!(expected, Kind::PolygonArea(_)) {
            document.polygon_area_cm2(7).unwrap()
        } else {
            document.perimeter_cm(7).unwrap()
        };
        assert!(document.move_point(4, Point { x: 190., y: 250. }));
        let after = if matches!(expected, Kind::PolygonArea(_)) {
            document.polygon_area_cm2(7).unwrap()
        } else {
            document.perimeter_cm(7).unwrap()
        };
        assert_ne!(before, after);
        assert!(
            !Document::from_bytes(&document.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}
#[test]
fn rectangular_marked_polygon_imports_and_tracks_both_distances() {
    let bytes = fs::read("tests/fixtures/rectangular_marked_polygon_original.gsp").unwrap();
    let mut document = Document::from_bytes(&bytes, None).unwrap();
    assert!(!document.partial_view);
    assert_eq!(
        document.objects[11].kind,
        Kind::RectangularMarkedTranslatedPolygon(11, 6, 7)
    );
    assert_eq!(document.encode().unwrap(), bytes);
    assert_eq!(
        document.polygon_points(12).unwrap()[0],
        Point { x: 200., y: 140. }
    );
    assert!(document.move_point(2, Point { x: 300., y: 120. }));
    assert!(document.move_point(4, Point { x: 150., y: 250. }));
    assert_eq!(
        document.polygon_points(12).unwrap()[0],
        Point { x: 250., y: 120. }
    );
    assert!(
        !Document::from_bytes(&document.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
#[test]
fn mixed_rectangular_polygon_fixtures_import_and_track_distance() {
    for (path, expected) in [
        (
            "tests/fixtures/rectangular_mixed_polygon_horizontal_original.gsp",
            Kind::MixedRectangularTranslatedPolygon(11, 7, 2., true),
        ),
        (
            "tests/fixtures/rectangular_mixed_polygon_vertical_original.gsp",
            Kind::MixedRectangularTranslatedPolygon(11, 6, 2., false),
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let mut document = Document::from_bytes(&bytes, None).unwrap();
        assert!(!document.partial_view, "{path}");
        assert_eq!(document.objects[11].kind, expected, "{path}");
        assert_eq!(document.encode().unwrap(), bytes, "{path}");
        let before = document.polygon_points(12).unwrap();
        let id = if matches!(
            expected,
            Kind::MixedRectangularTranslatedPolygon(_, _, _, true)
        ) {
            4
        } else {
            2
        };
        assert!(document.move_point(id, Point { x: 300., y: 250. }));
        assert_ne!(document.polygon_points(12).unwrap(), before);
        assert!(
            !Document::from_bytes(&document.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}
#[test]
fn marked_ratio_dilation_imports_and_tracks_ratio_points() {
    let bytes = fs::read("tests/fixtures/ratio_dilation_original.gsp").unwrap();
    let native = Document::from_bytes(&bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(
        native.objects[5].kind,
        Kind::RatioScaledPoint(5, 1, 2, 3, 4)
    );
    assert_eq!(native.position(6), Some(Point { x: 120., y: 120. }));
    assert_eq!(native.encode().unwrap(), bytes);

    let mut created = Document::new();
    let center = created.add(Kind::Free(Point { x: 100., y: 120. }));
    let origin = created.add(Kind::Free(Point { x: 180., y: 100. }));
    let base = created.add(Kind::Free(Point { x: 230., y: 100. }));
    let target = created.add(Kind::Free(Point { x: 205., y: 100. }));
    let source = created.add(Kind::Free(Point { x: 140., y: 120. }));
    let image = created.add(Kind::RatioScaledPoint(source, center, origin, base, target));
    assert_eq!(created.position(image), Some(Point { x: 120., y: 120. }));
    let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[5].kind, created.objects[5].kind);
    assert!(reopened.move_point(target, Point { x: 255., y: 100. }));
    assert_eq!(reopened.position(image), Some(Point { x: 160., y: 120. }));
    assert!(reopened.move_point(target, Point { x: 155., y: 100. }));
    assert_eq!(reopened.position(image), Some(Point { x: 80., y: 120. }));
    assert!(
        !Document::from_bytes(&reopened.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
#[test]
fn marked_ratio_circle_dilation_imports_and_tracks_ratio_points() {
    let bytes = fs::read("tests/fixtures/ratio_dilation_circle_original.gsp").unwrap();
    let native = Document::from_bytes(&bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(
        native.objects[7].kind,
        Kind::RatioScaledCircle(7, 1, 2, 3, 4)
    );
    assert_eq!(
        native.circle_points(8),
        Some((Point { x: 120., y: 120. }, 10.))
    );
    assert_eq!(native.encode().unwrap(), bytes);

    let mut created = Document::new();
    let center = created.add(Kind::Free(Point { x: 100., y: 120. }));
    let origin = created.add(Kind::Free(Point { x: 180., y: 100. }));
    let base = created.add(Kind::Free(Point { x: 230., y: 100. }));
    let target = created.add(Kind::Free(Point { x: 205., y: 100. }));
    let source_center = created.add(Kind::Free(Point { x: 140., y: 120. }));
    let radius_point = created.add(Kind::Free(Point { x: 160., y: 120. }));
    let source = created.add(Kind::Circle(source_center, radius_point));
    let image = created.add(Kind::RatioScaledCircle(
        source, center, origin, base, target,
    ));
    assert_eq!(
        created.circle_points(image),
        Some((Point { x: 120., y: 120. }, 10.))
    );
    let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[7].kind, created.objects[7].kind);
    assert!(reopened.move_point(target, Point { x: 255., y: 100. }));
    assert_eq!(
        reopened.circle_points(image),
        Some((Point { x: 160., y: 120. }, 30.))
    );
    assert!(reopened.move_point(target, Point { x: 155., y: 100. }));
    assert_eq!(
        reopened.circle_points(image),
        Some((Point { x: 80., y: 120. }, 10.))
    );
    assert!(
        !Document::from_bytes(&reopened.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
