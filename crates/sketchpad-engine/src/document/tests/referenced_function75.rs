use super::*;

#[test]
fn native_high_word_functions_decode_evaluate_and_roundtrip() {
    let high1 = include_bytes!(
        "../../../../../tests/fixtures/referenced_function75_high1_original_input.gsp"
    );
    let document = Document::from_bytes(high1, None).unwrap();
    assert!(matches!(
        document.objects[86].kind,
        Kind::ReferencedFunction(..)
    ));
    let source = document.numeric_value(86).unwrap();
    assert!((source - 1.84611147).abs() < 1e-7);
    for argument in [-2., 0., 3.] {
        assert_eq!(document.function_value(87, argument), Some(source));
    }
    assert_eq!(document.encode().unwrap(), high1);

    let changed_high1 = include_bytes!(
        "../../../../../tests/fixtures/referenced_function75_high1_t_half_rebind87_synthetic_input.gsp"
    );
    let changed_document = Document::from_bytes(changed_high1, None).unwrap();
    let changed_source = changed_document.numeric_value(86).unwrap();
    assert!((changed_source - 1.35869792).abs() < 1e-7);
    assert_eq!(
        changed_document.function_value(87, 0.),
        Some(changed_source)
    );
    assert!(changed_source < source);
    assert_eq!(changed_document.encode().unwrap(), changed_high1);

    let high2 = include_bytes!(
        "../../../../../tests/fixtures/referenced_function75_high2_original_input.gsp"
    );
    let document = Document::from_bytes(high2, None).unwrap();
    assert!(matches!(
        document.objects[13].kind,
        Kind::ReferencedFunction(..)
    ));
    assert_eq!(document.numeric_value(5), Some(4.));
    assert!((document.function_value(14, 0.).unwrap() - 5.).abs() < 1e-8);
    assert_eq!(document.encode().unwrap(), high2);

    let changed = include_bytes!(
        "../../../../../tests/fixtures/referenced_function75_high2_a2_synthetic_input.gsp"
    );
    let document = Document::from_bytes(changed, None).unwrap();
    assert_eq!(document.numeric_value(5), Some(2.));
    assert!((document.function_value(14, 0.).unwrap() - 3.).abs() < 1e-8);
    assert_eq!(document.encode().unwrap(), changed);
}
