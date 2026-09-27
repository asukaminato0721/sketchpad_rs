//! Current generic mapped geometry binding APIs; no raw packet bypass.
use sketchpad_rs::document::{Document, IterationDepth, Kind, Point};
fn derived_document(state: &str) -> (Document, usize) {
    let mut d = Document::new();
    let l = d.add(Kind::Free(Point {
        x: if state.contains("edited") { 100. } else { 80. },
        y: 180.,
    }));
    let r = d.add(Kind::Free(Point { x: 120., y: 180. }));
    let b = d.add(Kind::Free(Point { x: 160., y: 180. }));
    let c = d.add(Kind::Free(Point { x: 100., y: 240. }));
    let host = d.add(Kind::Segment(l, r));
    let a = d.add(Kind::SegmentMidpoint(host));
    let origin = d.add(Kind::Free(Point { x: 80., y: 80. }));
    let target = d.add(Kind::Free(Point { x: 200., y: 100. }));
    let mut images = vec![];
    for seed in [a, b, c] {
        let scaled = d.add(Kind::ScaledPoint(seed, origin, 0.75));
        images.push(d.add(Kind::VectorTranslatedPoint(scaled, origin, target)));
    }
    let segment = d.add(Kind::Segment(a, b));
    let circle = d.add(Kind::Circle(a, b));
    let circle_segment = d.add(Kind::CircleBySegment(a, segment));
    let polygon = d.add(Kind::Polygon(vec![a, b, c]));
    let ray = d.add(Kind::Ray(a, b));
    let line = d.add(Kind::Line(a, c));
    let arc = d.add(Kind::CenterArc(a, b, c));
    let bindings = if state.contains("multi") {
        vec![circle, circle_segment, polygon, ray, line, arc]
    } else {
        vec![circle]
    };
    let parent = d
        .add_mapped_point_iteration_with_bindings(
            &[a, b, c],
            &[images],
            &(origin..=arc).collect::<Vec<_>>(),
            &bindings,
            IterationDepth::Fixed(3),
        )
        .unwrap();
    for o in d.objects.iter_mut().take(parent - 1) {
        o.visible = false;
    }
    println!("{state} derivedA{a}, seeds[{a},{b},{c}], parent{parent}, sourcebindings{bindings:?}");
    (d, parent)
}
fn main() {
    let state = std::env::args().nth(1).unwrap();
    if state.starts_with("derived") {
        let (d, _) = derived_document(&state);
        d.verify_roundtrip().unwrap();
        std::fs::write(
            format!("/tmp/iteration_binding47_{state}_input.gsp"),
            d.encode().unwrap(),
        )
        .unwrap();
        return;
    }

    let mut d = Document::new();
    let a = d.add(Kind::Free(Point { x: 100., y: 180. }));
    let b = d.add(Kind::Free(Point {
        x: if state.contains("edited") { 180. } else { 160. },
        y: if state.contains("edited") { 200. } else { 180. },
    }));
    let c = d.add(Kind::Free(Point { x: 100., y: 240. }));
    let origin = d.add(Kind::Free(Point { x: 80., y: 80. }));
    let target = d.add(Kind::Free(Point { x: 200., y: 100. }));
    let mut images = vec![];
    for seed in [a, b, c] {
        let scaled = d.add(Kind::ScaledPoint(seed, origin, 0.75));
        images.push(d.add(Kind::VectorTranslatedPoint(scaled, origin, target)));
    }
    let segment = d.add(Kind::Segment(a, b));
    let circle = d.add(Kind::Circle(a, b));
    let circle_segment = d.add(Kind::CircleBySegment(a, segment));
    let polygon = d.add(Kind::Polygon(vec![a, b, c]));
    let ray = d.add(Kind::Ray(a, b));
    let line = d.add(Kind::Line(a, c));
    let arc = d.add(Kind::CenterArc(a, b, c));
    let mut extra = None;
    if state.starts_with("transformed_circle") {
        extra = Some(d.add(Kind::ScaledCircle(circle, origin, 0.5)));
    }
    if state.starts_with("transformed_polygon") {
        extra = Some(d.add(Kind::ScaledPolygon(polygon, origin, 0.5)));
    }
    let mut frozen = None;
    if state.starts_with("frozen_circle") {
        let center = d.add(Kind::SegmentMidpoint(segment));
        frozen = Some(center);
        extra = Some(d.add(Kind::Circle(center, b)));
    }
    let mut seeds = vec![a, b, c];
    let mut scalar_seed = None;
    if state.starts_with("undefined_circle") {
        let t = d.add_named_parameter("t", 0., (10, 40)).unwrap();
        let next = d.add_parameter_calculation(t, "t+1", (10, 70)).unwrap();
        let ratio = d
            .add_parameter_calculation(t, "1/((t-2)*(t-2))", (10, 100))
            .unwrap();
        extra = Some(d.add(Kind::CalculatedRatioScaledCircle(circle, origin, ratio)));
        scalar_seed = Some(t);
        seeds.push(t);
        images.push(next);
    }
    let bindings = if let Some(extra) = extra {
        vec![extra]
    } else if state.starts_with("circle_segment") {
        vec![circle_segment]
    } else if state.starts_with("circle") {
        vec![circle]
    } else if state.starts_with("polygon") {
        vec![polygon]
    } else if state.starts_with("ray") {
        vec![ray]
    } else if state.starts_with("line") {
        vec![line]
    } else if state.starts_with("arc") {
        vec![arc]
    } else {
        vec![circle, circle_segment, polygon, ray, line, arc]
    };
    let carried = (4..=d.objects.len())
        .filter(|id| Some(*id) != frozen && Some(*id) != scalar_seed)
        .collect::<Vec<_>>();
    let depth = if state.contains("numeric") {
        IterationDepth::Value(
            d.add_named_parameter(
                "n",
                if state.contains("edited") { 2. } else { 3. },
                (10, 40),
            )
            .unwrap(),
        )
    } else {
        IterationDepth::Fixed(3)
    };
    let parent = d
        .add_mapped_point_iteration_with_bindings(&seeds, &[images], &carried, &bindings, depth)
        .unwrap();
    if state.contains("final") {
        d.set_mapped_iteration_final_only(parent, true).unwrap();
    }
    for o in d.objects.iter_mut().take(parent - 1) {
        o.visible = false;
    }
    let bytes = d.encode().unwrap();
    std::fs::write(format!("/tmp/iteration_binding47_{state}_input.gsp"), bytes).unwrap();
    d.verify_roundtrip().unwrap();
    println!(
        "STATE {state} parent {parent} bindings {bindings:?} sourceB {}",
        b
    );
    for o in d.objects.iter().skip(parent - 1) {
        println!("{} {:?}", o.id, o.kind);
    }
}
