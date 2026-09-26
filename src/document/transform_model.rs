//! Shared transform semantics and native encoding identity.
//!
//! Imported header bytes, auxiliary records and caches remain owned by the
//! document serializer. This model preserves operation flavor, literal fields,
//! and ordered references without flattening live source chains.
use super::Kind;
use crate::table_records::NumericUnit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementType {
    Point,
    Segment,
    Ray,
    Line,
    Circle,
    Polygon,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NumericExpr {
    /// Preserve both native magnitude and conversion factor, including signed zero.
    Literal {
        magnitude: f64,
        unit: NumericUnit,
        scale: f64,
    },
    ObjectValue {
        id: usize,
        unit: NumericUnit,
    },
    DirectedAngle {
        first: usize,
        vertex: usize,
        last: usize,
    },
    DistanceRatio {
        origin: usize,
        base: usize,
        target: usize,
    },
}
impl NumericExpr {
    pub fn unit(&self) -> NumericUnit {
        match self {
            Self::Literal { unit, .. } | Self::ObjectValue { unit, .. } => *unit,
            Self::DirectedAngle { .. } => NumericUnit::Degrees,
            Self::DistanceRatio { .. } => NumericUnit::Scalar,
        }
    }
    pub fn references(&self) -> Vec<usize> {
        match *self {
            Self::Literal { .. } => vec![],
            Self::ObjectValue { id, .. } => vec![id],
            Self::DirectedAngle {
                first,
                vertex,
                last,
            } => vec![first, vertex, last],
            Self::DistanceRatio {
                origin,
                base,
                target,
            } => vec![origin, base, target],
        }
    }
    fn remap(&mut self, ids: &[usize]) -> Result<(), String> {
        let m = |id: &mut usize| -> Result<(), String> {
            *id = ids
                .get(*id)
                .copied()
                .filter(|id| *id != 0)
                .ok_or("Unresolved transform reference")?;
            Ok(())
        };
        match self {
            Self::Literal { .. } => {}
            Self::ObjectValue { id, .. } => m(id)?,
            Self::DirectedAngle {
                first,
                vertex,
                last,
            } => {
                m(first)?;
                m(vertex)?;
                m(last)?;
            }
            Self::DistanceRatio {
                origin,
                base,
                target,
            } => {
                m(origin)?;
                m(base)?;
                m(target)?;
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum DisplacementExpr {
    Cartesian {
        horizontal: NumericExpr,
        vertical: NumericExpr,
    },
    Polar {
        angle: NumericExpr,
        distance: NumericExpr,
    },
    Vector {
        start: usize,
        end: usize,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub enum Transform {
    Translate(DisplacementExpr),
    Rotate { center: usize, angle: NumericExpr },
    Dilate { center: usize, ratio: NumericExpr },
    Reflect { mirror: usize },
}
impl Transform {
    pub fn references(&self) -> Vec<usize> {
        match self {
            Self::Translate(DisplacementExpr::Cartesian {
                horizontal,
                vertical,
            }) => horizontal
                .references()
                .into_iter()
                .chain(vertical.references())
                .collect(),
            Self::Translate(DisplacementExpr::Polar { angle, distance }) => angle
                .references()
                .into_iter()
                .chain(distance.references())
                .collect(),
            Self::Translate(DisplacementExpr::Vector { start, end }) => vec![*start, *end],
            Self::Rotate { center, angle } => {
                std::iter::once(*center).chain(angle.references()).collect()
            }
            Self::Dilate { center, ratio } => {
                std::iter::once(*center).chain(ratio.references()).collect()
            }
            Self::Reflect { mirror } => vec![*mirror],
        }
    }
    fn remap(&mut self, ids: &[usize]) -> Result<(), String> {
        let m = |id: &mut usize| -> Result<(), String> {
            *id = ids
                .get(*id)
                .copied()
                .filter(|id| *id != 0)
                .ok_or("Unresolved transform reference")?;
            Ok(())
        };
        match self {
            Self::Translate(DisplacementExpr::Cartesian {
                horizontal,
                vertical,
            }) => {
                horizontal.remap(ids)?;
                vertical.remap(ids)?;
            }
            Self::Translate(DisplacementExpr::Polar { angle, distance }) => {
                angle.remap(ids)?;
                distance.remap(ids)?;
            }
            Self::Translate(DisplacementExpr::Vector { start, end }) => {
                m(start)?;
                m(end)?;
            }
            Self::Rotate { center, angle } => {
                m(center)?;
                angle.remap(ids)?;
            }
            Self::Dilate { center, ratio } => {
                m(center)?;
                ratio.remap(ids)?;
            }
            Self::Reflect { mirror } => m(mirror)?,
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeTransformFlavor {
    Vector,
    FixedCartesian,
    FixedHorizontal,
    FixedVertical,
    MarkedCartesian,
    FixedPolar,
    MarkedPolar,
    FixedRotation,
    PointAngleRotation,
    NumericRotation,
    FixedDilation,
    PointRatioDilation,
    NumericDilation,
    Reflection,
}
impl NativeTransformFlavor {
    pub fn code(self) -> u16 {
        match self {
            Self::Vector => 16,
            Self::FixedCartesian => 17,
            Self::FixedHorizontal => 18,
            Self::FixedVertical => 19,
            Self::MarkedCartesian => 20,
            Self::FixedPolar => 21,
            Self::MarkedPolar => 24,
            Self::FixedRotation => 27,
            Self::PointAngleRotation => 28,
            Self::NumericRotation => 29,
            Self::FixedDilation => 30,
            Self::PointRatioDilation => 32,
            Self::NumericDilation => 33,
            Self::Reflection => 34,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeLiteralRole {
    AngleDegrees,
    PolarMagnitude,
    PolarUnitsToPixels,
    HorizontalMagnitude,
    HorizontalUnitsToPixels,
    VerticalMagnitude,
    VerticalUnitsToPixels,
    FixedCentimeters,
    ScaleFactor,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeLiteral {
    pub role: NativeLiteralRole,
    pub value: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TransformDefinition {
    source: usize,
    element: ElementType,
    transform: Transform,
    flavor: NativeTransformFlavor,
}
impl TransformDefinition {
    pub fn new(
        source: usize,
        element: ElementType,
        transform: Transform,
        flavor: NativeTransformFlavor,
    ) -> Result<Self, String> {
        use NativeTransformFlavor as F;
        use NumericExpr as N;
        let literal = |e: &NumericExpr, unit| matches!(e,N::Literal { unit:u,.. } if *u==unit);
        let object = |e: &NumericExpr, unit| matches!(e,N::ObjectValue { unit:u,.. } if *u==unit);
        let valid = match (&transform, flavor) {
            (Transform::Translate(DisplacementExpr::Vector { .. }), F::Vector) => true,
            (
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: h,
                    vertical: v,
                }),
                F::FixedCartesian,
            ) => literal(h, NumericUnit::Length) && literal(v, NumericUnit::Length),
            (
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: h,
                    vertical: v,
                }),
                F::FixedHorizontal,
            ) => literal(h, NumericUnit::Length) && object(v, NumericUnit::Length),
            (
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: h,
                    vertical: v,
                }),
                F::FixedVertical,
            ) => object(h, NumericUnit::Length) && literal(v, NumericUnit::Length),
            (
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: h,
                    vertical: v,
                }),
                F::MarkedCartesian,
            ) => object(h, NumericUnit::Length) && object(v, NumericUnit::Length),
            (
                Transform::Translate(DisplacementExpr::Polar {
                    angle: a,
                    distance: d,
                }),
                F::FixedPolar,
            ) => literal(a, NumericUnit::Degrees) && literal(d, NumericUnit::Length),
            (
                Transform::Translate(DisplacementExpr::Polar {
                    angle: a,
                    distance: d,
                }),
                F::MarkedPolar,
            ) => literal(a, NumericUnit::Degrees) && object(d, NumericUnit::Length),
            (Transform::Rotate { angle, .. }, F::FixedRotation) => {
                literal(angle, NumericUnit::Degrees)
            }
            (Transform::Rotate { angle, .. }, F::NumericRotation) => {
                object(angle, NumericUnit::Degrees)
            }
            (
                Transform::Rotate {
                    angle: N::DirectedAngle { .. },
                    ..
                },
                F::PointAngleRotation,
            ) => true,
            (Transform::Dilate { ratio, .. }, F::FixedDilation) => {
                literal(ratio, NumericUnit::Scalar)
            }
            (Transform::Dilate { ratio, .. }, F::NumericDilation) => {
                object(ratio, NumericUnit::Scalar)
            }
            (
                Transform::Dilate {
                    ratio: N::DistanceRatio { .. },
                    ..
                },
                F::PointRatioDilation,
            ) => true,
            (Transform::Reflect { .. }, F::Reflection) => true,
            _ => false,
        };
        let straight = matches!(element, ElementType::Ray | ElementType::Line);
        if !valid
            || (straight
                && !matches!(
                    flavor,
                    F::FixedRotation | F::NumericRotation | F::FixedDilation | F::NumericDilation
                ))
        {
            return Err("Unsupported transform element/expression/wire flavor".into());
        }
        let result = Self {
            source,
            element,
            transform,
            flavor,
        };
        if result.native_references().contains(&0) {
            return Err("Zero transform reference".into());
        }
        Ok(result)
    }
    pub fn source(&self) -> usize {
        self.source
    }
    pub fn element(&self) -> ElementType {
        self.element
    }
    pub fn transform(&self) -> &Transform {
        &self.transform
    }
    pub fn wire_flavor(&self) -> NativeTransformFlavor {
        self.flavor
    }
    /// Native serialization order, including repeated references.
    pub fn native_references(&self) -> Vec<usize> {
        std::iter::once(self.source)
            .chain(self.transform.references())
            .collect()
    }
    /// Ordered parent list, matching the existing document dependency API.
    pub fn dependencies(&self) -> Vec<usize> {
        self.native_references()
    }
    pub fn remap_checked(&self, ids: &[usize]) -> Result<Self, String> {
        let mut result = self.clone();
        result.source = ids
            .get(self.source)
            .copied()
            .filter(|id| *id != 0)
            .ok_or("Unresolved transform source")?;
        result.transform.remap(ids)?;
        Ok(result)
    }
    pub fn native_literals(&self) -> Vec<NativeLiteral> {
        use NativeLiteralRole as R;
        use NativeTransformFlavor as F;
        use NumericExpr as N;
        let item = |role, value| NativeLiteral { role, value };
        match (&self.transform, self.flavor) {
            (
                Transform::Translate(DisplacementExpr::Polar {
                    angle: N::Literal { magnitude: a, .. },
                    distance:
                        N::Literal {
                            magnitude: d,
                            scale: u,
                            ..
                        },
                }),
                F::FixedPolar,
            ) => vec![
                item(R::AngleDegrees, *a),
                item(R::PolarUnitsToPixels, *u),
                item(R::PolarMagnitude, *d),
            ],
            (
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal:
                        N::Literal {
                            magnitude: x,
                            scale: xu,
                            ..
                        },
                    vertical:
                        N::Literal {
                            magnitude: y,
                            scale: yu,
                            ..
                        },
                }),
                F::FixedCartesian,
            ) => vec![
                item(R::HorizontalUnitsToPixels, *xu),
                item(R::HorizontalMagnitude, *x),
                item(R::VerticalUnitsToPixels, *yu),
                item(R::VerticalMagnitude, *y),
            ],
            (
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: N::Literal { magnitude, .. },
                    ..
                }),
                F::FixedHorizontal,
            )
            | (
                Transform::Translate(DisplacementExpr::Cartesian {
                    vertical: N::Literal { magnitude, .. },
                    ..
                }),
                F::FixedVertical,
            ) => vec![item(R::FixedCentimeters, *magnitude)],
            (
                Transform::Translate(DisplacementExpr::Polar {
                    angle: N::Literal { magnitude, .. },
                    ..
                }),
                F::MarkedPolar,
            )
            | (
                Transform::Rotate {
                    angle: N::Literal { magnitude, .. },
                    ..
                },
                F::FixedRotation,
            ) => vec![item(R::AngleDegrees, *magnitude)],
            (
                Transform::Dilate {
                    ratio: N::Literal { magnitude, .. },
                    ..
                },
                F::FixedDilation,
            ) => vec![item(R::ScaleFactor, *magnitude)],
            _ => vec![],
        }
    }
}
fn literal(magnitude: f64, unit: NumericUnit, scale: f64) -> NumericExpr {
    NumericExpr::Literal {
        magnitude,
        unit,
        scale,
    }
}
fn value(id: usize, unit: NumericUnit) -> NumericExpr {
    NumericExpr::ObjectValue { id, unit }
}
impl Kind {
    /// Project a supported legacy transform without changing the stored Kind.
    pub fn transform_definition(&self) -> Option<TransformDefinition> {
        use ElementType as E;
        use NativeTransformFlavor as F;
        let (source, element, transform, flavor) = match *self {
            Self::PolarTranslatedPoint(source, a, u, d) => (
                source,
                E::Point,
                Transform::Translate(DisplacementExpr::Polar {
                    angle: literal(a, NumericUnit::Degrees, 1.),
                    distance: literal(d, NumericUnit::Length, u),
                }),
                F::FixedPolar,
            ),
            Self::PolarTranslatedSegment(source, a, u, d) => (
                source,
                E::Segment,
                Transform::Translate(DisplacementExpr::Polar {
                    angle: literal(a, NumericUnit::Degrees, 1.),
                    distance: literal(d, NumericUnit::Length, u),
                }),
                F::FixedPolar,
            ),
            Self::PolarTranslatedCircle(source, a, u, d) => (
                source,
                E::Circle,
                Transform::Translate(DisplacementExpr::Polar {
                    angle: literal(a, NumericUnit::Degrees, 1.),
                    distance: literal(d, NumericUnit::Length, u),
                }),
                F::FixedPolar,
            ),
            Self::PolarTranslatedPolygon(source, a, u, d) => (
                source,
                E::Polygon,
                Transform::Translate(DisplacementExpr::Polar {
                    angle: literal(a, NumericUnit::Degrees, 1.),
                    distance: literal(d, NumericUnit::Length, u),
                }),
                F::FixedPolar,
            ),
            Self::FixedRectangularTranslatedPoint(source, xu, x, yu, y) => (
                source,
                E::Point,
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: literal(x, NumericUnit::Length, xu),
                    vertical: literal(y, NumericUnit::Length, yu),
                }),
                F::FixedCartesian,
            ),
            Self::FixedRectangularTranslatedSegment(source, xu, x, yu, y) => (
                source,
                E::Segment,
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: literal(x, NumericUnit::Length, xu),
                    vertical: literal(y, NumericUnit::Length, yu),
                }),
                F::FixedCartesian,
            ),
            Self::FixedRectangularTranslatedCircle(source, xu, x, yu, y) => (
                source,
                E::Circle,
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: literal(x, NumericUnit::Length, xu),
                    vertical: literal(y, NumericUnit::Length, yu),
                }),
                F::FixedCartesian,
            ),
            Self::FixedRectangularTranslatedPolygon(source, xu, x, yu, y) => (
                source,
                E::Polygon,
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: literal(x, NumericUnit::Length, xu),
                    vertical: literal(y, NumericUnit::Length, yu),
                }),
                F::FixedCartesian,
            ),
            Self::MarkedDistanceTranslatedPoint(source, d, a) => (
                source,
                E::Point,
                Transform::Translate(DisplacementExpr::Polar {
                    angle: literal(a, NumericUnit::Degrees, 1.),
                    distance: value(d, NumericUnit::Length),
                }),
                F::MarkedPolar,
            ),
            Self::MarkedDistanceTranslatedSegment(source, d, a) => (
                source,
                E::Segment,
                Transform::Translate(DisplacementExpr::Polar {
                    angle: literal(a, NumericUnit::Degrees, 1.),
                    distance: value(d, NumericUnit::Length),
                }),
                F::MarkedPolar,
            ),
            Self::MarkedDistanceTranslatedCircle(source, d, a) => (
                source,
                E::Circle,
                Transform::Translate(DisplacementExpr::Polar {
                    angle: literal(a, NumericUnit::Degrees, 1.),
                    distance: value(d, NumericUnit::Length),
                }),
                F::MarkedPolar,
            ),
            Self::MarkedDistanceTranslatedPolygon(source, d, a) => (
                source,
                E::Polygon,
                Transform::Translate(DisplacementExpr::Polar {
                    angle: literal(a, NumericUnit::Degrees, 1.),
                    distance: value(d, NumericUnit::Length),
                }),
                F::MarkedPolar,
            ),
            Self::MixedRectangularTranslatedPoint(source, d, f, h) => (
                source,
                E::Point,
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: if h {
                        literal(f, NumericUnit::Length, 37.79527559055118)
                    } else {
                        value(d, NumericUnit::Length)
                    },
                    vertical: if h {
                        value(d, NumericUnit::Length)
                    } else {
                        literal(f, NumericUnit::Length, 37.79527559055118)
                    },
                }),
                if h {
                    F::FixedHorizontal
                } else {
                    F::FixedVertical
                },
            ),
            Self::MixedRectangularTranslatedSegment(source, d, f, h) => (
                source,
                E::Segment,
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: if h {
                        literal(f, NumericUnit::Length, 37.79527559055118)
                    } else {
                        value(d, NumericUnit::Length)
                    },
                    vertical: if h {
                        value(d, NumericUnit::Length)
                    } else {
                        literal(f, NumericUnit::Length, 37.79527559055118)
                    },
                }),
                if h {
                    F::FixedHorizontal
                } else {
                    F::FixedVertical
                },
            ),
            Self::MixedRectangularTranslatedCircle(source, d, f, h) => (
                source,
                E::Circle,
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: if h {
                        literal(f, NumericUnit::Length, 37.79527559055118)
                    } else {
                        value(d, NumericUnit::Length)
                    },
                    vertical: if h {
                        value(d, NumericUnit::Length)
                    } else {
                        literal(f, NumericUnit::Length, 37.79527559055118)
                    },
                }),
                if h {
                    F::FixedHorizontal
                } else {
                    F::FixedVertical
                },
            ),
            Self::MixedRectangularTranslatedPolygon(source, d, f, h) => (
                source,
                E::Polygon,
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: if h {
                        literal(f, NumericUnit::Length, 37.79527559055118)
                    } else {
                        value(d, NumericUnit::Length)
                    },
                    vertical: if h {
                        value(d, NumericUnit::Length)
                    } else {
                        literal(f, NumericUnit::Length, 37.79527559055118)
                    },
                }),
                if h {
                    F::FixedHorizontal
                } else {
                    F::FixedVertical
                },
            ),
            Self::RectangularMarkedTranslatedPoint(source, h, v) => (
                source,
                E::Point,
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: value(h, NumericUnit::Length),
                    vertical: value(v, NumericUnit::Length),
                }),
                F::MarkedCartesian,
            ),
            Self::RectangularMarkedTranslatedSegment(source, h, v) => (
                source,
                E::Segment,
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: value(h, NumericUnit::Length),
                    vertical: value(v, NumericUnit::Length),
                }),
                F::MarkedCartesian,
            ),
            Self::RectangularMarkedTranslatedCircle(source, h, v) => (
                source,
                E::Circle,
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: value(h, NumericUnit::Length),
                    vertical: value(v, NumericUnit::Length),
                }),
                F::MarkedCartesian,
            ),
            Self::RectangularMarkedTranslatedPolygon(source, h, v) => (
                source,
                E::Polygon,
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: value(h, NumericUnit::Length),
                    vertical: value(v, NumericUnit::Length),
                }),
                F::MarkedCartesian,
            ),
            Self::VectorTranslatedPoint(source, start, end) => (
                source,
                E::Point,
                Transform::Translate(DisplacementExpr::Vector { start, end }),
                F::Vector,
            ),
            Self::VectorTranslatedSegment(source, start, end) => (
                source,
                E::Segment,
                Transform::Translate(DisplacementExpr::Vector { start, end }),
                F::Vector,
            ),
            Self::VectorTranslatedCircle(source, start, end) => (
                source,
                E::Circle,
                Transform::Translate(DisplacementExpr::Vector { start, end }),
                F::Vector,
            ),
            Self::VectorTranslatedPolygon(source, start, end) => (
                source,
                E::Polygon,
                Transform::Translate(DisplacementExpr::Vector { start, end }),
                F::Vector,
            ),
            Self::AngleRotatedPoint(source, center, first, vertex, last) => (
                source,
                E::Point,
                Transform::Rotate {
                    center,
                    angle: NumericExpr::DirectedAngle {
                        first,
                        vertex,
                        last,
                    },
                },
                F::PointAngleRotation,
            ),
            Self::AngleRotatedSegment(source, center, first, vertex, last) => (
                source,
                E::Segment,
                Transform::Rotate {
                    center,
                    angle: NumericExpr::DirectedAngle {
                        first,
                        vertex,
                        last,
                    },
                },
                F::PointAngleRotation,
            ),
            Self::AngleRotatedCircle(source, center, first, vertex, last) => (
                source,
                E::Circle,
                Transform::Rotate {
                    center,
                    angle: NumericExpr::DirectedAngle {
                        first,
                        vertex,
                        last,
                    },
                },
                F::PointAngleRotation,
            ),
            Self::AngleRotatedPolygon(source, center, first, vertex, last) => (
                source,
                E::Polygon,
                Transform::Rotate {
                    center,
                    angle: NumericExpr::DirectedAngle {
                        first,
                        vertex,
                        last,
                    },
                },
                F::PointAngleRotation,
            ),
            Self::RatioScaledPoint(source, center, origin, base, target) => (
                source,
                E::Point,
                Transform::Dilate {
                    center,
                    ratio: NumericExpr::DistanceRatio {
                        origin,
                        base,
                        target,
                    },
                },
                F::PointRatioDilation,
            ),
            Self::RatioScaledSegment(source, center, origin, base, target) => (
                source,
                E::Segment,
                Transform::Dilate {
                    center,
                    ratio: NumericExpr::DistanceRatio {
                        origin,
                        base,
                        target,
                    },
                },
                F::PointRatioDilation,
            ),
            Self::RatioScaledCircle(source, center, origin, base, target) => (
                source,
                E::Circle,
                Transform::Dilate {
                    center,
                    ratio: NumericExpr::DistanceRatio {
                        origin,
                        base,
                        target,
                    },
                },
                F::PointRatioDilation,
            ),
            Self::RatioScaledPolygon(source, center, origin, base, target) => (
                source,
                E::Polygon,
                Transform::Dilate {
                    center,
                    ratio: NumericExpr::DistanceRatio {
                        origin,
                        base,
                        target,
                    },
                },
                F::PointRatioDilation,
            ),
            Self::ReflectedPoint(source, mirror) => (
                source,
                E::Point,
                Transform::Reflect { mirror },
                F::Reflection,
            ),
            Self::ReflectedSegment(source, mirror) => (
                source,
                E::Segment,
                Transform::Reflect { mirror },
                F::Reflection,
            ),
            Self::ReflectedCircle(source, mirror) => (
                source,
                E::Circle,
                Transform::Reflect { mirror },
                F::Reflection,
            ),
            Self::ReflectedPolygon(source, mirror) => (
                source,
                E::Polygon,
                Transform::Reflect { mirror },
                F::Reflection,
            ),
            Self::ScaledPoint(source, center, factor) => (
                source,
                E::Point,
                Transform::Dilate {
                    center,
                    ratio: literal(factor, NumericUnit::Scalar, 1.),
                },
                F::FixedDilation,
            ),
            Self::ScaledSegment(source, center, factor) => (
                source,
                E::Segment,
                Transform::Dilate {
                    center,
                    ratio: literal(factor, NumericUnit::Scalar, 1.),
                },
                F::FixedDilation,
            ),
            Self::ScaledRay(source, center, factor) => (
                source,
                E::Ray,
                Transform::Dilate {
                    center,
                    ratio: literal(factor, NumericUnit::Scalar, 1.),
                },
                F::FixedDilation,
            ),
            Self::ScaledLine(source, center, factor) => (
                source,
                E::Line,
                Transform::Dilate {
                    center,
                    ratio: literal(factor, NumericUnit::Scalar, 1.),
                },
                F::FixedDilation,
            ),
            Self::ScaledCircle(source, center, factor) => (
                source,
                E::Circle,
                Transform::Dilate {
                    center,
                    ratio: literal(factor, NumericUnit::Scalar, 1.),
                },
                F::FixedDilation,
            ),
            Self::ScaledPolygon(source, center, factor) => (
                source,
                E::Polygon,
                Transform::Dilate {
                    center,
                    ratio: literal(factor, NumericUnit::Scalar, 1.),
                },
                F::FixedDilation,
            ),
            Self::RotatedPoint(source, center, angle) => (
                source,
                E::Point,
                Transform::Rotate {
                    center,
                    angle: literal(angle, NumericUnit::Degrees, 1.),
                },
                F::FixedRotation,
            ),
            Self::RotatedSegment(source, center, angle) => (
                source,
                E::Segment,
                Transform::Rotate {
                    center,
                    angle: literal(angle, NumericUnit::Degrees, 1.),
                },
                F::FixedRotation,
            ),
            Self::RotatedRay(source, center, angle) => (
                source,
                E::Ray,
                Transform::Rotate {
                    center,
                    angle: literal(angle, NumericUnit::Degrees, 1.),
                },
                F::FixedRotation,
            ),
            Self::RotatedLine(source, center, angle) => (
                source,
                E::Line,
                Transform::Rotate {
                    center,
                    angle: literal(angle, NumericUnit::Degrees, 1.),
                },
                F::FixedRotation,
            ),
            Self::RotatedCircle(source, center, angle) => (
                source,
                E::Circle,
                Transform::Rotate {
                    center,
                    angle: literal(angle, NumericUnit::Degrees, 1.),
                },
                F::FixedRotation,
            ),
            Self::RotatedPolygon(source, center, angle) => (
                source,
                E::Polygon,
                Transform::Rotate {
                    center,
                    angle: literal(angle, NumericUnit::Degrees, 1.),
                },
                F::FixedRotation,
            ),
            Self::CalculatedRatioScaledPoint(source, center, ratio) => (
                source,
                E::Point,
                Transform::Dilate {
                    center,
                    ratio: value(ratio, NumericUnit::Scalar),
                },
                F::NumericDilation,
            ),
            Self::CalculatedRatioScaledSegment(source, center, ratio) => (
                source,
                E::Segment,
                Transform::Dilate {
                    center,
                    ratio: value(ratio, NumericUnit::Scalar),
                },
                F::NumericDilation,
            ),
            Self::CalculatedRatioScaledRay(source, center, ratio) => (
                source,
                E::Ray,
                Transform::Dilate {
                    center,
                    ratio: value(ratio, NumericUnit::Scalar),
                },
                F::NumericDilation,
            ),
            Self::CalculatedRatioScaledLine(source, center, ratio) => (
                source,
                E::Line,
                Transform::Dilate {
                    center,
                    ratio: value(ratio, NumericUnit::Scalar),
                },
                F::NumericDilation,
            ),
            Self::CalculatedRatioScaledCircle(source, center, ratio) => (
                source,
                E::Circle,
                Transform::Dilate {
                    center,
                    ratio: value(ratio, NumericUnit::Scalar),
                },
                F::NumericDilation,
            ),
            Self::CalculatedRatioScaledPolygon(source, center, ratio) => (
                source,
                E::Polygon,
                Transform::Dilate {
                    center,
                    ratio: value(ratio, NumericUnit::Scalar),
                },
                F::NumericDilation,
            ),
            Self::CalculatedAngleRotatedPoint(source, center, angle) => (
                source,
                E::Point,
                Transform::Rotate {
                    center,
                    angle: value(angle, NumericUnit::Degrees),
                },
                F::NumericRotation,
            ),
            Self::CalculatedAngleRotatedSegment(source, center, angle) => (
                source,
                E::Segment,
                Transform::Rotate {
                    center,
                    angle: value(angle, NumericUnit::Degrees),
                },
                F::NumericRotation,
            ),
            Self::CalculatedAngleRotatedRay(source, center, angle) => (
                source,
                E::Ray,
                Transform::Rotate {
                    center,
                    angle: value(angle, NumericUnit::Degrees),
                },
                F::NumericRotation,
            ),
            Self::CalculatedAngleRotatedLine(source, center, angle) => (
                source,
                E::Line,
                Transform::Rotate {
                    center,
                    angle: value(angle, NumericUnit::Degrees),
                },
                F::NumericRotation,
            ),
            Self::CalculatedAngleRotatedCircle(source, center, angle) => (
                source,
                E::Circle,
                Transform::Rotate {
                    center,
                    angle: value(angle, NumericUnit::Degrees),
                },
                F::NumericRotation,
            ),
            Self::CalculatedAngleRotatedPolygon(source, center, angle) => (
                source,
                E::Polygon,
                Transform::Rotate {
                    center,
                    angle: value(angle, NumericUnit::Degrees),
                },
                F::NumericRotation,
            ),
            _ => return None,
        };
        TransformDefinition::new(source, element, transform, flavor).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_sixty_legacy_variants_preserve_dependencies_and_remapping() {
        let kinds = [
            Kind::PolarTranslatedPoint(1, -0., 17., 2.),
            Kind::PolarTranslatedSegment(1, -0., 17., 2.),
            Kind::PolarTranslatedCircle(1, -0., 17., 2.),
            Kind::PolarTranslatedPolygon(1, -0., 17., 2.),
            Kind::FixedRectangularTranslatedPoint(1, 17., -0., 23., 2.),
            Kind::FixedRectangularTranslatedSegment(1, 17., -0., 23., 2.),
            Kind::FixedRectangularTranslatedCircle(1, 17., -0., 23., 2.),
            Kind::FixedRectangularTranslatedPolygon(1, 17., -0., 23., 2.),
            Kind::MarkedDistanceTranslatedPoint(1, 3, -0.),
            Kind::MarkedDistanceTranslatedSegment(1, 3, -0.),
            Kind::MarkedDistanceTranslatedCircle(1, 3, -0.),
            Kind::MarkedDistanceTranslatedPolygon(1, 3, -0.),
            Kind::MixedRectangularTranslatedPoint(1, 3, -0., true),
            Kind::MixedRectangularTranslatedSegment(1, 3, -0., true),
            Kind::MixedRectangularTranslatedCircle(1, 3, -0., true),
            Kind::MixedRectangularTranslatedPolygon(1, 3, -0., true),
            Kind::RectangularMarkedTranslatedPoint(1, 3, 4),
            Kind::RectangularMarkedTranslatedSegment(1, 3, 4),
            Kind::RectangularMarkedTranslatedCircle(1, 3, 4),
            Kind::RectangularMarkedTranslatedPolygon(1, 3, 4),
            Kind::VectorTranslatedPoint(1, 3, 4),
            Kind::VectorTranslatedSegment(1, 3, 4),
            Kind::VectorTranslatedCircle(1, 3, 4),
            Kind::VectorTranslatedPolygon(1, 3, 4),
            Kind::AngleRotatedPoint(1, 2, 3, 4, 5),
            Kind::AngleRotatedSegment(1, 2, 3, 4, 5),
            Kind::AngleRotatedCircle(1, 2, 3, 4, 5),
            Kind::AngleRotatedPolygon(1, 2, 3, 4, 5),
            Kind::RatioScaledPoint(1, 2, 3, 4, 5),
            Kind::RatioScaledSegment(1, 2, 3, 4, 5),
            Kind::RatioScaledCircle(1, 2, 3, 4, 5),
            Kind::RatioScaledPolygon(1, 2, 3, 4, 5),
            Kind::ReflectedPoint(1, 2),
            Kind::ReflectedSegment(1, 2),
            Kind::ReflectedCircle(1, 2),
            Kind::ReflectedPolygon(1, 2),
            Kind::ScaledPoint(1, 2, -0.),
            Kind::ScaledSegment(1, 2, -0.),
            Kind::ScaledRay(1, 2, -0.),
            Kind::ScaledLine(1, 2, -0.),
            Kind::ScaledCircle(1, 2, -0.),
            Kind::ScaledPolygon(1, 2, -0.),
            Kind::RotatedPoint(1, 2, -0.),
            Kind::RotatedSegment(1, 2, -0.),
            Kind::RotatedRay(1, 2, -0.),
            Kind::RotatedLine(1, 2, -0.),
            Kind::RotatedCircle(1, 2, -0.),
            Kind::RotatedPolygon(1, 2, -0.),
            Kind::CalculatedRatioScaledPoint(1, 2, 3),
            Kind::CalculatedRatioScaledSegment(1, 2, 3),
            Kind::CalculatedRatioScaledRay(1, 2, 3),
            Kind::CalculatedRatioScaledLine(1, 2, 3),
            Kind::CalculatedRatioScaledCircle(1, 2, 3),
            Kind::CalculatedRatioScaledPolygon(1, 2, 3),
            Kind::CalculatedAngleRotatedPoint(1, 2, 3),
            Kind::CalculatedAngleRotatedSegment(1, 2, 3),
            Kind::CalculatedAngleRotatedRay(1, 2, 3),
            Kind::CalculatedAngleRotatedLine(1, 2, 3),
            Kind::CalculatedAngleRotatedCircle(1, 2, 3),
            Kind::CalculatedAngleRotatedPolygon(1, 2, 3),
        ];
        assert_eq!(kinds.len(), 60);
        let ids = [0, 11, 12, 13, 14, 15];
        for kind in kinds {
            let definition = kind.transform_definition().unwrap();
            assert_eq!(definition.dependencies(), kind.dependencies(), "{kind:?}");
            let remapped = definition.remap_checked(&ids).unwrap();
            assert_eq!(Some(remapped), kind.remap(&ids).transform_definition());
            assert!(definition.remap_checked(&[0, 0]).is_err());
        }
    }
    #[test]
    fn wire_flavors_keep_literal_fields_and_repeated_references() {
        let d = Kind::FixedRectangularTranslatedPoint(1, 17., -0., 23., 2.)
            .transform_definition()
            .unwrap();
        assert_eq!(d.wire_flavor().code(), 17);
        let values = d.native_literals();
        assert_eq!(
            values.iter().map(|v| v.value.to_bits()).collect::<Vec<_>>(),
            [
                17f64.to_bits(),
                (-0f64).to_bits(),
                23f64.to_bits(),
                2f64.to_bits()
            ]
        );
        let d = Kind::PolarTranslatedCircle(1, -0., 17., 2.)
            .transform_definition()
            .unwrap();
        assert_eq!(
            d.native_literals()
                .iter()
                .map(|v| v.value.to_bits())
                .collect::<Vec<_>>(),
            [(-0f64).to_bits(), 17f64.to_bits(), 2f64.to_bits()]
        );
        let d = Kind::AngleRotatedPoint(1, 1, 2, 1, 2)
            .transform_definition()
            .unwrap();
        assert_eq!(d.native_references(), [1, 1, 2, 1, 2]);
        assert_eq!(d.wire_flavor().code(), 28);
        assert!(
            Kind::Free(super::super::Point { x: 0., y: 0. })
                .transform_definition()
                .is_none()
        );
    }
    #[test]
    fn checked_constructor_does_not_enable_unproved_element_flavor_or_units() {
        let angle = NumericExpr::ObjectValue {
            id: 3,
            unit: NumericUnit::Scalar,
        };
        assert!(
            TransformDefinition::new(
                1,
                ElementType::Ray,
                Transform::Rotate { center: 2, angle },
                NativeTransformFlavor::NumericRotation
            )
            .is_err()
        );
        assert!(
            TransformDefinition::new(
                1,
                ElementType::Line,
                Transform::Reflect { mirror: 2 },
                NativeTransformFlavor::Reflection
            )
            .is_err()
        );
        assert!(
            TransformDefinition::new(
                1,
                ElementType::Ray,
                Transform::Translate(DisplacementExpr::Vector { start: 2, end: 3 }),
                NativeTransformFlavor::Vector
            )
            .is_err()
        );
        assert!(
            TransformDefinition::new(
                0,
                ElementType::Point,
                Transform::Reflect { mirror: 2 },
                NativeTransformFlavor::Reflection
            )
            .is_err()
        );
    }
}
