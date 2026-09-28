//! Presentation geometry for native three-point angle marks.
use super::{Document, Kind, Point};

impl Document {
    /// Open stroke paths for a supported angle mark. The three point references
    /// remain the semantic source; these samples are only for drawing/picking.
    pub fn angle_marker_strokes(&self, id: usize) -> Option<Vec<Vec<Point>>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::AngleMarker(first, vertex, last, class) = object.kind else {
            return None;
        };
        let (first, vertex, last) = (
            self.position(first)?,
            self.position(vertex)?,
            self.position(last)?,
        );
        let (ax, ay) = (first.x - vertex.x, first.y - vertex.y);
        let (bx, by) = (last.x - vertex.x, last.y - vertex.y);
        let (al, bl) = (ax.hypot(ay), bx.hypot(by));
        if !al.is_finite() || !bl.is_finite() || al <= 1e-9 || bl <= 1e-9 {
            return None;
        }
        let start = ay.atan2(ax);
        let sweep = ((ax / al) * (by / bl) - (ay / al) * (bx / bl))
            .atan2((ax / al) * (bx / bl) + (ay / al) * (by / bl));
        if !sweep.is_finite() || sweep.abs() <= 1e-9 {
            return None;
        }
        let base = (al.min(bl) * 0.12).clamp(10., 28.).min(al.min(bl) * 0.42);
        Some(
            (0..class)
                .map(|ring| {
                    let radius = base + f64::from(ring) * 4.;
                    (0..=24)
                        .map(|sample| {
                            let angle = start + sweep * f64::from(sample) / 24.;
                            Point {
                                x: vertex.x + radius * angle.cos(),
                                y: vertex.y + radius * angle.sin(),
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect(),
        )
    }
}
