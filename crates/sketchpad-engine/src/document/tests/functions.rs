use super::*;

#[test]
fn original_function_files_decode_as_editable_documents() {
    for (source, expression) in [
        (
            include_bytes!("../../../../../tests/fixtures/function_plot_identity.gsp").as_slice(),
            SimpleFunction::Identity,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/function_plot_square.gsp").as_slice(),
            SimpleFunction::Square,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/function_plot_x_plus_1.gsp").as_slice(),
            SimpleFunction::parse("x+1").unwrap(),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/function_plot_two_times_x_plus_1.gsp")
                .as_slice(),
            SimpleFunction::parse("2*(x+1)").unwrap(),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/function_plot_x_plus_half.gsp").as_slice(),
            SimpleFunction::parse("x+0.5").unwrap(),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/function_plot_sin_original.gsp").as_slice(),
            SimpleFunction::parse("sin(x)").unwrap(),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/function_plot_cos_original.gsp").as_slice(),
            SimpleFunction::parse("cos(x)").unwrap(),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/function_plot_tan_original.gsp").as_slice(),
            SimpleFunction::parse("tan(x)").unwrap(),
        ),
    ] {
        let mut doc = Document::from_bytes(source, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(doc.objects[6].kind, Kind::FunctionDefinition(expression));
        assert_eq!(doc.angle_unit_radians, expression.uses_trigonometry());
        assert!(matches!(
            doc.objects[7].kind,
            Kind::FunctionPlot(7, 6, _, _, 750)
        ));
        assert_eq!(doc.encode().unwrap(), source);
        let before = doc.function_plot_points(8, 3).unwrap();
        assert!(doc.move_point(1, Point { x: 228., y: 140. }));
        let after = doc.function_plot_points(8, 3).unwrap();
        assert!((after[1].x - before[1].x - 29.).abs() < 1e-9);
        assert!((after[1].y - before[1].y - 15.).abs() < 1e-9);
        let saved = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!saved.partial_view);
        assert_eq!(saved.objects[7].kind, doc.objects[7].kind);
    }
}

#[test]
fn native_derivatives_roundtrip_and_source_edit_refreshes_cached_formula() {
    for (source, expected_source, derivative) in [
        (
            include_bytes!("../../../../../tests/fixtures/function_derivative_identity_original.gsp")
                .as_slice(),
            1,
            SimpleFunction::parse("1").unwrap(),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/function_derivative_square_original.gsp")
                .as_slice(),
            2,
            SimpleFunction::parse("2*x").unwrap(),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/function_derivative_cubic_original.gsp")
                .as_slice(),
            1,
            SimpleFunction::parse("3*x^2").unwrap(),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/function_derivative_sine_original.gsp")
                .as_slice(),
            7,
            SimpleFunction::parse("cos(x)").unwrap(),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/function_derivative_sine_degrees_original.gsp")
                .as_slice(),
            7,
            SimpleFunction::parse("cos(x)").unwrap(),
        ),
    ] {
        let doc = Document::from_bytes(source, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(
            doc.objects.last().unwrap().kind,
            Kind::DerivativeFunction(expected_source, derivative)
        );
        assert_eq!(doc.encode().unwrap(), source);
    }

    let native = include_bytes!("../../../../../tests/fixtures/function_derivative_square_original.gsp");
    let mut doc = Document::from_bytes(native, None).unwrap();
    doc.set_function_expression(2, SimpleFunction::parse("x^3").unwrap())
        .unwrap();
    assert_eq!(
        doc.objects[2].kind,
        Kind::DerivativeFunction(2, SimpleFunction::parse("3*x^2").unwrap())
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[2].kind, doc.objects[2].kind);
    doc.undo();
    assert_eq!(doc.encode().unwrap(), native);
}

#[test]
fn generated_derivative_can_be_plotted_and_reopened() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/parametric_two_functions_moved_original.gsp"),
        None,
    )
    .unwrap();
    let derivative = doc.add_derivative_function(2).unwrap();
    assert_eq!(derivative, 3);
    assert_eq!(
        doc.objects[2].kind,
        Kind::DerivativeFunction(2, SimpleFunction::parse("2*x").unwrap())
    );
    let plot = doc
        .plot_function(derivative, Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(plot, 10);
    let before = doc.function_plot_points(plot, 5).unwrap();
    doc.set_function_expression(2, SimpleFunction::parse("x^3").unwrap())
        .unwrap();
    let after = doc.function_plot_points(plot, 5).unwrap();
    assert_ne!(after, before);
    assert_eq!(
        doc.objects[2].kind,
        Kind::DerivativeFunction(2, SimpleFunction::parse("3*x^2").unwrap())
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[2].kind, doc.objects[2].kind);
    assert_eq!(reopened.objects[plot - 1].kind, doc.objects[plot - 1].kind);
    assert_eq!(reopened.function_plot_points(plot, 5).unwrap(), after);

    let mut parametric = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/function_derivative_square_original.gsp"),
        None,
    )
    .unwrap();
    let curve = parametric
        .plot_parametric_curve(
            1,
            3,
            PlotPointsMode::Cartesian,
            -2.,
            2.,
            Point { x: 199., y: 125. },
        )
        .unwrap();
    assert_eq!(curve, 10);
    let reopened = Document::from_bytes(&parametric.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.objects[curve - 1].kind,
        parametric.objects[curve - 1].kind
    );

    let mut degrees = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/function_plot_sin_degrees_original.gsp"),
        None,
    )
    .unwrap();
    let derivative = degrees
        .add_derivative_function_with_angle_unit(7, false)
        .unwrap();
    assert!(!degrees.angle_unit_radians);
    let reopened = Document::from_bytes(&degrees.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(!reopened.angle_unit_radians);
    assert_eq!(
        reopened.objects[derivative - 1].kind,
        degrees.objects[derivative - 1].kind
    );
}

#[test]
fn adding_trigonometric_function_sets_radians_and_undo_restores_units() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/function_base_frame.gsp"),
        None,
    )
    .unwrap();
    assert!(!doc.angle_unit_radians);
    let sine = SimpleFunction::parse("sin(x)").unwrap();
    doc.plot_new_function(sine, Point { x: 199., y: 125. })
        .unwrap();
    assert!(doc.angle_unit_radians);
    assert!(
        (doc.function_plot_points(8, 3).unwrap()[1].y - doc.graph_frame(6).unwrap().origin.y).abs()
            < 1e-9
    );
    let saved = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!saved.partial_view);
    assert!(saved.angle_unit_radians);
    assert_eq!(saved.objects[6].kind, Kind::FunctionDefinition(sine));
    doc.undo();
    assert!(!doc.angle_unit_radians);
    doc.redo();
    assert!(doc.angle_unit_radians);
}

#[test]
fn degree_based_trigonometric_plot_imports_and_reopens() {
    let native = include_bytes!("../../../../../tests/fixtures/function_plot_sin_degrees_original.gsp");
    let imported = Document::from_bytes(native, None).unwrap();
    assert!(!imported.partial_view);
    assert!(!imported.angle_unit_radians);
    assert_eq!(imported.encode().unwrap(), native);
    let sine = SimpleFunction::parse("sin(x)").unwrap();
    assert_eq!(imported.objects[6].kind, Kind::FunctionDefinition(sine));

    let mut generated = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/function_base_frame.gsp"),
        None,
    )
    .unwrap();
    generated
        .plot_new_function_with_angle_unit(sine, Point { x: 199., y: 125. }, false)
        .unwrap();
    assert!(!generated.angle_unit_radians);
    let reopened = Document::from_bytes(&generated.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(!reopened.angle_unit_radians);
    assert_eq!(reopened.objects[6].kind, imported.objects[6].kind);
    assert_eq!(
        reopened.function_plot_points(8, 3),
        imported.function_plot_points(8, 3)
    );
}

#[test]
fn discontinuous_function_plots_leave_asymptote_gaps() {
    for (source, poles) in [
        (
            include_bytes!("../../../../../tests/fixtures/function_plot_reciprocal_original.gsp")
                .as_slice(),
            vec![0.0],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/function_plot_tan_original.gsp").as_slice(),
            vec![
                -3.0 * std::f64::consts::FRAC_PI_2,
                -std::f64::consts::FRAC_PI_2,
                std::f64::consts::FRAC_PI_2,
                3.0 * std::f64::consts::FRAC_PI_2,
            ],
        ),
    ] {
        let doc = Document::from_bytes(source, None).unwrap();
        assert!(!doc.partial_view);
        let frame = doc.graph_frame(6).unwrap();
        let segments = doc.function_plot_segments(8, 384).unwrap();
        assert!(segments.len() < 383);
        for (a, b) in segments {
            let x0 = frame.coordinates(GraphPoint { x: a.x, y: a.y }).unwrap().x;
            let x1 = frame.coordinates(GraphPoint { x: b.x, y: b.y }).unwrap().x;
            for pole in &poles {
                assert!(!(x0 < *pole && x1 > *pole));
            }
        }
    }
    let smooth = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/function_plot_square.gsp"),
        None,
    )
    .unwrap();
    assert_eq!(smooth.function_plot_segments(8, 384).unwrap().len(), 383);
}

#[test]
fn new_function_plot_reopens_and_tracks_drag_with_undo() {
    let mut doc = Document::new();
    let plot = doc
        .plot_new_function(SimpleFunction::Square, Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(plot, 8);
    let frame = doc.graph_frame(6).unwrap();
    let vertex = doc.function_plot_points(plot, 3).unwrap()[1];
    assert_eq!(
        vertex,
        Point {
            x: frame.origin.x,
            y: frame.origin.y
        }
    );
    let saved = doc.encode().unwrap();
    let mut reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[7].kind, doc.objects[7].kind);
    reopened.checkpoint();
    assert!(reopened.move_point(1, Point { x: 228., y: 140. }));
    assert_eq!(
        reopened.function_plot_points(plot, 3).unwrap()[1],
        Point { x: 228., y: 140. }
    );
    reopened.undo();
    assert_eq!(reopened.function_plot_points(plot, 3).unwrap()[1], vertex);
    reopened.redo();
    assert_eq!(
        reopened.function_plot_points(plot, 3).unwrap()[1],
        Point { x: 228., y: 140. }
    );
    let edited = Document::from_bytes(&reopened.encode().unwrap(), None).unwrap();
    assert_eq!(
        edited.function_plot_points(plot, 3).unwrap()[1],
        Point { x: 228., y: 140. }
    );
}

#[test]
fn plot_existing_function_reuses_definition_and_reopens() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/function_definition_identity.gsp"),
        None,
    )
    .unwrap();
    assert!(doc.plot_function(6, Point { x: 199., y: 125. }).is_err());
    let plot = doc.plot_function(7, Point { x: 199., y: 125. }).unwrap();
    assert_eq!(plot, 8);
    assert_eq!(doc.objects.len(), 8);
    assert!(matches!(
        doc.objects[7].kind,
        Kind::FunctionPlot(7, 6, _, _, 750)
    ));
    let encoded = doc.encode().unwrap();
    assert_eq!(
        encoded,
        include_bytes!("../../../../../tests/fixtures/function_plot_existing_generated.gsp")
    );
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.objects[6].kind,
        Kind::FunctionDefinition(SimpleFunction::Identity)
    );
    assert_eq!(reopened.objects[7].kind, doc.objects[7].kind);
    doc.undo();
    assert_eq!(doc.objects.len(), 7);
    doc.redo();
    assert_eq!(doc.objects.len(), 8);

    let mut fresh = Document::new();
    let definition = fresh
        .add_function_definition(SimpleFunction::Square)
        .unwrap();
    assert_eq!(definition, 1);
    let plot = fresh
        .plot_function(definition, Point { x: 199., y: 125. })
        .unwrap();
    assert_eq!(plot, 8);
    assert!(matches!(
        fresh.objects[7].kind,
        Kind::FunctionPlot(1, 7, _, _, 750)
    ));
    let reopened = Document::from_bytes(&fresh.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[7].kind, fresh.objects[7].kind);
}

#[test]
fn editing_saved_function_updates_plot_and_survives_reopen() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/function_plot_square.gsp"),
        None,
    )
    .unwrap();
    let before = doc.function_plot_points(8, 3).unwrap()[1];
    let expression = SimpleFunction::parse("x+1").unwrap();
    assert!(doc.set_function_expression(8, expression).is_err());
    doc.set_function_expression(7, expression).unwrap();
    let after = doc.function_plot_points(8, 3).unwrap()[1];
    assert!((after.x - before.x).abs() < 1e-9);
    assert!((after.y - before.y + 37.79527559055118).abs() < 1e-9);
    let saved = doc.encode().unwrap();
    let reopened = Document::from_bytes(&saved, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.objects[6].kind,
        Kind::FunctionDefinition(expression)
    );
    assert_eq!(reopened.function_plot_points(8, 3).unwrap()[1], after);
    doc.undo();
    assert_eq!(doc.function_plot_points(8, 3).unwrap()[1], before);
    doc.redo();
    assert_eq!(doc.function_plot_points(8, 3).unwrap()[1], after);

    let compound = SimpleFunction::parse("2*(x+1)").unwrap();
    doc.set_function_expression(7, compound).unwrap();
    assert_eq!(
        doc.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/function_plot_edited_compound_generated.gsp")
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects[6].kind, Kind::FunctionDefinition(compound));
    assert!(!reopened.partial_view);

    let sine = SimpleFunction::parse("sin(x)").unwrap();
    doc.set_function_expression_with_angle_unit(7, sine, true)
        .unwrap();
    assert!(doc.angle_unit_radians);
    assert_eq!(
        doc.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/function_plot_edited_sine_radians_generated.gsp")
    );
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.objects[6].kind, Kind::FunctionDefinition(sine));
    assert!(reopened.angle_unit_radians);
    doc.undo();
    assert_eq!(doc.objects[6].kind, Kind::FunctionDefinition(compound));
    assert!(!doc.angle_unit_radians);
}

#[test]
fn parametric_plots_import_render_and_cross_open() {
    for (mode, native, generated) in [
        (
            PlotPointsMode::Cartesian,
            include_bytes!("../../../../../tests/fixtures/parametric_rectangular_original.gsp")
                .as_slice(),
            include_bytes!("../../../../../tests/fixtures/parametric_rectangular_generated.gsp")
                .as_slice(),
        ),
        (
            PlotPointsMode::Polar,
            include_bytes!("../../../../../tests/fixtures/parametric_polar_original.gsp").as_slice(),
            include_bytes!("../../../../../tests/fixtures/parametric_polar_generated.gsp").as_slice(),
        ),
    ] {
        let original = Document::from_bytes(native, None).unwrap();
        assert!(!original.partial_view);
        assert_eq!(original.encode().unwrap(), native);
        assert!(
            matches!(original.objects[8].kind, Kind::ParametricPlot(1, 2, 8, 0., 1., 750, m) if m == mode)
        );

        let mut doc = Document::from_bytes(
            include_bytes!("../../../../../tests/fixtures/parametric_two_functions_moved_original.gsp"),
            None,
        )
        .unwrap();
        assert!(!doc.partial_view);
        let plot = doc
            .plot_parametric_curve(1, 2, mode, 0., 1., Point { x: 199., y: 125. })
            .unwrap();
        assert_eq!(plot, 9);
        assert_eq!(doc.encode().unwrap(), generated);
        assert_eq!(
            doc.grid_form[0] == GridForm::Polar,
            mode == PlotPointsMode::Polar
        );
        let frame = doc.graph_frame(8).unwrap();
        let segments = doc.parametric_plot_segments(plot, 101).unwrap();
        assert!(!segments.is_empty());
        assert_eq!(
            segments[0].0,
            Point {
                x: frame.origin.x,
                y: frame.origin.y
            }
        );
        let (x, y) = match mode {
            PlotPointsMode::Cartesian => (1., 1.),
            PlotPointsMode::Polar => (1_f64.cos(), 1_f64.sin()),
        };
        let endpoint = frame.world(x, y).unwrap();
        let actual = segments.last().unwrap().1;
        assert!((actual.x - endpoint.x).abs() < 1e-9);
        assert!((actual.y - endpoint.y).abs() < 1e-9);
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.objects[8].kind, doc.objects[8].kind);
    }

    let mut custom = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/parametric_two_functions_moved_original.gsp"),
        None,
    )
    .unwrap();
    custom
        .plot_parametric_curve(
            1,
            2,
            PlotPointsMode::Cartesian,
            -1.,
            2.,
            Point { x: 199., y: 125. },
        )
        .unwrap();
    assert_eq!(
        custom.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/parametric_custom_domain_generated.gsp")
    );

    let mut native = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/parametric_rectangular_original.gsp"),
        None,
    )
    .unwrap();
    let before = native
        .parametric_plot_segments(9, 101)
        .unwrap()
        .last()
        .unwrap()
        .1;
    native
        .set_function_expression(2, SimpleFunction::parse("x+1").unwrap())
        .unwrap();
    let after = native
        .parametric_plot_segments(9, 101)
        .unwrap()
        .last()
        .unwrap()
        .1;
    assert!((after.x - before.x).abs() < 1e-9);
    assert!((after.y - before.y + 37.79527559055118).abs() < 1e-9);
    let reopened = Document::from_bytes(&native.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened
            .parametric_plot_segments(9, 101)
            .unwrap()
            .last()
            .unwrap()
            .1,
        after
    );
}

#[test]
fn plot_property_edits_reopen_and_limit_render_samples() {
    let mut parametric = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/parametric_rectangular_original.gsp"),
        None,
    )
    .unwrap();
    assert!(parametric.set_plot_range(1, -1., 2., 64).is_err());
    parametric.set_plot_range(9, -1., 2., 64).unwrap();
    assert_eq!(
        parametric.parametric_plot_segments(9, 384).unwrap().len(),
        63
    );
    assert_eq!(
        parametric.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/parametric_range_edited_generated.gsp")
    );
    let reopened = Document::from_bytes(&parametric.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(matches!(
        reopened.objects[8].kind,
        Kind::ParametricPlot(1, 2, 8, -1., 2., 64, PlotPointsMode::Cartesian)
    ));
    parametric.undo();
    assert_eq!(
        parametric.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/parametric_rectangular_original.gsp")
    );
    parametric.redo();
    assert_eq!(
        parametric.parametric_plot_segments(9, 384).unwrap().len(),
        63
    );

    let mut function = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/function_plot_square.gsp"),
        None,
    )
    .unwrap();
    function.set_plot_range(8, -2., 2., 64).unwrap();
    assert_eq!(function.function_plot_segments(8, 384).unwrap().len(), 63);
    assert_eq!(
        function.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/function_range_edited_generated.gsp")
    );
    let reopened = Document::from_bytes(&function.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert!(matches!(
        reopened.objects[7].kind,
        Kind::FunctionPlot(7, 6, -2., 2., 64)
    ));

    let mut fresh_function = Document::new();
    fresh_function
        .plot_new_function(SimpleFunction::Square, Point { x: 199., y: 125. })
        .unwrap();
    fresh_function.set_plot_range(8, -2., 2., 64).unwrap();
    assert_eq!(
        fresh_function.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/function_new_range_generated.gsp")
    );

    let mut fresh_parametric = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/parametric_two_functions_moved_original.gsp"),
        None,
    )
    .unwrap();
    fresh_parametric
        .plot_parametric_curve(
            1,
            2,
            PlotPointsMode::Polar,
            0.,
            1.,
            Point { x: 199., y: 125. },
        )
        .unwrap();
    fresh_parametric.set_plot_range(9, -1., 2., 64).unwrap();
    assert_eq!(
        fresh_parametric
            .parametric_plot_segments(9, 384)
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        fresh_parametric.encode().unwrap(),
        include_bytes!("../../../../../tests/fixtures/parametric_new_range_generated.gsp")
    );
}

#[test]
fn extended_functions_and_variable_power_derivatives_remain_live_after_reopening() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/function_plot_identity.gsp"),
        None,
    )
    .unwrap();
    doc.set_function_expression(7, SimpleFunction::parse("x^x").unwrap())
        .unwrap();
    doc.set_plot_range(8, 0.5, 2., 17).unwrap();
    let derivative = doc.add_derivative_function(7).unwrap();
    let derivative_plot = doc
        .plot_function(derivative, Point { x: 199., y: 125. })
        .unwrap();
    doc.set_plot_range(derivative_plot, 0.5, 2., 17).unwrap();
    let before = doc.function_plot_points(derivative_plot, 17).unwrap();
    let bytes = doc.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.encode().unwrap(), bytes);
    assert_eq!(
        reopened.function_plot_points(derivative_plot, 17).unwrap(),
        before
    );

    doc.set_function_expression(7, SimpleFunction::parse("ln(sqrt(x))").unwrap())
        .unwrap();
    let updated = doc.function_plot_points(derivative_plot, 17).unwrap();
    assert_ne!(updated, before);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        reopened.function_plot_points(derivative_plot, 17).unwrap(),
        updated
    );
    assert_eq!(
        reopened.function_display_text(7).as_deref(),
        Some("f(x) = ln(sqrt(x))")
    );
    doc.undo();
    assert_eq!(
        doc.function_plot_points(derivative_plot, 17).unwrap(),
        before
    );
    doc.redo();
    assert_eq!(
        doc.function_plot_points(derivative_plot, 17).unwrap(),
        updated
    );
}

#[test]
fn logarithm_and_root_plots_skip_undefined_domains() {
    for text in ["sqrt(x)", "ln(x)", "log(x)"] {
        let mut doc = Document::from_bytes(
            include_bytes!("../../../../../tests/fixtures/function_plot_identity.gsp"),
            None,
        )
        .unwrap();
        doc.set_function_expression(7, SimpleFunction::parse(text).unwrap())
            .unwrap();
        doc.set_plot_range(8, -2., 2., 17).unwrap();
        assert!(doc.function_plot_points(8, 17).is_none());
        let segments = doc.function_plot_segments(8, 17).unwrap();
        assert!(!segments.is_empty());
        let frame = doc.graph_frame(6).unwrap();
        for (left, right) in segments {
            assert!(
                frame
                    .coordinates(crate::graph::GraphPoint {
                        x: left.x,
                        y: left.y
                    })
                    .unwrap()
                    .x
                    >= 0.
            );
            assert!(
                frame
                    .coordinates(crate::graph::GraphPoint {
                        x: right.x,
                        y: right.y
                    })
                    .unwrap()
                    .x
                    >= 0.
            );
        }
    }
}

#[test]
fn extended_function_generated_files_roundtrip_as_editable_documents() {
    for (bytes, expected) in [
        (
            include_bytes!("../../../../../tests/fixtures/function_extensions_generated.gsp").as_slice(),
            "sqrt(abs(x))+ln(x^2+1)+log(x^2+1)",
        ),
        (
            include_bytes!("../../../../../tests/fixtures/function_variable_derivative_generated.gsp")
                .as_slice(),
            "x^x",
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/function_extended_derivative_edited_generated.gsp"
            )
            .as_slice(),
            "ln(sqrt(x))",
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert_eq!(
            doc.function_expression(7).unwrap(),
            SimpleFunction::parse(expected).unwrap()
        );
        assert_eq!(doc.encode().unwrap(), bytes);
        if doc.objects.len() == 10 {
            assert_eq!(
                doc.function_expression(9).unwrap(),
                SimpleFunction::parse(expected)
                    .unwrap()
                    .symbolic_derivative()
                    .unwrap()
            );
        }
    }
}

#[test]
fn native_primitive_functions_preserve_tokens_and_domain_values() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/function_primitives_controls_native_created.gsp");
    let d = Document::from_bytes(bytes, None).unwrap();
    assert!(!d.partial_view);
    assert_eq!(d.objects.len(), 26);
    assert!(d.angle_unit_radians);
    for (id, name) in [
        (15, "trunc"),
        (17, "round"),
        (19, "sgn"),
        (21, "asin"),
        (23, "acos"),
        (25, "atan"),
    ] {
        let f = SimpleFunction::parse(&format!("{name} (x)")).unwrap();
        assert_eq!(d.objects[id - 1].kind, Kind::FunctionDefinition(f));
        assert!(matches!(d.objects[id].kind,Kind::FunctionPlot(source,_,_,_,_) if source==id));
    }
    for (x, trunc, round, sign) in [
        (-1.5, -1., -1., -1.),
        (-0.5, 0., 0., -1.),
        (0., 0., 0., 0.),
        (0.5, 0., 1., 1.),
        (1.5, 1., 2., 1.),
    ] {
        assert_eq!(d.function_value(15, x), Some(trunc));
        assert_eq!(d.function_value(17, x), Some(round));
        assert_eq!(d.function_value(19, x), Some(sign));
    }
    for (id, value) in [
        (21, 0.5_f64.asin()),
        (23, 0.5_f64.acos()),
        (25, 0.5_f64.atan()),
    ] {
        assert!((d.function_value(id, 0.5).unwrap() - value).abs() < 1e-12);
    }
    for id in [21, 23] {
        assert_eq!(d.function_value(id, 1.5), None);
        assert!(d.function_value(id, 1.).is_some());
    }
    assert_eq!(d.encode().unwrap(), bytes);
    d.verify_roundtrip().unwrap();
}

#[test]
fn corpus_primitive_roots_and_signed_function_captions_decode() {
    let music =
        include_bytes!("../../../../../tests/fixtures/function_negative_caption_music_corpus.gsp");
    let d = Document::from_bytes(music, None).unwrap();
    assert_eq!(d.objects[6].function_anchor, Some((-1, -18)));
    assert_eq!(
        d.objects[6].kind,
        Kind::FunctionDefinition(SimpleFunction::parse("5*sin(25*x)").unwrap())
    );
    assert_eq!(d.encode().unwrap(), music);
    d.verify_roundtrip().unwrap();
    let calendar =
        include_bytes!("../../../../../tests/fixtures/function_calendar_primitives_corpus.gsp");
    let d = Document::from_bytes(calendar, None).unwrap();
    assert_eq!(
        d.objects[2].kind,
        Kind::FunctionDefinition(SimpleFunction::parse("1-sgn(x-trunc(x))").unwrap())
    );
    for (x, value) in [(0., 1.), (4., 1.), (4.25, 0.), (-4.25, 2.)] {
        assert_eq!(d.function_value(3, x), Some(value));
    }
    assert_eq!(d.encode().unwrap(), calendar);
    d.verify_roundtrip().unwrap();
}

fn function_primitives_writer_document() -> Document {
    let mut d = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/source_exponents_base_port.gsp"),
        None,
    )
    .unwrap();
    d.set_parameter_value(6, -0.5).unwrap();
    let expressions = [
        "trunc(x)", "round(x)", "sgn(x)", "asin(x)", "acos(x)", "atan(x)", "pi", "e",
    ];
    for (i, expression) in expressions.iter().enumerate() {
        let function = d
            .add_function_definition(SimpleFunction::parse(expression).unwrap())
            .unwrap();
        d.objects[function - 1].function_anchor = Some((10, 150 + 26 * i as i16));
        d.objects[function - 1].function_name =
            Some(["f", "g", "h", "q", "r", "s", "p", "z"][i].to_owned());
        if i == 6 {
            d.objects[function - 1].function_anchor = Some((-10, -30));
            d.objects[function - 1].visible = false;
        }
        if i == 7 {
            d.objects[function - 1].function_anchor = Some((4100, 40));
            d.objects[function - 1].visible = false;
        }
        d.add_multi_calculation(&[function, 6], "f1(v2)", (225, 150 + 26 * i as i16))
            .unwrap();
        let plot = d
            .plot_function(function, Point { x: 520., y: 240. })
            .unwrap();
        d.objects[plot - 1].visible = i == 0;
    }
    d
}

#[test]
fn primitive_function_actual_writer_matches_native_and_replays_edits() {
    let mut d = function_primitives_writer_document();
    let original = include_bytes!(
        "../../../../../tests/fixtures/function_primitives_writer_original_native_verified.gsp"
    );
    let edited = include_bytes!(
        "../../../../../tests/fixtures/function_primitives_writer_edited_native_verified.gsp"
    );
    assert_eq!(d.encode().unwrap(), original);
    d.verify_roundtrip().unwrap();
    for (id, value) in [
        (10, 0.),
        (19, 0.),
        (22, -1.),
        (25, (-0.5_f64).asin()),
        (28, (-0.5_f64).acos()),
        (31, (-0.5_f64).atan()),
        (34, crate::table_records::CALCULATOR_PI),
        (37, std::f64::consts::E),
    ] {
        assert!((d.numeric_value(id).unwrap() - value).abs() < 1e-12);
        assert_eq!(
            d.objects[id - 1].kind.numeric_unit(),
            Some(NumericUnit::Scalar)
        );
    }
    let reopened = Document::from_bytes(original, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(reopened.objects[32].function_anchor, Some((-10, -30)));
    assert_eq!(reopened.objects[35].function_anchor, Some((4100, 40)));
    for constant in ["pi", "π", "e"] {
        assert_eq!(
            SimpleFunction::parse(constant)
                .unwrap()
                .symbolic_derivative()
                .unwrap(),
            SimpleFunction::parse("0").unwrap()
        );
    }
    d.set_parameter_value(6, 0.5).unwrap();
    d.set_function_expression(9, SimpleFunction::parse("trunc(x+1)").unwrap())
        .unwrap();
    d.set_function_expression_with_angle_unit(24, SimpleFunction::parse("asin(x)").unwrap(), false)
        .unwrap();
    for (id, value) in [
        (10, 1.),
        (19, 1.),
        (22, 1.),
        (25, 30.),
        (28, 60.),
        (31, 0.5_f64.atan().to_degrees()),
    ] {
        assert!((d.numeric_value(id).unwrap() - value).abs() < 1e-12);
    }
    assert_eq!(d.encode().unwrap(), edited);
    d.verify_roundtrip().unwrap();
    for _ in 0..3 {
        d.undo();
    }
    assert_eq!(d.encode().unwrap(), original);
    for _ in 0..3 {
        d.redo();
    }
    assert_eq!(d.encode().unwrap(), edited);
}

#[test]
fn native_referenced_function_definitions_stay_live_and_editable() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/function_reference_controls_native_created.gsp");
    let mut d = Document::from_bytes(bytes, None).unwrap();
    assert!(!d.partial_view);
    assert_eq!(d.objects.len(), 17);
    assert_eq!(d.function_sources(15), Some([6].as_slice()));
    assert_eq!(d.function_sources(17), Some([15].as_slice()));
    assert_eq!(d.objects[14].kind.dependencies(), [6]);
    assert_eq!(d.objects[16].kind.dependencies(), [15]);
    assert_eq!(d.objects[14].function_name.as_deref(), Some("f"));
    assert_eq!(d.objects[16].function_name.as_deref(), Some("g"));
    assert_eq!(d.function_value(15, 4.), Some(9.));
    assert_eq!(d.function_value(17, 4.), Some(11.));
    assert_eq!(
        d.function_display_text(17).as_deref(),
        Some("g(x) = f(x + 1)")
    );
    assert_eq!(d.encode().unwrap(), bytes);
    d.verify_roundtrip().unwrap();
    let before = d.function_plot_points(16, 3).unwrap();
    d.set_parameter_value(6, -3.).unwrap();
    assert_eq!(d.function_value(17, 4.), Some(-14.));
    assert_ne!(d.function_plot_points(16, 3).unwrap(), before);
    d.set_function_expression(
        15,
        SimpleFunction::parse_with_sources("v1*x^2-1", 1).unwrap(),
    )
    .unwrap();
    assert_eq!(d.function_value(17, 4.), Some(-76.));
    d.verify_roundtrip().unwrap();
    let saved = d.encode().unwrap();
    let history = d.undo.len();
    assert!(
        d.set_function_expression(15, SimpleFunction::parse_with_sources("f1(x)", 1).unwrap())
            .is_err()
    );
    assert_eq!(d.undo.len(), history);
    assert_eq!(d.encode().unwrap(), saved);
    d.undo();
    d.undo();
    assert_eq!(d.encode().unwrap(), bytes);
}

#[test]
fn corpus_quadratic_parameter_function_imports_and_recalculates_its_plot() {
    let bytes = include_bytes!("../../../../../tests/fixtures/function_reference_quadratic_corpus.gsp");
    let mut d = Document::from_bytes(bytes, None).unwrap();
    assert!(!d.partial_view);
    assert_eq!(d.function_sources(10), Some([7, 8, 9].as_slice()));
    for (id, value) in [(7, 2.), (8, -3.), (9, 5.)] {
        d.set_parameter_value(id, value).unwrap();
    }
    assert_eq!(d.function_value(10, 4.), Some(25.));
    d.verify_roundtrip().unwrap();
    for _ in 0..3 {
        d.undo();
    }
    assert_eq!(d.encode().unwrap(), bytes);
}

fn function_references_writer_document() -> Document {
    let mut d = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/source_exponents_base_port.gsp"),
        None,
    )
    .unwrap();
    let f = d
        .add_referenced_function_definition(
            &[6, 5],
            SimpleFunction::parse_with_sources("v1*x^2+v2", 2).unwrap(),
        )
        .unwrap();
    let g = d
        .add_referenced_function_definition(
            &[f, 6],
            SimpleFunction::parse_with_sources("f1(x+v2)", 2).unwrap(),
        )
        .unwrap();
    d.add_multi_calculation(&[g, 6], "f1(v2)", (300, 150))
        .unwrap();
    d.plot_function(f, Point { x: 520., y: 240. }).unwrap();
    let h = d
        .add_referenced_function_definition(
            &[6, f, 5],
            SimpleFunction::parse_with_sources("v1*f2(x)-v3", 3).unwrap(),
        )
        .unwrap();
    d.add_multi_calculation(&[h, 6], "f1(v2)", (300, 176))
        .unwrap();
    let hp = d.plot_function(h, Point { x: 520., y: 240. }).unwrap();
    d.objects[hp - 1].visible = false;
    let a = d
        .add_referenced_function_definition(
            &[8],
            SimpleFunction::parse_with_sources("v1", 1).unwrap(),
        )
        .unwrap();
    d.add_multi_calculation(&[a, 6], "f1(v2)", (300, 202))
        .unwrap();
    let b = d
        .add_referenced_function_definition(
            &[8],
            SimpleFunction::parse_with_sources("sin(v1)", 1).unwrap(),
        )
        .unwrap();
    d.add_multi_calculation(&[b, 6], "f1(v2)", (300, 228))
        .unwrap();
    let k = d
        .add_referenced_function_definition(
            &[6],
            SimpleFunction::parse_with_sources("1/(x-v1)", 1).unwrap(),
        )
        .unwrap();
    d.add_multi_calculation(&[k, 6], "f1(v2+1)", (300, 254))
        .unwrap();
    let kp = d.plot_function(k, Point { x: 520., y: 240. }).unwrap();
    d.objects[kp - 1].visible = false;
    let j = d
        .add_referenced_function_definition(
            &[k],
            SimpleFunction::parse_with_sources("f1(x+1)", 1).unwrap(),
        )
        .unwrap();
    let jp = d.plot_function(j, Point { x: 520., y: 240. }).unwrap();
    d.objects[jp - 1].visible = false;
    let c = d
        .add_referenced_function_definition(
            &[6],
            SimpleFunction::parse_with_sources("v1*x+1", 1).unwrap(),
        )
        .unwrap();
    let calc = d
        .add_multi_calculation(&[c, 6], "f1(v2)", (300, 280))
        .unwrap();
    let image = d.add(Kind::PolarTranslatedPoint(
        2,
        -90.,
        37.79527559055118,
        1.5875,
    ));
    let label = d.add_point_numeric_label(image, calc).unwrap();
    d.add_point_parameter_iteration(2, 6, image, calc, label, IterationDepth::Fixed(3))
        .unwrap();
    for (i, (id, name)) in [
        (f, "f"),
        (g, "g"),
        (h, "h"),
        (a, "a"),
        (b, "b"),
        (k, "k"),
        (j, "j"),
        (c, "c"),
    ]
    .into_iter()
    .enumerate()
    {
        d.objects[id - 1].function_anchor = Some((10, 150 + 26 * i as i16));
        d.objects[id - 1].function_name = Some(name.to_owned());
    }
    d
}

#[test]
fn referenced_function_actual_api_writer_matches_native_dependencies_and_iterations() {
    let mut d = function_references_writer_document();
    let original = include_bytes!(
        "../../../../../tests/fixtures/function_references_writer_original_native_verified.gsp"
    );
    let edited = include_bytes!(
        "../../../../../tests/fixtures/function_references_writer_edited_native_verified.gsp"
    );
    assert_eq!(d.objects.len(), 38);
    assert_eq!(d.encode().unwrap(), original);
    d.verify_roundtrip().unwrap();
    let reopened = Document::from_bytes(original, None).unwrap();
    assert!(!reopened.partial_view);
    for (id, name) in [
        (9, "f"),
        (10, "g"),
        (19, "h"),
        (22, "a"),
        (24, "b"),
        (26, "k"),
        (29, "j"),
        (31, "c"),
    ] {
        assert_eq!(
            reopened.objects[id - 1].function_name.as_deref(),
            Some(name)
        );
    }
    for (id, value) in [
        (11, 33.5875),
        (20, 17.5875),
        (23, std::f64::consts::FRAC_PI_2),
        (25, 1.),
        (27, 1.),
        (32, 5.),
    ] {
        assert!((d.numeric_value(id).unwrap() - value).abs() < 1e-12);
    }
    assert_eq!(d.function_value(26, 2.), None);
    assert_eq!(d.function_value(29, 1.), None);
    let frame = d.graph_frame(17).unwrap();
    let pole = frame.world(1., 0.).unwrap().x;
    assert!(
        d.function_plot_segments(30, 750)
            .unwrap()
            .iter()
            .all(|(a, b)| (a.x - pole) * (b.x - pole) > 0.)
    );
    let labels = d.point_parameter_iteration_labels(35).unwrap();
    assert_eq!(
        labels
            .iter()
            .map(|(_, text)| text.as_str())
            .collect::<Vec<_>>(),
        ["26.00", "677.00", "458330.00"]
    );
    assert_eq!(d.numeric_value(6), Some(2.));
    assert_eq!(d.encode().unwrap(), original);
    d.set_parameter_value(6, -1.).unwrap();
    d.set_function_expression(
        9,
        SimpleFunction::parse_with_sources("v1*x^3+v2", 2).unwrap(),
    )
    .unwrap();
    d.set_function_expression_with_angle_unit(
        24,
        SimpleFunction::parse_with_sources("sin(v1)", 1).unwrap(),
        false,
    )
    .unwrap();
    for (id, value) in [
        (11, 9.5875),
        (20, -4.175),
        (23, 90.),
        (25, 1.),
        (27, 1.),
        (32, 2.),
    ] {
        assert!((d.numeric_value(id).unwrap() - value).abs() < 1e-12);
    }
    let labels = d.point_parameter_iteration_labels(35).unwrap();
    assert_eq!(
        labels
            .iter()
            .map(|(_, text)| text.as_str())
            .collect::<Vec<_>>(),
        ["5.00", "26.00", "677.00"]
    );
    assert_eq!(d.encode().unwrap(), edited);
    d.verify_roundtrip().unwrap();
    for _ in 0..3 {
        d.undo();
    }
    assert_eq!(d.encode().unwrap(), original);
    for _ in 0..3 {
        d.redo();
    }
    assert_eq!(d.encode().unwrap(), edited);
}

#[test]
fn referenced_function_invalid_sources_do_not_mutate_the_document() {
    let mut d = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/source_exponents_base_port.gsp"),
        None,
    )
    .unwrap();
    let before = d.encode().unwrap();
    let history = d.undo.len();
    let numeric = SimpleFunction::parse_with_sources("v1*x", 1).unwrap();
    for sources in [vec![0], vec![999], vec![1], vec![]] {
        assert!(
            d.add_referenced_function_definition(&sources, numeric)
                .is_err()
        );
    }
    assert!(d.add_function_definition(numeric).is_err());
    assert!(
        d.plot_new_function(numeric, Point { x: 0., y: 0. })
            .is_err()
    );
    assert!(
        d.add_referenced_function_definition(
            &[6],
            SimpleFunction::parse_with_sources("f1(x)", 1).unwrap()
        )
        .is_err()
    );
    assert_eq!(d.undo.len(), history);
    assert_eq!(d.encode().unwrap(), before);
}
