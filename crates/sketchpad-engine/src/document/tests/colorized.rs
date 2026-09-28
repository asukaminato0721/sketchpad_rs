use super::*;
use crate::document::colorized::{decode_colorized, ColorBinding, ColorRangeBehavior};

fn rgb(r: u8, g: u8, b: u8) -> RgbColor {
    RgbColor {
        red: r,
        green: g,
        blue: b,
    }
}
fn base() -> Document {
    let mut d = Document::new();
    for (x, y) in [(100., 180.), (140., 180.), (100., 140.)] {
        d.add(Kind::Free(Point { x, y }));
    }
    d.add(Kind::Segment(1, 2));
    d.add(Kind::Ray(1, 2));
    d.add(Kind::Line(1, 2));
    d.add(Kind::Circle(1, 2));
    d.add(Kind::Polygon(vec![1, 2, 3]));
    d.add(Kind::CenterArc(1, 2, 3));
    d.add_named_parameter("q", 0.5, (10, 10)).unwrap();
    d
}

#[test]
fn colorized_geometry_keeps_identity_and_live_dependencies_after_reopen() {
    let mut d = base();
    for source in [1, 4, 5, 6, 7, 8, 9] {
        let id = d
            .add_colorized(
                source,
                ColorBinding::Grayscale { value: 10 },
                0.,
                1.,
                ColorRangeBehavior::Clamp,
            )
            .unwrap();
        assert_eq!(d.colorized_color(id), Some(rgb(127, 127, 127)));
        let Kind::Colorized(def) = &d.objects[id - 1].kind else {
            panic!()
        };
        assert_eq!(def.source(), source);
        assert_eq!(d.objects[id - 1].kind.dependencies(), vec![source, 10]);
        match source {
            1 => assert_eq!(d.position(id), d.position(source)),
            4..=6 => assert_eq!(d.line_points(id), d.line_points(source)),
            7 => assert_eq!(d.circle_points(id), d.circle_points(source)),
            8 => assert_eq!(d.polygon_points(id), d.polygon_points(source)),
            9 => assert_eq!(d.arc_points(id, 8), d.arc_points(source, 8)),
            _ => unreachable!(),
        }
    }
    let bytes = d.encode().unwrap();
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    assert!(!d.partial_view);
    d.set_parameter_value(10, 0.75).unwrap();
    assert_eq!(d.colorized_color(11), Some(rgb(191, 191, 191)));
    assert!(d.move_point(1, Point { x: 110., y: 190. }));
    assert_eq!(d.position(11), Some(Point { x: 110., y: 190. }));
    d.verify_roundtrip().unwrap();
}

#[test]
fn colorized_channels_use_native_order_and_preserve_own_visual_metadata() {
    let mut d = base();
    let b = d.add_named_parameter("b", 0.25, (10, 10)).unwrap();
    let g = d.add_named_parameter("g", 0.5, (10, 10)).unwrap();
    let r = d.add_named_parameter("r", 0.75, (10, 10)).unwrap();
    let id = d
        .add_colorized(
            8,
            ColorBinding::Rgb {
                red: r,
                green: g,
                blue: b,
            },
            0.,
            1.,
            ColorRangeBehavior::Reflect,
        )
        .unwrap();
    assert_eq!(d.objects[id - 1].kind.dependencies(), vec![8, b, g, r]);
    assert_eq!(d.colorized_color(id), Some(rgb(191, 127, 63)));
    d.objects[id - 1].visual_style = Some(VisualStyle::Polygon {
        opacity: 12345,
        color: rgb(1, 2, 3),
    });
    assert_eq!(
        d.effective_visual_style(id),
        Some(VisualStyle::Polygon {
            opacity: 12345,
            color: rgb(191, 127, 63)
        })
    );
    let hsv = d
        .add_colorized(
            7,
            ColorBinding::Hsv {
                hue: r,
                saturation: g,
                value: b,
            },
            0.,
            1.,
            ColorRangeBehavior::Clamp,
        )
        .unwrap();
    assert_eq!(d.colorized_color(hsv), Some(rgb(47, 31, 63)));
    let chain = d
        .add_colorized(
            id,
            ColorBinding::Grayscale { value: 10 },
            0.,
            1.,
            ColorRangeBehavior::Clamp,
        )
        .unwrap();
    assert_eq!(d.polygon_points(chain), d.polygon_points(8));
    assert_eq!(d.colorized_color(chain), Some(rgb(127, 127, 127)));
    d.verify_roundtrip().unwrap();
}

#[test]
fn colorized_native_range_behaviors_include_signed_integer_endpoints() {
    let mut d = base();
    let ids: Vec<_> = [
        ColorRangeBehavior::Clamp,
        ColorRangeBehavior::Cycle,
        ColorRangeBehavior::Reflect,
    ]
    .into_iter()
    .map(|mode| {
        d.add_colorized(8, ColorBinding::Grayscale { value: 10 }, 0., 1., mode)
            .unwrap()
    })
    .collect();
    for (value, expected) in [
        (-0.25, [0, 191, 63]),
        (1.25, [255, 63, 191]),
        (-1., [0, 255, 255]),
        (0., [0, 0, 0]),
        (1., [255, 255, 255]),
        (2., [255, 0, 0]),
    ] {
        d.set_parameter_value(10, value).unwrap();
        for (&id, c) in ids.iter().zip(expected) {
            assert_eq!(
                d.colorized_color(id),
                Some(rgb(c, c, c)),
                "q={value}, id={id}"
            );
        }
    }
    let reversed = d
        .add_colorized(
            8,
            ColorBinding::Grayscale { value: 10 },
            1.,
            0.,
            ColorRangeBehavior::Clamp,
        )
        .unwrap();
    d.set_parameter_value(10, 0.25).unwrap();
    assert_eq!(d.colorized_color(reversed), Some(rgb(191, 191, 191)));
    d.verify_roundtrip().unwrap();
}

#[test]
fn colorized_rejects_malformed_roles_layout_and_nonfinite_ranges() {
    let mut d = base();
    let count = d.objects.len();
    for (a, b) in [(f64::NAN, 1.), (0., f64::INFINITY)] {
        assert!(d
            .add_colorized(
                8,
                ColorBinding::Spectrum { value: 10 },
                a,
                b,
                ColorRangeBehavior::Clamp
            )
            .is_err());
    }
    assert!(d
        .add_colorized(
            10,
            ColorBinding::Spectrum { value: 10 },
            0.,
            1.,
            ColorRangeBehavior::Clamp
        )
        .is_err());
    assert!(d
        .add_colorized(
            8,
            ColorBinding::Spectrum { value: 1 },
            0.,
            1.,
            ColorRangeBehavior::Clamp
        )
        .is_err());
    assert_eq!(d.objects.len(), count);
    let id = d
        .add_colorized(
            8,
            ColorBinding::Spectrum { value: 10 },
            0.,
            1.,
            ColorRangeBehavior::Reflect,
        )
        .unwrap();
    let Kind::Colorized(def) = &d.objects[id - 1].kind else {
        panic!()
    };
    let mut header = [0; 28];
    header[..2].copy_from_slice(&75u16.to_le_bytes());
    header[6..8].copy_from_slice(&4u16.to_le_bytes());
    header[8..10].copy_from_slice(
        &crate::document::records::polygon_subtype(3)
            .unwrap()
            .to_le_bytes(),
    );
    let payload = def.native_payload();
    assert!(decode_colorized(&d.objects, 0, id, &header, &[8, 10], Some(&payload)).is_some());
    for refs in [vec![8], vec![8, 1], vec![8, 10, 10], vec![8, id]] {
        assert!(decode_colorized(&d.objects, 0, id, &header, &refs, Some(&payload)).is_none());
    }
    header[6] = 3;
    assert!(decode_colorized(&d.objects, 0, id, &header, &[8, 10], Some(&payload)).is_none());
}

#[test]
fn colorized_native_controls_decode_exact_and_match_channel_values() {
    for name in [
        "basic",
        "behaviors",
        "units",
        "endpoints",
        "sources_spectrum",
        "sources_rgb",
        "sources_hsv",
        "sources_chain_rgb",
    ] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/color_wrapper50_{name}_native_verified.gsp"
        ))
        .unwrap();
        let d = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(d.encode().unwrap(), bytes, "{name}");
        if name != "behaviors" {
            assert!(!d.partial_view, "{name}");
        }
        d.verify_roundtrip().unwrap();
        if name == "units" {
            for id in [7, 14, 21] {
                assert_eq!(d.colorized_color(id), Some(rgb(69, 249, 0)));
            }
        }
        if name == "sources_chain_rgb" {
            assert_eq!(d.colorized_color(34), Some(rgb(0, 111, 239)));
            assert_eq!(d.position(35), Some(Point { x: 230., y: 55. }));
        }
        if name == "endpoints" {
            for (id, c) in [(119, 255), (126, 0), (133, 255), (140, 0)] {
                assert_eq!(d.colorized_color(id), Some(rgb(c, c, c)));
            }
        }
    }
}

#[test]
fn colorized_dependency_remapping_cascades_and_undo_restores_exact_bytes() {
    let mut d = base();
    let unused = d.add(Kind::Free(Point { x: 0., y: 0. }));
    let id = d
        .add_colorized(
            8,
            ColorBinding::Grayscale { value: 10 },
            0.,
            1.,
            ColorRangeBehavior::Clamp,
        )
        .unwrap();
    d.selected.insert(unused);
    d.clear_selected().unwrap();
    d.verify_roundtrip().unwrap();
    let bytes = d.encode().unwrap();
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    let wrapper = d
        .objects
        .iter()
        .find(|o| matches!(o.kind, Kind::Colorized(_)))
        .unwrap()
        .id;
    assert_eq!(wrapper, id - 1);
    assert_eq!(d.colorized_color(wrapper), Some(rgb(127, 127, 127)));
    d.selected.insert(10);
    d.clear_selected().unwrap();
    assert!(d.objects[wrapper - 1].deleted);
    d.verify_roundtrip().unwrap();
    d.undo();
    assert_eq!(d.encode().unwrap(), bytes);
}

#[test]
fn colorized_context_freezes_omitted_wrapper_but_carried_wrapper_uses_substitution() {
    let mut d = base();
    let id = d
        .add_colorized(
            8,
            ColorBinding::Grayscale { value: 10 },
            0.,
            1.,
            ColorRangeBehavior::Clamp,
        )
        .unwrap();
    let mut carried = d.clone();
    carried.freeze_evaluation_inputs(&d, &[]).unwrap();
    carried.set_evaluation_number(10, 0.75).unwrap();
    assert_eq!(carried.colorized_color(id), Some(rgb(191, 191, 191)));
    let mut frozen = d.clone();
    frozen.freeze_evaluation_inputs(&d, &[id]).unwrap();
    frozen.set_evaluation_number(10, 0.75).unwrap();
    assert_eq!(frozen.colorized_color(id), Some(rgb(127, 127, 127)));
    assert_eq!(frozen.polygon_points(id), d.polygon_points(8));
    assert_eq!(carried.encode().unwrap(), d.encode().unwrap());
}

#[test]
fn colorized_equal_range_and_undefined_numeric_follow_native_controls() {
    let mut d = base();
    let equal = d
        .add_colorized(
            8,
            ColorBinding::Grayscale { value: 10 },
            1.,
            1.,
            ColorRangeBehavior::Clamp,
        )
        .unwrap();
    assert_eq!(d.colorized_color(equal), Some(rgb(255, 255, 255)));
    let calc = d.add_parameter_calculation(10, "1/q", (10, 10)).unwrap();
    let id = d
        .add_colorized(
            8,
            ColorBinding::Spectrum { value: calc },
            0.,
            1.,
            ColorRangeBehavior::Clamp,
        )
        .unwrap();
    d.set_parameter_value(10, 0.).unwrap();
    assert_eq!(d.colorized_color(id), None);
    assert_eq!(d.effective_visual_style(id), None);
    assert_eq!(d.polygon_points(id), d.polygon_points(8));
    d.verify_roundtrip().unwrap();
    d.set_parameter_value(10, 2.).unwrap();
    assert_eq!(d.colorized_color(id), Some(rgb(69, 249, 0)));
    let bytes = fs::read("../../tests/fixtures/color_wrapper50_edgecases_native_verified.gsp").unwrap();
    let d = Document::from_bytes(&bytes, None).unwrap();
    assert!(!d.partial_view);
    let area_color = d.colorized_color(82).unwrap();
    // The geometric Area is 0.5 within floating precision; optical Spectrum
    // interpolation can fall one channel unit below a sampled knot.
    assert_eq!(area_color.red, 69);
    assert!(area_color.green.abs_diff(249) <= 1);
    assert_eq!(area_color.blue, 0);
    for id in [50, 60, 70] {
        assert_eq!(d.colorized_color(id), None);
    }
    assert_eq!(d.encode().unwrap(), bytes);
    d.verify_roundtrip().unwrap();
}
