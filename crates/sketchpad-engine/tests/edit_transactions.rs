use sketchpad_engine::document::{Document, EvaluationCache, Kind, Point};

fn point(x: f64) -> Kind {
    Kind::Free(Point { x, y: 0. })
}

#[test]
fn public_edits_preserve_identity_and_update_history_and_evaluation() {
    let mut doc = Document::new();
    let a = doc.add(point(10.));
    let b = doc.add(point(30.));
    doc.add(Kind::Segment(a, b));
    let middle = doc.add(Kind::Midpoint(a, b));
    let mut doc = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    let mut cache = EvaluationCache::default();
    assert_eq!(cache.evaluate(&doc).position(middle).unwrap().x, 20.);
    doc.dirty = false;
    let history = doc.undo_len();
    let bytes = doc.encode().unwrap();
    assert!(doc.edit_object(a, |object| object.id = 99).is_err());
    assert!(doc.edit_object(0, |_| {}).is_err());
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(doc.undo_len(), history);
    assert!(!doc.dirty);

    doc.edit_object(a, |object| object.kind = point(20.))
        .unwrap();
    assert!(doc.dirty);
    assert_eq!(doc.undo_len(), history + 1);
    assert_eq!(cache.evaluate(&doc).position(middle).unwrap().x, 25.);
    doc.undo();
    assert_eq!(doc.encode().unwrap(), bytes);
    assert_eq!(cache.evaluate(&doc).position(middle).unwrap().x, 20.);
    doc.redo();
    assert_eq!(doc.position(a).unwrap().x, 20.);

    doc.dirty = false;
    let history = doc.undo_len();
    doc.edit_object(a, |_| {}).unwrap();
    assert!(!doc.dirty);
    assert_eq!(doc.undo_len(), history);
}

#[test]
fn nested_edits_commit_once_and_failed_edits_preserve_redo() {
    let mut doc = Document::new();
    let a = doc.add(point(10.));
    let b = doc.add(point(30.));
    let mut doc = Document::from_bytes(&doc.encode().unwrap(), None).unwrap();
    let before = doc.encode().unwrap();
    let history = doc.undo_len();
    doc.transaction(|doc| {
        doc.add(Kind::Segment(a, b));
        doc.transaction(|doc| {
            doc.add(Kind::Midpoint(a, b));
            doc.edit_object(a, |object| object.visible = false)
        })
    })
    .unwrap();
    assert_eq!(doc.undo_len(), history + 1);
    let after = doc.encode().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), before);
    doc.dirty = false;
    doc.selected.insert(a);
    let failure: Result<(), String> = doc.transaction(|doc| {
        doc.add(point(50.));
        doc.selected.clear();
        Err("Reject the operation".into())
    });
    assert!(failure.is_err());
    assert_eq!(doc.encode().unwrap(), before);
    assert_eq!(doc.selected, [a].into());
    assert!(!doc.dirty);
    assert_eq!(doc.undo_len(), history);
    assert_eq!(doc.redo_len(), 1);
    doc.redo();
    assert_eq!(doc.encode().unwrap(), after);
}

#[test]
fn unwinding_an_edit_preserves_the_original_document() {
    let mut doc = Document::new();
    doc.add(point(10.));
    let before = doc.encode().unwrap();
    let history = doc.undo_len();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _: Result<(), String> = doc.transaction(|doc| {
            doc.add(point(20.));
            panic!("Interrupted edit");
        });
    }));
    assert!(result.is_err());
    assert_eq!(doc.encode().unwrap(), before);
    assert_eq!(doc.undo_len(), history);
    doc.add(point(30.));
    assert_eq!(doc.undo_len(), history + 1);
}
