use super::Editor;
use super::system_fonts::{add_cjk_fallbacks, system_font};
use std::sync::Arc;

pub(crate) fn system_font_data(family: &str) -> Option<egui::FontData> {
    system_font(family.strip_prefix('@').unwrap_or(family)).and_then(|font| font.data())
}

impl Editor {
    pub(super) fn sync_document_fonts(&mut self, ctx: &egui::Context) {
        if !self.font_definitions_applied {
            add_cjk_fallbacks(&mut self.font_definitions);
        }
        let mut families = self
            .doc
            .objects
            .iter()
            .filter(|object| !object.deleted)
            .filter_map(|object| self.doc.text_font_state(object.id).map(|font| font.family))
            .collect::<Vec<_>>();
        families.extend(self.doc.text_font_families().unwrap_or_default());
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
