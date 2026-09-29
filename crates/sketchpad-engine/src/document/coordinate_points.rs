//! One literal coordinate and one live scalar or degree coordinate in a shared frame.
use super::records::{f64_at, put_record, u32_at};
use super::transform_model::NumericExpr;
use super::{Document, Kind, Object, Point};
use crate::table_records::NumericUnit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoordinatePointFlavor {
    FixedX,
    FixedY,
}
impl CoordinatePointFlavor {
    pub fn native_code(self) -> u32 {
        match self {
            Self::FixedX => 109,
            Self::FixedY => 110,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct CoordinatePointDefinition {
    frame: usize,
    x: NumericExpr,
    y: NumericExpr,
    flavor: CoordinatePointFlavor,
}
impl CoordinatePointDefinition {
    pub fn new(
        frame: usize,
        x: NumericExpr,
        y: NumericExpr,
        flavor: CoordinatePointFlavor,
    ) -> Result<Self, String> {
        let d = Self {
            frame,
            x,
            y,
            flavor,
        };
        if frame == 0 || d.parts().is_none() {
            return Err(
                "Select one finite literal and one live scalar or degree coordinate".into(),
            );
        }
        Ok(d)
    }
    fn parts(&self) -> Option<(usize, f64, NumericUnit)> {
        let (literal, value) = match self.flavor {
            CoordinatePointFlavor::FixedX => (&self.x, &self.y),
            CoordinatePointFlavor::FixedY => (&self.y, &self.x),
        };
        match (literal, value) {
            (
                NumericExpr::Literal {
                    magnitude,
                    unit: NumericUnit::Scalar,
                    scale,
                },
                NumericExpr::ObjectValue { id, unit },
            ) if magnitude.is_finite()
                && *scale == 1.
                && *id > 0
                && matches!(unit, NumericUnit::Scalar | NumericUnit::Degrees) =>
            {
                Some((*id, *magnitude, *unit))
            }
            _ => None,
        }
    }
    pub fn fixed_coordinate(&self) -> f64 {
        self.parts().unwrap().1
    }
    pub(super) fn set_fixed_coordinate(&mut self, value: f64) {
        let expr = match self.flavor {
            CoordinatePointFlavor::FixedX => &mut self.x,
            CoordinatePointFlavor::FixedY => &mut self.y,
        };
        if let NumericExpr::Literal { magnitude, .. } = expr {
            *magnitude = value;
        }
    }
    pub fn frame(&self) -> usize {
        self.frame
    }
    pub fn x(&self) -> &NumericExpr {
        &self.x
    }
    pub fn y(&self) -> &NumericExpr {
        &self.y
    }
    pub fn wire_flavor(&self) -> CoordinatePointFlavor {
        self.flavor
    }
    pub fn native_references(&self) -> Vec<usize> {
        vec![self.parts().unwrap().0, self.frame]
    }
    pub fn native_payload(&self) -> [u8; 12] {
        let mut p = [0; 12];
        p[..4].copy_from_slice(&self.flavor.native_code().to_le_bytes());
        p[4..].copy_from_slice(&self.parts().unwrap().1.to_le_bytes());
        p
    }
    pub(super) fn supported(&self, objects: &[Object], page: usize, id: usize) -> bool {
        let Some((value, _, unit)) = self.parts() else {
            return false;
        };
        self.native_references().iter().all(|r| {
            *r < id
                && objects
                    .get(r.saturating_sub(1))
                    .is_some_and(|o| o.id == *r && !o.deleted && o.page == page)
        }) && matches!(objects[self.frame - 1].kind, Kind::CoordinateSystem(..))
            && objects[value - 1].kind.numeric_unit() == Some(unit)
    }
    pub(super) fn remap(&self, ids: &[usize]) -> Self {
        let mut d = self.clone();
        d.frame = ids[d.frame];
        for expr in [&mut d.x, &mut d.y] {
            if let NumericExpr::ObjectValue { id, .. } = expr {
                *id = ids[*id];
            }
        }
        d
    }
}
pub(super) fn decode(
    objects: &[Object],
    page: usize,
    id: usize,
    h: &[u8],
    refs: &[usize],
    payload: Option<&[u8]>,
) -> Option<CoordinatePointDefinition> {
    if h.len() != 28 || h[6..10] != [1, 0, 56, 0] || refs.len() != 2 {
        return None;
    }
    let code = u16::from_le_bytes(h[..2].try_into().ok()?);
    let flavor = match code {
        109 => CoordinatePointFlavor::FixedX,
        110 => CoordinatePointFlavor::FixedY,
        _ => return None,
    };
    let p = payload?;
    if p.len() != 12 || u32_at(p, 0) != u32::from(code) {
        return None;
    }
    let literal = NumericExpr::Literal {
        magnitude: f64_at(p, 4),
        unit: NumericUnit::Scalar,
        scale: 1.,
    };
    let unit = objects.get(refs[0].checked_sub(1)?)?.kind.numeric_unit()?;
    let value = NumericExpr::ObjectValue { id: refs[0], unit };
    let (x, y) = match flavor {
        CoordinatePointFlavor::FixedX => (literal, value),
        CoordinatePointFlavor::FixedY => (value, literal),
    };
    let d = CoordinatePointDefinition::new(refs[1], x, y, flavor).ok()?;
    d.supported(objects, page, id).then_some(d)
}
pub(super) fn encode_group(
    o: &Object,
    d: &CoordinatePointDefinition,
    refs: &[u32],
) -> Result<Vec<u8>, String> {
    let mut h = [
        109, 0, 0, 0, 0, 0, 1, 0, 56, 0, 33, 2, 255, 0, 0, 0, 255, 255, 81, 92, 193, 189, 240, 255,
        0, 0, 0, 0,
    ];
    h[..2].copy_from_slice(&(d.flavor.native_code() as u16).to_le_bytes());
    h[2] = u8::from(!o.visible) | (u8::from(o.presentation_suppressed) << 1);
    h[4] = u8::from(o.tracing);
    if let Some(style) = o.visual_style {
        crate::style_records::set_visual_style(&mut h, style)?;
    }
    let mut g = Vec::new();
    put_record(&mut g, 0x07d0, &h);
    let mut r = (refs.len() as u32).to_le_bytes().to_vec();
    for id in refs {
        r.extend_from_slice(&id.to_le_bytes());
    }
    put_record(&mut g, 0x07d2, &r);
    put_record(&mut g, 0x07d3, &d.native_payload());
    if let Some(label) = &o.label {
        g.extend(crate::text_records::encode_point_label_records(label));
    }
    put_record(&mut g, 0x07d7, &[]);
    Ok(g)
}
impl Document {
    pub fn set_fixed_coordinate(&mut self, id: usize, value: f64) -> Result<(), String> {
        if !value.is_finite() || (!self.safe_structure && self.page_count == 1) {
            return Err("Coordinate must be finite in a safe document".into());
        }
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid coordinate point")?)
            .ok_or("Invalid coordinate point")?;
        if object.deleted
            || !matches!(&object.kind, Kind::CoordinatePoint(d) if d.supported(&self.objects, object.page, object.id))
        {
            return Err("Select a coordinate point with a fixed coordinate".into());
        }
        self.checkpoint();
        if let Kind::CoordinatePoint(d) = &mut self.objects[id - 1].kind {
            d.set_fixed_coordinate(value);
        }
        self.dirty = true;
        Ok(())
    }
    pub(super) fn evaluate_coordinate_point(&self, d: &CoordinatePointDefinition) -> Option<Point> {
        let (value, literal, _) = d.parts()?;
        // Native coordinate construction consumes the displayed numeric
        // magnitude. In particular, a degree value plots 360° at x=360.
        let value = self.numeric_value(value)?;
        let (x, y) = match d.flavor {
            CoordinatePointFlavor::FixedX => (literal, value),
            CoordinatePointFlavor::FixedY => (value, literal),
        };
        let p = self.graph_frame(d.frame)?.world(x, y)?;
        Some(Point { x: p.x, y: p.y })
    }
    pub fn add_coordinate_point(
        &mut self,
        definition: CoordinatePointDefinition,
    ) -> Result<usize, String> {
        if self.partial_view
            || !self.safe_structure
            || self.page_count != 1
            || !definition.supported(&self.objects, self.active_page, self.objects.len() + 1)
        {
            return Err("Select a live scalar or degree value and coordinate frame".into());
        }
        self.checkpoint();
        Ok(self.append_object(Kind::CoordinatePoint(definition)))
    }
}
