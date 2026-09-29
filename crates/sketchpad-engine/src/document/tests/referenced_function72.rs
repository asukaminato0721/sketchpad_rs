use super::*;

#[test]
fn original_calendar_grouped_unary_function_and_its_calculation_decode_live() {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../gsp-rs/tests/Samples/个人专栏/况永胜作品/2024年年历表.gsp"
    ))
    .unwrap();
    let document = Document::from_bytes(&bytes, None).unwrap();
    assert!(matches!(
        document.objects[4].kind,
        Kind::ReferencedFunction(..)
    ));
    assert!(matches!(
        document.objects[5].kind,
        Kind::ParameterCalculation(..)
    ));
    assert_eq!(document.numeric_value(1), Some(2024.));
    assert_eq!(document.function_value(5, 0.), None);
    for (x, expected) in [
        (1., 2023.),
        (2., 1011.),
        (3., 674.),
        (4., 505.),
        (100., 20.),
        (365., 5.),
        (400., 5.),
    ] {
        assert_eq!(document.function_value(5, x), Some(expected));
    }
    assert_eq!(document.numeric_value(6), Some(738885.));
    assert_eq!(document.encode().unwrap(), bytes);
}

#[test]
fn authored_grouped_unary_function_tracks_its_numeric_source_and_roundtrips() {
    let bytes = include_bytes!("../../../../../tests/fixtures/source_exponents_base_port.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    let function = document
        .add_referenced_function_definition(
            &[6],
            SimpleFunction::parse_with_sources("trunc((v1-1)/x)", 1).unwrap(),
        )
        .unwrap();
    document.set_parameter_value(6, 2024.).unwrap();
    assert_eq!(document.function_value(function, 2.), Some(1011.));
    document.set_parameter_value(6, 2025.).unwrap();
    assert_eq!(document.function_value(function, 2.), Some(1012.));
    assert_eq!(document.function_value(function, 0.), None);
    document.verify_roundtrip().unwrap();
    let saved = document.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert_eq!(reopened.function_value(function, 2.), Some(1012.));
    document.undo();
    assert_eq!(document.function_value(function, 2.), Some(1011.));
}
