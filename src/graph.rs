//! Decoding and evaluating the Cartesian graph objects used by Sketchpad 5.
//!
//! The document parser owns record framing and object IDs. This module accepts
//! one object's auxiliary records, so it can be used without duplicating that
//! parser or changing how unknown records are preserved.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoordinateFrame {
    pub origin: GraphPoint,
    pub x_unit: GraphPoint,
    pub y_unit: GraphPoint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridForm {
    Polar,
    Square,
    Rectangular,
}

impl GridForm {
    pub fn code(self) -> u8 {
        match self {
            Self::Polar => 0,
            Self::Square => 1,
            Self::Rectangular => 2,
        }
    }
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::Polar),
            1 => Some(Self::Square),
            2 => Some(Self::Rectangular),
            _ => None,
        }
    }
}

impl CoordinateFrame {
    pub fn world(self, x: f64, y: f64) -> Option<GraphPoint> {
        let point = GraphPoint {
            x: self.origin.x
                + x * (self.x_unit.x - self.origin.x)
                + y * (self.y_unit.x - self.origin.x),
            y: self.origin.y
                + x * (self.x_unit.y - self.origin.y)
                + y * (self.y_unit.y - self.origin.y),
        };
        (point.x.is_finite() && point.y.is_finite()).then_some(point)
    }
    /// Convert a screen-coordinate point to axis coordinates. The Y direction
    /// comes from the frame rather than from a hard-coded screen convention.
    pub fn coordinates(self, point: GraphPoint) -> Option<GraphPoint> {
        let xx = self.x_unit.x - self.origin.x;
        let xy = self.x_unit.y - self.origin.y;
        let yx = self.y_unit.x - self.origin.x;
        let yy = self.y_unit.y - self.origin.y;
        let dx = point.x - self.origin.x;
        let dy = point.y - self.origin.y;
        let determinant = xx.mul_add(yy, -xy * yx);
        let scale = xx.hypot(xy) * yx.hypot(yy);
        if !determinant.is_finite()
            || !scale.is_finite()
            || scale == 0.0
            || determinant.abs() <= 1e-12 * scale
        {
            return None;
        }
        let x = dx.mul_add(yy, -dy * yx) / determinant;
        let y = xx.mul_add(dy, -xy * dx) / determinant;
        (x.is_finite() && y.is_finite()).then_some(GraphPoint { x, y })
    }

    pub fn slope(self, a: GraphPoint, b: GraphPoint) -> Option<f64> {
        let a = self.coordinates(a)?;
        let b = self.coordinates(b)?;
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        if dx == 0.0 {
            return None;
        }
        let slope = dy / dx;
        slope.is_finite().then_some(slope)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GraphObject {
    /// Code 52: a unit point from the coordinate-system origin.
    HorizontalUnit { origin: usize, step: f64 },
    /// Code 54: the second unit point, constructed from the first.
    VerticalUnit { horizontal_unit: usize, step: f64 },
    /// Code 58: an axis through two points. `axis` is 1 for X, 0 for Y.
    Axis { start: usize, end: usize, axis: u32 },
    /// Code 60: an axis whose origin and unit scale come from a circle.
    CircleAxis { circle: usize, axis: u32 },
    /// Code 59: an axis scaled by a segment or measured distance.
    DistanceAxis {
        origin: usize,
        distance: usize,
        axis: u32,
    },
    /// Code 56: the matching Y-unit point for a single defining distance.
    DistanceVerticalUnit {
        x_axis: usize,
        step: f64,
        independent: bool,
    },
    /// Code 61: the coordinate system joining two axes.
    CoordinateSystem {
        x_axis: usize,
        y_axis: usize,
        form: GridForm,
        dotted: bool,
    },
    /// Code 87: a slope value relative to a coordinate system.
    Slope { line: usize, coordinates: usize },
}

impl GraphObject {
    pub fn dependencies(self) -> Vec<usize> {
        match self {
            Self::HorizontalUnit { origin, .. } => vec![origin],
            Self::VerticalUnit {
                horizontal_unit, ..
            } => vec![horizontal_unit],
            Self::Axis { start, end, .. } => vec![start, end],
            Self::CircleAxis { circle, .. } => vec![circle],
            Self::DistanceAxis {
                origin, distance, ..
            } => vec![origin, distance],
            Self::DistanceVerticalUnit { x_axis, .. } => vec![x_axis],
            Self::CoordinateSystem { x_axis, y_axis, .. } => vec![x_axis, y_axis],
            Self::Slope { line, coordinates } => vec![line, coordinates],
        }
    }
}

/// Page-level Snap Points flag, observed by dragging a free point in Sketchpad.
pub fn snap_points(page_record_payload: &[u8]) -> Option<bool> {
    if page_record_payload.len() != 36 {
        return None;
    }
    match page_record_payload[10] {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

fn u32_le(bytes: &[u8]) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

fn fixed_step(records: &[(u32, &[u8])], code: u16) -> Option<f64> {
    let payload = records.iter().find(|(kind, _)| *kind == 0x07d3)?.1;
    if payload.len() != 12 || u32_le(&payload[..4])? != u32::from(code) {
        return None;
    }
    let step = f64::from_le_bytes(payload[4..12].try_into().ok()?);
    (step.is_finite() && step > 0.0).then_some(step)
}

/// Decode a graph object from the object code, its already-normalized IDs,
/// and the auxiliary records between `0x07d0` and `0x07d7`.
///
/// `None` means that the code or payload is outside this observed subset.
/// Keep the original bytes and treat that object as unsupported in that case.
pub fn decode_graph_object(
    code: u16,
    refs: &[usize],
    records: &[(u32, &[u8])],
) -> Option<GraphObject> {
    if refs.contains(&0) {
        return None;
    }
    match (code, refs) {
        (52, [origin]) => Some(GraphObject::HorizontalUnit {
            origin: *origin,
            step: fixed_step(records, 52)?,
        }),
        (54 | 55, [horizontal_unit]) => Some(GraphObject::VerticalUnit {
            horizontal_unit: *horizontal_unit,
            step: fixed_step(records, code)?,
        }),
        (58, [start, end]) => {
            let payload = records.iter().find(|(kind, _)| *kind == 0x0905)?.1;
            let axis = u32_le(payload)?;
            matches!(axis, 0 | 1 | 0x1_0001).then_some(GraphObject::Axis {
                start: *start,
                end: *end,
                axis,
            })
        }
        (60, [circle]) => {
            let payload = records.iter().find(|(kind, _)| *kind == 0x0905)?.1;
            let axis = u32_le(payload)?;
            matches!(axis, 0 | 1).then_some(GraphObject::CircleAxis {
                circle: *circle,
                axis,
            })
        }
        (59, [origin, distance]) => {
            let payload = records.iter().find(|(kind, _)| *kind == 0x0905)?.1;
            let axis = u32_le(payload)?;
            matches!(axis, 0 | 1).then_some(GraphObject::DistanceAxis {
                origin: *origin,
                distance: *distance,
                axis,
            })
        }
        (56 | 57, [x_axis]) => Some(GraphObject::DistanceVerticalUnit {
            x_axis: *x_axis,
            step: fixed_step(records, code)?,
            independent: code == 57,
        }),
        (61, [x_axis, y_axis]) => {
            let payload = records.iter().find(|(kind, _)| *kind == 0x0904)?.1;
            (payload.len() == 8
                && GridForm::from_code(payload[0]).is_some()
                && payload[1] == 0
                && payload[2] <= 1
                && payload[3..] == [0; 5])
                .then_some(GraphObject::CoordinateSystem {
                    x_axis: *x_axis,
                    y_axis: *y_axis,
                    form: GridForm::from_code(payload[0])?,
                    dotted: payload[2] == 1,
                })
        }
        (87, [line, coordinates]) => {
            let format = records.iter().find(|(kind, _)| *kind == 0x0903)?.1;
            let style = records.iter().find(|(kind, _)| *kind == 0x07d8)?.1;
            let sample = &include_bytes!("../assets/graph_slope_measurement_group.bin")[64..96];
            // Sketchpad rewrites this style index from 2 to 1 after a drag.
            (format.len() == 32
                && u16::from_le_bytes(format[28..30].try_into().unwrap()) <= 5
                && format
                    .iter()
                    .zip(sample)
                    .enumerate()
                    .all(|(i, (actual, expected))| {
                        // Label position and the numeric presentation key vary.
                        matches!(i, 12..=15 | 22..=23 | 28..=29) || actual == expected
                    })
                && (style == [1, 0] || style == [2, 0]))
            .then_some(GraphObject::Slope {
                line: *line,
                coordinates: *coordinates,
            })
        }
        _ => None,
    }
}

/// Six original-app object groups produced by Graph > Define Coordinate System.
/// The returned IDs are `first_id..first_id + 6`; the last is the frame ID.
/// The caller must also update the page object count and append these groups
/// before the page trailer, as the normal document writer does for objects.
pub fn coordinate_system_groups(first_id: usize, origin: GraphPoint) -> Result<Vec<u8>, String> {
    if !origin.x.is_finite() || !origin.y.is_finite() || first_id == 0 {
        return Err("Invalid coordinate-system origin or first ID".into());
    }
    let shift = i64::try_from(first_id).map_err(|_| "Object ID overflow")? - 4;
    let mut groups = include_bytes!("../assets/graph_coordinate_system_groups.bin").to_vec();
    groups[44..52].copy_from_slice(&origin.x.to_le_bytes());
    groups[52..60].copy_from_slice(&origin.y.to_le_bytes());
    let mut offset = 0;
    while offset < groups.len() {
        let length = u32_le(&groups[offset..offset + 4]).ok_or("Invalid graph template")? as usize;
        let kind = u32_le(&groups[offset + 4..offset + 8]).ok_or("Invalid graph template")?;
        let payload = offset + 8;
        let end = payload
            .checked_add(length)
            .ok_or("Graph template overflow")?;
        if end > groups.len() {
            return Err("Invalid graph template length".into());
        }
        if kind == 0x07d2 {
            let count =
                u32_le(&groups[payload..payload + 4]).ok_or("Invalid graph references")? as usize;
            if length != 4 + count * 4 {
                return Err("Invalid graph references".into());
            }
            for index in 0..count {
                let at = payload + 4 + index * 4;
                let old = u32_le(&groups[at..at + 4]).ok_or("Invalid graph reference")?;
                let next = i64::from(old) + shift;
                let next = u32::try_from(next).map_err(|_| "Graph reference overflow")?;
                groups[at..at + 4].copy_from_slice(&next.to_le_bytes());
            }
        }
        offset = end + length % 2;
    }
    Ok(groups)
}

/// Five groups for Define Origin when the selected origin point already exists.
/// `first_id` is the new horizontal unit point; the frame is `first_id + 4`.
pub fn coordinate_system_groups_at_point(
    first_id: usize,
    origin_id: usize,
    origin: GraphPoint,
) -> Result<Vec<u8>, String> {
    let virtual_origin = first_id.checked_sub(1).ok_or("Invalid graph object ID")?;
    if origin_id == 0 || origin_id >= first_id {
        return Err("Invalid graph origin reference".into());
    }
    let mut groups = coordinate_system_groups(virtual_origin, origin)?;
    groups.drain(..68);
    let mut offset = 0;
    while offset < groups.len() {
        let length = u32_le(&groups[offset..offset + 4]).ok_or("Invalid graph template")? as usize;
        let kind = u32_le(&groups[offset + 4..offset + 8]).ok_or("Invalid graph template")?;
        let payload = offset + 8;
        let end = payload
            .checked_add(length)
            .ok_or("Graph template overflow")?;
        if end > groups.len() {
            return Err("Invalid graph template length".into());
        }
        if kind == 0x07d2 {
            let count =
                u32_le(&groups[payload..payload + 4]).ok_or("Invalid graph references")? as usize;
            if length != 4 + count * 4 {
                return Err("Invalid graph references".into());
            }
            for index in 0..count {
                let at = payload + 4 + index * 4;
                if u32_le(&groups[at..at + 4]) == Some(virtual_origin as u32) {
                    groups[at..at + 4].copy_from_slice(&(origin_id as u32).to_le_bytes());
                }
            }
        }
        offset = end + length % 2;
    }
    Ok(groups)
}

/// Two circle-based axes and their frame from Graph > Define Unit Circle.
pub fn unit_circle_groups(first_id: usize, circle_id: usize) -> Result<Vec<u8>, String> {
    let first = u32::try_from(first_id).map_err(|_| "Graph object ID overflow")?;
    let second = first.checked_add(1).ok_or("Graph object ID overflow")?;
    let circle = u32::try_from(circle_id).map_err(|_| "Circle ID overflow")?;
    if circle == 0 || circle >= first || first == 0 {
        return Err("Invalid circle coordinate-system references".into());
    }
    let mut groups = include_bytes!("../assets/graph_unit_circle_groups.bin").to_vec();
    if groups.len() != 312 {
        return Err("Invalid unit-circle template length".into());
    }
    for (start, code, count) in [(0, 60, 1), (116, 60, 1), (232, 61, 2)] {
        if u32_le(&groups[start + 8..start + 12]) != Some(code)
            || u32_le(&groups[start + 36..start + 40]) != Some(4 * (count + 1))
            || u32_le(&groups[start + 40..start + 44]) != Some(0x07d2)
            || u32_le(&groups[start + 44..start + 48]) != Some(count)
        {
            return Err("Invalid unit-circle template references".into());
        }
    }
    groups[48..52].copy_from_slice(&circle.to_le_bytes());
    groups[164..168].copy_from_slice(&circle.to_le_bytes());
    groups[280..284].copy_from_slice(&first.to_le_bytes());
    groups[284..288].copy_from_slice(&second.to_le_bytes());
    Ok(groups)
}

/// Native groups for Define Unit Distance(s), with an optional new origin.
/// The X axis is the first new axis; the caller supplies IDs in selection order.
pub fn distance_coordinate_groups(
    x_axis_id: usize,
    origin_id: usize,
    distances: &[usize],
    new_origin: Option<GraphPoint>,
) -> Result<(Vec<u8>, Vec<usize>), String> {
    if !matches!(distances.len(), 1 | 2) || x_axis_id == 0 || origin_id == 0 {
        return Err("Invalid defining distances".into());
    }
    let first = if new_origin.is_some() {
        x_axis_id.checked_sub(1).ok_or("Invalid graph origin ID")?
    } else {
        x_axis_id
    };
    if new_origin.is_some() && origin_id != first {
        return Err("Invalid new graph origin".into());
    }
    if new_origin.is_none() && origin_id >= first {
        return Err("Invalid selected graph origin".into());
    }
    if distances
        .iter()
        .any(|&distance| distance == 0 || distance >= first)
    {
        return Err("Invalid defining-distance reference".into());
    }
    let (template, sizes): (&[u8], &[usize]) = if distances.len() == 1 {
        (
            include_bytes!("../assets/graph_distance_square_groups.bin"),
            &[68, 120, 80, 120, 80],
        )
    } else {
        (
            include_bytes!("../assets/graph_distance_rectangular_groups.bin"),
            &[68, 120, 120, 80],
        )
    };
    if template.len() != sizes.iter().sum::<usize>() {
        return Err("Invalid defining-distance template".into());
    }
    let mut out = Vec::new();
    let mut offsets = vec![0];
    let mut at = 0;
    for (index, &size) in sizes.iter().enumerate() {
        let mut group = template[at..at + size].to_vec();
        at += size;
        if index == 0 {
            if let Some(origin) = new_origin {
                if !origin.x.is_finite() || !origin.y.is_finite() {
                    return Err("Invalid defining-distance origin".into());
                }
                group[44..52].copy_from_slice(&origin.x.to_le_bytes());
                group[52..60].copy_from_slice(&origin.y.to_le_bytes());
            } else {
                continue;
            }
        } else {
            let refs: Vec<usize> = if distances.len() == 1 {
                match index {
                    1 => vec![origin_id, distances[0]],
                    2 => vec![x_axis_id],
                    3 => vec![origin_id, x_axis_id + 1],
                    4 => vec![x_axis_id, x_axis_id + 2],
                    _ => unreachable!(),
                }
            } else {
                match index {
                    1 => vec![origin_id, distances[0]],
                    2 => vec![origin_id, distances[1]],
                    3 => vec![x_axis_id, x_axis_id + 1],
                    _ => unreachable!(),
                }
            };
            let mut record = 0;
            let mut patched = false;
            while record + 8 <= group.len() {
                let length =
                    u32_le(&group[record..record + 4]).ok_or("Invalid graph record")? as usize;
                let kind = u32_le(&group[record + 4..record + 8]).ok_or("Invalid graph record")?;
                let payload = record + 8;
                let end = payload.checked_add(length).ok_or("Graph record overflow")?;
                if end > group.len() {
                    return Err("Invalid graph record length".into());
                }
                if kind == 0x07d2 {
                    if length != 4 + 4 * refs.len()
                        || u32_le(&group[payload..payload + 4]) != Some(refs.len() as u32)
                    {
                        return Err("Invalid defining-distance references".into());
                    }
                    for (slot, reference) in refs.iter().enumerate() {
                        let value = u32::try_from(*reference).map_err(|_| "Graph ID overflow")?;
                        group[payload + 4 + slot * 4..payload + 8 + slot * 4]
                            .copy_from_slice(&value.to_le_bytes());
                    }
                    patched = true;
                    break;
                }
                record = end + length % 2;
            }
            if !patched {
                return Err("Missing defining-distance references".into());
            }
        }
        out.extend_from_slice(&group);
        offsets.push(out.len());
    }
    Ok((out, offsets))
}

pub fn set_coordinate_system_style(
    group: &mut [u8],
    form: GridForm,
    dotted: bool,
) -> Result<(), String> {
    let mut offset = 0;
    while offset + 8 <= group.len() {
        let length = u32_le(&group[offset..offset + 4]).ok_or("Invalid graph record")? as usize;
        let kind = u32_le(&group[offset + 4..offset + 8]).ok_or("Invalid graph record")?;
        let payload = offset + 8;
        let end = payload.checked_add(length).ok_or("Graph record overflow")?;
        if end > group.len() {
            return Err("Invalid graph record length".into());
        }
        if kind == 0x0904 {
            if length != 8
                || GridForm::from_code(group[payload]).is_none()
                || group[payload + 1] != 0
                || group[payload + 2] > 1
                || group[payload + 3..end] != [0; 5]
            {
                return Err("Unsupported coordinate-system grid record".into());
            }
            group[payload] = form.code();
            group[payload + 2] = u8::from(dotted);
            return Ok(());
        }
        offset = end + length % 2;
    }
    Err("Missing coordinate-system grid record".into())
}

pub fn set_vertical_unit_form(group: &mut [u8], form: GridForm, step: f64) -> Result<(), String> {
    let code: u16 = if form == GridForm::Rectangular {
        55
    } else {
        54
    };
    set_vertical_unit_code(group, code, step)
}

pub fn set_distance_vertical_unit_form(
    group: &mut [u8],
    independent: bool,
    step: f64,
) -> Result<(), String> {
    let code: u16 = if independent { 57 } else { 56 };
    set_vertical_unit_code(group, code, step)?;
    if group.len() < 36 || u32_le(&group[..4]) != Some(28) || u32_le(&group[4..8]) != Some(0x07d0) {
        return Err("Invalid defining-distance unit header".into());
    }
    if independent {
        group[10] &= !4;
    } else {
        group[10] |= 4;
    }
    Ok(())
}

fn set_vertical_unit_code(group: &mut [u8], code: u16, step: f64) -> Result<(), String> {
    if !step.is_finite() || step <= 0.0 {
        return Err("Invalid vertical graph unit".into());
    }
    let mut offset = 0;
    let mut header = false;
    let mut constraint = false;
    while offset + 8 <= group.len() {
        let length = u32_le(&group[offset..offset + 4]).ok_or("Invalid graph record")? as usize;
        let kind = u32_le(&group[offset + 4..offset + 8]).ok_or("Invalid graph record")?;
        let payload = offset + 8;
        let end = payload.checked_add(length).ok_or("Graph record overflow")?;
        if end > group.len() {
            return Err("Invalid graph record length".into());
        }
        if kind == 0x07d0 {
            if length != 28 {
                return Err("Invalid vertical unit header".into());
            }
            group[payload..payload + 2].copy_from_slice(&code.to_le_bytes());
            header = true;
        } else if kind == 0x07d3 {
            if length != 12 {
                return Err("Invalid vertical unit constraint".into());
            }
            group[payload..payload + 2].copy_from_slice(&code.to_le_bytes());
            group[payload + 4..payload + 12].copy_from_slice(&step.to_le_bytes());
            constraint = true;
        }
        offset = end + length % 2;
    }
    (header && constraint)
        .then_some(())
        .ok_or("Incomplete vertical unit group".into())
}

pub fn set_x_axis_trigonometric(group: &mut [u8], trigonometric: bool) -> Result<(), String> {
    let mut offset = 0;
    while offset + 8 <= group.len() {
        let length = u32_le(&group[offset..offset + 4]).ok_or("Invalid graph record")? as usize;
        let kind = u32_le(&group[offset + 4..offset + 8]).ok_or("Invalid graph record")?;
        let payload = offset + 8;
        let end = payload.checked_add(length).ok_or("Graph record overflow")?;
        if end > group.len() {
            return Err("Invalid graph record length".into());
        }
        if kind == 0x0905 {
            if length != 4 || !matches!(group[payload..end], [1, 0, 0, 0] | [1, 0, 1, 0]) {
                return Err("Unsupported X-axis format".into());
            }
            group[payload + 2] = u8::from(trigonometric);
            return Ok(());
        }
        offset = end + length % 2;
    }
    Err("Missing X-axis format record".into())
}

/// Original-app slope group. The line and frame IDs must refer to objects on
/// the same page. The caller should validate those dependencies before saving.
pub fn slope_measurement_group(line: usize, coordinates: usize) -> Result<Vec<u8>, String> {
    let line = u32::try_from(line).map_err(|_| "Line ID overflow")?;
    let coordinates = u32::try_from(coordinates).map_err(|_| "Coordinate-system ID overflow")?;
    if line == 0 || coordinates == 0 {
        return Err("Slope references must be nonzero".into());
    }
    let mut group = include_bytes!("../assets/graph_slope_measurement_group.bin").to_vec();
    group[48..52].copy_from_slice(&line.to_le_bytes());
    group[52..56].copy_from_slice(&coordinates.to_le_bytes());
    Ok(group)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object_groups(bytes: &[u8]) -> Vec<(u16, Vec<usize>, Vec<(u32, Vec<u8>)>)> {
        assert_eq!(&bytes[..4], b"GSP4");
        let mut offset = 4;
        let mut groups = Vec::new();
        while offset < bytes.len() {
            let len = u32_le(&bytes[offset..offset + 4]).unwrap() as usize;
            let kind = u32_le(&bytes[offset + 4..offset + 8]).unwrap();
            let payload = &bytes[offset + 8..offset + 8 + len];
            if kind == 0x07d0 {
                groups.push((
                    u16::from_le_bytes(payload[..2].try_into().unwrap()),
                    vec![],
                    vec![],
                ));
            } else if let Some((_, refs, records)) = groups.last_mut() {
                if kind == 0x07d2 {
                    let count = u32_le(&payload[..4]).unwrap() as usize;
                    *refs = (0..count)
                        .map(|i| u32_le(&payload[4 + 4 * i..8 + 4 * i]).unwrap() as usize)
                        .collect();
                }
                if kind != 0x07d7 {
                    records.push((kind, payload.to_vec()));
                }
            }
            offset += 8 + len + len % 2;
        }
        groups
    }

    #[test]
    fn decodes_original_slope_and_its_coordinate_system() {
        let bytes = include_bytes!("../tests/fixtures/wine_segment_slope.gsp");
        let groups = object_groups(bytes);
        assert_eq!(groups.len(), 10);
        let decoded: Vec<_> = groups
            .iter()
            .map(|(code, refs, records)| {
                let borrowed: Vec<_> = records
                    .iter()
                    .map(|(kind, data)| (*kind, data.as_slice()))
                    .collect();
                decode_graph_object(*code, refs, &borrowed)
            })
            .collect();
        assert_eq!(
            decoded[4],
            Some(GraphObject::HorizontalUnit {
                origin: 4,
                step: 37.79527559055118
            })
        );
        assert_eq!(
            decoded[5],
            Some(GraphObject::Axis {
                start: 4,
                end: 5,
                axis: 1
            })
        );
        assert_eq!(
            decoded[6],
            Some(GraphObject::VerticalUnit {
                horizontal_unit: 5,
                step: 37.79527559055118
            })
        );
        assert_eq!(
            decoded[7],
            Some(GraphObject::Axis {
                start: 4,
                end: 7,
                axis: 0
            })
        );
        assert_eq!(
            decoded[8],
            Some(GraphObject::CoordinateSystem {
                x_axis: 6,
                y_axis: 8,
                form: GridForm::Square,
                dotted: false,
            })
        );
        assert_eq!(
            decoded[9],
            Some(GraphObject::Slope {
                line: 3,
                coordinates: 9
            })
        );
    }

    #[test]
    fn slope_uses_axis_y_direction_and_changes_with_drag() {
        let frame = CoordinateFrame {
            origin: GraphPoint { x: 199.0, y: 125.0 },
            x_unit: GraphPoint {
                x: 236.79527559055118,
                y: 125.0,
            },
            y_unit: GraphPoint {
                x: 199.0,
                y: 87.20472440944882,
            },
        };
        let a = GraphPoint { x: 150.0, y: 120.0 };
        assert_eq!(frame.slope(a, GraphPoint { x: 250.0, y: 120.0 }), Some(0.0));
        let dragged = GraphPoint { x: 300.0, y: 170.0 };
        assert!((frame.slope(a, dragged).unwrap() + 1.0 / 3.0).abs() < 1e-12);
        let moved_a = GraphPoint { x: 150.0, y: 170.0 };
        assert!(
            (frame
                .slope(moved_a, GraphPoint { x: 250.0, y: 120.0 })
                .unwrap()
                - 0.5)
                .abs()
                < 1e-12
        );
    }

    #[test]
    fn slope_rejects_degenerate_frame_and_vertical_line() {
        let point = GraphPoint { x: 0.0, y: 0.0 };
        let frame = CoordinateFrame {
            origin: point,
            x_unit: GraphPoint { x: 1.0, y: 0.0 },
            y_unit: GraphPoint { x: 0.0, y: -1.0 },
        };
        assert_eq!(frame.slope(point, GraphPoint { x: 0.0, y: 2.0 }), None);
        assert_eq!(frame.slope(point, point), None);
        let bad = CoordinateFrame {
            y_unit: GraphPoint { x: 2.0, y: 0.0 },
            ..frame
        };
        assert_eq!(bad.coordinates(point), None);
    }

    #[test]
    fn templates_remap_internal_references() {
        let original = include_bytes!("../assets/graph_coordinate_system_groups.bin");
        assert_eq!(
            coordinate_system_groups(4, GraphPoint { x: 199.0, y: 125.0 }).unwrap(),
            original
        );
        let shifted = coordinate_system_groups(10, GraphPoint { x: 300.0, y: 200.0 }).unwrap();
        assert_eq!(
            f64::from_le_bytes(shifted[44..52].try_into().unwrap()),
            300.0
        );
        assert_eq!(
            f64::from_le_bytes(shifted[52..60].try_into().unwrap()),
            200.0
        );
        let groups = object_groups(&[b"GSP4".as_slice(), shifted.as_slice()].concat());
        assert_eq!(groups.len(), 6);
        assert_eq!(groups[1].1, vec![10]);
        assert_eq!(groups[2].1, vec![10, 11]);
        assert_eq!(groups[3].1, vec![11]);
        assert_eq!(groups[4].1, vec![10, 13]);
        assert_eq!(groups[5].1, vec![12, 14]);
        assert!(coordinate_system_groups(0, GraphPoint { x: 0.0, y: 0.0 }).is_err());
    }

    #[test]
    fn slope_template_references_are_patchable() {
        let group = slope_measurement_group(42, 99).unwrap();
        assert_eq!(u32_le(&group[48..52]), Some(42));
        assert_eq!(u32_le(&group[52..56]), Some(99));
        let groups = object_groups(&[b"GSP4".as_slice(), group.as_slice()].concat());
        assert_eq!(groups[0].1, vec![42, 99]);
        assert!(slope_measurement_group(0, 99).is_err());
    }

    #[test]
    fn page_snap_flag_matches_original_toggle() {
        fn page_record(bytes: &[u8]) -> &[u8] {
            let mut offset = 4;
            loop {
                let length = u32_le(&bytes[offset..offset + 4]).unwrap() as usize;
                let kind = u32_le(&bytes[offset + 4..offset + 8]).unwrap();
                if kind == 0x03e8 {
                    return &bytes[offset + 8..offset + 8 + length];
                }
                offset += 8 + length + length % 2;
            }
        }
        let normal = include_bytes!("../tests/fixtures/graph_defined_coordinate_system.gsp");
        let snapped = include_bytes!("../tests/fixtures/graph_snap_points_original.gsp");
        assert_eq!(snap_points(page_record(normal)), Some(false));
        assert_eq!(snap_points(page_record(snapped)), Some(true));
    }

    #[test]
    fn decodes_generated_slope_after_original_edit() {
        let bytes = include_bytes!("../tests/fixtures/graph_generated_moved_slope_wine.gsp");
        let groups = object_groups(bytes);
        assert_eq!(groups.len(), 10);
        let (code, refs, records) = &groups[9];
        let borrowed: Vec<_> = records
            .iter()
            .map(|(kind, data)| (*kind, data.as_slice()))
            .collect();
        assert_eq!(
            decode_graph_object(*code, refs, &borrowed),
            Some(GraphObject::Slope {
                line: 3,
                coordinates: 9
            })
        );
    }
}
