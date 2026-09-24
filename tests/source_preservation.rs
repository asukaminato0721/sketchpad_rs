use sketchpad_rs::document::Document;

const SOURCE_WITH_UNEVALUATED_CONSTRAINT: &[u8] =
    include_bytes!("fixtures/source_invalid_constraint.gsp");

#[test]
fn unedited_original_with_undefined_geometry_round_trips_byte_exactly() {
    let document = Document::from_bytes(SOURCE_WITH_UNEVALUATED_CONSTRAINT, None).unwrap();
    assert_eq!(document.encode().unwrap(), SOURCE_WITH_UNEVALUATED_CONSTRAINT);
}

#[test]
fn edited_original_with_undefined_geometry_is_refused() {
    let mut document = Document::from_bytes(SOURCE_WITH_UNEVALUATED_CONSTRAINT, None).unwrap();
    document.checkpoint();
    assert!(document.encode().is_err());
}
