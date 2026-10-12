//! Lossless helpers for the observed GSP4 embedded picture layout.
//!
//! Imported pictures can be independent or attached to one, two, or three
//! points. Writing covers a single independent picture and the proved point
//! and three-point forms emitted while materializing embedded tools.

use std::ops::Range;

const RESOURCE: u32 = 0x0514;
const OBJECT_HEADER: u32 = 0x07d0;
const IMAGE_SIZE: u32 = 0x090c;
const IMAGE_TRANSFORM: u32 = 0x08a8;
const IMAGE_RESOURCE: u32 = 0x1f44;
const OBJECT_END: u32 = 0x07d7;
const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
pub(crate) const INTERNAL_IMAGE_LABEL: [u8; 28] = [
    6, 0, 0, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 152, 110, 18, 193, 81, 92, 97, 65, 3, 0, 91, 49, 93,
    0,
];

pub(crate) fn internal_image_label(payload: &[u8]) -> bool {
    payload.len() == INTERNAL_IMAGE_LABEL.len()
        && payload[..25] == INTERNAL_IMAGE_LABEL[..25]
        && matches!(payload[25], b'1'..=b'7')
        && payload[26..] == INTERNAL_IMAGE_LABEL[26..]
}
const PARALLELOGRAM_HEADER: [u8; 28] = [
    100, 0, 0, 0, 0, 0, 16, 0, 78, 0, 33, 1, 255, 0, 0, 0, 255, 255, 81, 92, 63, 0, 0, 0, 0, 0, 0,
    0,
];
const POINT_HEADER: [u8; 28] = [
    99, 0, 0, 0, 0, 0, 16, 0, 78, 0, 33, 1, 255, 0, 0, 0, 255, 255, 81, 92, 0, 0, 0, 0, 0, 0, 0, 0,
];

#[derive(Clone, Debug)]
struct Record {
    start: usize,
    kind: u32,
    payload: Range<usize>,
    end: usize,
}

#[derive(Clone, Debug)]
pub struct ImageFile {
    bytes: Vec<u8>,
    resource: Record,
    header: Record,
    size: Record,
    transform: Record,
    group_end: usize,
    crops: Vec<CroppedPicture>,
    editable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CroppedPicture {
    pub group_offset: usize,
    pub source: u32,
    pub polygon: u32,
    pub dimensions: (u32, u32),
    pub visible: bool,
}

/// Decode crop groups independently of the writable single-picture wrapper.
/// The document decoder validates the recorded source dimensions against the
/// referenced picture after resolving page-local object IDs.
pub fn imported_cropped_image_catalog(bytes: &[u8]) -> Result<Vec<CroppedPicture>, String> {
    let records = records(bytes)?;
    let mut crops = Vec::new();
    for (index, header) in records.iter().enumerate() {
        let h = &bytes[header.payload.clone()];
        if header.kind != OBJECT_HEADER
            || h.len() != 28
            || u16::from_le_bytes([h[0], h[1]]) != 108
            || h[6..10] != [16, 0, 78, 0]
        {
            continue;
        }
        let Some(refs) = records.get(index + 1) else {
            continue;
        };
        let Some(size) = records.get(index + 2) else {
            continue;
        };
        let marker = records
            .get(index + 3)
            .filter(|record| record.kind == 0x232e);
        let Some(end) = records.get(index + if marker.is_some() { 4 } else { 3 }) else {
            continue;
        };
        if refs.kind != 0x07d2
            || refs.payload.len() != 12
            || size.kind != IMAGE_SIZE
            || size.payload.len() != 16
            || marker.is_some_and(|record| !record.payload.is_empty())
            || end.kind != OBJECT_END
            || !end.payload.is_empty()
        {
            continue;
        }
        let refs = &bytes[refs.payload.clone()];
        let dimensions = &bytes[size.payload.clone()];
        let source = u32_at(refs, 4);
        let polygon = u32_at(refs, 8);
        let source_dimensions = (u32_at(dimensions, 0), u32_at(dimensions, 4));
        let crop_dimensions = (u32_at(dimensions, 8), u32_at(dimensions, 12));
        if u32_at(refs, 0) != 2
            || source == 0
            || polygon == 0
            || source == polygon
            || source_dimensions.0 == 0
            || source_dimensions.1 == 0
            || crop_dimensions.0 == 0
            || crop_dimensions.1 == 0
        {
            continue;
        }
        crops.push(CroppedPicture {
            group_offset: header.start,
            source,
            polygon,
            dimensions: source_dimensions,
            visible: u32_at(h, 0) & 0x1_0000 == 0,
        });
    }
    Ok(crops)
}

/// A strictly decoded imported picture. Multi-picture documents retain their
/// original resource table and are not structurally writable yet.
#[derive(Clone, Debug)]
pub struct ReadOnlyImage {
    pub group_offset: usize,
    pub png: Vec<u8>,
    pub transform: [f64; 6],
}

/// A strictly decoded native attachment. Point references are local to the
/// page and are resolved by the document decoder.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReadOnlyImageAttachment {
    Point {
        point_ref: u32,
        scale_x: f64,
        scale_y: f64,
        rotation: f64,
    },
    Rectangle {
        bottom_right: u32,
        top_left: u32,
    },
    Parallelogram {
        top_left: u32,
        bottom_right: u32,
        bottom_left: u32,
    },
}

#[derive(Clone, Debug)]
pub struct ReadOnlyAttachedImage {
    pub group_offset: usize,
    pub png: Vec<u8>,
    pub dimensions: (u32, u32),
    pub attachment: ReadOnlyImageAttachment,
}

type ImageResources<'a> = (Vec<&'a [u8]>, Vec<(u32, u32)>);

fn validated_resources<'a>(
    bytes: &'a [u8],
    records: &[Record],
) -> Result<ImageResources<'a>, String> {
    let resources = records
        .iter()
        .filter(|record| record.kind == RESOURCE)
        .collect::<Vec<_>>();
    if resources.is_empty() || resources.len() > 64 {
        return Err("Unsupported picture resource count".into());
    }
    let mut pngs = Vec::with_capacity(resources.len());
    let mut dimensions = Vec::with_capacity(resources.len());
    for resource in resources {
        let payload = &bytes[resource.payload.clone()];
        if payload.len() < 8 {
            return Err("Picture resource header is truncated".into());
        }
        let png = &payload[8..];
        let size = png_dimensions(png)?;
        if (u32_at(payload, 0), u32_at(payload, 4)) != size {
            return Err("Picture resource dimensions disagree with PNG".into());
        }
        pngs.push(png);
        dimensions.push(size);
    }
    Ok((pngs, dimensions))
}

pub fn imported_attached_image_catalog(bytes: &[u8]) -> Result<Vec<ReadOnlyAttachedImage>, String> {
    let records = records(bytes)?;
    let (pngs, dimensions) = validated_resources(bytes, &records)?;
    let mut images = Vec::new();
    for (index, header) in records.iter().enumerate() {
        let h = &bytes[header.payload.clone()];
        if header.kind != OBJECT_HEADER || h.len() != 28 || h[6..10] != [16, 0, 78, 0] {
            continue;
        }
        let code = u16::from_le_bytes([h[0], h[1]]);
        let Some(refs) = records.get(index + 1) else {
            continue;
        };
        if refs.kind != 0x07d2 {
            continue;
        }
        let refs_bytes = &bytes[refs.payload.clone()];
        let (size_index, attachment) = match code {
            99 if refs.payload.len() == 8 && u32_at(refs_bytes, 0) == 1 => {
                let Some(operation) = records.get(index + 2) else {
                    continue;
                };
                if operation.kind != 0x07d3 || operation.payload.len() != 28 {
                    continue;
                }
                let op = &bytes[operation.payload.clone()];
                if u32_at(refs_bytes, 4) == 0 || u32_at(op, 0) != 99 {
                    continue;
                }
                let values = (0..3)
                    .map(|at| f64::from_le_bytes(op[4 + at * 8..12 + at * 8].try_into().unwrap()))
                    .collect::<Vec<_>>();
                if values.iter().any(|value| !value.is_finite())
                    || values[0] <= 0.
                    || values[1] <= 0.
                {
                    continue;
                }
                (
                    index + 3,
                    ReadOnlyImageAttachment::Point {
                        point_ref: u32_at(refs_bytes, 4),
                        scale_x: values[0],
                        scale_y: values[1],
                        rotation: values[2],
                    },
                )
            }
            85 if refs.payload.len() == 12 && u32_at(refs_bytes, 0) == 2 => (
                index + 2,
                ReadOnlyImageAttachment::Rectangle {
                    bottom_right: u32_at(refs_bytes, 4),
                    top_left: u32_at(refs_bytes, 8),
                },
            ),
            100 if refs.payload.len() == 16 && u32_at(refs_bytes, 0) == 3 => (
                index + 2,
                ReadOnlyImageAttachment::Parallelogram {
                    top_left: u32_at(refs_bytes, 4),
                    bottom_right: u32_at(refs_bytes, 8),
                    bottom_left: u32_at(refs_bytes, 12),
                },
            ),
            _ => continue,
        };
        if attachment.references().contains(&0) {
            continue;
        }
        let Some(size) = records.get(size_index) else {
            continue;
        };
        let auxiliary = records.get(size_index + 1);
        let resource_index = match auxiliary.map(|record| record.kind) {
            Some(0x232e) if auxiliary.is_some_and(|record| record.payload.is_empty()) => {
                size_index + 2
            }
            Some(0x07d5)
                if matches!(code, 99 | 100)
                    && auxiliary.is_some_and(|record| {
                        internal_image_label(&bytes[record.payload.clone()])
                    }) =>
            {
                size_index + 2
            }
            _ => size_index + 1,
        };
        let Some(resource) = records.get(resource_index) else {
            continue;
        };
        let Some(end) = records.get(resource_index + 1) else {
            continue;
        };
        if size.kind != IMAGE_SIZE
            || size.payload.len() != 16
            || resource.kind != IMAGE_RESOURCE
            || resource.payload.len() != 2
            || end.kind != OBJECT_END
            || !end.payload.is_empty()
        {
            continue;
        }
        let resource_id =
            u16::from_le_bytes(bytes[resource.payload.clone()].try_into().unwrap()) as usize;
        let Some(&expected) = dimensions.get(resource_id) else {
            continue;
        };
        let stored_size = &bytes[size.payload.clone()];
        let exact_size = [expected.0, expected.1, expected.0, expected.1];
        let observed_size = (0..4)
            .map(|at| u32_at(stored_size, at * 4))
            .collect::<Vec<_>>();
        if (code == 99 && observed_size != exact_size)
            || (matches!(code, 85 | 100) && observed_size[2..] != exact_size[2..])
        {
            continue;
        }
        images.push(ReadOnlyAttachedImage {
            group_offset: header.start,
            png: pngs[resource_id].to_vec(),
            dimensions: expected,
            attachment,
        });
    }
    Ok(images)
}

impl ReadOnlyImageAttachment {
    fn references(self) -> Vec<u32> {
        match self {
            Self::Point { point_ref, .. } => vec![point_ref],
            Self::Rectangle {
                bottom_right,
                top_left,
            } => vec![bottom_right, top_left],
            Self::Parallelogram {
                top_left,
                bottom_right,
                bottom_left,
            } => vec![top_left, bottom_right, bottom_left],
        }
    }
}

pub fn imported_image_catalog(bytes: &[u8]) -> Result<Vec<ReadOnlyImage>, String> {
    let records = records(bytes)?;
    let (pngs, dimensions) = validated_resources(bytes, &records)?;
    let mut images = Vec::new();
    for (index, header) in records.iter().enumerate() {
        let h = &bytes[header.payload.clone()];
        if header.kind != OBJECT_HEADER
            || h.len() != 28
            || h[..2] != [0, 0]
            || h[6..8] != [16, 0]
            || !matches!(h[8..10], [78, 0] | [98, 0])
        {
            continue;
        }
        let Some(size) = records.get(index + 1) else {
            continue;
        };
        let mut transform_index = index + 2;
        if records
            .get(transform_index)
            .is_some_and(|record| record.kind == 0x07d5)
        {
            transform_index += 1;
        }
        let Some(transform) = records.get(transform_index) else {
            continue;
        };
        let marker = records
            .get(transform_index + 1)
            .filter(|record| record.kind == 0x232e);
        let resource_index = transform_index + if marker.is_some() { 2 } else { 1 };
        let Some(resource_ref) = records.get(resource_index) else {
            continue;
        };
        let Some(terminator) = records.get(resource_index + 1) else {
            continue;
        };
        if size.kind != IMAGE_SIZE
            || size.payload.len() != 16
            || transform.kind != IMAGE_TRANSFORM
            || transform.payload.len() != 48
            || marker.is_some_and(|record| !record.payload.is_empty())
            || resource_ref.kind != IMAGE_RESOURCE
            || resource_ref.payload.len() != 2
            || terminator.kind != OBJECT_END
            || !terminator.payload.is_empty()
        {
            continue;
        }
        let resource_id =
            u16::from_le_bytes(bytes[resource_ref.payload.clone()].try_into().unwrap()) as usize;
        let Some(&expected) = dimensions.get(resource_id) else {
            continue;
        };
        let size_payload = &bytes[size.payload.clone()];
        if (0..4)
            .map(|at| u32_at(size_payload, at * 4))
            .collect::<Vec<_>>()
            != [expected.0, expected.1, expected.0, expected.1]
        {
            continue;
        }
        let mut matrix = [0.; 6];
        for (at, value) in matrix.iter_mut().enumerate() {
            *value = f64::from_le_bytes(
                bytes[transform.payload.start + at * 8..transform.payload.start + (at + 1) * 8]
                    .try_into()
                    .unwrap(),
            );
        }
        if matrix.iter().any(|value| !value.is_finite()) {
            continue;
        }
        images.push(ReadOnlyImage {
            group_offset: header.start,
            png: pngs[resource_id].to_vec(),
            transform: matrix,
        });
    }
    Ok(images)
}

fn u32_at(data: &[u8], start: usize) -> u32 {
    u32::from_le_bytes(data[start..start + 4].try_into().unwrap())
}

fn put_record(out: &mut Vec<u8>, kind: u32, payload: &[u8]) -> Result<(), String> {
    let payload_len = u32::try_from(payload.len()).map_err(|_| "GSP record is too large")?;
    out.extend_from_slice(&payload_len.to_le_bytes());
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(payload);
    if !payload.len().is_multiple_of(2) {
        out.push(0);
    }
    Ok(())
}

fn picture_resource(png: &[u8], dimensions: (u32, u32)) -> Result<Vec<u8>, String> {
    let payload_len = png
        .len()
        .checked_add(8)
        .filter(|&len| u32::try_from(len).is_ok())
        .ok_or("Picture resource is too large")?;
    let mut payload = Vec::with_capacity(payload_len);
    payload.extend_from_slice(&dimensions.0.to_le_bytes());
    payload.extend_from_slice(&dimensions.1.to_le_bytes());
    payload.extend_from_slice(png);
    let mut resource = Vec::new();
    put_record(&mut resource, RESOURCE, &payload)?;
    Ok(resource)
}

fn records(bytes: &[u8]) -> Result<Vec<Record>, String> {
    if bytes.get(..4) != Some(b"GSP4") {
        return Err("Expected a GSP4 document".into());
    }
    sketchpad_format::records_from(bytes, 4).map(|records| {
        records
            .into_iter()
            .map(|record| Record {
                kind: record.kind,
                start: record.start,
                end: record.end,
                payload: record.data,
            })
            .collect()
    })
}

fn png_dimensions(png: &[u8]) -> Result<(u32, u32), String> {
    if png.len() > 64 * 1024 * 1024 || png.get(..8) != Some(PNG_SIGNATURE) {
        return Err("Expected a bounded PNG resource".into());
    }
    let mut at = 8usize;
    let mut dimensions = None;
    let mut saw_pixels = false;
    loop {
        if png.len().saturating_sub(at) < 12 {
            return Err("Truncated PNG chunk".into());
        }
        let len = u32::from_be_bytes(png[at..at + 4].try_into().unwrap()) as usize;
        let end = at
            .checked_add(12)
            .and_then(|base| base.checked_add(len))
            .ok_or("PNG chunk length overflow")?;
        if end > png.len() {
            return Err("Truncated PNG chunk data".into());
        }
        let kind = &png[at + 4..at + 8];
        let expected_crc = u32::from_be_bytes(png[end - 4..end].try_into().unwrap());
        let mut crc = !0u32;
        for byte in &png[at + 4..end - 4] {
            crc ^= *byte as u32;
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xedb8_8320_u32.wrapping_mul(crc & 1));
            }
        }
        if !crc != expected_crc {
            return Err("Invalid PNG chunk CRC".into());
        }
        if at == 8 {
            if kind != b"IHDR" || len != 13 {
                return Err("PNG must begin with a 13-byte IHDR".into());
            }
            let width = u32::from_be_bytes(png[at + 8..at + 12].try_into().unwrap());
            let height = u32::from_be_bytes(png[at + 12..at + 16].try_into().unwrap());
            if width == 0 || height == 0 || width > 32_768 || height > 32_768 {
                return Err("Invalid PNG dimensions".into());
            }
            dimensions = Some((width, height));
        }
        if kind == b"IDAT" {
            saw_pixels = true;
        }
        at = end;
        if kind == b"IEND" {
            if len != 0 || at != png.len() || !saw_pixels {
                return Err("Invalid PNG ending".into());
            }
            return dimensions.ok_or("PNG dimensions are absent".into());
        }
    }
}

impl ImageFile {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let records = records(bytes)?;
        let resource_records: Vec<_> = records.iter().filter(|r| r.kind == RESOURCE).collect();
        let [resource] = resource_records.as_slice() else {
            return Err("Expected exactly one embedded picture resource".into());
        };
        let mut groups = Vec::new();
        for window in records.windows(5) {
            if window.iter().map(|r| r.kind).collect::<Vec<_>>()
                == [
                    OBJECT_HEADER,
                    IMAGE_SIZE,
                    IMAGE_TRANSFORM,
                    IMAGE_RESOURCE,
                    OBJECT_END,
                ]
                && bytes[window[0].payload.clone()].len() == 28
                && u32_at(bytes, window[0].payload.start) as u16 == 0
            {
                groups.push(window);
            }
        }
        let [group] = groups.as_slice() else {
            return Err("Expected one independent affine picture group".into());
        };
        let [header, size, transform, resource_ref, terminator] = *group else {
            unreachable!()
        };
        if size.payload.len() != 16
            || transform.payload.len() != 48
            || resource_ref.payload.len() != 2
            || !terminator.payload.is_empty()
            || bytes[resource_ref.payload.clone()] != [0, 0]
        {
            return Err("Unsupported picture group fields".into());
        }
        let payload = &bytes[resource.payload.clone()];
        if payload.len() < 8 {
            return Err("Picture resource header is truncated".into());
        }
        let dimensions = png_dimensions(&payload[8..])?;
        let image_size = &bytes[size.payload.clone()];
        if u32_at(payload, 0) != dimensions.0
            || u32_at(payload, 4) != dimensions.1
            || u32_at(image_size, 0) != dimensions.0
            || u32_at(image_size, 4) != dimensions.1
            || u32_at(image_size, 8) != dimensions.0
            || u32_at(image_size, 12) != dimensions.1
        {
            return Err("Picture size records disagree with PNG dimensions".into());
        }
        if bytes[transform.payload.clone()]
            .chunks_exact(8)
            .any(|chunk| !f64::from_le_bytes(chunk.try_into().unwrap()).is_finite())
        {
            return Err("Picture transform contains a non-finite value".into());
        }
        let crops: Vec<CroppedPicture> = records
            .windows(4)
            .filter_map(|window| {
                let [header, refs, size, end] = window else {
                    return None;
                };
                if [header.kind, refs.kind, size.kind, end.kind]
                    != [OBJECT_HEADER, 0x07d2, IMAGE_SIZE, OBJECT_END]
                    || header.payload.len() != 28
                    || u32_at(bytes, header.payload.start) as u16 != 108
                    || refs.payload.len() != 12
                    || u32_at(bytes, refs.payload.start) != 2
                    || size.payload.len() != 16
                    || !end.payload.is_empty()
                {
                    return None;
                }
                Some(CroppedPicture {
                    group_offset: header.start,
                    source: u32_at(bytes, refs.payload.start + 4),
                    polygon: u32_at(bytes, refs.payload.start + 8),
                    dimensions: (
                        u32_at(bytes, size.payload.start),
                        u32_at(bytes, size.payload.start + 4),
                    ),
                    visible: u32_at(bytes, header.payload.start) & 0x1_0000 == 0,
                })
            })
            .collect();
        let editable = records.iter().filter(|r| r.kind == IMAGE_SIZE).count() == 1 + crops.len()
            && records.iter().filter(|r| r.kind == IMAGE_TRANSFORM).count() == 1
            && records.iter().filter(|r| r.kind == IMAGE_RESOURCE).count() == 1
            && crops.iter().all(|crop| crop.dimensions == dimensions);
        Ok(Self {
            bytes: bytes.to_vec(),
            resource: (*resource).clone(),
            header: header.clone(),
            size: size.clone(),
            transform: transform.clone(),
            group_end: terminator.end,
            crops,
            editable,
        })
    }

    pub fn encode(&self) -> &[u8] {
        &self.bytes
    }

    pub fn png(&self) -> &[u8] {
        &self.bytes[self.resource.payload.start + 8..self.resource.payload.end]
    }

    pub fn resource_record(&self) -> &[u8] {
        &self.bytes[self.resource.start..self.resource.end]
    }

    pub fn picture_group(&self) -> &[u8] {
        &self.bytes[self.header.start..self.group_end]
    }

    pub fn picture_group_offset(&self) -> usize {
        self.header.start
    }

    pub fn png_size(png: &[u8]) -> Result<(u32, u32), String> {
        png_dimensions(png)
    }

    pub fn independent_from_png(png: &[u8], transform: [f64; 6]) -> Result<Self, String> {
        let mut picture = Self::parse(include_bytes!(
            "../../../tests/fixtures/image_port_tiny.gsp"
        ))?;
        picture.replace_png(png)?;
        picture.set_transform(transform)?;
        Ok(picture)
    }

    pub fn parallelogram_from_png(
        png: &[u8],
        top_left: u32,
        bottom_right: u32,
        bottom_left: u32,
    ) -> Result<(Vec<u8>, Vec<u8>), String> {
        if [top_left, bottom_right, bottom_left].contains(&0) {
            return Err("Picture attachment references must be positive".into());
        }
        let dimensions = png_dimensions(png)?;
        let mut group = Vec::new();
        put_record(&mut group, OBJECT_HEADER, &PARALLELOGRAM_HEADER)?;
        let mut references = 3_u32.to_le_bytes().to_vec();
        for id in [top_left, bottom_right, bottom_left] {
            references.extend_from_slice(&id.to_le_bytes());
        }
        put_record(&mut group, 0x07d2, &references)?;
        let mut size = Vec::with_capacity(16);
        for value in [dimensions.0, dimensions.1, dimensions.0, dimensions.1] {
            size.extend_from_slice(&value.to_le_bytes());
        }
        put_record(&mut group, IMAGE_SIZE, &size)?;
        put_record(&mut group, 0x07d5, &INTERNAL_IMAGE_LABEL)?;
        put_record(&mut group, IMAGE_RESOURCE, &0_u16.to_le_bytes())?;
        put_record(&mut group, OBJECT_END, &[])?;

        Ok((group, picture_resource(png, dimensions)?))
    }

    pub fn point_from_png(
        png: &[u8],
        anchor: u32,
        relative: [f64; 6],
        resource_id: u16,
    ) -> Result<(Vec<u8>, Vec<u8>), String> {
        if anchor == 0 || relative.iter().any(|value| !value.is_finite()) {
            return Err("Invalid point-attached picture placement".into());
        }
        let dimensions = png_dimensions(png)?;
        let [a, b, x, c, d, y] = relative;
        let scale_x = a.hypot(c);
        let scale_y = b.hypot(d);
        if scale_x <= 0. || scale_y <= 0. {
            return Err("Point-attached picture scale must be positive".into());
        }
        let sin = -c / scale_x;
        let cos = a / scale_x;
        let expected = [
            scale_y * sin,
            scale_y * cos,
            -a * f64::from(dimensions.0) / 2. - b * f64::from(dimensions.1) / 2.,
            -c * f64::from(dimensions.0) / 2. - d * f64::from(dimensions.1) / 2.,
        ];
        let observed = [b, d, x, y];
        if observed
            .iter()
            .zip(expected)
            .any(|(observed, expected)| (observed - expected).abs() > 1e-9)
        {
            return Err("Point-attached picture affine transform is unsupported".into());
        }

        let mut group = Vec::new();
        put_record(&mut group, OBJECT_HEADER, &POINT_HEADER)?;
        let mut references = 1_u32.to_le_bytes().to_vec();
        references.extend_from_slice(&anchor.to_le_bytes());
        put_record(&mut group, 0x07d2, &references)?;
        let mut operation = 99_u32.to_le_bytes().to_vec();
        for value in [scale_x, scale_y, sin.atan2(cos)] {
            operation.extend_from_slice(&value.to_le_bytes());
        }
        put_record(&mut group, 0x07d3, &operation)?;
        let mut size = Vec::with_capacity(16);
        for value in [dimensions.0, dimensions.1, dimensions.0, dimensions.1] {
            size.extend_from_slice(&value.to_le_bytes());
        }
        put_record(&mut group, IMAGE_SIZE, &size)?;
        put_record(&mut group, 0x07d5, &INTERNAL_IMAGE_LABEL)?;
        put_record(&mut group, IMAGE_RESOURCE, &resource_id.to_le_bytes())?;
        put_record(&mut group, OBJECT_END, &[])?;
        Ok((group, picture_resource(png, dimensions)?))
    }

    pub fn crops(&self) -> &[CroppedPicture] {
        &self.crops
    }

    pub fn cropped_picture_group(
        source: u32,
        polygon: u32,
        dimensions: (u32, u32),
    ) -> Result<Vec<u8>, String> {
        let template = include_bytes!("../../../tests/fixtures/image_original_cropped.gsp");
        let records = records(template)?;
        let group = records
            .windows(4)
            .find(|window| {
                window[0].kind == OBJECT_HEADER
                    && window[0].payload.len() == 28
                    && u32_at(template, window[0].payload.start) as u16 == 108
                    && window[1].kind == 0x07d2
                    && window[1].payload.len() == 12
                    && window[2].kind == IMAGE_SIZE
                    && window[2].payload.len() == 16
                    && window[3].kind == OBJECT_END
            })
            .ok_or("Missing cropped-picture template")?;
        let mut bytes = template[group[0].start..group[3].end].to_vec();
        let refs = group[1].payload.start - group[0].start;
        bytes[refs + 4..refs + 8].copy_from_slice(&source.to_le_bytes());
        bytes[refs + 8..refs + 12].copy_from_slice(&polygon.to_le_bytes());
        let size = group[2].payload.start - group[0].start;
        for at in [0, 8] {
            bytes[size + at..size + at + 4].copy_from_slice(&dimensions.0.to_le_bytes());
            bytes[size + at + 4..size + at + 8].copy_from_slice(&dimensions.1.to_le_bytes());
        }
        Ok(bytes)
    }

    pub fn can_edit(&self) -> bool {
        self.editable
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (
            u32_at(&self.bytes, self.size.payload.start),
            u32_at(&self.bytes, self.size.payload.start + 4),
        )
    }

    pub fn transform(&self) -> [f64; 6] {
        std::array::from_fn(|i| {
            f64::from_le_bytes(
                self.bytes[self.transform.payload.start + 8 * i
                    ..self.transform.payload.start + 8 * (i + 1)]
                    .try_into()
                    .unwrap(),
            )
        })
    }

    pub fn set_transform(&mut self, transform: [f64; 6]) -> Result<(), String> {
        if !self.editable {
            return Err("Picture has derived images whose rewrite is not verified".into());
        }
        if transform.iter().any(|value| !value.is_finite()) {
            return Err("Picture transform must be finite".into());
        }
        for (i, value) in transform.iter().enumerate() {
            let at = self.transform.payload.start + 8 * i;
            self.bytes[at..at + 8].copy_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }

    pub fn visible(&self) -> bool {
        u32_at(&self.bytes, self.header.payload.start) & 0x1_0000 == 0
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<(), String> {
        if !self.editable {
            return Err("Picture has derived images whose rewrite is not verified".into());
        }
        let at = self.header.payload.start;
        let mut raw = u32_at(&self.bytes, at);
        if visible {
            raw &= !0x1_0000
        } else {
            raw |= 0x1_0000
        };
        self.bytes[at..at + 4].copy_from_slice(&raw.to_le_bytes());
        Ok(())
    }

    pub fn replace_png(&mut self, png: &[u8]) -> Result<(), String> {
        if !self.editable || !self.crops.is_empty() {
            return Err("Picture has derived images whose rewrite is not verified".into());
        }
        let (width, height) = png_dimensions(png)?;
        let mut changed = self.bytes.clone();
        for at in [0, 8] {
            changed[self.size.payload.start + at..self.size.payload.start + at + 4]
                .copy_from_slice(&width.to_le_bytes());
            changed[self.size.payload.start + at + 4..self.size.payload.start + at + 8]
                .copy_from_slice(&height.to_le_bytes());
        }
        let payload_len = 8usize
            .checked_add(png.len())
            .ok_or("Picture resource length overflow")?;
        let payload_len =
            u32::try_from(payload_len).map_err(|_| "Picture resource is too large")?;
        let mut record = Vec::with_capacity(8 + payload_len as usize + payload_len as usize % 2);
        record.extend_from_slice(&payload_len.to_le_bytes());
        record.extend_from_slice(&RESOURCE.to_le_bytes());
        record.extend_from_slice(&width.to_le_bytes());
        record.extend_from_slice(&height.to_le_bytes());
        record.extend_from_slice(png);
        if payload_len % 2 != 0 {
            record.push(0);
        }
        changed.splice(self.resource.start..self.resource.end, record);
        *self = Self::parse(&changed)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_picture_round_trips_and_transform_is_local() {
        let source = include_bytes!("../../../tests/fixtures/image_original_insert.gsp");
        let mut picture = ImageFile::parse(source).unwrap();
        assert_eq!(picture.encode(), source);
        assert_eq!(picture.dimensions(), (255, 158));
        assert_eq!(picture.transform(), [1., 0., 118., 0., 1., 112.]);
        let original_png = picture.png().to_vec();
        picture.set_visible(false).unwrap();
        picture.set_transform([1., 0., 140., 0., 1., 112.]).unwrap();
        let reopened = ImageFile::parse(picture.encode()).unwrap();
        assert!(!reopened.visible());
        assert_eq!(reopened.transform()[2], 140.);
        assert_eq!(reopened.png(), original_png);
    }

    #[test]
    fn replaces_png_without_touching_other_records() {
        let source = include_bytes!("../../../tests/fixtures/image_original_insert.gsp");
        let tiny = include_bytes!("../../../tests/fixtures/image_tiny.png");
        let mut picture = ImageFile::parse(source).unwrap();
        picture.replace_png(tiny).unwrap();
        assert_eq!(picture.dimensions(), (16, 16));
        assert_eq!(picture.png(), tiny);
        assert_eq!(
            ImageFile::parse(picture.encode()).unwrap().encode(),
            picture.encode()
        );
    }

    #[test]
    fn corrupt_png_and_invalid_transform_are_rejected_transactionally() {
        let source = include_bytes!("../../../tests/fixtures/image_original_insert.gsp");
        let mut picture = ImageFile::parse(source).unwrap();
        let mut corrupt = picture.png().to_vec();
        corrupt[50] ^= 1;
        assert!(picture.replace_png(&corrupt).is_err());
        assert_eq!(picture.encode(), source);
        assert!(
            picture
                .set_transform([f64::NAN, 0., 0., 0., 1., 0.])
                .is_err()
        );
        assert_eq!(picture.encode(), source);
    }

    #[test]
    fn original_drag_reencodes_picture_and_updates_translation() {
        let edited = include_bytes!("../../../tests/fixtures/image_port_dragged_wine.gsp");
        let picture = ImageFile::parse(edited).unwrap();
        assert_eq!(picture.dimensions(), (16, 16));
        assert_eq!(picture.transform(), [4., 0., 150., 0., 4., 132.]);
        assert_eq!(picture.encode(), edited);
    }

    #[test]
    fn crop_is_decoded_and_known_fields_remain_editable() {
        let cropped = include_bytes!("../../../tests/fixtures/image_original_cropped.gsp");
        let mut picture = ImageFile::parse(cropped).unwrap();
        assert_eq!(picture.encode(), cropped);
        assert_eq!(
            imported_cropped_image_catalog(cropped).unwrap(),
            picture.crops()
        );
        assert!(picture.can_edit());
        assert_eq!(
            picture.crops(),
            &[CroppedPicture {
                group_offset: picture.crops()[0].group_offset,
                source: 1,
                polygon: 5,
                dimensions: (16, 16),
                visible: true,
            }]
        );
        assert!(!picture.visible());
        picture.set_visible(true).unwrap();
        picture.set_transform([4., 0., 0., 0., 4., 0.]).unwrap();
        let edited = picture.encode().to_vec();
        assert!(
            picture
                .replace_png(include_bytes!("../../../tests/fixtures/image_tiny.png"))
                .is_err()
        );
        assert_eq!(picture.encode(), edited);
        let reopened = ImageFile::parse(picture.encode()).unwrap();
        assert!(reopened.visible());
        assert_eq!(reopened.transform(), [4., 0., 0., 0., 4., 0.]);
    }

    #[test]
    fn multi_resource_paste_is_outside_the_safe_writer() {
        let pasted = include_bytes!("../../../tests/fixtures/image_original_pasted.gsp");
        assert!(ImageFile::parse(pasted).is_err());
    }

    #[test]
    fn one_picture_with_independent_geometry_remains_editable() {
        let source = include_bytes!("../../../tests/fixtures/image_port_polygon.gsp");
        let mut picture = ImageFile::parse(source).unwrap();
        assert!(picture.can_edit());
        assert_eq!(picture.picture_group()[..8], [28, 0, 0, 0, 0xd0, 7, 0, 0]);
        assert_eq!(&picture.resource_record()[4..8], &RESOURCE.to_le_bytes());
        picture.set_transform([4., 0., 140., 0., 4., 112.]).unwrap();
        let reopened = ImageFile::parse(picture.encode()).unwrap();
        assert_eq!(reopened.transform()[2], 140.);
        assert_eq!(reopened.encode().len(), source.len());
    }

    #[test]
    fn imported_png_builds_original_compatible_records() {
        let png = include_bytes!("../../../tests/fixtures/image_tiny.png");
        let picture = ImageFile::independent_from_png(png, [2., 0., 30., 0., 2., 40.]).unwrap();
        assert_eq!(picture.png(), png);
        assert_eq!(picture.transform(), [2., 0., 30., 0., 2., 40.]);
        assert_eq!(ImageFile::png_size(png).unwrap(), (16, 16));
        assert!(ImageFile::parse(picture.encode()).unwrap().can_edit());
    }
}
