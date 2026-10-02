//! Evaluated point traces shared by curve constructions.
use super::kind::is_point_kind;
use super::sampling_plan::SamplingPlan;
use super::{Document, Kind, Point};
use crate::function_records::FunctionPlotMode;
use crate::plot_points_records::PlotPointsMode;

enum Source {
    Mapped(super::sampled_mapped::SampledMappedEvaluator),
    Function {
        definition: usize,
        frame: crate::graph::CoordinateFrame,
        min: f64,
        max: f64,
        mode: FunctionPlotMode,
    },
    Parametric {
        x_definition: usize,
        y_definition: usize,
        frame: crate::graph::CoordinateFrame,
        min: f64,
        max: f64,
        mode: PlotPointsMode,
    },
    Numeric {
        sample: Box<Document>,
        plan: Option<std::sync::Arc<SamplingPlan>>,
        driver: usize,
        target: usize,
        min: f64,
        max: f64,
        scale: f64,
    },
    Geometric {
        sample: Box<Document>,
        plan: Option<std::sync::Arc<SamplingPlan>>,
        driver: usize,
        path: usize,
        target: usize,
        kind: Kind,
        interval: [f64; 2],
    },
}
pub(super) struct PointCurve<'a> {
    original: &'a Document,
    owner: usize,
    source: Source,
    pub(super) intervals: usize,
}
impl<'a> PointCurve<'a> {
    pub(super) fn sample_points(&mut self) -> Option<Vec<Option<Point>>> {
        self.original.cached_geometry(
            self.owner,
            super::geometry_cache::GeometryQuery::CurveSamples,
            self.intervals,
            || Some(self.sample_points_uncached(self.intervals)),
        )
    }

    pub(super) fn sample_points_uncached(&mut self, intervals: usize) -> Vec<Option<Point>> {
        (0..=intervals)
            .map(|i| self.at(i as f64 / intervals as f64))
            .collect()
    }

    pub(super) fn includes_maximum(&self) -> bool {
        matches!(self.source, Source::Numeric { .. })
    }

    pub(super) fn new(doc: &'a Document, id: usize) -> Option<Self> {
        let doc = doc.frozen_evaluation_document(id).unwrap_or(doc);
        let object = doc.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let (source, intervals) = match &object.kind {
            Kind::SampledMapped(definition) => (
                Source::Mapped(doc.sampled_mapped_evaluator(id)?),
                definition.samples().saturating_sub(1) as usize,
            ),
            Kind::FunctionPlot(definition, frame, min, max, samples, mode) => {
                if !min.is_finite() || !max.is_finite() || min >= max || *samples < 2 {
                    return None;
                }
                (
                    Source::Function {
                        definition: *definition,
                        frame: doc.graph_frame(*frame)?,
                        min: *min,
                        max: *max,
                        mode: *mode,
                    },
                    samples.saturating_sub(1) as usize,
                )
            }
            Kind::ParametricPlot(x, y, frame, min, max, samples, mode) => {
                if !min.is_finite() || !max.is_finite() || min >= max || *samples < 2 {
                    return None;
                }
                doc.function_expression(*x)?;
                doc.function_expression(*y)?;
                (
                    Source::Parametric {
                        x_definition: *x,
                        y_definition: *y,
                        frame: doc.graph_frame(*frame)?,
                        min: *min,
                        max: *max,
                        mode: *mode,
                    },
                    samples.saturating_sub(1) as usize,
                )
            }
            Kind::ParameterLocus(refs, min, max, samples) => {
                let (&target, &driver) = (refs.first()?, refs.get(1)?);
                let profile = super::kind::parameter_locus_driver_profile(&doc.objects, driver)?;
                if !min.is_finite()
                    || !max.is_finite()
                    || min >= max
                    || *samples < 2
                    || !is_point_kind(&doc.objects.get(target.checked_sub(1)?)?.kind)
                {
                    return None;
                }
                let plan = SamplingPlan::new(doc, driver, refs, None);
                let mut sample = Box::new(doc.clone_for_evaluation());
                sample.ensure_evaluation_context(doc)?;
                (
                    Source::Numeric {
                        sample,
                        plan,
                        driver,
                        target,
                        min: *min,
                        max: *max,
                        scale: profile.scale(),
                    },
                    samples.saturating_sub(1) as usize,
                )
            }
            Kind::Locus(refs, samples) => {
                let (&target, &path, &driver) = (refs.first()?, refs.get(1)?, refs.get(2)?);
                let kind = doc.objects.get(driver.checked_sub(1)?)?.kind.clone();
                if *samples < 2
                    || !is_point_kind(&doc.objects.get(target.checked_sub(1)?)?.kind)
                    || !matches!(
                        kind,
                        Kind::PointOnSegment(..)
                            | Kind::PointOnCircle(..)
                            | Kind::PointOnArc(..)
                            | Kind::PointOnRay(..)
                            | Kind::PointOnLine(..)
                            | Kind::PointOnCurve(..)
                            | Kind::PointOnPolygon(..)
                            | Kind::PointOnSector(..)
                            | Kind::PointOnCircularSegment(..)
                    )
                {
                    return None;
                }
                let plan = SamplingPlan::new(doc, driver, refs, Some(path));
                let mut sample = Box::new(doc.clone_for_evaluation());
                if let Some(plan) = &plan {
                    sample.freeze_scoped_evaluation_inputs(doc, &plan.external)?;
                } else {
                    sample.invalidate_evaluation_dependents(driver);
                }
                (
                    Source::Geometric {
                        sample,
                        plan,
                        driver,
                        path,
                        target,
                        kind,
                        interval: doc.locus_sampling_interval(id)?,
                    },
                    *samples as usize,
                )
            }
            _ => return None,
        };
        Some(Self {
            original: doc,
            owner: id,
            source,
            intervals: intervals.clamp(64, 4096),
        })
    }
    pub(super) fn literal_fraction(&self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        match &self.source {
            Source::Geometric {
                kind: Kind::PointOnCircle(..),
                ..
            } => Some(value.rem_euclid(1.)),
            Source::Geometric {
                kind: Kind::PointOnSegment(..) | Kind::PointOnRay(..) | Kind::PointOnLine(..),
                ..
            } => Some(value.clamp(0., 1.)),
            Source::Geometric {
                kind: Kind::PointOnArc(..),
                ..
            } => (0.0..=1.0).contains(&value).then_some(value),
            Source::Geometric {
                kind:
                    Kind::PointOnPolygon(..)
                    | Kind::PointOnSector(..)
                    | Kind::PointOnCircularSegment(..),
                ..
            } => (0.0..=1.0).contains(&value).then_some(value),
            Source::Geometric {
                kind: Kind::PointOnCurve(..),
                path,
                ..
            } => Self::new(self.original, *path)?.literal_fraction(value),
            Source::Numeric { .. } | Source::Function { .. } | Source::Parametric { .. } => {
                Some(value)
            }
            Source::Mapped(evaluator) => evaluator.literal_fraction(value),
            _ => None,
        }
    }
    pub(super) fn parameter_fraction(&self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        let fraction = match &self.source {
            Source::Numeric { min, max, .. }
            | Source::Function { min, max, .. }
            | Source::Parametric { min, max, .. } => (value - min) / (max - min),
            Source::Geometric { .. } => return self.normalized_numeric_fraction(value),
            Source::Mapped(_) => return self.normalized_numeric_fraction(value),
        };
        (0.0..=1.0).contains(&fraction).then_some(fraction)
    }
    /// Numeric points on an outer geometric locus consume a normalized driver fraction,
    /// even when its driver itself belongs to a raw-parameter locus.
    pub(super) fn normalized_numeric_fraction(&self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        match &self.source {
            Source::Geometric {
                kind:
                    Kind::PointOnCircle(..)
                    | Kind::PointOnPolygon(..)
                    | Kind::PointOnSector(..)
                    | Kind::PointOnCircularSegment(..),
                ..
            } => Some(value.rem_euclid(1.)),
            Source::Geometric {
                kind: Kind::PointOnCurve(..),
                path,
                ..
            } => {
                // A geometric locus driven by a function point has its own
                // normalized numeric domain. Direct numeric function points
                // retain their separate admission and parameter policy.
                match &self.original.objects.get(path.checked_sub(1)?)?.kind {
                    Kind::FunctionPlot(..) => (0.0..=1.0).contains(&value).then_some(value),
                    Kind::SampledMapped(definition)
                        if self
                            .original
                            .objects
                            .get(definition.path().checked_sub(1)?)?
                            .kind
                            .is_circle() =>
                    {
                        Some(value.rem_euclid(1.))
                    }
                    _ => Self::new(self.original, *path)?.normalized_numeric_fraction(value),
                }
            }
            Source::Geometric { .. } | Source::Numeric { .. } => {
                (0.0..=1.0).contains(&value).then_some(value)
            }
            Source::Function { .. } | Source::Parametric { .. } => None,
            Source::Mapped(_) => (0.0..=1.0).contains(&value).then_some(value),
        }
    }
    pub(super) fn at_literal(&mut self, value: f64) -> Option<Point> {
        let fraction = self.literal_fraction(value)?;
        self.at(fraction)
    }
    pub(super) fn at(&mut self, fraction: f64) -> Option<Point> {
        let point = match &mut self.source {
            Source::Mapped(evaluator) => evaluator.at(fraction)?,
            Source::Function {
                definition,
                frame,
                min,
                max,
                mode,
            } => {
                let parameter = *min + (*max - *min) * fraction;
                let result = self.original.function_value(*definition, parameter)?;
                let (x, y) = super::functions::function_plot_coordinates(*mode, parameter, result);
                let value = frame.world(x, y)?;
                Point {
                    x: value.x,
                    y: value.y,
                }
            }
            Source::Parametric {
                x_definition,
                y_definition,
                frame,
                min,
                max,
                mode,
            } => {
                let parameter = *min + (*max - *min) * fraction;
                let x = self.original.function_value(*x_definition, parameter)?;
                let y = self.original.function_value(*y_definition, parameter)?;
                let (x, y) = match mode {
                    PlotPointsMode::Cartesian => (x, y),
                    PlotPointsMode::Polar => (x * y.cos(), x * y.sin()),
                };
                let value = frame.world(x, y)?;
                Point {
                    x: value.x,
                    y: value.y,
                }
            }
            Source::Numeric {
                sample,
                plan,
                driver,
                target,
                min,
                max,
                scale,
            } => {
                let value = (*min + (*max - *min) * fraction) / *scale;
                self.original.with_sac_scope(
                    sample,
                    super::sac_evaluation::Scope::MappedSample {
                        owner: self.owner,
                        fraction: fraction.to_bits(),
                    },
                    |sample| {
                        match plan {
                            Some(plan) => plan.number(sample, value)?,
                            None => sample.set_evaluation_number(*driver, value)?,
                        }
                        sample.position(*target)
                    },
                )?
            }
            Source::Geometric {
                sample,
                plan,
                driver,
                path,
                target,
                kind: _,
                interval: [minimum, maximum],
            } => {
                // Use the original host geometry, retaining unrelated scoped inputs.
                let parameter = *minimum + (*maximum - *minimum) * fraction;
                let kind = self
                    .original
                    .locus_driver_kind_at(*driver, *path, parameter)?;
                self.original.with_sac_scope(
                    sample,
                    super::sac_evaluation::Scope::MappedSample {
                        owner: self.owner,
                        fraction: fraction.to_bits(),
                    },
                    |sample| {
                        match plan {
                            Some(plan) => plan.kind(sample, kind)?,
                            None => sample.objects.substitute_kind(*driver - 1, kind)?,
                        }
                        sample.position(*target)
                    },
                )?
            }
        };
        (point.x.is_finite() && point.y.is_finite()).then_some(point)
    }
}
