use super::*;

#[test]
fn plot_value_on_axis_tracks_its_axis_and_cross_opens() {
    let original_bytes =
        include_bytes!("../../../tests/fixtures/graph_plot_value_axis_original.gsp");
    let mut original = Document::from_bytes(original_bytes, None).unwrap();
    assert!(!original.partial_view);
    assert_eq!(original.objects[8].kind, Kind::PathValuePoint(1, 5));
    assert_eq!(
        original.objects[8].visual_style,
        Some(VisualStyle::Point {
            shape: PointStyle::Medium,
            color: RgbColor {
                red: 255,
                green: 0,
                blue: 0,
            },
        })
    );
    assert_eq!(original.encode().unwrap(), original_bytes);
    let initial = original.position(9).unwrap();
    assert!((initial.x - 236.79527559055118).abs() < 1e-9);
    assert!((initial.y - 125.).abs() < 1e-9);
    original.checkpoint();
    assert!(original.move_point(3, Point { x: 228., y: 140. }));
    let shifted = original.position(9).unwrap();
    assert!((shifted.x - initial.x - 29.).abs() < 1e-9);
    assert!((shifted.y - initial.y - 15.).abs() < 1e-9);
    original.undo();
    assert_eq!(original.position(9), Some(initial));
    original.redo();
    let edited = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
    assert!(!edited.partial_view);
    assert_eq!(edited.position(9), Some(shifted));

    let mut generated = Document::new();
    generated.add_default_parameter((10, 5)).unwrap();
    generated.tabulate_default_parameter(1, (10, 38)).unwrap();
    generated
        .add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(generated.plot_parameter_on_path(1, None).unwrap(), 9);
    let generated_bytes = generated.encode().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_TABLE_FONT_FIXTURES").is_some() {
        fs::write("/tmp/graph_plot_value_axis_font14_writer.gsp", &generated_bytes).unwrap();
    }
    assert_eq!(
        generated_bytes,
        include_bytes!("../../../tests/fixtures/graph_plot_value_axis_font14_writer_native_verified.gsp")
    );
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[8].kind, Kind::PathValuePoint(1, 5));

    let mut no_table = Document::new();
    no_table.add_default_parameter((10, 5)).unwrap();
    no_table
        .add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(no_table.plot_parameter_on_path(1, None).unwrap(), 8);
    assert_eq!(
        no_table.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/graph_plot_value_axis_no_table_generated.gsp")
    );
    no_table.undo();
    assert_eq!(no_table.objects.len(), 7);
    assert_eq!(no_table.plot_parameter_on_path(1, Some(6)).unwrap(), 8);
    assert_eq!(no_table.position(8).unwrap().x, 199.);
    assert!((no_table.position(8).unwrap().y - 87.20472440944882).abs() < 1e-9);
    assert_eq!(
        no_table.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/graph_plot_value_y_generated.gsp")
    );
}

#[test]
fn literal_axis_value_roundtrips_and_follows_axis_movement() {
    let original_bytes =
        include_bytes!("../../../tests/fixtures/graph_plot_value_literal_original.gsp");
    let mut original = Document::from_bytes(original_bytes, None).unwrap();
    assert!(!original.partial_view);
    assert_eq!(original.objects[6].kind, Kind::PathLiteralPoint(3, 2.5));
    assert_eq!(original.encode().unwrap(), original_bytes);
    assert_eq!(
        original.objects[6].visual_style,
        Some(VisualStyle::Point {
            shape: PointStyle::Medium,
            color: RgbColor {
                red: 255,
                green: 0,
                blue: 0,
            },
        })
    );
    let before = original.position(7).unwrap();
    assert!((before.x - (199. + 2.5 * 37.79527559055118)).abs() < 1e-9);
    assert_eq!(before.y, 125.);
    original.checkpoint();
    assert!(original.move_point(1, Point { x: 228., y: 140. }));
    let after = original.position(7).unwrap();
    assert!((after.x - before.x - 29.).abs() < 1e-9);
    assert!((after.y - before.y - 15.).abs() < 1e-9);
    original.undo();
    assert_eq!(original.position(7), Some(before));
    original.redo();
    let edited = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
    assert_eq!(edited.position(7), Some(after));
    original.checkpoint();
    original.objects[6].kind = Kind::PathLiteralPoint(3, 3.);
    let changed_value = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
    assert_eq!(changed_value.objects[6].kind, Kind::PathLiteralPoint(3, 3.));

    let mut x = Document::new();
    x.add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(x.plot_literal_on_path(2.5, None).unwrap(), 7);
    assert_eq!(
        x.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/graph_plot_value_literal_generated.gsp")
    );
    let mut y = Document::new();
    y.add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(y.plot_literal_on_path(-1.25, Some(5)).unwrap(), 7);
    assert_eq!(y.position(7).unwrap().x, 199.);
    assert!((y.position(7).unwrap().y - 172.24409448818898).abs() < 1e-9);
    assert_eq!(
        y.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/graph_plot_value_literal_y_generated.gsp")
    );
    assert!(y.plot_literal_on_path(f64::NAN, None).is_err());
}

#[test]
fn plotted_values_on_segments_track_their_endpoints() {
    let literal_bytes =
        include_bytes!("../../../tests/fixtures/graph_plot_segment_literal_original.gsp");
    let mut literal = Document::from_bytes(literal_bytes, None).unwrap();
    assert!(!literal.partial_view);
    assert_eq!(literal.objects[9].kind, Kind::PathLiteralPoint(3, 0.4));
    assert_eq!(literal.position(10), Some(Point { x: 190., y: 140. }));
    assert_eq!(literal.encode().unwrap(), literal_bytes);
    literal.checkpoint();
    assert!(literal.move_point(2, Point { x: 300., y: 200. }));
    assert_eq!(literal.position(10), Some(Point { x: 210., y: 152. }));
    literal.undo();
    assert_eq!(literal.position(10), Some(Point { x: 190., y: 140. }));
    literal.redo();
    let reopened = Document::from_bytes(&literal.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(10), Some(Point { x: 210., y: 152. }));

    let parameter_bytes =
        include_bytes!("../../../tests/fixtures/graph_plot_segment_parameter_original.gsp");
    let mut parameter = Document::from_bytes(parameter_bytes, None).unwrap();
    assert!(!parameter.partial_view);
    assert_eq!(parameter.objects[4].kind, Kind::PathValuePoint(1, 4));
    assert_eq!(parameter.position(5), Some(Point { x: 250., y: 170. }));
    assert_eq!(parameter.encode().unwrap(), parameter_bytes);
    parameter.checkpoint();
    assert!(parameter.move_point(3, Point { x: 300., y: 200. }));
    let reopened = Document::from_bytes(&parameter.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(5), Some(Point { x: 300., y: 200. }));

    let mut generated_literal = Document::new();
    generated_literal.add(Kind::Free(Point { x: 150., y: 120. }));
    generated_literal.add(Kind::Free(Point { x: 250., y: 170. }));
    generated_literal.add(Kind::Segment(1, 2));
    assert_eq!(
        generated_literal
            .plot_literal_on_path(0.4, Some(3))
            .unwrap(),
        4
    );
    assert!(
        generated_literal
            .plot_literal_on_path(1.2, Some(3))
            .is_err()
    );
    assert_eq!(
        generated_literal.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/graph_plot_segment_literal_generated.gsp")
    );
    let mut generated_parameter = Document::new();
    generated_parameter.add_default_parameter((10, 5)).unwrap();
    generated_parameter.add(Kind::Free(Point { x: 150., y: 120. }));
    generated_parameter.add(Kind::Free(Point { x: 250., y: 170. }));
    generated_parameter.add(Kind::Segment(2, 3));
    assert_eq!(
        generated_parameter
            .plot_parameter_on_path(1, Some(4))
            .unwrap(),
        5
    );
    assert_eq!(
        generated_parameter.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/graph_plot_segment_parameter_generated.gsp")
    );
}

#[test]
fn plotted_values_on_circles_lines_and_rays_roundtrip_and_follow_paths() {
    for (bytes, value, expected) in [
        (
            include_bytes!("../../../tests/fixtures/graph_plot_circle_literal_original.gsp")
                .as_slice(),
            0.25,
            Point {
                x: 323.,
                y: 217. - (27_f64.powi(2) + 115_f64.powi(2)).sqrt(),
            },
        ),
        (
            include_bytes!("../../../tests/fixtures/graph_plot_line_literal_original.gsp")
                .as_slice(),
            -0.5,
            Point { x: 100., y: 95. },
        ),
        (
            include_bytes!("../../../tests/fixtures/graph_plot_ray_literal_original.gsp")
                .as_slice(),
            2.,
            Point { x: 350., y: 220. },
        ),
    ] {
        let mut document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        assert_eq!(document.objects[3].kind, Kind::PathLiteralPoint(3, value));
        assert_eq!(document.encode().unwrap(), bytes);
        let position = document.position(4).unwrap();
        assert!((position.x - expected.x).abs() < 1e-9);
        assert!((position.y - expected.y).abs() < 1e-9);
        let start = document.position(1).unwrap();
        document.checkpoint();
        assert!(document.move_point(
            1,
            Point {
                x: start.x + 10.,
                y: start.y + 20.
            }
        ));
        let moved = document.position(4).unwrap();
        assert!(moved.x.is_finite() && moved.y.is_finite());
        let reopened = Document::from_bytes(&document.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(4), Some(moved));
        document.undo();
        assert_eq!(document.position(4), Some(position));
    }

    let mut circle = Document::new();
    circle.add(Kind::Free(Point { x: 199., y: 125. }));
    circle.add(Kind::Free(Point { x: 236., y: 125. }));
    circle.add(Kind::Circle(1, 2));
    assert_eq!(circle.plot_literal_on_path(0.25, Some(3)).unwrap(), 4);
    assert_eq!(circle.position(4), Some(Point { x: 199., y: 88. }));
    assert_eq!(
        circle.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/graph_plot_circle_literal_generated.gsp")
    );

    for (kind, value, expected, file) in [
        (
            Kind::Line(1, 2),
            -0.5,
            Point { x: 100., y: 95. },
            include_bytes!("../../../tests/fixtures/graph_plot_line_literal_generated.gsp")
                .as_slice(),
        ),
        (
            Kind::Ray(1, 2),
            2.,
            Point { x: 350., y: 220. },
            include_bytes!("../../../tests/fixtures/graph_plot_ray_literal_generated.gsp")
                .as_slice(),
        ),
    ] {
        let mut document = Document::new();
        document.add(Kind::Free(Point { x: 150., y: 120. }));
        document.add(Kind::Free(Point { x: 250., y: 170. }));
        document.add(kind);
        document.plot_literal_on_path(value, Some(3)).unwrap();
        assert_eq!(document.position(4), Some(expected));
        assert_eq!(document.encode().unwrap(), file);
    }
    let mut ray = Document::new();
    ray.add(Kind::Free(Point { x: 150., y: 120. }));
    ray.add(Kind::Free(Point { x: 250., y: 170. }));
    ray.add(Kind::Ray(1, 2));
    assert!(ray.plot_literal_on_path(-0.5, Some(3)).is_err());

    let mut live_circle = Document::new();
    live_circle.add_default_parameter((10, 5)).unwrap();
    live_circle.add(Kind::Free(Point { x: 199., y: 125. }));
    live_circle.add(Kind::Free(Point { x: 236., y: 125. }));
    live_circle.add(Kind::Circle(2, 3));
    assert_eq!(live_circle.plot_parameter_on_path(1, Some(4)).unwrap(), 5);
    assert!((live_circle.position(5).unwrap().x - 236.).abs() < 1e-9);
    assert_eq!(
        live_circle.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/graph_plot_circle_parameter_generated.gsp")
    );
    live_circle.checkpoint();
    assert!(live_circle.move_point(3, Point { x: 249., y: 125. }));
    assert!((live_circle.position(5).unwrap().x - 249.).abs() < 1e-9);
}

#[test]
fn two_parameter_table_data_roundtrips_and_tracks_frame() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/table_two_parameters_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/table_two_columns_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/plot_table_data_base_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/plot_table_data_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/table_two_columns_two_rows_original.gsp")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/plot_table_data_two_rows_base_original.gsp")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/plot_table_data_two_rows_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/table_t2_value2_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/table_two_columns_distinct_rows_original.gsp")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/plot_table_distinct_base_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/plot_table_distinct_original.gsp").as_slice(),
    ] {
        let document = Document::from_bytes(bytes, None).unwrap();
        assert!(!document.partial_view);
        assert_eq!(document.encode().unwrap(), bytes);
        assert!(
            matches!(&document.objects[0].kind, Kind::Parameter(group) if group.name() == "t[1]")
        );
        assert!(
            matches!(&document.objects[1].kind, Kind::Parameter(group) if group.name() == "t[2]")
        );
    }
    let mut original = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/plot_table_data_original.gsp"),
        None,
    )
    .unwrap();
    assert!(
        matches!(&original.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.rows() == 1)
    );
    assert_eq!(
        original.objects[9].kind,
        Kind::PlottedPoint(1, 9, PlotPointsMode::Cartesian, [1., 1.])
    );
    let before = original.position(10).unwrap();
    assert!((before.x - 236.79527559055118).abs() < 1e-9);
    assert!((before.y - 87.20472440944882).abs() < 1e-9);
    original.move_number(3, (25, 80)).unwrap();
    assert!(
        matches!(&original.objects[2].kind, Kind::ValueTablePair(_, _, group) if group.anchor() == (25, 80))
    );
    let moved = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
    assert!(!moved.partial_view);
    assert_eq!(moved.position(10), Some(before));
    original.checkpoint();
    assert!(original.move_point(4, Point { x: 219., y: 135. }));
    let shifted = original.position(10).unwrap();
    assert!((shifted.x - before.x - 20.).abs() < 1e-9);
    assert!((shifted.y - before.y - 10.).abs() < 1e-9);
    original.undo();
    assert_eq!(original.position(10), Some(before));

    let mut generated = Document::new();
    generated.add_default_parameter((10, 5)).unwrap();
    generated.add_default_parameter((10, 38)).unwrap();
    generated.tabulate_default_pair((10, 71)).unwrap();
    generated
        .add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(
        generated
            .plot_pair_table_data(3, PlotPointsMode::Cartesian)
            .unwrap(),
        vec![10]
    );
    assert_eq!(
        generated.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/plot_table_data_generated.gsp")
    );
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(10), Some(before));

    let mut two_rows = Document::new();
    two_rows.add_default_parameter((10, 5)).unwrap();
    two_rows.add_default_parameter((10, 38)).unwrap();
    two_rows.tabulate_default_pair((10, 71)).unwrap();
    two_rows.change_table_rows(3, true).unwrap();
    assert!(
        matches!(&two_rows.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.rows() == 2)
    );
    two_rows
        .add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(
        two_rows
            .plot_pair_table_data(3, PlotPointsMode::Cartesian)
            .unwrap(),
        vec![10]
    );
    assert_eq!(
        two_rows.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/plot_table_data_two_rows_generated.gsp")
    );
    assert_eq!(two_rows.position(10), Some(before));
    let restored = Document::from_bytes(&two_rows.encode().unwrap(), None).unwrap();
    assert!(!restored.partial_view);
    assert!(
        matches!(&restored.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.rows() == 2)
    );

    let distinct_original = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/plot_table_distinct_original.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(distinct_original.objects.len(), 11);
    assert!(
        matches!(&distinct_original.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.pair_values() == Some(vec![[1., 1.], [2., 1.]]))
    );
    assert_eq!(
        distinct_original.objects[9].kind,
        Kind::PlottedPoint(1, 9, PlotPointsMode::Cartesian, [1., 1.])
    );
    assert_eq!(
        distinct_original.objects[10].kind,
        Kind::PlottedPoint(1, 9, PlotPointsMode::Cartesian, [2., 1.])
    );
    assert!(
        (distinct_original.position(11).unwrap().x - (199. + 2. * 37.79527559055118)).abs() < 1e-9
    );

    let mut distinct = Document::new();
    distinct.add_default_parameter((10, 5)).unwrap();
    distinct.add_default_parameter((10, 38)).unwrap();
    distinct.tabulate_default_pair((10, 71)).unwrap();
    distinct.change_table_rows(3, true).unwrap();
    distinct.set_parameter_value(2, 2.).unwrap();
    assert!(matches!(&distinct.objects[1].kind, Kind::Parameter(group) if group.value() == 2.));
    assert!(
        matches!(&distinct.objects[2].kind, Kind::ValueTablePair(2, 1, group) if group.pair_values() == Some(vec![[1., 1.], [2., 1.]]))
    );
    distinct
        .add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(
        distinct
            .plot_pair_table_data(3, PlotPointsMode::Cartesian)
            .unwrap(),
        vec![10, 11]
    );
    assert_eq!(
        distinct.encode().unwrap(),
        include_bytes!("../../../tests/fixtures/plot_table_distinct_generated.gsp")
    );
    let reopened = Document::from_bytes(&distinct.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(11), distinct.position(11));
}

#[test]
fn calculations_import_and_append_after_measurements() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/calculation_two_plus_three_native.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/calculation_seven_plus_four_native.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/calculation_seven_minus_four_native.gsp")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/calculation_seven_times_four_native.gsp")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/calculation_seven_divide_four_native.gsp")
            .as_slice(),
    ] {
        let imported = Document::from_bytes(bytes, None).unwrap();
        assert!(!imported.partial_view);
        assert_eq!(imported.encode().unwrap(), bytes);
    }
    for (expression, expected, value) in [
        (
            "7 * 4",
            include_bytes!("../../../tests/fixtures/calculation_seven_times_four_port.gsp")
                .as_slice(),
            28.,
        ),
        (
            "7 / 4",
            include_bytes!("../../../tests/fixtures/calculation_seven_divide_four_port.gsp")
                .as_slice(),
            1.75,
        ),
    ] {
        let mut document = Document::new();
        let id = document
            .add_calculation_expression(expression, (10, 38))
            .unwrap();
        assert_eq!(document.encode().unwrap(), expected);
        let reopened = Document::from_bytes(expected, None).unwrap();
        assert!(matches!(&reopened.objects[id - 1].kind,
            Kind::Calculation(group) if group.value() == value));
    }
    let mut measured = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/wine_segment_length.gsp"),
        None,
    )
    .unwrap();
    let id = measured
        .add_calculation_expression("9 - 2", (10, 38))
        .unwrap();
    let bytes = measured.encode().unwrap();
    assert_eq!(
        bytes,
        include_bytes!("../../../tests/fixtures/measurement_and_calculation_port.gsp")
    );
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert!(matches!(&reopened.objects[id - 1].kind,
        Kind::Calculation(group) if group.value() == 7.));
}

#[test]
fn default_table_keeps_fractional_snapshots_and_native_edit() {
    let original = include_bytes!("../../../tests/fixtures/table_t1_tabulated_edited_original.gsp");
    let imported = Document::from_bytes(original, None).unwrap();
    assert!(!imported.partial_view);
    assert!(matches!(
        &imported.objects[1].kind,
        Kind::ValueTable(1, group) if group.default_values() == Some(vec![2.5])
    ));
    assert_eq!(imported.encode().unwrap(), original);

    let mut doc = Document::new();
    doc.add_default_parameter((10, 5)).unwrap();
    doc.set_parameter_value(1, 2.5).unwrap();
    doc.tabulate_default_parameter(1, (10, 38)).unwrap();
    doc.change_table_rows(2, true).unwrap();
    doc.set_parameter_value(1, 3.25).unwrap();
    assert!(matches!(
        &doc.objects[1].kind,
        Kind::ValueTable(1, group) if group.default_values() == Some(vec![2.5, 3.25])
    ));
    let bytes = doc.encode().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_TABLE_FONT_FIXTURES").is_some() {
        fs::write("/tmp/table_t1_snapshots_font14_writer.gsp", &bytes).unwrap();
    }
    assert_eq!(
        bytes,
        include_bytes!("../../../tests/fixtures/table_t1_snapshots_font14_writer_native_verified.gsp")
    );
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[1].kind, doc.objects[1].kind);

    doc.set_parameter_value(1, 4.5).unwrap();
    assert!(matches!(
        &doc.objects[1].kind,
        Kind::ValueTable(1, group) if group.default_values() == Some(vec![2.5, 4.5])
    ));
    doc.change_table_rows(2, false).unwrap();
    assert!(matches!(
        &doc.objects[1].kind,
        Kind::ValueTable(1, group) if group.default_values() == Some(vec![4.5])
    ));
}

#[test]
fn default_table_accepts_later_parameter_source() {
    let mut doc = Document::new();
    assert_eq!(doc.add(Kind::Free(Point { x: 300., y: 300. })), 1);
    assert_eq!(doc.add_default_parameter((10, 5)).unwrap(), 2);
    doc.set_parameter_value(2, 2.5).unwrap();
    assert_eq!(doc.tabulate_default_parameter(2, (10, 38)).unwrap(), 3);
    doc.change_table_rows(3, true).unwrap();
    doc.set_parameter_value(2, 3.25).unwrap();
    assert!(matches!(
        &doc.objects[2].kind,
        Kind::ValueTable(2, group)
            if group.source_id() == 2 && group.default_values() == Some(vec![2.5, 3.25])
    ));
    let bytes = doc.encode().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_TABLE_FONT_FIXTURES").is_some() {
        fs::write("/tmp/table_t1_late_source_snapshots_font14_writer.gsp", &bytes).unwrap();
    }
    assert_eq!(
        bytes,
        include_bytes!("../../../tests/fixtures/table_t1_late_source_snapshots_font14_writer_native_verified.gsp")
    );
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[2].kind, doc.objects[2].kind);

    doc.set_parameter_value(2, 4.5).unwrap();
    assert!(matches!(
        &doc.objects[2].kind,
        Kind::ValueTable(2, group) if group.default_values() == Some(vec![2.5, 4.5])
    ));
    doc.change_table_rows(3, false).unwrap();
    assert!(matches!(
        &doc.objects[2].kind,
        Kind::ValueTable(2, group) if group.default_values() == Some(vec![4.5])
    ));
}

#[test]
fn default_table_preserves_native_wide_values_and_mixed_snapshots() {
    for (bytes, value) in [
        (
            include_bytes!("../../../tests/fixtures/table_t1_tabulated_negative_original.gsp")
                .as_slice(),
            -2.5,
        ),
        (
            include_bytes!("../../../tests/fixtures/table_t1_tabulated_two_digit_original.gsp")
                .as_slice(),
            12.34,
        ),
    ] {
        let imported = Document::from_bytes(bytes, None).unwrap();
        assert!(!imported.partial_view);
        assert!(matches!(
            &imported.objects[1].kind,
            Kind::ValueTable(1, group) if group.default_values() == Some(vec![value])
        ));
        assert_eq!(imported.encode().unwrap(), bytes);
    }

    let mut doc = Document::new();
    doc.add_default_parameter((10, 5)).unwrap();
    doc.set_parameter_value(1, -2.5).unwrap();
    doc.tabulate_default_parameter(1, (10, 38)).unwrap();
    doc.change_table_rows(2, true).unwrap();
    doc.set_parameter_value(1, 12.34).unwrap();
    doc.change_table_rows(2, true).unwrap();
    doc.set_parameter_value(1, -99.99).unwrap();
    assert!(matches!(
        &doc.objects[1].kind,
        Kind::ValueTable(1, group)
            if group.default_values() == Some(vec![-2.5, 12.34, -99.99])
    ));
    let bytes = doc.encode().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_TABLE_FONT_FIXTURES").is_some() {
        fs::write("/tmp/table_t1_mixed_width_snapshots_font14_writer.gsp", &bytes).unwrap();
    }
    assert_eq!(
        bytes,
        include_bytes!("../../../tests/fixtures/table_t1_mixed_width_snapshots_font14_writer_native_verified.gsp")
    );
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[1].kind, doc.objects[1].kind);

    doc.set_parameter_value(1, 99.99).unwrap();
    assert!(matches!(
        &doc.objects[1].kind,
        Kind::ValueTable(1, group)
            if group.default_values() == Some(vec![-2.5, 12.34, 99.99])
    ));
    doc.change_table_rows(2, false).unwrap();
    assert!(matches!(
        &doc.objects[1].kind,
        Kind::ValueTable(1, group)
            if group.default_values() == Some(vec![-2.5, 99.99])
    ));
}
