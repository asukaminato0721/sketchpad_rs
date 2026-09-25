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
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            direction: AnimationDirection::Bidirectional,
            speed: 24.,
            once: false,
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
        let mut group = Self::parse(include_bytes!("../assets/action_hide_show_group.bin"))?;
        group.set_single_target(target)?;
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
        if target == 0 || self.target_ids().len() != 1 {
            return Err("A single-target button needs one nonzero target ID".into());
        }
        self.records[1].payload[4..8].copy_from_slice(&target.to_le_bytes());
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
        let mut group = Self::parse(include_bytes!("../assets/action_animation_group.bin"))?;
        group.set_single_target(target)?;
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
        if refs.len() != 8 || u32_at(refs, 0) != 1 || u32_at(refs, 4) == 0 {
            return Err("Animation button needs one nonzero target".into());
        }
        let settings = &self.records[2].payload;
        // The settings payload starts after the 36-byte header group,
        // 16-byte target record, and its own 8-byte record header.
        let default_settings = &include_bytes!("../assets/action_animation_group.bin")[60..96];
        if settings.len() != 36
            || settings.iter().zip(default_settings).enumerate().any(
                |(index, (actual, expected))| !matches!(index, 7 | 8..=11) && actual != expected,
            )
            || self.settings().is_err()
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

    pub fn settings(&self) -> Result<AnimationSettings, String> {
        let bytes = &self.records[2].payload;
        if bytes.len() != 36 {
            return Err("Animation settings are truncated".into());
        }
        let flags = bytes[7];
        if flags & !0xb0 != 0 {
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
            speed: f32::from_le_bytes(bytes[8..12].try_into().unwrap()),
            once: flags & 0x80 != 0,
        };
        settings.validate()?;
        Ok(settings)
    }

    pub fn set_settings(&mut self, settings: AnimationSettings) -> Result<(), String> {
        settings.validate()?;
        let direction = match settings.direction {
            AnimationDirection::Forward => 0x00,
            AnimationDirection::Backward => 0x10,
            AnimationDirection::Bidirectional => 0x20,
            AnimationDirection::Random => 0x30,
        };
        let bytes = &mut self.records[2].payload;
        bytes[7] = direction | if settings.once { 0x80 } else { 0 };
        bytes[8..12].copy_from_slice(&settings.speed.to_le_bytes());
        Ok(())
    }

    pub fn set_single_target(&mut self, target: u32) -> Result<(), String> {
        if target == 0 {
            return Err("Animation button target must be nonzero".into());
        }
        self.records[1].payload[4..8].copy_from_slice(&target.to_le_bytes());
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
        if old_len % 2 != bytes.len() % 2 {
            return Err("Animation button label byte parity differs".into());
        }
        let suffix = payload[24 + old_len..].to_vec();
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
                },
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_backward_original.gsp")
                    .as_slice(),
                AnimationSettings {
                    direction: Backward,
                    speed: 24.,
                    once: false,
                },
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_random_original.gsp").as_slice(),
                AnimationSettings {
                    direction: Random,
                    speed: 24.,
                    once: false,
                },
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_slow_original.gsp").as_slice(),
                AnimationSettings {
                    direction: Bidirectional,
                    speed: 4.,
                    once: false,
                },
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_fast_original.gsp").as_slice(),
                AnimationSettings {
                    direction: Bidirectional,
                    speed: 80.,
                    once: false,
                },
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_forward_once_original.gsp")
                    .as_slice(),
                AnimationSettings {
                    direction: Forward,
                    speed: 24.,
                    once: true,
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
