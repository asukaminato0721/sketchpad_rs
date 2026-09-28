use super::*;
use crate::document::EmbeddedToolInputKind;
use std::collections::BTreeSet;

#[test]
fn constrained_tool_outputs_choose_placement_at_invocation_on_constructed_hosts() {
    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: 200., y: 180. }));
    let b = document.add(Kind::Free(Point { x: 300., y: 180. }));
    let host = document.add(Kind::Segment(a, b));
    let point = document.add(Kind::PointOnSegment(host, 0.5));
    document.selected.extend([a, b, host, point]);
    let before = document.encode().unwrap();
    document
        .create_embedded_tool("Point from segment endpoints")
        .unwrap();
    let prototype = document.embedded_tool_document(0).unwrap();
    assert!(!prototype.partial_view);
    assert_eq!(prototype.objects[2].kind, Kind::Segment(2, 1));
    assert_eq!(prototype.objects[3].kind, Kind::ToolPointOnPath(3));
    let original_placement = document.objects[point - 1].kind.clone();
    let encoded = document.encode().unwrap();
    assert_eq!(
        encoded,
        include_bytes!(
            "../../../../../tests/fixtures/embedded_tool_constrained_point_created_writer_native_verified.gsp"
        )
    );
    document.undo();
    assert_eq!(document.encode().unwrap(), before);
    document.redo();
    assert_eq!(document.encode().unwrap(), encoded);
    assert_eq!(document.objects[point - 1].kind, original_placement);
    let outputs = document.invoke_embedded_tool(0, &[a, b]).unwrap();
    let Kind::PointOnSegment(result_host, fraction) = document.objects[outputs[1] - 1].kind else {
        panic!("Expected a materialized constrained point");
    };
    assert_eq!(result_host, outputs[0]);
    assert!((0.0..1.0).contains(&fraction));
    document.verify_roundtrip().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_TOOL_PATH_FIXTURES").is_some() {
        fs::write(
            "/tmp/embedded_tool_constrained_point_created_port.gsp",
            encoded,
        )
        .unwrap();
    }
}

#[test]
fn create_registered_point_construction_tool_preserves_drawing_and_structural_history() {
    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: 200., y: 180. }));
    let b = document.add(Kind::Free(Point { x: 260., y: 180. }));
    let segment = document.add(Kind::Segment(a, b));
    let midpoint = document.add(Kind::Midpoint(a, b));
    document.selected.extend([a, b, segment, midpoint]);
    let before = document.encode().unwrap();
    let history = document.undo.len();
    assert_eq!(
        document
            .create_embedded_tool("Segment and midpoint")
            .unwrap(),
        0
    );
    assert_eq!(document.undo.len(), history + 1);
    let summary = document.embedded_tools().unwrap();
    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].name.as_deref(), Some("Segment and midpoint"));
    assert_eq!(summary[0].object_codes, vec![0, 0, 2, 1]);
    assert_eq!(summary[0].input_ids.as_deref(), Some(&[2, 1][..]));
    assert_eq!(document.position(a), Some(Point { x: 200., y: 180. }));
    assert_eq!(
        document.position(midpoint),
        Some(Point { x: 230., y: 180. })
    );
    let encoded = document.encode().unwrap();
    assert_eq!(
        encoded,
        include_bytes!(
            "../../../../../tests/fixtures/embedded_tool_simple_created_writer_native_verified.gsp"
        )
    );
    assert_eq!(u32::from_le_bytes(encoded[16..20].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(encoded[36..40].try_into().unwrap()), 3);
    let entries = records(&encoded).unwrap();
    let tool_start = entries
        .iter()
        .position(|record| record.kind == 0x03ea)
        .unwrap();
    let title = entries[tool_start + 1].data.clone();
    assert_eq!(&encoded[title], b"Segment and midpoint\0\0");
    assert_eq!(
        entries[entries.len() - 3..]
            .iter()
            .map(|record| record.kind)
            .collect::<Vec<_>>(),
        vec![0x03eb, 0x0387, 0x0385]
    );
    let prototype = document.embedded_tool_document(0).unwrap();
    assert_eq!(prototype.objects[2].kind, Kind::Segment(2, 1));
    assert_eq!(prototype.objects[3].kind, Kind::Midpoint(2, 1));
    assert_eq!(prototype.objects[3].midpoint_host, Some(3));
    document.verify_roundtrip().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_TOOL_CREATION_FIXTURE").is_some() {
        fs::write("/tmp/embedded_tool_simple_created_port.gsp", &encoded).unwrap();
    }
    document.undo();
    assert!(document.embedded_tools().unwrap().is_empty());
    assert_eq!(document.encode().unwrap(), before);
    document.redo();
    assert_eq!(document.encode().unwrap(), encoded);
    let created = document.invoke_embedded_tool(0, &[b, a]).unwrap();
    assert_eq!(document.objects[created[0] - 1].kind, Kind::Segment(b, a));
    assert_eq!(
        document.position(created[1]),
        Some(Point { x: 230., y: 180. })
    );
    document.verify_roundtrip().unwrap();
}

#[test]
fn invalid_tool_creation_is_atomic_and_existing_tool_can_be_extended() {
    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: 100., y: 100. }));
    let before = document.encode().unwrap();
    document.selected.insert(a);
    let history = document.undo.len();
    for name in ["", "Only a point", "bad\0name"] {
        assert!(document.create_embedded_tool(name).is_err());
        assert_eq!(document.encode().unwrap(), before);
        assert_eq!(document.undo.len(), history);
        assert_eq!(document.selected, BTreeSet::from([a]));
    }
    let b = document.add(Kind::Free(Point { x: 140., y: 100. }));
    let segment = document.add(Kind::Segment(a, b));
    document.selected = BTreeSet::from([a, b, segment]);
    document.create_embedded_tool("First").unwrap();
    let first = document.embedded_tools().unwrap()[0].clone();
    assert_eq!(document.create_embedded_tool("第二个").unwrap(), 1);
    let summaries = document.embedded_tools().unwrap();
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0], first);
    assert_eq!(summaries[1].name.as_deref(), Some("第二个"));
    let bytes = document.encode().unwrap();
    assert_eq!(u32::from_le_bytes(bytes[16..20].try_into().unwrap()), 2);
    assert_eq!(u32::from_le_bytes(bytes[36..40].try_into().unwrap()), 4);
    document.verify_roundtrip().unwrap();
}

#[test]
fn create_path_given_prunes_unselected_geometry_and_unused_selected_endpoint() {
    for selected_endpoint in [false, true] {
        let mut document = Document::new();
        let a = document.add(Kind::Free(Point { x: 200., y: 180. }));
        let b = document.add(Kind::Free(Point { x: 300., y: 180. }));
        let host = document.add(Kind::Segment(a, b));
        let point = document.add(Kind::PointOnSegment(host, 0.5));
        document.selected.extend([host, point]);
        if selected_endpoint {
            document.selected.insert(a);
        }
        let before = document.encode().unwrap();
        let kinds: Vec<_> = document
            .objects
            .iter()
            .map(|object| object.kind.clone())
            .collect();
        let selected = document.selected.clone();
        let history = document.undo.len();
        document
            .create_embedded_tool("Point on selected path")
            .unwrap();
        assert_eq!(document.undo.len(), history + 1);
        assert_eq!(document.selected, selected);
        assert_eq!(
            document
                .objects
                .iter()
                .map(|object| object.kind.clone())
                .collect::<Vec<_>>(),
            kinds
        );
        let summary = &document.embedded_tools().unwrap()[0];
        assert_eq!(summary.object_codes, vec![0, 15]);
        assert_eq!(summary.input_ids.as_deref(), Some(&[1][..]));
        assert!(document.embedded_tool_inputs(0).unwrap()[0].is_path());
        let prototype = document.embedded_tool_document(0).unwrap();
        assert!(!prototype.partial_view);
        assert_eq!(prototype.objects.len(), 2);
        assert_eq!(
            prototype.objects[0].kind,
            Kind::ToolInput(crate::document::EmbeddedToolInputKind::Path)
        );
        assert_eq!(prototype.objects[1].kind, Kind::ToolPointOnPath(1));
        assert_eq!(prototype.position(1), None);
        assert_eq!(prototype.line_points(1), None);
        let created = document.encode().unwrap();
        assert_eq!(
            created,
            include_bytes!(
                "../../../../../tests/fixtures/embedded_tool_path_authored_segment_writer_native_verified.gsp"
            )
        );
        document.undo();
        assert_eq!(document.encode().unwrap(), before);
        document.redo();
        assert_eq!(document.encode().unwrap(), created);
        let outputs = document.invoke_embedded_tool(0, &[host]).unwrap();
        let Kind::PointOnSegment(bound, fraction) = document.objects[outputs[0] - 1].kind else {
            panic!("Expected a materialized path output")
        };
        assert_eq!(bound, host);
        assert!((0.0..1.0).contains(&fraction));
        assert!(document.move_point(b, Point { x: 320., y: 220. }));
        assert_eq!(
            document.position(outputs[0]),
            Some(Point {
                x: 200. + 120. * fraction,
                y: 180. + 40. * fraction
            })
        );
        document.verify_roundtrip().unwrap();
        if std::env::var_os("SKETCHPAD_WRITE_PATH_AUTHORING_FIXTURES").is_some()
            && !selected_endpoint
        {
            fs::write("/tmp/embedded_tool_path_authored_segment_port.gsp", created).unwrap();
            fs::write(
                "/tmp/embedded_tool_path_authored_segment_invoked_port.gsp",
                document.encode().unwrap(),
            )
            .unwrap();
        }
    }
}

#[test]
fn create_circle_path_given_accepts_live_straight_destination_and_retains_tool_namespace() {
    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: 200., y: 180. }));
    let b = document.add(Kind::Free(Point { x: 260., y: 180. }));
    let host = document.add(Kind::Circle(a, b));
    let point = document.add(Kind::PointOnCircle(host, 0., -1.));
    document.selected.extend([host, point]);
    document.create_embedded_tool("Point on a path").unwrap();
    let created = document.encode().unwrap();
    assert_eq!(
        created,
        include_bytes!(
            "../../../../../tests/fixtures/embedded_tool_path_authored_circle_writer_native_verified.gsp"
        )
    );
    assert!(document.embedded_tool_inputs(0).unwrap()[0].is_path());
    let segment = document.add(Kind::Segment(a, b));
    let output = document.invoke_embedded_tool(0, &[segment]).unwrap()[0];
    assert!(
        matches!(document.objects[output - 1].kind, Kind::PointOnSegment(id, _) if id == segment)
    );
    assert_eq!(
        document.embedded_tool_document(0).unwrap().objects[1].kind,
        Kind::ToolPointOnPath(1)
    );
    document.verify_roundtrip().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_PATH_AUTHORING_FIXTURES").is_some() {
        fs::write("/tmp/embedded_tool_path_authored_circle_port.gsp", created).unwrap();
        fs::write(
            "/tmp/embedded_tool_path_authored_circle_invoked_port.gsp",
            document.encode().unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn singleton_and_mixed_template_creation_fail_atomically() {
    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: 200., y: 180. }));
    let b = document.add(Kind::Free(Point { x: 300., y: 180. }));
    let host = document.add(Kind::Segment(a, b));
    let point = document.add(Kind::PointOnSegment(host, 0.5));
    let midpoint = document.add(Kind::Midpoint(a, b));
    for selected in [
        BTreeSet::from([host]),
        BTreeSet::from([point]),
        BTreeSet::from([host, point, midpoint]),
    ] {
        document.selected = selected.clone();
        let before = document.encode().unwrap();
        let history = document.undo.len();
        assert!(
            document
                .create_embedded_tool("Unsupported definition")
                .is_err()
        );
        assert_eq!(document.selected, selected);
        assert_eq!(document.undo.len(), history);
        assert_eq!(document.encode().unwrap(), before);
    }
}

#[test]
fn create_finite_segment_given_preserves_abstract_midpoint_recipe() {
    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: 200., y: 180. }));
    let b = document.add(Kind::Free(Point { x: 300., y: 180. }));
    let host = document.add(Kind::Segment(a, b));
    let midpoint = document.add(Kind::Midpoint(a, b));
    document.selected.extend([host, midpoint]);
    document
        .create_embedded_tool("Midpoint of selected segment")
        .unwrap();
    let prototype = document.embedded_tool_document(0).unwrap();
    assert_eq!(
        prototype.objects[0].kind,
        Kind::ToolInput(crate::document::EmbeddedToolInputKind::Segment)
    );
    assert_eq!(prototype.objects[1].kind, Kind::SegmentMidpoint(1));
    assert_eq!(prototype.line_points(1), None);
    let created = document.encode().unwrap();
    assert_eq!(
        created,
        include_bytes!(
            "../../../../../tests/fixtures/embedded_tool_segment_authored_midpoint_writer_native_verified.gsp"
        )
    );
    let result = document.invoke_embedded_tool(0, &[host]).unwrap()[0];
    assert_eq!(document.position(result), Some(Point { x: 250., y: 180. }));
    assert_eq!(document.objects[result - 1].midpoint_host, Some(host));
    let ray = document.add(Kind::Ray(a, b));
    let before = document.encode().unwrap();
    assert!(document.invoke_embedded_tool(0, &[ray]).is_err());
    assert_eq!(document.encode().unwrap(), before);
    document.verify_roundtrip().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_PATH_AUTHORING_FIXTURES").is_some() {
        fs::write(
            "/tmp/embedded_tool_segment_authored_midpoint_port.gsp",
            created,
        )
        .unwrap();
        fs::write(
            "/tmp/embedded_tool_segment_authored_midpoint_invoked_port.gsp",
            before,
        )
        .unwrap();
    }
}

#[test]
fn selected_endpoint_givens_include_unselected_midpoint_segment_as_hidden_helper() {
    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: 200., y: 180. }));
    let b = document.add(Kind::Free(Point { x: 300., y: 180. }));
    let host = document.add(Kind::Segment(a, b));
    let midpoint = document.add(Kind::Midpoint(a, b));
    document.selected.extend([a, b, midpoint]);
    document
        .create_embedded_tool("Midpoint from given endpoints")
        .unwrap();
    let prototype = document.embedded_tool_document(0).unwrap();
    assert_eq!(prototype.objects.len(), 4);
    assert_eq!(prototype.objects[2].kind, Kind::Segment(2, 1));
    assert!(prototype.objects[2].presentation_suppressed);
    assert!(!prototype.is_displayed(3));
    assert_eq!(prototype.objects[3].midpoint_host, Some(3));
    assert!(document.objects[host - 1].visible);
    let created = document.encode().unwrap();
    assert_eq!(
        created,
        include_bytes!(
            "../../../../../tests/fixtures/embedded_tool_hidden_helper_authored_writer_native_verified.gsp"
        )
    );
    let target_a = document.add(Kind::Free(Point { x: 400., y: 350. }));
    let target_b = document.add(Kind::Free(Point { x: 500., y: 350. }));
    let outputs = document
        .invoke_embedded_tool(0, &[target_a, target_b])
        .unwrap();
    assert!(document.objects[outputs[0] - 1].presentation_suppressed);
    assert!(!document.is_displayed(outputs[0]));
    assert!(!document.selected.contains(&outputs[0]));
    assert!(document.selected.contains(&outputs[1]));
    assert_eq!(
        document.position(outputs[1]),
        Some(Point { x: 450., y: 350. })
    );
    document.verify_roundtrip().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_PATH_AUTHORING_FIXTURES").is_some() {
        fs::write(
            "/tmp/embedded_tool_hidden_helper_authored_port.gsp",
            created,
        )
        .unwrap();
        fs::write(
            "/tmp/embedded_tool_hidden_helper_authored_invoked_port.gsp",
            document.encode().unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn point_givens_construct_selected_path_through_unselected_geometry_helper() {
    let mut document = Document::new();
    let a = document.add(Kind::Free(Point { x: 200., y: 180. }));
    let b = document.add(Kind::Free(Point { x: 300., y: 180. }));
    let helper = document.add(Kind::Segment(a, b));
    let circle = document.add(Kind::CircleBySegment(a, helper));
    let point = document.add(Kind::PointOnCircle(circle, 0., 1.));
    document.selected.extend([a, b, circle, point]);
    document
        .create_embedded_tool("Circle from selected point givens")
        .unwrap();
    assert!(
        document
            .embedded_tool_inputs(0)
            .unwrap()
            .iter()
            .all(|input| input.is_point())
    );
    let prototype = document.embedded_tool_document(0).unwrap();
    assert_eq!(prototype.objects[2].kind, Kind::Segment(2, 1));
    assert!(prototype.objects[2].presentation_suppressed);
    assert_eq!(prototype.objects[3].kind, Kind::CircleBySegment(2, 3));
    assert_eq!(prototype.objects[4].kind, Kind::ToolPointOnPath(4));
    let outputs = document.invoke_embedded_tool(0, &[a, b]).unwrap();
    assert!(!document.is_displayed(outputs[0]));
    assert!(
        matches!(document.objects[outputs[2] - 1].kind, Kind::PointOnCircle(host, _, _) if host == outputs[1])
    );
    document.verify_roundtrip().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_PATH_AUTHORING_FIXTURES").is_some() {
        fs::write(
            "/tmp/embedded_tool_transitive_geometry_authored_invoked_port.gsp",
            document.encode().unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn native_executes_port_authored_path_and_segment_given_definitions() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/embedded_tool_path_authored_native_invocation_verified.gsp"
    );
    let mut path = Document::from_bytes(bytes, None).unwrap();
    assert!(!path.partial_view);
    assert_eq!(path.objects.len(), 5);
    assert!(path.embedded_tool_inputs(0).unwrap()[0].is_path());
    let Kind::PointOnSegment(host, fraction) = path.objects[4].kind else {
        panic!("Expected the Native-created output")
    };
    assert_eq!(host, 3);
    assert!((0.0..1.0).contains(&fraction));
    assert_eq!(path.encode().unwrap(), bytes);
    assert!(path.move_point(2, Point { x: 320., y: 220. }));
    assert_eq!(
        path.position(5),
        Some(Point {
            x: 200. + 120. * fraction,
            y: 180. + 40. * fraction
        })
    );
    path.verify_roundtrip().unwrap();
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/embedded_tool_segment_authored_native_invocation_verified.gsp"
    );
    let mut segment = Document::from_bytes(bytes, None).unwrap();
    assert!(!segment.partial_view);
    assert_eq!(segment.objects.len(), 8);
    assert_eq!(
        segment.embedded_tool_inputs(0).unwrap()[0].kind(),
        Some(crate::document::EmbeddedToolInputKind::Segment)
    );
    assert_eq!(segment.objects[7].midpoint_host, Some(7));
    assert_eq!(segment.position(8), Some(Point { x: 450., y: 350. }));
    assert_eq!(segment.encode().unwrap(), bytes);
    assert!(segment.move_point(6, Point { x: 540., y: 370. }));
    assert_eq!(segment.position(8), Some(Point { x: 470., y: 360. }));
    segment.verify_roundtrip().unwrap();
}

#[test]
fn create_scalar_numeric_given_preserves_source_and_binds_live_values() {
    let mut document = Document::new();
    let given = document.add_named_parameter("t", 2., (20, 20)).unwrap();
    let result = document
        .add_parameter_calculation(given, "t+1", (20, 60))
        .unwrap();
    document.selected.extend([given, result]);
    let before = document.encode().unwrap();
    let selected = document.selected.clone();
    let history = document.undo.len();
    assert_eq!(document.create_embedded_tool("Scalar plus one").unwrap(), 0);
    assert_eq!(document.undo.len(), history + 1);
    assert_eq!(document.selected, selected);
    assert_eq!(document.numeric_value(given), Some(2.));
    assert_eq!(document.numeric_value(result), Some(3.));
    let prototype = document.embedded_tool_document(0).unwrap();
    assert_eq!(
        prototype.objects[0].kind,
        Kind::ToolInput(EmbeddedToolInputKind::Numeric)
    );
    assert!(
        matches!(&prototype.objects[1].kind, Kind::ParameterCalculation(1, group)
        if group.sources() == vec![1] && group.expression("t") == "(t + 1)")
    );
    let created = document.encode().unwrap();
    assert_eq!(
        created,
        include_bytes!(
            "../../../../../tests/fixtures/embedded_tool_numeric_authored_writer_native_verified.gsp"
        )
    );
    document.undo();
    assert_eq!(document.encode().unwrap(), before);
    document.redo();
    assert_eq!(document.encode().unwrap(), created);
    let input = document.add_named_parameter("n", 5., (200, 20)).unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_NUMERIC_AUTHORING_FIXTURES").is_some() {
        fs::write(
            "/tmp/embedded_tool_numeric_authored_native_input_port.gsp",
            document.encode().unwrap(),
        )
        .unwrap();
    }
    let outputs = document.invoke_embedded_tool(0, &[input]).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(document.numeric_value(outputs[0]), Some(6.));
    document.set_parameter_value(input, 7.).unwrap();
    assert_eq!(document.numeric_value(outputs[0]), Some(8.));
    document.verify_roundtrip().unwrap();
    assert_eq!(
        document.encode().unwrap(),
        include_bytes!(
            "../../../../../tests/fixtures/embedded_tool_numeric_authored_invoked_writer_native_verified.gsp"
        )
    );
    if std::env::var_os("SKETCHPAD_WRITE_NUMERIC_AUTHORING_FIXTURES").is_some() {
        fs::write("/tmp/embedded_tool_numeric_authored_port.gsp", created).unwrap();
        fs::write(
            "/tmp/embedded_tool_numeric_authored_invoked_port.gsp",
            document.encode().unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn numeric_authoring_rejects_unproved_categories_and_unselected_sources_atomically() {
    let mut document = Document::from_bytes(
        include_bytes!("../../../../../tests/fixtures/embedded_tool_scalar_on_area_native_verified.gsp"),
        None,
    )
    .unwrap();
    document.selected = BTreeSet::from([8, 9]);
    let before = document.encode().unwrap();
    let history = document.undo.len();
    assert!(document.create_embedded_tool("Unproved input").is_err());
    assert_eq!(document.encode().unwrap(), before);
    assert_eq!(document.undo.len(), history);
    assert_eq!(document.selected, BTreeSet::from([8, 9]));
    let mut scalar = Document::new();
    let a = scalar.add_named_parameter("t", 2., (20, 20)).unwrap();
    let b = scalar.add_named_parameter("n", 5., (200, 20)).unwrap();
    let output = scalar
        .add_parameter_calculation(a, "t+1", (20, 60))
        .unwrap();
    scalar.selected.extend([b, output]);
    let before = scalar.encode().unwrap();
    assert!(scalar.create_embedded_tool("Wrong source").is_err());
    assert_eq!(scalar.encode().unwrap(), before);
}

#[test]
fn create_measured_numeric_given_excludes_geometry_and_follows_live_measurement() {
    let mut document = Document::from_bytes(
        include_bytes!(
            "../../../../../tests/fixtures/embedded_tool_numeric_regular_measure_native_verified.gsp"
        ),
        None,
    )
    .unwrap();
    document.selected = BTreeSet::from([6, 7]);
    let before = document.encode().unwrap();
    let history = document.undo.len();
    assert_eq!(
        document.create_embedded_tool("Measured plus one").unwrap(),
        1
    );
    assert_eq!(document.undo.len(), history + 1);
    assert_eq!(document.objects.len(), 7);
    let prototype = document.embedded_tool_document(1).unwrap();
    assert_eq!(prototype.objects.len(), 2);
    assert_eq!(
        prototype.objects[0].kind,
        Kind::ToolInput(EmbeddedToolInputKind::Numeric)
    );
    assert!(matches!(
        prototype.objects[1].kind,
        Kind::ParameterCalculation(1, _)
    ));
    let created = document.encode().unwrap();
    assert_eq!(
        created,
        include_bytes!(
            "../../../../../tests/fixtures/embedded_tool_measurement_authored_writer_native_verified.gsp"
        )
    );
    document.undo();
    assert_eq!(document.encode().unwrap(), before);
    document.redo();
    let output = document.invoke_embedded_tool(1, &[6]).unwrap()[0];
    assert!((document.numeric_value(output).unwrap() - 6.).abs() < 1e-10);
    let Kind::SegmentLength(host) = document.objects[5].kind else {
        panic!("Expected length");
    };
    let Kind::Segment(a, b) = document.objects[host - 1].kind else {
        panic!("Expected segment");
    };
    let point = document.position(a).unwrap();
    let end = document.position(b).unwrap();
    document.objects[b - 1].kind = Kind::Free(Point {
        x: point.x + 2. * (end.x - point.x),
        y: point.y + 2. * (end.y - point.y),
    });
    assert!((document.numeric_value(output).unwrap() - 11.).abs() < 1e-10);
    document.verify_roundtrip().unwrap();
    assert_eq!(
        document.encode().unwrap(),
        include_bytes!(
            "../../../../../tests/fixtures/embedded_tool_measurement_authored_invoked_writer_native_verified.gsp"
        )
    );
    if std::env::var_os("SKETCHPAD_WRITE_NUMERIC_AUTHORING_FIXTURES").is_some() {
        fs::write("/tmp/embedded_tool_measurement_authored_port.gsp", created).unwrap();
        fs::write(
            "/tmp/embedded_tool_measurement_authored_invoked_port.gsp",
            document.encode().unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn create_degree_numeric_given_uses_generic_category_and_live_degree_input() {
    let mut document = Document::from_bytes(
        include_bytes!(
            "../../../../../tests/fixtures/embedded_tool_numeric_regular_degree_native_verified.gsp"
        ),
        None,
    )
    .unwrap();
    document.selected = BTreeSet::from([3, 5]);
    let source = document.encode().unwrap();
    assert_eq!(document.create_embedded_tool("Degree plus one").unwrap(), 1);
    let prototype = document.embedded_tool_document(1).unwrap();
    assert_eq!(prototype.objects.len(), 2);
    assert_eq!(
        prototype.objects[0].kind,
        Kind::ToolInput(EmbeddedToolInputKind::Numeric)
    );
    let created = document.encode().unwrap();
    assert_eq!(
        created,
        include_bytes!(
            "../../../../../tests/fixtures/embedded_tool_degree_authored_writer_native_verified.gsp"
        )
    );
    let output = document.invoke_embedded_tool(1, &[3]).unwrap()[0];
    assert_eq!(document.numeric_value(output), Some(36.));
    assert_eq!(
        document.objects[output - 1].kind.numeric_unit(),
        Some(crate::table_records::NumericUnit::Scalar)
    );
    document.set_parameter_value(3, 45.).unwrap();
    assert_eq!(document.numeric_value(output), Some(46.));
    document.verify_roundtrip().unwrap();
    document.undo(); // parameter value
    document.undo(); // invocation
    document.undo(); // definition
    assert_eq!(document.encode().unwrap(), source);
    if std::env::var_os("SKETCHPAD_WRITE_NUMERIC_AUTHORING_FIXTURES").is_some() {
        fs::write("/tmp/embedded_tool_degree_authored_port.gsp", created).unwrap();
    }
}

#[test]
fn native_executes_port_authored_numeric_definition_and_preserves_live_source() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/embedded_tool_numeric_authored_native_invocation_verified.gsp"
    );
    let mut document = Document::from_bytes(bytes, None).unwrap();
    assert_eq!(document.objects.len(), 4);
    assert_eq!(
        document.embedded_tool_inputs(0).unwrap()[0].kind(),
        Some(EmbeddedToolInputKind::Numeric)
    );
    assert_eq!(document.numeric_value(1), Some(2.));
    assert_eq!(document.numeric_value(2), Some(3.));
    assert_eq!(document.numeric_value(3), Some(5.));
    assert_eq!(document.numeric_value(4), Some(6.));
    assert!(
        matches!(&document.objects[3].kind, Kind::ParameterCalculation(3, group)
        if group.sources() == vec![3] && group.expression("n") == "(n + 1)")
    );
    assert_eq!(document.encode().unwrap(), bytes);
    document.set_parameter_value(3, 9.).unwrap();
    assert_eq!(document.numeric_value(4), Some(10.));
    document.verify_roundtrip().unwrap();
}

#[test]
fn create_two_numeric_givens_preserves_binding_order_and_semantic_references() {
    let mut document = Document::new();
    let t = document.add_named_parameter("t", 2., (20, 20)).unwrap();
    let u = document.add_named_parameter("u", 3., (20, 60)).unwrap();
    let result = document
        .add_multi_calculation(&[t, u], "v1+v2", document.next_calculation_anchor())
        .unwrap();
    document.selected.extend([t, u, result]);
    let source = document.encode().unwrap();
    document.create_embedded_tool("Two numbers sum").unwrap();
    let inputs = document.embedded_tool_inputs(0).unwrap();
    assert_eq!(
        inputs
            .iter()
            .map(|input| input.local_id)
            .collect::<Vec<_>>(),
        vec![2, 1]
    );
    assert!(
        inputs
            .iter()
            .all(|input| input.kind() == Some(EmbeddedToolInputKind::Numeric))
    );
    let prototype = document.embedded_tool_document(0).unwrap();
    assert!(
        matches!(&prototype.objects[2].kind, Kind::ParameterCalculation(2, group) if group.sources() == vec![2, 1])
    );
    let created = document.encode().unwrap();
    document.undo();
    assert_eq!(document.encode().unwrap(), source);
    document.redo();
    let n = document.add_named_parameter("n", 5., (280, 20)).unwrap();
    let m = document.add_named_parameter("m", 7., (280, 80)).unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_NUMERIC_GRAPH_FIXTURES").is_some() {
        fs::write(
            "/tmp/numeric_graph_two_sources_native_input_port.gsp",
            document.encode().unwrap(),
        )
        .unwrap();
    }
    let output = document.invoke_embedded_tool(0, &[n, m]).unwrap()[0];
    assert_eq!(document.numeric_value(output), Some(12.));
    assert!(
        matches!(&document.objects[output - 1].kind, Kind::ParameterCalculation(first, group) if *first == n && group.sources() == vec![n as u32, m as u32])
    );
    document.set_parameter_value(n, 9.).unwrap();
    assert_eq!(document.numeric_value(output), Some(16.));
    document.verify_roundtrip().unwrap();
    if std::env::var_os("SKETCHPAD_WRITE_NUMERIC_GRAPH_FIXTURES").is_some() {
        fs::write("/tmp/numeric_graph_two_sources_authored_port.gsp", created).unwrap();
        fs::write(
            "/tmp/numeric_graph_two_sources_invoked_port.gsp",
            document.encode().unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn create_nested_numeric_graph_includes_suppressed_unselected_helpers() {
    for helper_selected in [true, false] {
        let mut document = Document::new();
        let t = document.add_named_parameter("t", 2., (20, 20)).unwrap();
        let helper = document
            .add_parameter_calculation(t, "t+1", document.next_calculation_anchor())
            .unwrap();
        let result = document
            .add_parameter_calculation(helper, "v*2", document.next_calculation_anchor())
            .unwrap();
        document.selected.extend([t, result]);
        if helper_selected {
            document.selected.insert(helper);
        }
        let source = document.encode().unwrap();
        document
            .create_embedded_tool(if helper_selected {
                "Nested numeric results"
            } else {
                "Nested hidden helper"
            })
            .unwrap();
        let prototype = document.embedded_tool_document(0).unwrap();
        assert_eq!(prototype.objects.len(), 3);
        assert_eq!(
            prototype.objects[1].presentation_suppressed,
            !helper_selected
        );
        assert_eq!(prototype.is_displayed(2), helper_selected);
        assert!(
            matches!(&prototype.objects[2].kind, Kind::ParameterCalculation(2, group) if group.sources() == vec![2])
        );
        let created = document.encode().unwrap();
        document.undo();
        assert_eq!(document.encode().unwrap(), source);
        document.redo();
        let input = document.add_named_parameter("n", 5., (280, 20)).unwrap();
        if std::env::var_os("SKETCHPAD_WRITE_NUMERIC_GRAPH_FIXTURES").is_some() {
            let suffix = if helper_selected {
                "nested"
            } else {
                "nested_hidden"
            };
            fs::write(
                format!("/tmp/numeric_graph_{suffix}_native_input_port.gsp"),
                document.encode().unwrap(),
            )
            .unwrap();
        }
        let outputs = document.invoke_embedded_tool(0, &[input]).unwrap();
        assert_eq!(outputs.len(), 2);
        assert_eq!(document.numeric_value(outputs[0]), Some(6.));
        assert_eq!(document.numeric_value(outputs[1]), Some(12.));
        assert_eq!(
            document.objects[outputs[0] - 1].presentation_suppressed,
            !helper_selected
        );
        assert_eq!(
            document.selected,
            if helper_selected {
                BTreeSet::from([outputs[0], outputs[1]])
            } else {
                BTreeSet::from([outputs[1]])
            }
        );
        document.set_parameter_value(input, 7.).unwrap();
        assert_eq!(document.numeric_value(outputs[1]), Some(16.));
        document.verify_roundtrip().unwrap();
        if std::env::var_os("SKETCHPAD_WRITE_NUMERIC_GRAPH_FIXTURES").is_some() {
            let suffix = if helper_selected {
                "nested"
            } else {
                "nested_hidden"
            };
            fs::write(
                format!("/tmp/numeric_graph_{suffix}_authored_port.gsp"),
                created,
            )
            .unwrap();
            fs::write(
                format!("/tmp/numeric_graph_{suffix}_invoked_port.gsp"),
                document.encode().unwrap(),
            )
            .unwrap();
        }
    }
}

#[test]
fn create_length_parameter_and_measured_calculation_numeric_templates() {
    for measured in [false, true] {
        let mut document = Document::from_bytes(if measured {
            include_bytes!("../../../../../tests/fixtures/embedded_tool_numeric_regular_measure_native_verified.gsp").as_slice()
        } else {
            include_bytes!("../../../../../tests/fixtures/embedded_tool_scalar_on_length_native_verified.gsp").as_slice()
        }, None).unwrap();
        let input = if measured { 6 } else { 3 };
        let result = if measured {
            document
                .add_measured_length_calculation(input, 2, (20, 280))
                .unwrap()
        } else {
            let group =
                crate::table_records::ParameterCalculationGroup::from_expression_with_source_unit(
                    input as u32,
                    "l",
                    "l*2",
                    (20, 240),
                    NumericUnit::Length,
                )
                .unwrap();
            document.add(Kind::ParameterCalculation(input, group))
        };
        document.selected = BTreeSet::from([input, result]);
        let source = document.encode().unwrap();
        document
            .create_embedded_tool(if measured {
                "Measured times two"
            } else {
                "Length times two"
            })
            .unwrap();
        let prototype = document.embedded_tool_document(1).unwrap();
        assert_eq!(prototype.objects.len(), 2);
        assert_eq!(
            prototype.objects[0].kind,
            Kind::ToolInput(EmbeddedToolInputKind::Numeric)
        );
        assert!(matches!(
            prototype.objects[1].kind,
            Kind::ParameterCalculation(1, _)
        ));
        let created = document.encode().unwrap();
        document.undo();
        assert_eq!(document.encode().unwrap(), source);
        document.redo();
        let output = document.invoke_embedded_tool(1, &[input]).unwrap()[0];
        assert!((document.numeric_value(output).unwrap() - 10.).abs() < 1e-10);
        assert_eq!(
            document.objects[output - 1].kind.numeric_unit(),
            Some(NumericUnit::Length)
        );
        document.verify_roundtrip().unwrap();
        if std::env::var_os("SKETCHPAD_WRITE_NUMERIC_GRAPH_FIXTURES").is_some() {
            let suffix = if measured {
                "measured_multiply"
            } else {
                "length"
            };
            fs::write(
                format!("/tmp/numeric_graph_{suffix}_authored_port.gsp"),
                created,
            )
            .unwrap();
            fs::write(
                format!("/tmp/numeric_graph_{suffix}_invoked_port.gsp"),
                document.encode().unwrap(),
            )
            .unwrap();
        }
    }
}
