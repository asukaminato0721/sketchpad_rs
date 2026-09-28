use super::*;
use crate::style_records::{RgbColor, VisualStyle};

fn disks(doc: &Document, id: usize) -> Vec<(Point, f64)> {
    doc.circle_interior_iteration_images(id).unwrap()
}
fn assert_disks(doc: &Document, id: usize, expected: &[(f64, f64)]) {
    let actual = disks(doc, id);
    assert_eq!(actual.len(), expected.len());
    for ((center, radius), &(x, r)) in actual.iter().zip(expected) {
        assert!((center.x - x).abs() < 1e-9 && (center.y - 260.).abs() < 1e-9);
        assert!((radius - r).abs() < 1e-9);
    }
    let polygons = doc.polygon_iteration_images(id).unwrap();
    assert_eq!(polygons.len(), actual.len());
    for ((points, paths), (center, radius)) in polygons.iter().zip(actual) {
        assert_eq!(*paths, 1.);
        assert!(points.len() >= 32);
        for point in points {
            assert!(((point.x - center.x).hypot(point.y - center.y) - radius).abs() < 1e-9);
        }
    }
}

#[test]
fn native_disk_bindings_follow_scoped_geometry_frozen_hosts_and_undefined_final() {
    for (state, expected) in [
        ("live", vec![(240., 30.), (340., 40.), (440., 50.)]),
        ("edited", vec![(260., 50.), (380., 60.), (500., 70.)]),
        ("frozen_circle", vec![(140., 20.); 3]),
        ("final_only", vec![(440., 50.)]),
        ("undefined_final", vec![(240., 40.), (340., 20.)]),
        ("undefined_final_only", vec![]),
        ("recovered", vec![(240., 60.), (340., 40.), (440., 20.)]),
    ] {
        let bytes = fs::read(format!(
            "../../tests/fixtures/circle_interior_iteration65_{state}_native_saved.gsp"
        ))
        .unwrap();
        let doc = Document::from_bytes(&bytes, None).unwrap();
        assert!(!doc.partial_view, "{state}");
        assert!(matches!(
            doc.objects[13].kind,
            Kind::MappedPointIteration { .. }
        ));
        assert_eq!(doc.objects[14].kind, Kind::IterationBinding(12, 14));
        assert!(!doc.objects[10].visible && !doc.objects[11].visible);
        assert_eq!(
            doc.objects[14].visual_style,
            Some(VisualStyle::Polygon {
                color: RgbColor {
                    red: 0,
                    green: 128,
                    blue: 0
                },
                opacity: 65535,
            })
        );
        assert_disks(&doc, 15, &expected);
        assert_eq!(doc.encode().unwrap(), bytes, "{state}");
        doc.verify_roundtrip().unwrap();
    }
}

fn authored(frozen: bool) -> Document {
    let mut doc = Document::new();
    for point in [
        Point { x: 140., y: 260. },
        Point { x: 0., y: 0. },
        Point { x: 100., y: 0. },
    ] {
        doc.add(Kind::Free(point));
    }
    doc.add(Kind::VectorTranslatedPoint(1, 2, 3));
    doc.add_named_parameter("q", 0.5, (10, 10)).unwrap();
    doc.add_parameter_calculation(5, "q+.25", (10, 10)).unwrap();
    doc.add(Kind::Free(Point { x: 0., y: 0. }));
    doc.add(Kind::Free(Point { x: 40., y: 0. }));
    doc.add_calculated_ratio_dilation(8, 7, 5).unwrap();
    doc.add(Kind::VectorTranslatedPoint(1, 7, 9));
    doc.add(Kind::Circle(1, 10));
    doc.add_circle_interior(11).unwrap();
    doc.objects[11].visual_style = Some(VisualStyle::Polygon {
        color: RgbColor {
            red: 0,
            green: 128,
            blue: 0,
        },
        opacity: 65535,
    });
    doc.add_named_parameter("n", 3., (10, 10)).unwrap();
    let carried = if frozen {
        vec![4, 6, 9, 10, 12]
    } else {
        vec![4, 6, 9, 10, 11, 12]
    };
    assert_eq!(
        doc.add_mapped_point_iteration_with_bindings(
            &[1, 5],
            &[vec![4, 6]],
            &carried,
            &[12],
            IterationDepth::Value(13)
        )
        .unwrap(),
        14
    );
    for object in &mut doc.objects {
        object.visible = object.id == 15;
    }
    doc
}

#[test]
fn public_disk_iteration_edits_styles_visibility_depth_delete_and_undo_reopen() {
    let mut doc = authored(false);
    assert_disks(&doc, 15, &[(240., 30.), (340., 40.), (440., 50.)]);
    let first = doc.encode().unwrap();
    doc.verify_roundtrip().unwrap();
    assert_disks(
        &Document::from_bytes(&first, None).unwrap(),
        15,
        &[(240., 30.), (340., 40.), (440., 50.)],
    );
    doc.selected.insert(15);
    assert_eq!(
        doc.set_selected_color(RgbColor {
            red: 0,
            green: 0,
            blue: 255
        })
        .unwrap(),
        1
    );
    let blue = doc.encode().unwrap();
    assert_eq!(
        Document::from_bytes(&blue, None).unwrap().objects[14].visual_style,
        Some(VisualStyle::Polygon {
            color: RgbColor {
                red: 0,
                green: 0,
                blue: 255
            },
            opacity: 65535
        })
    );
    doc.undo();
    assert_eq!(doc.encode().unwrap(), first);
    doc.set_selected_visibility(false);
    assert!(!doc.is_displayed(15));
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert!(doc.is_displayed(15));
    doc.set_parameter_value(5, 1.).unwrap();
    assert_disks(&doc, 15, &[(240., 50.), (340., 60.), (440., 70.)]);
    doc.undo();
    doc.set_parameter_value(13, 2.).unwrap();
    assert_disks(&doc, 15, &[(240., 30.), (340., 40.)]);
    doc.undo();
    doc.selected.clear();
    doc.selected.insert(11);
    let removed = doc.clear_selected().unwrap();
    assert!(removed.contains(&12) && removed.contains(&14) && removed.contains(&15));
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_disks(&doc, 15, &[(240., 30.), (340., 40.), (440., 50.)]);
    assert_eq!(doc.encode().unwrap(), first);
    let mut frozen = authored(true);
    assert_disks(&frozen, 15, &[(140., 20.); 3]);
    frozen.set_parameter_value(5, 1.).unwrap();
    assert_disks(&frozen, 15, &[(140., 40.); 3]);
    frozen.verify_roundtrip().unwrap();
}
