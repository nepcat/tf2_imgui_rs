#[cfg(target_os = "linux")]
pub mod sdl2;
#[cfg(target_os = "windows")]
pub mod win32;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[cfg(target_os = "linux")]
    #[error("SDL2 error")]
    SDL2(#[from] sdl2::Error),

    #[cfg(target_os = "windows")]
    #[error("Win32 error")]
    Win32(#[from] win32::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Renderer {
    #[cfg(target_os = "linux")]
    SDL2(sdl2::SDL2),

    #[cfg(target_os = "windows")]
    Win32(win32::Win32),
}

impl Renderer {
    pub unsafe fn init(init_renderer: InitRenderer) -> Result<Self> {
        Ok(match init_renderer {
            #[cfg(target_os = "linux")]
            InitRenderer::SDL2 { window, renderer } => {
                Self::SDL2(sdl2::SDL2::init(window, renderer)?)
            }
            #[cfg(target_os = "windows")]
            InitRenderer::Win32 { window, renderer } => {
                Self::Win32(win32::Win32::init(window, renderer)?)
            }
        })
    }
}

pub enum InitRenderer {
    #[cfg(target_os = "linux")]
    SDL2 {
        window: *mut sdl2_sys::SDL_Window,
        renderer: sdl2::InitRenderer,
    },

    #[cfg(target_os = "windows")]
    Win32 {
        window: windows::Win32::Foundation::HWND,
        renderer: win32::InitRenderer,
    },
}
