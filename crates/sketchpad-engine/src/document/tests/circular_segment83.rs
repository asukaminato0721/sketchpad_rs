use super::*;

#[test]
fn native_circular_segment_closes_the_live_arc_with_its_chord() {
    for (bytes, start_x, end, visible) in [
        (
            include_bytes!("../../../../../tests/fixtures/circular_segment83_large_original_native_saved.gsp").as_slice(),
            438.,
            Point { x: 301.023763, y: 154.415977 },
            true,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/circular_segment83_large_hide6_synthetic_native_saved.gsp").as_slice(),
            438.,
            Point { x: 301.023763, y: 154.415977 },
            false,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/circular_segment83_large_end438to478_synthetic_native_saved.gsp").as_slice(),
            478.,
            Point { x: 280.343, y: 120.972 },
            true,
        ),
        (
            include_bytes!("../../../../../tests/fixtures/circular_segment83_large_end438to478_hide6_synthetic_native_saved.gsp").as_slice(),
            478.,
            Point { x: 280.343, y: 120.972 },
            false,
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(doc.objects[5].kind, Kind::CircularSegmentBoundary(5));
        assert_eq!(doc.objects[5].interior_order, Some(0));
        assert!(matches!(doc.objects[5].visual_style, Some(crate::style_records::VisualStyle::Polygon { .. })));
        assert_eq!(doc.is_displayed(6), visible);
        assert!(doc.objects[5].kind.is_filled_region());
        let outline = doc.filled_region_points(6).unwrap();
        assert_eq!(outline, doc.arc_points(5, 192).unwrap());
        assert!((outline[0].x - start_x).abs() < 1e-8);
        assert!((outline[0].y - 212.).abs() < 1e-8);
        assert!(outline.last().unwrap().distance(end) < 0.1);
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn circular_segment_rejects_unproved_marker_layout() {
    let original = include_bytes!(
        "../../../../../tests/fixtures/circular_segment83_large_original_native_saved.gsp"
    );
    let doc = Document::from_bytes(original, None).unwrap();
    let header = doc.objects[5].header_offset.unwrap();
    let end = records(original)
        .unwrap()
        .into_iter()
        .find(|record| record.kind == 0x07d7 && record.start > header)
        .unwrap()
        .start;
    let mut marked = original.to_vec();
    marked.splice(end..end, [0, 0, 0, 0, 0x2e, 0x23, 0, 0]);
    let guarded = Document::from_bytes(&marked, None).unwrap();
    assert_eq!(guarded.objects[5].kind, Kind::Unsupported(83));
    assert_eq!(guarded.encode().unwrap(), marked);
}

#[test]
fn circular_segment_writer_keeps_arc_reference_and_drawing_order_on_pages() {
    for page in [None, Some(0), Some(1)] {
        let mut doc = if let Some(page) = page {
            let mut doc = Document::from_bytes(
                include_bytes!("../../../../../tests/fixtures/two_page_blank_native.gsp"),
                None,
            )
            .unwrap();
            doc.active_page = page;
            doc
        } else {
            Document::new()
        };
        let center = doc.add(Kind::Free(Point { x: 300., y: 300. }));
        let start = doc.add(Kind::Free(Point { x: 380., y: 300. }));
        let end = doc.add(Kind::Free(Point { x: 300., y: 220. }));
        let arc = doc.add(Kind::CenterArc(center, start, end));
        let region = doc.add(Kind::CircularSegmentBoundary(arc));
        assert_eq!(doc.objects[region - 1].interior_order, Some(0));
        assert!(doc.filled_region_points(region).is_some());
        let bytes = doc.encode().unwrap();
        let saved = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(
            saved.objects[region - 1].kind,
            Kind::CircularSegmentBoundary(arc)
        );
        assert_eq!(saved.objects[region - 1].interior_order, Some(0));
        assert_eq!(saved.encode().unwrap(), bytes);
    }
}
