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
            include_bytes!("../../../tests/fixtures/locus_midpoint_original.gsp").as_slice(),
        ),
        (
            "function",
            include_bytes!("../../../tests/fixtures/function_plot_identity.gsp").as_slice(),
        ),
        (
            "parametric",
            include_bytes!("../../../tests/fixtures/parametric_rectangular_original.gsp")
                .as_slice(),
        ),
        (
            "polar",
            include_bytes!("../../../tests/fixtures/parametric_polar_original.gsp").as_slice(),
        ),
        (
            "parameter_locus",
            include_bytes!(
                "../../../tests/fixtures/numeric_cood_trace_added_locus_native_verified.gsp"
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
        doc.verify_roundtrip().unwrap();
        let reopened = Document::from_bytes(&saved, None).unwrap();
        for &id in &curves {
            assert_eq!(
                reopened.objects[id - 1].visual_style,
                doc.objects[id - 1].visual_style
            );
        }
        let points=curves.iter().flat_map(|id| {
            reopened.function_plot_segments(*id,384).or_else(||reopened.parametric_plot_segments(*id,384))
                .or_else(||reopened.locus_strokes(*id,384).map(|segments|segments.into_iter().map(|(a,b,_)|(a,b)).collect()))
                .unwrap_or_default().into_iter().flat_map(|(a,b)|[a,b])
        }).collect::<Vec<_>>();
        assert!(!points.is_empty(),"{name} must have defined geometry");
        let (min_x,max_x,min_y,max_y)=points.iter().fold((f64::INFINITY,f64::NEG_INFINITY,f64::INFINITY,f64::NEG_INFINITY),|(xmin,xmax,ymin,ymax),p|(xmin.min(p.x),xmax.max(p.x),ymin.min(p.y),ymax.max(p.y)));
        let vp = crate::render::Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000., 800.)),
            scale: (800./(max_x-min_x).max(1.)).min(600./(max_y-min_y).max(1.)) as f32,
            center: Point { x:(min_x+max_x)/2., y:(min_y+max_y)/2. },
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
            include_bytes!("../../../tests/fixtures/parametric_two_functions_moved_original.gsp"),
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
        include_bytes!("../../../tests/fixtures/locus_midpoint_base.gsp"),
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
        include_bytes!("../../../tests/fixtures/numeric_cood_trace_native.gsp"),
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
    for bytes in [
        include_bytes!("../../../tests/fixtures/locus_segment_target_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/locus_circle_target_original.gsp").as_slice(),
        include_bytes!("../../../tests/fixtures/locus_vector_polygon_target_original.gsp")
            .as_slice(),
    ] {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        let id = doc
            .objects
            .iter()
            .find(|o| matches!(o.kind, Kind::Locus(..)))
            .unwrap()
            .id;
        assert_eq!(doc.objects[id - 1].visual_style, None);
        doc.selected.insert(id);
        assert_eq!(
            doc.set_selected_color(RgbColor {
                red: 14,
                green: 150,
                blue: 210
            })
            .unwrap(),
            0
        );
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}
