#[derive(Debug, Default)]
pub struct Windows {
    pub instance: windows::Win32::Foundation::HMODULE,

    pub we_initialized_console: Option<crate::utils::windows::console::Type>,
}

impl Drop for Windows {
    fn drop(&mut self) {
        unsafe {
            if self.we_initialized_console == Some(crate::utils::windows::console::Type::Alloc) {
                let _ = windows::Win32::System::Console::FreeConsole();
            }
        }
    }
}
