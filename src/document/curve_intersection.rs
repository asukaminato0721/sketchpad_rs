//! Native98 ranks crossings on an ordered carrier, after domain filtering.
use super::kind::{is_arc_kind, is_point_kind, is_straight_kind};
use super::{Document, Kind, LineMode, Object, Point};
use crate::table_records::ParameterGroup;

fn point_curve(objects: &[Object], kind: &Kind) -> bool {
    match kind {
        Kind::FunctionPlot(..) => true,
        Kind::Locus(refs, _) | Kind::ParameterLocus(refs, ..) => refs
            .first()
            .and_then(|id| id.checked_sub(1))
            .and_then(|i| objects.get(i))
            .is_some_and(|o| !o.deleted && is_point_kind(&o.kind)),
        _ => false,
    }
}
pub(super) fn supported(
    objects: &[Object],
    carrier: usize,
    curve: usize,
    page: usize,
    before: usize,
) -> bool {
    let Some(a) = carrier.checked_sub(1).and_then(|i| objects.get(i)) else {
        return false;
    };
    let Some(b) = curve.checked_sub(1).and_then(|i| objects.get(i)) else {
        return false;
    };
    carrier < before
        && curve < before
        && !a.deleted
        && !b.deleted
        && a.page == page
        && b.page == page
        && (is_straight_kind(&a.kind) || a.kind.is_circle() || is_arc_kind(&a.kind))
        && !matches!(a.kind, Kind::ToolInput(..))
        && point_curve(objects, &b.kind)
}
pub(super) fn decode(
    objects: &[Object],
    page: usize,
    id: usize,
    header: &[u8],
    refs: &[usize],
    payload: Option<&[u8]>,
) -> Option<Kind> {
    let [carrier, curve] = refs else {
        return None;
    };
    let payload = payload?;
    if header.len() != 28
        || header[..2] != [98, 0]
        || header[6..10] != [1, 0, 45, 0]
        || payload.len() != 8
        || payload[..4] != [98, 0, 0, 0]
        || !supported(objects, *carrier, *curve, page, id)
    {
        return None;
    }
    Some(Kind::CurveIntersection(
        *carrier,
        *curve,
        u32::from_le_bytes(payload[4..8].try_into().ok()?),
    ))
}

enum Source {
    Function {
        definition: usize,
        frame: crate::graph::CoordinateFrame,
        min: f64,
        max: f64,
    },
    Parameter {
        sample: Box<Document>,
        parameter: ParameterGroup,
        driver: usize,
        target: usize,
        min: f64,
        max: f64,
    },
    Geometric {
        sample: Box<Document>,
        driver: usize,
        path: usize,
        target: usize,
        kind: Kind,
    },
}
struct PointCurve<'a> {
    original: &'a Document,
    source: Source,
    intervals: usize,
}
impl<'a> PointCurve<'a> {
    fn new(doc: &'a Document, id: usize) -> Option<Self> {
        let object = doc.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let (source, intervals) = match &object.kind {
            Kind::FunctionPlot(definition, frame, min, max, samples) => {
                if !min.is_finite() || !max.is_finite() || min >= max || *samples < 2 {
                    return None;
                }
                (
                    Source::Function {
                        definition: *definition,
                        frame: doc.graph_frame(*frame)?,
                        min: *min,
                        max: *max,
                    },
                    samples.saturating_sub(1) as usize,
                )
            }
            Kind::ParameterLocus(refs, min, max, samples) => {
                let (&target, &driver) = (refs.first()?, refs.get(1)?);
                let Kind::Parameter(parameter) = &doc.objects.get(driver.checked_sub(1)?)?.kind
                else {
                    return None;
                };
                if !min.is_finite()
                    || !max.is_finite()
                    || min >= max
                    || *samples < 2
                    || !is_point_kind(&doc.objects.get(target.checked_sub(1)?)?.kind)
                {
                    return None;
                }
                let mut sample = Box::new(doc.clone());
                sample.invalidate_evaluation_dependents(driver);
                (
                    Source::Parameter {
                        sample,
                        parameter: parameter.clone(),
                        driver,
                        target,
                        min: *min,
                        max: *max,
                    },
                    samples.saturating_sub(1) as usize,
                )
            }
            Kind::Locus(refs, samples) => {
                let (&target, &path, &driver) = (refs.first()?, refs.get(1)?, refs.get(2)?);
                let kind = doc.objects.get(driver.checked_sub(1)?)?.kind.clone();
                if *samples < 2
                    || !is_point_kind(&doc.objects.get(target.checked_sub(1)?)?.kind)
                    || !matches!(
                        kind,
                        Kind::PointOnSegment(..) | Kind::PointOnCircle(..) | Kind::PointOnArc(..)
                    )
                {
                    return None;
                }
                let mut sample = Box::new(doc.clone());
                sample.invalidate_evaluation_dependents(driver);
                (
                    Source::Geometric {
                        sample,
                        driver,
                        path,
                        target,
                        kind,
                    },
                    *samples as usize,
                )
            }
            _ => return None,
        };
        Some(Self {
            original: doc,
            source,
            intervals: intervals.clamp(64, 4096),
        })
    }
    fn at(&mut self, fraction: f64) -> Option<Point> {
        let point = match &mut self.source {
            Source::Function {
                definition,
                frame,
                min,
                max,
            } => {
                let x = *min + (*max - *min) * fraction;
                let value = frame.world(x, self.original.function_value(*definition, x)?)?;
                Point {
                    x: value.x,
                    y: value.y,
                }
            }
            Source::Parameter {
                sample,
                parameter,
                driver,
                target,
                min,
                max,
            } => {
                let value = (*min + (*max - *min) * fraction) / parameter.native_unit_scale();
                if !parameter.set_evaluation_value(value) {
                    return None;
                }
                sample.objects[*driver - 1].kind = Kind::Parameter(parameter.clone());
                sample.position(*target)?
            }
            Source::Geometric {
                sample,
                driver,
                path,
                target,
                kind,
            } => {
                sample.objects[*driver - 1].kind = match kind {
                    Kind::PointOnSegment(..) => Kind::PointOnSegment(*path, fraction),
                    Kind::PointOnArc(..) => Kind::PointOnArc(*path, fraction),
                    Kind::PointOnCircle(..) => {
                        let angle = fraction * std::f64::consts::TAU;
                        Kind::PointOnCircle(*path, angle.cos(), angle.sin())
                    }
                    _ => return None,
                };
                sample.position(*target)?
            }
        };
        (point.x.is_finite() && point.y.is_finite()).then_some(point)
    }
}

enum Carrier {
    Straight {
        a: Point,
        ux: f64,
        uy: f64,
        length: f64,
        mode: LineMode,
    },
    Round {
        center: Point,
        radius: f64,
        arc: Option<usize>,
    },
}
impl Carrier {
    fn new(doc: &Document, id: usize) -> Option<Self> {
        if let Some((a, b, mode)) = doc.line_points(id) {
            let length = a.distance(b);
            if length < 1e-12 || !length.is_finite() {
                return None;
            }
            return Some(Self::Straight {
                a,
                ux: (b.x - a.x) / length,
                uy: (b.y - a.y) / length,
                length,
                mode,
            });
        }
        if let Some((center, radius)) = doc.circle_points(id) {
            return (radius > 1e-12 && radius.is_finite()).then_some(Self::Round {
                center,
                radius,
                arc: None,
            });
        }
        let (center, radius, _, _, _) = doc.arc_geometry(id)?;
        (radius > 1e-12 && radius.is_finite()).then_some(Self::Round {
            center,
            radius,
            arc: Some(id),
        })
    }
    fn residual(&self, p: Point) -> f64 {
        match self {
            Self::Straight { a, ux, uy, .. } => (p.x - a.x) * uy - (p.y - a.y) * ux,
            Self::Round { center, radius, .. } => center.distance(p) - radius,
        }
    }
    fn parameter(&self, doc: &Document, p: Point) -> Option<f64> {
        match *self {
            Self::Straight {
                a,
                ux,
                uy,
                length,
                mode,
            } => {
                let t = (p.x - a.x) * ux + (p.y - a.y) * uy;
                let inside = match mode {
                    LineMode::Line => true,
                    LineMode::Ray => t >= -1e-8,
                    LineMode::Segment => t >= -1e-8 && t <= length + 1e-8,
                };
                inside.then_some(t)
            }
            Self::Round { center, arc, .. } => {
                let angle = (center.y - p.y).atan2(p.x - center.x);
                if let Some(id) = arc {
                    if !doc.circular_arc_contains(id, p) {
                        return None;
                    }
                    let (_, _, start, sweep, clockwise_y) = doc.arc_geometry(id)?;
                    let actual = if clockwise_y { angle } else { -angle };
                    Some(if sweep >= 0. {
                        (actual - start).rem_euclid(std::f64::consts::TAU)
                    } else {
                        (start - actual).rem_euclid(std::f64::consts::TAU)
                    })
                } else {
                    Some(0.)
                }
            }
        }
    }
}

impl Document {
    /// Ordered crossings. Curve tangencies and gaps do not create crossings.
    pub fn curve_intersections(&self, carrier: usize, curve: usize) -> Option<Vec<Point>> {
        let page = self.objects.get(curve.checked_sub(1)?)?.page;
        if !supported(&self.objects, carrier, curve, page, self.objects.len() + 1) {
            return None;
        }
        let carrier = Carrier::new(self, carrier)?;
        let mut curve = PointCurve::new(self, curve)?;
        let mut hits = Vec::<(f64, Point)>::new();
        let curve_order = matches!(carrier, Carrier::Round { arc: None, .. });
        let include_max = matches!(curve.source, Source::Parameter { .. });
        let mut record = |point: Point, curve_parameter: f64| {
            if carrier.residual(point).abs() > 1e-7 {
                return;
            }
            if let Some(parameter) = carrier.parameter(self, point) {
                if parameter.is_finite() && !hits.iter().any(|(_, old)| old.distance(point) <= 1e-7)
                {
                    hits.push((
                        if curve_order {
                            curve_parameter
                        } else {
                            parameter
                        },
                        point,
                    ));
                }
            }
        };
        let intervals = curve.intervals;
        let values: Vec<_> = (0..=intervals)
            .map(|i| {
                curve
                    .at(i as f64 / intervals as f64)
                    .map(|p| (p, carrier.residual(p)))
            })
            .collect();
        for i in 0..intervals {
            let (Some((_, left_value)), Some((_, right_value))) = (values[i], values[i + 1]) else {
                continue;
            };
            if !left_value.is_finite()
                || !right_value.is_finite()
                || left_value == 0.
                || right_value == 0.
                || left_value.is_sign_positive() == right_value.is_sign_positive()
            {
                continue;
            }
            let mut low = i as f64 / intervals as f64;
            let mut high = (i + 1) as f64 / intervals as f64;
            let mut low_value = left_value;
            let mut point = None;
            let mut parameter = low;
            for _ in 0..64 {
                let mid = (low + high) * 0.5;
                let Some(p) = curve.at(mid) else {
                    point = None;
                    break;
                };
                let value = carrier.residual(p);
                if !value.is_finite() {
                    point = None;
                    break;
                }
                point = Some(p);
                parameter = mid;
                if value.abs() <= 1e-10 {
                    break;
                }
                if value.is_sign_positive() == low_value.is_sign_positive() {
                    low = mid;
                    low_value = value;
                } else {
                    high = mid;
                }
            }
            if let Some(point) = point {
                record(point, parameter);
            }
        }
        // Function and geometric traces exclude the maximum; parameter traces
        // include both ends. All three include the minimum.
        if let Some((point, value)) = values[0] {
            if value == 0. {
                record(point, 0.);
            }
        }
        // A sampled zero is a crossing only when its two defined neighbors have
        // opposite signs. A touch or an overlapping span is not an intersection.
        for i in 1..intervals {
            let (Some((_, before)), Some((point, value)), Some((_, after))) =
                (values[i - 1], values[i], values[i + 1])
            else {
                continue;
            };
            if value == 0.
                && before != 0.
                && after != 0.
                && before.is_sign_positive() != after.is_sign_positive()
            {
                record(point, i as f64 / intervals as f64);
            }
        }
        if include_max && let Some((point, value)) = values[intervals] {
            if value == 0. {
                record(point, 1.);
            }
        }
        hits.sort_by(|a, b| a.0.total_cmp(&b.0));
        Some(hits.into_iter().map(|(_, point)| point).collect())
    }
    pub(super) fn curve_intersection_position(
        &self,
        carrier: usize,
        curve: usize,
        branch: u32,
    ) -> Option<Point> {
        let points = self.curve_intersections(carrier, curve)?;
        points
            .get((branch as usize).min(points.len().checked_sub(1)?))
            .copied()
    }
}
