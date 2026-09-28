use super::*;

#[test]
fn native_point_anchored_image_tracks_its_parent_and_preserves_records() {
    let states: [(&[u8], bool, Point); 4] = [
        (
            include_bytes!(
                "../../../../../tests/fixtures/anchored_image94_coin_original_input.gsp"
            ),
            true,
            Point { x: 686., y: 341. },
        ),
        (
            include_bytes!("../../../../../tests/fixtures/anchored_image94_coin_hide_input.gsp"),
            false,
            Point { x: 686., y: 341. },
        ),
        (
            include_bytes!("../../../../../tests/fixtures/anchored_image94_coin_move_input.gsp"),
            true,
            Point { x: 726., y: 361. },
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/anchored_image94_coin_move_hide_input.gsp"
            ),
            false,
            Point { x: 726., y: 361. },
        ),
    ];
    for (bytes, visible, point) in states {
        let doc = Document::from_bytes(bytes, None).unwrap();
        let Kind::Image { png, anchor, .. } = &doc.objects[28].kind else {
            panic!("native code99 image was not decoded")
        };
        assert_eq!(*anchor, Some(2));
        assert_eq!(
            crate::image_records::ImageFile::png_size(png).unwrap(),
            (193, 193)
        );
        assert_eq!(doc.objects[28].kind.dependencies(), [2]);
        assert_eq!(doc.position(2), Some(point));
        assert_eq!(doc.is_displayed(29), visible);
        let transform = doc.image_transform(29).unwrap();
        assert_eq!(transform, [1., 0., point.x - 96.5, 0., 1., point.y - 96.5]);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn native_anchored_image_follows_a_constructed_point() {
    let original = include_bytes!(
        "../../../../../tests/fixtures/anchored_image94_transformed_original_input.gsp"
    );
    let moved = include_bytes!(
        "../../../../../tests/fixtures/anchored_image94_transformed_distance_input.gsp"
    );
    let before = Document::from_bytes(original, None).unwrap();
    let after = Document::from_bytes(moved, None).unwrap();
    for (document, bytes) in [(&before, original.as_slice()), (&after, moved.as_slice())] {
        assert!(matches!(
            document.objects[9].kind,
            Kind::Image {
                anchor: Some(9),
                ..
            }
        ));
        assert_eq!(document.encode().unwrap(), bytes);
    }
    let parent_before = before.position(9).unwrap();
    let parent_after = after.position(9).unwrap();
    let image_before = before.image_transform(10).unwrap();
    let image_after = after.image_transform(10).unwrap();
    assert!((image_after[2] - image_before[2] - (parent_after.x - parent_before.x)).abs() < 1e-9);
    assert!((image_after[5] - image_before[5] - (parent_after.y - parent_before.y)).abs() < 1e-9);
    assert!((parent_after.y - parent_before.y + 18.8976).abs() < 1e-3);
}

#[test]
fn native_point_anchored_image_scales_about_its_parent() {
    let original =
        include_bytes!("../../../../../tests/fixtures/anchored_image94_scale_original_input.gsp");
    let smaller = include_bytes!(
        "../../../../../tests/fixtures/anchored_image94_scale_change_scale_input.gsp"
    );
    let baseline = Document::from_bytes(original, None).unwrap();
    let changed = Document::from_bytes(smaller, None).unwrap();
    for (doc, bytes) in [
        (&baseline, original.as_slice()),
        (&changed, smaller.as_slice()),
    ] {
        let Kind::Image { png, anchor, .. } = &doc.objects[64].kind else {
            panic!("scaled picture did not decode")
        };
        assert_eq!(*anchor, Some(64));
        let (width, height) = crate::image_records::ImageFile::png_size(png).unwrap();
        assert_eq!((width, height), (519, 365));
        let transform = doc.image_transform(65).unwrap();
        let parent = doc.position(64).unwrap();
        assert!((transform[2] + transform[0] * width as f64 / 2. - parent.x).abs() < 1e-9);
        assert!((transform[5] + transform[4] * height as f64 / 2. - parent.y).abs() < 1e-9);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
    assert!(baseline.image_transform(65).unwrap()[0] > 1.5);
    assert_eq!(changed.image_transform(65).unwrap()[0], 1.2);
}

#[test]
fn anchored_image_rejects_unproved_rotation_and_resource_index() {
    let original =
        include_bytes!("../../../../../tests/fixtures/anchored_image94_coin_original_input.gsp");
    let doc = Document::from_bytes(original, None).unwrap();
    let start = doc.objects[28].header_offset.unwrap() - 8;
    let rs = records(original).unwrap();
    let at = rs.iter().position(|r| r.start == start).unwrap();
    let operation = &rs[at + 2];
    let resource = &rs[at + 4];
    assert_eq!(operation.kind, 0x07d3);
    assert_eq!(resource.kind, 0x1f44);
    let mut rotated = original.to_vec();
    rotated[operation.data.start + 20..operation.data.start + 28]
        .copy_from_slice(&1.0_f64.to_le_bytes());
    let rejected = Document::from_bytes(&rotated, None).unwrap();
    assert_eq!(rejected.objects[28].kind, Kind::Unsupported(99));
    assert_eq!(rejected.encode().unwrap(), rotated);
    let mut absent_resource = original.to_vec();
    absent_resource[resource.data.clone()].copy_from_slice(&999_u16.to_le_bytes());
    let rejected = Document::from_bytes(&absent_resource, None).unwrap();
    assert_eq!(rejected.objects[28].kind, Kind::Unsupported(99));
    assert_eq!(rejected.encode().unwrap(), absent_resource);
}
