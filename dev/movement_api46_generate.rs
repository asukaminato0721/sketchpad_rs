//! Checkpoint 46 current public API Native cross-open controls.
use sketchpad_rs::{
    action_records::MovementSpeed,
    document::{Document, Kind, MovementValue, Point},
    graph::GridForm,
};
fn main() {
    let state = std::env::args().nth(1).unwrap();
    if state.contains("imported") {
        let fixture = if state.contains("rect") {
            "api_graph_rect"
        } else {
            "graph_x"
        };
        let path = format!("tests/fixtures/movement46_{fixture}_input.gsp");
        let mut d = Document::open(std::path::Path::new(&path)).unwrap();
        assert!(d.set_movement_value(2, MovementValue::Point(Point { x: 275., y: 200. })));
        if state.contains("rect") {
            assert!(d.set_movement_value(4, MovementValue::Point(Point { x: 350., y: 225. })));
        }
        d.verify_roundtrip().unwrap();
        std::fs::write(
            format!("/tmp/movement46_{state}_input.gsp"),
            d.encode().unwrap(),
        )
        .unwrap();
        println!("{state} imported writer");
        return;
    }
    let mut d = Document::new();
    let pairs = match state.as_str() {
        "api_parameter" | "api_parameter_reverse" => {
            let a = d.add_named_parameter("a", 1., (150, 160)).unwrap();
            let b = d.add_named_parameter("b", 3.25, (150, 200)).unwrap();
            let c = d.add_named_parameter("c", 7.125, (150, 240)).unwrap();
            if state.ends_with("reverse") {
                vec![(b, c), (a, b)]
            } else {
                vec![(a, b), (b, c)]
            }
        }
        "api_graph_rect"
        | "api_graph_rect_edited"
        | "api_graph_x_edited"
        | "api_graph_square_y_probe_edited" => {
            d.add_default_coordinate_system(Point { x: 200., y: 300. })
                .unwrap();
            if !state.contains("graph_x") && !state.contains("square_y") {
                d.set_grid_form(GridForm::Rectangular).unwrap();
            }
            d.set_grid_visible(false);
            let a = d.add(Kind::Free(Point { x: 275., y: 200. }));
            let b = d.add(Kind::Free(Point { x: 350., y: 225. }));
            if state.contains("graph_x") || state.contains("square_y") {
                vec![(2, a)]
            } else {
                vec![(2, a), (4, b)]
            }
        }
        "api_path_multi" => {
            let a = d.add(Kind::Free(Point { x: 120., y: 200. }));
            let b = d.add(Kind::Free(Point { x: 320., y: 200. }));
            let s = d.add(Kind::Segment(a, b));
            let p = d.add(Kind::PointOnSegment(s, 0.25));
            let c = d.add(Kind::Circle(a, b));
            let q = d.add(Kind::PointOnCircle(c, 0., 1.));
            let dest = d.add(Kind::Free(Point { x: 280., y: 300. }));
            vec![(p, dest), (q, dest)]
        }
        _ => {
            let a = d.add(Kind::Free(Point { x: 140., y: 200. }));
            let b = d.add(Kind::Free(Point { x: 240., y: 240. }));
            let c = d.add(Kind::Free(Point { x: 340., y: 300. }));
            vec![(a, b), (b, c)]
        }
    };
    d.add_movement_button_for_pairs(&pairs, MovementSpeed::Instant, true)
        .unwrap();
    if state.ends_with("edited") {
        assert!(d.set_movement_value(2, MovementValue::Point(Point { x: 275., y: 200. })));
        if state == "api_graph_rect_edited" {
            assert!(d.set_movement_value(4, MovementValue::Point(Point { x: 350., y: 225. })));
        }
    }
    if state.contains("square_y") {
        let target = d.add(Kind::Free(Point { x: 300., y: 300. }));
        let probe = d.add(Kind::VectorTranslatedPoint(4, 1, target));
        d.set_point_label(probe, "Y", true).unwrap();
    }
    let bytes = d.encode().unwrap();
    d.verify_roundtrip().unwrap();
    std::fs::write(format!("/tmp/movement46_{state}_input.gsp"), bytes).unwrap();
    println!("{state} {:?}", pairs);
}
