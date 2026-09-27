use super::*;

#[test]
fn multi_page_point_additions_refuse_cross_page_hosts() {
    let base = include_bytes!("../../../tests/fixtures/two_page_segment_constrained_native.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 0;
    document.add(Kind::PointOnSegment(4, 0.5));
    assert!(document.can_save().is_err());
}

#[test]
fn points_on_lines_and_rays_match_native_groups_and_move() {
    let group = |bytes: &[u8]| {
        let rs = records(bytes).unwrap();
        let start = rs
            .iter()
            .rfind(|record| record.kind == 0x07d0)
            .unwrap()
            .start;
        let end = rs
            .iter()
            .rfind(|record| record.kind == 0x07d7)
            .unwrap()
            .data
            .end;
        bytes[start..end].to_vec()
    };
    for (base, native, kind, moved, expected) in [
        (
            include_bytes!("../../../tests/fixtures/line.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/point_on_line_native.gsp").as_slice(),
            Kind::PointOnLine(3, 0.3),
            Point { x: 243., y: 212. },
            Kind::PointOnLine(3, -0.5),
        ),
        (
            include_bytes!("../../../tests/fixtures/ray.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/point_on_ray_native.gsp").as_slice(),
            Kind::PointOnRay(3, 0.3),
            Point { x: 603., y: 234.5 },
            Kind::PointOnRay(3, 1.75),
        ),
    ] {
        let imported = Document::from_bytes(native, None).unwrap();
        assert!(!imported.partial_view);
        assert_eq!(imported.objects[3].kind, kind);
        assert_eq!(imported.encode().unwrap(), native);

        let mut document = Document::from_bytes(base, None).unwrap();
        assert_eq!(document.add(kind), 4);
        let encoded = document.encode().unwrap();
        assert_eq!(group(&encoded), group(native));
        assert!(document.move_point(4, moved));
        assert_eq!(document.objects[3].kind, expected);
        let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.objects[3].kind, expected);
        assert_eq!(reopened.position(4), Some(moved));
    }
}

#[test]
fn multi_page_points_on_lines_and_rays_reopen() {
    for (base, kind) in [
        (
            include_bytes!("../../../tests/fixtures/two_page_line_native.gsp").as_slice(),
            Kind::PointOnLine(5, -0.5),
        ),
        (
            include_bytes!("../../../tests/fixtures/two_page_ray_native.gsp").as_slice(),
            Kind::PointOnRay(5, 1.5),
        ),
    ] {
        let mut document = Document::from_bytes(base, None).unwrap();
        assert_eq!(document.add(kind.clone()), 6);
        let encoded = document.encode().unwrap();
        let reopened = Document::from_bytes(&encoded, None).unwrap();
        assert_eq!(reopened.objects[5].kind, kind);
        assert_eq!(reopened.objects[5].page, 1);
        assert_eq!(reopened.encode().unwrap(), encoded);
    }
}
#[test]
fn native_ray_and_line_point_values_follow_moved_points() {
    for (native, moved, expected) in [
        (
            include_bytes!("../../../tests/fixtures/point_on_line_value_native.gsp").as_slice(),
            Point { x: 243., y: 212. },
            -0.5,
        ),
        (
            include_bytes!("../../../tests/fixtures/point_on_ray_value_native.gsp").as_slice(),
            Point { x: 603., y: 234.5 },
            1.75,
        ),
    ] {
        let mut document = Document::from_bytes(native, None).unwrap();
        assert!(!document.partial_view);
        assert!(matches!(
            document.objects[4].kind,
            Kind::ValueEquation {
                kind: ValueEquationKind::PointValue,
                first: 4,
                second: 3
            }
        ));
        assert!(
            matches!(document.value_equation_value(5), Some(EquationValue::PointValue(value)) if (value - 0.3).abs() < 1e-12)
        );
        assert_eq!(document.encode().unwrap(), native);
        assert!(document.move_point(4, moved));
        let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
        assert!(
            matches!(reopened.value_equation_value(5), Some(EquationValue::PointValue(value)) if (value - expected).abs() < 1e-12)
        );
    }
}
#[test]
fn native_arc_point_values_follow_moved_points() {
    for (native, point_id, moved, expected) in [
        (
            include_bytes!("../../../tests/fixtures/point_on_arc_value_native.gsp").as_slice(),
            5,
            Point {
                x: 209.17111318359585,
                y: 469.68286247339006,
            },
            0.4,
        ),
        (
            include_bytes!("../../../tests/fixtures/point_on_circle_arc_value_native.gsp")
                .as_slice(),
            7,
            Point {
                x: 436.54086027855686,
                y: 167.14548117567242,
            },
            0.8,
        ),
    ] {
        let mut document = Document::from_bytes(native, None).unwrap();
        assert!(!document.partial_view);
        let measurement = point_id + 1;
        assert!(matches!(
            document.objects[measurement - 1].kind,
            Kind::ValueEquation {
                kind: ValueEquationKind::PointValue,
                first,
                second: _
            } if first == point_id
        ));
        assert_eq!(document.encode().unwrap(), native);
        assert!(document.move_point(point_id, moved));
        let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
        assert!(
            matches!(reopened.value_equation_value(measurement), Some(EquationValue::PointValue(value)) if (value - expected).abs() < 1e-12)
        );
    }
}

#[test]
fn multi_page_constrained_point_additions_reopen() {
    for (index, base, kind, expected) in [
        (
            0,
            include_bytes!("../../../tests/fixtures/two_page_segment_constrained_native.gsp")
                .as_slice(),
            Kind::PointOnSegment(4, 0.75),
            include_bytes!("../../../tests/fixtures/two_page_segment_extra_point_port.gsp")
                .as_slice(),
        ),
        (
            1,
            include_bytes!("../../../tests/fixtures/two_page_circle_constrained_native.gsp")
                .as_slice(),
            Kind::PointOnCircle(4, -1., -0.),
            include_bytes!("../../../tests/fixtures/two_page_circle_extra_point_port.gsp")
                .as_slice(),
        ),
        (
            2,
            include_bytes!("../../../tests/fixtures/two_page_arc_constrained_native.gsp")
                .as_slice(),
            Kind::PointOnArc(5, 0.6887639439801854),
            include_bytes!("../../../tests/fixtures/two_page_arc_extra_point_port.gsp").as_slice(),
        ),
    ] {
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = 1;
        let id = document.add(kind);
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_multi_page_add_constraint_{}_{}.gsp",
            std::process::id(),
            index
        ));
        document.save_to(&path).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), expected);
        assert_eq!(document.objects[id - 1].page, 1);
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn multi_page_straight_and_circle_use_page_local_parents() {
    fn last_object_group(bytes: &[u8]) -> &[u8] {
        let entries = records(bytes).unwrap();
        let start = entries
            .iter()
            .rposition(|entry| entry.kind == 0x07d0)
            .unwrap();
        let end = start
            + entries[start..]
                .iter()
                .position(|entry| entry.kind == 0x07d7)
                .unwrap();
        &bytes[entries[start].start..entries[end].data.end]
    }
    let base = include_bytes!("../../../tests/fixtures/two_page_blank_native.gsp");
    for (index, first, second, expected, native) in [
        (
            0,
            Point { x: 111., y: 111. },
            Point { x: 291., y: 111. },
            include_bytes!("../../../tests/fixtures/two_page_segment_port.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/two_page_segment_native.gsp").as_slice(),
        ),
        (
            1,
            Point { x: 201., y: 111. },
            Point { x: 291., y: 111. },
            include_bytes!("../../../tests/fixtures/two_page_circle_port.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/two_page_circle_native.gsp").as_slice(),
        ),
        (
            2,
            Point { x: 111., y: 111. },
            Point { x: 291., y: 111. },
            include_bytes!("../../../tests/fixtures/two_page_line_port.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/two_page_line_native.gsp").as_slice(),
        ),
        (
            3,
            Point { x: 111., y: 111. },
            Point { x: 291., y: 111. },
            include_bytes!("../../../tests/fixtures/two_page_ray_port.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/two_page_ray_native.gsp").as_slice(),
        ),
    ] {
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = 1;
        let a = document.add(Kind::Free(first));
        let b = document.add(Kind::Free(second));
        let path_id = document.add(match index {
            0 => Kind::Segment(a, b),
            1 => Kind::Circle(a, b),
            2 => Kind::Line(a, b),
            3 => Kind::Ray(a, b),
            _ => unreachable!(),
        });
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_multi_page_path_{}_{}.gsp",
            std::process::id(),
            index
        ));
        document.save_to(&path).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), expected);
        assert_eq!(last_object_group(expected), last_object_group(native));
        assert_eq!(document.objects[path_id - 1].page, 1);
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn multi_page_constructed_paths_and_points_remap_across_pages() {
    let base = include_bytes!("../../../tests/fixtures/two_page_blank_native.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 0;
    let first = document.add(Kind::Free(Point { x: 150., y: 200. }));
    let second = document.add(Kind::Free(Point { x: 250., y: 200. }));
    document.add(Kind::Segment(first, second));
    document.active_page = 1;
    let center = document.add(Kind::Free(Point { x: 111., y: 111. }));
    let radius = document.add(Kind::Free(Point { x: 291., y: 111. }));
    let segment = document.add(Kind::Segment(center, radius));
    document.add(Kind::PointOnSegment(segment, 0.5));
    let circle = document.add(Kind::Circle(center, radius));
    document.add(Kind::PointOnCircle(circle, 0., -1.));
    document.add_link_page_button(0).unwrap();
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_multi_page_construct_mixed_{}.gsp",
        std::process::id()
    ));
    document.save_to(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        include_bytes!("../../../tests/fixtures/two_page_construct_mixed_port.gsp")
    );
    assert_eq!(document.objects[6].kind, Kind::Segment(5, 6));
    assert_eq!(document.objects[8].kind, Kind::Circle(5, 6));
    assert_eq!(document.objects[9].kind, Kind::PointOnCircle(9, 0., -1.));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn multi_page_path_addition_rejects_cross_page_parent() {
    let base = include_bytes!("../../../tests/fixtures/two_page_blank_native.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 1;
    let new_point = document.add(Kind::Free(Point { x: 200., y: 200. }));
    document.add(Kind::Segment(1, new_point));
    assert!(document.can_save().is_err());
}
#[test]
fn multi_page_midpoint_and_circle_by_segment_match_native_groups() {
    fn last_group(bytes: &[u8]) -> &[u8] {
        let entries = records(bytes).unwrap();
        let start = entries
            .iter()
            .rposition(|entry| entry.kind == 0x07d0)
            .unwrap();
        let end = start
            + entries[start..]
                .iter()
                .position(|entry| entry.kind == 0x07d7)
                .unwrap();
        &bytes[entries[start].start..entries[end].data.end]
    }
    let base = include_bytes!("../../../tests/fixtures/two_page_segment_native.gsp");
    for (index, kind, expected, native) in [
        (
            0,
            Kind::Midpoint(2, 3),
            include_bytes!("../../../tests/fixtures/two_page_midpoint_port.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/two_page_midpoint_native.gsp").as_slice(),
        ),
        (
            1,
            Kind::CircleBySegment(2, 4),
            include_bytes!("../../../tests/fixtures/two_page_circle_segment_port.gsp").as_slice(),
            include_bytes!("../../../tests/fixtures/two_page_circle_segment_native.gsp").as_slice(),
        ),
    ] {
        let mut document = Document::from_bytes(base, None).unwrap();
        document.active_page = 1;
        let id = document.add(kind.clone());
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_multi_page_mid_circle_{}_{}.gsp",
            std::process::id(),
            index
        ));
        document.save_to(&path).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), expected);
        assert_eq!(last_group(expected), last_group(native));
        assert_eq!(document.objects[id - 1].kind, kind);
        if index == 0 {
            assert_eq!(document.objects[id - 1].midpoint_host, Some(4));
        }
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn multi_page_midpoint_and_circle_by_new_segment_reopen() {
    let base = include_bytes!("../../../tests/fixtures/two_page_blank_native.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 1;
    let a = document.add(Kind::Free(Point { x: 111., y: 111. }));
    let b = document.add(Kind::Free(Point { x: 291., y: 111. }));
    let segment = document.add(Kind::Segment(a, b));
    let midpoint = document.add(Kind::Midpoint(a, b));
    let circle = document.add(Kind::CircleBySegment(a, segment));
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_multi_page_new_mid_circle_{}.gsp",
        std::process::id()
    ));
    document.save_to(&path).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        include_bytes!("../../../tests/fixtures/two_page_midpoint_circle_segment_mixed_port.gsp")
    );
    assert_eq!(document.objects[midpoint - 1].midpoint_host, Some(segment));
    assert_eq!(
        document.objects[circle - 1].kind,
        Kind::CircleBySegment(a, segment)
    );
    std::fs::remove_file(path).unwrap();
}
#[test]
fn multi_page_derived_lines_and_intersections_save() {
    fn object_group_with_code(bytes: &[u8], code: u16) -> &[u8] {
        let entries = records(bytes).unwrap();
        let start = entries
            .iter()
            .position(|entry| {
                entry.kind == 0x07d0
                    && entry.data.len() >= 2
                    && u16::from_le_bytes(
                        bytes[entry.data.start..entry.data.start + 2]
                            .try_into()
                            .unwrap(),
                    ) == code
            })
            .unwrap();
        let end = start
            + entries[start..]
                .iter()
                .position(|entry| entry.kind == 0x07d7)
                .unwrap();
        &bytes[entries[start].start..entries[end].data.end]
    }
    fn saved(mut document: Document, expected: &[u8], name: &str) -> Document {
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_multi_page_{}_{}.gsp",
            name,
            std::process::id()
        ));
        document.save_to(&path).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), expected);
        std::fs::remove_file(path).unwrap();
        document
    }
    let segment_base = include_bytes!("../../../tests/fixtures/two_page_segment_native.gsp");
    let mut document = Document::from_bytes(segment_base, None).unwrap();
    document.active_page = 1;
    let point = document.add(Kind::Free(Point { x: 201., y: 211. }));
    document.add(Kind::Parallel(point, 4));
    document.add(Kind::Perpendicular(point, 4));
    let reopened = saved(
        document,
        include_bytes!("../../../tests/fixtures/two_page_parallel_perpendicular_port.gsp"),
        "derived",
    );
    assert!(reopened.line_points(6).is_some());
    assert!(reopened.line_points(7).is_some());
    let generated =
        include_bytes!("../../../tests/fixtures/two_page_parallel_perpendicular_port.gsp");
    assert_eq!(
        object_group_with_code(generated, 6),
        object_group_with_code(
            include_bytes!("../../../tests/fixtures/two_page_parallel_native.gsp"),
            6,
        )
    );
    assert_eq!(
        object_group_with_code(generated, 5),
        object_group_with_code(
            include_bytes!("../../../tests/fixtures/two_page_perpendicular_native.gsp"),
            5,
        )
    );

    let mut document = Document::from_bytes(segment_base, None).unwrap();
    document.active_page = 1;
    let first = document.add(Kind::Free(Point { x: 201., y: 51. }));
    let second = document.add(Kind::Free(Point { x: 201., y: 211. }));
    let line = document.add(Kind::Line(first, second));
    let crossing = document.add(Kind::Intersection(4, line));
    let reopened = saved(
        document,
        include_bytes!("../../../tests/fixtures/two_page_straight_intersection_port.gsp"),
        "straight_crossing",
    );
    assert_eq!(
        reopened.position(crossing),
        Some(Point { x: 201., y: 111. })
    );
    assert_eq!(
        object_group_with_code(
            include_bytes!("../../../tests/fixtures/two_page_straight_intersection_port.gsp"),
            9,
        ),
        object_group_with_code(
            include_bytes!("../../../tests/fixtures/two_page_straight_intersection_native.gsp"),
            9,
        )
    );

    let mut document = Document::from_bytes(segment_base, None).unwrap();
    document.active_page = 1;
    let center = document.add(Kind::Free(Point { x: 201., y: 161. }));
    let radius = document.add(Kind::Free(Point { x: 281., y: 161. }));
    let circle = document.add(Kind::Circle(center, radius));
    let first = document.add(Kind::LineCircleIntersection(4, circle, 0));
    let second = document.add(Kind::LineCircleIntersection(4, circle, 1));
    let reopened = saved(
        document,
        include_bytes!("../../../tests/fixtures/two_page_line_circle_intersections_port.gsp"),
        "line_circle_crossings",
    );
    assert_ne!(reopened.position(first), reopened.position(second));
    assert!(reopened.position(first).is_some());
    assert!(reopened.position(second).is_some());

    let mut document = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/two_page_blank_native.gsp"),
        None,
    )
    .unwrap();
    document.active_page = 1;
    let a = document.add(Kind::Free(Point { x: 160., y: 160. }));
    let b = document.add(Kind::Free(Point { x: 260., y: 160. }));
    let first_circle = document.add(Kind::Circle(a, b));
    let c = document.add(Kind::Free(Point { x: 240., y: 160. }));
    let d = document.add(Kind::Free(Point { x: 340., y: 160. }));
    let second_circle = document.add(Kind::Circle(c, d));
    let first = document.add(Kind::CircleCircleIntersection(
        first_circle,
        second_circle,
        0,
    ));
    let second = document.add(Kind::CircleCircleIntersection(
        first_circle,
        second_circle,
        1,
    ));
    let reopened = saved(
        document,
        include_bytes!("../../../tests/fixtures/two_page_circle_circle_intersections_port.gsp"),
        "circle_crossings",
    );
    assert_ne!(reopened.position(first), reopened.position(second));
    assert!(reopened.position(first).is_some());
    assert!(reopened.position(second).is_some());
}

#[test]
fn multi_page_intersection_requires_finite_crossing() {
    let base = include_bytes!("../../../tests/fixtures/two_page_segment_native.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 1;
    let point = document.add(Kind::Free(Point { x: 201., y: 211. }));
    let parallel = document.add(Kind::Parallel(point, 4));
    document.add(Kind::Intersection(4, parallel));
    assert!(document.can_save().is_err());
}
#[test]
fn multi_page_bisector_and_arcs_save() {
    fn saved(mut document: Document, expected: &[u8], name: &str) {
        let path = std::env::temp_dir().join(format!(
            "sketchpad_rs_multi_page_{}_{}.gsp",
            name,
            std::process::id()
        ));
        document.save_to(&path).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), expected);
        std::fs::remove_file(path).unwrap();
    }
    let base = include_bytes!("../../../tests/fixtures/two_page_blank_native.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 1;
    let a = document.add(Kind::Free(Point { x: 160., y: 160. }));
    let b = document.add(Kind::Free(Point { x: 210., y: 100. }));
    let c = document.add(Kind::Free(Point { x: 260., y: 160. }));
    let bisector = document.add(Kind::Bisector(a, b, c));
    let arc = document.add(Kind::ThreePointArc(a, b, c));
    assert!(document.line_points(bisector).is_some());
    assert!(document.point_on_arc(arc, 0.5).is_some());
    saved(
        document,
        include_bytes!("../../../tests/fixtures/two_page_bisector_arc_port.gsp"),
        "bisector_arc",
    );

    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 1;
    let center = document.add(Kind::Free(Point { x: 160., y: 160. }));
    let radius = document.add(Kind::Free(Point { x: 260., y: 160. }));
    let circle = document.add(Kind::Circle(center, radius));
    let start = document.add(
        document
            .constrained_point(circle, Point { x: 260., y: 160. })
            .unwrap(),
    );
    let end = document.add(
        document
            .constrained_point(circle, Point { x: 160., y: 60. })
            .unwrap(),
    );
    let arc = document.add(Kind::ArcOnCircle(circle, start, end));
    assert!(document.point_on_arc(arc, 0.5).is_some());
    saved(
        document,
        include_bytes!("../../../tests/fixtures/two_page_arc_on_circle_port.gsp"),
        "arc_on_circle",
    );
}
#[test]
fn multi_page_polygon_interiors_save_on_both_pages() {
    let base = include_bytes!("../../../tests/fixtures/two_page_blank_native.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 1;
    let a = document.add(Kind::Free(Point { x: 120., y: 180. }));
    let b = document.add(Kind::Free(Point { x: 200., y: 80. }));
    let c = document.add(Kind::Free(Point { x: 300., y: 180. }));
    let polygon = document.add(Kind::Polygon(vec![a, b, c]));
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_multi_page_polygon_{}.gsp",
        std::process::id()
    ));
    document.save_to(&path).unwrap();
    let expected = include_bytes!("../../../tests/fixtures/two_page_polygon_port.gsp");
    assert_eq!(std::fs::read(&path).unwrap(), expected);
    assert_eq!(
        document.objects[polygon - 1].kind,
        Kind::Polygon(vec![a, b, c])
    );
    std::fs::remove_file(&path).unwrap();

    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 0;
    let a = document.add(Kind::Free(Point { x: 170., y: 170. }));
    let b = document.add(Kind::Free(Point { x: 220., y: 90. }));
    document.add(Kind::Polygon(vec![1, a, b]));
    document.active_page = 1;
    let a = document.add(Kind::Free(Point { x: 120., y: 180. }));
    let b = document.add(Kind::Free(Point { x: 200., y: 80. }));
    let c = document.add(Kind::Free(Point { x: 300., y: 180. }));
    document.add(Kind::Polygon(vec![a, b, c]));
    document.save_to(&path).unwrap();
    let expected = include_bytes!("../../../tests/fixtures/two_page_polygons_both_pages_port.gsp");
    assert_eq!(std::fs::read(&path).unwrap(), expected);
    let reopened = Document::from_bytes(expected, None).unwrap();
    assert_eq!(reopened.objects[3].kind, Kind::Polygon(vec![1, 2, 3]));
    assert_eq!(reopened.objects[7].kind, Kind::Polygon(vec![5, 6, 7]));
    std::fs::remove_file(path).unwrap();

    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 1;
    let a = document.add(Kind::Free(Point { x: 120., y: 180. }));
    let b = document.add(Kind::Free(Point { x: 200., y: 80. }));
    document.add(Kind::Polygon(vec![1, a, b]));
    assert!(document.can_save().is_err());
}
#[test]
fn multi_page_one_host_measurements_save() {
    let base = include_bytes!("../../../tests/fixtures/two_page_blank_native.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 1;
    let a = document.add(Kind::Free(Point { x: 120., y: 180. }));
    let b = document.add(Kind::Free(Point { x: 200., y: 80. }));
    let c = document.add(Kind::Free(Point { x: 300., y: 180. }));
    let polygon = document.add(Kind::Polygon(vec![a, b, c]));
    let segment = document.add(Kind::Segment(a, c));
    let circle = document.add(Kind::Circle(a, c));
    let arc = document.add(Kind::ThreePointArc(a, b, c));
    for kind in [
        Kind::SegmentLength(segment),
        Kind::PolygonPerimeter(polygon),
        Kind::CircleCircumference(circle),
        Kind::CircleRadius(circle),
        Kind::CircleArea(circle),
        Kind::PolygonArea(polygon),
        Kind::ArcAngle(arc),
        Kind::ArcLength(arc),
    ] {
        document.add(kind);
    }
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_multi_page_measurements_{}.gsp",
        std::process::id()
    ));
    document.save_to(&path).unwrap();
    let expected =
        include_bytes!("../../../tests/fixtures/two_page_one_host_measurements_port.gsp");
    assert_eq!(std::fs::read(&path).unwrap(), expected);
    let reopened = Document::from_bytes(expected, None).unwrap();
    assert!(
        reopened.objects[8..]
            .iter()
            .all(|object| object.kind.is_measurement())
    );
    std::fs::remove_file(path).unwrap();
}
#[test]
fn multi_page_point_measurements_append_after_existing_labels() {
    let base = include_bytes!("../../../tests/fixtures/two_page_one_host_measurements_port.gsp");
    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 1;
    let target = document.add(Kind::Free(Point { x: 210., y: 180. }));
    document.add(Kind::PointDistance(2, 4));
    document.add(Kind::AngleMeasurement(2, 3, 4));
    document.add(Kind::PointRatio(2, 4, target));
    let path = std::env::temp_dir().join(format!(
        "sketchpad_rs_multi_page_point_measures_{}.gsp",
        std::process::id()
    ));
    document.save_to(&path).unwrap();
    let expected = include_bytes!("../../../tests/fixtures/two_page_point_measurements_port.gsp");
    assert_eq!(std::fs::read(&path).unwrap(), expected);
    let reopened = Document::from_bytes(expected, None).unwrap();
    assert_eq!(reopened.objects[17].kind, Kind::PointDistance(2, 4));
    assert_eq!(reopened.objects[18].kind, Kind::AngleMeasurement(2, 3, 4));
    assert_eq!(reopened.objects[19].kind, Kind::PointRatio(2, 4, target));
    std::fs::remove_file(path).unwrap();

    let mut document = Document::from_bytes(base, None).unwrap();
    document.active_page = 1;
    document.add(Kind::PointDistance(1, 2));
    assert!(document.can_save().is_err());
}

#[test]
fn centered_arc_multi_page_construction_uses_local_references() {
    let base = include_bytes!("../../../tests/fixtures/two_page_blank_native.gsp");
    let mut d = Document::from_bytes(base, None).unwrap();
    d.active_page = 1;
    let center = d.add(Kind::Free(Point { x: 200., y: 180. }));
    let start = d.add(Kind::Free(Point { x: 260., y: 180. }));
    let end = d.add(Kind::Free(Point { x: 200., y: 60. }));
    let arc = d.add(Kind::CenterArc(center, start, end));
    let midpoint = d.add(Kind::PointOnArc(arc, 0.5));
    let bytes = d.encode().unwrap();
    let saved = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(
        saved.objects[arc - 1].kind,
        Kind::CenterArc(center, start, end)
    );
    assert_eq!(saved.position(midpoint), d.position(midpoint));
    assert_eq!(saved.encode().unwrap(), bytes);
    d.verify_roundtrip().unwrap();
}
