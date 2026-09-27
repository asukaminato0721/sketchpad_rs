use super::*;

fn near(actual: Point, expected: Point) {
    assert!(
        (actual.x - expected.x).abs() < 1e-9 && (actual.y - expected.y).abs() < 1e-9,
        "{actual:?} != {expected:?}"
    );
}
fn tokens(g: &ParameterCalculationGroup) -> usize {
    let b = [b"GSP4".as_slice(), g.as_bytes()].concat();
    let r = records(&b)
        .unwrap()
        .into_iter()
        .find(|r| r.kind == 0x0907)
        .unwrap();
    (r.data.len() - 92) / 2
}
#[test]
fn long_calculation_public_writer_live_edit_and_undo_remain_exact() {
    let bytes =
        include_bytes!("../../../tests/fixtures/calculation_tokens57_api_baseline_input.gsp");
    let mut d = Document::from_bytes(bytes, None).unwrap();
    assert!(!d.partial_view);
    assert_eq!(d.encode().unwrap(), bytes);
    assert!((d.numeric_value(2).unwrap() - 1.7).abs() < 1e-12);
    let Kind::ParameterCalculation(_, g) = &d.objects[1].kind else {
        panic!()
    };
    assert_eq!(tokens(g), 339);
    near(d.position(5).unwrap(), Point { x: 222., y: 180. });
    d.set_parameter_value(1, 0.02).unwrap();
    near(d.position(5).unwrap(), Point { x: 324., y: 180. });
    d.verify_roundtrip().unwrap();
    let encoded = d.encode().unwrap();
    let rd = Document::from_bytes(&encoded, None).unwrap();
    assert!(!rd.partial_view);
    near(rd.position(5).unwrap(), Point { x: 324., y: 180. });
    d.undo();
    near(d.position(5).unwrap(), Point { x: 222., y: 180. });
    d.selected.insert(1);
    let removed = d.clear_selected().unwrap();
    assert!(removed.contains(&2) && removed.contains(&5) && removed.contains(&6));
    d.verify_roundtrip().unwrap();
    d.undo();
    near(d.position(5).unwrap(), Point { x: 222., y: 180. });
    d.verify_roundtrip().unwrap();
}
#[test]
fn original_long_function_calculation_payload_tracks_all_inputs() {
    for (bytes, expected) in [
        (
            include_bytes!("../../../tests/fixtures/calculation_tokens57_spiral_scaled_input.gsp")
                .as_slice(),
            0.1415926535,
        ),
        (
            include_bytes!(
                "../../../tests/fixtures/calculation_tokens57_spiral_scaled_edited_input.gsp"
            )
            .as_slice(),
            0.8979323846,
        ),
    ] {
        let mut d = Document::from_bytes(bytes, None).unwrap();
        assert!(!d.partial_view);
        assert_eq!(d.encode().unwrap(), bytes);
        assert_eq!(d.numeric_value(53), Some(expected));
        near(
            d.position(56).unwrap(),
            Point {
                x: 120. + 100. * expected,
                y: 180.,
            },
        );
        let Kind::ParameterCalculation(_, g) = &d.objects[52].kind else {
            panic!()
        };
        assert_eq!(tokens(g), 339);
        assert_eq!(g.sources().len(), 51);
        assert_eq!(
            g.source_kinds().unwrap()[0],
            crate::table_records::CalculationSourceKind::Function
        );
        d.set_parameter_value(1, 49.).unwrap();
        let last = d.numeric_value(52).unwrap();
        assert_eq!(d.numeric_value(53), Some(last));
        near(
            d.position(56).unwrap(),
            Point {
                x: 120. + 100. * last,
                y: 180.,
            },
        );
        d.set_parameter_value(52, 0.25).unwrap();
        assert_eq!(d.numeric_value(53), Some(0.25));
        near(d.position(56).unwrap(), Point { x: 145., y: 180. });
        d.verify_roundtrip().unwrap();
    }
    for group in [
        include_bytes!("../../../tests/fixtures/calculation_layout57_spiral_137_group.bin")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/calculation_layout57_spiral_iter_137_group.bin")
            .as_slice(),
    ] {
        let g = ParameterCalculationGroup::parse(group).unwrap();
        assert_eq!(tokens(&g), 339);
        assert_eq!(g.sources().len(), 51);
        assert_eq!(g.unit(), NumericUnit::Scalar);
    }
}
#[test]
fn long_calculation_metadata_operand_and_resource_guards_stay_strict() {
    let source =
        ParameterCalculationGroup::from_expression(1, "q", &vec!["q"; 170].join("+"), (20, 20))
            .unwrap();
    assert_eq!(tokens(&source), 339);
    assert!(ParameterCalculationGroup::from_expression(
        1,
        "q",
        &vec!["q"; 2049].join("+"),
        (20, 20)
    )
    .is_err());
    let group = [b"GSP4".as_slice(), source.as_bytes()].concat();
    let rs = records(&group).unwrap();
    let payload = rs.iter().find(|r| r.kind == 0x0907).unwrap();
    for (offset, value) in [(36, 0u32), (44, 338)] {
        let mut wrong = group.clone();
        if offset == 44 {
            wrong[payload.data.start + offset..payload.data.start + offset + 2]
                .copy_from_slice(&(value as u16).to_le_bytes());
        } else {
            wrong[payload.data.start + offset..payload.data.start + offset + 4]
                .copy_from_slice(&value.to_le_bytes());
        }
        assert!(ParameterCalculationGroup::parse(&wrong[4..]).is_err());
    }
    let mut wrong = group.clone();
    wrong[payload.data.start + 92..payload.data.start + 94]
        .copy_from_slice(&0x6001u16.to_le_bytes());
    assert!(ParameterCalculationGroup::parse(&wrong[4..]).is_err());
    let mut wrong = group.clone();
    let refs = rs.iter().find(|r| r.kind == 0x07d2).unwrap();
    wrong[refs.data.start..refs.data.start + 4].copy_from_slice(&257u32.to_le_bytes());
    assert!(ParameterCalculationGroup::parse(&wrong[4..]).is_err());
    let mut wrong = Vec::new();
    for r in rs {
        let mut p = group[r.data.clone()].to_vec();
        if r.kind == 0x0907 {
            p.truncate(92);
            for index in 0..4097 {
                let word: u16 = if index % 2 == 0 { 0x6000 } else { 0x1000 };
                p.extend_from_slice(&word.to_le_bytes());
            }
            let declared_len = p.len() - 32;
            p[36..40].copy_from_slice(&(declared_len as u32).to_le_bytes());
            p[44..46].copy_from_slice(&4097u16.to_le_bytes());
        }
        put_record(&mut wrong, r.kind, &p);
    }
    assert!(ParameterCalculationGroup::parse(&wrong).is_err());
    assert!(ParameterCalculationGroup::from_expression_with_source_unit(
        1,
        "d",
        "sin(d)",
        (20, 20),
        NumericUnit::Length
    )
    .is_err());
}
