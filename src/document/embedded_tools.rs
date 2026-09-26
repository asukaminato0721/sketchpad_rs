use super::{Document, Record, records};
use std::ops::Range;

/// An embedded tool has its own canvas and object-reference namespace.
/// Its prototype records are retained separately from the drawing objects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmbeddedToolSummary {
    pub name: Option<String>,
    pub object_codes: Vec<u16>,
    /// Ordered local object IDs required when invoking the tool.
    /// None means the input declaration is missing or malformed.
    pub input_ids: Option<Vec<u32>>,
    pub complete: bool,
}

pub(super) fn document_record_indices(records: &[Record]) -> Vec<usize> {
    let mut depth = 0usize;
    records
        .iter()
        .enumerate()
        .filter_map(|(index, record)| match record.kind {
            0x03ea => {
                depth += 1;
                None
            }
            0x03eb if depth > 0 => {
                depth -= 1;
                None
            }
            _ => (depth == 0).then_some(index),
        })
        .collect()
}

pub(super) fn document_records(records: &[Record]) -> impl Iterator<Item = &Record> {
    document_record_indices(records)
        .into_iter()
        .map(|index| &records[index])
}

fn tool_ranges(records: &[Record]) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (index, record) in records.iter().enumerate() {
        match record.kind {
            0x03ea => {
                if depth == 0 {
                    start = index;
                }
                depth += 1;
            }
            0x03eb if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    ranges.push(start..index + 1);
                }
            }
            _ => {}
        }
    }
    if depth > 0 {
        ranges.push(start..records.len());
    }
    ranges
}

impl Document {
    /// Inspect preserved tool prototypes. Tool execution and editing are not
    /// implied by recognizing their canvas boundaries.
    pub fn embedded_tools(&self) -> Result<Vec<EmbeddedToolSummary>, String> {
        let entries = records(&self.original)?;
        Ok(tool_ranges(&entries)
            .into_iter()
            .map(|range| {
                let section = &entries[range];
                let name = section
                    .iter()
                    .find(|record| record.kind == 0x044c)
                    .map(|record| {
                        let payload = &self.original[record.data.clone()];
                        let end = payload
                            .iter()
                            .position(|byte| *byte == 0)
                            .unwrap_or(payload.len());
                        String::from_utf8_lossy(&payload[..end]).into_owned()
                    });
                let object_codes: Vec<u16> = section
                    .iter()
                    .filter(|record| record.kind == 0x07d0)
                    .filter_map(|record| {
                        self.original
                            .get(record.data.clone())
                            .and_then(|payload| payload.get(..2))
                            .map(|bytes| u16::from_le_bytes(bytes.try_into().unwrap()))
                    })
                    .collect();
                let mut declarations = section.iter().filter(|r| r.kind == 0x04b4);
                let input_ids = declarations.next().and_then(|record| {
                    if declarations.next().is_some() {
                        return None;
                    }
                    let payload = &self.original[record.data.clone()];
                    let count = u32::from_le_bytes(payload.get(..4)?.try_into().ok()?) as usize;
                    if count.checked_mul(4)?.checked_add(4)? != payload.len() {
                        return None;
                    }
                    let ids: Vec<u32> = payload[4..]
                        .chunks_exact(4)
                        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
                        .collect();
                    let unique: std::collections::BTreeSet<_> = ids.iter().copied().collect();
                    (unique.len() == ids.len()
                        && ids
                            .iter()
                            .all(|&id| id > 0 && id as usize <= object_codes.len()))
                    .then_some(ids)
                });
                EmbeddedToolSummary {
                    name,
                    object_codes,
                    input_ids,
                    complete: section.last().is_some_and(|record| record.kind == 0x03eb),
                }
            })
            .collect())
    }
}
