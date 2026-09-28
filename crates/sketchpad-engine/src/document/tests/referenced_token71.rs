use super::*;

fn text(doc: &Document, id: usize) -> String {
    doc.referenced_text_runs(id)
        .unwrap()
        .into_iter()
        .flatten()
        .map(|run| run.text)
        .collect()
}

#[test]
fn original_hidden_pi_caption_shares_live_value_with_plain_neighbor() {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../gsp-rs/tests/Samples/个人专栏/向忠作品/函数图象描点法作图演示平台.gsp"
    ))
    .unwrap();
    let doc = Document::from_bytes(&bytes, None).unwrap();
    assert!(doc.partial_view); // Other unsupported structures still guard edits.
    assert!(matches!(doc.objects[215].kind, Kind::ReferencedText { .. }));
    assert_eq!(doc.objects[215].visible, false);
    assert_eq!(text(&doc, 216), format!("{}π", text(&doc, 217)));
    assert_eq!(doc.encode().unwrap(), bytes);
}

#[test]
fn native_subscript_caption_uses_live_parent_value_and_precision() {
    let original = include_bytes!(
        "../../../../../tests/fixtures/referenced_token71_dash_original_native_saved.gsp"
    );
    let doc = Document::from_bytes(original, None).unwrap();
    assert!(matches!(doc.objects[8].kind, Kind::ReferencedText { .. }));
    assert_eq!(text(&doc, 9), "A₃");
    assert_eq!(doc.encode().unwrap(), original);

    // Native changed only the parent's wire reference, leaving cached "3"
    // intact. Its painted label uses calculation 6's current value/precision.
    let rebound = include_bytes!(
        "../../../../../tests/fixtures/referenced_token71_dash_rebind6_native_saved.gsp"
    );
    let doc = Document::from_bytes(rebound, None).unwrap();
    assert_eq!(text(&doc, 9), "A₅.₀₀");
    assert_eq!(doc.encode().unwrap(), rebound);
}
