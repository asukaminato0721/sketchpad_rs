use super::*;

#[test]
fn scientific_dimension_captions_follow_geometry_precision_and_copied_sources() {
    let bytes = include_bytes!("../../../tests/fixtures/scientific_dimension_native_verified.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    for (id, suffix) in [
        (7, "2.57 · 10⁶ cm"),
        (8, "6.72 · 10⁸ cm²"),
        (9, "2.65 · 10⁶ cm"),
    ] {
        let (_, lines) = crate::render::number_lines_for_object(&doc, id).unwrap();
        assert!(lines[0].ends_with(suffix), "{}", lines[0]);
    }
    doc.move_point(2, Point { x: 250., y: 180. });
    doc.set_numeric_precision(&[7, 8, 9], 4).unwrap();
    assert_eq!(
        doc.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/scientific_dimension_edited_port.gsp")
    );
    for (id, suffix) in [
        (7, "3.8509 · 10⁶ cm"),
        (8, "1.5130 · 10⁹ cm²"),
        (9, "3.9688 · 10⁶ cm"),
    ] {
        let (_, lines) = crate::render::number_lines_for_object(&doc, id).unwrap();
        assert!(lines[0].ends_with(suffix), "{}", lines[0]);
    }
    let full_value = doc.numeric_value(8).unwrap();
    doc.selected.insert(8);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = *pasted
        .iter()
        .find(|id| matches!(doc.objects[**id - 1].kind, Kind::LengthPairCalculation(..)))
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.numeric_value(copied), Some(full_value));
    let (_, lines) = crate::render::number_lines_for_object(&reopened, copied).unwrap();
    assert!(lines[0].ends_with("1.5130 · 10⁹ cm²"));
    let area_bytes = include_bytes!("../../../tests/fixtures/scientific_area_native_verified.gsp");
    let mut area = Document::from_bytes(area_bytes, None).unwrap();
    assert!(!area.partial_view);
    assert_eq!(area.encode().unwrap(), area_bytes);
    assert_eq!(area.measurement_text(4).unwrap(), "Area = 1.37 · 10⁶ cm²");
    let value = area.numeric_value(4).unwrap();
    area.set_numeric_precision(&[4], 0).unwrap();
    assert_eq!(area.measurement_text(4).unwrap(), "Area = 1 · 10⁶ cm²");
    assert_eq!(area.numeric_value(4), Some(value));
    assert_eq!(
        Document::from_bytes(&area.encode().unwrap(), None)
            .unwrap()
            .numeric_value(4),
        Some(value)
    );
}

#[test]
fn native_scientific_captions_preserve_boundary_rounding_units_and_labels() {
    for (bytes, captions) in [
        (
            include_bytes!("../../../tests/fixtures/scientific_calculation_native_verified.gsp")
                .as_slice(),
            vec![
                "1000000.00",
                "999999.99",
                "-1000000.00",
                "0.00",
                "1.80 · 10⁸°",
            ],
        ),
        (
            include_bytes!("../../../tests/fixtures/scientific_boundary_native_verified.gsp")
                .as_slice(),
            vec![
                "1.00 · 10⁶",
                "1.00 · 10⁶",
                "1.00 · 10⁷",
                "-1.00 · 10⁶",
                "1000000.0",
                "1.00000 · 10⁶",
            ],
        ),
        (
            include_bytes!("../../../tests/fixtures/scientific_rounding_native_verified.gsp")
                .as_slice(),
            vec![
                "1.00 · 10⁷",
                "-1.00 · 10⁷",
                "1.00 · 10⁸",
                "1.00 · 10⁹",
                "-2.86 · 10¹⁴",
            ],
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        for (index, caption) in captions.iter().enumerate() {
            let (_, lines) = crate::render::number_lines_for_object(&doc, index + 1).unwrap();
            assert!(lines[0].ends_with(caption), "{}", lines[0]);
        }
    }
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/scientific_calculation_native_verified.gsp"),
        None,
    )
    .unwrap();
    doc.set_numeric_precision(&[5], 5).unwrap();
    assert_eq!(doc.numeric_value(5), Some(180000000.));
    let point = doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let label = doc.add_point_numeric_label(point, 5).unwrap();
    assert_eq!(doc.point_numeric_label(label).unwrap().1, "1.80000 · 10⁸°");
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(
        reopened.point_numeric_label(label),
        doc.point_numeric_label(label)
    );
    assert_eq!(reopened.numeric_value(5), Some(180000000.));
}

#[test]
fn calculation_precision_import_edit_undo_copy_and_numeric_labels() {
    let bytes = include_bytes!("../../../tests/fixtures/calculation_precision_native_verified.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    for (id, caption) in [(8, "5.292 cm"), (9, "2.0000"), (11, "0.24690"), (12, "5")] {
        let (_, lines) = crate::render::number_lines_for_object(&doc, id).unwrap();
        assert!(lines[0].ends_with(caption), "{}", lines[0]);
    }
    assert_eq!(doc.numeric_value(12), Some(5.125));
    assert_eq!(doc.set_numeric_precision(&[4, 8, 9, 11, 12], 5).unwrap(), 4);
    let changed = doc.encode().unwrap();
    assert_eq!(doc.set_numeric_precision(&[4, 8, 9, 11, 12], 5).unwrap(), 0);
    for (ids, precision) in [
        (vec![4, 8, 1], 3),
        (vec![12], 6),
        (vec![0], 2),
        (vec![], 2),
        (vec![10], 3),
    ] {
        assert!(doc.set_numeric_precision(&ids, precision).is_err());
        assert_eq!(doc.encode().unwrap(), changed);
    }
    doc.undo();
    assert_eq!(doc.encode().unwrap(), bytes);
    doc.redo();
    assert_eq!(doc.encode().unwrap(), changed);
    let label = doc.add_point_numeric_label(5, 12).unwrap();
    assert_eq!(doc.point_numeric_label(label).unwrap().1, "5.12500");
    doc.selected.insert(12);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = *pasted
        .iter()
        .find(|id| matches!(doc.objects[**id - 1].kind, Kind::Calculation(..)))
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    let Kind::Calculation(group) = &reopened.objects[copied - 1].kind else {
        panic!("calculation")
    };
    assert_eq!(group.value_text(), "5.12500");
    assert_eq!(reopened.numeric_value(copied), Some(5.125));
    let angle = doc.add_calculation_expression("360°/7", (10, 260)).unwrap();
    doc.set_numeric_precision(&[angle], 4).unwrap();
    let Kind::Calculation(group) = &doc.objects[angle - 1].kind else {
        panic!("calculation")
    };
    assert_eq!(group.value_text(), "51.4286°");
    assert_eq!(doc.numeric_value(angle), Some(360. / 7.));
}

#[test]
fn numeric_group_visibility_preserves_records_on_hide_show_and_paste() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/calculation_precision_native_verified.gsp"),
        None,
    )
    .unwrap();
    for visible in [false, true] {
        doc.selected = [8, 9, 10, 11, 12].into_iter().collect();
        doc.set_selected_visibility(visible);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        doc.validate_reparsed(&reopened).unwrap();
        for id in [8, 9, 10, 11, 12] {
            assert_eq!(reopened.objects[id - 1].visible, visible);
        }
    }
    doc.selected = [10].into_iter().collect();
    doc.set_selected_visibility(false);
    doc.selected.insert(11);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    doc.selected = pasted.iter().copied().collect();
    doc.set_selected_visibility(true);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    doc.validate_reparsed(&reopened).unwrap();
    assert!(pasted.iter().all(|id| reopened.objects[*id - 1].visible));
    for bytes in [
        include_bytes!("../../../tests/fixtures/table_parameter_tabulated.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/table_two_columns_original.gsp").as_slice(),
    ] {
        let mut table = Document::from_bytes(bytes, None).unwrap();
        let id = table
            .objects
            .iter()
            .find(|o| matches!(o.kind, Kind::ValueTable(..) | Kind::ValueTablePair(..)))
            .unwrap()
            .id;
        for visible in [false, true] {
            table.selected.insert(id);
            table.set_selected_visibility(visible);
            let reopened = Document::from_bytes(&table.encode().unwrap(), None).unwrap();
            table.validate_reparsed(&reopened).unwrap();
            assert_eq!(reopened.objects[id - 1].visible, visible);
        }
    }
}

#[test]
fn extended_parameter_values_preserve_precision_and_dependencies() {
    let bytes =
        include_bytes!("../../../tests/fixtures/parameter_extended_range_native_verified.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    for (id, value, caption) in [
        (1, 1000.12345, "1000.12345"),
        (2, -123456789.98765, "-1.23457 · 10⁸"),
        (3, 12345678901234567890., "1.2 · 10¹⁹"),
    ] {
        assert_eq!(doc.numeric_value(id), Some(value));
        let Kind::Parameter(group) = &doc.objects[id - 1].kind else {
            panic!("parameter")
        };
        assert_eq!(group.value_text(), caption);
        let (_, lines) = crate::render::number_lines(&doc.objects[id - 1].kind).unwrap();
        assert_eq!(lines, vec![format!("{} = {caption}", group.name())]);
    }
    doc.set_parameter_value(1, 0.1234567890123456).unwrap();
    assert_eq!(doc.numeric_value(4), Some(0.2469135780246912));
    let Kind::Parameter(group) = &doc.objects[0].kind else {
        panic!("parameter")
    };
    assert_eq!(group.precision(), 5);
    assert_eq!(group.value_text(), "0.12346");
    let saved = doc.encode().unwrap();
    for value in [f64::NAN, f64::INFINITY, 1e32, 1e-31] {
        assert!(doc.set_parameter_value(1, value).is_err());
        assert_eq!(doc.encode().unwrap(), saved);
    }
    doc.selected.insert(4);
    let clipboard = doc.copy_selected().unwrap();
    let pasted = doc.paste(&clipboard).unwrap();
    let copied = *pasted
        .iter()
        .find(|id| matches!(doc.objects[**id - 1].kind, Kind::ParameterCalculation(..)))
        .unwrap();
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.numeric_value(copied), Some(0.2469135780246912));
    let mut table = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/table_parameter_tabulated.gsp"),
        None,
    )
    .unwrap();
    let saved = table.encode().unwrap();
    for value in [1000.12345, 0.12345] {
        assert!(table.set_parameter_value(1, value).is_err());
        assert_eq!(table.encode().unwrap(), saved);
    }
    let mut named = Document::from_bytes(
        include_bytes!(
            "../../../tests/fixtures/table_named_speed_precision2_snapshot_original.gsp"
        ),
        None,
    )
    .unwrap();
    for value in [1000.12345, 0.12345, 1.000001] {
        named.set_parameter_value(1, value).unwrap();
        let saved = named.encode().unwrap();
        assert!(named.change_table_rows(2, true).is_err());
        assert_eq!(named.encode().unwrap(), saved);
    }
}
#[test]
fn parameter_decimal_parsing_matches_native_rounding_boundary() {
    let bytes =
        include_bytes!("../../../tests/fixtures/parameter_rounding_boundary_native_verified.gsp");
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(doc.numeric_value(1), Some(0.15));
    assert_eq!(doc.numeric_value(2), Some(-4.));
    let mut animated = Document::new();
    let parameter = animated
        .add_named_parameter("x", 1000.12345, (10, 25))
        .unwrap();
    animated
        .animate_parameter_value(parameter, 1000.123456)
        .unwrap();
    assert_eq!(animated.numeric_value(parameter), Some(1000.12346));
    let reopened = Document::from_bytes(&animated.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.numeric_value(parameter), Some(1000.12346));
}

#[test]
fn live_iteration_depth_truncates_native_parameter_and_calculation_values() {
    let bytes =
        include_bytes!("../../../tests/fixtures/iteration_fractional_depth_native_verified.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(doc.point_iteration_images(4).unwrap().len(), 5);
    for (value, count) in [(5.9, 5), (6.1, 6), (128.9, 128)] {
        doc.set_parameter_value(2, value).unwrap();
        assert_eq!(doc.point_iteration_images(4).unwrap().len(), count);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert_eq!(reopened.point_iteration_images(4).unwrap().len(), count);
    }
    for value in [0.9, -2.5, 129.] {
        doc.set_parameter_value(2, value).unwrap();
        assert!(doc.point_iteration_images(4).is_none());
    }
    for (bytes, count) in [
        (
            include_bytes!(
                "../../../tests/fixtures/iteration_fractional_calculation_depth_port.gsp"
            )
            .as_slice(),
            5,
        ),
        (
            include_bytes!(
                "../../../tests/fixtures/iteration_fractional_calculation_depth_edited_port.gsp"
            )
            .as_slice(),
            7,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        assert_eq!(doc.point_iteration_images(5).unwrap().len(), count);
    }
}
#[test]
fn decimal_parameters_roundtrip_and_drive_path_points() {
    for (bytes, id, value) in [
        (
            include_bytes!("../../../tests/fixtures/table_t1_fraction_original.gsp").as_slice(),
            1,
            3.25,
        ),
        (
            include_bytes!("../../../tests/fixtures/table_t2_fraction_original.gsp").as_slice(),
            2,
            2.5,
        ),
        (
            include_bytes!("../../../tests/fixtures/table_t2_negative_original.gsp").as_slice(),
            2,
            -1.,
        ),
        (
            include_bytes!("../../../tests/fixtures/table_t2_negative_fraction_original.gsp")
                .as_slice(),
            2,
            -2.5,
        ),
        (
            include_bytes!("../../../tests/fixtures/table_t2_multidigit_original.gsp").as_slice(),
            2,
            12.34,
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        let Kind::Parameter(group) = &document.objects[id - 1].kind else {
            panic!("expected parameter")
        };
        assert!((group.value() - value).abs() < 1e-9);
        assert_eq!(document.encode().unwrap(), bytes);
    }
    let mut first = Document::new();
    first.add_default_parameter((10, 5)).unwrap();
    first.set_parameter_value(1, 3.25).unwrap();
    assert_eq!(
        first.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/table_t1_fraction_generated.gsp")
    );
    let mut second = Document::new();
    second.add_default_parameter((10, 5)).unwrap();
    second.add_default_parameter((10, 38)).unwrap();
    second.set_parameter_value(2, -2.5).unwrap();
    assert_eq!(
        second.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/table_t2_negative_fraction_generated.gsp")
    );
    second.set_parameter_value(2, 12.34).unwrap();
    assert_eq!(
        second.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/table_t2_multidigit_generated.gsp")
    );
    second.set_parameter_value(2, 100.).unwrap();
    assert_eq!(second.numeric_value(2), Some(100.));
    second.set_parameter_value(2, 2.555).unwrap();
    assert_eq!(second.numeric_value(2), Some(2.555));
    assert!(second.set_parameter_value(2, f64::NAN).is_err());

    let mut line = Document::new();
    line.add_default_parameter((10, 5)).unwrap();
    line.set_parameter_value(1, 3.25).unwrap();
    line.add(Kind::Free(Point { x: 150., y: 120. }));
    line.add(Kind::Free(Point { x: 180., y: 130. }));
    line.add(Kind::Line(2, 3));
    assert_eq!(line.plot_parameter_on_path(1, Some(4)).unwrap(), 5);
    assert_eq!(line.position(5), Some(Point { x: 247.5, y: 152.5 }));
    assert_eq!(
        line.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/graph_plot_line_fraction_parameter_generated.gsp")
    );
    let reopened = Document::from_bytes(&line.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(5), line.position(5));

    for bytes in [
        include_bytes!("../../../tests/fixtures/table_pair_fraction_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/plot_table_fraction_base_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/plot_table_fraction_original.gsp").as_slice(),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        assert_eq!(document.encode().unwrap(), bytes);
        assert!(
            matches!(&document.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.pair_values() == Some(vec![[2.5, 1.]]))
        );
    }
    let mut table = Document::new();
    table.add_default_parameter((10, 5)).unwrap();
    table.add_default_parameter((10, 38)).unwrap();
    table.tabulate_default_pair((10, 71)).unwrap();
    table.set_parameter_value(2, 2.5).unwrap();
    table
        .add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(
        table
            .plot_pair_table_data(3, PlotPointsMode::Cartesian)
            .unwrap(),
        vec![10]
    );
    assert_eq!(
        table.objects[9].kind,
        Kind::PlottedPoint(9, PlotPointsMode::Cartesian, [2.5, 1.])
    );
    assert!((table.position(10).unwrap().x - (199. + 2.5 * 37.79527559055118)).abs() < 1e-9);
    assert_eq!(
        table.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/plot_table_fraction_generated.gsp")
    );
    let reopened = Document::from_bytes(&table.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(10), table.position(10));
    let plotted = table.position(10);
    table.set_parameter_value(2, 3.25).unwrap();
    assert_eq!(table.position(10), plotted);
    assert!(
        matches!(&table.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.pair_values() == Some(vec![[3.25, 1.]]))
    );
    assert_eq!(
        table.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/plot_table_fraction_changed_generated.gsp")
    );
    let reopened = Document::from_bytes(&table.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(10), plotted);
    let original_changed = include_bytes!(
        "../../../tests/fixtures/plot_table_fraction_after_value_change_original.gsp"
    );
    let original = Document::from_bytes(original_changed, None).unwrap();
    assert!(!original.partial_view);
    assert_eq!(original.encode().unwrap(), original_changed);
    assert_eq!(original.position(10), plotted);
    assert!(
        matches!(&original.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.pair_values() == Some(vec![[3.25, 1.]]))
    );
}

#[test]
fn named_parameters_decode_create_and_edit() {
    for (bytes, name, precision) in [
        (
            include_bytes!("../../../tests/fixtures/table_parameter_custom.gsp").as_slice(),
            "a",
            2,
        ),
        (
            include_bytes!("../../../tests/fixtures/table_parameter_speed_original.gsp").as_slice(),
            "speed",
            1,
        ),
        (
            include_bytes!("../../../tests/fixtures/table_parameter_speed_precision2_original.gsp")
                .as_slice(),
            "speed",
            2,
        ),
        (
            include_bytes!("../../../tests/fixtures/table_parameter_test_original.gsp").as_slice(),
            "test",
            1,
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        let Kind::Parameter(group) = &document.objects[0].kind else {
            panic!("expected parameter")
        };
        assert_eq!(group.name(), name);
        assert_eq!(group.precision(), precision);
        assert_eq!(document.encode().unwrap(), bytes);
    }
    let mut document = Document::new();
    document.add_named_parameter("a", 2.5, (10, 5)).unwrap();
    let generated = document.encode().unwrap();
    let reopened = Document::from_bytes(&generated, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.encode().unwrap(), generated);
    document.set_parameter_value(1, 3.25).unwrap();
    assert_eq!(
        Document::from_bytes(&document.encode().unwrap(), None)
            .unwrap()
            .objects[0]
            .kind,
        document.objects[0].kind
    );
}

#[test]
fn multiple_named_parameters_can_follow_geometry_and_drive_a_plot() {
    let mut document = Document::new();
    assert_eq!(
        document.add_named_parameter("speed", 2.5, (10, 5)).unwrap(),
        1
    );
    document.add(Kind::Free(Point { x: 150., y: 120. }));
    document.add(Kind::Free(Point { x: 180., y: 130. }));
    document.add(Kind::Line(2, 3));
    assert_eq!(
        document
            .add_named_parameter("test", 0.25, (10, 38))
            .unwrap(),
        5
    );
    let before_duplicate = document.encode().unwrap();
    assert!(document.add_named_parameter("speed", 3., (10, 71)).is_err());
    assert_eq!(document.encode().unwrap(), before_duplicate);
    assert_eq!(document.plot_parameter_on_path(5, Some(4)).unwrap(), 6);
    assert_eq!(document.position(6), Some(Point { x: 157.5, y: 122.5 }));
    document.set_parameter_value(5, 0.5).unwrap();
    assert_eq!(document.position(6), Some(Point { x: 165., y: 125. }));
    let encoded = document.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(6), document.position(6));
    assert_eq!(reopened.encode().unwrap(), encoded);
}

#[test]
fn named_parameter_tables_decode_and_save() {
    for (bytes, source, table_id, name, value) in [
        (
            include_bytes!("../../../tests/fixtures/table_named_speed_snapshot_original.gsp")
                .as_slice(),
            1,
            2,
            "speed",
            "2.5",
        ),
        (
            include_bytes!(
                "../../../tests/fixtures/table_named_speed_precision2_snapshot_original.gsp"
            )
            .as_slice(),
            1,
            2,
            "speed",
            "3.25",
        ),
        (
            include_bytes!("../../../tests/fixtures/table_named_test_late_source_original.gsp")
                .as_slice(),
            5,
            7,
            "test",
            "0.25",
        ),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        assert!(
            matches!(&document.objects[table_id - 1].kind, Kind::ValueTable(id, group)
            if *id == source && group.named_label_value() == Some((name.into(), value.into())))
        );
        assert_eq!(document.encode().unwrap(), bytes);
    }
    let mut generated = Document::new();
    generated
        .add_named_parameter("speed", 2.5, (10, 5))
        .unwrap();
    let table_id = generated.tabulate_default_parameter(1, (10, 45)).unwrap();
    assert_eq!(table_id, 2);
    generated.change_table_rows(table_id, true).unwrap();
    assert!(
        matches!(&generated.objects[1].kind, Kind::ValueTable(1, table)
        if table.named_label_values() == Some(("speed".into(), vec!["2.5".into(), "2.5".into()])))
    );
    let encoded = generated.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.encode().unwrap(), encoded);
    generated.set_parameter_value(1, 3.25).unwrap();
    assert_eq!(
        crate::render::number_lines_for_object(&generated, table_id)
            .unwrap()
            .1,
        vec!["speed", "2.5", "3.25"]
    );
    generated.change_table_rows(table_id, true).unwrap();
    assert!(
        matches!(&generated.objects[1].kind, Kind::ValueTable(1, table)
        if table.named_label_values() == Some(("speed".into(), vec!["2.5".into(), "3.25".into(), "3.25".into()])))
    );
    generated.change_table_rows(table_id, false).unwrap();
    assert_eq!(
        crate::render::number_lines_for_object(&generated, table_id)
            .unwrap()
            .1,
        vec!["speed", "2.5", "3.25"]
    );
    let changed = generated.encode().unwrap();
    let reopened = Document::from_bytes(&changed, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.encode().unwrap(), changed);
    let original_two_rows =
        include_bytes!("../../../tests/fixtures/table_named_speed_two_rows_original.gsp");
    let original = Document::from_bytes(original_two_rows, None).unwrap();
    assert!(!original.partial_view);
    assert_eq!(original.encode().unwrap(), original_two_rows);
    assert!(
        matches!(&original.objects[1].kind, Kind::ValueTable(1, table)
        if table.named_label_values() == Some(("speed".into(), vec!["2.5".into(), "2.5".into()])))
    );
    let original_live_add =
        include_bytes!("../../../tests/fixtures/table_named_speed_live_add_original.gsp");
    let original = Document::from_bytes(original_live_add, None).unwrap();
    assert!(!original.partial_view);
    assert_eq!(original.encode().unwrap(), original_live_add);
    assert!(
        matches!(&original.objects[1].kind, Kind::ValueTable(1, table)
        if table.named_label_values() == Some(("speed".into(), vec!["3.25".into(), "3.25".into()])))
    );
}
