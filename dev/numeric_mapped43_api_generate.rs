//! Public API scalar and mixed mapped iteration Native controls.
//! cargo build --lib
//! rustc --edition=2024 -L dependency=target/debug/deps --extern sketchpad_rs=target/debug/libsketchpad_rs.rlib dev/numeric_mapped43_api_generate.rs -o /tmp/numeric_mapped43_api_generate
//! /tmp/numeric_mapped43_api_generate fibonacci|calculated|live|edited|mixed
use sketchpad_rs::document::{Document, IterationDepth, Kind, Point};
fn main() {
    let state = std::env::args().nth(1).unwrap_or("fibonacci".into());
    let mut d = Document::new();
    let a = if state == "calculated" {
        d.add_calculation_expression("1", (10, 30)).unwrap()
    } else if state == "mixed" {
        d.add(Kind::Free(Point { x: 100., y: 300. }))
    } else {
        d.add_named_parameter("a", 1., (10, 30)).unwrap()
    };
    let b = if state == "calculated" {
        d.add_calculation_expression("1", (10, 60)).unwrap()
    } else {
        d.add_named_parameter("b", 1., (10, 60)).unwrap()
    };
    let c = if state == "mixed" {
        d.add_parameter_calculation(b, "b+1", (10, 90)).unwrap()
    } else {
        d.add_multi_calculation(&[a, b], "v1+v2", (10, 90)).unwrap()
    };
    let f = d
        .add_default_coordinate_system(Point { x: 160., y: 360. })
        .unwrap();
    let p = if state == "mixed" {
        a
    } else {
        d.add(Kind::NumericCoordinatePoint(a, b, f))
    };
    let q = if state == "mixed" {
        d.add(Kind::NumericCoordinatePoint(b, b, f))
    } else {
        d.add(Kind::NumericCoordinatePoint(b, c, f))
    };
    let s = d.add(Kind::Segment(p, q));
    let n = if state == "live" || state == "edited" {
        Some(d.add_named_parameter("n", 3., (10, 120)).unwrap())
    } else {
        None
    };
    let branch = if state == "mixed" {
        vec![q, c]
    } else {
        vec![b, c]
    };
    let it = d
        .add_mapped_point_iteration_with_bindings(
            &[a, b],
            &[branch],
            &[c, p, q, s],
            &[p, s],
            n.map(IterationDepth::Value)
                .unwrap_or(IterationDepth::Fixed(3)),
        )
        .unwrap();
    if state == "edited" {
        d.set_parameter_value(a, 2.).unwrap();
        d.set_parameter_value(b, 1.).unwrap();
        d.set_parameter_value(n.unwrap(), 2.).unwrap();
    }
    d.verify_roundtrip().unwrap();
    std::fs::write(
        format!("/tmp/numeric_mapped43_api_{state}.gsp"),
        d.encode().unwrap(),
    )
    .unwrap();
    println!("{state}: a {a} b {b} c {c} frame {f} p {p} q {q} segment {s} iteration {it}");
    println!("points {:?}", d.mapped_point_binding_images(it + 1));
    println!("segments {:?}", d.branch_segment_iteration_images(it + 2));
}
