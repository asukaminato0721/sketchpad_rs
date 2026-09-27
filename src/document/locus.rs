use super::kind::{
    expected_locus_refs, expected_parameter_locus_refs, is_arc_kind, is_locus_polygon_kind,
    is_locus_straight_kind, is_point_kind,
};
use super::{Document, Kind, LineMode, Point};

impl Document {
    pub fn construct_locus(&mut self, driver: usize, target: usize) -> Result<usize, String> {
        if self.partial_view || self.page_count != 1 {
            return Err("Cannot construct a locus in this sketch safely".into());
        }
        if let Some(Kind::Parameter(group)) =
            self.objects.get(driver.saturating_sub(1)).map(|o| &o.kind)
        {
            let (min, max) = if group.unit() == crate::table_records::NumericUnit::Degrees {
                group.animation_domain()
            } else {
                (0., 1.)
            };
            return self.construct_parameter_locus(driver, target, min, max, 500);
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
    pub fn construct_parameter_locus(
        &mut self,
        driver: usize,
        target: usize,
        min: f64,
        max: f64,
        samples: u32,
    ) -> Result<usize, String> {
        if self.partial_view
            || self.page_count != 1
            || !min.is_finite()
            || !max.is_finite()
            || min >= max
            || !(2..=4096).contains(&samples)
        {
            return Err("Invalid parameter locus range or sketch".into());
        }
        let refs = expected_parameter_locus_refs(&self.objects, driver, target)
            .ok_or("Select a parameter and a dependent point")?;
        let Kind::Parameter(group) = &self.objects[driver - 1].kind else {
            unreachable!()
        };
        let scale = group.native_unit_scale();
        Ok(self.add(Kind::ParameterLocus(
            refs,
            min * scale,
            max * scale,
            samples,
        )))
    }

    fn parameter_locus_strokes(
        &self,
        refs: &[usize],
        min: f64,
        max: f64,
        samples: u32,
        subdivisions: usize,
    ) -> Option<Vec<(Point, Point, LineMode)>> {
        let (&target, &driver) = (refs.first()?, refs.get(1)?);
        let Kind::Parameter(parameter) = &self.objects.get(driver.checked_sub(1)?)?.kind else {
            return None;
        };
        if !min.is_finite() || !max.is_finite() || min >= max || !(2..=4096).contains(&samples) {
            return None;
        }
        let count = subdivisions.clamp(2, 4096).min(samples as usize);
        let mut sample = self.clone();
        let mut parameter = parameter.clone();
        let mut points = Vec::with_capacity(count);
        for index in 0..count {
            let fraction = index as f64 / (count - 1) as f64;
            let value = (min + (max - min) * fraction) / parameter.native_unit_scale();
            if !parameter.set_evaluation_value(value) {
                points.push(None);
                continue;
            }
            sample.objects[driver - 1].kind = Kind::Parameter(parameter.clone());
            points.push(sample.position(target));
        }
        Some(
            points
                .windows(2)
                .filter_map(|pair| Some((pair[0]?, pair[1]?, LineMode::Segment)))
                .collect(),
        )
    }

    // Keep sample slots even when the target is undefined. Neighboring valid
    // samples may be joined, but a missing sample must break a curve or sweep.
    fn sample_locus<T>(
        &self,
        id: usize,
        subdivisions: usize,
        mut evaluate: impl FnMut(&Document, usize) -> Option<T>,
    ) -> Option<Vec<Option<T>>> {
        let Kind::Locus(ref refs, samples) = self.objects.get(id.checked_sub(1)?)?.kind else {
            return None;
        };
        let (&target, &path, &driver) = (refs.first()?, refs.get(1)?, refs.get(2)?);
        self.objects.get(target.checked_sub(1)?)?;
        let driver_kind = &self.objects.get(driver.checked_sub(1)?)?.kind;
        if !matches!(
            driver_kind,
            Kind::PointOnSegment(..) | Kind::PointOnCircle(..) | Kind::PointOnArc(..)
        ) || samples < 2
        {
            return None;
        }
        let steps = subdivisions.clamp(2, 1000).min(samples as usize);
        let mut sample = self.clone();
        let mut values = Vec::with_capacity(steps + 1);
        for step in 0..=steps {
            let fraction = step as f64 / steps as f64;
            sample.objects[driver - 1].kind = match driver_kind {
                Kind::PointOnSegment(..) => Kind::PointOnSegment(path, fraction),
                Kind::PointOnCircle(..) => {
                    let angle = fraction * std::f64::consts::TAU;
                    Kind::PointOnCircle(path, angle.cos(), angle.sin())
                }
                Kind::PointOnArc(..) => Kind::PointOnArc(path, fraction),
                _ => unreachable!(),
            };
            values.push(evaluate(&sample, target));
        }
        Some(values)
    }

    pub fn locus_strokes(
        &self,
        id: usize,
        subdivisions: usize,
    ) -> Option<Vec<(Point, Point, LineMode)>> {
        if let Kind::ParameterLocus(ref refs, min, max, samples) =
            self.objects.get(id.checked_sub(1)?)?.kind
        {
            return self.parameter_locus_strokes(refs, min, max, samples, subdivisions);
        }
        let Kind::Locus(ref refs, _) = self.objects.get(id.checked_sub(1)?)?.kind else {
            return None;
        };
        let target_kind = &self.objects.get(refs.first()?.checked_sub(1)?)?.kind;
        if is_locus_straight_kind(target_kind) {
            return Some(
                self.sample_locus(id, subdivisions, |sample, target| {
                    sample.line_points(target)
                })?
                .into_iter()
                .flatten()
                .collect(),
            );
        }
        if target_kind.is_circle() {
            let circles = self.sample_locus(id, subdivisions, |sample, target| {
                sample.circle_points(target)
            })?;
            let units: Vec<_> = (0..=64)
                .map(|index| {
                    let angle = index as f64 * std::f64::consts::TAU / 64.;
                    (angle.cos(), angle.sin())
                })
                .collect();
            let mut strokes = Vec::new();
            for (center, radius) in circles.into_iter().flatten() {
                for pair in units.windows(2) {
                    let point = |unit: (f64, f64)| Point {
                        x: center.x + radius * unit.0,
                        y: center.y + radius * unit.1,
                    };
                    strokes.push((point(pair[0]), point(pair[1]), LineMode::Segment));
                }
            }
            return Some(strokes);
        }
        if is_arc_kind(target_kind) || is_locus_polygon_kind(target_kind) {
            let polygons = is_locus_polygon_kind(target_kind);
            let paths = self.sample_locus(id, subdivisions, |sample, target| {
                if polygons {
                    sample.polygon_points(target)
                } else {
                    sample.arc_points(target, 64)
                }
            })?;
            let mut strokes = Vec::new();
            for points in paths.into_iter().flatten() {
                for pair in points.windows(2) {
                    strokes.push((pair[0], pair[1], LineMode::Segment));
                }
                if polygons && let (Some(&last), Some(&first)) = (points.last(), points.first()) {
                    strokes.push((last, first, LineMode::Segment));
                }
            }
            return Some(strokes);
        }
        if !is_point_kind(target_kind) {
            return None;
        }
        let points =
            self.sample_locus(id, subdivisions, |sample, target| sample.position(target))?;
        Some(
            points
                .windows(2)
                .filter_map(|pair| Some((pair[0]?, pair[1]?, LineMode::Segment)))
                .collect(),
        )
    }

    fn sampled_locus_polygons(
        &self,
        id: usize,
        subdivisions: usize,
    ) -> Option<Vec<Option<Vec<Point>>>> {
        let Kind::Locus(ref refs, _) = self.objects.get(id.checked_sub(1)?)?.kind else {
            return None;
        };
        if !is_locus_polygon_kind(&self.objects.get(refs.first()?.checked_sub(1)?)?.kind) {
            return None;
        }
        self.sample_locus(id, subdivisions, |sample, target| {
            sample.polygon_points(target)
        })
    }

    pub fn locus_polygons(&self, id: usize, subdivisions: usize) -> Option<Vec<Vec<Point>>> {
        Some(
            self.sampled_locus_polygons(id, subdivisions)?
                .into_iter()
                .flatten()
                .collect(),
        )
    }

    pub fn locus_polygon_surfaces(
        &self,
        id: usize,
        subdivisions: usize,
    ) -> Option<Vec<Vec<Point>>> {
        let polygons = self.sampled_locus_polygons(id, subdivisions)?;
        let mut surfaces: Vec<_> = polygons.iter().flatten().cloned().collect();
        for pair in polygons.windows(2) {
            let (Some(before), Some(after)) = (&pair[0], &pair[1]) else {
                continue;
            };
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
