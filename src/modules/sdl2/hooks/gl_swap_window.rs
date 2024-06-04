#![allow(clippy::unit_arg)]
#![allow(irrefutable_let_patterns)] /* TODO: Me from the future, delete this! */

type Function = unsafe extern "system" fn(*mut sdl2_sys::SDL_Window);

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Null error")]
    Null(#[from] std::ffi::NulError),

    #[error("Failed to find original")]
    FindOriginal,
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct GLSwapWindow {
    /* TODO: Use retour */
    pub original: Function,
    pub address: *mut Function,
}

unsafe impl Send for GLSwapWindow {}
unsafe impl Sync for GLSwapWindow {}

impl GLSwapWindow {
    pub unsafe fn find_original(library: &crate::utils::module::Module) -> Result<Self> {
        /* Credits: https://github.com/8dcc/tf2-cheat/blob/a8bffac0f3daaf1320d4f1ab4dd89af5b8b0e9b5/src/globals.c#L76 */
        /* dlsym skips ENDBR64 instruction (4 bytes)
         * we are left with ff 25 16 ed 18 00
         */

        let wrapper_ptr = library
            .get_symbol("SDL_GL_SwapWindow")?
            .ok_or(Error::FindOriginal)?;
        /* skip op codes ff 25 (JMP) */
        let byte_offset = 2;
        /* now we are left with relative address (4 bytes), read it and use our util function to convert it to absolute */
        let address = crate::utils::memory::relative_to_absolute_i32(wrapper_ptr + byte_offset)
            as *mut Function;
        let original = address.read();

        Ok(Self { original, address })
    }

    pub unsafe fn hook_init(&self) -> bool {
        if std::ptr::eq(
            self.address.read() as *const Function,
            our as *const Function,
        ) {
            log::debug!("Hook is already initialized");
            false
        } else {
            log::debug!("Enabling hook");
            *self.address = our;
            true
        }
    }

    pub unsafe fn hook_restore(&self) -> bool {
        if std::ptr::eq(
            self.address.read() as *const Function,
            our as *const Function,
        ) {
            log::debug!("Disabling hook");
            *self.address = self.original;
            true
        } else {
            false
        }
    }
}

unsafe extern "system" fn our(window: *mut sdl2_sys::SDL_Window) {
    // log::debug!("Our hooked function called");
    match try_our(window) {
        Ok(result) => result,
        Err(error) => {
            log::error!("Failed to execute our hooked function, reason: {error:?}. Crashing!");
            std::process::exit(1);
        }
    }
}

unsafe fn try_our(window: *mut sdl2_sys::SDL_Window) -> anyhow::Result<()> {
    use anyhow::Context;
    let modules = crate::globals::MODULES
        .get()
        .context("Failed to get global modules")?;
    let original = &modules.sdl2.hooks.gl_swap_window.original;

    let imgui = crate::globals::IMGUI
        .get_or_try_init(|| {
            let original_context = sdl2_sys::SDL_GL_GetCurrentContext();
            let imgui = crate::imgui::ImGui::init(crate::imgui::renderer::InitRenderer::SDL2 {
                window,
                renderer: crate::imgui::renderer::sdl2::InitRenderer::OpenGL3 { original_context },
            })?;
            Ok(imgui) as anyhow::Result<_>
        })
        .context("Failed to initialize ImGui")?;

    /* Render only if backend matches! */
    if let crate::imgui::renderer::Renderer::SDL2(crate::imgui::renderer::sdl2::SDL2::OpenGL3(_)) =
        &imgui.renderer
    {
        imgui.render();
    }

    Ok(original(window))
}
