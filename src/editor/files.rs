use super::{Editor, OutputAction, PendingAction};
use sketchpad_rs::document::Document;
use sketchpad_rs::render::{Viewport, svg_page_with_button_offset};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn pdf_from_svg(svg: &str) -> Result<Vec<u8>, String> {
    let path = temporary_pdf_path("convert");
    let result = (|| -> Result<Vec<u8>, String> {
        let mut process = Command::new("rsvg-convert")
            .arg("--format=pdf")
            .arg(format!("--output={}", path.display()))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| error.to_string())?;
        process
            .stdin
            .take()
            .ok_or("Missing PDF converter stdin")?
            .write_all(svg.as_bytes())
            .map_err(|error| error.to_string())?;
        let output = process
            .wait_with_output()
            .map_err(|error| error.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
        }
        let pdf = std::fs::read(&path).map_err(|error| error.to_string())?;
        if !pdf.starts_with(b"%PDF-") {
            return Err("PDF converter returned invalid output".into());
        }
        Ok(pdf)
    })();
    let _ = std::fs::remove_file(path);
    result
}
fn temporary_pdf_path(purpose: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "sketchpad-{purpose}-{}-{unique}.pdf",
        std::process::id()
    ))
}

#[cfg(test)]
mod pdf_tests {
    use super::*;
    #[test]
    fn converts_visible_page_svg_to_pdf_when_converter_is_available() {
        if Command::new("rsvg-convert")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let pdf = pdf_from_svg("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"20\" height=\"20\"><circle cx=\"10\" cy=\"10\" r=\"5\"/></svg>").unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
        assert!(pdf.len() > 100);
    }
}

impl Editor {
    pub(super) fn open(&mut self, path: PathBuf) {
        match Document::open(&path) {
            Ok(d) => {
                self.replace_document(d);
                self.status = format!("Opened {}", path.display());
            }
            Err(e) => self.status = format!("Open failed: {e}"),
        }
    }
    pub(super) fn import_picture(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG picture", &["png"])
            .pick_file()
        else {
            return;
        };
        let result = (|| -> Result<usize, String> {
            let png = std::fs::read(&path).map_err(|error| error.to_string())?;
            self.doc.import_png(&png, self.center)
        })();
        match result {
            Ok(id) => {
                self.image_textures.clear();
                self.select_only(id);
                self.status = format!("Imported {}", path.display());
            }
            Err(error) => self.status = format!("Picture import refused: {error}"),
        }
    }
    pub(super) fn replace_document(&mut self, document: Document) {
        let clipboard = self.clipboard.take();
        *self = Self::new(document);
        self.clipboard = clipboard;
    }
    pub(super) fn save(&mut self, as_new: bool) -> bool {
        let path = if as_new || self.doc.path.is_none() {
            rfd::FileDialog::new()
                .add_filter("Sketchpad document", &["gsp"])
                .set_file_name("Sketch.gsp")
                .save_file()
        } else {
            self.doc.path.clone()
        };
        let Some(path) = path else { return false };
        let inserted_multi_page_additions = self.doc.has_pending_multi_page_additions();
        match self.doc.save_to(&path) {
            Ok(()) => {
                if inserted_multi_page_additions {
                    self.selection_order.clear();
                    self.image_textures.clear();
                    self.pending = None;
                    self.dragging = None;
                    self.inspected = None;
                    self.text_selected_range = None;
                    self.marked_center = None;
                    self.marked_mirror = None;
                    self.marked_vector = None;
                    self.marked_distance = None;
                    self.marked_rectangular_distances = None;
                    self.marked_angle = None;
                    self.marked_ratio = None;
                    self.marked_ratio_value = None;
                    self.trace_marks.clear();
                    self.animations.clear();
                    self.movements.clear();
                    self.presentations.clear();
                    self.animation_button_dialog = None;
                    self.movement_button_dialog = None;
                    self.scroll_button_dialog = None;
                    self.presentation_button_dialog = None;
                }
                self.status = format!("Saved {}", path.display());
                true
            }
            Err(e) => {
                self.status = format!("Save refused: {e}");
                false
            }
        }
    }
    pub(super) fn output_page(&mut self, viewport: &Viewport, action: OutputAction) {
        let svg =
            match svg_page_with_button_offset(&self.doc, viewport, self.button_offset(viewport)) {
                Ok(svg) => svg,
                Err(error) => {
                    self.status = format!("Output refused: {error}");
                    return;
                }
            };
        if matches!(action, OutputAction::Svg) {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("SVG image", &["svg"])
                .set_file_name("Sketch.svg")
                .save_file()
            else {
                return;
            };
            match std::fs::write(&path, svg) {
                Ok(()) => self.status = format!("Exported {}", path.display()),
                Err(error) => self.status = format!("Export failed: {error}"),
            }
            return;
        }
        let pdf = match pdf_from_svg(&svg) {
            Ok(pdf) => pdf,
            Err(error) => {
                self.status = format!("PDF conversion failed: {error}");
                return;
            }
        };
        if matches!(action, OutputAction::Pdf) {
            let Some(path) = rfd::FileDialog::new()
                .add_filter("PDF document", &["pdf"])
                .set_file_name("Sketch.pdf")
                .save_file()
            else {
                return;
            };
            match std::fs::write(&path, pdf) {
                Ok(()) => self.status = format!("Exported {}", path.display()),
                Err(error) => self.status = format!("Export failed: {error}"),
            }
            return;
        }
        let path = temporary_pdf_path("print");
        if let Err(error) = std::fs::write(&path, pdf) {
            self.status = format!("Print failed: {error}");
            return;
        }
        let result = Command::new("lp").arg(&path).output();
        let _ = std::fs::remove_file(&path);
        self.status = match result {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).trim().to_owned()
            }
            Ok(output) => format!(
                "Print failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            Err(error) => format!("Print failed: {error}"),
        };
    }
    pub(super) fn finish_pending_action(&mut self) {
        match self.pending_action.take() {
            Some(PendingAction::New) => self.replace_document(Document::new()),
            Some(PendingAction::Open(path)) => self.open(path),
            Some(PendingAction::Quit) => self.exit_requested = true,
            None => {}
        }
    }
}
