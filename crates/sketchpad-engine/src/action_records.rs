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
pub const BUTTON_METADATA: u32 = 0x232e;
pub const OBJECT_END: u32 = 0x07d7;
pub const BROKEN_PAGE_LINK_TARGET: usize = 0xbd66;

const MAX_POINT_ANIMATION_SPEED: f32 = 10_000.;
const MAX_PARAMETER_ANIMATION_SPEED: f32 = 400_000.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisibilityAction {
    Hide,
    Show,
}

/// Native Hide/Show policy word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisibilityOptions(u16);

impl VisibilityOptions {
    pub fn from_native(word: u16) -> Result<Self, String> {
        if word > 7 {
            return Err("Unsupported Hide/Show option word".into());
        }
        Ok(Self(word))
    }

    pub fn native_word(self) -> u16 {
        self.0
    }

    pub fn alternates(self) -> bool {
        self.0 & 4 != 0
    }

    pub fn select_after_show(self) -> bool {
        self.0 & 1 != 0
    }

    pub fn fade(self) -> bool {
        self.0 & 2 != 0
    }

    pub fn with_alternates(self, alternates: bool) -> Self {
        Self((self.0 & !4) | (u16::from(alternates) * 4))
    }

    pub fn with_select_after_show(self, select: bool) -> Self {
        Self((self.0 & !1) | u16::from(select))
    }

    pub fn with_fade(self, fade: bool) -> Self {
        Self((self.0 & !2) | (u16::from(fade) * 2))
    }
}

impl Default for VisibilityOptions {
    fn default() -> Self {
        Self(7)
    }
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
    /// Checks whether Native can represent and execute this settings block.
    /// Native imports zero-rate tracks and ignores Once Only for bidirectional
    /// motion even though its authoring dialog does not offer either choice.
    pub fn validate(self) -> Result<(), String> {
        let maximum_speed = if self.parameter.is_some() {
            // Imported reset-style parameter buttons use very large rates over
            // correspondingly tiny domains. Keep point speeds at the authored
            // boundary while admitting the observed finite parameter range.
            MAX_PARAMETER_ANIMATION_SPEED
        } else {
            MAX_POINT_ANIMATION_SPEED
        };
        if !self.speed.is_finite() || self.speed < 0. || self.speed > maximum_speed {
            return Err("Animation speed must be nonnegative and finite".into());
        }
        if let Some(parameter) = self.parameter
            && (!parameter.step.is_finite()
                || parameter.step <= 0.
                || !parameter.min.is_finite()
                || !parameter.max.is_finite()
                || parameter.min >= parameter.max
                || [self.speed, parameter.step, parameter.min, parameter.max]
                    .iter()
                    .any(|value| encode_parameter_number(*value).is_none()))
        {
            return Err("Invalid parameter Animation settings".into());
        }
        Ok(())
    }

    /// Applies the stricter choices exposed by Native's authoring dialog.
    pub(crate) fn validate_for_authoring(self) -> Result<(), String> {
        self.validate()?;
        if self.speed == 0. {
            return Err("Animation speed must be positive".into());
        }
        if self.once && self.direction == AnimationDirection::Bidirectional {
            return Err("Once Only is unavailable for bidirectional motion".into());
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

/// Ordered records, including the original optional label and empty metadata.
/// Required roles are unique after parsing, so access never depends on offsets.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ActionRecords {
    records: Vec<Record>,
}

#[derive(Clone, Copy)]
enum LabelParityPolicy {
    PreserveSuffix,
    MoveZeroSuffix,
    DropOpaqueAlignment,
}

impl ActionRecords {
    fn parse(bytes: &[u8], animation: bool) -> Result<Self, String> {
        let records = parse_record_list(bytes, if animation { 8 } else { 7 })?;
        let mut kinds = records.iter().map(|record| record.kind).peekable();
        for required in [OBJECT_HEADER, OBJECT_REFS] {
            if kinds.next() != Some(required) {
                return Err("Unexpected action button record order".into());
            }
        }
        if animation && kinds.next() != Some(ANIMATION_SETTINGS) {
            return Err("Missing Animation settings record".into());
        }
        if kinds.next() != Some(ACTION_PAYLOAD) {
            return Err("Missing action payload record".into());
        }
        if kinds.peek() == Some(&BUTTON_LABEL) {
            kinds.next();
        }
        if kinds.next() != Some(BUTTON_AUX) {
            return Err("Missing action auxiliary record".into());
        }
        if kinds.peek() == Some(&BUTTON_METADATA) {
            kinds.next();
        }
        if kinds.next() != Some(OBJECT_END) || kinds.next().is_some() {
            return Err("Unexpected action button record order".into());
        }
        let result = Self { records };
        if result
            .optional_record(BUTTON_METADATA)
            .is_some_and(|record| !record.payload.is_empty())
        {
            return Err("Unsupported nonempty action metadata".into());
        }
        if let Some(record) = result.optional_record(BUTTON_LABEL) {
            let label = &record.payload;
            if label.len() < 24 || u16_at(label, 22) as usize > label.len() - 24 {
                return Err("Action button label is truncated".into());
            }
        }
        Ok(result)
    }

    fn optional_record(&self, kind: u32) -> Option<&Record> {
        self.records.iter().find(|record| record.kind == kind)
    }

    fn record(&self, kind: u32) -> &Record {
        self.optional_record(kind)
            .expect("validated required action record")
    }

    fn record_mut(&mut self, kind: u32) -> &mut Record {
        self.records
            .iter_mut()
            .find(|record| record.kind == kind)
            .expect("validated required action record")
    }

    fn label_bytes(&self) -> &[u8] {
        let Some(record) = self.optional_record(BUTTON_LABEL) else {
            return &[];
        };
        let len = u16_at(&record.payload, 22) as usize;
        &record.payload[24..24 + len]
    }

    fn set_label(
        &mut self,
        label: &str,
        template: &[u8],
        parity_policy: LabelParityPolicy,
    ) -> Result<(), String> {
        let bytes = label.as_bytes();
        let len =
            u16::try_from(bytes.len()).map_err(|_| "Action button label exceeds 65535 bytes")?;
        if bytes == self.label_bytes() {
            return Ok(());
        }
        let existing = self.optional_record(BUTTON_LABEL);
        let inserting = existing.is_none();
        let mut record = if let Some(record) = existing {
            record.clone()
        } else {
            parse_record_list(template, 8)?
                .into_iter()
                .find(|record| record.kind == BUTTON_LABEL)
                .ok_or("Action template has no label record")?
        };
        let old_len = u16_at(&record.payload, 22) as usize;
        let mut suffix = record.payload[24 + old_len..].to_vec();
        if old_len % 2 != bytes.len() % 2 {
            // New labels use the Native template's text terminator convention.
            // Some action families retain zero alignment bytes. Sound labels
            // may additionally carry one opaque allocation byte after their
            // terminator, which Native drops when the label parity changes.
            let expected_suffix_len = if old_len.is_multiple_of(2) { 2 } else { 1 };
            let suffix_can_move = suffix.len() == expected_suffix_len
                && match parity_policy {
                    LabelParityPolicy::PreserveSuffix => false,
                    LabelParityPolicy::MoveZeroSuffix => suffix.iter().all(|byte| *byte == 0),
                    LabelParityPolicy::DropOpaqueAlignment => suffix.first() == Some(&0),
                };
            if !inserting && !suffix_can_move {
                return Err(
                    "Action button label byte parity differs; opaque suffix cannot be moved safely"
                        .into(),
                );
            }
            suffix = vec![0; if bytes.len().is_multiple_of(2) { 2 } else { 1 }];
        }
        record.payload.truncate(22);
        record.payload.extend_from_slice(&len.to_le_bytes());
        record.payload.extend_from_slice(bytes);
        record.payload.extend_from_slice(&suffix);
        record.padding = if !record.payload.len().is_multiple_of(2) {
            Some(record.padding.unwrap_or(0))
        } else {
            None
        };
        if inserting {
            let index = self
                .records
                .iter()
                .position(|record| record.kind == BUTTON_AUX)
                .expect("validated action auxiliary record");
            self.records.insert(index, record);
        } else {
            *self.record_mut(BUTTON_LABEL) = record;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HideShowButtonGroup {
    records: ActionRecords,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationButtonGroup {
    records: ActionRecords,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoundButtonGroup {
    records: ActionRecords,
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
    records: ActionRecords,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollPlacement {
    UpperLeft,
    Center,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollButtonGroup {
    records: ActionRecords,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationMode {
    Simultaneous,
    Sequential,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationStop {
    LastAction,
    FirstAction,
    ElapsedTime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresentationOptions {
    pub deselect: bool,
    pub stop_animations: bool,
    pub erase_traces: bool,
    pub stop: PresentationStop,
    /// Native stores presentation durations and pauses in tenths of a second.
    pub time_tenths: u16,
    /// Retained native progress/state word. Newly authored buttons use zero.
    pub native_state: u16,
}

impl PresentationOptions {
    pub fn for_mode(mode: PresentationMode) -> Self {
        Self {
            time_tenths: if mode == PresentationMode::Simultaneous {
                600
            } else {
                0
            },
            ..Self::default()
        }
    }
}

impl Default for PresentationOptions {
    fn default() -> Self {
        Self {
            deselect: false,
            stop_animations: false,
            erase_traces: false,
            stop: PresentationStop::LastAction,
            time_tenths: 600,
            native_state: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresentationButtonGroup {
    records: ActionRecords,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkUrlButtonGroup {
    records: [Record; 6],
    text_alignment: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkPageButtonGroup {
    records: Vec<Record>,
}

impl LinkPageButtonGroup {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let records = parse_record_list(bytes, 7)?;
        let kinds = records.iter().map(|record| record.kind).collect::<Vec<_>>();
        if kinds
            != [
                OBJECT_HEADER,
                ACTION_PAYLOAD,
                BUTTON_LABEL,
                BUTTON_AUX,
                0x08a2,
                OBJECT_END,
            ]
            && kinds
                != [
                    OBJECT_HEADER,
                    ACTION_PAYLOAD,
                    BUTTON_LABEL,
                    BUTTON_AUX,
                    0x08a2,
                    BUTTON_METADATA,
                    OBJECT_END,
                ]
        {
            return Err("Unexpected Page Link record order".into());
        }
        let group = Self { records };
        group.validate()?;
        Ok(group)
    }

    pub fn from_original_template(
        target_page: usize,
        label: &str,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        let mut group = Self::parse(include_bytes!("../../../assets/action_link_page_group.bin"))?;
        group.set_target_page(target_page)?;
        group.set_label(label)?;
        group.set_screen_anchor(anchor)?;
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
        // The alignment byte after the NUL may contain native allocation data.
        let valid_label = if label.len() >= 26 {
            let end = 24 + u16_at(label, 22) as usize;
            end < label.len() && label[end] == 0 && label.len() == (end + 2) & !1
        } else {
            false
        };
        if !valid_label
            || self.records[3].payload.len() != 2
            || self.records[4].payload.len() != 8
            || self.records[5..]
                .iter()
                .any(|record| !record.payload.is_empty())
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
        if self.label_utf8().is_ok_and(|current| current == label) {
            return Ok(());
        }
        set_zero_terminated_text(&mut self.records[2], 22, label)
    }

    pub fn screen_anchor(&self) -> (i16, i16) {
        caption_rect_anchor(&self.records[4].payload)
    }

    pub fn set_screen_anchor(&mut self, anchor: (i16, i16)) -> Result<(), String> {
        translate_caption_rect(&mut self.records[4].payload, anchor)?;
        self.records[1].payload[20..22].copy_from_slice(&anchor.0.to_le_bytes());
        self.records[1].payload[22..24].copy_from_slice(&anchor.1.to_le_bytes());
        Ok(())
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
        // Some native URL records pad their string payload to four bytes.
        let action = &records[1].payload;
        let text_alignment = if action.len() >= 26 {
            let end = 26usize + usize::from(u16_at(action, 24));
            if action.len() > (end + 2) & !1 { 4 } else { 2 }
        } else {
            2
        };
        let group = Self {
            records,
            text_alignment,
        };
        group.validate()?;
        Ok(group)
    }

    pub fn from_original_template(
        url: &str,
        label: &str,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        let mut group = Self::parse(include_bytes!("../../../assets/action_link_url_group.bin"))?;
        group.set_url(url)?;
        group.set_label(label)?;
        group.set_screen_anchor(anchor)?;
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
            // Native word 14 may contain opaque data (Ant: 0x3264); keep it verbatim.
            || u32_at(action, 16) != 0
            || !valid_url_zero_terminated_text(action, 24)
        {
            return Err("Unsupported Link URL action".into());
        }
        if label.len() < 26 || !valid_url_zero_terminated_text(label, 22) {
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
        if self.url_utf8().is_ok_and(|current| current == url) {
            return Ok(());
        }
        let action = &self.records[1].payload;
        let end = 26 + usize::from(u16_at(action, 24));
        if action[end + 1..].iter().any(|&byte| byte != 0) {
            return Err("Editing a URL with opaque native suffix bytes is not supported".into());
        }
        set_zero_terminated_text_aligned(&mut self.records[1], 24, url, self.text_alignment)?;
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
        if self.label_utf8().is_ok_and(|current| current == label) {
            return Ok(());
        }
        set_zero_terminated_text_aligned(&mut self.records[2], 22, label, self.text_alignment)
    }

    pub fn screen_anchor(&self) -> (i16, i16) {
        // Native rendering uses 08a2, not the duplicate action coordinates.
        caption_rect_anchor(&self.records[4].payload)
    }

    pub fn set_screen_anchor(&mut self, anchor: (i16, i16)) -> Result<(), String> {
        translate_caption_rect(&mut self.records[4].payload, anchor)?;
        self.records[1].payload[20..22].copy_from_slice(&anchor.0.to_le_bytes());
        self.records[1].payload[22..24].copy_from_slice(&anchor.1.to_le_bytes());
        Ok(())
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.records[0].payload[2] = u8::from(hidden);
    }

    pub fn encode(&self) -> Vec<u8> {
        encode_records(&self.records)
    }
}

fn caption_rect_anchor(rect: &[u8]) -> (i16, i16) {
    (
        i16::from_le_bytes(rect[..2].try_into().unwrap()),
        i16::from_le_bytes(rect[2..4].try_into().unwrap()),
    )
}

fn translate_caption_rect(rect: &mut [u8], anchor: (i16, i16)) -> Result<(), String> {
    let before = caption_rect_anchor(rect);
    let delta = [
        i32::from(anchor.0) - i32::from(before.0),
        i32::from(anchor.1) - i32::from(before.1),
    ];
    let mut shifted = [0u8; 8];
    for (index, coordinate) in rect.chunks_exact(2).enumerate() {
        let value =
            i32::from(i16::from_le_bytes(coordinate.try_into().unwrap())) + delta[index % 2];
        let value = i16::try_from(value)
            .map_err(|_| "Link caption rectangle exceeds native coordinate range")?;
        shifted[index * 2..index * 2 + 2].copy_from_slice(&value.to_le_bytes());
    }
    rect.copy_from_slice(&shifted);
    Ok(())
}

// Native URL buttons use either even-byte or four-byte string payload alignment.
// Alignment bytes after the terminator are opaque in both action and label
// records, and can remain nonzero after an original-application save.
fn valid_url_zero_terminated_text(payload: &[u8], length_offset: usize) -> bool {
    if payload.len() < length_offset + 4 {
        return false;
    }
    let end = length_offset + 2 + usize::from(u16_at(payload, length_offset));
    end < payload.len()
        && payload[end] == 0
        && [2usize, 4]
            .iter()
            .any(|alignment| payload.len() == (end + alignment) & !(alignment - 1))
}

fn set_zero_terminated_text(
    record: &mut Record,
    length_offset: usize,
    text: &str,
) -> Result<(), String> {
    set_zero_terminated_text_aligned(record, length_offset, text, 2)
}

fn set_zero_terminated_text_aligned(
    record: &mut Record,
    length_offset: usize,
    text: &str,
    alignment: usize,
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
    while !record.payload.len().is_multiple_of(alignment) {
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

fn parse_record_list(bytes: &[u8], max_records: usize) -> Result<Vec<Record>, String> {
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
            padding: (!len.is_multiple_of(2)).then(|| bytes[end]),
        });
        offset = padded_end;
        if parsed.len() > max_records {
            return Err("Unexpected extra record in action button group".into());
        }
    }
    Ok(parsed)
}

fn parse_records<const N: usize>(bytes: &[u8], kinds: [u32; N]) -> Result<[Record; N], String> {
    let records: [Record; N] = parse_record_list(bytes, N)?
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
        if !record.payload.len().is_multiple_of(2) {
            bytes.push(record.padding.unwrap_or(0));
        }
    }
    bytes
}

impl HideShowButtonGroup {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let records = ActionRecords::parse(bytes, false)?;
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
        let template: &[u8] = if label.len().is_multiple_of(2) {
            include_bytes!("../../../assets/action_hide_show_group.bin")
        } else {
            include_bytes!("../../../assets/action_hide_show_two_points_group.bin")
        };
        let mut group = Self::parse(template)?;
        group.set_targets(targets)?;
        group.set_label(label)?;
        group.set_screen_anchor(anchor);
        Ok(group)
    }

    fn validate(&self) -> Result<(), String> {
        let header = &self.records.record(OBJECT_HEADER).payload;
        if header.len() != 28 || u16_at(header, 0) != 62 {
            return Err("Hide/Show button needs a 28-byte object code 62 header".into());
        }
        let refs = &self.records.record(OBJECT_REFS).payload;
        if refs.len() < 8
            || !(refs.len() - 4).is_multiple_of(4)
            || u32_at(refs, 0) as usize != (refs.len() - 4) / 4
        {
            return Err("Hide/Show button has invalid object references".into());
        }
        if self.target_ids().contains(&0) {
            return Err("Hide/Show button references object zero".into());
        }
        let action = &self.records.record(ACTION_PAYLOAD).payload;
        if action.len() != 24
            || u32_at(action, 0) != ACTION_PAYLOAD
            || u32_at(action, 4) != 24
            || (!matches!(u16_at(action, 12), 0 | 1)
                || VisibilityOptions::from_native(u16_at(action, 14)).is_err())
        {
            return Err("Unsupported Hide/Show action payload".into());
        }
        if self.records.record(BUTTON_AUX).payload.len() != 2
            || !self.records.record(OBJECT_END).payload.is_empty()
        {
            return Err("Hide/Show button has invalid auxiliary or terminator record".into());
        }
        Ok(())
    }

    pub fn target_ids(&self) -> Vec<u32> {
        self.records.record(OBJECT_REFS).payload[4..]
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
        let payload = &mut self.records.record_mut(OBJECT_REFS).payload;
        payload.clear();
        payload.extend_from_slice(&count.to_le_bytes());
        for target in targets {
            payload.extend_from_slice(&target.to_le_bytes());
        }
        Ok(())
    }

    pub fn action(&self) -> VisibilityAction {
        if u16_at(&self.records.record(ACTION_PAYLOAD).payload, 12) == 0 {
            VisibilityAction::Hide
        } else {
            VisibilityAction::Show
        }
    }

    pub fn options(&self) -> VisibilityOptions {
        VisibilityOptions(u16_at(&self.records.record(ACTION_PAYLOAD).payload, 14))
    }

    pub fn set_options(&mut self, options: VisibilityOptions) {
        self.records.record_mut(ACTION_PAYLOAD).payload[14..16]
            .copy_from_slice(&options.native_word().to_le_bytes());
    }

    /// Retain both the Native label format and its separate visible word.
    pub fn caption_flags(&self) -> Option<u32> {
        self.records
            .optional_record(BUTTON_LABEL)
            .map(|record| u32_at(&record.payload, 0))
    }

    pub fn set_caption_flags(&mut self, flags: Option<u32>) -> Result<(), String> {
        let Some(flags) = flags else {
            self.records
                .records
                .retain(|record| record.kind != BUTTON_LABEL);
            return Ok(());
        };
        if self.records.optional_record(BUTTON_LABEL).is_none() {
            let template = ActionRecords::parse(
                include_bytes!("../../../assets/action_hide_show_two_points_group.bin"),
                false,
            )?;
            let index = self
                .records
                .records
                .iter()
                .position(|record| record.kind == BUTTON_AUX)
                .expect("validated action auxiliary record");
            self.records
                .records
                .insert(index, template.record(BUTTON_LABEL).clone());
            self.records.set_label(
                "",
                include_bytes!("../../../assets/action_hide_show_two_points_group.bin"),
                LabelParityPolicy::MoveZeroSuffix,
            )?;
        }
        self.records.record_mut(BUTTON_LABEL).payload[..4].copy_from_slice(&flags.to_le_bytes());
        Ok(())
    }

    pub fn set_action(&mut self, action: VisibilityAction) {
        let code: u16 = match action {
            VisibilityAction::Hide => 0,
            VisibilityAction::Show => 1,
        };
        self.records.record_mut(ACTION_PAYLOAD).payload[12..14]
            .copy_from_slice(&code.to_le_bytes());
    }

    pub fn set_link_destination(&mut self, linked: bool) {
        self.records.record_mut(ACTION_PAYLOAD).payload[18..20]
            .copy_from_slice(&u16::from(linked).to_le_bytes());
    }

    pub fn screen_anchor(&self) -> (i16, i16) {
        let payload = &self.records.record(ACTION_PAYLOAD).payload;
        (
            i16::from_le_bytes(payload[20..22].try_into().unwrap()),
            i16::from_le_bytes(payload[22..24].try_into().unwrap()),
        )
    }

    pub fn set_screen_anchor(&mut self, anchor: (i16, i16)) {
        self.records.record_mut(ACTION_PAYLOAD).payload[20..22]
            .copy_from_slice(&anchor.0.to_le_bytes());
        self.records.record_mut(ACTION_PAYLOAD).payload[22..24]
            .copy_from_slice(&anchor.1.to_le_bytes());
    }

    pub fn label_bytes(&self) -> &[u8] {
        self.records.label_bytes()
    }

    pub fn label_utf8(&self) -> Result<&str, String> {
        std::str::from_utf8(self.label_bytes())
            .map_err(|_| "Action button label is not UTF-8".into())
    }

    pub fn set_label(&mut self, label: &str) -> Result<(), String> {
        self.records.set_label(
            label,
            include_bytes!("../../../assets/action_hide_show_two_points_group.bin"),
            LabelParityPolicy::PreserveSuffix,
        )
    }

    /// Native action-driven captions rebuild their terminating zero and its
    /// alignment byte. This is separate from strict user text edits, whose
    /// arbitrary suffix bytes remain opaque and preserved.
    pub fn set_generated_label(&mut self, label: &str) -> Result<(), String> {
        let len =
            u16::try_from(label.len()).map_err(|_| "Action button label exceeds 65535 bytes")?;
        let Some(old) = self.records.optional_record(BUTTON_LABEL) else {
            if label.is_empty() {
                return Ok(());
            }
            return Err("An absent caption cannot acquire a generated label".into());
        };
        let old_len = u16_at(&old.payload, 22) as usize;
        let suffix = &old.payload[24 + old_len..];
        if suffix.len() != if old_len.is_multiple_of(2) { 2 } else { 1 }
            || suffix.first() != Some(&0)
        {
            return Err("Unsupported generated caption suffix".into());
        }
        let mut record = old.clone();
        record.payload.truncate(22);
        record.payload.extend_from_slice(&len.to_le_bytes());
        record.payload.extend_from_slice(label.as_bytes());
        record.payload.extend(std::iter::repeat_n(
            0,
            if label.len().is_multiple_of(2) { 2 } else { 1 },
        ));
        record.padding = None;
        *self.records.record_mut(BUTTON_LABEL) = record;
        Ok(())
    }

    pub fn is_hidden(&self) -> bool {
        self.records.record(OBJECT_HEADER).payload[2] != 0
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        let flags = &mut self.records.record_mut(OBJECT_HEADER).payload[2];
        *flags = (*flags & !1) | u8::from(hidden);
    }

    pub fn set_presentation_suppressed(&mut self, suppressed: bool) {
        let flags = &mut self.records.record_mut(OBJECT_HEADER).payload[2];
        *flags = (*flags & !2) | (u8::from(suppressed) << 1);
    }

    pub fn encode(&self) -> Vec<u8> {
        encode_records(&self.records.records)
    }
}

impl PresentationButtonGroup {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let records = ActionRecords::parse(bytes, false)?;
        let group = Self { records };
        group.validate()?;
        Ok(group)
    }

    pub fn from_original_template_targets(
        targets: &[u32],
        label: &str,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        let mut group = Self::parse(include_bytes!(
            "../../../assets/action_presentation_group.bin"
        ))?;
        group.set_targets(targets)?;
        group.set_label(label)?;
        group.set_screen_anchor(anchor);
        Ok(group)
    }

    fn validate(&self) -> Result<(), String> {
        let header = &self.records.record(OBJECT_HEADER).payload;
        let refs = &self.records.record(OBJECT_REFS).payload;
        let action = &self.records.record(ACTION_PAYLOAD).payload;
        if header.len() != 28 || u16_at(header, 0) != 62 {
            return Err("Presentation button needs an object code 62 header".into());
        }
        if refs.len() < 8
            || !(refs.len() - 4).is_multiple_of(4)
            || u32_at(refs, 0) as usize != (refs.len() - 4) / 4
            || self.target_ids().contains(&0)
        {
            return Err("Presentation button needs nonzero parent button IDs".into());
        }
        let flags = (action.len() == 24).then(|| u16_at(action, 14));
        let valid_settings = flags.is_some_and(|flags| {
            flags & !0x3f == 0
                && flags & 0x30 != 0x30
                && (flags & 1 == 0 || flags & 0x30 == 0)
                && u16_at(action, 18) as usize <= self.target_ids().len()
        });
        if action.len() != 24
            || u32_at(action, 0) != ACTION_PAYLOAD
            || u32_at(action, 4) != 24
            || u32_at(action, 8) != 12
            || u16_at(action, 12) != 7
            || !valid_settings
        {
            return Err("Unsupported Presentation button action".into());
        }
        if self.records.record(BUTTON_AUX).payload.len() != 2
            || !self.records.record(OBJECT_END).payload.is_empty()
        {
            return Err("Presentation button auxiliary records are invalid".into());
        }
        Ok(())
    }

    pub fn target_ids(&self) -> Vec<u32> {
        self.records.record(OBJECT_REFS).payload[4..]
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect()
    }

    pub fn set_targets(&mut self, targets: &[u32]) -> Result<(), String> {
        let count = u32::try_from(targets.len()).map_err(|_| "Too many Presentation targets")?;
        if count == 0 || targets.contains(&0) {
            return Err("Presentation button needs nonzero targets".into());
        }
        let refs = &mut self.records.record_mut(OBJECT_REFS).payload;
        refs.clear();
        refs.extend_from_slice(&count.to_le_bytes());
        for target in targets {
            refs.extend_from_slice(&target.to_le_bytes());
        }
        Ok(())
    }

    pub fn mode(&self) -> PresentationMode {
        if u16_at(&self.records.record(ACTION_PAYLOAD).payload, 14) & 1 == 0 {
            PresentationMode::Simultaneous
        } else {
            PresentationMode::Sequential
        }
    }

    pub fn options(&self) -> PresentationOptions {
        let action = &self.records.record(ACTION_PAYLOAD).payload;
        let flags = u16_at(action, 14);
        PresentationOptions {
            deselect: flags & 0x02 != 0,
            stop_animations: flags & 0x04 != 0,
            erase_traces: flags & 0x08 != 0,
            stop: match flags & 0x30 {
                0x10 => PresentationStop::FirstAction,
                0x20 => PresentationStop::ElapsedTime,
                _ => PresentationStop::LastAction,
            },
            time_tenths: u16_at(action, 16),
            native_state: u16_at(action, 18),
        }
    }

    pub fn set_mode(&mut self, mode: PresentationMode) {
        self.set_options(mode, PresentationOptions::for_mode(mode));
    }

    pub fn set_options(&mut self, mode: PresentationMode, options: PresentationOptions) {
        let stop = if mode == PresentationMode::Sequential {
            0
        } else {
            match options.stop {
                PresentationStop::LastAction => 0,
                PresentationStop::FirstAction => 0x10,
                PresentationStop::ElapsedTime => 0x20,
            }
        };
        let flags = u16::from(mode == PresentationMode::Sequential)
            | if options.deselect { 0x02 } else { 0 }
            | if options.stop_animations { 0x04 } else { 0 }
            | if options.erase_traces { 0x08 } else { 0 }
            | stop;
        let action = &mut self.records.record_mut(ACTION_PAYLOAD).payload;
        action[14..16].copy_from_slice(&flags.to_le_bytes());
        action[16..18].copy_from_slice(&options.time_tenths.to_le_bytes());
        action[18..20].copy_from_slice(&options.native_state.to_le_bytes());
    }

    pub fn screen_anchor(&self) -> (i16, i16) {
        let payload = &self.records.record(ACTION_PAYLOAD).payload;
        (
            i16::from_le_bytes(payload[20..22].try_into().unwrap()),
            i16::from_le_bytes(payload[22..24].try_into().unwrap()),
        )
    }

    pub fn set_screen_anchor(&mut self, anchor: (i16, i16)) {
        self.records.record_mut(ACTION_PAYLOAD).payload[20..22]
            .copy_from_slice(&anchor.0.to_le_bytes());
        self.records.record_mut(ACTION_PAYLOAD).payload[22..24]
            .copy_from_slice(&anchor.1.to_le_bytes());
    }

    pub fn label_utf8(&self) -> Result<&str, String> {
        std::str::from_utf8(self.records.label_bytes())
            .map_err(|_| "Action button label is not UTF-8".into())
    }

    pub fn set_label(&mut self, label: &str) -> Result<(), String> {
        self.records.set_label(
            label,
            include_bytes!("../../../assets/action_presentation_group.bin"),
            LabelParityPolicy::PreserveSuffix,
        )
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        let flags = &mut self.records.record_mut(OBJECT_HEADER).payload[2];
        *flags = (*flags & !1) | u8::from(hidden);
    }

    pub fn set_presentation_suppressed(&mut self, suppressed: bool) {
        let flags = &mut self.records.record_mut(OBJECT_HEADER).payload[2];
        *flags = (*flags & !2) | (u8::from(suppressed) << 1);
    }

    pub fn encode(&self) -> Vec<u8> {
        encode_records(&self.records.records)
    }
}

impl ScrollButtonGroup {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let records = ActionRecords::parse(bytes, false)?;
        let group = Self { records };
        group.validate()?;
        Ok(group)
    }

    pub fn from_original_template(
        target: u32,
        label: &str,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        let mut group = Self::parse(include_bytes!("../../../assets/action_scroll_group.bin"))?;
        group.set_target(target)?;
        group.set_label(label)?;
        group.set_screen_anchor(anchor);
        Ok(group)
    }

    fn validate(&self) -> Result<(), String> {
        let header = &self.records.record(OBJECT_HEADER).payload;
        let refs = &self.records.record(OBJECT_REFS).payload;
        let action = &self.records.record(ACTION_PAYLOAD).payload;
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
            || u16_at(action, 16) != 0
            || u16_at(action, 18) > 1
        {
            return Err("Unsupported Scroll button action".into());
        }
        if self.records.record(BUTTON_AUX).payload.len() != 2
            || !self.records.record(OBJECT_END).payload.is_empty()
        {
            return Err("Scroll button auxiliary records are invalid".into());
        }
        Ok(())
    }

    pub fn target_id(&self) -> u32 {
        u32_at(&self.records.record(OBJECT_REFS).payload, 4)
    }

    pub fn set_target(&mut self, target: u32) -> Result<(), String> {
        if target == 0 {
            return Err("Scroll target must be nonzero".into());
        }
        self.records.record_mut(OBJECT_REFS).payload[4..8].copy_from_slice(&target.to_le_bytes());
        Ok(())
    }

    pub fn placement(&self) -> ScrollPlacement {
        if u16_at(&self.records.record(ACTION_PAYLOAD).payload, 14) == 0 {
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
        self.records.record_mut(ACTION_PAYLOAD).payload[14..16]
            .copy_from_slice(&code.to_le_bytes());
    }

    pub fn screen_anchor(&self) -> (i16, i16) {
        let payload = &self.records.record(ACTION_PAYLOAD).payload;
        (
            i16::from_le_bytes(payload[20..22].try_into().unwrap()),
            i16::from_le_bytes(payload[22..24].try_into().unwrap()),
        )
    }

    pub fn set_screen_anchor(&mut self, anchor: (i16, i16)) {
        self.records.record_mut(ACTION_PAYLOAD).payload[20..22]
            .copy_from_slice(&anchor.0.to_le_bytes());
        self.records.record_mut(ACTION_PAYLOAD).payload[22..24]
            .copy_from_slice(&anchor.1.to_le_bytes());
    }

    pub fn label_utf8(&self) -> Result<&str, String> {
        std::str::from_utf8(self.records.label_bytes())
            .map_err(|_| "Action button label is not UTF-8".into())
    }

    pub fn set_label(&mut self, label: &str) -> Result<(), String> {
        self.records.set_label(
            label,
            include_bytes!("../../../assets/action_scroll_group.bin"),
            LabelParityPolicy::PreserveSuffix,
        )
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.records.record_mut(OBJECT_HEADER).payload[2] = u8::from(hidden);
    }

    pub fn encode(&self) -> Vec<u8> {
        encode_records(&self.records.records)
    }
}

impl MovementButtonGroup {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let records = ActionRecords::parse(bytes, false)?;
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
        Self::from_original_template_pairs(&[(source, destination)], label, anchor)
    }

    pub fn from_original_template_pairs(
        pairs: &[(u32, u32)],
        label: &str,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        let mut group = Self::parse(include_bytes!("../../../assets/action_movement_group.bin"))?;
        group.set_target_pairs(pairs)?;
        // Native omits 07d5 for a newly authored empty caption. Reusing the
        // template record for nonempty captions follows its zero suffix layout.
        group
            .records
            .records
            .retain(|record| record.kind != BUTTON_LABEL);
        group.set_label(label)?;
        group.set_screen_anchor(anchor);
        Ok(group)
    }

    fn validate(&self) -> Result<(), String> {
        let header = &self.records.record(OBJECT_HEADER).payload;
        let refs = &self.records.record(OBJECT_REFS).payload;
        let action = &self.records.record(ACTION_PAYLOAD).payload;
        if header.len() != 28 || u16_at(header, 0) != 62 {
            return Err("Movement button needs an object code 62 header".into());
        }
        if refs.len() < 12
            || !(refs.len() - 4).is_multiple_of(8)
            || u32_at(refs, 0) as usize != (refs.len() - 4) / 4
            || self.target_ids().contains(&0)
        {
            return Err("Movement button needs nonzero source/destination pairs".into());
        }
        if action.len() != 24
            || u32_at(action, 0) != ACTION_PAYLOAD
            || u32_at(action, 4) != 24
            || u32_at(action, 8) != 12
            || u16_at(action, 12) != 3
            || u16_at(action, 14) > 3
            || u16_at(action, 16) > 1
            || u16_at(action, 18) > 1
        {
            return Err("Unsupported Movement button action".into());
        }
        if self.records.record(BUTTON_AUX).payload.len() != 2
            || !self.records.record(OBJECT_END).payload.is_empty()
        {
            return Err("Movement button auxiliary records are invalid".into());
        }
        Ok(())
    }

    pub fn target_ids(&self) -> Vec<u32> {
        self.records.record(OBJECT_REFS).payload[4..]
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect()
    }

    pub fn target_pairs(&self) -> Vec<(u32, u32)> {
        self.target_ids()
            .chunks_exact(2)
            .map(|pair| (pair[0], pair[1]))
            .collect()
    }

    /// Replace the target list with one source/destination pair.
    pub fn set_targets(&mut self, source: u32, destination: u32) -> Result<(), String> {
        self.set_target_pairs(&[(source, destination)])
    }

    pub fn set_target_pairs(&mut self, pairs: &[(u32, u32)]) -> Result<(), String> {
        let count = pairs
            .len()
            .checked_mul(2)
            .and_then(|count| u32::try_from(count).ok())
            .ok_or("Too many Movement targets")?;
        if pairs.is_empty()
            || pairs
                .iter()
                .any(|&(source, destination)| source == 0 || destination == 0)
        {
            return Err("Movement targets must be nonzero pairs".into());
        }
        let payload = &mut self.records.record_mut(OBJECT_REFS).payload;
        payload.clear();
        payload.extend_from_slice(&count.to_le_bytes());
        for &(source, destination) in pairs {
            payload.extend_from_slice(&source.to_le_bytes());
            payload.extend_from_slice(&destination.to_le_bytes());
        }
        Ok(())
    }

    pub fn speed(&self) -> MovementSpeed {
        match u16_at(&self.records.record(ACTION_PAYLOAD).payload, 14) {
            0 => MovementSpeed::Slow,
            1 => MovementSpeed::Medium,
            2 => MovementSpeed::Fast,
            _ => MovementSpeed::Instant,
        }
    }

    pub fn set_speed(&mut self, speed: MovementSpeed) {
        self.records.record_mut(ACTION_PAYLOAD).payload[14..16]
            .copy_from_slice(&speed.code().to_le_bytes());
    }

    pub fn follows_destination(&self) -> bool {
        u16_at(&self.records.record(ACTION_PAYLOAD).payload, 16) == 0
    }

    pub fn set_follows_destination(&mut self, follow: bool) {
        let code: u16 = if follow { 0 } else { 1 };
        self.records.record_mut(ACTION_PAYLOAD).payload[16..18]
            .copy_from_slice(&code.to_le_bytes());
    }

    pub fn screen_anchor(&self) -> (i16, i16) {
        let payload = &self.records.record(ACTION_PAYLOAD).payload;
        (
            i16::from_le_bytes(payload[20..22].try_into().unwrap()),
            i16::from_le_bytes(payload[22..24].try_into().unwrap()),
        )
    }

    pub fn set_screen_anchor(&mut self, anchor: (i16, i16)) {
        self.records.record_mut(ACTION_PAYLOAD).payload[20..22]
            .copy_from_slice(&anchor.0.to_le_bytes());
        self.records.record_mut(ACTION_PAYLOAD).payload[22..24]
            .copy_from_slice(&anchor.1.to_le_bytes());
    }

    pub fn label_utf8(&self) -> Result<&str, String> {
        std::str::from_utf8(self.records.label_bytes())
            .map_err(|_| "Action button label is not UTF-8".into())
    }

    pub fn set_label(&mut self, label: &str) -> Result<(), String> {
        self.records.set_label(
            label,
            include_bytes!("../../../assets/action_movement_group.bin"),
            LabelParityPolicy::MoveZeroSuffix,
        )
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.records.record_mut(OBJECT_HEADER).payload[2] = u8::from(hidden);
    }

    pub fn encode(&self) -> Vec<u8> {
        encode_records(&self.records.records)
    }
}

impl AnimationButtonGroup {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let records = ActionRecords::parse(bytes, true)?;
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
        let mut group = Self::parse(include_bytes!("../../../assets/action_animation_group.bin"))?;
        group.set_targets(targets)?;
        group.set_label(label)?;
        group.set_screen_anchor(anchor);
        Ok(group)
    }

    fn validate(&self) -> Result<(), String> {
        let header = &self.records.record(OBJECT_HEADER).payload;
        let modern_header = header.len() == 28;
        let legacy_header = header.len() == 16 && header[6..12] == [10, 0, 23, 0, 1, 0];
        if u16_at(header, 0) != 62 || !(modern_header || legacy_header) {
            return Err("Animation button needs a supported object code 62 header".into());
        }
        let refs = &self.records.record(OBJECT_REFS).payload;
        if refs.len() < 8
            || !(refs.len() - 4).is_multiple_of(4)
            || u32_at(refs, 0) as usize != (refs.len() - 4) / 4
            || self.target_ids().contains(&0)
        {
            return Err("Animation button needs nonzero targets".into());
        }
        let settings = &self.records.record(ANIMATION_SETTINGS).payload;
        // The settings payload starts after the 36-byte header group,
        // 16-byte target record, and its own 8-byte record header.
        let default_settings =
            &include_bytes!("../../../assets/action_animation_group.bin")[60..96];
        if settings.len() != 4 + 32 * self.target_ids().len()
            || settings[..4] != default_settings[..4]
            || settings[4..].chunks_exact(32).any(|block| {
                block.iter().zip(&default_settings[4..]).enumerate().any(
                    |(index, (actual, expected))| !matches!(index, 3..=19) && actual != expected,
                )
            })
            || self.settings_list().is_err()
        {
            return Err("Unsupported Animation button settings".into());
        }
        let action = &self.records.record(ACTION_PAYLOAD).payload;
        if action.len() != 24
            || u32_at(action, 0) != ACTION_PAYLOAD
            || u32_at(action, 4) != 24
            || u32_at(action, 8) != 12
            || u32_at(action, 12) != 2
        {
            return Err("Unsupported Animation button action".into());
        }
        if self.records.record(BUTTON_AUX).payload.len() != 2
            || !self.records.record(OBJECT_END).payload.is_empty()
        {
            return Err("Animation button has invalid auxiliary or terminator record".into());
        }
        Ok(())
    }

    pub fn target_id(&self) -> u32 {
        u32_at(&self.records.record(OBJECT_REFS).payload, 4)
    }

    pub fn target_ids(&self) -> Vec<u32> {
        self.records.record(OBJECT_REFS).payload[4..]
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
        let bytes = &self.records.record(ANIMATION_SETTINGS).payload;
        if bytes.len() < 36 || !(bytes.len() - 4).is_multiple_of(32) {
            return Err("Animation settings are truncated".into());
        }
        bytes[4..]
            .chunks_exact(32)
            .map(|block| {
                let flags = block[3];
                let point_settings =
                    &include_bytes!("../../../assets/action_animation_group.bin")[72..84];
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
                let native_mode = flags & 0x07;
                let mode_is_known = if parameter.is_some() {
                    matches!(native_mode, 0 | 2 | 4)
                } else {
                    matches!(native_mode, 0 | 1)
                };
                if !mode_is_known || flags & !(if parameter.is_some() { 0xbf } else { 0xb1 }) != 0 {
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
        let bytes = &mut self.records.record_mut(ANIMATION_SETTINGS).payload;
        let start = 4 + 32 * index;
        let native_mode = bytes[start + 3] & 0x07;
        bytes[start + 3] = native_mode
            | direction
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
            let template = &include_bytes!("../../../assets/action_animation_group.bin")[72..84];
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
        let refs = &mut self.records.record_mut(OBJECT_REFS).payload;
        refs.clear();
        refs.extend_from_slice(&count.to_le_bytes());
        for target in targets {
            refs.extend_from_slice(&target.to_le_bytes());
        }
        if targets.len() != old_count {
            let default = &include_bytes!("../../../assets/action_animation_group.bin")[60..96];
            let settings = &mut self.records.record_mut(ANIMATION_SETTINGS).payload;
            settings.truncate(4);
            for _ in targets {
                settings.extend_from_slice(&default[4..36]);
            }
        }
        Ok(())
    }

    pub fn screen_anchor(&self) -> (i16, i16) {
        let payload = &self.records.record(ACTION_PAYLOAD).payload;
        (
            i16::from_le_bytes(payload[20..22].try_into().unwrap()),
            i16::from_le_bytes(payload[22..24].try_into().unwrap()),
        )
    }

    pub fn set_screen_anchor(&mut self, anchor: (i16, i16)) {
        self.records.record_mut(ACTION_PAYLOAD).payload[20..22]
            .copy_from_slice(&anchor.0.to_le_bytes());
        self.records.record_mut(ACTION_PAYLOAD).payload[22..24]
            .copy_from_slice(&anchor.1.to_le_bytes());
    }

    pub fn label_utf8(&self) -> Result<&str, String> {
        std::str::from_utf8(self.records.label_bytes())
            .map_err(|_| "Action button label is not UTF-8".into())
    }

    /// Modern action labels are UTF-8. The admitted 16-byte legacy header
    /// cohort stores its otherwise identical Animation labels in code page 936.
    pub fn label_text(&self) -> Result<String, String> {
        match self.label_utf8() {
            Ok(label) => Ok(label.to_owned()),
            Err(_) if self.records.record(OBJECT_HEADER).payload.len() == 16 => encoding_rs::GBK
                .decode_without_bom_handling_and_without_replacement(self.records.label_bytes())
                .map(std::borrow::Cow::into_owned)
                .ok_or_else(|| "Legacy Animation button label is not GBK".into()),
            Err(error) => Err(error),
        }
    }

    pub fn set_label(&mut self, label: &str) -> Result<(), String> {
        if self.label_text().is_ok_and(|current| current == label) {
            return Ok(());
        }
        self.records.set_label(
            label,
            include_bytes!("../../../assets/action_animation_group.bin"),
            LabelParityPolicy::MoveZeroSuffix,
        )
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.records.record_mut(OBJECT_HEADER).payload[2] = u8::from(hidden);
    }

    pub fn encode(&self) -> Vec<u8> {
        encode_records(&self.records.records)
    }
}

impl SoundButtonGroup {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let records = ActionRecords::parse(bytes, false)?;
        let group = Self { records };
        group.validate()?;
        Ok(group)
    }

    pub fn from_original_template_targets(
        targets: &[u32],
        label: &str,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        let mut records = ActionRecords::parse(
            include_bytes!("../../../assets/action_hide_show_two_points_group.bin"),
            false,
        )?;
        records.record_mut(ACTION_PAYLOAD).payload[12..20].fill(0);
        records.record_mut(ACTION_PAYLOAD).payload[12..14].copy_from_slice(&8_u16.to_le_bytes());
        let mut group = Self { records };
        group.set_targets(targets)?;
        group.records.set_label(
            label,
            include_bytes!("../../../assets/action_hide_show_two_points_group.bin"),
            LabelParityPolicy::MoveZeroSuffix,
        )?;
        group.set_screen_anchor(anchor);
        group.validate()?;
        Ok(group)
    }

    fn validate(&self) -> Result<(), String> {
        let header = &self.records.record(OBJECT_HEADER).payload;
        if header.len() != 28 || u16_at(header, 0) != 62 {
            return Err("Sound button needs a 28-byte object code 62 header".into());
        }
        let refs = &self.records.record(OBJECT_REFS).payload;
        let count = refs
            .len()
            .checked_sub(4)
            .filter(|length| length.is_multiple_of(4))
            .map(|length| length / 4);
        let Some(count @ 1..=2) = count else {
            return Err("Sound button needs one or two nonzero function targets".into());
        };
        if u32_at(refs, 0) as usize != count || self.target_ids().contains(&0) {
            return Err("Sound button needs one or two nonzero function targets".into());
        }
        let action = &self.records.record(ACTION_PAYLOAD).payload;
        if action.len() != 24
            || u32_at(action, 0) != ACTION_PAYLOAD
            || u32_at(action, 4) != 24
            || u32_at(action, 8) != 12
            || u16_at(action, 12) != 8
            || action[14..20] != [0; 6]
        {
            return Err("Unsupported Sound button action".into());
        }
        if self.records.record(BUTTON_AUX).payload.len() != 2
            || !self.records.record(OBJECT_END).payload.is_empty()
        {
            return Err("Sound button has invalid auxiliary or terminator record".into());
        }
        Ok(())
    }

    pub fn target_ids(&self) -> Vec<u32> {
        self.records.record(OBJECT_REFS).payload[4..]
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect()
    }

    pub fn set_targets(&mut self, targets: &[u32]) -> Result<(), String> {
        if !(1..=2).contains(&targets.len()) || targets.contains(&0) {
            return Err("Sound button needs one or two nonzero function targets".into());
        }
        let count = u32::try_from(targets.len()).map_err(|_| "Too many Sound targets")?;
        let payload = &mut self.records.record_mut(OBJECT_REFS).payload;
        payload.clear();
        payload.extend_from_slice(&count.to_le_bytes());
        for target in targets {
            payload.extend_from_slice(&target.to_le_bytes());
        }
        Ok(())
    }

    pub fn screen_anchor(&self) -> (i16, i16) {
        let payload = &self.records.record(ACTION_PAYLOAD).payload;
        (
            i16::from_le_bytes(payload[20..22].try_into().unwrap()),
            i16::from_le_bytes(payload[22..24].try_into().unwrap()),
        )
    }

    pub fn set_screen_anchor(&mut self, anchor: (i16, i16)) {
        self.records.record_mut(ACTION_PAYLOAD).payload[20..22]
            .copy_from_slice(&anchor.0.to_le_bytes());
        self.records.record_mut(ACTION_PAYLOAD).payload[22..24]
            .copy_from_slice(&anchor.1.to_le_bytes());
    }

    pub fn label_utf8(&self) -> Result<&str, String> {
        std::str::from_utf8(self.records.label_bytes())
            .map_err(|_| "Sound button label is not UTF-8".into())
    }

    pub fn set_label(&mut self, label: &str) -> Result<(), String> {
        self.records.set_label(
            label,
            include_bytes!("../../../assets/action_hide_show_two_points_group.bin"),
            LabelParityPolicy::DropOpaqueAlignment,
        )
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.records.record_mut(OBJECT_HEADER).payload[2] = u8::from(hidden);
    }

    pub fn encode(&self) -> Vec<u8> {
        encode_records(&self.records.records)
    }
}
