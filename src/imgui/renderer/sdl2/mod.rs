/*#[cfg(target_os = "windows")]
pub mod directx9;*/
#[cfg(target_os = "linux")]
pub mod opengl3;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Bad window")]
    BadWindow,

    #[cfg(target_os = "linux")]
    #[error("OpenGL3 error")]
    OpenGL3(#[from] opengl3::Error),
    /*#[cfg(target_os = "windows")]
    #[error("DirectX9 error")]
    DirectX9(#[from] directx9::Error),*/
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum SDL2 {
    #[cfg(target_os = "linux")]
    OpenGL3(opengl3::OpenGL3),
    /*#[cfg(target_os = "windows")]
    DirectX9(directx9::DirectX9),*/
}

impl SDL2 {
    pub unsafe fn init(
        window: *mut sdl2_sys::SDL_Window,
        init_renderer: InitRenderer,
    ) -> Result<Self> {
        if window.is_null() {
            return Err(Error::BadWindow);
        }

        Ok(match init_renderer {
            #[cfg(target_os = "linux")]
            InitRenderer::OpenGL3 { original_context } => {
                Self::OpenGL3(opengl3::OpenGL3::init(original_context, window)?)
            } /*#[cfg(target_os = "windows")]
              InitRenderer::DirectX9 { device } => {
              Self::DirectX9(directx9::DirectX9::init(window, device)?)
              }*/
        })
    }

    pub unsafe fn process_event(&self, event: *mut sdl2_sys::SDL_Event) -> bool {
        match self {
            #[cfg(target_os = "linux")]
            SDL2::OpenGL3(opengl3) => opengl3.process_event(event),
            /*#[cfg(target_os = "windows")]
            SDL2::DirectX9(directx9) => directx9.process_event(event),*/
        }
    }
}

pub enum InitRenderer {
    #[cfg(target_os = "linux")]
    OpenGL3 {
        original_context: sdl2_sys::SDL_GLContext,
    },
    /*#[cfg(target_os = "windows")]
    DirectX9 {
        device: *mut windows::Win32::Graphics::Direct3D9::IDirect3DDevice9,
    },*/
}
