use super::*;
use crate::document::transform_model::{
    DisplacementExpr, NativeTransformFlavor, NumericExpr, Transform, TransformDefinition,
};

fn transformed_arc(
    document: &mut Document,
    source: usize,
    transform: Transform,
    flavor: NativeTransformFlavor,
) -> usize {
    document.add(Kind::Transformed(
        TransformDefinition::new(source, ElementType::Arc, transform, flavor).unwrap(),
    ))
}

fn close_point(actual: Point, expected: Point) {
    assert!(
        actual.distance(expected) < 1e-9,
        "{actual:?} != {expected:?}"
    );
}

#[test]
fn analytic_arc_reflection_and_negative_dilation_preserve_parameter_direction_and_live_dependencies()
 {
    let mut doc = Document::new();
    let center = doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let start = doc.add(Kind::Free(Point { x: 140., y: 180. }));
    let end = doc.add(Kind::Free(Point { x: 100., y: 140. }));
    let source = doc.add(Kind::CenterArc(center, start, end));
    let origin = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let vertical = doc.add(Kind::Free(Point { x: 0., y: 100. }));
    let mirror = doc.add(Kind::Line(origin, vertical));
    let reflected = transformed_arc(
        &mut doc,
        source,
        Transform::Reflect { mirror },
        NativeTransformFlavor::Reflection,
    );
    let dilated = transformed_arc(
        &mut doc,
        reflected,
        Transform::Dilate {
            center: origin,
            ratio: NumericExpr::Literal {
                magnitude: -2.,
                unit: NumericUnit::Scalar,
                scale: 1.,
            },
        },
        NativeTransformFlavor::FixedDilation,
    );
    let constrained = doc.add(Kind::PointOnArc(dilated, 0.25));
    let angle = doc.add(Kind::ArcAngle(dilated));
    let length = doc.add(Kind::ArcLength(dilated));
    for t in [0., 0.25, 0.5, 1.] {
        let theta = t * std::f64::consts::FRAC_PI_2;
        let point = Point {
            x: 100. + 40. * theta.cos(),
            y: 180. - 40. * theta.sin(),
        };
        close_point(
            doc.point_on_arc(reflected, t).unwrap(),
            Point {
                x: -point.x,
                y: point.y,
            },
        );
        close_point(
            doc.point_on_arc(dilated, t).unwrap(),
            Point {
                x: 2. * point.x,
                y: -2. * point.y,
            },
        );
    }
    close_point(
        doc.position(constrained).unwrap(),
        doc.point_on_arc(dilated, 0.25).unwrap(),
    );
    assert!((doc.numeric_value(angle).unwrap() - 90.).abs() < 1e-9);
    assert!(
        (doc.numeric_value(length).unwrap() - 40. * std::f64::consts::PI / 37.79527559055118).abs()
            < 1e-9
    );
    let sequence = doc.transform_sequence(dilated).unwrap();
    assert_eq!(sequence.source, source);
    assert_eq!(sequence.element, ElementType::Arc);
    assert_eq!(sequence.matrices.len(), 2);
    assert!(matches!(
        doc.objects[source - 1].kind,
        Kind::CenterArc(1, 2, 3)
    ));
    let original = doc.encode().unwrap();
    doc.verify_roundtrip().unwrap();
    doc.checkpoint();
    doc.move_point(end, Point { x: 120., y: 140. });
    let expected_sweep = 40f64.atan2(20.);
    assert!((doc.numeric_value(angle).unwrap() - expected_sweep.to_degrees()).abs() < 1e-9);
    assert!(
        (doc.numeric_value(length).unwrap() - 80. * expected_sweep / 37.79527559055118).abs()
            < 1e-9
    );
    close_point(
        doc.point_on_arc(dilated, 1.).unwrap(),
        Point {
            x: 2. * (100. + 40. * expected_sweep.cos()),
            y: -2. * (180. - 40. * expected_sweep.sin()),
        },
    );
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), original);
    doc.redo();
    doc.verify_roundtrip().unwrap();
}

#[test]
fn transformed_degenerate_three_point_arc_retains_linear_path_values_without_becoming_segment_kind()
{
    let mut doc = Document::new();
    let a = doc.add(Kind::Free(Point { x: 100., y: 100. }));
    let middle = doc.add(Kind::Free(Point { x: 120., y: 100. }));
    let b = doc.add(Kind::Free(Point { x: 140., y: 100. }));
    let source = doc.add(Kind::ThreePointArc(a, middle, b));
    let origin = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let offset = doc.add(Kind::Free(Point { x: 20., y: 30. }));
    let image = transformed_arc(
        &mut doc,
        source,
        Transform::Translate(DisplacementExpr::Vector {
            start: origin,
            end: offset,
        }),
        NativeTransformFlavor::Vector,
    );
    for t in [0., 0.25, 0.5, 1.] {
        close_point(
            doc.point_on_arc(image, t).unwrap(),
            Point {
                x: 120. + 40. * t,
                y: 130.,
            },
        );
    }
    assert_eq!(
        doc.arc_points(image, 32),
        Some(vec![Point { x: 120., y: 130. }, Point { x: 160., y: 130. }])
    );
    assert_eq!(
        doc.transform_sequence(image).unwrap().element,
        ElementType::Arc
    );
    assert!(matches!(
        doc.objects[source - 1].kind,
        Kind::ThreePointArc(1, 2, 3)
    ));
    let angle = doc.add(Kind::ArcAngle(image));
    let length = doc.add(Kind::ArcLength(image));
    assert_eq!(doc.numeric_value(angle), Some(0.));
    assert!((doc.numeric_value(length).unwrap() - 40. / 37.79527559055118).abs() < 1e-10);
    doc.verify_roundtrip().unwrap();
    doc.checkpoint();
    doc.move_point(middle, Point { x: 120., y: 80. });
    assert_eq!(doc.arc_points(image, 32).unwrap().len(), 33);
    assert!((doc.numeric_value(angle).unwrap() - 180.).abs() < 1e-9);
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.numeric_value(angle), Some(0.));
}

#[test]
fn original_native_vector_arcs_keep_complete_groups_and_analytic_sampling() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/transformed_arc48_rolling_native_verified.gsp"
    );
    let doc = Document::from_bytes(bytes, None).unwrap();
    for id in [33, 83] {
        let definition = doc.objects[id - 1].kind.transform_definition().unwrap();
        assert_eq!(definition.element(), ElementType::Arc);
        assert_eq!(definition.wire_flavor(), NativeTransformFlavor::Vector);
        assert_eq!(doc.arc_points(id, 24).unwrap().len(), 25);
    }
    for t in [0., 0.25, 0.5, 1.] {
        let theta = std::f64::consts::PI * t;
        close_point(
            doc.point_on_arc(33, t).unwrap(),
            Point {
                x: 124. + 100. * theta.cos(),
                y: 398. - 100. * theta.sin(),
            },
        );
        let theta = -std::f64::consts::FRAC_PI_2 + std::f64::consts::PI * t;
        close_point(
            doc.point_on_arc(83, t).unwrap(),
            Point {
                x: 235. + 86. * theta.cos(),
                y: 249. - 86. * theta.sin(),
            },
        );
    }
    assert_eq!(doc.encode().unwrap().as_slice(), bytes);
    doc.verify_roundtrip().unwrap();
}

#[test]
fn every_shared_arc_transform_flavor_matches_independent_samples_and_reopens() {
    use DisplacementExpr as D;
    use NativeTransformFlavor as F;
    use Transform as T;
    let cm = 37.79527559055118;
    for flavor in [
        F::Vector,
        F::FixedCartesian,
        F::FixedHorizontal,
        F::FixedVertical,
        F::MarkedCartesian,
        F::FixedPolar,
        F::MarkedPolar,
        F::FixedRotation,
        F::PointAngleRotation,
        F::NumericRotation,
        F::FixedDilation,
        F::PointRatioDilation,
        F::NumericDilation,
        F::Reflection,
    ] {
        let mut doc = Document::new();
        doc.add(Kind::Free(Point { x: 100., y: 180. }));
        doc.add(Kind::Free(Point { x: 140., y: 180. }));
        doc.add(Kind::Free(Point { x: 100., y: 140. }));
        let arc = doc.add(Kind::CenterArc(1, 2, 3));
        let origin = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let offset = doc.add(Kind::Free(Point { x: 20., y: 30. }));
        let top = doc.add(Kind::Free(Point { x: 0., y: -40. }));
        let right = doc.add(Kind::Free(Point { x: 40., y: 0. }));
        let twice = doc.add(Kind::Free(Point { x: 80., y: 0. }));
        let mirror = doc.add(Kind::Line(origin, top));
        let values: Vec<_> = ["1 cm", "2 cm", "30°", "-0.5"]
            .into_iter()
            .map(|expression| {
                doc.add_calculation_expression(expression, (10, 10))
                    .unwrap()
            })
            .collect();
        let literal = |magnitude, unit| NumericExpr::Literal {
            magnitude,
            unit,
            scale: if unit == NumericUnit::Length { cm } else { 1. },
        };
        let object = |id, unit| NumericExpr::ObjectValue { id, unit };
        let transform = match flavor {
            F::Vector => T::Translate(D::Vector {
                start: origin,
                end: offset,
            }),
            F::FixedCartesian => T::Translate(D::Cartesian {
                horizontal: literal(1., NumericUnit::Length),
                vertical: literal(2., NumericUnit::Length),
            }),
            F::FixedHorizontal => T::Translate(D::Cartesian {
                horizontal: literal(1., NumericUnit::Length),
                vertical: object(values[1], NumericUnit::Length),
            }),
            F::FixedVertical => T::Translate(D::Cartesian {
                horizontal: object(values[0], NumericUnit::Length),
                vertical: literal(2., NumericUnit::Length),
            }),
            F::MarkedCartesian => T::Translate(D::Cartesian {
                horizontal: object(values[0], NumericUnit::Length),
                vertical: object(values[1], NumericUnit::Length),
            }),
            F::FixedPolar => T::Translate(D::Polar {
                angle: literal(30., NumericUnit::Degrees),
                distance: literal(1., NumericUnit::Length),
            }),
            F::MarkedPolar => T::Translate(D::Polar {
                angle: literal(30., NumericUnit::Degrees),
                distance: object(values[0], NumericUnit::Length),
            }),
            F::DualMarkedPolar => unreachable!("code 26 is not an Arc transform"),
            F::FixedRotation => T::Rotate {
                center: origin,
                angle: literal(30., NumericUnit::Degrees),
            },
            F::NumericRotation => T::Rotate {
                center: origin,
                angle: object(values[2], NumericUnit::Degrees),
            },
            F::PointAngleRotation => T::Rotate {
                center: origin,
                angle: NumericExpr::DirectedAngle {
                    first: top,
                    vertex: origin,
                    last: right,
                },
            },
            F::FixedDilation => T::Dilate {
                center: origin,
                ratio: literal(-0.5, NumericUnit::Scalar),
            },
            F::NumericDilation => T::Dilate {
                center: origin,
                ratio: object(values[3], NumericUnit::Scalar),
            },
            F::PointRatioDilation => T::Dilate {
                center: origin,
                ratio: NumericExpr::DistanceRatio {
                    origin,
                    base: right,
                    target: twice,
                },
            },
            F::Reflection => T::Reflect { mirror },
        };
        let image = transformed_arc(&mut doc, arc, transform, flavor);
        for t in [0., 0.25, 0.5, 1.] {
            let theta = t * std::f64::consts::FRAC_PI_2;
            let p = Point {
                x: 100. + 40. * theta.cos(),
                y: 180. - 40. * theta.sin(),
            };
            let expected = match flavor {
                F::Vector => Point {
                    x: p.x + 20.,
                    y: p.y + 30.,
                },
                F::FixedCartesian | F::FixedHorizontal | F::FixedVertical | F::MarkedCartesian => {
                    Point {
                        x: p.x + cm,
                        y: p.y - 2. * cm,
                    }
                }
                F::FixedPolar | F::MarkedPolar => Point {
                    x: p.x + cm * 30f64.to_radians().cos(),
                    y: p.y - cm * 0.5,
                },
                F::DualMarkedPolar => unreachable!("code 26 is not an Arc transform"),
                F::FixedRotation | F::NumericRotation => Point {
                    x: p.x * 30f64.to_radians().cos() + p.y * 0.5,
                    y: -p.x * 0.5 + p.y * 30f64.to_radians().cos(),
                },
                F::PointAngleRotation => Point { x: -p.y, y: p.x },
                F::FixedDilation | F::NumericDilation => Point {
                    x: -p.x * 0.5,
                    y: -p.y * 0.5,
                },
                F::PointRatioDilation => Point {
                    x: p.x * 2.,
                    y: p.y * 2.,
                },
                F::Reflection => Point { x: -p.x, y: p.y },
            };
            close_point(doc.point_on_arc(image, t).unwrap(), expected);
        }
        let point = doc.add(Kind::PointOnArc(image, 0.25));
        close_point(
            doc.position(point).unwrap(),
            doc.point_on_arc(image, 0.25).unwrap(),
        );
        doc.verify_roundtrip()
            .unwrap_or_else(|error| panic!("{flavor:?}: {error}"));
        let bytes = doc.encode().unwrap();
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(
            reopened.objects[image - 1].kind,
            doc.objects[image - 1].kind
        );
        for t in [0., 0.25, 0.5, 1.] {
            close_point(
                reopened.point_on_arc(image, t).unwrap(),
                doc.point_on_arc(image, t).unwrap(),
            );
        }
    }
}

#[test]
fn arc_transform_import_guards_reject_wrong_roles_and_invalid_literal_caches_losslessly() {
    let mut doc = Document::new();
    doc.add(Kind::Free(Point { x: 100., y: 180. }));
    doc.add(Kind::Free(Point { x: 140., y: 180. }));
    doc.add(Kind::Free(Point { x: 100., y: 140. }));
    let arc = doc.add(Kind::CenterArc(1, 2, 3));
    let origin = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let image = transformed_arc(
        &mut doc,
        arc,
        Transform::Rotate {
            center: origin,
            angle: NumericExpr::Literal {
                magnitude: 30.,
                unit: NumericUnit::Degrees,
                scale: 1.,
            },
        },
        NativeTransformFlavor::FixedRotation,
    );
    let bytes = doc.encode().unwrap();
    let imported = Document::from_bytes(&bytes, None).unwrap();
    let at = imported.objects[image - 1].header_offset.unwrap();
    let rs = records(&bytes).unwrap();
    let first = rs
        .iter()
        .position(|r| r.kind == 0x07d0 && r.data.start == at)
        .unwrap();
    let references = rs[first..].iter().find(|r| r.kind == 0x07d2).unwrap();
    let payload = rs[first..].iter().find(|r| r.kind == 0x07d3).unwrap();
    let cases = [
        (at + 8, 66u16.to_le_bytes().to_vec()),
        (references.data.start + 4, 1u32.to_le_bytes().to_vec()),
        (
            references.data.start + 8,
            (arc as u32).to_le_bytes().to_vec(),
        ),
        (
            references.data.start + 8,
            (image as u32).to_le_bytes().to_vec(),
        ),
        (payload.data.start + 4, f64::NAN.to_le_bytes().to_vec()),
        (
            payload.data.start + 20,
            f64::INFINITY.to_le_bytes().to_vec(),
        ),
        (payload.data.start + 28, 9u32.to_le_bytes().to_vec()),
    ];
    for (offset, value) in cases {
        let mut malformed = bytes.clone();
        malformed[offset..offset + value.len()].copy_from_slice(&value);
        let decoded = Document::from_bytes(&malformed, None).unwrap();
        assert!(
            matches!(decoded.objects[image - 1].kind, Kind::Unsupported(27)),
            "offset{offset}"
        );
        assert_eq!(decoded.encode().unwrap(), malformed);
    }
}

#[test]
fn transformed_arc_bindings_recompute_analytic_hosts_under_mapped_seed_substitution() {
    let mut doc = Document::new();
    doc.add(Kind::Free(Point { x: 100., y: 180. }));
    let seed = doc.add(Kind::Free(Point { x: 140., y: 180. }));
    doc.add(Kind::Free(Point { x: 100., y: 140. }));
    let source = doc.add(Kind::CenterArc(1, seed, 3));
    let origin = doc.add(Kind::Free(Point { x: 0., y: 0. }));
    let vertical = doc.add(Kind::Free(Point { x: 0., y: 100. }));
    let mirror = doc.add(Kind::Line(origin, vertical));
    let reflected = transformed_arc(
        &mut doc,
        source,
        Transform::Reflect { mirror },
        NativeTransformFlavor::Reflection,
    );
    let dilated = transformed_arc(
        &mut doc,
        reflected,
        Transform::Dilate {
            center: origin,
            ratio: NumericExpr::Literal {
                magnitude: -2.,
                unit: NumericUnit::Scalar,
                scale: 1.,
            },
        },
        NativeTransformFlavor::FixedDilation,
    );
    let image = doc.add(Kind::PolarTranslatedPoint(seed, 0., 1., 20.));
    let parent = doc
        .add_mapped_point_iteration_with_bindings(
            &[seed],
            &[vec![image]],
            &[source, reflected, dilated, image],
            &[dilated],
            IterationDepth::Fixed(3),
        )
        .unwrap();
    let arcs = doc.arc_iteration_images(parent + 1, 16).unwrap();
    assert_eq!(arcs.len(), 3);
    for (generation, points) in arcs.iter().enumerate() {
        let radius = 60. + 20. * generation as f64;
        close_point(
            points[0],
            Point {
                x: 200. + 2. * radius,
                y: -360.,
            },
        );
        close_point(
            *points.last().unwrap(),
            Point {
                x: 200.,
                y: -360. + 2. * radius,
            },
        );
    }
    assert_eq!(doc.position(seed), Some(Point { x: 140., y: 180. }));
    assert!(matches!(
        doc.objects[source - 1].kind,
        Kind::CenterArc(1, 2, 3)
    ));
    doc.verify_roundtrip().unwrap();
    doc.set_mapped_iteration_final_only(parent, true).unwrap();
    assert_eq!(
        doc.arc_iteration_images(parent + 1, 16),
        Some(vec![arcs[2].clone()])
    );
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.arc_iteration_images(parent + 1, 16), Some(arcs));
}

fn native_arc_literal(value: f64, unit: NumericUnit) -> NumericExpr {
    NumericExpr::Literal {
        magnitude: value,
        unit,
        scale: if unit == NumericUnit::Length {
            37.79527559055118
        } else {
            1.
        },
    }
}
fn native_arc_object(id: usize, unit: NumericUnit) -> NumericExpr {
    NumericExpr::ObjectValue { id, unit }
}

fn native_arc_control(state: &str) -> (Document, usize) {
    let mut d = Document::new();
    for (x, y) in [
        (100., 180.),
        (140., 180.),
        (100., 140.),
        (200., 200.),
        (80., 80.),
        (140., 100.),
        (220., 80.),
        (220., 300.),
    ] {
        d.add(Kind::Free(Point { x, y }));
    }
    d.add(Kind::Segment(7, 8));
    for (x, y) in [(200., 160.), (240., 200.), (280., 200.)] {
        d.add(Kind::Free(Point { x, y }));
    }
    d.add(Kind::CenterArc(1, 2, 3));
    for expr in ["1 cm", "2 cm", "30°"] {
        d.add_calculation_expression(expr, (10, 10)).unwrap();
    }
    d.add_named_parameter("q", -0.5, (10, 10)).unwrap();
    use DisplacementExpr as D;
    use NativeTransformFlavor as F;
    use Transform as T;
    let (transform, flavor) = match state {
        "vector" | "chain" | "chain_edited" => {
            (T::Translate(D::Vector { start: 5, end: 6 }), F::Vector)
        }
        "fixed_cartesian" => (
            T::Translate(D::Cartesian {
                horizontal: native_arc_literal(1., NumericUnit::Length),
                vertical: native_arc_literal(2., NumericUnit::Length),
            }),
            F::FixedCartesian,
        ),
        "fixed_horizontal" => (
            T::Translate(D::Cartesian {
                horizontal: native_arc_literal(1., NumericUnit::Length),
                vertical: native_arc_object(15, NumericUnit::Length),
            }),
            F::FixedHorizontal,
        ),
        "fixed_vertical" => (
            T::Translate(D::Cartesian {
                horizontal: native_arc_object(14, NumericUnit::Length),
                vertical: native_arc_literal(2., NumericUnit::Length),
            }),
            F::FixedVertical,
        ),
        "marked_cartesian" => (
            T::Translate(D::Cartesian {
                horizontal: native_arc_object(14, NumericUnit::Length),
                vertical: native_arc_object(15, NumericUnit::Length),
            }),
            F::MarkedCartesian,
        ),
        "fixed_polar" => (
            T::Translate(D::Polar {
                angle: native_arc_literal(30., NumericUnit::Degrees),
                distance: native_arc_literal(1., NumericUnit::Length),
            }),
            F::FixedPolar,
        ),
        "marked_polar" => (
            T::Translate(D::Polar {
                angle: native_arc_literal(30., NumericUnit::Degrees),
                distance: native_arc_object(14, NumericUnit::Length),
            }),
            F::MarkedPolar,
        ),
        "fixed_rotation" => (
            T::Rotate {
                center: 4,
                angle: native_arc_literal(30., NumericUnit::Degrees),
            },
            F::FixedRotation,
        ),
        "numeric_rotation" => (
            T::Rotate {
                center: 4,
                angle: native_arc_object(16, NumericUnit::Degrees),
            },
            F::NumericRotation,
        ),
        "point_angle_rotation" => (
            T::Rotate {
                center: 4,
                angle: NumericExpr::DirectedAngle {
                    first: 10,
                    vertex: 4,
                    last: 11,
                },
            },
            F::PointAngleRotation,
        ),
        "fixed_dilation" => (
            T::Dilate {
                center: 4,
                ratio: native_arc_literal(-0.5, NumericUnit::Scalar),
            },
            F::FixedDilation,
        ),
        "numeric_dilation" | "numeric_dilation_zero" => (
            T::Dilate {
                center: 4,
                ratio: native_arc_object(17, NumericUnit::Scalar),
            },
            F::NumericDilation,
        ),
        "point_ratio_dilation" => (
            T::Dilate {
                center: 4,
                ratio: NumericExpr::DistanceRatio {
                    origin: 4,
                    base: 11,
                    target: 12,
                },
            },
            F::PointRatioDilation,
        ),
        "reflection" => (T::Reflect { mirror: 9 }, F::Reflection),
        _ => panic!("unknown state"),
    };
    let id = d.add(Kind::Transformed(
        TransformDefinition::new(13, ElementType::Arc, transform, flavor).unwrap(),
    ));
    assert_eq!(id, 18);
    let final_id = if state.starts_with("chain") {
        d.add(Kind::Transformed(
            TransformDefinition::new(
                18,
                ElementType::Arc,
                T::Reflect { mirror: 9 },
                F::Reflection,
            )
            .unwrap(),
        ));
        d.add(Kind::Transformed(
            TransformDefinition::new(
                19,
                ElementType::Arc,
                T::Dilate {
                    center: 4,
                    ratio: native_arc_literal(-0.5, NumericUnit::Scalar),
                },
                F::FixedDilation,
            )
            .unwrap(),
        ))
    } else {
        18
    };
    let (point, _angle, _length) = if state == "numeric_dilation_zero" {
        (0, 0, 0)
    } else {
        (
            d.add(Kind::PointOnArc(final_id, 0.25)),
            d.add(Kind::ArcAngle(final_id)),
            d.add(Kind::ArcLength(final_id)),
        )
    };
    if state == "chain_edited" {
        assert!(d.move_point(2, Point { x: 160., y: 180. }));
    }
    for o in &mut d.objects {
        o.visible = [final_id, point].contains(&o.id);
    }
    d.verify_roundtrip().unwrap();
    let baseline = d.encode().unwrap();
    d = Document::from_bytes(&baseline, None).unwrap();
    assert_eq!(d.encode().unwrap(), baseline);
    if state == "numeric_dilation_zero" {
        d.set_parameter_value(17, 0.).unwrap();
    }
    d.verify_roundtrip().unwrap();
    let b = d.encode().unwrap();
    let r = Document::from_bytes(&b, None).unwrap();
    assert_eq!(r.encode().unwrap(), b);
    (d, final_id)
}

#[test]
fn fresh_arc_public_apis_reconstruct_native_verified_complete_object_groups() {
    for (state,bytes) in [
        ("fixed_cartesian",include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_fixed_cartesian_native_verified.gsp").as_slice()),
        ("fixed_horizontal",include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_fixed_horizontal_native_verified.gsp").as_slice()),
        ("marked_cartesian",include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_marked_cartesian_native_verified.gsp").as_slice()),
        ("point_angle_rotation",include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_point_angle_rotation_native_verified.gsp").as_slice()),
        ("chain",include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_chain_native_verified.gsp").as_slice()),
        ("chain_edited",include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_chain_edited_native_verified.gsp").as_slice()),
        ("numeric_dilation_zero",include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_numeric_dilation_zero_native_verified.gsp").as_slice()),
        ("fixed_polar", include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_fixed_polar_native_verified.gsp").as_slice()),
        ("fixed_vertical", include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_fixed_vertical_native_verified.gsp").as_slice()),
        ("marked_polar", include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_marked_polar_native_verified.gsp").as_slice()),
        ("vector", include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_vector_native_verified.gsp").as_slice()),
        ("fixed_dilation", include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_fixed_dilation_native_verified.gsp").as_slice()),
        ("fixed_rotation", include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_fixed_rotation_native_verified.gsp").as_slice()),
        ("numeric_dilation", include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_numeric_dilation_native_verified.gsp").as_slice()),
        ("numeric_rotation", include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_numeric_rotation_native_verified.gsp").as_slice()),
        ("point_ratio_dilation", include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_point_ratio_dilation_native_verified.gsp").as_slice()),
        ("reflection", include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_reflection_native_verified.gsp").as_slice()),
    ] {
        let (doc,id)=native_arc_control(state);
        assert_eq!(doc.encode().unwrap().as_slice(),bytes,"{state}");
        doc.verify_roundtrip().unwrap();
        let at=Document::from_bytes(bytes,None).unwrap().objects[id-1].header_offset.unwrap();
        assert_eq!(&bytes[at+6..at+10],&[12,0,67,0]);
        if state.starts_with("chain") {
            let radius=if state.contains("edited"){30.}else{20.};
            for t in [0.,0.25,0.5,1.] {
                let theta=t*std::f64::consts::FRAC_PI_2;
                close_point(doc.point_on_arc(id,t).unwrap(),Point{x:160.+radius*theta.cos(),y:200.+radius*theta.sin()});
            }
        } else if state=="numeric_dilation_zero" {
            assert!(doc.arc_points(id,16).is_none());
            assert_eq!(doc.objects[id-1].kind.transform_definition().unwrap().element(),ElementType::Arc);
        }
    }
}

#[test]
fn zero_numeric_arc_dilation_hides_dependent_geometry_and_recovers_after_undo() {
    let (mut doc, image) = native_arc_control("numeric_dilation");
    let downstream = transformed_arc(
        &mut doc,
        image,
        Transform::Translate(DisplacementExpr::Vector { start: 5, end: 6 }),
        NativeTransformFlavor::Vector,
    );
    let baseline = doc.encode().unwrap();
    doc = Document::from_bytes(&baseline, None).unwrap();
    let before = doc.arc_points(downstream, 16).unwrap();
    doc.set_parameter_value(17, 0.).unwrap();
    assert!(doc.point_on_arc(image, 0.5).is_none());
    assert!(doc.arc_points(image, 16).is_none());
    assert!(doc.arc_points(downstream, 16).is_none());
    assert!(doc.position(image + 1).is_none());
    assert!(doc.numeric_value(image + 2).is_none());
    assert!(doc.numeric_value(image + 3).is_none());
    assert_eq!(
        doc.objects[downstream - 1]
            .kind
            .transform_definition()
            .unwrap()
            .element(),
        ElementType::Arc
    );
    doc.verify_roundtrip().unwrap();
    doc.undo();
    assert_eq!(doc.encode().unwrap(), baseline);
    assert_eq!(doc.arc_points(downstream, 16), Some(before));
}

#[test]
fn shared_arc_carrier_preserves_three_point_and_circle_host_parameterization() {
    for circle_host in [false, true] {
        let mut doc = Document::new();
        let center = doc.add(Kind::Free(Point { x: 100., y: 180. }));
        let start = doc.add(Kind::Free(Point { x: 140., y: 180. }));
        let end = doc.add(Kind::Free(Point { x: 100., y: 140. }));
        let source = if circle_host {
            let circle = doc.add(Kind::Circle(center, start));
            let first = doc.add(Kind::PointOnCircle(circle, 1., 0.));
            let last = doc.add(Kind::PointOnCircle(circle, 0., 1.));
            doc.add(Kind::ArcOnCircle(circle, first, last))
        } else {
            let middle = doc.add(Kind::Free(Point {
                x: 100. + 800f64.sqrt(),
                y: 180. - 800f64.sqrt(),
            }));
            doc.add(Kind::ThreePointArc(start, middle, end))
        };
        let origin = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let vertical = doc.add(Kind::Free(Point { x: 0., y: 100. }));
        let mirror = doc.add(Kind::Line(origin, vertical));
        let image = transformed_arc(
            &mut doc,
            source,
            Transform::Reflect { mirror },
            NativeTransformFlavor::Reflection,
        );
        for t in [0., 0.25, 0.5, 1.] {
            let theta = t * std::f64::consts::FRAC_PI_2;
            close_point(
                doc.point_on_arc(image, t).unwrap(),
                Point {
                    x: -100. - 40. * theta.cos(),
                    y: 180. - 40. * theta.sin(),
                },
            );
        }
        let original_kind = doc.objects[source - 1].kind.clone();
        let bytes = doc.encode().unwrap();
        doc.verify_roundtrip().unwrap();
        let reopened = Document::from_bytes(&bytes, None).unwrap();
        assert_eq!(reopened.objects[source - 1].kind, original_kind);
        assert_eq!(
            reopened.objects[image - 1]
                .kind
                .transform_definition()
                .unwrap()
                .element(),
            ElementType::Arc
        );
        assert_eq!(reopened.arc_points(image, 24).unwrap().len(), 25);
    }
}

#[test]
fn native_zero_arc_control_retains_undefined_constraint_and_measurement_records() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/transformed_arc48_api_numeric_dilation_zero_leaves_native_verified.gsp"
    );
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert_eq!(doc.unitless_numeric_value(17), Some(0.));
    assert_eq!(
        doc.objects[17]
            .kind
            .transform_definition()
            .unwrap()
            .element(),
        ElementType::Arc
    );
    assert!(matches!(doc.objects[18].kind,Kind::PointOnArc(18,t) if t==0.25));
    assert!(matches!(doc.objects[19].kind, Kind::ArcAngle(18)));
    assert!(matches!(doc.objects[20].kind, Kind::ArcLength(18)));
    assert!(doc.arc_points(18, 16).is_none());
    assert!(doc.position(19).is_none());
    assert!(doc.numeric_value(20).is_none());
    assert!(doc.numeric_value(21).is_none());
    assert_eq!(doc.encode().unwrap().as_slice(), bytes);
    doc.verify_roundtrip().unwrap();
}

#[test]
fn original_native_arc_rotation_reflection_polar_and_ratio_groups_match_independent_operations() {
    for (bytes, ids) in [
        (
            include_bytes!(
                "../../../../../tests/fixtures/transformed_arc48_rotation_reflection_native_verified.gsp"
            )
            .as_slice(),
            vec![42, 43],
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/transformed_arc48_cartesian_native_verified.gsp"
            )
            .as_slice(),
            vec![65, 66],
        ),
        (
            include_bytes!(
                "../../../../../tests/fixtures/transformed_arc48_dilation_native_verified.gsp"
            )
            .as_slice(),
            vec![27],
        ),
    ] {
        let doc = Document::from_bytes(bytes, None).unwrap();
        for id in ids {
            let definition = doc.objects[id - 1].kind.transform_definition().unwrap();
            assert_eq!(definition.element(), ElementType::Arc);
            for t in [0., 0.25, 0.5, 1.] {
                let p = doc.point_on_arc(definition.source(), t).unwrap();
                let expected = match definition.transform() {
                    Transform::Rotate {
                        center,
                        angle:
                            NumericExpr::Literal {
                                magnitude, scale, ..
                            },
                    } => {
                        let c = doc.position(*center).unwrap();
                        let theta = (magnitude * scale).to_radians();
                        Point {
                            x: c.x + (p.x - c.x) * theta.cos() + (p.y - c.y) * theta.sin(),
                            y: c.y - (p.x - c.x) * theta.sin() + (p.y - c.y) * theta.cos(),
                        }
                    }
                    Transform::Reflect { mirror } => {
                        let (a, b, _) = doc.line_points(*mirror).unwrap();
                        let (dx, dy) = (b.x - a.x, b.y - a.y);
                        let projection =
                            ((p.x - a.x) * dx + (p.y - a.y) * dy) / (dx * dx + dy * dy);
                        Point {
                            x: 2. * (a.x + projection * dx) - p.x,
                            y: 2. * (a.y + projection * dy) - p.y,
                        }
                    }
                    Transform::Translate(DisplacementExpr::Polar {
                        angle:
                            NumericExpr::Literal {
                                magnitude: angle,
                                scale: angle_scale,
                                ..
                            },
                        distance:
                            NumericExpr::Literal {
                                magnitude: distance,
                                scale,
                                ..
                            },
                    }) => {
                        let theta = (angle * angle_scale).to_radians();
                        Point {
                            x: p.x + distance * scale * theta.cos(),
                            y: p.y - distance * scale * theta.sin(),
                        }
                    }
                    Transform::Dilate {
                        center,
                        ratio:
                            NumericExpr::DistanceRatio {
                                origin,
                                base,
                                target,
                            },
                    } => {
                        let c = doc.position(*center).unwrap();
                        let o = doc.position(*origin).unwrap();
                        let b = doc.position(*base).unwrap();
                        let target = doc.position(*target).unwrap();
                        let (dx, dy) = (b.x - o.x, b.y - o.y);
                        let ratio =
                            ((target.x - o.x) * dx + (target.y - o.y) * dy) / (dx * dx + dy * dy);
                        Point {
                            x: c.x + (p.x - c.x) * ratio,
                            y: c.y + (p.y - c.y) * ratio,
                        }
                    }
                    other => panic!("unexpected original transform {other:?}"),
                };
                close_point(doc.point_on_arc(id, t).unwrap(), expected);
            }
            assert!(doc.arc_points(id, 24).unwrap().len() > 2);
        }
        assert_eq!(doc.encode().unwrap().as_slice(), bytes);
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn native_transformed_three_point_and_circle_arc_sources_preserve_analytic_hosts() {
    for (bytes,source,image) in [
        (include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_vector_threepoint_native_verified.gsp").as_slice(),13,18),
        (include_bytes!("../../../../../tests/fixtures/transformed_arc48_api_vector_on_circle_native_verified.gsp").as_slice(),6,9),
    ] {
        let doc=Document::from_bytes(bytes,None).unwrap();
        assert!(matches!(doc.objects[source-1].kind,Kind::ThreePointArc(..)|Kind::ArcOnCircle(..)));
        assert_eq!(doc.objects[image-1].kind.transform_definition().unwrap().element(),ElementType::Arc);
        for t in [0.,0.25,0.5,1.] {
            let p=doc.point_on_arc(source,t).unwrap();
            close_point(doc.point_on_arc(image,t).unwrap(),Point{x:p.x+60.,y:p.y+20.});
        }
        close_point(doc.position(image+1).unwrap(),doc.point_on_arc(image,0.25).unwrap());
        assert_eq!(doc.encode().unwrap().as_slice(),bytes);
        doc.verify_roundtrip().unwrap();
    }
}

#[test]
fn native_collinear_three_point_transform_keeps_arc_identity_and_endpoint_order() {
    let bytes = include_bytes!(
        "../../../../../tests/fixtures/transformed_arc48_api_vector_threepoint_collinear_native_verified.gsp"
    );
    let doc = Document::from_bytes(bytes, None).unwrap();
    assert!(matches!(doc.objects[12].kind, Kind::ThreePointArc(..)));
    assert_eq!(
        doc.objects[17]
            .kind
            .transform_definition()
            .unwrap()
            .element(),
        ElementType::Arc
    );
    let points = doc.arc_points(18, 24).unwrap();
    assert_eq!(points.len(), 2);
    close_point(points[0], Point { x: 200., y: 200. });
    close_point(points[1], Point { x: 120., y: 200. });
    assert!(doc.line_points(18).is_none());
    assert_eq!(doc.encode().unwrap().as_slice(), bytes);
    doc.verify_roundtrip().unwrap();
}
