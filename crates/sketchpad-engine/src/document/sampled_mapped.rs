//! Intrinsic sampled application of an ordinary stored dependency graph.
//! Ordinary stored sampled substitutions; no tool authoring or invocation.
use super::iteration::{supported_mapped_point_iteration, supported_tool_mapped_point_iteration};
use super::kind::{is_arc_kind, is_point_kind};
use super::{Document, Kind, LineMode, LocusStroke, Object, Point};
use crate::style_records::VisualStyle;
use std::collections::BTreeSet;
#[derive(Clone, Debug, PartialEq)]
pub struct SampledMappedDefinition {
    output: usize,
    path: usize,
    seed: usize,
    carried: Vec<usize>,
    native_range: [u8; 40],
}
impl SampledMappedDefinition {
    /// Preserve an explicit native range profile, including its opaque cache.
    /// Document authoring additionally validates all source roles and pages.
    pub fn new(
        output: usize,
        path: usize,
        seed: usize,
        carried: Vec<usize>,
        native_range: [u8; 40],
    ) -> Result<Self, String> {
        let mut refs = vec![output, path, seed];
        refs.extend(carried);
        Self::from_native(&refs, &native_range)
            .ok_or_else(|| "Invalid sampled construction references or range".into())
    }
    pub fn output(&self) -> usize {
        self.output
    }
    pub fn path(&self) -> usize {
        self.path
    }
    pub fn seed(&self) -> usize {
        self.seed
    }
    pub fn carried(&self) -> &[usize] {
        &self.carried
    }
    pub fn range_payload(&self) -> &[u8; 40] {
        &self.native_range
    }
    pub fn minimum(&self) -> f64 {
        f64::from_le_bytes(self.native_range[..8].try_into().unwrap())
    }
    pub fn maximum(&self) -> f64 {
        f64::from_le_bytes(self.native_range[8..16].try_into().unwrap())
    }
    pub fn samples(&self) -> u32 {
        u32::from_le_bytes(self.native_range[16..20].try_into().unwrap())
    }
    pub fn native_references(&self) -> Vec<usize> {
        let mut refs = vec![self.output, self.path, self.seed];
        refs.extend(&self.carried);
        refs
    }
    pub(super) fn remap(&self, ids: &[usize]) -> Self {
        Self {
            output: ids[self.output],
            path: ids[self.path],
            seed: ids[self.seed],
            carried: self.carried.iter().map(|&i| ids[i]).collect(),
            native_range: self.native_range,
        }
    }
    pub(super) fn from_native(refs: &[usize], range: &[u8]) -> Option<Self> {
        let native_range: [u8; 40] = range.try_into().ok()?;
        let d = Self {
            output: *refs.first()?,
            path: *refs.get(1)?,
            seed: *refs.get(2)?,
            carried: refs.get(3..)?.to_vec(),
            native_range,
        };
        (d.native_references().iter().all(|&i| i > 0)
            && d.carried.last() == Some(&d.output)
            && d.carried.iter().copied().collect::<BTreeSet<_>>().len() == d.carried.len()
            && d.minimum().is_finite()
            && d.maximum().is_finite()
            && d.minimum() < d.maximum()
            && d.samples() >= 2
            && d.native_range[36..40] == [0; 4])
            .then_some(d)
    }
    pub(super) fn supported(
        &self,
        objects: &(impl super::object_store::ObjectRead + ?Sized),
        page: usize,
        before: usize,
    ) -> bool {
        self.supported_impl(objects, page, before, false)
    }
    fn supported_impl(
        &self,
        objects: &(impl super::object_store::ObjectRead + ?Sized),
        page: usize,
        before: usize,
        allow_tool_points: bool,
    ) -> bool {
        if self.native_references().iter().any(|&i| {
            objects
                .get(i.saturating_sub(1))
                .is_none_or(|o| i == 0 || i >= before || o.page != page || o.deleted)
        }) {
            return false;
        }
        if !is_point_kind(&objects[self.seed - 1].kind)
            || !is_point_kind(&objects[self.output - 1].kind)
            || matches!(objects[self.seed - 1].kind, Kind::ToolInput(_))
            || !allow_tool_points && matches!(objects[self.seed - 1].kind, Kind::ToolPointOnPath(_))
            || matches!(objects[self.output - 1].kind, Kind::ToolInput(_))
            || !allow_tool_points
                && matches!(objects[self.output - 1].kind, Kind::ToolPointOnPath(_))
        {
            return false;
        }
        let path = &objects[self.path - 1].kind;
        let flag = u32::from_le_bytes(self.native_range[20..24].try_into().unwrap());
        let closure = if path.is_circle() || path.is_polygon() {
            Some(0)
        } else if is_arc_kind(path) {
            Some(1)
        } else {
            super::kind::locus_straight_source_mode(objects, self.path)
                .map(|mode| match mode {
                    LineMode::Segment => 1,
                    LineMode::Ray => 2,
                    LineMode::Line => 3,
                })
                .or_else(|| {
                    if super::curve_points::free_supported(objects, self.path, page, before) {
                        super::locus::locus_curve_flags(objects, self.path)
                    } else {
                        None
                    }
                })
        };
        let path_ok = closure.is_some_and(|closure| flag == (4 | (closure << 16)))
            && u32::from_le_bytes(self.native_range[32..36].try_into().unwrap()) <= 1;
        path_ok
            && if allow_tool_points {
                supported_tool_mapped_point_iteration(
                    objects,
                    &[self.seed],
                    &[vec![self.output]],
                    &self.carried,
                )
            } else {
                supported_mapped_point_iteration(
                    objects,
                    &[self.seed],
                    &[vec![self.output]],
                    &self.carried,
                )
            }
    }
}
pub(super) fn decode_sampled_mapped(
    objects: &[Object],
    page: usize,
    before: usize,
    header: &[u8],
    refs: &[usize],
    range: &[u8],
    constraint: Option<&[u8]>,
) -> Option<SampledMappedDefinition> {
    if header.len() != 28
        || header[..2] != [102, 0]
        || header[6..10] != [6, 0, 90, 0]
        || constraint.is_some()
    {
        return None;
    }
    let definition = SampledMappedDefinition::from_native(refs, range)?;
    let allow_tool_points = objects
        .iter()
        .any(|object| matches!(object.kind, Kind::ToolInput(_) | Kind::ToolPointOnPath(_)));
    definition
        .supported_impl(objects, page, before, allow_tool_points)
        .then_some(definition)
}
enum PathGeometry {
    Straight(Point, Point),
    Polygon(Vec<Point>),
    Circle(Point, f64),
    LinearArc(Point, Point),
    Arc(Point, f64, f64, f64, bool),
    Mapped(Box<SampledMappedEvaluator>),
    Curve(Box<Document>, usize),
    Undefined,
}
impl PathGeometry {
    fn at(&mut self, value: f64) -> Option<Point> {
        if !value.is_finite() {
            return None;
        }
        if let Self::Polygon(vertices) = self {
            return super::geometry::polygon_path_point(vertices, value);
        }
        let p = match self {
            Self::Mapped(evaluator) => return evaluator.at(value),
            Self::Curve(document, path) => return document.point_on_curve(*path, value),
            Self::Straight(a, b) => Point {
                x: a.x + (b.x - a.x) * value,
                y: a.y + (b.y - a.y) * value,
            },
            Self::LinearArc(a, b) => {
                if !(0.0..=1.0).contains(&value) {
                    return None;
                }
                Point {
                    x: a.x + (b.x - a.x) * value,
                    y: a.y + (b.y - a.y) * value,
                }
            }
            Self::Circle(c, r) => Point {
                x: c.x + *r * value.cos(),
                y: c.y - *r * value.sin(),
            },
            Self::Arc(c, r, start, sweep, clockwise_y) => {
                if !(0.0..=1.0).contains(&value) {
                    return None;
                }
                let a = *start + *sweep * value;
                Point {
                    x: c.x + *r * a.cos(),
                    y: c.y
                        + if *clockwise_y {
                            -*r * a.sin()
                        } else {
                            *r * a.sin()
                        },
                }
            }
            Self::Polygon(_) => unreachable!(),
            Self::Undefined => return None,
        };
        (p.x.is_finite() && p.y.is_finite()).then_some(p)
    }
}
/// Prepare the scoped graph once. The evaluation caches hold only frozen
/// external boundaries, not lazy carried results. Replacing the seed leaves
/// those boundaries intact while immutable geometry queries reevaluate the DAG.
pub(super) struct SampledMappedEvaluator {
    parent: Box<Document>,
    owner: usize,
    sample: Box<Document>,
    path: PathGeometry,
    seed: usize,
    output: usize,
    min: f64,
    max: f64,
}
impl SampledMappedEvaluator {
    pub(super) fn new(doc: &Document, owner: usize, d: &SampledMappedDefinition) -> Option<Self> {
        let kind = &doc.objects.get(d.path.checked_sub(1)?)?.kind;
        let path = if kind.is_circle() {
            doc.circle_points(d.path)
                .map(|(c, r)| PathGeometry::Circle(c, r))
        } else if kind.is_polygon() {
            doc.polygon_points(d.path).map(PathGeometry::Polygon)
        } else if super::kind::locus_straight_source_mode(&doc.objects, d.path).is_some() {
            doc.point_constraint_line_points(d.path)
                .map(|(a, b, _)| PathGeometry::Straight(a, b))
        } else if matches!(kind, Kind::SampledMapped(_)) {
            doc.sampled_mapped_evaluator(d.path)
                .map(|evaluator| PathGeometry::Mapped(Box::new(evaluator)))
        } else if super::curve_points::free_supported(
            &doc.objects,
            d.path,
            doc.objects[d.path - 1].page,
            d.output,
        ) {
            Some(PathGeometry::Curve(Box::new(doc.clone()), d.path))
        } else {
            doc.arc_geometry(d.path)
                .map(|(c, r, a, s, y)| PathGeometry::Arc(c, r, a, s, y))
                .or_else(|| {
                    doc.linear_three_point_arc(d.path)
                        .map(|(a, b)| PathGeometry::LinearArc(a, b))
                })
        }
        .unwrap_or(PathGeometry::Undefined);
        let external = doc.mapped_external_inputs(&[d.seed], &[vec![d.output]], &d.carried)?;
        let mut sample = Box::new(doc.clone());
        sample.freeze_scoped_evaluation_inputs(doc, &external)?;
        // The local path seed is replaced at each sample. Other substitutions
        // established by an enclosing mapping still belong to this caller.
        sample.inherit_evaluation_seeds(doc)?;
        Some(Self {
            parent: Box::new(doc.clone_for_evaluation()),
            owner,
            sample,
            path,
            seed: d.seed,
            output: d.output,
            min: d.minimum(),
            max: d.maximum(),
        })
    }
    /// Native literal child parameters extend straight and polygon paths, wrap circles,
    /// and remain bounded on curved or linear arcs.
    pub(super) fn literal_fraction(&self, fraction: f64) -> Option<f64> {
        if !fraction.is_finite() {
            return None;
        }
        match self.path {
            PathGeometry::Straight(..) | PathGeometry::Polygon(..) => Some(fraction),
            PathGeometry::Circle(..) => Some(fraction.rem_euclid(1.)),
            PathGeometry::Arc(..) | PathGeometry::LinearArc(..) => {
                (0.0..=1.0).contains(&fraction).then_some(fraction)
            }
            PathGeometry::Mapped(ref evaluator) => evaluator.literal_fraction(fraction),
            PathGeometry::Curve(ref document, path) => {
                super::point_curve::PointCurve::new(document, path)?.literal_fraction(fraction)
            }
            PathGeometry::Undefined => None,
        }
    }
    /// PointValue reads the logical parameter rather than extrapolated position.
    pub(super) fn logical_point_value(&self, fraction: f64) -> Option<f64> {
        match self.path {
            PathGeometry::Straight(..) => fraction.is_finite().then(|| fraction.clamp(0., 1.)),
            PathGeometry::Polygon(..) => fraction
                .is_finite()
                .then(|| fraction.clamp(0., 1.).rem_euclid(1.)),
            _ => self.literal_fraction(fraction),
        }
    }
    pub(super) fn batch_points(&mut self, intervals: usize) -> Option<Vec<Option<Point>>> {
        let program =
            super::point_program::PointProgram::get(&self.sample, &[self.seed], &[self.output])?;
        let batch = program.bind(&self.sample)?;
        let inputs: Vec<_> = (0..=intervals)
            .map(|index| {
                let fraction = self.literal_fraction(index as f64 / intervals as f64)?;
                self.path.at(self.min + (self.max - self.min) * fraction)
            })
            .collect();
        let mut outputs = batch.evaluate(std::slice::from_ref(&inputs)).remove(0);
        for (output, input) in outputs.iter_mut().zip(inputs) {
            if input.is_none() {
                *output = None;
            }
            *output = output.filter(|p| p.x.is_finite() && p.y.is_finite());
        }
        Some(outputs)
    }

    pub(super) fn at(&mut self, fraction: f64) -> Option<Point> {
        self.at_with_retention(fraction, false)
    }
    pub(super) fn at_with_retention(&mut self, fraction: f64, retain: bool) -> Option<Point> {
        let fraction = self.literal_fraction(fraction)?;
        let value = self.min + (self.max - self.min) * fraction;
        let replacement = self.path.at(value)?;
        self.parent.with_sac_scope(
            &mut self.sample,
            if retain {
                super::sac_evaluation::Scope::CurveRefinement {
                    owner: self.owner,
                    fraction: fraction.to_bits(),
                }
            } else {
                super::sac_evaluation::Scope::MappedSample {
                    owner: self.owner,
                    fraction: fraction.to_bits(),
                }
            },
            |sample| {
                sample.set_evaluation_point(self.seed, replacement)?;
                sample
                    .position(self.output)
                    .filter(|p| p.x.is_finite() && p.y.is_finite())
            },
        )
    }
}
impl Document {
    pub fn add_sampled_mapped(
        &mut self,
        output: usize,
        path: usize,
        seed: usize,
        carried: &[usize],
        samples: u32,
    ) -> Result<usize, String> {
        if self.partial_view || !self.safe_structure || self.page_count != 1 {
            return Err("Cannot add a sampled construction in this sketch safely".into());
        }
        let kind = &self
            .objects
            .get(path.checked_sub(1).ok_or("Select a source path")?)
            .ok_or("Select a source path")?
            .kind;
        let (max, flags) = if kind.is_circle() {
            (std::f64::consts::TAU, 4u32)
        } else if kind.is_polygon() {
            (1., 4)
        } else {
            match super::kind::locus_straight_source_mode(&self.objects, path) {
                Some(LineMode::Ray) => (1., 0x20004),
                Some(LineMode::Line) => (1., 0x30004),
                _ => (1., 0x10004),
            }
        };
        let mut range = [0u8; 40];
        range[8..16].copy_from_slice(&max.to_le_bytes());
        range[16..20].copy_from_slice(&samples.to_le_bytes());
        range[20..24].copy_from_slice(&flags.to_le_bytes());
        self.add_sampled_mapped_with_range(output, path, seed, carried, range)
    }
    pub fn add_sampled_mapped_with_range(
        &mut self,
        output: usize,
        path: usize,
        seed: usize,
        carried: &[usize],
        range: [u8; 40],
    ) -> Result<usize, String> {
        if self.partial_view || !self.safe_structure || self.page_count != 1 {
            return Err("Cannot add a sampled construction in this sketch safely".into());
        }
        let definition = SampledMappedDefinition::new(output, path, seed, carried.to_vec(), range)?;
        if !definition.supported(&self.objects, self.active_page, self.objects.len() + 1) {
            return Err(
                "Select a supported point substitution and carried dependency graph".into(),
            );
        }
        self.checkpoint();
        Ok(self.append_object(Kind::SampledMapped(definition)))
    }
    pub(super) fn sampled_mapped_evaluator(&self, id: usize) -> Option<SampledMappedEvaluator> {
        if let Some(original) = self.frozen_evaluation_document(id) {
            return original.sampled_mapped_evaluator(id);
        }
        let o = self.objects.get(id.checked_sub(1)?)?;
        let Kind::SampledMapped(d) = &o.kind else {
            return None;
        };
        if o.deleted || !d.supported(&self.objects, o.page, id) {
            return None;
        }
        SampledMappedEvaluator::new(self, id, d)
    }
    pub fn sampled_mapped_point(&self, id: usize, fraction: f64) -> Option<Point> {
        self.sampled_mapped_evaluator(id)?.at(fraction)
    }
    /// Subdivisions are a presentation budget, not a semantic parameter limit.
    pub fn sampled_mapped_strokes(
        &self,
        id: usize,
        subdivisions: usize,
    ) -> Option<Vec<(Point, Point, super::LineMode)>> {
        let Kind::SampledMapped(d) = &self.objects.get(id.checked_sub(1)?)?.kind else {
            return None;
        };
        let n = subdivisions
            .clamp(2, 4096)
            .min(d.samples().saturating_sub(1) as usize);
        let mut curve = self.sampled_mapped_evaluator(id)?;
        let slots = curve
            .batch_points(n)
            .unwrap_or_else(|| (0..=n).map(|i| curve.at(i as f64 / n as f64)).collect());
        Some(
            slots
                .windows(2)
                .filter_map(|p| Some((p[0]?, p[1]?, super::LineMode::Segment)))
                .collect(),
        )
    }

    /// A sampled colorized point keeps its live spectrum color at every step.
    /// The saved sampled object's line style still controls width and pattern.
    pub fn sampled_mapped_styled_strokes(
        &self,
        id: usize,
        subdivisions: usize,
    ) -> Option<Vec<LocusStroke>> {
        self.cached_geometry(
            id,
            super::geometry_cache::GeometryQuery::SampledStrokes,
            subdivisions,
            || self.sampled_mapped_styled_strokes_uncached(id, subdivisions),
        )
    }

    pub(super) fn sampled_mapped_styled_strokes_uncached(
        &self,
        id: usize,
        subdivisions: usize,
    ) -> Option<Vec<LocusStroke>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::SampledMapped(d) = &object.kind else {
            return None;
        };
        if object.deleted
            || !matches!(
                self.objects.get(d.output.checked_sub(1)?)?.kind,
                Kind::Colorized(_)
            )
        {
            return None;
        }
        let VisualStyle::Stroke { line, .. } = self.effective_visual_style(id)? else {
            return None;
        };
        let n = subdivisions
            .clamp(2, 4096)
            .min(d.samples().saturating_sub(1) as usize);
        let mut curve = self.sampled_mapped_evaluator(id)?;
        let slots = (0..=n)
            .map(|index| {
                let point = curve.at(index as f64 / n as f64)?;
                let color = curve.sample.colorized_color(d.output)?;
                Some((point, color))
            })
            .collect::<Vec<_>>();
        Some(
            slots
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
        )
    }
}
