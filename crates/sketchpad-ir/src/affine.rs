//! Pure affine math and evaluated element views shared by engine and renderers.
use crate::transform_model::ElementType;
use crate::{LineMode, NumericUnit, Point};

/// A homogeneous affine matrix in screen coordinates, applied to column vectors.
/// The last row is always `[0, 0, 1]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AffineMatrix {
    rows: [[f64; 3]; 2],
    evaluation: MatrixEvaluation,
}
// Centered/native operations retain their arithmetic order. Expanding them
// into coefficients changes floating-point rounding and native cache bytes.
#[derive(Clone, Copy, Debug, PartialEq)]
#[doc(hidden)]
pub enum MatrixEvaluation {
    Expanded,
    Translation(f64, f64),
    Rotation(Point, f64),
    Dilation(Point, f64),
    Reflection(Point, Point),
}
impl AffineMatrix {
    pub const IDENTITY: Self = Self {
        rows: [[1., 0., 0.], [0., 1., 0.]],
        evaluation: MatrixEvaluation::Expanded,
    };

    pub fn from_rows(rows: [[f64; 3]; 2]) -> Self {
        Self {
            rows,
            evaluation: MatrixEvaluation::Expanded,
        }
    }

    /// Preserve the existing operation evaluation order independently of rows.
    #[doc(hidden)]
    pub fn from_evaluation(rows: [[f64; 3]; 2], evaluation: MatrixEvaluation) -> Self {
        Self { rows, evaluation }
    }

    pub fn rows(&self) -> [[f64; 3]; 2] {
        self.rows
    }

    pub fn apply(self, point: Point) -> Option<Point> {
        let [[a, b, tx], [c, d, ty]] = self.rows;
        let result = match self.evaluation {
            MatrixEvaluation::Expanded => Point {
                x: a * point.x + b * point.y + tx,
                y: c * point.x + d * point.y + ty,
            },
            MatrixEvaluation::Translation(dx, dy) => Point {
                x: point.x + dx,
                y: point.y + dy,
            },
            MatrixEvaluation::Rotation(center, theta) => {
                return rotate_point_about(point, center, theta);
            }
            MatrixEvaluation::Dilation(center, factor) => {
                return scale_point_about(point, center, factor);
            }
            MatrixEvaluation::Reflection(a, b) => return reflect_point(point, a, b),
        };
        (result.x.is_finite() && result.y.is_finite()).then_some(result)
    }

    /// Apply this matrix first, then `next` (the product is `next * self`).
    pub fn then(self, next: Self) -> Self {
        let [[a, b, tx], [c, d, ty]] = self.rows;
        let [[e, f, ux], [g, h, uy]] = next.rows;
        Self::from_rows([
            [e * a + f * c, e * b + f * d, e * tx + f * ty + ux],
            [g * a + h * c, g * b + h * d, g * tx + h * ty + uy],
        ])
    }
}

/// An evaluated view of a live source chain. Rebuild after dependency edits;
/// matrices are snapshots, while the document retains construction references.
#[derive(Clone, Debug, PartialEq)]
pub struct TransformSequence {
    pub source: usize,
    pub element: ElementType,
    /// Matrices in execution order, from the source to the final image.
    pub matrices: Vec<AffineMatrix>,
}
impl TransformSequence {
    pub fn apply(&self, point: Point) -> Option<Point> {
        self.matrices
            .iter()
            .try_fold(point, |point, matrix| matrix.apply(point))
    }

    /// Composition expands centered operations; floating-point rounding can
    /// differ from `apply`, which preserves each native operation's order.
    pub fn combined_matrix(&self) -> AffineMatrix {
        self.matrices
            .iter()
            .fold(AffineMatrix::IDENTITY, |matrix, next| matrix.then(*next))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TransformGeometry {
    Point(Point),
    Straight {
        start: Point,
        end: Point,
        mode: LineMode,
    },
    Circle {
        center: Point,
        radius: f64,
    },
    /// An analytic circular arc; signed sweep preserves construction orientation.
    Arc {
        center: Point,
        radius: f64,
        start: f64,
        sweep: f64,
        clockwise_y: bool,
    },
    Polygon(Vec<Point>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransformEvaluationError {
    NotTransform,
    InvalidReference(usize),
    WrongElement(usize),
    WrongUnit { id: usize, expected: NumericUnit },
    Undefined,
}

pub fn reflect_point(point: Point, a: Point, b: Point) -> Option<Point> {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let length_sq = dx * dx + dy * dy;
    if !length_sq.is_finite() || length_sq <= 1e-18 {
        return None;
    }
    let t = ((point.x - a.x) * dx + (point.y - a.y) * dy) / length_sq;
    let projected = Point {
        x: a.x + t * dx,
        y: a.y + t * dy,
    };
    let reflected = Point {
        x: 2. * projected.x - point.x,
        y: 2. * projected.y - point.y,
    };
    (reflected.x.is_finite() && reflected.y.is_finite()).then_some(reflected)
}

pub fn scale_point_about(source: Point, center: Point, factor: f64) -> Option<Point> {
    let result = Point {
        x: center.x + factor * (source.x - center.x),
        y: center.y + factor * (source.y - center.y),
    };
    (result.x.is_finite() && result.y.is_finite()).then_some(result)
}

pub fn rotate_point_about(source: Point, center: Point, theta: f64) -> Option<Point> {
    let dx = source.x - center.x;
    let dy = source.y - center.y;
    let rotated = Point {
        x: center.x + dx * theta.cos() + dy * theta.sin(),
        y: center.y - dx * theta.sin() + dy * theta.cos(),
    };
    (rotated.x.is_finite() && rotated.y.is_finite()).then_some(rotated)
}
