use super::*;

const BINARY: &[u8] =
    include_bytes!("../../../../../tests/fixtures/decimal_to_binary_imported.gsp");

fn object_group(bytes: &[u8], id: usize) -> Vec<u8> {
    let rs = records(bytes).unwrap();
    let start = rs
        .iter()
        .filter(|r| r.kind == 0x07d0)
        .nth(id - 1)
        .unwrap()
        .start;
    let end = rs
        .iter()
        .find(|r| r.start > start && r.kind == 0x07d7)
        .unwrap()
        .data
        .end;
    bytes[start..end].to_vec()
}

#[test]
fn imported_binary_parameter_and_calculations_remain_editable() {
    let mut doc = Document::from_bytes(BINARY, None).unwrap();
    assert_eq!(doc.numeric_value(1), Some(292.));
    assert!(matches!(doc.objects[34].kind, Kind::Unsupported(92)));
    let Kind::Parameter(group) = &doc.objects[0].kind else {
        panic!()
    };
    let settings = group.animation_settings();
    let properties = (
        group.precision(),
        group.display_mode(),
        group.includes_edit_box(),
    );
    doc.set_parameter_properties(1, 293., properties.0, properties.1, properties.2, settings)
        .unwrap();
    assert_eq!(doc.numeric_value(1), Some(293.));
    assert_eq!(doc.numeric_value(4), Some(1.));
    let edited = doc.encode().unwrap();
    // Only the parameter group changes. Keep every other native group byte exact.
    for id in 2..=36 {
        assert_eq!(
            object_group(&edited, id),
            object_group(BINARY, id),
            "object {id}"
        );
    }
    let reopened = Document::from_bytes(&edited, None).unwrap();
    assert_eq!(reopened.numeric_value(1), Some(293.));
    doc.undo();
    assert_eq!(doc.numeric_value(1), Some(292.));
    doc.redo();
    assert_eq!(doc.numeric_value(1), Some(293.));

    // The original formula uses two saved sources, regardless of UI selection.
    assert_eq!(doc.objects[3].kind.dependencies(), vec![1, 3]);
    doc.set_calculation_expression(4, "v1 - 2*v2 + 1").unwrap();
    assert_eq!(doc.numeric_value(4), Some(2.));
    let edited_formula = doc.encode().unwrap();
    for id in 2..=36 {
        if id != 4 {
            assert_eq!(
                object_group(&edited_formula, id),
                object_group(BINARY, id),
                "object {id}"
            );
        }
    }
    doc.verify_roundtrip().unwrap();
    let mut reopened = Document::from_bytes(&edited_formula, None).unwrap();
    assert_eq!(reopened.numeric_value(4), Some(2.));
    reopened.set_parameter_value(1, 294.).unwrap();
    assert_eq!(reopened.numeric_value(4), Some(1.));
    doc.undo();
    assert_eq!(doc.numeric_value(4), Some(1.));
    doc.redo();
    assert_eq!(doc.numeric_value(4), Some(2.));

    let state = doc.objects.clone();
    let history = doc.undo.len();
    for expression in ["v3+1", "v1+", "v1*1 cm", "f1(2)"] {
        assert!(
            doc.set_calculation_expression(4, expression).is_err(),
            "{expression}"
        );
        assert_eq!(doc.objects, state);
        assert_eq!(doc.undo.len(), history);
    }
}

#[test]
fn saved_multi_page_parameter_properties_and_formulas_roundtrip() {
    let mut doc = Document::new();
    let p = doc.add_named_parameter("n", 2., (10, 10)).unwrap();
    let c = doc.add_parameter_calculation(p, "n+1", (10, 50)).unwrap();
    let literal = doc.add_calculation_expression("2+3", (10, 90)).unwrap();
    let mut doc = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    doc.add_blank_page().unwrap();
    doc.add(Kind::Free(Point { x: 100., y: 200. }));
    doc.active_page = 0;
    let bytes = doc.encode().unwrap();
    let mut doc = Document::from_bytes(&bytes, None).unwrap();
    let Kind::Parameter(group) = &doc.objects[p - 1].kind else {
        panic!()
    };
    let mut settings = group.animation_settings();
    settings.domain = (-500., 500.);
    settings.keyboard_step = 2.;
    doc.set_parameter_properties(p, 293., 0, 1, true, settings)
        .unwrap();
    doc.set_calculation_expression(c, "v1*2").unwrap();
    doc.set_calculation_expression(literal, "3+4").unwrap();
    assert_eq!(doc.numeric_value(c), Some(586.));
    assert_eq!(doc.numeric_value(literal), Some(7.));
    doc.verify_roundtrip().unwrap();
    let encoded = doc.encode().unwrap();
    assert_eq!(object_group(&encoded, 4), object_group(&bytes, 4));
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.numeric_value(c), Some(586.));
    let Kind::Parameter(group) = &reopened.objects[p - 1].kind else {
        panic!()
    };
    assert_eq!(group.animation_settings(), settings);
    assert!(group.includes_edit_box());
    assert_eq!(group.display_mode(), 1);
    doc.active_page = 1;
    assert!(doc.set_calculation_expression(c, "v1+1").is_err());
    assert!(doc.set_parameter_value(p, 1.).is_err());
}

#[test]
fn measured_and_function_calculations_keep_their_sources_after_edits() {
    let mut doc = Document::new();
    let a = doc.add_calculation_expression("2 cm", (10, 10)).unwrap();
    let b = doc.add_calculation_expression("3 cm", (10, 50)).unwrap();
    let scaled = doc.add_measured_length_calculation(a, 2, (10, 90)).unwrap();
    let pair = doc
        .add_length_pair_calculation(a, b, false, (10, 130))
        .unwrap();
    let f = doc
        .add_function_definition(SimpleFunction::parse("x*x").unwrap())
        .unwrap();
    let call = doc.add_multi_calculation(&[f], "f1(2)", (10, 170)).unwrap();
    let bytes = doc.encode().unwrap();
    let mut doc = Document::from_bytes(&bytes, None).unwrap();
    assert!(doc.calculation_expression(scaled).unwrap().contains("v1"));
    assert!(doc.calculation_expression(pair).unwrap().contains("v2"));
    assert!(doc.calculation_expression(call).unwrap().contains("f1"));
    doc.set_calculation_expression(scaled, "v1*3").unwrap();
    doc.set_calculation_expression(pair, "(v1+v2)*2").unwrap();
    doc.set_calculation_expression(call, "f1(3)+1").unwrap();
    assert_eq!(doc.numeric_value(scaled), Some(6.));
    assert_eq!(doc.numeric_value(pair), Some(10.));
    assert_eq!(doc.numeric_value(call), Some(10.));
    doc.verify_roundtrip().unwrap();
    let mut doc = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    doc.set_calculation_expression(a, "4 cm").unwrap();
    assert_eq!(doc.numeric_value(scaled), Some(12.));
    assert_eq!(doc.numeric_value(pair), Some(14.));
}
