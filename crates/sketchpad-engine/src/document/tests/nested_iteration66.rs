//! Nested terminal scopes with explicit live/frozen descriptor boundaries.
use super::*;
fn fixture(prefix: &str, state: &str) -> (Vec<u8>, Document) {
    let bytes = fs::read(format!(
        "../../tests/fixtures/{prefix}_{state}_native_saved.gsp"
    ))
    .unwrap();
    let doc = Document::from_bytes(&bytes, None).unwrap();
    (bytes, doc)
}
fn near(actual: Point, x: f64) {
    assert!((actual.x - x).abs() < 1e-8, "{actual:?} versus{x}");
    assert!((actual.y - 180.).abs() < 1e-8);
}
fn check(doc: &Document, binding: usize, endpoint: usize, xs: &[f64], last: Option<f64>) {
    let images = doc.mapped_point_binding_images(binding).unwrap();
    assert_eq!(images.len(), xs.len());
    for (point, x) in images.into_iter().zip(xs) {
        near(point, *x);
    }
    match last {
        Some(x) => near(doc.position(endpoint).unwrap(), x),
        None => assert!(doc.position(endpoint).is_none()),
    }
}
#[test]
fn retained_native_nested_scopes_and_terminal_undefined_match_independent_geometry() {
    for (state, binding, endpoint, xs, last) in [
        (
            "scalar_live",
            16,
            17,
            vec![233.1, 272.8, 319.7],
            Some(319.7),
        ),
        ("scalar_frozen", 16, 17, vec![200.; 3], Some(200.)),
        (
            "scalar_edited",
            16,
            17,
            vec![315.3125, 385.7205, 473.6405],
            Some(473.6405),
        ),
        ("scalar_undefined", 16, 17, vec![101.5625, 200.], None),
        ("point_live", 18, 19, vec![420., 520., 620.], Some(620.)),
        ("point_frozen", 18, 19, vec![320.; 3], Some(320.)),
        ("scalar_frozen_isolated", 16, 17, vec![200.; 3], Some(200.)),
        ("point_frozen_isolated", 18, 19, vec![320.; 3], Some(320.)),
    ] {
        let (bytes, doc) = fixture("nested_iteration65", state);
        assert!(!doc.partial_view, "{state}");
        check(&doc, binding, endpoint, &xs, last);
        assert_eq!(doc.encode().unwrap(), bytes, "{state}");
        doc.verify_roundtrip().unwrap();
    }
}
#[test]
fn nested_descriptor_capture_is_distinct_from_binding_and_incomplete_local_ancestors() {
    for (state, xs) in [
        ("scalar_inner_only", vec![233.1, 272.8, 319.7]),
        ("scalar_binder_only", vec![200.; 3]),
    ] {
        let (bytes, doc) = fixture("nested_capture65", state);
        assert!(!doc.partial_view, "{state}");
        check(&doc, 16, 17, &xs, xs.last().copied());
        assert_eq!(doc.encode().unwrap(), bytes);
    }
    for state in ["scalar_omit_factor", "scalar_omit_image"] {
        let (bytes, doc) = fixture("nested_capture65", state);
        assert!(
            doc.partial_view,
            "Unproved capture phase must remain opaque: {state}"
        );
        assert!(!matches!(
            doc.objects[14].kind,
            Kind::MappedPointIteration { .. }
        ));
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}
#[test]
fn caller_scalar_external_inputs_and_inner_depth_remain_live_after_reopen_and_undo() {
    let (bytes, mut doc) = fixture("nested_iteration65", "scalar_live");
    doc.set_parameter_value(5, 1.1).unwrap();
    check(&doc, 16, 17, &[272.8, 319.7, 374.4], Some(374.4));
    doc.set_parameter_value(10, 3.).unwrap();
    let xs: Vec<_> = [1.2_f64, 1.3, 1.4]
        .into_iter()
        .map(|f| 100. + 100. * f.powi(4))
        .collect();
    check(&doc, 16, 17, &xs, xs.last().copied());
    let saved = doc.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    check(&reopened, 16, 17, &xs, xs.last().copied());
    assert_eq!(reopened.encode().unwrap(), saved);
    doc.undo();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), bytes);
    let (_, mut frozen) = fixture("nested_iteration65", "scalar_frozen");
    frozen.set_parameter_value(3, 2.).unwrap();
    check(&frozen, 16, 17, &[272.8; 3], Some(272.8));
}
#[test]
fn same_id_inner_point_replacement_shadows_outer_only_for_the_local_execution() {
    let (bytes, mut doc) = fixture("nested_iteration65", "point_live");
    doc.checkpoint();
    assert!(doc.move_point(2, Point { x: 210., y: 180. }));
    check(&doc, 18, 19, &[430., 530., 630.], Some(630.));
    near(doc.position(2).unwrap(), 210.);
    near(doc.position(13).unwrap(), 330.);
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), bytes);
}
#[test]
fn public_nested_authoring_hidden_original_and_dependency_delete_undo_are_serializable() {
    let mut doc = Document::new();
    let center = doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let seed = doc.add(Kind::Free(Point { x: 200., y: 180. }));
    let q = doc.add_named_parameter("q", 0., (0, 0)).unwrap();
    let nextq = doc.add_parameter_calculation(q, "q+1", (0, 0)).unwrap();
    let a = doc.add_named_parameter("a", 1., (0, 0)).unwrap();
    let factor = doc
        .add_multi_calculation(&[a, q], "v1+v2*0.1", (0, 0))
        .unwrap();
    let image = doc
        .add_calculated_ratio_dilation(seed, center, factor)
        .unwrap();
    let inner = doc
        .add_mapped_point_iteration_with_bindings(
            &[seed],
            &[vec![image]],
            &[factor, image],
            &[image],
            IterationDepth::Fixed(2),
        )
        .unwrap();
    let alias = doc.add_iteration_point_alias(inner + 1).unwrap();
    let outer = doc
        .add_mapped_point_iteration_with_bindings(
            &[q],
            &[vec![nextq]],
            &[nextq, factor, image, inner, inner + 1, alias],
            &[alias],
            IterationDepth::Fixed(3),
        )
        .unwrap();
    let terminal = doc.add_iteration_point_alias(outer + 1).unwrap();
    check(
        &doc,
        outer + 1,
        terminal,
        &[233.1, 272.8, 319.7],
        Some(319.7),
    );
    doc.verify_roundtrip().unwrap();
    let original = doc.encode().unwrap();
    doc.selected.insert(alias);
    doc.set_selected_visibility(false);
    check(
        &doc,
        outer + 1,
        terminal,
        &[233.1, 272.8, 319.7],
        Some(319.7),
    );
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
    doc.selected.clear();
    doc.selected.insert(a);
    doc.clear_selected().unwrap();
    assert!(doc.objects[terminal - 1].deleted);
    assert!(doc.position(terminal).is_none());
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
}

#[test]
fn scoped_mapping_reads_all_members_in_the_selected_entry_environment() {
    let (_, original) = fixture("nested_iteration65", "scalar_live");
    let mut caller = original.clone();
    caller.freeze_evaluation_inputs(&original, &[]).unwrap();
    caller.set_evaluation_number(3, 2.).unwrap();
    let images = caller
        .mapped_scoped_outputs(9, 11, false, |sample, id| sample.position(id))
        .unwrap();
    assert_eq!(images.len(), 2);
    near(images[0].unwrap(), 244.);
    near(images[1].unwrap(), 272.8);
    assert_eq!(caller.numeric_value(3), Some(2.));
    assert_eq!(original.numeric_value(3), Some(0.));
    near(caller.position(2).unwrap(), 200.);

    let mut frozen = original.clone();
    frozen.freeze_evaluation_inputs(&original, &[11]).unwrap();
    frozen.set_evaluation_number(3, 2.).unwrap();
    let images = frozen
        .mapped_scoped_outputs(9, 11, false, |sample, id| sample.position(id))
        .unwrap();
    assert_eq!(images.len(), 2);
    for image in images {
        near(image.unwrap(), 200.);
    }
    assert_eq!(frozen.numeric_value(3), Some(2.));
}
