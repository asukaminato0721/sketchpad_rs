//! Native sampled custom transformations of pictures (construction code 96).

use super::iteration::supported_mapped_point_iteration;
use super::kind::is_point_kind;
use super::{Document, Kind, Object, Point};
use crate::image_records::ImageFile;
use std::collections::BTreeSet;

/// Native's four choices for the sampled-picture "Allowed distortion" field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampledImageDistortion {
    None,
    Some,
    Most,
    All,
}

impl SampledImageDistortion {
    pub const fn native_resolution(self) -> f64 {
        match self {
            Self::None => 0.001,
            Self::Some => 0.25,
            Self::Most => 0.6,
            Self::All => 0.0,
        }
    }

    fn from_native_resolution(value: f64) -> Option<Self> {
        [Self::None, Self::Some, Self::Most, Self::All]
            .into_iter()
            .find(|distortion| distortion.native_resolution().to_bits() == value.to_bits())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SampledImageDefinition {
    output: usize,
    source: usize,
    seed: usize,
    carried: Vec<usize>,
    resolution: f64,
    native_range: [u8; 40],
}

impl SampledImageDefinition {
    pub const MIN_EDITABLE_SAMPLES: u32 = 5;
    pub const MAX_EDITABLE_SAMPLES: u32 = 250_000;

    pub fn output(&self) -> usize {
        self.output
    }

    pub fn source(&self) -> usize {
        self.source
    }

    pub fn seed(&self) -> usize {
        self.seed
    }

    pub fn carried(&self) -> &[usize] {
        &self.carried
    }

    pub fn resolution(&self) -> f64 {
        self.resolution
    }

    pub fn distortion(&self) -> SampledImageDistortion {
        SampledImageDistortion::from_native_resolution(self.resolution)
            .expect("decoded sampled-picture resolution")
    }

    pub fn range_payload(&self) -> &[u8; 40] {
        &self.native_range
    }

    pub fn samples(&self) -> u32 {
        u32::from_le_bytes(self.native_range[16..20].try_into().unwrap())
    }

    fn set_properties(&mut self, samples: u32, distortion: SampledImageDistortion) {
        self.native_range[16..20].copy_from_slice(&samples.to_le_bytes());
        self.resolution = distortion.native_resolution();
    }

    pub(super) fn same_structure(&self, other: &Self) -> bool {
        self.output == other.output
            && self.source == other.source
            && self.seed == other.seed
            && self.carried == other.carried
            && self.native_range[..16] == other.native_range[..16]
            && self.native_range[20..] == other.native_range[20..]
    }

    pub fn native_references(&self) -> Vec<usize> {
        let mut refs = vec![self.output, self.source, self.seed];
        refs.extend(&self.carried);
        refs
    }

    pub(super) fn from_native(refs: &[usize], resolution: f64, range: &[u8]) -> Option<Self> {
        let native_range: [u8; 40] = range.try_into().ok()?;
        let definition = Self {
            output: *refs.first()?,
            source: *refs.get(1)?,
            seed: *refs.get(2)?,
            carried: refs.get(3..)?.to_vec(),
            resolution,
            native_range,
        };
        (definition.native_references().iter().all(|id| *id > 0)
            && definition.carried.last() == Some(&definition.output)
            && definition
                .carried
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
                == definition.carried.len()
            && SampledImageDistortion::from_native_resolution(resolution).is_some()
            && f64::from_le_bytes(native_range[..8].try_into().ok()?) == 0.0
            && f64::from_le_bytes(native_range[8..16].try_into().ok()?) == 1.0
            && definition.samples() >= 2
            && u32::from_le_bytes(native_range[20..24].try_into().ok()?) == 0x0001_0004
            && native_range[32..36] == 1u32.to_le_bytes()
            && native_range[36..40] == [0; 4])
            .then_some(definition)
    }

    pub(super) fn supported(&self, objects: &[Object], page: usize, before: usize) -> bool {
        if self.native_references().iter().any(|id| {
            objects
                .get(id.saturating_sub(1))
                .is_none_or(|object| *id >= before || object.deleted || object.page != page)
        }) {
            return false;
        }
        let point = |id: usize| {
            is_point_kind(&objects[id - 1].kind)
                && !matches!(
                    objects[id - 1].kind,
                    Kind::ToolInput(_) | Kind::ToolPointOnPath(_)
                )
        };
        point(self.output)
            && point(self.seed)
            && matches!(objects[self.source - 1].kind, Kind::Image { .. })
            && supported_mapped_point_iteration(
                objects,
                &[self.seed],
                &[vec![self.output]],
                &self.carried,
            )
    }

    pub(super) fn remap(&self, ids: &[usize]) -> Self {
        Self {
            output: ids[self.output],
            source: ids[self.source],
            seed: ids[self.seed],
            carried: self.carried.iter().map(|id| ids[*id]).collect(),
            resolution: self.resolution,
            native_range: self.native_range,
        }
    }
}

pub(super) fn decode_sampled_image(
    objects: &[Object],
    page: usize,
    before: usize,
    header: &[u8],
    refs: &[usize],
    constraint: &[u8],
    range: &[u8],
) -> Option<SampledImageDefinition> {
    if header.len() != 28
        || header[..2] != [96, 0]
        || header[6..10] != [6, 0, 91, 0]
        || constraint.len() != 12
        || constraint[..4] != 96u32.to_le_bytes()
    {
        return None;
    }
    let definition = SampledImageDefinition::from_native(
        refs,
        f64::from_le_bytes(constraint[4..12].try_into().ok()?),
        range,
    )?;
    definition
        .supported(objects, page, before)
        .then_some(definition)
}

struct SampledImageEvaluator {
    sample: Box<Document>,
    seed: usize,
    output: usize,
}

impl SampledImageEvaluator {
    fn new(document: &Document, definition: &SampledImageDefinition) -> Option<Self> {
        let external = document.mapped_external_inputs(
            &[definition.seed],
            &[vec![definition.output]],
            &definition.carried,
        )?;
        let mut sample = Box::new(document.clone());
        sample.freeze_scoped_evaluation_inputs(document, &external)?;
        sample.inherit_evaluation_seeds(document)?;
        Some(Self {
            sample,
            seed: definition.seed,
            output: definition.output,
        })
    }

    fn map(&mut self, point: Point) -> Option<Point> {
        self.sample.set_evaluation_point(self.seed, point)?;
        self.sample
            .position(self.output)
            .filter(|point| point.x.is_finite() && point.y.is_finite())
    }
}

pub struct SampledImageMesh<'a> {
    pub source: usize,
    pub png: &'a [u8],
    pub vertices: Vec<(Point, [f32; 2])>,
    pub triangles: Vec<[u32; 3]>,
}

impl Document {
    /// Sample count and Native allowed-distortion choice for a sampled picture.
    pub fn sampled_image_properties(&self, id: usize) -> Option<(u32, SampledImageDistortion)> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::SampledImage(definition) = &object.kind else {
            return None;
        };
        (!object.deleted && object.page == self.active_page)
            .then(|| (definition.samples(), definition.distortion()))
    }

    /// Apply Native's sampled-picture Drawing panel as one verified transaction.
    pub fn set_sampled_image_properties(
        &mut self,
        id: usize,
        samples: u32,
        distortion: SampledImageDistortion,
    ) -> Result<bool, String> {
        if !(SampledImageDefinition::MIN_EDITABLE_SAMPLES
            ..=SampledImageDefinition::MAX_EDITABLE_SAMPLES)
            .contains(&samples)
        {
            return Err("Sample count must be 5–250000".into());
        }
        let index = id.checked_sub(1).ok_or("Invalid sampled picture")?;
        let object = self.objects.get(index).ok_or("Invalid sampled picture")?;
        let Kind::SampledImage(definition) = &object.kind else {
            return Err("Select a sampled transformed picture".into());
        };
        if object.deleted || object.page != self.active_page {
            return Err("Sampled picture is not on the active page".into());
        }
        if definition.samples() == samples && definition.distortion() == distortion {
            return Ok(false);
        }
        let mut candidate = self.clone();
        candidate.checkpoint();
        let mut object = candidate.objects.edit(index);
        let Kind::SampledImage(definition) = &mut object.kind else {
            unreachable!();
        };
        definition.set_properties(samples, distortion);
        drop(object);
        candidate.verify_roundtrip()?;
        *self = candidate;
        Ok(true)
    }

    /// Tessellate a sampled picture using a bounded live evaluation budget.
    pub fn sampled_image_mesh(
        &self,
        id: usize,
        maximum_cells: usize,
    ) -> Option<SampledImageMesh<'_>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let Kind::SampledImage(definition) = &object.kind else {
            return None;
        };
        if object.deleted || !definition.supported(&self.objects, object.page, id) {
            return None;
        }
        let source_object = self.objects.get(definition.source.checked_sub(1)?)?;
        let Kind::Image { png, .. } = &source_object.kind else {
            return None;
        };
        let (width, height) = ImageFile::png_size(png).ok()?;
        if width == 0 || height == 0 || maximum_cells == 0 {
            return None;
        }
        let target = usize::try_from(definition.samples())
            .ok()?
            .min(maximum_cells)
            .max(1);
        let aspect = f64::from(width) / f64::from(height);
        let columns = ((target as f64 * aspect).sqrt().round() as usize).clamp(1, width as usize);
        let rows = target.div_ceil(columns).clamp(1, height as usize);
        let transform = self.image_transform(definition.source)?;
        let mut evaluator = SampledImageEvaluator::new(self, definition)?;
        let mut mapped = Vec::with_capacity((columns + 1) * (rows + 1));
        let mut vertices = Vec::with_capacity((columns + 1) * (rows + 1));
        for row in 0..=rows {
            let v = row as f64 / rows as f64;
            for column in 0..=columns {
                let u = column as f64 / columns as f64;
                let x = u * f64::from(width);
                let y = v * f64::from(height);
                let source = Point {
                    x: transform[0] * x + transform[1] * y + transform[2],
                    y: transform[3] * x + transform[4] * y + transform[5],
                };
                let point = evaluator.map(source);
                mapped.push(point);
                vertices.push((point.unwrap_or(source), [u as f32, v as f32]));
            }
        }
        let mut triangles = Vec::with_capacity(columns * rows * 2);
        let stride = columns + 1;
        for row in 0..rows {
            for column in 0..columns {
                let top_left = row * stride + column;
                let top_right = top_left + 1;
                let bottom_left = top_left + stride;
                let bottom_right = bottom_left + 1;
                if [top_left, top_right, bottom_right]
                    .iter()
                    .all(|index| mapped[*index].is_some())
                {
                    triangles.push([top_left as u32, top_right as u32, bottom_right as u32]);
                }
                if [top_left, bottom_right, bottom_left]
                    .iter()
                    .all(|index| mapped[*index].is_some())
                {
                    triangles.push([top_left as u32, bottom_right as u32, bottom_left as u32]);
                }
            }
        }
        Some(SampledImageMesh {
            source: definition.source,
            png,
            vertices,
            triangles,
        })
    }
}
