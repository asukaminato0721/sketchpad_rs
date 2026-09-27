//! Native category-caption probes with ordinary API-authored target geometry.
use sketchpad_rs::document::{Document, Kind, Point};
fn main() {
    for category in ["circle","arc","polygon","ray","line","named_point"] {
        let mut d=Document::new();
        let a=d.add(Kind::Free(Point{x:140.,y:200.}));
        let b=d.add(Kind::Free(Point{x:240.,y:200.}));
        let c=d.add(Kind::Free(Point{x:140.,y:300.}));
        let target= match category {
            "circle"=> d.add(Kind::Circle(a,b)),
            "arc"=>d.add(Kind::CenterArc(a,b,c)),
            "polygon"=>d.add(Kind::Polygon(vec![a,b,c])),
            "ray"=>d.add(Kind::Ray(a,b)),
            "line"=>d.add(Kind::Line(a,b)),
            _=>{d.set_point_label(a,"T",true).unwrap();a},
        };
        d.verify_roundtrip().unwrap();
        std::fs::write(format!("/tmp/visibility48_category_{category}_base.gsp"),d.encode().unwrap()).unwrap();
        println!("{category} {target}");
    }
}
