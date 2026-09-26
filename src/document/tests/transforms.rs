use super::*;

#[test]
fn polar_translation_matches_original_and_updates() {
    let bytes = fs::read("tests/fixtures/point_translation.gsp").unwrap();
    let mut original = Document::from_bytes(&bytes, None).unwrap();
    assert!(matches!(
        original.objects[1].kind,
        Kind::PolarTranslatedPoint(1, 90., _, 1.)
    ));
    assert!(!original.partial_view);
    assert_eq!(original.encode().unwrap(), bytes);
    let source = original.position(1).unwrap();
    let translated = original.position(2).unwrap();
    assert!((translated.x - source.x).abs() < 1e-6);
    assert!((translated.y - source.y + 37.79527559055118).abs() < 1e-6);
    original.move_point(1, Point { x: 100., y: 200. });
    assert!((original.position(2).unwrap().y - 162.20472440944883).abs() < 1e-6);

    let mut created = Document::new();
    let a = created.add(Kind::Free(Point { x: 200., y: 200. }));
    let translated = created.add(Kind::PolarTranslatedPoint(a, 90., 37.79527559055118, 2.));
    let reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert_eq!(
        reopened.objects[translated - 1].kind,
        created.objects[translated - 1].kind
    );
    assert_eq!(reopened.position(translated), created.position(translated));
    assert!(!reopened.partial_view);

    let moved = Document::open(Path::new("tests/fixtures/wine_moved_translation.gsp")).unwrap();
    assert_eq!(moved.position(1), Some(Point { x: 241., y: 221. }));
    assert!(matches!(
        moved.objects[1].kind,
        Kind::PolarTranslatedPoint(1, 90., _, 1.)
    ));
    assert!((moved.position(2).unwrap().y - 183.20472440944883).abs() < 1e-6);
}
#[test]
fn reflected_point_matches_original_and_updates() {
    let bytes = fs::read("tests/fixtures/reflection.gsp").unwrap();
    let mut original = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(original.objects[13].kind, Kind::ReflectedPoint(1, 4));
    assert_eq!(original.objects[14].kind, Kind::ReflectedCircle(12, 4));
    assert_eq!(original.encode().unwrap(), bytes);
    let (a, b, _) = original.line_points(4).unwrap();
    assert_eq!(
        original.position(14),
        reflect_point(original.position(1).unwrap(), a, b)
    );
    let prior = original.position(14);
    original.move_point(1, Point { x: 300., y: 150. });
    assert_ne!(original.position(14), prior);
    let reopened = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.position(14), original.position(14));
    assert_eq!(reopened.circle_points(15), original.circle_points(15));

    let mut created = Document::new();
    let source = created.add(Kind::Free(Point { x: 50., y: 50. }));
    let a = created.add(Kind::Free(Point { x: 0., y: 100. }));
    let b = created.add(Kind::Free(Point { x: 200., y: 100. }));
    let mirror = created.add(Kind::Segment(a, b));
    let result = created.add(Kind::ReflectedPoint(source, mirror));
    assert_eq!(created.position(result), Some(Point { x: 50., y: 150. }));
    let encoded = created.encode().unwrap();
    let headers: Vec<_> = records(&encoded)
        .unwrap()
        .into_iter()
        .filter(|r| r.kind == 0x7d0)
        .collect();
    assert_eq!(&encoded[headers[4].data.clone()], &header(34));
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.objects[4].kind,
        Kind::ReflectedPoint(source, mirror)
    );
    assert_eq!(reopened.position(result), created.position(result));

    let radius_point = created.add(Kind::Free(Point { x: 70., y: 50. }));
    let circle = created.add(Kind::Circle(source, radius_point));
    let reflected_circle = created.add(Kind::ReflectedCircle(circle, mirror));
    assert_eq!(
        created.circle_points(reflected_circle),
        Some((Point { x: 50., y: 150. }, 20.))
    );
    let encoded = created.encode().unwrap();
    let headers: Vec<_> = records(&encoded)
        .unwrap()
        .into_iter()
        .filter(|r| r.kind == 0x7d0)
        .collect();
    assert_eq!(
        &encoded[headers[7].data.clone()],
        &[
            34, 0, 0, 0, 0, 0, 3, 0, 28, 0, 34, 1, 0, 128, 0, 0, 255, 255, 0, 0, 193, 189, 240,
            255, 0, 0, 0, 0
        ]
    );
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.circle_points(reflected_circle),
        created.circle_points(reflected_circle)
    );

    let wine = Document::open(Path::new("tests/fixtures/wine_moved_reflection.gsp")).unwrap();
    assert_eq!(wine.objects[4].kind, Kind::ReflectedPoint(1, 4));
    assert_eq!(wine.position(1), Some(Point { x: 241., y: 130. }));
    assert_eq!(wine.position(5), Some(Point { x: 241., y: 210. }));

    let wine_circle =
        Document::open(Path::new("tests/fixtures/wine_moved_reflected_circle.gsp")).unwrap();
    assert_eq!(wine_circle.objects[8].kind, Kind::ReflectedCircle(8, 4));
    let (center, radius) = wine_circle.circle_points(9).unwrap();
    assert_eq!(center, Point { x: 271., y: 139. });
    assert!((radius - 52.55473337388365).abs() < 1e-6);
}
#[test]
fn dilation_matches_original_and_updates() {
    let bytes = fs::read("tests/fixtures/scale.gsp").unwrap();
    let mut original = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(original.objects[3].kind, Kind::ScaledPoint(1, 3, 1. / 3.));
    assert_eq!(original.objects[12].kind, Kind::ScaledCircle(7, 1, 1. / 3.));
    assert_eq!(original.encode().unwrap(), bytes);
    let before = original.position(4);
    original.move_point(1, Point { x: 330., y: 237. });
    assert_ne!(original.position(4), before);
    assert_eq!(
        original.circle_points(13).unwrap().0,
        Point { x: 245., y: 317. }
    );

    let mut created = Document::new();
    let center = created.add(Kind::Free(Point { x: 100., y: 100. }));
    let source = created.add(Kind::Free(Point { x: 160., y: 100. }));
    let scaled = created.add(Kind::ScaledPoint(source, center, 0.5));
    let radius = created.add(Kind::Free(Point { x: 180., y: 100. }));
    let circle = created.add(Kind::Circle(source, radius));
    let scaled_circle = created.add(Kind::ScaledCircle(circle, center, 0.5));
    assert_eq!(created.position(scaled), Some(Point { x: 130., y: 100. }));
    assert_eq!(
        created.circle_points(scaled_circle),
        Some((Point { x: 130., y: 100. }, 10.))
    );
    let reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(scaled), created.position(scaled));
    assert_eq!(
        reopened.circle_points(scaled_circle),
        created.circle_points(scaled_circle)
    );

    let wine = Document::open(Path::new("tests/fixtures/wine_moved_dilation.gsp")).unwrap();
    assert_eq!(wine.position(1), Some(Point { x: 171., y: 131. }));
    assert_eq!(wine.position(3), Some(Point { x: 245.5, y: 115.5 }));
    assert_eq!(
        wine.circle_points(7),
        Some((Point { x: 235.5, y: 185.5 }, 15.))
    );
}
#[test]
fn rotation_matches_original_and_updates() {
    let bytes = fs::read("tests/fixtures/rotation.gsp").unwrap();
    let mut original = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(original.objects[4].kind, Kind::RotatedPoint(2, 1, 60.));
    assert_eq!(original.encode().unwrap(), bytes);
    let rotated = original.position(5).unwrap();
    assert!((rotated.x - 290.5).abs() < 1e-6);
    assert!((rotated.y - (188. - 157. * 60f64.to_radians().sin())).abs() < 1e-6);
    original.move_point(2, Point { x: 300., y: 188. });
    assert_ne!(original.position(5), Some(rotated));

    let mut created = Document::new();
    let center = created.add(Kind::Free(Point { x: 200., y: 180. }));
    let source = created.add(Kind::Free(Point { x: 300., y: 180. }));
    let result = created.add(Kind::RotatedPoint(source, center, 90.));
    let reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.objects[2].kind,
        Kind::RotatedPoint(source, center, 90.)
    );
    assert!((reopened.position(result).unwrap().x - 200.).abs() < 1e-6);
    assert!((reopened.position(result).unwrap().y - 80.).abs() < 1e-6);

    let wine = Document::open(Path::new("tests/fixtures/wine_moved_rotation.gsp")).unwrap();
    assert_eq!(wine.objects[2].kind, Kind::RotatedPoint(2, 1, 90.));
    assert_eq!(wine.position(2), Some(Point { x: 271., y: 160. }));
    let rotated = wine.position(3).unwrap();
    assert!((rotated.x - 180.).abs() < 1e-6);
    assert!((rotated.y - 109.).abs() < 1e-6);
}
#[test]
fn marked_angle_rotation_imports_and_tracks_angle_points() {
    let bytes = fs::read("tests/fixtures/angle_rotation_original.gsp").unwrap();
    let mut native = Document::from_bytes(&bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(
        native.objects[5].kind,
        Kind::AngleRotatedPoint(5, 1, 2, 3, 4)
    );
    assert_eq!(native.position(6), Some(Point { x: 130., y: 190. }));
    assert_eq!(native.encode().unwrap(), bytes);
    assert!(native.move_point(4, Point { x: 290., y: 190. }));
    assert_ne!(native.position(6), Some(Point { x: 130., y: 190. }));

    let mut created = Document::new();
    let center = created.add(Kind::Free(Point { x: 130., y: 150. }));
    let first = created.add(Kind::Free(Point { x: 230., y: 160. }));
    let vertex = created.add(Kind::Free(Point { x: 230., y: 220. }));
    let last = created.add(Kind::Free(Point { x: 290., y: 220. }));
    let source = created.add(Kind::Free(Point { x: 170., y: 150. }));
    let image = created.add(Kind::AngleRotatedPoint(source, center, first, vertex, last));
    assert_eq!(created.position(image), Some(Point { x: 130., y: 190. }));
    let mut reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[5].kind, created.objects[5].kind);
    assert!(reopened.move_point(last, Point { x: 290., y: 190. }));
    assert_ne!(reopened.position(image), created.position(image));
    assert!(
        !Document::from_bytes(&reopened.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
#[test]
fn marked_angle_rotation_of_shapes_imports_and_tracks_angle_points() {
    for (path, expected) in [
        (
            "tests/fixtures/angle_rotation_segment_original.gsp",
            Kind::AngleRotatedSegment(7, 1, 2, 3, 4),
        ),
        (
            "tests/fixtures/angle_rotation_circle_original.gsp",
            Kind::AngleRotatedCircle(7, 1, 2, 3, 4),
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let mut native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view, "{path}");
        assert_eq!(native.objects[7].kind, expected);
        assert_eq!(native.encode().unwrap(), bytes);
        let before = if matches!(&expected, Kind::AngleRotatedSegment(..)) {
            let (a, b, _) = native.line_points(8).unwrap();
            assert!((a.x - 90.).abs() < 1e-8 && (a.y - 190.).abs() < 1e-8);
            assert!((b.x - 90.).abs() < 1e-8 && (b.y - 230.).abs() < 1e-8);
            a
        } else {
            let (center, radius) = native.circle_points(8).unwrap();
            assert_eq!(radius, 40.);
            assert!((center.x - 90.).abs() < 1e-8 && (center.y - 190.).abs() < 1e-8);
            center
        };
        assert!(native.move_point(4, Point { x: 290., y: 190. }));
        let after = if matches!(&expected, Kind::AngleRotatedSegment(..)) {
            native.line_points(8).unwrap().0
        } else {
            native.circle_points(8).unwrap().0
        };
        assert_ne!(after, before);
    }
}
#[test]
fn fixed_rotation_of_shapes_imports_and_tracks_center() {
    for (path, expected) in [
        (
            "tests/fixtures/fixed_rotation_segment_original.gsp",
            Kind::RotatedSegment(4, 1, 90.),
        ),
        (
            "tests/fixtures/fixed_rotation_circle_original.gsp",
            Kind::RotatedCircle(4, 1, 90.),
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let mut native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view, "{path}");
        assert_eq!(native.objects[4].kind, expected);
        assert_eq!(native.encode().unwrap(), bytes);
        let rotated_center = if matches!(&expected, Kind::RotatedSegment(..)) {
            let (a, b, _) = native.line_points(5).unwrap();
            assert!((a.x - 170.).abs() < 1e-8 && (a.y - 110.).abs() < 1e-8);
            assert!((b.x - 170.).abs() < 1e-8 && (b.y - 70.).abs() < 1e-8);
            a
        } else {
            let (center, radius) = native.circle_points(5).unwrap();
            assert_eq!(radius, 40.);
            assert!((center.x - 170.).abs() < 1e-8 && (center.y - 110.).abs() < 1e-8);
            center
        };
        assert!(native.move_point(1, Point { x: 140., y: 150. }));
        let after = if matches!(&expected, Kind::RotatedSegment(..)) {
            native.line_points(5).unwrap().0
        } else {
            native.circle_points(5).unwrap().0
        };
        assert_ne!(rotated_center, after);
    }
}
#[test]
fn segment_dilations_and_reflection_import_and_track_dependencies() {
    for (path, expected) in [
        (
            "tests/fixtures/fixed_dilation_segment_original.gsp",
            Kind::ScaledSegment(4, 1, 0.5),
        ),
        (
            "tests/fixtures/ratio_dilation_segment_original.gsp",
            Kind::RatioScaledSegment(7, 1, 2, 3, 4),
        ),
        (
            "tests/fixtures/reflection_segment_original.gsp",
            Kind::ReflectedSegment(6, 3),
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let mut native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view, "{path}");
        let image = native.objects.len();
        assert_eq!(native.objects[image - 1].kind, expected);
        assert_eq!(native.encode().unwrap(), bytes);
        let (a, b, mode) = native.line_points(image).unwrap();
        assert_eq!(mode, LineMode::Segment);
        if matches!(&expected, Kind::ReflectedSegment(..)) {
            assert!((a.x - 40.).abs() < 1e-8 && (b.x - 0.).abs() < 1e-8);
            assert!(native.move_point(2, Point { x: 120., y: 220. }));
        } else {
            assert_eq!(a, Point { x: 120., y: 120. });
            assert_eq!(b, Point { x: 130., y: 120. });
            let moved = if matches!(&expected, Kind::ScaledSegment(..)) {
                native.move_point(1, Point { x: 110., y: 120. })
            } else {
                native.move_point(4, Point { x: 255., y: 100. })
            };
            assert!(moved);
        }
        assert_ne!(native.line_points(image).unwrap().0, a);
    }
}
