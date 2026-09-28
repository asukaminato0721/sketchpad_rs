use super::super::kind::{locus_straight_mode, locus_straight_source_mode};
use super::*;
use crate::document::colorized::{ColorBinding, ColorRangeBehavior};

struct Straight63Case {
    state: &'static str,
    row: usize,
    host: usize,
    driver: usize,
    target: usize,
    outer: usize,
    mode: LineMode,
    start_marker: usize,
    end_marker: usize,
    length: usize,
    live: (Point, Point),
    samples: usize,
    refs: &'static [usize],
    expected: &'static [Option<(Point, Point)>],
}
include!("locus_generic_straights63_cases.rs");

fn document63(state: &str) -> (Vec<u8>, Document) {
    let bytes = fs::read(format!(
        "../../tests/fixtures/locus_generic_straights63_{state}_native_saved.gsp"
    ))
    .unwrap();
    let doc = Document::from_bytes(&bytes, None).unwrap();
    (bytes, doc)
}
fn close63(a: Point, b: Point) {
    assert!(a.distance(b) < 1e-6, "{a:?} != {b:?}");
}

#[test]
fn native_generic_straight_families_reuse_exact_source_geometry_and_extent() {
    for case in straight63_cases() {
        let (bytes, doc) = document63(case.state);
        assert!(!doc.partial_view, "{} row{}", case.state, case.row);
        assert_eq!(doc.encode().unwrap(), bytes);
        assert_eq!(
            locus_straight_source_mode(&doc.objects, case.target),
            Some(case.mode)
        );
        let Kind::Locus(refs, _) = &doc.objects[case.outer - 1].kind else {
            panic!("{}", case.state);
        };
        assert_eq!(refs, case.refs);
        let (a, b, mode) = doc.line_points(case.target).unwrap();
        close63(a, case.live.0);
        close63(b, case.live.1);
        assert_eq!(mode, case.mode);
        close63(doc.position(case.start_marker).unwrap(), case.live.0);
        close63(doc.position(case.end_marker).unwrap(), case.live.1);
        assert!(
            (doc.numeric_value(case.length).unwrap() - a.distance(b) / 37.79527559055118).abs()
                < 1e-6
        );
        // Explicit presentation query budget; independent producer coordinates.
        let actual = doc.locus_strokes(case.outer, case.samples).unwrap();
        let expected: Vec<_> = case.expected.iter().flatten().collect();
        assert_eq!(
            actual.len(),
            expected.len(),
            "{} row{}",
            case.state,
            case.row
        );
        for ((a, b, mode), (x, y)) in actual.into_iter().zip(expected) {
            close63(a, *x);
            close63(b, *y);
            assert_eq!(mode, case.mode);
        }
        assert_eq!(
            doc.point_on_curve(case.outer, 0.5),
            None,
            "straight families are not point traces"
        );
    }
}

#[test]
fn public_straight_locus_authoring_preserves_roles_style_and_save_reopen() {
    for case in straight63_cases() {
        let (_, mut doc) = document63(case.state);
        let created = doc.construct_locus(case.driver, case.target).unwrap();
        let Kind::Locus(ref refs, _) = doc.objects[created - 1].kind else {
            panic!();
        };
        assert_eq!(refs, case.refs);
        let style = doc.objects[created - 1].visual_style;
        assert!(matches!(style, Some(VisualStyle::Stroke { .. })));
        let before = doc.locus_styled_strokes(created, case.samples).unwrap();
        let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
        assert!(!reopened.partial_view, "{} row{}", case.state, case.row);
        assert_eq!(reopened.objects[created - 1].visual_style, style);
        assert_eq!(
            reopened.locus_styled_strokes(created, case.samples),
            Some(before)
        );
    }
}

#[test]
fn independently_native_saved_api_straight_families_preserve_live_roles() {
    for case in straight63_cases() {
        let bytes = fs::read(format!(
            "../../tests/fixtures/locus_generic_straights63_api_{}_native_saved.gsp",
            case.state
        ))
        .unwrap();
        let doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view, "{} row{}", case.state, case.row);
        assert_eq!(doc.encode().unwrap(), bytes);
        let Kind::Locus(refs, samples) = &doc.objects[case.outer - 1].kind else {
            panic!();
        };
        assert_eq!(refs, case.refs);
        assert_eq!(*samples, 4);
        let (a, b, mode) = doc.line_points(case.target).unwrap();
        close63(a, case.live.0);
        close63(b, case.live.1);
        assert_eq!(mode, case.mode);
        let actual = doc.locus_strokes(case.outer, case.samples).unwrap();
        let expected: Vec<_> = case.expected.iter().flatten().collect();
        assert_eq!(actual.len(), expected.len());
        for ((a, b, mode), (x, y)) in actual.into_iter().zip(expected) {
            close63(a, *x);
            close63(b, *y);
            assert_eq!(mode, case.mode);
        }
    }
}

#[test]
fn scoped_color_bindings_are_evaluated_for_each_straight_family_member() {
    let (_, doc) = document63("chained_colorized");
    let cases: Vec<_> = straight63_cases()
        .into_iter()
        .filter(|c| c.state == "chained_colorized")
        .collect();
    for case in cases {
        let strokes = doc.locus_styled_strokes(case.outer, 4).unwrap();
        assert!(!strokes.is_empty());
        let colors: Vec<_> = strokes
            .iter()
            .map(|s| match s.style {
                VisualStyle::Stroke { color, .. } => color,
                _ => panic!(),
            })
            .collect();
        assert_ne!(colors.first(), colors.get(1));
        assert!(colors.iter().all(|c| *c
            != RgbColor {
                red: 0,
                green: 180,
                blue: 0
            }));
        if case.row == 1 {
            assert_eq!(
                colors.first(),
                Some(&RgbColor {
                    red: 0,
                    green: 0,
                    blue: 0
                })
            );
            assert_eq!(
                colors.last(),
                Some(&RgbColor {
                    red: 255,
                    green: 255,
                    blue: 255
                })
            );
        }
    }
}

#[test]
fn transformed_segment_hosts_follow_mirror_edits_undo_and_dependency_deletion() {
    let (bytes, mut doc) = document63("transformed_hosts");
    let case = straight63_cases()
        .into_iter()
        .find(|c| c.state == "transformed_hosts" && c.row == 0)
        .unwrap();
    let definition = doc.objects[case.host - 1]
        .kind
        .transform_definition()
        .unwrap();
    let super::super::transform_model::Transform::Reflect { mirror } = *definition.transform()
    else {
        panic!();
    };
    let Kind::Line(a, b) = doc.objects[mirror - 1].kind else {
        panic!();
    };
    doc.checkpoint();
    for id in [a, b] {
        let mut point = doc.position(id).unwrap();
        point.x += 10.;
        assert!(doc.move_point(id, point));
    }
    let edited = straight63_cases()
        .into_iter()
        .find(|c| c.state == "transformed_hosts_edited" && c.row == 0)
        .unwrap();
    let (x, y, mode) = doc.line_points(case.target).unwrap();
    close63(x, edited.live.0);
    close63(y, edited.live.1);
    assert_eq!(mode, case.mode);
    let reopened = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    assert_eq!(
        reopened.locus_strokes(case.outer, 3),
        doc.locus_strokes(case.outer, 3)
    );
    doc.undo();
    assert_eq!(doc.encode().unwrap(), bytes);
    doc.selected.insert(case.host);
    // This atlas also has independent later path observers. Until their native
    // references can be remapped, deletion must fail atomically.
    assert_eq!(
        doc.clear_selected().unwrap_err(),
        "Deleting an object before a surviving plot or function is not supported"
    );
    assert_eq!(doc.encode().unwrap(), bytes);
    assert!(!doc.objects.iter().any(|object| object.deleted));

    // Exercise the same dependency cascade through public authoring, without
    // unrelated observers that would require the unsupported remapping above.
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let b = doc.add(Kind::Free(Point { x: 50., y: 0. }));
    let c = doc.add(Kind::Free(Point { x: 0., y: 40. }));
    let base = doc.add(Kind::Segment(a, b));
    let mirror = doc.add(Kind::Line(a, c));
    let host = doc.add(Kind::ReflectedSegment(base, mirror));
    let driver = doc.add(Kind::PointOnSegment(host, 0.3));
    let target = doc.add(Kind::Segment(driver, c));
    let outer = doc.construct_locus(driver, target).unwrap();
    let authored = doc.encode().unwrap();
    doc.selected.insert(host);
    assert_eq!(
        doc.clear_selected().unwrap(),
        vec![host, driver, target, outer]
    );
    for id in [host, driver, target, outer] {
        assert!(doc.objects[id - 1].deleted);
    }
    assert!(
        !Document::from_bytes(&doc.encode().unwrap(), None)
            .unwrap()
            .partial_view
    );
    doc.undo();
    assert_eq!(doc.encode().unwrap(), authored);
}

#[test]
fn straight_family_color_provenance_matches_native_wrapper_order() {
    let (_, doc) = document63("colorized_source_transform");
    for case in straight63_cases()
        .into_iter()
        .filter(|c| c.state == "colorized_source_transform")
    {
        let strokes = doc.locus_styled_strokes(case.outer, 4).unwrap();
        let colors: Vec<_> = strokes
            .iter()
            .map(|s| match s.style {
                VisualStyle::Stroke { color, .. } => color,
                _ => panic!(),
            })
            .collect();
        if case.row == 0 {
            assert!(colors.iter().all(|color| *color
                == RgbColor {
                    red: 0,
                    green: 180,
                    blue: 0
                }));
        } else {
            assert_eq!(
                colors.first(),
                Some(&RgbColor {
                    red: 0,
                    green: 0,
                    blue: 0
                })
            );
            assert_eq!(
                colors.last(),
                Some(&RgbColor {
                    red: 255,
                    green: 255,
                    blue: 255
                })
            );
        }
    }
}

#[test]
fn straight_family_pen_edits_roundtrip_and_undo_without_changing_geometry() {
    let (_, mut doc) = document63("vector");
    let cases: Vec<_> = straight63_cases()
        .into_iter()
        .filter(|c| c.state == "vector")
        .collect();
    let original = doc.encode().unwrap();
    let geometry: Vec<_> = cases
        .iter()
        .map(|c| doc.locus_strokes(c.outer, 4).unwrap())
        .collect();
    doc.selected.insert(cases[0].outer);
    assert_eq!(doc.set_selected_line_width(LineWidth::Thick).unwrap(), 1);
    doc.selected.clear();
    doc.selected.insert(cases[1].outer);
    assert_eq!(
        doc.set_selected_line_pattern(LinePattern::Dashed).unwrap(),
        1
    );
    let encoded = doc.encode().unwrap();
    let reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    for (case, expected) in cases.iter().zip(geometry) {
        assert_eq!(reopened.locus_strokes(case.outer, 4), Some(expected));
        for stroke in reopened.locus_styled_strokes(case.outer, 4).unwrap() {
            let VisualStyle::Stroke { line, color } = stroke.style else {
                panic!();
            };
            assert_eq!(
                line.width,
                if case.row == 0 {
                    LineWidth::Thick
                } else {
                    LineWidth::Thin
                }
            );
            assert_eq!(
                line.pattern,
                if case.row == 1 {
                    LinePattern::Dashed
                } else {
                    LinePattern::Solid
                }
            );
            assert_eq!(
                color,
                RgbColor {
                    red: 0,
                    green: 180,
                    blue: 0
                }
            );
        }
    }
    doc.undo();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
}

#[test]
fn straight_sweeps_keep_wrapped_axis_tool_page_and_cycle_guards() {
    let (_, doc) = document63("vector");
    let case = straight63_cases()
        .into_iter()
        .find(|c| c.state == "vector" && c.row == 0)
        .unwrap();
    assert_eq!(locus_straight_mode(&Kind::GraphAxis(1, 2, 0)), None);
    assert_eq!(
        locus_straight_mode(&Kind::ToolInput(
            crate::document::EmbeddedToolInputKind::Segment
        )),
        None
    );
    for mode in 0..3 {
        let mut invalid = doc.clone();
        match mode {
            0 => invalid.objects[case.host - 1].page += 1,
            1 => invalid.objects[case.host - 1].deleted = true,
            _ => {
                invalid.objects[case.target - 1].kind =
                    Kind::VectorTranslatedSegment(case.target, 1, 2)
            }
        }
        assert!(
            super::super::kind::expected_locus_refs(&invalid.objects, case.driver, case.target)
                .is_none()
        );
    }
    let mut probe = Document::new();
    let a = probe.add(Kind::Free(Point { x: 0., y: 0. }));
    let b = probe.add(Kind::Free(Point { x: 50., y: 0. }));
    let axis = probe.add(Kind::GraphAxis(a, b, 0));
    let q = probe.add_named_parameter("q", 0.5, (10, 10)).unwrap();
    let wrapper = probe
        .add_colorized(
            axis,
            ColorBinding::Grayscale { value: q },
            0.,
            1.,
            ColorRangeBehavior::Clamp,
        )
        .unwrap();
    assert_eq!(locus_straight_source_mode(&probe.objects, wrapper), None);
}
