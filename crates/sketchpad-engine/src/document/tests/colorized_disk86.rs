use super::*;
use crate::document::colorized::{ColorBinding, ColorRangeBehavior};

#[test]
fn native_colorized_disk_retains_live_fill_identity_and_source_order() {
    for (bytes, visible, source_value, center_x, color) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/colorized_disk86_original_native_saved.gsp"
            )
            .as_slice(),
            true,
            10,
            601.7952755905512,
            (233, 0, 255),
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/colorized_disk86_hide17_native_saved.gsp"
            )
            .as_slice(),
            false,
            10,
            601.7952755905512,
            (233, 0, 255),
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/colorized_disk86_move_free5_native_saved.gsp"
            )
            .as_slice(),
            true,
            10,
            641.7952755905512,
            (233, 0, 255),
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/colorized_disk86_rebind_numeric11_native_saved.gsp"
            )
            .as_slice(),
            true,
            11,
            601.7952755905512,
            (255, 0, 0),
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        let object = &doc.objects[16];
        let Kind::Colorized(definition) = &object.kind else {
            panic!("expected code75 disk")
        };
        assert_eq!(definition.element(), ElementType::Disk);
        assert_eq!(definition.source(), 16);
        assert_eq!(
            definition.binding(),
            &ColorBinding::Spectrum {
                value: source_value
            }
        );
        assert_eq!(definition.behavior(), ColorRangeBehavior::Reflect);
        assert_eq!(object.construction_dependencies(), vec![16, source_value]);
        assert!(object.kind.is_filled_region());
        assert_eq!(doc.is_displayed(17), visible);
        let (center, radius) = doc.circle_interior_geometry(17).unwrap();
        assert!(
            (center.x - center_x).abs() < 1e-8,
            "center={center:?}, expected={center_x}"
        );
        assert!(radius > 0.);
        assert_eq!(doc.filled_region_points(17).unwrap().len(), 192);
        assert_eq!(
            doc.colorized_color(17),
            Some(RgbColor {
                red: color.0,
                green: color.1,
                blue: color.2
            })
        );
        assert!(matches!(
            doc.effective_visual_style(17),
            Some(VisualStyle::Polygon { opacity: 32767, .. })
        ));
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn disk_color_cycle_uses_shared_color_binding() {
    let bytes =
        include_bytes!("../../../../../tests/fixtures/colorized_disk86_original_native_saved.gsp");
    let doc = Document::from_bytes(bytes, None).unwrap();
    let at = doc.objects[16].constraint_offset.unwrap();
    let mut changed = bytes.to_vec();
    changed[at + 4] = 0x11;
    let cycled = Document::from_bytes(&changed, None).unwrap();
    let Kind::Colorized(definition) = &cycled.objects[16].kind else { panic!("expected colorized disk") };
    assert_eq!(definition.behavior(), ColorRangeBehavior::Cycle);
    assert_eq!(cycled.circle_interior_geometry(17), doc.circle_interior_geometry(17));
    assert!(cycled.colorized_color(17).is_some());
    assert_eq!(cycled.encode().unwrap(), changed);
}

#[test]
fn native_grayscale_disk_uses_same_live_carrier() {
    for (bytes, visible, value, gray) in [
        (
            include_bytes!("../../../../../tests/fixtures/colorized_disk86_gray_original_native_saved.gsp").as_slice(),
            true, 10, 0,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/colorized_disk86_gray_hide15_native_saved.gsp").as_slice(),
            false, 10, 0,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/colorized_disk86_gray_rebind_numeric8_native_saved.gsp").as_slice(),
            true, 8, 124,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        let object = &doc.objects[14];
        let Kind::Colorized(definition) = &object.kind else { panic!("expected grayscale disk") };
        assert_eq!(definition.element(), ElementType::Disk);
        assert_eq!(definition.source(), 14);
        assert_eq!(definition.binding(), &ColorBinding::Grayscale { value });
        assert_eq!(doc.is_displayed(15), visible);
        assert!(doc.circle_interior_geometry(15).is_some());
        assert_eq!(doc.colorized_color(15), Some(RgbColor { red: gray, green: gray, blue: gray }));
        assert!(matches!(doc.effective_visual_style(15), Some(VisualStyle::Polygon { opacity: 65535, .. })));
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn disk_transforms_use_shared_transform_definitions() {
    assert!(
        crate::document::transform_model::TransformDefinition::new(
            1,
            ElementType::Disk,
            crate::document::transform_model::Transform::Reflect { mirror: 2 },
            NativeTransformFlavor::Reflection,
        )
        .is_ok()
    );
}
