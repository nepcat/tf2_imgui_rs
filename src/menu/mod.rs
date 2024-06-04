#![allow(clippy::missing_safety_doc)]

use imgui_rs::root as imgui_rs;

#[derive(Debug, Default)]
pub struct Menu {
    pub is_enabled: bool,
}

impl Menu {
    pub unsafe fn render(&self) {
        /* Render cursor when menu is open */
        if let Some(imgui_io) = imgui_rs::ImGui::GetIO().as_mut() {
            imgui_io.MouseDrawCursor = self.is_enabled;
        }
        /* Render window(s) */
        if self.is_enabled {
            imgui_rs::ImGui::ShowDemoWindow(core::ptr::null_mut());
        }
    }

    pub unsafe fn update_input(&mut self) {
        /* Update menu status */
        if imgui_rs::ImGui::IsKeyPressed(imgui_rs::ImGuiKey::ImGuiKey_Insert, false) {
            self.is_enabled = !self.is_enabled;
        }
    }
}
