//! Native code93 searches ordered carrier events strictly ahead of its origin.
use super::geometry::intersect_straight;
use super::kind::is_straight_kind;
use super::{Document, Kind, LineMode, Object, Point};

pub(super) fn supported(
    objects: &[Object],
    straight: usize,
    polygon: usize,
    page: usize,
    before: usize,
) -> bool {
    let Some(a) = straight.checked_sub(1).and_then(|i| objects.get(i)) else {
        return false;
    };
    let Some(b) = polygon.checked_sub(1).and_then(|i| objects.get(i)) else {
        return false;
    };
    straight < before
        && polygon < before
        && !a.deleted
        && !b.deleted
        && a.page == page
        && b.page == page
        && is_straight_kind(&a.kind)
        && !matches!(
            a.kind,
            Kind::GraphAxis(..)
                | Kind::CircleAxis(..)
                | Kind::DistanceAxis(..)
                | Kind::ToolInput(..)
        )
        && b.kind.is_polygon()
}

pub(super) fn decode(
    objects: &[Object],
    page: usize,
    id: usize,
    header: &[u8],
    refs: &[usize],
    constraint: Option<&[u8]>,
) -> Option<Kind> {
    let [straight, polygon] = refs else {
        return None;
    };
    (header.len() == 28
        && header[..2] == [93, 0]
        && header[6..10] == [1, 0, 45, 0]
        && constraint.is_none()
        && supported(objects, *straight, *polygon, page, id))
    .then_some(Kind::BoundaryIntersection(*straight, *polygon))
}

impl Document {
    /// Whether these live, same-page parents can define a directed boundary point.
    pub fn can_construct_boundary_intersection(&self, straight: usize, polygon: usize) -> bool {
        supported(
            &self.objects,
            straight,
            polygon,
            self.active_page,
            self.objects.len() + 1,
        )
    }
    pub(super) fn boundary_intersection(&self, straight: usize, polygon: usize) -> Option<Point> {
        let (a, b, _) = self.line_points(straight)?;
        let vertices = self.polygon_points(polygon)?;
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let length = dx.hypot(dy);
        if length < 1e-12 || !length.is_finite() || vertices.len() < 3 {
            return None;
        }
        let ux = dx / length;
        let uy = dy / length;
        let mut best: Option<(f64, Point)> = None;
        let mut event = |p: Point| {
            let distance = (p.x - a.x) * ux + (p.y - a.y) * uy;
            if distance > 0. && distance.is_finite() && best.is_none_or(|(old, _)| distance < old) {
                best = Some((distance, p));
            }
        };
        for i in 0..vertices.len() {
            let c = vertices[i];
            let d = vertices[(i + 1) % vertices.len()];
            if let Some(p) = intersect_straight((a, b, LineMode::Line), (c, d, LineMode::Segment)) {
                event(p);
            } else {
                // Collinear spans contribute their endpoints, never an arbitrary
                // point from the overlapping continuum. Includes retraced edges.
                let on_carrier = |p: Point| {
                    let vx = p.x - a.x;
                    let vy = p.y - a.y;
                    (vx * uy - vy * ux).abs() <= vx.hypot(vy).max(1.) * 1e-12
                };
                if on_carrier(c) && on_carrier(d) {
                    event(c);
                    event(d);
                }
            }
        }
        best.map(|(_, point)| point)
    }
}
