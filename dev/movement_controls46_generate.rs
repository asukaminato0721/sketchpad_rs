//! Native movement input controls. Button bytes are added by movement_controls46_wire.py.
use sketchpad_rs::document::{Document, Kind, Point};
fn main() {
    let state = std::env::args().nth(1).unwrap();
    let mut d = Document::new();
    let pairs: Vec<(usize, usize)> = match state.as_str() {
        "parameter" | "parameter_length" | "parameter_mismatch" => {
            let a = d.add_named_parameter("a", 1., (150, 160)).unwrap();
            let b = d.add_named_parameter("b", 3.25, (150, 200)).unwrap();
            let c = d
                .add_calculation_expression(
                    if state == "parameter" {
                        "7.125"
                    } else {
                        "7.125 cm"
                    },
                    (150, 240),
                )
                .unwrap();
            vec![(a, b), (b, c)]
        }
        "graph" => {
            d.add_default_coordinate_system(Point { x: 200., y: 300. })
                .unwrap();
            d.set_grid_visible(false);
            let a = d.add(Kind::Free(Point { x: 275., y: 200. }));
            let b = d.add(Kind::Free(Point { x: 350., y: 225. }));
            vec![(2, a), (4, b)]
        }
        "overlap" | "overlap_slow" => {
            let a = d.add(Kind::Free(Point { x: 140., y: 200. }));
            let b = d.add(Kind::Free(Point { x: 240., y: 240. }));
            let c = d.add(Kind::Free(Point { x: 340., y: 300. }));
            vec![(a, b), (b, c)]
        }
        "arc" | "polygon" => {
            let a = d.add(Kind::Free(Point { x: 120., y: 200. }));
            let b = d.add(Kind::Free(Point { x: 320., y: 200. }));
            let c = d.add(Kind::Free(Point { x: 120., y: 300. }));
            let h = d.add(if state == "arc" {
                Kind::CenterArc(a, b, c)
            } else {
                Kind::Polygon(vec![a, b, c])
            });
            let p = d.add(if state == "arc" {
                Kind::PointOnArc(h, 0.25)
            } else {
                Kind::PointOnPolygon(h, 0, 0.25)
            });
            let q = d.add(Kind::Free(Point { x: 280., y: 300. }));
            vec![(p, q)]
        }
        _ => {
            let a = d.add(Kind::Free(Point { x: 120., y: 200. }));
            let b = d.add(Kind::Free(Point { x: 320., y: 200. }));
            let h = d.add(match state.as_str() {
                "circle" => Kind::Circle(a, b),
                "ray" => Kind::Ray(a, b),
                "line" => Kind::Line(a, b),
                _ => Kind::Segment(a, b),
            });
            let p = d.add(match state.as_str() {
                "circle" => Kind::PointOnCircle(h, 0., 1.),
                "ray" => Kind::PointOnRay(h, 0.25),
                "line" => Kind::PointOnLine(h, 0.25),
                _ => Kind::PointOnSegment(h, 0.25),
            });
            let q = d.add(Kind::Free(Point { x: 280., y: 300. }));
            vec![(p, q)]
        }
    };
    let bytes = d.encode().unwrap();
    std::fs::write(format!("/tmp/movement46_{state}_base.gsp"), bytes).unwrap();
    println!("{state} {} {:?}", d.objects.len(), pairs);
    for o in &d.objects {
        println!("{} {:?}", o.id, o.kind);
    }
}
