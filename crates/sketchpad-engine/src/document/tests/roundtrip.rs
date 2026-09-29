use super::*;

#[test]
fn fixtures_preserve_bytes() {
    for name in [
        "point",
        "point_hidden",
        "points",
        "segment",
        "circle",
        "circle_center_radius",
        "middle_point",
        "line",
        "line_intersection",
        "line_circle_intersection",
        "circle_circle_intersection",
        "bisector",
        "three_point_arc",
        "ray",
        "parallel",
        "perp",
        "example_tools",
        "wine_moved_demo",
        "wine_moved_length",
        "reflection",
        "wine_moved_reflection",
        "wine_moved_reflected_circle",
        "scale",
        "wine_moved_dilation",
        "rotation",
        "wine_moved_rotation",
        "wine_deleted_source",
        "wine_cut_segment",
        "wine_pasted_segment",
        "wine_segment_length",
        "wine_point_distance",
        "wine_moved_distance",
        "wine_circle_circumference",
        "wine_moved_circumference",
        "wine_circle_radius",
        "wine_moved_radius",
        "wine_point_angle",
        "wine_moved_angle",
        "wine_moved_generated_angle",
        "wine_circle_area",
        "wine_circle_area_radius",
        "wine_moved_area",
        "wine_moved_generated_area",
        "wine_arc_angle",
        "wine_moved_arc_angle",
        "wine_arc_length",
        "wine_moved_arc_length",
        "wine_arc_measurements_on_circle",
        "wine_segment_slope",
        "wine_point_ratio",
        "wine_moved_ratio",
        "wine_opposite_ratio",
        "wine_offline_ratio",
        "wine_polygon_perimeter",
        "wine_moved_generated_polygon_perimeter",
        "action_hide_show",
        "action_hide_show_clicked",
        "action_hide_show_reclicked",
        "action_generated_english",
        "action_generated_english_clicked",
        "action_port_generated",
        "action_port_clicked",
        "action_port_wine_clicked",
    ] {
        let p = format!("../../tests/fixtures/{name}.gsp");
        let b = fs::read(p).unwrap();
        let d = Document::from_bytes(&b, None).unwrap();
        assert_eq!(d.encode().unwrap(), b);
    }

    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../gsp-rs/tests/fixtures/gsp/两个三角形标记全等.gsp"
    );
    let bytes = fs::read(path).unwrap();
    let document = Document::from_bytes(&bytes, None).unwrap();
    assert!(matches!(
        document.objects[16].kind,
        Kind::SegmentMarker {
            host: 9,
            count: 1,
            ..
        }
    ));
    assert!(matches!(
        document.objects[20].kind,
        Kind::SegmentMarker {
            host: 12,
            count: 3,
            ..
        }
    ));
    assert_eq!(document.segment_marker_strokes(17).unwrap().len(), 1);
    assert_eq!(document.segment_marker_strokes(21).unwrap().len(), 3);
    assert_eq!(document.encode().unwrap(), bytes);

    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../gsp-rs/tests/Samples/个人专栏/唐家军作品/课程3笔记.gsp"
    );
    let bytes = fs::read(path).unwrap();
    let document = Document::from_bytes(&bytes, None).unwrap();
    assert!(matches!(
        document.objects[299].kind,
        Kind::SegmentArrow {
            host: 298,
            forward: true,
            ..
        }
    ));
    assert_eq!(document.segment_marker_strokes(300).unwrap().len(), 2);
    assert_eq!(document.encode().unwrap(), bytes);

    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../gsp-rs/tests/Samples/个人专栏/唐家军作品/课时9笔记.gsp"
    );
    let bytes = fs::read(path).unwrap();
    let document = Document::from_bytes(&bytes, None).unwrap();
    assert!(matches!(
        document.objects[608].kind,
        Kind::SegmentArrow {
            host: 599,
            forward: false,
            ..
        }
    ));
    assert_eq!(document.encode().unwrap(), bytes);
}
#[test]
fn new_roundtrip_and_dependencies() {
    let mut d = Document::new();
    let a = d.add(Kind::Free(Point { x: 10., y: 20. }));
    let b = d.add(Kind::Free(Point { x: 30., y: 40. }));
    let segment = d.add(Kind::Segment(a, b));
    let m = d.add(Kind::Midpoint(a, b));
    let marker = d.add(Kind::SegmentMarker {
        host: segment,
        parameter: 0.5,
        count: 2,
    });
    let arrow = d.add(Kind::SegmentArrow {
        host: segment,
        parameter: 0.25,
        forward: false,
    });
    assert_eq!(d.position(m), Some(Point { x: 20., y: 30. }));
    let strokes = d.segment_marker_strokes(marker).unwrap();
    assert_eq!(strokes.len(), 2);
    assert!(
        strokes
            .iter()
            .all(|stroke| (stroke[0].distance(stroke[1]) - 14.).abs() < 1e-12)
    );
    assert_eq!(d.segment_marker_strokes(arrow).unwrap().len(), 2);
    let reopened = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
    assert_eq!(reopened.position(m), d.position(m));
    assert_eq!(reopened.segment_marker_strokes(marker), Some(strokes));
    assert_eq!(
        reopened.segment_marker_strokes(arrow),
        d.segment_marker_strokes(arrow)
    );
}
