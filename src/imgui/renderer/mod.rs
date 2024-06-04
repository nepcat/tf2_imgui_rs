pub mod sdl2;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("SDL2 error")]
    SDL2(#[from] sdl2::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Renderer {
    SDL2(sdl2::SDL2),
}

impl Renderer {
    pub unsafe fn init(init_renderer: InitRenderer) -> Result<Self> {
        Ok(match init_renderer {
            InitRenderer::SDL2 { window, renderer } => {
                Self::SDL2(sdl2::SDL2::init(window, renderer)?)
            }
        })
    }
}

pub enum InitRenderer {
    SDL2 {
        window: *mut sdl2_sys::SDL_Window,
        renderer: sdl2::InitRenderer,
    },
}
