//! Native point label ordering controls for literal and measured rotation.
use sketchpad_rs::document::{Document, Kind, Point};
fn main() {
    let prefix = std::env::args().nth(1).unwrap();
    for numeric in [false, true] {
        let mut d = Document::from_bytes(
            include_bytes!("../tests/fixtures/numeric_typed_parameters_native_verified.gsp"),
            None,
        )
        .unwrap();
        d.set_parameter_value(1, 60.).unwrap();
        let a = d.add(Kind::Free(Point { x: 240., y: 250. }));
        let o = d.add(Kind::Free(Point { x: 140., y: 250. }));
        let p = if numeric {
            d.add_calculated_angle_rotation(a, o, 1).unwrap()
        } else {
            d.add(Kind::RotatedPoint(a, o, 90.))
        };
        d.set_point_label(p, "T", true).unwrap();
        d.verify_roundtrip().unwrap();
        let mode = if numeric { "numeric" } else { "fixed" };
        let path = format!("{prefix}_{mode}.gsp");
        let b = d.encode().unwrap();
        std::fs::write(&path, &b).unwrap();
        println!("{path}: {} bytes point={:?}", b.len(), d.position(p));
    }
}
