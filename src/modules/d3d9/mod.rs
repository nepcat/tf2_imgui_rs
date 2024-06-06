#![allow(clippy::missing_safety_doc)]

pub mod hooks;

#[cfg(target_os = "windows")]
pub static FILEPATH: &str = "d3d9.dll";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to get device vtable")]
    DeviceVtable,

    #[error("Hooks error")]
    Hooks(#[from] hooks::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

pub struct D3D9 {
    device_vtable: *const windows::Win32::Graphics::Direct3D9::IDirect3DDevice9_Vtbl,
    /* Updated on IDirect3DDevice9's EndScene hook */
    /*pub device:
        once_cell::sync::OnceCell<*mut windows::Win32::Graphics::Direct3D9::IDirect3DDevice9>,*/
    pub hooks: hooks::Hooks,
}

unsafe impl Send for D3D9 {}
unsafe impl Sync for D3D9 {}

impl D3D9 {
    pub unsafe fn new(shader_api_dx9: &super::shader_api_dx9::ShaderApiDX9) -> Result<Self> {
        let Some(device) = shader_api_dx9
            .variables
            .material_system
            .shader_api_dx9
            .shader_device_dx8
            .gp_d3d_device
            .as_ref()
        else {
            return Err(Error::DeviceVtable);
        };
        let device_vtable = windows::core::Interface::vtable(device) as *const _;

        let hooks = hooks::Hooks::find_original(&(*device_vtable))?;

        Ok(Self {
            device_vtable,
            hooks,
            /* Updated on IDirect3DDevice9's EndScene hook */
            // device: Default::default(),
        })
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
