//! Intrinsic closed sector boundary. The live Arc dependency remains the model;
//! sampled outlines are presentation values, not stored polygon vertices.
use super::kind::is_arc_kind;
use super::transform_model::ElementType;
use super::transform_evaluator::TransformGeometry;
use super::{Document, Kind, Object, Point};

pub(super) type SectorArc = (Point, f64, f64, f64, bool);

pub(super) fn supported_source(objects: &[Object], arc: usize, page: usize, before: usize) -> bool {
    arc.checked_sub(1)
        .and_then(|i| objects.get(i))
        .is_some_and(|o| arc < before && !o.deleted && o.page == page && is_arc_kind(&o.kind))
}
pub(super) fn supported_host(objects: &[Object], host: usize, page: usize, before: usize) -> bool {
    host.checked_sub(1)
        .and_then(|i| objects.get(i))
        .is_some_and(|o| {
            host < before
                && !o.deleted
                && o.page == page
                && o.kind.is_sector()
        })
}
fn arc_at((center, radius, start, sweep, clockwise_y): SectorArc, t: f64) -> Point {
    let angle = start + sweep * t;
    Point {
        x: center.x + radius * angle.cos(),
        y: center.y
            + if clockwise_y {
                -radius * angle.sin()
            } else {
                radius * angle.sin()
            },
    }
}
fn segment_projection(a: Point, b: Point, p: Point) -> (f64, Point) {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / (dx * dx + dy * dy)).clamp(0., 1.);
    (
        t,
        Point {
            x: a.x + t * dx,
            y: a.y + t * dy,
        },
    )
}
fn arc_projection(geometry: SectorArc, p: Point) -> f64 {
    let (center, _, start, sweep, clockwise_y) = geometry;
    let angle = (if clockwise_y {
        center.y - p.y
    } else {
        p.y - center.y
    })
    .atan2(p.x - center.x);
    let progress = if sweep >= 0. {
        (angle - start).rem_euclid(std::f64::consts::TAU)
    } else {
        (start - angle).rem_euclid(std::f64::consts::TAU)
    };
    if progress <= sweep.abs() {
        progress / sweep.abs()
    } else if p.distance(arc_at(geometry, 0.)) <= p.distance(arc_at(geometry, 1.)) {
        0.
    } else {
        1.
    }
}
impl Document {
    /// Add a live sector even when its structurally valid Arc is undefined.
    pub fn add_sector_boundary(&mut self, arc: usize) -> Result<usize, String> {
        if !supported_source(&self.objects, arc, self.active_page, self.objects.len() + 1) {
            return Err("Sector boundary requires a live Arc on this page".into());
        }
        Ok(self.add(Kind::SectorBoundary(arc)))
    }
    /// Native retains finite parameters outside [0,1] as undefined points.
    pub fn add_point_on_sector(&mut self, host: usize, t: f64) -> Result<usize, String> {
        if !t.is_finite()
            || !supported_host(
                &self.objects,
                host,
                self.active_page,
                self.objects.len() + 1,
            )
        {
            return Err("Sector point requires a live sector and a finite parameter".into());
        }
        Ok(self.add(Kind::PointOnSector(host, t)))
    }
    pub(super) fn sector_arc_geometry(&self, id: usize) -> Option<SectorArc> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        if object
            .kind
            .transform_definition()
            .is_some_and(|definition| definition.element() == ElementType::Sector)
        {
            return match self.evaluate_transform(id).ok()? {
                TransformGeometry::Sector {
                    center,
                    radius,
                    start,
                    sweep,
                    clockwise_y,
                } => Some((center, radius, start, sweep, clockwise_y)),
                _ => None,
            };
        }
        let Kind::SectorBoundary(source) = object.kind else {
            return None;
        };
        if let Some(value) = self.evaluation_arc(id) {
            return value;
        }
        let geometry = self.arc_geometry(source)?;
        let (c, r, a, s, _) = geometry;
        (c.x.is_finite()
            && c.y.is_finite()
            && r.is_finite()
            && r > 0.
            && a.is_finite()
            && s.is_finite()
            && s.abs() > 0.)
            .then_some(geometry)
    }
    pub(super) fn undefined_sector(&self, id: usize) -> bool {
        self.objects
            .get(id.wrapping_sub(1))
            .is_some_and(|o| !o.deleted && o.kind.is_sector())
            && self.sector_arc_geometry(id).is_none()
    }
    /// Native Point68 traverses center→start→arc→center by physical perimeter.
    pub fn point_on_sector(&self, id: usize, t: f64) -> Option<Point> {
        if !t.is_finite() || !(0.0..=1.).contains(&t) {
            return None;
        }
        let g = self.sector_arc_geometry(id)?;
        let (c, r, _, s, _) = g;
        let arc = r * s.abs();
        let distance = t * (2. * r + arc);
        if distance <= r {
            let start = arc_at(g, 0.);
            let q = distance / r;
            Some(Point {
                x: c.x + (start.x - c.x) * q,
                y: c.y + (start.y - c.y) * q,
            })
        } else if distance <= r + arc {
            Some(arc_at(g, (distance - r) / arc))
        } else {
            let end = arc_at(g, 1.);
            let q = ((distance - r - arc) / r).clamp(0., 1.);
            Some(Point {
                x: end.x + (c.x - end.x) * q,
                y: end.y + (c.y - end.y) * q,
            })
        }
    }
    /// Exact nearest boundary parameter; presentation subdivisions do not affect dragging.
    pub fn sector_boundary_projection(&self, id: usize, p: Point) -> Option<f64> {
        if !p.x.is_finite() || !p.y.is_finite() {
            return None;
        }
        let g = self.sector_arc_geometry(id)?;
        let (c, r, _, s, _) = g;
        let length = r * s.abs();
        let perimeter = 2. * r + length;
        let (first, fp) = segment_projection(c, arc_at(g, 0.), p);
        let arc = arc_projection(g, p);
        let ap = arc_at(g, arc);
        let (last, lp) = segment_projection(arc_at(g, 1.), c, p);
        [
            (first * r / perimeter, fp),
            ((r + arc * length) / perimeter, ap),
            ((r + length + last * r) / perimeter, lp),
        ]
        .into_iter()
        .min_by(|(ta, a), (tb, b)| {
            a.distance(p)
                .total_cmp(&b.distance(p))
                .then_with(|| ta.total_cmp(tb))
        })
        .map(|(t, _)| t)
    }
    /// Closed outline for fill/stroke presentation. Analytic queries remain exact.
    pub fn sector_boundary_points(&self, id: usize, subdivisions: usize) -> Option<Vec<Point>> {
        let g = self.sector_arc_geometry(id)?;
        let n = subdivisions.clamp(2, 65536);
        let mut out = Vec::with_capacity(n + 3);
        out.push(g.0);
        out.extend((0..=n).map(|i| arc_at(g, i as f64 / n as f64)));
        out.push(g.0);
        Some(out)
    }
    /// Sampled presentation outline for filled shapes; this does not change path typing.
    pub fn filled_region_points(&self, id: usize) -> Option<Vec<Point>> {
        if self
            .objects
            .get(id.checked_sub(1)?)
            .is_some_and(|o| matches!(o.kind, Kind::CircleInterior(_)))
        {
            return self.circle_interior_points(id, 192);
        }
        if self
            .objects
            .get(id.checked_sub(1)?)
            .is_some_and(|o| o.kind.is_sector())
        {
            let mut points = self.sector_boundary_points(id, 192)?;
            // Polygon ear clipping closes the vertex loop itself. A repeated center
            // blocks the final ear and leaves the central radial triangle unfilled.
            points.pop();
            Some(points)
        } else {
            self.polygon_points(id)
        }
    }
    pub fn sector_perimeter_pixels(&self, id: usize) -> Option<f64> {
        let (_, r, _, s, _) = self.sector_arc_geometry(id)?;
        Some(r * (2. + s.abs()))
    }
    pub fn sector_arc_length_pixels(&self, id: usize) -> Option<f64> {
        let (_, r, _, s, _) = self.sector_arc_geometry(id)?;
        Some(r * s.abs())
    }
    pub fn sector_area_pixels2(&self, id: usize) -> Option<f64> {
        let (_, r, _, s, _) = self.sector_arc_geometry(id)?;
        Some(0.5 * r * r * s.abs())
    }
    pub fn sector_contains(&self, id: usize, p: Point) -> bool {
        if !p.x.is_finite() || !p.y.is_finite() {
            return false;
        }
        let Some(g) = self.sector_arc_geometry(id) else {
            return false;
        };
        let (c, r, start, sweep, cw) = g;
        let distance = c.distance(p);
        if distance <= 1e-9 {
            return true;
        }
        if distance > r + 1e-9 {
            return false;
        }
        let a = (if cw { c.y - p.y } else { p.y - c.y }).atan2(p.x - c.x);
        let progress = if sweep >= 0. {
            (a - start).rem_euclid(std::f64::consts::TAU)
        } else {
            (start - a).rem_euclid(std::f64::consts::TAU)
        };
        progress <= sweep.abs() + 1e-10
    }
}
