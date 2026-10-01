use super::Document;
use std::fs;
use std::io::Write;
use std::path::Path;

impl Document {
    pub fn verify_roundtrip(&self) -> Result<(), String> {
        let bytes = self.encode()?;
        let reparsed = Self::from_bytes(&bytes, None)?;
        self.validate_reparsed_document(&reparsed)
    }
    pub fn save_to(&mut self, path: &Path) -> Result<(), String> {
        let reordered_additions = self.has_pending_multi_page_additions();
        let data = self.encode()?;
        let reparsed = Self::from_bytes(&data, Some(path.to_path_buf()))?;
        let had_deletion = self.objects.iter().any(|o| o.deleted);
        self.validate_reparsed_document(&reparsed)?;
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let stem = path.file_name().ok_or("Invalid path")?.to_string_lossy();
        let mut n = 0;
        let temp = loop {
            let p = parent.join(format!(".{stem}.{}.{}.tmp", std::process::id(), n));
            if !p.exists() {
                break p;
            }
            n += 1;
        };
        let result = (|| -> Result<(), String> {
            let mut f = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temp)
                .map_err(|e| e.to_string())?;
            f.write_all(&data).map_err(|e| e.to_string())?;
            f.sync_all().map_err(|e| e.to_string())?;
            fs::rename(&temp, path).map_err(|e| e.to_string())?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result?;
        self.measurement_names = reparsed.measurement_names;
        self.objects = reparsed.objects;
        self.marked_coordinate_system = reparsed.marked_coordinate_system;
        self.frame_styles = reparsed.frame_styles;
        self.page_names = reparsed.page_names;
        if had_deletion || reordered_additions {
            self.selected.clear();
        }
        self.original = data;
        self.merge_aliases.clear();
        self.run_font_families.clear();
        self.source_count = self.objects.len();
        self.custom_transform_definitions = reparsed.custom_transform_definitions;
        self.path = Some(path.to_path_buf());
        self.safe_structure = reparsed.safe_structure;
        self.image_read_only = reparsed.image_read_only;
        self.partial_view = reparsed.partial_view;
        self.undo.clear();
        self.redo.clear();
        self.dirty = false;
        Ok(())
    }
}
