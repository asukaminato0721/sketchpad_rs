use super::kind::{
    expected_locus_refs, expected_parameter_locus_refs, is_arc_kind, is_locus_curve_collection,
    is_locus_polygon_kind, is_locus_straight_kind, is_point_kind, parameter_locus_driver_profile,
};
use super::sampling_plan::SamplingPlan;
use super::transform_model::{ElementType, NativeTransformFlavor, NumericExpr, Transform};
use super::{Document, Kind, LineMode, Object, Point};
use crate::style_records::VisualStyle;

/// One independently evaluated member of a straight sweep. Rendering clips its
/// carrier and retains its style instead of joining differently coloured samples.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocusStroke {
    pub start: Point,
    pub end: Point,
    pub mode: LineMode,
    pub style: VisualStyle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocusSurface {
    pub points: Vec<Point>,
    pub style: VisualStyle,
}

fn circle_strokes(center: Point, radius: f64) -> Vec<(Point, Point, LineMode)> {
    let point = |step: usize| {
        let angle = step as f64 * std::f64::consts::TAU / 64.;
        Point {
            x: center.x + radius * angle.cos(),
            y: center.y + radius * angle.sin(),
        }
    };
    (0..64)
        .map(|index| (point(index), point(index + 1), LineMode::Segment))
        .collect()
}

/// Point-output loci are sampled as independent traces in a mapped scope.
/// A locus used as the driver's own path has a separate nested-domain contract.
pub(super) fn is_point_locus(objects: &[Object], kind: &Kind) -> bool {
    let refs = match kind {
        Kind::Locus(refs, _) | Kind::ParameterLocus(refs, ..) => refs,
        _ => return false,
    };
    let (Some(&target), Some(&path)) = (refs.first(), refs.get(1)) else {
        return false;
    };
    let (Some(target), Some(path)) = (target.checked_sub(1), path.checked_sub(1)) else {
        return false;
    };
    objects.get(target).is_some_and(|o| is_point_kind(&o.kind)) && objects.get(path).is_some()
}

/// A sampled family whose members can be replayed efficiently inside a mapped
/// scope. These must not be joined like a point trace: every sampled line or
/// circle is rendered as a separate member and may carry its own dynamic color.
pub(super) fn is_mapped_stroke_locus(objects: &[Object], kind: &Kind) -> bool {
    let target = match kind {
        Kind::Locus(refs, _) | Kind::ParameterLocus(refs, ..) => *refs.first().unwrap_or(&0),
        _ => return false,
    };
    target
        .checked_sub(1)
        .and_then(|index| objects.get(index))
        .is_some_and(|object| {
            !object.deleted
                && (is_locus_straight_kind(&object.kind)
                    || object.kind.is_circle()
                    || is_arc_kind(&object.kind))
        })
}

pub(super) fn is_mapped_polygon_locus(objects: &[Object], kind: &Kind) -> bool {
    let target = match kind {
        Kind::Locus(refs, _) | Kind::ParameterLocus(refs, ..) => *refs.first().unwrap_or(&0),
        _ => return false,
    };
    target
        .checked_sub(1)
        .and_then(|index| objects.get(index))
        .is_some_and(|object| {
            !object.deleted
                && (is_locus_polygon_kind(&object.kind)
                    || is_locus_surface_collection(objects, object.id))
        })
}

/// A sampled polygon family can itself be swept by a later locus. IDs must
/// descend at every level so malformed native cycles cannot recurse here.
pub(super) fn is_locus_surface_collection(objects: &[Object], id: usize) -> bool {
    let Some(object) = id.checked_sub(1).and_then(|index| objects.get(index)) else {
        return false;
    };
    let refs = match &object.kind {
        Kind::Locus(refs, _) | Kind::ParameterLocus(refs, ..) if !object.deleted => refs,
        _ => return false,
    };
    let Some(&target) = refs.first().filter(|&&target| target < id) else {
        return false;
    };
    target
        .checked_sub(1)
        .and_then(|index| objects.get(index))
        .is_some_and(|target| is_locus_polygon_kind(&target.kind))
        || is_locus_surface_collection(objects, target)
}

pub(super) fn locus_curve_flags(objects: &[Object], path: usize) -> Option<u32> {
    let source = objects.get(path.checked_sub(1)?)?;
    match &source.kind {
        Kind::FunctionPlot(..) | Kind::ParametricPlot(..) => Some(1),
        Kind::SampledMapped(definition) => {
            if definition.path() >= path {
                return None;
            }
            let path_object = objects.get(definition.path().checked_sub(1)?)?;
            if path_object.kind.is_circle() || path_object.kind.is_polygon() {
                Some(0)
            } else if matches!(
                path_object.kind,
                Kind::SampledMapped(_) | Kind::Locus(..) | Kind::ParameterLocus(..)
            ) {
                locus_curve_flags(objects, definition.path())
            } else {
                Some(1)
            }
        }
        Kind::ParameterLocus(..) => Some(1),
        Kind::Locus(refs, _) => {
            let inner_path = *refs.get(1)?;
            let inner_driver = *refs.get(2)?;
            if inner_driver >= path {
                return None;
            }
            Some(locus_driver_range_info_for_path(objects, inner_driver, inner_path)?.1)
        }
        _ => None,
    }
}

fn locus_driver_straight_mode(objects: &[Object], path: usize) -> Option<LineMode> {
    super::kind::locus_straight_source_mode(objects, path).or_else(|| {
        matches!(
            objects.get(path.checked_sub(1)?)?.kind,
            Kind::GraphAxis(_, _, 0 | 1 | 0x1_0000 | 0x1_0001)
        )
        .then_some(LineMode::Line)
    })
}

/// Native loci can name the full source segment while constraining their
/// driver to a shortened copy of that segment.  This is used to keep a sweep
/// away from a singular endpoint without changing its normalized domain.
fn compatible_shortened_segment_host(objects: &[Object], path: usize, host: usize) -> bool {
    if path == host {
        return true;
    }
    let Some((source_a, source_b)) =
        objects
            .get(path.saturating_sub(1))
            .and_then(|object| match object.kind {
                Kind::Segment(a, b) if !object.deleted => Some((a, b)),
                _ => None,
            })
    else {
        return false;
    };
    let Some((host_a, host_b)) =
        objects
            .get(host.saturating_sub(1))
            .and_then(|object| match object.kind {
                Kind::Segment(a, b) if !object.deleted => Some((a, b)),
                _ => None,
            })
    else {
        return false;
    };
    let is_inner = |point: usize, source: usize, center: usize| {
        objects.get(point.saturating_sub(1)).is_some_and(|object| {
            let Kind::Transformed(definition) = &object.kind else {
                return false;
            };
            if definition.source() != source
                || definition.element() != ElementType::Point
                || definition.wire_flavor() != NativeTransformFlavor::FixedDilation
            {
                return false;
            }
            matches!(
                definition.transform(),
                Transform::Dilate {
                    center: transform_center,
                    ratio: NumericExpr::Literal {
                        magnitude,
                        unit: crate::table_records::NumericUnit::Scalar,
                        scale,
                    },
                } if *transform_center == center
                    && *magnitude > 0.
                    && *magnitude < 1.
                    && *scale == 1.
            )
        })
    };
    (host_b == source_b && is_inner(host_a, source_b, source_a))
        || (host_a == source_b && is_inner(host_b, source_b, source_a))
        || (host_b == source_a && is_inner(host_a, source_a, source_b))
        || (host_a == source_a && is_inner(host_b, source_a, source_b))
}

/// Whether the driver's saved construction already uses the locus record's
/// path. Native loci may instead name another independent path and temporarily
/// place an existing point on it only while sampling the locus.
pub(super) fn locus_driver_uses_stored_path(
    objects: &[Object],
    driver: usize,
    path: usize,
) -> bool {
    let Some(moving) = objects.get(driver.saturating_sub(1)) else {
        return false;
    };
    match moving.kind {
        Kind::PointOnSegment(host, _) => compatible_shortened_segment_host(objects, path, host),
        Kind::PointOnRay(host, _)
        | Kind::PointOnLine(host, _)
        | Kind::PointOnCircle(host, ..)
        | Kind::PointOnArc(host, _)
        | Kind::PointOnCurve(host, _)
        | Kind::PointOnPolygon(host, ..)
        | Kind::PointOnSector(host, _)
        | Kind::PointOnCircularSegment(host, _) => host == path,
        _ => false,
    }
}

/// Resolve a locus driver against its explicit native path. Some native loci
/// retain a point's independent saved construction here; the locus record
/// supplies a different path constraint only while evaluating the sweep.
pub(super) fn locus_driver_range_info_for_path(
    objects: &[Object],
    driver: usize,
    path: usize,
) -> Option<(f64, u32)> {
    let moving = objects.get(driver.checked_sub(1)?)?;
    let host = objects.get(path.checked_sub(1)?)?;
    let uses_stored_path = locus_driver_uses_stored_path(objects, driver, path);
    let straight = locus_driver_straight_mode(objects, path);
    let inferred = || {
        if let Some(mode) = straight {
            return Some(match mode {
                LineMode::Segment => (1., 1),
                LineMode::Ray => (1., 2),
                LineMode::Line => (1., 3),
            });
        }
        if host.kind.is_circle() {
            Some((std::f64::consts::TAU, 0))
        } else if is_arc_kind(&host.kind) {
            Some((1., 1))
        } else if host.kind.is_polygon()
            || super::sector_boundary::supported_host(objects, path, moving.page, objects.len() + 1)
        {
            Some((1., 0))
        } else if super::curve_points::free_supported(objects, path, moving.page, objects.len() + 1)
        {
            Some((1., locus_curve_flags(objects, path)?))
        } else {
            None
        }
    };
    let info = match moving.kind {
        Kind::PointOnSegment(host, _)
            if straight == Some(LineMode::Segment)
                && compatible_shortened_segment_host(objects, path, host) =>
        {
            Some((1., 1))
        }
        Kind::PointOnRay(host, _) if host == path && straight == Some(LineMode::Ray) => {
            Some((1., 2))
        }
        Kind::PointOnLine(host, _) if host == path && straight == Some(LineMode::Line) => {
            Some((1., 3))
        }
        Kind::PointOnCircle(host, ..) if host == path && objects[path - 1].kind.is_circle() => {
            Some((std::f64::consts::TAU, 0))
        }
        Kind::PointOnArc(host, _) if host == path && is_arc_kind(&objects[path - 1].kind) => {
            Some((1., 1))
        }
        Kind::PointOnPolygon(host, ..) if host == path && objects[path - 1].kind.is_polygon() => {
            Some((1., 0))
        }
        Kind::PointOnSector(host, _)
            if host == path
                && super::sector_boundary::supported_host(objects, path, moving.page, driver) =>
        {
            Some((1., 0))
        }
        Kind::PointOnCircularSegment(host, _)
            if host == path
                && super::sector_boundary::supported_circular_segment_host(
                    objects,
                    path,
                    moving.page,
                    driver,
                ) =>
        {
            Some((1., 0))
        }
        Kind::PointOnCurve(host, _)
            if host == path
                && super::curve_points::free_supported(objects, path, moving.page, driver) =>
        {
            Some((1., locus_curve_flags(objects, path)?))
        }
        _ if is_point_kind(&moving.kind) && !uses_stored_path => inferred(),
        _ => None,
    };
    let valid_order = if !uses_stored_path {
        path != driver
    } else {
        path < driver
    };
    (!moving.deleted && !host.deleted && valid_order && moving.page == host.page).then_some(info?)
}

/// Imported sampled-circle drivers also retain a proved open display profile.
/// This metadata does not change the driver's periodic parameter domain.
pub(super) fn locus_driver_range_matches(
    objects: &[Object],
    driver: usize,
    path: usize,
    minimum: f64,
    maximum: f64,
    closure: u32,
) -> bool {
    let Some((expected_maximum, expected_closure)) =
        locus_driver_range_info_for_path(objects, driver, path)
    else {
        return false;
    };
    if !minimum.is_finite() || !maximum.is_finite() || minimum >= maximum {
        return false;
    }
    let straight = match objects[driver - 1].kind {
        Kind::PointOnRay(host, _) if host == path => Some(LineMode::Ray),
        Kind::PointOnLine(host, _) if host == path => Some(LineMode::Line),
        _ if !locus_driver_uses_stored_path(objects, driver, path) => {
            locus_driver_straight_mode(objects, path)
        }
        _ => None,
    };
    match straight {
        Some(LineMode::Ray) => return minimum >= 0. && closure == 2,
        Some(LineMode::Line) => return closure == 3,
        _ => {}
    }
    if minimum != 0. || (maximum - expected_maximum).abs() >= 1e-12 {
        return false;
    }
    if closure == expected_closure {
        return true;
    }
    closure == 1 && objects.get(driver.saturating_sub(1)).is_some_and(|moving| {
        let Kind::PointOnCurve(host, _) = moving.kind else { return false; };
        objects.get(host.saturating_sub(1)).is_some_and(|source| {
            matches!(&source.kind, Kind::SampledMapped(definition)
                if objects.get(definition.path().saturating_sub(1)).is_some_and(|path| path.kind.is_circle()))
        })
    })
}

impl Document {
    pub(super) fn locus_sampling_interval(&self, id: usize) -> Option<[f64; 2]> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::Locus(refs, _) = &object.kind else {
            return None;
        };
        let path = *refs.get(1)?;
        let driver = *refs.get(2)?;
        let (maximum, closure) = locus_driver_range_info_for_path(&self.objects, driver, path)?;
        let minimum = if closure == 3 { -1. } else { 0. };
        Some(object.locus_interval.unwrap_or([minimum, maximum]))
    }

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
            || is_locus_curve_collection(&self.objects, target)
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
            || samples < 2
        {
            return Err("Invalid parameter locus range or sketch".into());
        }
        let refs = expected_parameter_locus_refs(&self.objects, driver, target)
            .ok_or("Select a supported numeric driver and a dependent object")?;
        let scale = parameter_locus_driver_profile(&self.objects, driver)
            .ok_or("Select a supported numeric driver")?
            .scale();
        Ok(self.add(Kind::ParameterLocus(
            refs,
            min * scale,
            max * scale,
            samples,
        )))
    }

    fn sample_parameter_locus<T>(
        &self,
        refs: &[usize],
        min: f64,
        max: f64,
        samples: u32,
        subdivisions: usize,
        mut evaluate: impl FnMut(&Document, usize) -> Option<T>,
    ) -> Option<Vec<Option<T>>> {
        let (&target, &driver) = (refs.first()?, refs.get(1)?);
        let profile = parameter_locus_driver_profile(&self.objects, driver)?;
        if !min.is_finite() || !max.is_finite() || min >= max || samples < 2 {
            return None;
        }
        let count = subdivisions.clamp(2, 4096).min(samples as usize);
        let plan = SamplingPlan::new(self, driver, refs, None);
        let mut sample = self.clone_for_evaluation();
        sample.ensure_evaluation_context(self)?;
        let mut values = Vec::with_capacity(count);
        for index in 0..count {
            let fraction = index as f64 / (count - 1) as f64;
            let value = (min + (max - min) * fraction) / profile.scale();
            let updated = match &plan {
                Some(plan) => plan.number(&mut sample, value),
                None => sample.set_evaluation_number(driver, value),
            };
            if updated.is_none() {
                values.push(None);
                continue;
            }
            values.push(evaluate(&sample, target));
        }
        Some(values)
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
        let [minimum, maximum] = self.locus_sampling_interval(id)?;
        if samples < 2 {
            return None;
        }
        let steps = subdivisions.clamp(2, 1000).min(samples as usize);
        let plan = SamplingPlan::new(self, driver, refs, Some(path));
        let mut sample = self.clone_for_evaluation();
        let moving = self.objects.get(driver.checked_sub(1)?)?;
        let transient_driver = !locus_driver_uses_stored_path(&self.objects, driver, path);
        let mut nested_curve = (matches!(moving.kind, Kind::PointOnCurve(host, _) if host == path)
            || transient_driver
                && super::curve_points::free_supported(&self.objects, path, moving.page, driver))
        .then(|| super::point_curve::PointCurve::new(self, path))
        .flatten();
        if let Some(plan) = &plan {
            sample.freeze_scoped_evaluation_inputs(self, &plan.external)?;
        } else if nested_curve.is_some() {
            sample.ensure_evaluation_context(self)?;
        }
        if plan.is_none() {
            sample.invalidate_evaluation_dependents(driver);
        }
        let mut values = Vec::with_capacity(steps + 1);
        for step in 0..=steps {
            let fraction = step as f64 / steps as f64;
            let parameter = minimum + (maximum - minimum) * fraction;
            if let Some(curve) = &mut nested_curve {
                let Some(point) = curve.at_literal(parameter) else {
                    values.push(None);
                    continue;
                };
                match &plan {
                    Some(plan) => plan.point(&mut sample, point)?,
                    None => sample.set_evaluation_point(driver, point)?,
                }
            } else {
                let Some(kind) = self.locus_driver_kind_at(driver, path, parameter) else {
                    values.push(None);
                    continue;
                };
                match &plan {
                    Some(plan) => plan.kind(&mut sample, kind)?,
                    None => sample.objects[driver - 1].kind = kind,
                }
            }
            values.push(evaluate(&sample, target));
        }
        Some(values)
    }

    fn sample_any_locus<T>(
        &self,
        id: usize,
        subdivisions: usize,
        evaluate: impl FnMut(&Document, usize) -> Option<T>,
    ) -> Option<Vec<Option<T>>> {
        match &self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::ParameterLocus(refs, min, max, samples) => {
                self.sample_parameter_locus(refs, *min, *max, *samples, subdivisions, evaluate)
            }
            Kind::Locus(..) => self.sample_locus(id, subdivisions, evaluate),
            _ => None,
        }
    }

    /// Replay a geometric driver without modifying the original or losing polygon edge phase.
    pub(super) fn locus_driver_kind_at(
        &self,
        driver: usize,
        path: usize,
        parameter: f64,
    ) -> Option<Kind> {
        if !parameter.is_finite() {
            return None;
        }
        let kind = &self.objects.get(driver.checked_sub(1)?)?.kind;
        match kind {
            Kind::PointOnSegment(host, _)
                if compatible_shortened_segment_host(&self.objects, path, *host) =>
            {
                Some(Kind::PointOnSegment(*host, parameter))
            }
            Kind::PointOnRay(host, _) if *host == path => Some(Kind::PointOnRay(path, parameter)),
            Kind::PointOnLine(host, _) if *host == path => Some(Kind::PointOnLine(path, parameter)),
            Kind::PointOnCircle(host, ..) if *host == path => {
                Some(Kind::PointOnCircle(path, parameter.cos(), parameter.sin()))
            }
            Kind::PointOnArc(host, _) if *host == path => Some(Kind::PointOnArc(path, parameter)),
            Kind::PointOnCurve(host, _) if *host == path => {
                Some(Kind::PointOnCurve(path, parameter))
            }
            Kind::PointOnSector(host, _) if *host == path => {
                Some(Kind::PointOnSector(path, parameter))
            }
            Kind::PointOnCircularSegment(host, _) if *host == path => {
                Some(Kind::PointOnCircularSegment(path, parameter))
            }
            Kind::PointOnPolygon(host, ..) if *host == path => {
                self.locus_polygon_driver_kind_at(path, parameter)
            }
            _ if is_point_kind(kind)
                && !locus_driver_uses_stored_path(&self.objects, driver, path) =>
            {
                locus_driver_range_info_for_path(&self.objects, driver, path)?;
                match locus_driver_straight_mode(&self.objects, path) {
                    Some(LineMode::Segment) => Some(Kind::PointOnSegment(path, parameter)),
                    Some(LineMode::Ray) => Some(Kind::PointOnRay(path, parameter)),
                    Some(LineMode::Line) => Some(Kind::PointOnLine(path, parameter)),
                    None if self.objects[path - 1].kind.is_circle() => {
                        Some(Kind::PointOnCircle(path, parameter.cos(), parameter.sin()))
                    }
                    None if is_arc_kind(&self.objects[path - 1].kind) => {
                        Some(Kind::PointOnArc(path, parameter))
                    }
                    None if self.objects[path - 1].kind.is_polygon() => {
                        self.locus_polygon_driver_kind_at(path, parameter)
                    }
                    None if self.objects[path - 1].kind.is_sector() => {
                        Some(Kind::PointOnSector(path, parameter))
                    }
                    None if self.objects[path - 1].kind.is_circular_segment() => {
                        Some(Kind::PointOnCircularSegment(path, parameter))
                    }
                    None => Some(Kind::PointOnCurve(path, parameter)),
                }
            }
            _ => None,
        }
    }

    fn locus_polygon_driver_kind_at(&self, path: usize, parameter: f64) -> Option<Kind> {
        let vertices = self.polygon_points(path)?;
        let lengths: Vec<_> = vertices
            .iter()
            .enumerate()
            .map(|(i, point)| point.distance(vertices[(i + 1) % vertices.len()]))
            .collect();
        let perimeter: f64 = lengths.iter().sum();
        if !perimeter.is_finite() || perimeter <= 1e-9 {
            return None;
        }
        let mut remaining = parameter.rem_euclid(1.) * perimeter;
        for (edge, length) in lengths.into_iter().enumerate() {
            if length <= 1e-9 {
                continue;
            }
            if remaining <= length {
                return Some(Kind::PointOnPolygon(path, edge, remaining / length));
            }
            remaining -= length;
        }
        None
    }

    pub fn locus_styled_strokes(&self, id: usize, subdivisions: usize) -> Option<Vec<LocusStroke>> {
        let target = match &self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::Locus(refs, _) | Kind::ParameterLocus(refs, ..) => *refs.first()?,
            _ => return None,
        };
        let style = self
            .effective_visual_style(id)
            .or_else(|| self.effective_visual_style(target))?;
        self.locus_styled_strokes_with_style(id, subdivisions, style)
    }

    /// A carried collection owns its local sampling domain. Replay each outer
    /// scope independently, retaining its binding's stroke presentation.
    pub fn mapped_collection_strokes(
        &self,
        id: usize,
        subdivisions: usize,
    ) -> Option<Vec<LocusStroke>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationBinding(source, iteration) = object.kind else {
            return None;
        };
        let source_kind = &self.objects.get(source.checked_sub(1)?)?.kind;
        let stroke_locus = is_mapped_stroke_locus(&self.objects, source_kind);
        let point_locus = is_point_locus(&self.objects, source_kind);
        let sampled = matches!(source_kind, Kind::SampledMapped(..));
        let function_plot = matches!(source_kind, Kind::FunctionPlot(..));
        if object.deleted || (!stroke_locus && !point_locus && !sampled && !function_plot) {
            return None;
        }
        let style = self.effective_visual_style(id)?;
        Some(
            self.mapped_scoped_outputs(source, iteration, false, |sample, source| {
                if stroke_locus || point_locus {
                    sample.locus_styled_strokes_with_style(source, subdivisions, style)
                } else if function_plot {
                    Some(
                        sample
                            .function_plot_segments(source, subdivisions)?
                            .into_iter()
                            .map(|(start, end)| LocusStroke {
                                start,
                                end,
                                mode: LineMode::Segment,
                                style,
                            })
                            .collect(),
                    )
                } else {
                    Some(
                        sample
                            .sampled_mapped_strokes(source, subdivisions)?
                            .into_iter()
                            .map(|(start, end, mode)| LocusStroke {
                                start,
                                end,
                                mode,
                                style,
                            })
                            .collect(),
                    )
                }
            })?
            .into_iter()
            .flatten()
            .flatten()
            .collect(),
        )
    }

    fn locus_styled_strokes_with_style(
        &self,
        id: usize,
        subdivisions: usize,
        style: VisualStyle,
    ) -> Option<Vec<LocusStroke>> {
        let refs = match &self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::Locus(refs, _) | Kind::ParameterLocus(refs, ..) => refs,
            _ => return None,
        };
        let target = *refs.first()?;
        let target_kind = &self.objects.get(target.checked_sub(1)?)?.kind;
        if is_point_kind(target_kind) {
            let VisualStyle::Stroke { line, color } = style else {
                return None;
            };
            let driver = match &self.objects.get(id.checked_sub(1)?)?.kind {
                Kind::Locus(refs, _) => *refs.get(2)?,
                Kind::ParameterLocus(refs, ..) => *refs.get(1)?,
                _ => return None,
            };
            let sampled_color = if let Kind::Colorized(definition) = target_kind {
                let affected = self.dependency_index().ok()?.affected_by(driver);
                definition
                    .binding()
                    .native_operands()
                    .iter()
                    .any(|operand| affected.contains(operand))
            } else {
                false
            };
            let color = if matches!(target_kind, Kind::Colorized(_)) && !sampled_color {
                self.colorized_color(target)?
            } else {
                color
            };
            let samples = self.sample_any_locus(id, subdivisions, |sample, target| {
                let point = sample.position(target)?;
                let color = if sampled_color {
                    sample.colorized_color(target)?
                } else {
                    color
                };
                Some((point, color))
            })?;
            return Some(
                samples
                    .windows(2)
                    .filter_map(|pair| {
                        let ((start, _), (end, color)) = (pair[0]?, pair[1]?);
                        Some(LocusStroke {
                            start,
                            end,
                            mode: LineMode::Segment,
                            style: VisualStyle::Stroke { line, color },
                        })
                    })
                    .collect(),
            );
        }
        if !is_locus_straight_kind(target_kind)
            && !target_kind.is_circle()
            && !is_arc_kind(target_kind)
        {
            return None;
        }
        let VisualStyle::Stroke { line, color } = style else {
            return None;
        };
        let sampled_color = matches!(
            self.objects.get(target.checked_sub(1)?)?.kind,
            Kind::Colorized(_)
        );
        Some(
            self.sample_any_locus(id, subdivisions, |sample, target| {
                let color = if sampled_color {
                    sample.colorized_color(target)?
                } else {
                    color
                };
                let style = VisualStyle::Stroke { line, color };
                if is_locus_straight_kind(&sample.objects.get(target - 1)?.kind) {
                    let (start, end, mode) = sample.line_points(target)?;
                    return Some(vec![LocusStroke {
                        start,
                        end,
                        mode,
                        style,
                    }]);
                }
                if sample.objects.get(target - 1)?.kind.is_circle() {
                    let (center, radius) = sample.circle_points(target)?;
                    return Some(
                        circle_strokes(center, radius)
                            .into_iter()
                            .map(|(start, end, mode)| LocusStroke {
                                start,
                                end,
                                mode,
                                style,
                            })
                            .collect(),
                    );
                }
                Some(
                    sample
                        .arc_points(target, 64)?
                        .windows(2)
                        .map(|pair| LocusStroke {
                            start: pair[0],
                            end: pair[1],
                            mode: LineMode::Segment,
                            style,
                        })
                        .collect(),
                )
            })?
            .into_iter()
            .flatten()
            .flatten()
            .collect(),
        )
    }

    pub fn locus_strokes(
        &self,
        id: usize,
        subdivisions: usize,
    ) -> Option<Vec<(Point, Point, LineMode)>> {
        let refs = match &self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::Locus(refs, _) | Kind::ParameterLocus(refs, ..) => refs,
            _ => return None,
        };
        let target_kind = &self.objects.get(refs.first()?.checked_sub(1)?)?.kind;
        if is_locus_curve_collection(&self.objects, refs[0]) {
            let inner_subdivisions = subdivisions.isqrt().max(2);
            return Some(
                self.sample_any_locus(id, subdivisions, |sample, target| {
                    match &sample.objects.get(target.checked_sub(1)?)?.kind {
                        Kind::SampledMapped(_) => {
                            sample.sampled_mapped_strokes(target, inner_subdivisions)
                        }
                        Kind::Locus(..) | Kind::ParameterLocus(..) => {
                            sample.locus_strokes(target, inner_subdivisions)
                        }
                        _ => None,
                    }
                })?
                .into_iter()
                .flatten()
                .flatten()
                .collect(),
            );
        }
        if is_locus_straight_kind(target_kind) {
            return Some(
                self.sample_any_locus(id, subdivisions, |sample, target| {
                    sample.line_points(target)
                })?
                .into_iter()
                .flatten()
                .collect(),
            );
        }
        if target_kind.is_circle() {
            let circles = self.sample_any_locus(id, subdivisions, |sample, target| {
                sample.circle_points(target)
            })?;
            let mut strokes = Vec::new();
            for (center, radius) in circles.into_iter().flatten() {
                strokes.extend(circle_strokes(center, radius));
            }
            return Some(strokes);
        }
        if is_arc_kind(target_kind) || is_locus_polygon_kind(target_kind) {
            let polygons = is_locus_polygon_kind(target_kind);
            let paths = self.sample_any_locus(id, subdivisions, |sample, target| {
                if polygons {
                    sample.filled_region_points(target)
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
            self.sample_any_locus(id, subdivisions, |sample, target| sample.position(target))?;
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
        let refs = match &self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::Locus(refs, _) | Kind::ParameterLocus(refs, ..) => refs,
            _ => return None,
        };
        if !is_locus_polygon_kind(&self.objects.get(refs.first()?.checked_sub(1)?)?.kind) {
            return None;
        }
        self.sample_any_locus(id, subdivisions, |sample, target| {
            sample.filled_region_points(target)
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
        let target = match &self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::Locus(refs, _) | Kind::ParameterLocus(refs, ..) => *refs.first()?,
            _ => return None,
        };
        let polygons = self.sampled_locus_polygons(id, subdivisions)?;
        let mut surfaces: Vec<_> = polygons.iter().flatten().cloned().collect();
        if self
            .objects
            .get(target.checked_sub(1)?)?
            .kind
            .is_circle_interior()
        {
            return Some(surfaces);
        }
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

    pub fn locus_surfaces(&self, id: usize, subdivisions: usize) -> Option<Vec<LocusSurface>> {
        let target = match &self.objects.get(id.checked_sub(1)?)?.kind {
            Kind::Locus(refs, _) | Kind::ParameterLocus(refs, ..) => *refs.first()?,
            _ => return None,
        };
        if is_locus_surface_collection(&self.objects, target) {
            let inner_subdivisions = subdivisions.isqrt().max(2);
            return Some(
                self.sample_any_locus(id, subdivisions, |sample, target| {
                    sample.locus_surfaces(target, inner_subdivisions)
                })?
                .into_iter()
                .flatten()
                .flatten()
                .collect(),
            );
        }
        let samples = self.sample_any_locus(id, subdivisions, |sample, target| {
            let points = sample.filled_region_points(target)?;
            let style = sample.effective_visual_style(target)?;
            matches!(style, VisualStyle::Polygon { .. }).then_some((points, style))
        })?;
        let mut surfaces = samples
            .iter()
            .flatten()
            .map(|(points, style)| LocusSurface {
                points: points.clone(),
                style: *style,
            })
            .collect::<Vec<_>>();
        if self
            .objects
            .get(target.checked_sub(1)?)?
            .kind
            .is_circle_interior()
        {
            return Some(surfaces);
        }
        for pair in samples.windows(2) {
            let (Some((before, _)), Some((after, style))) = (&pair[0], &pair[1]) else {
                continue;
            };
            if before.len() != after.len() {
                return None;
            }
            for index in 0..before.len() {
                let next = (index + 1) % before.len();
                surfaces.push(LocusSurface {
                    points: vec![before[index], before[next], after[next], after[index]],
                    style: *style,
                });
            }
        }
        Some(surfaces)
    }

    pub fn mapped_collection_surfaces(
        &self,
        id: usize,
        subdivisions: usize,
    ) -> Option<Vec<LocusSurface>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::IterationBinding(source, iteration) = object.kind else {
            return None;
        };
        let source_kind = &self.objects.get(source.checked_sub(1)?)?.kind;
        if object.deleted || !is_mapped_polygon_locus(&self.objects, source_kind) {
            return None;
        }
        Some(
            self.mapped_scoped_outputs(source, iteration, false, |sample, source| {
                sample.locus_surfaces(source, subdivisions)
            })?
            .into_iter()
            .flatten()
            .flatten()
            .collect(),
        )
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
