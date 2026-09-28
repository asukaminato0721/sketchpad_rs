//! Bounded GSP4 support for numeric parameters and verified snapshot tables.
//! Unknown layouts and values are refused before writing.

const TEMPLATE_DOCUMENT: &[u8] = include_bytes!("../../../tests/fixtures/table_parameter_tabulated.gsp");
const DEFAULT_PARAMETER: &[u8] = include_bytes!("../../../tests/fixtures/table_parameter_default.gsp");
const TWO_PARAMETERS: &[u8] = include_bytes!("../../../tests/fixtures/table_two_parameters_original.gsp");
const NAMED_PARAMETER: &[u8] =
    include_bytes!("../../../tests/fixtures/table_parameter_speed_original.gsp");
const TWO_COLUMN_TABLE: &[u8] = include_bytes!("../../../tests/fixtures/table_two_columns_original.gsp");
#[cfg(test)]
const TWO_ROW_PAIR_TABLE: &[u8] =
    include_bytes!("../../../tests/fixtures/table_two_columns_two_rows_original.gsp");
#[cfg(test)]
const THREE_ROW_PAIR_TABLE: &[u8] =
    include_bytes!("../../../tests/fixtures/table_pair_three_rows_distinct_original.gsp");
#[cfg(test)]
const FOUR_ROW_PAIR_TABLE: &[u8] =
    include_bytes!("../../../tests/fixtures/table_pair_four_rows_original.gsp");
const CALCULATION: &[u8] = include_bytes!("../../../tests/fixtures/table_calculate_2_plus_3.gsp");
const MEASURED_CALCULATION: &[u8] = include_bytes!("../../../assets/measurement_times_two_group.bin");
const MEASURED_CALCULATION_TWO_DIGIT: &[u8] =
    include_bytes!("../../../assets/measurement_times_twelve_group.bin");
const PARAMETER_CALCULATION_NATIVE: &[u8] =
    include_bytes!("../../../tests/fixtures/parameter_calculation_chain_native.gsp");

// Round the stored binary value directly. Multiplying by a floating decimal
// scale can turn a value below a half into an exact half (for example 3/40).
fn rounded_fixed_magnitude(value: f64, precision: usize) -> Option<u128> {
    if !value.is_finite() || precision > 18 {
        return None;
    }
    let bits = value.abs().to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1u64 << 52) - 1);
    let (mantissa, exponent) = if exponent == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1u64 << 52), exponent - 1023 - 52)
    };
    let numerator = u128::from(mantissa).checked_mul(5u128.pow(precision as u32))?;
    let shift = exponent + precision as i32;
    if shift >= 0 {
        return numerator.checked_mul(1u128.checked_shl(shift as u32)?);
    }
    let denominator_shift = (-shift) as u32;
    if denominator_shift > 128 {
        return Some(0);
    }
    if denominator_shift == 128 {
        return Some(u128::from(numerator >= (1u128 << 127)));
    }
    let quotient = numerator >> denominator_shift;
    let remainder = numerator & ((1u128 << denominator_shift) - 1);
    Some(quotient + u128::from(remainder >= (1u128 << (denominator_shift - 1))))
}

fn format_fixed_numeric_value(value: f64, precision: usize) -> String {
    if let Some(magnitude) = rounded_fixed_magnitude(value, precision) {
        let sign = if value.is_sign_negative() { "-" } else { "" };
        if precision == 0 {
            return format!("{sign}{magnitude}");
        }
        let factor = 10u128.pow(precision as u32);
        return format!(
            "{sign}{}.{:0precision$}",
            magnitude / factor,
            magnitude % factor
        );
    }
    format!("{value:.precision$}")
}

/// Native numeric captions round halfway values away from zero and use scientific
/// notation only when the unrounded magnitude is greater than one million.
pub fn format_numeric_value(value: f64, precision: usize) -> String {
    if !value.is_finite() || value.abs() <= 1e6 {
        return format_fixed_numeric_value(value, precision);
    }
    let mut exponent = value.abs().log10().floor() as i32;
    let mut mantissa = value / 10_f64.powi(exponent);
    let factor = 10_f64.powi(precision as i32);
    let carries = rounded_fixed_magnitude(mantissa, precision).map_or_else(
        || (mantissa.abs() * factor).round() / factor >= 10.,
        |rounded| rounded >= 10u128.pow(precision as u32) * 10,
    );
    if carries {
        exponent += 1;
        mantissa /= 10.;
    }
    let exponent = exponent
        .to_string()
        .chars()
        .map(|digit| match digit {
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            '-' => '⁻',
            _ => unreachable!(),
        })
        .collect::<String>();
    format!(
        "{} · 10{exponent}",
        format_fixed_numeric_value(mantissa, precision)
    )
}

fn calculation_precision(bytes: &[u8]) -> Result<usize, String> {
    let record = records(bytes, 0)?
        .into_iter()
        .find(|record| record.kind == 0x0907)
        .ok_or("Missing calculation precision")?;
    let field = bytes
        .get(record.payload + 28..record.payload + 30)
        .ok_or("Missing calculation precision")?;
    let precision = u16::from_le_bytes(field.try_into().unwrap());
    if precision > 5 {
        return Err("Calculation precision must be from 0 to 5".into());
    }
    Ok(usize::from(precision))
}

fn calculation_bytes_with_precision(bytes: &[u8], precision: u16) -> Result<Vec<u8>, String> {
    if precision > 5 {
        return Err("Calculation precision must be from 0 to 5".into());
    }
    let record = records(bytes, 0)?
        .into_iter()
        .find(|record| record.kind == 0x0907)
        .ok_or("Missing calculation precision")?;
    let mut updated = bytes.to_vec();
    let field = updated
        .get_mut(record.payload + 28..record.payload + 30)
        .ok_or("Missing calculation precision")?;
    field.copy_from_slice(&precision.to_le_bytes());
    Ok(updated)
}

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
    // Sampling a native parameter trace must use the exact floating-point
    // driver without rewriting its saved decimal spelling or presentation.
    evaluation_value: Option<[u8; 8]>,
}

fn decimal_words(value: f64) -> Result<Vec<u16>, String> {
    if !value.is_finite() {
        return Err("Parameter value must be finite".into());
    }
    let text = if value == 0. {
        "0".into()
    } else {
        value.to_string()
    };
    let words = text
        .chars()
        .map(|ch| match ch {
            '-' => Ok(4097),
            '.' => Ok(10),
            '0'..='9' => Ok(ch as u16 - '0' as u16),
            _ => Err("Unsupported parameter number spelling"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    if words.len() > 32 {
        return Err("Parameter value exceeds 32 native numeric tokens".into());
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
    if !(1..=32).contains(&whole.len())
        || fraction.len() > 30
        || decimal.is_some() && fraction.is_empty()
        || whole.iter().chain(fraction).any(|digit| *digit > 9)
        || whole.len() > 1 && whole[0] == 0
    {
        return Err("Unsupported parameter digits".into());
    }
    let mut text = String::new();
    if negative {
        text.push('-');
    }
    for digit in whole {
        text.push(char::from(b'0' + *digit as u8));
    }
    if decimal.is_some() {
        text.push('.');
        for digit in fraction {
            text.push(char::from(b'0' + *digit as u8));
        }
    }
    text.parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .ok_or_else(|| "Invalid parameter value".into())
}

struct ParameterLiteral {
    value: f64,
    suffix: Option<u16>,
    scientific: bool,
}

fn parameter_literal(words: &[u16], unit: NumericUnit) -> Result<ParameterLiteral, String> {
    if unit == NumericUnit::Scalar {
        return Ok(ParameterLiteral {
            value: value_from_words(words)?,
            suffix: None,
            scientific: false,
        });
    }
    let at = words
        .iter()
        .position(|word| matches!(word, 0x0100..=0x0102 | 0x0201))
        .ok_or("Parameter value units disagree with its header")?;
    let suffix = words[at];
    if !matches!(
        (unit, suffix),
        (NumericUnit::Length, 0x0201) | (NumericUnit::Degrees, 0x0100..=0x0102)
    ) {
        return Err("Parameter value units disagree with its header".into());
    }
    // Actual native parameter words [13, 0x0100] encode pi radians,
    // displayed as 180 degrees. Other constant/unit combinations and
    // expressions have not been established as parameter literals.
    let mut value = if suffix == 0x0100 && words[..at] == [13] {
        CALCULATOR_PI
    } else {
        value_from_words(&words[..at])?
    };
    let scientific = match &words[at + 1..] {
        [] => false,
        // The three observed native typed scientific literals put the unit
        // before multiplication by 10^-5. This is a number representation,
        // not a general calculation or an inferred cache value.
        [0x1002, 1, 0, 0x1004, 0x1001, 5] if matches!(suffix, 0x0101 | 0x0201) => true,
        _ => return Err("Unsupported parameter scientific literal".into()),
    };
    if scientific {
        value = format!("{value}e-5")
            .parse()
            .map_err(|_| "Invalid scientific parameter value")?;
    }
    Ok(ParameterLiteral {
        value,
        suffix: Some(suffix),
        scientific,
    })
}

fn shift_decimal_words(words: &[u16], places: usize) -> Vec<u16> {
    let negative = words.first() == Some(&4097);
    let words = &words[usize::from(negative)..];
    let decimal = words
        .iter()
        .position(|word| *word == 10)
        .unwrap_or(words.len());
    let mut digits = words
        .iter()
        .copied()
        .filter(|word| *word != 10)
        .collect::<Vec<_>>();
    let split = decimal + places;
    digits.resize(digits.len().max(split), 0);
    let mut whole = digits[..split].to_vec();
    while whole.len() > 1 && whole[0] == 0 {
        whole.remove(0);
    }
    let mut fraction = digits[split..].to_vec();
    while fraction.last() == Some(&0) {
        fraction.pop();
    }
    let mut result = if negative { vec![4097] } else { Vec::new() };
    result.extend(whole);
    if !fraction.is_empty() {
        result.push(10);
        result.extend(fraction);
    }
    result
}

impl ParameterGroup {
    pub fn default_value() -> Self {
        Self {
            bytes: object_group(DEFAULT_PARAMETER, 1),
            evaluation_value: None,
        }
    }

    pub fn second_default_value() -> Self {
        Self {
            bytes: object_group(TWO_PARAMETERS, 2),
            evaluation_value: None,
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
            evaluation_value: None,
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
        let whole_digits = value.abs().to_string().split('.').next().unwrap().len();
        let value_extra = 4 * (whole_digits.saturating_sub(1) as i16 + i16::from(value < 0.));
        let width =
            80 + letter_width + 10 * group.precision().saturating_sub(1) as i16 + value_extra;
        updated[bbox + 4..bbox + 6].copy_from_slice(&(left + width).to_le_bytes());
        Self::parse(&updated)
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let rs = records(bytes, 0)?;
        let kinds = rs.iter().map(|r| r.kind).collect::<Vec<_>>();
        if !matches!(
            kinds.as_slice(),
            [0x07d0, 0x0907, 0x07d5, 0x07d8, 0x08a3, 0x07d7]
                | [0x07d0, 0x0907, 0x07d8, 0x08a3, 0x07d7]
                | [0x07d0, 0x0907, 0x07d5, 0x07d8, 0x08a3, 0x232e, 0x07d7]
        ) || rs.iter().any(|r| r.kind == 0x232e && r.end != r.payload)
        {
            return Err("Unsupported parameter record sequence".into());
        }
        let font = rs.iter().find(|r| r.kind == 0x07d8).unwrap();
        let bbox_record = rs.iter().find(|r| r.kind == 0x08a3).unwrap();
        let label_record = rs.iter().find(|r| r.kind == 0x07d5);
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
        let unit = match bytes.get(16..18) {
            Some([79, 0]) => NumericUnit::Scalar,
            Some([80, 0]) => NumericUnit::Length,
            Some([81, 0]) => NumericUnit::Degrees,
            _ => return Err("Unsupported parameter units".into()),
        };
        let literal = parameter_literal(&words, unit)?;
        let display_mode = u16::from_le_bytes(payload[26..28].try_into().unwrap());
        // Native mode zero displays the literal definition instead of its
        // stored name. Only an ordinary, labeled centimeter literal has been
        // established in this mode; do not infer other combinations.
        let display_mode_supported = matches!(display_mode, 1 | 3)
            || display_mode == 0
                && unit == NumericUnit::Length
                && !literal.scientific
                && label_record.is_some()
                && kinds.len() == 6;
        let precision = u16::from_le_bytes(payload[28..30].try_into().unwrap());
        let step = f64::from_le_bytes(payload[52..60].try_into().unwrap());
        let domain = (
            f64::from_le_bytes(payload[60..68].try_into().unwrap()),
            f64::from_le_bytes(payload[68..76].try_into().unwrap()),
        );
        if precision > 5
            || !step.is_finite()
            || step <= 0.
            || !domain.0.is_finite()
            || !domain.1.is_finite()
            || domain.0 > domain.1
        {
            return Err("Unsupported parameter precision or animation domain".into());
        }
        // The native numeric prefix identifies the value representation. Colors,
        // label flags, font indices and persisted caches do not identify its math.
        let template = object_group(DEFAULT_PARAMETER, 1);
        let base = records(&template, 0)?;
        let expected = &template[base[1].payload..base[1].end];
        let immutable = [0..12, 16..22, 24..26, 32..36, 40..44, 46..48];
        if immutable
            .iter()
            .any(|range| payload[range.clone()] != expected[range.clone()])
            || !display_mode_supported
            || rs[0].end - rs[0].payload != 28
            || u32::from_le_bytes(bytes[rs[0].payload..rs[0].payload + 4].try_into().unwrap())
                & 0xffff
                != 0
            || !matches!(
                u16::from_le_bytes(
                    bytes[rs[0].payload + 8..rs[0].payload + 10]
                        .try_into()
                        .unwrap()
                ),
                79..=81
            )
            || font.end - font.payload != 2
            || bbox_record.end - bbox_record.payload != 8
            || rs.last().unwrap().end != rs.last().unwrap().payload
        {
            return Err("Unsupported parameter variant".into());
        }
        if let Some(record) = label_record {
            let label = crate::text_records::LabelAux::parse(&bytes[record.payload..record.end])?;
            let name = label
                .name_utf8()
                .map_err(|_| "Parameter name is not UTF-8")?;
            if name.is_empty() || name.len() > 256 || name.chars().any(char::is_control) {
                return Err("Unsupported parameter name".into());
            }
        }
        let group = Self {
            bytes: bytes.to_vec(),
            evaluation_value: None,
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

    /// The explicitly stored name; an absent label returns an empty string.
    /// Native generated names require the containing page's naming counter.
    pub fn name(&self) -> &str {
        let rs = records(&self.bytes, 0).unwrap();
        let Some(label) = rs.iter().find(|r| r.kind == 0x07d5) else {
            return "";
        };
        let payload = label.payload;
        let len =
            u16::from_le_bytes(self.bytes[payload + 22..payload + 24].try_into().unwrap()) as usize;
        std::str::from_utf8(&self.bytes[payload + 24..payload + 24 + len]).unwrap()
    }

    pub(crate) fn with_generated_name(
        &self,
        number: u32,
        font: &crate::text_style_records::FontState,
    ) -> Result<Self, String> {
        if number == 0 || !self.name().is_empty() || self.display_mode() == 0 {
            return Err("Parameter cannot receive a generated name".into());
        }
        if font.family != "Tahoma"
            || font.size_code != 18
            || font.bold
            || font.italic
            || font.underline
            || self.literal().scientific
        {
            return Err(
                "Generated parameter captions need a verified font and literal layout".into(),
            );
        }
        // Native controls establish the normal-18 caption metrics, including
        // two-digit subscripts. Restore short caches left by missing names.
        let mut width = 58 + 7 * (number.to_string().len() as i32 - 1);
        for ch in format_fixed_numeric_value(self.value(), self.precision()).chars() {
            width += match ch {
                '0'..='9' => 10,
                '.' => 6,
                '-' => 10,
                _ => return Err("Generated parameter caption needs verified glyph metrics".into()),
            };
        }
        width += match self.unit() {
            NumericUnit::Scalar => 0,
            NumericUnit::Degrees => 9,
            NumericUnit::Length => 42,
            _ => return Err("Unsupported generated parameter units".into()),
        };
        let name = format!("t[{number}]");
        let named = object_group(NAMED_PARAMETER, 1);
        let named_records = records(&named, 0)?;
        let mut label = named[named_records[2].payload..named_records[2].payload + 22].to_vec();
        label[..2].copy_from_slice(&((name.len() + 2) as u16).to_le_bytes());
        label.extend_from_slice(&(name.len() as u16).to_le_bytes());
        label.extend_from_slice(name.as_bytes());
        label.push(0);
        if name.len() % 2 == 0 {
            label.push(0);
        }
        let rs = records(&self.bytes, 0)?;
        let mut bytes = Vec::new();
        for record in rs {
            if record.kind == 0x07d5 {
                continue;
            }
            if record.kind == 0x07d8 {
                put_record(&mut bytes, 0x07d5, &label);
            }
            bytes.extend_from_slice(&self.bytes[record.start..record.end]);
        }
        let updated = records(&bytes, 0)?;
        let bbox = updated
            .iter()
            .find(|record| record.kind == 0x08a3)
            .ok_or("Missing parameter caption box")?
            .payload;
        let anchor = self.anchor();
        let right = i16::try_from(i32::from(anchor.0) + width)
            .map_err(|_| "Parameter caption box overflow")?;
        let bottom = anchor
            .1
            .checked_add(32)
            .ok_or("Parameter caption box overflow")?;
        bytes[bbox + 4..bbox + 6].copy_from_slice(&right.to_le_bytes());
        bytes[bbox + 6..bbox + 8].copy_from_slice(&bottom.to_le_bytes());
        let mut result = Self::parse(&bytes)?;
        result.evaluation_value = self.evaluation_value;
        Ok(result)
    }

    pub fn display_mode(&self) -> u16 {
        let rs = records(&self.bytes, 0).unwrap();
        u16::from_le_bytes(
            self.bytes[rs[1].payload + 26..rs[1].payload + 28]
                .try_into()
                .unwrap(),
        )
    }

    /// A canonical numeric definition before display rounding. Its use as a
    /// native mode-zero caption is established for centimeter literals.
    pub fn literal_definition(&self) -> String {
        format!("{}{}", self.value(), self.unit().suffix())
    }

    pub fn precision(&self) -> usize {
        let rs = records(&self.bytes, 0).unwrap();
        u16::from_le_bytes(
            self.bytes[rs[1].payload + 28..rs[1].payload + 30]
                .try_into()
                .unwrap(),
        ) as usize
    }
    pub fn unit(&self) -> NumericUnit {
        match u16::from_le_bytes(self.bytes[16..18].try_into().unwrap()) {
            80 => NumericUnit::Length,
            81 => NumericUnit::Degrees,
            _ => NumericUnit::Scalar,
        }
    }
    pub(crate) fn native_unit_scale(&self) -> f64 {
        match self.unit() {
            NumericUnit::Length => 37.79527559055118,
            NumericUnit::Degrees => CALCULATOR_PI / 180.,
            _ => 1.,
        }
    }
    /// The native parameter-curve descriptor identifies its driver value type.
    /// Angle ranges are radians; scalar ranges are plain numbers. Length curves
    /// remain unsupported until their descriptor is established natively.
    pub(crate) fn locus_unit_tag(&self) -> Option<u32> {
        match self.unit() {
            NumericUnit::Scalar => Some(1),
            NumericUnit::Degrees => Some(0x0001_0000),
            _ => None,
        }
    }
    fn literal(&self) -> ParameterLiteral {
        let rs = records(&self.bytes, 0).unwrap();
        let words = self.bytes[rs[1].payload + 92..rs[1].end]
            .chunks_exact(2)
            .map(|word| u16::from_le_bytes(word.try_into().unwrap()))
            .collect::<Vec<_>>();
        parameter_literal(&words, self.unit()).unwrap()
    }
    pub fn value(&self) -> f64 {
        if let Some(value) = self.evaluation_value {
            return f64::from_le_bytes(value);
        }
        let literal = self.literal();
        if literal.suffix == Some(0x0100) {
            literal.value / self.native_unit_scale()
        } else {
            literal.value
        }
    }

    pub fn value_text(&self) -> String {
        format!(
            "{}{}",
            format_numeric_value(self.value(), self.precision()),
            self.unit().suffix()
        )
    }
    pub fn animation_domain(&self) -> (f64, f64) {
        let rs = records(&self.bytes, 0).unwrap();
        let payload = &self.bytes[rs[1].payload..rs[1].end];
        (
            f64::from_le_bytes(payload[60..68].try_into().unwrap()) / self.native_unit_scale(),
            f64::from_le_bytes(payload[68..76].try_into().unwrap()) / self.native_unit_scale(),
        )
    }

    pub fn set_value(&mut self, value: f64) -> Result<(), String> {
        if self.evaluation_value.is_none() && value == self.value() {
            return Ok(());
        }
        let literal = self.literal();
        let mut words = decimal_words(value)?;
        let fractional_digits = words
            .iter()
            .position(|word| *word == 10)
            .map_or(0, |at| words.len() - at - 1);
        let precision = self.precision().max(fractional_digits.min(5));
        if literal.scientific {
            // Shift decimal digits rather than dividing floats: 0.000025 must
            // retain the exact mantissa 2.5, not 2.4999999999999996.
            words = shift_decimal_words(&words, 5);
        } else if literal.suffix == Some(0x0100) {
            // Public parameter values and display precision are degrees. Only
            // the preserved literal encoding uses radians.
            words = decimal_words(value * self.native_unit_scale())?;
        }
        let old_value = self.value();
        let extra = |value: f64| {
            let text = value.abs().to_string();
            let digits = text.split('.').next().unwrap().len();
            (digits.saturating_sub(1)) as i32 + i32::from(value < 0.)
        };
        let width_delta = 4 * (extra(value) - extra(old_value))
            + 10 * (precision as i32 - self.precision() as i32);
        let rs = records(&self.bytes, 0)?;
        let mut payload = self.bytes[rs[1].payload..rs[1].end].to_vec();
        payload.truncate(92);
        payload[28..30].copy_from_slice(&(precision as u16).to_le_bytes());
        let precision_steps = [1_f64, 0.1, 0.01, 0.001, 0.0001, 0.00001];
        let step = f64::from_le_bytes(payload[52..60].try_into().unwrap());
        if step == precision_steps[self.precision()] * self.native_unit_scale() {
            payload[52..60].copy_from_slice(
                &(precision_steps[precision] * self.native_unit_scale()).to_le_bytes(),
            );
        }
        match self.unit() {
            NumericUnit::Length => words.push(0x0201),
            NumericUnit::Degrees => words.push(literal.suffix.unwrap()),
            _ => {}
        }
        if literal.scientific {
            words.extend_from_slice(&[0x1002, 1, 0, 0x1004, 0x1001, 5]);
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
        let right = changed.iter().find(|r| r.kind == 0x08a3).unwrap().payload + 4;
        let old_right = i16::from_le_bytes(updated[right..right + 2].try_into().unwrap());
        let new_right = i16::try_from(i32::from(old_right) + width_delta)
            .map_err(|_| "Parameter box coordinate overflow")?;
        updated[right..right + 2].copy_from_slice(&new_right.to_le_bytes());
        Self::parse(&updated)?;
        self.bytes = updated;
        self.evaluation_value = None;
        Ok(())
    }
    pub(crate) fn set_evaluation_value(&mut self, value: f64) -> bool {
        if !value.is_finite() {
            return false;
        }
        self.evaluation_value = Some(value.to_le_bytes());
        true
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
        let at = rs.iter().find(|r| r.kind == 0x08a3).unwrap().payload;
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
        let bbox = rs.iter().find(|r| r.kind == 0x08a3).unwrap().payload;
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeasuredCalculationGroup {
    bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LengthPairCalculationGroup {
    bytes: Vec<u8>,
}

impl LengthPairCalculationGroup {
    pub fn set_precision(&mut self, precision: u16) -> Result<(), String> {
        let updated = calculation_bytes_with_precision(&self.bytes, precision)?;
        *self = Self::parse(&updated)?;
        Ok(())
    }
    pub fn precision(&self) -> usize {
        calculation_precision(&self.bytes).unwrap()
    }

    pub fn new(
        first: u32,
        second: u32,
        subtract: bool,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        Self::with_operator(first, second, u8::from(subtract), anchor)
    }
    pub fn with_operator(
        first: u32,
        second: u32,
        operator: u8,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        if operator > 3 {
            return Err("Use +, -, *, or /".into());
        }
        let mut bytes = include_bytes!("../../../assets/length_sum_pair_group.bin").to_vec();
        bytes[48..52].copy_from_slice(&first.to_le_bytes());
        bytes[52..56].copy_from_slice(&second.to_le_bytes());
        bytes[76..78].copy_from_slice(&anchor.0.to_le_bytes());
        bytes[78..80].copy_from_slice(&anchor.1.to_le_bytes());
        bytes[158..160].copy_from_slice(&(0x1000u16 + u16::from(operator)).to_le_bytes());
        Self::parse(&bytes)
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        calculation_precision(bytes)?;
        let template = include_bytes!("../../../assets/length_sum_pair_group.bin");
        let rs = records(bytes, 0)?;
        if bytes.len() != template.len()
            || rs.iter().map(|r| r.kind).collect::<Vec<_>>()
                != [0x07d0, 0x07d2, 0x0907, 0x07d8, 0x07d7]
            || rs.iter().map(|r| r.end - r.payload).collect::<Vec<_>>() != [28, 12, 98, 2, 0]
            || bytes[48..52] == [0; 4]
            || bytes[52..56] == [0; 4]
            || !matches!(
                u16::from_le_bytes(bytes[158..160].try_into().unwrap()),
                0x1000..=0x1003
            )
        {
            return Err("Unsupported two-length calculation".into());
        }
        for (i, (&a, &b)) in bytes.iter().zip(template).enumerate() {
            let mutable = i == 10
                || (26..28).contains(&i)
                || (48..56).contains(&i)
                || (76..156).contains(&i)
                || (158..160).contains(&i);
            if !mutable && a != b {
                return Err("Unsupported two-length calculation style".into());
            }
        }
        // Length and token counts must agree even when the native cache varies.
        if u32::from_le_bytes(bytes[100..104].try_into().unwrap()) != 66
            || u16::from_le_bytes(bytes[108..110].try_into().unwrap()) != 3
        {
            return Err("Invalid two-length expression size".into());
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }
    pub fn subtracts(&self) -> bool {
        self.bytes[158] == 1
    }
    pub fn operator(&self) -> u8 {
        self.bytes[158]
    }
    pub fn operator_text(&self) -> &'static str {
        match self.operator() {
            0 => "+",
            1 => "−",
            2 => "·",
            _ => "÷",
        }
    }
    pub fn length_power(&self) -> u8 {
        match self.operator() {
            0 | 1 => 1,
            2 => 2,
            _ => 0,
        }
    }
    pub fn value(&self, a: f64, b: f64) -> Option<f64> {
        let value = match self.operator() {
            0 => a + b,
            1 => a - b,
            2 => a * b,
            3 if b != 0. => a / b,
            _ => return None,
        };
        value.is_finite().then_some(value)
    }
    pub fn sources(&self) -> [u32; 2] {
        [
            u32::from_le_bytes(self.bytes[48..52].try_into().unwrap()),
            u32::from_le_bytes(self.bytes[52..56].try_into().unwrap()),
        ]
    }
    pub fn remap_sources(&self, first: u32, second: u32) -> Result<Self, String> {
        let mut bytes = self.bytes.clone();
        bytes[48..52].copy_from_slice(&first.to_le_bytes());
        bytes[52..56].copy_from_slice(&second.to_le_bytes());
        Self::parse(&bytes)
    }
    pub fn anchor(&self) -> (i16, i16) {
        (
            i16::from_le_bytes(self.bytes[76..78].try_into().unwrap()),
            i16::from_le_bytes(self.bytes[78..80].try_into().unwrap()),
        )
    }
    pub fn set_anchor(&mut self, anchor: (i16, i16)) -> Result<(), String> {
        self.bytes[76..78].copy_from_slice(&anchor.0.to_le_bytes());
        self.bytes[78..80].copy_from_slice(&anchor.1.to_le_bytes());
        Self::parse(&self.bytes)?;
        Ok(())
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

pub use sketchpad_ir::NumericUnit;

// Native calculation subtype belongs to the engine's record codec.
trait NumericUnitWire {
    fn subtype(self) -> u16;
}
impl NumericUnitWire for NumericUnit {
    fn subtype(self) -> u16 {
        match self {
            Self::Length => 48,
            Self::Degrees => 49,
            Self::Area => 50,
            _ => 51,
        }
    }
}

// Native token counts are u16, independently observed above 256 (339 tokens).
// Bound expression work separately from the 256-addressable source operands.
const MAX_CALCULATION_TOKENS: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterCalculationGroup {
    bytes: Vec<u8>,
    // Native subtype 51 covers generic quantities, including higher unit powers.
    inferred_unit: Option<NumericUnit>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalculationSourceKind {
    Numeric,
    Function,
}

// Native calculator degree conversion, verified with tan(90).
pub(crate) const CALCULATOR_PI: f64 = 3.1415926535898;

pub(crate) const CALCULATION_FUNCTIONS: &[(&str, u16)] = &[
    ("sin", 0x2000),
    ("cos", 0x2001),
    ("tan", 0x2002),
    ("asin", 0x2003),
    ("acos", 0x2004),
    ("atan", 0x2005),
    ("abs", 0x2006),
    ("sqrt", 0x2007),
    ("ln", 0x2008),
    ("log", 0x2009),
    ("sgn", 0x200a),
    ("round", 0x200b),
    ("trunc", 0x200c),
];

fn calculation_function_name(token: u16) -> Option<&'static str> {
    CALCULATION_FUNCTIONS
        .iter()
        .find_map(|(name, code)| (*code == token).then_some(*name))
}

#[derive(Clone, Debug)]
enum ParameterExpr {
    Source(usize),
    FunctionSource(usize, Box<ParameterExpr>),
    Literal(f64, String),
    Binary(u16, Box<ParameterExpr>, Box<ParameterExpr>),
    Call(u16, Box<ParameterExpr>),
    Neg(Box<ParameterExpr>),
    Group(Box<ParameterExpr>),
}

impl ParameterExpr {
    fn source_kinds(&self, kinds: &mut [Option<CalculationSourceKind>]) -> Result<(), String> {
        use CalculationSourceKind::{Function, Numeric};
        match self {
            Self::Source(index) | Self::FunctionSource(index, _) => {
                let kind = if matches!(self, Self::Source(_)) {
                    Numeric
                } else {
                    Function
                };
                let slot = kinds
                    .get_mut(*index)
                    .ok_or("Invalid calculation source token")?;
                if slot.is_some_and(|previous| previous != kind) {
                    return Err("Calculation source is used as both numeric and function".into());
                }
                *slot = Some(kind);
                if let Self::FunctionSource(_, argument) = self {
                    argument.source_kinds(kinds)?;
                }
            }
            Self::Binary(_, left, right) => {
                left.source_kinds(kinds)?;
                right.source_kinds(kinds)?;
            }
            Self::Neg(argument) | Self::Group(argument) | Self::Call(_, argument) => {
                argument.source_kinds(kinds)?
            }
            Self::Literal(_, _) => {}
        }
        Ok(())
    }
    fn eval(&self, source: f64) -> Option<f64> {
        let value = self.eval_raw(source)?;
        value.is_finite().then_some(value)
    }
    fn eval_raw(&self, source: f64) -> Option<f64> {
        self.eval_raw_values(&[source])
    }
    fn eval_raw_values(&self, sources: &[f64]) -> Option<f64> {
        self.eval_raw_context(sources, &[], &|_, _, _| None)
    }
    fn eval_raw_context(
        &self,
        sources: &[f64],
        units: &[NumericUnit],
        functions: &dyn Fn(usize, f64, NumericUnit) -> Option<f64>,
    ) -> Option<f64> {
        let value = match self {
            Self::Source(index) => *sources.get(*index)?,
            Self::FunctionSource(index, argument) => functions(
                *index,
                argument.eval_raw_context(sources, units, functions)?,
                argument.units(units).ok()?,
            )?,
            Self::Neg(argument) => -argument.eval_raw_context(sources, units, functions)?,
            Self::Group(argument) => argument.eval_raw_context(sources, units, functions)?,
            Self::Literal(value, _) => *value,
            Self::Call(function, argument) => {
                let value = argument.eval_raw_context(sources, units, functions)?;
                // Native calculator PI constant, present at executable offset 0x272690.
                let angle = value / 180. * CALCULATOR_PI;
                match function {
                    0x2000 => angle.sin(),
                    0x2001 => angle.cos(),
                    0x2002 => angle.tan(),
                    0x2003 => value.asin() / CALCULATOR_PI * 180.,
                    0x2004 => value.acos() / CALCULATOR_PI * 180.,
                    0x2005 => value.atan() / CALCULATOR_PI * 180.,
                    0x2006 => value.abs(),
                    0x2007 => value.sqrt(),
                    0x2008 => value.ln(),
                    0x2009 => value.log10(),
                    0x200a => {
                        if value == 0. {
                            0.
                        } else {
                            value.signum()
                        }
                    }
                    0x200b => (value + 0.5).floor(),
                    0x200c => value.trunc(),
                    _ => return None,
                }
            }
            Self::Binary(operator, left, right) => {
                let left = left.eval_raw_context(sources, units, functions)?;
                let right = right.eval_raw_context(sources, units, functions)?;
                match operator {
                    0x1000 => left + right,
                    0x1001 => left - right,
                    0x1002 => left * right,
                    0x1003 => left / right,
                    0x1004 => left.powf(right),
                    _ => return None,
                }
            }
        };
        Some(value)
    }

    fn unit(&self, source: NumericUnit) -> Result<NumericUnit, String> {
        self.units(&[source])
    }
    fn units(&self, sources: &[NumericUnit]) -> Result<NumericUnit, String> {
        use NumericUnit::{Degrees, Length, Scalar};
        match self {
            Self::FunctionSource(index, argument) => {
                sources
                    .get(*index)
                    .ok_or("Missing calculation function source")?;
                argument.units(sources)?;
                Ok(Scalar)
            }
            Self::Source(index) => sources
                .get(*index)
                .copied()
                .ok_or_else(|| "Missing calculation source units".into()),
            Self::Neg(argument) | Self::Group(argument) => argument.units(sources),
            Self::Literal(_, text) => Ok(if text.ends_with('°') {
                Degrees
            } else if text.ends_with(" cm") || text.ends_with(" px") {
                Length
            } else {
                Scalar
            }),
            Self::Call(function, argument) => {
                let unit = argument.units(sources)?;
                match *function {
                    0x2000..=0x2002 if matches!(unit, Scalar | Degrees) => Ok(Scalar),
                    0x2003..=0x2005 if unit == Scalar => Ok(Degrees),
                    0x2006 => Ok(unit),
                    0x2007 => {
                        let (length, degrees) = unit.dimensions();
                        Ok(if length % 2 == 0 && degrees % 2 == 0 {
                            NumericUnit::from_dimensions(length / 2, degrees / 2)
                        } else {
                            Scalar
                        })
                    }
                    0x2008..=0x200a => Ok(Scalar),
                    0x200b..=0x200c => Ok(unit),
                    _ => Err("Function uses incompatible or unsupported units".into()),
                }
            }
            Self::Binary(op, a, b) => {
                let (left, right) = (a.units(sources)?, b.units(sources)?);
                match *op {
                    0x1000 | 0x1001 if left == right => Ok(left),
                    // Native adds unlike quantities by magnitude and drops
                    // their units, including scalar variables and compound units.
                    0x1000 => Ok(Scalar),
                    // The same magnitude rule applies to subtraction, including
                    // bare decimals. Matching quantities retain their units.
                    0x1001 => Ok(Scalar),
                    0x1002 | 0x1003 => {
                        let (al, ad) = left.dimensions();
                        let (bl, bd) = right.dimensions();
                        let dimensions = if *op == 0x1002 {
                            al.checked_add(bl).zip(ad.checked_add(bd))
                        } else {
                            al.checked_sub(bl).zip(ad.checked_sub(bd))
                        };
                        dimensions
                            .map(|(l, d)| NumericUnit::from_dimensions(l, d))
                            .ok_or_else(|| "Calculation unit exponent overflow".into())
                    }
                    0x1004 => {
                        // An exponent supplies its numeric magnitude, even if
                        // its source/literal carries physical or angle units.
                        if left == Scalar || right != Scalar {
                            return Ok(Scalar);
                        }
                        // Native retains dimensions only for a literal exponent.
                        // Source values, computations, and parenthesized literals
                        // discard units even when their numeric value is an integer.
                        let power = match &**b {
                            Self::Literal(value, _) => *value,
                            _ => return Ok(Scalar),
                        };
                        if !power.is_finite() {
                            return Err("Calculation unit exponent is undefined".into());
                        }
                        if power.fract() != 0. {
                            return Ok(Scalar);
                        }
                        if power < i32::MIN as f64 || power > i32::MAX as f64 {
                            return Err("Calculation unit exponent overflow".into());
                        }
                        let (length, degrees) = left.dimensions();
                        length
                            .checked_mul(power as i32)
                            .zip(degrees.checked_mul(power as i32))
                            .map(|(l, d)| NumericUnit::from_dimensions(l, d))
                            .ok_or_else(|| "Calculation unit exponent overflow".into())
                    }
                    _ => Err("Calculation uses incompatible or unsupported units".into()),
                }
            }
        }
    }

    fn caption_layout(
        &self,
        base: NumericCaptionLayout,
        sources: &[NumericCaptionLayout],
    ) -> Result<NumericCaptionLayout, String> {
        match self {
            Self::Source(index) => sources
                .get(*index)
                .copied()
                .ok_or_else(|| "Missing numeric caption source layout".into()),
            Self::Literal(_, text) if !text.contains('/') && !text.contains('÷') => Ok(base),
            Self::Neg(argument) | Self::Group(argument) => argument.caption_layout(base, sources),
            Self::Binary(op, left, right) if matches!(*op, 0x1000..=0x1003) => {
                let left = left.caption_layout(base, sources)?;
                let right = right.caption_layout(base, sources)?;
                if *op == 0x1003 {
                    Ok(NumericCaptionLayout {
                        above: left
                            .height()?
                            .checked_add(1)
                            .ok_or("Numeric caption layout overflow")?,
                        below: right
                            .height()?
                            .checked_add(2)
                            .ok_or("Numeric caption layout overflow")?,
                    })
                } else {
                    Ok(NumericCaptionLayout {
                        above: left.above.max(right.above),
                        below: left.below.max(right.below),
                    })
                }
            }
            _ => Err("Numeric caption expression layout needs a native control".into()),
        }
    }

    fn caption_is_flat(&self) -> bool {
        match self {
            Self::Source(_) | Self::Literal(_, _) => true,
            Self::Neg(argument) | Self::Group(argument) => argument.caption_is_flat(),
            Self::Binary(op, left, right) => {
                matches!(*op, 0x1000..=0x1002) && left.caption_is_flat() && right.caption_is_flat()
            }
            Self::Call(_, _) | Self::FunctionSource(_, _) => false,
        }
    }

    fn display(&self, source_name: &str) -> String {
        self.display_names(&[source_name])
    }
    fn display_names(&self, source_names: &[&str]) -> String {
        match self {
            Self::FunctionSource(index, argument) => format!(
                "{}({})",
                source_names.get(*index).copied().unwrap_or("?"),
                argument.display_names(source_names)
            ),
            Self::Source(index) => source_names.get(*index).copied().unwrap_or("?").into(),
            Self::Neg(argument) => format!("−{}", argument.display_names(source_names)),
            Self::Group(argument) => argument.display_names(source_names),
            Self::Literal(_, text) => text.clone(),
            Self::Call(function, argument) => format!(
                "{}({})",
                calculation_function_name(*function).unwrap(),
                argument.display_names(source_names)
            ),
            Self::Binary(operator, left, right) => format!(
                "({} {} {})",
                left.display_names(source_names),
                match operator {
                    0x1000 => "+",
                    0x1001 => "−",
                    0x1002 => "·",
                    0x1003 => "÷",
                    0x1004 => "^",
                    _ => "?",
                },
                right.display_names(source_names)
            ),
        }
    }
}

struct ParameterExprParser<'a> {
    words: &'a [u16],
    at: usize,
}

impl<'a> ParameterExprParser<'a> {
    fn expression(&mut self) -> Result<ParameterExpr, String> {
        self.binary(0)
    }
    fn binary(&mut self, min_precedence: u8) -> Result<ParameterExpr, String> {
        let mut left = self.atom()?;
        while let Some(&operator @ (0x1000..=0x1004)) = self.words.get(self.at) {
            let precedence = match operator {
                0x1000 | 0x1001 => 1,
                0x1002 | 0x1003 => 2,
                _ => 3,
            };
            if precedence < min_precedence {
                break;
            }
            self.at += 1;
            let next = if operator == 0x1004 {
                precedence
            } else {
                precedence + 1
            };
            let right = self.binary(next)?;
            left = ParameterExpr::Binary(operator, Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn atom(&mut self) -> Result<ParameterExpr, String> {
        let word = *self
            .words
            .get(self.at)
            .ok_or("Missing calculation operand")?;
        if matches!(word, 0x1000 | 0x1001) {
            self.at += 1;
            let argument = self.binary(3)?;
            return Ok(if word == 0x1001 {
                ParameterExpr::Neg(Box::new(argument))
            } else {
                argument
            });
        }
        if calculation_function_name(word).is_some() || (0x7000..=0x70ff).contains(&word) {
            self.at += 1;
            // A function has an implicit opener. A following token 11 is a
            // normal grouped subexpression; its trailing operators still belong
            // to the argument until the function's one unmatched closing token.
            let argument = self.expression()?;
            if self.words.get(self.at) != Some(&12) {
                return Err("Unclosed function argument".into());
            }
            self.at += 1;
            // Our writer wraps an entire argument in one explicit group. Avoid
            // adding a redundant pair when displaying that established layout.
            let argument = match argument {
                ParameterExpr::Group(inner) => *inner,
                other => other,
            };
            return Ok(if word >= 0x7000 {
                ParameterExpr::FunctionSource((word - 0x7000) as usize, Box::new(argument))
            } else {
                ParameterExpr::Call(word, Box::new(argument))
            });
        }
        if word == 13 || word == 14 {
            self.at += 1;
            if word == 13 && self.words.get(self.at) == Some(&0x0100) {
                self.at += 1;
                // Native captions show pi radians as 180 degrees, while the
                // original pi/unit words remain untouched in the group.
                return Ok(ParameterExpr::Literal(180., "180°".into()));
            }
            return Ok(ParameterExpr::Literal(
                if word == 13 {
                    CALCULATOR_PI
                } else {
                    std::f64::consts::E
                },
                if word == 13 { "π" } else { "e" }.into(),
            ));
        }
        if word == 11 {
            self.at += 1;
            let result = self.expression()?;
            if self.words.get(self.at) != Some(&12) {
                return Err("Unclosed calculation parenthesis".into());
            }
            self.at += 1;
            return Ok(ParameterExpr::Group(Box::new(result)));
        }
        if (0x6000..=0x60ff).contains(&word) {
            self.at += 1;
            return Ok(ParameterExpr::Source((word - 0x6000) as usize));
        }
        if word <= 10 {
            let mut digits = 0;
            let mut text = String::new();
            let mut terminal_decimal = false;
            while let Some(&digit @ 0..=9) = self.words.get(self.at) {
                digits += 1;
                text.push(char::from(b'0' + digit as u8));
                self.at += 1;
            }
            if digits > 1 && word == 0 {
                return Err("Leading zero in calculation".into());
            }
            if self.words.get(self.at) == Some(&10) {
                self.at += 1;
                text.push('.');
                let fraction_start = self.at;
                while let Some(&digit @ 0..=9) = self.words.get(self.at) {
                    text.push(char::from(b'0' + digit as u8));
                    self.at += 1;
                }
                if self.at == fraction_start {
                    if digits == 0 {
                        return Err("Decimal point needs fractional digits".into());
                    }
                    // Actual source calculations retain a terminal separator
                    // in their words but native captions omit it (240. -> 240).
                    terminal_decimal = true;
                    text.pop();
                }
            }
            let mut value = text
                .parse::<f64>()
                .map_err(|_| "Invalid calculation literal")?;
            if self.words.get(self.at) == Some(&0x0100) {
                if terminal_decimal {
                    return Err("Unsupported radian calculation literal".into());
                }
                // These are the authentic decimal-radian literals observed in
                // the corpus. Independent native controls establish the full
                // degree value and these six-significant-digit captions; the
                // literal caption does not follow the result's precision.
                let caption = match text.as_str() {
                    "1" => "57.2958°",
                    "2" => "114.592°",
                    "3" => "171.887°",
                    _ => return Err("Unsupported radian calculation literal".into()),
                };
                value = value * 180. / CALCULATOR_PI;
                text = caption.into();
                self.at += 1;
            } else if self.words.get(self.at) == Some(&0x0101) {
                if terminal_decimal {
                    return Err("Trailing degree calculation literals are not yet supported".into());
                }
                self.at += 1;
                text.push('°');
            } else if self.words.get(self.at) == Some(&0x0201) {
                // Native centimeter literals retain their numeric magnitude;
                // the suffix supplies Length independently of the result type.
                self.at += 1;
                text.push_str(" cm");
            } else if self.words.get(self.at) == Some(&0x0200) {
                // Native length quantities use centimeters internally; the
                // pixel suffix converts the literal magnitude into that unit.
                self.at += 1;
                value /= 37.79527559055118;
                text.push_str(" px");
            }
            return Ok(ParameterExpr::Literal(value, text));
        }
        Err("Unsupported parameter calculation token".into())
    }
}

fn calculation_words(expression: &str, source_name: &str) -> Result<Vec<u16>, String> {
    let names = [(source_name, 0)];
    calculation_words_for_sources(
        expression,
        if source_name.is_empty() { &[] } else { &names },
    )
}
fn calculation_words_for_sources(
    expression: &str,
    source_names: &[(&str, usize)],
) -> Result<Vec<u16>, String> {
    let mut words = Vec::new();
    let mut input = expression;
    let mut parentheses = Vec::new();
    while !input.is_empty() {
        let ch = input.chars().next().unwrap();
        if ch.is_whitespace() {
            input = &input[ch.len_utf8()..];
        } else if let Some((name, code)) = CALCULATION_FUNCTIONS.iter().find(|(name, _)| {
            input
                .strip_prefix(*name)
                .is_some_and(|rest| rest.trim_start().starts_with('('))
        }) {
            words.extend([*code, 11]);
            input = input[name.len()..].trim_start();
            input = &input[1..];
            parentheses.push(true);
        } else if ch == '(' {
            words.push(11);
            parentheses.push(false);
            input = &input[1..];
        } else if ch == ')' {
            words.push(12);
            if parentheses.pop().ok_or("Unexpected closing parenthesis")? {
                words.push(12);
            }
            input = &input[1..];
        } else if let Some((source_name, index)) = source_names.iter().find(|(name, _)| {
            input.strip_prefix(*name).is_some_and(|tail| {
                !tail
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
            })
        }) {
            let rest = &input[source_name.len()..];
            if source_name.starts_with('f') && rest.trim_start().starts_with('(') {
                words.push(0x7000 + *index as u16);
                input = &rest.trim_start()[1..];
                parentheses.push(false);
            } else {
                words.push(0x6000 + *index as u16);
                input = rest;
            }
        } else if input.strip_prefix("rad").is_some_and(|rest| {
            !rest
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        }) {
            words.push(0x0100);
            input = &input[3..];
        } else if input.strip_prefix("cm").is_some_and(|rest| {
            !rest
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        }) {
            words.push(0x0201);
            input = &input[2..];
        } else if input.strip_prefix("px").is_some_and(|rest| {
            !rest
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        }) {
            words.push(0x0200);
            input = &input[2..];
        } else if ch == 'π' || input.starts_with("pi") {
            words.push(13);
            input = &input[if ch == 'π' { ch.len_utf8() } else { 2 }..];
        } else if ch == 'e' {
            words.push(14);
            input = &input[1..];
        } else if ch.is_ascii_digit() {
            words.push((ch as u8 - b'0') as u16);
            input = &input[1..];
        } else {
            words.push(match ch {
                '+' => 0x1000,
                '-' | '−' => 0x1001,
                '*' | '×' | '·' => 0x1002,
                '/' | '÷' => 0x1003,
                '^' => 0x1004,
                '(' => 11,
                ')' => 12,
                '.' => 10,
                '°' => 0x0101,
                _ => return Err("Unsupported calculation character".into()),
            });
            input = &input[ch.len_utf8()..];
        }
    }
    if !(1..=MAX_CALCULATION_TOKENS).contains(&words.len()) {
        return Err(format!(
            "Calculation must use 1–{MAX_CALCULATION_TOKENS} tokens"
        ));
    }
    Ok(words)
}

fn calculation_tree(words: &[u16]) -> Result<ParameterExpr, String> {
    let mut parser = ParameterExprParser { words, at: 0 };
    let expression = parser.expression()?;
    if parser.at != words.len() {
        return Err("Unexpected calculation token".into());
    }
    Ok(expression)
}

impl ParameterCalculationGroup {
    pub(crate) fn with_label(&self, label: &crate::text_records::LabelAux) -> Result<Self, String> {
        let entries = records(&self.bytes, 0)?;
        if entries.iter().any(|record| record.kind == 0x07d5
            && &self.bytes[record.payload..record.payload + u32::from_le_bytes(self.bytes[record.start..record.start + 4].try_into().unwrap()) as usize] == label.payload()) {
            return Ok(self.clone());
        }
        let has_label = entries.iter().any(|record| record.kind == 0x07d5);
        let mut bytes = Vec::new();
        for record in entries {
            if record.kind == 0x07d5 || (!has_label && record.kind == 0x07d8) {
                put_record(&mut bytes, 0x07d5, label.payload());
                if record.kind == 0x07d5 { continue; }
            }
            bytes.extend_from_slice(&self.bytes[record.start..record.end]);
        }
        let mut parsed = Self::parse(&bytes)?;
        parsed.inferred_unit = self.inferred_unit;
        Ok(parsed)
    }

    pub fn set_precision(&mut self, precision: u16) -> Result<(), String> {
        let updated = calculation_bytes_with_precision(&self.bytes, precision)?;
        let mut parsed = Self::parse(&updated)?;
        parsed.inferred_unit = self.inferred_unit;
        *self = parsed;
        Ok(())
    }

    pub fn from_expression(
        source: u32,
        source_name: &str,
        expression: &str,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        Self::from_expression_with_source_unit(
            source,
            source_name,
            expression,
            anchor,
            NumericUnit::Scalar,
        )
    }
    pub fn from_expression_with_source_unit(
        source: u32,
        source_name: &str,
        expression: &str,
        anchor: (i16, i16),
        source_unit: NumericUnit,
    ) -> Result<Self, String> {
        if source == 0 || source_name.is_empty() {
            return Err("Select a parameter".into());
        }
        let words = calculation_words(expression, source_name)?;
        let base = object_group(PARAMETER_CALCULATION_NATIVE, 2);
        let rs = records(&base, 0)?;
        let mut payload = base[rs[2].payload..rs[2].end].to_vec();
        payload.truncate(92);
        payload[36..40].copy_from_slice(&(60 + 2 * words.len() as u32).to_le_bytes());
        payload[44..46].copy_from_slice(&(words.len() as u16).to_le_bytes());
        for word in words {
            payload.extend_from_slice(&word.to_le_bytes());
        }
        let mut bytes = base[..rs[2].start].to_vec();
        put_record(&mut bytes, 0x0907, &payload);
        bytes.extend_from_slice(&base[rs[2].end..]);
        bytes[48..52].copy_from_slice(&source.to_le_bytes());
        bytes[72..74].copy_from_slice(&anchor.0.to_le_bytes());
        bytes[74..76].copy_from_slice(&anchor.1.to_le_bytes());
        let group = Self::parse(&bytes)?;
        let unit = group.expression_unit(source_unit)?;
        bytes[16..18].copy_from_slice(&unit.subtype().to_le_bytes());
        let mut parsed = Self::parse(&bytes)?;
        if unit.subtype() == 51 && unit != NumericUnit::Scalar {
            parsed.inferred_unit = Some(unit);
        }
        Ok(parsed)
    }
    pub fn from_sources(
        sources: &[u32],
        expression: &str,
        anchor: (i16, i16),
        units: &[NumericUnit],
    ) -> Result<Self, String> {
        if !(1..=256).contains(&sources.len())
            || sources.contains(&0)
            || sources.len() != units.len()
        {
            return Err("Select 2–256 numeric sources".into());
        }
        let names = (1..=sources.len())
            .map(|n| format!("v{n}"))
            .collect::<Vec<_>>();
        let mut aliases = names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect::<Vec<_>>();
        aliases.extend([("a", 0), ("b", 1)]);
        let function_names = (1..=sources.len())
            .map(|n| format!("f{n}"))
            .collect::<Vec<_>>();
        aliases.extend(
            function_names
                .iter()
                .enumerate()
                .map(|(i, name)| (name.as_str(), i)),
        );
        let words = calculation_words_for_sources(expression, &aliases)?;
        let tree = calculation_tree(&words)?;
        if sources.len() == 1 {
            let mut kinds = vec![None];
            tree.source_kinds(&mut kinds)?;
            if kinds != [Some(CalculationSourceKind::Function)] {
                return Err("Select 2–256 numeric sources or one function".into());
            }
        }
        let unit = tree.units(units)?;
        let base = object_group(PARAMETER_CALCULATION_NATIVE, 2);
        let rs = records(&base, 0)?;
        let mut bytes = base[..rs[1].start].to_vec();
        let mut refs = (sources.len() as u32).to_le_bytes().to_vec();
        for id in sources {
            refs.extend_from_slice(&id.to_le_bytes());
        }
        put_record(&mut bytes, 0x07d2, &refs);
        let mut payload = base[rs[2].payload..rs[2].end].to_vec();
        payload.truncate(92);
        payload[12..14].copy_from_slice(&anchor.0.to_le_bytes());
        payload[14..16].copy_from_slice(&anchor.1.to_le_bytes());
        payload[36..40].copy_from_slice(&(60 + 2 * words.len() as u32).to_le_bytes());
        payload[44..46].copy_from_slice(&(words.len() as u16).to_le_bytes());
        for word in words {
            payload.extend_from_slice(&word.to_le_bytes());
        }
        put_record(&mut bytes, 0x0907, &payload);
        bytes.extend_from_slice(&base[rs[2].end..]);
        bytes[16..18].copy_from_slice(&unit.subtype().to_le_bytes());
        let mut parsed = Self::parse(&bytes)?;
        if unit.subtype() == 51 && unit != NumericUnit::Scalar {
            parsed.inferred_unit = Some(unit);
        }
        Ok(parsed)
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        calculation_precision(bytes)?;
        let rs = records(bytes, 0)?;
        let reference = rs.get(1).ok_or("Missing calculation references")?;
        let payload = &bytes[reference.payload..reference.end];
        if reference.kind != 0x07d2 || payload.len() < 8 {
            return Err("Missing calculation references".into());
        }
        let count = u32::from_le_bytes(payload[..4].try_into().unwrap()) as usize;
        if !(1..=256).contains(&count)
            || payload.len() != 4 + 4 * count
            || payload[4..].chunks_exact(4).any(|id| id == [0; 4])
        {
            return Err("Invalid calculation reference count".into());
        }
        let mut normalized = bytes[..reference.start].to_vec();
        let mut refs = 1_u32.to_le_bytes().to_vec();
        refs.extend_from_slice(&payload[4..8]);
        put_record(&mut normalized, 0x07d2, &refs);
        normalized.extend_from_slice(&bytes[reference.end..]);
        Self::validate_single_layout(&normalized)?;
        let group = Self {
            bytes: bytes.to_vec(),
            inferred_unit: None,
        };
        group.expression_tree()?;
        Ok(group)
    }
    fn validate_single_layout(bytes: &[u8]) -> Result<(), String> {
        let rs = records(bytes, 0)?;
        let kinds = rs.iter().map(|r| r.kind).collect::<Vec<_>>();
        let font_index = match kinds.as_slice() {
            [0x07d0, 0x07d2, 0x0907, 0x07d8, 0x07d7] => 3,
            [0x07d0, 0x07d2, 0x0907, 0x07d8, 0x232e, 0x07d7] if rs[4].payload == rs[4].end => 3,
            [0x07d0, 0x07d2, 0x0907, 0x07d5, 0x07d8, 0x07d7] => {
                crate::text_records::LabelAux::parse(&bytes[rs[3].payload..rs[3].end])?;
                4
            }
            [0x07d0, 0x07d2, 0x0907, 0x07d5, 0x07d8, 0x232e, 0x07d7]
                if rs[5].payload == rs[5].end =>
            {
                crate::text_records::LabelAux::parse(&bytes[rs[3].payload..rs[3].end])?;
                4
            }
            _ => return Err("Unsupported parameter calculation records".into()),
        };
        let payload = &bytes[rs[2].payload..rs[2].end];
        let token_count = payload.len().saturating_sub(92) / 2;
        if rs[0].end - rs[0].payload != 28
            || rs[1].end - rs[1].payload != 8
            || !(1..=MAX_CALCULATION_TOKENS).contains(&token_count)
            || payload.len() != 92 + 2 * token_count
            || rs[font_index].end - rs[font_index].payload != 2
            || rs.last().unwrap().end != rs.last().unwrap().payload
            || u32::from_le_bytes(bytes[8..12].try_into().unwrap()) & 0xffff != 48
            || !matches!(
                u16::from_le_bytes(bytes[16..18].try_into().unwrap()),
                48..=51
            )
            || u32::from_le_bytes(payload[36..40].try_into().unwrap()) as usize
                != payload.len() - 32
            || u16::from_le_bytes(payload[44..46].try_into().unwrap()) as usize != token_count
        {
            return Err("Unsupported parameter calculation header".into());
        }
        let template = object_group(PARAMETER_CALCULATION_NATIVE, 2);
        let template_records = records(&template, 0)?;
        let expected = &template[template_records[2].payload..template_records[2].end];
        for range in [0..12, 16..22, 24..26, 32..36, 40..44, 46..48] {
            if payload[range.clone()] != expected[range] {
                return Err("Unsupported calculation numeric representation".into());
            }
        }
        if !matches!(
            u16::from_le_bytes(payload[26..28].try_into().unwrap()),
            0 | 1 | 3
        ) {
            return Err("Unsupported calculation display mode".into());
        }
        Ok(())
    }
    fn expression_tree(&self) -> Result<ParameterExpr, String> {
        let rs = records(&self.bytes, 0)?;
        let words = self.bytes[rs[2].payload + 92..rs[2].end]
            .chunks_exact(2)
            .map(|word| u16::from_le_bytes(word.try_into().unwrap()))
            .collect::<Vec<_>>();
        let count = self.sources().len();
        if !words
            .iter()
            .any(|w| (0x6000..=0x60ff).contains(w) || (0x7000..=0x70ff).contains(w))
            || words.iter().any(|w| {
                ((0x6000..=0x60ff).contains(w) || (0x7000..=0x70ff).contains(w))
                    && (*w & 0xff) as usize >= count
            })
        {
            return Err("Invalid calculation source token".into());
        }
        let tree = calculation_tree(&words)?;
        tree.source_kinds(&mut vec![None; count])?;
        Ok(tree)
    }
    pub fn source_kinds(&self) -> Result<Vec<CalculationSourceKind>, String> {
        let mut kinds = vec![None; self.sources().len()];
        self.expression_tree()?.source_kinds(&mut kinds)?;
        // Retained unused references are still concrete numeric dependencies.
        Ok(kinds
            .into_iter()
            .map(|kind| kind.unwrap_or(CalculationSourceKind::Numeric))
            .collect())
    }
    pub fn value(&self, source: f64) -> Option<f64> {
        self.expression_tree().ok()?.eval(source)
    }
    pub fn value_with_source_unit(&self, source: f64, unit: NumericUnit) -> Option<f64> {
        let expression = self.expression_tree().ok()?;
        expression.unit(unit).ok()?;
        expression.eval(source)
    }
    pub fn expression(&self, source_name: &str) -> String {
        self.expression_tree().unwrap().display(source_name)
    }
    pub fn sources(&self) -> Vec<u32> {
        let rs = records(&self.bytes, 0).unwrap();
        self.bytes[rs[1].payload + 4..rs[1].end]
            .chunks_exact(4)
            .map(|id| u32::from_le_bytes(id.try_into().unwrap()))
            .collect()
    }
    pub fn source_id(&self) -> u32 {
        self.sources()[0]
    }
    pub fn value_for_sources(&self, values: &[f64], units: &[NumericUnit]) -> Option<f64> {
        self.display_value_for_sources(values, units)
            .filter(|value| value.is_finite())
    }
    /// Function indexes share the native reference list with numeric indexes.
    /// Their value slots are placeholders; the callback evaluates each call at
    /// its argument's current magnitude. Undefined calls suppress the result.
    pub fn value_for_sources_with_functions(
        &self,
        values: &[f64],
        units: &[NumericUnit],
        functions: &dyn Fn(usize, f64, NumericUnit) -> Option<f64>,
    ) -> Option<f64> {
        self.display_value_with_functions(values, units, functions)
            .filter(|value| value.is_finite())
    }
    pub(crate) fn display_value_with_functions(
        &self,
        values: &[f64],
        units: &[NumericUnit],
        functions: &dyn Fn(usize, f64, NumericUnit) -> Option<f64>,
    ) -> Option<f64> {
        if values.len() != self.sources().len()
            || units.len() != values.len()
            || values.iter().any(|value| !value.is_finite())
        {
            return None;
        }
        let tree = self.expression_tree().ok()?;
        if tree.units(units).ok()? != self.unit() {
            return None;
        }
        tree.eval_raw_context(values, units, functions)
    }
    /// Caption evaluation retains an infinite result from finite inputs.
    /// Geometry and downstream numeric sources still use `value_for_sources`.
    pub(crate) fn display_value_for_sources(
        &self,
        values: &[f64],
        units: &[NumericUnit],
    ) -> Option<f64> {
        self.display_value_with_functions(values, units, &|_, _, _| None)
    }
    pub fn expression_for_sources(&self, names: &[&str]) -> String {
        self.expression_tree().unwrap().display_names(names)
    }
    pub fn units_for_sources(&self, units: &[NumericUnit]) -> Result<NumericUnit, String> {
        self.expression_tree()?.units(units)
    }
    pub fn remap_sources(&self, ids: &[u32]) -> Result<Self, String> {
        if ids.len() != self.sources().len() || ids.contains(&0) {
            return Err("Invalid calculation sources".into());
        }
        let mut bytes = self.bytes.clone();
        let rs = records(&bytes, 0)?;
        for (index, id) in ids.iter().enumerate() {
            let at = rs[1].payload + 4 + 4 * index;
            bytes[at..at + 4].copy_from_slice(&id.to_le_bytes());
        }
        let mut parsed = Self::parse(&bytes)?;
        parsed.inferred_unit = self.inferred_unit;
        Ok(parsed)
    }
    /// Bind an abstract native tool calculation to concrete numeric sources.
    /// Native writes the result category after applying the expression's units.
    pub fn specialize_tool_sources(
        &self,
        ids: &[u32],
        units: &[NumericUnit],
    ) -> Result<Self, String> {
        if units.len() != ids.len() {
            return Err("Invalid calculation source units".into());
        }
        let remapped = self.remap_sources(ids)?;
        let unit = remapped.units_for_sources(units)?;
        let mut bytes = remapped.bytes;
        bytes[16..18].copy_from_slice(&unit.subtype().to_le_bytes());
        let mut group = Self::parse(&bytes)?;
        if unit.subtype() == 51 && unit != NumericUnit::Scalar {
            group.inferred_unit = Some(unit);
        }
        Ok(group)
    }

    pub fn with_source_units(mut self, units: &[NumericUnit]) -> Result<Self, String> {
        let inferred = self.units_for_sources(units)?;
        if inferred == self.unit() {
            return Ok(self);
        }
        let subtype = u16::from_le_bytes(self.bytes[16..18].try_into().unwrap());
        // Native squared angles use subtype 50 too; the expression supplies
        // the actual base unit rather than the quantity category in the header.
        // Native can retain an older physical category after a formula becomes
        // unitless (the retained length-minus-decimal control has subtype48 but
        // no native result suffix). Expression units govern its live value.
        if subtype == 51
            || (inferred == NumericUnit::Scalar && units.contains(&self.unit()))
            || (subtype == 50 && inferred.dimensions() == (0, 2))
        {
            self.inferred_unit = (inferred != self.unit()).then_some(inferred);
        } else if inferred != self.unit() {
            return Err("Calculation result units disagree with its header".into());
        }
        Ok(self)
    }

    pub fn with_visibility(&self, visible: bool) -> Result<Self, String> {
        let mut bytes = self.bytes.clone();
        bytes[10] = (bytes[10] & !1) | u8::from(!visible);
        let mut parsed = Self::parse(&bytes)?;
        parsed.inferred_unit = self.inferred_unit;
        Ok(parsed)
    }
    pub fn with_presentation(&self, visible: bool, suppressed: bool) -> Result<Self, String> {
        let mut parsed = self.with_visibility(visible)?;
        parsed.bytes[10] = (parsed.bytes[10] & !2) | (u8::from(suppressed) << 1);
        Ok(parsed)
    }
    pub fn unit(&self) -> NumericUnit {
        if let Some(unit) = self.inferred_unit {
            return unit;
        }
        match u16::from_le_bytes(self.bytes[16..18].try_into().unwrap()) {
            48 => NumericUnit::Length,
            49 => NumericUnit::Degrees,
            50 => NumericUnit::Area,
            _ => NumericUnit::Scalar,
        }
    }
    pub fn expression_unit(&self, source: NumericUnit) -> Result<NumericUnit, String> {
        self.expression_tree()?.unit(source)
    }
    pub fn display_mode(&self) -> u16 {
        let rs = records(&self.bytes, 0).unwrap();
        u16::from_le_bytes(
            self.bytes[rs[2].payload + 26..rs[2].payload + 28]
                .try_into()
                .unwrap(),
        )
    }
    /// Native flag2 suppresses the caption in the supported display modes.
    /// It is separate from visibility's flag1 bit.
    pub fn caption_suppressed(&self) -> bool {
        self.bytes[10] & 2 != 0
    }
    pub fn precision(&self) -> usize {
        let rs = records(&self.bytes, 0).unwrap();
        let at = rs[2].payload + 28;
        u16::from_le_bytes(self.bytes[at..at + 2].try_into().unwrap()) as usize
    }
    pub fn value_text(&self, value: f64) -> String {
        if value.is_nan() {
            return "undefined".into();
        }
        let number = if value == f64::INFINITY {
            "∞".into()
        } else if value == f64::NEG_INFINITY {
            "−∞".into()
        } else {
            format_numeric_value(value, self.precision())
        };
        format!("{number}{}", self.unit().suffix())
    }
    pub fn anchor(&self) -> (i16, i16) {
        let rs = records(&self.bytes, 0).unwrap();
        let at = rs[2].payload + 12;
        (
            i16::from_le_bytes(self.bytes[at..at + 2].try_into().unwrap()),
            i16::from_le_bytes(self.bytes[at + 2..at + 4].try_into().unwrap()),
        )
    }
    /// Preserve native expression and caches while mapping its local font.
    pub fn with_caption_font(&self, index: u16) -> Result<Self, String> {
        let mut bytes = self.bytes.clone();
        let rs = records(&bytes, 0)?;
        let at = rs
            .iter()
            .find(|record| record.kind == 0x07d8)
            .ok_or("Missing numeric caption font")?
            .payload;
        bytes[at..at + 2].copy_from_slice(&index.to_le_bytes());
        let mut group = Self::parse(&bytes)?;
        group.inferred_unit = self.inferred_unit;
        Ok(group)
    }

    pub(crate) fn expression_caption_layout(
        &self,
        base: NumericCaptionLayout,
        sources: &[NumericCaptionLayout],
    ) -> Result<NumericCaptionLayout, String> {
        if sources.len() != self.sources().len() {
            return Err("Invalid numeric caption source layouts".into());
        }
        self.expression_tree()?.caption_layout(base, sources)
    }

    pub(crate) fn caption_is_flat(&self) -> bool {
        self.expression_tree()
            .is_ok_and(|tree| tree.caption_is_flat())
    }

    pub fn set_anchor(&mut self, anchor: (i16, i16)) -> Result<(), String> {
        let rs = records(&self.bytes, 0)?;
        let at = rs[2].payload + 12;
        self.bytes[at..at + 2].copy_from_slice(&anchor.0.to_le_bytes());
        self.bytes[at + 2..at + 4].copy_from_slice(&anchor.1.to_le_bytes());
        Self::parse(&self.bytes)?;
        Ok(())
    }
    pub fn remap_source(&self, source: u32) -> Result<Self, String> {
        let mut ids = self.sources();
        ids[0] = source;
        self.remap_sources(&ids)
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

fn measured_factor(bytes: &[u8], digit_count: usize) -> u8 {
    let first = bytes[156];
    if digit_count == 2 {
        first * 10 + bytes[158]
    } else {
        first
    }
}

impl MeasuredCalculationGroup {
    pub fn set_precision(&mut self, precision: u16) -> Result<(), String> {
        let updated = calculation_bytes_with_precision(&self.bytes, precision)?;
        *self = Self::parse(&updated)?;
        Ok(())
    }
    pub fn precision(&self) -> usize {
        calculation_precision(&self.bytes).unwrap()
    }

    pub fn multiply(source: u32, factor: u8, anchor: (i16, i16)) -> Result<Self, String> {
        Self::scaled(source, factor, anchor, false)
    }
    pub fn divide(source: u32, divisor: u8, anchor: (i16, i16)) -> Result<Self, String> {
        Self::scaled(source, divisor, anchor, true)
    }
    fn scaled(source: u32, factor: u8, anchor: (i16, i16), divide: bool) -> Result<Self, String> {
        if source == 0 || factor > 99 {
            return Err("Select a measurement and a factor from 0 to 99".into());
        }
        if divide && factor == 0 {
            return Err("Cannot divide by zero".into());
        }
        let mut bytes = if factor < 10 {
            MEASURED_CALCULATION.to_vec()
        } else {
            MEASURED_CALCULATION_TWO_DIGIT.to_vec()
        };
        bytes[48..52].copy_from_slice(&source.to_le_bytes());
        if divide {
            bytes[154..156].copy_from_slice(&0x1003u16.to_le_bytes());
        }
        if factor < 10 {
            bytes[156..158].copy_from_slice(&u16::from(factor).to_le_bytes());
        } else {
            bytes[156..158].copy_from_slice(&u16::from(factor / 10).to_le_bytes());
            bytes[158..160].copy_from_slice(&u16::from(factor % 10).to_le_bytes());
        }
        bytes[72..74].copy_from_slice(&anchor.0.to_le_bytes());
        bytes[74..76].copy_from_slice(&anchor.1.to_le_bytes());
        Self::parse(&bytes)
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        calculation_precision(bytes)?;
        let template = match bytes.len() {
            176 => MEASURED_CALCULATION,
            178 => MEASURED_CALCULATION_TWO_DIGIT,
            _ => return Err("Unsupported measured calculation length".into()),
        };
        let digit_count = (bytes.len() - 174) / 2;
        let records = records(bytes, 0)?;
        if records.iter().map(|record| record.kind).collect::<Vec<_>>()
            != [0x07d0, 0x07d2, 0x0907, 0x07d8, 0x07d7]
            || records[0].end - records[0].payload != 28
            || records[1].end - records[1].payload != 8
            || records[2].end - records[2].payload != 96 + 2 * digit_count
            || records[3].end - records[3].payload != 2
            || records[4].end != records[4].payload
            || u32::from_le_bytes(bytes[8..12].try_into().unwrap()) & 0xffff != 48
            || u32::from_le_bytes(bytes[48..52].try_into().unwrap()) == 0
            || u32::from_le_bytes(bytes[96..100].try_into().unwrap())
                != (64 + 2 * digit_count) as u32
            || u16::from_le_bytes(bytes[104..106].try_into().unwrap()) != (2 + digit_count) as u16
            || bytes[152..154] != [0, 0x60]
            || !matches!(
                u16::from_le_bytes(bytes[154..156].try_into().unwrap()),
                0x1002 | 0x1003
            )
            || bytes[156 + 2 * digit_count..158 + 2 * digit_count] != [2, 0]
            || (0..digit_count).any(|digit| {
                u16::from_le_bytes(bytes[156 + digit * 2..158 + digit * 2].try_into().unwrap()) > 9
            })
            || (digit_count == 2 && bytes[156] == 0)
            || (bytes[154] == 3 && measured_factor(bytes, digit_count) == 0)
        {
            return Err("Unsupported measured calculation expression".into());
        }
        for (index, (&actual, &expected)) in bytes.iter().zip(template).enumerate() {
            let variable = index == 10
                || (26..28).contains(&index)
                || (48..52).contains(&index)
                || (72..76).contains(&index)
                || (76..152).contains(&index)
                || (154..156).contains(&index)
                || (156..156 + 2 * digit_count).contains(&index);
            if !variable && actual != expected {
                return Err("Unsupported measured calculation style".into());
            }
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    pub fn source_id(&self) -> u32 {
        u32::from_le_bytes(self.bytes[48..52].try_into().unwrap())
    }
    pub fn factor(&self) -> u8 {
        measured_factor(&self.bytes, (self.bytes.len() - 174) / 2)
    }
    pub fn divides(&self) -> bool {
        self.bytes[154] == 3
    }
    pub fn anchor(&self) -> (i16, i16) {
        (
            i16::from_le_bytes(self.bytes[72..74].try_into().unwrap()),
            i16::from_le_bytes(self.bytes[74..76].try_into().unwrap()),
        )
    }
    pub fn set_anchor(&mut self, anchor: (i16, i16)) -> Result<(), String> {
        self.bytes[72..74].copy_from_slice(&anchor.0.to_le_bytes());
        self.bytes[74..76].copy_from_slice(&anchor.1.to_le_bytes());
        Self::parse(&self.bytes)?;
        Ok(())
    }
    pub fn remap_source(&self, source: u32) -> Result<Self, String> {
        let mut result = self.clone();
        result.bytes[48..52].copy_from_slice(&source.to_le_bytes());
        Self::parse(&result.bytes)
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl CalculationGroup {
    pub fn display_mode(&self) -> u16 {
        let rs = records(&self.bytes, 0).unwrap();
        u16::from_le_bytes(
            self.bytes[rs[1].payload + 26..rs[1].payload + 28]
                .try_into()
                .unwrap(),
        )
    }

    /// The explicitly stored calculation name, without inventing a caption.
    pub fn name(&self) -> Option<&str> {
        let rs = records(&self.bytes, 0).ok()?;
        let label = rs.iter().find(|record| record.kind == 0x07d5)?;
        let len = u16::from_le_bytes(
            self.bytes[label.payload + 22..label.payload + 24]
                .try_into()
                .ok()?,
        ) as usize;
        std::str::from_utf8(&self.bytes[label.payload + 24..label.payload + 24 + len])
            .ok()
            .filter(|name| !name.is_empty())
    }

    /// Whether the source-free calculation uses the native angle header.
    pub fn is_native_radian_angle(&self) -> bool {
        self.bytes.get(16..18) == Some(&[49, 0])
    }

    /// Edit an expression while retaining its numeric unit and presentation
    /// records. A change of quantity family is a separate operation.
    pub fn set_expression(&mut self, expression: &str) -> Result<(), String> {
        let words = calculation_words(expression, "")?;
        if calculation_tree(&words)?.unit(NumericUnit::Scalar)? != self.unit() {
            return Err("Expression must retain the calculation's numeric unit".into());
        }
        let rs = records(&self.bytes, 0)?;
        let mut payload = self.bytes[rs[1].payload..rs[1].end].to_vec();
        payload.truncate(92);
        payload[36..40].copy_from_slice(&(60 + 2 * words.len() as u32).to_le_bytes());
        payload[44..46].copy_from_slice(&(words.len() as u16).to_le_bytes());
        for word in words {
            payload.extend_from_slice(&word.to_le_bytes());
        }
        let mut bytes = self.bytes[..rs[1].start].to_vec();
        put_record(&mut bytes, 0x0907, &payload);
        bytes.extend_from_slice(&self.bytes[rs[1].end..]);
        let parsed = Self::parse(&bytes)?;
        *self = parsed;
        Ok(())
    }

    pub fn set_precision(&mut self, precision: u16) -> Result<(), String> {
        let updated = calculation_bytes_with_precision(&self.bytes, precision)?;
        *self = Self::parse(&updated)?;
        Ok(())
    }
    pub fn precision(&self) -> usize {
        calculation_precision(&self.bytes).unwrap()
    }

    pub fn literal_two_plus_three() -> Self {
        Self {
            bytes: object_group(CALCULATION, 2),
        }
    }
    pub fn from_expression(expression: &str) -> Result<Self, String> {
        if let Ok(group) = Self::small_integer_expression(expression) {
            return Ok(group);
        }
        let words = calculation_words(expression, "")?;
        let tree = calculation_tree(&words)?;
        let unit = tree.unit(NumericUnit::Scalar)?;
        let base = Self::literal_two_plus_three();
        let rs = records(&base.bytes, 0)?;
        let mut payload = base.bytes[rs[1].payload..rs[1].end].to_vec();
        payload.truncate(92);
        payload[36..40].copy_from_slice(&(60 + 2 * words.len() as u32).to_le_bytes());
        payload[44..46].copy_from_slice(&(words.len() as u16).to_le_bytes());
        for word in &words {
            payload.extend_from_slice(&word.to_le_bytes());
        }
        let mut bytes = base.bytes[..rs[1].start].to_vec();
        put_record(&mut bytes, 0x0907, &payload);
        bytes.extend_from_slice(&base.bytes[rs[1].end..]);
        if words.contains(&0x0100) || matches!(unit, NumericUnit::Length | NumericUnit::Area) {
            bytes[16..18].copy_from_slice(&unit.subtype().to_le_bytes());
        }
        Self::parse(&bytes)
    }
    fn expression_tree(&self) -> Result<ParameterExpr, String> {
        let rs = records(&self.bytes, 0)?;
        let words = self.bytes[rs[1].payload + 92..rs[1].end]
            .chunks_exact(2)
            .map(|word| u16::from_le_bytes(word.try_into().unwrap()))
            .collect::<Vec<_>>();
        if words.iter().any(|word| (0x6000..=0x60ff).contains(word)) {
            return Err("Literal calculation has a source reference".into());
        }
        let tree = calculation_tree(&words)?;
        let unit = tree.unit(NumericUnit::Scalar)?;
        let subtype = u16::from_le_bytes(self.bytes[16..18].try_into().unwrap());
        // Typed native quantities must agree with the expression. Subtype51
        // is also the legacy generic expression container, including angles.
        if subtype != 51 && subtype != unit.subtype() {
            return Err("Calculation quantity subtype disagrees with expression".into());
        }
        Ok(tree)
    }
    pub fn unit(&self) -> NumericUnit {
        self.expression_tree()
            .unwrap()
            .unit(NumericUnit::Scalar)
            .unwrap()
    }
    pub fn evaluated_value(&self) -> Option<f64> {
        self.expression_tree().ok()?.eval(0.)
    }
    pub fn value_text(&self) -> String {
        let value = self.value();
        if value.is_nan() {
            return "undefined".into();
        }
        let number = if value == f64::INFINITY {
            "∞".into()
        } else if value == f64::NEG_INFINITY {
            "−∞".into()
        } else {
            format_numeric_value(value, self.precision())
        };
        format!("{number}{}", self.unit().suffix())
    }
    pub fn small_integer_expression(expression: &str) -> Result<Self, String> {
        let compact = expression
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<String>();
        let Some(op_at) = compact.find(['+', '-', '−', '*', '×', '·', '/', '÷']) else {
            return Err("Enter a calculation such as 12 + 34".into());
        };
        let operator_char = compact[op_at..].chars().next().unwrap();
        let left_text = &compact[..op_at];
        let right_text = &compact[op_at + operator_char.len_utf8()..];
        let valid_operand = |text: &str| {
            (1..=2).contains(&text.len())
                && text.bytes().all(|byte| byte.is_ascii_digit())
                && (text.len() == 1 || !text.starts_with('0'))
        };
        if !valid_operand(left_text) || !valid_operand(right_text) {
            return Err("Calculation operands must be integers from 0 to 99".into());
        }
        let left: u32 = left_text.parse().unwrap();
        let right: u32 = right_text.parse().unwrap();
        let operator = match operator_char {
            '+' => 0,
            '-' | '−' => 1,
            '*' | '×' | '·' => 2,
            '/' | '÷' => 3,
            _ => return Err("Calculation needs +, -, *, or /".into()),
        };
        if operator == 3 && right == 0 {
            return Err("Cannot divide by zero".into());
        }
        let mut group = Self::literal_two_plus_three();
        let rs = records(&group.bytes, 0)?;
        let mut payload = group.bytes[rs[1].payload..rs[1].end].to_vec();
        payload.truncate(92);
        let mut tokens = Vec::new();
        for digit in left_text.bytes() {
            tokens.push(u16::from(digit - b'0'));
        }
        tokens.push(0x1000 + operator);
        for digit in right_text.bytes() {
            tokens.push(u16::from(digit - b'0'));
        }
        payload[44..46].copy_from_slice(&((tokens.len()) as u16).to_le_bytes());
        for token in tokens {
            payload.extend_from_slice(&token.to_le_bytes());
        }
        let inner_length = (payload.len() - 32) as u32;
        payload[36..40].copy_from_slice(&inner_length.to_le_bytes());
        let mut bytes = group.bytes[..rs[1].start].to_vec();
        put_record(&mut bytes, 0x0907, &payload);
        bytes.extend_from_slice(&group.bytes[rs[1].end..]);
        group.bytes = bytes;
        let bbox = records(&group.bytes, 0)?[3].payload;
        let x = i16::from_le_bytes(group.bytes[bbox..bbox + 2].try_into().unwrap());
        let y = i16::from_le_bytes(group.bytes[bbox + 2..bbox + 4].try_into().unwrap());
        let result = match operator {
            0 => left as i32 + right as i32,
            1 => left as i32 - right as i32,
            2 => (left * right) as i32,
            3 => 0,
            _ => unreachable!(),
        };
        let width = if operator == 3 {
            88 + 10 * (left_text.len().max(right_text.len()) - 1)
        } else {
            let base = [114, 111, 95][operator as usize];
            let result_digits = result.unsigned_abs().to_string().len();
            base + 10 * (left_text.len() + right_text.len() - 2 + result_digits - 1)
                + if result < 0 { 10 } else { 0 }
        } as i16;
        group.bytes[bbox + 4..bbox + 6].copy_from_slice(&(x + width).to_le_bytes());
        group.bytes[bbox + 6..bbox + 8]
            .copy_from_slice(&(y + if operator == 3 { 55 } else { 26 }).to_le_bytes());
        Self::parse(&group.bytes)
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        calculation_precision(bytes)?;
        let rs = records(bytes, 0)?;
        let font_index = match rs.iter().map(|r| r.kind).collect::<Vec<_>>().as_slice() {
            [0x07d0, 0x0907, 0x07d8, 0x08a3, 0x07d7] => 2,
            [0x07d0, 0x0907, 0x07d5, 0x07d8, 0x08a3, 0x07d7] => {
                crate::text_records::LabelAux::parse(&bytes[rs[2].payload..rs[2].end])?;
                3
            }
            _ => return Err("Unsupported calculation record sequence".into()),
        };
        if rs[0].end - rs[0].payload != 28
            || rs[font_index].end - rs[font_index].payload != 2
            || rs[font_index + 1].end - rs[font_index + 1].payload != 8
            || rs.last().unwrap().end != rs.last().unwrap().payload
            || u32::from_le_bytes(bytes[8..12].try_into().unwrap()) & 0xffff != 0
            || !matches!(
                u16::from_le_bytes(bytes[16..18].try_into().unwrap()),
                48..=51
            )
        {
            return Err("Unsupported calculation record lengths or subtype".into());
        }
        let payload = &bytes[rs[1].payload..rs[1].end];
        if !(94..=604).contains(&payload.len())
            || payload.len() % 2 != 0
            || u32::from_le_bytes(payload[36..40].try_into().unwrap()) as usize
                != payload.len() - 32
            || u16::from_le_bytes(payload[44..46].try_into().unwrap()) as usize
                != (payload.len() - 92) / 2
        {
            return Err("Unsupported calculation byte length".into());
        }
        let template = object_group(CALCULATION, 2);
        let base = records(&template, 0)?;
        let expected = &template[base[1].payload..base[1].end];
        for range in [0..12, 16..22, 24..26, 32..36, 40..44, 46..48] {
            if payload[range.clone()] != expected[range] {
                return Err("Unsupported calculation numeric representation".into());
            }
        }
        if !matches!(
            u16::from_le_bytes(payload[26..28].try_into().unwrap()),
            0 | 1 | 3
        ) {
            return Err("Unsupported calculation display mode".into());
        }
        let result = Self {
            bytes: bytes.to_vec(),
        };
        result.expression_tree()?;
        let anchor = result.anchor();
        let bbox = rs[font_index + 1].payload;
        let word = |offset| {
            i16::from_le_bytes(bytes[bbox + offset..bbox + offset + 2].try_into().unwrap())
        };
        if (word(0), word(2)) != anchor
            || !(1..=2048).contains(&(i32::from(word(4)) - i32::from(word(0))))
            || !(1..=2048).contains(&(i32::from(word(6)) - i32::from(word(2))))
        {
            return Err("Calculation anchor and box disagree".into());
        }
        Ok(result)
    }
    fn parts_from_payload(payload: &[u8]) -> Result<(u32, u32, u8), String> {
        let tokens = payload[92..]
            .chunks_exact(2)
            .map(|word| u16::from_le_bytes(word.try_into().unwrap()))
            .collect::<Vec<_>>();
        let Some(op_at) = tokens
            .iter()
            .position(|word| (0x1000..=0x1003).contains(word))
        else {
            return Err("Unsupported calculation expression".into());
        };
        let right_end = tokens.len();
        let left = &tokens[..op_at];
        let right = &tokens[op_at + 1..right_end];
        let valid_digits = |digits: &[u16]| {
            (1..=2).contains(&digits.len())
                && digits.iter().all(|digit| *digit <= 9)
                && (digits.len() == 1 || digits[0] != 0)
        };
        if !valid_digits(left) || !valid_digits(right) || right.iter().any(|digit| *digit >= 0x1000)
        {
            return Err("Unsupported calculation expression".into());
        }
        let value = |digits: &[u16]| {
            digits
                .iter()
                .fold(0u32, |value, digit| value * 10 + u32::from(*digit))
        };
        let (left, right, operator) = (value(left), value(right), (tokens[op_at] - 0x1000) as u8);
        if operator == 3 && right == 0 {
            return Err("Cannot divide by zero".into());
        }
        Ok((left, right, operator))
    }
    fn parts(&self) -> (u32, u32, u8) {
        let rs = records(&self.bytes, 0).unwrap();
        Self::parts_from_payload(&self.bytes[rs[1].payload..rs[1].end]).unwrap()
    }
    pub fn expression(&self) -> String {
        let rs = records(&self.bytes, 0).unwrap();
        if Self::parts_from_payload(&self.bytes[rs[1].payload..rs[1].end]).is_err() {
            return self.expression_tree().unwrap().display("");
        }
        let (left, right, op) = self.parts();
        let operator = match op {
            0 => "+",
            1 => "−",
            2 => "·",
            3 => "÷",
            _ => unreachable!(),
        };
        format!("{left} {operator} {right}")
    }
    pub fn value(&self) -> f64 {
        self.expression_tree()
            .ok()
            .and_then(|tree| tree.eval_raw(0.))
            .unwrap_or(f64::NAN)
    }
    pub fn anchor(&self) -> (i16, i16) {
        let rs = records(&self.bytes, 0).unwrap();
        let at = rs[1].payload + 12;
        (
            i16::from_le_bytes(self.bytes[at..at + 2].try_into().unwrap()),
            i16::from_le_bytes(self.bytes[at + 2..at + 4].try_into().unwrap()),
        )
    }
    pub(crate) fn expression_caption_layout(
        &self,
        base: NumericCaptionLayout,
    ) -> Result<NumericCaptionLayout, String> {
        self.expression_tree()?.caption_layout(base, &[])
    }

    /// Persisted native caption bounds, including the original layout cache.
    pub fn bbox(&self) -> ((i16, i16), (i16, i16)) {
        let rs = records(&self.bytes, 0).unwrap();
        let at = rs
            .iter()
            .find(|record| record.kind == 0x08a3)
            .unwrap()
            .payload;
        let word = |offset| {
            i16::from_le_bytes(self.bytes[at + offset..at + offset + 2].try_into().unwrap())
        };
        ((word(0), word(2)), (word(4), word(6)))
    }

    pub fn set_anchor(&mut self, anchor: (i16, i16)) -> Result<(), String> {
        let rs = records(&self.bytes, 0)?;
        let before = self.anchor();
        let bbox = rs
            .iter()
            .find(|record| record.kind == 0x08a3)
            .unwrap()
            .payload;
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

pub(crate) fn table_value_text(value: f64) -> Result<String, String> {
    if !value.is_finite()
        || value.abs() > 99.99
        || (value * 100. - (value * 100.).round()).abs() > 1e-7
    {
        return Err(
            "This table supports values within ±99.99 with up to two decimal places".into(),
        );
    }
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
    let rest = text.strip_prefix("000<SS2<T")?;
    let (name_len, rest) = rest.split_once('x')?;
    let name_len = name_len.parse::<usize>().ok()?;
    if !(1..=32).contains(&name_len) || rest.len() < name_len {
        return None;
    }
    let name = rest.get(..name_len)?;
    let rest = rest.get(name_len..)?;
    if !name.as_bytes()[0].is_ascii_alphabetic()
        || !name
            .bytes()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == b'_')
    {
        return None;
    }
    let values = table_decimal_rows(rest.strip_prefix(">>")?, true)?;
    Some((name.to_owned(), values))
}

/// Rich-text run lengths count UTF-8 bytes. Historical ASCII-minus named rows
/// remain readable, while new writers use the observed native en dash.
fn table_decimal_rows(mut rest: &str, allow_ascii_minus: bool) -> Option<Vec<String>> {
    let mut values = Vec::new();
    while !rest.is_empty() {
        if values.len() >= 256 {
            return None;
        }
        let (length, after) = rest.strip_prefix("0<H<T")?.split_once('x')?;
        let length = length.parse::<usize>().ok()?;
        if !(1..=8).contains(&length) {
            return None;
        }
        let value = after.get(..length)?;
        rest = after.get(length..)?.strip_prefix(">>")?;
        let decimals = value
            .split_once('.')
            .map_or(0, |(_, fraction)| fraction.len());
        if decimals > 2 {
            return None;
        }
        let decimal = value
            .strip_prefix('–')
            .map_or_else(|| value.to_owned(), |absolute| format!("-{absolute}"));
        if !decimal.is_ascii() {
            return None;
        }
        let numeric = decimal.parse::<f64>().ok()?;
        table_value_text(numeric).ok()?;
        if named_table_value_text(numeric, decimals).ok()? != value
            && !(allow_ascii_minus
                && value.starts_with('-')
                && format_fixed_numeric_value(numeric, decimals) == value)
        {
            return None;
        }
        values.push(value.to_owned());
    }
    (!values.is_empty()).then_some(values)
}

pub fn named_table_value_text(value: f64, precision: usize) -> Result<String, String> {
    table_value_text(value)?;
    if precision > 2 {
        return Err("Named tables support up to two decimal places".into());
    }
    Ok(format_fixed_numeric_value(value, precision).replacen('-', "–", 1))
}

// Native automatically numbered headers encode the base letter and subscript
// as separate text runs rather than the plain-name grammar.
fn indexed_table_parts(layout: &[u8]) -> Option<(String, Vec<String>)> {
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
    let rest = text.strip_prefix("000<-<SS2<T1xt>><T")?;
    let (length, rest) = rest.split_once('x')?;
    let length = length.parse::<usize>().ok()?;
    if !(1..=10).contains(&length) || rest.len() < length {
        return None;
    }
    let index = rest.get(..length)?;
    let rest = rest.get(length..)?;
    let number = index.parse::<u32>().ok()?;
    if number == 0 || number.to_string() != index {
        return None;
    }
    let values = table_decimal_rows(rest.strip_prefix(">>")?, false)?;
    Some((format!("t[{number}]"), values))
}

fn canonical_indexed_table(
    source: u32,
    name: &str,
    values: &[String],
    anchor: (i16, i16),
    font: u16,
) -> Result<Vec<u8>, String> {
    let index = name
        .strip_prefix("t[")
        .and_then(|name| name.strip_suffix(']'))
        .ok_or("Invalid indexed table name")?;
    if source == 0 || !(1..=256).contains(&values.len()) {
        return Err("Invalid indexed table source or rows".into());
    }
    let base = object_group(
        include_bytes!("../../../tests/fixtures/numeric_parameter_table_all_unnamed_native_created.gsp"),
        15,
    );
    let rs = records(&base, 0)?;
    let mut layout = base[rs[2].payload..rs[2].payload + 28].to_vec();
    layout[12..14].copy_from_slice(&anchor.0.to_le_bytes());
    layout[14..16].copy_from_slice(&anchor.1.to_le_bytes());
    layout[16..18].copy_from_slice(&((values.len() + 2) as u16).to_le_bytes());
    layout.extend_from_slice(format!("000<-<SS2<T1xt>><T{}x{index}>>", index.len()).as_bytes());
    for value in values {
        layout.extend_from_slice(format!("0<H<T{}x{value}>>", value.len()).as_bytes());
    }
    while layout.len() % 4 != 0 {
        layout.push(0);
    }
    if indexed_table_parts(&layout) != Some((name.to_owned(), values.to_vec())) {
        return Err("Invalid indexed table text".into());
    }
    let length = layout.len() as u32;
    layout[4..8].copy_from_slice(&length.to_le_bytes());
    let mut refs = base[rs[1].payload..rs[1].end].to_vec();
    refs[4..8].copy_from_slice(&source.to_le_bytes());
    let mut group = Vec::new();
    put_record(&mut group, 0x07d0, &base[rs[0].payload..rs[0].end]);
    put_record(&mut group, 0x07d2, &refs);
    put_record(&mut group, 0x090d, &layout);
    put_record(&mut group, 0x07d8, &font.to_le_bytes());
    put_record(&mut group, 0x07d7, &[]);
    Ok(group)
}

fn canonical_named_table(
    source: u32,
    name: &str,
    values: &[String],
    anchor: (i16, i16),
    font: u16,
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
    put_record(&mut group, 0x07d8, &font.to_le_bytes());
    put_record(&mut group, 0x07d7, &[]);
    Ok(group)
}

impl TableGroup {
    pub(crate) fn with_font_reference(mut self, font: u16) -> Result<Self, String> {
        let rs = records(&self.bytes, 0)?;
        let record = rs
            .iter()
            .find(|record| record.kind == 0x07d8)
            .filter(|record| record.end - record.payload == 2)
            .ok_or("Invalid table font reference")?;
        self.bytes[record.payload..record.end].copy_from_slice(&font.to_le_bytes());
        Self::parse(&self.bytes)
    }

    pub fn default_snapshot() -> Self {
        Self {
            bytes: table_template(),
        }
    }

    pub(crate) fn indexed_snapshot(
        source: u32,
        parameter: &ParameterGroup,
        anchor: (i16, i16),
        font: u16,
    ) -> Result<Self, String> {
        if parameter.unit() != NumericUnit::Scalar || parameter.display_mode() == 0 {
            return Err("Indexed tables require a named scalar parameter".into());
        }
        let value = named_table_value_text(parameter.value(), parameter.precision())?;
        Self::parse(&canonical_indexed_table(
            source,
            parameter.name(),
            &[value],
            anchor,
            font,
        )?)
    }

    pub fn named_snapshot(
        source: u32,
        parameter: &ParameterGroup,
        anchor: (i16, i16),
    ) -> Result<Self, String> {
        if parameter.unit() != NumericUnit::Scalar {
            return Err("This table supports scalar parameters".into());
        }
        table_value_text(parameter.value())?;
        let value = named_table_value_text(parameter.value(), parameter.precision())?;
        let bytes = canonical_named_table(source, parameter.name(), &[value], anchor, 1)?;
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
        let old_records = records(&self.bytes, 0)?;
        let new_records = records(&changed, 0)?;
        let old_layout = &self.bytes[old_records[2].payload..old_records[2].end];
        let text_end = |rows: &[[f64; 2]]| -> Result<usize, String> {
            rows.iter().try_fold(72, |end, [x, y]| {
                Ok(end + 4 + table_value_text(*x)?.len() + 3 + table_value_text(*y)?.len())
            })
        };
        let old_text_end = text_end(&self.pair_values().ok_or("Invalid pair table")?)?;
        let new_text_end = text_end(&all)?;
        let old_padding = &old_layout[old_text_end..];
        let new_padding = new_records[2].end - new_records[2].payload - new_text_end;
        // Alignment bytes retain their native spelling when the row widths
        // are unchanged. Different row widths require fresh alignment.
        if old_padding.len() == new_padding {
            changed[new_records[2].end - new_padding..new_records[2].end]
                .copy_from_slice(old_padding);
        }
        *self = Self::parse_pair(&changed)?;
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
            && normalized
                == canonical_named_table(
                    source,
                    &name,
                    &values,
                    anchor,
                    u16::from_le_bytes(
                        bytes[rs[3].payload..rs[3].end]
                            .try_into()
                            .map_err(|_| "Invalid table font reference")?,
                    ),
                )?
        {
            return Ok(Self {
                bytes: bytes.to_vec(),
            });
        }
        if u32::from_le_bytes(layout[4..8].try_into().unwrap()) as usize == layout.len()
            && let Some((name, values)) = indexed_table_parts(layout)
            && rows == values.len()
            && normalized
                == canonical_indexed_table(
                    source,
                    &name,
                    &values,
                    anchor,
                    u16::from_le_bytes(
                        bytes[rs[3].payload..rs[3].end]
                            .try_into()
                            .map_err(|_| "Invalid table font reference")?,
                    ),
                )?
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
        let layout = &self.bytes[rs[2].payload..rs[2].end];
        // Legacy t[1] tables have fixed two-decimal rows and a specific font
        // reference. Preserve that API only for the complete observed layout;
        // newly generated t[1] captions can have other precisions and fonts.
        if let Some(values) = default_table_values(layout, self.rows()) {
            let mut normalized = self.bytes.clone();
            normalized[10] &= !1;
            if self.rows() == 2 && normalized[rs[2].end - 2..rs[2].end] == [b't', 0] {
                normalized[rs[2].end - 2] = 0;
            }
            if canonical_table_values(self.source_id(), self.anchor(), &values)
                .ok()
                .as_deref()
                == Some(normalized.as_slice())
            {
                return None;
            }
        }
        named_table_parts(layout).or_else(|| indexed_table_parts(layout))
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
        let old_records = records(&self.bytes, 0)?;
        let marker =
            self.rows() == 2 && self.bytes[old_records[2].end - 2..old_records[2].end] == [b't', 0];
        let mut changed = canonical_table_values(self.source_id(), self.anchor(), &values)?;
        changed[10] = self.bytes[10];
        if marker {
            let new_records = records(&changed, 0)?;
            let end = new_records[2].end;
            if changed[end - 2..end] != [0, 0] {
                return Err("Table edit cannot preserve native padding".into());
            }
            changed[end - 2] = b't';
        }
        *self = Self::parse(&changed)?;
        Ok(())
    }

    pub fn add_named_snapshot(&mut self, value: &str) -> Result<(), String> {
        let (name, mut values) = self.named_label_values().ok_or("Not a named table")?;
        *values.last_mut().ok_or("Table has no rows")? = value.to_owned();
        values.push(value.to_owned());
        self.replace_named_rows(&name, &values)?;
        Ok(())
    }

    fn replace_named_rows(&mut self, name: &str, values: &[String]) -> Result<(), String> {
        let rs = records(&self.bytes, 0)?;
        let indexed = indexed_table_parts(&self.bytes[rs[2].payload..rs[2].end]).is_some();
        let mut bytes = if indexed {
            canonical_indexed_table(
                self.source_id(),
                name,
                values,
                self.anchor(),
                u16::from_le_bytes(
                    self.bytes[rs[3].payload..rs[3].end]
                        .try_into()
                        .map_err(|_| "Invalid table font reference")?,
                ),
            )?
        } else {
            canonical_named_table(
                self.source_id(),
                name,
                values,
                self.anchor(),
                u16::from_le_bytes(
                    self.bytes[rs[3].payload..rs[3].end]
                        .try_into()
                        .map_err(|_| "Invalid table font reference")?,
                ),
            )?
        };
        bytes[10] = self.bytes[10];
        *self = Self::parse(&bytes)?;
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
            self.replace_named_rows(&name, &values)?;
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
    #[test]
    fn indexed_table_text_requires_valid_byte_lengths_and_matching_native_metadata() {
        let bytes = object_group(
            include_bytes!(
                "../../../tests/fixtures/numeric_parameter_table_all_unnamed_native_created.gsp"
            ),
            15,
        );
        assert!(TableGroup::parse(&bytes).is_ok());
        let rs = records(&bytes, 0).unwrap();
        for (offset, value) in [
            (rs[1].payload, 2),
            (rs[2].payload + 16, 4),
            (rs[2].end - 1, b'!'),
        ] {
            let mut invalid = bytes.clone();
            invalid[offset] = value;
            assert!(TableGroup::parse(&invalid).is_err());
        }
        let mut layout = bytes[rs[2].payload..rs[2].end].to_vec();
        let at = layout.windows(4).position(|text| text == b"1.00").unwrap();
        layout[at..at + 4].copy_from_slice("é00".as_bytes());
        assert!(indexed_table_parts(&layout).is_none());
    }

    #[test]
    fn native_binary_fraction_rounding_avoids_false_decimal_ties() {
        // Independent native scalar controls: 3/40, 1/8, and -1/8 at precision 2.
        assert_eq!(format_numeric_value(3. / 40., 2), "0.07");
        assert_eq!(format_numeric_value(1. / 8., 2), "0.13");
        assert_eq!(format_numeric_value(-1. / 8., 2), "-0.13");
        assert_eq!(format_numeric_value(-3. / 40., 2), "-0.07");
    }

    #[test]
    fn parameter_calculation_chain_imports_and_evaluates() {
        let native = include_bytes!("../../../tests/fixtures/parameter_calculation_chain_native.gsp");
        let expected = [2., 4., 5., 5., 2.5];
        for (ordinal, value) in (2..=6).zip(expected) {
            let group =
                super::ParameterCalculationGroup::parse(&super::object_group(native, ordinal))
                    .unwrap();
            assert_eq!(group.source_id(), 1);
            assert_eq!(group.value(2.), Some(value));
            assert!(group.expression("t₁").contains("t₁"));
        }
    }
    #[test]
    fn measured_two_digit_native_group_imports() {
        let native = include_bytes!("../../../tests/fixtures/calculation_length_times_twelve_native.gsp");
        let group = super::object_group(native, 5);
        let parsed = super::MeasuredCalculationGroup::parse(&group).unwrap();
        assert_eq!(parsed.factor(), 12);
    }
    use super::*;

    #[test]
    fn multiple_source_calculations_indexes_units_bounds_and_remap() {
        let group = ParameterCalculationGroup::from_sources(
            &[2, 4, 6],
            "sqrt((v1+v2)*v3)",
            (10, 30),
            &[NumericUnit::Scalar; 3],
        )
        .unwrap();
        assert_eq!(group.sources(), vec![2, 4, 6]);
        assert_eq!(
            group.value_for_sources(&[2., 3., 5.], &[NumericUnit::Scalar; 3]),
            Some(5.)
        );
        assert_eq!(group.anchor(), (10, 30));
        assert_eq!(
            group.remap_sources(&[8, 10, 12]).unwrap().sources(),
            vec![8, 10, 12]
        );
        let length = ParameterCalculationGroup::from_sources(
            &[2, 4],
            "(a+b)/2",
            (10, 30),
            &[NumericUnit::Length; 2],
        )
        .unwrap();
        assert_eq!(length.unit(), NumericUnit::Length);
        assert_eq!(
            length.value_for_sources(&[2., 4.], &[NumericUnit::Length; 2]),
            Some(3.)
        );
        let angle = ParameterCalculationGroup::from_sources(
            &[2, 4],
            "v1/v2",
            (10, 30),
            &[NumericUnit::Degrees, NumericUnit::Scalar],
        )
        .unwrap();
        assert_eq!(angle.unit(), NumericUnit::Degrees);
        assert_eq!(
            angle.value_for_sources(&[90., 2.], &[NumericUnit::Degrees, NumericUnit::Scalar]),
            Some(45.)
        );
        let mixed = ParameterCalculationGroup::from_sources(
            &[2, 4],
            "v1+v2",
            (10, 30),
            &[NumericUnit::Degrees, NumericUnit::Scalar],
        )
        .unwrap();
        assert_eq!(mixed.unit(), NumericUnit::Scalar);
        assert_eq!(
            mixed.value_for_sources(&[90., 2.], &[NumericUnit::Degrees, NumericUnit::Scalar]),
            Some(92.)
        );
        assert!(
            ParameterCalculationGroup::from_sources(
                &[2, 4],
                "v3+1",
                (10, 30),
                &[NumericUnit::Scalar; 2]
            )
            .is_err()
        );
        let ids = (1..=51).collect::<Vec<_>>();
        let expr = (1..=51)
            .map(|i| format!("v{i}"))
            .collect::<Vec<_>>()
            .join("+");
        let largest = ParameterCalculationGroup::from_sources(
            &ids,
            &expr,
            (10, 30),
            &[NumericUnit::Scalar; 51],
        )
        .unwrap();
        assert_eq!(
            largest.value_for_sources(&[1.; 51], &[NumericUnit::Scalar; 51]),
            Some(51.)
        );
        assert!(
            ParameterCalculationGroup::from_sources(
                &[1; 257],
                "v1",
                (10, 30),
                &[NumericUnit::Scalar; 257]
            )
            .is_err()
        );
        let mut invalid_index = largest.as_bytes().to_vec();
        let numeric = records(&invalid_index, 0).unwrap()[2];
        let last = numeric.end - 2;
        invalid_index[last..last + 2].copy_from_slice(&0x6033_u16.to_le_bytes());
        assert!(ParameterCalculationGroup::parse(&invalid_index).is_err());
        let boundary = ParameterCalculationGroup::from_sources(
            &[1; 256],
            "v256",
            (10, 30),
            &[NumericUnit::Scalar; 256],
        )
        .unwrap();
        assert_eq!(
            boundary.value_for_sources(&[2.; 256], &[NumericUnit::Scalar; 256]),
            Some(2.)
        );
        assert!(
            ParameterCalculationGroup::from_sources(
                &[1; 256],
                "v257",
                (10, 30),
                &[NumericUnit::Scalar; 256],
            )
            .is_err()
        );
        let mut broken = group.as_bytes().to_vec();
        broken[44..48].copy_from_slice(&2_u32.to_le_bytes());
        assert!(ParameterCalculationGroup::parse(&broken).is_err());
    }

    #[test]
    fn source_free_expressions_functions_units_and_undefined_values() {
        for (expression, value, unit) in [
            (
                "pi+e",
                CALCULATOR_PI + std::f64::consts::E,
                NumericUnit::Scalar,
            ),
            ("1.25*4+.5", 5.5, NumericUnit::Scalar),
            ("100+3", 103., NumericUnit::Scalar),
            ("10000000000", 1e10, NumericUnit::Scalar),
            ("sqrt(2)^2", 2., NumericUnit::Scalar),
            ("sin(30°)", 0.5, NumericUnit::Scalar),
            ("asin(.5)", 30., NumericUnit::Degrees),
            ("-0.5+2", 1.5, NumericUnit::Scalar),
            ("-2^2", -4., NumericUnit::Scalar),
            ("(-2)^2", 4., NumericUnit::Scalar),
            ("2^-3", 0.125, NumericUnit::Scalar),
            ("round(-4.5)", -4., NumericUnit::Scalar),
            ("-30°", -30., NumericUnit::Degrees),
        ] {
            let mut group = CalculationGroup::from_expression(expression).unwrap();
            assert_eq!(group.unit(), unit);
            assert!((group.value() - value).abs() < 1e-12, "{expression}");
            group.set_anchor((150, 200)).unwrap();
            assert_eq!(group.anchor(), (150, 200));
            assert_eq!(CalculationGroup::parse(group.as_bytes()).unwrap(), group);
        }
        let mixed_literal = CalculationGroup::from_expression("30°+2").unwrap();
        assert_eq!(mixed_literal.unit(), NumericUnit::Scalar);
        assert_eq!(mixed_literal.value(), 32.);
        for expression in ["0/0", "sqrt(-1)", "asin(2)"] {
            let group = CalculationGroup::from_expression(expression).unwrap();
            assert!(group.evaluated_value().is_none());
            assert_eq!(group.value_text(), "undefined");
        }
        for (expression, text) in [("1/0", "∞"), ("-1/0", "−∞"), ("ln(0)", "−∞")] {
            let group = CalculationGroup::from_expression(expression).unwrap();
            assert!(group.evaluated_value().is_none());
            assert_eq!(group.value_text(), text);
        }
        for expression in [
            "v+2",
            "sin()",
            "(2+3",
            "2)3",
            "1".repeat(MAX_CALCULATION_TOKENS + 1).as_str(),
        ] {
            assert!(
                CalculationGroup::from_expression(expression).is_err(),
                "{expression}"
            );
        }
        assert_eq!(
            CalculationGroup::from_expression("2+3").unwrap().as_bytes(),
            CalculationGroup::literal_two_plus_three().as_bytes()
        );
        assert!(
            CalculationGroup::from_expression("asin(.5)")
                .unwrap()
                .value_text()
                .ends_with('°')
        );
    }

    #[test]
    fn source_free_quantity_expressions_keep_dimensions_and_edit_history_data() {
        for (expression, value, unit) in [
            ("0.3 cm", 0.3, NumericUnit::Length),
            ("2 cm+3 cm", 5., NumericUnit::Length),
            ("2 cm*3 cm", 6., NumericUnit::Area),
            ("sqrt(3 cm*3 cm)", 3., NumericUnit::Length),
            ("3 cm/2 cm", 1.5, NumericUnit::Scalar),
            ("sin(pi rad/6)", 0.5, NumericUnit::Scalar),
            ("abs(-pi rad/6)", 30., NumericUnit::Degrees),
            ("1.+2.", 3., NumericUnit::Scalar),
        ] {
            let group = CalculationGroup::from_expression(expression).unwrap();
            assert_eq!(group.unit(), unit, "{expression}");
            assert!(
                (group.evaluated_value().unwrap() - value).abs() < 1e-12,
                "{expression}"
            );
            assert_eq!(CalculationGroup::parse(group.as_bytes()).unwrap(), group);
        }
        let mut length = CalculationGroup::from_expression("0.3 cm").unwrap();
        length.set_expression("sqrt(2 cm*2 cm)").unwrap();
        assert_eq!(length.unit(), NumericUnit::Length);
        assert_eq!(length.evaluated_value(), Some(2.));
        let retained = length.clone();
        assert!(length.set_expression("3").is_err());
        assert_eq!(length, retained);
        let mut malformed = retained.as_bytes().to_vec();
        malformed[16..18].copy_from_slice(&50_u16.to_le_bytes());
        assert!(CalculationGroup::parse(&malformed).is_err());
        let undefined = CalculationGroup::from_expression("sqrt(-1*(1 cm)^2)").unwrap();
        assert_eq!(undefined.unit(), NumericUnit::Length);
        assert_eq!(undefined.evaluated_value(), None);
    }

    #[test]
    fn inverse_functions_constants_units_and_domains() {
        for (expression, expected, unit) in [
            ("asin(n)", 30., NumericUnit::Degrees),
            ("acos(n)", 60., NumericUnit::Degrees),
            ("atan(n)", 26.56505117707799, NumericUnit::Degrees),
            ("sin(asin(n))", 0.5, NumericUnit::Scalar),
            ("n+pi", 0.5 + CALCULATOR_PI, NumericUnit::Scalar),
            ("n+π", 0.5 + CALCULATOR_PI, NumericUnit::Scalar),
            ("n+e", 0.5 + std::f64::consts::E, NumericUnit::Scalar),
            ("acos(n)+10°", 70., NumericUnit::Degrees),
        ] {
            let group =
                ParameterCalculationGroup::from_expression(1, "n", expression, (10, 30)).unwrap();
            assert_eq!(group.unit(), unit);
            assert!(
                (group.value(0.5).unwrap() - expected).abs() < 1e-12,
                "{expression}"
            );
            assert_eq!(
                ParameterCalculationGroup::parse(group.as_bytes()).unwrap(),
                group
            );
        }
        for function in ["asin", "acos"] {
            let group = ParameterCalculationGroup::from_expression(
                1,
                "n",
                &format!("{function}(n)"),
                (10, 30),
            )
            .unwrap();
            assert!(group.value(1.01).is_none());
            assert!(group.value(-1.01).is_none());
            assert!(group.value(1.).is_some());
            assert!(group.value(-1.).is_some());
            assert!(
                ParameterCalculationGroup::from_expression_with_source_unit(
                    1,
                    "v",
                    &format!("{function}(v)"),
                    (10, 30),
                    NumericUnit::Degrees
                )
                .is_err()
            );
        }
        let prefix = ParameterCalculationGroup::from_expression(1, "p", "p+pi", (10, 30)).unwrap();
        assert_eq!(prefix.value(2.), Some(2. + CALCULATOR_PI));
        let e_parameter =
            ParameterCalculationGroup::from_expression(1, "e", "e+1", (10, 30)).unwrap();
        assert_eq!(e_parameter.value(2.), Some(3.));
        assert!(ParameterCalculationGroup::from_expression(1, "n", "pi+e", (10, 30)).is_err());
        let mixed =
            ParameterCalculationGroup::from_expression(1, "n", "asin(n)+n", (10, 30)).unwrap();
        assert_eq!(mixed.unit(), NumericUnit::Scalar);
        assert!((mixed.value(0.5).unwrap() - 30.5).abs() < 1e-12);
    }

    #[test]
    fn calculator_functions_nested_units_and_domains() {
        for (expression, expected) in [
            ("sin(n)", 0.5),
            ("cos(n)", 3_f64.sqrt() / 2.),
            ("tan(n)", 1. / 3_f64.sqrt()),
            ("abs(n-33)", 3.),
            ("sqrt(n+6)", 6.),
            ("ln(n-29)", 0.),
            ("log(n+70)", 2.),
            ("sgn(n-33)", -1.),
            ("round(n/12)", 3.),
            ("trunc(n/12)", 2.),
            ("sqrt(abs(n-34))", 2.),
            ("sin (n)+cos(n+60)", 0.5),
        ] {
            let group =
                ParameterCalculationGroup::from_expression(1, "n", expression, (10, 30)).unwrap();
            assert!(
                (group.value(30.).unwrap() - expected).abs() < 1e-12,
                "{expression}"
            );
            assert_eq!(
                ParameterCalculationGroup::parse(group.as_bytes()).unwrap(),
                group
            );
        }
        for expression in [
            "sin(n",
            "sin n",
            "sqrt()",
            "sin(n))",
            "ln((n)",
            "unknown(n)",
        ] {
            assert!(
                ParameterCalculationGroup::from_expression(1, "n", expression, (10, 30)).is_err(),
                "{expression}"
            );
        }
        for expression in ["sqrt(n-31)", "ln(n-30)", "log(n-31)"] {
            let group =
                ParameterCalculationGroup::from_expression(1, "n", expression, (10, 30)).unwrap();
            assert!(group.value(30.).is_none(), "{expression}");
        }
        let tangent =
            ParameterCalculationGroup::from_expression(1, "n", "tan(n)", (10, 30)).unwrap();
        assert!((tangent.value(90.).unwrap() / -286411383293069.44 - 1.).abs() < 1e-12);
        let decimal =
            ParameterCalculationGroup::from_expression(1, "n", "n*0.15", (10, 30)).unwrap();
        assert_eq!(decimal.value(-30.), Some(-4.5));
        let angle = ParameterCalculationGroup::from_expression_with_source_unit(
            1,
            "v",
            "sin(v/2)",
            (10, 30),
            NumericUnit::Degrees,
        )
        .unwrap();
        assert_eq!(angle.unit(), NumericUnit::Scalar);
        assert!(
            (angle
                .value_with_source_unit(60., NumericUnit::Degrees)
                .unwrap()
                - 0.5)
                .abs()
                < 1e-12
        );
        let length = ParameterCalculationGroup::from_expression_with_source_unit(
            1,
            "v",
            "abs(v-v*2)",
            (10, 30),
            NumericUnit::Length,
        )
        .unwrap();
        assert_eq!(length.unit(), NumericUnit::Length);
        assert_eq!(
            length.value_with_source_unit(3., NumericUnit::Length),
            Some(3.)
        );
        for expression in ["sin(v)"] {
            assert!(
                ParameterCalculationGroup::from_expression_with_source_unit(
                    1,
                    "v",
                    expression,
                    (10, 30),
                    NumericUnit::Length
                )
                .is_err()
            );
        }
        for (expression, expected) in [("round(n*0.15)", -4.), ("trunc(n*0.15)", -4.)] {
            let group =
                ParameterCalculationGroup::from_expression(1, "n", expression, (10, 30)).unwrap();
            assert_eq!(group.value(-30.), Some(expected));
        }
        let zero = ParameterCalculationGroup::from_expression(1, "n", "sgn(n)", (10, 30)).unwrap();
        assert_eq!(zero.value(-0.), Some(0.));
    }

    #[test]
    fn quantity_functions_preserve_rounding_units_and_strip_log_and_sign_units() {
        use NumericUnit::{Area, Compound, Degrees, Length, Scalar};
        for unit in [
            Length,
            Area,
            Degrees,
            Compound {
                length: 4,
                degrees: 0,
            },
        ] {
            for (expression, expected_unit) in [
                ("ln(v)", Scalar),
                ("log(v)", Scalar),
                ("sgn(v)", Scalar),
                ("round(v)", unit),
                ("trunc(v)", unit),
            ] {
                let mut group = ParameterCalculationGroup::from_expression_with_source_unit(
                    1,
                    "v",
                    expression,
                    (10, 30),
                    unit,
                )
                .unwrap();
                assert_eq!(group.unit(), expected_unit);
                for value in [4.5_f64, -4.5, 0.] {
                    let expected = match expression {
                        "ln(v)" => (value > 0.).then(|| value.ln()),
                        "log(v)" => (value > 0.).then(|| value.log10()),
                        "sgn(v)" => Some(if value == 0. { 0. } else { value.signum() }),
                        "round(v)" => Some((value + 0.5).floor()),
                        _ => Some(value.trunc()),
                    };
                    assert_eq!(group.value_for_sources(&[value], &[unit]), expected);
                }
                group.set_precision(0).unwrap();
                group.set_anchor((50, 90)).unwrap();
                let remapped = group.remap_source(7).unwrap();
                assert_eq!(remapped.unit(), expected_unit);
                assert_eq!(
                    ParameterCalculationGroup::parse(remapped.as_bytes())
                        .unwrap()
                        .with_source_units(&[unit])
                        .unwrap()
                        .unit(),
                    expected_unit
                );
            }
        }
        for (expression, expected_unit, expected) in [
            ("round(sqrt(v))", Length, 3.),
            ("ln(round(v))", Scalar, 8_f64.ln()),
            ("round(ln(v))", Scalar, 2.),
        ] {
            let group = ParameterCalculationGroup::from_expression_with_source_unit(
                1,
                "v",
                expression,
                (10, 30),
                Area,
            )
            .unwrap();
            assert_eq!(group.unit(), expected_unit);
            assert_eq!(group.value_for_sources(&[7.9173], &[Area]), Some(expected));
        }
    }

    #[test]
    fn dimensional_roots_powers_and_unit_algebra() {
        use NumericUnit::{Area, Compound, Degrees, Length, Scalar};
        for (unit, expression, value, expected_unit, expected_value, suffix) in [
            (Area, "sqrt(v)", 9., Length, 3., " cm"),
            (Length, "sqrt(v)", 9., Scalar, 3., ""),
            (Degrees, "sqrt(v)", 90., Scalar, 90_f64.sqrt(), ""),
            (Area, "v^0.5", 9., Scalar, 3., ""),
            (
                Area,
                "v^2",
                9.,
                Compound {
                    length: 4,
                    degrees: 0,
                },
                81.,
                " cm⁴",
            ),
            (
                Degrees,
                "v^2",
                90.,
                Compound {
                    length: 0,
                    degrees: 2,
                },
                8100.,
                "°²",
            ),
            (Area, "sqrt(v^2)", 9., Area, 9., " cm²"),
            (Area, "sqrt(sqrt(v^2))", 9., Length, 3., " cm"),
            (Length, "v^(1+2)", 2., Scalar, 8., ""),
            (Length, "v^-2", 2., Scalar, 0.25, ""),
            (Area, "v^(2)", 9., Scalar, 81., ""),
            (Length, "v^2.0", 3., Area, 9., " cm²"),
            (Length, "v^sqrt(4)", 3., Scalar, 9., ""),
            (Area, "v*v/v", 9., Area, 9., " cm²"),
        ] {
            let mut group = ParameterCalculationGroup::from_expression_with_source_unit(
                1,
                "v",
                expression,
                (10, 30),
                unit,
            )
            .unwrap();
            assert_eq!(group.unit(), expected_unit, "{expression}");
            assert_eq!(
                group.value_for_sources(&[value], &[unit]),
                Some(expected_value)
            );
            assert_eq!(group.unit().suffix(), suffix);
            group.set_precision(5).unwrap();
            group.set_anchor((50, 80)).unwrap();
            let remapped = group.remap_source(7).unwrap();
            assert_eq!(remapped.unit(), expected_unit);
            assert_eq!(
                ParameterCalculationGroup::parse(remapped.as_bytes())
                    .unwrap()
                    .with_source_units(&[unit])
                    .unwrap()
                    .unit(),
                expected_unit
            );
        }
        let negative = ParameterCalculationGroup::from_expression_with_source_unit(
            1,
            "v",
            "sqrt(v)",
            (10, 30),
            Area,
        )
        .unwrap();
        assert_eq!(negative.value_for_sources(&[-1.], &[Area]), None);
        let source_power =
            ParameterCalculationGroup::from_sources(&[1, 2], "v1^v2", (10, 30), &[Length, Scalar])
                .unwrap();
        assert_eq!(source_power.unit(), Scalar);
        assert_eq!(
            source_power.value_for_sources(&[3., 2.], &[Length, Scalar]),
            Some(9.)
        );
        let mut wrong_header = negative.as_bytes().to_vec();
        wrong_header[16..18].copy_from_slice(&49u16.to_le_bytes());
        assert!(
            ParameterCalculationGroup::parse(&wrong_header)
                .unwrap()
                .with_source_units(&[Area])
                .is_err()
        );
        let literal = CalculationGroup::from_expression("90°^2").unwrap();
        assert_eq!(literal.value_text(), "8100.00°²");
    }

    #[test]
    fn original_table_roundtrips_and_anchor_moves() {
        let original = table_template();
        let mut table = TableGroup::parse(&original).unwrap();
        assert_eq!(table.source_id(), 1);
        assert_eq!(table.anchor(), (10, 38));
        table.set_anchor((100, 116));
        TableGroup::parse(table.as_bytes()).unwrap();
        let dragged = include_bytes!("../../../tests/fixtures/table_table_dragged.gsp");
        let rs = records(dragged, 4).unwrap();
        let start = rs.iter().filter(|r| r.kind == 0x07d0).nth(1).unwrap().start;
        let end = rs.iter().filter(|r| r.kind == 0x07d7).nth(1).unwrap().end;
        assert_eq!(table.as_bytes(), &dragged[start..end]);
    }

    #[test]
    fn generated_table_has_valid_refs_and_rejects_other_parameter_values() {
        let source = include_bytes!("../../../tests/fixtures/table_parameter_default.gsp");
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
                include_bytes!("../../../tests/fixtures/table_parameter_custom.gsp"),
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
        let original = include_bytes!("../../../tests/fixtures/table_parameter_two_rows.gsp");
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
        let dragged = include_bytes!("../../../tests/fixtures/table_parameter_dragged.gsp");
        assert_eq!(parameter.as_bytes(), object_group(dragged, 1));
        let blank = include_bytes!("../../../tests/fixtures/text_object_blank.gsp");
        let output = append_default_parameter(blank, (140, 95)).unwrap();
        assert_eq!(object_group(&output, 1), parameter.as_bytes());
        assert!(append_default_parameter(&output, (10, 5)).is_err());
    }

    #[test]
    fn animated_parameter_keeps_precise_native_value_and_domain() {
        let bytes = include_bytes!("../../../tests/fixtures/parameter_animated_native_saved.gsp");
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
                include_bytes!("../../../tests/fixtures/action_animation_parameter_native_rate2.gsp")
                    .as_slice(),
                5.7..5.8,
            ),
            (
                include_bytes!("../../../tests/fixtures/action_animation_parameter_native_discrete.gsp")
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
            include_bytes!("../../../tests/fixtures/table_parameter_custom.gsp"),
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
                include_bytes!("../../../tests/fixtures/table_parameter_speed_precision2_original.gsp"),
                1
            )
        );
        let original_test = object_group(
            include_bytes!("../../../tests/fixtures/table_parameter_test_original.gsp"),
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
                include_bytes!("../../../tests/fixtures/table_named_speed_snapshot_original.gsp")
                    .as_slice(),
                1,
                2,
            ),
            (
                include_bytes!(
                    "../../../tests/fixtures/table_named_speed_precision2_snapshot_original.gsp"
                )
                .as_slice(),
                1,
                2,
            ),
            (
                include_bytes!("../../../tests/fixtures/table_named_test_late_source_original.gsp")
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
        let original = include_bytes!("../../../tests/fixtures/table_named_speed_two_rows_original.gsp");
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

        let live_add = include_bytes!("../../../tests/fixtures/table_named_speed_live_add_original.gsp");
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
        let blank = include_bytes!("../../../tests/fixtures/text_object_blank.gsp");
        let output = append_literal_sum(blank).unwrap();
        assert_eq!(object_group(&output, 1), calculation.as_bytes());
    }

    #[test]
    fn bounded_arithmetic_imports_native_and_generates_groups() {
        for (document, input, expression, value) in [
            (
                include_bytes!("../../../tests/fixtures/calculation_two_plus_three_native.gsp")
                    .as_slice(),
                "2+3",
                "2 + 3",
                5.,
            ),
            (
                include_bytes!("../../../tests/fixtures/calculation_seven_plus_four_native.gsp")
                    .as_slice(),
                "7+4",
                "7 + 4",
                11.,
            ),
            (
                include_bytes!("../../../tests/fixtures/calculation_seven_minus_four_native.gsp")
                    .as_slice(),
                "7-4",
                "7 − 4",
                3.,
            ),
            (
                include_bytes!("../../../tests/fixtures/calculation_seven_times_four_native.gsp")
                    .as_slice(),
                "7*4",
                "7 · 4",
                28.,
            ),
            (
                include_bytes!("../../../tests/fixtures/calculation_seven_divide_four_native.gsp")
                    .as_slice(),
                "7/4",
                "7 ÷ 4",
                1.75,
            ),
            (
                include_bytes!("../../../tests/fixtures/calculation_twelve_plus_thirtyfour_native.gsp")
                    .as_slice(),
                "12+34",
                "12 + 34",
                46.,
            ),
            (
                include_bytes!("../../../tests/fixtures/calculation_twelve_minus_thirtyfour_native.gsp")
                    .as_slice(),
                "12-34",
                "12 − 34",
                -22.,
            ),
            (
                include_bytes!("../../../tests/fixtures/calculation_twelve_times_thirtyfour_native.gsp")
                    .as_slice(),
                "12*34",
                "12 · 34",
                408.,
            ),
            (
                include_bytes!("../../../tests/fixtures/calculation_twelve_divide_thirtyfour_native.gsp")
                    .as_slice(),
                "12/34",
                "12 ÷ 34",
                12. / 34.,
            ),
        ] {
            let original = object_group(document, 1);
            let imported = CalculationGroup::parse(&original).unwrap();
            assert_eq!(imported.as_bytes(), original);
            assert_eq!(imported.expression(), expression);
            assert_eq!(imported.value(), value);
            let generated = CalculationGroup::small_integer_expression(input).unwrap();
            assert_eq!(generated.expression(), expression);
            assert_eq!(generated.value(), value);
            CalculationGroup::parse(generated.as_bytes()).unwrap();
        }
        assert!(CalculationGroup::small_integer_expression("7/0").is_err());
        assert!(CalculationGroup::small_integer_expression("100+3").is_err());
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

/// Read a supported group's local numeric caption font index.
pub(crate) fn numeric_caption_font_index(bytes: &[u8]) -> Result<usize, String> {
    let rs = records(bytes, 0)?;
    let record = rs
        .iter()
        .find(|record| record.kind == 0x07d8)
        .ok_or("Missing numeric caption font")?;
    if record.end - record.payload != 2 {
        return Err("Unsupported numeric caption font record".into());
    }
    Ok(u16::from_le_bytes(bytes[record.payload..record.end].try_into().unwrap()) as usize)
}

/// Native places the following caption one pixel after a cached bbox bottom.
pub(crate) fn cached_numeric_caption_step(bbox: ((i16, i16), (i16, i16))) -> Result<i32, String> {
    let height = i32::from(bbox.1.1) - i32::from(bbox.0.1);
    if !(1..=2048).contains(&height) {
        return Err("Unsupported numeric caption bounds".into());
    }
    Ok(height + 1)
}

/// A mathematical caption's extents on either side of its equation axis.
/// Fractions compose these extents; counting division tokens loses alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NumericCaptionLayout {
    pub above: i32,
    pub below: i32,
}

impl NumericCaptionLayout {
    pub fn flat(height: i32) -> Result<Self, String> {
        if height <= 0 {
            return Err("Invalid numeric caption height".into());
        }
        Ok(Self {
            above: height / 2,
            below: height - height / 2,
        })
    }
    pub fn height(self) -> Result<i32, String> {
        self.above
            .checked_add(self.below)
            .ok_or_else(|| "Numeric caption layout overflow".into())
    }
}

/// Proven Native caption shapes use the same row metrics in ordinary captions
/// and expanded math sources. Unit decorations belong to the displayed value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NumericCaptionShape {
    Flat,
    SquareUnit,
}
impl NumericCaptionShape {
    pub(crate) fn layout(
        self,
        mut base: NumericCaptionLayout,
    ) -> Result<NumericCaptionLayout, String> {
        if self == Self::SquareUnit {
            base.above = base
                .above
                .checked_add(2)
                .ok_or("Numeric caption layout overflow")?;
        }
        Ok(base)
    }
    pub(crate) fn source_layout(
        self,
        base: NumericCaptionLayout,
    ) -> Result<NumericCaptionLayout, String> {
        Ok(base)
    }
}
