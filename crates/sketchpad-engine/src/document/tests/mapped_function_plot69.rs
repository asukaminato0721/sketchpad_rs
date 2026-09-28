use super::*;

fn plotted_mapping(carry_plot: bool) -> (Document, usize, usize) {
    let mut doc = Document::new();
    let plot = doc
        .plot_new_function(SimpleFunction::Identity, Point { x: 300., y: 300. })
        .unwrap();
    let seed = 1;
    let tail = doc.add(Kind::Free(Point { x: 20., y: 20. }));
    let head = doc.add(Kind::Free(Point { x: 60., y: 20. }));
    let next = doc.add(Kind::VectorTranslatedPoint(seed, tail, head));
    let output = doc.add(Kind::PointOnCurve(plot, 0.5));
    let carried = (2..=plot)
        .filter(|id| carry_plot || *id != plot)
        .chain([next, output])
        .collect::<Vec<_>>();
    let root = doc
        .add_mapped_point_iteration_with_bindings(
            &[seed],
            &[vec![next]],
            &carried,
            &[output],
            IterationDepth::Fixed(3),
        )
        .unwrap();
    (doc, root + 1, seed)
}

#[test]
fn carried_function_plot_follows_each_outer_frame_and_public_edit() {
    let (mut doc, binding, seed) = plotted_mapping(true);
    assert!(!doc.partial_view);
    assert_eq!(
        doc.point_iteration_images(binding).unwrap(),
        [340., 380., 420.].map(|x| Point { x, y: 300. })
    );
    let original = doc.encode().unwrap();
    doc.verify_roundtrip().unwrap();
    doc.checkpoint();
    assert!(doc.move_point(seed, Point { x: 300., y: 360. }));
    assert_eq!(
        doc.point_iteration_images(binding).unwrap(),
        [340., 380., 420.].map(|x| Point { x, y: 360. })
    );
    let edited = doc.encode().unwrap();
    let reopened = Document::from_bytes(&edited, None).unwrap();
    assert_eq!(
        reopened.point_iteration_images(binding),
        doc.point_iteration_images(binding)
    );
    assert_eq!(reopened.encode().unwrap(), edited);
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
}

#[test]
fn omitted_function_plot_still_follows_its_carried_frame() {
    let (mut doc, binding, seed) = plotted_mapping(false);
    assert_eq!(
        doc.point_iteration_images(binding).unwrap(),
        [340., 380., 420.].map(|x| Point { x, y: 300. })
    );
    doc.verify_roundtrip().unwrap();
    assert!(doc.move_point(seed, Point { x: 300., y: 360. }));
    assert_eq!(
        doc.point_iteration_images(binding).unwrap(),
        [340., 380., 420.].map(|x| Point { x, y: 360. })
    );
    let bytes = doc.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(
        reopened.point_iteration_images(binding),
        doc.point_iteration_images(binding)
    );
    reopened.verify_roundtrip().unwrap();
}

#[test]
fn original_function_plot_intersection_iterates_and_public_edit_moves_each_generation() {
    let original = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../gsp-rs/tests/Samples/个人专栏/高峻清作品/等边三角形迭代(gjq).gsp"
    ))
    .unwrap();
    let mut doc = Document::from_bytes(&original, None).unwrap();
    assert!(!doc.partial_view);
    assert!(matches!(doc.objects[7].kind, Kind::FunctionPlot(..)));
    assert!(matches!(
        doc.objects[10].kind,
        Kind::CurveIntersection(10, 8, 1)
    ));
    assert!(matches!(
        doc.objects[16].kind,
        Kind::MappedPointIteration { .. }
    ));
    assert!(matches!(
        doc.objects[17].kind,
        Kind::IterationBinding(11, 17)
    ));
    let before = doc.point_iteration_images(18).unwrap();
    assert_eq!(before.len(), 3);
    for (point, expected) in before.iter().zip([
        Point {
            x: 747.4633376088657,
            y: 430.4094488189827,
        },
        Point {
            x: 780.1950064132983,
            y: 335.9212598427116,
        },
        Point {
            x: 812.9266752177309,
            y: 203.63779527593272,
        },
    ]) {
        assert!(point.distance(expected) < 1e-8, "{point:?} vs {expected:?}");
    }
    assert_eq!(doc.encode().unwrap(), original);
    doc.checkpoint();
    assert!(doc.move_point(1, Point { x: 732., y: 526. }));
    let after = doc.point_iteration_images(18).unwrap();
    for (old, new) in before.iter().zip(&after) {
        assert!((new.x - old.x - 50.).abs() < 1e-8);
        assert!((new.y - old.y - 20.).abs() < 1e-8);
    }
    let edited = doc.encode().unwrap();
    let reopened = Document::from_bytes(&edited, None).unwrap();
    assert_eq!(reopened.point_iteration_images(18), Some(after));
    assert_eq!(reopened.encode().unwrap(), edited);
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
}
