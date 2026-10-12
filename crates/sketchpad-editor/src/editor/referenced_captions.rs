use super::Editor;
use crate::render::Viewport;
use sketchpad_engine::document::Kind;

impl Editor {
    pub(super) fn referenced_action_hit(&self, v: &Viewport, p: egui::Pos2) -> Option<usize> {
        for object in self
            .doc
            .objects()
            .iter()
            .rev()
            .filter(|object| self.doc.is_displayed(object.id))
        {
            let Kind::ReferencedText { parents, .. } = &object.kind else {
                continue;
            };
            let (_, rect) = self.doc.text_content(object.id)?;
            let rows = self.doc.referenced_text_runs(object.id)?;
            let origin = v.screen(sketchpad_engine::document::Point {
                x: f64::from(rect.left),
                y: f64::from(rect.top),
            });
            if let Some(layout) = self.math_text_layout(object.id) {
                for run in layout.runs {
                    if let Some(target) = run
                        .reference
                        .and_then(|slot| parents.get(slot - 1))
                        .copied()
                        && self
                            .doc
                            .objects()
                            .get(target - 1)
                            .is_some_and(|o| !o.deleted && o.kind.is_action_button())
                        && egui::Rect::from_min_size(
                            origin + run.position.to_vec2(),
                            run.galley.size(),
                        )
                        .contains(p)
                    {
                        return Some(target);
                    }
                }
                continue;
            }
            let size = self
                .doc
                .text_font_state(object.id)
                .map_or(16., |style| f32::from(style.size_code));
            let mut y = origin.y;
            for row in rows {
                let height = row
                    .iter()
                    .map(|run| run.style.font.size_code.map_or(size, f32::from))
                    .fold(size, f32::max);
                let mut x = origin.x;
                for run in row {
                    let (run_size, shift) = run
                        .style
                        .font
                        .script_metrics(run.style.font.size_code.map_or(size, f32::from));
                    let width = run.text.chars().count() as f32 * run_size * 0.65;
                    if let Some(target) = run
                        .reference
                        .and_then(|slot| parents.get(slot - 1))
                        .copied()
                        && self
                            .doc
                            .objects()
                            .get(target - 1)
                            .is_some_and(|object| !object.deleted && object.kind.is_action_button())
                        && egui::Rect::from_min_size(
                            egui::pos2(x, y - shift),
                            egui::vec2(width, height * 1.1),
                        )
                        .contains(p)
                    {
                        return Some(target);
                    }
                    x += width;
                }
                y += (height * 1.1).max(19.);
            }
        }
        None
    }
}

impl Editor {
    pub(super) fn add_selected_referenced_caption(&mut self) {
        let parents = self.selection_order.clone();
        if parents.is_empty() {
            self.status = "Select the live sources for a caption".into();
            return;
        }
        let mut markup = String::from("<H");
        for slot in 1..=parents.len() {
            if slot > 1 {
                markup.push_str("<T1x >");
            }
            markup.push_str(&format!("<?x{slot:X}>"));
        }
        markup.push('>');
        match self.doc.add_referenced_text(parents, &markup, (20, 80)) {
            Ok(id) => {
                self.select_only(id);
                self.status = "Created a caption linked to the selected sources".into();
            }
            Err(error) => self.status = error,
        }
    }
}
