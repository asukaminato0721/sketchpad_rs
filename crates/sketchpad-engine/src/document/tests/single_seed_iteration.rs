use super::*;

#[test]
fn generic_single_seed_api_reuses_canonical_point_and_segment_layouts() {
    for segment_binding in [false, true] {
        let mut doc = Document::new();
        let seed = doc.add(Kind::Free(Point { x: 140., y: 250. }));
        let image = doc.add(Kind::PolarTranslatedPoint(seed, 0., 1., 40.));
        let segment = doc.add(Kind::Segment(seed, image));
        let carried = if segment_binding {
            vec![image, segment]
        } else {
            vec![image]
        };
        let source = if segment_binding { segment } else { image };
        let parent = doc
            .add_mapped_point_iteration_with_bindings(
                &[seed],
                &[vec![image]],
                &carried,
                &[source],
                IterationDepth::Fixed(3),
            )
            .unwrap();
        let original = doc.encode().unwrap();
        doc.verify_roundtrip().unwrap();
        if segment_binding {
            assert!(matches!(
                doc.objects[parent - 1].kind,
                Kind::SegmentIteration(..)
            ));
            let images = doc.branch_segment_iteration_images(parent + 1).unwrap();
            assert_eq!(images.len(), 3);
            assert_eq!(
                images[2],
                (Point { x: 260., y: 250. }, Point { x: 300., y: 250. })
            );
        } else {
            assert!(matches!(
                doc.objects[parent - 1].kind,
                Kind::PointIteration(..)
            ));
            assert_eq!(
                doc.mapped_point_binding_images(parent + 1).unwrap(),
                vec![
                    Point { x: 220., y: 250. },
                    Point { x: 260., y: 250. },
                    Point { x: 300., y: 250. }
                ]
            );
        }
        doc.set_mapped_iteration_final_only(parent, true).unwrap();
        assert!(matches!(
            doc.objects[parent - 1].kind,
            Kind::MappedPointIteration {
                final_only: true,
                ..
            }
        ));
        assert_eq!(
            if segment_binding {
                doc.branch_segment_iteration_images(parent + 1)
                    .unwrap()
                    .len()
            } else {
                doc.mapped_point_binding_images(parent + 1).unwrap().len()
            },
            1
        );
        doc.verify_roundtrip().unwrap();
        doc.undo();
        assert_eq!(doc.encode().unwrap(), original);
        doc.redo();
        doc.set_mapped_iteration_final_only(parent, false).unwrap();
        assert_eq!(doc.encode().unwrap(), original);
        doc.verify_roundtrip().unwrap();
    }
}

fn single_geometry(state: &str) -> (Document, usize) {
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 140., y: 250. }));
    let b = doc.add(Kind::Free(Point { x: 240., y: 250. }));
    let u = doc.add(Kind::Free(Point { x: 80., y: 80. }));
    let v = doc.add(Kind::Free(Point { x: 120., y: 80. }));
    let host = doc.add(Kind::Segment(a, b));
    let seed = doc.add(if state == "derived" {
        Kind::Midpoint(a, b)
    } else {
        Kind::PointOnSegment(host, 0.25)
    });
    let early = if matches!(state, "frozen" | "live" | "host_frozen") {
        doc.add(Kind::Segment(seed, b))
    } else {
        0
    };
    let m = if early != 0 {
        doc.add(Kind::SegmentMidpoint(early))
    } else {
        seed
    };
    let n = doc.add(Kind::VectorTranslatedPoint(m, u, v));
    let q = if state == "branches" {
        doc.add(Kind::VectorTranslatedPoint(seed, v, u))
    } else {
        n
    };
    let segment = if early != 0 {
        early
    } else {
        doc.add(Kind::Segment(seed, b))
    };
    let last = segment.max(n).max(q);
    doc.selected
        .extend((1..=last).filter(|id| ![seed, n, segment, b].contains(id)));
    doc.set_selected_visibility(false);
    doc.selected.clear();
    let mut carried = vec![n, segment];
    if state == "live" {
        carried.insert(0, m);
    }
    if state == "host_frozen" {
        carried = vec![m, n];
    }
    let branches = if state == "branches" {
        carried.push(q);
        vec![vec![n], vec![q]]
    } else {
        vec![vec![n]]
    };
    let root = doc
        .add_mapped_point_iteration_with_bindings(
            &[seed],
            &branches,
            &carried,
            &if state == "host_frozen" {
                vec![n]
            } else {
                vec![segment, n]
            },
            IterationDepth::Fixed(3),
        )
        .unwrap();
    (doc, root)
}

#[test]
fn single_derived_path_and_branch_api_writers_match_native_generations() {
    for (state, bytes, line_x, point_x) in [
        (
            "derived",
            include_bytes!("../../../../../tests/fixtures/single_seed45_api_derived_native_verified.gsp")
                .as_slice(),
            vec![230., 270., 310.],
            vec![270., 310., 350.],
        ),
        (
            "geometric",
            include_bytes!(
                "../../../../../tests/fixtures/single_seed45_api_geometric_native_verified.gsp"
            )
            .as_slice(),
            vec![205., 245., 285.],
            vec![245., 285., 325.],
        ),
        (
            "branches",
            include_bytes!(
                "../../../../../tests/fixtures/single_seed45_api_branches_native_verified.gsp"
            )
            .as_slice(),
            vec![205., 125., 245., 165., 85., 285., 205., 125., 45.],
            vec![245., 165., 285., 205., 125., 325., 245., 165., 85.],
        ),
        (
            "frozen",
            include_bytes!("../../../../../tests/fixtures/single_seed45_api_frozen_native_verified.gsp")
                .as_slice(),
            vec![242.5; 3],
            vec![242.5; 3],
        ),
    ] {
        let (doc, root) = single_geometry(state);
        let original_seed = doc.objects[5].kind.clone();
        assert_eq!(doc.encode().unwrap().as_slice(), bytes, "{state}");
        let lines = doc.branch_segment_iteration_images(root + 1).unwrap();
        let points = doc.mapped_point_binding_images(root + 2).unwrap();
        assert_eq!(lines.len(), line_x.len());
        assert_eq!(points.len(), point_x.len());
        for (line, x) in lines.iter().zip(line_x) {
            assert!(line.0.distance(Point { x, y: 250. }) < 1e-9);
            assert_eq!(line.1, Point { x: 240., y: 250. });
        }
        for (point, x) in points.iter().zip(point_x) {
            assert!(point.distance(Point { x, y: 250. }) < 1e-9);
        }
        assert_eq!(doc.objects[5].kind, original_seed);
        doc.verify_roundtrip().unwrap();
        let reopened = Document::from_bytes(bytes, None).unwrap();
        assert_eq!(
            reopened.branch_segment_iteration_images(root + 1),
            Some(lines)
        );
        assert_eq!(reopened.mapped_point_binding_images(root + 2), Some(points));
    }
}

#[test]
fn reflected_polygon_sources_keep_the_existing_matrix_sequence_and_native_geometry() {
    for bytes in [
        include_bytes!(
            "../../../../../tests/fixtures/reflected_polygon_chain45_corpus_native_verified.gsp"
        )
        .as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/reflected_polygon_chain45_isolated_native_verified.gsp"
        )
        .as_slice(),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        for (source, mirror, image) in [(54, 5, 61), (55, 5, 62)] {
            let transform = doc.objects[image - 1].kind.transform_definition().unwrap();
            assert_eq!(transform.source(), source);
            assert_eq!(transform.element(), ElementType::Polygon);
            assert_eq!(transform.wire_flavor(), NativeTransformFlavor::Reflection);
            assert!(doc.objects[source - 1].kind.is_polygon());
            assert_eq!(
                doc.objects[image - 1].kind.dependencies(),
                vec![source, mirror]
            );
        }
        let vertices = doc.polygon_points(62).unwrap();
        for (point, expected) in vertices.iter().zip([
            Point {
                x: 410.7737675209622,
                y: 302.02911684736046,
            },
            Point {
                x: 437.62625115332287,
                y: 281.7401993879789,
            },
            Point {
                x: 461.53732413032475,
                y: 235.36030000888354,
            },
        ]) {
            assert!(point.distance(expected) < 1e-9);
        }
        assert_eq!(doc.encode().unwrap().as_slice(), bytes);
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn original_single_scalar_triangle_and_point_grid_keep_live_geometry() {
    let triangle_bytes =
        include_bytes!("../../../../../tests/fixtures/single_seed45_triangle_corpus_native_verified.gsp");
    let triangle = Document::from_bytes(triangle_bytes, None).unwrap();
    assert!(
        matches!(&triangle.objects[14].kind,Kind::MappedPointIteration{seeds,branches,..} if seeds==&[3] && branches==&[vec![4]])
    );
    for (binding, source, center) in [(16, 9, 1), (17, 8, 5), (18, 6, 7)] {
        let (a, b, _) = triangle.line_points(source).unwrap();
        let center = triangle.position(center).unwrap();
        let images = triangle.branch_segment_iteration_images(binding).unwrap();
        assert_eq!(images.len(), 4);
        for (index, (first, last)) in images.iter().enumerate() {
            let ratio = (index as f64 + 2.) / 5.;
            let expected = |p: Point| Point {
                x: center.x + (p.x - center.x) * ratio,
                y: center.y + (p.y - center.y) * ratio,
            };
            assert!(first.distance(expected(a)) < 1e-9);
            assert!(last.distance(expected(b)) < 1e-9);
        }
    }
    assert_eq!(triangle.encode().unwrap().as_slice(), triangle_bytes);
    triangle.verify_roundtrip().unwrap();
    let grid_bytes =
        include_bytes!("../../../../../tests/fixtures/single_seed45_table_corpus_native_verified.gsp");
    let grid = Document::from_bytes(grid_bytes, None).unwrap();
    let step = 37.79527559055118;
    for (binding, source, count, dx, dy) in [(9, 7, 5, 0., -step), (13, 11, 7, step, 0.)] {
        let (first, last, _) = grid.line_points(source).unwrap();
        let images = grid.branch_segment_iteration_images(binding).unwrap();
        assert_eq!(images.len(), count);
        for (index, (a, b)) in images.iter().enumerate() {
            let expected = |p: Point| Point {
                x: p.x + (index + 1) as f64 * dx,
                y: p.y + (index + 1) as f64 * dy,
            };
            assert!(a.distance(expected(first)) < 1e-9);
            assert!(b.distance(expected(last)) < 1e-9);
        }
    }
    assert_eq!(grid.encode().unwrap().as_slice(), grid_bytes);
    grid.verify_roundtrip().unwrap();
}

#[test]
fn authored_double_reflection_source_and_mirror_edits_match_native_bytes() {
    let mut doc = Document::new();
    for (x, y) in [
        (100., 160.),
        (180., 140.),
        (140., 220.),
        (240., 80.),
        (240., 280.),
        (80., 320.),
        (460., 320.),
    ] {
        doc.add(Kind::Free(Point { x, y }));
    }
    doc.add(Kind::Segment(4, 5));
    doc.add(Kind::Segment(6, 7));
    doc.add(Kind::Polygon(vec![1, 2, 3]));
    doc.add(Kind::ReflectedPolygon(10, 8));
    doc.add(Kind::ReflectedPolygon(11, 9));
    for object in &mut doc.objects {
        object.visible = [8, 9, 12].contains(&object.id);
    }
    for (edit,bytes,expected) in [
        (None,include_bytes!("../../../../../tests/fixtures/reflected_polygon_chain45_api_original_native_verified.gsp").as_slice(),[(380.,480.),(300.,500.),(340.,420.)]),
        (Some((1,Point{x:110.,y:190.})),include_bytes!("../../../../../tests/fixtures/reflected_polygon_chain45_api_source_edited_native_verified.gsp").as_slice(),[(370.,450.),(300.,500.),(340.,420.)]),
        (Some((7,Point{x:460.,y:360.})),include_bytes!("../../../../../tests/fixtures/reflected_polygon_chain45_api_mirror_edited_native_verified.gsp").as_slice(),[(340.1095890410959,538.9589041095891),(257.69863013698625,541.8630136986301),(313.4794520547946,471.94520547945206)]),
    ] {
        let mut state = doc.clone();
        if let Some((id,position))=edit {state.checkpoint();state.move_point(id,position);}
        for (point,(x,y)) in state.polygon_points(12).unwrap().iter().zip(expected) {assert!(point.distance(Point{x,y})<1e-9);}
        assert_eq!(state.encode().unwrap().as_slice(),bytes);
        state.verify_roundtrip().unwrap();
        if edit.is_some() {state.undo();assert_eq!(state.encode().unwrap(),doc.encode().unwrap());state.redo();assert_eq!(state.encode().unwrap().as_slice(),bytes);}
    }
}

#[test]
fn authored_carried_order_and_native_midpoint_host_freezing_are_preserved() {
    let (live, root) = single_geometry("live");
    assert!(
        matches!(&live.objects[root-1].kind,Kind::MappedPointIteration{carried,..} if carried==&[7,8,9])
    );
    assert_eq!(live.objects[7].midpoint_host, Some(7));
    assert_eq!(
        live.encode().unwrap().as_slice(),
        include_bytes!(
            "../../../../../tests/fixtures/single_seed45_api_live_ascending_native_verified.gsp"
        )
    );
    for (point, x) in live
        .mapped_point_binding_images(root + 2)
        .unwrap()
        .iter()
        .zip([281.25, 300.625, 310.3125])
    {
        assert!(point.distance(Point { x, y: 250. }) < 1e-9);
    }
    live.verify_roundtrip().unwrap();
    let (frozen, root) = single_geometry("host_frozen");
    assert_eq!(
        frozen.encode().unwrap().as_slice(),
        include_bytes!("../../../../../tests/fixtures/single_seed45_api_host_frozen_native_verified.gsp")
    );
    assert_eq!(
        frozen.mapped_point_binding_images(root + 1).unwrap(),
        vec![Point { x: 242.5, y: 250. }; 3]
    );
    frozen.verify_roundtrip().unwrap();
}

#[test]
fn single_scalar_and_calculation_seeds_reconstruct_native_api_controls() {
    for calculated in [false, true] {
        let mut doc = Document::new();
        let a = if calculated {
            doc.add_calculation_expression("1", (10, 30)).unwrap()
        } else {
            doc.add_named_parameter("a", 1., (10, 30)).unwrap()
        };
        let b = doc.add_named_parameter("b", 1., (10, 60)).unwrap();
        let c = doc
            .add_multi_calculation(&[a, b], "v1+v2", (10, 90))
            .unwrap();
        let frame = doc
            .add_default_coordinate_system(Point { x: 160., y: 360. })
            .unwrap();
        let p = doc.add(Kind::NumericCoordinatePoint(a, b, frame));
        let q = doc.add(Kind::NumericCoordinatePoint(b, c, frame));
        let segment = doc.add(Kind::Segment(p, q));
        let root = doc
            .add_mapped_point_iteration_with_bindings(
                &[a],
                &[vec![c]],
                &[c, p, q, segment],
                &[p, segment],
                IterationDepth::Fixed(3),
            )
            .unwrap();
        let kind = doc.objects[a - 1].kind.clone();
        let golden: &[u8] = if calculated {
            include_bytes!(
                "../../../../../tests/fixtures/single_seed45_api_calculated_scalar_native_verified.gsp"
            )
        } else {
            include_bytes!("../../../../../tests/fixtures/single_seed45_api_scalar_native_verified.gsp")
        };
        assert_eq!(doc.encode().unwrap().as_slice(), golden);
        let points = doc.mapped_point_binding_images(root + 1).unwrap();
        let segments = doc.branch_segment_iteration_images(root + 2).unwrap();
        let unit = 37.79527559055118;
        for i in 0..3 {
            let expected = Point {
                x: 160. + (i + 2) as f64 * unit,
                y: 360. - unit,
            };
            assert!(points[i].distance(expected) < 1e-9);
            assert!(segments[i].0.distance(expected) < 1e-9);
            assert!(
                segments[i].1.distance(Point {
                    x: 160. + unit,
                    y: 360. - (i + 3) as f64 * unit
                }) < 1e-9
            );
        }
        assert_eq!(doc.objects[a - 1].kind, kind);
        assert_eq!(doc.numeric_value(a), Some(1.));
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn canonical_single_api_point_segment_and_polygon_branch_writers_match_native() {
    for (state, golden) in [
        (
            "point",
            include_bytes!(
                "../../../../../tests/fixtures/single_seed45_api_direct_point_native_verified.gsp"
            )
            .as_slice(),
        ),
        (
            "segment",
            include_bytes!(
                "../../../../../tests/fixtures/single_seed45_api_direct_segment_native_verified.gsp"
            )
            .as_slice(),
        ),
        (
            "polygon",
            include_bytes!(
                "../../../../../tests/fixtures/single_seed45_api_branch_polygon_native_verified.gsp"
            )
            .as_slice(),
        ),
    ] {
        let mut doc = Document::new();
        let a = doc.add(Kind::Free(Point { x: 140., y: 250. }));
        let u = doc.add(Kind::Free(Point { x: 80., y: 80. }));
        let v = doc.add(Kind::Free(Point { x: 120., y: 80. }));
        let (branches, carried, source) = if state == "polygon" {
            let n = doc.add(Kind::PolarTranslatedPoint(a, 0., 40., 1.));
            let q = doc.add(Kind::PolarTranslatedPoint(a, 90., 40., 1.));
            let polygon = doc.add(Kind::Polygon(vec![a, n, q]));
            (vec![vec![n], vec![q]], vec![q, n, polygon], n)
        } else {
            let n = doc.add(Kind::VectorTranslatedPoint(a, u, v));
            if state == "point" {
                (vec![vec![n]], vec![n], n)
            } else {
                let segment = doc.add(Kind::Segment(a, n));
                (vec![vec![n]], vec![n, segment], segment)
            }
        };
        let root = doc
            .add_mapped_point_iteration_with_bindings(
                &[a],
                &branches,
                &carried,
                &[source],
                IterationDepth::Fixed(3),
            )
            .unwrap();
        assert_eq!(doc.encode().unwrap().as_slice(), golden);
        doc.verify_roundtrip().unwrap();
        let reopened = Document::from_bytes(golden, None).unwrap();
        if state == "segment" {
            assert_eq!(
                reopened.branch_segment_iteration_images(root + 1),
                doc.branch_segment_iteration_images(root + 1)
            );
        } else {
            let images = doc.mapped_point_binding_images(root + 1).unwrap();
            assert_eq!(images.len(), if state == "polygon" { 9 } else { 3 });
            assert_eq!(reopened.mapped_point_binding_images(root + 1), Some(images));
        }
    }
}
