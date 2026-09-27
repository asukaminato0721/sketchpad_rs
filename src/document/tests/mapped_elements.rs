use super::*;
use crate::document::mapped_element::{MappedElementDefinition, decode_mapped_element};
use crate::document::transform_model::{
    DisplacementExpr, NativeTransformFlavor, NumericExpr, Transform, TransformDefinition,
};

fn near(actual: Point, expected: Point) {
    assert!(
        actual.distance(expected) < 1e-9,
        "{actual:?} != {expected:?}"
    );
}
fn base() -> Document {
    let mut d = Document::new();
    for (x, y) in [(180., 180.), (220., 180.)] {
        d.add(Kind::Free(Point { x, y }));
    }
    d.add(Kind::Circle(1, 2));
    d.add(Kind::PointOnCircle(3, 1., 0.));
    for (x, y) in [(240., 200.), (180., 240.), (180., 140.)] {
        d.add(Kind::Free(Point { x, y }));
    }
    d.add(Kind::Transformed(
        TransformDefinition::new(
            7,
            ElementType::Point,
            Transform::Dilate {
                center: 1,
                ratio: NumericExpr::DistanceRatio {
                    origin: 1,
                    base: 7,
                    target: 4,
                },
            },
            NativeTransformFlavor::PointRatioDilation,
        )
        .unwrap(),
    ));
    d.add_named_parameter("q", 0.5, (10, 10)).unwrap();
    d.add(Kind::Transformed(
        TransformDefinition::new(
            4,
            ElementType::Point,
            Transform::Dilate {
                center: 8,
                ratio: NumericExpr::ObjectValue {
                    id: 9,
                    unit: NumericUnit::Scalar,
                },
            },
            NativeTransformFlavor::NumericDilation,
        )
        .unwrap(),
    ));
    d
}

#[test]
fn mapped_element_replaces_derived_seed_and_preserves_omitted_carried_boundaries() {
    for frozen in [false, true] {
        let mut d = base();
        let seed_kind = d.objects[3].kind.clone();
        let application = d
            .add_mapped_element(10, 4, 5, if frozen { &[10] } else { &[8, 10] })
            .unwrap();
        near(
            d.position(application).unwrap(),
            Point {
                x: 210.,
                y: if frozen { 190. } else { 200. },
            },
        );
        near(d.position(10).unwrap(), Point { x: 200., y: 180. });
        assert_eq!(d.objects[3].kind, seed_kind);
        let baseline = d.encode().unwrap();
        d = Document::from_bytes(&baseline, None).unwrap();
        d.checkpoint();
        assert!(d.move_point(5, Point { x: 260., y: 180. }));
        near(d.position(application).unwrap(), Point { x: 220., y: 180. });
        d.verify_roundtrip().unwrap();
        d.undo();
        assert_eq!(d.encode().unwrap(), baseline);
        near(
            d.position(application).unwrap(),
            Point {
                x: 210.,
                y: if frozen { 190. } else { 200. },
            },
        );
    }
}

#[test]
fn retained_native_mapping_observers_prove_substitution_and_frozen_values() {
    for (bytes, frozen) in [
        (
            include_bytes!(
                "../../../tests/fixtures/mapped_construction49_mapped_observers_native_verified.gsp"
            )
            .as_slice(),
            false,
        ),
        (
            include_bytes!(
                "../../../tests/fixtures/mapped_construction49_frozen_observers_native_verified.gsp"
            )
            .as_slice(),
            true,
        ),
        (
            include_bytes!(
                "../../../tests/fixtures/mapped_construction49_mapped_marked_native_verified.gsp"
            )
            .as_slice(),
            false,
        ),
        (
            include_bytes!(
                "../../../tests/fixtures/mapped_construction49_frozen_marked_native_verified.gsp"
            )
            .as_slice(),
            true,
        ),
    ] {
        let d = Document::from_bytes(bytes, None).unwrap();
        assert!(matches!(d.objects[10].kind, Kind::MappedElement(_)));
        near(
            d.position(11).unwrap(),
            Point {
                x: 210.,
                y: if frozen { 190. } else { 200. },
            },
        );
        near(d.position(12).unwrap(), Point { x: 180., y: 240. });
        let (a, b, _) = d.line_points(13).unwrap();
        near(a, Point { x: 180., y: 180. });
        near(b, d.position(11).unwrap());
        assert_eq!(d.encode().unwrap().as_slice(), bytes);
        d.verify_roundtrip().unwrap();
    }
}

#[test]
fn mapped_element_scoped_context_preserves_outer_iteration_substitutions() {
    let mut d = base();
    let application = d.add_mapped_element(10, 4, 5, &[8, 10]).unwrap();
    let target = d.add(Kind::Transformed(
        TransformDefinition::new(
            5,
            ElementType::Point,
            Transform::Translate(DisplacementExpr::Vector { start: 6, end: 7 }),
            NativeTransformFlavor::Vector,
        )
        .unwrap(),
    ));
    let iteration = d
        .add_mapped_point_iteration_with_bindings(
            &[5],
            &[vec![target]],
            &[target, application],
            &[application],
            IterationDepth::Fixed(2),
        )
        .unwrap();
    let binding = iteration + 1;
    let images = d.mapped_point_binding_images(binding).unwrap();
    assert_eq!(images.len(), 2);
    near(images[0], Point { x: 210., y: 100. });
    near(images[1], Point { x: 210., y: 0. });
    near(d.position(application).unwrap(), Point { x: 210., y: 200. });
    d.verify_roundtrip().unwrap();
}

#[test]
fn mapped_element_decoder_guards_roles_headers_and_packet_end_without_changing_bytes() {
    let d = base();
    let mut header = [0u8; 28];
    header[..2].copy_from_slice(&101u16.to_le_bytes());
    header[6..10].copy_from_slice(&[1, 0, 89, 0]);
    let good = [10, 5, 4, 8, 10];
    assert!(decode_mapped_element(&d.objects, 0, 11, &header, &good, None).is_some());
    for refs in [
        vec![10, 9, 4, 8, 10],
        vec![10, 5, 9, 8, 10],
        vec![10, 5, 4, 8, 8],
        vec![10, 5, 4, 8, 8, 10],
        vec![10, 11, 4, 8, 10],
        vec![10, 5, 4, 0, 10],
    ] {
        assert!(
            decode_mapped_element(&d.objects, 0, 11, &header, &refs, None).is_none(),
            "{refs:?}"
        );
    }
    assert!(decode_mapped_element(&d.objects, 0, 11, &header, &good, Some(&[0; 12])).is_none());
    header[8] = 4;
    assert!(decode_mapped_element(&d.objects, 0, 11, &header, &good, None).is_none());
    assert!(MappedElementDefinition::new(10, ElementType::Circle, 4, 5, vec![8, 10]).is_err());
    let bytes = include_bytes!(
        "../../../tests/fixtures/mapped_construction49_mapped_observers_native_verified.gsp"
    );
    let original = Document::from_bytes(bytes, None).unwrap();
    let at = original.objects[10].header_offset.unwrap();
    let mut mutated = bytes.to_vec();
    mutated[at + 8] = 4;
    let reopened = Document::from_bytes(&mutated, None).unwrap();
    assert!(matches!(reopened.objects[10].kind, Kind::Unsupported(101)));
    assert_eq!(reopened.encode().unwrap(), mutated);
}

#[test]
fn mapped_element_saved_references_remap_after_unrelated_deletion_and_cascade_with_sources() {
    let mut d = base();
    let application = d.add_mapped_element(10, 4, 5, &[8, 10]).unwrap();
    let observer = d.add(Kind::Segment(1, application));
    let baseline = d.encode().unwrap();
    d = Document::from_bytes(&baseline, None).unwrap();
    d.selected.insert(6);
    let deleted = d.clear_selected().unwrap();
    assert_eq!(deleted, vec![6]);
    let bytes = d.encode().unwrap();
    let mut reopened = Document::from_bytes(&bytes, None).unwrap();
    let Kind::MappedElement(def) = &reopened.objects[9].kind else {
        panic!("mapped source was lost")
    };
    assert_eq!(def.native_references(), vec![9, 5, 4, 7, 9]);
    near(reopened.position(10).unwrap(), Point { x: 210., y: 200. });
    near(
        reopened.line_points(11).unwrap().1,
        Point { x: 210., y: 200. },
    );
    reopened.verify_roundtrip().unwrap();
    reopened.selected.insert(8);
    let deleted = reopened.clear_selected().unwrap();
    assert!(deleted.contains(&9) && deleted.contains(&10) && deleted.contains(&11));
    reopened.verify_roundtrip().unwrap();
    reopened.undo();
    assert_eq!(reopened.encode().unwrap(), bytes);
    d.undo();
    assert_eq!(d.encode().unwrap(), baseline);
    assert_eq!(observer, 12);
}

#[test]
fn mapped_element_shared_scalar_inputs_are_live_and_undefined_output_recovers() {
    let mut d = base();
    let application = d.add_mapped_element(10, 4, 5, &[8, 10]).unwrap();
    let original = d.objects[3].kind.clone();
    d.set_parameter_value(9, 1.).unwrap();
    near(d.position(application).unwrap(), Point { x: 240., y: 200. });
    d.set_parameter_value(9, 0.).unwrap();
    near(d.position(application).unwrap(), Point { x: 180., y: 200. });
    let baseline = d.encode().unwrap();
    d = Document::from_bytes(&baseline, None).unwrap();
    d.checkpoint();
    assert!(d.move_point(7, Point { x: 180., y: 180. }));
    assert!(d.position(application).is_none());
    d.verify_roundtrip().unwrap();
    d.undo();
    assert_eq!(d.encode().unwrap(), baseline);
    assert_eq!(d.objects[3].kind, original);
    near(d.position(application).unwrap(), Point { x: 180., y: 200. });
}

#[test]
fn original_native_derived_point_applications_reconstruct_the_stored_carried_environment() {
    let bytes =
        include_bytes!("../../../tests/fixtures/transformed_arc48_cartesian_native_verified.gsp");
    let d = Document::from_bytes(bytes, None).unwrap();
    let origin = d.position(1).unwrap();
    let base = d.position(9).unwrap();
    let (dx, dy) = (base.x - origin.x, base.y - origin.y);
    let scale = d.unitless_numeric_value(14).unwrap();
    for (id, replacement) in [(23, 22), (26, 25)] {
        let Kind::MappedElement(def) = &d.objects[id - 1].kind else {
            panic!("original{id} not recovered")
        };
        assert_eq!(def.native_references(), vec![15, replacement, 10, 11, 15]);
        let p = d.position(replacement).unwrap();
        let ratio = ((p.x - origin.x) * dx + (p.y - origin.y) * dy) / (dx * dx + dy * dy);
        let center = Point {
            x: origin.x + dx * ratio,
            y: origin.y + dy * ratio,
        };
        near(
            d.position(id).unwrap(),
            Point {
                x: center.x + (p.x - center.x) * scale,
                y: center.y + (p.y - center.y) * scale,
            },
        );
    }
    assert_eq!(d.encode().unwrap().as_slice(), bytes);
    d.verify_roundtrip().unwrap();
}

#[test]
fn public_mapping_api_reconstructs_native_verified_authoring_groups() {
    for (state, bytes) in [
        (
            "mapped",
            include_bytes!(
                "../../../tests/fixtures/mapped_elements49_api_mapped_native_verified.gsp"
            )
            .as_slice(),
        ),
        (
            "frozen",
            include_bytes!(
                "../../../tests/fixtures/mapped_elements49_api_frozen_native_verified.gsp"
            )
            .as_slice(),
        ),
        (
            "edited",
            include_bytes!(
                "../../../tests/fixtures/mapped_elements49_api_edited_native_verified.gsp"
            )
            .as_slice(),
        ),
    ] {
        let mut d = base();
        let application = d
            .add_mapped_element(10, 4, 5, if state == "frozen" { &[10] } else { &[8, 10] })
            .unwrap();
        let observer = d.add(Kind::Segment(1, application));
        if state == "edited" {
            assert!(d.move_point(5, Point { x: 260., y: 180. }));
        }
        for o in &mut d.objects {
            o.visible = [1, application, observer].contains(&o.id);
        }
        assert_eq!(d.encode().unwrap().as_slice(), bytes, "{state}");
        d.verify_roundtrip().unwrap();
        let expected = match state {
            "frozen" => Point { x: 210., y: 190. },
            "edited" => Point { x: 220., y: 180. },
            _ => Point { x: 210., y: 200. },
        };
        near(d.position(application).unwrap(), expected);
        let reopened = Document::from_bytes(bytes, None).unwrap();
        near(reopened.position(application).unwrap(), expected);
    }
}
