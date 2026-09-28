use super::*;

#[test]
fn polar_translation_matches_original_and_updates() {
    let bytes = fs::read("../../tests/fixtures/point_translation.gsp").unwrap();
    let mut original = Document::from_bytes(&bytes, None).unwrap();
    let definition = original.objects[1].kind.transform_definition().unwrap();
    assert!(transform_is(
        &original.objects[1].kind,
        NativeTransformFlavor::FixedPolar,
        &[ElementType::Point],
    ));
    assert_eq!(definition.source(), 1);
    assert!(transform_literal_is(
        &original.objects[1].kind,
        NativeLiteralRole::AngleDegrees,
        90.
    ));
    assert!(transform_literal_is(
        &original.objects[1].kind,
        NativeLiteralRole::PolarMagnitude,
        1.
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

    let moved = Document::open(Path::new("../../tests/fixtures/wine_moved_translation.gsp")).unwrap();
    assert_eq!(moved.position(1), Some(Point { x: 241., y: 221. }));
    let definition = moved.objects[1].kind.transform_definition().unwrap();
    assert!(transform_is(
        &moved.objects[1].kind,
        NativeTransformFlavor::FixedPolar,
        &[ElementType::Point],
    ));
    assert_eq!(definition.source(), 1);
    assert!(transform_literal_is(
        &moved.objects[1].kind,
        NativeLiteralRole::AngleDegrees,
        90.
    ));
    assert!(transform_literal_is(
        &moved.objects[1].kind,
        NativeLiteralRole::PolarMagnitude,
        1.
    ));
    assert!((moved.position(2).unwrap().y - 183.20472440944883).abs() < 1e-6);
}
#[test]
fn reflected_point_matches_original_and_updates() {
    let bytes = fs::read("../../tests/fixtures/reflection.gsp").unwrap();
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

    let wine = Document::open(Path::new("../../tests/fixtures/wine_moved_reflection.gsp")).unwrap();
    assert_eq!(wine.objects[4].kind, Kind::ReflectedPoint(1, 4));
    assert_eq!(wine.position(1), Some(Point { x: 241., y: 130. }));
    assert_eq!(wine.position(5), Some(Point { x: 241., y: 210. }));

    let wine_circle =
        Document::open(Path::new("../../tests/fixtures/wine_moved_reflected_circle.gsp")).unwrap();
    assert_eq!(wine_circle.objects[8].kind, Kind::ReflectedCircle(8, 4));
    let (center, radius) = wine_circle.circle_points(9).unwrap();
    assert_eq!(center, Point { x: 271., y: 139. });
    assert!((radius - 52.55473337388365).abs() < 1e-6);
}
#[test]
fn dilation_matches_original_and_updates() {
    let bytes = fs::read("../../tests/fixtures/scale.gsp").unwrap();
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

    let wine = Document::open(Path::new("../../tests/fixtures/wine_moved_dilation.gsp")).unwrap();
    assert_eq!(wine.position(1), Some(Point { x: 171., y: 131. }));
    assert_eq!(wine.position(3), Some(Point { x: 245.5, y: 115.5 }));
    assert_eq!(
        wine.circle_points(7),
        Some((Point { x: 235.5, y: 185.5 }, 15.))
    );
}
#[test]
fn rotation_matches_original_and_updates() {
    let bytes = fs::read("../../tests/fixtures/rotation.gsp").unwrap();
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

    let wine = Document::open(Path::new("../../tests/fixtures/wine_moved_rotation.gsp")).unwrap();
    assert_eq!(wine.objects[2].kind, Kind::RotatedPoint(2, 1, 90.));
    assert_eq!(wine.position(2), Some(Point { x: 271., y: 160. }));
    let rotated = wine.position(3).unwrap();
    assert!((rotated.x - 180.).abs() < 1e-6);
    assert!((rotated.y - 109.).abs() < 1e-6);
}
#[test]
fn marked_angle_rotation_imports_and_tracks_angle_points() {
    let bytes = fs::read("../../tests/fixtures/angle_rotation_original.gsp").unwrap();
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
            "../../tests/fixtures/angle_rotation_segment_original.gsp",
            Kind::AngleRotatedSegment(7, 1, 2, 3, 4),
        ),
        (
            "../../tests/fixtures/angle_rotation_circle_original.gsp",
            Kind::AngleRotatedCircle(7, 1, 2, 3, 4),
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let mut native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view, "{path}");
        assert_eq!(native.objects[7].kind, expected);
        assert_eq!(native.encode().unwrap(), bytes);
        let before = if transform_is(
            &expected,
            NativeTransformFlavor::PointAngleRotation,
            &[ElementType::Segment],
        ) {
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
        let after = if transform_is(
            &expected,
            NativeTransformFlavor::PointAngleRotation,
            &[ElementType::Segment],
        ) {
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
            "../../tests/fixtures/fixed_rotation_segment_original.gsp",
            Kind::RotatedSegment(4, 1, 90.),
        ),
        (
            "../../tests/fixtures/fixed_rotation_circle_original.gsp",
            Kind::RotatedCircle(4, 1, 90.),
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let mut native = Document::from_bytes(&bytes, None).unwrap();
        assert!(!native.partial_view, "{path}");
        assert_eq!(native.objects[4].kind, expected);
        assert_eq!(native.encode().unwrap(), bytes);
        let rotated_center = if transform_is(
            &expected,
            NativeTransformFlavor::FixedRotation,
            &[ElementType::Segment],
        ) {
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
        let after = if transform_is(
            &expected,
            NativeTransformFlavor::FixedRotation,
            &[ElementType::Segment],
        ) {
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
            "../../tests/fixtures/fixed_dilation_segment_original.gsp",
            Kind::ScaledSegment(4, 1, 0.5),
        ),
        (
            "../../tests/fixtures/ratio_dilation_segment_original.gsp",
            Kind::RatioScaledSegment(7, 1, 2, 3, 4),
        ),
        (
            "../../tests/fixtures/reflection_segment_original.gsp",
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
        if transform_is(
            &expected,
            NativeTransformFlavor::Reflection,
            &[ElementType::Segment],
        ) {
            assert!((a.x - 40.).abs() < 1e-8 && (b.x - 0.).abs() < 1e-8);
            assert!(native.move_point(2, Point { x: 120., y: 220. }));
        } else {
            assert_eq!(a, Point { x: 120., y: 120. });
            assert_eq!(b, Point { x: 130., y: 120. });
            let moved = if transform_is(
                &expected,
                NativeTransformFlavor::FixedDilation,
                &[ElementType::Segment],
            ) {
                native.move_point(1, Point { x: 110., y: 120. })
            } else {
                native.move_point(4, Point { x: 255., y: 100. })
            };
            assert!(moved);
        }
        assert_ne!(native.line_points(image).unwrap().0, a);
    }
}

#[test]
fn fixed_transform_chains_match_native_and_follow_source_center_and_mirror() {
    let close = |a: Point, b: Point| {
        assert!(
            (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9,
            "{a:?} != {b:?}"
        );
    };
    for name in ["segment", "circle"] {
        let bytes = if name == "segment" {
            include_bytes!("../../../../../tests/fixtures/vector_rotated_segment_native.gsp").as_slice()
        } else {
            include_bytes!("../../../../../tests/fixtures/vector_rotated_circle_native.gsp").as_slice()
        };
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        let rotated = doc.add(if name == "segment" {
            Kind::RotatedSegment(6, 1, 45.)
        } else {
            Kind::RotatedCircle(6, 1, 45.)
        });
        let scaled = doc.add(if name == "segment" {
            Kind::ScaledSegment(rotated, 1, 1.25)
        } else {
            Kind::ScaledCircle(rotated, 1, 1.25)
        });
        let mirror = doc.add(Kind::Segment(1, 3));
        let reflected = doc.add(if name == "segment" {
            Kind::ReflectedSegment(scaled, mirror)
        } else {
            Kind::ReflectedCircle(scaled, mirror)
        });
        let verify_geometry = |doc: &Document| {
            let center = doc.position(1).unwrap();
            let (ma, mb, _) = doc.line_points(mirror).unwrap();
            let image = |p: Point| {
                let dx = p.x - center.x;
                let dy = p.y - center.y;
                let sin = 45f64.to_radians().sin();
                let cos = 45f64.to_radians().cos();
                let p = Point {
                    x: center.x + 1.25 * (dx * cos + dy * sin),
                    y: center.y + 1.25 * (-dx * sin + dy * cos),
                };
                reflect_point(p, ma, mb).unwrap()
            };
            if name == "segment" {
                let (a, b, _) = doc.line_points(6).unwrap();
                let (actual_a, actual_b, mode) = doc.line_points(reflected).unwrap();
                assert_eq!(mode, LineMode::Segment);
                close(actual_a, image(a));
                close(actual_b, image(b));
            } else {
                let (c, r) = doc.circle_points(6).unwrap();
                let (actual_c, actual_r) = doc.circle_points(reflected).unwrap();
                close(actual_c, image(c));
                assert!((actual_r - r * 1.25).abs() < 1e-9);
            }
        };
        verify_geometry(&doc);
        doc.verify_roundtrip().unwrap();
        let expected = if name == "segment" {
            include_bytes!(
                "../../../../../tests/fixtures/fixed_transform_chain_segment_native_verified.gsp"
            )
            .as_slice()
        } else {
            include_bytes!(
                "../../../../../tests/fixtures/fixed_transform_chain_circle_native_verified.gsp"
            )
            .as_slice()
        };
        assert_eq!(doc.encode().unwrap(), expected);
        let mut reopened = Document::from_bytes(expected, None).unwrap();
        assert!(!reopened.partial_view);
        verify_geometry(&reopened);
        assert!(reopened.move_point(3, Point { x: 250., y: 190. }));
        verify_geometry(&reopened);
        assert!(reopened.move_point(1, Point { x: 150., y: 170. }));
        verify_geometry(&reopened);
        reopened.verify_roundtrip().unwrap();
        assert_eq!(
            reopened.encode().unwrap(),
            if name == "segment" {
                include_bytes!("../../../../../tests/fixtures/fixed_transform_chain_segment_edited_native_verified.gsp").as_slice()
            } else {
                include_bytes!("../../../../../tests/fixtures/fixed_transform_chain_circle_edited_native_verified.gsp").as_slice()
            }
        );
        if std::env::var_os("SKETCHPAD_WRITE_FIXED_CHAIN_FIXTURE").is_some() {
            fs::write(
                format!("/tmp/fixed_transform_chain_{name}_writer.gsp"),
                doc.encode().unwrap(),
            )
            .unwrap();
            fs::write(
                format!("/tmp/fixed_transform_chain_{name}_edited_writer.gsp"),
                reopened.encode().unwrap(),
            )
            .unwrap();
        }
        reopened.selected.insert(reflected);
        let copied = reopened.paste(&reopened.copy_selected().unwrap()).unwrap();
        let copy = copied
            .into_iter()
            .find(|id| {
                transform_is(
                    &reopened.objects[id - 1].kind,
                    NativeTransformFlavor::Reflection,
                    &[ElementType::Segment, ElementType::Circle],
                )
            })
            .unwrap();
        if name == "segment" {
            let (a, b, _) = reopened.line_points(reflected).unwrap();
            let (copied_a, copied_b, _) = reopened.line_points(copy).unwrap();
            close(
                copied_a,
                Point {
                    x: a.x + 20.,
                    y: a.y + 20.,
                },
            );
            close(
                copied_b,
                Point {
                    x: b.x + 20.,
                    y: b.y + 20.,
                },
            );
        } else {
            let (center, radius) = reopened.circle_points(reflected).unwrap();
            let (copied_center, copied_radius) = reopened.circle_points(copy).unwrap();
            close(
                copied_center,
                Point {
                    x: center.x + 20.,
                    y: center.y + 20.,
                },
            );
            assert_eq!(copied_radius, radius);
        }
        reopened.selected.clear();
        reopened.selected.insert(1);
        reopened.clear_selected().unwrap();
        assert!(reopened.objects[reflected - 1].deleted);
        assert!(!reopened.objects[copy - 1].deleted);
        reopened.verify_roundtrip().unwrap();
    }
}

#[test]
fn fixed_segment_transform_chains_keep_ray_and_line_sources_unsupported() {
    for ray in [false, true] {
        for transform in [27, 30, 34] {
            let mut doc = Document::new();
            let a = doc.add(Kind::Free(Point { x: 100., y: 100. }));
            let b = doc.add(Kind::Free(Point { x: 150., y: 120. }));
            let source = doc.add(if ray {
                Kind::Ray(a, b)
            } else {
                Kind::Line(a, b)
            });
            let mirror = doc.add(Kind::Segment(a, b));
            doc.add(match transform {
                27 => Kind::RotatedSegment(source, a, 45.),
                30 => Kind::ScaledSegment(source, a, 2.),
                _ => Kind::ReflectedSegment(source, mirror),
            });
            assert!(doc.encode().is_err());
        }
    }
    let mut bytes =
        include_bytes!("../../../../../tests/fixtures/fixed_transform_chain_segment_native_verified.gsp")
            .to_vec();
    let header = records(&bytes)
        .unwrap()
        .into_iter()
        .filter(|r| r.kind == 0x07d0)
        .nth(6)
        .unwrap();
    bytes[header.data.start + 6..header.data.start + 8].copy_from_slice(&3u16.to_le_bytes());
    let doc = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(doc.objects[6].kind, Kind::Unsupported(27));
    assert_eq!(doc.encode().unwrap(), bytes);
}

#[test]
fn fixed_rotated_ray_and_line_preserve_native_modes_edits_and_dependencies() {
    let close = |a: Point, b: Point| {
        assert!(
            (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9,
            "{a:?} != {b:?}"
        )
    };
    for (name, bytes, edited) in [
        (
            "ray",
            include_bytes!("../../../../../tests/fixtures/fixed_rotation_ray_original.gsp").as_slice(),
            include_bytes!("../../../../../tests/fixtures/fixed_rotation_ray_edited_native_verified.gsp")
                .as_slice(),
        ),
        (
            "line",
            include_bytes!("../../../../../tests/fixtures/fixed_rotation_line_original.gsp").as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/fixed_rotation_line_edited_native_verified.gsp"
            )
            .as_slice(),
        ),
    ] {
        let mode = if name == "ray" {
            LineMode::Ray
        } else {
            LineMode::Line
        };
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(
            doc.objects[4].kind,
            if name == "ray" {
                Kind::RotatedRay(4, 1, 90.)
            } else {
                Kind::RotatedLine(4, 1, 90.)
            }
        );
        assert!(!doc.objects[4].kind.is_segment());
        assert_eq!(
            super::super::kind::straight_kind_mode(&doc.objects[4].kind),
            Some(mode)
        );
        assert_eq!(doc.objects[4].kind.is_line(), name == "line");
        assert_eq!(doc.encode().unwrap(), bytes);
        let (a, b, actual_mode) = doc.line_points(5).unwrap();
        close(a, Point { x: 170., y: 110. });
        close(b, Point { x: 170., y: 70. });
        assert_eq!(actual_mode, mode);
        for (id, point) in [
            (1, Point { x: 150., y: 170. }),
            (2, Point { x: 180., y: 220. }),
            (3, Point { x: 260., y: 200. }),
        ] {
            assert!(doc.move_point(id, point));
        }
        let (a, b, actual_mode) = doc.line_points(5).unwrap();
        close(a, Point { x: 200., y: 140. });
        close(b, Point { x: 180., y: 60. });
        assert_eq!(actual_mode, mode);
        assert_eq!(doc.encode().unwrap(), edited);
        doc.verify_roundtrip().unwrap();
        let mut reopened = Document::from_bytes(edited, None).unwrap();
        reopened.selected.insert(5);
        let pasted = reopened.paste(&reopened.copy_selected().unwrap()).unwrap();
        let copy = pasted
            .into_iter()
            .find(|id| {
                transform_is(
                    &reopened.objects[id - 1].kind,
                    NativeTransformFlavor::FixedRotation,
                    &[ElementType::Ray, ElementType::Line],
                )
            })
            .unwrap();
        let (ca, cb, cmode) = reopened.line_points(copy).unwrap();
        close(
            ca,
            Point {
                x: a.x + 20.,
                y: a.y + 20.,
            },
        );
        close(
            cb,
            Point {
                x: b.x + 20.,
                y: b.y + 20.,
            },
        );
        assert_eq!(cmode, mode);
        reopened.selected.clear();
        reopened.selected.insert(1);
        reopened.clear_selected().unwrap();
        assert!(reopened.objects[4].deleted);
        assert!(!reopened.objects[copy - 1].deleted);
        reopened.verify_roundtrip().unwrap();
        let mut fresh = Document::new();
        let center = fresh.add(Kind::Free(Point { x: 130., y: 150. }));
        let a = fresh.add(Kind::Free(Point { x: 170., y: 190. }));
        let b = fresh.add(Kind::Free(Point { x: 210., y: 190. }));
        let source = fresh.add(if name == "ray" {
            Kind::Ray(a, b)
        } else {
            Kind::Line(a, b)
        });
        let image = fresh.add(if name == "ray" {
            Kind::RotatedRay(source, center, 45.)
        } else {
            Kind::RotatedLine(source, center, 45.)
        });
        assert_eq!(fresh.line_points(image).unwrap().2, mode);
        fresh.verify_roundtrip().unwrap();
        let saved = fresh.encode().unwrap();
        assert_eq!(
            saved,
            if name == "ray" {
                include_bytes!(
                    "../../../../../tests/fixtures/fixed_rotation_ray_generated_native_verified.gsp"
                )
                .as_slice()
            } else {
                include_bytes!(
                    "../../../../../tests/fixtures/fixed_rotation_line_generated_native_verified.gsp"
                )
                .as_slice()
            }
        );
        if std::env::var_os("SKETCHPAD_WRITE_ROTATED_STRAIGHT_FIXTURE").is_some() {
            std::fs::write(
                format!("/tmp/fixed_rotation_{name}_generated_port.gsp"),
                &saved,
            )
            .unwrap();
        }
        let saved_header = records(&saved)
            .unwrap()
            .into_iter()
            .filter(|r| r.kind == 0x7d0)
            .nth(4)
            .unwrap();
        assert_eq!(
            &saved[saved_header.data.start + 6..saved_header.data.start + 10],
            if name == "ray" {
                &[2, 0, 13, 0]
            } else {
                &[2, 0, 12, 0]
            }
        );
        let mut malformed = bytes.to_vec();
        let h = records(bytes)
            .unwrap()
            .into_iter()
            .filter(|r| r.kind == 0x7d0)
            .nth(4)
            .unwrap();
        malformed[h.data.start + 8] = if name == "ray" { 12 } else { 13 };
        let invalid = Document::from_bytes(&malformed, None).unwrap();
        assert_eq!(invalid.objects[4].kind, Kind::Unsupported(27));
        assert_eq!(invalid.encode().unwrap(), malformed);
        let mut wrong_mode = Document::new();
        let a = wrong_mode.add(Kind::Free(Point { x: 100., y: 100. }));
        let b = wrong_mode.add(Kind::Free(Point { x: 150., y: 120. }));
        let source = wrong_mode.add(if name == "ray" {
            Kind::Line(a, b)
        } else {
            Kind::Ray(a, b)
        });
        wrong_mode.add(if name == "ray" {
            Kind::RotatedRay(source, a, 45.)
        } else {
            Kind::RotatedLine(source, a, 45.)
        });
        assert!(wrong_mode.encode().is_err());
        let mut chained = fresh.clone();
        chained.add(if name == "ray" {
            Kind::RotatedRay(image, center, 45.)
        } else {
            Kind::RotatedLine(image, center, 45.)
        });
        chained.verify_roundtrip().unwrap();
        let mut mirror = fresh.clone();
        let reflected = mirror.add(Kind::ReflectedPoint(1, image));
        close(
            mirror.position(reflected).unwrap(),
            Point {
                x: 186.5685424949238,
                y: 206.5685424949238,
            },
        );
        mirror.verify_roundtrip().unwrap();
        assert_eq!(
            mirror.encode().unwrap(),
            if name == "ray" {
                include_bytes!(
                    "../../../../../tests/fixtures/reflected_point_rotated_ray_mirror_native_verified.gsp"
                )
                .as_slice()
            } else {
                include_bytes!("../../../../../tests/fixtures/reflected_point_rotated_line_mirror_native_verified.gsp").as_slice()
            }
        );
    }
}

#[test]
fn fixed_scaled_ray_and_line_native_modes_zero_dependents_and_edits() {
    for (ray, bytes, zero) in [
        (
            true,
            include_bytes!("../../../../../tests/fixtures/fixed_dilation_ray_original.gsp").as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/fixed_dilation_ray_zero_point_native_verified.gsp"
            )
            .as_slice(),
        ),
        (
            false,
            include_bytes!("../../../../../tests/fixtures/fixed_dilation_line_original.gsp").as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/fixed_dilation_line_zero_point_native_verified.gsp"
            )
            .as_slice(),
        ),
    ] {
        let mode = if ray { LineMode::Ray } else { LineMode::Line };
        let kind = |factor| {
            if ray {
                Kind::ScaledRay(4, 1, factor)
            } else {
                Kind::ScaledLine(4, 1, factor)
            }
        };
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects[4].kind, kind(2.));
        assert!(!doc.objects[4].kind.is_segment());
        assert_eq!(doc.encode().unwrap(), bytes);
        let mut malformed = bytes.to_vec();
        let header = records(bytes)
            .unwrap()
            .into_iter()
            .filter(|r| r.kind == 0x7d0)
            .nth(4)
            .unwrap();
        malformed[header.data.start + 8] = if ray { 12 } else { 13 };
        let rejected = Document::from_bytes(&malformed, None).unwrap();
        assert_eq!(rejected.objects[4].kind, Kind::Unsupported(30));
        assert_eq!(rejected.encode().unwrap(), malformed);
        let mut chained = doc.clone();
        chained.add(if ray {
            Kind::ScaledRay(5, 1, 2.)
        } else {
            Kind::ScaledLine(5, 1, 2.)
        });
        chained.verify_roundtrip().unwrap();

        assert_eq!(
            doc.line_points(5),
            Some((Point { x: 210., y: 230. }, Point { x: 290., y: 230. }, mode))
        );
        let positive_point = if ray {
            include_bytes!("../../../../../tests/fixtures/fixed_dilation_ray_point_native_verified.gsp")
                .as_slice()
        } else {
            include_bytes!("../../../../../tests/fixtures/fixed_dilation_line_point_native_verified.gsp")
                .as_slice()
        };
        let positive_point = Document::from_bytes(positive_point, None).unwrap();
        assert!(!positive_point.partial_view);
        assert_eq!(positive_point.position(6), Some(Point { x: 250., y: 230. }));
        positive_point.verify_roundtrip().unwrap();
        let zero = Document::from_bytes(zero, None).unwrap();
        assert!(!zero.partial_view);
        assert_eq!(zero.objects[4].kind, kind(0.));
        assert!(zero.line_points(5).is_none());
        assert!(zero.position(6).is_none());
        zero.verify_roundtrip().unwrap();
        doc.move_point(1, Point { x: 150., y: 170. });
        doc.move_point(2, Point { x: 180., y: 220. });
        assert_eq!(
            doc.line_points(5),
            Some((Point { x: 210., y: 270. }, Point { x: 270., y: 210. }, mode))
        );
        doc.verify_roundtrip().unwrap();
        let mut edited = doc.clone();
        edited.move_point(3, Point { x: 260., y: 200. });
        assert_eq!(
            edited.line_points(5),
            Some((Point { x: 210., y: 270. }, Point { x: 370., y: 230. }, mode))
        );
        assert_eq!(
            edited.encode().unwrap(),
            if ray {
                include_bytes!(
                    "../../../../../tests/fixtures/fixed_dilation_ray_edited_native_verified.gsp"
                )
                .as_slice()
            } else {
                include_bytes!(
                    "../../../../../tests/fixtures/fixed_dilation_line_edited_native_verified.gsp"
                )
                .as_slice()
            }
        );
        let image = doc.add(kind(-2.));
        assert_eq!(
            doc.line_points(image),
            Some((Point { x: 90., y: 70. }, Point { x: 30., y: 130. }, mode))
        );
        let point = doc.add(if ray {
            Kind::PointOnRay(image, 0.5)
        } else {
            Kind::PointOnLine(image, 0.5)
        });
        assert_eq!(doc.position(point), Some(Point { x: 60., y: 100. }));
        doc.verify_roundtrip().unwrap();
        doc.selected.insert(5);
        let copied = doc.paste(&doc.copy_selected().unwrap()).unwrap();
        let copy = copied
            .into_iter()
            .find(|&id| {
                transform_is(
                    &doc.objects[id - 1].kind,
                    NativeTransformFlavor::FixedDilation,
                    &[ElementType::Ray, ElementType::Line],
                )
            })
            .unwrap();
        assert_eq!(
            doc.line_points(copy),
            Some((Point { x: 230., y: 290. }, Point { x: 290., y: 230. }, mode))
        );
        doc.verify_roundtrip().unwrap();
        doc.selected.clear();
        doc.selected.insert(4);
        doc.clear_selected().unwrap();
        assert!(doc.objects[4].deleted);
        assert!(doc.objects[image - 1].deleted);
        assert!(doc.objects[point - 1].deleted);
        assert!(!doc.objects[copy - 1].deleted);
        doc.verify_roundtrip().unwrap();
        for factor in [2., -2.] {
            let mut fresh = Document::new();
            let center = fresh.add(Kind::Free(Point { x: 130., y: 150. }));
            let a = fresh.add(Kind::Free(Point { x: 170., y: 190. }));
            let b = fresh.add(Kind::Free(Point { x: 210., y: 190. }));
            let source = fresh.add(if ray {
                Kind::Ray(a, b)
            } else {
                Kind::Line(a, b)
            });
            let image = fresh.add(if ray {
                Kind::ScaledRay(source, center, factor)
            } else {
                Kind::ScaledLine(source, center, factor)
            });
            fresh.verify_roundtrip().unwrap();
            let golden = match (ray, factor > 0.) {
                (true, true) => include_bytes!("../../../../../tests/fixtures/fixed_dilation_ray_positive_generated_native_verified.gsp").as_slice(),
                (true, false) => include_bytes!("../../../../../tests/fixtures/fixed_dilation_ray_negative_generated_native_verified.gsp").as_slice(),
                (false, true) => include_bytes!("../../../../../tests/fixtures/fixed_dilation_line_positive_generated_native_verified.gsp").as_slice(),
                (false, false) => include_bytes!("../../../../../tests/fixtures/fixed_dilation_line_negative_generated_native_verified.gsp").as_slice(),
            };
            assert_eq!(fresh.encode().unwrap(), golden);
            if factor > 0. {
                let reflected = fresh.add(Kind::ReflectedPoint(center, image));
                assert_eq!(fresh.position(reflected), Some(Point { x: 130., y: 310. }));
                fresh.verify_roundtrip().unwrap();
                assert_eq!(
                    fresh.encode().unwrap(),
                    if ray {
                        include_bytes!("../../../../../tests/fixtures/reflected_point_scaled_ray_mirror_native_verified.gsp").as_slice()
                    } else {
                        include_bytes!("../../../../../tests/fixtures/reflected_point_scaled_line_mirror_native_verified.gsp").as_slice()
                    }
                );
            }
        }
        let mut invalid = Document::new();
        let center = invalid.add(Kind::Free(Point { x: 100., y: 100. }));
        let endpoint = invalid.add(Kind::Free(Point { x: 150., y: 120. }));
        let source = invalid.add(if ray {
            Kind::Line(center, endpoint)
        } else {
            Kind::Ray(center, endpoint)
        });
        invalid.add(if ray {
            Kind::ScaledRay(source, center, 2.)
        } else {
            Kind::ScaledLine(source, center, 2.)
        });
        assert!(invalid.encode().is_err());
    }
    for (mode, bytes) in [
        (
            LineMode::Ray,
            include_bytes!(
                "../../../../../tests/fixtures/fixed_dilation_ray_negative_native_verified.gsp"
            )
            .as_slice(),
        ),
        (
            LineMode::Line,
            include_bytes!(
                "../../../../../tests/fixtures/fixed_dilation_line_negative_native_verified.gsp"
            )
            .as_slice(),
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(
            doc.line_points(5),
            Some((Point { x: 50., y: 70. }, Point { x: -30., y: 70. }, mode))
        );
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn fixed_straight_transform_chains_preserve_native_modes_styles_and_zero_ancestors() {
    let style = Some(VisualStyle::Stroke {
        line: SegmentLineStyle {
            width: LineWidth::Thick,
            pattern: LinePattern::Dashed,
        },
        color: RgbColor {
            red: 255,
            green: 0,
            blue: 255,
        },
    });
    let rotate = |p: Point, c: Point, degrees: f64| {
        let (s, co) = degrees.to_radians().sin_cos();
        Point {
            x: c.x + co * (p.x - c.x) + s * (p.y - c.y),
            y: c.y + co * (p.y - c.y) - s * (p.x - c.x),
        }
    };
    let scale = |p: Point, c: Point, factor: f64| Point {
        x: c.x + factor * (p.x - c.x),
        y: c.y + factor * (p.y - c.y),
    };
    let close = |a: Point, b: Point| {
        assert!(
            (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9,
            "{a:?} != {b:?}"
        )
    };
    for (ray, rotation, bytes) in [
        (
            true,
            true,
            include_bytes!(
                "../../../../../tests/fixtures/fixed_straight_chain_rotation_ray_original.gsp"
            )
            .as_slice(),
        ),
        (
            false,
            true,
            include_bytes!(
                "../../../../../tests/fixtures/fixed_straight_chain_rotation_line_original.gsp"
            )
            .as_slice(),
        ),
        (
            true,
            false,
            include_bytes!(
                "../../../../../tests/fixtures/fixed_straight_chain_dilation_ray_original.gsp"
            )
            .as_slice(),
        ),
        (
            false,
            false,
            include_bytes!(
                "../../../../../tests/fixtures/fixed_straight_chain_dilation_line_original.gsp"
            )
            .as_slice(),
        ),
    ] {
        let mode = if ray { LineMode::Ray } else { LineMode::Line };
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        assert_eq!(
            doc.objects[5].kind,
            if ray {
                Kind::RotatedRay(5, 1, 90.)
            } else {
                Kind::RotatedLine(5, 1, 90.)
            }
        );
        assert_eq!(
            doc.objects[6].kind,
            if ray {
                Kind::ScaledRay(5, 1, -2.)
            } else {
                Kind::ScaledLine(5, 1, -2.)
            }
        );
        for object in &doc.objects[4..7] {
            assert_eq!(object.visual_style, style);
            assert!(!object.kind.is_segment());
        }
        for moved in [false, true] {
            if moved {
                doc.move_point(1, Point { x: 150., y: 170. });
                doc.move_point(2, Point { x: 180., y: 220. });
                doc.move_point(3, Point { x: 260., y: 200. });
            }
            let c = doc.position(1).unwrap();
            let host = |p| {
                if rotation {
                    rotate(p, c, 45.)
                } else {
                    scale(p, c, 2.)
                }
            };
            let (a, b) = (
                host(doc.position(2).unwrap()),
                host(doc.position(3).unwrap()),
            );
            for (id, ea, eb) in [
                (5, a, b),
                (6, rotate(a, c, 90.), rotate(b, c, 90.)),
                (7, scale(a, c, -2.), scale(b, c, -2.)),
            ] {
                let (actual_a, actual_b, actual_mode) = doc.line_points(id).unwrap();
                close(actual_a, ea);
                close(actual_b, eb);
                assert_eq!(actual_mode, mode);
            }
            doc.verify_roundtrip().unwrap();
            if moved {
                let golden = match (rotation, ray) {
                    (true, true) => include_bytes!("../../../../../tests/fixtures/fixed_straight_chain_rotation_ray_edited_native_verified.gsp").as_slice(),
                    (true, false) => include_bytes!("../../../../../tests/fixtures/fixed_straight_chain_rotation_line_edited_native_verified.gsp").as_slice(),
                    (false, true) => include_bytes!("../../../../../tests/fixtures/fixed_straight_chain_dilation_ray_edited_native_verified.gsp").as_slice(),
                    (false, false) => include_bytes!("../../../../../tests/fixtures/fixed_straight_chain_dilation_line_edited_native_verified.gsp").as_slice(),
                };
                assert_eq!(doc.encode().unwrap(), golden);
            }
        }

        let mut fresh = Document::new();
        let c = fresh.add(Kind::Free(Point { x: 130., y: 150. }));
        let a = fresh.add(Kind::Free(Point { x: 170., y: 190. }));
        let b = fresh.add(Kind::Free(Point { x: 210., y: 190. }));
        let source = fresh.add(if ray {
            Kind::Ray(a, b)
        } else {
            Kind::Line(a, b)
        });
        let image = fresh.add(match (rotation, ray) {
            (true, true) => Kind::RotatedRay(source, c, 45.),
            (true, false) => Kind::RotatedLine(source, c, 45.),
            (false, true) => Kind::ScaledRay(source, c, 2.),
            (false, false) => Kind::ScaledLine(source, c, 2.),
        });
        fresh.selected.insert(image);
        fresh
            .set_selected_color(RgbColor {
                red: 255,
                green: 0,
                blue: 255,
            })
            .unwrap();
        fresh.set_selected_line_width(LineWidth::Thick).unwrap();
        fresh
            .set_selected_line_pattern(LinePattern::Dashed)
            .unwrap();
        fresh.add(if ray {
            Kind::RotatedRay(image, c, 90.)
        } else {
            Kind::RotatedLine(image, c, 90.)
        });
        fresh.add(if ray {
            Kind::ScaledRay(image, c, -2.)
        } else {
            Kind::ScaledLine(image, c, -2.)
        });
        fresh.verify_roundtrip().unwrap();
        let golden = match (rotation, ray) {
            (true, true) => include_bytes!("../../../../../tests/fixtures/fixed_straight_chain_rotation_ray_generated_native_verified.gsp").as_slice(),
            (true, false) => include_bytes!("../../../../../tests/fixtures/fixed_straight_chain_rotation_line_generated_native_verified.gsp").as_slice(),
            (false, true) => include_bytes!("../../../../../tests/fixtures/fixed_straight_chain_dilation_ray_generated_native_verified.gsp").as_slice(),
            (false, false) => include_bytes!("../../../../../tests/fixtures/fixed_straight_chain_dilation_line_generated_native_verified.gsp").as_slice(),
        };
        assert_eq!(fresh.encode().unwrap(), golden);
        doc.selected.insert(6);
        doc.selected.insert(7);
        let copies = doc.paste(&doc.copy_selected().unwrap()).unwrap();
        for original in [6, 7] {
            let copy = *copies
                .iter()
                .find(|&&id| {
                    if original == 6 {
                        transform_is(
                            &doc.objects[id - 1].kind,
                            NativeTransformFlavor::FixedRotation,
                            &[ElementType::Ray, ElementType::Line],
                        ) && transform_literal_is(
                            &doc.objects[id - 1].kind,
                            NativeLiteralRole::AngleDegrees,
                            90.,
                        )
                    } else {
                        transform_is(
                            &doc.objects[id - 1].kind,
                            NativeTransformFlavor::FixedDilation,
                            &[ElementType::Ray, ElementType::Line],
                        ) && transform_literal_is(
                            &doc.objects[id - 1].kind,
                            NativeLiteralRole::ScaleFactor,
                            -2.,
                        )
                    }
                })
                .unwrap();
            let (a, b, _) = doc.line_points(original).unwrap();
            let (ca, cb, cm) = doc.line_points(copy).unwrap();
            close(
                ca,
                Point {
                    x: a.x + 20.,
                    y: a.y + 20.,
                },
            );
            close(
                cb,
                Point {
                    x: b.x + 20.,
                    y: b.y + 20.,
                },
            );
            assert_eq!(cm, mode);
            assert_eq!(doc.objects[copy - 1].visual_style, style);
        }
        doc.verify_roundtrip().unwrap();
        doc.selected.clear();
        doc.selected.insert(4);
        doc.clear_selected().unwrap();
        assert!(doc.objects[4..7].iter().all(|object| object.deleted));
        assert!(copies.iter().all(|id| !doc.objects[id - 1].deleted));
        doc.verify_roundtrip().unwrap();
    }
    for (ray, negative, zero) in [
        (true, include_bytes!("../../../../../tests/fixtures/fixed_straight_chain_dilation_ray_negative_control_native_verified.gsp").as_slice(), include_bytes!("../../../../../tests/fixtures/fixed_straight_chain_dilation_ray_zero_control_native_verified.gsp").as_slice()),
        (false, include_bytes!("../../../../../tests/fixtures/fixed_straight_chain_dilation_line_negative_control_native_verified.gsp").as_slice(), include_bytes!("../../../../../tests/fixtures/fixed_straight_chain_dilation_line_zero_control_native_verified.gsp").as_slice()),
    ] {
        let mode = if ray { LineMode::Ray } else { LineMode::Line };
        let negative = Document::from_bytes(negative, None).unwrap();
        assert!(!negative.partial_view);
        assert_eq!(negative.line_points(6), Some((Point { x: 50., y: 230. }, Point { x: 50., y: 310. }, mode)));
        assert_eq!(negative.line_points(7), Some((Point { x: 290., y: 310. }, Point { x: 450., y: 310. }, mode)));
        negative.verify_roundtrip().unwrap();
        let mut doc = Document::from_bytes(zero, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), zero);
        for id in [5, 6, 7] { assert!(doc.line_points(id).is_none()); }
        doc.selected.insert(6); doc.selected.insert(7);
        let copies = doc.paste(&doc.copy_selected().unwrap()).unwrap();
        for id in copies { if doc.objects[id - 1].kind.fixed_transform_straight_mode().is_some() && !matches!(doc.objects[id - 1].kind, Kind::Ray(..) | Kind::Line(..)) { assert!(doc.line_points(id).is_none()); } }
        doc.verify_roundtrip().unwrap();
    }
    for bytes in [
        include_bytes!(
            "../../../../../tests/fixtures/fixed_straight_chain_primitive_ray_style_native_verified.gsp"
        )
        .as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/fixed_straight_chain_primitive_line_style_native_verified.gsp"
        )
        .as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        for id in [4, 6, 7] {
            assert_eq!(doc.objects[id - 1].visual_style, style);
        }
        doc.verify_roundtrip().unwrap();
    }
    let mut unsupported = Document::new();
    let a = unsupported.add(Kind::Free(Point { x: 100., y: 100. }));
    let b = unsupported.add(Kind::Free(Point { x: 150., y: 120. }));
    let c = unsupported.add(Kind::Free(Point { x: 120., y: 150. }));
    let bisector = unsupported.add(Kind::Bisector(a, b, c));
    let image = unsupported.add(Kind::RotatedRay(bisector, a, 45.));
    assert_eq!(unsupported.line_points(image).unwrap().2, LineMode::Ray);
    unsupported.verify_roundtrip().unwrap();
}

#[test]
fn calculated_angle_ray_and_line_require_numeric_values_and_preserve_live_modes() {
    for (ray, bytes) in [
        (
            true,
            include_bytes!("../../../../../tests/fixtures/numeric_angle_ray_original.gsp").as_slice(),
        ),
        (
            false,
            include_bytes!("../../../../../tests/fixtures/numeric_angle_line_original.gsp").as_slice(),
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(
            doc.objects
                .iter()
                .all(|object| !matches!(object.kind, Kind::Unsupported(..)))
        );
        assert_eq!(
            doc.objects[7].kind,
            if ray {
                Kind::CalculatedAngleRotatedRay(4, 1, 6)
            } else {
                Kind::CalculatedAngleRotatedLine(4, 1, 6)
            }
        );
        assert_eq!(doc.objects[7].visual_style, doc.objects[3].visual_style);
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.verify_roundtrip().unwrap();
        let mut malformed = bytes.to_vec();
        let header = records(bytes)
            .unwrap()
            .into_iter()
            .filter(|record| record.kind == 0x7d0)
            .nth(7)
            .unwrap();
        malformed[header.data.start + 8] = if ray { 12 } else { 13 };
        let unsupported = Document::from_bytes(&malformed, None).unwrap();
        assert_eq!(unsupported.objects[7].kind, Kind::Unsupported(29));
        assert_eq!(unsupported.encode().unwrap(), malformed);
    }
    for ray in [true, false] {
        let mut doc = Document::new();
        let center = doc.add(Kind::Free(Point { x: 130., y: 150. }));
        let a = doc.add(Kind::Free(Point { x: 170., y: 190. }));
        let b = doc.add(Kind::Free(Point { x: 210., y: 190. }));
        let source = doc.add(if ray {
            Kind::Ray(a, b)
        } else {
            Kind::Line(a, b)
        });
        let n = doc.add_named_parameter("n", 4., (50, 30)).unwrap();
        let angle = doc
            .add_parameter_calculation(n, "360°/n", (50, 60))
            .unwrap();
        doc.add_parameter_calculation(n, "n/2", (50, 90)).unwrap();
        doc.selected.insert(source);
        doc.set_selected_color(RgbColor {
            red: 255,
            green: 0,
            blue: 255,
        })
        .unwrap();
        doc.set_selected_line_width(LineWidth::Thick).unwrap();
        doc.set_selected_line_pattern(LinePattern::Dashed).unwrap();
        let before = doc.encode().unwrap();
        assert!(
            doc.add_calculated_angle_rotation(source, center, a)
                .is_err()
        );
        assert_eq!(doc.encode().unwrap(), before);
        let image = doc
            .add_calculated_angle_rotation(source, center, angle)
            .unwrap();
        assert_eq!(
            doc.objects[image - 1].kind,
            if ray {
                Kind::CalculatedAngleRotatedRay(source, center, angle)
            } else {
                Kind::CalculatedAngleRotatedLine(source, center, angle)
            }
        );
        assert!(!doc.objects[image - 1].kind.is_segment());
        assert_eq!(
            doc.objects[image - 1].visual_style,
            doc.objects[source - 1].visual_style
        );
        let mode = if ray { LineMode::Ray } else { LineMode::Line };
        let close = |a: Point, b: Point| {
            assert!(
                (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9,
                "{a:?} != {b:?}"
            )
        };
        let (ra, rb, actual_mode) = doc.line_points(image).unwrap();
        close(ra, Point { x: 170., y: 110. });
        close(rb, Point { x: 170., y: 70. });
        assert_eq!(actual_mode, mode);
        assert_eq!(
            doc.encode().unwrap(),
            if ray {
                include_bytes!(
                    "../../../../../tests/fixtures/numeric_angle_ray_generated_native_verified.gsp"
                )
                .as_slice()
            } else {
                include_bytes!(
                    "../../../../../tests/fixtures/numeric_angle_line_generated_native_verified.gsp"
                )
                .as_slice()
            }
        );
        doc.verify_roundtrip().unwrap();
        let mut fresh_undefined = doc.clone();
        fresh_undefined.set_parameter_value(n, 0.).unwrap();
        assert!(fresh_undefined.line_points(image).is_none());
        fresh_undefined.verify_roundtrip().unwrap();
        fresh_undefined.set_parameter_value(n, 4.).unwrap();
        assert_eq!(fresh_undefined.line_points(image), doc.line_points(image));
        fresh_undefined.verify_roundtrip().unwrap();
        let mut doc = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        doc.set_parameter_value(n, 8.).unwrap();
        let (ra, rb, _) = doc.line_points(image).unwrap();
        close(
            ra,
            Point {
                x: 186.5685424949238,
                y: 150.,
            },
        );
        close(
            rb,
            Point {
                x: 214.8528137423857,
                y: 121.7157287525381,
            },
        );
        let mut moved = doc.clone();
        assert!(moved.move_point(center, Point { x: 140., y: 160. }));
        assert!(moved.move_point(a, Point { x: 180., y: 210. }));
        assert!(moved.move_point(b, Point { x: 240., y: 220. }));
        assert_eq!(
            moved.encode().unwrap(),
            if ray {
                include_bytes!(
                    "../../../../../tests/fixtures/numeric_angle_ray_edited_native_verified.gsp"
                )
                .as_slice()
            } else {
                include_bytes!(
                    "../../../../../tests/fixtures/numeric_angle_line_edited_native_verified.gsp"
                )
                .as_slice()
            }
        );
        doc.set_parameter_value(n, -4.).unwrap();
        assert_eq!(
            doc.encode().unwrap(),
            if ray {
                include_bytes!(
                    "../../../../../tests/fixtures/numeric_angle_ray_negative_native_verified.gsp"
                )
                .as_slice()
            } else {
                include_bytes!(
                    "../../../../../tests/fixtures/numeric_angle_line_negative_native_verified.gsp"
                )
                .as_slice()
            }
        );
        let (ra, rb, _) = doc.line_points(image).unwrap();
        close(ra, Point { x: 90., y: 190. });
        close(rb, Point { x: 90., y: 230. });
        doc.set_parameter_value(n, 0.).unwrap();
        assert!(doc.numeric_value(angle).is_none());
        assert!(doc.line_points(image).is_none());
        assert_eq!(
            doc.encode().unwrap(),
            if ray {
                include_bytes!(
                    "../../../../../tests/fixtures/numeric_angle_ray_undefined_native_verified.gsp"
                )
                .as_slice()
            } else {
                include_bytes!(
                    "../../../../../tests/fixtures/numeric_angle_line_undefined_native_verified.gsp"
                )
                .as_slice()
            }
        );
        doc.verify_roundtrip().unwrap();
        doc.set_parameter_value(n, 4.).unwrap();
        doc.verify_roundtrip().unwrap();
        doc.selected.clear();
        doc.selected.insert(image);
        let copied = doc.paste(&doc.copy_selected().unwrap()).unwrap();
        let copy = *copied
            .iter()
            .find(|&&id| {
                transform_is(
                    &doc.objects[id - 1].kind,
                    NativeTransformFlavor::NumericRotation,
                    &[ElementType::Ray, ElementType::Line],
                )
            })
            .unwrap();
        let (ra, rb, copied_mode) = doc.line_points(copy).unwrap();
        close(ra, Point { x: 190., y: 130. });
        close(rb, Point { x: 190., y: 90. });
        assert_eq!(copied_mode, mode);
        doc.verify_roundtrip().unwrap();
        doc.selected.clear();
        doc.selected.insert(angle);
        doc.clear_selected().unwrap();
        assert!(doc.objects[image - 1].deleted);
        assert!(!doc.objects[copy - 1].deleted);
        doc.verify_roundtrip().unwrap();
        let mut invalid = Document::new();
        let a = invalid.add(Kind::Free(Point { x: 100., y: 100. }));
        let b = invalid.add(Kind::Free(Point { x: 150., y: 120. }));
        let opposite = invalid.add(if ray {
            Kind::Line(a, b)
        } else {
            Kind::Ray(a, b)
        });
        let n = invalid.add_named_parameter("n", 4., (50, 30)).unwrap();
        let angle = invalid
            .add_parameter_calculation(n, "360°/n", (50, 60))
            .unwrap();
        invalid.add(if ray {
            Kind::CalculatedAngleRotatedRay(opposite, a, angle)
        } else {
            Kind::CalculatedAngleRotatedLine(opposite, a, angle)
        });
        assert!(invalid.encode().is_err());
    }
}

fn numeric_ratio_straight_document(ray: bool, reciprocal: bool) -> Document {
    let mut doc = Document::new();
    doc.add(Kind::Free(Point { x: 130., y: 150. }));
    let a = doc.add(Kind::Free(Point { x: 170., y: 190. }));
    let b = doc.add(Kind::Free(Point { x: 210., y: 190. }));
    let source = doc.add(if ray {
        Kind::Ray(a, b)
    } else {
        Kind::Line(a, b)
    });
    let n = doc.add_named_parameter("n", 4., (50, 30)).unwrap();
    doc.add_parameter_calculation(n, "360°/n", (50, 60))
        .unwrap();
    doc.add_parameter_calculation(n, if reciprocal { "2/n" } else { "n/2" }, (50, 90))
        .unwrap();
    doc.selected.insert(source);
    doc.set_selected_color(RgbColor {
        red: 255,
        green: 0,
        blue: 255,
    })
    .unwrap();
    doc.set_selected_line_width(LineWidth::Thick).unwrap();
    doc.set_selected_line_pattern(LinePattern::Dashed).unwrap();
    doc
}

#[test]
fn calculated_ratio_ray_and_line_preserve_native_modes_and_live_domains() {
    for (ray, original, generated, edited, negative, zero, undefined) in [
        (
            true,
            include_bytes!("../../../../../tests/fixtures/numeric_ratio_ray_original.gsp").as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/numeric_ratio_ray_generated_native_verified.gsp"
            )
            .as_slice(),
            include_bytes!("../../../../../tests/fixtures/numeric_ratio_ray_edited_native_verified.gsp")
                .as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/numeric_ratio_ray_negative_native_verified.gsp"
            )
            .as_slice(),
            include_bytes!("../../../../../tests/fixtures/numeric_ratio_ray_zero_native_verified.gsp")
                .as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/numeric_ratio_ray_undefined_native_verified.gsp"
            )
            .as_slice(),
        ),
        (
            false,
            include_bytes!("../../../../../tests/fixtures/numeric_ratio_line_original.gsp").as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/numeric_ratio_line_generated_native_verified.gsp"
            )
            .as_slice(),
            include_bytes!("../../../../../tests/fixtures/numeric_ratio_line_edited_native_verified.gsp")
                .as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/numeric_ratio_line_negative_native_verified.gsp"
            )
            .as_slice(),
            include_bytes!("../../../../../tests/fixtures/numeric_ratio_line_zero_native_verified.gsp")
                .as_slice(),
            include_bytes!(
                "../../../../../tests/fixtures/numeric_ratio_line_undefined_native_verified.gsp"
            )
            .as_slice(),
        ),
    ] {
        let expected = if ray {
            Kind::CalculatedRatioScaledRay(4, 1, 7)
        } else {
            Kind::CalculatedRatioScaledLine(4, 1, 7)
        };
        let mode = if ray { LineMode::Ray } else { LineMode::Line };
        let doc = Document::from_bytes(original, None).unwrap();
        assert!(
            doc.objects
                .iter()
                .all(|object| !matches!(object.kind, Kind::Unsupported(..)))
        );
        assert_eq!(doc.objects[7].kind, expected);
        assert_eq!(doc.encode().unwrap(), original);
        assert_eq!(doc.objects[7].visual_style, doc.objects[3].visual_style);
        doc.verify_roundtrip().unwrap();
        let mut malformed = original.to_vec();
        let h = records(original)
            .unwrap()
            .into_iter()
            .filter(|r| r.kind == 0x7d0)
            .nth(7)
            .unwrap();
        malformed[h.data.start + 8] = if ray { 12 } else { 13 };
        let rejected = Document::from_bytes(&malformed, None).unwrap();
        assert_eq!(rejected.objects[7].kind, Kind::Unsupported(33));
        assert_eq!(rejected.encode().unwrap(), malformed);

        let mut doc = numeric_ratio_straight_document(ray, false);
        let before = doc.encode().unwrap();
        assert!(doc.add_calculated_ratio_dilation(4, 1, 2).is_err());
        assert_eq!(doc.encode().unwrap(), before);
        let image = doc.add_calculated_ratio_dilation(4, 1, 7).unwrap();
        assert_eq!(image, 8);
        assert_eq!(doc.objects[7].kind, expected);
        assert!(!doc.objects[7].kind.is_segment());
        assert_eq!(doc.objects[7].visual_style, doc.objects[3].visual_style);
        assert_eq!(doc.encode().unwrap(), generated);
        assert_eq!(
            doc.line_points(8),
            Some((Point { x: 210., y: 230. }, Point { x: 290., y: 230. }, mode))
        );
        doc.verify_roundtrip().unwrap();
        doc.set_parameter_value(5, 8.).unwrap();
        assert!(doc.move_point(1, Point { x: 140., y: 160. }));
        assert!(doc.move_point(2, Point { x: 180., y: 210. }));
        assert!(doc.move_point(3, Point { x: 240., y: 220. }));
        assert_eq!(
            doc.line_points(8),
            Some((Point { x: 300., y: 360. }, Point { x: 540., y: 400. }, mode))
        );
        assert_eq!(doc.encode().unwrap(), edited);
        doc.verify_roundtrip().unwrap();
        let mut doc = Document::from_bytes(generated, None).unwrap();
        doc.set_parameter_value(5, -4.).unwrap();
        assert_eq!(
            doc.line_points(8),
            Some((Point { x: 50., y: 70. }, Point { x: -30., y: 70. }, mode))
        );
        assert_eq!(doc.encode().unwrap(), negative);
        doc.set_parameter_value(5, 0.).unwrap();
        assert_eq!(doc.unitless_numeric_value(7), Some(0.));
        assert!(doc.line_points(8).is_none());
        assert_eq!(doc.encode().unwrap(), zero);
        doc.verify_roundtrip().unwrap();
        doc.set_parameter_value(5, 4.).unwrap();
        assert_eq!(doc.line_points(8).unwrap().2, mode);
        doc.selected.clear();
        doc.selected.insert(8);
        let copied = doc.paste(&doc.copy_selected().unwrap()).unwrap();
        let copy = *copied
            .iter()
            .find(|&&id| {
                transform_is(
                    &doc.objects[id - 1].kind,
                    NativeTransformFlavor::NumericDilation,
                    &[ElementType::Ray, ElementType::Line],
                )
            })
            .unwrap();
        assert_eq!(
            doc.line_points(copy),
            Some((Point { x: 230., y: 250. }, Point { x: 310., y: 250. }, mode))
        );
        doc.verify_roundtrip().unwrap();
        doc.selected.clear();
        doc.selected.insert(7);
        doc.clear_selected().unwrap();
        assert!(doc.objects[7].deleted);
        assert!(!doc.objects[copy - 1].deleted);
        doc.verify_roundtrip().unwrap();

        let mut fresh_zero = numeric_ratio_straight_document(ray, false);
        fresh_zero.set_parameter_value(5, 0.).unwrap();
        assert_eq!(
            fresh_zero.add_calculated_ratio_dilation(4, 1, 7).unwrap(),
            8
        );
        assert!(fresh_zero.line_points(8).is_none());
        assert_eq!(fresh_zero.encode().unwrap(), zero);
        fresh_zero.verify_roundtrip().unwrap();
        fresh_zero.set_parameter_value(5, 4.).unwrap();
        assert!(fresh_zero.line_points(8).is_some());
        fresh_zero.verify_roundtrip().unwrap();
        let mut reciprocal = numeric_ratio_straight_document(ray, true);
        reciprocal.set_parameter_value(5, 0.).unwrap();
        let before = reciprocal.encode().unwrap();
        assert!(reciprocal.add_calculated_ratio_dilation(4, 1, 7).is_err());
        assert_eq!(reciprocal.encode().unwrap(), before);
        reciprocal.set_parameter_value(5, 4.).unwrap();
        reciprocal.add_calculated_ratio_dilation(4, 1, 7).unwrap();
        reciprocal.set_parameter_value(5, 0.).unwrap();
        assert!(reciprocal.unitless_numeric_value(7).is_none());
        assert!(reciprocal.line_points(8).is_none());
        assert_eq!(reciprocal.encode().unwrap(), undefined);
        reciprocal.verify_roundtrip().unwrap();
        let mut reciprocal = Document::from_bytes(undefined, None).unwrap();
        reciprocal.set_parameter_value(5, 4.).unwrap();
        assert_eq!(
            reciprocal.line_points(8),
            Some((Point { x: 150., y: 170. }, Point { x: 170., y: 170. }, mode))
        );
        reciprocal.verify_roundtrip().unwrap();
        let transformed = reciprocal.add(if ray {
            Kind::ScaledRay(4, 1, 2.)
        } else {
            Kind::ScaledLine(4, 1, 2.)
        });
        let chained = reciprocal
            .add_calculated_ratio_dilation(transformed, 1, 7)
            .unwrap();
        assert_eq!(reciprocal.line_points(chained).unwrap().2, mode);
        reciprocal.verify_roundtrip().unwrap();
    }
}
