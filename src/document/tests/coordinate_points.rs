use super::*;
use crate::document::coordinate_points::{CoordinatePointDefinition, CoordinatePointFlavor};
use crate::document::transform_model::NumericExpr;
fn definition(
    frame: usize,
    value: usize,
    literal: f64,
    flavor: CoordinatePointFlavor,
) -> CoordinatePointDefinition {
    let l = NumericExpr::Literal {
        magnitude: literal,
        unit: NumericUnit::Scalar,
        scale: 1.,
    };
    let v = NumericExpr::ObjectValue {
        id: value,
        unit: NumericUnit::Scalar,
    };
    let (x, y) = match flavor {
        CoordinatePointFlavor::FixedX => (l, v),
        CoordinatePointFlavor::FixedY => (v, l),
    };
    CoordinatePointDefinition::new(frame, x, y, flavor).unwrap()
}
fn near(a: Point, b: Point) {
    assert!(
        (a.x - b.x).abs() < 1e-8 && (a.y - b.y).abs() < 1e-8,
        "{a:?} != {b:?}"
    );
}
#[test]
fn coordinate_points_author_reopen_live_frame_value_undo_and_delete() {
    let mut d = Document::new();
    let frame = d
        .add_default_coordinate_system(Point { x: 180., y: 180. })
        .unwrap();
    let q = d.add_named_parameter("q", 2., (10, 10)).unwrap();
    let a = d
        .add_coordinate_point(definition(frame, q, 1.5, CoordinatePointFlavor::FixedX))
        .unwrap();
    let b = d
        .add_coordinate_point(definition(frame, q, 1.5, CoordinatePointFlavor::FixedY))
        .unwrap();
    let unit = 37.79527559055118;
    near(
        d.position(a).unwrap(),
        Point {
            x: 180. + 1.5 * unit,
            y: 180. - 2. * unit,
        },
    );
    near(
        d.position(b).unwrap(),
        Point {
            x: 180. + 2. * unit,
            y: 180. - 1.5 * unit,
        },
    );
    d.verify_roundtrip().unwrap();
    let bytes = d.encode().unwrap();
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    assert!(!d.partial_view);
    assert_eq!(d.encode().unwrap(), bytes);
    assert_eq!(d.objects[a - 1].kind.dependencies(), vec![q, frame]);
    assert!(d.set_fixed_coordinate(a, f64::INFINITY).is_err());
    assert!(d.set_fixed_coordinate(q, 1.).is_err());
    d.set_fixed_coordinate(a, 2.5).unwrap();
    d.verify_roundtrip().unwrap();
    near(
        d.position(a).unwrap(),
        Point {
            x: 180. + 2.5 * unit,
            y: 180. - 2. * unit,
        },
    );
    d.undo();
    d.set_parameter_value(q, -1.).unwrap();
    near(
        d.position(a).unwrap(),
        Point {
            x: 180. + 1.5 * unit,
            y: 180. + unit,
        },
    );
    d.undo();
    near(
        d.position(a).unwrap(),
        Point {
            x: 180. + 1.5 * unit,
            y: 180. - 2. * unit,
        },
    );
    assert!(d.move_point(1, Point { x: 200., y: 190. }));
    near(
        d.position(b).unwrap(),
        Point {
            x: 200. + 2. * unit,
            y: 190. - 1.5 * unit,
        },
    );
    d.verify_roundtrip().unwrap();
    d.selected.insert(q);
    let removed = d.clear_selected().unwrap();
    assert!(removed.contains(&a) && removed.contains(&b));
    d.verify_roundtrip().unwrap();
    d.undo();
    assert!(!d.objects[a - 1].deleted);
    d.verify_roundtrip().unwrap();
}
#[test]
fn coordinate_points_scalar_iterations_reevaluate_and_preserve_omitted_values() {
    for frozen in [false, true] {
        let mut d = Document::new();
        let frame = d
            .add_default_coordinate_system(Point { x: 180., y: 180. })
            .unwrap();
        let q = d.add_named_parameter("q", 1., (10, 10)).unwrap();
        let next = d.add_parameter_calculation(q, "q+1", (10, 10)).unwrap();
        let xy = d
            .add_coordinate_point(definition(frame, next, 1.5, CoordinatePointFlavor::FixedX))
            .unwrap();
        let carried = if frozen { vec![xy] } else { vec![next, xy] };
        let parent = d
            .add_mapped_point_iteration_with_bindings(
                &[q],
                &[vec![next]],
                &carried,
                &[xy],
                IterationDepth::Fixed(3),
            )
            .unwrap();
        let binding = parent + 1;
        let values = d.point_iteration_images(binding).unwrap();
        assert_eq!(values.len(), 3);
        for (i, p) in values.iter().enumerate() {
            near(
                *p,
                Point {
                    x: 180. + 1.5 * 37.79527559055118,
                    y: 180. - if frozen { 2. } else { i as f64 + 3. } * 37.79527559055118,
                },
            );
        }
        d.verify_roundtrip().unwrap();
        let d = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
        assert_eq!(d.point_iteration_images(binding), Some(values));
    }
    for (bytes, frozen) in [
        (
            include_bytes!(
                "../../../tests/fixtures/coordinate_point52_iteration_live_native_saved.gsp"
            )
            .as_slice(),
            false,
        ),
        (
            include_bytes!(
                "../../../tests/fixtures/coordinate_point52_iteration_frozen_native_saved.gsp"
            )
            .as_slice(),
            true,
        ),
    ] {
        let d = Document::from_bytes(bytes, None).unwrap();
        assert!(!d.partial_view);
        assert_eq!(d.encode().unwrap(), bytes);
        d.verify_roundtrip().unwrap();
        let images = d.point_iteration_images(11).unwrap();
        assert_eq!(images.len(), 3);
        for (i, p) in images.iter().enumerate() {
            near(
                *p,
                Point {
                    x: 240. + 1.5 * 37.79527559055118,
                    y: 340. - if frozen { 2. } else { i as f64 + 3. } * 37.79527559055118,
                },
            );
        }
    }
}
#[test]
fn coordinate_points_strict_roles_and_payload_preserve_signed_literal() {
    let mut d = Document::new();
    let frame = d
        .add_default_coordinate_system(Point { x: 180., y: 180. })
        .unwrap();
    let q = d.add_named_parameter("q", 2., (10, 10)).unwrap();
    let def = definition(frame, q, -0., CoordinatePointFlavor::FixedX);
    assert_eq!(
        u64::from_le_bytes(def.native_payload()[4..].try_into().unwrap()),
        (-0f64).to_bits()
    );
    assert!(d
        .add_coordinate_point(definition(q, frame, 1., CoordinatePointFlavor::FixedX))
        .is_err());
    let mut h = [0u8; 28];
    h[6..10].copy_from_slice(&[1, 0, 56, 0]);
    h[..2].copy_from_slice(&109u16.to_le_bytes());
    assert!(crate::document::coordinate_points::decode(
        &d.objects,
        0,
        q + 1,
        &h,
        &[q, frame],
        Some(&def.native_payload())
    )
    .is_some());
    let mut payload = def.native_payload();
    payload[..4].copy_from_slice(&110u32.to_le_bytes());
    assert!(crate::document::coordinate_points::decode(
        &d.objects,
        0,
        q + 1,
        &h,
        &[q, frame],
        Some(&payload)
    )
    .is_none());
    payload[4..].copy_from_slice(&f64::NAN.to_le_bytes());
    assert!(crate::document::coordinate_points::decode(
        &d.objects,
        0,
        q + 1,
        &h,
        &[q, frame],
        Some(&payload)
    )
    .is_none());
}
#[test]
fn coordinate_points_native_controls_match_independent_world_values_exactly() {
    for (bytes, q) in [
        (
            include_bytes!(
                "../../../tests/fixtures/referenced_roles52_coordinate_baseline_native_saved.gsp"
            )
            .as_slice(),
            2.,
        ),
        (
            include_bytes!(
                "../../../tests/fixtures/referenced_roles52_coordinate_edited_native_saved.gsp"
            )
            .as_slice(),
            -1.,
        ),
    ] {
        let d = Document::from_bytes(bytes, None).unwrap();
        assert!(!d.partial_view);
        assert_eq!(d.encode().unwrap(), bytes);
        for (id, x, y) in [(8, 0., q), (9, 1.5, q), (10, q, 0.), (11, q, 1.5)] {
            near(
                d.position(id).unwrap(),
                Point {
                    x: 240. + x * 37.79527559055118,
                    y: 260. - y * 37.79527559055118,
                },
            );
        }
        d.verify_roundtrip().unwrap();
    }
}

#[test]
fn coordinate_points_existing_multi_page_literal_edits_keep_other_records() {
    let mut d = Document::new();
    let frame = d
        .add_default_coordinate_system(Point { x: 180., y: 180. })
        .unwrap();
    let q = d.add_named_parameter("q", 2., (10, 10)).unwrap();
    let id = d
        .add_coordinate_point(definition(frame, q, 1.5, CoordinatePointFlavor::FixedY))
        .unwrap();
    let mut d = Document::from_bytes(&d.encode().unwrap(), None).unwrap();
    d.add_blank_page().unwrap();
    let bytes = d.encode().unwrap();
    let mut d = Document::from_bytes(&bytes, None).unwrap();
    d.set_fixed_coordinate(id, -2.5).unwrap();
    d.verify_roundtrip().unwrap();
    let edited = d.encode().unwrap();
    let reopened = Document::from_bytes(&edited, None).unwrap();
    let Kind::CoordinatePoint(def) = &reopened.objects[id - 1].kind else {
        panic!()
    };
    assert_eq!(def.fixed_coordinate(), -2.5);
    let old = records(&bytes).unwrap();
    let new = records(&edited).unwrap();
    assert_eq!(old.len(), new.len());
    let mut changes = 0;
    for (a, b) in old.iter().zip(new.iter()) {
        assert_eq!(a.kind, b.kind);
        if bytes[a.data.clone()] != edited[b.data.clone()] {
            assert_eq!(a.kind, 0x07d3);
            changes += 1;
        }
    }
    assert_eq!(changes, 1);
    d.undo();
    d.verify_roundtrip().unwrap();
}

#[test]
fn coordinate_points_public_api_native_writers_keep_nonuniform_basis() {
    for (bytes, value, sx, sy) in [
        (
            include_bytes!(
                "../../../tests/fixtures/coordinate_point52_api_baseline_native_saved.gsp"
            )
            .as_slice(),
            2.,
            37.79527559055118,
            37.79527559055118,
        ),
        (
            include_bytes!(
                "../../../tests/fixtures/coordinate_point52_api_edited_native_saved.gsp"
            )
            .as_slice(),
            -1.,
            37.79527559055118,
            37.79527559055118,
        ),
        (
            include_bytes!(
                "../../../tests/fixtures/coordinate_point52_api_unequal_native_saved.gsp"
            )
            .as_slice(),
            2.,
            20.,
            30.,
        ),
    ] {
        let d = Document::from_bytes(bytes, None).unwrap();
        assert!(!d.partial_view);
        assert_eq!(d.encode().unwrap(), bytes);
        d.verify_roundtrip().unwrap();
        for (id, x, y) in [
            (8, 0., value),
            (9, 1.5, value),
            (10, value, 0.),
            (11, value, 1.5),
        ] {
            near(
                d.position(id).unwrap(),
                Point {
                    x: 180. + x * sx,
                    y: 180. - y * sy,
                },
            );
        }
    }
}
