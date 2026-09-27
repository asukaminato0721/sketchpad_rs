//! Native original typed-Length CircleBySegment source and generic bindings.
use sketchpad_rs::document::{Document, IterationDepth, Kind, Point};
fn main() {
    let state = std::env::args().nth(1).unwrap();
    let original = Document::open(std::path::Path::new(
        "tests/fixtures/iteration_binding47_original_circle_segment_native_verified.gsp",
    ))
    .unwrap();
    if state == "original_source" {
        let mut d = original;
        for o in &mut d.objects {
            o.visible = false;
        }
        d.objects[4].visible = true;
        assert!(d.move_point(2, Point { x: 160., y: 180. }));
        d.verify_roundtrip().unwrap();
        std::fs::write(
            "/tmp/iteration_binding47_numeric_radius_original_source_input.gsp",
            d.encode().unwrap(),
        )
        .unwrap();
        return;
    }
    let mut d = Document::new();
    let center = d.add(Kind::Free(Point { x: 100., y: 180. }));
    let parameter = d.add(original.objects[2].kind.clone());
    d.set_parameter_value(
        parameter,
        if state.contains("two") {
            2.
        } else if state.contains("negative") {
            -1.
        } else if state.contains("zero") {
            0.
        } else {
            1.
        },
    )
    .unwrap();
    let circle = d.add(Kind::CircleBySegment(center, parameter));
    let p = if state.contains("negative") || state.contains("zero") {
        None
    } else {
        Some(d.add(Kind::PointOnCircle(circle, 1., 0.)))
    };
    if state.starts_with("base") {
        d.objects[parameter - 1].visible = false;
        d.verify_roundtrip().unwrap();
        std::fs::write(
            format!("/tmp/iteration_binding47_numeric_radius_{state}_input.gsp"),
            d.encode().unwrap(),
        )
        .unwrap();
        println!("{state} visiblecenter{center}, length{parameter}, circle{circle}, point{p:?}");
        return;
    }
    let image = if state.starts_with("vector") {
        let start = d.add(Kind::Free(Point { x: 80., y: 80. }));
        let end = d.add(Kind::Free(Point { x: 100., y: 80. }));
        d.add(Kind::VectorTranslatedPoint(center, start, end))
    } else {
        d.add(Kind::PolarTranslatedPoint(center, 0., 1., 20.))
    };
    let root = d
        .add_mapped_point_iteration_with_bindings(
            &[center],
            &[vec![image]],
            &[circle, image],
            &[circle],
            IterationDepth::Fixed(3),
        )
        .unwrap();
    for o in d.objects.iter_mut().take(root - 1) {
        o.visible = false;
    }
    d.verify_roundtrip().unwrap();
    std::fs::write(
        format!("/tmp/iteration_binding47_numeric_radius_{state}_input.gsp"),
        d.encode().unwrap(),
    )
    .unwrap();
    println!(
        "{state} center{center} length{parameter} circle{circle} point{p:?} image{image} root{root}"
    );
}
