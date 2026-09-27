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
