use super::*;

#[test]
fn straight_intersection_handles_parallel_and_rays() {
    let a = Point { x: 0., y: 0. };
    let b = Point { x: 1., y: 0. };
    let c = Point { x: 2., y: -1. };
    let d = Point { x: 2., y: 1. };
    assert_eq!(
        intersect_straight((a, b, LineMode::Segment), (c, d, LineMode::Line)),
        None
    );
    assert_eq!(
        intersect_straight((a, b, LineMode::Ray), (c, d, LineMode::Line)),
        Some(Point { x: 2., y: 0. })
    );
    assert_eq!(
        intersect_straight(
            (a, b, LineMode::Line),
            (
                Point { x: 0., y: 1. },
                Point { x: 1., y: 1. },
                LineMode::Line
            )
        ),
        None
    );
}
#[test]
fn circle_intersections_update_and_reopen() {
    let original =
        Document::open(Path::new("tests/fixtures/circle_circle_intersection.gsp")).unwrap();
    assert_eq!(
        original.objects[6].kind,
        Kind::CircleCircleIntersection(6, 5, 0)
    );
    assert_eq!(
        original.objects[7].kind,
        Kind::CircleCircleIntersection(6, 5, 1)
    );
    assert!(original.position(7).is_some());
    assert!(original.position(8).is_some());
    let original_line_circle =
        Document::open(Path::new("tests/fixtures/line_circle_intersection.gsp")).unwrap();
    assert_eq!(
        original_line_circle.objects[6].kind,
        Kind::LineCircleIntersection(3, 6, 0)
    );
    assert!(original_line_circle.position(7).is_some());

    let mut document = Document::new();
    let left = document.add(Kind::Free(Point { x: -10., y: 0. }));
    let right = document.add(Kind::Free(Point { x: 10., y: 0. }));
    let origin = document.add(Kind::Free(Point { x: 0., y: 0. }));
    let radius = document.add(Kind::Free(Point { x: 5., y: 0. }));
    let other_center = document.add(Kind::Free(Point { x: 4., y: 0. }));
    let other_radius = document.add(Kind::Free(Point { x: 9., y: 0. }));
    let line = document.add(Kind::Segment(left, right));
    let first_circle = document.add(Kind::Circle(origin, radius));
    let second_circle = document.add(Kind::Circle(other_center, other_radius));
    let near = document.add(Kind::LineCircleIntersection(line, first_circle, 0));
    let far = document.add(Kind::LineCircleIntersection(line, first_circle, 1));
    let first_branch = document.add(Kind::CircleCircleIntersection(
        first_circle,
        second_circle,
        0,
    ));
    let second_branch = document.add(Kind::CircleCircleIntersection(
        first_circle,
        second_circle,
        1,
    ));
    assert_eq!(document.position(near), Some(Point { x: 5., y: 0. }));
    assert_eq!(document.position(far), Some(Point { x: -5., y: 0. }));
    assert!(document.position(first_branch).unwrap().y > 0.);
    assert!(document.position(second_branch).unwrap().y < 0.);
    let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    for id in [near, far, first_branch, second_branch] {
        assert_eq!(reopened.position(id), document.position(id));
    }
    document.move_point(other_center, Point { x: 30., y: 0. });
    assert!(document.position(first_branch).is_none());
    assert!(document.position(second_branch).is_none());
}
#[test]
fn line_circle_second_root_stays_undefined_outside_segment() {
    let mut document = Document::new();
    let center = document.add(Kind::Free(Point { x: 0., y: 0. }));
    let end = document.add(Kind::Free(Point { x: 10., y: 0. }));
    let radius = document.add(Kind::Free(Point { x: 0., y: 5. }));
    let segment = document.add(Kind::Segment(center, end));
    let circle = document.add(Kind::Circle(center, radius));
    let second = document.add(Kind::LineCircleIntersection(segment, circle, 1));
    assert!(document.position(second).is_none());
    let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.position(second), document.position(second));
}
#[test]
fn line_circle_record_requires_straight_object_first() {
    let mut document = Document::new();
    let center = document.add(Kind::Free(Point { x: 0., y: 0. }));
    let radius = document.add(Kind::Free(Point { x: 0., y: 5. }));
    let circle = document.add(Kind::Circle(center, radius));
    let end = document.add(Kind::Free(Point { x: 10., y: 0. }));
    let segment = document.add(Kind::Segment(center, end));
    let intersection = document.add(Kind::LineCircleIntersection(segment, circle, 0));
    assert_eq!(
        document.position(intersection),
        Some(Point { x: 5., y: 0. })
    );
    let encoded = document.encode().unwrap();
    assert_eq!(
        Document::from_bytes(&encoded, None)
            .unwrap()
            .position(intersection),
        document.position(intersection)
    );
    document.objects[intersection - 1].kind = Kind::LineCircleIntersection(circle, segment, 0);
    assert!(document.encode().is_err());
}
#[test]
fn angle_bisector_matches_original_record_and_updates() {
    let original = Document::open(Path::new("tests/fixtures/bisector.gsp")).unwrap();
    assert_eq!(original.objects[3].kind, Kind::Bisector(1, 2, 3));
    assert!(!original.partial_view);
    assert_eq!(original.line_points(4).unwrap().2, LineMode::Ray);
    let bytes = fs::read("tests/fixtures/bisector.gsp").unwrap();
    let object_headers: Vec<_> = records(&bytes)
        .unwrap()
        .into_iter()
        .filter(|record| record.kind == 0x7d0)
        .collect();
    assert_eq!(&bytes[object_headers[3].data.clone()], &header(7));

    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: -1., y: 1. }));
    let vertex = document.add(Kind::Free(Point { x: 0., y: 0. }));
    let c = document.add(Kind::Free(Point { x: 1., y: 1. }));
    let id = document.add(Kind::Bisector(a, vertex, c));
    assert_eq!(
        document.line_points(id),
        Some((
            Point { x: 0., y: 0. },
            Point { x: 0., y: 1. },
            LineMode::Ray
        ))
    );
    let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.line_points(id), document.line_points(id));
    document.move_point(vertex, Point { x: 0., y: 2. });
    assert_ne!(document.line_points(id), reopened.line_points(id));
}
#[test]
fn three_point_arc_matches_original_record_and_updates() {
    let original = Document::open(Path::new("tests/fixtures/three_point_arc.gsp")).unwrap();
    assert_eq!(original.objects[3].kind, Kind::ThreePointArc(1, 2, 3));
    assert!(!original.partial_view);
    assert!(original.arc_points(4, 32).is_some());
    let bytes = fs::read("tests/fixtures/three_point_arc.gsp").unwrap();
    let object_headers: Vec<_> = records(&bytes)
        .unwrap()
        .into_iter()
        .filter(|record| record.kind == 0x7d0)
        .collect();
    assert_eq!(&bytes[object_headers[3].data.clone()], &header(81));

    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: -1., y: 0. }));
    let mid = document.add(Kind::Free(Point { x: 0., y: -1. }));
    let end = document.add(Kind::Free(Point { x: 1., y: 0. }));
    let id = document.add(Kind::ThreePointArc(a, mid, end));
    let points = document.arc_points(id, 32).unwrap();
    assert!(points.first().unwrap().distance(Point { x: -1., y: 0. }) < 1e-9);
    assert!(points[16].distance(Point { x: 0., y: -1. }) < 1e-9);
    assert!(points.last().unwrap().distance(Point { x: 1., y: 0. }) < 1e-9);
    let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.arc_points(id, 32), document.arc_points(id, 32));
    document.move_point(mid, Point { x: 0., y: -2. });
    assert_ne!(reopened.arc_points(id, 32), document.arc_points(id, 32));
    document.move_point(mid, Point { x: 0., y: 0. });
    assert_eq!(
        document.arc_points(id, 32),
        Some(vec![Point { x: -1., y: 0. }, Point { x: 1., y: 0. },])
    );
}
#[test]
fn circle_by_segment_matches_original_and_updates() {
    let original = Document::open(Path::new("tests/fixtures/circle_center_radius.gsp")).unwrap();
    assert_eq!(original.objects[4].kind, Kind::CircleBySegment(4, 3));
    assert!(!original.partial_view);
    assert!(original.circle_points(5).is_some());
    let bytes = fs::read("tests/fixtures/circle_center_radius.gsp").unwrap();
    let object_headers: Vec<_> = records(&bytes)
        .unwrap()
        .into_iter()
        .filter(|record| record.kind == 0x7d0)
        .collect();
    assert_eq!(&bytes[object_headers[4].data.clone()], &header(4));

    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: 0., y: 0. }));
    let b = document.add(Kind::Free(Point { x: 5., y: 0. }));
    let center = document.add(Kind::Free(Point { x: 10., y: 10. }));
    let segment = document.add(Kind::Segment(a, b));
    let circle = document.add(Kind::CircleBySegment(center, segment));
    assert_eq!(
        document.circle_points(circle),
        Some((Point { x: 10., y: 10. }, 5.))
    );
    let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
    assert_eq!(
        reopened.circle_points(circle),
        document.circle_points(circle)
    );
    document.move_point(b, Point { x: 7., y: 0. });
    assert_eq!(
        document.circle_points(circle),
        Some((Point { x: 10., y: 10. }, 7.))
    );
}
#[test]
fn constrained_points_match_original_and_reopen_after_drag() {
    let segment_fixture = fs::read("tests/fixtures/point_on_segment.gsp").unwrap();
    let mut segment_doc = Document::from_bytes(&segment_fixture, None).unwrap();
    assert!(matches!(
        segment_doc.objects[3].kind,
        Kind::PointOnSegment(3, _)
    ));
    assert_eq!(segment_doc.encode().unwrap(), segment_fixture);
    let before = segment_doc.position(4).unwrap();
    segment_doc.move_point(1, Point { x: 270., y: 180. });
    assert_ne!(segment_doc.position(4), Some(before));
    assert!(segment_doc.move_point(4, Point { x: 300., y: 200. }));
    let reparsed = Document::from_bytes(&segment_doc.encode().unwrap(), None).unwrap();
    assert_eq!(reparsed.position(4), segment_doc.position(4));

    let circle_fixture = fs::read("tests/fixtures/point_on_circle.gsp").unwrap();
    let mut circle_doc = Document::from_bytes(&circle_fixture, None).unwrap();
    assert!(matches!(
        circle_doc.objects[3].kind,
        Kind::PointOnCircle(3, _, _)
    ));
    assert_eq!(circle_doc.encode().unwrap(), circle_fixture);
    let (center, radius) = circle_doc.circle_points(3).unwrap();
    assert!((circle_doc.position(4).unwrap().distance(center) - radius).abs() < 1e-6);
    assert!(circle_doc.move_point(
        4,
        Point {
            x: center.x,
            y: center.y + radius
        }
    ));
    let reparsed = Document::from_bytes(&circle_doc.encode().unwrap(), None).unwrap();
    assert_eq!(reparsed.position(4), circle_doc.position(4));

    let mut created = Document::new();
    let a = created.add(Kind::Free(Point { x: 0., y: 0. }));
    let b = created.add(Kind::Free(Point { x: 10., y: 0. }));
    let segment = created.add(Kind::Segment(a, b));
    let on_segment = created.add(
        created
            .constrained_point(segment, Point { x: 4., y: 2. })
            .unwrap(),
    );
    let circle = created.add(Kind::Circle(a, b));
    let on_circle = created.add(
        created
            .constrained_point(circle, Point { x: 0., y: 9. })
            .unwrap(),
    );
    assert_eq!(
        created.objects[on_circle - 1].kind,
        Kind::PointOnCircle(circle, 0., -1.)
    );
    assert_eq!(created.position(on_segment), Some(Point { x: 4., y: 0. }));
    assert_eq!(created.position(on_circle), Some(Point { x: 0., y: 10. }));
    let reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.position(on_segment), created.position(on_segment));
    assert_eq!(reopened.position(on_circle), created.position(on_circle));
}
#[test]
fn arc_on_circle_matches_original_and_updates() {
    let bytes = fs::read("tests/fixtures/arc_on_circle.gsp").unwrap();
    let mut original = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(original.objects[5].kind, Kind::ArcOnCircle(3, 4, 5));
    assert_eq!(original.encode().unwrap(), bytes);
    let points = original.arc_points(6, 48).unwrap();
    assert!((points[0].distance(original.position(4).unwrap())) < 1e-6);
    assert!((points[48].distance(original.position(5).unwrap())) < 1e-6);
    original.move_point(4, Point { x: 300., y: 300. });
    assert_ne!(original.arc_points(6, 48).unwrap(), points);

    let mut created = Document::new();
    let center = created.add(Kind::Free(Point { x: 100., y: 100. }));
    let radius = created.add(Kind::Free(Point { x: 140., y: 100. }));
    let circle = created.add(Kind::Circle(center, radius));
    let start = created.add(Kind::PointOnCircle(circle, 1., 0.));
    let end = created.add(Kind::PointOnCircle(circle, 0., 1.));
    let arc = created.add(Kind::ArcOnCircle(circle, start, end));
    let generated = created.encode().unwrap();
    let reopened = Document::from_bytes(&generated, None).unwrap();
    assert_eq!(
        reopened.objects[arc - 1].kind,
        Kind::ArcOnCircle(circle, start, end)
    );
    assert_eq!(reopened.arc_points(arc, 48), created.arc_points(arc, 48));
    assert!(!reopened.partial_view);
}
#[test]
fn point_on_arc_matches_original_and_updates() {
    let bytes = fs::read("tests/fixtures/point_on_arc.gsp").unwrap();
    let mut original = Document::from_bytes(&bytes, None).unwrap();
    assert!(matches!(original.objects[4].kind, Kind::PointOnArc(4, _)));
    assert_eq!(original.encode().unwrap(), bytes);
    assert!(original.position(5).is_some());
    original.move_point(1, Point { x: 330., y: 220. });
    let before_drag = original.position(5).unwrap();
    assert!(original.move_point(5, Point { x: 300., y: 160. }));
    assert_ne!(original.position(5), Some(before_drag));
    let reopened = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.position(5), original.position(5));

    let mut created = Document::new();
    let a = created.add(Kind::Free(Point { x: 100., y: 200. }));
    let b = created.add(Kind::Free(Point { x: 150., y: 150. }));
    let c = created.add(Kind::Free(Point { x: 200., y: 200. }));
    let arc = created.add(Kind::ThreePointArc(a, b, c));
    let p = created.add(Kind::PointOnArc(arc, 0.5));
    let reopened = Document::from_bytes(&created.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.position(p), created.position(p));
    assert_eq!(reopened.objects[p - 1].kind, Kind::PointOnArc(arc, 0.5));
}

#[test]
fn native_straight_direction_controls_pin_circle_branches_and_line_parameters() {
    let rows = [
        (7, 8, 9, 180., 180.),
        (16, 17, 18, 480., 180.),
        (25, 26, 27, 780., 180.),
        (35, 36, 37, 220., 480.),
        (45, 46, 47, 620., 480.),
    ];
    for (name, t, reverse, offbase, clipped) in [
        ("probe", 0.5, false, false, false),
        ("quarter", 0.25, false, false, false),
        ("three_quarter", 0.75, false, false, false),
        ("reversed", 0.5, true, false, false),
        ("offbase", 0.5, false, true, false),
        ("clipped", 0.5, false, false, true),
    ] {
        let bytes = fs::read(format!(
            "tests/fixtures/straight_direction_{name}_native_verified.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert!(!d.partial_view);
        assert_eq!(d.objects.len(), 47);
        for (index, &(first, second, parameter, x, y)) in rows.iter().enumerate() {
            let (x, y) = if offbase && index >= 3 {
                (x + 40., y + 30.)
            } else {
                (x, y)
            };
            let sign = if reverse { -1. } else { 1. };
            let (dx, dy) = if index == 4 {
                (0., 60. * sign)
            } else {
                (60. * sign, 0.)
            };
            assert_eq!(
                d.position(first),
                Some(Point {
                    x: x + dx,
                    y: y + dy
                }),
                "{name} branch0 row{index}"
            );
            let expected_second = if clipped && matches!(index, 0 | 2) {
                None
            } else {
                Some(Point {
                    x: x - dx,
                    y: y - dy,
                })
            };
            assert_eq!(
                d.position(second),
                expected_second,
                "{name} branch1 row{index}"
            );
            let offset = if clipped && index < 3 {
                50.
            } else {
                200. * (t - 0.5) * sign
            };
            let expected = if index == 4 {
                Point { x, y: y + offset }
            } else {
                Point { x: x + offset, y }
            };
            assert_eq!(
                d.position(parameter),
                Some(expected),
                "{name} parameter row{index}"
            );
        }
        assert_eq!(d.encode().unwrap(), bytes);
        d.verify_roundtrip().unwrap();
    }
}

#[test]
fn edited_derived_line_anchors_keep_native_center_and_magnitude() {
    let bytes =
        include_bytes!("../../../tests/fixtures/straight_direction_probe_native_verified.gsp");
    let edited =
        include_bytes!("../../../tests/fixtures/straight_direction_offbase_native_verified.gsp");
    let reversed =
        include_bytes!("../../../tests/fixtures/straight_direction_reversed_native_verified.gsp");
    let mut d = Document::from_bytes(bytes, None).unwrap();
    for id in [30, 31, 40, 41] {
        let p = d.position(id).unwrap();
        assert!(d.move_point(
            id,
            Point {
                x: p.x + 40.,
                y: p.y + 30.
            }
        ));
    }
    assert_eq!(d.position(37), Some(Point { x: 260., y: 510. }));
    assert_eq!(d.position(47), Some(Point { x: 660., y: 510. }));
    assert_eq!(d.encode().unwrap(), edited);
    d.verify_roundtrip().unwrap();
    let mut d = Document::from_bytes(bytes, None).unwrap();
    for (a, b) in [(1, 2), (10, 11), (19, 20), (28, 29), (38, 39)] {
        let pa = d.position(a).unwrap();
        let pb = d.position(b).unwrap();
        assert!(d.move_point(a, pb));
        assert!(d.move_point(b, pa));
    }
    assert_eq!(d.position(45), Some(Point { x: 620., y: 420. }));
    assert_eq!(d.position(46), Some(Point { x: 620., y: 540. }));
    assert_eq!(d.encode().unwrap(), reversed);
    d.verify_roundtrip().unwrap();
}

fn center_arc_document(end: Point) -> Document {
    let mut d = Document::new();
    let center = d.add(Kind::Free(Point { x: 200., y: 180. }));
    let start = d.add(Kind::Free(Point { x: 260., y: 180. }));
    let end = d.add(Kind::Free(end));
    let arc = d.add(Kind::CenterArc(center, start, end));
    d.add(Kind::ArcAngle(arc));
    d.add(Kind::ArcLength(arc));
    let midpoint = d.add(Kind::PointOnArc(arc, 0.5));
    d.add(Kind::Segment(center, midpoint));
    d
}

#[test]
fn centered_arcs_import_native_geometry_and_match_native_writer() {
    for (bytes, angle) in [
        (
            include_bytes!("../../../tests/fixtures/center_arc_quarter_native_verified.gsp")
                .as_slice(),
            Some(90.),
        ),
        (
            include_bytes!("../../../tests/fixtures/center_arc_radial_native_verified.gsp")
                .as_slice(),
            Some(90.),
        ),
        (
            include_bytes!("../../../tests/fixtures/center_arc_major_native_verified.gsp")
                .as_slice(),
            Some(270.),
        ),
        (
            include_bytes!("../../../tests/fixtures/center_arc_coincident_native_verified.gsp")
                .as_slice(),
            None,
        ),
    ] {
        let d = Document::from_bytes(bytes, None).unwrap();
        assert!(!d.partial_view);
        assert!(matches!(d.objects[3].kind, Kind::CenterArc(1, 2, _)));
        if let Some(angle) = angle {
            assert!((d.numeric_value(5).unwrap() - angle).abs() < 1e-12);
            assert!(
                (d.numeric_value(6).unwrap() - angle.to_radians() * 60. / 37.79527559055118).abs()
                    < 1e-12
            );
            let theta = angle.to_radians() * 0.5;
            assert!(
                d.position(7).unwrap().distance(Point {
                    x: 200. + 60. * theta.cos(),
                    y: 180. - 60. * theta.sin()
                }) < 1e-10
            );
        } else {
            assert_eq!(d.numeric_value(5), None);
            assert_eq!(d.numeric_value(6), None);
            assert_eq!(d.position(7), None);
            assert_eq!(d.arc_points(4, 20), None);
        }
        assert_eq!(d.encode().unwrap(), bytes);
        d.verify_roundtrip().unwrap();
    }
    let d = center_arc_document(Point { x: 200., y: 120. });
    assert_eq!(
        d.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/center_arc_quarter_native_verified.gsp")
    );
}

#[test]
fn centered_arcs_drive_points_values_measurements_and_edit_history() {
    let mut d = center_arc_document(Point { x: 200., y: 60. });
    let t = d.add_named_parameter("t", 0.25, (10, 120)).unwrap();
    let image = d.plot_parameter_on_path(t, Some(4)).unwrap();
    let literal = d.plot_literal_on_path(0.5, Some(4)).unwrap();
    let value = d.add(Kind::ValueEquation {
        kind: crate::value_equation_records::ValueEquationKind::PointValue,
        first: image,
        second: 4,
    });
    let scalar = d.add_parameter_calculation(5, "v/180°", (10, 160)).unwrap();
    assert!((d.numeric_value(value).unwrap() - 0.25).abs() < 1e-12);
    assert_eq!(d.numeric_value(scalar), Some(0.5));
    assert_eq!(d.position(literal), d.position(7));
    let before = d.encode().unwrap();
    d.set_parameter_value(t, 1.).unwrap();
    assert!(
        d.position(image)
            .unwrap()
            .distance(Point { x: 200., y: 120. })
            < 1e-10
    );
    d.checkpoint();
    assert!(d.move_point(3, Point { x: 200., y: 240. }));
    assert_eq!(d.numeric_value(5), Some(270.));
    assert_eq!(d.numeric_value(scalar), Some(1.5));
    d.verify_roundtrip().unwrap();
    let after = d.encode().unwrap();
    assert_eq!(
        after,
        include_bytes!("../../../tests/fixtures/center_arc_dynamic_writer_native_verified.gsp")
    );
    d.undo();
    d.undo();
    assert_eq!(d.encode().unwrap(), before);
    d.redo();
    d.redo();
    assert_eq!(d.encode().unwrap(), after);
    d.checkpoint();
    assert!(d.move_point(3, Point { x: 320., y: 180. }));
    assert_eq!(d.position(image), None);
    assert_eq!(d.position(literal), None);
    assert_eq!(d.numeric_value(5), None);
    d.verify_roundtrip().unwrap();
    assert_eq!(
        d.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/center_arc_undefined_writer_native_verified.gsp")
    );
}

#[test]
fn centered_arc_original_clock_keeps_small_sweeps_and_source_records() {
    let bytes = include_bytes!("../../../tests/fixtures/center_arc_clock_corpus.gsp");
    let d = Document::from_bytes(bytes, None).unwrap();
    assert!(matches!(
        d.objects.iter().find(|o| o.id == 94).unwrap().kind,
        Kind::CenterArc(6, 90, 93)
    ));
    assert!((d.numeric_value(95).unwrap() - 0.001).abs() < 1e-12);
    let center = d.position(6).unwrap();
    let radius = center.distance(d.position(90).unwrap());
    assert!((radius - 98.5).abs() < 1e-10);
    assert!((center.distance(d.arc_points(94, 2).unwrap()[1]) - radius).abs() < 1e-10);
    assert_eq!(d.encode().unwrap(), bytes);
    d.verify_roundtrip().unwrap();
}
