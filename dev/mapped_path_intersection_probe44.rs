//! Actual Native controls for shared path/intersection mapped-iteration DAGs.
//! cargo build --lib
//! rustc --edition=2024 -L dependency=target/debug/deps --extern sketchpad_rs=target/debug/libsketchpad_rs.rlib dev/mapped_path_intersection_probe44.rs -o /tmp/mapped_path_intersection_probe44
//! /tmp/mapped_path_intersection_probe44 /tmp/mapped_path44
use sketchpad_rs::document::{Document, IterationDepth, Kind, Point};
fn main() {
    let prefix = std::env::args().nth(1).expect("output prefix");
    for name in [
        "path_segment",
        "path_circle",
        "line_line",
        "line_circle",
        "circle_circle",
        "line_circle_undefined",
    ] {
        let mut d = Document::new();
        let a = d.add(Kind::Free(Point { x: 140., y: 250. }));
        let b = d.add(Kind::Free(Point { x: 240., y: 250. }));
        let u = d.add(Kind::Free(Point { x: 80., y: 80. }));
        let v = d.add(Kind::Free(Point { x: 120., y: 80. }));
        let w = d.add(Kind::Free(Point { x: 80., y: 120. }));
        let s = d.add(Kind::Segment(a, b));
        let m = d.add(Kind::Midpoint(a, b));
        let r = d.add(Kind::VectorTranslatedPoint(b, u, v));
        let n = match name {
            "path_segment" => d.add(Kind::PointOnSegment(s, 0.25)),
            "path_circle" => {
                let c = d.add(Kind::Circle(m, a));
                d.add(Kind::PointOnCircle(c, 0., 1.))
            }
            "line_line" => {
                let ar = d.add(Kind::VectorTranslatedPoint(a, u, v));
                let mr = d.add(Kind::VectorTranslatedPoint(m, u, w));
                let l = d.add(Kind::Line(a, ar));
                let k = d.add(Kind::Line(m, mr));
                d.add(Kind::Intersection(l, k))
            }
            "line_circle" => {
                let c = d.add(Kind::Circle(m, a));
                let mr = d.add(Kind::VectorTranslatedPoint(m, u, w));
                let l = d.add(Kind::Line(m, mr));
                d.add(Kind::LineCircleIntersection(l, c, 0))
            }
            "circle_circle" => {
                let c = d.add(Kind::Circle(a, b));
                let k = d.add(Kind::Circle(b, a));
                d.add(Kind::CircleCircleIntersection(c, k, 0))
            }
            "line_circle_undefined" => {
                // Radius shrinks because A moves halfway toward fixed B;
                // fixed horizontal line y=280 intersects at first but eventually
                // lies beyond the current radius. One valid generated prefix.
                let c = d.add(Kind::Circle(a, b));
                let p = d.add(Kind::Free(Point { x: 100., y: 280. }));
                let q = d.add(Kind::Free(Point { x: 400., y: 280. }));
                let l = d.add(Kind::Line(p, q));
                d.add(Kind::LineCircleIntersection(l, c, 0))
            }
            _ => unreachable!(),
        };
        for id in [a, b, n, r] {
            println!("{name} point {id}: {:?}", d.position(id));
        }
        if std::env::var_os("SKIP_LABELS").is_none() {
            if std::env::var("LABEL_ONLY").map_or(true, |x| x == "A") {
                d.set_point_label(a, "A", true).unwrap();
            }
            if std::env::var("LABEL_ONLY").map_or(true, |x| x == "B") {
                d.set_point_label(b, "B", true).unwrap();
            }
            if std::env::var("LABEL_ONLY").map_or(true, |x| x == "N") {
                d.set_point_label(n, "N", true).unwrap();
            }
            if std::env::var("LABEL_ONLY").map_or(true, |x| x == "R") {
                d.set_point_label(r, "R", true).unwrap();
            }
        }
        d.selected
            .extend((1..=n).filter(|id| ![a, b, s, n, r].contains(id)));
        d.set_selected_visibility(false);
        d.selected.clear();
        std::fs::write(format!("{prefix}_{name}_base.gsp"), d.encode().unwrap()).unwrap();
        let carried = if std::env::var_os("OMIT_PREREQUISITES").is_some() {
            vec![s, n, r]
        } else {
            (3..=n).collect::<Vec<_>>()
        };
        let mut api_document = d.clone();
        let parent = d.add(Kind::MappedPointIteration {
            seeds: vec![a, b],
            branches: vec![if name == "line_circle_undefined" {
                vec![m, b]
            } else {
                vec![n, r]
            }],
            carried: carried.clone(),
            depth: IterationDepth::Fixed(3),
            final_only: false,
        });
        let binding = d.add(Kind::IterationBinding(s, parent));
        let point_binding = if name == "line_circle_undefined" {
            Some(d.add(Kind::IterationBinding(n, parent)))
        } else {
            None
        };
        let bytes = d.encode().unwrap();
        if name != "line_circle_undefined" {
            api_document
                .add_mapped_point_iteration_with_bindings(
                    &[a, b],
                    &[vec![n, r]],
                    &carried,
                    &[s],
                    IterationDepth::Fixed(3),
                )
                .unwrap();
            api_document.verify_roundtrip().unwrap();
            assert_eq!(api_document.encode().unwrap(), bytes, "public API {name}");
            println!("Public API exact {name}");
        }
        let path = format!("{prefix}_{name}.gsp");
        std::fs::write(&path, &bytes).unwrap();
        println!(
            "{path}: {} bytes; definition={parent}, binding={binding}, target={n},{r}, point_binding={point_binding:?}",
            bytes.len()
        );
    }
}
