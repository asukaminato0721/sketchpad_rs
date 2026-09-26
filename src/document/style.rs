use super::Document;
use crate::style_records::{
    LinePattern, LineWidth, PointStyle, RgbColor, SegmentLineStyle, VisualStyle,
};

impl Document {
    fn update_selected_visual_style(
        &mut self,
        update: impl Fn(VisualStyle) -> Option<VisualStyle>,
    ) -> Result<usize, String> {
        if self.partial_view && self.page_count == 1 {
            return Err("Cannot edit styles in a sketch with unsupported records safely".into());
        }
        let changes: Vec<_> = self
            .objects
            .iter()
            .filter_map(|object| {
                if object.deleted
                    || object.page != self.active_page
                    || !self.selected.contains(&object.id)
                {
                    return None;
                }
                let old = object.visual_style?;
                let new = update(old)?;
                (new != old).then_some((object.id, new))
            })
            .collect();
        if !changes.is_empty() {
            self.checkpoint();
            for (id, style) in &changes {
                self.objects[*id - 1].visual_style = Some(*style);
            }
        }
        Ok(changes.len())
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
}
