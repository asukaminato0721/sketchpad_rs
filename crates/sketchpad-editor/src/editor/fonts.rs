use super::Editor;
use crate::render::system_fonts::{add_cjk_fallbacks, system_font};
use std::sync::Arc;

pub(crate) fn system_font_data(family: &str) -> Option<egui::FontData> {
    system_font(family.strip_prefix('@').unwrap_or(family)).and_then(|font| font.data())
}

impl Editor {
    pub(super) fn sync_document_fonts(&mut self, ctx: &egui::Context) {
        self.font_context = Some(ctx.clone());
        if !self.font_definitions_applied {
            add_cjk_fallbacks(&mut self.font_definitions);
        }
        let mut families = self
            .doc
            .objects
            .iter()
            .filter(|object| !object.deleted)
            // Source font families are collected together below. Resolving each
            // object's source style would reparse the entire file per object.
            .filter_map(|object| {
                object
                    .text_style_override
                    .as_ref()
                    .map(|font| font.family.clone())
            })
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

    pub(super) fn math_text_layout(&self, id: usize) -> Option<crate::render::math_text::Layout> {
        self.doc.text_math_nodes(id)?;
        let fallback;
        let ctx = if let Some(ctx) = &self.font_context {
            ctx
        } else {
            fallback = crate::render::math_text::font_context(&self.doc);
            &fallback
        };
        crate::render::math_text::measured_layout(&self.doc, id, ctx)
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

#[cfg(test)]
mod tests {
    use super::*;
    use sketchpad_engine::document::Document;

    #[test]
    fn font_sync_includes_source_families_and_unsaved_style_edits() {
        let doc = Document::from_bytes(
            include_bytes!("../../../../tests/fixtures/text_mixed_adventor_he_original.gsp"),
            None,
        )
        .unwrap();
        let mut editor = Editor::new(doc);
        let ctx = egui::Context::default();
        editor.sync_document_fonts(&ctx);
        // Font discovery must work even when a requested font is not installed.
        for family in editor.doc.text_font_families().unwrap() {
            assert!(editor.loaded_fonts.contains_key(&family));
        }

        editor.doc.selected.insert(1);
        editor
            .doc
            .set_selected_text_family("Test Caption Font")
            .unwrap();
        editor.sync_document_fonts(&ctx);
        assert!(editor.loaded_fonts.contains_key("Test Caption Font"));

        editor
            .doc
            .format_text_range_family(1, 0, 2, "Test Run Font")
            .unwrap();
        editor.sync_document_fonts(&ctx);
        assert!(editor.loaded_fonts.contains_key("Test Run Font"));

        ctx.begin_pass(Default::default());
        ctx.fonts_mut(|fonts| {
            let font = editor.canvas_font_id(Some("Test Run Font"), 16.0);
            assert!(fonts.has_glyphs(&font, "中文界面与文档"));
        });
        ctx.end_pass().textures_delta.clear();
    }
}
