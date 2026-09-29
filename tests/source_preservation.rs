use sketchpad_rs::document::{Document, Kind};

const SOURCE_WITH_UNEVALUATED_CONSTRAINT: &[u8] =
    include_bytes!("fixtures/source_invalid_constraint.gsp");

#[test]
fn unedited_original_with_undefined_geometry_round_trips_byte_exactly() {
    let document = Document::from_bytes(SOURCE_WITH_UNEVALUATED_CONSTRAINT, None).unwrap();
    assert_eq!(
        document.encode().unwrap(),
        SOURCE_WITH_UNEVALUATED_CONSTRAINT
    );
}

#[test]
fn an_undo_checkpoint_without_an_edit_keeps_the_original_payload() {
    let mut document = Document::from_bytes(SOURCE_WITH_UNEVALUATED_CONSTRAINT, None).unwrap();
    document.checkpoint();
    assert_eq!(
        document.encode().unwrap(),
        SOURCE_WITH_UNEVALUATED_CONSTRAINT
    );
}

#[test]
fn a_non_finite_point_edit_is_refused() {
    let mut document = Document::from_bytes(SOURCE_WITH_UNEVALUATED_CONSTRAINT, None).unwrap();
    document.checkpoint();
    // Editing an existing free point must still validate coordinates, even
    // when unrelated original constructions cannot be evaluated.
    let Kind::Free(point) = &mut document.objects[1].kind else {
        panic!("expected a free point in the source fixture");
    };
    point.x = f64::NAN;
    assert!(
        document
            .encode()
            .unwrap_err()
            .contains("Non-finite coordinates")
    );
}
