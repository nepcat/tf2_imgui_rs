pub mod gl_swap_window;
pub mod poll_event;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("GLSwapWindow error")]
    GLSwapWindow(#[from] gl_swap_window::Error),
    #[error("PollEvent error")]
    PollEvent(#[from] poll_event::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Hooks {
    pub gl_swap_window: gl_swap_window::GLSwapWindow,
    pub poll_event: poll_event::PollEvent,
}

impl Hooks {
    pub unsafe fn find_original(library: &crate::utils::module::Module) -> Result<Self> {
        Ok(Self {
            gl_swap_window: gl_swap_window::GLSwapWindow::find_original(library)?,
            poll_event: poll_event::PollEvent::find_original(library)?,
        })
    }

    pub unsafe fn init(&self) {
        self.gl_swap_window.hook_init();
        self.poll_event.hook_init();
    }

    pub unsafe fn restore(&self) {
        self.gl_swap_window.hook_restore();
        self.poll_event.hook_restore();
    }
}
