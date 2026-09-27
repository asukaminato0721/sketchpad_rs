use super::*;
use crate::document::transform_model::{
    DisplacementExpr, NumericExpr, Transform, TransformDefinition,
};
fn close(a: Point, b: Point) {
    assert!(
        (a.x - b.x).abs() < 1e-8 && (a.y - b.y).abs() < 1e-8,
        "{a:?} != {b:?}"
    );
}
fn shifted(p: Point) -> Point {
    Point {
        x: p.x + 40.,
        y: p.y + 20.,
    }
}
fn reflected(p: Point, a: Point, b: Point) -> Point {
    let x = b.x - a.x;
    let y = b.y - a.y;
    let t = ((p.x - a.x) * x + (p.y - a.y) * y) / (x * x + y * y);
    Point {
        x: 2. * (a.x + t * x) - p.x,
        y: 2. * (a.y + t * y) - p.y,
    }
}
fn rotated(p: Point, c: Point) -> Point {
    let (s, k) = (-25f64).to_radians().sin_cos();
    let x = p.x - c.x;
    let y = p.y - c.y;
    Point {
        x: c.x + x * k - y * s,
        y: c.y + x * s + y * k,
    }
}
fn verify_chain(d: &Document, source: usize, targets: [usize; 4]) {
    let (mut a, mut b, mode) = d.line_points(source).unwrap();
    let (m, n, _) = d.line_points(11).unwrap();
    let c = d.position(4).unwrap();
    for (stage, target) in targets.into_iter().enumerate() {
        let f = |p| match stage {
            0 => shifted(p),
            1 => reflected(p, m, n),
            2 => {
                let r = 30f64.to_radians();
                Point {
                    x: p.x + r.cos() * 37.79527559055118,
                    y: p.y - r.sin() * 37.79527559055118,
                }
            }
            _ => rotated(p, c),
        };
        a = f(a);
        b = f(b);
        let (x, y, actual) = d.line_points(target).unwrap();
        close(x, a);
        close(y, b);
        assert_eq!(actual, mode);
    }
}
#[test]
fn native_straight_transform_chains_preserve_ordered_frames_and_exact_bytes() {
    for (state, source) in [
        ("ray", 6),
        ("line", 7),
        ("parallel", 8),
        ("perpendicular", 9),
        ("bisector", 10),
        ("ray_edited", 6),
        ("perpendicular_edited", 9),
    ] {
        let bytes = fs::read(format!(
            "tests/fixtures/straight_transform54_{state}_native_saved.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert!(!d.partial_view, "{state}");
        assert_eq!(d.encode().unwrap(), bytes);
        verify_chain(&d, source, [14, 15, 16, 17]);
        for (id, flavor) in [
            (14, NativeTransformFlavor::Vector),
            (15, NativeTransformFlavor::Reflection),
            (16, NativeTransformFlavor::FixedPolar),
            (17, NativeTransformFlavor::FixedRotation),
        ] {
            assert!(transform_is(
                &d.objects[id - 1].kind,
                flavor,
                &[ElementType::Ray, ElementType::Line]
            ));
        }
        for o in &d.objects {
            if matches!(o.kind, Kind::PointDistance(..)) {
                assert!(d.numeric_value(o.id).unwrap().abs() < 1e-9);
            }
        }
    }
}
fn authored(ray: bool) -> Document {
    let mut d = Document::new();
    for p in [(150., 250.), (250., 200.), (180., 150.), (200., 200.)] {
        d.add(Kind::Free(Point { x: p.0, y: p.1 }));
    }
    d.add(Kind::Segment(1, 2));
    d.add(Kind::Ray(1, 2));
    d.add(Kind::Line(1, 2));
    d.add(Kind::Parallel(3, 5));
    d.add(Kind::Perpendicular(3, 5));
    d.add(Kind::Bisector(1, 4, 2));
    d.add(Kind::Line(3, 4));
    d.add(Kind::Free(Point { x: 100., y: 100. }));
    d.add(Kind::Free(Point { x: 140., y: 120. }));
    let e = if ray {
        ElementType::Ray
    } else {
        ElementType::Line
    };
    let source = if ray { 6 } else { 9 };
    let literal = |magnitude, unit, scale| NumericExpr::Literal {
        magnitude,
        unit,
        scale,
    };
    for (source, t, f) in [
        (
            source,
            Transform::Translate(DisplacementExpr::Vector { start: 12, end: 13 }),
            NativeTransformFlavor::Vector,
        ),
        (
            14,
            Transform::Reflect { mirror: 11 },
            NativeTransformFlavor::Reflection,
        ),
        (
            15,
            Transform::Translate(DisplacementExpr::Polar {
                angle: literal(30., NumericUnit::Degrees, 1.),
                distance: literal(1., NumericUnit::Length, 37.79527559055118),
            }),
            NativeTransformFlavor::FixedPolar,
        ),
        (
            16,
            Transform::Rotate {
                center: 4,
                angle: literal(25., NumericUnit::Degrees, 1.),
            },
            NativeTransformFlavor::FixedRotation,
        ),
    ] {
        d.add(Kind::Transformed(
            TransformDefinition::new(source, e, t, f).unwrap(),
        ));
    }
    d
}
#[test]
fn public_generic_straight_chains_edit_undo_and_keep_constraint_domains() {
    for ray in [false, true] {
        let mut d = authored(ray);
        let source = if ray { 6 } else { 9 };
        verify_chain(&d, source, [14, 15, 16, 17]);
        let (a, b, m) = d.line_points(17).unwrap();
        let behind = Point {
            x: 2. * a.x - b.x,
            y: 2. * a.y - b.y,
        };
        let kind = d.constrained_point(17, behind).unwrap();
        let p = d.add(kind);
        assert!(
            matches!(d.objects[p-1].kind,Kind::PointOnRay(_,_) if ray)
                || matches!(d.objects[p-1].kind,Kind::PointOnLine(_,_) if !ray)
        );
        close(
            d.position(p).unwrap(),
            if m == LineMode::Ray { a } else { behind },
        );
        d.checkpoint();
        assert!(d.move_point(1, Point { x: 120., y: 270. }));
        verify_chain(&d, source, [14, 15, 16, 17]);
        d.undo();
        close(d.position(1).unwrap(), Point { x: 150., y: 250. });
        d.redo();
        verify_chain(&d, source, [14, 15, 16, 17]);
        d.verify_roundtrip().unwrap();
        let r = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
        assert!(!r.partial_view);
        verify_chain(&r, source, [14, 15, 16, 17]);
    }
}
#[test]
fn degenerate_live_straight_source_suppresses_and_restores_entire_chain() {
    let mut d = authored(true);
    d.checkpoint();
    let p = d.position(2).unwrap();
    assert!(d.move_point(1, p));
    let (a, b, _) = d.line_points(6).unwrap();
    close(a, b);
    for id in [14, 15, 16, 17] {
        assert!(d.line_points(id).is_none(), "{id}");
    }
    d.verify_roundtrip().unwrap();
    d.undo();
    verify_chain(&d, 6, [14, 15, 16, 17]);
}
#[test]
fn malformed_straight_transform_identity_and_literals_stay_opaque() {
    let bytes = fs::read("tests/fixtures/straight_transform54_ray_input.gsp").unwrap();
    let d = Document::from_bytes(&bytes, None).unwrap();
    let rs = records(&bytes).unwrap();
    let h = d.objects[13].header_offset.unwrap();
    let i = rs
        .iter()
        .position(|r| r.kind == 0x7d0 && r.data.start == h)
        .unwrap();
    let refs = rs[i..].iter().find(|r| r.kind == 0x7d2).unwrap();
    let reflected = d.objects[14].header_offset.unwrap();
    let ri = rs.iter().position(|r| r.kind == 0x7d0 && r.data.start == reflected).unwrap();
    let reflected_refs = rs[ri..].iter().find(|r| r.kind == 0x7d2).unwrap();
    let polar = d.objects[15].header_offset.unwrap();
    let pi = rs
        .iter()
        .position(|r| r.kind == 0x7d0 && r.data.start == polar)
        .unwrap();
    let payload = rs[pi..].iter().find(|r| r.kind == 0x7d3).unwrap();
    for (at, value, badid) in [
        (h + 8, 12u16.to_le_bytes().to_vec(), 14),
        (h + 8, 44u16.to_le_bytes().to_vec(), 14),
        (h + 6, 1u16.to_le_bytes().to_vec(), 14),
        (refs.data.start + 4, 5u32.to_le_bytes().to_vec(), 14),
        (reflected_refs.data.start + 4, 1u32.to_le_bytes().to_vec(), 15),
        (payload.data.start + 20, f64::NAN.to_le_bytes().to_vec(), 16),
    ] {
        let mut b = bytes.clone();
        b[at..at + value.len()].copy_from_slice(&value);
        let invalid = Document::from_bytes(&b, None).unwrap();
        assert!(matches!(
            invalid.objects[badid - 1].kind,
            Kind::Unsupported(_)
        ));
        assert_eq!(invalid.encode().unwrap(), b);
    }
}
