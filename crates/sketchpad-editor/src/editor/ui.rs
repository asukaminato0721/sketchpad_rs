use super::Editor;
use crate::render::Viewport;

impl Editor {
    pub(super) fn ui(&mut self, root: &mut egui::Ui) -> Viewport {
        self.language.activate();
        if self.pending_action.is_none() {
            if root.input_mut(|input| {
                input.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::S,
                )
            }) {
                self.save(true);
            } else if root
                .input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::S))
            {
                self.save(false);
            }
        }
        if !root.ctx().egui_wants_keyboard_input() && self.pending_action.is_none() {
            if root.input(|input| input.key_pressed(egui::Key::Escape)) {
                self.stop_all_sounds();
                if matches!(self.tool, super::Tool::Custom(_)) {
                    self.cancel_custom_tool();
                }
            }
            if root.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::C)) {
                self.copy_selected();
            }
            if root.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::X)) {
                self.cut_selected();
            }
            if root.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::V)) {
                self.paste();
            }
            let adjustment = root.input(|input| {
                if input.key_pressed(egui::Key::Plus) {
                    Some(1.)
                } else if input.key_pressed(egui::Key::Minus) {
                    Some(-1.)
                } else {
                    None
                }
            });
            if let Some(direction) = adjustment {
                let parameter = self.doc.selected.iter().find_map(|&id| {
                    let object = self.doc.objects.get(id.checked_sub(1)?)?;
                    let sketchpad_engine::document::Kind::Parameter(group) = &object.kind else {
                        return None;
                    };
                    Some((id, group.value(), group.animation_settings().keyboard_step))
                });
                if let Some((id, value, step)) = parameter {
                    match self.doc.set_parameter_value(id, value + direction * step) {
                        Ok(()) => self.status = format!("Adjusted parameter {id}"),
                        Err(error) => self.status = error,
                    }
                }
            }
        }
        if root.input(|input| {
            input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace)
        }) && !root.ctx().egui_wants_keyboard_input()
            && self.pending_action.is_none()
        {
            self.clear_selected();
        }
        self.menu_ui(root);
        self.motion_controller_ui(root);
        self.text_palette_ui(root);
        self.tool_strip_ui(root);
        self.status_ui(root);
        self.page_tabs_ui(root);
        self.tools_ui(root);
        self.about_ui(root);
        let rect = self.canvas_ui(root);
        self.document_dialogs_ui(root);
        self.preferences_dialog_ui(root);
        self.action_dialogs_ui(root);
        self.calculation_dialogs_ui(root);
        self.properties_dialog_ui(root);
        self.function_dialogs_ui(root);
        self.plot_dialogs_ui(root);
        self.transform_dialogs_ui(root);
        self.custom_transform_dialogs_ui(root);
        self.custom_tool_dialog_ui(root);
        let viewport = Viewport {
            rect,
            scale: self.scale,
            center: self.center,
        };
        self.print_dialogs_ui(root, &viewport);
        if let Some(action) = self.pending_output.take() {
            self.output_page(&viewport, action);
        }
        viewport
    }
}
