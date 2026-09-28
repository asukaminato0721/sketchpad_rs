//! Shared transform semantics and native encoding identity.
//!
//! Imported header bytes, auxiliary records and caches remain owned by the
//! document serializer. This model preserves operation flavor, literal fields,
//! and ordered references without flattening live source chains.
use crate::NumericUnit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementType {
    Point,
    Segment,
    Ray,
    Line,
    Circle,
    Arc,
    Polygon,
    Sector,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NumericExpr {
    /// Preserve both native magnitude and conversion factor, including signed zero.
    Literal {
        magnitude: f64,
        unit: NumericUnit,
        scale: f64,
    },
    /// Expected operation unit. Length operands in marked translations also
    /// accept unitless values interpreted as signed centimeters by Native.
    ObjectValue { id: usize, unit: NumericUnit },
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
    fn references_mut(&mut self) -> Vec<&mut usize> {
        match self {
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
    fn references_mut(&mut self) -> Vec<&mut usize> {
        match self {
            Self::Translate(DisplacementExpr::Cartesian {
                horizontal,
                vertical,
            }) => horizontal
                .references_mut()
                .into_iter()
                .chain(vertical.references_mut())
                .collect(),
            Self::Translate(DisplacementExpr::Polar { angle, distance }) => angle
                .references_mut()
                .into_iter()
                .chain(distance.references_mut())
                .collect(),
            Self::Translate(DisplacementExpr::Vector { start, end }) => vec![start, end],
            Self::Rotate { center, angle } => std::iter::once(center)
                .chain(angle.references_mut())
                .collect(),
            Self::Dilate { center, ratio } => std::iter::once(center)
                .chain(ratio.references_mut())
                .collect(),
            Self::Reflect { mirror } => vec![mirror],
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
    DualMarkedPolar,
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
            Self::DualMarkedPolar => 26,
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
    /// Compatibility construction preserves legacy malformed-value contracts.
    /// New semantic constructions should use the validated `new` constructor.
    #[doc(hidden)]
    pub fn from_compatibility_parts(
        source: usize,
        element: ElementType,
        transform: Transform,
        flavor: NativeTransformFlavor,
    ) -> Self {
        Self {
            source,
            element,
            transform,
            flavor,
        }
    }
    pub fn new(
        source: usize,
        element: ElementType,
        transform: Transform,
        flavor: NativeTransformFlavor,
    ) -> Result<Self, String> {
        use NativeTransformFlavor as F;
        use NumericExpr as N;
        let literal = |e: &NumericExpr, unit| {
            matches!(e,N::Literal { unit:u,scale,.. }
            if *u==unit && (unit==NumericUnit::Length || *scale==1.))
        };
        let centimeters = |e: &NumericExpr| {
            matches!(e,N::Literal { unit:NumericUnit::Length,scale,.. }
            if *scale==37.79527559055118)
        };
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
            ) => centimeters(h) && object(v, NumericUnit::Length),
            (
                Transform::Translate(DisplacementExpr::Cartesian {
                    horizontal: h,
                    vertical: v,
                }),
                F::FixedVertical,
            ) => object(h, NumericUnit::Length) && centimeters(v),
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
            (
                Transform::Translate(DisplacementExpr::Polar {
                    angle: a,
                    distance: d,
                }),
                F::DualMarkedPolar,
            ) => {
                element == ElementType::Point
                    && object(a, NumericUnit::Degrees)
                    && object(d, NumericUnit::Length)
            }
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
            || (element == ElementType::Sector
                && !matches!(flavor, F::FixedRotation | F::Reflection))
            || (straight
                && !matches!(
                    flavor,
                    F::Vector
                        | F::FixedPolar
                        | F::Reflection
                        | F::FixedRotation
                        | F::NumericRotation
                        | F::FixedDilation
                        | F::NumericDilation
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
    // Match Kind's structural remapping contract, including unresolved zero
    // IDs; checked document operations validate those references separately.
    #[doc(hidden)]
    pub fn remap_unchecked(&self, ids: &[usize]) -> Self {
        let mut result = self.clone();
        result.source = ids[result.source];
        for id in result.transform.references_mut() {
            *id = ids[*id];
        }
        result
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
