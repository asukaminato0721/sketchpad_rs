use super::*;
use crate::document::LocusStroke;

fn bounds(strokes: &[LocusStroke]) -> (f64, f64, f64, f64) {
    strokes.iter().flat_map(|s| [s.start, s.end]).fold(
        (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ),
        |(x0, y0, x1, y1), point| {
            (
                x0.min(point.x),
                y0.min(point.y),
                x1.max(point.x),
                y1.max(point.y),
            )
        },
    )
}

#[test]
fn original_point_locus_collection_decodes_styles_edits_and_reopens() {
    let original = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../gsp-rs/tests/Samples/个人专栏/贺基旭作品/多边形轨迹迭代（hjx4882）.gsp"
    ))
    .unwrap();
    let mut doc = Document::from_bytes(&original, None).unwrap();
    assert!(!doc.partial_view);
    assert!(matches!(doc.objects[16].kind, Kind::Locus(..)));
    assert!(matches!(
        doc.objects[17].kind,
        Kind::MappedPointIteration { .. }
    ));
    assert!(matches!(
        doc.objects[18].kind,
        Kind::IterationBinding(17, 18)
    ));
    let style = doc.effective_visual_style(19).unwrap();
    let strokes = doc.mapped_collection_strokes(19, 96).unwrap();
    assert_eq!(strokes.len(), 3 * 96);
    assert!(
        strokes
            .iter()
            .all(|s| s.style == style && s.mode == LineMode::Segment)
    );
    // Native renders the source locus separately. Here it has collapsed to
    // one point, so the three visible red traces belong to binding 19.
    let original_source = doc.locus_strokes(17, 96).unwrap();
    assert_eq!(original_source.len(), 96);
    assert!(original_source.iter().all(|(start, end, _)| *start == *end));
    let initial = bounds(&strokes);
    for (actual, expected) in [initial.0, initial.1, initial.2, initial.3]
        .into_iter()
        .zip([457.970, 142.354, 802.035, 445.524])
    {
        assert!((actual - expected).abs() < 0.005);
    }
    assert_eq!(doc.encode().unwrap(), original);
    doc.verify_roundtrip().unwrap();

    doc.selected.insert(19);
    doc.set_selected_visibility(false);
    assert!(!doc.is_displayed(19));
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);

    doc.set_parameter_value(13, 0.5).unwrap();
    // The public parameter edit makes the direct source locus visible as a
    // fourth, small triangle; it does not add another mapped generation.
    let edited_source = doc.locus_strokes(17, 96).unwrap();
    assert_eq!(edited_source.len(), 96);
    assert!(
        edited_source
            .iter()
            .any(|(start, end, _)| start.distance(*end) > 1.)
    );
    let changed = bounds(&doc.mapped_collection_strokes(19, 96).unwrap());
    for (actual, expected) in [changed.0, changed.1, changed.2, changed.3]
        .into_iter()
        .zip([429.799, 109.246, 831.208, 462.945])
    {
        assert!((actual - expected).abs() < 0.005);
    }
    let edited = doc.encode().unwrap();
    let mut edited_hidden = doc.clone();
    edited_hidden.selected.insert(19);
    edited_hidden.set_selected_visibility(false);
    assert_eq!(
        edited_hidden.encode().unwrap(),
        include_bytes!(
            "../../../../../tests/fixtures/scoped_point_locus70_edited_hide_binding_native_saved.gsp"
        )
    );
    assert_eq!(edited_hidden.locus_strokes(17, 96), Some(edited_source));
    assert!(!edited_hidden.is_displayed(19));
    let reopened = Document::from_bytes(&edited, None).unwrap();
    assert_eq!(reopened.encode().unwrap(), edited);
    assert_eq!(
        reopened.mapped_collection_strokes(19, 96),
        doc.mapped_collection_strokes(19, 96)
    );
}

#[test]
fn public_point_locus_collection_authors_three_independent_live_traces() {
    let mut doc = Document::new();
    let start = doc.add(Kind::Free(Point { x: 100., y: 100. }));
    let end = doc.add(Kind::Free(Point { x: 200., y: 100. }));
    let path = doc.add(Kind::Segment(start, end));
    let driver = doc.add(Kind::PointOnSegment(path, 0.25));
    let zero = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let plus = doc.add(Kind::Free(Point { x: 20., y: 0. }));
    let seed = doc.add(Kind::Free(Point { x: 0., y: 100. }));
    let target = doc.add(Kind::VectorTranslatedPoint(driver, zero, seed));
    let locus = doc.construct_locus(driver, target).unwrap();
    let next = doc.add(Kind::VectorTranslatedPoint(seed, zero, plus));
    let root = doc
        .add_mapped_point_iteration_with_bindings(
            &[seed],
            &[vec![next]],
            &[target, locus, next],
            &[locus],
            IterationDepth::Fixed(3),
        )
        .unwrap();
    let binding = root + 1;
    let original = doc.mapped_collection_strokes(binding, 64).unwrap();
    assert_eq!(original.len(), 192);
    for (generation, (left, right)) in [(120., 220.), (140., 240.), (160., 260.)]
        .into_iter()
        .enumerate()
    {
        let row = &original[generation * 64..(generation + 1) * 64];
        assert_eq!(row.first().unwrap().start, Point { x: left, y: 200. });
        assert_eq!(row.last().unwrap().end, Point { x: right, y: 200. });
    }
    doc.verify_roundtrip().unwrap();
    assert!(doc.move_point(end, Point { x: 240., y: 100. }));
    let edited = doc.mapped_collection_strokes(binding, 64).unwrap();
    assert_eq!(edited.len(), 192);
    for (generation, (left, right)) in [(120., 260.), (140., 280.), (160., 300.)]
        .into_iter()
        .enumerate()
    {
        let row = &edited[generation * 64..(generation + 1) * 64];
        assert_eq!(row.first().unwrap().start, Point { x: left, y: 200. });
        assert_eq!(row.last().unwrap().end, Point { x: right, y: 200. });
    }
    let bytes = doc.encode().unwrap();
    let reopened = Document::from_bytes(&bytes, None).unwrap();
    assert_eq!(
        reopened.mapped_collection_strokes(binding, 64),
        Some(edited)
    );
    assert_eq!(reopened.encode().unwrap(), bytes);
}
