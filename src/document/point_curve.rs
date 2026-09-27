//! Evaluated point traces shared by curve constructions.
use super::kind::is_point_kind;
use super::{Document, Kind, Point};
use crate::table_records::ParameterGroup;

enum Source {
    Function {
        definition: usize,
        frame: crate::graph::CoordinateFrame,
        min: f64,
        max: f64,
    },
    Parameter {
        sample: Box<Document>,
        parameter: ParameterGroup,
        driver: usize,
        target: usize,
        min: f64,
        max: f64,
    },
    Geometric {
        sample: Box<Document>,
        driver: usize,
        path: usize,
        target: usize,
        kind: Kind,
    },
}
pub(super) struct PointCurve<'a> {
    original: &'a Document,
    source: Source,
    pub(super) intervals: usize,
}
impl<'a> PointCurve<'a> {
    pub(super) fn includes_maximum(&self) -> bool {
        matches!(self.source, Source::Parameter { .. })
    }

    pub(super) fn new(doc: &'a Document, id: usize) -> Option<Self> {
        let object = doc.objects.get(id.checked_sub(1)?)?;
        if object.deleted {
            return None;
        }
        let (source, intervals) = match &object.kind {
            Kind::FunctionPlot(definition, frame, min, max, samples) => {
                if !min.is_finite() || !max.is_finite() || min >= max || *samples < 2 {
                    return None;
                }
                (
                    Source::Function {
                        definition: *definition,
                        frame: doc.graph_frame(*frame)?,
                        min: *min,
                        max: *max,
                    },
                    samples.saturating_sub(1) as usize,
                )
            }
            Kind::ParameterLocus(refs, min, max, samples) => {
                let (&target, &driver) = (refs.first()?, refs.get(1)?);
                let Kind::Parameter(parameter) = &doc.objects.get(driver.checked_sub(1)?)?.kind
                else {
                    return None;
                };
                if !min.is_finite()
                    || !max.is_finite()
                    || min >= max
                    || *samples < 2
                    || !is_point_kind(&doc.objects.get(target.checked_sub(1)?)?.kind)
                {
                    return None;
                }
                let mut sample = Box::new(doc.clone());
                sample.invalidate_evaluation_dependents(driver);
                (
                    Source::Parameter {
                        sample,
                        parameter: parameter.clone(),
                        driver,
                        target,
                        min: *min,
                        max: *max,
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
                        Kind::PointOnSegment(..) | Kind::PointOnCircle(..) | Kind::PointOnArc(..)
                    )
                {
                    return None;
                }
                let mut sample = Box::new(doc.clone());
                sample.invalidate_evaluation_dependents(driver);
                (
                    Source::Geometric {
                        sample,
                        driver,
                        path,
                        target,
                        kind,
                    },
                    *samples as usize,
                )
            }
            _ => return None,
        };
        Some(Self {
            original: doc,
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
                kind: Kind::PointOnSegment(..),
                ..
            } => Some(value.clamp(0., 1.)),
            Source::Geometric {
                kind: Kind::PointOnArc(..),
                ..
            } => (0.0..=1.0).contains(&value).then_some(value),
            Source::Parameter { .. } => Some(value),
            _ => None,
        }
    }
    pub(super) fn parameter_fraction(&self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        let fraction = match &self.source {
            Source::Parameter { min, max, .. } => (value - min) / (max - min),
            Source::Geometric {
                kind: Kind::PointOnCircle(..),
                ..
            } => return Some(value.rem_euclid(1.)),
            Source::Geometric { .. } => value,
            Source::Function { .. } => return None,
        };
        (0.0..=1.0).contains(&fraction).then_some(fraction)
    }
    pub(super) fn at_literal(&mut self, value: f64) -> Option<Point> {
        let fraction = self.literal_fraction(value)?;
        self.at(fraction)
    }
    pub(super) fn at(&mut self, fraction: f64) -> Option<Point> {
        let point = match &mut self.source {
            Source::Function {
                definition,
                frame,
                min,
                max,
            } => {
                let x = *min + (*max - *min) * fraction;
                let value = frame.world(x, self.original.function_value(*definition, x)?)?;
                Point {
                    x: value.x,
                    y: value.y,
                }
            }
            Source::Parameter {
                sample,
                parameter,
                driver,
                target,
                min,
                max,
            } => {
                let value = (*min + (*max - *min) * fraction) / parameter.native_unit_scale();
                if !parameter.set_evaluation_value(value) {
                    return None;
                }
                sample.objects[*driver - 1].kind = Kind::Parameter(parameter.clone());
                sample.position(*target)?
            }
            Source::Geometric {
                sample,
                driver,
                path,
                target,
                kind,
            } => {
                sample.objects[*driver - 1].kind = match kind {
                    Kind::PointOnSegment(..) => Kind::PointOnSegment(*path, fraction),
                    Kind::PointOnArc(..) => Kind::PointOnArc(*path, fraction),
                    Kind::PointOnCircle(..) => {
                        let angle = fraction * std::f64::consts::TAU;
                        Kind::PointOnCircle(*path, angle.cos(), angle.sin())
                    }
                    _ => return None,
                };
                sample.position(*target)?
            }
        };
        (point.x.is_finite() && point.y.is_finite()).then_some(point)
    }
}
