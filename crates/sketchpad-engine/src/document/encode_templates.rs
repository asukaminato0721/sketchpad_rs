use super::{Document, Kind, Object};
use crate::graph;

impl Document {
    pub(super) fn arc_length_measurement_group(
        references: &[u32],
    ) -> Result<(Vec<u8>, usize), String> {
        if !matches!(references.len(), 1 | 3) {
            return Err("Arc length measurement needs one arc or a circle and two points".into());
        }
        let mut group =
            include_bytes!("../../../../assets/arc_length_measurement_group.bin").to_vec();
        if references.len() == 3 {
            group[36..40].copy_from_slice(&16u32.to_le_bytes());
            group[44..48].copy_from_slice(&3u32.to_le_bytes());
            group.splice(52..52, [0; 8]);
        }
        for (index, reference) in references.iter().enumerate() {
            group[48 + index * 4..52 + index * 4].copy_from_slice(&reference.to_le_bytes());
        }
        Ok((group, 56 + references.len() * 4))
    }

    pub(super) fn ratio_measurement_group(references: &[u32]) -> Result<(Vec<u8>, usize), String> {
        if !matches!(references.len(), 2 | 3) {
            return Err("Ratio measurement needs two segments or three points".into());
        }
        let mut group = include_bytes!("../../../../assets/ratio_measurement_group.bin").to_vec();
        if references.len() == 2 {
            group[36..40].copy_from_slice(&12u32.to_le_bytes());
            group[44..48].copy_from_slice(&2u32.to_le_bytes());
            group.drain(56..60);
        }
        for (index, reference) in references.iter().enumerate() {
            group[48 + index * 4..52 + index * 4].copy_from_slice(&reference.to_le_bytes());
        }
        Ok((group, 56 + references.len() * 4))
    }

    pub(super) fn place_measurement(
        group: &mut [u8],
        format_offset: usize,
        row: &mut usize,
    ) -> Result<(), String> {
        let y = row
            .checked_mul(28)
            .and_then(|v| v.checked_add(5))
            .ok_or("Too many measurement labels")?;
        let y = u16::try_from(y).map_err(|_| "Too many measurement labels")?;
        group[format_offset + 14..format_offset + 16].copy_from_slice(&y.to_le_bytes());
        *row += 1;
        Ok(())
    }
    pub(super) fn graph_template_group(&self, object: &Object) -> Result<Option<Vec<u8>>, String> {
        let ordinary = include_bytes!("../../../../assets/graph_coordinate_system_groups.bin");
        let circle = include_bytes!("../../../../assets/graph_unit_circle_groups.bin");
        let distance = include_bytes!("../../../../assets/graph_distance_square_groups.bin");
        let rectangular =
            include_bytes!("../../../../assets/graph_distance_rectangular_groups.bin");
        let kind = |id: usize| self.objects.get(id.checked_sub(1)?).map(|o| &o.kind);
        let template: &[u8] = match object.kind {
            Kind::Free(_)
                if matches!(self.objects.get(object.id).map(|o| &o.kind),
                Some(Kind::GraphHorizontalUnit(origin, _)) if *origin == object.id) =>
            {
                &ordinary[..68]
            }
            Kind::Free(_)
                if matches!(self.objects.get(object.id).map(|o| &o.kind),
                Some(Kind::DistanceAxis(origin, _, 1)) if *origin == object.id) =>
            {
                &distance[..68]
            }
            Kind::GraphHorizontalUnit(..) => &ordinary[68..148],
            Kind::GraphVerticalUnit(..) => &ordinary[268..348],
            Kind::GraphAxis(_, _, 1 | 0x1_0001) => &ordinary[148..268],
            Kind::GraphAxis(_, unit, 0)
                if matches!(kind(unit), Some(Kind::DistanceVerticalUnit(..))) =>
            {
                &distance[268..388]
            }
            Kind::GraphAxis(_, _, 0) => &ordinary[348..468],
            Kind::CircleAxis(_, 1) => &circle[..116],
            Kind::CircleAxis(_, 0) => &circle[116..232],
            Kind::DistanceAxis(_, _, 1) => &distance[68..188],
            Kind::DistanceAxis(_, _, 0) => &rectangular[188..308],
            Kind::DistanceVerticalUnit(..) => &distance[188..268],
            Kind::CoordinateSystem(x_axis, y_axis) => match kind(x_axis) {
                Some(Kind::CircleAxis(..)) => &circle[232..312],
                Some(Kind::DistanceAxis(..))
                    if matches!(kind(y_axis), Some(Kind::DistanceAxis(..))) =>
                {
                    &rectangular[308..388]
                }
                Some(Kind::DistanceAxis(..)) => &distance[388..468],
                _ => &ordinary[468..548],
            },
            _ => return Ok(None),
        };
        let mut group = template.to_vec();
        if let Kind::Free(point) = object.kind {
            group[44..52].copy_from_slice(&point.x.to_le_bytes());
            group[52..60].copy_from_slice(&point.y.to_le_bytes());
        } else {
            for (index, parent) in object.kind.dependencies().into_iter().enumerate() {
                group[48 + index * 4..52 + index * 4].copy_from_slice(
                    &u32::try_from(parent)
                        .map_err(|_| "Graph reference exceeds native index range")?
                        .to_le_bytes(),
                );
            }
        }
        if object.visible {
            group[10] &= !1;
        } else if matches!(object.kind, Kind::GraphVerticalUnit(..)) {
            group[10] = 1;
        } else {
            group[10] |= 1;
        }
        let (form, dotted) = self
            .coordinate_system_for_object(object.id)
            .and_then(|frame| self.frame_styles.get(&frame).copied())
            .unwrap_or((self.grid_form[object.page], self.grid_dotted[object.page]));
        match object.kind {
            Kind::CoordinateSystem(..) => {
                graph::set_coordinate_system_style(&mut group, form, dotted)?
            }
            Kind::GraphHorizontalUnit(_, step) => {
                graph::set_horizontal_unit_step(&mut group, step)?
            }
            Kind::GraphVerticalUnit(_, step) => {
                graph::set_vertical_unit_form(&mut group, form, step)?
            }
            Kind::DistanceVerticalUnit(_, step, independent) => {
                graph::set_distance_vertical_unit_form(&mut group, independent, step)?
            }
            Kind::GraphAxis(_, _, axis @ (1 | 0x1_0001)) => {
                graph::set_x_axis_trigonometric(&mut group, axis == 0x1_0001)?
            }
            _ => {}
        }
        if let Some(label) = &object.label {
            let mut framed = b"GSP4".to_vec();
            framed.extend_from_slice(&group);
            let entries = super::records::records(&framed)?;
            if entries.iter().any(|record| record.kind == 0x07d8) {
                group = Self::caption_group_with_label(&group, Some(label))?;
            } else {
                let end = entries
                    .iter()
                    .find(|record| record.kind == 0x07d7)
                    .ok_or("Missing graph object terminator")?
                    .start
                    - 4;
                group.splice(
                    end..end,
                    crate::text_records::encode_point_label_records(label),
                );
            }
        }
        Ok(Some(group))
    }
}
