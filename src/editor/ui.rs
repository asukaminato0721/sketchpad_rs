use super::Editor;
use sketchpad_rs::render::Viewport;

impl Editor {
    pub(super) fn ui(&mut self, root: &mut egui::Ui) -> Viewport {
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
            if root.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::C)) {
                self.copy_selected();
            }
            if root.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::X)) {
                self.cut_selected();
            }
            if root.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::V)) {
                self.paste();
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
        self.tools_ui(root);
        self.status_ui(root);
        let rect = self.canvas_ui(root);
        self.document_dialogs_ui(root);
        self.action_dialogs_ui(root);
        self.calculation_dialogs_ui(root);
        self.function_dialogs_ui(root);
        self.plot_dialogs_ui(root);
        self.transform_dialogs_ui(root);
        let viewport = Viewport {
            rect,
            scale: self.scale,
            center: self.center,
        };
        if let Some(action) = self.pending_output.take() {
            self.output_page(&viewport, action);
        }
        viewport
    }
}
