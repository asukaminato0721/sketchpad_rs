use super::Editor;
use sketchpad_rs::document::colorized::{ColorBinding, ColorRangeBehavior};
#[derive(Clone, Copy)]
pub(super) enum ColorModel {
    Spectrum,
    Grayscale,
    Rgb,
    Hsv,
}
impl Editor {
    pub(super) fn colorize_selected(&mut self, model: ColorModel) {
        let values: Vec<_> = self
            .selection_order
            .iter()
            .copied()
            .filter(|id| {
                self.doc
                    .objects
                    .get(id.saturating_sub(1))
                    .is_some_and(|o| o.kind.numeric_unit().is_some())
            })
            .collect();
        let sources: Vec<_> = self
            .selection_order
            .iter()
            .copied()
            .filter(|id| !values.contains(id))
            .collect();
        let binding = match (model, values.as_slice()) {
            (ColorModel::Spectrum, [value]) => ColorBinding::Spectrum { value: *value },
            (ColorModel::Grayscale, [value]) => ColorBinding::Grayscale { value: *value },
            (ColorModel::Rgb, [red, green, blue]) => ColorBinding::Rgb {
                red: *red,
                green: *green,
                blue: *blue,
            },
            (ColorModel::Hsv, [hue, saturation, value]) => ColorBinding::Hsv {
                hue: *hue,
                saturation: *saturation,
                value: *value,
            },
            _ => {
                self.status = match model {
                    ColorModel::Spectrum | ColorModel::Grayscale => {
                        "Select geometry and one numeric value"
                    }
                    ColorModel::Rgb => {
                        "Select geometry and numeric values in red, green, blue order"
                    }
                    ColorModel::Hsv => {
                        "Select geometry and numeric values in hue, saturation, brightness order"
                    }
                }
                .into();
                return;
            }
        };
        if sources.is_empty() {
            self.status = "Select geometry and its numeric color values".into();
            return;
        }
        // Preflight the complete selection; a command is one undoable operation.
        let mut candidate = self.doc.clone();
        let mut outputs = Vec::new();
        for source in &sources {
            match candidate.add_colorized(
                *source,
                binding.clone(),
                0.,
                1.,
                ColorRangeBehavior::Reflect,
            ) {
                Ok(id) => {
                    candidate.objects[*source - 1].visible = false;
                    outputs.push(id);
                }
                Err(error) => {
                    self.status = error;
                    return;
                }
            }
        }
        candidate.undo.truncate(self.doc.undo.len() + 1);
        self.doc = candidate;
        self.doc.selected = outputs.iter().copied().collect();
        self.selection_order = outputs;
        self.status = "Created live colors from the selected values (range 0–1, reflected)".into();
    }
}
