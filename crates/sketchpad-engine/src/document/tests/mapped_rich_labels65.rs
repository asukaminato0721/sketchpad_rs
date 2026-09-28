//! Shared rich-caption iteration through the ordinary scoped text resolver.
use super::*;

const FRACTION: &str = "<H<T5xMATH ><?x2><T3x + ></<T1x1><T1x2>>>";
fn compact(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}
fn images(doc: &Document, binding: usize, xs: &[f64]) -> Vec<String> {
    let images = doc.mapped_numeric_label_images(binding).unwrap();
    assert_eq!(images.len(), xs.len());
    images
        .into_iter()
        .zip(xs)
        .map(|((point, text), &x)| {
            assert_eq!(point, Point { x, y: 180. });
            text
        })
        .collect()
}
fn numeric(texts: &[String], expected: &[f64]) {
    assert_eq!(texts.len(), expected.len());
    for (text, value) in texts.iter().zip(expected) {
        let text = compact(text);
        let number = text
            .strip_prefix("MATH")
            .unwrap()
            .strip_suffix("+(1)/(2)")
            .unwrap();
        assert_eq!(number.parse::<f64>().unwrap(), *value, "{text}");
    }
}
fn authored(frozen: bool, undefined: bool) -> (Document, usize, usize, usize) {
    let mut doc = Document::new();
    let point = doc.add(Kind::Free(Point { x: 200., y: 180. }));
    let start = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let end = doc.add(Kind::Free(Point { x: 100., y: 0. }));
    let next = doc.add(Kind::VectorTranslatedPoint(point, start, end));
    let q = doc.add_named_parameter("q", 1., (0, 0)).unwrap();
    let nextq = doc.add_parameter_calculation(q, "q+1", (0, 0)).unwrap();
    let value = doc
        .add_parameter_calculation(q, if undefined { "1/(q-3)" } else { "q*10" }, (0, 0))
        .unwrap();
    let label = doc
        .add_point_referenced_text(point, value, FRACTION)
        .unwrap();
    let depth = doc.add_named_parameter("n", 3., (0, 0)).unwrap();
    let mut carried = vec![next, nextq];
    if !frozen {
        carried.push(value);
    }
    carried.push(label);
    let root = doc
        .add_mapped_point_iteration_with_bindings(
            &[point, q],
            &[vec![next, nextq]],
            &carried,
            &[next, label],
            IterationDepth::Value(depth),
        )
        .unwrap();
    assert!(matches!(
        doc.objects[root - 1].kind,
        Kind::MappedPointIteration { .. }
    ));
    (doc, root + 2, label, q)
}

#[test]
fn native_rich_mapped_captions_keep_exact_bytes_positions_and_shared_roles() {
    for state in ["numeric_live", "numeric_frozen", "plain", "function"] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/mapped_rich_labels65_{state}_native_saved.gsp"
        ))
        .unwrap();
        let doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view, "{state}");
        let binding = if state.starts_with("numeric") { 12 } else { 13 };
        let labels = images(&doc, binding, &[300., 400., 500.]);
        match state {
            "numeric_live" => numeric(&labels, &[20., 30., 40.]),
            "numeric_frozen" => numeric(&labels, &[10., 10., 10.]),
            "plain" => assert!(labels.iter().all(|s| compact(s) == "TEXTLIVE_TEXT")),
            "function" => assert!(labels.iter().all(|s| compact(s) == "FUNCTIONf(x)=x²")),
            _ => unreachable!(),
        }
        assert_eq!(doc.encode().unwrap(), bytes, "{state}");
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn mapped_rich_frozen_content_tracks_original_edits_without_per_step_substitution() {
    let bytes =
        fs::read("../../tests/fixtures/mapped_rich_labels65_numeric_frozen_native_saved.gsp")
            .unwrap();
    let mut doc = Document::from_bytes(&bytes, None).unwrap();
    doc.set_parameter_value(5, 2.).unwrap();
    numeric(&images(&doc, 12, &[300., 400., 500.]), &[20.; 3]);
    doc.checkpoint();
    assert!(doc.move_point(1, Point { x: 210., y: 180. }));
    numeric(&images(&doc, 12, &[310., 410., 510.]), &[20.; 3]);
    let saved = doc.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    numeric(&images(&reopened, 12, &[310., 410., 510.]), &[20.; 3]);
    assert_eq!(reopened.encode().unwrap(), saved);
    doc.undo();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), bytes);
}

#[test]
fn public_rich_binding_authoring_fonts_visibility_delete_and_undo_share_the_source() {
    let (mut doc, binding, label, q) = authored(false, false);
    numeric(
        &images(&doc, binding, &[300., 400., 500.]),
        &[20., 30., 40.],
    );
    doc.verify_roundtrip().unwrap();
    let original = doc.encode().unwrap();
    doc.selected.insert(label);
    let font = doc.text_font_state(label).unwrap();
    doc.toggle_selected_text_style(TextStyleCommand::Bold)
        .unwrap();
    assert_eq!(doc.text_font_state(label).unwrap().bold, !font.bold);
    assert_eq!(
        doc.numeric_label_font_state(binding),
        doc.text_font_state(label)
    );
    let styled = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(
        styled.numeric_label_font_state(binding),
        doc.numeric_label_font_state(binding)
    );
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
    doc.selected.clear();
    doc.selected.insert(binding);
    doc.set_selected_visibility(false);
    assert!(!doc.objects[binding - 1].visible);
    numeric(
        &images(&doc, binding, &[300., 400., 500.]),
        &[20., 30., 40.],
    );
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
    doc.selected.clear();
    doc.selected.insert(q);
    doc.clear_selected().unwrap();
    assert!(doc.objects[label - 1].deleted);
    assert!(doc.objects[binding - 1].deleted);
    assert!(doc.mapped_numeric_label_images(binding).is_none());
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
}

#[test]
fn undefined_rich_numeric_content_keeps_anchor_other_output_and_fraction_then_recovers() {
    let (mut doc, binding, _, q) = authored(false, true);
    let labels = images(&doc, binding, &[300., 400., 500.]);
    assert!(compact(&labels[1]).contains("undefined"));
    assert!(compact(&labels[1]).ends_with("+(1)/(2)"));
    assert_eq!(
        doc.mapped_point_binding_images(binding - 1).unwrap().len(),
        3
    );
    doc.verify_roundtrip().unwrap();
    let original = doc.encode().unwrap();
    doc.set_parameter_value(q, 5.).unwrap();
    let labels = images(&doc, binding, &[300., 400., 500.]);
    assert!(labels.iter().all(|s| !s.contains("undefined")));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(images(&reopened, binding, &[300., 400., 500.]), labels);
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
}

#[test]
fn unsupported_reference_roles_do_not_enter_a_value_iteration_scope() {
    let (mut doc, _, _, q) = authored(false, false);
    for markup in ["<?xA0002>", "<?xB0002>", "<?xE0002>"] {
        let label = doc.add_point_referenced_text(1, q, markup).unwrap();
        let count = doc.objects.len();
        assert!(
            doc.add_mapped_point_iteration_with_bindings(
                &[1, q],
                &[vec![4, 6]],
                &[4, 6, label],
                &[label],
                IterationDepth::Fixed(3),
            )
            .is_err()
        );
        assert_eq!(doc.objects.len(), count);
    }
    doc.verify_roundtrip().unwrap();
}
