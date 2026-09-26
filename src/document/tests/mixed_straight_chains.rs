use super::*;
use crate::style_records::{LinePattern, LineWidth, RgbColor};
fn build(reciprocal: bool) -> Document {
    let mut d = Document::new();
    for p in [(130., 150.), (170., 190.), (210., 190.)] {
        d.add(Kind::Free(Point { x: p.0, y: p.1 }));
    }
    d.add(Kind::Ray(2, 3));
    d.add(Kind::Line(2, 3));
    d.add_named_parameter("n", 4., (50, 30)).unwrap();
    d.add_parameter_calculation(6, "360°/n", (50, 60)).unwrap();
    d.add_parameter_calculation(6, if reciprocal { "2/n" } else { "n/2" }, (50, 90))
        .unwrap();
    d.add_named_parameter("j", 4., (450, 30)).unwrap();
    d.add_parameter_calculation(9, "360°/j", (450, 60)).unwrap();
    d.add_parameter_calculation(9, "j/2", (450, 90)).unwrap();
    d.selected.extend([4, 5]);
    d.set_selected_color(RgbColor {
        red: 255,
        green: 0,
        blue: 255,
    })
    .unwrap();
    d.set_selected_line_width(LineWidth::Thick).unwrap();
    d.set_selected_line_pattern(LinePattern::Dashed).unwrap();
    d.selected.clear();
    d.add(Kind::RotatedRay(4, 1, 45.));
    d.add(Kind::RotatedLine(5, 1, 45.));
    d.add(Kind::ScaledRay(4, 1, 2.));
    d.add(Kind::ScaledLine(5, 1, 2.));
    d.add_calculated_angle_rotation(4, 1, 7).unwrap();
    d.add_calculated_angle_rotation(5, 1, 7).unwrap();
    d.add_calculated_ratio_dilation(4, 1, 8).unwrap();
    d.add_calculated_ratio_dilation(5, 1, 8).unwrap();
    for source in 12..=19 {
        let ray = source % 2 == 0;
        d.add(if ray {
            Kind::RotatedRay(source, 1, 90.)
        } else {
            Kind::RotatedLine(source, 1, 90.)
        });
        d.add(if ray {
            Kind::ScaledRay(source, 1, 2.)
        } else {
            Kind::ScaledLine(source, 1, 2.)
        });
        d.add_calculated_angle_rotation(source, 1, 10).unwrap();
        d.add_calculated_ratio_dilation(source, 1, 11).unwrap();
    }
    d
}

fn close(a: Point, b: Point) {
    assert!(
        (a.x - b.x).abs() < 1e-8 && (a.y - b.y).abs() < 1e-8,
        "{a:?} != {b:?}"
    );
}
fn golden(d: &Document, name: &str) {
    d.verify_roundtrip().unwrap();
    assert_eq!(
        d.encode().unwrap(),
        fs::read(format!(
            "tests/fixtures/mixed_chain_writer_{name}_native_verified.gsp"
        ))
        .unwrap()
    );
}
#[test]
fn native_mixed_straight_sources_preserve_modes_refs_and_strokes() {
    for source in [
        "fixed_rotation",
        "fixed_dilation",
        "numeric_angle",
        "numeric_ratio",
    ] {
        for (mode, extent) in [("ray", LineMode::Ray), ("line", LineMode::Line)] {
            let bytes = fs::read(format!(
                "tests/fixtures/mixed_straight_chain_{source}_{mode}_original.gsp"
            ))
            .unwrap();
            let d = Document::from_bytes(&bytes, None).unwrap();
            assert!(!d.partial_view);
            assert_eq!(d.objects.len(), 15);
            assert_eq!(d.encode().unwrap(), bytes);
            let kinds = if mode == "ray" {
                [
                    Kind::RotatedRay(8, 1, 90.),
                    Kind::ScaledRay(8, 1, 2.),
                    Kind::CalculatedAngleRotatedRay(8, 1, 10),
                    Kind::CalculatedRatioScaledRay(8, 1, 11),
                ]
            } else {
                [
                    Kind::RotatedLine(8, 1, 90.),
                    Kind::ScaledLine(8, 1, 2.),
                    Kind::CalculatedAngleRotatedLine(8, 1, 10),
                    Kind::CalculatedRatioScaledLine(8, 1, 11),
                ]
            };
            for (id, kind) in (12..=15).zip(kinds) {
                assert_eq!(d.objects[id - 1].kind, kind);
                assert_eq!(d.line_points(id).unwrap().2, extent);
                assert_eq!(d.objects[id - 1].visual_style, d.objects[7].visual_style);
                assert!(!kind.is_segment());
            }
        }
    }
}
#[test]
fn mixed_straight_writer_chains_follow_live_inputs_and_recover() {
    let fresh = build(false);
    assert_eq!(fresh.objects.len(), 51);
    for source in 12..=19 {
        let (a, b, mode) = fresh.line_points(source).unwrap();
        let center = fresh.position(1).unwrap();
        for (index, id) in (20 + (source - 12) * 4..24 + (source - 12) * 4).enumerate() {
            let (x, y, child_mode) = fresh.line_points(id).unwrap();
            assert_eq!(child_mode, mode);
            let expected = |p: Point| {
                let dx = p.x - center.x;
                let dy = p.y - center.y;
                if index == 0 || index == 2 {
                    Point {
                        x: center.x + dy,
                        y: center.y - dx,
                    }
                } else {
                    Point {
                        x: center.x + 2. * dx,
                        y: center.y + 2. * dy,
                    }
                }
            };
            close(x, expected(a));
            close(y, expected(b));
            assert_eq!(
                fresh.objects[id - 1].visual_style,
                fresh.objects[3].visual_style
            );
        }
    }
    golden(&fresh, "fresh");
    let mut edited = fresh.clone();
    edited.set_parameter_value(6, 8.).unwrap();
    edited.set_parameter_value(9, 8.).unwrap();
    for (id, x, y) in [(1, 140., 160.), (2, 180., 210.), (3, 240., 220.)] {
        assert!(edited.move_point(id, Point { x, y }));
    }
    for id in 12..=51 {
        assert_ne!(edited.line_points(id), fresh.line_points(id));
    }
    golden(&edited, "edited");
    let mut negative = fresh.clone();
    negative.set_parameter_value(6, -4.).unwrap();
    negative.set_parameter_value(9, -4.).unwrap();
    golden(&negative, "negative");
    for reciprocal in [false, true] {
        let mut d = build(reciprocal);
        d.set_parameter_value(6, 0.).unwrap();
        for id in (16..=19).chain(36..=51) {
            assert!(d.line_points(id).is_none(), "{id}");
        }
        for id in (12..=15).chain(20..=35) {
            assert!(d.line_points(id).is_some(), "{id}");
        }
        for o in &mut d.objects {
            o.visible = matches!(o.id,6..=11|16..=19|36..=51);
        }
        golden(
            &d,
            if reciprocal {
                "source_undefined"
            } else {
                "source_zero"
            },
        );
        let mut reopened = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
        reopened.set_parameter_value(6, 4.).unwrap();
        for id in 12..=51 {
            assert!(reopened.line_points(id).is_some());
        }
        reopened.verify_roundtrip().unwrap();
        d.set_parameter_value(6, 4.).unwrap();
        for id in 12..=51 {
            assert!(d.line_points(id).is_some());
        }
        d.verify_roundtrip().unwrap();
    }
    let mut d = fresh.clone();
    d.set_parameter_value(9, 0.).unwrap();
    for source in 12..=19 {
        let first = 20 + (source - 12) * 4;
        assert!(d.line_points(first).is_some());
        assert!(d.line_points(first + 1).is_some());
        assert!(d.line_points(first + 2).is_none());
        assert!(d.line_points(first + 3).is_none());
    }
    golden(&d, "child_zero");
    let mut d = fresh.clone();
    d.objects[13].kind = Kind::ScaledRay(4, 1, 0.);
    d.objects[14].kind = Kind::ScaledLine(5, 1, 0.);
    for id in (14..=15).chain(28..=35) {
        assert!(d.line_points(id).is_none());
    }
    for o in &mut d.objects {
        o.visible = matches!(o.id,6..=11|14..=15|28..=35);
    }
    golden(&d, "fixed_zero");
    let mut d = fresh.clone();
    d.selected.extend(44..=47);
    let copied = d.paste(&d.copy_selected().unwrap()).unwrap();
    assert_eq!(copied.len(), 14);
    // The four selected images are the last four dependency-ordered copies.
    for (id, original) in (62..=65).zip(44..=47) {
        let (a, b, mode) = d.line_points(id).unwrap();
        let (x, y, original_mode) = d.line_points(original).unwrap();
        assert_eq!(mode, original_mode);
        close(
            a,
            Point {
                x: x.x + 20.,
                y: x.y + 20.,
            },
        );
        close(
            b,
            Point {
                x: y.x + 20.,
                y: y.y + 20.,
            },
        );
    }
    d.selected.clear();
    d.selected.insert(6);
    d.clear_selected().unwrap();
    for id in 36..=51 {
        assert!(d.objects[id - 1].deleted);
    }
    for id in 62..=65 {
        assert!(!d.objects[id - 1].deleted);
        assert!(d.line_points(id).is_some());
    }
    golden(&d, "copied_deleted");
}

#[test]
fn mixed_straight_transform_guards_keep_unproved_hosts_and_units_closed() {
    let mut d = build(false);
    let before = d.encode().unwrap();
    assert!(d.add_calculated_angle_rotation(18, 1, 11).is_err());
    assert!(d.add_calculated_ratio_dilation(16, 1, 10).is_err());
    assert_eq!(d.encode().unwrap(), before);
    for kind in [
        Kind::GraphAxis(2, 3, 1),
        Kind::Parallel(2, 5),
        Kind::Perpendicular(2, 5),
        Kind::Bisector(2, 1, 3),
    ] {
        assert_eq!(kind.fixed_transform_straight_mode(), None);
        let id = d.add(kind);
        assert!(d.add_calculated_angle_rotation(id, 1, 10).is_err());
        assert!(d.add_calculated_ratio_dilation(id, 1, 11).is_err());
    }
    let mut bad = build(false);
    bad.objects[35].kind = Kind::RotatedLine(16, 1, 90.);
    assert!(bad.encode().is_err());
    let mut bad = build(false);
    bad.objects[38].kind = Kind::CalculatedRatioScaledLine(16, 1, 11);
    assert!(bad.encode().is_err());
}
