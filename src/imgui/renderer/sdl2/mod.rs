pub mod opengl3;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("OpenGL3 error")]
    OpenGL3(#[from] opengl3::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum SDL2 {
    OpenGL3(opengl3::OpenGL3),
}

impl SDL2 {
    pub unsafe fn init(
        window: *mut sdl2_sys::SDL_Window,
        init_renderer: InitRenderer,
    ) -> Result<Self> {
        Ok(match init_renderer {
            InitRenderer::OpenGL3 { original_context } => {
                Self::OpenGL3(opengl3::OpenGL3::init(original_context, window)?)
            }
        })
    }
}

pub enum InitRenderer {
    OpenGL3 {
        original_context: sdl2_sys::SDL_GLContext,
    },
}
