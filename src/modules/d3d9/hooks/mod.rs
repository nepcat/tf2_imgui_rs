pub mod i_direct3d_device_9;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IDirect3DDevice9 error")]
    IDirect3DDevice9(#[from] i_direct3d_device_9::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Hooks {
    pub i_direct3d_device_9: i_direct3d_device_9::IDirect3DDevice9,
}

impl Hooks {
    pub unsafe fn find_original(
        device_vtable: &windows::Win32::Graphics::Direct3D9::IDirect3DDevice9_Vtbl,
    ) -> Result<Self> {
        Ok(Self {
            i_direct3d_device_9: i_direct3d_device_9::IDirect3DDevice9::find_original(
                device_vtable,
            )?,
        })
    }

    pub unsafe fn init(&self) -> Result<()> {
        self.i_direct3d_device_9.hooks_init()?;

        Ok(())
    }

    pub unsafe fn restore(&self) -> Result<()> {
        self.i_direct3d_device_9.hooks_restore()?;

        Ok(())
    }
}
