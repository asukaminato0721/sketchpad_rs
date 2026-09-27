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
