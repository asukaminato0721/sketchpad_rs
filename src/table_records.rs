//! Bounded GSP4 support for a one-column snapshot table of the default t[1]
//! parameter. Unknown table layouts and values are refused before writing.

const TEMPLATE_DOCUMENT: &[u8] = include_bytes!("../tests/fixtures/table_parameter_tabulated.gsp");
const DEFAULT_PARAMETER: &[u8] = include_bytes!("../tests/fixtures/table_parameter_default.gsp");

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

impl TableGroup {
    pub fn default_snapshot() -> Self { Self { bytes: table_template() } }

    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let rs = records(bytes, 0)?;
        if rs.iter().map(|r| r.kind).collect::<Vec<_>>()
            != [0x07d0, 0x07d2, 0x090d, 0x07d8, 0x07d7]
        { return Err("Unsupported table record sequence".into()); }
        if rs[0].end - rs[0].payload != 28
            || u16::from_le_bytes(bytes[rs[0].payload..rs[0].payload + 2].try_into().unwrap()) != 91
            || rs[1].end - rs[1].payload != 8
            || u32::from_le_bytes(bytes[rs[1].payload..rs[1].payload + 4].try_into().unwrap()) != 1
            || rs[2].end - rs[2].payload != 64
            || u32::from_le_bytes(bytes[rs[2].payload..rs[2].payload + 4].try_into().unwrap()) != 0x090d
            || bytes[rs[3].payload..rs[3].end] != [1, 0]
        { return Err("Unsupported one-column table layout".into()); }
        let template = table_template();
        if bytes.len() != template.len() { return Err("Unsupported table byte length".into()); }
        let base = records(&template, 0)?;
        for (index, (actual, expected)) in bytes.iter().zip(template.iter()).enumerate() {
            let mutable = (rs[1].payload + 4..rs[1].payload + 8).contains(&index)
                || (rs[2].payload + 12..rs[2].payload + 16).contains(&index);
            if !mutable && actual != expected { return Err("Unsupported table variant".into()); }
        }
        if base.len() != rs.len() { return Err("Invalid table template".into()); }
        Ok(Self { bytes: bytes.to_vec() })
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
        let dragged = include_bytes!("../tests/fixtures/table_parameter_dragged.gsp");
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
}
