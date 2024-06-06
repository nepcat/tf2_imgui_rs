#![allow(irrefutable_let_patterns)]

type Function = unsafe extern "system" fn(
    thisptr: *mut windows::Win32::Graphics::Direct3D9::IDirect3DDevice9,
) -> windows::core::HRESULT;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Detour error")]
    Hooks(#[from] retour::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct EndScene {
    pub original: Function,
    pub hook: retour::GenericDetour<Function>,
}

impl EndScene {
    pub unsafe fn find_original(
        device_vtable: &windows::Win32::Graphics::Direct3D9::IDirect3DDevice9_Vtbl,
    ) -> Result<Self> {
        let original = std::mem::transmute(device_vtable.EndScene);

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
) -> windows::core::HRESULT {
    // log::debug!("Our hooked function called");
    match try_our(thisptr) {
        Ok(result) => result,
        Err(error) => {
            log::error!("Failed to execute our hooked function, reason: {error:?}. Crashing!");
            std::process::exit(1);
        }
    }
}

unsafe fn try_our(
    thisptr: *mut windows::Win32::Graphics::Direct3D9::IDirect3DDevice9,
) -> anyhow::Result<windows::core::HRESULT> {
    use anyhow::Context;
    let modules = crate::globals::MODULES
        .get()
        .context("Failed to get global modules")?;
    let d3d9_module = &modules.d3d9;
    let tf2_module = &modules.tf2;
    let detour = &d3d9_module.hooks.i_direct3d_device_9.end_scene.hook;

    if thisptr.is_null() {
        /* Early return */
        return Ok(detour.call(thisptr));
    }

    /*if !thisptr.is_null() && d3d9_module.device.get().is_none() {
        log::debug!("Setting d3d9 module as {thisptr:?}");
        let _ = d3d9_module.device.set(thisptr);
    }*/

    let imgui = crate::globals::IMGUI
        .get_or_try_init(|| {
            let imgui = crate::imgui::ImGui::init(crate::imgui::renderer::InitRenderer::Win32 {
                window: tf2_module.hwnd,
                renderer: crate::imgui::renderer::win32::InitRenderer::DirectX9 { device: thisptr },
            })?;
            log::debug!("Menu initialized succesfully!");
            Ok(imgui) as anyhow::Result<_>
        })
        .context("Failed to initialize ImGui")?;

    /* Render only if backend matches! */
    if let crate::imgui::renderer::Renderer::Win32(
        crate::imgui::renderer::win32::Win32::DirectX9(_),
    ) = &imgui.renderer
    {
        imgui.render();
    }

    Ok(detour.call(thisptr))
}
