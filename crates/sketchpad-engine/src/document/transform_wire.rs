//! Native authoring for the shared transform model. Imported opaque records
//! remain owned by the preserving document serializer.
use super::records::{header, polygon_subtype, put_record};
use super::transform_model::{
    ElementType, NativeLiteralRole, NativeTransformFlavor, TransformDefinition,
};
use crate::text_records::LabelAux;

pub(super) struct PolygonWire {
    pub vertex_count: usize,
    pub interior_order: i32,
    pub vector_locus_target: bool,
}

pub(super) struct TransformWirePresentation<'a> {
    pub visible: bool,
    pub label: Option<&'a LabelAux>,
    pub polygon: Option<PolygonWire>,
    pub region_order: Option<i32>,
    pub visual_style: Option<crate::style_records::VisualStyle>,
}

pub(super) fn encode_group(
    definition: &TransformDefinition,
    presentation: TransformWirePresentation<'_>,
) -> Result<Vec<u8>, String> {
    use ElementType as E;
    use NativeTransformFlavor as F;
    let flavor = definition.wire_flavor();
    let element = definition.element();
    let code = flavor.code();
    let mut h = header(match element {
        E::Point => code,
        E::Segment => 2,
        E::Ray => 64,
        E::Line => 63,
        E::Circle => 3,
        E::Arc => 80,
        E::Polygon => 8,
        E::Sector => 82,
        E::CircularSegment => 83,
        E::Disk => 10,
        E::Image => return Err("Writing transformed pictures is not supported".into()),
    });
    h[..2].copy_from_slice(&code.to_le_bytes());
    let modern_stroke = matches!(
        flavor,
        F::Vector
            | F::MarkedAnglePolar
            | F::MarkedPolar
            | F::PointAngleFixedPolar
            | F::PointAngleMarkedPolar
            | F::PointAngleRotation
            | F::FixedRotation
            | F::NumericRotation
            | F::NumericDilation
            | F::FixedDilation
            | F::PointRatioDilation
    );
    match element {
        E::Point => {}
        E::Ray | E::Line => h[18..20].copy_from_slice(&[0x51, 0x5c]),
        E::Segment if modern_stroke => h[18..20].copy_from_slice(&[0x51, 0x5c]),
        E::Circle if matches!(flavor, F::Reflection | F::FixedDilation) => {
            h[18..20].copy_from_slice(&[0, 0]);
        }
        E::Circle if modern_stroke => h[18..20].copy_from_slice(&[0x51, 0x5c]),
        E::Arc if modern_stroke => h[18..20].copy_from_slice(&[0x51, 0x5c]),
        E::Polygon => {
            let polygon = presentation
                .polygon
                .as_ref()
                .ok_or("Missing polygon transform metadata")?;
            h[8..10].copy_from_slice(
                &polygon_subtype(polygon.vertex_count)
                    .ok_or("Polygon has too many vertices")?
                    .to_le_bytes(),
            );
            h[20..24].copy_from_slice(&polygon.interior_order.to_le_bytes());
            if !matches!(flavor, F::Reflection | F::FixedPolar | F::FixedCartesian) {
                h[18..20].copy_from_slice(&[0x51, 0x5c]);
            }
            if flavor == F::Vector && polygon.vector_locus_target {
                h[18..20].copy_from_slice(&[0x39, 0x20]);
            }
        }
        E::Sector | E::CircularSegment | E::Disk => {
            h[20..24].copy_from_slice(
                &presentation
                    .region_order
                    .ok_or("Missing region transform drawing order")?
                    .to_le_bytes(),
            );
        }
        _ => {}
    }
    if !presentation.visible {
        h[2] = 1;
    }
    if let Some(style) = presentation.visual_style {
        crate::style_records::set_visual_style(&mut h, style)?;
    }
    let mut group = Vec::new();
    put_record(&mut group, 0x07d0, &h);
    let references = definition.native_references();
    let mut payload = (references.len() as u32).to_le_bytes().to_vec();
    for reference in references {
        payload.extend_from_slice(
            &u32::try_from(reference)
                .map_err(|_| "Transform reference exceeds the native index range")?
                .to_le_bytes(),
        );
    }
    put_record(&mut group, 0x07d2, &payload);
    if let Some(payload) = literal_payload(definition)? {
        put_record(&mut group, 0x07d3, &payload);
    }
    if let Some(label) = presentation.label {
        group.extend_from_slice(&crate::text_records::encode_point_label_records(label));
    }
    put_record(&mut group, 0x07d7, &[]);
    Ok(group)
}

fn literal_payload(definition: &TransformDefinition) -> Result<Option<Vec<u8>>, String> {
    use NativeLiteralRole as R;
    use NativeTransformFlavor as F;
    let literals = definition.native_literals();
    let value = |role| {
        literals
            .iter()
            .find(|literal| literal.role == role)
            .map(|literal| literal.value)
            .ok_or("Missing native transform literal")
    };
    let payload = match definition.wire_flavor() {
        F::MarkedAnglePolar | F::PointAngleFixedPolar => {
            let units = value(R::PolarUnitsToPixels)?;
            let magnitude = value(R::PolarMagnitude)?;
            let mut payload = vec![0; 28];
            payload[..4].copy_from_slice(&u32::from(definition.wire_flavor().code()).to_le_bytes());
            payload[4..12].copy_from_slice(&(units * magnitude).to_le_bytes());
            payload[12..20].copy_from_slice(&magnitude.to_le_bytes());
            payload[20..24]
                .copy_from_slice(&if units == 96. { 2_u32 } else { 1_u32 }.to_le_bytes());
            payload
        }
        F::FixedPolar | F::MarkedPolar => {
            let angle = value(R::AngleDegrees)?;
            let mut payload = if definition.wire_flavor() == F::FixedPolar {
                include_bytes!("../../../../assets/polar_translation_payload.bin").to_vec()
            } else {
                include_bytes!("../../../../assets/marked_distance_translation_payload.bin")
                    .to_vec()
            };
            // Preserve the original exact cardinal template, including its cache.
            if angle != 90. {
                payload[4..12].copy_from_slice(&(-angle.to_radians().sin()).to_le_bytes());
                payload[12..20].copy_from_slice(&angle.to_radians().cos().to_le_bytes());
            }
            payload[20..28].copy_from_slice(&angle.to_le_bytes());
            if definition.wire_flavor() == F::FixedPolar {
                let units = value(R::PolarUnitsToPixels)?;
                let magnitude = value(R::PolarMagnitude)?;
                payload[32..40].copy_from_slice(&(units * magnitude).to_le_bytes());
                payload[40..48].copy_from_slice(&magnitude.to_le_bytes());
                if units == 96. {
                    payload[48..52].copy_from_slice(&2_u32.to_le_bytes());
                }
            }
            payload
        }
        F::FixedCartesian => {
            let mut payload =
                include_bytes!("../../../../assets/rectangular_fixed_translation_payload.bin")
                    .to_vec();
            let x = value(R::HorizontalMagnitude)?;
            let y = value(R::VerticalMagnitude)?;
            let x_scale = value(R::HorizontalUnitsToPixels)?;
            let y_scale = value(R::VerticalUnitsToPixels)?;
            // Tag 0 stores literal pixel displacement; tags 1 and 2 store
            // centimeters and inches. Validation rejects every other scale,
            // while this lower-level helper retains its tag-1 fallback for
            // literal-bit tests.
            let unit_tag = |scale: f64| {
                if (scale - 1.).abs() <= 1e-9 {
                    0
                } else {
                    u32::from((scale - 96.).abs() <= 1e-9) + 1
                }
            };
            payload[4..12].copy_from_slice(&(x_scale * x).to_le_bytes());
            payload[12..20].copy_from_slice(&x.to_le_bytes());
            payload[20..24].copy_from_slice(&unit_tag(x_scale).to_le_bytes());
            payload[24..32].copy_from_slice(&(y_scale * y).to_le_bytes());
            payload[32..40].copy_from_slice(&y.to_le_bytes());
            payload[40..44].copy_from_slice(&unit_tag(y_scale).to_le_bytes());
            payload
        }
        F::FixedHorizontal | F::FixedVertical => {
            let mut payload =
                include_bytes!("../../../../assets/mixed_rectangular_translation_payload.bin")
                    .to_vec();
            let centimeters = value(R::FixedCentimeters)?;
            payload[0] = definition.wire_flavor().code() as u8;
            payload[4..12].copy_from_slice(&(centimeters * 37.79527559055118).to_le_bytes());
            payload[12..20].copy_from_slice(&centimeters.to_le_bytes());
            payload
        }
        F::FixedDilation => {
            let mut payload = vec![30, 0, 0, 0];
            payload.extend_from_slice(&value(R::ScaleFactor)?.to_le_bytes());
            payload
        }
        F::FixedRotation => {
            let mut payload = include_bytes!("../../../../assets/rotation_payload.bin").to_vec();
            let angle = value(R::AngleDegrees)?;
            payload[4..12].copy_from_slice(&(-angle.to_radians().sin()).to_le_bytes());
            payload[12..20].copy_from_slice(&angle.to_radians().cos().to_le_bytes());
            payload[20..28].copy_from_slice(&angle.to_le_bytes());
            payload
        }
        _ => return Ok(None),
    };
    Ok(Some(payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Kind;

    #[test]
    fn cardinal_translation_preserves_native_cache_and_literal_bits() {
        let definition = Kind::PolarTranslatedPoint(1, 90., 17., -0.)
            .transform_definition()
            .unwrap();
        let payload = literal_payload(&definition).unwrap().unwrap();
        let template = include_bytes!("../../../../assets/polar_translation_payload.bin");
        assert_eq!(&payload[..20], &template[..20]);
        assert_eq!(&payload[32..40], &(-0f64).to_le_bytes());
        assert_eq!(&payload[40..48], &(-0f64).to_le_bytes());

        let definition = Kind::FixedRectangularTranslatedPoint(1, 17., -0., 23., 2.)
            .transform_definition()
            .unwrap();
        let payload = literal_payload(&definition).unwrap().unwrap();
        assert_eq!(&payload[4..12], &(-0f64).to_le_bytes());
        assert_eq!(&payload[12..20], &(-0f64).to_le_bytes());
        assert_eq!(&payload[24..32], &46f64.to_le_bytes());
        assert_eq!(&payload[32..40], &2f64.to_le_bytes());

        let definition = Kind::FixedRectangularTranslatedPoint(1, 1., 12., 1., -8.)
            .transform_definition()
            .unwrap();
        let payload = literal_payload(&definition).unwrap().unwrap();
        assert_eq!(&payload[4..12], &12f64.to_le_bytes());
        assert_eq!(&payload[12..20], &12f64.to_le_bytes());
        assert_eq!(&payload[20..24], &0u32.to_le_bytes());
        assert_eq!(&payload[24..32], &(-8f64).to_le_bytes());
        assert_eq!(&payload[32..40], &(-8f64).to_le_bytes());
        assert_eq!(&payload[40..44], &0u32.to_le_bytes());
    }

    #[test]
    fn repeated_references_and_signed_polygon_rank_remain_wire_data() {
        let definition = Kind::VectorTranslatedPolygon(7, 7, 3)
            .transform_definition()
            .unwrap();
        let group = encode_group(
            &definition,
            TransformWirePresentation {
                visible: false,
                label: None,
                polygon: Some(PolygonWire {
                    vertex_count: 5,
                    interior_order: -3,
                    vector_locus_target: true,
                }),
                region_order: None,
                visual_style: None,
            },
        )
        .unwrap();
        let mut document = b"GSP4".to_vec();
        document.extend_from_slice(&group);
        let records = super::super::records::records(&document).unwrap();
        assert_eq!(
            records.iter().map(|record| record.kind).collect::<Vec<_>>(),
            vec![0x07d0, 0x07d2, 0x07d7]
        );
        let header = &document[records[0].data.clone()];
        assert_eq!(&header[..3], &[16, 0, 1]);
        assert_eq!(&header[8..10], &54u16.to_le_bytes());
        assert_eq!(&header[18..20], &[0x39, 0x20]);
        assert_eq!(&header[20..24], &(-3i32).to_le_bytes());
        let references = &document[records[1].data.clone()];
        assert_eq!(references, [3u32, 7, 7, 3].map(u32::to_le_bytes).concat());
    }
}
