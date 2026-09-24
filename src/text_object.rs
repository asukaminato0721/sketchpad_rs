//! Encoder for an original Sketchpad 5 free-text object's literal UTF-8 run.
//!
//! The Text tool stores the byte length twice: as hexadecimal text after
//! `<T`, and as the 0x08fc markup byte count. The 0x08fc inner size is rounded
//! to two bytes; its record payload is rounded to four bytes.

const OBJECT_HEADER: [u8; 28] = [
    0, 0, 0, 0, 0, 0, 0, 0, 19, 0, 33, 1, 0, 0, 0, 0, 255, 255, 57, 32, 193, 189, 240, 255, 0, 0,
    0, 0,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextRect {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}

impl TextRect {
    pub fn roomy_at(left: i16, top: i16, text: &str) -> Result<Self, String> {
        let mut width: i32 = 8;
        let mut lines: i32 = 0;
        for line in text.split('\n') {
            lines += 1;
            let mut line_width = 8_i32;
            for ch in line.chars() {
                line_width = line_width
                    .checked_add(if ch.is_ascii() { 10 } else { 22 })
                    .ok_or("Text box width overflow")?;
            }
            width = width.max(line_width);
        }
        let right =
            i16::try_from(i32::from(left) + width).map_err(|_| "Text box x coordinate overflow")?;
        let height = 23_i32
            .checked_add(
                (lines - 1)
                    .checked_mul(19)
                    .ok_or("Text box height overflow")?,
            )
            .ok_or("Text box height overflow")?;
        let bottom =
            i16::try_from(i32::from(top) + height).map_err(|_| "Text box y coordinate overflow")?;
        Ok(Self {
            left,
            top,
            right,
            bottom,
        })
    }

    fn validate(self) -> Result<(), String> {
        if self.right <= self.left || self.bottom <= self.top {
            Err("Text rectangle must have positive width and height".into())
        } else {
            Ok(())
        }
    }
}

pub fn encode_plain_text_group(text: &str, rect: TextRect) -> Result<Vec<u8>, String> {
    if text.is_empty() {
        return Err("Text object cannot be empty".into());
    }
    if text.bytes().any(|byte| matches!(byte, 0 | b'\r')) {
        return Err("Text contains a NUL or carriage return".into());
    }
    rect.validate()?;

    let mut markup = b"<VL".to_vec();
    for line in text.split('\n') {
        markup.extend_from_slice(format!("<T{:X}x", line.len()).as_bytes());
        markup.extend_from_slice(line.as_bytes());
        markup.push(b'>');
    }
    markup.push(b'>');
    let markup_len = u16::try_from(markup.len()).map_err(|_| "Text markup exceeds 65535 bytes")?;

    let mut rich = Vec::new();
    rich.extend_from_slice(&0x08fc_u32.to_le_bytes());
    rich.extend_from_slice(&0_u32.to_le_bytes()); // 2-byte aligned inner size
    rich.extend_from_slice(&4_u32.to_le_bytes());
    rich.extend_from_slice(&rect.left.to_le_bytes());
    rich.extend_from_slice(&rect.top.to_le_bytes());
    rich.extend_from_slice(&markup_len.to_le_bytes());
    rich.extend_from_slice(&markup);
    rich.push(0);
    if rich.len() % 2 != 0 {
        rich.push(0);
    }
    let inner_len = u32::try_from(rich.len()).map_err(|_| "Text payload is too large")?;
    rich[4..8].copy_from_slice(&inner_len.to_le_bytes());
    while rich.len() % 4 != 0 {
        rich.push(0);
    }

    let mut bbox = Vec::with_capacity(8);
    for coordinate in [rect.left, rect.top, rect.right, rect.bottom] {
        bbox.extend_from_slice(&coordinate.to_le_bytes());
    }

    let mut group = Vec::new();
    append_record(&mut group, 0x07d0, &OBJECT_HEADER)?;
    append_record(&mut group, 0x08fc, &rich)?;
    append_record(&mut group, 0x07d8, &[0, 0])?;
    append_record(&mut group, 0x0898, &bbox)?;
    append_record(&mut group, 0x07d7, &[])?;
    Ok(group)
}

/// Add one free-text group to a verified blank, one-page GSP4 document.
/// Other groups are refused because their ordinal references need model-level
/// validation before a structural rewrite.
pub fn add_to_blank_document(blank: &[u8], text: &str, rect: TextRect) -> Result<Vec<u8>, String> {
    if blank.get(..4) != Some(b"GSP4") {
        return Err("Expected a GSP4 document".into());
    }
    let mut at = 4;
    let mut page_count_offset = None;
    let mut trailer_offset = None;
    let mut page_count = 0;
    while at < blank.len() {
        if at + 8 > blank.len() {
            return Err("Truncated record header".into());
        }
        let len = u32::from_le_bytes(blank[at..at + 4].try_into().unwrap()) as usize;
        let kind = u32::from_le_bytes(blank[at + 4..at + 8].try_into().unwrap());
        let end = at
            .checked_add(8 + len + len % 2)
            .ok_or("Record length overflow")?;
        if end > blank.len() {
            return Err("Truncated record payload".into());
        }
        match kind {
            0x03e8 => {
                if len < 28 {
                    return Err("Page header is too short".into());
                }
                page_count += 1;
                page_count_offset = Some(at + 8 + 24);
            }
            0x07d0 => return Err("Document is not blank".into()),
            0x03e9 => trailer_offset = Some(at),
            _ => {}
        }
        at = end;
    }
    if page_count != 1 {
        return Err("Expected exactly one page".into());
    }
    let count_at = page_count_offset.unwrap();
    if u32::from_le_bytes(blank[count_at..count_at + 4].try_into().unwrap()) != 0 {
        return Err("Blank page has a nonzero object count".into());
    }
    let trailer = trailer_offset.ok_or("Missing page trailer")?;
    let group = encode_plain_text_group(text, rect)?;
    let mut output = Vec::with_capacity(blank.len() + group.len());
    output.extend_from_slice(&blank[..trailer]);
    output.extend_from_slice(&group);
    output.extend_from_slice(&blank[trailer..]);
    output[count_at..count_at + 4].copy_from_slice(&1_u32.to_le_bytes());
    Ok(output)
}

fn append_record(out: &mut Vec<u8>, kind: u32, payload: &[u8]) -> Result<(), String> {
    let len = u32::try_from(payload.len()).map_err(|_| "Record payload is too large")?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(payload);
    if payload.len() % 2 != 0 {
        out.push(0);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object_group(bytes: &[u8]) -> &[u8] {
        let start = bytes
            .windows(8)
            .position(|w| w == [28, 0, 0, 0, 0xd0, 7, 0, 0])
            .unwrap();
        let end_rel = bytes[start..]
            .windows(8)
            .position(|w| w == [0, 0, 0, 0, 0xd7, 7, 0, 0])
            .unwrap();
        &bytes[start..start + end_rel + 8]
    }

    #[test]
    fn generated_a_group_matches_original_except_opaque_padding() {
        let original = include_bytes!("../tests/fixtures/text_object_original_A.gsp");
        let generated = encode_plain_text_group(
            "A",
            TextRect {
                left: 91,
                top: 81,
                right: 109,
                bottom: 104,
            },
        )
        .unwrap();
        assert_eq!(&generated, object_group(original));
    }

    #[test]
    fn generated_hello_group_matches_original_except_opaque_padding() {
        let original = include_bytes!("../tests/fixtures/text_object_original_Hello.gsp");
        let generated = encode_plain_text_group(
            "Hello",
            TextRect {
                left: 91,
                top: 81,
                right: 137,
                bottom: 104,
            },
        )
        .unwrap();
        assert_eq!(&generated, object_group(original));
    }

    #[test]
    fn generated_multiline_group_matches_original() {
        let original = include_bytes!("../tests/fixtures/text_object_original_multiline.gsp");
        let generated = encode_plain_text_group(
            "A\nB",
            TextRect {
                left: 91,
                top: 81,
                right: 109,
                bottom: 123,
            },
        )
        .unwrap();
        assert_eq!(&generated, object_group(original));
    }

    #[test]
    fn length_prefix_counts_utf8_bytes_and_uses_hex() {
        let group =
            encode_plain_text_group("猫", TextRect::roomy_at(91, 81, "猫").unwrap()).unwrap();
        assert!(
            group
                .windows(b"<VL<T3x\xe7\x8c\xab>>".len())
                .any(|w| w == b"<VL<T3x\xe7\x8c\xab>>")
        );
        let group = encode_plain_text_group(
            "1234567890ABCDEF",
            TextRect::roomy_at(91, 81, "1234567890ABCDEF").unwrap(),
        )
        .unwrap();
        assert!(
            group
                .windows(b"<VL<T10x1234567890ABCDEF>>".len())
                .any(|w| w == b"<VL<T10x1234567890ABCDEF>>")
        );
    }

    #[test]
    fn blank_document_gain_is_one_group_and_one_object_count() {
        let blank = include_bytes!("../tests/fixtures/text_object_blank.gsp");
        let out =
            add_to_blank_document(blank, "Hello", TextRect::roomy_at(91, 81, "Hello").unwrap())
                .unwrap();
        assert_eq!(
            out.len(),
            blank.len()
                + encode_plain_text_group("Hello", TextRect::roomy_at(91, 81, "Hello").unwrap())
                    .unwrap()
                    .len()
        );
        assert!(
            add_to_blank_document(&out, "Again", TextRect::roomy_at(91, 81, "Again").unwrap())
                .is_err()
        );
    }
}
