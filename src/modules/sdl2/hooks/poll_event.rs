#![allow(irrefutable_let_patterns)] /* TODO: Me from the future, delete this! */

#[cfg(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "windows", target_arch = "x86_64")
))]
type FunctionResult = std::os::raw::c_int;

/*#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
type FunctionResult = i64;*/

#[cfg(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "windows", target_arch = "x86_64")
))]
type Function = unsafe extern "system" fn(*mut sdl2_sys::SDL_Event) -> FunctionResult;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Null error")]
    Null(#[from] std::ffi::NulError),

    #[error("Failed to find original")]
    FindOriginal,
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct PollEvent {
    /* TODO: Use retour */
    pub original: Function,
    pub address: *mut Function,
}

unsafe impl Send for PollEvent {}
unsafe impl Sync for PollEvent {}

impl PollEvent {
    pub unsafe fn find_original(library: &crate::utils::module::Module) -> Result<Self> {
        /* Credits: https://github.com/8dcc/tf2-cheat/blob/a8bffac0f3daaf1320d4f1ab4dd89af5b8b0e9b5/src/globals.c#L91 */

        /* Linux:
         * dlsym skips ENDBR64 instruction (4 bytes)
         * ff 25
         * 2e 68 19 00
         * Windows:
         * 48 ff 25
         * 51 bc 13 00 */

        let wrapper_ptr = library
            .get_symbol("SDL_PollEvent")?
            .ok_or(Error::FindOriginal)?;
        /* skip op codes ff 25 (JMP) */
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        let byte_offset = 2;
        /* skip op codes 48 ff 25 (JMP) */
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        let byte_offset = 3;

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

unsafe extern "system" fn our(event: *mut sdl2_sys::SDL_Event) -> FunctionResult {
    // log::debug!("Our hooked function called");
    match try_our(event) {
        Ok(result) => result,
        Err(error) => {
            log::error!("Failed to execute our hooked function, reason: {error:?}. Crashing!");
            std::process::exit(1);
        }
    }
}

unsafe fn try_our(event: *mut sdl2_sys::SDL_Event) -> anyhow::Result<FunctionResult> {
    use anyhow::Context;
    let modules = &crate::globals::MODULES
        .get()
        .context("Failed to get global modules")?;
    let original = &modules.sdl2.hooks.poll_event.original;

    let result = original(event);
    let Some(event) = event.as_mut() else {
        /* Early return on NULL event */
        return Ok(result);
    };

    /* Update for windows:
     * TF2 doesn't use SDL to create window(s), but instead their native WinApi functions (even on vulkan)
     * I'm still gonna keep this part of the code because its 100% functional, but it will be left unused */

    /* On windows initialize menu here! */
    #[cfg(target_os = "windows")]
    let imgui = {
        if event.window.windowID > 0
            && let Some(device) = modules.d3d9.device.get()
            && let window = sdl2_sys::SDL_GetWindowFromID(event.window.windowID)
            && !window.is_null()
        {
            let result = crate::globals::IMGUI
                .get_or_try_init(|| {
                    let imgui =
                        crate::imgui::ImGui::init(crate::imgui::renderer::InitRenderer::SDL2 {
                            window,
                            renderer: crate::imgui::renderer::sdl2::InitRenderer::DirectX9 {
                                device: (*device),
                            },
                        })?;
                    log::debug!("Menu initialized succesfully!");
                    Ok(imgui) as anyhow::Result<_>
                })
                .context("Failed to initialize ImGui")?;
            Some(result)
        } else {
            log::debug!(
                "Failed to initialize menu, window id {}",
                event.window.windowID
            );
            None
        }
    };
    /* On linux its initialized inside gl_swap_window hook */
    #[cfg(target_os = "linux")]
    let imgui = crate::globals::IMGUI.get();

    if let Some(imgui) = imgui {
        if let crate::imgui::renderer::Renderer::SDL2(sdl2) = &imgui.renderer {
            /* Threads scare me! */
            imgui.set_current_context();

            if result != 0 && sdl2.process_event(event) && imgui.menu.read().is_enabled {
                let event_type = std::mem::transmute::<u32, sdl2_sys::SDL_EventType>(event.type_);
                match event_type {
                    /* Only filter input events */
                    sdl2_sys::SDL_EventType::SDL_MOUSEMOTION
                    | sdl2_sys::SDL_EventType::SDL_MOUSEBUTTONDOWN
                    | sdl2_sys::SDL_EventType::SDL_MOUSEBUTTONUP
                    | sdl2_sys::SDL_EventType::SDL_MOUSEWHEEL
                    | sdl2_sys::SDL_EventType::SDL_KEYDOWN
                    | sdl2_sys::SDL_EventType::SDL_KEYUP
                    | sdl2_sys::SDL_EventType::SDL_TEXTEDITING
                    | sdl2_sys::SDL_EventType::SDL_TEXTINPUT => {
                        event.type_ = sdl2_sys::SDL_EventType::SDL_FIRSTEVENT as _
                    }
                    _ => {}
                };
            }
        }
    }

    Ok(result)
}
