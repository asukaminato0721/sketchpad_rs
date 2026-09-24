//! Graph > Plot Value on Axis records observed for a parameter and an axis.

const TEMPLATE: &[u8; 64] = include_bytes!("../assets/graph_plot_value_axis_group.bin");

fn word(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

/// Returns the local IDs of the value and its path.
pub fn decode_group(group: &[u8]) -> Result<[u32; 2], String> {
    if group.len() != 64
        || word(group, 0) != 28
        || word(group, 4) != 0x07d0
        || word(group, 8) != 95
        || word(group, 36) != 12
        || word(group, 40) != 0x07d2
        || word(group, 44) != 2
        || word(group, 56) != 0
        || word(group, 60) != 0x07d7
    {
        return Err("Unsupported Plot Value on Axis group".into());
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
}
