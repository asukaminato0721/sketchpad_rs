use super::*;

#[test]
fn rotated_segment_iteration_preserves_edges_and_remaps_center() {
    let bytes = include_bytes!("../../../../../tests/fixtures/rotated_segment_iteration_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(
        doc.objects[4].kind,
        Kind::SegmentIteration(1, 3, 4, IterationDepth::Fixed(4))
    );
    let segments = doc.segment_iteration_images(5).unwrap();
    let points = doc.point_iteration_images(5).unwrap();
    assert_eq!(segments.len(), 4);
    for (i, (a, b)) in segments.iter().enumerate() {
        assert!((a.distance(*b) - 100.).abs() < 1e-10);
        assert!(b.distance(points[i]) < 1e-10);
        if i > 0 {
            assert!(a.distance(segments[i - 1].1) < 1e-10);
        }
    }
    doc.move_point(2, Point { x: 270., y: 200. });
    doc.move_point(1, Point { x: 380., y: 190. });
    let seed = doc.position(1).unwrap();
    let center = doc.position(2).unwrap();
    let radius = seed.distance(center);
    for (a, b) in doc.segment_iteration_images(5).unwrap() {
        assert!((a.distance(center) - radius).abs() < 1e-10);
        assert!((b.distance(center) - radius).abs() < 1e-10);
        assert!((a.distance(b) - radius).abs() < 1e-10);
    }
    doc.selected.insert(5);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = pasted
        .iter()
        .copied()
        .find(|id| {
            matches!(
                doc.objects[*id - 1].kind,
                Kind::SegmentIteration(_, _, _, _)
            )
        })
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.segment_iteration_images(copied).unwrap().len(), 4);
    let mut cleared = Document::from_bytes(bytes, None).unwrap();
    cleared.selected.insert(2);
    assert_eq!(cleared.clear_selected().unwrap(), vec![2, 3, 4, 5, 6, 7]);
    assert_eq!(
        Document::from_bytes(&cleared.encode().unwrap(), None)
            .unwrap()
            .objects
            .len(),
        1
    );
}

#[test]
fn carried_circle_iteration_native_fixture_and_movement() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/point_translation_carried_circle_native_verified.gsp"
    );
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(
        doc.objects[3].kind,
        Kind::CircleIteration(1, 2, 3, IterationDepth::Fixed(3))
    );
    assert!(doc.circle_iteration_images(4).is_none());
    assert!(doc.circle_iteration_images(5).is_none());
    let circles = doc.circle_iteration_images(6).unwrap();
    assert_eq!(circles.len(), 3);
    assert_eq!(circles[0].0, doc.position(2).unwrap());
    let radius = doc.position(1).unwrap().distance(doc.position(2).unwrap());
    for (i, (center, r)) in circles.iter().enumerate() {
        assert!((*r - radius).abs() < 1e-9);
        assert!((center.x - (100. + (i + 1) as f64 * radius)).abs() < 1e-9);
    }
    doc.move_point(1, Point { x: 120., y: 140. });
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!((reopened.circle_iteration_images(6).unwrap()[0].0.x - 120. - radius).abs() < 1e-9);
    doc.selected.insert(3);
    assert_eq!(doc.clear_selected().unwrap(), vec![3, 4, 5, 6]);
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn multiple_source_native_import_changes_copy_and_delete() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/calculation_multi_scalar_native_verified.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(doc.objects[2].kind.dependencies(), vec![1, 2]);
    assert_eq!(doc.numeric_value(3), Some(5.));
    doc.set_parameter_value(2, 4.).unwrap();
    assert_eq!(doc.numeric_value(3), Some(6.));
    let product = doc
        .add_multi_calculation(&[1, 2, 3], "(v1+v2)*v3", (10, 130))
        .unwrap();
    assert_eq!(doc.numeric_value(product), Some(36.));
    doc.selected.insert(product);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = *pasted.last().unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.numeric_value(copied), Some(36.));
    assert!(
        reopened.objects[copied - 1]
            .kind
            .dependencies()
            .iter()
            .all(|id| *id > product)
    );
    doc.selected.clear();
    doc.selected.insert(2);
    let deleted = doc.clear_selected().unwrap();
    assert!(deleted.contains(&3));
    assert!(deleted.contains(&product));
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn multiple_measurement_units_and_live_depth_follow_both_sources() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 100., y: 160. }));
    let b = doc.add(Kind::Free(Point {
        x: 175.59055118110236,
        y: 160.,
    }));
    let segment = doc.add(Kind::Segment(a, b));
    let length = doc.add(Kind::SegmentLength(segment));
    let scale = doc.add_named_parameter("k", 2., (10, 30)).unwrap();
    let result = doc
        .add_multi_calculation(&[length, scale], "v1*v2", (10, 65))
        .unwrap();
    assert_eq!(
        doc.objects[result - 1].kind.numeric_unit(),
        Some(NumericUnit::Length)
    );
    assert!((doc.numeric_value(result).unwrap() - 4.).abs() < 1e-12);
    doc.set_parameter_value(scale, 3.).unwrap();
    assert!((doc.numeric_value(result).unwrap() - 6.).abs() < 1e-12);
    doc.move_point(
        b,
        Point {
            x: 213.38582677165354,
            y: 160.,
        },
    );
    assert!((doc.numeric_value(result).unwrap() - 9.).abs() < 1e-12);
    let depth = doc
        .add_multi_calculation(&[scale, scale], "v1+v2", (10, 100))
        .unwrap();
    let image = doc.add(Kind::FixedRectangularTranslatedPoint(
        a,
        37.79527559055118,
        0.5,
        37.79527559055118,
        0.,
    ));
    let iteration = doc
        .add_point_iteration(a, image, IterationDepth::Value(depth))
        .unwrap();
    assert_eq!(doc.point_iteration_images(iteration).unwrap().len(), 6);
    doc.set_parameter_value(scale, 4.).unwrap();
    assert_eq!(doc.point_iteration_images(iteration).unwrap().len(), 8);
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
    let wrong = doc
        .add_multi_calculation(&[scale, scale], "sqrt(v1-v2-1)", (10, 135))
        .unwrap();
    assert!(doc.numeric_value(wrong).is_none());
    assert!(
        crate::render::number_lines_for_object(&doc, wrong)
            .unwrap()
            .1[0]
            .contains("undefined")
    );
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
