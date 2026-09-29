use super::records::{put_record, records, u32_at};
use super::{Document, ImagePlacement, Kind, Object, Point};
use crate::graph::GraphPoint;

const HEADER: [u8; 28] = [
    0x7a, 0, 0, 0, 0, 0, 0x0b, 0, 0x61, 0, 0x21, 1, 0, 0, 0, 0, 0xff, 0xff, 0x51, 0x5c, 0xc1, 0xbd,
    0xf0, 0xff, 0, 0, 0, 0,
];
const EXPRESSION: [u8; 92] = [
    0xfc, 0x08, 0x00, 0x00, 0x16, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x05, 0x00,
    0x03, 0x00, 0x3c, 0x30, 0x3e, 0x00, 0x53, 0x00, 0x06, 0x00, 0x00, 0x00, 0x02, 0x00, 0x65, 0x00,
    0x07, 0x09, 0x00, 0x00, 0x3c, 0x00, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe8, 0x00, 0x7b, 0x01,
    0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const LABEL: [u8; 26] = [
    6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, b'f', 0,
];

pub(super) fn decode_data_function(
    objects: &[Object],
    page: usize,
    header: &[u8],
    refs: &[usize],
    group: &[u8],
) -> Option<Kind> {
    let [source, frame] = refs else { return None };
    let framed = [b"GSP4".as_slice(), group].concat();
    let rs = records(&framed).ok()?;
    if header != HEADER
        || rs.iter().map(|record| record.kind).collect::<Vec<_>>()
            != [0x07d0, 0x07d2, 0x07d3, 0x0907, 0x07d5, 0x07d8, 0x07d7]
    {
        return None;
    }
    let attributes = &framed[rs[2].data.clone()];
    let expression = &framed[rs[3].data.clone()];
    if attributes.len() != 16
        || u32_at(attributes, 0) != 122
        || u32_at(attributes, 8) != 0
        || u32_at(attributes, 12) != 0
        || expression.len() != 92
        || expression[..22] != EXPRESSION[..22]
        || expression[26..30] != EXPRESSION[26..30]
        || expression[32..56] != EXPRESSION[32..56]
        || expression[60..] != EXPRESSION[60..]
    {
        return None;
    }
    let smoothing = u32_at(attributes, 4);
    if smoothing > 20
        || objects
            .get(source.checked_sub(1)?)
            .is_none_or(|object| object.deleted || object.page != page)
        || !objects.get(frame.checked_sub(1)?).is_some_and(|object| {
            !object.deleted
                && object.page == page
                && matches!(object.kind, Kind::CoordinateSystem(..))
        })
    {
        return None;
    }
    Some(Kind::DataFunction {
        source: *source,
        frame: *frame,
        smoothing,
    })
}

pub(super) fn data_function_group(
    source: usize,
    frame: usize,
    smoothing: u32,
) -> Result<Vec<u8>, String> {
    if smoothing > 20 {
        return Err("Data-function smoothing must be between 0 and 20".into());
    }
    let source = u32::try_from(source).map_err(|_| "Picture reference is too large")?;
    let frame = u32::try_from(frame).map_err(|_| "Coordinate-system reference is too large")?;
    let mut group = Vec::new();
    put_record(&mut group, 0x07d0, &HEADER);
    let mut references = Vec::with_capacity(12);
    references.extend_from_slice(&2_u32.to_le_bytes());
    references.extend_from_slice(&source.to_le_bytes());
    references.extend_from_slice(&frame.to_le_bytes());
    put_record(&mut group, 0x07d2, &references);
    let mut attributes = [0_u8; 16];
    attributes[..4].copy_from_slice(&122_u32.to_le_bytes());
    attributes[4..8].copy_from_slice(&smoothing.to_le_bytes());
    put_record(&mut group, 0x07d3, &attributes);
    put_record(&mut group, 0x0907, &EXPRESSION);
    put_record(&mut group, 0x07d5, &LABEL);
    put_record(&mut group, 0x07d8, &[0, 0]);
    put_record(&mut group, 0x07d7, &[]);
    Ok(group)
}

impl Document {
    fn picture_function_samples(
        &self,
        source: usize,
        frame: usize,
        smoothing: u32,
    ) -> Option<Vec<GraphPoint>> {
        let Kind::Image {
            png,
            placement: ImagePlacement::Independent(_),
        } = &self.objects.get(source.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let transform = self.image_transform(source)?;
        // The current bounded writer admits axis-aligned pictures. Arbitrary
        // affine images can cross a vertical function line more than once.
        if transform[1] != 0.0 || transform[3] != 0.0 || transform[0] == 0.0 {
            return None;
        }
        let rgba = image::load_from_memory_with_format(png, image::ImageFormat::Png)
            .ok()?
            .to_rgba8();
        let graph = self.graph_frame(frame)?;
        let width = rgba.width();
        let height = rgba.height();
        if width < 2 || height == 0 {
            return None;
        }
        let mut tops = Vec::with_capacity(width as usize);
        for x in 0..width {
            let top = (0..height).find(|&y| rgba.get_pixel(x, y).0[3] != 0)?;
            tops.push(f64::from(top));
        }
        if smoothing > 0 {
            let radius = smoothing as usize;
            let original = tops.clone();
            for (index, value) in tops.iter_mut().enumerate() {
                let start = index.saturating_sub(radius);
                let end = (index + radius + 1).min(original.len());
                *value = original[start..end].iter().sum::<f64>() / (end - start) as f64;
            }
        }
        let mut samples = tops
            .into_iter()
            .enumerate()
            .map(|(x, y)| {
                graph.coordinates(GraphPoint {
                    x: transform[0] * x as f64 + transform[1] * y + transform[2],
                    y: transform[3] * x as f64 + transform[4] * y + transform[5],
                })
            })
            .collect::<Option<Vec<_>>>()?;
        if samples.windows(2).all(|pair| pair[0].x > pair[1].x) {
            samples.reverse();
        }
        samples
            .windows(2)
            .all(|pair| pair[0].x < pair[1].x)
            .then_some(samples)
    }

    pub(super) fn data_function_value(&self, id: usize, argument: f64) -> Option<f64> {
        let Kind::DataFunction {
            source,
            frame,
            smoothing,
        } = self.objects.get(id.checked_sub(1)?)?.kind
        else {
            return None;
        };
        let samples = self.picture_function_samples(source, frame, smoothing)?;
        if argument < samples.first()?.x || argument > samples.last()?.x {
            return None;
        }
        match samples.binary_search_by(|sample| sample.x.total_cmp(&argument)) {
            Ok(index) => Some(samples[index].y),
            Err(0) => None,
            Err(index) if index == samples.len() => None,
            Err(index) => {
                let a = samples[index - 1];
                let b = samples[index];
                let t = (argument - a.x) / (b.x - a.x);
                let value = a.y + t * (b.y - a.y);
                value.is_finite().then_some(value)
            }
        }
    }

    pub(super) fn data_function_supported(&self, id: usize) -> bool {
        let Some(Kind::DataFunction {
            source,
            frame,
            smoothing,
        }) = self
            .objects
            .get(id.saturating_sub(1))
            .map(|object| &object.kind)
        else {
            return false;
        };
        self.picture_function_samples(*source, *frame, *smoothing)
            .is_some()
    }

    pub fn add_picture_function(&mut self, source: usize, origin: Point) -> Result<usize, String> {
        if self.partial_view || !self.safe_structure || self.page_count != 1 {
            return Err("Cannot define a picture function in this sketch".into());
        }
        if !self
            .objects
            .get(source.saturating_sub(1))
            .is_some_and(|object| {
                !object.deleted
                    && object.page == self.active_page
                    && matches!(
                        object.kind,
                        Kind::Image {
                            placement: ImagePlacement::Independent(_),
                            ..
                        }
                    )
            })
        {
            return Err("Select one independent picture".into());
        }
        let frame = match self.active_coordinate_system() {
            Some(frame) => frame,
            None => self.add_default_coordinate_system(origin)?,
        };
        if self.picture_function_samples(source, frame, 3).is_none() {
            return Err("The picture's top edge is not a single-valued function".into());
        }
        self.checkpoint();
        Ok(self.append_object(Kind::DataFunction {
            source,
            frame,
            smoothing: 3,
        }))
    }
}
