use super::*;

fn point_value(doc: &Document, id: usize) -> f64 {
    match doc.value_equation_value(id) {
        Some(EquationValue::PointValue(value)) => value,
        other => panic!("Expected defined polygon point value, got {other:?}"),
    }
}

#[test]
fn native_polygon_point_values_project_independent_points_onto_live_boundary() {
    for (name, expected) in [
        ("vertex", 1. / 3.),
        ("edge", 5. / 12.),
        ("exterior", 5. / 12.),
        ("interior", 1. / 6.),
        ("corner_outside", 1. / 3.),
    ] {
        let source = fs::read(format!(
            "tests/fixtures/polygon_point_values49_{name}_native_verified.gsp"
        ))
        .unwrap();
        let mut doc = Document::from_bytes(&source, None).unwrap();
        assert!(!doc.partial_view, "{name}");
        assert!(matches!(
            doc.objects[6].kind,
            Kind::ValueEquation {
                kind: ValueEquationKind::PointValue,
                first: 5,
                second: 6
            }
        ));
        assert!((point_value(&doc, 7) - expected).abs() < 1e-12, "{name}");
        assert_eq!(doc.encode().unwrap(), source);
        doc.checkpoint();
        assert!(doc.move_point(5, Point { x: 200., y: 140. }));
        assert!((point_value(&doc, 7) - 1. / 6.).abs() < 1e-12);
        doc.checkpoint();
        assert!(doc.move_point(2, Point { x: 500., y: 100. }));
        let expected = 100. / (700. + 50000f64.sqrt());
        assert!((point_value(&doc, 7) - expected).abs() < 1e-12);
        doc.verify_roundtrip().unwrap();
        doc.undo();
        assert!((point_value(&doc, 7) - 1. / 6.).abs() < 1e-12);
        doc.redo();
        assert!((point_value(&doc, 7) - expected).abs() < 1e-12);
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn polygon_point_value_drives_calculation_and_plotted_point_after_source_edits() {
    let source =
        fs::read("tests/fixtures/polygon_point_values49_edge_native_verified.gsp").unwrap();
    let mut doc = Document::from_bytes(&source, None).unwrap();
    let calculation = doc.add_parameter_calculation(7, "2*v", (10, 45)).unwrap();
    let plotted = doc.plot_parameter_on_path(calculation, Some(6)).unwrap();
    let before = doc.encode().unwrap();
    assert!((doc.numeric_value(calculation).unwrap() - 5. / 6.).abs() < 1e-12);
    doc.checkpoint();
    assert!(doc.move_point(5, Point { x: 200., y: 140. }));
    assert!((doc.numeric_value(calculation).unwrap() - 1. / 3.).abs() < 1e-12);
    assert!(
        doc.position(plotted)
            .unwrap()
            .distance(Point { x: 300., y: 100. })
            < 1e-9
    );
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), before);
    doc.redo();
    doc.verify_roundtrip().unwrap();
}

#[test]
fn polygon_point_values_accept_vertices_and_literal_paths_and_preserve_edge_metadata() {
    let mut doc = Document::new();
    let vertices = [
        Point { x: 100., y: 100. },
        Point { x: 300., y: 100. },
        Point { x: 300., y: 200. },
        Point { x: 100., y: 200. },
    ]
    .map(|point| doc.add(Kind::Free(point)));
    let polygon = doc.add(Kind::Polygon(vertices.to_vec()));
    let literal = doc.plot_literal_on_path(0.25, Some(polygon)).unwrap();
    let on_edge = doc.add(Kind::PointOnPolygon(polygon, 3, 1.));
    for (first, expected) in [(vertices[1], 1. / 3.), (literal, 0.25), (on_edge, 1.)] {
        let measurement = doc.add(Kind::ValueEquation {
            kind: ValueEquationKind::PointValue,
            first,
            second: polygon,
        });
        assert!((point_value(&doc, measurement) - expected).abs() < 1e-12);
    }
    doc.verify_roundtrip().unwrap();
}

#[test]
fn original_polygon_point_values_keep_vertex_and_literal_sources_and_opaque_bytes() {
    let source = include_bytes!(
        "../../../tests/fixtures/polygon_point_values49_corpus_vertices_original.gsp"
    );
    let doc = Document::from_bytes(source, None).unwrap();
    let lengths = [1105f64.sqrt(), 1109f64.sqrt(), 765f64.sqrt(), 985f64.sqrt()];
    let perimeter: f64 = lengths.iter().sum();
    for (offset, expected) in [
        0.,
        lengths[0],
        lengths[..2].iter().sum(),
        lengths[..3].iter().sum(),
    ]
    .into_iter()
    .enumerate()
    {
        let id = 10 + offset;
        assert!(
            matches!(doc.objects[id-1].kind, Kind::ValueEquation {kind:ValueEquationKind::PointValue, first, second:9} if first==5+offset)
        );
        assert!((point_value(&doc, id) - expected / perimeter).abs() < 1e-12);
    }
    assert_eq!(doc.encode().unwrap(), source);
    let source = include_bytes!(
        "../../../tests/fixtures/polygon_point_values49_corpus_literal_original.gsp"
    );
    let doc = Document::from_bytes(source, None).unwrap();
    assert!(matches!(
        doc.objects[20].kind,
        Kind::PathLiteralPoint(19, 0.)
    ));
    assert!(matches!(
        doc.objects[21].kind,
        Kind::ValueEquation {
            kind: ValueEquationKind::PointValue,
            first: 21,
            second: 19
        }
    ));
    assert_eq!(point_value(&doc, 22), 0.);
    assert_eq!(doc.encode().unwrap(), source);
}
