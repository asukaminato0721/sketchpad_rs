use super::Editor;
use std::process::Command;
use std::sync::Arc;

fn system_font_data(family: &str) -> Option<egui::FontData> {
    let lookup = family.strip_prefix('@').unwrap_or(family);
    let output = Command::new("fc-match")
        .arg("-f")
        .arg("%{file}\n%{index}\n")
        .arg(lookup)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let result = std::str::from_utf8(&output.stdout).ok()?;
    let mut lines = result.lines();
    let path = lines.next()?;
    let index = lines.next()?.parse::<u32>().ok()? & 0xffff;
    if path.is_empty() {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let mut data = egui::FontData::from_owned(bytes);
    data.index = index;
    Some(data)
}

impl Editor {
    pub(super) fn sync_document_fonts(&mut self, ctx: &egui::Context) {
        let families = self
            .doc
            .objects
            .iter()
            .filter(|object| !object.deleted)
            .filter_map(|object| self.doc.text_font_state(object.id).map(|font| font.family))
            .collect::<Vec<_>>();
        let mut changed = !self.font_definitions_applied;
        for family in families {
            if self.loaded_fonts.contains_key(&family) {
                continue;
            }
            let Some(data) = system_font_data(&family) else {
                self.loaded_fonts.insert(family, false);
                continue;
            };
            let key = format!("system:{family}");
            self.font_definitions
                .font_data
                .insert(key.clone(), Arc::new(data));
            let mut chain = vec![key];
            if let Some(fallbacks) = self
                .font_definitions
                .families
                .get(&egui::FontFamily::Proportional)
            {
                chain.extend(fallbacks.iter().cloned());
            }
            self.font_definitions
                .families
                .insert(egui::FontFamily::Name(family.clone().into()), chain);
            self.loaded_fonts.insert(family, true);
            changed = true;
        }
        if changed {
            ctx.set_fonts(self.font_definitions.clone());
            self.font_definitions_applied = true;
        }
    }

    pub(super) fn canvas_font_id(&self, family: Option<&str>, size: f32) -> egui::FontId {
        let font_family = family
            .filter(|family| self.loaded_fonts.get(*family) == Some(&true))
            .map_or(egui::FontFamily::Proportional, |family| {
                egui::FontFamily::Name(family.into())
            });
        egui::FontId::new(size, font_family)
    }
}
