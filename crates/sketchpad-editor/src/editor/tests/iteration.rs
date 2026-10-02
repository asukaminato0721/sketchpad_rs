use super::*;

#[test]
fn selected_triangle_and_two_mappings_create_branch_iteration() {
    let mut doc = Document::new();
    let seed = doc.add(Kind::Free(Point { x: 80., y: 220. }));
    let first = doc.add(Kind::PolarTranslatedPoint(seed, 90., 37.79527559055118, 1.));
    let second = doc.add(Kind::FixedRectangularTranslatedPoint(
        seed,
        37.79527559055118,
        1.,
        37.79527559055118,
        1.,
    ));
    let polygon = doc.add(Kind::Polygon(vec![seed, first, second]));
    let edges = [
        doc.add(Kind::Segment(seed, first)),
        doc.add(Kind::Segment(first, second)),
        doc.add(Kind::Segment(second, seed)),
    ];
    let n = doc.add_named_parameter("n", 3., (10, 5)).unwrap();
    let mut editor = Editor::new(doc);
    editor.selection_order = vec![
        edges[2], second, n, polygon, seed, edges[0], first, edges[1],
    ];
    editor
        .doc
        .selected
        .extend(editor.selection_order.iter().copied());
    editor.iterate_selected(2);
    assert!(matches!(
        editor.doc.objects[8].kind,
        Kind::BranchingPolygonIteration {
            depth: IterationDepth::Value(8),
            ..
        }
    ));
    assert_eq!(editor.doc.branch_iteration_offsets(9).unwrap().len(), 9);
    assert!(editor.status.contains("depth 3"));
    let points = editor.doc.polygon_iteration_images(10).unwrap()[0]
        .0
        .clone();
    let center = Point {
        x: points.iter().map(|p| p.x).sum::<f64>() / 3.,
        y: points.iter().map(|p| p.y).sum::<f64>() / 3.,
    };
    let vp = Viewport {
        rect: egui::Rect::from_min_size(egui::pos2(20., 30.), egui::vec2(500., 400.)),
        scale: 2.,
        center: Point { x: 200., y: 180. },
    };
    assert_eq!(editor.hit(&vp, vp.screen(center)), Some(10));
    editor.doc.set_parameter_value(n, 2.).unwrap();
    assert_eq!(editor.doc.branch_iteration_offsets(9).unwrap().len(), 5);
}

#[test]
fn attached_numeric_label_hit_target_follows_world_coordinates() {
    let document = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/point_numeric_label_port.gsp"),
        None,
    )
    .unwrap();
    let editor = Editor::new(document);
    let vp = Viewport {
        rect: egui::Rect::from_min_size(egui::pos2(20., 30.), egui::vec2(500., 400.)),
        scale: 2.,
        center: Point { x: 250., y: 180. },
    };
    let point = vp.screen(editor.doc.position(3).unwrap());
    assert_eq!(editor.hit(&vp, point + egui::vec2(17., 0.)), Some(4));
}

#[test]
fn selected_point_parameter_mapping_creates_numeric_iteration() {
    let mut document = Document::new();
    let seed = document.add(Kind::Free(Point { x: 330., y: 180. }));
    let center = document.add(Kind::Free(Point { x: 250., y: 180. }));
    let image = document.add(Kind::RotatedPoint(seed, center, 60.));
    let parameter = document.add_named_parameter("a", 0., (10, 5)).unwrap();
    let calculation = document
        .add_parameter_calculation(parameter, "a+1", (10, 35))
        .unwrap();
    let label = document
        .add_point_numeric_label(image, calculation)
        .unwrap();
    let mut editor = Editor::new(document);
    editor.selection_order = vec![label, calculation, image, parameter, seed];
    editor
        .doc
        .selected
        .extend([seed, image, parameter, calculation, label]);
    editor.iterate_selected(3);
    assert_eq!(
        editor.doc.objects[6].kind,
        Kind::PointParameterIteration(
            seed,
            parameter,
            image,
            calculation,
            label,
            sketchpad_engine::document::IterationDepth::Fixed(3)
        )
    );
    assert_eq!(
        editor
            .doc
            .point_parameter_iteration_labels(7)
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        editor.doc.selected.iter().copied().collect::<Vec<_>>(),
        vec![7]
    );
}

#[test]
fn numeric_iteration_selection_maps_second_formula_source() {
    let mut document = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/numeric_iteration_multi_port.gsp"),
        None,
    )
    .unwrap();
    document.selected.insert(7);
    document.clear_selected().unwrap();
    let mut editor = Editor::new(document);
    editor.selection_order = vec![6, 2, 3, 1, 5];
    editor
        .doc
        .selected
        .extend(editor.selection_order.iter().copied());
    editor.iterate_selected(3);
    let iteration = editor
        .doc
        .objects
        .iter()
        .find(|object| !object.deleted && matches!(object.kind, Kind::PointParameterIteration(..)))
        .unwrap();
    assert!(matches!(
        iteration.kind,
        Kind::PointParameterIteration(
            1,
            3,
            2,
            5,
            6,
            sketchpad_engine::document::IterationDepth::Fixed(3)
        )
    ));
    assert_eq!(
        editor
            .doc
            .point_parameter_iteration_labels(iteration.id)
            .unwrap()
            .iter()
            .map(|(_, text)| text.as_str())
            .collect::<Vec<_>>(),
        ["5.00", "7.00", "9.00"]
    );
}

#[test]
fn numeric_iteration_selection_uses_calculation_source_and_distinct_depth() {
    let mut document = Document::new();
    let seed = document.add(Kind::Free(Point { x: 100., y: 180. }));
    let image = document.add(Kind::PolarTranslatedPoint(
        seed,
        0.,
        37.79527559055118,
        0.79375,
    ));
    let parameter = document.add_named_parameter("a", 0., (10, 5)).unwrap();
    let calculation = document
        .add_parameter_calculation(parameter, "a+1", (10, 35))
        .unwrap();
    let label = document
        .add_point_numeric_label(image, calculation)
        .unwrap();
    let depth_parameter = document.add_named_parameter("n", 5., (10, 65)).unwrap();
    let mut editor = Editor::new(document);
    editor.selection_order = vec![depth_parameter, label, image, parameter, seed, calculation];
    editor
        .doc
        .selected
        .extend(editor.selection_order.iter().copied());
    editor.iterate_selected(3);
    assert_eq!(
        editor.doc.objects[6].kind,
        Kind::PointParameterIteration(
            seed,
            parameter,
            image,
            calculation,
            label,
            sketchpad_engine::document::IterationDepth::Value(depth_parameter)
        )
    );
    assert_eq!(
        editor
            .doc
            .point_parameter_iteration_labels(7)
            .unwrap()
            .len(),
        5
    );
    let vp = Viewport {
        rect: egui::Rect::from_min_size(egui::pos2(20., 30.), egui::vec2(500., 400.)),
        scale: 2.,
        center: Point { x: 250., y: 180. },
    };
    let last = editor.doc.point_parameter_iteration_labels(7).unwrap()[4].0;
    assert_eq!(
        editor.hit(&vp, vp.screen(last) + egui::vec2(17., 0.)),
        Some(7)
    );
    editor.doc.set_parameter_value(depth_parameter, 2.).unwrap();
    assert_ne!(
        editor.hit(&vp, vp.screen(last) + egui::vec2(17., 0.)),
        Some(7)
    );
    assert_eq!(
        editor
            .doc
            .point_parameter_iteration_labels(7)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        editor.doc.selected.iter().copied().collect::<Vec<_>>(),
        vec![7]
    );
}

#[test]
fn calculated_marked_angle_and_depth_create_regular_polygon() {
    let mut document = Document::new();
    let n = document.add_named_parameter("n", 5., (10, 5)).unwrap();
    let center = document.add(Kind::Free(Point { x: 200., y: 160. }));
    let depth = document
        .add_parameter_calculation(n, "n-1", (10, 35))
        .unwrap();
    let seed = document.add(Kind::Free(Point { x: 250., y: 160. }));
    let angle = document
        .add_parameter_calculation(n, "360°/n", (10, 65))
        .unwrap();
    let mut editor = Editor::new(document);
    editor.select_only(angle);
    editor.mark_angle_selected();
    assert_eq!(editor.marked_angle, Some(MarkedAngle::Value(angle)));
    editor.select_only(seed);
    editor.selection_order.push(center);
    editor.doc.selected.insert(center);
    editor.rotate_selected();
    let image = editor.doc.objects.len();
    assert_eq!(
        editor.doc.objects[image - 1].kind,
        Kind::CalculatedAngleRotatedPoint(seed, center, angle)
    );
    let segment = editor.doc.add(Kind::Segment(seed, image));
    editor.doc.selected.clear();
    editor.doc.selected.extend([depth, seed, image, segment]);
    editor.selection_order = vec![segment, depth, image, seed];
    editor.iterate_selected(4);
    let iteration = 8;
    assert_eq!(
        editor.doc.objects[iteration - 1].kind,
        Kind::SegmentIteration(seed, image, segment, IterationDepth::Value(depth))
    );
    editor.doc.set_parameter_value(n, 6.).unwrap();
    let lines = editor.doc.segment_iteration_images(iteration).unwrap();
    assert_eq!(lines.len(), 5);
    assert!(
        lines
            .last()
            .unwrap()
            .1
            .distance(editor.doc.position(seed).unwrap())
            < 1e-9
    );
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn selected_point_mapping_and_calculation_create_live_depth_iteration() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/point_calculated_rotation_live_depth_port.gsp"
    );
    let mut document = Document::from_bytes(bytes, None).unwrap();
    document.selected.insert(7);
    document.clear_selected().unwrap();
    let mut editor = Editor::new(document);
    editor.doc.selected.extend([3, 6, 5]);
    editor.selection_order = vec![5, 6, 3];
    editor.iterate_selected(4);
    let id = *editor.doc.selected.iter().next().unwrap();
    assert!(matches!(
        editor.doc.objects[id - 1].kind,
        Kind::PointIteration(3, 6, IterationDepth::Value(5))
    ));
    editor.doc.set_parameter_value(1, 6.).unwrap();
    assert_eq!(editor.doc.point_iteration_images(id).unwrap().len(), 5);
    let literal = editor
        .doc
        .add_calculation_expression("2+3", (10, 150))
        .unwrap();
    editor.doc.selected.clear();
    editor.doc.selected.extend([3, 6, literal]);
    editor.selection_order = vec![literal, 3, 6];
    editor.iterate_selected(4);
    let live = *editor.doc.selected.iter().next().unwrap();
    assert!(
        matches!(editor.doc.objects[live-1].kind,Kind::PointIteration(3,6,IterationDepth::Value(value)) if value==literal)
    );
    assert_eq!(editor.doc.point_iteration_images(live).unwrap().len(), 5);
    assert!(
        !Document::from_bytes(&editor.doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
}

#[test]
fn mapped_midpoint_iteration_point_binding_is_pickable_and_styles_survive_save() {
    let doc = Document::from_bytes(
        include_bytes!(
            "../../../../../tests/fixtures/mapped_midpoint_triangle_native_verified.gsp"
        ),
        None,
    )
    .unwrap();
    let mut editor = Editor::new(doc);
    let vp = Viewport {
        rect: egui::Rect::from_min_size(egui::pos2(20., 30.), egui::vec2(1000., 800.)),
        scale: 2.,
        center: Point { x: 750., y: 350. },
    };
    let point = editor.doc.mapped_point_binding_images(14).unwrap()[0];
    assert_eq!(editor.hit(&vp, vp.screen(point)), Some(14));
    editor.doc.selected.insert(14);
    editor
        .doc
        .set_selected_point_style(sketchpad_engine::style_records::PointStyle::Large)
        .unwrap();
    editor.doc.verify_roundtrip().unwrap();
    editor.doc.objects.edit(13).visible = false;
    assert_ne!(editor.hit(&vp, vp.screen(point)), Some(14));
}
