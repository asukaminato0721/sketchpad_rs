//! Lossless helpers for Sketchpad object label records.
//!
//! A 0x07d5 record lives inside an object group. Its fields describe that
//! object's visible name and its screen offset; it has no object-ordinal
//! reference. The tail after the byte-counted name is deliberately opaque:
//! original files contain nonzero bytes there, so rewriting it is unsafe.

pub const LABEL_AUX_KIND: u32 = 0x07d5;
pub const OBJECT_AUX_KIND: u32 = 0x07d8;
pub const RICH_TEXT_KIND: u32 = 0x08fc;
pub const TEXT_BOX_KIND: u32 = 0x0898;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelAux {
    payload: Vec<u8>,
}

impl LabelAux {
    pub fn parse(payload: &[u8]) -> Result<Self, String> {
        if payload.len() < 24 {
            return Err("Label payload is shorter than 24 bytes".into());
        }
        let name_len = u16::from_le_bytes(payload[22..24].try_into().unwrap()) as usize;
        if name_len > payload.len() - 24 {
            return Err("Label name exceeds its record".into());
        }
        Ok(Self {
            payload: payload.to_vec(),
        })
    }

    pub fn new_point(name: &str, visible: bool, offset: (i16, i16)) -> Result<Self, String> {
        let mut payload = vec![0; 24];
        payload[0..2].copy_from_slice(&6_u16.to_le_bytes());
        payload[2..4].copy_from_slice(&(visible as u16).to_le_bytes());
        payload[6..8].copy_from_slice(&offset.0.to_le_bytes());
        payload[8..10].copy_from_slice(&offset.1.to_le_bytes());
        let mut label = Self { payload };
        label.set_name(name)?;
        Ok(label)
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn into_payload(self) -> Vec<u8> {
        self.payload
    }

    pub fn name_bytes(&self) -> &[u8] {
        let len = u16::from_le_bytes(self.payload[22..24].try_into().unwrap()) as usize;
        &self.payload[24..24 + len]
    }

    pub fn name_utf8(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.name_bytes())
    }

    pub fn trailing_bytes(&self) -> &[u8] {
        &self.payload[24 + self.name_bytes().len()..]
    }

    pub fn visible(&self) -> bool {
        u16::from_le_bytes(self.payload[2..4].try_into().unwrap()) != 0
    }

    pub fn offset(&self) -> (i16, i16) {
        (
            i16::from_le_bytes(self.payload[6..8].try_into().unwrap()),
            i16::from_le_bytes(self.payload[8..10].try_into().unwrap()),
        )
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.payload[2..4].copy_from_slice(&(visible as u16).to_le_bytes());
    }

    pub fn set_offset(&mut self, offset: (i16, i16)) {
        self.payload[6..8].copy_from_slice(&offset.0.to_le_bytes());
        self.payload[8..10].copy_from_slice(&offset.1.to_le_bytes());
    }

    pub fn set_name(&mut self, name: &str) -> Result<(), String> {
        let bytes = name.as_bytes();
        let len = u16::try_from(bytes.len()).map_err(|_| "Label name exceeds 65535 bytes")?;
        let old_parity = self.name_bytes().len() % 2;
        // Preserve the existing suffix when the new name has the same byte
        // parity. Its second byte is nonzero in several original files.
        let suffix = self.trailing_bytes().to_vec();
        if old_parity != bytes.len() % 2 && suffix.get(1).is_some_and(|&b| b != 0) {
            return Err(
                "Cannot change label byte parity without discarding an opaque suffix byte".into(),
            );
        }
        self.payload.truncate(22);
        self.payload.extend_from_slice(&len.to_le_bytes());
        self.payload.extend_from_slice(bytes);
        if old_parity == bytes.len() % 2 && !suffix.is_empty() {
            self.payload.extend_from_slice(&suffix);
        } else {
            self.payload.push(suffix.first().copied().unwrap_or(0));
            if bytes.len() % 2 == 0 {
                self.payload.push(0);
            }
        }
        Ok(())
    }
}

/// The original point-label group places these records immediately after its
/// 0x07d0 header, before its 0x0899 coordinate record.
pub fn encode_point_label_records(label: &LabelAux) -> Vec<u8> {
    let mut out = Vec::new();
    append_record(&mut out, LABEL_AUX_KIND, label.payload());
    append_record(&mut out, OBJECT_AUX_KIND, &[0, 0]);
    out
}

fn append_record(out: &mut Vec<u8>, kind: u32, payload: &[u8]) {
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(payload);
    if payload.len() % 2 != 0 {
        out.push(0);
    }
}

/// Parse a 0x08fc rich-text payload without interpreting its markup.
/// Keeping the whole payload lets the caller change an anchor while carrying
/// forward every styling and expression byte unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RichTextPayload {
    payload: Vec<u8>,
}

impl RichTextPayload {
    pub fn parse(payload: &[u8]) -> Result<Self, String> {
        if payload.len() < 18
            || u32::from_le_bytes(payload[..4].try_into().unwrap()) != RICH_TEXT_KIND
        {
            return Err("Unsupported rich-text payload header".into());
        }
        let markup_len = u16::from_le_bytes(payload[16..18].try_into().unwrap()) as usize;
        if markup_len > payload.len() - 18 {
            return Err("Rich-text markup exceeds its record".into());
        }
        Ok(Self {
            payload: payload.to_vec(),
        })
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn anchor(&self) -> (i16, i16) {
        (
            i16::from_le_bytes(self.payload[12..14].try_into().unwrap()),
            i16::from_le_bytes(self.payload[14..16].try_into().unwrap()),
        )
    }

    pub fn set_anchor(&mut self, anchor: (i16, i16)) {
        self.payload[12..14].copy_from_slice(&anchor.0.to_le_bytes());
        self.payload[14..16].copy_from_slice(&anchor.1.to_le_bytes());
    }

    pub fn markup_bytes(&self) -> &[u8] {
        let len = u16::from_le_bytes(self.payload[16..18].try_into().unwrap()) as usize;
        &self.payload[18..18 + len]
    }

    /// Decode only the simple literal form used by the original plain text
    /// fixture. Dynamic text, formatting runs, and hot references stay raw.
    pub fn simple_text(&self) -> Option<&str> {
        let markup = self.markup_bytes();
        let content = markup.strip_prefix(b"<VL<T4x")?.strip_suffix(b">>")?;
        (!content.contains(&b'<') && !content.contains(&b'>'))
            .then(|| std::str::from_utf8(content).ok())
            .flatten()
    }

    pub fn replace_simple_text(&mut self, replacement: &str) -> Result<(), String> {
        let old = self
            .simple_text()
            .ok_or("Rich text is not a simple literal label")?;
        if replacement.len() != old.len() {
            return Err("Changing text byte length requires unsupported layout updates".into());
        }
        if replacement
            .bytes()
            .any(|byte| matches!(byte, b'<' | b'>' | 0))
        {
            return Err("Literal text contains a markup delimiter".into());
        }
        let old_suffix = &self.payload[18 + self.markup_bytes().len()..];
        let internal_size = u32::from_le_bytes(self.payload[4..8].try_into().unwrap()) as usize;
        if internal_size != self.payload.len() || old_suffix.iter().any(|&byte| byte != 0) {
            return Err("Rich text has unsupported trailing data".into());
        }
        self.payload[25..25 + replacement.len()].copy_from_slice(replacement.as_bytes());
        Ok(())
    }
}

/// Two opposite corners of a plain text object's 0x0898 screen rectangle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextBox {
    payload: [u8; 8],
}

impl TextBox {
    pub fn parse(payload: &[u8]) -> Result<Self, String> {
        let payload: [u8; 8] = payload.try_into().map_err(|_| "Text box must be 8 bytes")?;
        Ok(Self { payload })
    }

    pub fn corners(&self) -> ((i16, i16), (i16, i16)) {
        let word = |at| i16::from_le_bytes(self.payload[at..at + 2].try_into().unwrap());
        ((word(0), word(2)), (word(4), word(6)))
    }

    pub fn translate(&mut self, dx: i16, dy: i16) -> Result<(), String> {
        let ((x1, y1), (x2, y2)) = self.corners();
        let words = [
            x1.checked_add(dx).ok_or("Text x coordinate overflow")?,
            y1.checked_add(dy).ok_or("Text y coordinate overflow")?,
            x2.checked_add(dx).ok_or("Text x coordinate overflow")?,
            y2.checked_add(dy).ok_or("Text y coordinate overflow")?,
        ];
        for (i, value) in words.into_iter().enumerate() {
            self.payload[i * 2..i * 2 + 2].copy_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }

    pub fn payload(&self) -> &[u8; 8] {
        &self.payload
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record_payloads(bytes: &[u8], kind: u32) -> Vec<&[u8]> {
        assert_eq!(&bytes[..4], b"GSP4");
        let mut out = Vec::new();
        let mut i = 4;
        while i < bytes.len() {
            let len = u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap()) as usize;
            let record_kind = u32::from_le_bytes(bytes[i + 4..i + 8].try_into().unwrap());
            if record_kind == kind {
                out.push(&bytes[i + 8..i + 8 + len]);
            }
            i += 8 + len + len % 2;
        }
        out
    }

    #[test]
    fn point_fixture_label_decodes_and_reencodes_exactly() {
        let bytes = include_bytes!("../tests/fixtures/text_point_label.gsp");
        let payload = record_payloads(bytes, LABEL_AUX_KIND)[0];
        let label = LabelAux::parse(payload).unwrap();
        assert_eq!(label.name_utf8().unwrap(), "A");
        assert_eq!(label.offset(), (8, 5));
        assert!(label.visible());
        assert_eq!(label.payload(), payload);
        assert_eq!(
            LabelAux::new_point("A", true, (8, 5)).unwrap().payload(),
            payload
        );
        let block = encode_point_label_records(&label);
        let start = bytes
            .windows(8)
            .position(|part| part == [26, 0, 0, 0, 0xd5, 7, 0, 0])
            .unwrap();
        assert_eq!(&block, &bytes[start..start + block.len()]);
    }

    #[test]
    fn hidden_ray_label_and_nonzero_tail_are_preserved() {
        let bytes = include_bytes!("../tests/fixtures/text_ray_label_hide.gsp");
        let payloads = record_payloads(bytes, LABEL_AUX_KIND);
        let hidden = LabelAux::parse(payloads.last().unwrap()).unwrap();
        assert_eq!(hidden.name_utf8().unwrap(), "k");
        assert!(!hidden.visible());
        assert_eq!(hidden.offset(), (-11, 5));
        assert_eq!(hidden.payload(), *payloads.last().unwrap());

        let mut unusual = LabelAux::parse(&[
            6, 0, 1, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, b't', b'[',
            b'1', b']', 0, 0x78,
        ])
        .unwrap();
        unusual.set_name("P[1]").unwrap();
        assert_eq!(unusual.name_utf8().unwrap(), "P[1]");
        assert_eq!(unusual.trailing_bytes(), &[0, 0x78]);
    }

    #[test]
    fn rich_text_anchor_can_change_without_touching_markup() {
        let bytes = include_bytes!("../tests/fixtures/text_on_point.gsp");
        let payload = record_payloads(bytes, RICH_TEXT_KIND)[0];
        let mut text = RichTextPayload::parse(payload).unwrap();
        assert_eq!(text.anchor(), (102, 78));
        assert_eq!(text.markup_bytes(), b"<VL<T4xtext>>");
        assert_eq!(text.simple_text(), Some("text"));
        text.set_anchor((222, -17));
        assert_eq!(text.anchor(), (222, -17));
        assert_eq!(&text.payload()[..12], &payload[..12]);
        assert_eq!(&text.payload()[16..], &payload[16..]);
        let mut bbox = TextBox::parse(record_payloads(bytes, TEXT_BOX_KIND)[0]).unwrap();
        assert_eq!(bbox.corners(), ((102, 78), (139, 101)));
        bbox.translate(120, -95).unwrap();
        assert_eq!(bbox.corners(), ((222, -17), (259, 6)));
    }

    #[test]
    fn simple_text_replacement_preserves_wrapper_and_lengths() {
        let bytes = include_bytes!("../tests/fixtures/text_on_point.gsp");
        let payload = record_payloads(bytes, RICH_TEXT_KIND)[0];
        let mut text = RichTextPayload::parse(payload).unwrap();
        text.replace_simple_text("More").unwrap();
        assert_eq!(text.simple_text(), Some("More"));
        assert_eq!(text.payload().len() % 2, 0);
        assert_eq!(
            u32::from_le_bytes(text.payload()[4..8].try_into().unwrap()) as usize,
            text.payload().len()
        );
        assert_eq!(RichTextPayload::parse(text.payload()).unwrap(), text);
        assert!(text.replace_simple_text("<expr>").is_err());
        assert!(text.replace_simple_text("Hello").is_err());
    }

    #[test]
    fn dragged_label_retains_object_position_and_changes_signed_offset() {
        let bytes = include_bytes!("../tests/fixtures/text_point_label_dragged.gsp");
        let label = LabelAux::parse(record_payloads(bytes, LABEL_AUX_KIND)[0]).unwrap();
        assert_eq!(label.name_utf8().unwrap(), "A");
        assert_eq!(label.offset(), (8, -34));
    }

    #[test]
    fn rename_rejects_opaque_tail_loss() {
        let mut label = LabelAux::parse(&[
            6, 0, 1, 0, 0, 0, 8, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, b't', b'[',
            b'1', b']', 0, 0x78,
        ])
        .unwrap();
        let original = label.payload().to_vec();
        assert!(label.set_name("ABC").is_err());
        assert_eq!(label.payload(), original);
    }

    #[test]
    fn malformed_lengths_fail_before_field_access() {
        assert!(LabelAux::parse(&[0; 23]).is_err());
        let mut truncated = [0; 24];
        truncated[22] = 3;
        assert!(LabelAux::parse(&truncated).is_err());
        assert!(RichTextPayload::parse(&[0; 17]).is_err());
    }
}
