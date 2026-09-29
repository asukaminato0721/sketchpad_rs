//! Bounded inspection and rewriting of the original application's free-text
//! font records. The writer accepts only the verified one-object Hello fixture.

const HELLO: &[u8] = include_bytes!("../../../tests/fixtures/text_object_generated_Hello.gsp");
const BOLD_OFF: &[u8] = include_bytes!("../../../tests/fixtures/text_style_bold_off.gsp");
const ITALIC_ON: &[u8] = include_bytes!("../../../tests/fixtures/text_style_italic_on.gsp");
const UNDERLINE_ON: &[u8] = include_bytes!("../../../tests/fixtures/text_style_underline_on.gsp");
const LARGER: &[u8] = include_bytes!("../../../tests/fixtures/text_style_larger.gsp");
const CJK_FONT: &[u8] = include_bytes!("../../../tests/fixtures/text_style_font_cjk.gsp");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerifiedHelloStyle {
    BoldOff,
    ItalicOn,
    UnderlineOn,
    IncreaseSize,
    NotoSansCjkJp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontState {
    pub family: String,
    /// The application's size code. The source uses 24; Increase Size uses 36.
    pub size_code: u16,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

#[derive(Clone, Copy)]
struct Record {
    kind: u32,
    start: usize,
    payload: usize,
    end: usize,
}

fn records(bytes: &[u8]) -> Result<Vec<Record>, String> {
    if bytes.get(..4) != Some(b"GSP4") {
        return Err("Expected GSP4".into());
    }
    let mut at = 4;
    let mut out = Vec::new();
    while at < bytes.len() {
        if at + 8 > bytes.len() {
            return Err("Truncated GSP record".into());
        }
        let len = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
        let payload = at + 8;
        let end = payload
            .checked_add(len)
            .and_then(|v| v.checked_add(len % 2))
            .ok_or("GSP record length overflow")?;
        if end > bytes.len() {
            return Err("Truncated GSP payload".into());
        }
        out.push(Record {
            kind: u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap()),
            start: at,
            payload,
            end,
        });
        at = end;
    }
    Ok(out)
}

/// Font tables in embedded tool canvases belong to independent namespaces.
fn drawing_records(records: &[Record]) -> impl Iterator<Item = &Record> {
    let mut depth = 0usize;
    records.iter().filter(move |record| match record.kind {
        0x03ea => {
            depth += 1;
            false
        }
        0x03eb if depth > 0 => {
            depth -= 1;
            false
        }
        _ => depth == 0,
    })
}

#[cfg(test)]
fn one<'a>(records: &[Record], bytes: &'a [u8], kind: u32) -> Result<&'a [u8], String> {
    let mut matches = records.iter().filter(|r| r.kind == kind);
    let record = matches.next().ok_or("Missing font record")?;
    if matches.next().is_some() {
        return Err("Multiple font records".into());
    }
    let len =
        u32::from_le_bytes(bytes[record.start..record.start + 4].try_into().unwrap()) as usize;
    Ok(&bytes[record.payload..record.payload + len])
}

/// Read the active font fields, while rejecting unknown Boolean encodings.
pub fn decode_font_state(bytes: &[u8]) -> Result<FontState, String> {
    decode_font_state_at(bytes, 0)
}

/// Resolve the style index stored in a free-text object's 0x07d8 record.
pub fn decode_font_state_at(bytes: &[u8], index: usize) -> Result<FontState, String> {
    let rs = records(bytes)?;
    let record = drawing_records(&rs)
        .filter(|record| record.kind == 0x04b3)
        .nth(index)
        .ok_or("Missing indexed font style")?;
    let len =
        u32::from_le_bytes(bytes[record.start..record.start + 4].try_into().unwrap()) as usize;
    let style = &bytes[record.payload..record.payload + len];
    if style.len() != 20 {
        return Err("Unsupported font style record".into());
    }
    let family_index = u16::from_le_bytes(style[..2].try_into().unwrap()) as usize;
    let family_record = drawing_records(&rs)
        .filter(|record| record.kind == 0x04b1)
        .nth(family_index)
        .ok_or("Missing indexed font family")?;
    let family_len = u32::from_le_bytes(
        bytes[family_record.start..family_record.start + 4]
            .try_into()
            .unwrap(),
    ) as usize;
    let family_record = &bytes[family_record.payload..family_record.payload + family_len];
    decode_font_payloads(style, family_record)
}

pub(crate) fn decode_font_payloads(
    style: &[u8],
    family_record: &[u8],
) -> Result<FontState, String> {
    if style.len() != 20 {
        return Err("Unsupported font style record".into());
    }
    let family = decode_font_family_payload(family_record)?;
    let word = |at| u16::from_le_bytes(style[at..at + 2].try_into().unwrap());
    let flag = |at| -> Result<bool, String> {
        match word(at) {
            1 => Ok(false),
            2 => Ok(true),
            _ => Err("Unsupported text style flag".into()),
        }
    };
    let size_code = word(2);
    if !(8..=512).contains(&size_code) {
        return Err("Unsupported text size".into());
    }
    Ok(FontState {
        family,
        size_code,
        bold: flag(4)?,
        italic: flag(6)?,
        underline: flag(8)?,
    })
}

pub fn encode_style_payload(template: &[u8], style: &FontState) -> Result<[u8; 20], String> {
    if template.len() != 20 || !(8..=512).contains(&style.size_code) {
        return Err("Unsupported font style template".into());
    }
    let mut payload: [u8; 20] = template.try_into().unwrap();
    payload[2..4].copy_from_slice(&style.size_code.to_le_bytes());
    for (at, enabled) in [(4, style.bold), (6, style.italic), (8, style.underline)] {
        payload[at..at + 2].copy_from_slice(&(if enabled { 2u16 } else { 1u16 }).to_le_bytes());
    }
    Ok(payload)
}

pub fn font_families(bytes: &[u8]) -> Result<Vec<String>, String> {
    let rs = records(bytes)?;
    drawing_records(&rs)
        .filter(|record| record.kind == 0x04b1)
        .map(|record| {
            let len = u32::from_le_bytes(bytes[record.start..record.start + 4].try_into().unwrap())
                as usize;
            let payload = &bytes[record.payload..record.payload + len];
            decode_font_family_payload(payload)
        })
        .collect()
}

pub(crate) fn decode_font_family_payload(family_record: &[u8]) -> Result<String, String> {
    if family_record.len() < 4 {
        return Err("Unsupported font family record".into());
    }
    let end = family_record[2..]
        .iter()
        .position(|byte| *byte == 0)
        .ok_or("Unterminated font name")?
        + 2;
    let family = std::str::from_utf8(&family_record[2..end])
        .map_err(|_| "Invalid font name")?
        .to_owned();
    if family.is_empty() || family.len() > 255 {
        return Err("Invalid font name".into());
    }
    Ok(family)
}

pub fn encode_font_family_payload(name: &str) -> Result<Vec<u8>, String> {
    if name.is_empty() || name.len() > 255 || name.chars().any(char::is_control) {
        return Err("Font family is empty, too long, or contains control characters".into());
    }
    let len = (2 + name.len() + 1 + 2).max(12);
    let mut payload = vec![0; len + len % 2];
    payload[2..2 + name.len()].copy_from_slice(name.as_bytes());
    Ok(payload)
}

fn fixture(variant: VerifiedHelloStyle) -> &'static [u8] {
    match variant {
        VerifiedHelloStyle::BoldOff => BOLD_OFF,
        VerifiedHelloStyle::ItalicOn => ITALIC_ON,
        VerifiedHelloStyle::UnderlineOn => UNDERLINE_ON,
        VerifiedHelloStyle::IncreaseSize => LARGER,
        VerifiedHelloStyle::NotoSansCjkJp => CJK_FONT,
    }
}

/// Replace only the font table, active style, rich-text run, and text box
/// records with original-app records from the requested verified variant.
/// The exact source check prevents applying single-object data to sketches
/// with other text, labels, or font references.
pub fn style_verified_hello(source: &[u8], variant: VerifiedHelloStyle) -> Result<Vec<u8>, String> {
    if source != HELLO {
        return Err("Expected the verified unstyled Hello sketch".into());
    }
    let source_records = records(source)?;
    let target = fixture(variant);
    let target_records = records(target)?;
    let selected = [0x04b1, 0x04b3, 0x273c, 0x08fc, 0x0898];
    let mut output = b"GSP4".to_vec();
    for record in &source_records {
        let ordinal = source_records
            .iter()
            .take_while(|candidate| candidate.start < record.start)
            .filter(|candidate| candidate.kind == record.kind)
            .count();
        let replacement = if selected.contains(&record.kind) {
            target_records
                .iter()
                .filter(|candidate| candidate.kind == record.kind)
                .nth(ordinal)
                .ok_or("Missing verified style record")?
        } else {
            record
        };
        let backing = if selected.contains(&record.kind) {
            target
        } else {
            source
        };
        output.extend_from_slice(&backing[replacement.start..replacement.end]);
    }
    let rebuilt = records(&output)?;
    if rebuilt.iter().filter(|r| r.kind == 0x07d0).count() != 1
        || rebuilt.iter().filter(|r| r.kind == 0x08fc).count() != 1
    {
        return Err("Unexpected text object count".into());
    }
    decode_font_state(&output)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_state_matches_original_menu_variants() {
        let baseline = decode_font_state(HELLO).unwrap();
        assert_eq!(baseline.family, "Tahoma");
        assert_eq!(baseline.size_code, 24);
        assert!(baseline.bold);
        assert!(!baseline.italic && !baseline.underline);
        assert!(!decode_font_state(BOLD_OFF).unwrap().bold);
        assert!(decode_font_state(ITALIC_ON).unwrap().italic);
        assert!(decode_font_state(UNDERLINE_ON).unwrap().underline);
        assert_eq!(decode_font_state(LARGER).unwrap().size_code, 36);
        assert_eq!(
            decode_font_state(CJK_FONT).unwrap().family,
            "@Noto Sans CJK JP"
        );
    }

    #[test]
    fn bounded_writer_preserves_other_records() {
        let output = style_verified_hello(HELLO, VerifiedHelloStyle::NotoSansCjkJp).unwrap();
        let rs = records(&output).unwrap();
        assert_eq!(rs.iter().filter(|r| r.kind == 0x07d0).count(), 1);
        assert_eq!(
            decode_font_state(&output).unwrap().family,
            "@Noto Sans CJK JP"
        );
        assert!(style_verified_hello(&output, VerifiedHelloStyle::BoldOff).is_err());
        assert!(
            style_verified_hello(&HELLO[..HELLO.len() - 1], VerifiedHelloStyle::BoldOff).is_err()
        );
        assert!(records(&output[..output.len() - 1]).is_err());
    }

    #[test]
    fn point_label_style_keeps_label_payload() {
        let original = include_bytes!("../../../tests/fixtures/text_point_label.gsp");
        let styled = include_bytes!("../../../tests/fixtures/text_style_label_bold_off.gsp");
        assert!(decode_font_state(original).unwrap().bold);
        assert!(!decode_font_state(styled).unwrap().bold);
        let before = records(original).unwrap();
        let after = records(styled).unwrap();
        assert_eq!(
            one(&before, original, 0x07d5).unwrap(),
            one(&after, styled, 0x07d5).unwrap()
        );
    }

    #[test]
    fn tool_local_font_tables_do_not_consume_drawing_indices() {
        let mut bytes = b"GSP4".to_vec();
        let mut append = |kind: u32, payload: &[u8]| {
            bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&kind.to_le_bytes());
            bytes.extend_from_slice(payload);
            if !payload.len().is_multiple_of(2) {
                bytes.push(0);
            }
        };
        append(0x03ea, &[]);
        let family = encode_font_family_payload("Only in tool").unwrap();
        append(0x04b1, &family);
        let local_style = FontState {
            family: "Only in tool".into(),
            size_code: 18,
            bold: false,
            italic: false,
            underline: false,
        };
        let payload = encode_style_payload(&[0; 20], &local_style).unwrap();
        append(0x04b3, &payload);
        append(0x03eb, &[]);
        bytes.extend_from_slice(&HELLO[4..]);
        assert_eq!(
            font_families(&bytes).unwrap(),
            font_families(HELLO).unwrap()
        );
        assert_eq!(
            decode_font_state_at(&bytes, 0).unwrap(),
            decode_font_state(HELLO).unwrap()
        );
        assert!(decode_font_state_at(&bytes, 1).is_err());
    }

    #[test]
    fn indexed_styles_resolve_independent_font_families() {
        let mixed = include_bytes!("../../../tests/fixtures/text_two_font_cjk_first.gsp");
        assert_eq!(decode_font_state_at(mixed, 0).unwrap().family, "Tahoma");
        assert_eq!(
            decode_font_state_at(mixed, 1).unwrap().family,
            "@Noto Sans CJK JP"
        );
        assert!(decode_font_state_at(mixed, 2).is_err());
    }
}
