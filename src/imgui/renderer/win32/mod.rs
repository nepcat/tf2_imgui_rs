pub mod directx9;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Bad window")]
    BadWindow,

    #[error("DirectX9 error")]
    DirectX9(#[from] directx9::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Win32 {
    DirectX9(directx9::DirectX9),
}

impl Win32 {
    pub unsafe fn init(
        window: windows::Win32::Foundation::HWND,
        init_renderer: InitRenderer,
    ) -> Result<Self> {
        if window.0 == 0 {
            return Err(Error::BadWindow);
        }

        Ok(match init_renderer {
            InitRenderer::DirectX9 { device } => {
                Self::DirectX9(directx9::DirectX9::init(window, device)?)
            }
        })
    }
}

pub enum InitRenderer {
    DirectX9 {
        device: *mut windows::Win32::Graphics::Direct3D9::IDirect3DDevice9,
    },
}
