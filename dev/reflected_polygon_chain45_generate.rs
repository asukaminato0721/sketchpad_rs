//! Public API authoring and source/mirror edits of a compound polygon reflection.
//! cargo build --lib; rustc --edition=2024 -L dependency=target/debug/deps --extern sketchpad_rs=target/debug/libsketchpad_rs.rlib dev/reflected_polygon_chain45_generate.rs -o /tmp/reflected_polygon_chain45_generate
use sketchpad_rs::document::{Document, Kind, Point};
fn reflect(p: Point, a: Point, b: Point) -> Point {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / (dx * dx + dy * dy);
    Point {
        x: 2. * (a.x + t * dx) - p.x,
        y: 2. * (a.y + t * dy) - p.y,
    }
}
fn main() {
    let prefix = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "tests/fixtures/reflected_polygon_chain45_api".into());
    let mut original = Document::new();
    for p in [
        (100., 160.),
        (180., 140.),
        (140., 220.),
        (240., 80.),
        (240., 280.),
        (80., 320.),
        (460., 320.),
    ] {
        original.add(Kind::Free(Point { x: p.0, y: p.1 }));
    }
    let first_mirror = original.add(Kind::Segment(4, 5));
    let second_mirror = original.add(Kind::Segment(6, 7));
    let source = original.add(Kind::Polygon(vec![1, 2, 3]));
    let first = original.add(Kind::ReflectedPolygon(source, first_mirror));
    let last = original.add(Kind::ReflectedPolygon(first, second_mirror));
    for o in &mut original.objects {
        o.visible = [first_mirror, second_mirror, last].contains(&o.id)
    }
    for state in ["original", "source_edited", "mirror_edited"] {
        let mut doc = original.clone();
        if state == "source_edited" {
            assert!(doc.move_point(1, Point { x: 110., y: 190. }));
        }
        if state == "mirror_edited" {
            assert!(doc.move_point(7, Point { x: 460., y: 360. }));
        }
        let expected: Vec<_> = doc
            .polygon_points(source)
            .unwrap()
            .into_iter()
            .map(|p| {
                reflect(
                    reflect(p, doc.position(4).unwrap(), doc.position(5).unwrap()),
                    doc.position(6).unwrap(),
                    doc.position(7).unwrap(),
                )
            })
            .collect();
        let actual = doc.polygon_points(last).unwrap();
        for (a, b) in actual.iter().zip(&expected) {
            assert!((a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9)
        }
        doc.verify_roundtrip().unwrap();
        let bytes = doc.encode().unwrap();
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.encode().unwrap(), bytes);
        assert_eq!(reopened.polygon_points(last), Some(actual.clone()));
        let path = format!("{prefix}_{state}_native_verified.gsp");
        std::fs::write(&path, &bytes).unwrap();
        std::fs::write(
            format!("/tmp/reflected_polygon_chain45_api_{state}_expected.gsp"),
            &bytes,
        )
        .unwrap();
        println!(
            "{state} bytes={} IDs mirrors={first_mirror},{second_mirror} source={source} first={first} last={last} expected={expected:?} path={path}",
            bytes.len()
        );
    }
}
