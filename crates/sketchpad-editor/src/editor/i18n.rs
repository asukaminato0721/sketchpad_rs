//! UI translations. Document text and MCP identifiers remain unchanged.
use serde::{Deserialize, Serialize};
use std::{cell::Cell, collections::BTreeMap, path::PathBuf, sync::OnceLock};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum Language {
    #[default]
    #[serde(rename = "en")]
    English,
    #[serde(rename = "zh-CN")]
    Chinese,
}

thread_local! { static LANGUAGE: Cell<Language> = const { Cell::new(Language::English) }; }

impl Language {
    pub(super) fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Chinese => "zh-CN",
        }
    }
    pub(super) fn activate(self) {
        LANGUAGE.set(self);
    }
    fn config_path() -> Option<PathBuf> {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .map(|base| base.join("sketchpad_rs/language.json"))
    }
    pub(super) fn load() -> Self {
        Self::config_path()
            .and_then(|path| std::fs::read(path).ok())
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }
    pub(super) fn save(self) -> Result<(), String> {
        let path = Self::config_path().ok_or("No configuration directory is available")?;
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
        std::fs::write(&temporary, serde_json::to_vec(&self).unwrap())
            .map_err(|e| e.to_string())?;
        std::fs::rename(temporary, path).map_err(|e| e.to_string())
    }
}

fn catalog() -> &'static BTreeMap<String, String> {
    static CATALOG: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("i18n/zh-CN.json")).expect("valid Chinese UI catalog")
    })
}

fn templates() -> &'static Vec<(&'static String, &'static String)> {
    static TEMPLATES: OnceLock<Vec<(&String, &String)>> = OnceLock::new();
    TEMPLATES.get_or_init(|| {
        let mut entries: Vec<_> = catalog()
            .iter()
            .filter(|(key, _)| key.contains('{'))
            .collect();
        // Prefer specific messages over broad prefixes such as "Page {}".
        entries.sort_by_key(|(key, _)| std::cmp::Reverse(key.len()));
        entries
    })
}

/// Translate interface text at presentation time so stored status messages can
/// change language with the rest of the UI. Missing entries use English.
pub(super) fn tr(text: impl AsRef<str>) -> String {
    let text = text.as_ref();
    if LANGUAGE.get() == Language::English {
        return text.to_owned();
    }
    if let Some(translated) = catalog().get(text) {
        return translated.clone();
    }
    for (source, translated) in templates() {
        if let Some(result) = translate_template(source, translated, text) {
            return result;
        }
    }
    text.to_owned()
}

fn translate_template(source: &str, translated: &str, text: &str) -> Option<String> {
    let mut rest = text;
    let mut template = source;
    let mut values = Vec::new();
    while let Some(open) = template.find('{') {
        rest = rest.strip_prefix(&template[..open])?;
        let close = template[open..].find('}')? + open;
        let key = template[open + 1..close].split(':').next()?;
        template = &template[close + 1..];
        let literal = template.split('{').next()?;
        let end = if literal.is_empty() {
            rest.len()
        } else {
            rest.find(literal)?
        };
        values.push((key, &rest[..end]));
        rest = &rest[end..];
    }
    if rest != template {
        return None;
    }
    let mut output = String::new();
    let mut remaining = translated;
    let mut anonymous = 0;
    while let Some(open) = remaining.find('{') {
        output.push_str(&remaining[..open]);
        let close = remaining[open..].find('}')? + open;
        let key = remaining[open + 1..close].split(':').next()?;
        let value = if key.is_empty() {
            let value = values
                .iter()
                .filter(|(name, _)| name.is_empty())
                .nth(anonymous)?
                .1;
            anonymous += 1;
            value
        } else {
            values.iter().find(|(name, _)| *name == key)?.1
        };
        output.push_str(value);
        remaining = &remaining[close + 1..];
    }
    output.push_str(remaining);
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn switching_translates_ui_and_preserves_unknown_document_text() {
        Language::Chinese.activate();
        assert_eq!(tr("File"), "文件");
        assert_eq!(tr("Page 3"), "第 3 页");
        assert_eq!(tr("Page 3: Triangle"), "第 3 页：Triangle");
        assert_eq!(tr("My triangle ABC"), "My triangle ABC");
        Language::English.activate();
        assert_eq!(tr("File"), "File");
    }
    #[test]
    fn translated_templates_keep_values_and_allow_named_reordering() {
        assert_eq!(
            translate_template("{a} then {b}", "{b}，然后 {a}", "A then B").as_deref(),
            Some("B，然后 A")
        );
        assert_eq!(
            translate_template("Page {}", "第 {} 页", "Page 12").as_deref(),
            Some("第 12 页")
        );
        assert!(translate_template("Page {}", "第 {} 页", "My Page 12").is_none());
        assert_eq!(
            translate_template(
                "Size: {:.1} × {:.1}",
                "尺寸：{:.1} × {:.1}",
                "Size: 210.0 × 297.0"
            )
            .as_deref(),
            Some("尺寸：210.0 × 297.0")
        );
    }
}
