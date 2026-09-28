use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

// The first match respects the user's locale for shared Han glyphs. The language
// matches also cover systems with separate Chinese, Japanese, and Korean fonts.
const CJK_PATTERNS: &[&str] = &[
    "sans-serif:charset=4e2d",
    "sans-serif:lang=zh-cn",
    "sans-serif:lang=zh-tw",
    "sans-serif:lang=ja",
    "sans-serif:lang=ko",
];

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct SystemFont {
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

    pub(super) fn data(&self) -> Option<egui::FontData> {
        let bytes = std::fs::read(&self.path).ok()?;
        let mut data = egui::FontData::from_owned(bytes);
        data.index = self.index;
        Some(data)
    }
}

pub(super) fn system_font(lookup: &str) -> Option<SystemFont> {
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

pub(super) fn add_cjk_fallbacks(fonts: &mut egui::FontDefinitions) {
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
}
