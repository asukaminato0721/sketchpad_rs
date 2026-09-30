//! Native given categories, interpreted only inside an isolated tool canvas.

/// Categories independently established by native custom-tool controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmbeddedToolInputKind {
    Point,
    /// A path placeholder shared by native segment and circle controls.
    Path,
    Segment,
    /// A generic numeric magnitude; its units are determined when bound.
    Numeric,
    /// A scalar Parameter specifically, retaining Native's stricter input
    /// category instead of accepting every scalar-valued construction.
    ScalarParameter,
}

/// A given's category is independent of its construction code. Native tools
/// use code zero for several distinct categories of input placeholder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmbeddedToolInput {
    pub local_id: usize,
    pub native_class: u16,
    pub native_subtype: u16,
}

impl EmbeddedToolInput {
    pub fn kind(self) -> Option<EmbeddedToolInputKind> {
        match (self.native_class, self.native_subtype) {
            (1, 4) => Some(EmbeddedToolInputKind::Point),
            (1, 5) => Some(EmbeddedToolInputKind::Path),
            (1, 14) => Some(EmbeddedToolInputKind::Segment),
            (11, 33) => Some(EmbeddedToolInputKind::Numeric),
            (11, 79) => Some(EmbeddedToolInputKind::ScalarParameter),
            _ => None,
        }
    }

    pub fn is_point(self) -> bool {
        self.kind() == Some(EmbeddedToolInputKind::Point)
    }

    pub fn is_path(self) -> bool {
        self.kind() == Some(EmbeddedToolInputKind::Path)
    }
}

/// Validate the independently observed numeric given representation, without
/// interpreting native cache words as a value or an animation domain.
pub(super) fn numeric_given_layout(bytes: &[u8]) -> Result<(), String> {
    use crate::table_records::ParameterCalculationGroup;
    let framed = [b"GSP4".as_slice(), bytes].concat();
    let bytes = framed.as_slice();
    let rs = super::records::records(bytes)?;
    if rs.iter().map(|r| r.kind).collect::<Vec<_>>()
        != [0x07d0, 0x0907, 0x07d5, 0x07d8, 0x08a3, 0x07d7]
        || rs[1].data.len() != 94
        || rs[3].data.len() != 2
        || rs[4].data.len() != 8
        || !rs[5].data.is_empty()
    {
        return Err("Unsupported numeric tool given records".into());
    }
    let payload = &bytes[rs[1].data.clone()];
    let sample = ParameterCalculationGroup::from_expression(1, "t", "t+1", (0, 0))?;
    let sample_bytes = [b"GSP4".as_slice(), sample.as_bytes()].concat();
    let sr = super::records::records(&sample_bytes)?;
    let expected = &sample_bytes[sr[2].data.clone()];
    for range in [0..12, 16..22, 24..26, 32..36, 40..44, 46..48] {
        if payload[range.clone()] != expected[range] {
            return Err("Unsupported numeric tool given representation".into());
        }
    }
    if super::records::u32_at(payload, 36) != 62
        || u16::from_le_bytes(payload[44..46].try_into().unwrap()) != 1
        || payload[92..94] != [0, 0]
        || payload[26..28] != [3, 0]
        || u16::from_le_bytes(payload[28..30].try_into().unwrap()) > 5
    {
        return Err("Unsupported numeric tool given definition".into());
    }
    crate::text_records::LabelAux::parse(&bytes[rs[2].data.clone()])?;
    Ok(())
}

/// Validate the scalar-Parameter placeholder retained by older registered
/// tools. Native stores the same inert value/cache body as a generic numeric
/// given; subtype 79 supplies the stricter binding category.
pub(super) fn scalar_parameter_given_layout(bytes: &[u8]) -> Result<(), String> {
    numeric_given_layout(bytes)
}
