use super::*;
use crate::document::transform_model::{DisplacementExpr, Transform};
fn near(a: Point, b: Point) {
    assert!(
        (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9,
        "{a:?} != {b:?}"
    );
}
fn expected_vector(d: &Document, source: usize, start: usize, end: usize) -> Point {
    let p = d.position(source).unwrap();
    let a = d.position(start).unwrap();
    let b = d.position(end).unwrap();
    Point {
        x: p.x + b.x - a.x,
        y: p.y + b.y - a.y,
    }
}
#[test]
fn inherited_point_vectors_native_alias_chain_and_canonical_writer_agree() {
    for (bytes, red, green) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/inherited_point_transform53_baseline_native_saved.gsp"
            )
            .as_slice(),
            Point { x: 270., y: 220. },
            Point { x: 330., y: 240. },
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/inherited_point_transform53_edited_native_saved.gsp"
            )
            .as_slice(),
            Point { x: 290., y: 190. },
            Point { x: 360., y: 200. },
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/inherited_point_transform53_frozen_native_saved.gsp"
            )
            .as_slice(),
            Point { x: 270., y: 210. },
            Point { x: 330., y: 230. },
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/inherited_point_transform53_canonical_native_saved.gsp"
            )
            .as_slice(),
            Point { x: 270., y: 220. },
            Point { x: 330., y: 240. },
        ),
    ] {
        let d = Document::from_bytes(bytes, None).unwrap();
        assert!(!d.partial_view);
        assert_eq!(d.encode().unwrap(), bytes);
        d.verify_roundtrip().unwrap();
        near(d.position(15).unwrap(), red);
        near(d.position(16).unwrap(), green);
        near(red, expected_vector(&d, 11, 13, 14));
        near(green, expected_vector(&d, 15, 13, 14));
        for (id, source) in [(15, 11), (16, 15)] {
            let def = d.objects[id - 1].kind.transform_definition().unwrap();
            assert_eq!(def.element(), ElementType::Point);
            assert_eq!(def.source(), source);
            assert_eq!(def.wire_flavor(), NativeTransformFlavor::Vector);
            assert_eq!(
                def.transform(),
                &Transform::Translate(DisplacementExpr::Vector { start: 13, end: 14 })
            );
            assert_eq!(d.objects[id - 1].kind.dependencies(), vec![source, 13, 14]);
        }
        assert_eq!(
            d.objects[14].visual_style,
            Some(VisualStyle::Point {
                shape: PointStyle::Large,
                color: RgbColor {
                    red: 255,
                    green: 0,
                    blue: 0
                }
            })
        );
        assert_eq!(
            d.objects[15].visual_style,
            Some(VisualStyle::Point {
                shape: PointStyle::Large,
                color: RgbColor {
                    red: 0,
                    green: 128,
                    blue: 0
                }
            })
        );
    }
}
#[test]
fn inherited_point_vectors_live_source_anchor_style_edits_undo_delete_and_reopen() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/inherited_point_transform53_baseline_native_saved.gsp"
    );
    let mut d = Document::from_bytes(bytes, None).unwrap();
    assert!(d.move_point(5, Point { x: 260., y: 180. }));
    near(d.position(15).unwrap(), Point { x: 280., y: 200. });
    assert!(d.move_point(14, Point { x: 150., y: 90. }));
    near(d.position(15).unwrap(), Point { x: 290., y: 190. });
    near(d.position(16).unwrap(), Point { x: 360., y: 200. });
    d.selected.insert(15);
    d.set_selected_color(RgbColor {
        red: 0,
        green: 0,
        blue: 255,
    })
    .unwrap();
    d.verify_roundtrip().unwrap();
    let encoded = d.encode().unwrap();
    let mut reopened = Document::from_bytes(&encoded, None).unwrap();
    assert!(!reopened.partial_view);
    near(reopened.position(16).unwrap(), Point { x: 360., y: 200. });
    assert_eq!(
        reopened.objects[14].visual_style,
        Some(VisualStyle::Point {
            shape: PointStyle::Large,
            color: RgbColor {
                red: 0,
                green: 0,
                blue: 255
            }
        })
    );
    let old = records(bytes).unwrap();
    let new = records(&encoded).unwrap();
    for id in [15, 16] {
        let at = d.objects[id - 1].header_offset.unwrap();
        let oldheader = old.iter().find(|r| r.data.start == at).unwrap();
        let newheader = new.iter().filter(|r| r.kind == 0x7d0).nth(id - 1).unwrap();
        assert_eq!(
            &bytes[oldheader.data.start + 6..oldheader.data.start + 10],
            &encoded[newheader.data.start + 6..newheader.data.start + 10]
        );
    }
    reopened.selected.insert(11);
    let removed = reopened.clear_selected().unwrap();
    assert!(removed.contains(&15) && removed.contains(&16));
    reopened.verify_roundtrip().unwrap();
    reopened.undo();
    near(reopened.position(16).unwrap(), Point { x: 360., y: 200. });
    reopened.verify_roundtrip().unwrap();
}
#[test]
fn inherited_point_vector_accepts_point_sources_and_rejects_invalid_operand_roles() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/inherited_point_transform53_baseline_native_saved.gsp"
    );
    let d = Document::from_bytes(bytes, None).unwrap();
    let rs = records(bytes).unwrap();
    let at = d.objects[14].header_offset.unwrap();
    let index = rs.iter().position(|r| r.data.start == at).unwrap();
    let r = rs[index + 1].data.clone();
    let mut wrong = bytes.to_vec();
    wrong[r.start + 4..r.start + 8].copy_from_slice(&1u32.to_le_bytes());
    let pd = Document::from_bytes(&wrong, None).unwrap();
    assert_eq!(pd.objects[14].kind, Kind::VectorTranslatedPoint(1, 13, 14));
    near(pd.position(15).unwrap(), expected_vector(&pd, 1, 13, 14));
    let mut wrong = bytes.to_vec();
    wrong[at + 6..at + 8].copy_from_slice(&2u16.to_le_bytes());
    let pd = Document::from_bytes(&wrong, None).unwrap();
    assert_eq!(pd.objects[14].kind, Kind::Unsupported(16));
    let mut wrong = bytes.to_vec();
    wrong[r.start + 8..r.start + 12].copy_from_slice(&3u32.to_le_bytes());
    let pd = Document::from_bytes(&wrong, None).unwrap();
    assert_eq!(pd.objects[14].kind, Kind::Unsupported(16));
    let mut payload = Vec::new();
    put_record(&mut payload, 0x07d3, &[16, 0, 0, 0]);
    let mut wrong = bytes.to_vec();
    wrong.splice(rs[index + 2].start..rs[index + 2].start, payload);
    let pd = Document::from_bytes(&wrong, None).unwrap();
    assert_eq!(pd.objects[14].kind, Kind::Unsupported(16));
}
