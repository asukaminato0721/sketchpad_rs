use super::Document;
use crate::style_records::{
    LinePattern, LineWidth, PointStyle, RgbColor, SegmentLineStyle, VisualStyle,
};

impl Document {
    fn update_visual_style(
        &mut self,
        ids: &std::collections::BTreeSet<usize>,
        update: impl Fn(VisualStyle) -> Option<VisualStyle>,
    ) -> Result<usize, String> {
        if self.partial_view && self.page_count == 1 {
            return Err("Cannot edit styles in a sketch with unsupported records safely".into());
        }
        let changes: Vec<_> = self
            .objects
            .iter()
            .filter_map(|object| {
                if object.deleted || object.page != self.active_page || !ids.contains(&object.id) {
                    return None;
                }
                let old = object.visual_style?;
                let new = update(old)?;
                (new != old).then_some((object.id, new))
            })
            .collect();
        if changes.is_empty() {
            return Ok(0);
        }
        let rs = super::records(&self.original)?;
        let upgrade = changes.iter().any(|(id, style)| {
            let Some(offset) = self.objects[*id - 1].header_offset else {
                return false;
            };
            let Some(record) = rs.iter().find(|record| record.data.start == offset) else {
                return false;
            };
            if record.kind != 0x07d0 || record.data.len() != 16 {
                return false;
            }
            let mut header = self.original[record.data.clone()].to_vec();
            crate::style_records::set_visual_style(&mut header, *style).is_err()
        });
        let mut candidate = self.clone();
        let mut ids: Vec<_> = (0..=self.objects.len()).collect();
        if upgrade {
            if !candidate.needs_legacy_upgrade()? {
                return Err("This short object header cannot store the selected style".into());
            }
            // Upgrading also compacts any previously deleted groups. Keep selected
            // objects and pending style changes attached to their surviving IDs.
            for (index, object) in self.objects.iter().filter(|o| !o.deleted).enumerate() {
                ids[object.id] = index + 1;
            }
            candidate.checkpoint_structure();
            candidate.upgrade_legacy_document()?;
            candidate.selected = self
                .selected
                .iter()
                .filter_map(|id| {
                    self.objects
                        .get(id.checked_sub(1)?)
                        .filter(|o| !o.deleted)
                        .map(|_| ids[*id])
                })
                .collect();
        } else {
            candidate.checkpoint();
        }
        for (id, style) in &changes {
            candidate.objects[ids[*id] - 1].visual_style = Some(*style);
        }
        candidate.verify_roundtrip()?;
        *self = candidate;
        Ok(changes.len())
    }
    fn update_selected_visual_style(
        &mut self,
        update: impl Fn(VisualStyle) -> Option<VisualStyle>,
    ) -> Result<usize, String> {
        let selected = self.selected.clone();
        self.update_visual_style(&selected, update)
    }
    pub fn set_selected_point_style(&mut self, shape: PointStyle) -> Result<usize, String> {
        self.update_selected_visual_style(|style| match style {
            VisualStyle::Point { color, .. } => Some(VisualStyle::Point { shape, color }),
            _ => None,
        })
    }
    pub fn set_selected_line_width(&mut self, width: LineWidth) -> Result<usize, String> {
        self.update_selected_visual_style(|style| match style {
            VisualStyle::Stroke { line, color } => Some(VisualStyle::Stroke {
                line: SegmentLineStyle { width, ..line },
                color,
            }),
            _ => None,
        })
    }
    pub fn set_selected_line_pattern(&mut self, pattern: LinePattern) -> Result<usize, String> {
        self.update_selected_visual_style(|style| match style {
            VisualStyle::Stroke { line, color } => Some(VisualStyle::Stroke {
                line: SegmentLineStyle { pattern, ..line },
                color,
            }),
            _ => None,
        })
    }
    pub fn set_selected_color(&mut self, color: RgbColor) -> Result<usize, String> {
        self.update_selected_visual_style(|style| {
            Some(match style {
                VisualStyle::Point { shape, .. } => VisualStyle::Point { shape, color },
                VisualStyle::Stroke { line, .. } => VisualStyle::Stroke { line, color },
                VisualStyle::Polygon { opacity, .. } => VisualStyle::Polygon { color, opacity },
            })
        })
    }
    pub fn set_object_polygon_opacity(&mut self, id: usize, opacity: u16) -> Result<bool, String> {
        let object = self
            .objects
            .get(id.checked_sub(1).ok_or("Invalid object ID")?)
            .ok_or("Invalid object ID")?;
        if object.deleted || object.page != self.active_page {
            return Err("Object is not on the active page".into());
        }
        if !matches!(object.visual_style, Some(VisualStyle::Polygon { .. })) {
            return Err("Object does not have polygon opacity".into());
        }
        let changed = self.update_visual_style(
            &std::collections::BTreeSet::from([id]),
            |style| match style {
                VisualStyle::Polygon { color, .. } => Some(VisualStyle::Polygon { color, opacity }),
                _ => None,
            },
        )?;
        Ok(changed != 0)
    }
}
