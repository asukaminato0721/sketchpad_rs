//! Lossless helpers for the observed GSP4 embedded picture layout.
//!
//! This deliberately supports only a single independent affine picture with
//! one embedded PNG resource. Other picture constructions must be probed before
//! they are made writable.

use std::ops::Range;

const RESOURCE: u32 = 0x0514;
const OBJECT_HEADER: u32 = 0x07d0;
const IMAGE_SIZE: u32 = 0x090c;
const IMAGE_TRANSFORM: u32 = 0x08a8;
const IMAGE_RESOURCE: u32 = 0x1f44;
const OBJECT_END: u32 = 0x07d7;
const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

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
    crops: Vec<CroppedPicture>,
    editable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CroppedPicture {
    pub source: u32,
    pub polygon: u32,
    pub dimensions: (u32, u32),
    pub visible: bool,
}

fn u32_at(data: &[u8], start: usize) -> u32 {
    u32::from_le_bytes(data[start..start + 4].try_into().unwrap())
}

fn records(bytes: &[u8]) -> Result<Vec<Record>, String> {
    if bytes.get(..4) != Some(b"GSP4") {
        return Err("Expected a GSP4 document".into());
    }
    let mut at = 4usize;
    let mut records = Vec::new();
    while at < bytes.len() {
        if bytes.len() - at < 8 {
            return Err("Truncated GSP record header".into());
        }
        let len = u32_at(bytes, at) as usize;
        let kind = u32_at(bytes, at + 4);
        let start = at.checked_add(8).ok_or("GSP record offset overflow")?;
        let payload_end = start.checked_add(len).ok_or("GSP record length overflow")?;
        let end = payload_end
            .checked_add(len % 2)
            .ok_or("GSP padding overflow")?;
        if end > bytes.len() {
            return Err("Truncated GSP record payload".into());
        }
        records.push(Record {
            start: at,
            kind,
            payload: start..payload_end,
            end,
        });
        at = end;
    }
    Ok(records)
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
            || terminator.payload.len() != 0
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
        let crops = records
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
        let editable = records.iter().filter(|r| r.kind == IMAGE_SIZE).count() == 1
            && records.iter().filter(|r| r.kind == IMAGE_TRANSFORM).count() == 1
            && records.iter().filter(|r| r.kind == IMAGE_RESOURCE).count() == 1
            && records.iter().filter(|r| r.kind == OBJECT_HEADER).count() == 1;
        Ok(Self {
            bytes: bytes.to_vec(),
            resource: (*resource).clone(),
            header: header.clone(),
            size: size.clone(),
            transform: transform.clone(),
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

    pub fn crops(&self) -> &[CroppedPicture] {
        &self.crops
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
        if !self.editable {
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
        let source = include_bytes!("../tests/fixtures/image_original_insert.gsp");
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
        let source = include_bytes!("../tests/fixtures/image_original_insert.gsp");
        let tiny = include_bytes!("../tests/fixtures/image_tiny.png");
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
        let source = include_bytes!("../tests/fixtures/image_original_insert.gsp");
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
        let edited = include_bytes!("../tests/fixtures/image_port_dragged_wine.gsp");
        let picture = ImageFile::parse(edited).unwrap();
        assert_eq!(picture.dimensions(), (16, 16));
        assert_eq!(picture.transform(), [4., 0., 150., 0., 4., 132.]);
        assert_eq!(picture.encode(), edited);
    }

    #[test]
    fn crop_is_decoded_but_blocks_unverified_picture_edits() {
        let cropped = include_bytes!("../tests/fixtures/image_original_cropped.gsp");
        let mut picture = ImageFile::parse(cropped).unwrap();
        assert_eq!(picture.encode(), cropped);
        assert!(!picture.can_edit());
        assert_eq!(
            picture.crops(),
            &[CroppedPicture {
                source: 1,
                polygon: 5,
                dimensions: (16, 16),
                visible: true,
            }]
        );
        assert!(!picture.visible());
        assert!(picture.set_visible(true).is_err());
        assert!(picture.set_transform([4., 0., 0., 0., 4., 0.]).is_err());
        assert!(
            picture
                .replace_png(include_bytes!("../tests/fixtures/image_tiny.png"))
                .is_err()
        );
        assert_eq!(picture.encode(), cropped);
    }

    #[test]
    fn multi_resource_paste_is_outside_the_safe_writer() {
        let pasted = include_bytes!("../tests/fixtures/image_original_pasted.gsp");
        assert!(ImageFile::parse(pasted).is_err());
    }
}
