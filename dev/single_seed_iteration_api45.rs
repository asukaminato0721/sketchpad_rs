//! Fresh public API controls for scalar single mappings and legacy canonical forms.
use sketchpad_rs::document::{Document, IterationDepth, Kind, Point};
fn main() {
    for state in [
        "scalar",
        "calculated_scalar",
        "direct_point",
        "direct_segment",
        "branch_polygon",
    ] {
        let mut d = Document::new();
        let (seeds, branches, carried, bindings) =
            if state == "scalar" || state == "calculated_scalar" {
                let a = if state == "scalar" {
                    d.add_named_parameter("a", 1., (10, 30)).unwrap()
                } else {
                    d.add_calculation_expression("1", (10, 30)).unwrap()
                };
                let b = d.add_named_parameter("b", 1., (10, 60)).unwrap();
                let c = d.add_multi_calculation(&[a, b], "v1+v2", (10, 90)).unwrap();
                let f = d
                    .add_default_coordinate_system(Point { x: 160., y: 360. })
                    .unwrap();
                let p = d.add(Kind::NumericCoordinatePoint(a, b, f));
                let q = d.add(Kind::NumericCoordinatePoint(b, c, f));
                let s = d.add(Kind::Segment(p, q));
                (vec![a], vec![vec![c]], vec![c, p, q, s], vec![p, s])
            } else {
                let a = d.add(Kind::Free(Point { x: 140., y: 250. }));
                let u = d.add(Kind::Free(Point { x: 80., y: 80. }));
                let v = d.add(Kind::Free(Point { x: 120., y: 80. }));
                if state == "branch_polygon" {
                    let n = d.add(Kind::PolarTranslatedPoint(a, 0., 40., 1.));
                    let q = d.add(Kind::PolarTranslatedPoint(a, 90., 40., 1.));
                    let p = d.add(Kind::Polygon(vec![a, n, q]));
                    (vec![a], vec![vec![n], vec![q]], vec![q, n, p], vec![n])
                } else {
                    let n = d.add(Kind::VectorTranslatedPoint(a, u, v));
                    if state == "direct_point" {
                        (vec![a], vec![vec![n]], vec![n], vec![n])
                    } else {
                        let s = d.add(Kind::Segment(a, n));
                        (vec![a], vec![vec![n]], vec![n, s], vec![s])
                    }
                }
            };
        let it = d
            .add_mapped_point_iteration_with_bindings(
                &seeds,
                &branches,
                &carried,
                &bindings,
                IterationDepth::Fixed(3),
            )
            .unwrap();
        d.verify_roundtrip().unwrap();
        let bytes = d.encode().unwrap();
        std::fs::write(format!("/tmp/single_seed45_api_{state}.gsp"), &bytes).unwrap();
        std::fs::write(
            format!("/tmp/single_seed45_api_expected_{state}.gsp"),
            bytes,
        )
        .unwrap();
        println!("{state}: iteration{it} {:?}", d.objects[it - 1].kind);
    }
}
