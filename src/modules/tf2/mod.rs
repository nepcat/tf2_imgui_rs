#![allow(clippy::missing_safety_doc)]

pub mod hooks;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Couldn't find game's window")]
    FindWindow,

    #[error("Found multiple Valve001 windows")]
    MultipleValve001,

    #[error("Hooks error")]
    Hooks(#[from] hooks::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

pub struct TF2 {
    pub hwnd: windows::Win32::Foundation::HWND,

    pub hooks: hooks::Hooks,
}

impl TF2 {
    pub unsafe fn new() -> Result<Self> {
        let mut volvo_windows = crate::utils::windows::hwnd::get_current_process_volvo_windows();
        if volvo_windows.len() > 1 {
            return Err(Error::MultipleValve001);
        }
        let Some(hwnd) = volvo_windows.pop() else {
            return Err(Error::FindWindow);
        };

        let hooks = hooks::Hooks::find_original(hwnd)?;

        Ok(Self { hwnd, hooks })
    }

    pub unsafe fn hooks_init(&self) -> Result<()> {
        self.hooks.init()?;

        Ok(())
    }

    pub unsafe fn hooks_restore(&self) -> Result<()> {
        self.hooks.restore()?;

        Ok(())
    }
}
