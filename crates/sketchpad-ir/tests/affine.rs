use sketchpad_ir::affine::{
    MatrixEvaluation, reflect_point, rotate_point_about, scale_point_about,
};
use sketchpad_ir::transform_model::ElementType;
use sketchpad_ir::{AffineMatrix, LineMode, Point, TransformGeometry, TransformSequence};

fn bits(a: Point, b: Point) {
    assert_eq!(a.x.to_bits(), b.x.to_bits());
    assert_eq!(a.y.to_bits(), b.y.to_bits());
}

#[test]
fn centered_evaluation_keeps_the_original_arithmetic_order() {
    let center = Point { x: 1e16, y: -1e16 };
    let point = Point {
        x: 1e16 + 4.,
        y: -1e16 + 8.,
    };
    let factor = -0.5;
    let matrix = AffineMatrix::from_evaluation(
        [
            [factor, 0., center.x * (1. - factor)],
            [0., factor, center.y * (1. - factor)],
        ],
        MatrixEvaluation::Dilation(center, factor),
    );
    bits(
        matrix.apply(point).unwrap(),
        scale_point_about(point, center, factor).unwrap(),
    );
    let theta = 0.123;
    let rotation = AffineMatrix::from_evaluation(
        AffineMatrix::IDENTITY.rows(),
        MatrixEvaluation::Rotation(center, theta),
    );
    bits(
        rotation.apply(point).unwrap(),
        rotate_point_about(point, center, theta).unwrap(),
    );
}

#[test]
fn sequence_execution_and_matrix_composition_follow_source_order() {
    let translation = AffineMatrix::from_rows([[1., 0., 2.], [0., 1., -1.]]);
    let scaling = AffineMatrix::from_rows([[2., 0., 0.], [0., 3., 0.]]);
    let sequence = TransformSequence {
        source: 9,
        element: ElementType::Point,
        matrices: vec![translation, scaling],
    };
    let point = Point { x: 1., y: 4. };
    assert_eq!(sequence.apply(point), Some(Point { x: 6., y: 9. }));
    assert_eq!(
        sequence.combined_matrix().apply(point),
        sequence.apply(point)
    );
    assert_ne!(
        scaling.then(translation).apply(point),
        sequence.apply(point)
    );
    let straight = TransformGeometry::Straight {
        start: point,
        end: Point { x: 3., y: 4. },
        mode: LineMode::Ray,
    };
    assert!(matches!(
        straight,
        TransformGeometry::Straight {
            mode: LineMode::Ray,
            ..
        }
    ));
}

#[test]
fn undefined_math_does_not_produce_nonfinite_geometry() {
    let a = Point { x: 0., y: 0. };
    let p = Point { x: 1., y: 2. };
    assert_eq!(reflect_point(p, a, a), None);
    assert_eq!(rotate_point_about(p, a, f64::INFINITY), None);
    assert_eq!(scale_point_about(p, a, f64::INFINITY), None);
    let mirror = Point { x: 0., y: 3. };
    assert_eq!(reflect_point(p, a, mirror), Some(Point { x: -1., y: 2. }));
    assert_eq!(
        AffineMatrix::from_rows([[f64::NAN, 0., 0.], [0., 1., 0.]]).apply(p),
        None
    );
}
