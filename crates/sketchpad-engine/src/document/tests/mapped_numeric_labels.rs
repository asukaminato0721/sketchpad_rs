use super::*;

fn base(frozen: bool) -> (Document, usize) {
    base_with_prefix(frozen, false)
}
fn base_with_prefix(frozen: bool, prefix: bool) -> (Document, usize) {
    let mut d = Document::new();
    if prefix {
        d.add(Kind::Free(Point { x: 0., y: 0. }));
    }
    let id = |n| n + usize::from(prefix);
    for (x, y) in [(100., 180.), (0., 180.), (40., 180.)] {
        d.add(Kind::Free(Point { x, y }));
    }
    d.add(Kind::VectorTranslatedPoint(id(1), id(2), id(3)));
    d.add_named_parameter("q", 0.5, (10, 10)).unwrap();
    d.add_parameter_calculation(id(5), "q+0.25", (10, 10))
        .unwrap();
    d.add_parameter_calculation(id(5), "q*10", (10, 10))
        .unwrap();
    d.add_point_numeric_label(id(4), id(7)).unwrap();
    d.add_named_parameter("n", 3., (10, 10)).unwrap();
    let carried: Vec<_> = if frozen {
        vec![4, 6, 8]
    } else {
        vec![4, 6, 7, 8]
    }
    .into_iter()
    .map(id)
    .collect();
    let parent = d
        .add_mapped_point_iteration_with_bindings(
            &[id(1), id(5)],
            &[vec![id(4), id(6)]],
            &carried,
            &[id(4), id(8)],
            IterationDepth::Value(id(9)),
        )
        .unwrap();
    assert_eq!(parent, id(10));
    (d, parent + 2)
}
fn check(d: &Document, id: usize, xs: &[f64], values: &[f64]) {
    let images = d.mapped_numeric_label_images(id).unwrap();
    assert_eq!(images.len(), xs.len());
    for ((point, text), (&x, &value)) in images.iter().zip(xs.iter().zip(values)) {
        assert_eq!(*point, Point { x, y: 180. });
        assert!(
            (text.parse::<f64>().unwrap() - value).abs() < 1e-9,
            "{text} != {value}"
        );
    }
}

#[test]
fn mapped_numeric_labels_replace_point_and_scalar_seeds_in_the_same_scope() {
    let (mut d, label) = base(false);
    check(&d, label, &[180., 220., 260.], &[7.5, 10., 12.5]);
    assert_eq!(d.position(4), Some(Point { x: 140., y: 180. }));
    assert_eq!(d.numeric_value(7), Some(5.));
    let bytes = d.encode().unwrap();
    d = Document::from_bytes(&bytes, None).unwrap();
    assert!(!d.partial_view);
    d.set_parameter_value(5, 1.).unwrap();
    d.set_parameter_value(9, 2.).unwrap();
    assert!(d.move_point(1, Point { x: 110., y: 180. }));
    check(&d, label, &[190., 230.], &[12.5, 15.]);
    d.verify_roundtrip().unwrap();
}

#[test]
fn mapped_numeric_label_omitted_numeric_host_stays_frozen_and_finalonly_is_live() {
    let (mut d, label) = base(true);
    check(&d, label, &[180., 220., 260.], &[5., 5., 5.]);
    d.set_parameter_value(5, 0.75).unwrap();
    check(&d, label, &[180., 220., 260.], &[7.5, 7.5, 7.5]);
    if let Kind::MappedPointIteration { final_only, .. } = &mut d.objects[9].kind {
        *final_only = true
    } else {
        panic!()
    }
    check(&d, label, &[260.], &[7.5]);
    d.verify_roundtrip().unwrap();
    let bytes = d.encode().unwrap();
    let d = Document::from_bytes(&bytes, None).unwrap();
    check(&d, label, &[260.], &[7.5]);
}

#[test]
fn mapped_numeric_labels_do_not_hide_other_outputs_when_value_is_undefined() {
    let (mut d, label) = base(false);
    let donor = d.add_parameter_calculation(5, "1/(q-1)", (10, 10)).unwrap();
    let formula = d.objects[donor - 1].kind.clone();
    d.objects[6].kind = formula;
    let labels = d.mapped_numeric_label_images(label).unwrap();
    assert_eq!(labels.len(), 3);
    assert_eq!(labels[1].1, "undefined");
    assert_eq!(d.mapped_point_binding_images(11).unwrap().len(), 3);
}

#[test]
fn mapped_numeric_labels_keep_dependency_remapping_and_undo_exact() {
    let (mut d, label_with_prefix) = base_with_prefix(false, true);
    let label = label_with_prefix - 1;
    let unused = 1;
    d.verify_roundtrip().unwrap();
    check(
        &d,
        label_with_prefix,
        &[180., 220., 260.],
        &[7.5, 10., 12.5],
    );
    d.selected.insert(unused);
    d.clear_selected().unwrap();
    let bytes = d.encode().unwrap();
    d = Document::from_bytes(&bytes, None).unwrap();
    check(&d, label, &[180., 220., 260.], &[7.5, 10., 12.5]);
    d.selected.insert(7);
    d.clear_selected().unwrap();
    assert!(d.objects[label - 1].deleted);
    d.verify_roundtrip().unwrap();
    d.undo();
    assert_eq!(d.encode().unwrap(), bytes);
    check(&d, label, &[180., 220., 260.], &[7.5, 10., 12.5]);
}

#[test]
fn retained_native_general_mapped_numeric_labels_match_values_positions_and_bytes() {
    for state in [
        "baseline",
        "edited",
        "frozen",
        "spaced_baseline",
        "spaced_edited",
        "spaced_frozen",
        "api_baseline",
        "api_edited",
        "api_frozen",
    ] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/mapped_numeric_label51_{state}_native_verified.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert!(!d.partial_view, "{state}");
        assert!(matches!(
            d.objects[9].kind,
            Kind::MappedPointIteration { .. }
        ));
        let spaced = state.starts_with("spaced_") || state.starts_with("api_");
        let (xs, values) = if state.ends_with("edited") {
            (
                if spaced {
                    vec![310., 410.]
                } else {
                    vec![190., 230.]
                },
                vec![12.5, 15.],
            )
        } else {
            (
                if spaced {
                    vec![300., 400., 500.]
                } else {
                    vec![180., 220., 260.]
                },
                if state.ends_with("frozen") {
                    vec![5.; 3]
                } else {
                    vec![7.5, 10., 12.5]
                },
            )
        };
        check(&d, 12, &xs, &values);
        assert_eq!(d.mapped_point_binding_images(11).unwrap().len(), xs.len());
        assert_eq!(d.encode().unwrap(), bytes, "{state}");
        d.verify_roundtrip().unwrap();
    }
}

#[test]
fn generic_label_authoring_preserves_exact_legacy_point_parameter_packet_roles() {
    let mut d = Document::new();
    for (x, y) in [(100., 180.), (0., 180.), (40., 180.)] {
        d.add(Kind::Free(Point { x, y }));
    }
    d.add(Kind::VectorTranslatedPoint(1, 2, 3));
    d.add_named_parameter("q", 0.5, (10, 10)).unwrap();
    d.add_parameter_calculation(5, "q+0.25", (10, 10)).unwrap();
    d.add_point_numeric_label(4, 6).unwrap();
    d.add_named_parameter("n", 3., (10, 10)).unwrap();
    let parent = d
        .add_mapped_point_iteration_with_bindings(
            &[1, 5],
            &[vec![4, 6]],
            &[6, 4, 7],
            &[4, 7],
            IterationDepth::Value(8),
        )
        .unwrap();
    assert!(matches!(
        d.objects[parent - 1].kind,
        Kind::PointParameterIteration(1, 5, 4, 6, 7, _)
    ));
    check(&d, parent + 2, &[180., 220., 260.], &[1., 1.25, 1.5]);
    assert_eq!(d.mapped_point_binding_images(parent + 1).unwrap().len(), 3);
    d.verify_roundtrip().unwrap();
    let bytes = d.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    check(&reopened, parent + 2, &[180., 220., 260.], &[1., 1.25, 1.5]);
    assert_eq!(reopened.encode().unwrap(), bytes);
}
