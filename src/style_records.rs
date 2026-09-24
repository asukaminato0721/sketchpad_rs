//! Point-style field in a Sketchpad 5 object header (`0x07d0`).
//!
//! Changing the style of a selected free point in the original application
//! changes payload byte 11. The same field occurs in 16- and 28-byte headers.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum PointStyle {
    Dot = 0,
    Small = 1,
    Medium = 2,
    Large = 3,
}

impl PointStyle {
    pub fn from_byte(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Dot),
            1 => Some(Self::Small),
            2 => Some(Self::Medium),
            3 => Some(Self::Large),
            _ => None,
        }
    }
}

fn is_point_header(payload: &[u8]) -> bool {
    (payload.len() == 16 || payload.len() == 28)
        && u16::from_le_bytes([payload[0], payload[1]]) == 0
}

pub fn point_style(payload: &[u8]) -> Option<PointStyle> {
    is_point_header(payload).then(|| PointStyle::from_byte(payload[11]))?
}

pub fn set_point_style(payload: &mut [u8], style: PointStyle) -> Result<(), String> {
    if !is_point_header(payload) {
        return Err("Expected a point object header".into());
    }
    payload[11] = style as u8;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_point_header(bytes: &[u8]) -> &[u8] {
        assert_eq!(&bytes[..4], b"GSP4");
        let mut offset = 4;
        while offset < bytes.len() {
            let length = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
            let kind = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
            let payload = &bytes[offset + 8..offset + 8 + length];
            if kind == 0x07d0 && is_point_header(payload) {
                return payload;
            }
            offset += 8 + length + length % 2;
        }
        panic!("fixture has no point header");
    }

    #[test]
    fn original_menu_variants_match_header_byte() {
        let fixtures: &[(&[u8], PointStyle)] = &[
            (include_bytes!("../tests/fixtures/style_dot_point.gsp"), PointStyle::Dot),
            (include_bytes!("../tests/fixtures/style_small_point.gsp"), PointStyle::Small),
            (include_bytes!("../tests/fixtures/style_large_point.gsp"), PointStyle::Large),
        ];
        for (bytes, style) in fixtures {
            let payload = first_point_header(bytes);
            assert_eq!(point_style(payload), Some(*style));
        }
        let default = first_point_header(include_bytes!("../tests/fixtures/point.gsp"));
        assert_eq!(point_style(default), Some(PointStyle::Medium));
    }

    #[test]
    fn editing_a_style_preserves_every_other_header_byte() {
        let original = first_point_header(include_bytes!("../tests/fixtures/point.gsp"));
        let mut changed = original.to_vec();
        set_point_style(&mut changed, PointStyle::Large).unwrap();
        assert_eq!(point_style(&changed), Some(PointStyle::Large));
        changed[11] = original[11];
        assert_eq!(changed, original);
    }

    #[test]
    fn non_point_and_unknown_styles_are_rejected() {
        assert_eq!(point_style(&[]), None);
        let mut header = first_point_header(include_bytes!("../tests/fixtures/point.gsp")).to_vec();
        header[11] = 4;
        assert_eq!(point_style(&header), None);
        header[0] = 1;
        assert!(set_point_style(&mut header, PointStyle::Dot).is_err());
    }
}
