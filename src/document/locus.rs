use super::kind::{
    expected_locus_refs, is_arc_kind, is_locus_polygon_kind, is_locus_straight_kind,
};
use super::{Document, Kind, LineMode, Point};

impl Document {
    pub fn construct_locus(&mut self, driver: usize, target: usize) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot construct a locus in this sketch safely".into());
        }
        let refs = expected_locus_refs(&self.objects, driver, target)
            .ok_or("Select a constrained point and a dependent object")?;
        let samples = if is_locus_polygon_kind(&self.objects[target - 1].kind) {
            125
        } else if is_locus_straight_kind(&self.objects[target - 1].kind)
            || self.objects[target - 1].kind.is_circle()
            || is_arc_kind(&self.objects[target - 1].kind)
        {
            250
        } else {
            500
        };
        Ok(self.add(Kind::Locus(refs, samples)))
    }
    pub fn locus_strokes(
        &self,
        id: usize,
        subdivisions: usize,
    ) -> Option<Vec<(Point, Point, LineMode)>> {
        let Kind::Locus(ref refs, samples) = self.objects.get(id.checked_sub(1)?)?.kind else {
            return None;
        };
        let (&target, &driver) = (refs.first()?, refs.get(2)?);
        let driver_kind = &self.objects.get(driver.checked_sub(1)?)?.kind;
        let straight_target =
            is_locus_straight_kind(&self.objects.get(target.checked_sub(1)?)?.kind);
        let circle_target = self.objects.get(target - 1)?.kind.is_circle();
        let arc_target = is_arc_kind(&self.objects.get(target - 1)?.kind);
        let polygon_target = is_locus_polygon_kind(&self.objects.get(target - 1)?.kind);
        let mut sample = self.clone();
        let steps = subdivisions.clamp(2, 1000).min(samples as usize);
        let mut segments = Vec::with_capacity(
            (steps + 1)
                * if circle_target || arc_target {
                    64
                } else if polygon_target {
                    3
                } else {
                    1
                },
        );
        let mut previous = None;
        let circle_units = circle_target.then(|| {
            (0..=64)
                .map(|index| {
                    let angle = index as f64 * std::f64::consts::TAU / 64.;
                    (angle.cos(), angle.sin())
                })
                .collect::<Vec<_>>()
        });
        for step in 0..=steps {
            let fraction = step as f64 / steps as f64;
            sample.objects.get_mut(driver - 1)?.kind = match driver_kind {
                Kind::PointOnSegment(_, _) => Kind::PointOnSegment(refs[1], fraction),
                Kind::PointOnCircle(_, _, _) => {
                    let angle = fraction * std::f64::consts::TAU;
                    Kind::PointOnCircle(refs[1], angle.cos(), angle.sin())
                }
                Kind::PointOnArc(_, _) => Kind::PointOnArc(refs[1], fraction),
                _ => return None,
            };
            if straight_target {
                let (start, end, mode) = sample.line_points(target)?;
                segments.push((start, end, mode));
            } else if let Some(units) = &circle_units {
                let (center, radius) = sample.circle_points(target)?;
                for pair in units.windows(2) {
                    let start = Point {
                        x: center.x + radius * pair[0].0,
                        y: center.y + radius * pair[0].1,
                    };
                    let end = Point {
                        x: center.x + radius * pair[1].0,
                        y: center.y + radius * pair[1].1,
                    };
                    segments.push((start, end, LineMode::Segment));
                }
            } else if arc_target {
                for pair in sample.arc_points(target, 64)?.windows(2) {
                    segments.push((pair[0], pair[1], LineMode::Segment));
                }
            } else if polygon_target {
                let points = sample.polygon_points(target)?;
                for index in 0..points.len() {
                    segments.push((
                        points[index],
                        points[(index + 1) % points.len()],
                        LineMode::Segment,
                    ));
                }
            } else {
                let current = sample.position(target)?;
                if let Some(start) = previous {
                    segments.push((start, current, LineMode::Segment));
                }
                previous = Some(current);
            }
        }
        Some(segments)
    }

    pub fn locus_polygons(&self, id: usize, subdivisions: usize) -> Option<Vec<Vec<Point>>> {
        let Kind::Locus(ref refs, samples) = self.objects.get(id.checked_sub(1)?)?.kind else {
            return None;
        };
        let (&target, &driver) = (refs.first()?, refs.get(2)?);
        if !is_locus_polygon_kind(&self.objects.get(target.checked_sub(1)?)?.kind) {
            return None;
        }
        let driver_kind = &self.objects.get(driver.checked_sub(1)?)?.kind;
        let steps = subdivisions.clamp(2, 1000).min(samples as usize);
        let mut sample = self.clone();
        let mut polygons = Vec::with_capacity(steps + 1);
        for step in 0..=steps {
            let fraction = step as f64 / steps as f64;
            sample.objects.get_mut(driver - 1)?.kind = match driver_kind {
                Kind::PointOnSegment(_, _) => Kind::PointOnSegment(refs[1], fraction),
                Kind::PointOnCircle(_, _, _) => {
                    let angle = fraction * std::f64::consts::TAU;
                    Kind::PointOnCircle(refs[1], angle.cos(), angle.sin())
                }
                Kind::PointOnArc(_, _) => Kind::PointOnArc(refs[1], fraction),
                _ => return None,
            };
            polygons.push(sample.polygon_points(target)?);
        }
        Some(polygons)
    }

    pub fn locus_polygon_surfaces(
        &self,
        id: usize,
        subdivisions: usize,
    ) -> Option<Vec<Vec<Point>>> {
        let polygons = self.locus_polygons(id, subdivisions)?;
        let mut surfaces = polygons.clone();
        for pair in polygons.windows(2) {
            let (before, after) = (&pair[0], &pair[1]);
            if before.len() != after.len() {
                return None;
            }
            for index in 0..before.len() {
                let next = (index + 1) % before.len();
                surfaces.push(vec![before[index], before[next], after[next], after[index]]);
            }
        }
        Some(surfaces)
    }

    pub fn locus_segments(&self, id: usize, subdivisions: usize) -> Option<Vec<(Point, Point)>> {
        Some(
            self.locus_strokes(id, subdivisions)?
                .into_iter()
                .map(|(start, end, _)| (start, end))
                .collect(),
        )
    }
}
