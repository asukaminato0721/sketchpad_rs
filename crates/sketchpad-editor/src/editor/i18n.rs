//! UI translations. Document text and MCP identifiers remain unchanged.
use serde::{Deserialize, Serialize};
use std::{
    cell::Cell,
    collections::BTreeMap,
    io,
    path::{Path, PathBuf},
    sync::OnceLock,
};

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
        directories::ProjectDirs::from("", "", "sketchpad_rs")
            .map(|dirs| dirs.config_dir().join("language.json"))
    }
    fn legacy_config_path() -> Option<PathBuf> {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .filter(|path| path.is_absolute())
            .map(|base| base.join("sketchpad_rs/language.json"))
    }
    pub(super) fn load() -> Self {
        Self::config_path()
            .map(|path| Self::load_from(&path, Self::legacy_config_path().as_deref()))
            .unwrap_or_default()
    }
    fn load_from(path: &Path, legacy: Option<&Path>) -> Self {
        for path in std::iter::once(path).chain(legacy.filter(|legacy| *legacy != path)) {
            match std::fs::read(path) {
                Ok(bytes) => match serde_json::from_slice(&bytes) {
                    Ok(language) => return language,
                    Err(error) => {
                        log::warn!(
                            "Could not read language preference {}: {error}",
                            path.display()
                        );
                        return Self::default();
                    }
                },
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => {
                    log::warn!(
                        "Could not read language preference {}: {error}",
                        path.display()
                    );
                    return Self::default();
                }
            }
        }
        Self::default()
    }
    pub(super) fn save(self) -> Result<(), String> {
        let path = Self::config_path().ok_or("No configuration directory is available")?;
        self.save_to(&path)
            .map_err(|error| format!("{}: {error}", path.display()))
    }
    fn save_to(self, path: &Path) -> io::Result<()> {
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("No configuration directory"))?;
        std::fs::create_dir_all(parent)?;
        // Create the temporary file on the same volume. Persist replaces an
        // existing preference file on Windows as well as Unix.
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        serde_json::to_writer(temporary.as_file_mut(), &self).map_err(io::Error::other)?;
        temporary.as_file().sync_all()?;
        temporary.persist(path).map_err(|error| error.error)?;
        Ok(())
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
    fn language_save_replaces_existing_file_and_supports_unicode_paths() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("用户设置/language.json");
        assert_eq!(Language::load_from(&path, None), Language::English);
        Language::Chinese.save_to(&path).unwrap();
        assert_eq!(Language::load_from(&path, None), Language::Chinese);
        Language::English.save_to(&path).unwrap();
        assert_eq!(Language::load_from(&path, None), Language::English);
        assert_eq!(
            std::fs::read_dir(path.parent().unwrap()).unwrap().count(),
            1
        );
    }

    #[test]
    fn legacy_language_is_used_only_when_platform_file_is_absent() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("platform/language.json");
        let legacy = directory.path().join("legacy/language.json");
        Language::Chinese.save_to(&legacy).unwrap();
        assert_eq!(Language::load_from(&path, Some(&legacy)), Language::Chinese);
        Language::English.save_to(&path).unwrap();
        assert_eq!(Language::load_from(&path, Some(&legacy)), Language::English);
        std::fs::write(&path, b"invalid JSON").unwrap();
        assert_eq!(Language::load_from(&path, Some(&legacy)), Language::English);
        assert_eq!(std::fs::read(&path).unwrap(), b"invalid JSON");
    }

    #[test]
    fn failed_language_save_removes_temporary_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("language.json");
        std::fs::create_dir(&path).unwrap();
        assert!(Language::Chinese.save_to(&path).is_err());
        assert!(path.is_dir());
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[test]
    fn replacing_document_keeps_interface_language() {
        let mut editor = super::super::Editor::new(sketchpad_engine::document::Document::new());
        editor.language = Language::Chinese;
        editor.replace_document(sketchpad_engine::document::Document::new());
        assert_eq!(editor.language, Language::Chinese);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_language_path_uses_roaming_app_data() {
        let base = directories::BaseDirs::new().unwrap();
        assert_eq!(
            Language::config_path().unwrap(),
            base.config_dir().join("sketchpad_rs/config/language.json")
        );
    }

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
