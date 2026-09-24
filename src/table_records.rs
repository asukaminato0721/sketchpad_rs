//! Bounded GSP4 support for a one-column snapshot table of the default t[1]
//! parameter. Unknown table layouts and values are refused before writing.

const TEMPLATE_DOCUMENT: &[u8] = include_bytes!("../tests/fixtures/table_parameter_tabulated.gsp");
const DEFAULT_PARAMETER: &[u8] = include_bytes!("../tests/fixtures/table_parameter_default.gsp");
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
        if at + 8 > bytes.len() { return Err("Truncated GSP record".into()); }
        let len = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
        let payload = at + 8;
        let end = payload.checked_add(len + len % 2).ok_or("GSP record length overflow")?;
        if end > bytes.len() { return Err("Truncated GSP payload".into()); }
        result.push(Record {
            kind: u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap()),
            start: at, payload, end,
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
    let start = rs.iter().filter(|r| r.kind == 0x07d0).nth(ordinal - 1).unwrap().start;
    let end = rs.iter().filter(|r| r.kind == 0x07d7).nth(ordinal - 1).unwrap().end;
    document[start..end].to_vec()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterGroup { bytes: Vec<u8> }

impl ParameterGroup {
    pub fn default_value() -> Self { Self { bytes: object_group(DEFAULT_PARAMETER, 1) } }

    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let rs = records(bytes, 0)?;
        if rs.iter().map(|r| r.kind).collect::<Vec<_>>()
            != [0x07d0, 0x0907, 0x07d5, 0x07d8, 0x08a3, 0x07d7]
        { return Err("Unsupported parameter record sequence".into()); }
        let template = object_group(DEFAULT_PARAMETER, 1);
        let base = records(&template, 0)?;
        if bytes.len() != template.len() { return Err("Unsupported parameter byte length".into()); }
        for (i, (actual, expected)) in bytes.iter().zip(&template).enumerate() {
            let mutable = (rs[1].payload + 12..rs[1].payload + 16).contains(&i)
                || (rs[1].payload + 22..rs[1].payload + 24).contains(&i)
                || (rs[4].payload..rs[4].end).contains(&i);
            if !mutable && actual != expected { return Err("Unsupported parameter variant".into()); }
        }
        if base.len() != rs.len() { return Err("Invalid parameter template".into()); }
        let group = Self { bytes: bytes.to_vec() };
        let at = group.anchor();
        let bbox = group.bbox();
        if bbox.0 != at || bbox.1.0 - bbox.0.0 != 94 || bbox.1.1 - bbox.0.1 != 32 {
            return Err("Parameter anchor and box disagree".into());
        }
        Ok(group)
    }

    pub fn name(&self) -> &str { "t[1]" }
    pub fn value(&self) -> f64 { 1.0 }

    pub fn anchor(&self) -> (i16, i16) {
        let rs = records(&self.bytes, 0).unwrap();
        let at = rs[1].payload + 12;
        (i16::from_le_bytes(self.bytes[at..at + 2].try_into().unwrap()),
         i16::from_le_bytes(self.bytes[at + 2..at + 4].try_into().unwrap()))
    }

    pub fn bbox(&self) -> ((i16, i16), (i16, i16)) {
        let rs = records(&self.bytes, 0).unwrap();
        let at = rs[4].payload;
        let word = |offset| i16::from_le_bytes(self.bytes[at + offset..at + offset + 2].try_into().unwrap());
        ((word(0), word(2)), (word(4), word(6)))
    }

    pub fn set_anchor(&mut self, anchor: (i16, i16)) -> Result<(), String> {
        let rs = records(&self.bytes, 0)?;
        let before = self.anchor();
        let delta = (i32::from(anchor.0) - i32::from(before.0), i32::from(anchor.1) - i32::from(before.1));
        let at = rs[1].payload + 12;
        self.bytes[at..at + 2].copy_from_slice(&anchor.0.to_le_bytes());
        self.bytes[at + 2..at + 4].copy_from_slice(&anchor.1.to_le_bytes());
        let bbox = rs[4].payload;
        for (index, difference) in [(0, delta.0), (2, delta.1), (4, delta.0), (6, delta.1)] {
            let old = i16::from_le_bytes(self.bytes[bbox + index..bbox + index + 2].try_into().unwrap());
            let new = i16::try_from(i32::from(old) + difference).map_err(|_| "Parameter box coordinate overflow")?;
            self.bytes[bbox + index..bbox + index + 2].copy_from_slice(&new.to_le_bytes());
        }
        Ok(())
    }

    pub fn as_bytes(&self) -> &[u8] { &self.bytes }
}

pub fn append_default_parameter(blank: &[u8], anchor: (i16, i16)) -> Result<Vec<u8>, String> {
    append_to_blank(blank, ParameterGroup::default_value().with_anchor(anchor)?.as_bytes())
}

impl ParameterGroup {
    fn with_anchor(mut self, anchor: (i16, i16)) -> Result<Self, String> {
        self.set_anchor(anchor)?;
        Ok(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalculationGroup { bytes: Vec<u8> }

impl CalculationGroup {
    pub fn literal_two_plus_three() -> Self { Self { bytes: object_group(CALCULATION, 2) } }
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let rs = records(bytes, 0)?;
        if rs.iter().map(|r| r.kind).collect::<Vec<_>>() != [0x07d0, 0x0907, 0x07d8, 0x08a3, 0x07d7] {
            return Err("Unsupported calculation record sequence".into());
        }
        let template = object_group(CALCULATION, 2);
        if bytes.len() != template.len() { return Err("Unsupported calculation byte length".into()); }
        for (i, (actual, expected)) in bytes.iter().zip(&template).enumerate() {
            let mutable = (rs[1].payload + 12..rs[1].payload + 16).contains(&i)
                || (rs[3].payload..rs[3].end).contains(&i);
            if !mutable && actual != expected { return Err("Unsupported calculation variant".into()); }
        }
        Ok(Self { bytes: bytes.to_vec() })
    }
    pub fn expression(&self) -> &str { "2 + 3" }
    pub fn value(&self) -> f64 { 5.0 }
    pub fn as_bytes(&self) -> &[u8] { &self.bytes }
}

pub fn append_literal_sum(blank: &[u8]) -> Result<Vec<u8>, String> {
    append_to_blank(blank, CalculationGroup::literal_two_plus_three().as_bytes())
}

fn append_to_blank(blank: &[u8], group: &[u8]) -> Result<Vec<u8>, String> {
    if blank.get(..4) != Some(b"GSP4") { return Err("Expected GSP4".into()); }
    let rs = records(blank, 4)?;
    let preamble = rs.iter().find(|r| r.kind == 0x03e8).ok_or("Missing page preamble")?;
    if rs.iter().filter(|r| r.kind == 0x03e8).count() != 1
        || rs.iter().any(|r| r.kind == 0x07d0)
        || blank.get(preamble.payload + 24..preamble.payload + 28) != Some(&0_u32.to_le_bytes())
    { return Err("Expected blank one-page document".into()); }
    let trailer = rs.iter().find(|r| r.kind == 0x03e9).ok_or("Missing page trailer")?;
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
    if payload.len() % 2 != 0 { out.push(0); }
}

fn canonical_table(source: u32, anchor: (i16, i16), rows: usize) -> Result<Vec<u8>, String> {
    if source == 0 || !(1..=256).contains(&rows) { return Err("Invalid table source or row count".into()); }
    let base = table_template();
    let rs = records(&base, 0)?;
    let mut layout = base[rs[2].payload..rs[2].end].to_vec();
    layout[12..14].copy_from_slice(&anchor.0.to_le_bytes());
    layout[14..16].copy_from_slice(&anchor.1.to_le_bytes());
    layout[16..18].copy_from_slice(&((rows + 2) as u16).to_le_bytes());
    for _ in 1..rows { layout.extend_from_slice(SNAPSHOT_ROW); }
    while layout.len() % 4 != 0 { layout.push(0); }
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
    pub fn default_snapshot() -> Self { Self { bytes: table_template() } }

    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let rs = records(bytes, 0)?;
        if rs.iter().map(|r| r.kind).collect::<Vec<_>>()
            != [0x07d0, 0x07d2, 0x090d, 0x07d8, 0x07d7]
        { return Err("Unsupported table record sequence".into()); }
        if rs[1].end - rs[1].payload != 8 || rs[2].end - rs[2].payload < 64 {
            return Err("Unsupported table payload length".into());
        }
        let source = u32::from_le_bytes(bytes[rs[1].payload + 4..rs[1].payload + 8].try_into().unwrap());
        let at = rs[2].payload;
        let anchor = (
            i16::from_le_bytes(bytes[at + 12..at + 14].try_into().unwrap()),
            i16::from_le_bytes(bytes[at + 14..at + 16].try_into().unwrap()),
        );
        let row_count = u16::from_le_bytes(bytes[at + 16..at + 18].try_into().unwrap()) as usize;
        let rows = row_count.checked_sub(2).ok_or("Invalid table row count")?;
        if bytes != canonical_table(source, anchor, rows)?.as_slice() {
            return Err("Unsupported one-column table variant".into());
        }
        Ok(Self { bytes: bytes.to_vec() })
    }

    pub fn rows(&self) -> usize {
        let rs = records(&self.bytes, 0).unwrap();
        u16::from_le_bytes(self.bytes[rs[2].payload + 16..rs[2].payload + 18].try_into().unwrap()) as usize - 2
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
        u32::from_le_bytes(self.bytes[rs[1].payload + 4..rs[1].payload + 8].try_into().unwrap())
    }

    pub fn anchor(&self) -> (i16, i16) {
        let rs = records(&self.bytes, 0).unwrap();
        let at = rs[2].payload + 12;
        (i16::from_le_bytes(self.bytes[at..at + 2].try_into().unwrap()),
         i16::from_le_bytes(self.bytes[at + 2..at + 4].try_into().unwrap()))
    }

    pub fn set_anchor(&mut self, anchor: (i16, i16)) {
        let rs = records(&self.bytes, 0).unwrap();
        let at = rs[2].payload + 12;
        self.bytes[at..at + 2].copy_from_slice(&anchor.0.to_le_bytes());
        self.bytes[at + 2..at + 4].copy_from_slice(&anchor.1.to_le_bytes());
    }

    pub fn as_bytes(&self) -> &[u8] { &self.bytes }
}

/// Append a one-row table to the original single default t[1]=1.00 parameter.
/// The source value and name are verified so the table's cached row is accurate.
pub fn append_default_parameter_table(document: &[u8], anchor: (i16, i16)) -> Result<Vec<u8>, String> {
    if document.get(..4) != Some(b"GSP4") { return Err("Expected GSP4".into()); }
    let rs = records(document, 4)?;
    let base = records(DEFAULT_PARAMETER, 4)?;
    let preamble = rs.iter().find(|r| r.kind == 0x03e8).ok_or("Missing page preamble")?;
    if rs.iter().filter(|r| r.kind == 0x03e8).count() != 1
        || rs.iter().filter(|r| r.kind == 0x07d0).count() != 1
        || document.get(preamble.payload + 24..preamble.payload + 28) != Some(&1_u32.to_le_bytes())
    { return Err("Expected one-page one-parameter document".into()); }
    let start = rs.iter().position(|r| r.kind == 0x07d0).unwrap();
    let end = rs[start..].iter().position(|r| r.kind == 0x07d7).map(|n| start+n).ok_or("Unterminated parameter")?;
    let param = &rs[start..=end];
    if param.iter().map(|r| r.kind).collect::<Vec<_>>()
        != [0x07d0, 0x0907, 0x07d5, 0x07d8, 0x08a3, 0x07d7]
    { return Err("Unsupported parameter layout".into()); }
    let baseline_start = base.iter().position(|r| r.kind == 0x07d0).unwrap();
    let baseline = &base[baseline_start..baseline_start + param.len()];
    let actual = &document[param[1].payload..param[1].end];
    let expected = &DEFAULT_PARAMETER[baseline[1].payload..baseline[1].end];
    if actual.len() != expected.len()
        || actual.iter().zip(expected).enumerate().any(|(i, (a,b))| !matches!(i, 22 | 23) && a != b)
        || document[param[2].payload + 22..param[2].payload + 28] != *b"\x04\0t[1]"
    { return Err("Only the original t[1] = 1.00 parameter is supported".into()); }
    let selection = rs.iter().find(|r| r.kind == 0x2328 && r.start > param[5].start)
        .ok_or("Missing parameter selection record")?;
    if selection.end - selection.payload != 4 || document[selection.payload..selection.end] != 1_u32.to_le_bytes()
    { return Err("Unsupported selection record".into()); }
    let mut table = TableGroup::default_snapshot();
    table.set_anchor(anchor);
    let mut output = document.to_vec();
    output[preamble.payload + 24..preamble.payload + 28].copy_from_slice(&2_u32.to_le_bytes());
    output[selection.payload..selection.end].copy_from_slice(&2_u32.to_le_bytes());
    output.splice(selection.start..selection.start, table.as_bytes().iter().copied());
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
        assert_eq!(TableGroup::parse(&output[start..end]).unwrap().anchor(), (100, 116));
        assert!(append_default_parameter_table(
            include_bytes!("../tests/fixtures/table_parameter_custom.gsp"), (10, 38)).is_err());
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
