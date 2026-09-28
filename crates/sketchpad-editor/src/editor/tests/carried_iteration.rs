use super::*;

#[test]
fn selected_rotation_creates_point_iteration() {
    let mut document = Document::new();
    let seed = document.add(Kind::Free(Point { x: 350., y: 180. }));
    let center = document.add(Kind::Free(Point { x: 250., y: 180. }));
    let image = document.add(Kind::RotatedPoint(seed, center, 60.));
    let mut editor = Editor::new(document);
    editor.selection_order = vec![image, seed];
    editor.doc.selected.extend([seed, image]);
    editor.iterate_selected(4);
    assert_eq!(
        editor.doc.objects[3].kind,
        Kind::PointIteration(seed, image, IterationDepth::Fixed(4))
    );
    assert_eq!(editor.doc.point_iteration_images(4).unwrap().len(), 4);
    assert_eq!(
        editor.doc.selected.iter().copied().collect::<Vec<_>>(),
        vec![4]
    );
}

#[test]
fn source_free_angle_can_be_marked_and_used_for_rotation() {
    let mut doc = Document::new();
    let angle = doc
        .add_calculation_expression("asin(.5)", (10, 30))
        .unwrap();
    let center = doc.add(Kind::Free(Point { x: 250., y: 180. }));
    let seed = doc.add(Kind::Free(Point { x: 310., y: 180. }));
    let mut editor = Editor::new(doc);
    editor.select_only(angle);
    editor.mark_angle_selected();
    assert!(matches!(editor.marked_angle, Some(MarkedAngle::Value(id)) if id == angle));
    assert!(editor.doc.selected.is_empty());
    let image = editor
        .doc
        .add_calculated_angle_rotation(seed, center, angle)
        .unwrap();
    assert!(editor.doc.position(image).unwrap().y < 180.);
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn selected_circle_iteration_fixed_live_and_picking() {
    for live in [false, true] {
        let mut document = Document::new();
        let seed = document.add(Kind::Free(Point { x: 100., y: 160. }));
        let image = document.add(Kind::FixedRectangularTranslatedPoint(
            seed,
            37.79527559055118,
            0.5,
            37.79527559055118,
            0.,
        ));
        let circle = document.add(Kind::Circle(seed, image));
        let mut selected = vec![circle, image, seed];
        let count = if live {
            let id = document.add_named_parameter("n", 3., (10, 30)).unwrap();
            selected.push(id);
            Some(id)
        } else {
            None
        };
        let mut editor = Editor::new(document);
        editor.selection_order = selected.clone();
        editor.doc.selected.extend(selected);
        editor.iterate_selected(3);
        let iteration = if live { 5 } else { 4 };
        assert_eq!(
            editor.doc.objects[iteration - 1].kind,
            Kind::CircleIteration(
                seed,
                image,
                circle,
                count.map_or(IterationDepth::Fixed(3), IterationDepth::Value)
            )
        );
        let binding = iteration + 2;
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(500., 400.)),
            scale: 2.,
            center: Point { x: 200., y: 160. },
        };
        let (center, radius) = editor.doc.circle_iteration_images(binding).unwrap()[2];
        let at = viewport.screen(Point {
            x: center.x,
            y: center.y + radius,
        });
        assert_eq!(editor.hit(&viewport, at), Some(binding));
        editor.doc.selected.clear();
        editor.doc.selected.insert(binding);
        editor.doc.set_selected_visibility(false);
        assert_ne!(editor.hit(&viewport, at), Some(binding));
        assert!(
            !Document::from_bytes(&editor.doc.encode().unwrap(), None)
                .unwrap()
                .partial_view
        );
    }
}

#[test]
fn selected_rotation_and_segment_create_carried_iteration() {
    let mut document = Document::new();
    let seed = document.add(Kind::Free(Point { x: 350., y: 180. }));
    let center = document.add(Kind::Free(Point { x: 250., y: 180. }));
    let image = document.add(Kind::RotatedPoint(seed, center, 60.));
    let segment = document.add(Kind::Segment(seed, image));
    let mut editor = Editor::new(document);
    editor.selection_order = vec![segment, image, seed];
    editor.doc.selected.extend([seed, image, segment]);
    editor.iterate_selected(4);
    assert_eq!(
        editor.doc.objects[4].kind,
        Kind::SegmentIteration(seed, image, segment, IterationDepth::Fixed(4))
    );
    assert_eq!(editor.doc.segment_iteration_images(5).unwrap().len(), 4);
    assert_eq!(
        editor.doc.selected.iter().copied().collect::<Vec<_>>(),
        vec![5]
    );
}

#[test]
fn selected_translation_and_segment_create_carried_iteration() {
    let mut document = Document::new();
    let seed = document.add(Kind::Free(Point { x: 150., y: 180. }));
    let image = document.add(Kind::FixedRectangularTranslatedPoint(
        seed,
        37.79527559055118,
        1.,
        37.79527559055118,
        1.,
    ));
    let segment = document.add(Kind::Segment(seed, image));
    let mut editor = Editor::new(document);
    editor.selection_order = vec![segment, image, seed];
    editor.doc.selected.extend([seed, image, segment]);
    editor.iterate_selected(4);
    assert_eq!(
        editor.doc.objects[3].kind,
        Kind::SegmentIteration(seed, image, segment, IterationDepth::Fixed(4))
    );
    assert_eq!(
        editor.doc.selected.iter().copied().collect::<Vec<_>>(),
        vec![4]
    );
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn calculated_rotation_ray_and_scalar_formula_create_live_iteration() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/ray_calculated_rotation_live_depth_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    doc.selected.insert(8);
    doc.clear_selected().unwrap();
    let mut editor = Editor::new(doc);
    editor.doc.selected.extend([3, 6, 7, 5]);
    editor.selection_order = vec![7, 5, 6, 3];
    editor.iterate_selected(4);
    let id = *editor.doc.selected.iter().next().unwrap();
    assert_eq!(
        editor.doc.objects[id - 1].kind,
        Kind::RayIteration(3, 6, 7, 5)
    );
    editor.doc.set_parameter_value(1, 6.).unwrap();
    assert_eq!(editor.doc.ray_iteration_images(id).unwrap().len(), 5);
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn selected_translation_ray_and_parameter_create_live_iteration() {
    let mut document = Document::new();
    let seed = document.add(Kind::Free(Point { x: 150., y: 180. }));
    let image = document.add(Kind::FixedRectangularTranslatedPoint(
        seed,
        37.79527559055118,
        1.,
        37.79527559055118,
        1.,
    ));
    let ray = document.add(Kind::Ray(seed, image));
    let parameter = document.add_named_parameter("n", 4., (10, 5)).unwrap();
    let mut editor = Editor::new(document);
    editor.selection_order = vec![parameter, ray, image, seed];
    editor.doc.selected.extend([seed, image, ray, parameter]);
    editor.iterate_selected(3);
    assert_eq!(
        editor.doc.objects[4].kind,
        Kind::RayIteration(seed, image, ray, parameter)
    );
    assert_eq!(editor.doc.point_iteration_images(5).unwrap().len(), 4);
    assert_eq!(editor.doc.ray_iteration_images(5).unwrap().len(), 4);
    assert_eq!(
        editor.doc.selected.iter().copied().collect::<Vec<_>>(),
        vec![5]
    );
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}
