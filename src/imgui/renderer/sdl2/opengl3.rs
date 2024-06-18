#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create context")]
    CreateContext,

    #[error("Invalid original context")]
    InvalidOriginalContext,

    #[error("Failed to impl for SDL2 OpenGL")]
    ImplSDL2OpenGL,

    #[error("Failed to impl for OpenGL3")]
    ImplOpenGL3,
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct OpenGL3 {
    original_context: sdl2_sys::SDL_GLContext,
    our_context: sdl2_sys::SDL_GLContext,

    window: *mut sdl2_sys::SDL_Window,
}

impl OpenGL3 {
    pub unsafe fn init(
        original_context: sdl2_sys::SDL_GLContext,
        window: *mut sdl2_sys::SDL_Window,
    ) -> Result<Self> {
        /* Check if original context is good */
        if original_context.is_null() {
            return Err(Error::InvalidOriginalContext);
        }

        /* Create our own SDL OpenGL context */
        let our_context = {
            let our_context = sdl2_sys::SDL_GL_CreateContext(window);
            if our_context.is_null() {
                return Err(Error::CreateContext);
            }
            /* Destructor to free our SDL OpenGL context in case we fail */
            scopeguard::guard(our_context, |our_context| {
                sdl2_sys::SDL_GL_DeleteContext(our_context);
            })
        };

        /* Implement ImGui for SDL2 GL */
        if !imgui_rs::ImGui_ImplSDL2_InitForOpenGL(window as _, (*our_context) as _) {
            return Err(Error::ImplSDL2OpenGL);
        }
        /* Destructor to shutdown our impl SDL2 GL in case we fail */
        let our_impl_sdl2_lock_guard = scopeguard::guard((), |_| {
            imgui_rs::ImGui_ImplSDL2_Shutdown();
        });

        /* Implement ImGui for OpenGL3 */
        if !imgui_rs::ImGui_ImplOpenGL3_Init(core::ptr::null_mut()) {
            return Err(Error::ImplSDL2OpenGL);
        }
        /* Destructor to shutdown our impl OpenGL3 in case we fail */
        let our_impl_opengl3_lock_guard = scopeguard::guard((), |_| {
            imgui_rs::ImGui_ImplOpenGL3_Shutdown();
        });

        scopeguard::ScopeGuard::into_inner(our_impl_opengl3_lock_guard);
        scopeguard::ScopeGuard::into_inner(our_impl_sdl2_lock_guard);
        Ok(Self {
            original_context,
            our_context: scopeguard::ScopeGuard::into_inner(our_context),
            window,
        })
    }

    pub unsafe fn set_context(&self, our: bool) -> bool {
        let requested_context = match our {
            true => self.our_context,
            false => self.original_context,
        };
        sdl2_sys::SDL_GL_MakeCurrent(self.window, requested_context) == 0
    }

    pub unsafe fn new_frame(&self) {
        imgui_rs::ImGui_ImplSDL2_NewFrame();
        imgui_rs::ImGui_ImplOpenGL3_NewFrame();
    }

    pub unsafe fn end_frame(&self, draw_data: *mut imgui_rs::ImDrawData) {
        imgui_rs::ImGui_ImplOpenGL3_RenderDrawData(draw_data);
    }

    pub unsafe fn process_event(&self, event: *mut sdl2_sys::SDL_Event) -> bool {
        imgui_rs::ImGui_ImplSDL2_ProcessEvent(event as _)
    }
}

impl Drop for OpenGL3 {
    fn drop(&mut self) {
        unsafe {
            if let Some(io) = imgui_rs::ImGui::GetIO().as_ref() {
                if !io.BackendRendererUserData.is_null() {
                    imgui_rs::ImGui_ImplOpenGL3_Shutdown();
                }
                if !io.BackendPlatformUserData.is_null() {
                    imgui_rs::ImGui_ImplSDL2_Shutdown();
                }
            }
            // just in case switch to original context
            self.set_context(false);
            sdl2_sys::SDL_GL_DeleteContext(self.our_context);
        }
    }
}
