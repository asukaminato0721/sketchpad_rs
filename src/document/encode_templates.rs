use super::{Document, Kind, Object};
use crate::graph;
use crate::graph::GraphPoint;

impl Document {
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
        let distance_x_axis = match object.kind {
            Kind::Free(_)
                if matches!(
                    self.objects.get(object.id).map(|next| &next.kind),
                    Some(Kind::DistanceAxis(origin, _, 1)) if *origin == object.id
                ) =>
            {
                object.id.checked_add(1)
            }
            Kind::DistanceAxis(_, _, 1) => Some(object.id),
            Kind::DistanceAxis(_, _, 0) => object.id.checked_sub(1),
            Kind::DistanceVerticalUnit(axis, _, _) => Some(axis),
            Kind::GraphAxis(_, unit, 0) => match self
                .objects
                .get(unit.saturating_sub(1))
                .map(|other| &other.kind)
            {
                Some(Kind::DistanceVerticalUnit(axis, _, _)) => Some(*axis),
                _ => None,
            },
            Kind::CoordinateSystem(axis, _)
                if matches!(
                    self.objects
                        .get(axis.saturating_sub(1))
                        .map(|other| &other.kind),
                    Some(Kind::DistanceAxis(_, _, 1))
                ) =>
            {
                Some(axis)
            }
            _ => None,
        };
        if let Some(x_axis_id) = distance_x_axis {
            let Kind::DistanceAxis(origin_id, first_distance, 1) = self
                .objects
                .get(x_axis_id.saturating_sub(1))
                .ok_or("Missing defining-distance X axis")?
                .kind
            else {
                return Err("Invalid defining-distance X axis".into());
            };
            let new_origin =
                if origin_id.checked_add(1) == Some(x_axis_id) && origin_id > self.source_count {
                    let point = self
                        .objects
                        .get(origin_id - 1)
                        .ok_or("Missing graph origin")?;
                    let Kind::Free(position) = point.kind else {
                        return Err("Invalid defining-distance origin".into());
                    };
                    Some(graph::GraphPoint {
                        x: position.x,
                        y: position.y,
                    })
                } else {
                    None
                };
            let y = self
                .objects
                .get(x_axis_id)
                .ok_or("Missing defining-distance Y axis")?;
            let (distances, frame_id) = match y.kind {
                Kind::DistanceAxis(other, second_distance, 0) if other == origin_id => {
                    (vec![first_distance, second_distance], x_axis_id + 2)
                }
                Kind::DistanceVerticalUnit(host, step, _)
                    if host == x_axis_id && step.is_finite() && step > 0. =>
                {
                    let y_axis = self
                        .objects
                        .get(x_axis_id + 1)
                        .ok_or("Missing defining-distance Y axis")?;
                    if !matches!(y_axis.kind, Kind::GraphAxis(origin, unit, 0) if origin == origin_id && unit == x_axis_id + 1)
                    {
                        return Err("Invalid defining-distance Y axis".into());
                    }
                    (vec![first_distance], x_axis_id + 3)
                }
                _ => return Err("Unsupported defining-distance construction".into()),
            };
            let frame = self
                .objects
                .get(frame_id - 1)
                .ok_or("Missing defining-distance frame")?;
            if !matches!(frame.kind, Kind::CoordinateSystem(a,b) if a == x_axis_id && b == frame_id - 1)
            {
                return Err("Invalid defining-distance frame".into());
            }
            let (bytes, offsets) =
                graph::distance_coordinate_groups(x_axis_id, origin_id, &distances, new_origin)?;
            let first_id = if new_origin.is_some() {
                origin_id
            } else {
                x_axis_id
            };
            let index = object
                .id
                .checked_sub(first_id)
                .ok_or("Invalid graph object ID")?;
            let end = *offsets.get(index + 1).ok_or("Invalid graph object ID")?;
            let mut group = bytes[offsets[index]..end].to_vec();
            if !object.visible {
                group[10] |= 1;
            }
            if matches!(object.kind, Kind::CoordinateSystem(_, _)) {
                let (form, dotted) = self
                    .frame_styles
                    .get(&object.id)
                    .copied()
                    .unwrap_or((self.grid_form[object.page], self.grid_dotted[object.page]));
                graph::set_coordinate_system_style(&mut group, form, dotted)?;
            }
            if let Kind::DistanceVerticalUnit(_, step, independent) = object.kind {
                graph::set_distance_vertical_unit_form(&mut group, independent, step)?;
            }
            return Ok(Some(group));
        }
        let circle_axis_start = match object.kind {
            Kind::CircleAxis(_, 1) => Some(object.id),
            Kind::CircleAxis(_, 0) => object.id.checked_sub(1),
            Kind::CoordinateSystem(x_axis, _)
                if matches!(
                    self.objects.get(x_axis.saturating_sub(1)).map(|o| &o.kind),
                    Some(Kind::CircleAxis(_, 1))
                ) =>
            {
                Some(x_axis)
            }
            _ => None,
        };
        if let Some(first) = circle_axis_start {
            if first == 0 {
                return Err("Invalid unit-circle graph object ID".into());
            }
            let [Some(x_axis), Some(y_axis), Some(frame)] = self
                .objects
                .get(first - 1..first + 2)
                .map(|slice| [slice.get(0), slice.get(1), slice.get(2)])
                .unwrap_or([None; 3])
            else {
                return Err("Incomplete unit-circle coordinate system".into());
            };
            let Kind::CircleAxis(circle, 1) = x_axis.kind else {
                return Err("Invalid unit-circle X axis".into());
            };
            if !matches!(y_axis.kind, Kind::CircleAxis(other, 0) if other == circle)
                || !matches!(frame.kind, Kind::CoordinateSystem(a, b) if a == first && b == first + 1)
            {
                return Err("Invalid unit-circle coordinate system".into());
            }
            let bytes = graph::unit_circle_groups(first, circle)?;
            let offsets = [0, 116, 232, 312];
            let index = object
                .id
                .checked_sub(first)
                .ok_or("Invalid graph object ID")?;
            if index >= 3 {
                return Err("Invalid graph object ID".into());
            }
            let mut group = bytes[offsets[index]..offsets[index + 1]].to_vec();
            if !object.visible {
                group[10] |= 1;
            }
            if matches!(object.kind, Kind::CoordinateSystem(_, _)) {
                let (form, dotted) = self
                    .frame_styles
                    .get(&object.id)
                    .copied()
                    .unwrap_or((self.grid_form[object.page], self.grid_dotted[object.page]));
                graph::set_coordinate_system_style(&mut group, form, dotted)?;
            }
            return Ok(Some(group));
        }
        let shared_x_unit = match object.kind {
            Kind::GraphHorizontalUnit(_, _) => Some(object.id),
            Kind::GraphAxis(_, unit, 1 | 0x1_0001) => Some(unit),
            Kind::GraphVerticalUnit(unit, _) => Some(unit),
            Kind::GraphAxis(_, y_unit, 0) => {
                match y_unit
                    .checked_sub(1)
                    .and_then(|at| self.objects.get(at))
                    .map(|o| &o.kind)
                {
                    Some(Kind::GraphVerticalUnit(unit, _)) => Some(*unit),
                    _ => None,
                }
            }
            Kind::CoordinateSystem(x_axis, _) => {
                match x_axis
                    .checked_sub(1)
                    .and_then(|at| self.objects.get(at))
                    .map(|o| &o.kind)
                {
                    Some(Kind::GraphAxis(_, unit, 1 | 0x1_0001)) => Some(*unit),
                    _ => None,
                }
            }
            _ => None,
        };
        if let Some(x_unit_id) = shared_x_unit
            && x_unit_id > self.source_count
            && let Some(Kind::GraphHorizontalUnit(origin_id, _)) =
                self.objects.get(x_unit_id - 1).map(|o| &o.kind)
            && !(*origin_id > self.source_count && origin_id.checked_add(1) == Some(x_unit_id))
        {
            let [
                Some(x_unit),
                Some(x_axis),
                Some(y_unit),
                Some(y_axis),
                Some(frame),
            ] = self
                .objects
                .get(x_unit_id - 1..x_unit_id + 4)
                .map(|slice| {
                    [
                        slice.get(0),
                        slice.get(1),
                        slice.get(2),
                        slice.get(3),
                        slice.get(4),
                    ]
                })
                .unwrap_or([None; 5])
            else {
                return Err("Incomplete coordinate-system group".into());
            };
            let origin_id = *origin_id;
            let y_step = match y_unit.kind {
                Kind::GraphVerticalUnit(host, value)
                    if host == x_unit_id && value.is_finite() && value > 0.0 =>
                {
                    value
                }
                _ => return Err("Unsupported vertical unit".into()),
            };
            let form = self
                .frame_styles
                .get(&frame.id)
                .map_or(self.grid_form[object.page], |style| style.0);
            if !matches!(x_unit.kind, Kind::GraphHorizontalUnit(host, value) if host == origin_id && value.is_finite() && value > 0.0)
                || !matches!(x_axis.kind, Kind::GraphAxis(a,b,1 | 0x1_0001) if a == origin_id && b == x_unit_id)
                || !matches!(y_axis.kind, Kind::GraphAxis(a,b,0) if a == origin_id && b == x_unit_id+2)
                || !matches!(frame.kind, Kind::CoordinateSystem(a,b) if a == x_unit_id+1 && b == x_unit_id+3)
            {
                return Err("Unsupported coordinate-system construction".into());
            }
            let point = self
                .position(origin_id)
                .ok_or("Undefined coordinate-system origin")?;
            let bytes = graph::coordinate_system_groups_at_point(
                x_unit_id,
                origin_id,
                GraphPoint {
                    x: point.x,
                    y: point.y,
                },
            )?;
            let offsets = [0, 80, 200, 280, 400, 480];
            let index = object
                .id
                .checked_sub(x_unit_id)
                .ok_or("Invalid graph object ID")?;
            if index >= 5 {
                return Err("Invalid graph object ID".into());
            }
            let mut group = bytes[offsets[index]..offsets[index + 1]].to_vec();
            if !object.visible {
                group[10] |= 1;
            }
            if matches!(object.kind, Kind::CoordinateSystem(_, _)) {
                let (form, dotted) = self
                    .frame_styles
                    .get(&object.id)
                    .copied()
                    .unwrap_or((self.grid_form[object.page], self.grid_dotted[object.page]));
                graph::set_coordinate_system_style(&mut group, form, dotted)?;
            }
            if let Kind::GraphHorizontalUnit(_, step) = object.kind {
                graph::set_horizontal_unit_step(&mut group, step)?;
            }
            if matches!(object.kind, Kind::GraphVerticalUnit(_, _)) {
                graph::set_vertical_unit_form(&mut group, form, y_step)?;
            }
            if let Kind::GraphAxis(_, _, axis @ (1 | 0x1_0001)) = object.kind {
                graph::set_x_axis_trigonometric(&mut group, axis == 0x1_0001)?;
            }
            return Ok(Some(group));
        }
        let first = match object.kind {
            Kind::Free(_) if self.objects.get(object.id).is_some_and(|next| matches!(next.kind, Kind::GraphHorizontalUnit(host, _) if host == object.id)) => Some(object.id),
            Kind::GraphHorizontalUnit(_, _) => object.id.checked_sub(1),
            Kind::GraphAxis(_, _, 1 | 0x1_0001) => object.id.checked_sub(2),
            Kind::GraphVerticalUnit(_, _) => object.id.checked_sub(3),
            Kind::GraphAxis(_, _, 0) => object.id.checked_sub(4),
            Kind::CoordinateSystem(_, _) => object.id.checked_sub(5),
            _ => None,
        };
        let Some(first) = first else {
            return Ok(None);
        };
        if first == 0 {
            return Err("Invalid coordinate-system object ID".into());
        }
        let [
            Some(origin),
            Some(x_unit),
            Some(x_axis),
            Some(y_unit),
            Some(y_axis),
            Some(frame),
        ] = self
            .objects
            .get(first - 1..first + 5)
            .map(|slice| {
                [
                    slice.get(0),
                    slice.get(1),
                    slice.get(2),
                    slice.get(3),
                    slice.get(4),
                    slice.get(5),
                ]
            })
            .unwrap_or([None; 6])
        else {
            return Err("Incomplete coordinate-system group".into());
        };
        let form = self
            .frame_styles
            .get(&frame.id)
            .map_or(self.grid_form[object.page], |style| style.0);
        let y_step = match y_unit.kind {
            Kind::GraphVerticalUnit(host, value)
                if host == first + 1 && value.is_finite() && value > 0.0 =>
            {
                value
            }
            _ => return Err("Unsupported vertical unit".into()),
        };
        if !matches!(origin.kind, Kind::Free(_))
            || !matches!(x_unit.kind, Kind::GraphHorizontalUnit(host, value) if host == first && value.is_finite() && value > 0.0)
            || !matches!(x_axis.kind, Kind::GraphAxis(a,b,1 | 0x1_0001) if a == first && b == first+1)
            || !matches!(y_axis.kind, Kind::GraphAxis(a,b,0) if a == first && b == first+3)
            || !matches!(frame.kind, Kind::CoordinateSystem(a,b) if a == first+2 && b == first+4)
        {
            return Err("Unsupported coordinate-system construction".into());
        }
        let point = self
            .position(first)
            .ok_or("Undefined coordinate-system origin")?;
        let bytes = graph::coordinate_system_groups(
            first,
            GraphPoint {
                x: point.x,
                y: point.y,
            },
        )?;
        let offsets = [0, 68, 148, 268, 348, 468, 548];
        let index = object.id - first;
        let mut group = bytes[offsets[index]..offsets[index + 1]].to_vec();
        if object.visible {
            group[10] &= !1;
        } else if matches!(object.kind, Kind::GraphVerticalUnit(_, _)) {
            // The native hidden vertical-unit group drops its 0x40000
            // presentation flag instead of combining it with the hide bit.
            group[10] = 1;
        } else {
            group[10] |= 1;
        }
        if matches!(object.kind, Kind::CoordinateSystem(_, _)) {
            let (form, dotted) = self
                .frame_styles
                .get(&object.id)
                .copied()
                .unwrap_or((form, self.grid_dotted[object.page]));
            graph::set_coordinate_system_style(&mut group, form, dotted)?;
        }
        if let Kind::GraphHorizontalUnit(_, step) = object.kind {
            graph::set_horizontal_unit_step(&mut group, step)?;
        }
        if matches!(object.kind, Kind::GraphVerticalUnit(_, _)) {
            graph::set_vertical_unit_form(&mut group, form, y_step)?;
        }
        if let Kind::GraphAxis(_, _, axis @ (1 | 0x1_0001)) = object.kind {
            graph::set_x_axis_trigonometric(&mut group, axis == 0x1_0001)?;
        }
        Ok(Some(group))
    }
}
