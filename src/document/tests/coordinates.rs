use super::*;

#[test]
fn graph_frame_and_slope_reopen_after_edit() {
    let original = Document::open(Path::new("tests/fixtures/wine_segment_slope.gsp")).unwrap();
    assert_eq!(
        original.objects[4].kind,
        Kind::GraphHorizontalUnit(4, 37.79527559055118)
    );
    assert_eq!(original.objects[8].kind, Kind::CoordinateSystem(6, 8));
    assert_eq!(original.objects[9].kind, Kind::SlopeMeasurement(3, 9));
    assert_eq!(original.slope_value(10), Some(0.0));

    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 150., y: 120. }));
    let b = doc.add(Kind::Free(Point { x: 250., y: 120. }));
    let line = doc.add(Kind::Segment(a, b));
    let frame = doc
        .add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(frame, 9);
    let slope = doc.add(Kind::SlopeMeasurement(line, frame));
    assert_eq!(doc.slope_value(slope), Some(0.0));
    doc.set_grid_visible(false);
    assert!(!doc.grid_visible());
    doc.undo();
    assert!(doc.grid_visible());
    doc.redo();
    assert!(!doc.grid_visible());
    let bytes = doc.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(reopened.objects.len(), 10);
    assert!(!reopened.partial_view);
    assert!(!reopened.grid_visible());
    assert_eq!(reopened.snap_points, vec![false]);
    assert_eq!(reopened.slope_value(slope), Some(0.0));
    let mut reopened = reopened;
    assert!(reopened.move_point(b, Point { x: 300., y: 170. }));
    assert!((reopened.slope_value(slope).unwrap() + 1. / 3.).abs() < 1e-12);
    let edited = Document::from_bytes(&reopened.encode().unwrap(), None).unwrap();
    assert!((edited.slope_value(slope).unwrap() + 1. / 3.).abs() < 1e-12);
}

#[test]
fn blank_graph_frame_has_page_link_and_persistent_grid_state() {
    let mut doc = Document::new();
    let frame = doc
        .add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(frame, 6);
    doc.set_grid_visible(false);
    let bytes = doc.encode().unwrap();
    let page_link = records(&bytes)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x232d)
        .unwrap();
    assert_eq!(u32_at(&bytes, page_link.data.start), 6);
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.active_coordinate_system(), Some(frame));
    assert!(!reopened.grid_visible());
    assert_eq!(reopened.snap_points, vec![false]);
}

#[test]
fn marking_an_older_coordinate_system_updates_the_page_link_and_new_plot_reference() {
    let source =
        include_bytes!("../../../tests/fixtures/graph_two_coordinate_systems_original.gsp");
    let native_marked =
        include_bytes!("../../../tests/fixtures/graph_mark_coordinate_system_original.gsp");
    let native = Document::from_bytes(native_marked, None).unwrap();
    assert!(!native.partial_view);
    assert_eq!(native.active_coordinate_system(), Some(9));
    assert_eq!(native.encode().unwrap(), native_marked);
    let mut doc = Document::from_bytes(source, None).unwrap();
    assert!(!doc.partial_view);
    assert_eq!(doc.active_coordinate_system(), Some(14));
    assert_eq!(doc.coordinate_system_for_object(4), Some(9));
    assert_eq!(doc.coordinate_system_for_object(1), Some(14));
    assert_eq!(doc.encode().unwrap(), source);

    assert_eq!(doc.mark_coordinate_system(4).unwrap(), 9);
    assert_eq!(doc.active_coordinate_system(), Some(9));
    doc.undo();
    assert_eq!(doc.active_coordinate_system(), Some(14));
    doc.redo();
    assert_eq!(doc.active_coordinate_system(), Some(9));
    let marked = doc.encode().unwrap();
    let mut expected = source.to_vec();
    let link = records(source)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x232d)
        .unwrap();
    expected[link.data].copy_from_slice(&9_u32.to_le_bytes());
    assert_eq!(marked, expected);
    let reopened = Document::from_bytes(&marked, None).unwrap();
    assert_eq!(reopened.active_coordinate_system(), Some(9));

    let point = doc
        .plot_points_at(PlotPointsMode::Cartesian, [1.0, 2.0])
        .unwrap();
    assert!(matches!(
        doc.objects[point - 1].kind,
        Kind::PlottedPoint(_, 9, _, _)
    ));
    let plotted = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(plotted.active_coordinate_system(), Some(9));
    assert!(matches!(
        plotted.objects[point - 1].kind,
        Kind::PlottedPoint(_, 9, _, _)
    ));
}

#[test]
fn selected_point_can_define_a_second_coordinate_system() {
    let source = include_bytes!("../../../tests/fixtures/graph_defined_coordinate_system.gsp");
    let mut doc = Document::from_bytes(source, None).unwrap();
    let frame = doc.add_coordinate_system_at_point(1).unwrap();
    assert_eq!(frame, 14);
    assert_eq!(doc.active_coordinate_system(), Some(14));
    assert_eq!(doc.objects.len(), 14);
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.active_coordinate_system(), Some(14));
    assert_eq!(
        reopened.objects[9].kind,
        Kind::GraphHorizontalUnit(1, 37.79527559055118)
    );
    assert_eq!(reopened.objects[13].kind, Kind::CoordinateSystem(11, 13));
    assert_eq!(reopened.graph_frame(14).unwrap().origin.x, 150.0);
}

#[test]
fn circle_defined_coordinate_system_tracks_its_radius_and_round_trips() {
    let native = include_bytes!("../../../tests/fixtures/graph_unit_circle_original.gsp");
    let mut original = Document::from_bytes(native, None).unwrap();
    assert!(!original.partial_view);
    assert_eq!(original.encode().unwrap(), native);
    assert_eq!(original.active_coordinate_system(), Some(6));
    let radius = original.circle_points(3).unwrap().1;
    let frame = original.graph_frame(6).unwrap();
    assert!((frame.x_unit.x - frame.origin.x - radius).abs() < 1e-12);
    assert!((frame.origin.y - frame.y_unit.y - radius).abs() < 1e-12);
    assert!(original.move_point(2, Point { x: 383., y: 217. }));
    let moved = original.graph_frame(6).unwrap();
    assert!((moved.x_unit.x - moved.origin.x - 60.).abs() < 1e-12);

    let mut generated =
        Document::from_bytes(include_bytes!("../../../tests/fixtures/circle.gsp"), None).unwrap();
    assert_eq!(generated.add_coordinate_system_from_circle(3).unwrap(), 6);
    let bytes = generated.encode().unwrap();
    let native_styles: Vec<_> = records(native)
        .unwrap()
        .into_iter()
        .filter(|record| record.kind == 0x04b3)
        .collect();
    let generated_styles: Vec<_> = records(&bytes)
        .unwrap()
        .into_iter()
        .filter(|record| record.kind == 0x04b3)
        .collect();
    assert_eq!(generated_styles.len(), 2);
    assert_eq!(
        &bytes[generated_styles[1].data.clone()],
        &native[native_styles[1].data.clone()]
    );
    let mut reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[3].kind, Kind::CircleAxis(3, 1));
    assert_eq!(reopened.objects[4].kind, Kind::CircleAxis(3, 0));
    assert_eq!(reopened.objects[5].kind, Kind::CoordinateSystem(4, 5));
    assert_eq!(reopened.active_coordinate_system(), Some(6));
    assert_eq!(reopened.graph_frame(6), generated.graph_frame(6));
    assert!(reopened.set_grid_form(GridForm::Rectangular).is_err());
    reopened.set_grid_form(GridForm::Polar).unwrap();
    let polar = Document::from_bytes(&reopened.encode().unwrap(), None).unwrap();
    assert_eq!(polar.grid_form[0], GridForm::Polar);
}

#[test]
fn distance_defined_frames_decode_and_follow_their_sources() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/graph_unit_segment_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/graph_unit_segment_point_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/graph_two_segments_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/graph_two_segments_point_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/graph_unit_distance_value_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/graph_unit_segment_dragged_by_wine.gsp").as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        assert!(
            doc.graph_frame(doc.active_coordinate_system().unwrap())
                .is_some()
        );
    }
    let mut square = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/graph_unit_segment_original.gsp"),
        None,
    )
    .unwrap();
    let original = square.graph_frame(8).unwrap();
    assert!((original.x_unit.x - original.origin.x - 188.73261509341728).abs() < 1e-6);
    assert!((original.origin.y - original.y_unit.y - 188.73261509341728).abs() < 1e-6);
    square.move_point(2, Point { x: 168., y: 204. });
    let moved = square.graph_frame(8).unwrap();
    assert!((moved.x_unit.x - moved.origin.x - 100.).abs() < 1e-9);
    assert!((moved.origin.y - moved.y_unit.y - 100.).abs() < 1e-9);
    let saved_square = Document::from_bytes(&square.encode().unwrap(), None).unwrap();
    assert_eq!(saved_square.graph_frame(8), Some(moved));
    square.set_grid_visible(false);
    assert!(!square.is_displayed(4));
    assert!(square.is_displayed(3));
    let native_dragged = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/graph_unit_segment_dragged_by_wine.gsp"),
        None,
    )
    .unwrap();
    let dragged_frame = native_dragged.graph_frame(8).unwrap();
    assert!((dragged_frame.x_unit.x - dragged_frame.origin.x - 192f64.hypot(103.)).abs() < 1e-9);
    assert!((dragged_frame.origin.y - dragged_frame.y_unit.y - 192f64.hypot(103.)).abs() < 1e-9);

    let mut rectangular = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/graph_two_segments_original.gsp"),
        None,
    )
    .unwrap();
    let frame = rectangular.graph_frame(10).unwrap();
    assert_eq!(frame.x_unit.x - frame.origin.x, 100.);
    assert_eq!(frame.origin.y - frame.y_unit.y, 150.);
    rectangular.move_point(5, Point { x: 350., y: 200. });
    let changed = rectangular.graph_frame(10).unwrap();
    assert_eq!(changed.x_unit.x - changed.origin.x, 100.);
    assert_eq!(changed.origin.y - changed.y_unit.y, 200.);
}

#[test]
fn generated_distance_defined_frames_reopen_with_their_grid_style() {
    let sources: [(&[u8], Option<usize>, &[usize], usize, GridForm); 6] = [
        (
            include_bytes!("../../../tests/fixtures/segment.gsp"),
            None,
            &[3],
            8,
            GridForm::Square,
        ),
        (
            include_bytes!("../../../tests/fixtures/segment.gsp"),
            Some(2),
            &[3],
            7,
            GridForm::Square,
        ),
        (
            include_bytes!("../../../tests/fixtures/graph_two_segments_base.gsp"),
            None,
            &[3, 6],
            10,
            GridForm::Rectangular,
        ),
        (
            include_bytes!("../../../tests/fixtures/graph_two_segments_base.gsp"),
            Some(1),
            &[3, 6],
            9,
            GridForm::Rectangular,
        ),
        (
            include_bytes!("../../../tests/fixtures/wine_point_distance.gsp"),
            None,
            &[6],
            11,
            GridForm::Square,
        ),
        (
            include_bytes!("../../../tests/fixtures/wine_segment_length.gsp"),
            None,
            &[4],
            9,
            GridForm::Square,
        ),
    ];
    for (source, origin, distances, expected_id, form) in sources {
        let mut doc = Document::from_bytes(source, None).unwrap();
        assert_eq!(
            doc.add_coordinate_system_from_distances(
                origin,
                distances,
                Point { x: 199., y: 125. },
            )
            .unwrap(),
            expected_id
        );
        let bytes = doc.encode().unwrap();
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.active_coordinate_system(), Some(expected_id));
        assert_eq!(reopened.grid_form[0], form);
        assert_eq!(
            reopened.graph_frame(expected_id),
            doc.graph_frame(expected_id)
        );
        assert!(records(&bytes).unwrap().iter().any(|record| {
            record.kind == 0x04b3 && bytes[record.data.start + 2..record.data.start + 4] == [14, 0]
        }));
    }
    let polar = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/graph_unit_segment_polar_generated.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(polar.grid_form[0], GridForm::Polar);
}

#[test]
fn one_distance_rectangular_unit_edits_and_grid_form_conversions_round_trip() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/graph_unit_segment_rectangular_original.gsp")
            .as_slice(),
        include_bytes!(
            "../../../tests/fixtures/graph_unit_segment_rectangular_to_square_original.gsp"
        )
        .as_slice(),
        include_bytes!(
            "../../../tests/fixtures/graph_unit_segment_rectangular_to_polar_original.gsp"
        )
        .as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
    let mut generated = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/graph_unit_segment_generated.gsp"),
        None,
    )
    .unwrap();
    generated.set_grid_form(GridForm::Rectangular).unwrap();
    assert!(generated.is_displayed(6));
    let rectangular = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert_eq!(rectangular.grid_form[0], GridForm::Rectangular);
    assert_eq!(
        rectangular.objects[5].kind,
        Kind::DistanceVerticalUnit(5, 168f64.hypot(86.), true)
    );
    assert_eq!(rectangular.graph_frame(8), generated.graph_frame(8));

    let mut edited = rectangular;
    edited.move_point(2, Point { x: 168., y: 204. });
    let frame = edited.graph_frame(8).unwrap();
    assert_eq!(frame.x_unit.x - frame.origin.x, 100.);
    assert!((frame.origin.y - frame.y_unit.y - 168f64.hypot(86.)).abs() < 1e-9);
    assert!(edited.move_point(6, Point { x: 199., y: 45. }));
    let moved = edited.graph_frame(8).unwrap();
    assert_eq!(moved.origin.y - moved.y_unit.y, 80.);
    let saved = Document::from_bytes(&edited.encode().unwrap(), None).unwrap();
    assert_eq!(saved.graph_frame(8), Some(moved));

    let mut polar_edited = saved;
    polar_edited.set_grid_form(GridForm::Polar).unwrap();
    let polar = Document::from_bytes(&polar_edited.encode().unwrap(), None).unwrap();
    assert_eq!(polar.grid_form[0], GridForm::Polar);
    assert_eq!(
        polar.objects[5].kind,
        Kind::DistanceVerticalUnit(5, 80., true)
    );
    assert_eq!(
        polar.graph_frame(8).unwrap().origin.y - polar.graph_frame(8).unwrap().y_unit.y,
        80.
    );

    edited.set_grid_form(GridForm::Square).unwrap();
    let mut square = Document::from_bytes(&edited.encode().unwrap(), None).unwrap();
    assert_eq!(square.grid_form[0], GridForm::Square);
    assert_eq!(
        square.objects[5].kind,
        Kind::DistanceVerticalUnit(5, 80., false)
    );
    assert_eq!(
        square.graph_frame(8).unwrap().origin.y - square.graph_frame(8).unwrap().y_unit.y,
        100.
    );
    square.set_grid_form(GridForm::Polar).unwrap();
    let dependent_polar = Document::from_bytes(&square.encode().unwrap(), None).unwrap();
    assert_eq!(dependent_polar.grid_form[0], GridForm::Polar);
    assert_eq!(
        dependent_polar.objects[5].kind,
        Kind::DistanceVerticalUnit(5, 80., false)
    );
    assert_eq!(
        dependent_polar.graph_frame(8).unwrap().origin.y
            - dependent_polar.graph_frame(8).unwrap().y_unit.y,
        100.
    );
}

#[test]
fn two_distance_frame_can_switch_between_rectangular_and_polar() {
    let native = include_bytes!("../../../tests/fixtures/graph_two_segments_polar_original.gsp");
    let parsed = Document::from_bytes(native, None).unwrap();
    assert!(!parsed.partial_view);
    assert_eq!(parsed.grid_form[0], GridForm::Polar);
    assert_eq!(parsed.encode().unwrap(), native);

    let mut doc = Document::from_bytes(
        include_bytes!("../../../tests/fixtures/graph_two_segments_generated.gsp"),
        None,
    )
    .unwrap();
    assert!(doc.set_grid_form(GridForm::Square).is_err());
    doc.set_grid_form(GridForm::Polar).unwrap();
    let saved = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(saved.grid_form[0], GridForm::Polar);
    assert_eq!(saved.graph_frame(10), doc.graph_frame(10));
    doc.set_grid_form(GridForm::Rectangular).unwrap();
    assert_eq!(doc.grid_form[0], GridForm::Rectangular);
}

#[test]
fn marking_another_frame_preserves_each_frames_grid_form() {
    let source =
        include_bytes!("../../../tests/fixtures/graph_two_coordinate_systems_original.gsp");
    let mut doc = Document::from_bytes(source, None).unwrap();
    doc.set_grid_form(GridForm::Rectangular).unwrap();
    assert_eq!(
        doc.frame_styles.get(&14),
        Some(&(GridForm::Rectangular, false))
    );
    doc.mark_coordinate_system(4).unwrap();
    assert_eq!(doc.grid_form[0], GridForm::Square);
    assert_eq!(doc.frame_styles.get(&9), Some(&(GridForm::Square, false)));
    assert!(!doc.is_displayed(7));
    assert!(doc.is_displayed(12));
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.active_coordinate_system(), Some(9));
    assert_eq!(
        reopened.frame_styles.get(&9),
        Some(&(GridForm::Square, false))
    );
    assert_eq!(
        reopened.frame_styles.get(&14),
        Some(&(GridForm::Rectangular, false))
    );
    assert!(!reopened.is_displayed(7));
    assert!(reopened.is_displayed(12));
    assert_eq!(reopened.objects[11].kind, doc.objects[11].kind);
}

#[test]
fn dotted_grid_matches_original_and_survives_undo_and_save() {
    let original = fs::read("tests/fixtures/graph_dotted_grid_original.gsp").unwrap();
    let parsed = Document::from_bytes(&original, None).unwrap();
    assert!(!parsed.partial_view);
    assert_eq!(parsed.grid_dotted, vec![true]);
    assert_eq!(parsed.encode().unwrap(), original);

    let mut doc = Document::new();
    doc.add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    doc.set_grid_dotted(true);
    assert_eq!(doc.grid_dotted, vec![true]);
    doc.undo();
    assert_eq!(doc.grid_dotted, vec![false]);
    doc.redo();
    assert_eq!(doc.grid_dotted, vec![true]);
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.grid_dotted, vec![true]);
    assert_eq!(reopened.encode().unwrap(), encoded);

    let mut edited_original = parsed;
    edited_original.set_grid_dotted(false);
    let encoded = edited_original.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert_eq!(reopened.grid_dotted, vec![false]);
}

#[test]
fn show_grid_and_snap_points_use_distinct_original_records() {
    let hidden = fs::read("tests/fixtures/graph_hidden_by_menu_original.gsp").unwrap();
    let hidden = Document::from_bytes(&hidden, None).unwrap();
    assert!(!hidden.partial_view);
    assert!(!hidden.grid_enabled());
    assert_eq!(hidden.snap_points, vec![false]);
    assert!(hidden.is_displayed(1));
    assert!((2..=7).all(|id| !hidden.is_displayed(id)));
    assert_eq!(
        hidden.encode().unwrap(),
        fs::read("tests/fixtures/graph_hidden_by_menu_original.gsp").unwrap()
    );

    let snapped = fs::read("tests/fixtures/graph_snap_points_original.gsp").unwrap();
    let mut snapped = Document::from_bytes(&snapped, None).unwrap();
    assert!(!snapped.partial_view);
    assert!(snapped.grid_enabled());
    assert!(snapped.grid_visible());
    assert_eq!(snapped.snap_points, vec![true]);
    assert_eq!(
        snapped.position(1),
        Some(Point {
            x: 236.79527559055117,
            y: 162.79527559055117,
        })
    );
    snapped.checkpoint();
    assert!(snapped.move_point(1, Point { x: 270., y: 175. }));
    assert_eq!(
        snapped.position(1),
        Some(Point {
            x: 274.59055118110234,
            y: 162.79527559055117,
        })
    );
    let reopened = Document::from_bytes(&snapped.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.snap_points, vec![true]);
    assert_eq!(reopened.position(1), snapped.position(1));

    snapped.set_snap_points(false);
    assert_eq!(snapped.snap_points, vec![false]);
    snapped.undo();
    assert_eq!(snapped.snap_points, vec![true]);
    snapped.redo();
    assert_eq!(snapped.snap_points, vec![false]);
}

#[test]
fn polar_and_rectangular_grid_forms_reopen_and_edit() {
    for (path, form) in [
        (
            "tests/fixtures/graph_polar_grid_original.gsp",
            GridForm::Polar,
        ),
        (
            "tests/fixtures/graph_rectangular_grid_original.gsp",
            GridForm::Rectangular,
        ),
        (
            "tests/fixtures/graph_rectangular_grid_dragged_original.gsp",
            GridForm::Rectangular,
        ),
    ] {
        let bytes = fs::read(path).unwrap();
        let doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view, "{path}");
        assert_eq!(doc.grid_form, vec![form]);
        assert_eq!(doc.encode().unwrap(), bytes);
    }

    let mut polar = Document::new();
    polar
        .add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    polar.set_grid_form(GridForm::Polar).unwrap();
    assert_eq!(polar.grid_form, vec![GridForm::Polar]);
    polar.undo();
    assert_eq!(polar.grid_form, vec![GridForm::Square]);
    polar.redo();
    assert_eq!(polar.grid_form, vec![GridForm::Polar]);
    assert_eq!(
        polar.encode().unwrap(),
        fs::read("tests/fixtures/graph_polar_grid_generated.gsp").unwrap()
    );
    polar.set_grid_dotted(true);
    assert_eq!(
        polar.encode().unwrap(),
        fs::read("tests/fixtures/graph_polar_dotted_generated.gsp").unwrap()
    );

    let mut rectangular = Document::new();
    rectangular
        .add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    rectangular.set_grid_form(GridForm::Rectangular).unwrap();
    assert!(rectangular.is_displayed(4));
    assert_eq!(
        rectangular.encode().unwrap(),
        fs::read("tests/fixtures/graph_rectangular_grid_generated.gsp").unwrap()
    );
    assert!(rectangular.move_point(4, Point { x: 199., y: 60. }));
    assert_eq!(rectangular.position(4), Some(Point { x: 199., y: 60. }));
    let reopened = Document::from_bytes(&rectangular.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.grid_form, vec![GridForm::Rectangular]);
    assert_eq!(reopened.position(4), Some(Point { x: 199., y: 60. }));

    let mut original = Document::open(Path::new(
        "tests/fixtures/graph_rectangular_grid_original.gsp",
    ))
    .unwrap();
    assert!(original.move_point(4, Point { x: 199., y: 60. }));
    let reopened = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.position(4), Some(Point { x: 199., y: 60. }));

    let mut changed = Document::open(Path::new(
        "tests/fixtures/graph_rectangular_grid_original.gsp",
    ))
    .unwrap();
    changed.set_grid_form(GridForm::Polar).unwrap();
    let reopened = Document::from_bytes(&changed.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.grid_form, vec![GridForm::Polar]);
    assert_eq!(
        reopened.position(4),
        Some(Point {
            x: 199.,
            y: 125. - 37.79527559055118
        })
    );
}

#[test]
fn polar_snap_matches_original_grid_intersection() {
    let original =
        Document::open(Path::new("tests/fixtures/graph_polar_snap_original.gsp")).unwrap();
    assert!(!original.partial_view);
    assert_eq!(original.grid_form, vec![GridForm::Polar]);
    assert_eq!(original.snap_points, vec![true]);
    let expected = original.position(1).unwrap();

    let mut doc = Document::new();
    let point = doc.add(Kind::Free(Point { x: 210., y: 140. }));
    doc.add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    doc.set_grid_form(GridForm::Polar).unwrap();
    doc.set_snap_points(true);
    assert!(doc.move_point(point, Point { x: 250., y: 170. }));
    let actual = doc.position(point).unwrap();
    assert!(actual.distance(expected) < 1e-9);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(reopened.position(point).unwrap().distance(expected) < 1e-9);
}

#[test]
fn trigonometric_axis_matches_original_and_roundtrips() {
    let bytes = fs::read("tests/fixtures/graph_trigonometric_axis_original.gsp").unwrap();
    let mut original = Document::from_bytes(&bytes, None).unwrap();
    assert!(!original.partial_view);
    assert!(original.axis_trigonometric());
    assert_eq!(original.objects[2].kind, Kind::GraphAxis(1, 2, 0x1_0001));
    assert_eq!(original.encode().unwrap(), bytes);
    original.set_axis_trigonometric(false).unwrap();
    assert!(!original.axis_trigonometric());
    original.undo();
    assert!(original.axis_trigonometric());
    original.redo();
    assert!(!original.axis_trigonometric());
    let reopened = Document::from_bytes(&original.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects[2].kind, Kind::GraphAxis(1, 2, 1));

    let mut generated = Document::new();
    generated
        .add_default_coordinate_system(Point { x: 199., y: 125. })
        .unwrap();
    generated.set_axis_trigonometric(true).unwrap();
    assert_eq!(
        generated.encode().unwrap(),
        fs::read("tests/fixtures/graph_trigonometric_axis_generated.gsp").unwrap()
    );
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(reopened.axis_trigonometric());
}

#[test]
fn corpus_coordinate_labels_and_display_formats_feed_calculations() {
    for (bytes, id, calculation, kind) in [
        (include_bytes!("../../../tests/fixtures/coordinate_abscissa_labeled_corpus.gsp").as_slice(), 15, 17, CoordinateMeasureKind::Abscissa),
        (include_bytes!("../../../tests/fixtures/coordinate_ordinate_formatted_corpus.gsp").as_slice(), 18, 19, CoordinateMeasureKind::Ordinate),
    ] {
        let original = Document::from_bytes(bytes, None).unwrap();
        assert!(matches!(original.objects[id - 1].kind, Kind::CoordinateMeasurement { kind: actual, .. } if actual == kind));
        assert!(original.objects[id - 1].label.is_some());
        let initial = original.numeric_value(id).unwrap();
        assert!(initial.is_finite());
        assert!(original.numeric_value(calculation).unwrap().is_finite());
        assert_eq!(original.encode().unwrap(), bytes);
        // Move a free dependency in the original graph, retaining the corpus layout.
        let mut changed = None;
        for object in &original.objects {
            let Kind::Free(point) = object.kind else { continue; };
            let mut candidate = original.clone();
            candidate.checkpoint();
            if candidate.move_point(object.id, Point { x: point.x + 11., y: point.y - 13. })
                && candidate.numeric_value(id).is_some_and(|value| (value - initial).abs() > 1e-9)
            {
                changed = Some(candidate);
                break;
            }
        }
        let edited = changed.expect("Coordinate measurement must follow a movable dependency");
        assert!(edited.numeric_value(calculation).unwrap().is_finite());
        // Other unsupported/undefined constructions in these original pages
        // still prevent saving edits; their raw imports remain byte preserving.
    }
}

#[test]
fn formatted_labeled_coordinates_save_live_values_and_calculations() {
    for (kind, mode) in [(CoordinateMeasureKind::Abscissa, 1_u16), (CoordinateMeasureKind::Ordinate, 3)] {
        let mut bytes = crate::coordinate_measure_records::encode_on_original_base(kind, &[1, 9]).unwrap();
        let native = include_bytes!("../../../tests/fixtures/coordinate_abscissa_labeled_corpus.gsp");
        let native_records = records(native).unwrap();
        let header = native_records.iter().filter(|r| r.kind == 0x07d0).nth(14).unwrap();
        let label = native_records.iter().find(|r| r.start > header.start && r.kind == 0x07d5).unwrap();
        let caption = native[label.start..label.data.end + (label.data.len() & 1)].to_vec();
        let rs = records(&bytes).unwrap();
        let aux = rs.iter().find(|r| r.kind == 0x0903).unwrap();
        let at = aux.data.start;
        bytes[at + 22..at + 24].copy_from_slice(&0x524d_u16.to_le_bytes());
        bytes[at + 26..at + 28].copy_from_slice(&mode.to_le_bytes());
        let style = rs.iter().find(|r| r.start > aux.start && r.kind == 0x07d8).unwrap();
        bytes.splice(style.start..style.start, caption);
        let mut doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
        assert!(doc.objects[9].label.is_some());
        let calculation = doc.add_parameter_calculation(10, "2*v+1", (10, 80)).unwrap();
        let authored = doc.encode().unwrap();
        let before = doc.numeric_value(10).unwrap();
        doc.checkpoint();
        assert!(doc.move_point(1, Point { x: 170., y: 100. }));
        let after = doc.numeric_value(10).unwrap();
        assert!((after - before).abs() > 0.1);
        assert!((doc.numeric_value(calculation).unwrap() - (2. * after + 1.)).abs() < 1e-12);
        doc.verify_roundtrip().unwrap();
        let saved = doc.encode().unwrap();
        let reopened = Document::from_bytes(&saved, None).unwrap();
        assert_eq!(reopened.numeric_value(10), Some(after));
        assert_eq!(reopened.numeric_value(calculation), doc.numeric_value(calculation));
        assert_eq!(reopened.objects[9].label, doc.objects[9].label);
        assert_eq!(reopened.encode().unwrap(), saved);
        doc.undo();
        assert_eq!(doc.encode().unwrap(), authored);
        doc.redo();
        assert_eq!(doc.encode().unwrap(), saved);
    }
}

#[test]
fn coordinate_display_tail_native_controls_keep_live_values_and_writer_records() {
    for (bytes,golden,kind,initial,delta) in [
        (include_bytes!("../../../tests/fixtures/coordinate_display_tail_x_native_verified.gsp").as_slice(),include_bytes!("../../../tests/fixtures/coordinate_display_tail_x_writer_native_verified.gsp").as_slice(),CoordinateMeasureKind::Abscissa,-1.2964583333333337,1.),
        (include_bytes!("../../../tests/fixtures/coordinate_display_tail_y_native_verified.gsp").as_slice(),include_bytes!("../../../tests/fixtures/coordinate_display_tail_y_writer_native_verified.gsp").as_slice(),CoordinateMeasureKind::Ordinate,0.1322916666666667,2.),
    ] {
        let mut d=Document::from_bytes(bytes,None).unwrap();
        assert!(!d.partial_view);
        assert!(matches!(d.objects[9].kind,Kind::CoordinateMeasurement{kind:actual,first:1,second:None,frame:9} if actual==kind));
        assert!((d.numeric_value(10).unwrap()-initial).abs()<1e-12);
        assert_eq!(d.encode().unwrap(),bytes);
        let calc=d.add_parameter_calculation(10,"2*v+1",(10,80)).unwrap();
        let baseline=d.encode().unwrap();
        let p=d.position(1).unwrap();d.checkpoint();
        assert!(d.move_point(1,Point{x:p.x+37.79527559055118,y:p.y-75.59055118110236}));
        d.set_measurement_precision(&[10],3).unwrap();
        assert!((d.numeric_value(10).unwrap()-initial-delta).abs()<1e-12);
        assert!((d.numeric_value(calc).unwrap()-(2.*(initial+delta)+1.)).abs()<1e-12);
        assert_eq!(d.encode().unwrap(),golden);
        d.verify_roundtrip().unwrap();
        let saved=records(golden).unwrap();
        let aux=saved.iter().find(|r|r.kind==0x0903).unwrap();
        assert_eq!(&golden[aux.data.start+30..aux.data.end],&0x003e_u16.to_le_bytes());
        d.undo();d.undo();assert_eq!(d.encode().unwrap(),baseline);
        d.redo();d.redo();assert_eq!(d.encode().unwrap(),golden);
    }
}

#[test]
fn coordinate_display_tail_original_sine_graph_unblocks_dependent_calculation() {
    let bytes=include_bytes!("../../../tests/fixtures/coordinate_display_tail_sine_corpus.gsp");
    let d=Document::from_bytes(bytes,None).unwrap();
    assert!(matches!(d.objects[69].kind,Kind::CoordinateMeasurement{kind:CoordinateMeasureKind::Abscissa,first:45,second:None,frame:29}));
    assert!(matches!(d.objects[82].kind,Kind::CoordinateMeasurement{kind:CoordinateMeasureKind::Ordinate,first:33,second:None,frame:29}));
    assert!((d.numeric_value(74).unwrap()-(d.numeric_value(70).unwrap()-d.numeric_value(71).unwrap())).abs()<1e-12);
    // This original still contains unsupported functions and actions; strict
    // editing verification is covered by the complete Native controls above.
    assert!(d.partial_view);
    assert_eq!(d.encode().unwrap(),bytes);
    let initial=d.numeric_value(70).unwrap();
    let mut changed=None;
    for o in &d.objects {
        let Kind::Free(p)=o.kind else {continue;};
        let mut candidate=d.clone();candidate.checkpoint();
        if candidate.move_point(o.id,Point{x:p.x+11.,y:p.y-13.})
            && candidate.numeric_value(70).is_some_and(|v|(v-initial).abs()>1e-9) {
            changed=Some(candidate);break;
        }
    }
    let mut edited=changed.expect("The coordinate must retain its geometric dependencies");
    assert!((edited.numeric_value(74).unwrap()-(edited.numeric_value(70).unwrap()-edited.numeric_value(71).unwrap())).abs()<1e-12);
    edited.undo();assert_eq!(edited.encode().unwrap(),bytes);
}
