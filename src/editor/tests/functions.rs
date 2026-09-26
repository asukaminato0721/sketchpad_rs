use super::*;

#[test]
fn parameter_locus_construction_accepts_both_selection_orders() {
    for order in [vec![1, 9], vec![9, 1]] {
        let doc = Document::from_bytes(include_bytes!("../../../tests/fixtures/numeric_cood_trace_native.gsp"), None).unwrap();
        let mut editor = Editor::new(doc);
        editor.selection_order = order.clone();
        editor.doc.selected.extend(order);
        editor.construct_locus();
        assert!(matches!(editor.doc.objects.last().unwrap().kind, Kind::ParameterLocus(_, 0., 1., 500)));
        assert_eq!(editor.doc.objects.len(), 11);
        assert_eq!(editor.doc.selected.len(), 1);
        editor.doc.verify_roundtrip().unwrap();
    }
}
