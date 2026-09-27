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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Document, Kind};
    use crate::table_records::NumericUnit;

    const SCALAR_TOOL: &[u8] =
        include_bytes!("../../tests/fixtures/embedded_tool_scalar_created_native_verified.gsp");

    #[test]
    fn numeric_given_is_abstract_and_calculation_preserves_native_expression() {
        let drawing = Document::from_bytes(SCALAR_TOOL, None).unwrap();
        assert_eq!(drawing.encode().unwrap(), SCALAR_TOOL);
        let inputs = drawing.embedded_tool_inputs(0).unwrap();
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].kind(), Some(EmbeddedToolInputKind::Numeric));
        let prototype = drawing.embedded_tool_document(0).unwrap();
        assert!(!prototype.partial_view);
        assert_eq!(
            prototype.objects[0].kind,
            Kind::ToolInput(EmbeddedToolInputKind::Numeric)
        );
        assert_eq!(prototype.objects[0].kind.numeric_unit(), None);
        assert!(prototype.encode().is_err());
        let ordinary = Document::from_bytes(&prototype.original, None).unwrap();
        assert_eq!(ordinary.objects[0].kind, Kind::Unsupported(0));
        assert_eq!(ordinary.objects[1].kind, Kind::Unsupported(48));
        assert!(ordinary.partial_view);
        let Kind::ParameterCalculation(source, group) = &prototype.objects[1].kind else {
            panic!("Expected the abstract native calculation template");
        };
        assert_eq!(*source, 1);
        assert_eq!(group.sources(), [1]);
        assert_eq!(group.expression("t"), "(t + 1)");
        for unit in [
            NumericUnit::Scalar,
            NumericUnit::Length,
            NumericUnit::Degrees,
            NumericUnit::Area,
        ] {
            let bound = group.specialize_tool_sources(&[7], &[unit]).unwrap();
            assert_eq!(bound.sources(), [7]);
            assert_eq!(bound.unit(), NumericUnit::Scalar);
            assert_eq!(bound.value_for_sources(&[5.], &[unit]), Some(6.));
            assert_eq!(&bound.as_bytes()[16..18], &[51, 0]);
        }
        assert!(group.specialize_tool_sources(&[7], &[]).is_err());
        // The separately created multiplication template retains the bound
        // quantity, as independently saved Native length and angle controls do.
        let multiplied = Document::from_bytes(include_bytes!(
            "../../tests/fixtures/embedded_tool_numeric_multiply_length_native_verified.gsp"), None).unwrap();
        let product = multiplied.embedded_tool_document(0).unwrap();
        let Kind::ParameterCalculation(_, formula) = &product.objects[1].kind else {
            panic!("Expected the native multiplication template");
        };
        for (unit, subtype, value) in [
            (NumericUnit::Length, 48_u16, 5.),
            (NumericUnit::Degrees, 49_u16, 35.),
        ] {
            let bound = formula.specialize_tool_sources(&[9], &[unit]).unwrap();
            assert_eq!(bound.sources(), [9]);
            assert_eq!(bound.unit(), unit);
            assert_eq!(&bound.as_bytes()[16..18], &subtype.to_le_bytes());
            assert_eq!(bound.value_for_sources(&[value], &[unit]), Some(value * 2.));
        }

    }

    #[test]
    fn numeric_given_rejects_nonzero_definition_without_guessing_a_unit() {
        let drawing = Document::from_bytes(SCALAR_TOOL, None).unwrap();
        let mut prototype = drawing.embedded_tool_document(0).unwrap().original.clone();
        let rs = crate::document::records(&prototype).unwrap();
        let value = rs.iter().find(|r| r.kind == 0x0907).unwrap().data.start;
        prototype[value + 92..value + 94].copy_from_slice(&1_u16.to_le_bytes());
        let input = EmbeddedToolInput {
            local_id: 1,
            native_class: 11,
            native_subtype: 33,
        };
        assert!(
            Document::from_bytes_with_tool_inputs(&prototype, None, &[(1, input)].into()).is_err()
        );
    }
}
