use super::{Editor, OutputAction, PageSetup, PaperSize, PendingAction};
use crate::render::{Viewport, svg_page_on_paper, svg_page_with_button_offset};
use sketchpad_engine::document::Document;
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

fn png_from_svg(svg: &str) -> Result<Vec<u8>, String> {
    let mut process = Command::new("rsvg-convert")
        .arg("--format=png")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    process
        .stdin
        .take()
        .ok_or("Missing preview converter stdin")?
        .write_all(svg.as_bytes())
        .map_err(|error| error.to_string())?;
    let output = process
        .wait_with_output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    if !output.stdout.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err("Preview converter returned invalid output".into());
    }
    Ok(output.stdout)
}

impl PageSetup {
    fn dimensions_mm(self) -> (f32, f32) {
        let dimensions = match self.paper_size {
            PaperSize::A4 => (210., 297.),
            PaperSize::Letter => (215.9, 279.4),
        };
        if self.landscape {
            (dimensions.1, dimensions.0)
        } else {
            dimensions
        }
    }

    fn paper_name(self) -> &'static str {
        match self.paper_size {
            PaperSize::A4 => "A4",
            PaperSize::Letter => "Letter",
        }
    }
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

impl Editor {
    fn print_viewport(&self, viewport: &Viewport) -> Result<(Viewport, f32, f32), String> {
        const PX_PER_MM: f32 = 96. / 25.4;
        let (width_mm, height_mm) = self.page_setup.dimensions_mm();
        let margin = self.page_setup.margin_mm * PX_PER_MM;
        let printable = egui::Rect::from_min_max(
            egui::pos2(margin, margin),
            egui::pos2(
                width_mm * PX_PER_MM - margin,
                height_mm * PX_PER_MM - margin,
            ),
        );
        if printable.width() <= 0. || printable.height() <= 0. {
            return Err("Page margins leave no printable area".into());
        }
        let scale = if self.page_setup.fit_to_page {
            let ratio = (printable.width() / viewport.rect.width())
                .min(printable.height() / viewport.rect.height());
            viewport.scale * ratio
        } else {
            viewport.scale * f32::from(self.page_setup.scale_percent) / 100.
        };
        Ok((
            Viewport {
                rect: printable,
                scale,
                center: viewport.center,
            },
            width_mm,
            height_mm,
        ))
    }

    fn print_svg(&self, viewport: &Viewport) -> Result<String, String> {
        let (print_viewport, width_mm, height_mm) = self.print_viewport(viewport)?;
        svg_page_on_paper(
            &self.doc,
            &print_viewport,
            self.button_offset(&print_viewport),
            width_mm,
            height_mm,
        )
    }

    fn refresh_print_preview(&mut self, context: &egui::Context, viewport: &Viewport) {
        self.print_preview_texture = None;
        self.print_preview_error = None;
        let result = (|| -> Result<egui::TextureHandle, String> {
            let svg = self.print_svg(viewport)?;
            let png = png_from_svg(&svg)?;
            let decoded = image::load_from_memory(&png)
                .map_err(|error| error.to_string())?
                .to_rgba8();
            let size = [decoded.width() as usize, decoded.height() as usize];
            let image = egui::ColorImage::from_rgba_unmultiplied(size, decoded.as_raw());
            Ok(context.load_texture("print-preview", image, egui::TextureOptions::LINEAR))
        })();
        match result {
            Ok(texture) => self.print_preview_texture = Some(texture),
            Err(error) => self.print_preview_error = Some(error),
        }
    }

    pub(super) fn print_dialogs_ui(&mut self, root: &mut egui::Ui, viewport: &Viewport) {
        if self.page_setup_dialog {
            let before = self.page_setup;
            let mut close = false;
            egui::Modal::new(egui::Id::new("page_setup")).show(root.ctx(), |ui| {
                ui.heading("Page Setup");
                egui::ComboBox::from_label("Paper")
                    .selected_text(self.page_setup.paper_name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.page_setup.paper_size,
                            PaperSize::A4,
                            "A4 (210 × 297 mm)",
                        );
                        ui.selectable_value(
                            &mut self.page_setup.paper_size,
                            PaperSize::Letter,
                            "Letter (8.5 × 11 in)",
                        );
                    });
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.page_setup.landscape, false, "Portrait");
                    ui.selectable_value(&mut self.page_setup.landscape, true, "Landscape");
                });
                ui.add(
                    egui::Slider::new(&mut self.page_setup.margin_mm, 0.0..=40.0)
                        .suffix(" mm")
                        .text("Margins"),
                );
                ui.checkbox(&mut self.page_setup.fit_to_page, "Fit current view to page");
                ui.add_enabled(
                    !self.page_setup.fit_to_page,
                    egui::Slider::new(&mut self.page_setup.scale_percent, 25..=400)
                        .suffix("%")
                        .text("Scale"),
                );
                ui.label("Page setup affects PDF export, Print Preview, and Print.");
                close = ui.button("Done").clicked();
            });
            if self.page_setup != before {
                self.print_preview_texture = None;
                self.print_preview_error = None;
            }
            if close {
                self.page_setup_dialog = false;
            }
        }

        if self.print_preview_dialog {
            if self.print_preview_texture.is_none() && self.print_preview_error.is_none() {
                self.refresh_print_preview(root.ctx(), viewport);
            }
            let mut open = true;
            let mut refresh = false;
            let mut print = false;
            egui::Window::new("Print Preview")
                .open(&mut open)
                .default_size(egui::vec2(620., 720.))
                .show(root.ctx(), |ui| {
                    let (width, height) = self.page_setup.dimensions_mm();
                    ui.label(format!(
                        "{} · {} · {:.1} × {:.1} mm · {:.1} mm margins",
                        self.page_setup.paper_name(),
                        if self.page_setup.landscape {
                            "Landscape"
                        } else {
                            "Portrait"
                        },
                        width,
                        height,
                        self.page_setup.margin_mm
                    ));
                    ui.horizontal(|ui| {
                        refresh = ui.button("Refresh").clicked();
                        print = ui.button("Print…").clicked();
                    });
                    ui.separator();
                    if let Some(texture) = &self.print_preview_texture {
                        let available = ui.available_size();
                        let source = texture.size_vec2();
                        let factor = (available.x / source.x)
                            .min(available.y / source.y)
                            .min(1.0);
                        ui.add(
                            egui::Image::new(texture)
                                .fit_to_exact_size(source * factor)
                                .maintain_aspect_ratio(true),
                        );
                    } else if let Some(error) = &self.print_preview_error {
                        ui.colored_label(
                            egui::Color32::DARK_RED,
                            format!("Preview unavailable: {error}"),
                        );
                        ui.label("Install rsvg-convert to enable PDF output and preview.");
                    }
                });
            if refresh {
                self.refresh_print_preview(root.ctx(), viewport);
            }
            if print {
                self.pending_output = Some(OutputAction::Print);
            }
            self.print_preview_dialog = open;
            if !open {
                self.print_preview_texture = None;
                self.print_preview_error = None;
            }
        }
    }

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
        let tool_folder = self.tool_folder.take();
        let preferences = self.preferences;
        *self = Self::new(document);
        self.clipboard = clipboard;
        self.tool_folder = tool_folder;
        self.preferences = preferences;
        self.preferences_draft = preferences;
        self.sync_document_preferences();
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
        let object_ids_will_change =
            inserted_multi_page_additions || self.doc.objects.iter().any(|object| object.deleted);
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
                    self.clear_animation_tracks();
                    self.movements.clear();
                    self.presentations.clear();
                    self.animation_button_dialog = None;
                    self.movement_button_dialog = None;
                    self.scroll_button_dialog = None;
                    self.presentation_button_dialog = None;
                }
                let cleared_custom_transforms =
                    object_ids_will_change && !self.custom_transforms.is_empty();
                if cleared_custom_transforms {
                    self.custom_transforms.clear();
                    self.define_custom_transform_dialog = None;
                    self.edit_custom_transforms_dialog = false;
                }
                self.status = if cleared_custom_transforms {
                    format!(
                        "Saved {}; cleared custom-transform definitions after object IDs changed",
                        path.display()
                    )
                } else {
                    format!("Saved {}", path.display())
                };
                true
            }
            Err(e) => {
                self.status = format!("Save refused: {e}");
                false
            }
        }
    }
    pub(super) fn output_page(&mut self, viewport: &Viewport, action: OutputAction) {
        let svg = match if matches!(action, OutputAction::Svg) {
            svg_page_with_button_offset(&self.doc, viewport, self.button_offset(viewport))
        } else {
            self.print_svg(viewport)
        } {
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

#[cfg(test)]
mod pdf_tests {
    use super::*;

    #[test]
    fn print_svg_uses_page_setup_and_printable_clip() {
        let mut editor = Editor::new(Document::new());
        let viewport = Viewport {
            rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800., 600.)),
            scale: 1.,
            center: sketchpad_engine::document::Point { x: 400., y: 300. },
        };
        let portrait = editor.print_svg(&viewport).unwrap();
        assert!(portrait.contains("width=\"210mm\" height=\"297mm\""));
        assert!(portrait.contains("<clipPath id=\"sketch-clip\">"));

        editor.page_setup.landscape = true;
        editor.page_setup.margin_mm = 20.;
        editor.page_setup.fit_to_page = false;
        editor.page_setup.scale_percent = 75;
        let landscape = editor.print_svg(&viewport).unwrap();
        assert!(landscape.contains("width=\"297mm\" height=\"210mm\""));
        assert_ne!(landscape, portrait);
    }

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
