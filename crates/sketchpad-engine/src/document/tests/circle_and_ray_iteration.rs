use super::*;

#[test]
fn calculated_circle_iteration_updates_copies_and_clears() {
    let mut doc = Document::new();
    let depth = doc.add_named_parameter("n", 5., (10, 30)).unwrap();
    let center = doc.add(Kind::Free(Point { x: 250., y: 180. }));
    let seed = doc.add(Kind::Free(Point { x: 310., y: 180. }));
    let angle = doc
        .add_parameter_calculation(depth, "360°/n", (10, 60))
        .unwrap();
    let count = doc
        .add_parameter_calculation(depth, "n-1", (10, 90))
        .unwrap();
    let image = doc
        .add_calculated_angle_rotation(seed, center, angle)
        .unwrap();
    let circle = doc.add(Kind::Circle(seed, image));
    assert!(
        doc.add_circle_iteration(seed, image, circle, IterationDepth::Value(angle))
            .is_err()
    );
    let iteration = doc
        .add_circle_iteration(seed, image, circle, IterationDepth::Value(count))
        .unwrap();
    let binding = iteration + 2;
    for n in [5., 6.] {
        doc.set_parameter_value(depth, n).unwrap();
        doc.move_point(center, Point { x: 200., y: 160. });
        doc.move_point(seed, Point { x: 270., y: 160. });
        let circles = doc.circle_iteration_images(binding).unwrap();
        assert_eq!(circles.len(), n as usize - 1);
        let origin = doc.position(center).unwrap();
        let chord = 140. * (std::f64::consts::PI / n).sin();
        for (c, r) in &circles {
            assert!((c.distance(origin) - 70.).abs() < 1e-9);
            assert!((*r - chord).abs() < 1e-9);
        }
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.circle_iteration_images(binding), Some(circles));
    }
    doc.selected.insert(iteration);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    assert_eq!(
        pasted
            .iter()
            .filter(|id| matches!(doc.objects[**id - 1].kind, Kind::IterationBinding(..)))
            .count(),
        2
    );
    let copied = *pasted
        .iter()
        .find(|id| matches!(doc.objects[**id - 1].kind, Kind::CircleIteration(..)))
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.circle_iteration_images(copied + 2).unwrap().len(),
        5
    );
    doc.set_parameter_value(depth, 2.5).unwrap();
    assert_eq!(doc.circle_iteration_images(binding).unwrap().len(), 1);
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
    doc.selected.clear();
    doc.selected.insert(depth);
    assert!(doc.clear_selected().unwrap().contains(&binding));
}

#[test]
fn carried_segment_iteration_import_generation_and_parent_edit() {
    let native_bytes =
        include_bytes!("../../../../../tests/fixtures/point_translation_carried_segment_native.gsp");
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(
        native.objects[3].kind,
        Kind::SegmentIteration(1, 2, 3, IterationDepth::Fixed(3))
    );
    assert_eq!(native.objects[4].kind, Kind::IterationBinding(2, 4));
    assert_eq!(native.objects[5].kind, Kind::IterationBinding(3, 4));

    let port_bytes =
        include_bytes!("../../../../../tests/fixtures/point_translation_carried_segment_port.gsp");
    let mut port = Document::from_bytes(port_bytes, None).unwrap();
    assert!(!port.partial_view);
    assert_eq!(port.encode().unwrap(), port_bytes);
    let lines = port.segment_iteration_images(4).unwrap();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].0, port.position(2).unwrap());
    assert_eq!(port.point_iteration_images(4).unwrap().len(), 3);
    assert!(port.move_point(1, Point { x: 120., y: 140. }));
    let mut moved = Document::from_bytes(&port.encode().unwrap(), None).unwrap();
    let lines = moved.segment_iteration_images(4).unwrap();
    assert!((lines[0].0.x - (120. + 37.79527559055118)).abs() < 1e-8);
    let seed = moved.position(1).unwrap();
    let first = moved.position(2).unwrap();
    assert!((lines[2].1.y - (seed.y + 4. * (first.y - seed.y))).abs() < 1e-8);
    moved.selected.insert(4);
    assert_eq!(moved.clear_selected().unwrap(), vec![4, 5, 6]);
    assert_eq!(
        Document::from_bytes(&moved.encode().unwrap(), None)
            .unwrap()
            .objects
            .len(),
        3
    );
}

#[test]
fn rotated_ray_iteration_preserves_orbit_and_dynamic_depth() {
    for (bytes, iteration, depth, seed, center) in [
        (
            &include_bytes!("../../../../../tests/fixtures/ray_rotation_live_depth_port.gsp")[..],
            6,
            1,
            3,
            2,
        ),
        (
            &include_bytes!("../../../../../tests/fixtures/ray_calculated_rotation_live_depth_port.gsp")
                [..],
            8,
            5,
            3,
            2,
        ),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        for n in [5., 6.] {
            doc.set_parameter_value(1, n).unwrap();
            doc.move_point(center, Point { x: 210., y: 170. });
            doc.move_point(seed, Point { x: 270., y: 170. });
            let expected = if depth == 1 {
                n as usize
            } else {
                n as usize - 1
            };
            let rays = doc.ray_iteration_images(iteration).unwrap();
            assert_eq!(rays.len(), expected);
            let points = doc.point_iteration_images(iteration).unwrap();
            let origin = doc.position(center).unwrap();
            for (i, (a, b)) in rays.iter().enumerate() {
                assert!((a.distance(origin) - 60.).abs() < 1e-9);
                assert!((b.distance(origin) - 60.).abs() < 1e-9);
                assert!(b.distance(points[i]) < 1e-9);
                if i > 0 {
                    assert!(a.distance(rays[i - 1].1) < 1e-9);
                }
            }
            let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
            assert!(!reopened.partial_view);
            assert_eq!(reopened.ray_iteration_images(iteration), Some(rays));
        }
        doc.selected.insert(iteration);
        let clipboard = doc.copy_selected().unwrap();
        let pasted = doc.paste(&clipboard).unwrap();
        assert_eq!(
            pasted
                .iter()
                .filter(|id| matches!(doc.objects[**id - 1].kind, Kind::IterationBinding(..)))
                .count(),
            2
        );
        let copied = pasted
            .iter()
            .copied()
            .find(|id| matches!(doc.objects[*id - 1].kind, Kind::RayIteration(..)))
            .unwrap();
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(
            reopened.ray_iteration_images(copied).unwrap().len(),
            if depth == 1 { 6 } else { 5 }
        );
        let mut cleared = Document::from_bytes(bytes, None).unwrap();
        cleared.selected.insert(depth);
        assert!(cleared.clear_selected().unwrap().contains(&iteration));
        assert!(
            !Document::from_bytes(&cleared.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}

#[test]
fn ray_iteration_rejects_angle_depth_and_handles_undefined_calculations() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/ray_calculated_rotation_live_depth_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(doc.add_ray_iteration(3, 6, 7, 4).is_err());
    doc.set_parameter_value(1, 2.5).unwrap();
    assert_eq!(doc.ray_iteration_images(8).unwrap().len(), 1);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.ray_iteration_images(8).unwrap().len(), 1);
    doc.set_parameter_value(1, 0.).unwrap();
    assert!(doc.position(6).is_none());
    assert!(doc.ray_iteration_images(8).is_none());
}

#[test]
fn carried_ray_iteration_import_generation_and_live_parameter() {
    let native_bytes = include_bytes!(
        "../../../../../tests/fixtures/point_translation_carried_ray_parameter_native.gsp"
    );
    let mut native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(native.objects[4].kind, Kind::RayIteration(1, 2, 3, 4));
    assert_eq!(native.objects[5].kind, Kind::IterationBinding(2, 5));
    assert_eq!(native.objects[6].kind, Kind::IterationBinding(3, 5));
    assert_eq!(native.point_iteration_images(5).unwrap().len(), 4);
    assert_eq!(native.ray_iteration_images(5).unwrap().len(), 4);
    native.set_parameter_value(4, 5.).unwrap();
    let edited = Document::from_bytes(&native.encode().unwrap(), None).unwrap();
    assert!(!edited.partial_view);
    assert_eq!(edited.point_iteration_images(5).unwrap().len(), 5);
    assert_eq!(edited.ray_iteration_images(5).unwrap().len(), 5);
    native.set_parameter_value(4, 5.5).unwrap();
    assert_eq!(native.ray_iteration_images(5).unwrap().len(), 5);

    let port_bytes =
        include_bytes!("../../../../../tests/fixtures/point_translation_carried_ray_depth5_port.gsp");
    let mut port = Document::from_bytes(port_bytes, None).unwrap();
    assert!(!port.partial_view);
    assert_eq!(port.encode().unwrap(), port_bytes);
    assert_eq!(port.point_iteration_images(5).unwrap().len(), 5);
    assert_eq!(port.ray_iteration_images(5).unwrap().len(), 5);
    assert!(port.move_point(1, Point { x: 120., y: 140. }));
    let mut moved = Document::from_bytes(&port.encode().unwrap(), None).unwrap();
    let rays = moved.ray_iteration_images(5).unwrap();
    let seed = moved.position(1).unwrap();
    let first = moved.position(2).unwrap();
    assert!((rays[0].0.x - first.x).abs() < 1e-8);
    assert!((rays[4].0.y - (seed.y + 5. * (first.y - seed.y))).abs() < 1e-8);
    assert!(moved.add_ray_iteration(1, 2, 3, 1).is_err());
    moved.selected.insert(5);
    assert_eq!(moved.clear_selected().unwrap(), vec![5, 6, 7]);
    assert_eq!(
        Document::from_bytes(&moved.encode().unwrap(), None)
            .unwrap()
            .objects
            .len(),
        4
    );
}
