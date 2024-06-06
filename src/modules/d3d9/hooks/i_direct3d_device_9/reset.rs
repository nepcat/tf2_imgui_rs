#![allow(irrefutable_let_patterns)]

type Function = unsafe extern "system" fn(
    thisptr: *mut windows::Win32::Graphics::Direct3D9::IDirect3DDevice9,
    params: *mut windows::Win32::Graphics::Direct3D9::D3DPRESENT_PARAMETERS,
) -> windows::core::HRESULT;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Detour error")]
    Hooks(#[from] retour::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Reset {
    pub original: Function,
    pub hook: retour::GenericDetour<Function>,
}

impl Reset {
    pub unsafe fn find_original(
        device_vtable: &windows::Win32::Graphics::Direct3D9::IDirect3DDevice9_Vtbl,
    ) -> Result<Self> {
        let original = std::mem::transmute(device_vtable.Reset);

        Ok(Self {
            original,
            hook: retour::GenericDetour::new(original, our)?,
        })
    }

    pub unsafe fn hook_init(&self) -> Result<bool> {
        Ok(match self.hook.is_enabled() {
            true => {
                log::debug!("Hook is already initialized");
                false
            }
            false => {
                log::debug!("Enabling hook");
                self.hook.enable()?;
                true
            }
        })
    }

    pub unsafe fn hook_restore(&self) -> Option<Result<()>> {
        match self.hook.is_enabled() {
            true => {
                log::debug!("Disabling hook");
                Some(match self.hook.disable() {
                    Ok(_) => Ok(()),
                    Err(error) => Err(Error::Hooks(error)),
                })
            }
            false => None,
        }
    }
}

unsafe extern "system" fn our(
    thisptr: *mut windows::Win32::Graphics::Direct3D9::IDirect3DDevice9,
    params: *mut windows::Win32::Graphics::Direct3D9::D3DPRESENT_PARAMETERS,
) -> windows::core::HRESULT {
    // log::debug!("Our hooked function called");
    match try_our(thisptr, params) {
        Ok(result) => result,
        Err(error) => {
            log::error!("Failed to execute our hooked function, reason: {error:?}. Crashing!");
            std::process::exit(1);
        }
    }
}

unsafe fn try_our(
    thisptr: *mut windows::Win32::Graphics::Direct3D9::IDirect3DDevice9,
    params: *mut windows::Win32::Graphics::Direct3D9::D3DPRESENT_PARAMETERS,
) -> anyhow::Result<windows::core::HRESULT> {
    use anyhow::Context;
    let modules = crate::globals::MODULES
        .get()
        .context("Failed to get global modules")?;
    let d3d9_module = &modules.d3d9;
    let detour = &d3d9_module.hooks.i_direct3d_device_9.reset.hook;

    Ok(
        if let Some(imgui) = crate::globals::IMGUI.get()
            && /*let crate::imgui::renderer::Renderer::SDL2(
                crate::imgui::renderer::sdl2::SDL2::DirectX9(directx9),*/
                let crate::imgui::renderer::Renderer::Win32(
                    crate::imgui::renderer::win32::Win32::DirectX9(directx9)
            ) = &imgui.renderer
        {
            directx9.update_device_objects(false);
            let result = detour.call(thisptr, params);
            directx9.update_device_objects(true);
            result
        } else {
            /* ImGui is not initialized yet */
            detour.call(thisptr, params)
        },
    )
}
