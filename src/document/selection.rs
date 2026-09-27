use super::kind::{is_arc_kind, is_point_kind};
use super::{Clipboard, ClipboardObject, Document, Kind, Object};
use crate::function_records;
use crate::graph::GridForm;
use std::collections::BTreeSet;

impl Document {
    // Native auxiliary objects have defined geometry while flag 2 suppresses
    // presentation. Ordinary hide/show leaves this independent flag intact.
    // Clipboard copies convert suppressed prerequisites to ordinary hidden.
    fn is_presentation_suppressed(&self, id: usize) -> bool {
        id.checked_sub(1).and_then(|index| self.objects.get(index)).is_some_and(|object| {
            object.presentation_suppressed
        })
    }

    pub fn is_displayed(&self, id: usize) -> bool {
        let Some(object) = id.checked_sub(1).and_then(|at| self.objects.get(at)) else {
            return false;
        };
        if !object.visible
            || object.deleted
            || object.page != self.active_page
            || self.is_presentation_suppressed(id)
        {
            return false;
        }
        if matches!(object.kind, Kind::DistanceVerticalUnit(_, _, false)) {
            return false;
        }
        if matches!(object.kind, Kind::GraphVerticalUnit(_, _)) {
            let form = self
                .coordinate_system_for_object(id)
                .and_then(|frame| self.frame_styles.get(&frame))
                .map_or(self.grid_form[object.page], |style| style.0);
            if form != GridForm::Rectangular {
                return false;
            }
        }
        !self.objects.iter().any(|frame| {
            let Kind::CoordinateSystem(x_axis, y_axis) = frame.kind else {
                return false;
            };
            if frame.visible || frame.deleted || frame.page != object.page {
                return false;
            }
            if let (Some(Kind::CircleAxis(circle, 1)), Some(Kind::CircleAxis(other, 0))) = (
                x_axis
                    .checked_sub(1)
                    .and_then(|at| self.objects.get(at))
                    .map(|o| &o.kind),
                y_axis
                    .checked_sub(1)
                    .and_then(|at| self.objects.get(at))
                    .map(|o| &o.kind),
            ) && circle == other
            {
                return [frame.id, x_axis, y_axis].contains(&id);
            }
            if let Some(Kind::DistanceAxis(origin, _, 1)) = x_axis
                .checked_sub(1)
                .and_then(|at| self.objects.get(at))
                .map(|o| &o.kind)
            {
                let unit = match y_axis
                    .checked_sub(1)
                    .and_then(|at| self.objects.get(at))
                    .map(|o| &o.kind)
                {
                    Some(Kind::GraphAxis(_, unit, 0)) => Some(*unit),
                    _ => None,
                };
                return [
                    Some(frame.id),
                    Some(x_axis),
                    Some(y_axis),
                    Some(*origin),
                    unit,
                ]
                .contains(&Some(id));
            }
            let Some(Kind::GraphAxis(origin, x_unit, 1 | 0x1_0001)) = x_axis
                .checked_sub(1)
                .and_then(|at| self.objects.get(at))
                .map(|o| &o.kind)
            else {
                return false;
            };
            let Some(Kind::GraphAxis(_, y_unit, 0)) = y_axis
                .checked_sub(1)
                .and_then(|at| self.objects.get(at))
                .map(|o| &o.kind)
            else {
                return false;
            };
            [frame.id, x_axis, y_axis, *origin, *x_unit, *y_unit].contains(&id)
        })
    }
    pub fn dependencies_of(&self, id: usize) -> Option<Vec<usize>> {
        let object = self.objects.get(id.checked_sub(1)?)?;
        let mut dependencies = object.kind.dependencies();
        if let Some(host) = object.midpoint_host {
            dependencies.push(host);
        }
        Some(dependencies)
    }
    fn copy_has_selected_ancestor(&self, root: usize) -> Result<bool, String> {
        let mut stack = self.dependencies_of(root).ok_or("Invalid selection")?
            .into_iter().map(|parent| (root, parent)).collect::<Vec<_>>();
        let mut visited = BTreeSet::new();
        while let Some((child, parent)) = stack.pop() {
            if parent == 0 || parent >= child {
                return Err("Clipboard has an invalid dependency".into());
            }
            if !visited.insert(parent) {
                continue;
            }
            let object = self.objects.get(parent - 1).ok_or("Invalid selection")?;
            if object.deleted || object.page != self.active_page
                || matches!(object.kind, Kind::Unsupported(_)) {
                return Err(format!("Object {parent} cannot be copied safely"));
            }
            if self.selected.contains(&parent) {
                return Ok(true);
            }
            stack.extend(self.dependencies_of(parent).ok_or("Invalid selection")?
                .into_iter().map(|ancestor| (parent, ancestor)));
        }
        Ok(false)
    }

    pub fn copy_selected(&self) -> Result<Clipboard, String> {
        let mut included = BTreeSet::new();
        let mut pending: Vec<usize> = self.selected.iter().copied().collect();
        if pending.is_empty() {
            return Err("Select objects to copy".into());
        }
        // Ordinary point constructions are retained when the copied selection
        // includes an ancestor; isolated selected points become free snapshots.
        let mut frozen_points = std::collections::BTreeMap::new();
        for &id in &self.selected {
            let object = self.objects.get(id.checked_sub(1).ok_or("Invalid selection")?)
                .ok_or("Invalid selection")?;
            if object.deleted || object.page != self.active_page {
                return Err(format!("Object {id} cannot be copied safely"));
            }
            if matches!(object.kind, Kind::NumericCoordinatePoint(..)
                | Kind::Midpoint(..) | Kind::SegmentMidpoint(_))
                && !self.copy_has_selected_ancestor(id)? {
                frozen_points.insert(id, self.position(id)
                    .ok_or("Selected point is undefined")?);
            }
        }
        for object in self
            .objects
            .iter()
            .filter(|object| !object.deleted && object.page == self.active_page)
        {
            match object.kind {
                Kind::FunctionPlot(definition, _, _, _, _)
                    if self.selected.contains(&definition) =>
                {
                    pending.push(object.id)
                }
                Kind::ParametricPlot(x, y, _, _, _, _, _)
                    if self.selected.contains(&x) || self.selected.contains(&y) =>
                {
                    pending.push(object.id)
                }
                _ => {}
            }
        }
        while let Some(id) = pending.pop() {
            if !included.insert(id) {
                continue;
            }
            let object = self
                .objects
                .get(id.checked_sub(1).ok_or("Invalid selection")?)
                .ok_or("Invalid selection")?;
            if object.deleted
                || object.page != self.active_page
                || matches!(object.kind, Kind::Unsupported(_))
            {
                return Err(format!("Object {id} cannot be copied safely"));
            }
            if !frozen_points.contains_key(&id) {
                pending.extend(self.dependencies_of(id).ok_or("Invalid selection")?);
            }
            if matches!(
                object.kind,
                Kind::PointParameterIteration(..)
                    | Kind::BranchingPolygonIteration { .. }
                    | Kind::MappedPointIteration { .. }
                    | Kind::SegmentIteration(_, _, _, _)
                    | Kind::CircleIteration(..)
                    | Kind::RayIteration(_, _, _, _)
                    | Kind::PointIteration(_, _, _)
            ) {
                pending.extend(self.objects.iter().filter(|child| {
                    !child.deleted && child.page == self.active_page
                        && matches!(child.kind, Kind::IterationBinding(_, parent) | Kind::IterationValues(parent, _) if parent == id)
                }).map(|child| child.id));
            }
        }
        let mut hide_on_paste = BTreeSet::new();
        for &id in &included {
            if frozen_points.contains_key(&id) {
                continue;
            }
            let object = &self.objects[id - 1];
            let host = match object.kind {
                Kind::Midpoint(..) => object.midpoint_host,
                Kind::SegmentMidpoint(host) => Some(host),
                _ => None,
            };
            if let Some(host) = host {
                if !self.selected.contains(&host) && included.contains(&host) {
                    hide_on_paste.insert(host);
                }
            }
        }
        let rename_on_paste = self.objects.iter().find_map(|object| {
            let Kind::ParametricPlot(x, y, _, _, _, _, _) = object.kind else {
                return None;
            };
            if !included.contains(&object.id)
                || self.selected.contains(&x) == self.selected.contains(&y)
            {
                return None;
            }
            Some(if self.selected.contains(&x) { x } else { y })
        });
        for object in self
            .objects
            .iter()
            .filter(|object| included.contains(&object.id))
        {
            let frame = match object.kind {
                Kind::FunctionPlot(_, frame, _, _, _)
                | Kind::ParametricPlot(_, _, frame, _, _, _, _) => frame,
                _ => continue,
            };
            if self.selected.contains(&frame) {
                continue;
            }
            let mut stack = vec![frame];
            while let Some(id) = stack.pop() {
                let kind = &self.objects[id - 1].kind;
                if !matches!(
                    kind,
                    Kind::GraphHorizontalUnit(_, _)
                        | Kind::GraphVerticalUnit(_, _)
                        | Kind::GraphAxis(_, _, _)
                        | Kind::CircleAxis(_, _)
                        | Kind::DistanceAxis(_, _, _)
                        | Kind::DistanceVerticalUnit(_, _, _)
                        | Kind::CoordinateSystem(_, _)
                ) || !hide_on_paste.insert(id)
                {
                    continue;
                }
                stack.extend(kind.dependencies());
            }
        }
        Ok(Clipboard {
            hide_on_paste,
            rename_on_paste,
            objects: included
                .into_iter()
                .map(|id| -> Result<_, String> {
                    let object = &self.objects[id - 1];
                    let function_anchor = match object.kind {
                        Kind::FunctionDefinition(expression) => {
                            Some(if let Some(anchor) = object.function_anchor {
                                anchor
                            } else {
                                function_records::group_expression_anchor(
                                    &function_records::encode_definition_group(expression)?,
                                )?
                            })
                        }
                        Kind::DerivativeFunction(source, expression) => {
                            Some(if let Some(anchor) = object.function_anchor {
                                anchor
                            } else {
                                function_records::group_expression_anchor(
                                    &function_records::encode_derivative_group(source, expression)?,
                                )?
                            })
                        }
                        _ => None,
                    };
                    Ok(ClipboardObject {
                        old_id: id,
                        kind: frozen_points.get(&id)
                            .map_or_else(|| object.kind.clone(), |point| Kind::Free(*point)),
                        visible: object.visible && !self.is_presentation_suppressed(id),
                        tracing: object.tracing,
                        midpoint_host: if frozen_points.contains_key(&id) {
                            None
                        } else {
                            object.midpoint_host
                        },
                        visual_style: object.visual_style,
                        interior_order: object.interior_order,
                        text_style: self.text_font_state(id),
                        label: object.label.clone(),
                        rich_text: self.text_rich_template(id),
                        frame_style: self.frame_styles.get(&id).copied(),
                        function_anchor,
                        function_name: matches!(object.kind, Kind::FunctionDefinition(_))
                            .then(|| object.function_name.clone().unwrap_or_else(|| "f".into())),
                        measurement_precision: object.measurement_precision,
                        measurement_anchor: object.measurement_anchor,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
        })
    }
    pub fn paste(&mut self, clipboard: &Clipboard) -> Result<Vec<usize>, String> {
        if clipboard.is_empty() {
            return Err("Clipboard is empty".into());
        }
        if self.page_count == 1 && !self.safe_structure {
            return Err("Pasting objects in this document cannot be saved safely".into());
        }
        let mut candidate = self.clone();
        if candidate.needs_legacy_upgrade()? {
            candidate.checkpoint_structure();
            candidate.upgrade_legacy_document()?;
        } else {
            candidate.checkpoint();
        }
        candidate.selected.clear();
        let max_source = clipboard.objects.last().ok_or("Clipboard is empty")?.old_id;
        let mut ids = vec![0; max_source + 1];
        let mut created = Vec::with_capacity(clipboard.len());
        for ClipboardObject {
            old_id,
            kind,
            visible,
            tracing,
            midpoint_host,
            visual_style,
            interior_order,
            text_style,
            label,
            rich_text,
            frame_style,
            function_anchor,
            function_name,
            measurement_precision,
            measurement_anchor,
        } in &clipboard.objects
        {
            if kind
                .dependencies()
                .into_iter()
                .chain(*midpoint_host)
                .any(|id| id >= ids.len() || ids[id] == 0)
            {
                return Err("Clipboard has an unresolved dependency".into());
            }
            let mut remapped = kind.remap_checked(&ids)?;
            if let Kind::Free(ref mut point) = remapped {
                point.x += 20.;
                point.y += 20.;
            }
            if let Kind::PlainText { ref mut rect, .. } = remapped {
                rect.left = rect.left.checked_add(20).ok_or("Pasted text is off page")?;
                rect.top = rect.top.checked_add(20).ok_or("Pasted text is off page")?;
                rect.right = rect
                    .right
                    .checked_add(20)
                    .ok_or("Pasted text is off page")?;
                rect.bottom = rect
                    .bottom
                    .checked_add(20)
                    .ok_or("Pasted text is off page")?;
            }
            let shifted_anchor = |(x, y): (i16, i16)| -> Result<(i16, i16), String> {
                Ok((
                    x.checked_add(20).ok_or("Pasted number is off page")?,
                    y.checked_add(20).ok_or("Pasted number is off page")?,
                ))
            };
            match &mut remapped {
                Kind::Parameter(group) => group.set_anchor(shifted_anchor(group.anchor())?)?,
                Kind::Calculation(group) => group.set_anchor(shifted_anchor(group.anchor())?)?,
                Kind::MeasuredCalculation(_, group) => {
                    group.set_anchor(shifted_anchor(group.anchor())?)?
                }
                Kind::LengthPairCalculation(_, _, group) => {
                    group.set_anchor(shifted_anchor(group.anchor())?)?
                }
                Kind::ParameterCalculation(_, group) => {
                    group.set_anchor(shifted_anchor(group.anchor())?)?
                }
                Kind::ValueTable(_, group) | Kind::ValueTablePair(_, _, group) => {
                    group.set_anchor(shifted_anchor(group.anchor())?);
                }
                _ => {}
            }
            let function_anchor = (*function_anchor).map(shifted_anchor).transpose()?;
            let mut function_name = function_name.clone();
            if clipboard.rename_on_paste == Some(*old_id) {
                let used = candidate
                    .objects
                    .iter()
                    .filter(|object| matches!(object.kind, Kind::FunctionDefinition(_)))
                    .map(|object| object.function_name.as_deref().unwrap_or("f"))
                    .collect::<BTreeSet<_>>();
                let next = (b'g'..=b'z')
                    .map(|letter| (letter as char).to_string())
                    .find(|name| !used.contains(name.as_str()))
                    .ok_or("No available function name")?;
                function_name = Some(next);
            }
            let id = candidate.objects.len() + 1;
            let interior_order = interior_order.or_else(|| remapped.is_polygon().then(||
                if matches!(remapped, Kind::Polygon(_)) { 0 } else { 1 }));
            candidate.objects.push(Object {
                id,
                page: candidate.active_page,
                kind: remapped,
                visible: *visible && !clipboard.hide_on_paste.contains(old_id),
                presentation_suppressed: false,
                tracing: *tracing,
                deleted: false,
                midpoint_host: midpoint_host.map(|host| ids[host]),
                point_offset: None,
                constraint_offset: None,
                header_offset: None,
                label: label.clone(),
                visual_style: *visual_style,
                interior_order,
                text_style_override: text_style.clone(),
                rich_text_override: rich_text.clone(),
                function_anchor,
                function_name,
                measurement_precision: *measurement_precision,
                measurement_anchor: measurement_anchor
                    .map(|(x, y)| {
                        Ok::<_, String>((
                            x.checked_add(20).ok_or("Measurement x is out of range")?,
                            y.checked_add(20).ok_or("Measurement y is out of range")?,
                        ))
                    })
                    .transpose()?,
            });
            if let Some(style) = frame_style {
                candidate.frame_styles.insert(id, *style);
            }
            ids[*old_id] = id;
            created.push(id);
        }
        candidate.selected.extend(created.iter().copied());
        let encoded = candidate.encode()?;
        let reparsed = Self::from_bytes(&encoded, None)?;
        if candidate.has_pending_multi_page_additions() {
            let mut survivors: Vec<_> = candidate
                .objects
                .iter()
                .filter(|object| !object.deleted)
                .collect();
            survivors
                .sort_by_key(|object| (object.page, object.id > candidate.source_count, object.id));
            candidate.validate_reparsed_in_order(&reparsed, survivors, true)?;
        } else {
            candidate.validate_reparsed(&reparsed)?;
        }
        *self = candidate;
        Ok(created)
    }
    pub fn clear_selected(&mut self) -> Result<Vec<usize>, String> {
        let mut removed: BTreeSet<usize> = self
            .objects
            .iter()
            .filter(|o| !o.deleted && self.selected.contains(&o.id))
            .map(|o| o.id)
            .collect();
        if removed.is_empty() {
            return Ok(vec![]);
        }
        if self.page_count == 1 && !self.safe_structure {
            return Err("Deleting objects in this document cannot be saved safely".into());
        }
        if removed
            .iter()
            .any(|id| matches!(self.objects[id - 1].kind, Kind::Image { .. }))
        {
            return Err("Removing a picture resource is not yet supported".into());
        }
        loop {
            let before = removed.len();
            for object in self.objects.iter().filter(|o| !o.deleted) {
                if self
                    .dependencies_of(object.id)
                    .unwrap_or_default()
                    .iter()
                    .any(|id| removed.contains(id))
                {
                    removed.insert(object.id);
                }
            }
            if removed.len() == before {
                break;
            }
        }
        let mut candidate = self.clone();
        candidate.checkpoint();
        for object in &mut candidate.objects {
            if removed.contains(&object.id) {
                object.deleted = true;
            }
        }
        if !candidate.objects.iter().any(|object| {
            object.page == candidate.active_page
                && !object.deleted
                && matches!(object.kind, Kind::CoordinateSystem(_, _))
        }) {
            candidate.grid_dotted[candidate.active_page] = false;
            candidate.snap_points[candidate.active_page] = false;
            candidate.grid_form[candidate.active_page] = GridForm::Square;
        }
        candidate.selected.clear();
        let encoded = candidate.encode()?;
        let reparsed = Self::from_bytes(&encoded, None)?;
        if candidate.page_count > 1 {
            let mut survivors: Vec<_> = candidate
                .objects
                .iter()
                .filter(|object| !object.deleted)
                .collect();
            survivors
                .sort_by_key(|object| (object.page, object.id > candidate.source_count, object.id));
            candidate.validate_reparsed_in_order(&reparsed, survivors, true)?;
        } else {
            candidate.validate_reparsed(&reparsed)?;
        }
        *self = candidate;
        Ok(removed.into_iter().collect())
    }
    pub fn set_selected_visibility(&mut self, visible: bool) {
        let ids: Vec<_> = self.objects.iter()
            .filter(|object| {
                !object.deleted && self.selected.contains(&object.id)
                    && object.visible != visible && !self.is_presentation_suppressed(object.id)
            })
            .map(|object| object.id)
            .collect();
        if ids.is_empty() {
            return;
        }
        self.checkpoint();
        for id in ids {
            self.objects[id - 1].visible = visible;
        }
        if !visible {
            self.selected.clear();
        }
    }
    pub fn set_selected_tracing(&mut self, tracing: bool) -> Result<usize, String> {
        if self.partial_view && self.page_count == 1 {
            return Err("Cannot edit tracing in this sketch safely".into());
        }
        let ids: Vec<usize> = self
            .selected
            .iter()
            .copied()
            .filter(|&id| {
                self.objects.get(id - 1).is_some_and(|o| {
                    !o.deleted
                        && o.page == self.active_page
                        && (is_point_kind(&o.kind)
                            || self.line_points(id).is_some()
                            || o.kind.is_circle()
                            || is_arc_kind(&o.kind)
                            || o.kind.is_polygon())
                        && o.tracing != tracing
                })
            })
            .collect();
        if !ids.is_empty() {
            self.checkpoint();
            for id in &ids {
                self.objects[*id - 1].tracing = tracing;
            }
        }
        Ok(ids.len())
    }
    pub fn show_all_hidden(&mut self) {
        let ids: Vec<_> = self.objects.iter()
            .filter(|object| {
                !object.deleted && object.page == self.active_page
                    && !object.visible && !self.is_presentation_suppressed(object.id)
            })
            .map(|object| object.id)
            .collect();
        if ids.is_empty() {
            return;
        }
        self.checkpoint();
        for id in ids {
            self.objects[id - 1].visible = true;
        }
    }
}
