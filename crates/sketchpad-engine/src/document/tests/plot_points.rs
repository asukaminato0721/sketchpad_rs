use super::*;

#[test]
fn plotted_points_open_move_and_reopen() {
    for (bytes, mode) in [
        (
            include_bytes!("../../../../../tests/fixtures/plot_points_table_single_original.gsp")
                .as_slice(),
            PlotPointsMode::Cartesian,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/plot_points_table_polar_original.gsp")
                .as_slice(),
            PlotPointsMode::Polar,
        ),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert!(matches!(doc.objects[8].kind, Kind::PlottedPoint(8, m, _) if m == mode));
        assert_eq!(
            doc.objects[8].visual_style,
            Some(VisualStyle::Point {
                shape: PointStyle::Medium,
                color: RgbColor {
                    red: 255,
                    green: 0,
                    blue: 0
                },
            })
        );
        let before = doc.position(9).unwrap();
        doc.checkpoint();
        assert!(doc.move_point(3, Point { x: 228., y: 140. }));
        let after = doc.position(9).unwrap();
        assert!((after.x - before.x - 29.).abs() < 1e-9);
        assert!((after.y - before.y - 15.).abs() < 1e-9);
        doc.undo();
        assert_eq!(doc.position(9), Some(before));
        doc.redo();
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(9), Some(after));
    }
}

#[test]
fn new_plotted_points_save_and_reopen() {
    fn last_object_group(bytes: &[u8]) -> &[u8] {
        let entries = records(bytes).unwrap();
        let start = entries.iter().rfind(|r| r.kind == 0x07d0).unwrap().start;
        let end = entries.iter().rfind(|r| r.kind == 0x07d7).unwrap().data.end;
        &bytes[start..end]
    }
    for (mode, rows) in [
        (PlotPointsMode::Cartesian, 1),
        (PlotPointsMode::Polar, 1),
        (PlotPointsMode::Cartesian, 2),
    ] {
        let mut doc = Document::new();
        assert_eq!(doc.add_default_parameter((10, 5)).unwrap(), 1);
        assert_eq!(doc.tabulate_default_parameter(1, (10, 38)).unwrap(), 2);
        if rows == 2 {
            doc.change_table_rows(2, true).unwrap();
        }
        assert_eq!(
            doc.add_default_coordinate_system(Point { x: 199., y: 125. })
                .unwrap(),
            8
        );
        assert_eq!(doc.plot_points_at(mode, [1., 1.]).unwrap(), 9);
        assert_eq!(
            doc.objects[8].visual_style,
            Some(VisualStyle::Point {
                shape: PointStyle::Medium,
                color: RgbColor {
                    red: 255,
                    green: 0,
                    blue: 0
                },
            })
        );
        let expected = doc.position(9).unwrap();
        let saved = doc.encode().unwrap();
        let original = match mode {
            PlotPointsMode::Cartesian => {
                include_bytes!("../../../../../tests/fixtures/plot_points_table_single_original.gsp")
                    .as_slice()
            }
            PlotPointsMode::Polar => {
                include_bytes!("../../../../../tests/fixtures/plot_points_table_polar_original.gsp")
                    .as_slice()
            }
        };
        assert_eq!(last_object_group(&saved), last_object_group(original));
        let reopened = Document::from_bytes(&saved, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects.len(), 9);
        assert_eq!(reopened.position(9), Some(expected));
    }
}

#[test]
fn named_parameter_table_plot_matches_original_point_group() {
    let original = include_bytes!("../../../../../tests/fixtures/plot_points_named_speed_original.gsp");
    let native = Document::from_bytes(original, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), original);
    assert!(matches!(
        native.objects[8].kind,
        Kind::PlottedPoint(8, PlotPointsMode::Cartesian, [1.0, 1.0])
    ));
    let mut doc = Document::new();
    doc.add_named_parameter("speed", 2.5, (10, 5)).unwrap();
    doc.tabulate_default_parameter(1, (10, 45)).unwrap();
    doc.add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(
        doc.plot_points_at(PlotPointsMode::Cartesian, [1., 1.]),
        Ok(9)
    );
    let saved = doc.encode().unwrap();
    let last = |bytes: &[u8]| {
        let entries = records(bytes).unwrap();
        let start = entries.iter().rfind(|r| r.kind == 0x07d0).unwrap().start;
        let end = entries.iter().rfind(|r| r.kind == 0x07d7).unwrap().data.end;
        bytes[start..end].to_vec()
    };
    assert_eq!(last(&saved), last(original));
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(9), native.position(9));
}

#[test]
fn custom_named_plot_coordinates_match_original_groups() {
    let original_cases: [(PlotPointsMode, [f64; 2], &[u8]); 2] = [
        (
            PlotPointsMode::Cartesian,
            [2.5, -1.25],
            include_bytes!(
                "../../../../../tests/fixtures/plot_points_named_speed_custom_cartesian_original.gsp"
            ),
        ),
        (
            PlotPointsMode::Polar,
            [2.5, 30.0],
            include_bytes!(
                "../../../../../tests/fixtures/plot_points_named_speed_custom_polar_original.gsp"
            ),
        ),
    ];
    for (mode, coordinates, original) in original_cases {
        let native = Document::from_bytes(original, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), original);
        let mut doc = Document::new();
        doc.add_named_parameter("speed", 2.5, (10, 5)).unwrap();
        doc.tabulate_default_parameter(1, (10, 45)).unwrap();
        doc.add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(doc.plot_points_at(mode, coordinates), Ok(9));
        let saved = doc.encode().unwrap();
        let last = |bytes: &[u8]| {
            let entries = records(bytes).unwrap();
            let start = entries.iter().rfind(|r| r.kind == 0x07d0).unwrap().start;
            let end = entries.iter().rfind(|r| r.kind == 0x07d7).unwrap().data.end;
            bytes[start..end].to_vec()
        };
        assert_eq!(last(&saved), last(original));
        let reopened = Document::from_bytes(&saved, None).unwrap();
        assert_eq!(reopened.position(9), native.position(9));
    }
}

#[test]
fn repeated_plot_points_round_trip_and_reject_invalid_coordinates() {
    let mut doc = Document::new();
    doc.add_named_parameter("speed", 2.5, (10, 5)).unwrap();
    doc.tabulate_default_parameter(1, (10, 45)).unwrap();
    doc.add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(
        doc.plot_points_at(PlotPointsMode::Cartesian, [2.5, -1.25]),
        Ok(9)
    );
    assert_eq!(
        doc.plot_points_at(PlotPointsMode::Polar, [2.5, 30.0]),
        Ok(10)
    );
    let before_invalid = doc.encode().unwrap();
    assert!(
        doc.plot_points_at(PlotPointsMode::Cartesian, [f64::NAN, 1.0])
            .is_err()
    );
    assert_eq!(doc.encode().unwrap(), before_invalid);
    doc.undo();
    assert_eq!(doc.objects.len(), 9);
    doc.redo();
    assert_eq!(doc.objects.len(), 10);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(9), doc.position(9));
    assert_eq!(reopened.position(10), doc.position(10));
}

#[test]
fn frame_only_plot_point_matches_original() {
    let original = include_bytes!("../../../../../tests/fixtures/plot_points_frame_only_original.gsp");
    let native = Document::from_bytes(original, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), original);
    assert!(matches!(
        native.objects[6].kind,
        Kind::PlottedPoint(6, PlotPointsMode::Cartesian, [1.0, 1.0])
    ));
    let mut doc = Document::new();
    doc.add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(
        doc.plot_points_at(PlotPointsMode::Cartesian, [1.0, 1.0]),
        Ok(7)
    );
    let generated = doc.encode().unwrap();
    let last = |bytes: &[u8]| {
        let entries = records(bytes).unwrap();
        let start = entries.iter().rfind(|r| r.kind == 0x07d0).unwrap().start;
        let end = entries.iter().rfind(|r| r.kind == 0x07d7).unwrap().data.end;
        bytes[start..end].to_vec()
    };
    assert_eq!(last(&generated), last(original));
    let reopened = Document::from_bytes(&generated, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(7), native.position(7));
}

#[test]
fn plot_points_anchor_to_first_object_in_mixed_sketches() {
    for (original, text_first) in [
        (
            include_bytes!("../../../../../tests/fixtures/plot_points_prepoint_original.gsp").as_slice(),
            false,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/plot_points_pretext_original.gsp").as_slice(),
            true,
        ),
    ] {
        let native = Document::from_bytes(original, None).unwrap();
        assert!(!native.partial_view);
        assert_eq!(native.encode().unwrap(), original);
        assert!(matches!(
            native.objects[7].kind,
            Kind::PlottedPoint(7, PlotPointsMode::Cartesian, [1.0, 1.0])
        ));
        let mut doc = Document::new();
        if text_first {
            doc.add_text("hello", Point { x: 10., y: 5. }).unwrap();
        } else {
            doc.add(Kind::Free(Point { x: 100., y: 100. }));
        }
        doc.add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(
            doc.plot_points_at(PlotPointsMode::Cartesian, [1., 1.]),
            Ok(8)
        );
        let generated = doc.encode().unwrap();
        let last = |bytes: &[u8]| {
            let entries = records(bytes).unwrap();
            let start = entries.iter().rfind(|r| r.kind == 0x07d0).unwrap().start;
            let end = entries.iter().rfind(|r| r.kind == 0x07d7).unwrap().data.end;
            bytes[start..end].to_vec()
        };
        assert_eq!(last(&generated), last(original));
        let reopened = Document::from_bytes(&generated, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.position(8), native.position(8));
    }
}

#[test]
fn polar_table_data_matches_original_point_group() {
    let original = include_bytes!("../../../../../tests/fixtures/plot_table_fraction_polar_original.gsp");
    let native = Document::from_bytes(original, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), original);
    assert!(matches!(
        native.objects[9].kind,
        Kind::PlottedPoint(9, PlotPointsMode::Polar, [2.5, angle])
            if angle == plot_points_records::DEGREES_TO_RADIANS
    ));
    let base = include_bytes!("../../../../../tests/fixtures/plot_table_fraction_base_original.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(
        generated.plot_pair_table_data(3, PlotPointsMode::Polar),
        Ok(vec![10])
    );
    let saved = generated.encode().unwrap();
    let last = |bytes: &[u8]| {
        let entries = records(bytes).unwrap();
        let start = entries.iter().rfind(|r| r.kind == 0x07d0).unwrap().start;
        let end = entries.iter().rfind(|r| r.kind == 0x07d7).unwrap().data.end;
        bytes[start..end].to_vec()
    };
    assert_eq!(last(&saved), last(original));
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert_eq!(reopened.position(10), native.position(10));
}

#[test]
fn distinct_polar_table_rows_plot_and_round_trip() {
    let original = include_bytes!("../../../../../tests/fixtures/plot_table_distinct_polar_original.gsp");
    let native = Document::from_bytes(original, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), original);
    for (id, radius) in [(10, 1.0), (11, 2.0)] {
        assert!(matches!(
            native.objects[id - 1].kind,
            Kind::PlottedPoint(9, PlotPointsMode::Polar, [r, angle])
                if r == radius && angle == plot_points_records::DEGREES_TO_RADIANS
        ));
    }
    let base = include_bytes!("../../../../../tests/fixtures/plot_table_distinct_base_original.gsp");
    let mut generated = Document::from_bytes(base, None).unwrap();
    assert_eq!(
        generated.plot_pair_table_data(3, PlotPointsMode::Polar),
        Ok(vec![10, 11])
    );
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    for id in [10, 11] {
        assert_eq!(reopened.objects[id - 1].kind, native.objects[id - 1].kind);
        assert_eq!(reopened.position(id), native.position(id));
    }
}

#[test]
fn three_row_pair_table_edits_live_row_and_plots_distinct_values() {
    let original =
        include_bytes!("../../../../../tests/fixtures/table_pair_three_rows_distinct_original.gsp");
    let native = Document::from_bytes(original, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.encode().unwrap(), original);
    assert!(matches!(
        &native.objects[2].kind,
        Kind::ValueTablePair(2, 1, group)
            if group.pair_values() == Some(vec![[1., 1.], [2., 1.], [2., 1.]])
    ));

    let base = include_bytes!("../../../../../tests/fixtures/plot_table_distinct_base_original.gsp");
    let mut doc = Document::from_bytes(base, None).unwrap();
    doc.change_table_rows(3, true).unwrap();
    assert!(matches!(
        &doc.objects[2].kind,
        Kind::ValueTablePair(2, 1, group)
            if group.pair_values() == Some(vec![[1., 1.], [2., 1.], [2., 1.]])
    ));
    doc.set_parameter_value(2, 3.25).unwrap();
    assert!(matches!(
        &doc.objects[2].kind,
        Kind::ValueTablePair(2, 1, group)
            if group.pair_values() == Some(vec![[1., 1.], [2., 1.], [3.25, 1.]])
    ));
    assert_eq!(
        doc.plot_pair_table_data(3, PlotPointsMode::Polar),
        Ok(vec![10, 11, 12])
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[11].kind, doc.objects[11].kind);
    assert_eq!(reopened.position(12), doc.position(12));
}

#[test]
fn five_distinct_pair_rows_plot_and_stay_fixed_after_parameter_edit() {
    let native_four = include_bytes!("../../../../../tests/fixtures/table_pair_four_rows_original.gsp");
    let imported = Document::from_bytes(native_four, None).unwrap();
    assert!(!imported.partial_view);
    assert_eq!(imported.encode().unwrap(), native_four);
    let cross_opened =
        include_bytes!("../../../../../tests/fixtures/table_pair_five_distinct_polar_generated.gsp");
    let imported = Document::from_bytes(cross_opened, None).unwrap();
    assert!(!imported.partial_view);
    assert_eq!(imported.encode().unwrap(), cross_opened);

    let base = include_bytes!("../../../../../tests/fixtures/plot_table_distinct_base_original.gsp");
    let mut doc = Document::from_bytes(base, None).unwrap();
    for next in [3.25, 4.5, 5.75] {
        doc.change_table_rows(3, true).unwrap();
        doc.set_parameter_value(2, next).unwrap();
    }
    assert!(matches!(
        &doc.objects[2].kind,
        Kind::ValueTablePair(2, 1, group)
            if group.pair_values() == Some(vec![[1., 1.], [2., 1.], [3.25, 1.], [4.5, 1.], [5.75, 1.]])
    ));
    assert_eq!(
        doc.plot_pair_table_data(3, PlotPointsMode::Polar),
        Ok(vec![10, 11, 12, 13, 14])
    );
    let plotted = (10..=14)
        .map(|id| doc.position(id).unwrap())
        .collect::<Vec<_>>();
    doc.set_parameter_value(2, 6.0).unwrap();
    assert!(matches!(
        &doc.objects[2].kind,
        Kind::ValueTablePair(2, 1, group)
            if group.pair_values().unwrap().last() == Some(&[6., 1.])
    ));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        (10..=14)
            .map(|id| reopened.position(id).unwrap())
            .collect::<Vec<_>>(),
        plotted
    );
}

#[test]
fn removing_pair_rows_preserves_earlier_cached_values() {
    let five =
        include_bytes!("../../../../../tests/fixtures/table_pair_five_distinct_polar_generated.gsp");
    let mut plotted_doc = Document::from_bytes(five, None).unwrap();
    let positions = (10..=14)
        .map(|id| plotted_doc.position(id).unwrap())
        .collect::<Vec<_>>();
    plotted_doc.change_table_rows(3, false).unwrap();
    assert_eq!(
        plotted_doc.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/table_pair_four_rows_after_remove_generated.gsp")
    );
    assert_eq!(
        (10..=14)
            .map(|id| plotted_doc.position(id).unwrap())
            .collect::<Vec<_>>(),
        positions
    );

    let base = include_bytes!("../../../../../tests/fixtures/plot_table_distinct_base_original.gsp");
    let mut doc = Document::from_bytes(base, None).unwrap();
    for next in [3.25, 4.5, 5.75] {
        doc.change_table_rows(3, true).unwrap();
        doc.set_parameter_value(2, next).unwrap();
    }
    doc.change_table_rows(3, false).unwrap();
    assert!(matches!(
        &doc.objects[2].kind,
        Kind::ValueTablePair(2, 1, group)
            if group.pair_values() == Some(vec![[1., 1.], [2., 1.], [3.25, 1.], [5.75, 1.]])
    ));
    doc.change_table_rows(3, false).unwrap();
    assert!(matches!(
        &doc.objects[2].kind,
        Kind::ValueTablePair(2, 1, group)
            if group.pair_values() == Some(vec![[1., 1.], [2., 1.], [5.75, 1.]])
    ));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[2].kind, doc.objects[2].kind);

    doc.change_table_rows(3, false).unwrap();
    doc.change_table_rows(3, false).unwrap();
    assert!(doc.change_table_rows(3, false).is_err());
    assert!(matches!(
        &doc.objects[2].kind,
        Kind::ValueTablePair(2, 1, group)
            if group.pair_values() == Some(vec![[5.75, 1.]])
    ));
}

#[test]
fn pair_table_can_be_plotted_again_after_parameter_edit() {
    let five =
        include_bytes!("../../../../../tests/fixtures/table_pair_five_distinct_polar_generated.gsp");
    let mut doc = Document::from_bytes(five, None).unwrap();
    let first_positions = (10..=14)
        .map(|id| doc.position(id).unwrap())
        .collect::<Vec<_>>();
    doc.set_parameter_value(2, 6.0).unwrap();
    assert_eq!(
        doc.plot_pair_table_data(3, PlotPointsMode::Polar),
        Ok(vec![15, 16, 17, 18, 19])
    );
    assert_eq!(
        (10..=14)
            .map(|id| doc.position(id).unwrap())
            .collect::<Vec<_>>(),
        first_positions
    );
    let bytes = doc.encode().unwrap();
    assert_eq!(
        bytes,
        include_bytes!("../../../../../tests/fixtures/table_pair_replotted_polar_generated.gsp")
    );
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(19), doc.position(19));
}

#[test]
fn pair_table_uses_later_parameter_and_frame_ids() {
    let mut doc = Document::new();
    assert_eq!(doc.add(Kind::Free(Point { x: 300., y: 300. })), 1);
    assert_eq!(doc.add_default_parameter((10, 5)).unwrap(), 2);
    assert_eq!(doc.add_default_parameter((10, 38)).unwrap(), 3);
    doc.set_parameter_value(3, 2.0).unwrap();
    assert_eq!(doc.tabulate_default_pair((10, 71)).unwrap(), 4);
    doc.change_table_rows(4, true).unwrap();
    doc.set_parameter_value(3, 3.25).unwrap();
    assert!(matches!(
        &doc.objects[3].kind,
        Kind::ValueTablePair(3, 2, group)
            if group.pair_source_ids() == Some([3, 2])
                && group.pair_values() == Some(vec![[2., 1.], [3.25, 1.]])
    ));
    assert_eq!(
        doc.add_default_coordinate_system(Point { x: 199., y: 125. })
            .unwrap(),
        10
    );
    assert_eq!(
        doc.plot_pair_table_data(4, PlotPointsMode::Polar),
        Ok(vec![11, 12])
    );
    let positions = [doc.position(11).unwrap(), doc.position(12).unwrap()];
    let bytes = doc.encode().unwrap();
    assert_eq!(
        bytes,
        include_bytes!("../../../../../tests/fixtures/table_pair_late_sources_polar_generated.gsp")
    );
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(11), Some(positions[0]));
    assert_eq!(reopened.position(12), Some(positions[1]));
}

#[test]
fn pair_table_caches_and_plots_both_parameter_values() {
    let mut doc = Document::new();
    doc.add_default_parameter((10, 5)).unwrap();
    doc.add_default_parameter((10, 38)).unwrap();
    doc.set_parameter_value(1, 2.5).unwrap();
    doc.set_parameter_value(2, 3.25).unwrap();
    doc.tabulate_default_pair((10, 71)).unwrap();
    doc.change_table_rows(3, true).unwrap();
    doc.set_parameter_value(1, 4.0).unwrap();
    doc.set_parameter_value(2, 5.75).unwrap();
    assert!(matches!(
        &doc.objects[2].kind,
        Kind::ValueTablePair(2, 1, group)
            if group.pair_values() == Some(vec![[3.25, 2.5], [5.75, 4.0]])
    ));
    doc.add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(
        doc.plot_pair_table_data(3, PlotPointsMode::Cartesian),
        Ok(vec![10, 11])
    );
    let positions = [doc.position(10).unwrap(), doc.position(11).unwrap()];
    let bytes = doc.encode().unwrap();
    assert_eq!(
        bytes,
        include_bytes!("../../../../../tests/fixtures/table_pair_both_columns_cartesian_generated.gsp")
    );
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.position(10), Some(positions[0]));
    assert_eq!(reopened.position(11), Some(positions[1]));

    doc.set_parameter_value(1, 4.5).unwrap();
    assert!(matches!(
        &doc.objects[2].kind,
        Kind::ValueTablePair(2, 1, group)
            if group.pair_values() == Some(vec![[3.25, 2.5], [5.75, 4.5]])
    ));
    assert_eq!(doc.position(10), Some(positions[0]));
    assert_eq!(doc.position(11), Some(positions[1]));
    doc.change_table_rows(3, false).unwrap();
    assert!(matches!(
        &doc.objects[2].kind,
        Kind::ValueTablePair(2, 1, group)
            if group.pair_values() == Some(vec![[5.75, 4.5]])
    ));
}

#[test]
fn pair_table_preserves_native_wide_cells_and_plots_mixed_rows() {
    for (bytes, expected) in [
        (
            include_bytes!("../../../../../tests/fixtures/table_pair_negative_x_original.gsp").as_slice(),
            [-2.5, 1.0],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/table_pair_two_digit_y_original.gsp")
                .as_slice(),
            [1.0, 12.34],
        ),
    ] {
        let imported = Document::from_bytes(bytes, None).unwrap();
        assert!(!imported.partial_view);
        assert!(matches!(
            &imported.objects[2].kind,
            Kind::ValueTablePair(2, 1, group)
                if group.pair_values() == Some(vec![expected])
        ));
        assert_eq!(imported.encode().unwrap(), bytes);
    }

    let mut doc = Document::new();
    doc.add_default_parameter((10, 5)).unwrap();
    doc.add_default_parameter((10, 38)).unwrap();
    doc.set_parameter_value(1, 12.34).unwrap();
    doc.set_parameter_value(2, -2.5).unwrap();
    doc.tabulate_default_pair((10, 71)).unwrap();
    doc.change_table_rows(3, true).unwrap();
    doc.set_parameter_value(1, -3.25).unwrap();
    doc.set_parameter_value(2, 12.34).unwrap();
    doc.change_table_rows(3, true).unwrap();
    doc.set_parameter_value(1, 2.5).unwrap();
    doc.set_parameter_value(2, 3.25).unwrap();
    assert!(matches!(
        &doc.objects[2].kind,
        Kind::ValueTablePair(2, 1, group)
            if group.pair_values()
                == Some(vec![[-2.5, 12.34], [12.34, -3.25], [3.25, 2.5]])
    ));
    doc.add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(
        doc.plot_pair_table_data(3, PlotPointsMode::Cartesian),
        Ok(vec![10, 11, 12])
    );
    let plotted = (10..=12)
        .map(|id| doc.position(id).unwrap())
        .collect::<Vec<_>>();
    let bytes = doc.encode().unwrap();
    assert_eq!(
        bytes,
        include_bytes!("../../../../../tests/fixtures/table_pair_mixed_width_plot_generated.gsp")
    );
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        (10..=12)
            .map(|id| reopened.position(id).unwrap())
            .collect::<Vec<_>>(),
        plotted
    );

    doc.set_parameter_value(2, -99.99).unwrap();
    assert!(matches!(
        &doc.objects[2].kind,
        Kind::ValueTablePair(2, 1, group)
            if group.pair_values()
                == Some(vec![[-2.5, 12.34], [12.34, -3.25], [-99.99, 2.5]])
    ));
    assert_eq!(
        (10..=12)
            .map(|id| doc.position(id).unwrap())
            .collect::<Vec<_>>(),
        plotted
    );
}

#[test]
fn original_two_row_plotted_point_is_editable() {
    let bytes = include_bytes!("../../../../../tests/fixtures/plot_points_table_two_rows_original.gsp");
    let mut doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert!(matches!(doc.objects[1].kind, Kind::ValueTable(1, ref table) if table.rows() == 2));
    assert_eq!(doc.encode().unwrap(), bytes);
    let before = doc.position(9).unwrap();
    doc.checkpoint();
    assert!(doc.move_point(3, Point { x: 228., y: 140. }));
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    let after = reopened.position(9).unwrap();
    assert!((after.x - before.x - 29.).abs() < 1e-9);
    assert!((after.y - before.y - 15.).abs() < 1e-9);
}

#[test]
fn wine_saved_integrated_plots_preserve_all_bytes() {
    for (bytes, mode) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/plot_points_integrated_cartesian_saved_by_wine.gsp"
            )
            .as_slice(),
            PlotPointsMode::Cartesian,
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/plot_points_integrated_polar_saved_by_wine.gsp"
            )
            .as_slice(),
            PlotPointsMode::Polar,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert!(matches!(doc.objects[8].kind, Kind::PlottedPoint(8, m, _) if m == mode));
        assert!(doc.position(9).is_some());
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}
