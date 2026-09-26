//! Lossless GSP4 record helpers for Sketchpad 5 Hide/Show and default Animation Action Buttons.
//!
//! This module handles one object group. The document writer must still update
//! the page's object count and place the encoded group before its trailer.

pub const OBJECT_HEADER: u32 = 0x07d0;
pub const OBJECT_REFS: u32 = 0x07d2;
pub const ANIMATION_SETTINGS: u32 = 0x07d3;
pub const ACTION_PAYLOAD: u32 = 0x0906;
pub const BUTTON_LABEL: u32 = 0x07d5;
pub const BUTTON_AUX: u32 = 0x07d8;
pub const OBJECT_END: u32 = 0x07d7;
pub const BROKEN_PAGE_LINK_TARGET: usize = 0xbd66;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisibilityAction {
    Hide,
    Show,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationDirection {
    Forward,
    Backward,
    Bidirectional,
    Random,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationSettings {
    pub direction: AnimationDirection,
    pub speed: f32,
    pub once: bool,
    pub parameter: Option<ParameterAnimationSettings>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterAnimationSettings {
    pub discrete: bool,
    pub step: f32,
    pub min: f32,
    pub max: f32,
}

impl Default for ParameterAnimationSettings {
    fn default() -> Self {
        Self {
            discrete: false,
            step: 1.,
            min: -100.,
            max: 100.,
        }
    }
}

fn decode_parameter_number(stored: f32) -> Option<f32> {
    if !stored.is_finite() {
        return None;
    }
    if stored == 0. {
        return Some(0.);
    }
    let exponent = (stored.abs().log2().floor() as i32 - 2).div_euclid(2);
    let value = stored / 2_f32.powi(exponent + 2);
    value.is_finite().then_some(value)
}

fn encode_parameter_number(value: f32) -> Option<f32> {
    if !value.is_finite() {
        return None;
    }
    if value == 0. {
        return Some(0.);
    }
    let exponent = value.abs().log2().floor() as i32;
    let stored = value * 2_f32.powi(exponent + 2);
    stored.is_finite().then_some(stored)
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            direction: AnimationDirection::Bidirectional,
            speed: 24.,
            once: false,
            parameter: None,
        }
    }
}

impl AnimationSettings {
    pub fn validate(self) -> Result<(), String> {
        if !self.speed.is_finite() || self.speed <= 0. || self.speed > 10_000. {
            return Err("Animation speed must be positive and finite".into());
        }
        if self.once && self.direction == AnimationDirection::Bidirectional {
            return Err("Once Only is unavailable for bidirectional motion".into());
        }
        if let Some(parameter) = self.parameter {
            if !parameter.step.is_finite()
                || parameter.step <= 0.
                || !parameter.min.is_finite()
                || !parameter.max.is_finite()
                || parameter.min >= parameter.max
                || [self.speed, parameter.step, parameter.min, parameter.max]
                    .iter()
                    .any(|value| encode_parameter_number(*value).is_none())
            {
                return Err("Invalid parameter Animation settings".into());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Record {
    kind: u32,
    payload: Vec<u8>,
    padding: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HideShowButtonGroup {
    records: [Record; 6],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationButtonGroup {
    records: [Record; 7],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovementSpeed {
    Slow,
    Medium,
    Fast,
    Instant,
}

impl MovementSpeed {
    fn code(self) -> u16 {
        match self {
            Self::Slow => 0,
            Self::Medium => 1,
            Self::Fast => 2,
            Self::Instant => 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovementButtonGroup {
    records: [Record; 6],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollPlacement {
    UpperLeft,
    Center,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollButtonGroup {
    records: [Record; 6],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationMode {
    Simultaneous,
    Sequential,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresentationButtonGroup {
    records: [Record; 6],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkUrlButtonGroup {
    records: [Record; 6],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkPageButtonGroup {
    records: [Record; 6],
}

impl LinkPageButtonGroup {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let records = parse_records(
            bytes,
            [
                OBJECT_HEADER,
                ACTION_PAYLOAD,
                BUTTON_LABEL,
                BUTTON_AUX,
                0x08a2,
                OBJECT_END,
            ],
        )?;
        let group = Self { records };
        group.validate()?;
        Ok(group)
    }

    pub fn from_original_template(
        target_page: usize,
        label: &str,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        let mut group = Self::parse(include_bytes!("../assets/action_link_page_group.bin"))?;
        group.set_target_page(target_page)?;
        group.set_label(label)?;
        group.set_screen_anchor(anchor);
        Ok(group)
    }

    fn validate(&self) -> Result<(), String> {
        let header = &self.records[0].payload;
        let action = &self.records[1].payload;
        let label = &self.records[2].payload;
        if header.len() != 28 || u16_at(header, 0) != 0 {
            return Err("Page Link needs an object code 0 header".into());
        }
        if action.len() != 24
            || u32_at(action, 0) != ACTION_PAYLOAD
            || u32_at(action, 4) != 24
            || u32_at(action, 8) != 12
            || u16_at(action, 12) != 5
        {
            return Err("Unsupported Page Link action".into());
        }
        let broken_link_label =
            if label.len() >= 26 && self.target_page() == BROKEN_PAGE_LINK_TARGET {
                let end = 24 + u16_at(label, 22) as usize;
                end < label.len() && label[end] == 0 && label.len() == (end + 2) & !1
            } else {
                false
            };
        if label.len() < 26
            || !(valid_zero_terminated_text(label, 22) || broken_link_label)
            || self.records[3].payload.len() != 2
            || self.records[4].payload.len() != 8
            || !self.records[5].payload.is_empty()
        {
            return Err("Page Link label or auxiliary records are invalid".into());
        }
        Ok(())
    }

    pub fn target_page(&self) -> usize {
        u16_at(&self.records[1].payload, 14) as usize
    }

    pub fn target_button_ordinal(&self) -> Option<usize> {
        let ordinal = u32_at(&self.records[1].payload, 16) as usize;
        (ordinal != 0).then_some(ordinal)
    }

    pub fn set_target_button_ordinal(&mut self, ordinal: Option<usize>) -> Result<(), String> {
        let value = u32::try_from(ordinal.unwrap_or(0))
            .map_err(|_| "Destination button ordinal is too large")?;
        if ordinal == Some(0) {
            return Err("Destination button ordinals start at 1".into());
        }
        self.records[1].payload[16..20].copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    pub fn set_page_object_index(&mut self, index: usize) -> Result<(), String> {
        let value = u16::try_from(index).map_err(|_| "Page has too many objects")?;
        self.records[3]
            .payload
            .copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    pub fn set_target_page(&mut self, target_page: usize) -> Result<(), String> {
        let page = u16::try_from(target_page).map_err(|_| "Page Link target exceeds 65535")?;
        self.records[1].payload[14..16].copy_from_slice(&page.to_le_bytes());
        Ok(())
    }

    pub fn label_utf8(&self) -> Result<&str, String> {
        let payload = &self.records[2].payload;
        let len = u16_at(payload, 22) as usize;
        std::str::from_utf8(&payload[24..24 + len])
            .map_err(|_| "Page Link label is not UTF-8".into())
    }

    pub fn set_label(&mut self, label: &str) -> Result<(), String> {
        set_zero_terminated_text(&mut self.records[2], 22, label)
    }

    pub fn screen_anchor(&self) -> (i16, i16) {
        let payload = &self.records[1].payload;
        (
            i16::from_le_bytes(payload[20..22].try_into().unwrap()),
            i16::from_le_bytes(payload[22..24].try_into().unwrap()),
        )
    }

    pub fn set_screen_anchor(&mut self, anchor: (i16, i16)) {
        self.records[1].payload[20..22].copy_from_slice(&anchor.0.to_le_bytes());
        self.records[1].payload[22..24].copy_from_slice(&anchor.1.to_le_bytes());
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.records[0].payload[2] = u8::from(hidden);
    }

    pub fn encode(&self) -> Vec<u8> {
        encode_records(&self.records)
    }
}

impl LinkUrlButtonGroup {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let records = parse_records(
            bytes,
            [
                OBJECT_HEADER,
                ACTION_PAYLOAD,
                BUTTON_LABEL,
                BUTTON_AUX,
                0x08a2,
                OBJECT_END,
            ],
        )?;
        let group = Self { records };
        group.validate()?;
        Ok(group)
    }

    pub fn from_original_template(
        url: &str,
        label: &str,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        let mut group = Self::parse(include_bytes!("../assets/action_link_url_group.bin"))?;
        group.set_url(url)?;
        group.set_label(label)?;
        group.set_screen_anchor(anchor);
        Ok(group)
    }

    fn validate(&self) -> Result<(), String> {
        let header = &self.records[0].payload;
        let action = &self.records[1].payload;
        let label = &self.records[2].payload;
        if header.len() != 28 || u16_at(header, 0) != 0 {
            return Err("Link URL button needs an object code 0 header".into());
        }
        if action.len() < 28
            || u32_at(action, 0) != ACTION_PAYLOAD
            || u32_at(action, 4) as usize != action.len()
            || u32_at(action, 8) != 12
            || u16_at(action, 12) != 6
            || u16_at(action, 14) != 0
            || u32_at(action, 16) != 0
            || !valid_zero_terminated_text(action, 24)
        {
            return Err("Unsupported Link URL action".into());
        }
        if label.len() < 26 || !valid_zero_terminated_text(label, 22) {
            return Err("Link URL label is truncated".into());
        }
        if self.records[3].payload.len() != 2
            || self.records[4].payload.len() != 8
            || !self.records[5].payload.is_empty()
        {
            return Err("Link URL auxiliary records are invalid".into());
        }
        Ok(())
    }

    pub fn url_utf8(&self) -> Result<&str, String> {
        let payload = &self.records[1].payload;
        let len = u16_at(payload, 24) as usize;
        std::str::from_utf8(&payload[26..26 + len]).map_err(|_| "Link URL is not UTF-8".into())
    }

    pub fn label_utf8(&self) -> Result<&str, String> {
        let payload = &self.records[2].payload;
        let len = u16_at(payload, 22) as usize;
        std::str::from_utf8(&payload[24..24 + len])
            .map_err(|_| "Link URL label is not UTF-8".into())
    }

    pub fn set_url(&mut self, url: &str) -> Result<(), String> {
        set_zero_terminated_text(&mut self.records[1], 24, url)?;
        let payload_len = self.records[1].payload.len() as u32;
        self.records[1].payload[4..8].copy_from_slice(&payload_len.to_le_bytes());
        Ok(())
    }

    pub fn set_page_object_index(&mut self, index: usize) -> Result<(), String> {
        let value = u16::try_from(index).map_err(|_| "Page has too many objects")?;
        self.records[3]
            .payload
            .copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    pub fn set_label(&mut self, label: &str) -> Result<(), String> {
        set_zero_terminated_text(&mut self.records[2], 22, label)
    }

    pub fn screen_anchor(&self) -> (i16, i16) {
        let payload = &self.records[1].payload;
        (
            i16::from_le_bytes(payload[20..22].try_into().unwrap()),
            i16::from_le_bytes(payload[22..24].try_into().unwrap()),
        )
    }

    pub fn set_screen_anchor(&mut self, anchor: (i16, i16)) {
        self.records[1].payload[20..22].copy_from_slice(&anchor.0.to_le_bytes());
        self.records[1].payload[22..24].copy_from_slice(&anchor.1.to_le_bytes());
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.records[0].payload[2] = u8::from(hidden);
    }

    pub fn encode(&self) -> Vec<u8> {
        encode_records(&self.records)
    }
}

fn valid_zero_terminated_text(payload: &[u8], length_offset: usize) -> bool {
    if payload.len() < length_offset + 4 {
        return false;
    }
    let length = u16_at(payload, length_offset) as usize;
    let start = length_offset + 2;
    let Some(end) = start.checked_add(length) else {
        return false;
    };
    end < payload.len()
        && payload[end] == 0
        && payload[end + 1..].iter().all(|&byte| byte == 0)
        && payload.len() == (end + 2) & !1
}

fn set_zero_terminated_text(
    record: &mut Record,
    length_offset: usize,
    text: &str,
) -> Result<(), String> {
    let bytes = text.as_bytes();
    let length = u16::try_from(bytes.len()).map_err(|_| "Link URL text is too long")?;
    if bytes.contains(&0) {
        return Err("Link URL text cannot contain a NUL byte".into());
    }
    record.payload.truncate(length_offset);
    record.payload.extend_from_slice(&length.to_le_bytes());
    record.payload.extend_from_slice(bytes);
    record.payload.push(0);
    if record.payload.len() % 2 != 0 {
        record.payload.push(0);
    }
    record.padding = None;
    Ok(())
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn parse_records<const N: usize>(bytes: &[u8], kinds: [u32; N]) -> Result<[Record; N], String> {
    let mut offset = 0usize;
    let mut parsed = Vec::new();
    while offset < bytes.len() {
        if bytes.len() - offset < 8 {
            return Err("Action button record header is truncated".into());
        }
        let len = u32_at(bytes, offset) as usize;
        let kind = u32_at(bytes, offset + 4);
        let start = offset
            .checked_add(8)
            .ok_or("Action button record offset overflow")?;
        let end = start
            .checked_add(len)
            .ok_or("Action button record length overflow")?;
        let padded_end = end
            .checked_add(len % 2)
            .ok_or("Action button padding overflow")?;
        if padded_end > bytes.len() {
            return Err("Action button record payload is truncated".into());
        }
        parsed.push(Record {
            kind,
            payload: bytes[start..end].to_vec(),
            padding: (len % 2 != 0).then(|| bytes[end]),
        });
        offset = padded_end;
        if parsed.len() > N {
            return Err("Unexpected extra record in action button group".into());
        }
    }
    let records: [Record; N] = parsed
        .try_into()
        .map_err(|_| "Wrong number of action button records")?;
    if std::array::from_fn::<_, N, _>(|index| records[index].kind) != kinds {
        return Err("Unexpected action button record order".into());
    }
    Ok(records)
}

fn encode_records(records: &[Record]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for record in records {
        bytes.extend_from_slice(&(record.payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&record.kind.to_le_bytes());
        bytes.extend_from_slice(&record.payload);
        if record.payload.len() % 2 != 0 {
            bytes.push(record.padding.unwrap_or(0));
        }
    }
    bytes
}

impl HideShowButtonGroup {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let records = parse_records(
            bytes,
            [
                OBJECT_HEADER,
                OBJECT_REFS,
                ACTION_PAYLOAD,
                BUTTON_LABEL,
                BUTTON_AUX,
                OBJECT_END,
            ],
        )?;
        let group = Self { records };
        group.validate()?;
        Ok(group)
    }

    pub fn from_original_template(
        target: u32,
        label: &str,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        Self::from_original_template_targets(&[target], label, anchor)
    }

    pub fn from_original_template_targets(
        targets: &[u32],
        label: &str,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        let template: &[u8] = if label.len() % 2 == 0 {
            include_bytes!("../assets/action_hide_show_group.bin")
        } else {
            include_bytes!("../assets/action_hide_show_two_points_group.bin")
        };
        let mut group = Self::parse(template)?;
        group.set_targets(targets)?;
        group.set_label(label)?;
        group.set_screen_anchor(anchor);
        Ok(group)
    }

    fn validate(&self) -> Result<(), String> {
        let header = &self.records[0].payload;
        if header.len() != 28 || u16_at(header, 0) != 62 {
            return Err("Hide/Show button needs a 28-byte object code 62 header".into());
        }
        let refs = &self.records[1].payload;
        if refs.len() < 8
            || (refs.len() - 4) % 4 != 0
            || u32_at(refs, 0) as usize != (refs.len() - 4) / 4
        {
            return Err("Hide/Show button has invalid object references".into());
        }
        if self.target_ids().iter().any(|&id| id == 0) {
            return Err("Hide/Show button references object zero".into());
        }
        let action = &self.records[2].payload;
        if action.len() != 24
            || u32_at(action, 0) != ACTION_PAYLOAD
            || u32_at(action, 4) != 24
            || !matches!((u16_at(action, 12), u16_at(action, 14)), (0 | 1, 7))
        {
            return Err("Unsupported Hide/Show action payload".into());
        }
        let label = &self.records[3].payload;
        if label.len() < 24 || u16_at(label, 22) as usize > label.len() - 24 {
            return Err("Hide/Show button label is truncated".into());
        }
        if self.records[4].payload.len() != 2 || !self.records[5].payload.is_empty() {
            return Err("Hide/Show button has invalid auxiliary or terminator record".into());
        }
        Ok(())
    }

    pub fn target_ids(&self) -> Vec<u32> {
        self.records[1].payload[4..]
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect()
    }

    pub fn set_single_target(&mut self, target: u32) -> Result<(), String> {
        if self.target_ids().len() != 1 {
            return Err("A single-target button needs one nonzero target ID".into());
        }
        self.set_targets(&[target])?;
        Ok(())
    }

    pub fn set_targets(&mut self, targets: &[u32]) -> Result<(), String> {
        let count = u32::try_from(targets.len()).map_err(|_| "Too many button targets")?;
        if count == 0 || targets.contains(&0) {
            return Err("A Hide/Show button needs nonzero target IDs".into());
        }
        let payload = &mut self.records[1].payload;
        payload.clear();
        payload.extend_from_slice(&count.to_le_bytes());
        for target in targets {
            payload.extend_from_slice(&target.to_le_bytes());
        }
        Ok(())
    }

    pub fn action(&self) -> VisibilityAction {
        if u16_at(&self.records[2].payload, 12) == 0 {
            VisibilityAction::Hide
        } else {
            VisibilityAction::Show
        }
    }

    pub fn set_action(&mut self, action: VisibilityAction) {
        let code: u16 = match action {
            VisibilityAction::Hide => 0,
            VisibilityAction::Show => 1,
        };
        self.records[2].payload[12..14].copy_from_slice(&code.to_le_bytes());
    }

    pub fn set_link_destination(&mut self, linked: bool) {
        self.records[2].payload[18..20].copy_from_slice(&u16::from(linked).to_le_bytes());
    }

    pub fn screen_anchor(&self) -> (i16, i16) {
        let payload = &self.records[2].payload;
        (
            i16::from_le_bytes(payload[20..22].try_into().unwrap()),
            i16::from_le_bytes(payload[22..24].try_into().unwrap()),
        )
    }

    pub fn set_screen_anchor(&mut self, anchor: (i16, i16)) {
        self.records[2].payload[20..22].copy_from_slice(&anchor.0.to_le_bytes());
        self.records[2].payload[22..24].copy_from_slice(&anchor.1.to_le_bytes());
    }

    pub fn label_bytes(&self) -> &[u8] {
        let payload = &self.records[3].payload;
        let len = u16_at(payload, 22) as usize;
        &payload[24..24 + len]
    }

    pub fn label_utf8(&self) -> Result<&str, String> {
        std::str::from_utf8(self.label_bytes())
            .map_err(|_| "Action button label is not UTF-8".into())
    }

    pub fn set_label(&mut self, label: &str) -> Result<(), String> {
        let bytes = label.as_bytes();
        let len =
            u16::try_from(bytes.len()).map_err(|_| "Action button label exceeds 65535 bytes")?;
        let old_len = self.label_bytes().len();
        let old = &self.records[3].payload;
        let suffix = old[24 + old_len..].to_vec();
        if old_len % 2 != bytes.len() % 2 {
            return Err(
                "Action button label byte parity differs; opaque suffix cannot be moved safely"
                    .into(),
            );
        }
        let payload = &mut self.records[3].payload;
        payload.truncate(22);
        payload.extend_from_slice(&len.to_le_bytes());
        payload.extend_from_slice(bytes);
        payload.extend_from_slice(&suffix);
        self.records[3].padding = (payload.len() % 2 != 0).then_some(0);
        Ok(())
    }

    pub fn is_hidden(&self) -> bool {
        self.records[0].payload[2] != 0
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.records[0].payload[2] = u8::from(hidden);
    }

    pub fn encode(&self) -> Vec<u8> {
        encode_records(&self.records)
    }
}

impl PresentationButtonGroup {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let records = parse_records(
            bytes,
            [
                OBJECT_HEADER,
                OBJECT_REFS,
                ACTION_PAYLOAD,
                BUTTON_LABEL,
                BUTTON_AUX,
                OBJECT_END,
            ],
        )?;
        let group = Self { records };
        group.validate()?;
        Ok(group)
    }

    pub fn from_original_template_targets(
        targets: &[u32],
        label: &str,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        let mut group = Self::parse(include_bytes!("../assets/action_presentation_group.bin"))?;
        group.set_targets(targets)?;
        group.set_label(label)?;
        group.set_screen_anchor(anchor);
        Ok(group)
    }

    fn validate(&self) -> Result<(), String> {
        let header = &self.records[0].payload;
        let refs = &self.records[1].payload;
        let action = &self.records[2].payload;
        let label = &self.records[3].payload;
        if header.len() != 28 || u16_at(header, 0) != 62 {
            return Err("Presentation button needs an object code 62 header".into());
        }
        if refs.len() < 8
            || (refs.len() - 4) % 4 != 0
            || u32_at(refs, 0) as usize != (refs.len() - 4) / 4
            || self.target_ids().contains(&0)
        {
            return Err("Presentation button needs nonzero parent button IDs".into());
        }
        if action.len() != 24
            || u32_at(action, 0) != ACTION_PAYLOAD
            || u32_at(action, 4) != 24
            || u32_at(action, 8) != 12
            || u16_at(action, 12) != 7
            || !matches!((u16_at(action, 14), u32_at(action, 16)), (0, 600) | (1, 0))
        {
            return Err("Unsupported Presentation button action".into());
        }
        if label.len() < 24 || u16_at(label, 22) as usize > label.len() - 24 {
            return Err("Presentation button label is truncated".into());
        }
        if self.records[4].payload.len() != 2 || !self.records[5].payload.is_empty() {
            return Err("Presentation button auxiliary records are invalid".into());
        }
        Ok(())
    }

    pub fn target_ids(&self) -> Vec<u32> {
        self.records[1].payload[4..]
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect()
    }

    pub fn set_targets(&mut self, targets: &[u32]) -> Result<(), String> {
        let count = u32::try_from(targets.len()).map_err(|_| "Too many Presentation targets")?;
        if count == 0 || targets.contains(&0) {
            return Err("Presentation button needs nonzero targets".into());
        }
        let refs = &mut self.records[1].payload;
        refs.clear();
        refs.extend_from_slice(&count.to_le_bytes());
        for target in targets {
            refs.extend_from_slice(&target.to_le_bytes());
        }
        Ok(())
    }

    pub fn mode(&self) -> PresentationMode {
        if u16_at(&self.records[2].payload, 14) == 0 {
            PresentationMode::Simultaneous
        } else {
            PresentationMode::Sequential
        }
    }

    pub fn set_mode(&mut self, mode: PresentationMode) {
        let (code, delay): (u16, u32) = match mode {
            PresentationMode::Simultaneous => (0, 600),
            PresentationMode::Sequential => (1, 0),
        };
        self.records[2].payload[14..16].copy_from_slice(&code.to_le_bytes());
        self.records[2].payload[16..20].copy_from_slice(&delay.to_le_bytes());
    }

    pub fn screen_anchor(&self) -> (i16, i16) {
        let payload = &self.records[2].payload;
        (
            i16::from_le_bytes(payload[20..22].try_into().unwrap()),
            i16::from_le_bytes(payload[22..24].try_into().unwrap()),
        )
    }

    pub fn set_screen_anchor(&mut self, anchor: (i16, i16)) {
        self.records[2].payload[20..22].copy_from_slice(&anchor.0.to_le_bytes());
        self.records[2].payload[22..24].copy_from_slice(&anchor.1.to_le_bytes());
    }

    pub fn label_utf8(&self) -> Result<&str, String> {
        let payload = &self.records[3].payload;
        let len = u16_at(payload, 22) as usize;
        std::str::from_utf8(&payload[24..24 + len])
            .map_err(|_| "Presentation button label is not UTF-8".into())
    }

    pub fn set_label(&mut self, label: &str) -> Result<(), String> {
        let bytes = label.as_bytes();
        let len =
            u16::try_from(bytes.len()).map_err(|_| "Presentation button label is too long")?;
        let payload = &mut self.records[3].payload;
        let old_len = u16_at(payload, 22) as usize;
        if old_len % 2 != bytes.len() % 2 {
            return Err("Presentation button label byte parity differs".into());
        }
        let suffix = payload[24 + old_len..].to_vec();
        payload.truncate(22);
        payload.extend_from_slice(&len.to_le_bytes());
        payload.extend_from_slice(bytes);
        payload.extend_from_slice(&suffix);
        self.records[3].padding = (payload.len() % 2 != 0).then_some(0);
        Ok(())
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.records[0].payload[2] = u8::from(hidden);
    }

    pub fn encode(&self) -> Vec<u8> {
        encode_records(&self.records)
    }
}

impl ScrollButtonGroup {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let records = parse_records(
            bytes,
            [
                OBJECT_HEADER,
                OBJECT_REFS,
                ACTION_PAYLOAD,
                BUTTON_LABEL,
                BUTTON_AUX,
                OBJECT_END,
            ],
        )?;
        let group = Self { records };
        group.validate()?;
        Ok(group)
    }

    pub fn from_original_template(
        target: u32,
        label: &str,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        let mut group = Self::parse(include_bytes!("../assets/action_scroll_group.bin"))?;
        group.set_target(target)?;
        group.set_label(label)?;
        group.set_screen_anchor(anchor);
        Ok(group)
    }

    fn validate(&self) -> Result<(), String> {
        let header = &self.records[0].payload;
        let refs = &self.records[1].payload;
        let action = &self.records[2].payload;
        let label = &self.records[3].payload;
        if header.len() != 28 || u16_at(header, 0) != 62 {
            return Err("Scroll button needs an object code 62 header".into());
        }
        if refs.len() != 8 || u32_at(refs, 0) != 1 || u32_at(refs, 4) == 0 {
            return Err("Scroll button needs one point target".into());
        }
        if action.len() != 24
            || u32_at(action, 0) != ACTION_PAYLOAD
            || u32_at(action, 4) != 24
            || u32_at(action, 8) != 12
            || u16_at(action, 12) != 4
            || u16_at(action, 14) > 1
            || u32_at(action, 16) != 0
        {
            return Err("Unsupported Scroll button action".into());
        }
        if label.len() < 24 || u16_at(label, 22) as usize > label.len() - 24 {
            return Err("Scroll button label is truncated".into());
        }
        if self.records[4].payload.len() != 2 || !self.records[5].payload.is_empty() {
            return Err("Scroll button auxiliary records are invalid".into());
        }
        Ok(())
    }

    pub fn target_id(&self) -> u32 {
        u32_at(&self.records[1].payload, 4)
    }

    pub fn set_target(&mut self, target: u32) -> Result<(), String> {
        if target == 0 {
            return Err("Scroll target must be nonzero".into());
        }
        self.records[1].payload[4..8].copy_from_slice(&target.to_le_bytes());
        Ok(())
    }

    pub fn placement(&self) -> ScrollPlacement {
        if u16_at(&self.records[2].payload, 14) == 0 {
            ScrollPlacement::UpperLeft
        } else {
            ScrollPlacement::Center
        }
    }

    pub fn set_placement(&mut self, placement: ScrollPlacement) {
        let code: u16 = match placement {
            ScrollPlacement::UpperLeft => 0,
            ScrollPlacement::Center => 1,
        };
        self.records[2].payload[14..16].copy_from_slice(&code.to_le_bytes());
    }

    pub fn screen_anchor(&self) -> (i16, i16) {
        let payload = &self.records[2].payload;
        (
            i16::from_le_bytes(payload[20..22].try_into().unwrap()),
            i16::from_le_bytes(payload[22..24].try_into().unwrap()),
        )
    }

    pub fn set_screen_anchor(&mut self, anchor: (i16, i16)) {
        self.records[2].payload[20..22].copy_from_slice(&anchor.0.to_le_bytes());
        self.records[2].payload[22..24].copy_from_slice(&anchor.1.to_le_bytes());
    }

    pub fn label_utf8(&self) -> Result<&str, String> {
        let payload = &self.records[3].payload;
        let len = u16_at(payload, 22) as usize;
        std::str::from_utf8(&payload[24..24 + len])
            .map_err(|_| "Scroll button label is not UTF-8".into())
    }

    pub fn set_label(&mut self, label: &str) -> Result<(), String> {
        let bytes = label.as_bytes();
        let len = u16::try_from(bytes.len()).map_err(|_| "Scroll button label is too long")?;
        let payload = &mut self.records[3].payload;
        let old_len = u16_at(payload, 22) as usize;
        if old_len % 2 != bytes.len() % 2 {
            return Err("Scroll button label byte parity differs".into());
        }
        let suffix = payload[24 + old_len..].to_vec();
        payload.truncate(22);
        payload.extend_from_slice(&len.to_le_bytes());
        payload.extend_from_slice(bytes);
        payload.extend_from_slice(&suffix);
        self.records[3].padding = (payload.len() % 2 != 0).then_some(0);
        Ok(())
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.records[0].payload[2] = u8::from(hidden);
    }

    pub fn encode(&self) -> Vec<u8> {
        encode_records(&self.records)
    }
}

impl MovementButtonGroup {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let records = parse_records(
            bytes,
            [
                OBJECT_HEADER,
                OBJECT_REFS,
                ACTION_PAYLOAD,
                BUTTON_LABEL,
                BUTTON_AUX,
                OBJECT_END,
            ],
        )?;
        let group = Self { records };
        group.validate()?;
        Ok(group)
    }

    pub fn from_original_template(
        source: u32,
        destination: u32,
        label: &str,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        let mut group = Self::parse(include_bytes!("../assets/action_movement_group.bin"))?;
        group.set_targets(source, destination)?;
        group.set_label(label)?;
        group.set_screen_anchor(anchor);
        Ok(group)
    }

    fn validate(&self) -> Result<(), String> {
        let header = &self.records[0].payload;
        let refs = &self.records[1].payload;
        let action = &self.records[2].payload;
        let label = &self.records[3].payload;
        if header.len() != 28 || u16_at(header, 0) != 62 {
            return Err("Movement button needs an object code 62 header".into());
        }
        if refs.len() != 12 || u32_at(refs, 0) != 2 || u32_at(refs, 4) == 0 || u32_at(refs, 8) == 0
        {
            return Err("Movement button needs a source and destination".into());
        }
        if action.len() != 24
            || u32_at(action, 0) != ACTION_PAYLOAD
            || u32_at(action, 4) != 24
            || u32_at(action, 8) != 12
            || u16_at(action, 12) != 3
            || u16_at(action, 14) > 3
            || u16_at(action, 16) > 1
            || u16_at(action, 18) != 0
        {
            return Err("Unsupported Movement button action".into());
        }
        if label.len() < 24 || u16_at(label, 22) as usize > label.len() - 24 {
            return Err("Movement button label is truncated".into());
        }
        if self.records[4].payload.len() != 2 || !self.records[5].payload.is_empty() {
            return Err("Movement button auxiliary records are invalid".into());
        }
        Ok(())
    }

    pub fn target_ids(&self) -> [u32; 2] {
        [
            u32_at(&self.records[1].payload, 4),
            u32_at(&self.records[1].payload, 8),
        ]
    }

    pub fn set_targets(&mut self, source: u32, destination: u32) -> Result<(), String> {
        if source == 0 || destination == 0 {
            return Err("Movement targets must be nonzero".into());
        }
        self.records[1].payload[4..8].copy_from_slice(&source.to_le_bytes());
        self.records[1].payload[8..12].copy_from_slice(&destination.to_le_bytes());
        Ok(())
    }

    pub fn speed(&self) -> MovementSpeed {
        match u16_at(&self.records[2].payload, 14) {
            0 => MovementSpeed::Slow,
            1 => MovementSpeed::Medium,
            2 => MovementSpeed::Fast,
            _ => MovementSpeed::Instant,
        }
    }

    pub fn set_speed(&mut self, speed: MovementSpeed) {
        self.records[2].payload[14..16].copy_from_slice(&speed.code().to_le_bytes());
    }

    pub fn follows_destination(&self) -> bool {
        u16_at(&self.records[2].payload, 16) == 0
    }

    pub fn set_follows_destination(&mut self, follow: bool) {
        let code: u16 = if follow { 0 } else { 1 };
        self.records[2].payload[16..18].copy_from_slice(&code.to_le_bytes());
    }

    pub fn screen_anchor(&self) -> (i16, i16) {
        let payload = &self.records[2].payload;
        (
            i16::from_le_bytes(payload[20..22].try_into().unwrap()),
            i16::from_le_bytes(payload[22..24].try_into().unwrap()),
        )
    }

    pub fn set_screen_anchor(&mut self, anchor: (i16, i16)) {
        self.records[2].payload[20..22].copy_from_slice(&anchor.0.to_le_bytes());
        self.records[2].payload[22..24].copy_from_slice(&anchor.1.to_le_bytes());
    }

    pub fn label_utf8(&self) -> Result<&str, String> {
        let payload = &self.records[3].payload;
        let len = u16_at(payload, 22) as usize;
        std::str::from_utf8(&payload[24..24 + len])
            .map_err(|_| "Movement button label is not UTF-8".into())
    }

    pub fn set_label(&mut self, label: &str) -> Result<(), String> {
        let bytes = label.as_bytes();
        let len = u16::try_from(bytes.len()).map_err(|_| "Movement button label is too long")?;
        let payload = &mut self.records[3].payload;
        let old_len = u16_at(payload, 22) as usize;
        if old_len % 2 != bytes.len() % 2 {
            return Err("Movement button label byte parity differs".into());
        }
        let suffix = payload[24 + old_len..].to_vec();
        payload.truncate(22);
        payload.extend_from_slice(&len.to_le_bytes());
        payload.extend_from_slice(bytes);
        payload.extend_from_slice(&suffix);
        self.records[3].padding = (payload.len() % 2 != 0).then_some(0);
        Ok(())
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.records[0].payload[2] = u8::from(hidden);
    }

    pub fn encode(&self) -> Vec<u8> {
        encode_records(&self.records)
    }
}

impl AnimationButtonGroup {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let records = parse_records(
            bytes,
            [
                OBJECT_HEADER,
                OBJECT_REFS,
                ANIMATION_SETTINGS,
                ACTION_PAYLOAD,
                BUTTON_LABEL,
                BUTTON_AUX,
                OBJECT_END,
            ],
        )?;
        let group = Self { records };
        group.validate()?;
        Ok(group)
    }

    pub fn from_original_template(
        target: u32,
        label: &str,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        Self::from_original_template_targets(&[target], label, anchor)
    }

    pub fn from_original_template_targets(
        targets: &[u32],
        label: &str,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        let mut group = Self::parse(include_bytes!("../assets/action_animation_group.bin"))?;
        group.set_targets(targets)?;
        group.set_label(label)?;
        group.set_screen_anchor(anchor);
        Ok(group)
    }

    fn validate(&self) -> Result<(), String> {
        let header = &self.records[0].payload;
        if header.len() != 28 || u16_at(header, 0) != 62 {
            return Err("Animation button needs a 28-byte object code 62 header".into());
        }
        let refs = &self.records[1].payload;
        if refs.len() < 8
            || (refs.len() - 4) % 4 != 0
            || u32_at(refs, 0) as usize != (refs.len() - 4) / 4
            || self.target_ids().contains(&0)
        {
            return Err("Animation button needs nonzero targets".into());
        }
        let settings = &self.records[2].payload;
        // The settings payload starts after the 36-byte header group,
        // 16-byte target record, and its own 8-byte record header.
        let default_settings = &include_bytes!("../assets/action_animation_group.bin")[60..96];
        if settings.len() != 4 + 32 * self.target_ids().len()
            || settings[..4] != default_settings[..4]
            || settings[4..].chunks_exact(32).any(|block| {
                block.iter().zip(&default_settings[4..]).enumerate().any(
                    |(index, (actual, expected))| {
                        !matches!(index, 3 | 4..=19) && actual != expected
                    },
                )
            })
            || self.settings_list().is_err()
        {
            return Err("Unsupported Animation button settings".into());
        }
        let action = &self.records[3].payload;
        if action.len() != 24
            || u32_at(action, 0) != ACTION_PAYLOAD
            || u32_at(action, 4) != 24
            || u32_at(action, 8) != 12
            || u32_at(action, 12) != 2
        {
            return Err("Unsupported Animation button action".into());
        }
        let label = &self.records[4].payload;
        if label.len() < 24 || u16_at(label, 22) as usize > label.len() - 24 {
            return Err("Animation button label is truncated".into());
        }
        if self.records[5].payload.len() != 2 || !self.records[6].payload.is_empty() {
            return Err("Animation button has invalid auxiliary or terminator record".into());
        }
        Ok(())
    }

    pub fn target_id(&self) -> u32 {
        u32_at(&self.records[1].payload, 4)
    }

    pub fn target_ids(&self) -> Vec<u32> {
        self.records[1].payload[4..]
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect()
    }

    pub fn settings(&self) -> Result<AnimationSettings, String> {
        if self.target_ids().len() != 1 {
            return Err("Animation button has multiple target settings".into());
        }
        self.settings_list()?
            .into_iter()
            .next()
            .ok_or("Animation button has no target".into())
    }

    pub fn settings_list(&self) -> Result<Vec<AnimationSettings>, String> {
        let bytes = &self.records[2].payload;
        if bytes.len() < 36 || (bytes.len() - 4) % 32 != 0 {
            return Err("Animation settings are truncated".into());
        }
        bytes[4..]
            .chunks_exact(32)
            .map(|block| {
                let flags = block[3];
                let point_settings =
                    &include_bytes!("../assets/action_animation_group.bin")[72..84];
                let parameter = if block[8..20] == *point_settings {
                    None
                } else {
                    let number = |offset| {
                        decode_parameter_number(f32::from_le_bytes(
                            block[offset..offset + 4].try_into().unwrap(),
                        ))
                        .ok_or("Unsupported parameter Animation value".to_string())
                    };
                    Some(ParameterAnimationSettings {
                        discrete: flags & 0x08 != 0,
                        step: number(8)?,
                        min: number(12)?,
                        max: number(16)?,
                    })
                };
                if flags & !(if parameter.is_some() { 0xb8 } else { 0xb0 }) != 0 {
                    return Err("Unsupported animation direction flags".into());
                }
                let direction = match flags & 0x30 {
                    0x00 => AnimationDirection::Forward,
                    0x10 => AnimationDirection::Backward,
                    0x20 => AnimationDirection::Bidirectional,
                    0x30 => AnimationDirection::Random,
                    _ => unreachable!(),
                };
                let settings = AnimationSettings {
                    direction,
                    speed: if parameter.is_some() {
                        decode_parameter_number(f32::from_le_bytes(block[4..8].try_into().unwrap()))
                            .ok_or("Unsupported parameter Animation rate")?
                    } else {
                        f32::from_le_bytes(block[4..8].try_into().unwrap())
                    },
                    once: flags & 0x80 != 0,
                    parameter,
                };
                settings.validate()?;
                Ok(settings)
            })
            .collect()
    }

    pub fn set_settings(&mut self, settings: AnimationSettings) -> Result<(), String> {
        if self.target_ids().len() != 1 {
            return Err("Animation button has multiple target settings".into());
        }
        self.set_settings_at(0, settings)
    }

    pub fn set_settings_list(&mut self, settings: &[AnimationSettings]) -> Result<(), String> {
        if settings.len() != self.target_ids().len() {
            return Err("Animation settings count differs from targets".into());
        }
        for (index, &value) in settings.iter().enumerate() {
            self.set_settings_at(index, value)?;
        }
        Ok(())
    }

    pub fn set_settings_at(
        &mut self,
        index: usize,
        settings: AnimationSettings,
    ) -> Result<(), String> {
        settings.validate()?;
        if index >= self.target_ids().len() {
            return Err("Animation target index is out of range".into());
        }
        let direction = match settings.direction {
            AnimationDirection::Forward => 0x00,
            AnimationDirection::Backward => 0x10,
            AnimationDirection::Bidirectional => 0x20,
            AnimationDirection::Random => 0x30,
        };
        let bytes = &mut self.records[2].payload;
        let start = 4 + 32 * index;
        bytes[start + 3] = direction
            | if settings.once { 0x80 } else { 0 }
            | if settings.parameter.is_some_and(|value| value.discrete) {
                0x08
            } else {
                0
            };
        let stored_speed = if settings.parameter.is_some() {
            encode_parameter_number(settings.speed).ok_or("Invalid parameter Animation rate")?
        } else {
            settings.speed
        };
        bytes[start + 4..start + 8].copy_from_slice(&stored_speed.to_le_bytes());
        if let Some(parameter) = settings.parameter {
            for (offset, value) in [
                (8, parameter.step),
                (12, parameter.min),
                (16, parameter.max),
            ] {
                let encoded =
                    encode_parameter_number(value).ok_or("Invalid parameter Animation value")?;
                bytes[start + offset..start + offset + 4].copy_from_slice(&encoded.to_le_bytes());
            }
        } else {
            let template = &include_bytes!("../assets/action_animation_group.bin")[72..84];
            bytes[start + 8..start + 20].copy_from_slice(template);
        }
        Ok(())
    }

    pub fn set_single_target(&mut self, target: u32) -> Result<(), String> {
        if self.target_ids().len() != 1 {
            return Err("A single-target Animation button needs one target".into());
        }
        self.set_targets(&[target])?;
        Ok(())
    }

    pub fn set_targets(&mut self, targets: &[u32]) -> Result<(), String> {
        let count = u32::try_from(targets.len()).map_err(|_| "Too many Animation targets")?;
        if count == 0 || targets.contains(&0) {
            return Err("Animation button needs nonzero targets".into());
        }
        let old_count = self.target_ids().len();
        let refs = &mut self.records[1].payload;
        refs.clear();
        refs.extend_from_slice(&count.to_le_bytes());
        for target in targets {
            refs.extend_from_slice(&target.to_le_bytes());
        }
        if targets.len() != old_count {
            let default = &include_bytes!("../assets/action_animation_group.bin")[60..96];
            let settings = &mut self.records[2].payload;
            settings.truncate(4);
            for _ in targets {
                settings.extend_from_slice(&default[4..36]);
            }
        }
        Ok(())
    }

    pub fn screen_anchor(&self) -> (i16, i16) {
        let payload = &self.records[3].payload;
        (
            i16::from_le_bytes(payload[20..22].try_into().unwrap()),
            i16::from_le_bytes(payload[22..24].try_into().unwrap()),
        )
    }

    pub fn set_screen_anchor(&mut self, anchor: (i16, i16)) {
        self.records[3].payload[20..22].copy_from_slice(&anchor.0.to_le_bytes());
        self.records[3].payload[22..24].copy_from_slice(&anchor.1.to_le_bytes());
    }

    pub fn label_utf8(&self) -> Result<&str, String> {
        let payload = &self.records[4].payload;
        let len = u16_at(payload, 22) as usize;
        std::str::from_utf8(&payload[24..24 + len])
            .map_err(|_| "Animation button label is not UTF-8".into())
    }

    pub fn set_label(&mut self, label: &str) -> Result<(), String> {
        let bytes = label.as_bytes();
        let len = u16::try_from(bytes.len()).map_err(|_| "Animation button label is too long")?;
        let payload = &mut self.records[4].payload;
        let old_len = u16_at(payload, 22) as usize;
        let mut suffix = payload[24 + old_len..].to_vec();
        if old_len % 2 != bytes.len() % 2 {
            if suffix.len() != if old_len % 2 == 0 { 2 } else { 1 }
                || suffix.iter().any(|byte| *byte != 0)
            {
                return Err("Animation button label byte parity differs".into());
            }
            suffix = vec![0; if bytes.len() % 2 == 0 { 2 } else { 1 }];
        }
        payload.truncate(22);
        payload.extend_from_slice(&len.to_le_bytes());
        payload.extend_from_slice(bytes);
        payload.extend_from_slice(&suffix);
        self.records[4].padding = (payload.len() % 2 != 0).then_some(0);
        Ok(())
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.records[0].payload[2] = u8::from(hidden);
    }

    pub fn encode(&self) -> Vec<u8> {
        encode_records(&self.records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_presentation_modes_round_trip() {
        for (file, expected, targets) in [
            (
                include_bytes!("../tests/fixtures/action_presentation_default_original.gsp")
                    .as_slice(),
                PresentationMode::Simultaneous,
                vec![3, 4],
            ),
            (
                include_bytes!(
                    "../tests/fixtures/action_presentation_sequential_scroll_original.gsp"
                )
                .as_slice(),
                PresentationMode::Sequential,
                vec![4, 5],
            ),
        ] {
            let bytes = presentation_group(file);
            let group = PresentationButtonGroup::parse(&bytes).unwrap();
            assert_eq!(group.target_ids(), targets);
            assert_eq!(group.mode(), expected);
            assert_eq!(group.encode(), bytes);
        }
        let mut generated = PresentationButtonGroup::from_original_template_targets(
            &[3, 4],
            "Run Presentation",
            (10, 65),
        )
        .unwrap();
        generated.set_mode(PresentationMode::Sequential);
        let reopened = PresentationButtonGroup::parse(&generated.encode()).unwrap();
        assert_eq!(reopened.target_ids(), vec![3, 4]);
        assert_eq!(reopened.mode(), PresentationMode::Sequential);
        assert_eq!(reopened.label_utf8().unwrap(), "Run Presentation");
    }

    #[test]
    fn native_link_url_lengths_round_trip() {
        for (file, url) in [
            (
                include_bytes!("../tests/fixtures/action_link_url_default_original.gsp").as_slice(),
                "http://kejianyuan.com",
            ),
            (
                include_bytes!("../tests/fixtures/action_link_url_custom_original.gsp").as_slice(),
                "https://example.invalid/sketchpad",
            ),
            (
                include_bytes!("../tests/fixtures/action_link_url_even_original.gsp").as_slice(),
                "https://example.org/",
            ),
        ] {
            let bytes = link_url_group(file);
            let group = LinkUrlButtonGroup::parse(&bytes).unwrap();
            assert_eq!(group.url_utf8().unwrap(), url);
            assert_eq!(group.label_utf8().unwrap(), url);
            assert_eq!(group.screen_anchor(), (10, 5));
            assert_eq!(group.encode(), bytes);
        }
        let generated = LinkUrlButtonGroup::from_original_template(
            "https://example.org/",
            "Open page",
            (10, 35),
        )
        .unwrap();
        let reopened = LinkUrlButtonGroup::parse(&generated.encode()).unwrap();
        assert_eq!(reopened.url_utf8().unwrap(), "https://example.org/");
        assert_eq!(reopened.label_utf8().unwrap(), "Open page");
        assert_eq!(reopened.screen_anchor(), (10, 35));
    }

    #[test]
    fn native_page_link_destinations_round_trip() {
        for (file, destination) in [
            (
                include_bytes!("../tests/fixtures/action_link_page_default_original.gsp")
                    .as_slice(),
                0,
            ),
            (
                include_bytes!("../tests/fixtures/action_link_page_two_original.gsp").as_slice(),
                1,
            ),
        ] {
            let bytes = link_page_group(file);
            let group = LinkPageButtonGroup::parse(&bytes).unwrap();
            assert_eq!(group.target_page(), destination);
            assert_eq!(group.label_utf8().unwrap(), "链接");
            assert_eq!(group.screen_anchor(), (10, 5));
            assert_eq!(group.encode(), bytes);
        }
    }

    #[test]
    fn native_page_link_can_select_a_button_on_destination_page() {
        let native = include_bytes!(
            "../tests/fixtures/action_link_page_activate_hide_show_native_created.gsp"
        );
        let group = LinkPageButtonGroup::parse(&link_page_group(native)).unwrap();
        assert_eq!(group.target_page(), 0);
        assert_eq!(group.target_button_ordinal(), Some(1));
        assert_eq!(u16_at(&group.records[3].payload, 0), 1);
        let mut generated =
            LinkPageButtonGroup::from_original_template(0, "Go to Page 1", (10, 5)).unwrap();
        generated.set_target_button_ordinal(Some(1)).unwrap();
        assert_eq!(
            LinkPageButtonGroup::parse(&generated.encode())
                .unwrap()
                .target_button_ordinal(),
            Some(1)
        );
    }

    #[test]
    fn native_scroll_button_modes_round_trip() {
        for (file, expected) in [
            (
                include_bytes!("../tests/fixtures/action_scroll_top_left_original.gsp").as_slice(),
                ScrollPlacement::UpperLeft,
            ),
            (
                include_bytes!("../tests/fixtures/action_scroll_center_original.gsp").as_slice(),
                ScrollPlacement::Center,
            ),
        ] {
            let bytes = action_group(file);
            let group = ScrollButtonGroup::parse(&bytes).unwrap();
            assert_eq!(group.target_id(), 1);
            assert_eq!(group.placement(), expected);
            assert_eq!(group.label_utf8().unwrap(), "滚动");
            assert_eq!(group.screen_anchor(), (10, 5));
            assert_eq!(group.encode(), bytes);
        }
        let mut generated =
            ScrollButtonGroup::from_original_template(2, "Scroll", (10, 35)).unwrap();
        generated.set_placement(ScrollPlacement::Center);
        let reopened = ScrollButtonGroup::parse(&generated.encode()).unwrap();
        assert_eq!(reopened.target_id(), 2);
        assert_eq!(reopened.placement(), ScrollPlacement::Center);
        assert_eq!(reopened.label_utf8().unwrap(), "Scroll");
        assert_eq!(reopened.screen_anchor(), (10, 35));
    }

    #[test]
    fn native_movement_settings_round_trip() {
        for (file, speed, follow) in [
            (
                include_bytes!("../tests/fixtures/action_movement_default_original.gsp").as_slice(),
                MovementSpeed::Medium,
                true,
            ),
            (
                include_bytes!("../tests/fixtures/action_movement_slow_original.gsp").as_slice(),
                MovementSpeed::Slow,
                true,
            ),
            (
                include_bytes!("../tests/fixtures/action_movement_fast_original.gsp").as_slice(),
                MovementSpeed::Fast,
                true,
            ),
            (
                include_bytes!("../tests/fixtures/action_movement_instant_original.gsp").as_slice(),
                MovementSpeed::Instant,
                true,
            ),
            (
                include_bytes!("../tests/fixtures/action_movement_initial_original.gsp").as_slice(),
                MovementSpeed::Medium,
                false,
            ),
        ] {
            let bytes = action_group(file);
            let group = MovementButtonGroup::parse(&bytes).unwrap();
            assert_eq!(group.target_ids(), [1, 2]);
            assert_eq!(group.speed(), speed);
            assert_eq!(group.follows_destination(), follow);
            assert_eq!(group.label_utf8().unwrap(), "移动点");
            assert_eq!(group.screen_anchor(), (10, 5));
            assert_eq!(group.encode(), bytes);
        }
        let mut generated =
            MovementButtonGroup::from_original_template(3, 4, "Move to Point", (20, 35)).unwrap();
        generated.set_speed(MovementSpeed::Instant);
        generated.set_follows_destination(false);
        let reopened = MovementButtonGroup::parse(&generated.encode()).unwrap();
        assert_eq!(reopened.target_ids(), [3, 4]);
        assert_eq!(reopened.speed(), MovementSpeed::Instant);
        assert!(!reopened.follows_destination());
        assert_eq!(reopened.screen_anchor(), (20, 35));
    }

    #[test]
    fn native_animation_button_round_trips_and_template_remaps_target() {
        let native = include_bytes!("../tests/fixtures/action_animation_original.gsp");
        let bytes = action_group(native);
        let group = AnimationButtonGroup::parse(&bytes).unwrap();
        assert_eq!(group.target_id(), 4);
        assert_eq!(group.screen_anchor(), (10, 5));
        assert_eq!(group.label_utf8().unwrap(), "动画点");
        assert_eq!(group.encode(), bytes);
        let generated =
            AnimationButtonGroup::from_original_template(7, "Animate Point", (30, 40)).unwrap();
        let reopened = AnimationButtonGroup::parse(&generated.encode()).unwrap();
        assert_eq!(reopened.target_id(), 7);
        assert_eq!(reopened.screen_anchor(), (30, 40));
        assert_eq!(reopened.label_utf8().unwrap(), "Animate Point");
        assert_eq!(reopened.settings().unwrap(), AnimationSettings::default());
    }

    #[test]
    fn two_point_animation_settings_are_independent() {
        let default = include_bytes!("../tests/fixtures/action_animation_two_points_original.gsp");
        let asymmetric =
            include_bytes!("../tests/fixtures/action_animation_two_points_second_backward.gsp");
        let clicked = include_bytes!("../tests/fixtures/action_animation_two_points_clicked.gsp");
        for file in [
            default.as_slice(),
            asymmetric.as_slice(),
            clicked.as_slice(),
        ] {
            let bytes = action_group(file);
            let group = AnimationButtonGroup::parse(&bytes).unwrap();
            assert_eq!(group.target_ids(), vec![4, 5]);
            assert_eq!(group.encode(), bytes);
        }
        let second = AnimationButtonGroup::parse(&action_group(asymmetric)).unwrap();
        assert!(second.settings().is_err());
        assert_eq!(
            second.settings_list().unwrap(),
            vec![
                AnimationSettings::default(),
                AnimationSettings {
                    direction: AnimationDirection::Backward,
                    ..AnimationSettings::default()
                },
            ]
        );
        let mut generated =
            AnimationButtonGroup::from_original_template_targets(&[4, 5], "Animate Point", (10, 5))
                .unwrap();
        generated
            .set_settings_at(
                1,
                AnimationSettings {
                    direction: AnimationDirection::Backward,
                    ..AnimationSettings::default()
                },
            )
            .unwrap();
        assert_eq!(
            AnimationButtonGroup::parse(&generated.encode())
                .unwrap()
                .settings_list()
                .unwrap(),
            second.settings_list().unwrap()
        );
    }

    #[test]
    fn native_animation_direction_speed_and_once_settings_round_trip() {
        use AnimationDirection::*;
        for (file, expected) in [
            (
                include_bytes!("../tests/fixtures/action_animation_forward_original.gsp")
                    .as_slice(),
                AnimationSettings {
                    direction: Forward,
                    speed: 24.,
                    once: false,
                    ..AnimationSettings::default()
                },
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_backward_original.gsp")
                    .as_slice(),
                AnimationSettings {
                    direction: Backward,
                    speed: 24.,
                    once: false,
                    ..AnimationSettings::default()
                },
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_random_original.gsp").as_slice(),
                AnimationSettings {
                    direction: Random,
                    speed: 24.,
                    once: false,
                    ..AnimationSettings::default()
                },
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_slow_original.gsp").as_slice(),
                AnimationSettings {
                    direction: Bidirectional,
                    speed: 4.,
                    once: false,
                    ..AnimationSettings::default()
                },
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_fast_original.gsp").as_slice(),
                AnimationSettings {
                    direction: Bidirectional,
                    speed: 80.,
                    once: false,
                    ..AnimationSettings::default()
                },
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_forward_once_original.gsp")
                    .as_slice(),
                AnimationSettings {
                    direction: Forward,
                    speed: 24.,
                    once: true,
                    ..AnimationSettings::default()
                },
            ),
        ] {
            let bytes = action_group(file);
            let group = AnimationButtonGroup::parse(&bytes).unwrap();
            assert_eq!(group.settings().unwrap(), expected);
            assert_eq!(group.encode(), bytes);
            let mut generated =
                AnimationButtonGroup::from_original_template(4, "Animate Point", (10, 5)).unwrap();
            generated.set_settings(expected).unwrap();
            assert_eq!(
                AnimationButtonGroup::parse(&generated.encode())
                    .unwrap()
                    .settings()
                    .unwrap(),
                expected
            );
        }
    }

    #[test]
    fn native_parameter_animation_numbers_and_discrete_mode_round_trip() {
        for (value, encoded) in [
            (0.5, 1.),
            (1., 4.),
            (2., 16.),
            (2.5, 20.),
            (-3., -24.),
            (7., 112.),
            (100., 25_600.),
        ] {
            assert_eq!(encode_parameter_number(value), Some(encoded));
            assert_eq!(decode_parameter_number(encoded), Some(value));
        }
        for (bytes, expected) in [
            (
                include_bytes!("../tests/fixtures/action_animation_parameter_native_rate2.gsp")
                    .as_slice(),
                AnimationSettings {
                    direction: AnimationDirection::Bidirectional,
                    speed: 2.,
                    once: false,
                    parameter: Some(ParameterAnimationSettings::default()),
                },
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_parameter_native_discrete.gsp")
                    .as_slice(),
                AnimationSettings {
                    direction: AnimationDirection::Bidirectional,
                    speed: 2.,
                    once: false,
                    parameter: Some(ParameterAnimationSettings {
                        discrete: true,
                        step: 0.5,
                        min: -3.,
                        max: 7.,
                    }),
                },
            ),
            (
                include_bytes!(
                    "../tests/fixtures/action_animation_parameter_custom_continuous.gsp"
                )
                .as_slice(),
                AnimationSettings {
                    direction: AnimationDirection::Bidirectional,
                    speed: 2.5,
                    once: false,
                    parameter: Some(ParameterAnimationSettings {
                        discrete: false,
                        step: 1.,
                        min: -3.,
                        max: 7.,
                    }),
                },
            ),
        ] {
            let group = AnimationButtonGroup::parse(&action_group(bytes)).unwrap();
            assert_eq!(group.settings().unwrap(), expected);
        }
    }

    fn action_group(file: &[u8]) -> Vec<u8> {
        let mut offset = 4;
        let mut start = None;
        while offset < file.len() {
            let len = u32_at(file, offset) as usize;
            let kind = u32_at(file, offset + 4);
            if kind == OBJECT_HEADER && u16_at(file, offset + 8) == 62 {
                start = Some(offset);
            }
            offset += 8 + len + len % 2;
            if start.is_some() && kind == OBJECT_END {
                return file[start.unwrap()..offset].to_vec();
            }
        }
        panic!("action group not found");
    }

    fn presentation_group(file: &[u8]) -> Vec<u8> {
        let mut offset = 4;
        let mut start = None;
        let mut is_presentation = false;
        while offset < file.len() {
            let len = u32_at(file, offset) as usize;
            let kind = u32_at(file, offset + 4);
            if kind == OBJECT_HEADER && u16_at(file, offset + 8) == 62 {
                start = Some(offset);
                is_presentation = false;
            }
            if start.is_some()
                && kind == ACTION_PAYLOAD
                && len == 24
                && u16_at(file, offset + 8 + 12) == 7
            {
                is_presentation = true;
            }
            offset += 8 + len + len % 2;
            if kind == OBJECT_END && start.is_some() {
                if is_presentation {
                    return file[start.unwrap()..offset].to_vec();
                }
                start = None;
            }
        }
        panic!("Presentation group not found");
    }

    fn link_url_group(file: &[u8]) -> Vec<u8> {
        let mut offset = 4;
        let mut start = None;
        let mut is_link = false;
        while offset < file.len() {
            let len = u32_at(file, offset) as usize;
            let kind = u32_at(file, offset + 4);
            if kind == OBJECT_HEADER && u16_at(file, offset + 8) == 0 {
                start = Some(offset);
                is_link = false;
            }
            if start.is_some()
                && kind == ACTION_PAYLOAD
                && len >= 28
                && u16_at(file, offset + 20) == 6
            {
                is_link = true;
            }
            offset += 8 + len + len % 2;
            if kind == OBJECT_END && start.is_some() {
                if is_link {
                    return file[start.unwrap()..offset].to_vec();
                }
                start = None;
            }
        }
        panic!("Link URL group not found");
    }

    fn link_page_group(file: &[u8]) -> Vec<u8> {
        let mut offset = 4;
        let mut start = None;
        let mut is_link = false;
        while offset < file.len() {
            let len = u32_at(file, offset) as usize;
            let kind = u32_at(file, offset + 4);
            if kind == OBJECT_HEADER && u16_at(file, offset + 8) == 0 {
                start = Some(offset);
                is_link = false;
            }
            if start.is_some()
                && kind == ACTION_PAYLOAD
                && len == 24
                && u16_at(file, offset + 20) == 5
            {
                is_link = true;
            }
            offset += 8 + len + len % 2;
            if kind == OBJECT_END && start.is_some() {
                if is_link {
                    return file[start.unwrap()..offset].to_vec();
                }
                start = None;
            }
        }
        panic!("Page Link group not found");
    }

    fn object_hidden(file: &[u8], target: usize) -> bool {
        let mut offset = 4;
        let mut ordinal = 0;
        while offset < file.len() {
            let len = u32_at(file, offset) as usize;
            let kind = u32_at(file, offset + 4);
            if kind == OBJECT_HEADER {
                ordinal += 1;
                if ordinal == target {
                    return file[offset + 8 + 2] != 0;
                }
            }
            offset += 8 + len + len % 2;
        }
        panic!("target object not found");
    }

    #[test]
    fn original_hide_click_and_reclick_round_trip_exactly() {
        let fixtures: [&[u8]; 3] = [
            include_bytes!("../tests/fixtures/action_hide_show.gsp"),
            include_bytes!("../tests/fixtures/action_hide_show_clicked.gsp"),
            include_bytes!("../tests/fixtures/action_hide_show_reclicked.gsp"),
        ];
        let groups = fixtures.map(action_group);
        let hide = HideShowButtonGroup::parse(&groups[0]).unwrap();
        let show = HideShowButtonGroup::parse(&groups[1]).unwrap();
        let restored = HideShowButtonGroup::parse(&groups[2]).unwrap();
        assert_eq!(hide.encode(), groups[0]);
        assert_eq!(show.encode(), groups[1]);
        assert_eq!(restored.encode(), groups[2]);
        assert_eq!(hide.target_ids(), vec![3]);
        assert_eq!(hide.screen_anchor(), (10, 5));
        assert_eq!(hide.action(), VisibilityAction::Hide);
        assert_eq!(show.action(), VisibilityAction::Show);
        assert_eq!(restored.action(), VisibilityAction::Hide);
        assert_eq!(hide.label_utf8().unwrap(), "隐藏线段");
        assert_eq!(show.label_utf8().unwrap(), "显示线段");
        assert!(!object_hidden(fixtures[0], 3));
        assert!(object_hidden(fixtures[1], 3));
        assert!(!object_hidden(fixtures[2], 3));
    }

    #[test]
    fn two_target_hide_show_groups_preserve_both_references() {
        let before = include_bytes!("../tests/fixtures/action_hide_show_two_points_original.gsp");
        let after = include_bytes!("../tests/fixtures/action_hide_show_two_points_clicked.gsp");
        let before_group = action_group(before);
        let after_group = action_group(after);
        let hide = HideShowButtonGroup::parse(&before_group).unwrap();
        let show = HideShowButtonGroup::parse(&after_group).unwrap();
        assert_eq!(hide.encode(), before_group);
        assert_eq!(show.encode(), after_group);
        assert_eq!(hide.target_ids(), vec![1, 2]);
        assert_eq!(show.target_ids(), vec![1, 2]);
        assert_eq!(hide.action(), VisibilityAction::Hide);
        assert_eq!(show.action(), VisibilityAction::Show);
        assert_eq!(hide.label_utf8().unwrap(), "隐藏点");
        assert_eq!(show.label_utf8().unwrap(), "显示点");
        assert!(!object_hidden(before, 1));
        assert!(!object_hidden(before, 2));
        assert!(object_hidden(after, 1));
        assert!(object_hidden(after, 2));
        assert!(!object_hidden(after, 3));

        let generated =
            HideShowButtonGroup::from_original_template_targets(&[1, 2], "Hide Objects", (10, 5))
                .unwrap();
        assert_eq!(generated.target_ids(), vec![1, 2]);
        assert_eq!(
            HideShowButtonGroup::parse(&generated.encode())
                .unwrap()
                .label_utf8()
                .unwrap(),
            "Hide Objects"
        );
    }

    #[test]
    fn generated_english_button_cross_opens_and_executes_in_sketchpad() {
        let generated = action_group(include_bytes!(
            "../tests/fixtures/action_generated_english.gsp"
        ));
        let clicked = action_group(include_bytes!(
            "../tests/fixtures/action_generated_english_clicked.gsp"
        ));
        let before = HideShowButtonGroup::parse(&generated).unwrap();
        let after = HideShowButtonGroup::parse(&clicked).unwrap();
        assert_eq!(before.encode(), generated);
        assert_eq!(after.encode(), clicked);
        assert_eq!(before.label_utf8().unwrap(), "Hide Segment");
        assert_eq!(before.action(), VisibilityAction::Hide);
        assert_eq!(after.action(), VisibilityAction::Show);
        assert_eq!(after.target_ids(), vec![3]);
        assert!(!object_hidden(
            include_bytes!("../tests/fixtures/action_generated_english.gsp"),
            3
        ));
        assert!(object_hidden(
            include_bytes!("../tests/fixtures/action_generated_english_clicked.gsp"),
            3
        ));
    }

    #[test]
    fn template_changes_known_fields_and_preserves_opaque_label_suffix() {
        let mut group =
            HideShowButtonGroup::from_original_template(7, "Hide Segment", (30, 40)).unwrap();
        assert_eq!(group.target_ids(), vec![7]);
        assert_eq!(group.label_utf8().unwrap(), "Hide Segment");
        assert_eq!(group.screen_anchor(), (30, 40));
        group.set_action(VisibilityAction::Show);
        group.set_label("Show Segment").unwrap();
        group.set_hidden(true);
        let reparsed = HideShowButtonGroup::parse(&group.encode()).unwrap();
        assert_eq!(reparsed.action(), VisibilityAction::Show);
        assert!(reparsed.is_hidden());
        assert_eq!(reparsed.label_utf8().unwrap(), "Show Segment");
        assert_eq!(&reparsed.records[3].payload[36..], &[0, 3]);
    }

    #[test]
    fn malformed_or_lossy_edits_are_rejected() {
        let template = include_bytes!("../assets/action_hide_show_group.bin");
        assert!(HideShowButtonGroup::parse(&template[..template.len() - 1]).is_err());
        let mut group = HideShowButtonGroup::parse(template).unwrap();
        assert!(group.set_single_target(0).is_err());
        assert!(group.set_label("Hi").is_ok());
        assert!(group.set_label("Odd").is_err());
        let mut bytes = template.to_vec();
        bytes[8] = 0;
        assert!(HideShowButtonGroup::parse(&bytes).is_err());
    }
}
