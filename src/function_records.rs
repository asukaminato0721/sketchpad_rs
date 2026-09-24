//! Conservative support for the two function expressions observed in original
//! Sketchpad 5 files. This module deliberately rejects other expression trees.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SimpleFunction {
    Identity,
    Square,
}

impl SimpleFunction {
    pub fn parse(input: &str) -> Result<Self, String> {
        match input.trim().replace(' ', "").as_str() {
            "x" => Ok(Self::Identity),
            "x^2" | "x²" => Ok(Self::Square),
            _ => Err("Supported functions are x and x^2".into()),
        }
    }
    pub fn display(self) -> &'static str {
        match self { Self::Identity => "f(x) = x", Self::Square => "f(x) = x²" }
    }
    pub fn evaluate(self, x: f64) -> f64 {
        match self {
            Self::Identity => x,
            Self::Square => x * x,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FunctionPlot {
    pub expression: SimpleFunction,
    pub definition_id: usize,
    pub coordinate_system_id: usize,
    pub x_min: f64,
    pub x_max: f64,
    pub samples: u32,
}

#[derive(Clone, Copy, Debug)]
struct Record {
    kind: u32,
    start: usize,
    payload_start: usize,
    end: usize,
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn records(bytes: &[u8]) -> Result<Vec<Record>, String> {
    if !bytes.starts_with(b"GSP4") {
        return Err("Expected a GSP4 document".into());
    }
    let mut records = Vec::new();
    let mut start = 4usize;
    while start < bytes.len() {
        if bytes.len() - start < 8 {
            return Err("Truncated record header".into());
        }
        let length = u32_at(bytes, start) as usize;
        let payload_start = start + 8;
        let end = payload_start
            .checked_add(length)
            .and_then(|end| end.checked_add(length & 1))
            .ok_or("Record length overflow")?;
        if end > bytes.len() {
            return Err("Truncated record payload".into());
        }
        records.push(Record {
            kind: u32_at(bytes, start + 4),
            start,
            payload_start,
            end,
        });
        start = end;
    }
    Ok(records)
}

fn payload<'a>(bytes: &'a [u8], record: Record) -> &'a [u8] {
    let length = u32_at(bytes, record.start) as usize;
    &bytes[record.payload_start..record.payload_start + length]
}

pub fn decode_simple_expression(data: &[u8]) -> Result<SimpleFunction, String> {
    // The 0x0907 payload has a 92-byte wrapper followed by little-endian
    // postfix words. Bytes 48..92 contain session-dependent presentation data.
    if data.len() < 94
        || &data[..18]
            != &include_bytes!("../tests/fixtures/function_plot_identity.gsp")[0x96c..0x97e]
        || &data[18..22] != b"<0>\0"
        || u32_at(data, 32) != 0x0907
    {
        return Err("Unsupported function expression wrapper".into());
    }
    match (data.len(), u32_at(data, 36), u32_at(data, 44), &data[92..]) {
        (94, 62, 1, [15, 0]) => Ok(SimpleFunction::Identity),
        (98, 66, 3, [15, 0, 4, 16, 2, 0]) => Ok(SimpleFunction::Square),
        _ => Err("Unsupported function expression".into()),
    }
}

fn group_ranges(bytes: &[u8], records: &[Record]) -> Result<Vec<(usize, usize, u16)>, String> {
    let mut groups = Vec::new();
    let mut open = None;
    for (index, record) in records.iter().enumerate() {
        if record.kind == 0x07d0 {
            if open.is_some() || payload(bytes, *record).len() < 2 {
                return Err("Malformed object group".into());
            }
            open = Some((
                index,
                u16::from_le_bytes(payload(bytes, *record)[..2].try_into().unwrap()),
            ));
        } else if record.kind == 0x07d7 {
            if let Some((start, code)) = open.take() {
                groups.push((start, index, code));
            }
        }
    }
    if open.is_some() {
        return Err("Unterminated object group".into());
    }
    Ok(groups)
}

pub fn decode_minimal_plot(bytes: &[u8]) -> Result<FunctionPlot, String> {
    let records = records(bytes)?;
    let groups = group_ranges(bytes, &records)?;
    let mut found = None;
    for (ordinal, &(start, end, code)) in groups.iter().enumerate() {
        if code != 72 {
            continue;
        }
        if found.is_some() {
            return Err("Multiple function plots are unsupported".into());
        }
        let refs = records[start + 1..end]
            .iter()
            .find(|record| record.kind == 0x07d2)
            .ok_or("Function plot has no dependencies")?;
        let refs = payload(bytes, *refs);
        if refs.len() != 12 || u32_at(refs, 0) != 2 {
            return Err("Unsupported function plot dependencies".into());
        }
        let definition_id = u32_at(refs, 4) as usize;
        let coordinate_system_id = u32_at(refs, 8) as usize;
        if definition_id == 0 || definition_id >= ordinal + 1 || coordinate_system_id == 0 {
            return Err("Invalid function plot dependencies".into());
        }
        let &(definition_start, definition_end, definition_code) = groups
            .get(definition_id - 1)
            .ok_or("Missing function definition")?;
        if definition_code != 0 {
            return Err("Unsupported function definition object".into());
        }
        let expression_record = records[definition_start + 1..definition_end]
            .iter()
            .find(|record| record.kind == 0x0907)
            .ok_or("Function definition has no expression")?;
        let expression = decode_simple_expression(payload(bytes, *expression_record))?;
        let &(coordinates_start, _, coordinates_code) = groups
            .get(coordinate_system_id - 1)
            .ok_or("Missing coordinate system")?;
        if coordinates_code != 61 || coordinates_start >= start {
            return Err("Function plot references no coordinate system".into());
        }
        let descriptor = records[start + 1..end]
            .iter()
            .find(|record| record.kind == 0x0902)
            .ok_or("Function plot has no range")?;
        let descriptor = payload(bytes, *descriptor);
        if descriptor.len() != 40 || u32_at(descriptor, 20) & 0xffff != 0 {
            return Err("Unsupported function plot mode".into());
        }
        let x_min = f64::from_le_bytes(descriptor[0..8].try_into().unwrap());
        let x_max = f64::from_le_bytes(descriptor[8..16].try_into().unwrap());
        let samples = u32_at(descriptor, 16);
        if !x_min.is_finite()
            || !x_max.is_finite()
            || x_min >= x_max
            || !(2..=4096).contains(&samples)
        {
            return Err("Invalid function plot range".into());
        }
        found = Some(FunctionPlot {
            expression,
            definition_id,
            coordinate_system_id,
            x_min,
            x_max,
            samples,
        });
    }
    found.ok_or_else(|| "No Cartesian function plot".into())
}

fn original_template(expression: SimpleFunction) -> &'static [u8] {
    match expression {
        SimpleFunction::Identity => include_bytes!("../tests/fixtures/function_plot_identity.gsp"),
        SimpleFunction::Square => include_bytes!("../tests/fixtures/function_plot_square.gsp"),
    }
}

pub fn decode_plot_range(data: &[u8]) -> Result<(f64, f64, u32), String> {
    if data.len() != 40 || u32_at(data, 20) & 0xffff != 0 {
        return Err("Unsupported function plot mode".into());
    }
    let x_min = f64::from_le_bytes(data[0..8].try_into().unwrap());
    let x_max = f64::from_le_bytes(data[8..16].try_into().unwrap());
    let samples = u32_at(data, 16);
    if !x_min.is_finite() || !x_max.is_finite() || x_min >= x_max || !(2..=4096).contains(&samples) {
        return Err("Invalid function plot range".into());
    }
    Ok((x_min, x_max, samples))
}

pub fn encode_definition_group(expression: SimpleFunction) -> Result<Vec<u8>, String> {
    let source = original_template(expression);
    let rs = records(source)?;
    let groups = group_ranges(source, &rs)?;
    let (first, last, code) = groups[6];
    if code != 0 { return Err("Invalid function definition template".into()); }
    Ok(source[rs[first].start..rs[last].end].to_vec())
}

pub fn encode_plot_group(expression: SimpleFunction, definition_id: usize, frame_id: usize) -> Result<Vec<u8>, String> {
    let both = encode_plot_groups(expression, definition_id, frame_id)?;
    let definition_length = encode_definition_group(expression)?.len();
    Ok(both[definition_length..].to_vec())
}

/// Encode the two observed object groups, changing only the plot's references.
pub fn encode_plot_groups(
    expression: SimpleFunction,
    definition_id: usize,
    coordinate_system_id: usize,
) -> Result<Vec<u8>, String> {
    if definition_id == 0 || coordinate_system_id == 0 || coordinate_system_id >= definition_id {
        return Err("Invalid plot object IDs".into());
    }
    let definition_id = u32::try_from(definition_id).map_err(|_| "Too many objects")?;
    let coordinate_system_id =
        u32::try_from(coordinate_system_id).map_err(|_| "Too many objects")?;
    let source = original_template(expression);
    let records = records(source)?;
    let groups = group_ranges(source, &records)?;
    if groups.len() != 8 || groups[6].2 != 0 || groups[7].2 != 72 {
        return Err("Function template has unexpected groups".into());
    }
    let start = records[groups[6].0].start;
    let end = records[groups[7].1].end;
    let mut encoded = source[start..end].to_vec();
    let ref_record = records[groups[7].0 + 1..groups[7].1]
        .iter()
        .find(|record| record.kind == 0x07d2)
        .ok_or("Function plot template has no dependencies")?;
    let ref_offset = ref_record.payload_start - start;
    encoded[ref_offset + 4..ref_offset + 8].copy_from_slice(&definition_id.to_le_bytes());
    encoded[ref_offset + 8..ref_offset + 12].copy_from_slice(&coordinate_system_id.to_le_bytes());
    Ok(encoded)
}

/// Standalone writer proof: assemble original function groups on the port's
/// six-object coordinate-system document. The shared document writer can use
/// `encode_plot_groups` when function objects are integrated later.
pub fn encode_minimal_plot_document(expression: SimpleFunction) -> Result<Vec<u8>, String> {
    let mut bytes = include_bytes!("../tests/fixtures/function_base_frame.gsp").to_vec();
    let records = records(&bytes)?;
    let page = records
        .iter()
        .find(|record| record.kind == 0x03e8)
        .ok_or("Missing page record")?;
    if payload(&bytes, *page).len() < 28 || u32_at(&bytes, page.payload_start + 24) != 6 {
        return Err("Function base has unexpected object count".into());
    }
    let link = records
        .iter()
        .find(|record| record.kind == 0x232d)
        .ok_or("Missing coordinate-system page link")?;
    if payload(&bytes, *link) != 6u32.to_le_bytes() {
        return Err("Function base has unexpected coordinate system".into());
    }
    bytes[page.payload_start + 24..page.payload_start + 28].copy_from_slice(&8u32.to_le_bytes());
    let mut appended = encode_plot_groups(expression, 7, 6)?;
    appended.extend_from_slice(&4u32.to_le_bytes());
    appended.extend_from_slice(&0x2328u32.to_le_bytes());
    appended.extend_from_slice(&8u32.to_le_bytes());
    bytes.splice(link.start..link.start, appended);
    let decoded = decode_minimal_plot(&bytes)?;
    if decoded.expression != expression
        || decoded.definition_id != 7
        || decoded.coordinate_system_id != 6
    {
        return Err("Encoded function plot failed to reparse".into());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_identity_and_square_decode() {
        for (expression, source) in [
            (
                SimpleFunction::Identity,
                include_bytes!("../tests/fixtures/function_plot_identity.gsp").as_slice(),
            ),
            (
                SimpleFunction::Square,
                include_bytes!("../tests/fixtures/function_plot_square.gsp").as_slice(),
            ),
        ] {
            let plot = decode_minimal_plot(source).unwrap();
            assert_eq!(plot.expression, expression);
            assert_eq!(
                (plot.definition_id, plot.coordinate_system_id, plot.samples),
                (7, 6, 750)
            );
            assert!((plot.x_min + plot.x_max).abs() < 1e-12);
        }
    }

    #[test]
    fn expression_rejects_unknown_postfix_without_guessing() {
        let source = include_bytes!("../tests/fixtures/function_plot_identity.gsp");
        let records = records(source).unwrap();
        let expression = records.iter().find(|record| record.kind == 0x0907).unwrap();
        let mut data = payload(source, *expression).to_vec();
        assert_eq!(
            decode_simple_expression(&data),
            Ok(SimpleFunction::Identity)
        );
        data[92] = 14;
        assert!(decode_simple_expression(&data).is_err());
    }

    #[test]
    fn number_new_function_uses_the_same_identity_expression() {
        let source = include_bytes!("../tests/fixtures/function_definition_identity.gsp");
        let records = records(source).unwrap();
        let expression = records.iter().find(|record| record.kind == 0x0907).unwrap();
        assert_eq!(
            decode_simple_expression(payload(source, *expression)),
            Ok(SimpleFunction::Identity)
        );
    }

    #[test]
    fn template_groups_and_minimal_document_reparse() {
        for expression in [SimpleFunction::Identity, SimpleFunction::Square] {
            let bytes = encode_minimal_plot_document(expression).unwrap();
            let plot = decode_minimal_plot(&bytes).unwrap();
            assert_eq!(plot.expression, expression);
            assert_eq!(
                expression.evaluate(2.0),
                match expression {
                    SimpleFunction::Identity => 2.0,
                    SimpleFunction::Square => 4.0,
                }
            );
        }
    }

    #[test]
    fn moved_original_plot_keeps_expression_and_dependencies() {
        let moved = include_bytes!("../tests/fixtures/function_port_square_moved_wine.gsp");
        let plot = decode_minimal_plot(moved).unwrap();
        assert_eq!(plot.expression, SimpleFunction::Square);
        assert_eq!((plot.definition_id, plot.coordinate_system_id), (7, 6));
    }
}
