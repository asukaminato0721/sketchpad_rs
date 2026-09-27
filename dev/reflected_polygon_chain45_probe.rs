//! Read-only analytic reflection chain check against original corpus payload.
use sketchpad_rs::document::{Document, Point};
fn main() {
    let b = std::fs::read("tests/fixtures/reflected_polygon_chain45_corpus_native_verified.gsp")
        .unwrap();
    let d = Document::from_bytes(&b, None).unwrap();
    assert_eq!(d.encode().unwrap(), b);
    let anchors = std::env::args().any(|a| a == "anchors");
    if anchors || std::env::args().any(|a| a == "isolate") {
        let mut isolated = b.clone();
        for object in &d.objects {
            if let Some(at) = object.header_offset {
                isolated[at + 2] = if object.id == 62 || (anchors && [1, 2].contains(&object.id)) {
                    isolated[at + 2] & !1
                } else {
                    isolated[at + 2] | 1
                };
            }
        }
        std::fs::write(
            if anchors {
                "tests/fixtures/reflected_polygon_chain45_anchors_native_verified.gsp"
            } else {
                "tests/fixtures/reflected_polygon_chain45_isolated_native_verified.gsp"
            },
            &isolated,
        )
        .unwrap();
        std::fs::write(
            if anchors {
                "/tmp/reflected_polygon_chain45_anchors_expected.gsp"
            } else {
                "/tmp/reflected_polygon_chain45_isolated_expected.gsp"
            },
            isolated,
        )
        .unwrap();
    }
    let (a, z, _) = d.line_points(5).unwrap();
    let dx = z.x - a.x;
    let dy = z.y - a.y;
    println!(
        "anchor1 {:?} anchor2 {:?}",
        d.position(1).unwrap(),
        d.position(2).unwrap()
    );
    println!("mirror5 {a:?} {z:?}");
    for source in 54..=60 {
        for p in d.polygon_points(source).unwrap() {
            let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / (dx * dx + dy * dy);
            let q = Point {
                x: 2. * (a.x + t * dx) - p.x,
                y: 2. * (a.y + t * dy) - p.y,
            };
            println!("source{source} {p:?} reflected{} {q:?}", source + 7);
        }
    }
}
