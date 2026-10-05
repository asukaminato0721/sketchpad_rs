//! Native98 ranks crossings on an ordered carrier, after domain filtering.
use super::kind::{is_arc_kind, is_point_kind, is_straight_kind};
use super::object_store::ObjectRead;
use super::point_curve::PointCurve;
use super::{Document, Kind, LineMode, Point};
use crate::function_records::FunctionPlotMode;
use std::collections::BTreeSet;
use std::sync::Arc;

fn point_curve(objects: &(impl ObjectRead + ?Sized), kind: &Kind) -> bool {
    match kind {
        Kind::FunctionPlot(..) | Kind::ParametricPlot(..) | Kind::SampledMapped(_) => true,
        Kind::Locus(refs, _) | Kind::ParameterLocus(refs, ..) => refs
            .first()
            .and_then(|id| id.checked_sub(1))
            .and_then(|i| objects.get(i))
            .is_some_and(|o| !o.deleted && is_point_kind(&o.kind)),
        _ => false,
    }
}

fn function_plot_pair(objects: &(impl ObjectRead + ?Sized), first: usize, second: usize) -> bool {
    let Some(Kind::FunctionPlot(_, first_frame, first_min, first_max, _, first_mode)) = first
        .checked_sub(1)
        .and_then(|index| objects.get(index))
        .map(|object| &object.kind)
    else {
        return false;
    };
    let Some(Kind::FunctionPlot(_, second_frame, second_min, second_max, _, second_mode)) = second
        .checked_sub(1)
        .and_then(|index| objects.get(index))
        .map(|object| &object.kind)
    else {
        return false;
    };
    first_frame == second_frame
        && *first_mode == FunctionPlotMode::YFromX
        && *second_mode == FunctionPlotMode::YFromX
        && first_min.max(*second_min) < first_max.min(*second_max)
}

pub(super) fn supported(
    objects: &(impl ObjectRead + ?Sized),
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
        && (((is_straight_kind(&a.kind) || a.kind.is_circle() || is_arc_kind(&a.kind))
            && !matches!(a.kind, Kind::ToolInput(..))
            && point_curve(objects, &b.kind))
            || function_plot_pair(objects, carrier, curve))
}
pub(super) fn decode(
    objects: &(impl ObjectRead + ?Sized),
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
    fn function_plot_intersections(&self, first: usize, second: usize) -> Option<Vec<Point>> {
        let Kind::FunctionPlot(
            first_definition,
            frame,
            first_min,
            first_max,
            first_samples,
            FunctionPlotMode::YFromX,
        ) = self.objects.get(first.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let Kind::FunctionPlot(
            second_definition,
            second_frame,
            second_min,
            second_max,
            second_samples,
            FunctionPlotMode::YFromX,
        ) = self.objects.get(second.checked_sub(1)?)?.kind
        else {
            return None;
        };
        if frame != second_frame {
            return None;
        }
        self.function_expression(first_definition)?;
        self.function_expression(second_definition)?;
        let frame = self.graph_frame(frame)?;
        fn numeric_inputs(
            document: &Document,
            function: usize,
            seen: &mut BTreeSet<usize>,
            inputs: &mut BTreeSet<usize>,
        ) -> Option<()> {
            if !seen.insert(function) {
                return Some(());
            }
            for &source in document.function_sources(function)? {
                let object = document.objects.get(source.checked_sub(1)?)?;
                if object.kind.is_function() {
                    numeric_inputs(document, source, seen, inputs)?;
                } else if object.kind.numeric_unit().is_some() {
                    inputs.insert(source);
                } else {
                    return None;
                }
            }
            Some(())
        }
        let mut inputs = BTreeSet::new();
        let mut seen = BTreeSet::new();
        numeric_inputs(self, first_definition, &mut seen, &mut inputs)?;
        numeric_inputs(self, second_definition, &mut seen, &mut inputs)?;
        let mut evaluation = self.clone();
        evaluation
            .freeze_scoped_evaluation_inputs(self, &inputs.into_iter().collect::<Vec<_>>())?;
        let minimum = first_min.max(second_min);
        let maximum = first_max.min(second_max);
        if minimum >= maximum {
            return None;
        }
        let intervals = first_samples
            .max(second_samples)
            .saturating_sub(1)
            .clamp(64, 4096) as usize;
        let sample = |x: f64| {
            let first = evaluation.function_value(first_definition, x)?;
            let second = evaluation.function_value(second_definition, x)?;
            [first, second, first - second]
                .into_iter()
                .all(f64::is_finite)
                .then_some((first, second, first - second))
        };
        let samples: Vec<_> = (0..=intervals)
            .map(|index| {
                let x = minimum + (maximum - minimum) * index as f64 / intervals as f64;
                (x, sample(x))
            })
            .collect();
        let has_poles = self.function_may_have_poles(first_definition)
            || self.function_may_have_poles(second_definition);
        let bracket_crosses_gap =
            |left_x: f64, left: (f64, f64, f64), right_x: f64, right: (f64, f64, f64)| {
                has_poles
                    && [0.25, 0.5, 0.75].into_iter().any(|fraction| {
                        let x = left_x + (right_x - left_x) * fraction;
                        let Some((first, second, _)) = sample(x) else {
                            return true;
                        };
                        [(first, left.0, right.0), (second, left.1, right.1)]
                            .into_iter()
                            .any(|(value, a, b)| {
                                let low = a.min(b);
                                let high = a.max(b);
                                let tolerance = 1e-3 * high.abs().max(low.abs()).max(1.);
                                value < low - tolerance || value > high + tolerance
                            })
                    })
            };
        let mut hits = Vec::<(f64, Point)>::new();
        let mut record = |x: f64, first: f64, second: f64| {
            let value = frame.world(x, (first + second) * 0.5)?;
            let point = Point {
                x: value.x,
                y: value.y,
            };
            if !hits.iter().any(|(_, old)| old.distance(point) <= 1e-7) {
                hits.push((x, point));
            }
            Some(())
        };
        for pair in samples.windows(2) {
            let [(left_x, Some(left)), (right_x, Some(right))] = pair else {
                continue;
            };
            if left.2 == 0.
                || right.2 == 0.
                || left.2.is_sign_positive() == right.2.is_sign_positive()
                || bracket_crosses_gap(*left_x, *left, *right_x, *right)
            {
                continue;
            }
            let (mut low, mut high, mut low_value) = (*left_x, *right_x, left.2);
            let mut root = None;
            for _ in 0..64 {
                let x = (low + high) * 0.5;
                let Some(value) = sample(x) else {
                    root = None;
                    break;
                };
                root = Some((x, value.0, value.1));
                if value.2.abs() <= 1e-10 {
                    break;
                }
                if value.2.is_sign_positive() == low_value.is_sign_positive() {
                    low = x;
                    low_value = value.2;
                } else {
                    high = x;
                }
            }
            if let Some((x, first, second)) = root {
                record(x, first, second)?;
            }
        }
        if let (x, Some((first, second, difference))) = samples[0]
            && difference == 0.
        {
            record(x, first, second)?;
        }
        for window in samples.windows(3) {
            let [(_, Some(before)), (x, Some(value)), (_, Some(after))] = window else {
                continue;
            };
            if value.2 == 0.
                && before.2 != 0.
                && after.2 != 0.
                && before.2.is_sign_positive() != after.2.is_sign_positive()
            {
                record(*x, value.0, value.1)?;
            }
        }
        hits.sort_by(|a, b| a.0.total_cmp(&b.0));
        Some(hits.into_iter().map(|(_, point)| point).collect())
    }

    /// Ordered crossings. Curve tangencies and gaps do not create crossings.
    pub fn curve_intersections(&self, carrier: usize, curve: usize) -> Option<Vec<Point>> {
        self.shared_curve_intersections(carrier, curve)
            .map(|points| (*points).clone())
    }

    fn shared_curve_intersections(&self, carrier: usize, curve: usize) -> Option<Arc<Vec<Point>>> {
        use super::sac_evaluation::{Query, Value};
        if let Some(Value::Geometry(value)) =
            self.sac_query(Query::CurveIntersections(carrier, curve))
        {
            return value.and_then(|value| value.get());
        }
        self.curve_intersections_uncached(carrier, curve)
            .map(Arc::new)
    }

    pub(super) fn curve_intersections_uncached(
        &self,
        carrier: usize,
        curve: usize,
    ) -> Option<Vec<Point>> {
        let page = self.objects.get(curve.checked_sub(1)?)?.page;
        if !supported(&self.objects, carrier, curve, page, self.objects.len() + 1) {
            return None;
        }
        if function_plot_pair(&self.objects, carrier, curve) {
            return self.function_plot_intersections(carrier, curve);
        }
        let carrier = Carrier::new(self, carrier)?;
        let mut curve = PointCurve::new(self, curve)?.for_refinement();
        let mut hits = Vec::<(f64, Point)>::new();
        let curve_order = matches!(carrier, Carrier::Round { arc: None, .. });
        let include_max = curve.includes_maximum();
        let mut record = |point: Point, curve_parameter: f64| {
            if carrier.residual(point).abs() > 1e-7 {
                return;
            }
            if let Some(parameter) = carrier.parameter(self, point)
                && parameter.is_finite()
                && !hits.iter().any(|(_, old)| old.distance(point) <= 1e-7)
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
        };
        let intervals = curve.intervals;
        let values: Vec<_> = curve
            .sample_points()?
            .into_iter()
            .map(|point| point.map(|p| (p, carrier.residual(p))))
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
        if let Some((point, value)) = values[0]
            && value == 0.
        {
            record(point, 0.);
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
        if include_max
            && let Some((point, value)) = values[intervals]
            && value == 0.
        {
            record(point, 1.);
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
        let points = self.shared_curve_intersections(carrier, curve)?;
        points
            .get((branch as usize).min(points.len().checked_sub(1)?))
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branches_share_ordered_roots_and_keep_edits_and_snapshots_isolated() {
        let mut doc = Document::new();
        let center = doc.add(Kind::Free(Point { x: 0., y: 0. }));
        let edge = doc.add(Kind::Free(Point { x: 10., y: 0. }));
        let circle = doc.add(Kind::Circle(center, edge));
        let seed = doc.add(Kind::PointOnCircle(circle, 0.3, 0.));
        let target = doc.add(Kind::Midpoint(seed, center));
        let curve = doc.add(Kind::Locus(vec![target, circle, seed, target], 100));
        let a = doc.add(Kind::Free(Point { x: -20., y: 1. }));
        let b = doc.add(Kind::Free(Point { x: 20., y: 1. }));
        let carrier = doc.add(Kind::Line(a, b));
        let first = doc.add(Kind::CurveIntersection(carrier, curve, 0));
        let second = doc.add(Kind::CurveIntersection(carrier, curve, 1));
        let old = doc.evaluated();
        let first_point = old.position(first).unwrap();
        let before = old.sac_evaluation().unwrap().graph.statistics().executed;
        let second_point = old.position(second).unwrap();
        // Only the second branch point executes; its pair query is already cached.
        assert_eq!(
            old.sac_evaluation().unwrap().graph.statistics().executed - before,
            1
        );
        let roots = old.shared_curve_intersections(carrier, curve).unwrap();
        assert_eq!(first_point, roots[0]);
        assert_eq!(second_point, roots[1]);
        assert_eq!(roots.len(), 2);
        assert!(roots[0].x < roots[1].x);
        assert_eq!(old.position(first), Some(roots[0]));
        assert_eq!(old.position(second), Some(roots[1]));
        assert!(Arc::ptr_eq(
            &roots,
            &old.shared_curve_intersections(carrier, curve).unwrap()
        ));
        let mut sample = old.clone_for_evaluation();
        for (fraction, y, count) in [(0, 1., 2), (1, 20., 0), (0, 2., 2)] {
            old.with_sac_scope(
                &mut sample,
                super::super::sac_evaluation::Scope::MappedSample {
                    owner: curve,
                    fraction,
                },
                |sample| {
                    sample
                        .set_evaluation_point(a, Point { x: -20., y })
                        .unwrap();
                    sample.set_evaluation_point(b, Point { x: 20., y }).unwrap();
                    let first = sample.shared_curve_intersections(carrier, curve).unwrap();
                    let second = sample.shared_curve_intersections(carrier, curve).unwrap();
                    assert!(Arc::ptr_eq(&first, &second));
                    assert_eq!(first.len(), count);
                    assert!(first.iter().all(|point| (point.y - y).abs() < 1e-7));
                },
            );
        }
        doc.move_point(a, Point { x: -20., y: 20. });
        doc.move_point(b, Point { x: 20., y: 20. });
        assert!(
            doc.evaluated()
                .shared_curve_intersections(carrier, curve)
                .unwrap()
                .is_empty()
        );
        doc.move_point(edge, Point { x: 100., y: 0. });
        let updated = doc.evaluated();
        assert_eq!(
            updated
                .shared_curve_intersections(carrier, curve)
                .unwrap()
                .len(),
            2
        );
        assert_eq!(old.position(first), Some(roots[0]));
        assert_eq!(old.position(second), Some(roots[1]));
        doc.move_point(b, Point { x: -20., y: 20. });
        assert!(
            doc.evaluated()
                .shared_curve_intersections(carrier, curve)
                .is_none()
        );
    }
}
