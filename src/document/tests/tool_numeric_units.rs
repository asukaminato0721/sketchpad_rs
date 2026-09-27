use super::*;

#[test]
fn native_bare_literal_addition_to_degree_parameter_is_unitless_and_live() {
    let bytes = include_bytes!(
        "../../../tests/fixtures/embedded_tool_numeric_regular_degree_native_verified.gsp"
    );
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert_eq!(
        document.objects[2].kind.numeric_unit(),
        Some(NumericUnit::Degrees)
    );
    assert_eq!(
        document.objects[4].kind.numeric_unit(),
        Some(NumericUnit::Scalar)
    );
    assert_eq!(document.numeric_value(3), Some(35.));
    assert_eq!(document.numeric_value(5), Some(36.));
    assert_eq!(document.encode().unwrap(), bytes);
    document.set_parameter_value(3, 47.).unwrap();
    assert_eq!(document.numeric_value(5), Some(48.));
    assert_eq!(
        document.objects[4].kind.numeric_unit(),
        Some(NumericUnit::Scalar)
    );
    document.verify_roundtrip().unwrap();
}

#[test]
fn native_bare_literal_addition_to_length_measurement_is_unitless_and_live() {
    let bytes = include_bytes!(
        "../../../tests/fixtures/embedded_tool_numeric_regular_measure_native_verified.gsp"
    );
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert_eq!(
        document.objects[5].kind.numeric_unit(),
        Some(NumericUnit::Length)
    );
    assert_eq!(
        document.objects[6].kind.numeric_unit(),
        Some(NumericUnit::Scalar)
    );
    assert!((document.numeric_value(6).unwrap() - 5.).abs() < 1e-10);
    assert!((document.numeric_value(7).unwrap() - 6.).abs() < 1e-10);
    assert_eq!(document.encode().unwrap(), bytes);
    let Kind::SegmentLength(host) = document.objects[5].kind else {
        panic!("Expected the native length measurement")
    };
    let Kind::Segment(a, b) = document.objects[host - 1].kind else {
        panic!("Expected its measured segment")
    };
    let start = document.position(a).unwrap();
    let end = document.position(b).unwrap();
    assert!(document.move_point(
        b,
        Point {
            x: start.x + 2. * (end.x - start.x),
            y: start.y + 2. * (end.y - start.y)
        }
    ));
    assert!((document.numeric_value(7).unwrap() - 11.).abs() < 1e-10);
    document.verify_roundtrip().unwrap();
}
