use super::*;
use crate::document::EmbeddedToolInputKind;

#[test]
fn numeric_tool_givens_bind_typed_parameters_measurements_and_calculations_atomically() {
    for (suffix, bytes, source, expected) in [
        (
            "scalar",
            include_bytes!(
                "../../../../../tests/fixtures/embedded_tool_scalar_created_native_verified.gsp"
            )
            .as_slice(),
            1,
            3.,
        ),
        (
            "length",
            include_bytes!(
                "../../../../../tests/fixtures/embedded_tool_scalar_on_length_native_verified.gsp"
            )
            .as_slice(),
            3,
            6.,
        ),
        (
            "degree",
            include_bytes!(
                "../../../../../tests/fixtures/embedded_tool_scalar_on_degree_native_verified.gsp"
            )
            .as_slice(),
            3,
            36.,
        ),
        (
            "measure",
            include_bytes!(
                "../../../../../tests/fixtures/embedded_tool_scalar_on_measure_native_verified.gsp"
            )
            .as_slice(),
            6,
            6.,
        ),
        (
            "area",
            include_bytes!(
                "../../../../../tests/fixtures/embedded_tool_scalar_on_area_native_verified.gsp"
            )
            .as_slice(),
            8,
            6.,
        ),
    ] {
        let mut document = Document::from_bytes(bytes, None).unwrap();
        let descriptors = document.embedded_tool_inputs(0).unwrap();
        assert_eq!(descriptors[0].kind(), Some(EmbeddedToolInputKind::Numeric));
        let prototype = document.embedded_tool_document(0).unwrap();
        assert!(!prototype.partial_view);
        assert_eq!(
            prototype.objects[0].kind,
            Kind::ToolInput(EmbeddedToolInputKind::Numeric)
        );
        assert_eq!(prototype.objects[0].kind.numeric_unit(), None);
        assert_eq!(prototype.numeric_value(1), None);
        assert!(prototype.encode().is_err());
        assert!(!document.partial_view);
        let native_font_control = Document::from_bytes(
            include_bytes!("../../../../../tests/fixtures/embedded_tool_numeric_24bold_caption_flow_native_verified.gsp"),
            None,
        ).unwrap();
        let native_caption_font = native_font_control
            .text_font_state(native_font_control.objects.last().unwrap().id)
            .unwrap();
        let before = document.encode().unwrap();
        let history = document.undo.len();
        let output = document
            .invoke_embedded_tool(0, &[source])
            .unwrap_or_else(|error| panic!("{suffix}: {error}"))[0];
        assert_eq!(
            document.objects[output - 1].kind.numeric_unit(),
            Some(NumericUnit::Scalar)
        );
        assert!((document.numeric_value(output).unwrap() - expected).abs() < 1e-10);
        assert_eq!(document.text_font_state(output), Some(native_caption_font));
        assert_eq!(document.undo.len(), history + 1);
        let saved = document.encode().unwrap();
        document.verify_roundtrip().unwrap();
        document.undo();
        assert_eq!(document.encode().unwrap(), before);
        document.redo();
        assert_eq!(document.encode().unwrap(), saved);
        let second = document.invoke_embedded_tool(0, &[output]).unwrap()[0];
        assert!((document.numeric_value(second).unwrap() - expected - 1.).abs() < 1e-10);
        document.verify_roundtrip().unwrap();
        if std::env::var_os("SKETCHPAD_WRITE_TOOL_NUMERIC_FIXTURES").is_some() {
            fs::write(
                format!("/tmp/embedded_tool_numeric_{suffix}_invoked_port.gsp"),
                document.encode().unwrap(),
            )
            .unwrap();
        }
        if matches!(document.objects[source - 1].kind, Kind::Parameter(_)) {
            document.set_parameter_value(source, 7.).unwrap();
            assert_eq!(document.numeric_value(output), Some(8.));
            assert_eq!(document.numeric_value(second), Some(9.));
            document.verify_roundtrip().unwrap();
        }
        let point = document.add(Kind::Free(Point { x: 300., y: 280. }));
        let before = document.encode().unwrap();
        let history = document.undo.len();
        assert!(
            document
                .invoke_embedded_tool(0, &[point])
                .unwrap_err()
                .contains("numeric value")
        );
        assert_eq!(document.encode().unwrap(), before);
        assert_eq!(document.undo.len(), history);
    }
}

#[test]
fn numeric_tool_multiplication_specializes_units_and_tracks_nested_results() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/embedded_tool_numeric_multiply_length_native_verified.gsp"
    );
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert!(!document.partial_view);
    let prototype = document.embedded_tool_document(0).unwrap();
    assert!(!prototype.partial_view);
    let before = document.encode().unwrap();
    let output = document.invoke_embedded_tool(0, &[6]).unwrap()[0];
    assert_eq!(
        document.objects[output - 1].kind.numeric_unit(),
        Some(NumericUnit::Length)
    );
    assert!((document.numeric_value(output).unwrap() - 10.).abs() < 1e-10);
    let saved = document.encode().unwrap();
    document.undo();
    assert_eq!(document.encode().unwrap(), before);
    document.redo();
    assert_eq!(document.encode().unwrap(), saved);
    let nested = document.invoke_embedded_tool(0, &[output]).unwrap()[0];
    assert_eq!(
        document.objects[nested - 1].kind.numeric_unit(),
        Some(NumericUnit::Length)
    );
    assert!((document.numeric_value(nested).unwrap() - 20.).abs() < 1e-10);
    let Kind::SegmentLength(host) = document.objects[5].kind else {
        panic!("Expected measured segment")
    };
    let Kind::Segment(a, b) = document.objects[host - 1].kind else {
        panic!("Expected source segment")
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
    assert!((document.numeric_value(output).unwrap() - 20.).abs() < 1e-10);
    assert!((document.numeric_value(nested).unwrap() - 40.).abs() < 1e-10);
    document.verify_roundtrip().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_TOOL_NUMERIC_FIXTURES").is_some() {
        fs::write(
            "/tmp/embedded_tool_numeric_product_invoked_port.gsp",
            document.encode().unwrap(),
        )
        .unwrap();
    }
}
