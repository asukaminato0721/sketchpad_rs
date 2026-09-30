//! Native given categories, interpreted only inside an isolated tool canvas.

/// Categories independently established by native custom-tool controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmbeddedToolInputKind {
    Point,
    /// A path placeholder shared by native segment and circle controls.
    Path,
    Segment,
    /// A polygon of arbitrary supported vertex count.
    Polygon,
    /// The active Cartesian or polar coordinate system used by graph objects.
    CoordinateSystem,
    /// A generic numeric magnitude; its units are determined when bound.
    Numeric,
    /// A distance-valued magnitude with Native's explicit length restriction.
    Distance,
    /// A function definition or another supported function-valued object.
    Function,
    /// A scalar Parameter specifically, retaining Native's stricter input
    /// category instead of accepting every scalar-valued construction.
    ScalarParameter,
    /// An angle Parameter specifically, retaining Native's `Match Angle`
    /// category instead of accepting every degree-valued construction.
    AngleParameter,
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
            (1, 15) => Some(EmbeddedToolInputKind::Polygon),
            (1, 39) => Some(EmbeddedToolInputKind::CoordinateSystem),
            (11, 33) => Some(EmbeddedToolInputKind::Numeric),
            (11, 48) => Some(EmbeddedToolInputKind::Distance),
            (11, 51) => Some(EmbeddedToolInputKind::Numeric),
            (11, 57) => Some(EmbeddedToolInputKind::Function),
            (11, 79) => Some(EmbeddedToolInputKind::ScalarParameter),
            (11, 81) => Some(EmbeddedToolInputKind::AngleParameter),
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

/// Validate the older source-free Calculation representation of a generic
/// numeric given. Native presents subtype 51 with the same Measurement Value
/// matching category as subtype 33, while retaining this distinct body.
pub(super) fn calculation_given_layout(bytes: &[u8]) -> Result<(), String> {
    crate::table_records::CalculationGroup::parse(bytes)
        .map(|_| ())
        .map_err(|error| format!("Unsupported calculation tool given: {error}"))
}

/// Validate the source-free literal body retained by a distance given. Its
/// literal zero is intentionally unitless; subtype 48 carries the binding unit.
pub(super) fn distance_given_layout(bytes: &[u8]) -> Result<(), String> {
    if bytes.get(16..18) != Some(&48_u16.to_le_bytes()) {
        return Err("Unsupported distance tool given subtype".into());
    }
    let mut normalized = bytes.to_vec();
    normalized[16..18].copy_from_slice(&51_u16.to_le_bytes());
    crate::table_records::CalculationGroup::parse(&normalized)
        .map(|_| ())
        .map_err(|error| format!("Unsupported distance tool given: {error}"))
}

/// Validate the scalar-Parameter placeholder retained by older registered
/// tools. Native stores the same inert value/cache body as a generic numeric
/// given; subtype 79 supplies the stricter binding category.
pub(super) fn scalar_parameter_given_layout(bytes: &[u8]) -> Result<(), String> {
    numeric_given_layout(bytes)
}

/// Validate the angle-Parameter placeholder retained by an older registered
/// tool. Subtype 81 supplies the degree-valued matching category.
pub(super) fn angle_parameter_given_layout(bytes: &[u8]) -> Result<(), String> {
    numeric_given_layout(bytes)
}

/// Validate the inert expression/cache body used by a native Function given.
/// Subtype 57, rather than that body, supplies the binding category.
pub(super) fn function_given_layout(bytes: &[u8]) -> Result<(), String> {
    let framed = [b"GSP4".as_slice(), bytes].concat();
    let bytes = framed.as_slice();
    let rs = super::records::records(bytes)?;
    if rs.iter().map(|record| record.kind).collect::<Vec<_>>()
        != [0x07d0, 0x0907, 0x07d5, 0x07d8, 0x08a3, 0x07d7]
        || bytes[rs[3].data.clone()] != [0, 0]
        || rs[4].data.len() != 8
        || !rs[5].data.is_empty()
    {
        return Err("Unsupported function tool given records".into());
    }
    crate::function_records::decode_simple_expression(&bytes[rs[1].data.clone()])?;
    crate::text_records::LabelAux::parse(&bytes[rs[2].data.clone()])?;
    Ok(())
}
