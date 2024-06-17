#![allow(clippy::missing_safety_doc)]

use imgui_rs::root as imgui_rs;

pub mod renderer;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create ImGui context")]
    Context,

    #[error("Failed to get ImGui IO")]
    IO,

    #[error("Renderer error")]
    Renderer(#[from] renderer::Error),

    #[error("Failed to load font")]
    Font,
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct ImGui {
    context: *mut imgui_rs::ImGuiContext,

    pub menu: parking_lot::RwLock<crate::menu::Menu>,

    pub renderer: renderer::Renderer,
}

unsafe impl Send for ImGui {}
unsafe impl Sync for ImGui {}

impl ImGui {
    pub unsafe fn init(init_renderer: renderer::InitRenderer) -> Result<Self> {
        /* Create ImGui context */
        let imgui_context = {
            let imgui_context = imgui_rs::ImGui::CreateContext(core::ptr::null_mut());
            if imgui_context.is_null() {
                return Err(Error::Context);
            }
            /* Destructor to free imgui context in case we fail */
            scopeguard::guard(imgui_context, |imgui_context| {
                imgui_rs::ImGui::DestroyContext(imgui_context);
            })
        };

        /* Get ImGui IO */
        let Some(imgui_io) = imgui_rs::ImGui::GetIO().as_mut() else {
            return Err(Error::IO);
        };
        /* Disable ini file */
        // imgui_io.IniFilename = core::ptr::null();
        #[cfg(any(debug_assertions, feature = "developer"))]
        {
            imgui_io.LogFilename = c_str_macro::c_str!("imgui.log").as_ptr();
        }
        #[cfg(not(any(debug_assertions, feature = "developer")))]
        {
            imgui_io.LogFilename = core::ptr::null();
        }

        /* Initialize our renderer */
        let renderer = renderer::Renderer::init(init_renderer)?;

        log::debug!("ImGui for {renderer:#?} initialized succesfully!");
        Ok(Self {
            context: scopeguard::ScopeGuard::into_inner(imgui_context),
            menu: Default::default(),
            renderer,
        })
    }

    pub unsafe fn set_current_context(&self) {
        imgui_rs::ImGui::SetCurrentContext(self.context);
    }

    pub unsafe fn render(&self) {
        /* Fix colors
         * On linux its done by using our own GL context for imgui
         * On windows its done by modifing D3D9 values,
         * see imgui/renderer/win32/directx9.rs's Colors struct */
        match &self.renderer {
            #[cfg(target_os = "linux")]
            renderer::Renderer::SDL2(sdl2) => match sdl2 {
                renderer::sdl2::SDL2::OpenGL3(opengl3) => {
                    opengl3.set_context(true);
                }
            },
            _ => {}
        };
        /* Set ImGui context so game doesn't fuck up when switching render threads! */
        self.set_current_context();

        /* Prepare new renderer frame */
        match &self.renderer {
            #[cfg(target_os = "linux")]
            renderer::Renderer::SDL2(sdl2) => match sdl2 {
                renderer::sdl2::SDL2::OpenGL3(opengl3) => opengl3.new_frame(),
                /*#[cfg(target_os = "windows")]
                renderer::sdl2::SDL2::DirectX9(directx9) => directx9.new_frame(),*/
            },
            #[cfg(target_os = "windows")]
            renderer::Renderer::Win32(win32) => match win32 {
                renderer::win32::Win32::DirectX9(directx9) => directx9.new_frame(),
            },
        };
        /* New ImGui frame */
        imgui_rs::ImGui::NewFrame();

        /* Update input */
        self.update_input();

        /* Draw your menu */
        self.menu.read().render();

        /* ImGui end frame */
        imgui_rs::ImGui::EndFrame();
        imgui_rs::ImGui::Render();
        let imgui_draw_data = imgui_rs::ImGui::GetDrawData();
        /* Renderer end frame */
        match &self.renderer {
            #[cfg(target_os = "linux")]
            renderer::Renderer::SDL2(sdl2) => match sdl2 {
                renderer::sdl2::SDL2::OpenGL3(opengl3) => opengl3.end_frame(imgui_draw_data),
                /*#[cfg(target_os = "windows")]
                renderer::sdl2::SDL2::DirectX9(directx9) => directx9.end_frame(imgui_draw_data),*/
            },
            #[cfg(target_os = "windows")]
            renderer::Renderer::Win32(win32) => match win32 {
                renderer::win32::Win32::DirectX9(directx9) => directx9.end_frame(imgui_draw_data),
            },
        };

        /* Restore old colors */
        match &self.renderer {
            #[cfg(target_os = "linux")]
            renderer::Renderer::SDL2(sdl2) => match sdl2 {
                renderer::sdl2::SDL2::OpenGL3(opengl3) => {
                    opengl3.set_context(false);
                }
            },
            _ => {}
        };
    }

    unsafe fn update_input(&self) {
        /* Update menu input */
        self.menu.write().update_input();
        /* Unloading */
        {
            let mut unloading_lock = crate::globals::UNLOADING.lock();
            if imgui_rs::ImGui::IsKeyPressed(imgui_rs::ImGuiKey::ImGuiKey_Home, false)
                && !(*unloading_lock)
            {
                *unloading_lock = true;
                std::thread::spawn(|| match crate::try_destroy() {
                    Ok(_) => {
                        log::info!("Unloaded succesfully!");
                    }
                    Err(error) => {
                        log::error!("Failed to unload, reason: {:?}", error)
                    }
                });
            }
        }
    }
}

impl Drop for ImGui {
    fn drop(&mut self) {
        unsafe {
            imgui_rs::ImGui::DestroyContext(self.context);
        }
    }
}
