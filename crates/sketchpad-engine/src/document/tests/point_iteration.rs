use super::*;

#[test]
fn rotated_point_iteration_updates_center_and_copies_dependencies() {
    let bytes = include_bytes!("../../../../../tests/fixtures/rotated_point_iteration_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(
        doc.objects[3].kind,
        Kind::PointIteration(1, 3, IterationDepth::Fixed(4))
    );
    let images = doc.point_iteration_images(4).unwrap();
    assert_eq!(images.len(), 4);
    let seed = doc.position(1).unwrap();
    let center = doc.position(2).unwrap();
    for (i, p) in images.iter().enumerate() {
        let expected =
            rotate_point_about(seed, center, (60. * (i + 2) as f64).to_radians()).unwrap();
        assert!(p.distance(expected) < 1e-10);
        assert!((p.distance(center) - 100.).abs() < 1e-10);
    }
    doc.move_point(2, Point { x: 270., y: 200. });
    doc.move_point(1, Point { x: 380., y: 190. });
    let center = doc.position(2).unwrap();
    let seed = doc.position(1).unwrap();
    let radius = seed.distance(center);
    for p in doc.point_iteration_images(4).unwrap() {
        assert!((p.distance(center) - radius).abs() < 1e-10);
    }
    doc.selected.insert(4);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = pasted
        .iter()
        .copied()
        .find(|id| matches!(doc.objects[*id - 1].kind, Kind::PointIteration(_, _, _)))
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.point_iteration_images(copied).unwrap().len(), 4);
    assert!(
        matches!(reopened.objects[copied-1].kind,Kind::PointIteration(seed,image,IterationDepth::Fixed(4)) if seed!=1 && image!=3)
    );
    let mut cleared = Document::from_bytes(bytes, None).unwrap();
    cleared.selected.insert(2);
    assert_eq!(cleared.clear_selected().unwrap(), vec![2, 3, 4, 5]);
    assert_eq!(
        Document::from_bytes(&cleared.encode().unwrap(), None)
            .unwrap()
            .objects
            .len(),
        1
    );
}

#[test]
fn translated_point_iteration_import_generation_and_parent_edit() {
    let native_bytes =
        include_bytes!("../../../../../tests/fixtures/point_translation_iteration_native.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(
        native.objects[2].kind,
        Kind::PointIteration(1, 2, IterationDepth::Fixed(3))
    );
    assert_eq!(native.objects[3].kind, Kind::IterationBinding(2, 3));
    let images = native.point_iteration_images(3).unwrap();
    assert_eq!(images.len(), 3);
    assert!((images[0].x - (280. + 2. * 37.79527559055118)).abs() < 1e-8);

    let port_bytes =
        include_bytes!("../../../../../tests/fixtures/point_translation_iteration_depth5_port.gsp");
    let mut port = Document::from_bytes(port_bytes, None).unwrap();
    assert!(!port.partial_view);
    assert_eq!(port.encode().unwrap(), port_bytes);
    assert_eq!(port.point_iteration_images(3).unwrap().len(), 5);
    assert!(port.move_point(1, Point { x: 120., y: 210. }));
    let mut moved = Document::from_bytes(&port.encode().unwrap(), None).unwrap();
    let images = moved.point_iteration_images(3).unwrap();
    assert!((images[0].x - (120. + 2. * 37.79527559055118)).abs() < 1e-8);
    assert!((images[4].y - 210.).abs() < 1e-8);
    assert_eq!(
        moved
            .add_point_iteration(1, 2, IterationDepth::Fixed(0))
            .is_err(),
        true
    );
    moved.selected.insert(3);
    assert_eq!(moved.clear_selected().unwrap(), vec![3, 4]);
    assert_eq!(
        Document::from_bytes(&moved.encode().unwrap(), None)
            .unwrap()
            .objects
            .len(),
        2
    );
}
#[test]
fn regular_polygon_calculated_rotation_and_depth_update_and_roundtrip() {
    let bytes = include_bytes!("../../../../../tests/fixtures/regular_polygon_iteration_native.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(doc.parameter_calculation_value(5), Some(72.));
    assert_eq!(
        doc.objects[7].kind,
        Kind::SegmentIteration(4, 6, 7, IterationDepth::Value(3))
    );
    for n in [5., 6., 9.] {
        doc.set_parameter_value(1, n).unwrap();
        doc.move_point(2, Point { x: 200., y: 160. });
        doc.move_point(4, Point { x: 250., y: 160. });
        let lines = doc.segment_iteration_images(8).unwrap();
        assert_eq!(lines.len(), n as usize - 1);
        assert!(lines.last().unwrap().1.distance(doc.position(4).unwrap()) < 1e-9);
        for (i, (a, b)) in lines.iter().enumerate() {
            assert!((a.distance(doc.position(2).unwrap()) - 50.).abs() < 1e-9);
            assert!((b.distance(doc.position(2).unwrap()) - 50.).abs() < 1e-9);
            if i > 0 {
                assert!(a.distance(lines[i - 1].1) < 1e-9);
            }
        }
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.segment_iteration_images(8), Some(lines));
    }
    doc.selected.insert(8);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    assert_eq!(pasted.len(), 10);
    let copied = pasted
        .iter()
        .copied()
        .find(|id| matches!(doc.objects[*id - 1].kind, Kind::SegmentIteration(..)))
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.segment_iteration_images(copied).unwrap().len(), 8);
    let mut cleared = Document::from_bytes(bytes, None).unwrap();
    cleared.selected.insert(1);
    assert_eq!(
        cleared.clear_selected().unwrap(),
        vec![1, 3, 5, 6, 7, 8, 9, 10]
    );
    assert_eq!(
        Document::from_bytes(&cleared.encode().unwrap(), None)
            .unwrap()
            .objects
            .len(),
        2
    );
}

#[test]
fn point_iteration_live_depth_updates_copies_and_clears() {
    let bytes = include_bytes!("../../../../../tests/fixtures/point_live_depth_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(
        doc.objects[3].kind,
        Kind::PointIteration(2, 3, IterationDepth::Value(1))
    );
    assert_eq!(doc.point_iteration_images(4).unwrap().len(), 4);
    doc.set_parameter_value(1, 6.).unwrap();
    doc.move_point(2, Point { x: 110., y: 170. });
    let points = doc.point_iteration_images(4).unwrap();
    assert_eq!(points.len(), 6);
    assert!((points[5].x - (110. + 7. * 0.5 * 37.79527559055118)).abs() < 1e-10);
    assert!(points.iter().all(|p| p.y == 170.));
    doc.selected.insert(4);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    assert_eq!(pasted.len(), 5);
    let copied = pasted
        .iter()
        .copied()
        .find(|id| matches!(doc.objects[*id - 1].kind, Kind::PointIteration(..)))
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.point_iteration_images(copied).unwrap().len(), 6);
    let mut cleared = Document::from_bytes(bytes, None).unwrap();
    cleared.selected.insert(1);
    assert_eq!(cleared.clear_selected().unwrap(), vec![1, 4, 5]);
    assert!(
        !Document::from_bytes(&cleared.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
    let literal = doc.add_calculation_expression("2+3", (10, 40)).unwrap();
    let literal_iteration = doc
        .add_point_iteration(2, 3, IterationDepth::Value(literal))
        .unwrap();
    assert_eq!(
        doc.point_iteration_images(literal_iteration).unwrap().len(),
        5
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(
        reopened
            .point_iteration_images(literal_iteration)
            .unwrap()
            .len(),
        5
    );
    doc.set_parameter_value(1, 2.5).unwrap();
    assert_eq!(doc.point_iteration_images(4).unwrap().len(), 2);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.point_iteration_images(4).unwrap().len(), 2);
}

#[test]
fn calculated_rotation_point_iteration_uses_scalar_calculation_depth() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/point_calculated_rotation_live_depth_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(
        doc.objects[6].kind,
        Kind::PointIteration(3, 6, IterationDepth::Value(5))
    );
    assert!(
        doc.add_point_iteration(3, 6, IterationDepth::Value(4))
            .is_err()
    );
    for n in [5., 6., 9.] {
        doc.set_parameter_value(1, n).unwrap();
        doc.move_point(2, Point { x: 210., y: 170. });
        doc.move_point(3, Point { x: 270., y: 170. });
        let points = doc.point_iteration_images(7).unwrap();
        assert_eq!(points.len(), n as usize - 1);
        assert!(points.last().unwrap().distance(doc.position(3).unwrap()) < 1e-9);
        assert!(
            points
                .iter()
                .all(|p| (p.distance(doc.position(2).unwrap()) - 60.).abs() < 1e-9)
        );
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.point_iteration_images(7), Some(points));
    }
    doc.selected.insert(7);
    let pasted = doc.paste(&doc.copy_selected().unwrap()).unwrap();
    assert_eq!(pasted.len(), 8);
    let copied = pasted
        .iter()
        .copied()
        .find(|id| matches!(doc.objects[*id - 1].kind, Kind::PointIteration(..)))
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.point_iteration_images(copied).unwrap().len(), 8);
    let mut cleared = Document::from_bytes(bytes, None).unwrap();
    cleared.selected.insert(5);
    assert_eq!(cleared.clear_selected().unwrap(), vec![5, 7, 8]);
    assert!(
        !Document::from_bytes(&cleared.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
