use super::*;

#[test]
fn native_two_resource_images_keep_distinct_pngs_transforms_and_visibility() {
    for (bytes, visible20, visible21, transform20, transform21) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/image_objects91_original_native_saved.gsp"
            )
            .as_slice(),
            true,
            true,
            [1., 0., 280., 0., 1., 332.],
            [1., 0., 280., 0., 1., 378.],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/image_objects91_hide20_native_saved.gsp")
                .as_slice(),
            false,
            true,
            [1., 0., 280., 0., 1., 332.],
            [1., 0., 280., 0., 1., 378.],
        ),
        (
            include_bytes!("../../../../../tests/fixtures/image_objects91_hide21_native_saved.gsp")
                .as_slice(),
            true,
            false,
            [1., 0., 280., 0., 1., 332.],
            [1., 0., 280., 0., 1., 378.],
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/image_objects91_hide_both_native_saved.gsp"
            )
            .as_slice(),
            false,
            false,
            [1., 0., 280., 0., 1., 332.],
            [1., 0., 280., 0., 1., 378.],
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/image_objects91_move20_x40_native_saved.gsp"
            )
            .as_slice(),
            true,
            true,
            [1., 0., 320., 0., 1., 332.],
            [1., 0., 280., 0., 1., 378.],
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/image_objects91_move21_y40_native_saved.gsp"
            )
            .as_slice(),
            true,
            true,
            [1., 0., 280., 0., 1., 332.],
            [1., 0., 280., 0., 1., 418.],
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        let Kind::Image {
            png: first,
            transform: first_transform,
            ..
        } = &doc.objects[19].kind
        else {
            panic!("object20 image missing")
        };
        let Kind::Image {
            png: second,
            transform: second_transform,
            ..
        } = &doc.objects[20].kind
        else {
            panic!("object21 image missing")
        };
        assert_eq!(
            crate::image_records::ImageFile::png_size(first).unwrap(),
            (253, 35)
        );
        assert_eq!(
            crate::image_records::ImageFile::png_size(second).unwrap(),
            (321, 35)
        );
        assert_ne!(first, second);
        assert_eq!(*first_transform, transform20);
        assert_eq!(*second_transform, transform21);
        assert_eq!(doc.is_displayed(20), visible20);
        assert_eq!(doc.is_displayed(21), visible21);
        assert!(doc.partial_view);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn empty_native_marker_keeps_single_resource_image_as_read_only_object() {
    let bytes = include_bytes!("../../../../../tests/fixtures/image_objects91_marker_original.gsp");
    let doc = Document::from_bytes(bytes, None).unwrap();
    let Kind::Image { png, transform, .. } = &doc.objects[80].kind else {
        panic!("marked image missing")
    };
    assert_eq!(
        crate::image_records::ImageFile::png_size(png).unwrap(),
        (197, 171)
    );
    assert_eq!(*transform, [1., 0., 579., 0., 1., -43.]);
    assert!(doc.is_displayed(81));
    assert_eq!(doc.encode().unwrap(), bytes);
}
