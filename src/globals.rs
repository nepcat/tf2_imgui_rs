/* Various libraries, set on lib.rs -> try_init() -> ThisLib::init()  */
pub static MODULES: once_cell::sync::OnceCell<crate::modules::Modules> =
    once_cell::sync::OnceCell::new();

/* Both ImGui and Menu */
pub static IMGUI: once_cell::sync::OnceCell<crate::imgui::ImGui> = once_cell::sync::OnceCell::new();

/* Check if we are unloading */
lazy_static::lazy_static! {
    pub static ref UNLOADING: parking_lot::Mutex<bool> = Default::default();
}
