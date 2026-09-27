use super::*;
#[test]
fn round_intersection_command_supports_arc_circle_and_two_linear_arcs_in_both_orders() {
    for state in ["arc_circle_upper", "linear_both", "both_linear"] {
        let bytes = std::fs::read(format!(
            "tests/fixtures/circle_arc57_{state}_native_saved.gsp"
        ))
        .unwrap();
        let original = Document::from_bytes(&bytes, None).unwrap();
        let (a, b) = original
            .objects
            .iter()
            .find_map(|o| match o.kind {
                Kind::CircleCircleIntersection(a, b, _) => Some((a, b)),
                _ => None,
            })
            .unwrap();
        for order in [[a, b], [b, a]] {
            let mut e = Editor::new(original.clone());
            e.selection_order = order.to_vec();
            e.doc.selected = order.into_iter().collect();
            let before = e.doc.objects.len();
            e.intersection();
            assert_eq!(e.doc.objects.len(), before + 2, "{state}");
            assert_eq!(e.selection_order, vec![before + 1, before + 2]);
            for id in [before + 1, before + 2] {
                assert!(
                    matches!(e.doc.objects[id-1].kind,Kind::CircleCircleIntersection(x,y,_) if [x,y]==order)
                );
                assert!(e.doc.position(id).is_some(), "{state}");
            }
            e.doc.verify_roundtrip().unwrap();
            e.doc.undo();
            e.doc.undo();
            assert_eq!(e.doc.objects.len(), before);
        }
    }
}
