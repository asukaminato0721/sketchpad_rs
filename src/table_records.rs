//! Bounded GSP4 support for a one-column snapshot table of the default t[1]
//! parameter. Unknown table layouts and values are refused before writing.

const TEMPLATE_DOCUMENT: &[u8] = include_bytes!("../tests/fixtures/table_parameter_tabulated.gsp");
const DEFAULT_PARAMETER: &[u8] = include_bytes!("../tests/fixtures/table_parameter_default.gsp");
const TWO_PARAMETERS: &[u8] = include_bytes!("../tests/fixtures/table_two_parameters_original.gsp");
const TWO_COLUMN_TABLE: &[u8] = include_bytes!("../tests/fixtures/table_two_columns_original.gsp");
const TWO_ROW_PAIR_TABLE: &[u8] =
    include_bytes!("../tests/fixtures/table_two_columns_two_rows_original.gsp");
const CALCULATION: &[u8] = include_bytes!("../tests/fixtures/table_calculate_2_plus_3.gsp");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableGroup {
    bytes: Vec<u8>,
}

#[derive(Clone, Copy)]
struct Record {
    kind: u32,
    start: usize,
    payload: usize,
    end: usize,
}

fn records(bytes: &[u8], start: usize) -> Result<Vec<Record>, String> {
    let mut at = start;
    let mut result = Vec::new();
    while at < bytes.len() {
        if at + 8 > bytes.len() {
            return Err("Truncated GSP record".into());
        }
        let len = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
        let payload = at + 8;
        let end = payload
            .checked_add(len + len % 2)
            .ok_or("GSP record length overflow")?;
        if end > bytes.len() {
            return Err("Truncated GSP payload".into());
        }
        result.push(Record {
            kind: u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap()),
            start: at,
            payload,
            end,
        });
        at = end;
    }
    Ok(result)
}

fn table_template() -> Vec<u8> {
    let rs = records(TEMPLATE_DOCUMENT, 4).unwrap();
    let start = rs.iter().filter(|r| r.kind == 0x07d0).nth(1).unwrap().start;
    let end = rs.iter().filter(|r| r.kind == 0x07d7).nth(1).unwrap().end;
    TEMPLATE_DOCUMENT[start..end].to_vec()
}

fn object_group(document: &[u8], ordinal: usize) -> Vec<u8> {
    let rs = records(document, 4).unwrap();
    let start = rs
        .iter()
        .filter(|r| r.kind == 0x07d0)
        .nth(ordinal - 1)
        .unwrap()
        .start;
    let end = rs
        .iter()
        .filter(|r| r.kind == 0x07d7)
        .nth(ordinal - 1)
        .unwrap()
        .end;
    document[start..end].to_vec()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterGroup {
    bytes: Vec<u8>,
}

fn decimal_words(value: f64) -> Result<Vec<u16>, String> {
    if !value.is_finite() || value.abs() > 99.99 {
        return Err("Parameter value must be within -99.99 to 99.99".into());
    }
    let cents = (value.abs() * 100.).round();
    if (value.abs() * 100. - cents).abs() > 1e-7 {
        return Err("Parameter value has more than two decimal places".into());
    }
    let cents = cents as u16;
    let whole = cents / 100;
    let fraction = cents % 100;
    let mut words = Vec::new();
    if value.is_sign_negative() && cents != 0 {
        words.push(4097);
    }
    if whole >= 10 {
        words.push(whole / 10);
    }
    words.push(whole % 10);
    if fraction != 0 {
        words.push(10);
        words.push(fraction / 10);
        if fraction % 10 != 0 {
            words.push(fraction % 10);
        }
    }
    Ok(words)
}

fn value_from_words(words: &[u16]) -> Result<f64, String> {
    if words.is_empty() || words.len() > 6 {
        return Err("Unsupported parameter digits".into());
    }
    let (negative, digits) = if words[0] == 4097 {
        (true, &words[1..])
    } else {
        (false, words)
    };
    let decimal = digits.iter().position(|word| *word == 10);
    let (whole, fraction) = match decimal {
        Some(index) => (&digits[..index], &digits[index + 1..]),
        None => (digits, &[][..]),
    };
    if !(1..=2).contains(&whole.len())
        || fraction.len() > 2
        || decimal.is_some() && fraction.is_empty()
        || whole.iter().chain(fraction).any(|digit| *digit > 9)
    {
        return Err("Unsupported parameter digits".into());
    }
    let whole_value = whole.iter().fold(0_u16, |value, digit| value * 10 + *digit);
    let fraction_value = fraction
        .iter()
        .fold(0_u16, |value, digit| value * 10 + *digit);
    let fraction_cents = if fraction.len() == 1 {
        fraction_value * 10
    } else {
        fraction_value
    };
    let value = (whole_value * 100 + fraction_cents) as f64 / 100.;
    let value = if negative { -value } else { value };
    if decimal_words(value)? != words {
        return Err("Noncanonical parameter digits".into());
    }
    Ok(value)
}

impl ParameterGroup {
    pub fn default_value() -> Self {
        Self {
            bytes: object_group(DEFAULT_PARAMETER, 1),
        }
    }

    pub fn second_default_value() -> Self {
        Self {
            bytes: object_group(TWO_PARAMETERS, 2),
        }
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let rs = records(bytes, 0)?;
        if rs.iter().map(|r| r.kind).collect::<Vec<_>>()
            != [0x07d0, 0x0907, 0x07d5, 0x07d8, 0x08a3, 0x07d7]
        {
            return Err("Unsupported parameter record sequence".into());
        }
        let payload_len = rs[1].end - rs[1].payload;
        if !(94..=104).contains(&payload_len) || payload_len % 2 != 0 {
            return Err("Unsupported parameter value length".into());
        }
        let word_count = (payload_len - 92) / 2;
        let payload = &bytes[rs[1].payload..rs[1].end];
        if u32::from_le_bytes(payload[36..40].try_into().unwrap()) as usize != payload_len - 32
            || u16::from_le_bytes(payload[44..46].try_into().unwrap()) as usize != word_count
        {
            return Err("Invalid parameter value header".into());
        }
        let words = (0..word_count)
            .map(|i| u16::from_le_bytes(payload[92 + i * 2..94 + i * 2].try_into().unwrap()))
            .collect::<Vec<_>>();
        value_from_words(&words)?;
        let templates = [
            object_group(DEFAULT_PARAMETER, 1),
            object_group(TWO_PARAMETERS, 2),
        ];
        let supported = templates.iter().any(|template| {
            let base = records(template, 0).unwrap();
            rs.iter().enumerate().all(|(record_index, actual)| {
                let expected = base[record_index];
                if record_index == 1 {
                    let actual_payload = &bytes[actual.payload..actual.end];
                    let expected_payload = &template[expected.payload..expected.end];
                    (0..92).all(|i| {
                        let mutable = (12..16).contains(&i)
                            || (22..24).contains(&i)
                            || (30..32).contains(&i)
                            || (36..40).contains(&i)
                            || (44..46).contains(&i);
                        mutable || actual_payload[i] == expected_payload[i]
                    })
                } else {
                    let actual_record = &bytes[actual.start..actual.end];
                    let expected_record = &template[expected.start..expected.end];
                    actual_record.len() == expected_record.len()
                        && actual_record.iter().zip(expected_record).enumerate().all(
                            |(i, (a, e))| {
                                record_index == 0 && i == 10
                                    || record_index == 4 && i >= 8
                                    || a == e
                            },
                        )
                }
            })
        });
        if !supported {
            return Err("Unsupported parameter variant".into());
        }
        let group = Self {
            bytes: bytes.to_vec(),
        };
        let at = group.anchor();
        let bbox = group.bbox();
        let width = i32::from(bbox.1.0) - i32::from(bbox.0.0);
        let height = i32::from(bbox.1.1) - i32::from(bbox.0.1);
        if bbox.0 != at || !(1..=2048).contains(&width) || !(1..=2048).contains(&height) {
            return Err("Parameter anchor and box disagree".into());
        }
        Ok(group)
    }

    pub fn name(&self) -> &str {
        let rs = records(&self.bytes, 0).unwrap();
        if self.bytes[rs[2].payload + 26] == b'2' {
            "t[2]"
        } else {
            "t[1]"
        }
    }
    pub fn value(&self) -> f64 {
        let rs = records(&self.bytes, 0).unwrap();
        let payload = &self.bytes[rs[1].payload..rs[1].end];
        let words = payload[92..]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()))
            .collect::<Vec<_>>();
        value_from_words(&words).unwrap()
    }

    pub fn set_value(&mut self, value: f64) -> Result<(), String> {
        let words = decimal_words(value)?;
        let old_value = self.value();
        let extra = |value: f64| i32::from(value.abs() >= 10.) + i32::from(value < 0.);
        let width_delta = 4 * (extra(value) - extra(old_value));
        let rs = records(&self.bytes, 0)?;
        let mut payload = self.bytes[rs[1].payload..rs[1].end].to_vec();
        payload.truncate(92);
        for word in &words {
            payload.extend_from_slice(&word.to_le_bytes());
        }
        let inner_length = (payload.len() - 32) as u32;
        payload[36..40].copy_from_slice(&inner_length.to_le_bytes());
        payload[44..46].copy_from_slice(&(words.len() as u16).to_le_bytes());
        let mut updated = self.bytes[..rs[1].start].to_vec();
        put_record(&mut updated, 0x0907, &payload);
        updated.extend_from_slice(&self.bytes[rs[1].end..]);
        let changed = records(&updated, 0)?;
        let right = changed[4].payload + 4;
        let old_right = i16::from_le_bytes(updated[right..right + 2].try_into().unwrap());
        let new_right = i16::try_from(i32::from(old_right) + width_delta)
            .map_err(|_| "Parameter box coordinate overflow")?;
        updated[right..right + 2].copy_from_slice(&new_right.to_le_bytes());
        Self::parse(&updated)?;
        self.bytes = updated;
        Ok(())
    }

    pub fn anchor(&self) -> (i16, i16) {
        let rs = records(&self.bytes, 0).unwrap();
        let at = rs[1].payload + 12;
        (
            i16::from_le_bytes(self.bytes[at..at + 2].try_into().unwrap()),
            i16::from_le_bytes(self.bytes[at + 2..at + 4].try_into().unwrap()),
        )
    }

    pub fn bbox(&self) -> ((i16, i16), (i16, i16)) {
        let rs = records(&self.bytes, 0).unwrap();
        let at = rs[4].payload;
        let word = |offset| {
            i16::from_le_bytes(self.bytes[at + offset..at + offset + 2].try_into().unwrap())
        };
        ((word(0), word(2)), (word(4), word(6)))
    }

    pub fn set_anchor(&mut self, anchor: (i16, i16)) -> Result<(), String> {
        let rs = records(&self.bytes, 0)?;
        let before = self.anchor();
        let delta = (
            i32::from(anchor.0) - i32::from(before.0),
            i32::from(anchor.1) - i32::from(before.1),
        );
        let bbox = rs[4].payload;
        let mut updates = Vec::new();
        for (index, difference) in [(0, delta.0), (2, delta.1), (4, delta.0), (6, delta.1)] {
            let old = i16::from_le_bytes(
                self.bytes[bbox + index..bbox + index + 2]
                    .try_into()
                    .unwrap(),
            );
            let new = i16::try_from(i32::from(old) + difference)
                .map_err(|_| "Parameter box coordinate overflow")?;
            updates.push((index, new));
        }
        let at = rs[1].payload + 12;
        self.bytes[at..at + 2].copy_from_slice(&anchor.0.to_le_bytes());
        self.bytes[at + 2..at + 4].copy_from_slice(&anchor.1.to_le_bytes());
        for (index, new) in updates {
            self.bytes[bbox + index..bbox + index + 2].copy_from_slice(&new.to_le_bytes());
        }
        Ok(())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

pub fn append_default_parameter(blank: &[u8], anchor: (i16, i16)) -> Result<Vec<u8>, String> {
    append_to_blank(
        blank,
        ParameterGroup::default_value()
            .with_anchor(anchor)?
            .as_bytes(),
    )
}

impl ParameterGroup {
    fn with_anchor(mut self, anchor: (i16, i16)) -> Result<Self, String> {
        self.set_anchor(anchor)?;
        Ok(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalculationGroup {
    bytes: Vec<u8>,
}

impl CalculationGroup {
    pub fn literal_two_plus_three() -> Self {
        Self {
            bytes: object_group(CALCULATION, 2),
        }
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let rs = records(bytes, 0)?;
        if rs.iter().map(|r| r.kind).collect::<Vec<_>>() != [0x07d0, 0x0907, 0x07d8, 0x08a3, 0x07d7]
        {
            return Err("Unsupported calculation record sequence".into());
        }
        let template = object_group(CALCULATION, 2);
        if bytes.len() != template.len() {
            return Err("Unsupported calculation byte length".into());
        }
        for (i, (actual, expected)) in bytes.iter().zip(&template).enumerate() {
            let mutable = i == 10
                || (rs[1].payload + 12..rs[1].payload + 16).contains(&i)
                || (rs[3].payload..rs[3].end).contains(&i);
            if !mutable && actual != expected {
                return Err("Unsupported calculation variant".into());
            }
        }
        let result = Self {
            bytes: bytes.to_vec(),
        };
        let anchor = result.anchor();
        let bbox = rs[3].payload;
        let word = |offset| {
            i16::from_le_bytes(
                result.bytes[bbox + offset..bbox + offset + 2]
                    .try_into()
                    .unwrap(),
            )
        };
        if (word(0), word(2)) != anchor
            || i32::from(word(4)) - i32::from(word(0)) != 114
            || i32::from(word(6)) - i32::from(word(2)) != 26
        {
            return Err("Calculation anchor and box disagree".into());
        }
        Ok(result)
    }
    pub fn expression(&self) -> &str {
        "2 + 3"
    }
    pub fn value(&self) -> f64 {
        5.0
    }
    pub fn anchor(&self) -> (i16, i16) {
        let rs = records(&self.bytes, 0).unwrap();
        let at = rs[1].payload + 12;
        (
            i16::from_le_bytes(self.bytes[at..at + 2].try_into().unwrap()),
            i16::from_le_bytes(self.bytes[at + 2..at + 4].try_into().unwrap()),
        )
    }
    pub fn set_anchor(&mut self, anchor: (i16, i16)) -> Result<(), String> {
        let rs = records(&self.bytes, 0)?;
        let before = self.anchor();
        let bbox = rs[3].payload;
        let delta = (
            i32::from(anchor.0) - i32::from(before.0),
            i32::from(anchor.1) - i32::from(before.1),
        );
        let mut updates = Vec::new();
        for (index, difference) in [(0, delta.0), (2, delta.1), (4, delta.0), (6, delta.1)] {
            let old = i16::from_le_bytes(
                self.bytes[bbox + index..bbox + index + 2]
                    .try_into()
                    .unwrap(),
            );
            updates.push((
                index,
                i16::try_from(i32::from(old) + difference)
                    .map_err(|_| "Calculation box coordinate overflow")?,
            ));
        }
        let at = rs[1].payload + 12;
        self.bytes[at..at + 2].copy_from_slice(&anchor.0.to_le_bytes());
        self.bytes[at + 2..at + 4].copy_from_slice(&anchor.1.to_le_bytes());
        for (index, value) in updates {
            self.bytes[bbox + index..bbox + index + 2].copy_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

pub fn append_literal_sum(blank: &[u8]) -> Result<Vec<u8>, String> {
    append_to_blank(blank, CalculationGroup::literal_two_plus_three().as_bytes())
}

fn append_to_blank(blank: &[u8], group: &[u8]) -> Result<Vec<u8>, String> {
    if blank.get(..4) != Some(b"GSP4") {
        return Err("Expected GSP4".into());
    }
    let rs = records(blank, 4)?;
    let preamble = rs
        .iter()
        .find(|r| r.kind == 0x03e8)
        .ok_or("Missing page preamble")?;
    if rs.iter().filter(|r| r.kind == 0x03e8).count() != 1
        || rs.iter().any(|r| r.kind == 0x07d0)
        || blank.get(preamble.payload + 24..preamble.payload + 28) != Some(&0_u32.to_le_bytes())
    {
        return Err("Expected blank one-page document".into());
    }
    let trailer = rs
        .iter()
        .find(|r| r.kind == 0x03e9)
        .ok_or("Missing page trailer")?;
    let mut output = blank.to_vec();
    output[preamble.payload + 24..preamble.payload + 28].copy_from_slice(&1_u32.to_le_bytes());
    let mut insertion = group.to_vec();
    put_record(&mut insertion, 0x2328, &1_u32.to_le_bytes());
    output.splice(trailer.start..trailer.start, insertion);
    records(&output, 4)?;
    Ok(output)
}

const SNAPSHOT_ROW: &[u8] = b"0<H<T4x1.00>>";

fn put_record(out: &mut Vec<u8>, kind: u32, payload: &[u8]) {
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(payload);
    if payload.len() % 2 != 0 {
        out.push(0);
    }
}

fn canonical_table(source: u32, anchor: (i16, i16), rows: usize) -> Result<Vec<u8>, String> {
    if source == 0 || !(1..=256).contains(&rows) {
        return Err("Invalid table source or row count".into());
    }
    let base = table_template();
    let rs = records(&base, 0)?;
    let mut layout = base[rs[2].payload..rs[2].end].to_vec();
    layout[12..14].copy_from_slice(&anchor.0.to_le_bytes());
    layout[14..16].copy_from_slice(&anchor.1.to_le_bytes());
    layout[16..18].copy_from_slice(&((rows + 2) as u16).to_le_bytes());
    for _ in 1..rows {
        layout.extend_from_slice(SNAPSHOT_ROW);
    }
    while layout.len() % 4 != 0 {
        layout.push(0);
    }
    let layout_len = layout.len() as u32;
    layout[4..8].copy_from_slice(&layout_len.to_le_bytes());
    let mut refs = base[rs[1].payload..rs[1].end].to_vec();
    refs[4..8].copy_from_slice(&source.to_le_bytes());
    let mut group = Vec::new();
    put_record(&mut group, 0x07d0, &base[rs[0].payload..rs[0].end]);
    put_record(&mut group, 0x07d2, &refs);
    put_record(&mut group, 0x090d, &layout);
    put_record(&mut group, 0x07d8, &base[rs[3].payload..rs[3].end]);
    put_record(&mut group, 0x07d7, &[]);
    Ok(group)
}

impl TableGroup {
    pub fn default_snapshot() -> Self {
        Self {
            bytes: table_template(),
        }
    }

    pub fn pair_snapshot() -> Self {
        Self {
            bytes: object_group(TWO_COLUMN_TABLE, 3),
        }
    }

    pub fn pair_snapshot_rows(rows: usize, anchor: (i16, i16)) -> Result<Self, String> {
        let mut group = match rows {
            1 => Self::pair_snapshot(),
            2 => Self {
                bytes: object_group(TWO_ROW_PAIR_TABLE, 3),
            },
            _ => {
                return Err(
                    "Only one or two rows of the default parameter pair are supported".into(),
                );
            }
        };
        group.set_anchor(anchor);
        Ok(group)
    }

    pub fn parse_pair(bytes: &[u8]) -> Result<Self, String> {
        let template = match bytes.len() {
            182 => Self::pair_snapshot(),
            206 => Self {
                bytes: object_group(TWO_ROW_PAIR_TABLE, 3),
            },
            _ => return Err("Unsupported two-column table length".into()),
        };
        let rs = records(bytes, 0)?;
        if rs.iter().map(|r| r.kind).collect::<Vec<_>>() != [0x07d0, 0x07d2, 0x090d, 0x07d8, 0x07d7]
            || bytes.len() != template.bytes.len()
            || rs[1].end - rs[1].payload != 12
            || rs[2].end - rs[2].payload != if bytes.len() == 182 { 100 } else { 124 }
        {
            return Err("Unsupported two-column table layout".into());
        }
        for (i, (actual, expected)) in bytes.iter().zip(&template.bytes).enumerate() {
            let mutable = i == 10
                || (rs[2].payload + 12..rs[2].payload + 16).contains(&i)
                || (rs[2].end - 3..rs[2].end).contains(&i);
            let row_value =
                (143..147).contains(&i) || bytes.len() == 206 && (168..172).contains(&i);
            if !mutable && !row_value && actual != expected {
                return Err("Unsupported two-column table variant".into());
            }
        }
        for at in if bytes.len() == 206 {
            &[143, 168][..]
        } else {
            &[143][..]
        } {
            let value = &bytes[*at..*at + 4];
            if !value[0].is_ascii_digit()
                || value[1] != b'.'
                || !value[2].is_ascii_digit()
                || !value[3].is_ascii_digit()
            {
                return Err("Unsupported pair table row value".into());
            }
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    pub fn pair_source_ids(&self) -> Option<[u32; 2]> {
        let rs = records(&self.bytes, 0).ok()?;
        (rs[1].end - rs[1].payload == 12).then(|| {
            [
                u32::from_le_bytes(
                    self.bytes[rs[1].payload + 4..rs[1].payload + 8]
                        .try_into()
                        .unwrap(),
                ),
                u32::from_le_bytes(
                    self.bytes[rs[1].payload + 8..rs[1].payload + 12]
                        .try_into()
                        .unwrap(),
                ),
            ]
        })
    }

    pub fn pair_values(&self) -> Option<Vec<[f64; 2]>> {
        self.pair_source_ids()?;
        let positions: &[usize] = if self.bytes.len() == 206 {
            &[143, 168]
        } else {
            &[143]
        };
        positions
            .iter()
            .map(|&i| {
                let text = std::str::from_utf8(&self.bytes[i..i + 4]).ok()?;
                Some([text.parse::<f64>().ok()?, 1.])
            })
            .collect()
    }

    pub fn set_pair_row_value(&mut self, row: usize, value: f64) -> Result<(), String> {
        if !(0.0..10.0).contains(&value)
            || decimal_words(value).is_err()
            || self.pair_source_ids() != Some([2, 1])
        {
            return Err("Unsupported pair table value".into());
        }
        let offset = match (self.bytes.len(), row) {
            (182, 0) | (206, 0) => 143,
            (206, 1) => 168,
            _ => return Err("Invalid pair table row".into()),
        };
        self.bytes[offset..offset + 4].copy_from_slice(format!("{value:.2}").as_bytes());
        Self::parse_pair(&self.bytes)?;
        Ok(())
    }

    pub fn set_last_pair_value(&mut self, value: f64) -> Result<(), String> {
        self.set_pair_row_value(self.rows() - 1, value)
    }

    pub fn for_source(source: u32, anchor: (i16, i16)) -> Result<Self, String> {
        Ok(Self {
            bytes: canonical_table(source, anchor, 1)?,
        })
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let rs = records(bytes, 0)?;
        if rs.iter().map(|r| r.kind).collect::<Vec<_>>() != [0x07d0, 0x07d2, 0x090d, 0x07d8, 0x07d7]
        {
            return Err("Unsupported table record sequence".into());
        }
        if rs[1].end - rs[1].payload != 8 || rs[2].end - rs[2].payload < 64 {
            return Err("Unsupported table payload length".into());
        }
        let source = u32::from_le_bytes(
            bytes[rs[1].payload + 4..rs[1].payload + 8]
                .try_into()
                .unwrap(),
        );
        let at = rs[2].payload;
        let anchor = (
            i16::from_le_bytes(bytes[at + 12..at + 14].try_into().unwrap()),
            i16::from_le_bytes(bytes[at + 14..at + 16].try_into().unwrap()),
        );
        let row_count = u16::from_le_bytes(bytes[at + 16..at + 18].try_into().unwrap()) as usize;
        let rows = row_count.checked_sub(2).ok_or("Invalid table row count")?;
        let mut normalized = bytes.to_vec();
        normalized[10] &= !1;
        // A two-row table saved after Plot Points has an observed `t` in
        // the penultimate padding byte of 0x090d. It is outside the row text.
        if rows == 2 && normalized[rs[2].end - 2] == b't' && normalized[rs[2].end - 1] == 0 {
            normalized[rs[2].end - 2] = 0;
        }
        if normalized != canonical_table(source, anchor, rows)? {
            return Err("Unsupported one-column table variant".into());
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    pub fn rows(&self) -> usize {
        let rs = records(&self.bytes, 0).unwrap();
        u16::from_le_bytes(
            self.bytes[rs[2].payload + 16..rs[2].payload + 18]
                .try_into()
                .unwrap(),
        ) as usize
            - 2
    }

    pub fn add_default_snapshot(&mut self) -> Result<(), String> {
        self.bytes = canonical_table(self.source_id(), self.anchor(), self.rows() + 1)?;
        Ok(())
    }

    pub fn remove_last_snapshot(&mut self) -> Result<(), String> {
        let rows = self.rows().checked_sub(1).ok_or("Table has no rows")?;
        self.bytes = canonical_table(self.source_id(), self.anchor(), rows)?;
        Ok(())
    }

    pub fn source_id(&self) -> u32 {
        let rs = records(&self.bytes, 0).unwrap();
        u32::from_le_bytes(
            self.bytes[rs[1].payload + 4..rs[1].payload + 8]
                .try_into()
                .unwrap(),
        )
    }

    pub fn anchor(&self) -> (i16, i16) {
        let rs = records(&self.bytes, 0).unwrap();
        let at = rs[2].payload + 12;
        (
            i16::from_le_bytes(self.bytes[at..at + 2].try_into().unwrap()),
            i16::from_le_bytes(self.bytes[at + 2..at + 4].try_into().unwrap()),
        )
    }

    pub fn set_anchor(&mut self, anchor: (i16, i16)) {
        let rs = records(&self.bytes, 0).unwrap();
        let at = rs[2].payload + 12;
        self.bytes[at..at + 2].copy_from_slice(&anchor.0.to_le_bytes());
        self.bytes[at + 2..at + 4].copy_from_slice(&anchor.1.to_le_bytes());
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Append a one-row table to the original single default t[1]=1.00 parameter.
/// The source value and name are verified so the table's cached row is accurate.
pub fn append_default_parameter_table(
    document: &[u8],
    anchor: (i16, i16),
) -> Result<Vec<u8>, String> {
    if document.get(..4) != Some(b"GSP4") {
        return Err("Expected GSP4".into());
    }
    let rs = records(document, 4)?;
    let base = records(DEFAULT_PARAMETER, 4)?;
    let preamble = rs
        .iter()
        .find(|r| r.kind == 0x03e8)
        .ok_or("Missing page preamble")?;
    if rs.iter().filter(|r| r.kind == 0x03e8).count() != 1
        || rs.iter().filter(|r| r.kind == 0x07d0).count() != 1
        || document.get(preamble.payload + 24..preamble.payload + 28) != Some(&1_u32.to_le_bytes())
    {
        return Err("Expected one-page one-parameter document".into());
    }
    let start = rs.iter().position(|r| r.kind == 0x07d0).unwrap();
    let end = rs[start..]
        .iter()
        .position(|r| r.kind == 0x07d7)
        .map(|n| start + n)
        .ok_or("Unterminated parameter")?;
    let param = &rs[start..=end];
    if param.iter().map(|r| r.kind).collect::<Vec<_>>()
        != [0x07d0, 0x0907, 0x07d5, 0x07d8, 0x08a3, 0x07d7]
    {
        return Err("Unsupported parameter layout".into());
    }
    let baseline_start = base.iter().position(|r| r.kind == 0x07d0).unwrap();
    let baseline = &base[baseline_start..baseline_start + param.len()];
    let actual = &document[param[1].payload..param[1].end];
    let expected = &DEFAULT_PARAMETER[baseline[1].payload..baseline[1].end];
    if actual.len() != expected.len()
        || actual
            .iter()
            .zip(expected)
            .enumerate()
            .any(|(i, (a, b))| !matches!(i, 22 | 23 | 30 | 31) && a != b)
        || document.get(param[2].payload + 22..param[2].payload + 28) != Some(&b"\x04\0t[1]"[..])
    {
        return Err("Only the original t[1] = 1.00 parameter is supported".into());
    }
    let selection = rs
        .iter()
        .find(|r| r.kind == 0x2328 && r.start > param[5].start)
        .ok_or("Missing parameter selection record")?;
    if selection.end - selection.payload != 4
        || document[selection.payload..selection.end] != 1_u32.to_le_bytes()
    {
        return Err("Unsupported selection record".into());
    }
    let mut table = TableGroup::default_snapshot();
    table.set_anchor(anchor);
    let mut output = document.to_vec();
    output[preamble.payload + 24..preamble.payload + 28].copy_from_slice(&2_u32.to_le_bytes());
    output[selection.payload..selection.end].copy_from_slice(&2_u32.to_le_bytes());
    output.splice(
        selection.start..selection.start,
        table.as_bytes().iter().copied(),
    );
    records(&output, 4)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_table_roundtrips_and_anchor_moves() {
        let original = table_template();
        let mut table = TableGroup::parse(&original).unwrap();
        assert_eq!(table.source_id(), 1);
        assert_eq!(table.anchor(), (10, 38));
        table.set_anchor((100, 116));
        TableGroup::parse(table.as_bytes()).unwrap();
        let dragged = include_bytes!("../tests/fixtures/table_table_dragged.gsp");
        let rs = records(dragged, 4).unwrap();
        let start = rs.iter().filter(|r| r.kind == 0x07d0).nth(1).unwrap().start;
        let end = rs.iter().filter(|r| r.kind == 0x07d7).nth(1).unwrap().end;
        assert_eq!(table.as_bytes(), &dragged[start..end]);
    }

    #[test]
    fn generated_table_has_valid_refs_and_rejects_other_parameter_values() {
        let source = include_bytes!("../tests/fixtures/table_parameter_default.gsp");
        let output = append_default_parameter_table(source, (100, 116)).unwrap();
        let rs = records(&output, 4).unwrap();
        let start = rs.iter().filter(|r| r.kind == 0x07d0).nth(1).unwrap().start;
        let end = rs.iter().filter(|r| r.kind == 0x07d7).nth(1).unwrap().end;
        assert_eq!(
            TableGroup::parse(&output[start..end]).unwrap().anchor(),
            (100, 116)
        );
        assert!(
            append_default_parameter_table(
                include_bytes!("../tests/fixtures/table_parameter_custom.gsp"),
                (10, 38)
            )
            .is_err()
        );
    }

    #[test]
    fn second_snapshot_matches_original_saved_table() {
        let mut table = TableGroup::default_snapshot();
        table.set_anchor((100, 116));
        table.add_default_snapshot().unwrap();
        assert_eq!(table.rows(), 2);
        let original = include_bytes!("../tests/fixtures/table_parameter_two_rows.gsp");
        let rs = records(original, 4).unwrap();
        let start = rs.iter().filter(|r| r.kind == 0x07d0).nth(1).unwrap().start;
        let end = rs.iter().filter(|r| r.kind == 0x07d7).nth(1).unwrap().end;
        assert_eq!(table.as_bytes(), &original[start..end]);
        TableGroup::parse(table.as_bytes()).unwrap();
        table.remove_last_snapshot().unwrap();
        assert_eq!(table.rows(), 1);
    }

    #[test]
    fn parameter_move_matches_original_and_blank_insertion_is_bounded() {
        let mut parameter = ParameterGroup::default_value();
        assert_eq!(parameter.name(), "t[1]");
        assert_eq!(parameter.value(), 1.0);
        ParameterGroup::parse(parameter.as_bytes()).unwrap();
        parameter.set_anchor((140, 95)).unwrap();
        let dragged = include_bytes!("../tests/fixtures/table_parameter_dragged.gsp");
        assert_eq!(parameter.as_bytes(), object_group(dragged, 1));
        let blank = include_bytes!("../tests/fixtures/text_object_blank.gsp");
        let output = append_default_parameter(blank, (140, 95)).unwrap();
        assert_eq!(object_group(&output, 1), parameter.as_bytes());
        assert!(append_default_parameter(&output, (10, 5)).is_err());
    }

    #[test]
    fn literal_calculation_template_is_bounded() {
        let calculation = CalculationGroup::literal_two_plus_three();
        CalculationGroup::parse(calculation.as_bytes()).unwrap();
        assert_eq!(calculation.expression(), "2 + 3");
        assert_eq!(calculation.value(), 5.0);
        let blank = include_bytes!("../tests/fixtures/text_object_blank.gsp");
        let output = append_literal_sum(blank).unwrap();
        assert_eq!(object_group(&output, 1), calculation.as_bytes());
    }
}
