use super::*;
use crate::document::colorized::{ColorBinding, ColorRangeBehavior};

#[test]
fn native_ten_point_polygon_wrapper_and_iteration_keep_independent_visibility() {
    for (bytes, wrapper_visible, binding_visible) in [
        (include_bytes!("../../../../../tests/fixtures/colorized_polygon89_original_native_saved.gsp").as_slice(), true, true),
        (include_bytes!("../../../../../tests/fixtures/colorized_polygon89_hide_wrapper14_native_saved.gsp").as_slice(), false, true),
        (include_bytes!("../../../../../tests/fixtures/colorized_polygon89_hide_binding23_native_saved.gsp").as_slice(), true, false),
        (include_bytes!("../../../../../tests/fixtures/colorized_polygon89_hide_wrapper14_binding23_native_saved.gsp").as_slice(), false, false),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        let Kind::Colorized(ref definition) = doc.objects[13].kind else { panic!("polygon wrapper missing") };
        assert_eq!(definition.element(), ElementType::Polygon);
        assert_eq!(definition.source(), 12);
        assert_eq!(definition.binding(), &ColorBinding::Spectrum { value: 13 });
        assert_eq!(definition.behavior(), ColorRangeBehavior::Reflect);
        assert_eq!(doc.objects[13].construction_dependencies(), vec![12, 13]);
        assert!(doc.objects[13].kind.is_filled_region());
        assert_eq!(doc.filled_region_points(14).unwrap().len(), 10);
        assert!(matches!(doc.effective_visual_style(14), Some(VisualStyle::Polygon { .. })));
        assert!(matches!(doc.objects[21].kind, Kind::MappedPointIteration { .. }));
        assert_eq!(doc.objects[22].kind, Kind::IterationBinding(14, 22));
        assert_eq!(doc.is_displayed(14), wrapper_visible);
        assert_eq!(doc.is_displayed(23), binding_visible);
        let generations = doc.colorized_polygon_iteration_images(23).unwrap();
        assert!(!generations.is_empty());
        assert!(generations.iter().all(|(points, _)| points.len() == 10));
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn native_range_and_point_controls_keep_geometry_and_color_separate() {
    let original = Document::from_bytes(
        include_bytes!(
            "../../../../../tests/fixtures/colorized_polygon89_original_native_saved.gsp"
        ),
        None,
    )
    .unwrap();
    let range_bytes = include_bytes!(
        "../../../../../tests/fixtures/colorized_polygon89_range_hi2_native_saved.gsp"
    );
    let range = Document::from_bytes(range_bytes, None).unwrap();
    assert_eq!(
        original.filled_region_points(14),
        range.filled_region_points(14)
    );
    assert_ne!(original.colorized_color(14), range.colorized_color(14));
    assert_eq!(original.numeric_value(13), range.numeric_value(13));
    assert_eq!(range.encode().unwrap(), range_bytes);

    let moved_bytes = include_bytes!(
        "../../../../../tests/fixtures/colorized_polygon89_move_free2_native_saved.gsp"
    );
    let moved = Document::from_bytes(moved_bytes, None).unwrap();
    assert_ne!(
        original.filled_region_points(14),
        moved.filled_region_points(14)
    );
    assert_ne!(original.numeric_value(13), moved.numeric_value(13));
    assert_eq!(moved.encode().unwrap(), moved_bytes);
}

#[test]
fn original_gray_rgb_and_hsv_polygon_packets_use_the_same_filled_object_role() {
    for (bytes, id, source, operation) in [
        (
            include_bytes!("../../../../../tests/fixtures/colorized_polygon89_gray_original.gsp")
                .as_slice(),
            17,
            16,
            0x22,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/colorized_polygon89_rgb_original.gsp")
                .as_slice(),
            41,
            34,
            0x32,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/colorized_polygon89_hsv_original.gsp")
                .as_slice(),
            114,
            113,
            0x42,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        let Kind::Colorized(ref definition) = doc.objects[id - 1].kind else {
            panic!("unsupported colorized polygon in object {id}")
        };
        assert_eq!(definition.element(), ElementType::Polygon);
        assert_eq!(definition.source(), source);
        assert_eq!(definition.native_operation(), operation);
        assert_eq!(definition.range(), (0., 1.));
        assert!(doc.objects[id - 1].kind.is_filled_region());
        assert!(matches!(
            doc.objects[id - 1].visual_style,
            Some(VisualStyle::Polygon { .. })
        ));
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn native_depth_controls_add_generations_with_live_polygon_colors() {
    let mut previous_count = 0;
    let mut previous_colors = std::collections::HashSet::new();
    for bytes in [
        include_bytes!("../../../../../tests/fixtures/colorized_polygon89_depth1_native_saved.gsp")
            .as_slice(),
        include_bytes!("../../../../../tests/fixtures/colorized_polygon89_depth2_native_saved.gsp")
            .as_slice(),
        include_bytes!("../../../../../tests/fixtures/colorized_polygon89_depth3_native_saved.gsp")
            .as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        let images = doc.colorized_polygon_iteration_images(23).unwrap();
        let colors = images
            .iter()
            .map(|(_, color)| (color.red, color.green, color.blue))
            .collect::<std::collections::HashSet<_>>();
        assert!(images.len() > previous_count);
        assert!(colors.len() > previous_colors.len());
        assert!(previous_colors.is_subset(&colors));
        assert_eq!(doc.encode().unwrap(), bytes);
        previous_count = images.len();
        previous_colors = colors;
    }
}
