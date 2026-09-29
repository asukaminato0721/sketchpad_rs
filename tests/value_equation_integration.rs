use sketchpad_rs::document::{Document, Kind, Point};
use sketchpad_rs::value_equation_records::ValueEquationKind;

#[test]
fn original_equation_variants_decode_and_update_after_drag() {
    let cases: &[(&[u8], ValueEquationKind, usize, &str)] = &[
        (
            include_bytes!("fixtures/value_equation_generated_point.gsp"),
            ValueEquationKind::PointValue,
            6,
            "0.50",
        ),
        (
            include_bytes!("fixtures/value_equation_generated_line.gsp"),
            ValueEquationKind::LineEquation,
            11,
            "y =",
        ),
        (
            include_bytes!("fixtures/value_equation_generated_circle.gsp"),
            ValueEquationKind::CircleEquation,
            11,
            "²",
        ),
    ];
    for &(bytes, kind, id, expected) in cases {
        let doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        assert!(
            matches!(doc.objects[id - 1].kind, Kind::ValueEquation { kind: decoded, .. } if decoded == kind)
        );
        assert!(doc.value_equation_text(id).unwrap().contains(expected));
        assert_eq!(doc.encode().unwrap(), bytes);
    }
}

#[test]
fn original_circle_point_value_uses_normalized_turn() {
    let bytes = include_bytes!("fixtures/value_equation_circle_point_value_original.gsp");
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert!(matches!(
        doc.objects[4].kind,
        Kind::ValueEquation {
            kind: ValueEquationKind::PointValue,
            first: 4,
            second: 3
        }
    ));
    assert_eq!(
        doc.value_equation_text(5).as_deref(),
        Some("Value of Point = 0.38")
    );
    assert_eq!(doc.encode().unwrap(), bytes);
}

#[test]
fn port_writes_live_point_line_and_circle_results() {
    type EquationCase = (&'static [u8], ValueEquationKind, usize, usize, usize, Point);
    let cases: &[EquationCase] = &[
        (
            include_bytes!("fixtures/value_equation_generated_point_base.gsp"),
            ValueEquationKind::PointValue,
            5,
            4,
            5,
            Point { x: 220., y: 120. },
        ),
        (
            include_bytes!("fixtures/value_equation_generated_line_base.gsp"),
            ValueEquationKind::LineEquation,
            4,
            10,
            3,
            Point { x: 270., y: 150. },
        ),
        (
            include_bytes!("fixtures/value_equation_generated_circle_base.gsp"),
            ValueEquationKind::CircleEquation,
            4,
            10,
            3,
            Point { x: 240., y: 120. },
        ),
    ];
    for &(bytes, kind, first, second, moved, position) in cases {
        let mut doc = Document::from_bytes(bytes, None).unwrap();
        assert!(!doc.partial_view);
        let id = doc.add(Kind::ValueEquation {
            kind,
            first,
            second,
        });
        let before = doc.value_equation_text(id).unwrap();
        let encoded = doc.encode().unwrap();
        let mut reopened = Document::from_bytes(&encoded, None).unwrap();
        assert!(!reopened.partial_view);
        assert_eq!(reopened.value_equation_text(id).unwrap(), before);
        reopened.checkpoint();
        assert!(reopened.move_point(moved, position));
        let after = reopened.value_equation_text(id).unwrap();
        assert_ne!(after, before);
        let edited = reopened.encode().unwrap();
        assert_eq!(
            Document::from_bytes(&edited, None)
                .unwrap()
                .value_equation_text(id)
                .unwrap(),
            after
        );
    }
}
