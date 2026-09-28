use super::*;

#[test]
fn native_colorized_disk_mapping_decodes_live_fills_and_separate_circle_binding() {
    let mut original_disks = None;
    for (bytes, wrapper_visible, root_visible, stroke_visible, fill_visible) in [
        (include_bytes!("../../../../../tests/fixtures/colorized_iteration88_original_native_saved.gsp").as_slice(), true, true, true, true),
        (include_bytes!("../../../../../tests/fixtures/colorized_iteration88_hide_wrapper17_native_saved.gsp").as_slice(), false, true, true, true),
        (include_bytes!("../../../../../tests/fixtures/colorized_iteration88_hide_root18_native_saved.gsp").as_slice(), true, false, true, true),
        (include_bytes!("../../../../../tests/fixtures/colorized_iteration88_hide_stroke_binding19_native_saved.gsp").as_slice(), true, true, false, true),
        (include_bytes!("../../../../../tests/fixtures/colorized_iteration88_hide_fill_binding20_native_saved.gsp").as_slice(), true, true, true, false),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(matches!(doc.objects[17].kind, Kind::MappedPointIteration { .. }));
        assert_eq!(doc.objects[18].kind, Kind::IterationBinding(15, 18));
        assert_eq!(doc.objects[19].kind, Kind::IterationBinding(17, 18));
        assert_eq!(doc.is_displayed(17), wrapper_visible);
        assert_eq!(doc.is_displayed(18), root_visible);
        assert_eq!(doc.is_displayed(19), stroke_visible);
        assert_eq!(doc.is_displayed(20), fill_visible);
        assert!(doc.circle_iteration_images(19).unwrap().len() > 5);
        let disks = doc.colorized_disk_iteration_images(20).unwrap();
        assert!(disks.len() > 5);
        assert!(disks.iter().any(|(_, _, color)| *color != disks[0].2));
        if let Some(original) = &original_disks {
            assert_eq!(&disks, original);
        } else {
            original_disks = Some(disks.clone());
        }
        let rightmost = disks.iter().find(|(center, _, _)|
            (center.x - 809.7480314960635).abs() < 1e-6 && (center.y - 390.).abs() < 1e-6).unwrap();
        assert_eq!(rightmost.2, RgbColor { red: 255, green: 0, blue: 0 });
        assert!(matches!(doc.effective_visual_style(20), Some(VisualStyle::Polygon { opacity: 32767, .. })));
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn native_scalar_rebinding_changes_each_generation_color_without_moving_disks() {
    let original = Document::from_bytes(
        include_bytes!(
            "../../../../../tests/fixtures/colorized_iteration88_original_native_saved.gsp"
        ),
        None,
    )
    .unwrap();
    let rebound_bytes = include_bytes!(
        "../../../../../tests/fixtures/colorized_disk86_rebind_numeric11_native_saved.gsp"
    );
    let rebound = Document::from_bytes(rebound_bytes, None).unwrap();
    let before = original.colorized_disk_iteration_images(20).unwrap();
    let after = rebound.colorized_disk_iteration_images(20).unwrap();
    assert_eq!(before.len(), after.len());
    let mismatch = before
        .iter()
        .zip(&after)
        .find(|((p, r, _), (q, s, _))| p != q || r != s);
    assert!(mismatch.is_none(), "mismatch={mismatch:?}");
    for x in [
        643.3858267716537,
        684.9763779527561,
        726.5669291338586,
        768.157480314961,
        809.7480314960635,
    ] {
        let (_, _, color) = after
            .iter()
            .find(|(center, _, _)| (center.x - x).abs() < 1e-6 && (center.y - 390.).abs() < 1e-6)
            .unwrap();
        assert_eq!(
            *color,
            RgbColor {
                red: 255,
                green: 0,
                blue: 0
            }
        );
    }
    assert_eq!(rebound.encode().unwrap(), rebound_bytes);
}

#[test]
fn moving_free_source_translates_every_colored_generation() {
    let original = Document::from_bytes(
        include_bytes!(
            "../../../../../tests/fixtures/colorized_iteration88_original_native_saved.gsp"
        ),
        None,
    )
    .unwrap();
    let moved_bytes = include_bytes!(
        "../../../../../tests/fixtures/colorized_disk86_move_free5_native_saved.gsp"
    );
    let moved = Document::from_bytes(moved_bytes, None).unwrap();
    let before = original.colorized_disk_iteration_images(20).unwrap();
    let after = moved.colorized_disk_iteration_images(20).unwrap();
    assert_eq!(before.len(), after.len());
    assert!(
        before
            .iter()
            .zip(&after)
            .all(|((p, r, c), (q, s, d))| (q.x - p.x - 40.).abs() < 1e-8
                && (q.y - p.y).abs() < 1e-8
                && (s - r).abs() < 1e-8
                && c == d)
    );
    assert_eq!(moved.encode().unwrap(), moved_bytes);
}
