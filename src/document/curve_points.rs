//! Native free (15/42) and numeric (95) point definitions on point traces.
use super::kind::is_point_kind;
use super::{Kind, Object};

pub(super) fn supported(objects: &[Object], path: usize, page: usize, before: usize) -> bool {
    let Some(object) = path.checked_sub(1).and_then(|i| objects.get(i)) else {
        return false;
    };
    if path >= before || object.deleted || object.page != page {
        return false;
    }
    let refs = match &object.kind {
        Kind::Locus(refs, _) | Kind::ParameterLocus(refs, ..) => refs,
        _ => return false,
    };
    refs.first()
        .and_then(|id| id.checked_sub(1))
        .and_then(|i| objects.get(i))
        .is_some_and(|target| {
            !target.deleted
                && target.page == page
                && target.id < path
                && is_point_kind(&target.kind)
        })
        && refs.iter().all(|id| {
            *id < path
                && id
                    .checked_sub(1)
                    .and_then(|i| objects.get(i))
                    .is_some_and(|parent| !parent.deleted && parent.page == page)
        })
}

/// Interactive function points use15/58; numeric95 admission remains separate.
pub(super) fn free_supported(objects: &[Object], path: usize, page: usize, before: usize) -> bool {
    supported(objects, path, page, before) || path.checked_sub(1).and_then(|i| objects.get(i)).is_some_and(|o| {
        path < before && !o.deleted && o.page == page
            && matches!(o.kind, Kind::FunctionPlot(_, _, min, max, samples) if min.is_finite() && max.is_finite() && min < max && samples >= 2)
            && o.kind.dependencies().iter().all(|id| *id < path && id.checked_sub(1).and_then(|i|objects.get(i)).is_some_and(|p| !p.deleted && p.page == page))
    })
}

pub(super) fn free_constraint_subtype(objects: &[Object], path: usize) -> Option<u8> {
    match &objects.get(path.checked_sub(1)?)?.kind {
        Kind::FunctionPlot(..) => Some(58),
        Kind::Locus(..) | Kind::ParameterLocus(..) => Some(42),
        _ => None,
    }
}

pub(super) fn decode(
    objects: &[Object],
    path: usize,
    page: usize,
    before: usize,
    header: &[u8],
    payload: &[u8],
) -> Option<Kind> {
    if header.len() != 28
        || header[..2] != [15, 0]
        || header[6..10] != [1, 0, 10, 0]
        || payload.len() != 12
        || free_constraint_subtype(objects, path).is_none_or(|subtype| payload[..4] != [15, 0, subtype, 0])
        || !free_supported(objects, path, page, before)
    {
        return None;
    }
    let t = f64::from_le_bytes(payload[4..12].try_into().ok()?);
    t.is_finite().then_some(Kind::PointOnCurve(path, t))
}

use super::point_curve::PointCurve;
use super::{Document, Point};
impl Document {
    pub fn point_on_curve(&self, path: usize, fraction: f64) -> Option<Point> {
        let object = self.objects.get(path.checked_sub(1)?)?;
        if !fraction.is_finite()
            || !free_supported(&self.objects, path, object.page, self.objects.len() + 1)
        {
            return None;
        }
        PointCurve::new(self, path)?.at_literal(fraction)
    }
    pub(super) fn plotted_curve_position(&self, path: usize, value: f64) -> Option<Point> {
        let object = self.objects.get(path.checked_sub(1)?)?;
        if !supported(&self.objects, path, object.page, self.objects.len() + 1) {
            return None;
        }
        let mut curve = PointCurve::new(self, path)?;
        let fraction = curve.parameter_fraction(value)?;
        if !(0.0..=1.0).contains(&fraction) {
            return None;
        }
        curve.at(fraction)
    }
    pub(super) fn curve_point_value(&self, path: usize, fraction: f64) -> Option<f64> {
        self.point_on_curve(path, fraction)?;
        let t = PointCurve::new(self, path)?.literal_fraction(fraction)?;
        match self.objects.get(path.checked_sub(1)?)?.kind {
            Kind::Locus(..) => Some(t),
            Kind::ParameterLocus(_, min, max, _) | Kind::FunctionPlot(_, _, min, max, _) => Some(min + (max - min) * t),
            _ => None,
        }
    }
    /// Nearest evaluated point; sampled chords bracket the search but never
    /// replace the actual dependent point calculation.
    pub fn curve_parameter(&self, path: usize, point: Point) -> Option<f64> {
        let object = self.objects.get(path.checked_sub(1)?)?;
        if !point.x.is_finite()
            || !point.y.is_finite()
            || !free_supported(&self.objects, path, object.page, self.objects.len() + 1)
        {
            return None;
        }
        let mut curve = PointCurve::new(self, path)?;
        let intervals = curve.intervals;
        let mut best: Option<(f64, f64)> = None;
        let mut interval = 0;
        let mut update = |t: f64, p: Point| {
            let distance = p.distance(point);
            if best.is_none_or(|(old_distance, old_t)| {
                distance < old_distance || distance == old_distance && t < old_t
            }) {
                best = Some((distance, t));
            }
        };
        let mut previous = curve.at_literal(0.);
        if let Some(p) = previous {
            update(0., p);
        }
        let mut chord_best = f64::INFINITY;
        for i in 0..intervals {
            let end = curve.at_literal((i + 1) as f64 / intervals as f64);
            if let Some(p) = end {
                update((i + 1) as f64 / intervals as f64, p);
            }
            if let (Some(a), Some(b)) = (previous, end) {
                let (dx, dy) = (b.x - a.x, b.y - a.y);
                let length = dx * dx + dy * dy;
                let fraction = if length > 1e-18 {
                    (((point.x - a.x) * dx + (point.y - a.y) * dy) / length).clamp(0., 1.)
                } else {
                    0.
                };
                let distance = Point {
                    x: a.x + fraction * dx,
                    y: a.y + fraction * dy,
                }
                .distance(point);
                if distance < chord_best {
                    chord_best = distance;
                    interval = i;
                }
            }
            previous = end;
        }
        let mut best = best?;
        let distance = |curve: &mut PointCurve<'_>, t: f64| {
            curve
                .at(t)
                .map(|p| p.distance(point))
                .unwrap_or(f64::INFINITY)
        };
        for i in interval.saturating_sub(1)..=(interval + 1).min(intervals - 1) {
            let (mut low, mut high) = (
                i as f64 / intervals as f64,
                (i + 1) as f64 / intervals as f64,
            );
            const RATIO: f64 = 0.6180339887498949;
            let (mut a, mut b) = (high - (high - low) * RATIO, low + (high - low) * RATIO);
            let (mut da, mut db) = (distance(&mut curve, a), distance(&mut curve, b));
            for _ in 0..64 {
                if da <= db {
                    high = b;
                    b = a;
                    db = da;
                    a = high - (high - low) * RATIO;
                    da = distance(&mut curve, a);
                } else {
                    low = a;
                    a = b;
                    da = db;
                    b = low + (high - low) * RATIO;
                    db = distance(&mut curve, b);
                }
            }
            for (d, t) in [(da, a), (db, b)] {
                if d < best.0 || d == best.0 && t < best.1 {
                    best = (d, t);
                }
            }
        }
        Some(best.1)
    }
}
