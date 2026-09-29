//! Presentation geometry for native segment congruence and direction marks.
use super::{Document, Kind, LineMode, Point};

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
        let (start, end, LineMode::Segment) = self.line_points(host)? else {
            return None;
        };
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let length = dx.hypot(dy);
        if !length.is_finite() || length <= 1e-9 {
            return None;
        }
        let tangent = Point {
            x: dx / length,
            y: dy / length,
        };
        let normal = Point {
            x: -tangent.y,
            y: tangent.x,
        };
        let center = Point {
            x: start.x + dx * parameter,
            y: start.y + dy * parameter,
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
}
