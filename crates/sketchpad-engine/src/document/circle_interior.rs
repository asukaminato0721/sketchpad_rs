//! An intrinsic filled disk retaining its own identity and live Circle source.
//! Sampled outlines are presentation values, never stored polygon vertices.
use super::{Document, Kind, Object, Point};

pub(super) fn supported_source(
    objects: &[Object],
    source: usize,
    page: usize,
    before: usize,
) -> bool {
    source
        .checked_sub(1)
        .and_then(|i| objects.get(i))
        .is_some_and(|o| source < before && !o.deleted && o.page == page && o.kind.is_circle())
}

impl Document {
    pub fn add_circle_interior(&mut self, source: usize) -> Result<usize, String> {
        if (self.partial_view && self.page_count == 1)
            || (!self.safe_structure && self.page_count == 1)
            || !supported_source(
                &self.objects,
                source,
                self.active_page,
                self.objects.len() + 1,
            )
        {
            return Err("Select a supported circle in this page".into());
        }
        self.checkpoint();
        Ok(self.append_object(Kind::CircleInterior(source)))
    }

    /// Analytic disk geometry; source visibility does not control the interior.
    pub fn circle_interior_geometry(&self, id: usize) -> Option<(Point, f64)> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let Kind::CircleInterior(source) = object.kind else {
            return None;
        };
        if let Some(value) = self.evaluation_circle(id) {
            return value;
        }
        let (center, radius) = self.circle_points(source)?;
        (center.x.is_finite() && center.y.is_finite() && radius.is_finite() && radius > 0.)
            .then_some((center, radius))
    }

    pub fn circle_interior_contains(&self, id: usize, point: Point) -> bool {
        if !point.x.is_finite() || !point.y.is_finite() {
            return false;
        }
        self.circle_interior_geometry(id)
            .is_some_and(|(c, r)| (point.x - c.x).hypot(point.y - c.y) <= r)
    }

    /// An open vertex loop for fill triangulation; the renderer closes it.
    pub fn circle_interior_points(&self, id: usize, subdivisions: usize) -> Option<Vec<Point>> {
        let (center, radius) = self.circle_interior_geometry(id)?;
        let n = subdivisions.clamp(3, 65536);
        Some(
            (0..n)
                .map(|i| {
                    let angle = std::f64::consts::TAU * i as f64 / n as f64;
                    Point {
                        x: center.x + radius * angle.cos(),
                        y: center.y + radius * angle.sin(),
                    }
                })
                .collect(),
        )
    }
}
