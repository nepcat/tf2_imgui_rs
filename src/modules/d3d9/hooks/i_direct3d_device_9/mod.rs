pub mod end_scene;
pub mod reset;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("EndScene error")]
    EndScene(#[from] end_scene::Error),

    #[error("Reset error")]
    Reset(#[from] reset::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct IDirect3DDevice9 {
    pub end_scene: end_scene::EndScene,
    pub reset: reset::Reset,
}

impl IDirect3DDevice9 {
    pub unsafe fn find_original(
        device_vtable: &windows::Win32::Graphics::Direct3D9::IDirect3DDevice9_Vtbl,
    ) -> Result<Self> {
        Ok(Self {
            end_scene: end_scene::EndScene::find_original(device_vtable)?,
            reset: reset::Reset::find_original(device_vtable)?,
        })
    }

    pub unsafe fn hooks_init(&self) -> Result<()> {
        self.end_scene.hook_init()?;
        self.reset.hook_init()?;

        Ok(())
    }

    pub unsafe fn hooks_restore(&self) -> Result<()> {
        if let Some(Err(error)) = self.end_scene.hook_restore() {
            return Err(Error::EndScene(error));
        }
        if let Some(Err(error)) = self.reset.hook_restore() {
            return Err(Error::Reset(error));
        }

        Ok(())
    }
}
