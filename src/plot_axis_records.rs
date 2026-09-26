//! Graph > Plot Value on Axis/Path records for numeric values on axes and segments.

const TEMPLATE: &[u8; 64] = include_bytes!("../assets/graph_plot_value_axis_group.bin");
const LITERAL_TEMPLATE: &[u8; 80] = include_bytes!("../assets/graph_plot_value_literal_group.bin");

fn word(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

/// Returns the local IDs of the value and its path.
pub fn decode_group(group: &[u8]) -> Result<[u32; 2], String> {
    if group.len() < 64
        || word(group, 0) != 28
        || word(group, 4) != 0x07d0
        || word(group, 8) != 95
        || word(group, 36) != 12
        || word(group, 40) != 0x07d2
        || word(group, 44) != 2
    {
        return Err("Unsupported Plot Value on Axis group".into());
    }
    // Native calculation-driven path points can carry a point label and its
    // caption font after the references. These are presentation, not sources.
    let mut end = 56;
    if word(group, end + 4) == 0x07d5 {
        let len = word(group, end) as usize;
        let data = end.checked_add(8).and_then(|start| {
            start.checked_add(len).and_then(|stop| group.get(start..stop))
        }).ok_or("Invalid Plot Value point label")?;
        crate::text_records::LabelAux::parse(data)?;
        end += 8 + len + len % 2;
        if group.get(end..end + 10).is_none()
            || word(group, end) != 2 || word(group, end + 4) != 0x07d8 {
            return Err("Missing Plot Value point label font".into());
        }
        end += 10;
    }
    if group.len() != end + 8 || word(group, end) != 0 || word(group, end + 4) != 0x07d7 {
        return Err("Unsupported Plot Value on Axis suffix".into());
    }
    let refs = [word(group, 48), word(group, 52)];
    if refs[0] == 0 || refs[1] == 0 || refs[0] == refs[1] {
        return Err("Invalid Plot Value on Axis references".into());
    }
    Ok(refs)
}

pub fn encode_group(references: [u32; 2]) -> Result<Vec<u8>, String> {
    let mut group = TEMPLATE.to_vec();
    group[48..52].copy_from_slice(&references[0].to_le_bytes());
    group[52..56].copy_from_slice(&references[1].to_le_bytes());
    decode_group(&group)?;
    Ok(group)
}

/// A fixed numeric coordinate stored directly on a plotted point (code 123).
pub fn decode_literal_group(group: &[u8]) -> Result<(u32, f64), String> {
    if group.len() != 80
        || word(group, 0) != 28
        || word(group, 4) != 0x07d0
        || word(group, 8) != 123
        || word(group, 36) != 8
        || word(group, 40) != 0x07d2
        || word(group, 44) != 1
        || word(group, 52) != 12
        || word(group, 56) != 0x07d3
        || word(group, 60) != 123
        || word(group, 72) != 0
        || word(group, 76) != 0x07d7
    {
        return Err("Unsupported literal Plot Value on Axis group".into());
    }
    let axis = word(group, 48);
    let value = f64::from_le_bytes(group[64..72].try_into().unwrap());
    if axis == 0 || !value.is_finite() {
        return Err("Invalid literal Plot Value on Axis data".into());
    }
    Ok((axis, value))
}

pub fn encode_literal_group(axis: u32, value: f64) -> Result<Vec<u8>, String> {
    let mut group = LITERAL_TEMPLATE.to_vec();
    group[48..52].copy_from_slice(&axis.to_le_bytes());
    group[64..72].copy_from_slice(&value.to_le_bytes());
    decode_literal_group(&group)?;
    Ok(group)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_axis_value_group_references_parameter_and_x_axis() {
        assert_eq!(decode_group(TEMPLATE).unwrap(), [1, 5]);
        assert_eq!(encode_group([1, 5]).unwrap(), TEMPLATE.as_slice());
        assert_eq!(
            decode_group(&encode_group([2, 7]).unwrap()).unwrap(),
            [2, 7]
        );
    }

    #[test]
    fn original_literal_group_stores_its_axis_and_number() {
        assert_eq!(decode_literal_group(LITERAL_TEMPLATE).unwrap(), (3, 2.5));
        assert_eq!(
            encode_literal_group(3, 2.5).unwrap(),
            LITERAL_TEMPLATE.as_slice()
        );
        assert_eq!(
            decode_literal_group(&encode_literal_group(5, -1.25).unwrap()).unwrap(),
            (5, -1.25)
        );
    }
}
