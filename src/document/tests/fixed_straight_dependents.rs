use super::*;

#[test]
fn rotated_straight_constrained_points_keep_native_modes_and_live_dependencies() {
    for (ray, expected, probe) in [
        (
            true,
            include_bytes!(
                "../../../tests/fixtures/rotated_ray_boundary_point_native_verified.gsp"
            )
            .as_slice(),
            Point {
                x: 200.71067811865476,
                y: 135.85786437626905,
            },
        ),
        (
            false,
            include_bytes!(
                "../../../tests/fixtures/rotated_line_boundary_point_native_verified.gsp"
            )
            .as_slice(),
            Point {
                x: 172.42640687119285,
                y: 164.14213562373095,
            },
        ),
    ] {
        let mut document = Document::new();
        for point in [
            Point { x: 130., y: 150. },
            Point { x: 170., y: 190. },
            Point { x: 210., y: 190. },
        ] {
            document.add(Kind::Free(point));
        }
        document.add(if ray {
            Kind::Ray(2, 3)
        } else {
            Kind::Line(2, 3)
        });
        document.add(if ray {
            Kind::RotatedRay(4, 1, 45.)
        } else {
            Kind::RotatedLine(4, 1, 45.)
        });
        let constrained = document.constrained_point(5, probe).unwrap();
        let fraction = match constrained {
            Kind::PointOnRay(5, t) if ray => t,
            Kind::PointOnLine(5, t) if !ray => t,
            _ => panic!("Wrong constrained point mode"),
        };
        assert!((fraction - if ray { 0.5 } else { -0.5 }).abs() < 1e-12);
        document.add(constrained);
        assert_eq!(document.encode().unwrap(), expected);
        let original = document.position(6).unwrap();
        assert!(original.distance(probe) < 1e-10);
        document.checkpoint();
        assert!(document.move_point(2, Point { x: 180., y: 200. }));
        let moved = document.position(6).unwrap();
        let (a, b, _) = document.line_points(5).unwrap();
        assert!(
            moved.distance(Point {
                x: a.x + fraction * (b.x - a.x),
                y: a.y + fraction * (b.y - a.y)
            }) < 1e-10
        );
        assert!(moved.distance(original) > 1.);
        document.verify_roundtrip().unwrap();
        document.undo();
        assert_eq!(document.encode().unwrap(), expected);
        document.selected.insert(6);
        let clip = document.copy_selected().unwrap();
        let pasted = document.paste(&clip).unwrap();
        let copy = *pasted.last().unwrap();
        assert!(
            document.position(copy).unwrap().distance(Point {
                x: probe.x + 20.,
                y: probe.y + 20.
            }) < 1e-10
        );
        document.verify_roundtrip().unwrap();
        document.selected.clear();
        document.selected.insert(4);
        document.clear_selected().unwrap();
        assert!(document.objects[4].deleted && document.objects[5].deleted);
        document.verify_roundtrip().unwrap();
    }
}
