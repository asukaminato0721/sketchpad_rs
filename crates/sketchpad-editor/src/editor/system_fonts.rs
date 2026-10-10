use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

// The first match respects the user's locale for shared Han glyphs. The language
// matches also cover systems with separate Chinese, Japanese, and Korean fonts.
const CJK_PATTERNS: &[&str] = &[
    "Noto Sans CJK SC",
    "Noto Sans CJK TC",
    "Noto Sans CJK JP",
    "Noto Sans CJK KR",
    // Require representative locale-specific glyphs as well as the common
    // Han character below. A Japanese fallback containing 中, for example,
    // need not contain simplified Chinese 变.
    "sans-serif:charset=53d8",
    "sans-serif:charset=8b8a",
    "sans-serif:charset=5909",
    "sans-serif:charset=ac00",
    "sans-serif:charset=4e2d",
    "sans-serif:lang=zh-cn",
    "sans-serif:lang=zh-tw",
    "sans-serif:lang=ja",
    "sans-serif:lang=ko",
];

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SystemFont {
    path: PathBuf,
    index: u32,
}

impl SystemFont {
    fn from_match(result: &str) -> Option<Self> {
        let mut lines = result.lines();
        let path = lines.next()?;
        // Fontconfig stores a variable-font instance in the upper bits; egui
        // expects just the face index within a TTC/OTC collection.
        let index = lines.next()?.parse::<u32>().ok()? & 0xffff;
        if path.is_empty() {
            return None;
        }
        Some(Self {
            path: path.into(),
            index,
        })
    }

    pub(crate) fn data(&self) -> Option<egui::FontData> {
        let bytes = std::fs::read(&self.path).ok()?;
        let mut data = egui::FontData::from_owned(bytes);
        data.index = self.index;
        Some(data)
    }
}

pub(crate) fn system_font(lookup: &str) -> Option<SystemFont> {
    let output = Command::new("fc-match")
        .arg("-f")
        .arg("%{file}\n%{index}\n")
        .arg("--")
        .arg(lookup)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    SystemFont::from_match(std::str::from_utf8(&output.stdout).ok()?)
}

fn append_fallback(fonts: &mut egui::FontDefinitions, key: String, data: egui::FontData) {
    fonts.font_data.insert(key.clone(), Arc::new(data));
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts.families.entry(family).or_default().push(key.clone());
    }
}

fn add_bundled_cjk_fallback(fonts: &mut egui::FontDefinitions) {
    // Keep this in the executable. Windows installations need neither
    // fontconfig nor an installed Chinese language pack to display the UI.
    append_fallback(
        fonts,
        "bundled:NotoSansCJKsc".into(),
        egui::FontData::from_static(include_bytes!(
            "../../../../assets/fonts/NotoSansCJKsc-Regular.otf"
        )),
    );
}

pub(crate) fn add_cjk_fallbacks(fonts: &mut egui::FontDefinitions) {
    let mut seen = HashSet::new();
    for pattern in CJK_PATTERNS {
        let Some(font) = system_font(pattern) else {
            continue;
        };
        if !seen.insert(font.clone()) {
            continue;
        }
        let Some(data) = font.data() else { continue };
        append_fallback(fonts, format!("cjk:{pattern}"), data);
    }
    add_bundled_cjk_fallback(fonts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fontconfig_collection_index_and_invalid_matches() {
        let font = SystemFont::from_match("/fonts/Noto Sans CJK.ttc\n65538\n").unwrap();
        assert_eq!(font.path, PathBuf::from("/fonts/Noto Sans CJK.ttc"));
        assert_eq!(font.index, 2);
        for invalid in ["", "\n0\n", "/font.ttf\n", "/font.ttf\ninvalid\n"] {
            assert!(SystemFont::from_match(invalid).is_none());
        }
    }

    #[test]
    fn fallback_preserves_latin_fonts_in_both_default_families() {
        let mut fonts = egui::FontDefinitions::default();
        let original = fonts.families.clone();
        let data = fonts.font_data.values().next().unwrap().as_ref().clone();
        append_fallback(&mut fonts, "test-cjk".into(), data);
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            let chain = &fonts.families[&family];
            assert_eq!(&chain[..chain.len() - 1], original[&family]);
            assert_eq!(chain.last().unwrap(), "test-cjk");
        }
    }

    #[test]
    fn bundled_font_covers_chinese_ui_without_system_fonts() {
        let mut definitions = egui::FontDefinitions::default();
        add_bundled_cjk_fallback(&mut definitions);
        // Document-specific fonts inherit the same fallback chain.
        let document_family = egui::FontFamily::Name("Document font".into());
        definitions.families.insert(
            document_family.clone(),
            definitions.families[&egui::FontFamily::Proportional].clone(),
        );
        let ctx = egui::Context::default();
        ctx.set_fonts(definitions);
        ctx.begin_pass(Default::default());
        let catalog = include_str!("i18n/zh-CN.json");
        ctx.fonts_mut(|fonts| {
            for family in [
                egui::FontFamily::Proportional,
                egui::FontFamily::Monospace,
                document_family,
            ] {
                let font = egui::FontId::new(16.0, family);
                for text in [
                    catalog,
                    "中文简体繁體變化，。！？教学积件二次函数中垂线偏振光",
                ] {
                    // Check CJK characters and full-width punctuation. Latin
                    // glyphs come from egui's existing default fonts.
                    for ch in text.chars().filter(|ch| *ch >= '\u{2e80}') {
                        assert!(fonts.has_glyph(&font, ch), "Missing glyph: {ch} ({font:?})");
                    }
                }
                let galley = fonts.layout_no_wrap(
                    "中文界面：文件、编辑、构造、度量".into(),
                    font,
                    egui::Color32::BLACK,
                );
                assert!(galley.size().x > 0.0 && galley.size().y > 0.0);
            }
        });
        ctx.end_pass().textures_delta.clear();
    }
}
