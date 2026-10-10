use super::*;

fn is_curve(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::FunctionPlot(..)
            | Kind::ParametricPlot(..)
            | Kind::Locus(..)
            | Kind::ParameterLocus(..)
    )
}

#[test]
fn native_curve_strokes_are_editable_and_svg_keeps_color_width_and_pattern() {
    for (name, bytes) in [
        (
            "point_locus",
            include_bytes!("../../../../../tests/fixtures/locus_midpoint_original.gsp").as_slice(),
        ),
        (
            "function",
            include_bytes!("../../../../../tests/fixtures/function_plot_identity.gsp").as_slice(),
        ),
        (
            "parametric",
            include_bytes!("../../../../../tests/fixtures/parametric_rectangular_original.gsp")
                .as_slice(),
        ),
        (
            "polar",
            include_bytes!("../../../../../tests/fixtures/parametric_polar_original.gsp")
                .as_slice(),
        ),
        (
            "parameter_locus",
            include_bytes!(
                "../../../../../tests/fixtures/numeric_cood_trace_added_locus_native_verified.gsp"
            )
            .as_slice(),
        ),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view, "{name}");
        assert_eq!(doc.encode().unwrap(), bytes);
        let curves: Vec<_> = doc
            .objects
            .iter()
            .filter(|o| is_curve(&o.kind))
            .map(|o| o.id)
            .collect();
        assert!(!curves.is_empty());
        assert!(curves.iter().all(|id| matches!(
            doc.objects[*id - 1].visual_style,
            Some(VisualStyle::Stroke { .. })
        )));
        doc.selected.extend(&curves);
        assert_eq!(
            doc.set_selected_color(RgbColor {
                red: 14,
                green: 150,
                blue: 210
            })
            .unwrap(),
            curves.len()
        );
        assert_eq!(
            doc.set_selected_line_width(LineWidth::Thick).unwrap(),
            curves.len()
        );
        let pattern = if name == "polar" {
            LinePattern::Dotted
        } else {
            LinePattern::Dashed
        };
        assert_eq!(
            doc.set_selected_line_pattern(pattern).unwrap(),
            curves.len()
        );
        let saved = doc.encode().unwrap();
        let native_verified = match name {
            "point_locus" => include_bytes!(
                "../../../../../tests/fixtures/curve_style_point_locus_native_verified.gsp"
            )
            .as_slice(),
            "function" => include_bytes!(
                "../../../../../tests/fixtures/curve_style_function_native_verified.gsp"
            )
            .as_slice(),
            "parametric" => include_bytes!(
                "../../../../../tests/fixtures/curve_style_parametric_native_verified.gsp"
            )
            .as_slice(),
            "polar" => include_bytes!(
                "../../../../../tests/fixtures/curve_style_polar_native_verified.gsp"
            )
            .as_slice(),
            "parameter_locus" => include_bytes!(
                "../../../../../tests/fixtures/curve_style_parameter_locus_native_verified.gsp"
            )
            .as_slice(),
            _ => unreachable!(),
        };
        assert_eq!(
            saved.as_slice(),
            native_verified,
            "{name} native saved bytes"
        );
        doc.verify_roundtrip().unwrap();
        let reopened = Document::from_bytes(&saved, None).unwrap();
        for &id in &curves {
            assert_eq!(
                reopened.objects[id - 1].visual_style,
                doc.objects[id - 1].visual_style
            );
        }
        let points = curves
            .iter()
            .flat_map(|id| {
                reopened
                    .function_plot_segments(*id, 384)
                    .or_else(|| reopened.parametric_plot_segments(*id, 384))
                    .or_else(|| {
                        reopened
                            .locus_strokes(*id, 384)
                            .map(|segments| segments.into_iter().map(|(a, b, _)| (a, b)).collect())
                    })
                    .unwrap_or_default()
                    .into_iter()
                    .flat_map(|(a, b)| [a, b])
            })
            .collect::<Vec<_>>();
        assert!(!points.is_empty(), "{name} must have defined geometry");
        let (min_x, max_x, min_y, max_y) = points.iter().fold(
            (
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ),
            |(xmin, xmax, ymin, ymax), p| {
                (xmin.min(p.x), xmax.max(p.x), ymin.min(p.y), ymax.max(p.y))
            },
        );
        let vp = crate::render::Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000., 800.)),
            scale: (800. / (max_x - min_x).max(1.)).min(600. / (max_y - min_y).max(1.)) as f32,
            center: Point {
                x: (min_x + max_x) / 2.,
                y: (min_y + max_y) / 2.,
            },
        };
        let svg = crate::render::svg_page(&reopened, &vp).unwrap();
        assert!(
            svg.contains("stroke=\"#0e96d2\" stroke-width=\"4\""),
            "{name}"
        );
        assert!(
            svg.contains(if pattern == LinePattern::Dotted {
                "stroke-dasharray=\"2 5\""
            } else {
                "stroke-dasharray=\"9 6\""
            }),
            "{name}"
        );
        if std::env::var_os("SKETCHPAD_WRITE_CURVE_FIXTURE").is_some() {
            fs::write(format!("/tmp/curve_style_{name}.gsp"), saved).unwrap();
        }
    }
}

#[test]
fn authored_curve_defaults_match_native_templates_and_sweeps_keep_their_layout() {
    let mut function = Document::new();
    function
        .plot_new_function(SimpleFunction::Square, Point { x: 199., y: 125. })
        .unwrap();
    function.verify_roundtrip().unwrap();
    let id = function
        .objects
        .iter()
        .find(|o| matches!(o.kind, Kind::FunctionPlot(..)))
        .unwrap()
        .id;
    assert!(matches!(
        function.objects[id - 1].visual_style,
        Some(VisualStyle::Stroke {
            line: SegmentLineStyle {
                width: LineWidth::Thin,
                pattern: LinePattern::Solid
            },
            color: RgbColor {
                red: 128,
                green: 0,
                blue: 128
            }
        })
    ));
    for mode in [PlotPointsMode::Cartesian, PlotPointsMode::Polar] {
        let mut parametric = Document::from_bytes(
            include_bytes!(
                "../../../../../tests/fixtures/parametric_two_functions_moved_original.gsp"
            ),
            None,
        )
        .unwrap();
        let id = parametric
            .plot_parametric_curve(1, 2, mode, 0., 1., Point { x: 199., y: 125. })
            .unwrap();
        assert!(matches!(
            parametric.objects[id - 1].visual_style,
            Some(VisualStyle::Stroke {
                line: SegmentLineStyle {
                    width: LineWidth::Medium,
                    ..
                },
                color: RgbColor {
                    red: 128,
                    green: 0,
                    blue: 128
                }
            })
        ));
        parametric.verify_roundtrip().unwrap();
    }
    let mut point = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/locus_midpoint_base.gsp"),
        None,
    )
    .unwrap();
    let id = point.construct_locus(4, 7).unwrap();
    assert_eq!(
        point.objects[id - 1].visual_style,
        Some(super::super::kind::point_locus_visual_style())
    );
    point.verify_roundtrip().unwrap();
    let mut parameter = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/numeric_cood_trace_native.gsp"),
        None,
    )
    .unwrap();
    let id = parameter
        .construct_parameter_locus(1, 9, 0., 1., 500)
        .unwrap();
    assert_eq!(
        parameter.objects[id - 1].visual_style,
        Some(super::super::kind::point_locus_visual_style())
    );
    parameter.verify_roundtrip().unwrap();
    let endpoint = parameter.add(Kind::Free(Point { x: 420., y: 180. }));
    let segment = parameter.add(Kind::Segment(9, endpoint));
    let family = parameter
        .construct_parameter_locus(1, segment, 0., 1., 8)
        .unwrap();
    assert_eq!(parameter.locus_strokes(family, 4).unwrap().len(), 4);
    let reopened = Document::from_bytes(&parameter.encode().unwrap(), None).unwrap();
    assert!(matches!(
        reopened.objects[family - 1].kind,
        Kind::ParameterLocus(_, 0., 1., 8)
    ));
    let mut calculated = Document::new();
    let source = calculated.add(Kind::Free(Point { x: 10., y: 0. }));
    let center = calculated.add(Kind::Free(Point { x: 0., y: 0. }));
    let parameter = calculated.add_named_parameter("n", 1., (10, 20)).unwrap();
    let calculation = calculated
        .add_parameter_calculation(parameter, "n+1", (10, 50))
        .unwrap();
    let image = calculated.add(Kind::CalculatedRatioScaledPoint(
        source,
        center,
        calculation,
    ));
    let trace = calculated
        .construct_parameter_locus(calculation, image, 0., 2., 8)
        .unwrap();
    let strokes = calculated.locus_strokes(trace, 4).unwrap();
    assert_eq!(strokes.len(), 3);
    assert!((strokes[0].0.x - 0.).abs() < 1e-9);
    assert!((strokes[2].1.x - 20.).abs() < 1e-9);
    let endpoint = calculated.add(Kind::Free(Point { x: 0., y: 10. }));
    let segment = calculated.add(Kind::Segment(image, endpoint));
    let family = calculated
        .construct_parameter_locus(calculation, segment, 0., 2., 8)
        .unwrap();
    assert_eq!(calculated.locus_strokes(family, 4).unwrap().len(), 4);
    let vertical = calculated.add(Kind::Free(Point { x: 0., y: 10. }));
    let angle = calculated.add(Kind::AngleMeasurement(source, center, vertical));
    let angle_calculation = calculated
        .add_parameter_calculation(angle, "v/2", (10, 80))
        .unwrap();
    let rotated = calculated.add(Kind::CalculatedAngleRotatedPoint(
        source,
        center,
        angle_calculation,
    ));
    let angle_trace = calculated
        .construct_parameter_locus(angle_calculation, rotated, 0., 180., 8)
        .unwrap();
    let strokes = calculated.locus_strokes(angle_trace, 4).unwrap();
    assert!((strokes[0].0.x - 10.).abs() < 1e-9);
    assert!((strokes[2].1.x + 10.).abs() < 1e-9);
    let mut reopened = Document::from_bytes(&calculated.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.plot_range(trace), Some((0., 2., 8)));
    assert_eq!(reopened.plot_range(angle_trace), Some((0., 180., 8)));
    assert_eq!(reopened.locus_strokes(trace, 4).unwrap().len(), 3);
    assert_eq!(reopened.locus_strokes(family, 4).unwrap().len(), 4);
    assert_eq!(reopened.locus_strokes(angle_trace, 4).unwrap().len(), 3);
    reopened.set_plot_range(angle_trace, -90., 90., 16).unwrap();
    let edited = Document::from_bytes(&reopened.encode().unwrap(), None).unwrap();
    assert_eq!(edited.plot_range(angle_trace), Some((-90., 90., 16)));
    for (bytes, expected) in [
        (
            include_bytes!("../../../../../tests/fixtures/locus_segment_target_original.gsp")
                .as_slice(),
            Some(VisualStyle::Stroke {
                line: SegmentLineStyle {
                    width: LineWidth::Medium,
                    pattern: LinePattern::Solid,
                },
                color: RgbColor {
                    red: 102,
                    green: 102,
                    blue: 178,
                },
            }),
        ),
        (
            include_bytes!("../../../../../tests/fixtures/locus_circle_target_original.gsp")
                .as_slice(),
            None,
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/locus_vector_polygon_target_original.gsp"
            )
            .as_slice(),
            Some(VisualStyle::Polygon {
                color: RgbColor {
                    red: 255,
                    green: 178,
                    blue: 102,
                },
                opacity: 32767,
            }),
        ),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        let id = doc
            .objects
            .iter()
            .find(|o| matches!(o.kind, Kind::Locus(..)))
            .unwrap()
            .id;
        assert_eq!(doc.objects[id - 1].visual_style, expected);
        assert_eq!(doc.encode().unwrap(), bytes);
        doc.selected.insert(id);
        assert_eq!(
            doc.set_selected_color(RgbColor {
                red: 14,
                green: 150,
                blue: 210
            })
            .unwrap(),
            usize::from(expected.is_some())
        );
        if let Some(VisualStyle::Stroke { line, .. }) = expected {
            let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
            assert_eq!(
                reopened.objects[id - 1].visual_style,
                Some(VisualStyle::Stroke {
                    line,
                    color: RgbColor {
                        red: 14,
                        green: 150,
                        blue: 210
                    }
                })
            );
        } else if let Some(VisualStyle::Polygon { opacity, .. }) = expected {
            let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
            assert_eq!(
                reopened.objects[id - 1].visual_style,
                Some(VisualStyle::Polygon {
                    color: RgbColor {
                        red: 14,
                        green: 150,
                        blue: 210
                    },
                    opacity,
                })
            );
        } else {
            assert_eq!(doc.encode().unwrap(), bytes);
        }
    }

    let custom_defaults = crate::document::ObjectColorDefaults {
        point_locus: RgbColor {
            red: 12,
            green: 34,
            blue: 56,
        },
        plot: RgbColor {
            red: 65,
            green: 43,
            blue: 21,
        },
        ..crate::document::ObjectColorDefaults::default()
    };
    let mut custom_styles = crate::document::ObjectStyleDefaults::default();
    assert!(custom_styles.set_line_width(crate::document::ObjectClass::Plot, LineWidth::Thick));
    assert!(custom_styles.set_line_pattern(
        crate::document::ObjectClass::PointLocus,
        LinePattern::Dashed
    ));
    let mut custom_plot = Document::new();
    custom_plot.set_object_color_defaults(custom_defaults);
    custom_plot.set_object_style_defaults(custom_styles);
    custom_plot
        .plot_new_function(SimpleFunction::Square, Point { x: 199., y: 125. })
        .unwrap();
    let plot = custom_plot
        .objects
        .iter()
        .find(|object| matches!(object.kind, Kind::FunctionPlot(..)))
        .unwrap();
    assert_eq!(
        custom_plot.object_class(plot.id),
        Some(crate::document::ObjectClass::Plot)
    );
    assert!(matches!(
        plot.visual_style,
        Some(VisualStyle::Stroke {
            line: SegmentLineStyle {
                width: LineWidth::Thick,
                ..
            },
            color,
        }) if color == custom_defaults.plot
    ));
    custom_plot.verify_roundtrip().unwrap();

    let mut custom_locus = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/locus_midpoint_base.gsp"),
        None,
    )
    .unwrap();
    custom_locus.set_object_color_defaults(custom_defaults);
    custom_locus.set_object_style_defaults(custom_styles);
    let locus = custom_locus.construct_locus(4, 7).unwrap();
    assert_eq!(
        custom_locus.object_class(locus),
        Some(crate::document::ObjectClass::PointLocus)
    );
    assert!(matches!(
        custom_locus.objects[locus - 1].visual_style,
        Some(VisualStyle::Stroke {
            line: SegmentLineStyle {
                pattern: LinePattern::Dashed,
                ..
            },
            color,
        }) if color == custom_defaults.point_locus
    ));
    custom_locus.verify_roundtrip().unwrap();
}
