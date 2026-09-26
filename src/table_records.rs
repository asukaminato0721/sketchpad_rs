//! Bounded GSP4 support for numeric parameters and verified snapshot tables.
//! Unknown layouts and values are refused before writing.

const TEMPLATE_DOCUMENT: &[u8] = include_bytes!("../tests/fixtures/table_parameter_tabulated.gsp");
const DEFAULT_PARAMETER: &[u8] = include_bytes!("../tests/fixtures/table_parameter_default.gsp");
const TWO_PARAMETERS: &[u8] = include_bytes!("../tests/fixtures/table_two_parameters_original.gsp");
const NAMED_PARAMETER: &[u8] =
    include_bytes!("../tests/fixtures/table_parameter_speed_original.gsp");
const TWO_COLUMN_TABLE: &[u8] = include_bytes!("../tests/fixtures/table_two_columns_original.gsp");
#[cfg(test)]
const TWO_ROW_PAIR_TABLE: &[u8] =
    include_bytes!("../tests/fixtures/table_two_columns_two_rows_original.gsp");
#[cfg(test)]
const THREE_ROW_PAIR_TABLE: &[u8] =
    include_bytes!("../tests/fixtures/table_pair_three_rows_distinct_original.gsp");
#[cfg(test)]
const FOUR_ROW_PAIR_TABLE: &[u8] =
    include_bytes!("../tests/fixtures/table_pair_four_rows_original.gsp");
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
    if words.is_empty() || words.len() > 32 {
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
    if !(1..=6).contains(&whole.len())
        || fraction.len() > 30
        || decimal.is_some() && fraction.is_empty()
        || whole.iter().chain(fraction).any(|digit| *digit > 9)
        || whole.len() > 1 && whole[0] == 0
    {
        return Err("Unsupported parameter digits".into());
    }
    let whole_value = whole
        .iter()
        .fold(0_u32, |value, digit| value * 10 + u32::from(*digit));
    let mut place = 1.;
    let mut value = f64::from(whole_value);
    for digit in fraction {
        place /= 10.;
        value += f64::from(*digit) * place;
    }
    let value = if negative { -value } else { value };
    value
        .is_finite()
        .then_some(value)
        .ok_or("Invalid parameter value".into())
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

    pub fn named(name: &str, value: f64) -> Result<Self, String> {
        let valid = (1..=32).contains(&name.len())
            && name.as_bytes()[0].is_ascii_alphabetic()
            && name
                .bytes()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == b'_');
        if !valid {
            return Err("Name must contain 1–32 ASCII letters, digits, or underscores and begin with a letter".into());
        }
        let mut group = Self {
            bytes: object_group(NAMED_PARAMETER, 1),
        };
        group.set_value(value)?;
        let rs = records(&group.bytes, 0)?;
        let named = object_group(NAMED_PARAMETER, 1);
        let named_records = records(&named, 0)?;
        let mut label = named[named_records[2].payload..named_records[2].payload + 22].to_vec();
        label.extend_from_slice(&(name.len() as u16).to_le_bytes());
        label.extend_from_slice(name.as_bytes());
        label.push(0);
        if name.len() % 2 == 0 {
            label.push(0);
        }
        let mut updated = group.bytes[..rs[2].start].to_vec();
        put_record(&mut updated, 0x07d5, &label);
        updated.extend_from_slice(&group.bytes[rs[2].end..]);
        let changed = records(&updated, 0)?;
        let bbox = changed[4].payload;
        let left = i16::from_le_bytes(updated[bbox..bbox + 2].try_into().unwrap());
        let letter_width = name
            .bytes()
            .map(|ch| match ch.to_ascii_lowercase() {
                b't' | b'i' | b'l' => 4,
                b's' => 7,
                _ => 8,
            })
            .sum::<i16>();
        let value_extra = 4 * (i16::from(value.abs() >= 10.) + i16::from(value < 0.));
        let width = 80 + letter_width + if group.precision() == 2 { 10 } else { 0 } + value_extra;
        updated[bbox + 4..bbox + 6].copy_from_slice(&(left + width).to_le_bytes());
        Self::parse(&updated)
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let rs = records(bytes, 0)?;
        if rs.iter().map(|r| r.kind).collect::<Vec<_>>()
            != [0x07d0, 0x0907, 0x07d5, 0x07d8, 0x08a3, 0x07d7]
        {
            return Err("Unsupported parameter record sequence".into());
        }
        let payload_len = rs[1].end - rs[1].payload;
        if !(94..=156).contains(&payload_len) || payload_len % 2 != 0 {
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
        let precision = u16::from_le_bytes(payload[28..30].try_into().unwrap());
        let step = f64::from_le_bytes(payload[52..60].try_into().unwrap());
        if !(precision == 1 && step == 0.1 || precision == 2 && step == 0.01) {
            return Err("Unsupported parameter precision".into());
        }
        let templates = [
            object_group(DEFAULT_PARAMETER, 1),
            object_group(TWO_PARAMETERS, 2),
            object_group(NAMED_PARAMETER, 1),
        ];
        let supported = templates.iter().any(|template| {
            let base = records(template, 0).unwrap();
            rs.iter().enumerate().all(|(record_index, actual)| {
                let expected = base[record_index];
                if record_index == 1 {
                    let actual_payload = &bytes[actual.payload..actual.end];
                    let expected_payload = &template[expected.payload..expected.end];
                    (0..92).all(|i| {
                        let animated_cache = actual_payload[80..88] == [0; 8]
                            && expected_payload[80..88] == 1_f64.to_le_bytes();
                        let mutable = (12..16).contains(&i)
                            || (22..24).contains(&i)
                            || (28..32).contains(&i)
                            || (36..40).contains(&i)
                            || (44..46).contains(&i)
                            || (52..60).contains(&i)
                            || animated_cache && (80..88).contains(&i);
                        mutable || actual_payload[i] == expected_payload[i]
                    })
                } else if record_index == 2 {
                    let actual_payload = &bytes[actual.payload..actual.end];
                    if actual_payload.len() < 26 {
                        false
                    } else if u32::from_le_bytes(actual_payload[..4].try_into().unwrap()) == 6 {
                        let expected_record = &template[expected.start..expected.end];
                        &bytes[actual.start..actual.end] == expected_record
                    } else {
                        let named = object_group(NAMED_PARAMETER, 1);
                        let label_records = records(&named, 0).unwrap();
                        let expected_payload =
                            &named[label_records[2].payload..label_records[2].end];
                        actual_payload[..22] == expected_payload[..22] && {
                            let name_len =
                                u16::from_le_bytes(actual_payload[22..24].try_into().unwrap())
                                    as usize;
                            let name = actual_payload.get(24..24 + name_len);
                            name_len > 0
                                && name_len <= 32
                                && actual_payload.len() == 24 + name_len + 1 + ((name_len + 1) % 2)
                                && actual_payload[24 + name_len..].iter().all(|ch| *ch == 0)
                                && name.is_some_and(|name| {
                                    name[0].is_ascii_alphabetic()
                                        && name
                                            .iter()
                                            .all(|ch| ch.is_ascii_alphanumeric() || *ch == b'_')
                                })
                        }
                    }
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
        let payload = rs[2].payload;
        let len =
            u16::from_le_bytes(self.bytes[payload + 22..payload + 24].try_into().unwrap()) as usize;
        std::str::from_utf8(&self.bytes[payload + 24..payload + 24 + len]).unwrap()
    }

    pub fn precision(&self) -> usize {
        let rs = records(&self.bytes, 0).unwrap();
        u16::from_le_bytes(
            self.bytes[rs[1].payload + 28..rs[1].payload + 30]
                .try_into()
                .unwrap(),
        ) as usize
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

    pub fn animation_domain(&self) -> (f64, f64) {
        let rs = records(&self.bytes, 0).unwrap();
        let payload = &self.bytes[rs[1].payload..rs[1].end];
        (
            f64::from_le_bytes(payload[60..68].try_into().unwrap()),
            f64::from_le_bytes(payload[68..76].try_into().unwrap()),
        )
    }

    pub fn set_value(&mut self, value: f64) -> Result<(), String> {
        let words = decimal_words(value)?;
        let promote_precision =
            self.precision() == 1 && (value * 10. - (value * 10.).round()).abs() > 1e-7;
        let old_value = self.value();
        let extra = |value: f64| i32::from(value.abs() >= 10.) + i32::from(value < 0.);
        let width_delta =
            4 * (extra(value) - extra(old_value)) + if promote_precision { 10 } else { 0 };
        let rs = records(&self.bytes, 0)?;
        let mut payload = self.bytes[rs[1].payload..rs[1].end].to_vec();
        payload.truncate(92);
        if promote_precision {
            payload[28..30].copy_from_slice(&2_u16.to_le_bytes());
            payload[52..60].copy_from_slice(&0.01_f64.to_le_bytes());
        }
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

fn put_record(out: &mut Vec<u8>, kind: u32, payload: &[u8]) {
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(payload);
    if payload.len() % 2 != 0 {
        out.push(0);
    }
}

fn table_value_text(value: f64) -> Result<String, String> {
    decimal_words(value)?;
    let rendered = if value < 0.0 {
        format!("–{:.2}", -value)
    } else {
        format!("{value:.2}")
    };
    Ok(format!("T{}x{rendered}>>", rendered.len()))
}

fn take_table_value(text: &str) -> Option<(f64, &str)> {
    let rest = text.strip_prefix('T')?;
    let (width, following) = rest.split_once('x')?;
    let width = width.parse::<usize>().ok()?;
    if !(4..=8).contains(&width) {
        return None;
    }
    let rendered = following.get(..width)?;
    let rest = following.get(width..)?.strip_prefix(">>")?;
    let ascii = rendered
        .strip_prefix('–')
        .map_or_else(|| rendered.to_owned(), |absolute| format!("-{absolute}"));
    let value = ascii.parse::<f64>().ok()?;
    let consumed = &text[..text.len() - rest.len()];
    (table_value_text(value).ok()?.as_str() == consumed).then_some((value, rest))
}

fn canonical_table_values(
    source: u32,
    anchor: (i16, i16),
    values: &[f64],
) -> Result<Vec<u8>, String> {
    if source == 0 || !(1..=256).contains(&values.len()) {
        return Err("Invalid table source or row count".into());
    }
    let base = table_template();
    let rs = records(&base, 0)?;
    let mut layout = base[rs[2].payload..rs[2].payload + 51].to_vec();
    layout[12..14].copy_from_slice(&anchor.0.to_le_bytes());
    layout[14..16].copy_from_slice(&anchor.1.to_le_bytes());
    layout[16..18].copy_from_slice(&((values.len() + 2) as u16).to_le_bytes());
    for &value in values {
        layout.extend_from_slice(format!("0<H<{}", table_value_text(value)?).as_bytes());
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

fn default_table_values(layout: &[u8], rows: usize) -> Option<Vec<f64>> {
    let text = layout.get(51..)?;
    let used = text
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(text.len());
    if text.len() - used > 3 {
        return None;
    }
    let mut rest = std::str::from_utf8(&text[..used]).ok()?;
    let mut values = Vec::with_capacity(rows);
    for _ in 0..rows {
        let (value, following) = take_table_value(rest.strip_prefix("0<H<")?)?;
        rest = following;
        values.push(value);
    }
    rest.is_empty().then_some(values)
}

fn pair_table_values(layout: &[u8], rows: usize) -> Option<Vec<[f64; 2]>> {
    for padding in 0..=3 {
        let Some(text) = layout.get(72..layout.len().checked_sub(padding)?) else {
            continue;
        };
        let Ok(mut rest) = std::str::from_utf8(text) else {
            continue;
        };
        let mut values = Vec::with_capacity(rows);
        let mut valid = true;
        for _ in 0..rows {
            let Some((x, following)) = rest.strip_prefix("0<H<").and_then(take_table_value) else {
                valid = false;
                break;
            };
            let Some((y, following)) = following.strip_prefix("<H<").and_then(take_table_value)
            else {
                valid = false;
                break;
            };
            values.push([x, y]);
            rest = following;
        }
        if valid && rest.is_empty() {
            return Some(values);
        }
    }
    None
}

fn canonical_pair_table(
    values: &[[f64; 2]],
    anchor: (i16, i16),
    sources: [u32; 2],
) -> Result<Vec<u8>, String> {
    if !(1..=256).contains(&values.len()) {
        return Err("Pair table requires one to 256 rows".into());
    }
    if sources[0] == 0 || sources[1] == 0 || sources[0] == sources[1] {
        return Err("Invalid pair table sources".into());
    }
    let base = object_group(TWO_COLUMN_TABLE, 3);
    let rs = records(&base, 0)?;
    let mut layout = base[rs[2].payload..rs[2].payload + 72].to_vec();
    layout[12..14].copy_from_slice(&anchor.0.to_le_bytes());
    layout[14..16].copy_from_slice(&anchor.1.to_le_bytes());
    layout[16..18].copy_from_slice(&((values.len() + 2) as u16).to_le_bytes());
    for &[x, y] in values {
        let row = format!("0<H<{}<H<{}", table_value_text(x)?, table_value_text(y)?);
        layout.extend_from_slice(row.as_bytes());
    }
    let used_len = layout.len();
    while layout.len() % 4 != 0 {
        layout.push(0);
    }
    // Keep the opaque alignment bytes observed in the original two- and
    // three-row files, so existing generated files retain their exact bytes.
    match (values.len(), used_len) {
        (2, 122) => layout[122..124].copy_from_slice(&[10, 72]),
        (3, 147) => layout[147] = 230,
        _ => {}
    }
    let length = layout.len() as u32;
    layout[4..8].copy_from_slice(&length.to_le_bytes());
    let mut group = Vec::new();
    put_record(&mut group, 0x07d0, &base[rs[0].payload..rs[0].end]);
    let mut refs = base[rs[1].payload..rs[1].end].to_vec();
    refs[4..8].copy_from_slice(&sources[0].to_le_bytes());
    refs[8..12].copy_from_slice(&sources[1].to_le_bytes());
    put_record(&mut group, 0x07d2, &refs);
    put_record(&mut group, 0x090d, &layout);
    put_record(&mut group, 0x07d8, &base[rs[3].payload..rs[3].end]);
    put_record(&mut group, 0x07d7, &[]);
    Ok(group)
}

fn named_table_parts(layout: &[u8]) -> Option<(String, Vec<String>)> {
    if layout.len() < 28 || layout.len() % 4 != 0 {
        return None;
    }
    let text = &layout[28..];
    let used = text
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(text.len());
    if text.len() - used > 3 || text[used..].iter().any(|byte| *byte != 0) {
        return None;
    }
    let text = std::str::from_utf8(&text[..used]).ok()?;
    if !text.is_ascii() {
        return None;
    }
    let rest = text.strip_prefix("000<SS2<T")?;
    let (name_len, rest) = rest.split_once('x')?;
    let name_len = name_len.parse::<usize>().ok()?;
    if !(1..=32).contains(&name_len) || rest.len() < name_len {
        return None;
    }
    let (name, rest) = rest.split_at(name_len);
    if !name.as_bytes()[0].is_ascii_alphabetic()
        || !name
            .bytes()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == b'_')
    {
        return None;
    }
    let mut rest = rest.strip_prefix(">>")?;
    let mut values = Vec::new();
    while !rest.is_empty() {
        if values.len() >= 256 {
            return None;
        }
        rest = rest.strip_prefix("0<H<T")?;
        let (value_len, after_len) = rest.split_once('x')?;
        let value_len = value_len.parse::<usize>().ok()?;
        if !(3..=6).contains(&value_len) || after_len.len() < value_len {
            return None;
        }
        let (value, after_value) = after_len.split_at(value_len);
        rest = after_value.strip_prefix(">>")?;
        let decimals = value.split_once('.')?.1.len();
        if !(1..=2).contains(&decimals) {
            return None;
        }
        let numeric = value.parse::<f64>().ok()?;
        decimal_words(numeric).ok()?;
        if format!("{numeric:.decimals$}") != value {
            return None;
        }
        values.push(value.to_owned());
    }
    (!values.is_empty()).then_some((name.to_owned(), values))
}

fn canonical_named_table(
    source: u32,
    name: &str,
    values: &[String],
    anchor: (i16, i16),
) -> Result<Vec<u8>, String> {
    if source == 0 || name.len() > 32 || !(1..=256).contains(&values.len()) {
        return Err("Invalid named table source or value".into());
    }
    let base = table_template();
    let rs = records(&base, 0)?;
    let mut layout = base[rs[2].payload..rs[2].payload + 28].to_vec();
    layout[12..14].copy_from_slice(&anchor.0.to_le_bytes());
    layout[14..16].copy_from_slice(&anchor.1.to_le_bytes());
    layout[16..18].copy_from_slice(&((values.len() + 2) as u16).to_le_bytes());
    layout.extend_from_slice(format!("000<SS2<T{}x{name}>>", name.len()).as_bytes());
    for value in values {
        layout.extend_from_slice(format!("0<H<T{}x{value}>>", value.len()).as_bytes());
    }
    while layout.len() % 4 != 0 {
        layout.push(0);
    }
    if named_table_parts(&layout) != Some((name.to_owned(), values.to_vec())) {
        return Err("Invalid named table text".into());
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

    pub fn named_snapshot(
        source: u32,
        parameter: &ParameterGroup,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        let value = format!("{:.*}", parameter.precision(), parameter.value());
        let bytes = canonical_named_table(source, parameter.name(), &[value], anchor)?;
        Self::parse(&bytes)
    }

    pub fn pair_snapshot() -> Self {
        Self {
            bytes: object_group(TWO_COLUMN_TABLE, 3),
        }
    }

    pub fn pair_snapshot_rows(rows: usize, anchor: (i16, i16)) -> Result<Self, String> {
        Self::pair_snapshot_rows_for_sources(rows, anchor, [2, 1])
    }

    pub fn pair_snapshot_rows_for_sources(
        rows: usize,
        anchor: (i16, i16),
        sources: [u32; 2],
    ) -> Result<Self, String> {
        if !(1..=256).contains(&rows) {
            return Err("Pair table requires one to 256 rows".into());
        }
        Ok(Self {
            bytes: canonical_pair_table(&vec![[1.0, 1.0]; rows], anchor, sources)?,
        })
    }

    pub fn parse_pair(bytes: &[u8]) -> Result<Self, String> {
        let rs = records(bytes, 0)?;
        if rs.iter().map(|r| r.kind).collect::<Vec<_>>() != [0x07d0, 0x07d2, 0x090d, 0x07d8, 0x07d7]
            || rs[1].end - rs[1].payload != 12
            || rs[2].end - rs[2].payload < 100
        {
            return Err("Unsupported two-column table layout".into());
        }
        let rows = u16::from_le_bytes(
            bytes[rs[2].payload + 16..rs[2].payload + 18]
                .try_into()
                .unwrap(),
        ) as usize;
        let rows = rows.checked_sub(2).ok_or("Invalid pair table row count")?;
        if !(1..=256).contains(&rows) {
            return Err("Unsupported pair table row count".into());
        }
        let sources = [
            u32::from_le_bytes(
                bytes[rs[1].payload + 4..rs[1].payload + 8]
                    .try_into()
                    .unwrap(),
            ),
            u32::from_le_bytes(
                bytes[rs[1].payload + 8..rs[1].payload + 12]
                    .try_into()
                    .unwrap(),
            ),
        ];
        let layout = &bytes[rs[2].payload..rs[2].end];
        let values = pair_table_values(layout, rows).ok_or("Invalid pair table rows")?;
        let anchor = (
            i16::from_le_bytes(layout[12..14].try_into().unwrap()),
            i16::from_le_bytes(layout[14..16].try_into().unwrap()),
        );
        let template = canonical_pair_table(&values, anchor, sources)?;
        let template_records = records(&template, 0)?;
        if bytes.len() != template.len()
            || rs[2].end - rs[2].payload != template_records[2].end - template_records[2].payload
        {
            return Err("Unsupported two-column table length".into());
        }
        let padding = template_records[2].end
            - template_records[2].payload
            - (72
                + values
                    .iter()
                    .map(|[x, y]| {
                        4 + table_value_text(*x).unwrap().len()
                            + 3
                            + table_value_text(*y).unwrap().len()
                    })
                    .sum::<usize>());
        for (i, (actual, expected)) in bytes.iter().zip(&template).enumerate() {
            let mutable = i == 10
                || (rs[2].payload + 12..rs[2].payload + 16).contains(&i)
                || (rs[2].end - padding..rs[2].end).contains(&i);
            if !mutable && actual != expected {
                return Err("Unsupported two-column table variant".into());
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
        let rs = records(&self.bytes, 0).ok()?;
        pair_table_values(&self.bytes[rs[2].payload..rs[2].end], self.rows())
    }

    pub fn set_pair_row_value(&mut self, row: usize, value: f64) -> Result<(), String> {
        let mut values = self
            .pair_values()
            .and_then(|values| values.get(row).copied())
            .ok_or("Invalid pair table row")?;
        values[0] = value;
        self.set_pair_row_values(row, values)
    }

    pub fn set_pair_row_values(&mut self, row: usize, values: [f64; 2]) -> Result<(), String> {
        let mut all = self.pair_values().ok_or("Invalid pair table")?;
        *all.get_mut(row).ok_or("Invalid pair table row")? = values;
        let mut changed = canonical_pair_table(
            &all,
            self.anchor(),
            self.pair_source_ids().ok_or("Invalid pair table sources")?,
        )?;
        changed[10] = self.bytes[10];
        self.bytes = changed;
        Self::parse_pair(&self.bytes)?;
        Ok(())
    }

    pub fn set_last_pair_value(&mut self, value: f64) -> Result<(), String> {
        self.set_pair_row_value(self.rows() - 1, value)
    }

    pub fn set_last_pair_axis_value(&mut self, axis: usize, value: f64) -> Result<(), String> {
        if axis > 1 {
            return Err("Invalid pair table column".into());
        }
        let row = self.rows() - 1;
        let mut values = self
            .pair_values()
            .and_then(|values| values.get(row).copied())
            .ok_or("Invalid pair table row")?;
        values[axis] = value;
        self.set_pair_row_values(row, values)
    }

    pub fn for_source(source: u32, anchor: (i16, i16)) -> Result<Self, String> {
        Self::for_source_value(source, anchor, 1.0)
    }

    pub fn for_source_value(source: u32, anchor: (i16, i16), value: f64) -> Result<Self, String> {
        Ok(Self {
            bytes: canonical_table_values(source, anchor, &[value])?,
        })
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let rs = records(bytes, 0)?;
        if rs.iter().map(|r| r.kind).collect::<Vec<_>>() != [0x07d0, 0x07d2, 0x090d, 0x07d8, 0x07d7]
        {
            return Err("Unsupported table record sequence".into());
        }
        if rs[1].end - rs[1].payload != 8 || rs[2].end - rs[2].payload < 28 {
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
        let layout = &bytes[rs[2].payload..rs[2].end];
        if u32::from_le_bytes(layout[4..8].try_into().unwrap()) as usize == layout.len()
            && let Some((name, values)) = named_table_parts(layout)
            && rows == values.len()
            && normalized == canonical_named_table(source, &name, &values, anchor)?
        {
            return Ok(Self {
                bytes: bytes.to_vec(),
            });
        }
        // A two-row table saved after Plot Points has an observed `t` in
        // the penultimate padding byte of 0x090d. It is outside the row text.
        if rows == 2 && normalized[rs[2].end - 2] == b't' && normalized[rs[2].end - 1] == 0 {
            normalized[rs[2].end - 2] = 0;
        }
        let values = default_table_values(layout, rows).ok_or("Invalid default table rows")?;
        if normalized != canonical_table_values(source, anchor, &values)? {
            return Err("Unsupported one-column table variant".into());
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    pub fn named_label_values(&self) -> Option<(String, Vec<String>)> {
        let rs = records(&self.bytes, 0).ok()?;
        named_table_parts(&self.bytes[rs[2].payload..rs[2].end])
    }

    pub fn named_label_value(&self) -> Option<(String, String)> {
        let (name, values) = self.named_label_values()?;
        Some((name, values.into_iter().next()?))
    }

    pub fn default_values(&self) -> Option<Vec<f64>> {
        if self.named_label_values().is_some() {
            return None;
        }
        let rs = records(&self.bytes, 0).ok()?;
        default_table_values(&self.bytes[rs[2].payload..rs[2].end], self.rows())
    }

    pub fn set_last_default_value(&mut self, value: f64) -> Result<(), String> {
        let mut values = self
            .default_values()
            .ok_or("Not a default parameter table")?;
        *values.last_mut().ok_or("Table has no rows")? = value;
        self.bytes = canonical_table_values(self.source_id(), self.anchor(), &values)?;
        Ok(())
    }

    pub fn add_named_snapshot(&mut self, value: &str) -> Result<(), String> {
        let (name, mut values) = self.named_label_values().ok_or("Not a named table")?;
        *values.last_mut().ok_or("Table has no rows")? = value.to_owned();
        values.push(value.to_owned());
        self.bytes = canonical_named_table(self.source_id(), &name, &values, self.anchor())?;
        Ok(())
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
        let mut values = self
            .default_values()
            .ok_or("Not a default parameter table")?;
        values.push(*values.last().ok_or("Table has no rows")?);
        self.bytes = canonical_table_values(self.source_id(), self.anchor(), &values)?;
        Ok(())
    }

    pub fn remove_last_snapshot(&mut self) -> Result<(), String> {
        if let Some((name, mut values)) = self.named_label_values() {
            if values.len() <= 1 {
                return Err("Table has only one row".into());
            }
            values.pop();
            self.bytes = canonical_named_table(self.source_id(), &name, &values, self.anchor())?;
            return Ok(());
        }
        let mut values = self
            .default_values()
            .ok_or("Not a default parameter table")?;
        if values.len() <= 1 {
            return Err("Table has only one row".into());
        }
        let live = values.pop().unwrap();
        *values.last_mut().unwrap() = live;
        self.bytes = canonical_table_values(self.source_id(), self.anchor(), &values)?;
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

    pub fn remap_sources(&self, ids: &[usize]) -> Result<Self, String> {
        let mut group = self.clone();
        let rs = records(&group.bytes, 0)?;
        let references = rs.get(1).ok_or("Missing table references")?;
        if references.kind != 0x07d2 {
            return Err("Missing table source references".into());
        }
        let count = match references.end - references.payload {
            8 => 1,
            12 => 2,
            _ => return Err("Unsupported table source count".into()),
        };
        for index in 0..count {
            let at = references.payload + 4 + index * 4;
            let old = u32::from_le_bytes(group.bytes[at..at + 4].try_into().unwrap()) as usize;
            let new = ids
                .get(old)
                .copied()
                .filter(|id| *id != 0)
                .ok_or("Table source was deleted")?;
            group.bytes[at..at + 4].copy_from_slice(
                &u32::try_from(new)
                    .map_err(|_| "Too many table sources")?
                    .to_le_bytes(),
            );
        }
        if count == 1 {
            Self::parse(&group.bytes)?;
        } else {
            Self::parse_pair(&group.bytes)?;
        }
        Ok(group)
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
    fn animated_parameter_keeps_precise_native_value_and_domain() {
        let bytes = include_bytes!("../tests/fixtures/parameter_animated_native_saved.gsp");
        let group = object_group(bytes, 1);
        let mut parameter = ParameterGroup::parse(&group).unwrap();
        assert!((parameter.value() - 1.957).abs() < 1e-12);
        assert_eq!(parameter.animation_domain(), (-100., 100.));
        assert_eq!(parameter.as_bytes(), group);
        parameter.set_value(2.5).unwrap();
        assert_eq!(parameter.value(), 2.5);
        ParameterGroup::parse(parameter.as_bytes()).unwrap();
        for (bytes, range) in [
            (
                include_bytes!("../tests/fixtures/action_animation_parameter_native_rate2.gsp")
                    .as_slice(),
                5.7..5.8,
            ),
            (
                include_bytes!("../tests/fixtures/action_animation_parameter_native_discrete.gsp")
                    .as_slice(),
                6.7..6.9,
            ),
        ] {
            let original = object_group(bytes, 1);
            let parsed = ParameterGroup::parse(&original).unwrap();
            assert!(range.contains(&parsed.value()));
            assert_eq!(parsed.as_bytes(), original);
        }
    }

    #[test]
    fn named_parameter_decodes_and_matches_original_groups() {
        let custom = object_group(
            include_bytes!("../tests/fixtures/table_parameter_custom.gsp"),
            1,
        );
        let parsed = ParameterGroup::parse(&custom).unwrap();
        assert_eq!(parsed.name(), "a");
        assert_eq!(parsed.value(), 2.5);
        assert_eq!(parsed.precision(), 2);
        assert_eq!(ParameterGroup::parse(&custom).unwrap().as_bytes(), custom);

        let speed = object_group(NAMED_PARAMETER, 1);
        let parsed = ParameterGroup::parse(&speed).unwrap();
        assert_eq!(parsed.name(), "speed");
        assert_eq!(parsed.value(), 2.5);
        assert_eq!(parsed.precision(), 1);
        assert_eq!(
            ParameterGroup::named("speed", 2.5).unwrap().as_bytes(),
            speed
        );
        assert_eq!(
            ParameterGroup::named("speed", 3.25).unwrap().as_bytes(),
            object_group(
                include_bytes!("../tests/fixtures/table_parameter_speed_precision2_original.gsp"),
                1
            )
        );
        let original_test = object_group(
            include_bytes!("../tests/fixtures/table_parameter_test_original.gsp"),
            1,
        );
        let original_test = ParameterGroup::parse(&original_test).unwrap();
        let generated_test = ParameterGroup::named("test", 2.5).unwrap();
        assert_eq!(generated_test.name(), "test");
        assert_eq!(generated_test.bbox(), original_test.bbox());
        assert_eq!(ParameterGroup::named("test", 3.25).unwrap().name(), "test");
        assert!(ParameterGroup::named("3bad", 1.).is_err());
    }

    #[test]
    fn named_parameter_tables_match_original_saved_groups() {
        for (document, parameter_id, table_id) in [
            (
                include_bytes!("../tests/fixtures/table_named_speed_snapshot_original.gsp")
                    .as_slice(),
                1,
                2,
            ),
            (
                include_bytes!(
                    "../tests/fixtures/table_named_speed_precision2_snapshot_original.gsp"
                )
                .as_slice(),
                1,
                2,
            ),
            (
                include_bytes!("../tests/fixtures/table_named_test_late_source_original.gsp")
                    .as_slice(),
                5,
                7,
            ),
        ] {
            let parameter = ParameterGroup::parse(&object_group(document, parameter_id)).unwrap();
            let original = object_group(document, table_id);
            let table = TableGroup::parse(&original).unwrap();
            assert_eq!(table.source_id(), parameter_id as u32);
            assert_eq!(table.named_label_value().unwrap().0, parameter.name());
            assert_eq!(
                TableGroup::named_snapshot(parameter_id as u32, &parameter, table.anchor())
                    .unwrap()
                    .as_bytes(),
                original
            );
            assert!(table.clone().add_default_snapshot().is_err());
        }
        let original = include_bytes!("../tests/fixtures/table_named_speed_two_rows_original.gsp");
        let parameter = ParameterGroup::parse(&object_group(original, 1)).unwrap();
        let two_rows = object_group(original, 2);
        let parsed = TableGroup::parse(&two_rows).unwrap();
        assert_eq!(parsed.named_label_values().unwrap().1, ["2.5", "2.5"]);
        let mut generated = TableGroup::named_snapshot(1, &parameter, parsed.anchor()).unwrap();
        generated.add_named_snapshot("2.5").unwrap();
        assert_eq!(generated.as_bytes(), two_rows);
        generated.remove_last_snapshot().unwrap();
        assert_eq!(generated.rows(), 1);
        assert!(generated.remove_last_snapshot().is_err());

        let live_add = include_bytes!("../tests/fixtures/table_named_speed_live_add_original.gsp");
        let original_group = object_group(live_add, 2);
        let original_table = TableGroup::parse(&original_group).unwrap();
        assert_eq!(
            original_table.named_label_values().unwrap().1,
            ["3.25", "3.25"]
        );
        let old_parameter = ParameterGroup::named("speed", 2.5).unwrap();
        let mut generated =
            TableGroup::named_snapshot(1, &old_parameter, original_table.anchor()).unwrap();
        generated.add_named_snapshot("3.25").unwrap();
        assert_eq!(generated.as_bytes(), original_group);
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

    #[test]
    fn original_pair_tables_decode_through_four_rows() {
        for (source, expected) in [
            (TWO_COLUMN_TABLE, vec![[1., 1.]]),
            (TWO_ROW_PAIR_TABLE, vec![[1., 1.], [1., 1.]]),
            (THREE_ROW_PAIR_TABLE, vec![[1., 1.], [2., 1.], [2., 1.]]),
            (
                FOUR_ROW_PAIR_TABLE,
                vec![[1., 1.], [2., 1.], [2., 1.], [2., 1.]],
            ),
        ] {
            let group = object_group(source, 3);
            let table = TableGroup::parse_pair(&group)
                .unwrap_or_else(|error| panic!("pair table: {error}"));
            assert_eq!(table.rows(), expected.len());
            assert_eq!(table.pair_values(), Some(expected));
            assert_eq!(table.as_bytes(), group);
        }
    }

    #[test]
    fn generated_pair_rows_reopen_after_value_edits() {
        for rows in [1, 2, 3, 4, 5, 8, 16] {
            let mut table = TableGroup::pair_snapshot_rows(rows, (10, 71)).unwrap();
            for row in 0..rows {
                table.set_pair_row_value(row, (row % 10) as f64).unwrap();
            }
            let reopened = TableGroup::parse_pair(table.as_bytes()).unwrap();
            assert_eq!(reopened.rows(), rows);
            assert_eq!(reopened.pair_values(), table.pair_values());
        }
        let limit = TableGroup::pair_snapshot_rows(256, (10, 71)).unwrap();
        assert_eq!(
            TableGroup::parse_pair(limit.as_bytes()).unwrap().rows(),
            256
        );
        assert!(TableGroup::pair_snapshot_rows(257, (10, 71)).is_err());
    }
    #[test]
    fn table_source_ids_can_shift_after_an_earlier_object_is_deleted() {
        let ids = [0, 0, 1, 2];
        let one = TableGroup::for_source_value(2, (10, 38), 3.25).unwrap();
        let shifted = one.remap_sources(&ids).unwrap();
        assert_eq!(shifted.source_id(), 1);
        assert_eq!(shifted.default_values(), Some(vec![3.25]));
        assert!(one.remap_sources(&[0, 0, 0]).is_err());

        let pair = TableGroup::pair_snapshot_rows_for_sources(2, (10, 71), [3, 2]).unwrap();
        let shifted = pair.remap_sources(&ids).unwrap();
        assert_eq!(shifted.pair_source_ids(), Some([2, 1]));
        assert_eq!(shifted.pair_values(), pair.pair_values());
    }
}
