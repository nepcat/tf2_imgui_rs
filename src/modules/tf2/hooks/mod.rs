pub mod wnd_proc;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("WndProc error")]
    WndProc(#[from] wnd_proc::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Hooks {
    pub wnd_proc: wnd_proc::WndProc,
}

impl Hooks {
    pub unsafe fn find_original(hwnd: windows::Win32::Foundation::HWND) -> Result<Self> {
        Ok(Self {
            wnd_proc: wnd_proc::WndProc::find_original(hwnd)?,
        })
    }

    pub unsafe fn init(&self) -> Result<()> {
        self.wnd_proc.hook_init()?;

        Ok(())
    }

    pub unsafe fn restore(&self) -> Result<()> {
        self.wnd_proc.hook_restore()?;

        Ok(())
    }
}
