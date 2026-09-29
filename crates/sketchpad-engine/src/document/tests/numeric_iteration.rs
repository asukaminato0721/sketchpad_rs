use super::*;

#[test]
fn point_numeric_label_updates_and_remaps_both_parents() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/point_numeric_label_native_verified.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(doc.objects[3].kind, Kind::PointNumericLabel(3, 2));
    assert_eq!(
        doc.point_numeric_label(4),
        Some((Point { x: 280., y: 180. }, "3.00".into()))
    );
    doc.set_parameter_value(1, 4.).unwrap();
    doc.move_point(3, Point { x: 340., y: 210. });
    assert_eq!(
        doc.point_numeric_label(4),
        Some((Point { x: 340., y: 210. }, "5.00".into()))
    );
    doc.selected.insert(4);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let cloned = *pasted.last().unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.point_numeric_label(cloned),
        Some((Point { x: 360., y: 230. }, "5.00".into()))
    );
    assert!(
        matches!(reopened.objects[cloned-1].kind,Kind::PointNumericLabel(point,value) if point!=3 && value!=2)
    );
    let mut cleared = Document::from_bytes(bytes, None).unwrap();
    cleared.selected.insert(1);
    assert_eq!(cleared.clear_selected().unwrap(), vec![1, 2, 4]);
    let reopened = Document::from_bytes(&cleared.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects.len(), 1);
    assert!(reopened.position(1).is_some());
    assert!(doc.add_point_numeric_label(2, 3).is_err());
}

#[test]
fn branching_polygon_native_paths_edits_copy_and_depth_parent_deletion() {
    let bytes = include_bytes!("../../../../../tests/fixtures/branch_polygon_iteration_native.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert!(
        matches!(&doc.objects[8].kind,Kind::BranchingPolygonIteration { seed:1,first:4,second:2,polygon:5,segments,depth:IterationDepth::Value(3) } if segments==&[6,7,8])
    );
    let offsets = doc.branch_iteration_offsets(9).unwrap();
    assert_eq!(offsets.len(), 14);
    assert_eq!(offsets.iter().map(|(_, paths)| paths).sum::<f64>(), 30.);
    assert_eq!(doc.polygon_iteration_images(10).unwrap().len(), 14);
    assert!(doc.point_iteration_images(9).is_none());
    assert!(matches!(
        doc.objects[9].visual_style,
        Some(VisualStyle::Polygon { opacity: 32767, .. })
    ));
    doc.set_parameter_value(3, 2.).unwrap();
    doc.move_point(1, Point { x: 80., y: 200. });
    let offsets = doc.branch_iteration_offsets(9).unwrap();
    assert_eq!(offsets.len(), 5);
    assert_eq!(offsets.iter().map(|(_, paths)| paths).sum::<f64>(), 6.);
    assert_eq!(offsets[3].1, 2.);
    let polygons = doc.polygon_iteration_images(10).unwrap();
    let base = doc.polygon_points(5).unwrap();
    for (i, (points, _)) in polygons.iter().enumerate() {
        for (point, source) in points.iter().zip(&base) {
            assert!((point.x - source.x - offsets[i].0.x).abs() < 1e-10);
            assert!((point.y - source.y - offsets[i].0.y).abs() < 1e-10);
        }
        let sides = doc.branch_segment_iteration_images(11).unwrap();
        assert!(points[0].distance(sides[i].0) < 1e-10);
        assert!(points[1].distance(sides[i].1) < 1e-10);
    }
    doc.selected.insert(10);
    let clipboard = doc.copy_selected().unwrap();
    assert_eq!(clipboard.len(), 13);
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = pasted
        .into_iter()
        .find(|id| {
            matches!(
                doc.objects[*id - 1].kind,
                Kind::BranchingPolygonIteration { .. }
            )
        })
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.branch_iteration_offsets(copied),
        doc.branch_iteration_offsets(9)
    );
    assert_eq!(
        reopened
            .objects
            .iter()
            .filter(|o| matches!(o.kind,Kind::IterationBinding(_,parent) if parent==copied))
            .count(),
        4
    );
    assert!(
        matches!(reopened.objects[copied-1].kind,Kind::BranchingPolygonIteration { seed,first,second,polygon,depth:IterationDepth::Value(n),.. } if seed!=1 && first!=4 && second!=2 && polygon!=5 && n!=3)
    );
    let mut cleared = Document::from_bytes(bytes, None).unwrap();
    cleared.selected.insert(3);
    assert_eq!(
        cleared.clear_selected().unwrap(),
        vec![3, 9, 10, 11, 12, 13]
    );
    let reopened = Document::from_bytes(&cleared.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects.len(), 7);
}

#[test]
fn branching_polygon_fixed_depth_and_mapping_validation() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/branch_polygon_iteration_fixed_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert!(matches!(
        doc.objects[7].kind,
        Kind::BranchingPolygonIteration {
            depth: IterationDepth::Fixed(2),
            ..
        }
    ));
    assert_eq!(doc.branch_iteration_offsets(8).unwrap().len(), 5);
    assert!(
        doc.add_branching_polygon_iteration(1, 2, 2, 4, &[], IterationDepth::Fixed(2))
            .is_err()
    );
    assert!(
        doc.add_branching_polygon_iteration(1, 2, 3, 4, &[5, 5], IterationDepth::Fixed(2))
            .is_err()
    );
    assert!(
        doc.add_branching_polygon_iteration(1, 2, 3, 4, &[], IterationDepth::Fixed(0))
            .is_err()
    );
    let id = doc
        .add_branching_polygon_iteration(1, 2, 3, 4, &[], IterationDepth::Fixed(128))
        .unwrap();
    assert_eq!(doc.branch_iteration_offsets(id).unwrap().len(), 8384);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened
            .objects
            .iter()
            .filter(|o| matches!(o.kind,Kind::IterationBinding(_,parent) if parent==id))
            .count(),
        1
    );
    doc.selected.clear();
    doc.selected.insert(1);
    doc.clear_selected().unwrap();
    assert!(
        Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .objects
            .is_empty()
    );
}

#[test]
fn numeric_iteration_formats_precision_without_rounding_its_state() {
    let native_bytes = include_bytes!(
        "../../../../../tests/fixtures/point_parameter_iteration_integer_precision_native_verified.gsp"
    );
    let native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(native.point_numeric_label(5).unwrap().1, "1");
    assert_eq!(
        native
            .point_parameter_iteration_labels(7)
            .unwrap()
            .into_iter()
            .map(|(_, text)| text)
            .collect::<Vec<_>>(),
        ["1", "2", "2", "3"]
    );
    let mut doc = Document::new();
    let seed = doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let image = doc.add(Kind::PolarTranslatedPoint(seed, 0., 37.79527559055118, 1.));
    let parameter = doc.add_named_parameter("a", 0., (10, 5)).unwrap();
    let formula =
        ParameterCalculationGroup::from_expression(parameter as u32, "a", "a+0.5", (10, 35))
            .unwrap();
    let mut bytes = formula.as_bytes().to_vec();
    bytes[88..90].copy_from_slice(&0u16.to_le_bytes());
    let formula = ParameterCalculationGroup::parse(&bytes).unwrap();
    let calculation = doc.add(Kind::ParameterCalculation(parameter, formula));
    let label = doc.add_point_numeric_label(image, calculation).unwrap();
    let iteration = doc
        .add_point_parameter_iteration(
            seed,
            parameter,
            image,
            calculation,
            label,
            IterationDepth::Fixed(3),
        )
        .unwrap();
    assert_eq!(doc.parameter_calculation_value(calculation), Some(0.5));
    assert_eq!(doc.point_numeric_label(label).unwrap().1, "1");
    assert_eq!(
        doc.point_parameter_iteration_labels(iteration)
            .unwrap()
            .into_iter()
            .map(|(_, text)| text)
            .collect::<Vec<_>>(),
        ["1", "2", "2"]
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.point_parameter_iteration_labels(iteration),
        doc.point_parameter_iteration_labels(iteration)
    );
}

#[test]
fn numeric_iteration_parameter_depth_native_edit_copy_and_deletion() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/point_parameter_iteration_depth_parameter_native.gsp"
    );
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(
        doc.objects[6].kind,
        Kind::PointParameterIteration(1, 3, 5, 4, 6, IterationDepth::Value(2))
    );
    assert_eq!(doc.point_numeric_label(6).unwrap().1, "1");
    assert!(crate::render::number_lines_for_object(&doc, 4).unwrap().1[0].ends_with(" = 1"));
    assert_eq!(doc.point_parameter_iteration_labels(7).unwrap().len(), 5);
    let values = |doc: &Document, id| {
        doc.point_parameter_iteration_labels(id)
            .unwrap()
            .into_iter()
            .map(|(_, text)| text)
            .collect::<Vec<_>>()
    };
    assert_eq!(values(&doc, 7), ["2", "3", "4", "5", "6"]);
    doc.set_parameter_value(2, 3.).unwrap();
    doc.set_parameter_value(3, 2.).unwrap();
    doc.move_point(1, Point { x: 100., y: 180. });
    assert_eq!(values(&doc, 7), ["4", "5", "6"]);
    for (i, (point, _)) in doc
        .point_parameter_iteration_labels(7)
        .unwrap()
        .into_iter()
        .enumerate()
    {
        assert!((point.x - (100. + (i + 2) as f64 * 37.79527559055118)).abs() < 1e-8);
        assert_eq!(point.y, 180.);
    }
    doc.selected.insert(7);
    let clipboard = doc.copy_selected().unwrap();
    assert_eq!(clipboard.len(), 10);
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = pasted
        .into_iter()
        .find(|id| matches!(doc.objects[*id - 1].kind, Kind::PointParameterIteration(..)))
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(values(&reopened, copied), ["4", "5", "6"]);
    assert!(
        matches!(reopened.objects[copied - 1].kind, Kind::PointParameterIteration(seed, parameter, image, calculation, label, IterationDepth::Value(n)) if seed != 1 && parameter != 3 && image != 5 && calculation != 4 && label != 6 && n != 2)
    );
    assert_eq!(
        reopened
            .objects
            .iter()
            .filter(|o| matches!(o.kind, Kind::IterationBinding(_, parent) if parent == copied))
            .count(),
        2
    );
    assert!(
        reopened.objects.iter().any(|o| !o.visible
            && matches!(o.kind, Kind::IterationValues(parent, _) if parent == copied))
    );
    let mut cleared = Document::from_bytes(bytes, None).unwrap();
    cleared.selected.insert(2);
    assert_eq!(cleared.clear_selected().unwrap(), vec![2, 7, 8, 9, 10]);
    let reopened = Document::from_bytes(&cleared.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects.len(), 5);
    assert_eq!(reopened.point_numeric_label(5).unwrap().1, "1");
    assert!(
        doc.add_point_parameter_iteration(1, 3, 5, 4, 6, IterationDepth::Value(3))
            .is_err()
    );
    doc.set_parameter_value(2, 2.5).unwrap();
    doc.set_parameter_value(2, 0.).unwrap();
    assert!(
        doc.add_point_parameter_iteration(1, 3, 5, 4, 6, IterationDepth::Value(2))
            .is_err()
    );
    assert!(doc.point_parameter_iteration_labels(7).is_none());
    assert!(Document::from_bytes(&doc.encode().unwrap(), None).is_ok());
}

#[test]
fn point_parameter_iteration_import_edits_copy_and_deletion() {
    let native_bytes =
        include_bytes!("../../../../../tests/fixtures/point_parameter_iteration_native.gsp");
    let mut native = Document::from_bytes(native_bytes, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), native_bytes);
    assert_eq!(
        native.objects[6].kind,
        Kind::PointParameterIteration(4, 2, 5, 3, 6, IterationDepth::Fixed(3))
    );
    assert_eq!(native.objects[9].kind, Kind::IterationValues(7, vec![3]));
    assert!(!native.objects[9].visible);
    let values = |doc: &Document, id| {
        doc.point_parameter_iteration_labels(id)
            .unwrap()
            .into_iter()
            .map(|(_, text)| text)
            .collect::<Vec<_>>()
    };
    assert_eq!(values(&native, 7), ["2", "3", "4"]);
    native.set_parameter_value(2, 2.).unwrap();
    native.move_point(1, Point { x: 250., y: 180. });
    native.move_point(4, Point { x: 330., y: 180. });
    assert_eq!(values(&native, 7), ["4", "5", "6"]);
    for (i, (point, _)) in native
        .point_parameter_iteration_labels(7)
        .unwrap()
        .iter()
        .enumerate()
    {
        let expected = rotate_point_about(
            native.position(4).unwrap(),
            native.position(1).unwrap(),
            (30. * (i + 2) as f64).to_radians(),
        )
        .unwrap();
        assert!(point.distance(expected) < 1e-10);
    }
    native.selected.insert(7);
    let clipboard = native.copy_selected().unwrap();
    let pasted = native.paste(&clipboard).unwrap();
    let copied = pasted
        .into_iter()
        .find(|id| {
            matches!(
                native.objects[*id - 1].kind,
                Kind::PointParameterIteration(..)
            )
        })
        .unwrap();
    let reopened = Document::from_bytes(&native.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(values(&reopened, copied), ["4", "5", "6"]);
    assert_eq!(
        reopened
            .objects
            .iter()
            .filter(|o| matches!(o.kind, Kind::IterationBinding(_, parent) if parent == copied))
            .count(),
        2
    );
    assert!(
        reopened.objects.iter().any(|o| !o.visible
            && matches!(o.kind, Kind::IterationValues(parent, _) if parent == copied))
    );
    assert!(
        matches!(reopened.objects[copied - 1].kind, Kind::PointParameterIteration(seed, parameter, image, calculation, label, IterationDepth::Fixed(3)) if seed != 4 && parameter != 2 && image != 5 && calculation != 3 && label != 6)
    );
    let mut cleared = Document::from_bytes(native_bytes, None).unwrap();
    cleared.selected.insert(2);
    assert_eq!(
        cleared.clear_selected().unwrap(),
        vec![2, 3, 6, 7, 8, 9, 10]
    );
    let reopened = Document::from_bytes(&cleared.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects.len(), 3);
}

#[test]
fn point_parameter_iteration_creation_roundtrip_and_invalid_mapping() {
    let bytes = include_bytes!("../../../../../tests/fixtures/point_parameter_iteration_port.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert!(
        doc.add_point_parameter_iteration(1, 4, 3, 5, 6, IterationDepth::Fixed(0))
            .is_err()
    );
    assert!(
        doc.add_point_parameter_iteration(2, 4, 3, 5, 6, IterationDepth::Fixed(3))
            .is_err()
    );
    let wrong_label = doc.add_point_numeric_label(1, 5).unwrap();
    assert!(
        doc.add_point_parameter_iteration(1, 4, 3, 5, wrong_label, IterationDepth::Fixed(3))
            .is_err()
    );
    let id = doc
        .add_point_parameter_iteration(1, 4, 3, 5, 6, IterationDepth::Fixed(2))
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.point_parameter_iteration_labels(id).unwrap().len(),
        2
    );
    doc.objects[id - 1].kind =
        Kind::PointParameterIteration(1, 4, 3, 5, 6, IterationDepth::Fixed(129));
    assert!(doc.can_save().is_err());
    let edited = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/point_parameter_iteration_edited_port.gsp"),
        None,
    )
    .unwrap();
    assert!(!edited.partial_view);
    assert_eq!(
        edited
            .point_parameter_iteration_labels(7)
            .unwrap()
            .into_iter()
            .map(|(_, text)| text)
            .collect::<Vec<_>>(),
        ["4.00", "5.00", "6.00"]
    );
}
