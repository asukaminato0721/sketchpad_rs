use super::*;

#[test]
fn native_free_text_color_runs_keep_color_visibility_and_position() {
    for (bytes, visible, date_color, x_shift) in [
        (
            include_bytes!("../../../../../tests/fixtures/rich_style92_original_native_saved.gsp")
                .as_slice(),
            true,
            [0, 0, 255],
            0,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/rich_style92_hide14_native_saved.gsp")
                .as_slice(),
            false,
            [0, 0, 255],
            0,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/rich_style92_g2_native_saved.gsp")
                .as_slice(),
            true,
            [0, 1, 255],
            0,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/rich_style92_l050_native_saved.gsp")
                .as_slice(),
            true,
            [0, 0, 79],
            0,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/rich_style92_box_x40_native_saved.gsp")
                .as_slice(),
            true,
            [0, 0, 255],
            40,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(matches!(&doc.objects[13].kind, Kind::PlainText { .. }));
        assert_eq!(doc.is_displayed(14), visible);
        let rows = doc.text_rich_runs(14).unwrap();
        let colors = rows
            .iter()
            .flatten()
            .filter_map(|run| run.style.rgb_color())
            .collect::<Vec<_>>();
        assert!(colors.contains(&date_color));
        let (_, rect) = doc.text_content(14).unwrap();
        assert_eq!(rect.left, 193 + x_shift);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn unproved_zero_channel_controls_remain_opaque_and_lossless() {
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/rich_style92_r0_native_saved.gsp").as_slice(),
        include_bytes!("../../../../../tests/fixtures/rich_style92_g0_native_saved.gsp").as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(doc.objects[13].kind, Kind::Unsupported(0));
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}
