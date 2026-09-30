//! Presentation geometry for native segment congruence and direction marks.
use super::kind::is_segment_kind;
use super::{Document, Kind, LineMode, Object, Point};

pub(super) fn arrow_host_supported(
    objects: &[Object],
    host: usize,
    page: usize,
    before: usize,
) -> bool {
    host < before
        && host
            .checked_sub(1)
            .and_then(|index| objects.get(index))
            .is_some_and(|object| {
                !object.deleted
                    && object.page == page
                    && (is_segment_kind(&object.kind)
                        || matches!(object.kind, Kind::SampledMapped(_))
                            && super::curve_points::supported(objects, host, page, before))
            })
}

impl Document {
    /// Native ticks are fourteen units long and adjacent ticks are spaced six
    /// units along the host. Direction arrows use ten-unit arms. All dimensions
    /// are independent of host length.
    pub fn segment_marker_strokes(&self, id: usize) -> Option<Vec<[Point; 2]>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let (host, parameter, marker) = match object.kind {
            Kind::SegmentMarker {
                host,
                parameter,
                count,
            } if (1..=3).contains(&count) => (host, parameter, Ok(count)),
            Kind::SegmentArrow {
                host,
                parameter,
                forward,
            } => (host, parameter, Err(forward)),
            _ => return None,
        };
        if !parameter.is_finite() {
            return None;
        }
        let (center, tangent) = match marker {
            Ok(_) => self.segment_marker_frame(host, parameter)?,
            Err(_) => self.direction_arrow_frame(host, parameter)?,
        };
        let normal = Point {
            x: -tangent.y,
            y: tangent.x,
        };
        match marker {
            Ok(count) => Some(
                (0..count)
                    .map(|index| {
                        let offset = (f64::from(index) - f64::from(count - 1) / 2.) * 6.;
                        let at = Point {
                            x: center.x + tangent.x * offset,
                            y: center.y + tangent.y * offset,
                        };
                        [
                            Point {
                                x: at.x - normal.x * 7.,
                                y: at.y - normal.y * 7.,
                            },
                            Point {
                                x: at.x + normal.x * 7.,
                                y: at.y + normal.y * 7.,
                            },
                        ]
                    })
                    .collect(),
            ),
            Err(forward) => {
                let direction = if forward { 1. } else { -1. };
                let back = Point {
                    x: center.x - tangent.x * direction * 10.,
                    y: center.y - tangent.y * direction * 10.,
                };
                Some(vec![
                    [
                        center,
                        Point {
                            x: back.x + normal.x * 10.,
                            y: back.y + normal.y * 10.,
                        },
                    ],
                    [
                        center,
                        Point {
                            x: back.x - normal.x * 10.,
                            y: back.y - normal.y * 10.,
                        },
                    ],
                ])
            }
        }
    }

    fn segment_marker_frame(&self, host: usize, parameter: f64) -> Option<(Point, Point)> {
        let (start, end, LineMode::Segment) = self.line_points(host)? else {
            return None;
        };
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let length = dx.hypot(dy);
        if !length.is_finite() || length <= 1e-9 {
            return None;
        }
        Some((
            Point {
                x: start.x + dx * parameter,
                y: start.y + dy * parameter,
            },
            Point {
                x: dx / length,
                y: dy / length,
            },
        ))
    }

    fn direction_arrow_frame(&self, host: usize, parameter: f64) -> Option<(Point, Point)> {
        if let Some(frame) = self.segment_marker_frame(host, parameter) {
            return Some(frame);
        }
        if !matches!(
            self.objects.get(host.checked_sub(1)?)?.kind,
            Kind::SampledMapped(_)
        ) {
            return None;
        }
        let mut curve = super::point_curve::PointCurve::new(self, host)?;
        let center = curve.at_literal(parameter)?;
        let step = 1. / curve.intervals.max(64) as f64;
        for radius in 1..=8 {
            let offset = step * f64::from(radius);
            let first = (parameter - offset).max(0.);
            let last = (parameter + offset).min(1.);
            if first == last {
                continue;
            }
            let (Some(start), Some(end)) = (curve.at_literal(first), curve.at_literal(last)) else {
                continue;
            };
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let length = dx.hypot(dy);
            if length.is_finite() && length > 1e-9 {
                return Some((
                    center,
                    Point {
                        x: dx / length,
                        y: dy / length,
                    },
                ));
            }
        }
        None
    }
}
