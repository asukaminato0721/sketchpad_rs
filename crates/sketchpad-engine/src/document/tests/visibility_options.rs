use super::*;
use crate::action_records::{HideShowButtonGroup, VisibilityOptions};

macro_rules! native_case {
    ($state:literal) => {
        (
            $state,
            include_bytes!(concat!(
                "../../../../../tests/fixtures/visibility48_",
                $state,
                "_input.gsp"
            ))
            .as_slice(),
            include_bytes!(concat!(
                "../../../../../tests/fixtures/visibility48_",
                $state,
                "_step1_native_verified.gsp"
            ))
            .as_slice(),
            include_bytes!(concat!(
                "../../../../../tests/fixtures/visibility48_",
                $state,
                "_step2_native_verified.gsp"
            ))
            .as_slice(),
        )
    };
}

fn compare_visibility_state(doc: &Document, native: &[u8], context: &str) {
    let expected = Document::from_bytes(native, None).unwrap();
    assert!(!expected.partial_view, "{context}");
    assert_eq!(doc.objects.len(), expected.objects.len(), "{context}");
    for (actual, expected) in doc.objects.iter().zip(&expected.objects) {
        assert_eq!(actual.kind, expected.kind, "{context} object{}", actual.id);
        assert_eq!(
            actual.visible, expected.visible,
            "{context} object{}",
            actual.id
        );
        assert_eq!(
            actual.visual_style, expected.visual_style,
            "{context} object{}",
            actual.id
        );
    }
    let encoded = doc.encode().unwrap();
    // Native also rewrites document frames and font records when activating a
    // button. Compare the action records independently of that unrelated rewrite.
    assert_eq!(
        visibility_records(&encoded, true),
        visibility_records(native, true),
        "{context} action records"
    );
    assert_eq!(
        visibility_records(&encoded, false),
        visibility_records(&doc.original, false),
        "{context} preserved opaque auxiliary records"
    );
    doc.verify_roundtrip().unwrap();
}

fn visibility_records(bytes: &[u8], semantic: bool) -> Vec<Vec<(u32, Vec<u8>)>> {
    let mut offset = 4;
    let mut groups = Vec::new();
    let mut current = Vec::new();
    let mut is_visibility = false;
    while offset < bytes.len() {
        let length = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        let kind = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
        let payload = &bytes[offset + 8..offset + 8 + length];
        if kind == 0x07d0 {
            current.clear();
            is_visibility = u16::from_le_bytes(payload[..2].try_into().unwrap()) == 62;
        }
        if is_visibility
            && if semantic {
                matches!(kind, 0x07d2 | 0x0906 | 0x07d5 | 0x232e)
            } else {
                kind == 0x07d8
            }
        {
            let end = if semantic && kind == 0x07d5 {
                25 + u16::from_le_bytes(payload[22..24].try_into().unwrap()) as usize
            } else {
                payload.len()
            };
            current.push((kind, payload[..end].to_vec()));
        }
        if is_visibility && kind == 0x07d7 {
            groups.push(std::mem::take(&mut current));
            is_visibility = false;
        }
        offset += 8 + length + length % 2;
    }
    groups
}

#[test]
fn all_native_visibility_options_repeat_their_verified_action() {
    for (state, input, first, second) in [
        native_case!("a0_o0"),
        native_case!("a0_o1"),
        native_case!("a0_o2"),
        native_case!("a0_o3"),
        native_case!("a0_o4"),
        native_case!("a0_o5"),
        native_case!("a0_o6"),
        native_case!("a0_o7"),
        native_case!("a1_o0"),
        native_case!("a1_o1"),
        native_case!("a1_o2"),
        native_case!("a1_o3"),
        native_case!("a1_o4"),
        native_case!("a1_o5"),
        native_case!("a1_o6"),
        native_case!("a1_o7"),
    ] {
        let mut doc = Document::from_bytes(input, None).unwrap();
        assert!(!doc.partial_view, "{state}");
        assert_eq!(doc.encode().unwrap(), input, "{state} initial");
        let Kind::HideShowButton { options, .. } = &doc.objects[3].kind else {
            panic!("{state}")
        };
        let options = *options;
        doc.activate_hide_show_button(4).unwrap();
        compare_visibility_state(&doc, first, &format!("{state} first"));
        doc.activate_hide_show_button(4).unwrap();
        compare_visibility_state(&doc, second, &format!("{state} second"));
        let Kind::HideShowButton {
            options: actual, ..
        } = doc.objects[3].kind
        else {
            unreachable!()
        };
        assert_eq!(actual, options);
    }
}

#[test]
fn caption_format_and_prefix_follow_controlled_native_fields() {
    for (state, input, first, second) in [
        native_case!("a0_o6_flip7"),
        native_case!("a0_o7_flip6"),
        native_case!("a0_o6_caption3"),
        native_case!("a0_o6_caption5"),
        native_case!("a0_o6_caption7"),
        native_case!("a0_o7_caption6"),
        native_case!("a0_o6_caption7_mismatch"),
        native_case!("a0_o6_caption7_english"),
    ] {
        let mut doc = Document::from_bytes(input, None).unwrap();
        assert_eq!(doc.encode().unwrap(), input);
        doc.activate_hide_show_button(4).unwrap();
        compare_visibility_state(&doc, first, &format!("{state} first"));
        doc.activate_hide_show_button(4).unwrap();
        compare_visibility_state(&doc, second, &format!("{state} second"));
        doc.undo();
        compare_visibility_state(&doc, first, &format!("{state} undo"));
        doc.redo();
        compare_visibility_state(&doc, second, &format!("{state} redo"));
    }
}

#[test]
fn hidden_child_materializes_native_caption_without_unhiding_button() {
    let (_, input, first, second) = native_case!("a0_o4_missing_child");
    let mut doc = Document::from_bytes(input, None).unwrap();
    assert!(!doc.objects[3].visible);
    assert!(matches!(
        doc.objects[3].kind,
        Kind::HideShowButton {
            caption_flags: None,
            ..
        }
    ));
    assert!(doc.activate_hide_show_button(4).is_err());
    assert_eq!(doc.encode().unwrap(), input);
    doc.activate_hide_show_button_as_child(4).unwrap();
    compare_visibility_state(&doc, first, "missing first");
    assert!(!doc.objects[3].visible);
    doc.undo();
    assert_eq!(doc.encode().unwrap(), input);
    doc.redo();
    compare_visibility_state(&doc, first, "missing redo");
    doc.activate_hide_show_button_as_child(4).unwrap();
    compare_visibility_state(&doc, second, "missing second");
}

#[test]
fn visibility_policy_caption_creation_editing_and_styles_round_trip() {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 140., y: 200. }));
    let b = doc.add(Kind::Free(Point { x: 240., y: 240. }));
    doc.objects[b - 1].visible = false;
    let button = doc
        .add_visibility_button_for_targets(
            &[a, b],
            VisibilityAction::Show,
            VisibilityOptions::from_native(1).unwrap(),
        )
        .unwrap();
    let created = doc.encode().unwrap();
    doc.activate_hide_show_button(button).unwrap();
    assert!(doc.objects[a - 1].visible && doc.objects[b - 1].visible);
    assert!(matches!(
        doc.objects[button - 1].kind,
        Kind::HideShowButton {
            action: VisibilityAction::Show,
            ..
        }
    ));
    doc.activate_hide_show_button(button).unwrap();
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), created);
    doc.redo();
    doc.set_hide_show_button_options(button, VisibilityOptions::from_native(5).unwrap())
        .unwrap();
    doc.set_hide_show_button_caption(button, "我的按钮")
        .unwrap();
    doc.activate_hide_show_button(button).unwrap();
    assert!(
        matches!(&doc.objects[button-1].kind,Kind::HideShowButton{label,caption_flags:Some(7),..}if label=="我的按钮")
    );
    doc.set_hide_show_button_caption(button, "Static label!")
        .unwrap();
    doc.verify_roundtrip().unwrap();
    doc.set_hide_show_button_caption(button, "").unwrap();
    doc.verify_roundtrip().unwrap();
    let absent = doc.encode().unwrap();
    let parsed = Document::from_bytes(&absent, None).unwrap();
    assert!(matches!(
        parsed.objects[button - 1].kind,
        Kind::HideShowButton {
            caption_flags: None,
            ..
        }
    ));
    doc.reset_hide_show_button_caption(button).unwrap();
    doc.selected.insert(a);
    doc.set_selected_color(RgbColor {
        red: 0,
        green: 0,
        blue: 255,
    })
    .unwrap();
    let style = doc.objects[a - 1].visual_style;
    doc.activate_hide_show_button(button).unwrap();
    assert_eq!(doc.objects[a - 1].visual_style, style);
    doc.verify_roundtrip().unwrap();
    let before = doc.encode().unwrap();
    assert!(
        doc.set_hide_show_button_caption(button, &"x".repeat(65536))
            .is_err()
    );
    assert_eq!(doc.encode().unwrap(), before);
    assert!(VisibilityOptions::from_native(8).is_err());
}

#[test]
fn resized_source_caption_then_other_page_additions_reopen_strictly() {
    let mut doc = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/visibility48_a0_o6_input.gsp"),
        None,
    )
    .unwrap();
    doc.add_blank_page().unwrap();
    let page_point = doc.add(Kind::Free(Point { x: 180., y: 220. }));
    doc.active_page = 0;
    let source = doc.encode().unwrap();
    let mut doc = Document::from_bytes(&source, None).unwrap();
    doc.active_page = 0;
    doc.activate_hide_show_button(4).unwrap();
    assert!(
        matches!(&doc.objects[3].kind,Kind::HideShowButton{label,caption_flags:Some(4),..}if label=="显示点")
    );
    doc.active_page = 1;
    let new_point = doc.add(Kind::Free(Point { x: 260., y: 220. }));
    doc.add(Kind::Segment(page_point, new_point));
    doc.add_link_page_button_with_target(0, Some(1)).unwrap();
    doc.verify_roundtrip().unwrap();
    let bytes = doc.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(reopened.page_count, 2);
    assert!(
        reopened
            .objects
            .iter()
            .all(|object| !matches!(object.kind, Kind::Unsupported { .. }))
    );
    assert_eq!(reopened.encode().unwrap(), bytes);
    assert!(
        reopened
            .objects
            .iter()
            .any(|object| object.page == 1 && matches!(object.kind, Kind::Segment(..)))
    );
}

#[test]
fn native_geometric_category_and_named_point_captions_match() {
    for (state, input, first, second) in [
        native_case!("category_circle"),
        native_case!("category_arc"),
        native_case!("category_polygon"),
        native_case!("category_ray"),
        native_case!("category_line"),
        native_case!("category_named_point"),
    ] {
        let mut doc = Document::from_bytes(input, None).unwrap();
        let button = doc
            .objects
            .iter()
            .find(|object| matches!(object.kind, Kind::HideShowButton { .. }))
            .unwrap()
            .id;
        assert_eq!(doc.encode().unwrap(), input);
        doc.activate_hide_show_button(button).unwrap();
        compare_visibility_state(&doc, first, &format!("{state} first"));
        doc.activate_hide_show_button(button).unwrap();
        compare_visibility_state(&doc, second, &format!("{state} second"));
    }
}

#[test]
fn generated_caption_rebuilds_known_native_alignment_and_materializes_absent_record() {
    let mut group = HideShowButtonGroup::parse(include_bytes!(
        "../../../../../tests/fixtures/visibility48_original_a0_o5_group.bin"
    ))
    .unwrap();
    assert!(group.set_label("显示点").is_err());
    group.set_generated_label("显示点").unwrap();
    assert_eq!(group.label_utf8().unwrap(), "显示点");
    assert_eq!(group.caption_flags(), Some(4));
    let mut absent = HideShowButtonGroup::parse(include_bytes!(
        "../../../../../tests/fixtures/visibility48_original_a0_o1_group.bin"
    ))
    .unwrap();
    assert_eq!(absent.caption_flags(), None);
    absent.set_generated_label("").unwrap();
    assert!(absent.set_generated_label("显示点").is_err());
    absent.set_caption_flags(Some(4)).unwrap();
    absent.set_generated_label("显示点").unwrap();
    assert_eq!(absent.caption_flags(), Some(4));
}
