#![allow(irrefutable_let_patterns)] /* TODO: Me from the future, delete this! */

type Function = unsafe extern "system" fn(*mut sdl2_sys::SDL_Event) -> std::os::raw::c_int;

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

        /* dlsym skips ENDBR64 instruction (4 bytes)
         * we are left with ff 25 2e 68 19 00
         */

        let wrapper_ptr = library
            .get_symbol("SDL_PollEvent")?
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

unsafe extern "system" fn our(event: *mut sdl2_sys::SDL_Event) -> std::os::raw::c_int {
    // log::debug!("Our hooked function called");
    match try_our(event) {
        Ok(result) => result,
        Err(error) => {
            log::error!("Failed to execute our hooked function, reason: {error:?}. Crashing!");
            std::process::exit(1);
        }
    }
}

unsafe fn try_our(event: *mut sdl2_sys::SDL_Event) -> anyhow::Result<std::os::raw::c_int> {
    use anyhow::Context;
    let original = &crate::globals::MODULES
        .get()
        .context("Failed to get global modules")?
        .sdl2
        .hooks
        .poll_event
        .original;

    let result = original(event);

    if let Some(imgui) = crate::globals::IMGUI.get() {
        if let crate::imgui::renderer::Renderer::SDL2(
            crate::imgui::renderer::sdl2::SDL2::OpenGL3(opengl3),
        ) = &imgui.renderer
        {
            /* Threads scare me! */
            imgui.set_current_context();

            if result != 0 && opengl3.process_event(event) && imgui.menu.read().is_enabled {
                if let Some(event) = event.as_mut() {
                    let event_type =
                        std::mem::transmute::<u32, sdl2_sys::SDL_EventType>(event.type_);
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
    }

    Ok(result)
}
