use super::*;
use std::ops::Range;

fn displayed_rows(doc: &Document, id: usize) -> Vec<String> {
    doc.referenced_text_runs(id)
        .unwrap()
        .into_iter()
        .map(|row| row.into_iter().map(|run| run.text).collect())
        .collect()
}
fn caption_payload(bytes: &[u8], kind: u32) -> Range<usize> {
    let rs = records(bytes).unwrap();
    let first = rs
        .iter()
        .position(|record| {
            record.kind == 0x7d0
                && bytes[record.data.start..record.data.start + 2] == 73u16.to_le_bytes()
        })
        .unwrap();
    rs[first..]
        .iter()
        .take_while(|record| record.kind != 0x7d7)
        .find(|record| record.kind == kind)
        .unwrap()
        .data
        .clone()
}
fn caption_text(doc: &Document, id: usize) -> &super::super::referenced_text::ReferencedText {
    let Kind::ReferencedText { text, .. } = &doc.objects[id - 1].kind else {
        panic!("Expected referenced caption")
    };
    text
}
fn source_caption(doc: &Document, id: usize) -> String {
    let Kind::HideShowButton { label, .. } = &doc.objects[id - 1].kind else {
        panic!("Expected Hide/Show button")
    };
    label.clone()
}

#[test]
fn authored_referenced_caption_uses_live_point_parameter_and_action_sources() {
    let mut doc = Document::new();
    let point = doc.add(Kind::Free(Point { x: 220., y: 160. }));
    doc.set_point_label(point, "Alpha", false).unwrap();
    let parameter = doc.add_named_parameter("n", 2., (10, 30)).unwrap();
    let button = doc.add_hide_show_button(point).unwrap();
    let caption = doc
        .add_referenced_text(
            vec![point, parameter, button],
            "<VL<?x1><?1x2<T5xCACHE>><SW2<?x3>>>",
            (30, 80),
        )
        .unwrap();
    assert_eq!(
        displayed_rows(&doc, caption),
        [
            "Alpha".to_owned(),
            "2.0".to_owned(),
            source_caption(&doc, button)
        ]
    );
    let runs = doc.referenced_text_runs(caption).unwrap();
    assert_eq!(runs[0][0].reference, Some(1));
    assert_eq!(runs[2][0].reference, Some(3));
    assert_eq!(runs[2][0].style.font.bold, Some(true));
    let before = doc.encode().unwrap();
    let reopened = Document::from_bytes(&before, None).unwrap();
    assert!(!reopened.partial_view);
    assert_eq!(
        displayed_rows(&reopened, caption),
        displayed_rows(&doc, caption)
    );
    assert_eq!(reopened.encode().unwrap(), before);
    doc.set_point_label(point, "Beta", false).unwrap();
    doc.set_parameter_value(parameter, 4.).unwrap();
    doc.activate_hide_show_button(button).unwrap();
    assert!(!doc.objects[point - 1].visible);
    assert_eq!(
        displayed_rows(&doc, caption),
        [
            "Beta".to_owned(),
            "4.0".to_owned(),
            source_caption(&doc, button)
        ]
    );
    doc.verify_roundtrip().unwrap();
    let mut mapped = reopened.clone();
    mapped.freeze_evaluation_inputs(&reopened, &[]).unwrap();
    mapped.set_evaluation_scalar(parameter, 8.).unwrap();
    assert_eq!(displayed_rows(&mapped, caption)[1], "8.0");
    assert_eq!(mapped.encode().unwrap(), before);
}

#[test]
fn imported_referenced_caption_moves_without_rewriting_cache_or_nonzero_padding() {
    let mut authored = Document::new();
    let parameter = authored.add_named_parameter("n", 2., (10, 30)).unwrap();
    let caption = authored
        .add_referenced_text(vec![parameter], "<?1x1<T1x0>>", (30, 80))
        .unwrap();
    let mut original = authored.encode().unwrap();
    let rich = caption_payload(&original, 0x8fc);
    let len = u16::from_le_bytes(
        original[rich.start + 16..rich.start + 18]
            .try_into()
            .unwrap(),
    ) as usize;
    for byte in &mut original[rich.start + 18 + len + 1..rich.end] {
        *byte = 0xe6;
    }
    let mut doc = Document::from_bytes(&original, None).unwrap();
    assert_eq!(doc.encode().unwrap(), original);
    assert_eq!(displayed_rows(&doc, caption), ["2.0"]);
    let stored = caption_text(&doc, caption).rich_bytes().to_vec();
    doc.move_text(caption, Point { x: 200., y: 140. }).unwrap();
    let changed = doc.encode().unwrap();
    let new_rich = caption_payload(&changed, 0x8fc);
    let mut expected = stored.clone();
    expected[12..14].copy_from_slice(&200i16.to_le_bytes());
    expected[14..16].copy_from_slice(&140i16.to_le_bytes());
    assert_eq!(&changed[new_rich], expected);
    let reopened = Document::from_bytes(&changed, None).unwrap();
    assert_eq!(caption_text(&reopened, caption).stored_origin(), (200, 140));
    assert_eq!(displayed_rows(&reopened, caption), ["2.0"]);
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
    doc.redo();
    assert_eq!(doc.encode().unwrap(), changed);
    let before_bad_move = doc.encode().unwrap();
    assert!(
        doc.move_text(caption, Point { x: 40000., y: 140. })
            .is_err()
    );
    assert_eq!(doc.encode().unwrap(), before_bad_move);
}

#[test]
fn malformed_referenced_caption_fields_stay_opaque_and_byte_exact() {
    let mut doc = Document::new();
    let parameter = doc.add_named_parameter("n", 2., (10, 30)).unwrap();
    let caption = doc
        .add_referenced_text(vec![parameter], "<VL<?1x1<T1x0>>>", (30, 80))
        .unwrap();
    let original = doc.encode().unwrap();
    let rich = caption_payload(&original, 0x8fc);
    let refs = caption_payload(&original, 0x7d2);
    let font = caption_payload(&original, 0x7d8);
    let mut variants = vec![];
    let at = original
        .windows(5)
        .position(|part| part == b"<?1x1")
        .unwrap();
    let mut bad = original.clone();
    bad[at + 4] = b'2';
    variants.push(("unbound slot", bad));
    let mut bad = original.clone();
    bad[rich.start + 18 + 1] = b'Q';
    variants.push(("unknown layout", bad));
    let mut bad = original.clone();
    bad[rich.start + 16] ^= 1;
    variants.push(("byte count", bad));
    let mut bad = original.clone();
    bad[font.clone()].copy_from_slice(&u16::MAX.to_le_bytes());
    variants.push(("missing font", bad));
    let mut bad = original.clone();
    bad[refs.start + 4..refs.start + 8].copy_from_slice(&(caption as u32).to_le_bytes());
    variants.push(("forward parent", bad));
    let mut bad = original.clone();
    bad[refs.start..refs.start + 4].copy_from_slice(&2u32.to_le_bytes());
    variants.push(("parent count", bad));
    for (reason, bytes) in variants {
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert!(
            matches!(reopened.objects[caption - 1].kind, Kind::Unsupported(73)),
            "{reason}"
        );
        assert!(reopened.partial_view, "{reason}");
        assert_eq!(reopened.encode().unwrap(), bytes, "{reason}");
    }
}

#[test]
fn referenced_caption_rejects_cross_page_and_incompatible_live_sources() {
    let mut doc = Document::new();
    let point = doc.add(Kind::Free(Point { x: 120., y: 160. }));
    doc.set_point_label(point, "Alpha", false).unwrap();
    let caption = doc
        .add_referenced_text(vec![point], "<?x1>", (30, 80))
        .unwrap();
    let before = doc.encode().unwrap();
    for parents in [vec![0], vec![caption + 1]] {
        assert!(doc.add_referenced_text(parents, "<?x1>", (30, 80)).is_err());
        assert_eq!(doc.encode().unwrap(), before);
    }
    let chained = doc.add_referenced_text(vec![caption], "<?x1>", (30, 100)).unwrap();
    assert_eq!(displayed_rows(&doc, chained), ["Alpha"]);
    let text = doc
        .add_text("Unreferenced literal", Point { x: 30., y: 140. })
        .unwrap();
    let live = doc.add_referenced_text(vec![text], "<?x1>", (30, 80)).unwrap();
    assert_eq!(displayed_rows(&doc, live), ["Unreferenced literal"]);
    let circle = doc.add(Kind::Circle(point, point));
    assert!(doc.add_referenced_text(vec![circle], "<?x1>", (30, 80)).is_err());
    let mut cross_page = doc.clone();
    cross_page.objects[point - 1].page = 1;
    assert!(!super::super::referenced_captions::supported(
        &[point],
        caption_text(&cross_page, caption),
        &cross_page.objects,
        0,
        caption
    ));
    assert!(
        cross_page
            .add_referenced_text(vec![point], "<?x1>", (30, 80))
            .is_err()
    );
    cross_page.dirty = true;
    assert!(cross_page.can_save().is_err());
}

#[test]
fn original_native_url_caption_profile_is_live_and_preserved_exactly() {
    let bytes = include_bytes!("../../../../../tests/fixtures/referenced_rich51_url_visible_input.gsp");
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert!(!doc.partial_view);
    assert!(matches!(doc.objects[2].kind,Kind::ReferencedText{ref parents,..} if parents==&[2]));
    assert_eq!(displayed_rows(&doc, 3), ["OWN_URL"]);
    assert_eq!(doc.text_font_state(3).unwrap().size_code, 24);
    assert_eq!(doc.encode().unwrap(), bytes);
    let raw = include_bytes!("../../../../../tests/fixtures/referenced_rich51_original_url291_group.bin");
    let mut at = 0;
    let mut rich = None;
    while at < raw.len() {
        let len = u32::from_le_bytes(raw[at..at + 4].try_into().unwrap()) as usize;
        let kind = u32::from_le_bytes(raw[at + 4..at + 8].try_into().unwrap());
        if kind == 0x8fc {
            rich = Some(&raw[at + 8..at + 8 + len]);
        }
        at += 8 + len + len % 2;
    }
    let parsed =
        super::super::referenced_text::ReferencedText::from_rich(rich.unwrap(), 1).unwrap();
    assert_eq!(parsed.referenced_slots(), [1]);
    let rows = parsed.evaluate(|_, cached| {
        assert_eq!(cached, Some("☆课件园☆"));
        Some("Updated URL caption".into())
    });
    assert_eq!(rows[0][0].text, "Updated URL caption");
    assert_eq!(parsed.rich_bytes(), rich.unwrap());
}
