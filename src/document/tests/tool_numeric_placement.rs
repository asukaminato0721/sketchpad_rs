use super::*;
use crate::table_records::CalculationGroup;

fn remove_last_native_output(document: &mut Document) -> (i16, i16) {
    let id = document.objects.last().unwrap().id;
    let Kind::ParameterCalculation(_, group) = &document.objects[id - 1].kind else {
        panic!("Expected the retained Native invocation result");
    };
    let anchor = group.anchor();
    document.selected.clear();
    document.selected.insert(id);
    assert_eq!(document.clear_selected().unwrap(), [id]);
    anchor
}

#[test]
fn tool_numeric_placement_follows_native_parameter_and_measurement_bottoms() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/embedded_tool_scalar_invoked_native_verified.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/embedded_tool_scalar_on_length_native_verified.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/embedded_tool_scalar_on_degree_native_verified.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/embedded_tool_scalar_on_measure_native_verified.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/embedded_tool_scalar_on_area_native_verified.gsp").as_slice(),
    ] {
        let mut document = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(document.encode().unwrap(), bytes);
        let expected = remove_last_native_output(&mut document);
        assert_eq!(document.next_tool_numeric_anchor().unwrap(), expected);
        let last = document.objects.iter().filter(|object| !object.deleted
            && object.kind.numeric_unit().is_some()).last().unwrap().id;
        let before = document.next_tool_numeric_anchor().unwrap();
        if matches!(document.objects[last - 1].kind, Kind::Parameter(_)) {
            document.move_number(last, (300, 300)).unwrap();
            assert_eq!(document.next_tool_numeric_anchor().unwrap(),
                (300, 300 + before.1 - 120));
            document.undo();
            assert_eq!(document.next_tool_numeric_anchor().unwrap(), before);
        }
    }
}

#[test]
fn tool_numeric_placement_uses_last_id_and_current_calculation_anchor() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/embedded_tool_numeric_flow_last_caption_native_verified.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/embedded_tool_numeric_flow_moved_last_caption_native_verified.gsp").as_slice(),
    ] {
        let mut document = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(document.encode().unwrap(), bytes);
        let expected = remove_last_native_output(&mut document);
        assert_eq!(document.next_tool_numeric_anchor().unwrap(), expected);
        document.move_number(4, (200, 400)).unwrap();
        assert_eq!(document.next_tool_numeric_anchor().unwrap(), (200, 427));
        document.undo();
        assert_eq!(document.next_tool_numeric_anchor().unwrap(), expected);
    }
}

#[test]
fn tool_numeric_placement_uses_new_literal_bounds_and_checks_overflow() {
    let mut document = Document::new();
    let mut group = CalculationGroup::from_expression("2+3").unwrap();
    group.set_anchor((120, 200)).unwrap();
    let height = group.bbox().1.1 - group.bbox().0.1 + 1;
    let id = document.add(Kind::Calculation(group));
    assert_eq!(document.next_tool_numeric_anchor().unwrap(), (120, 200 + height));
    document.move_number(id, (120, i16::MAX - height)).unwrap();
    assert_eq!(document.next_tool_numeric_anchor().unwrap(), (120, i16::MAX));
    document.move_number(id, (120, i16::MAX - height + 1)).unwrap();
    assert!(document.next_tool_numeric_anchor().is_err());
}

#[test]
fn tool_fraction_placement_matches_native_expression_tree_heights() {
    for (bytes, expected, source, value) in [
        (include_bytes!("../../../tests/fixtures/embedded_tool_numeric_fraction_single_flow_native_verified.gsp").as_slice(), (20, 216), 3, 2.5),
        (include_bytes!("../../../tests/fixtures/embedded_tool_numeric_fraction_nested_flow_native_verified.gsp").as_slice(), (20, 245), 2, 0.5),
        (include_bytes!("../../../tests/fixtures/embedded_tool_numeric_fraction_24bold_flow_native_verified.gsp").as_slice(), (20, 130), 1, 1.0),
    ] {
        let mut document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        assert_eq!(document.encode().unwrap(), bytes);
        assert_eq!(remove_last_native_output(&mut document), expected);
        assert_eq!(document.next_tool_numeric_anchor().unwrap(), expected);
        let output = document.invoke_embedded_tool(0, &[source]).unwrap()[0];
        assert_eq!(document.numeric_value(output), Some(value));
        let Kind::ParameterCalculation(_, group) = &document.objects[output - 1].kind else { panic!("Expected fraction calculation") };
        assert_eq!(group.anchor(), expected);
        document.verify_roundtrip().unwrap();
        let native_writer: &[u8] = match expected.1 {
            216 => include_bytes!("../../../tests/fixtures/embedded_tool_numeric_fraction_single_invoked_writer_native_verified.gsp"),
            245 => include_bytes!("../../../tests/fixtures/embedded_tool_numeric_fraction_nested_invoked_writer_native_verified.gsp"),
            130 => include_bytes!("../../../tests/fixtures/embedded_tool_numeric_fraction_24bold_invoked_writer_native_verified.gsp"),
            _ => unreachable!(),
        };
        assert_eq!(document.encode().unwrap(), native_writer);
        if std::env::var_os("SKETCHPAD_WRITE_TOOL_FRACTION_FIXTURES").is_some() {
            std::fs::write(format!("/tmp/embedded_tool_numeric_fraction_{}_writer.gsp", expected.1), document.encode().unwrap()).unwrap();
        }
        document.move_number(output, (300, 300)).unwrap();
        let step = if expected.1 == 245 { 85 } else { 56 };
        assert_eq!(document.next_tool_numeric_anchor().unwrap(), (300, 300 + step));
        document.undo();
        assert_eq!(document.next_tool_numeric_anchor().unwrap(), (20, expected.1 + step));
    }
}

#[test]
fn tool_measurement_caption_shapes_and_fonts_match_native_flow() {
    let controls: &[&[u8]] = &[
        include_bytes!("../../../tests/fixtures/embedded_tool_numeric_point_distance_caption_flow_native_verified.gsp"),
        include_bytes!("../../../tests/fixtures/embedded_tool_numeric_angle_caption_flow_native_verified.gsp"),
        include_bytes!("../../../tests/fixtures/embedded_tool_numeric_slope_caption_flow_native_verified.gsp"),
        include_bytes!("../../../tests/fixtures/embedded_tool_numeric_point_value_caption_flow_native_verified.gsp"),
        include_bytes!("../../../tests/fixtures/embedded_tool_numeric_circle_radius_caption_flow_native_verified.gsp"),
        include_bytes!("../../../tests/fixtures/embedded_tool_numeric_circle_area_caption_flow_native_verified.gsp"),
        include_bytes!("../../../tests/fixtures/embedded_tool_numeric_annotation_18normal_caption_flow_native_verified.gsp"),
        include_bytes!("../../../tests/fixtures/embedded_tool_numeric_annotation_24bold_caption_flow_native_verified.gsp"),
        include_bytes!("../../../tests/fixtures/embedded_tool_numeric_annotation_24normal_caption_flow_native_verified.gsp"),
    ];
    for (case, bytes) in controls.iter().enumerate() {
        let mut document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        assert_eq!(document.encode().unwrap(), *bytes);
        let last = document.objects.last().unwrap().id;
        let Kind::ParameterCalculation(source, _) = document.objects[last - 1].kind else { panic!("Expected Native output") };
        let native_value = document.numeric_value(last).unwrap();
        let expected = remove_last_native_output(&mut document);
        assert_eq!(document.next_tool_numeric_anchor().unwrap(), expected, "case{case}");
        let output = document.invoke_embedded_tool(0, &[source]).unwrap()[0];
        assert_eq!(document.numeric_value(output), Some(native_value));
        let Kind::ParameterCalculation(_, group) = &document.objects[output - 1].kind else { panic!("Expected specialized output") };
        assert_eq!(group.anchor(), expected);
        document.verify_roundtrip().unwrap();
        if std::env::var_os("SKETCHPAD_WRITE_TOOL_MEASUREMENT_FIXTURES").is_some() {
            std::fs::write(format!("/tmp/embedded_tool_numeric_measurement_flow_case{case}_writer.gsp"), document.encode().unwrap()).unwrap();
        }
        document.move_measurement_caption(source, (300, 300)).unwrap();
        document.selected.clear();
        document.selected.insert(output);
        document.clear_selected().unwrap();
        assert_eq!(document.next_tool_numeric_anchor().unwrap(), (300, 300 + expected.1 - 120));
        document.undo();
        document.undo();
        assert_eq!(document.next_tool_numeric_anchor().unwrap(), (20, expected.1 + 27));
    }
}

#[test]
fn tool_named_fraction_source_uses_native_alias_and_subscript_extent() {
    let bytes = include_bytes!("../../../tests/fixtures/embedded_tool_numeric_named_fraction_caption_flow_native_verified.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert_eq!(document.encode().unwrap(), bytes);
    assert_eq!(document.objects[1].label.as_ref().unwrap().name_utf8().unwrap(), "m[1]");
    assert_eq!(document.parameter_calculation_expression(3).as_deref(), Some("(m[1] ÷ 2)"));
    let expected = remove_last_native_output(&mut document);
    assert_eq!(expected, (20, 212));
    assert_eq!(document.next_tool_numeric_anchor().unwrap(), expected);
    let output = document.invoke_embedded_tool(0, &[2]).unwrap()[0];
    assert_eq!(document.numeric_value(output), Some(0.5));
    assert_eq!(document.parameter_calculation_expression(output).as_deref(), Some("(m[1] ÷ 2)"));
    document.verify_roundtrip().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_TOOL_MEASUREMENT_FIXTURES").is_some() {
        std::fs::write("/tmp/embedded_tool_numeric_named_fraction_writer.gsp", document.encode().unwrap()).unwrap();
    }
    document.selected.clear(); document.selected.insert(output); document.clear_selected().unwrap();
    document.selected.clear(); document.selected.insert(3); document.clear_selected().unwrap();
    assert_eq!(document.next_tool_numeric_anchor().unwrap(), (20, 155));
}

#[test]
fn default_font_math_caption_uses_native_zero_style_metrics() {
    let bytes = include_bytes!("../../../tests/fixtures/embedded_tool_numeric_measured_multiply_invoked_native_verified.gsp");
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert_eq!(document.encode().unwrap(), bytes);
    let id = document.objects.last().unwrap().id;
    let anchor = match &document.objects[id - 1].kind {
        Kind::MeasuredCalculation(_, g) => g.anchor(),
        Kind::ParameterCalculation(_, g) => g.anchor(),
        other => panic!("Unexpected Native result{other:?}"),
    };
    document.selected.insert(id); document.clear_selected().unwrap();
    assert_eq!(anchor, (20, 301));
    assert_eq!(document.next_tool_numeric_anchor().unwrap(), anchor);
}
