use super::*;
use crate::document::transform_model::{NumericExpr, Transform};

fn near(actual: Point, expected: Point) {
    assert!(
        (actual.x - expected.x).abs() < 1e-9 && (actual.y - expected.y).abs() < 1e-9,
        "{actual:?} != {expected:?}"
    );
}

#[test]
fn inherited_scalar_dilations_native_profiles_and_canonical_writer_agree() {
    for (bytes, expected) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/inherited_numeric_transform56_baseline_native_saved.gsp"
            )
            .as_slice(),
            Point { x: 225., y: 200. },
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/inherited_numeric_transform56_canonical_native_saved.gsp"
            )
            .as_slice(),
            Point { x: 225., y: 200. },
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/inherited_numeric_transform56_edited_native_saved.gsp"
            )
            .as_slice(),
            Point { x: 262.5, y: 187.5 },
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/inherited_numeric_transform56_frozen_native_saved.gsp"
            )
            .as_slice(),
            Point { x: 225., y: 195. },
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/inherited_numeric_transform56_zero_native_saved.gsp"
            )
            .as_slice(),
            Point { x: 180., y: 180. },
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/inherited_numeric_transform56_negative_native_saved.gsp"
            )
            .as_slice(),
            Point { x: 135., y: 160. },
        ),
    ] {
        let d = Document::from_bytes(bytes, None).unwrap();
        assert!(!d.partial_view);
        assert_eq!(d.encode().unwrap(), bytes);
        d.verify_roundtrip().unwrap();
        near(d.position(18).unwrap(), expected);
        let definition = d.objects[17].kind.transform_definition().unwrap();
        assert_eq!(definition.element(), ElementType::Point);
        assert_eq!(
            definition.wire_flavor(),
            NativeTransformFlavor::NumericDilation
        );
        assert_eq!(
            definition.transform(),
            &Transform::Dilate {
                center: 1,
                ratio: NumericExpr::ObjectValue {
                    id: 17,
                    unit: NumericUnit::Scalar
                }
            }
        );
        assert_eq!(d.objects[17].kind.dependencies(), vec![15, 1, 17]);
        assert_eq!(
            d.objects[17].visual_style,
            Some(VisualStyle::Point {
                shape: PointStyle::Medium,
                color: RgbColor {
                    red: 255,
                    green: 0,
                    blue: 0
                }
            })
        );
    }
}

#[test]
fn inherited_scalar_dilation_live_edits_preserve_identity_undo_and_deletion() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/inherited_numeric_transform56_baseline_native_saved.gsp"
    );
    let mut d = Document::from_bytes(bytes, None).unwrap();
    d.set_parameter_value(17, -0.5).unwrap();
    near(d.position(18).unwrap(), Point { x: 135., y: 160. });
    d.set_parameter_value(17, 0.).unwrap();
    near(d.position(18).unwrap(), Point { x: 180., y: 180. });
    d.undo();
    near(d.position(18).unwrap(), Point { x: 135., y: 160. });
    d.set_parameter_value(17, 0.5).unwrap();
    assert!(d.move_point(13, Point { x: 90., y: 70. }));
    let source = d.position(15).unwrap();
    near(
        d.position(18).unwrap(),
        Point {
            x: (source.x + 180.) * 0.5,
            y: (source.y + 180.) * 0.5,
        },
    );
    let encoded = d.encode().unwrap();
    let mut reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    reopened.verify_roundtrip().unwrap();
    let at = reopened.objects[17].header_offset.unwrap();
    assert_eq!(&encoded[at + 6..at + 10], &[1, 0, 89, 0]);
    near(reopened.position(18).unwrap(), d.position(18).unwrap());
    reopened.selected.insert(11);
    let removed = reopened.clear_selected().unwrap();
    assert!(removed.contains(&15) && removed.contains(&18) && removed.contains(&19));
    reopened.verify_roundtrip().unwrap();
    reopened.undo();
    near(reopened.position(18).unwrap(), d.position(18).unwrap());
    reopened.verify_roundtrip().unwrap();
}

#[test]
fn inherited_scalar_dilation_accepts_point_sources_and_rejects_invalid_operands() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/inherited_numeric_transform56_baseline_native_saved.gsp"
    );
    let d = Document::from_bytes(bytes, None).unwrap();
    let rs = records(bytes).unwrap();
    let at = d.objects[17].header_offset.unwrap();
    let index = rs.iter().position(|r| r.data.start == at).unwrap();
    let refs = rs[index + 1].data.clone();
    for (offset, id) in [(8, 17u32), (12, 1), (4, 18)] {
        let mut wrong = bytes.to_vec();
        wrong[refs.start + offset..refs.start + offset + 4].copy_from_slice(&id.to_le_bytes());
        assert_eq!(
            Document::from_bytes(&wrong, None).unwrap().objects[17].kind,
            Kind::Unsupported(33)
        );
    }
    let mut wrong = bytes.to_vec();
    let source = d.objects[14].header_offset.unwrap();
    wrong[source + 8..source + 10].copy_from_slice(&4u16.to_le_bytes());
    let decoded = Document::from_bytes(&wrong, None).unwrap();
    assert!(crate::document::kind::is_point_kind(
        &decoded.objects[14].kind
    ));
    assert!(decoded.objects[17].kind.is_point());
    assert!(decoded.position(18).is_some());
    let mut wrong = bytes.to_vec();
    wrong[at + 6..at + 8].copy_from_slice(&2u16.to_le_bytes());
    assert_eq!(
        Document::from_bytes(&wrong, None).unwrap().objects[17].kind,
        Kind::Unsupported(33)
    );
    for payload in [&[][..], &[33, 0, 0, 0][..], &[33, 0, 0, 0, 0][..]] {
        let mut record = Vec::new();
        put_record(&mut record, 0x07d3, payload);
        let mut wrong = bytes.to_vec();
        wrong.splice(rs[index + 2].start..rs[index + 2].start, record);
        assert_eq!(
            Document::from_bytes(&wrong, None).unwrap().objects[17].kind,
            Kind::Unsupported(33)
        );
    }
}
