/* Note about SDL2+DirectX9 renderer:
 * It works, but TF2 doesn't use SDL for window creation on Windows
 * Because of this, we will not be able to get SDL_Window on SDL_PollEvent() hook
 * to initialize this renderer  */

use imgui_rs::root as imgui_rs;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid DX9 device")]
    InvalidDevice,

    #[error("Failed to impl for SDL2 D3D")]
    ImplSDL2D3D,

    #[error("Failed to impl for DX9")]
    ImplDX9,
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct DirectX9 {
    device: *mut windows::Win32::Graphics::Direct3D9::IDirect3DDevice9,

    window: *mut sdl2_sys::SDL_Window,
}

impl DirectX9 {
    pub unsafe fn init(
        window: *mut sdl2_sys::SDL_Window,
        device: *mut windows::Win32::Graphics::Direct3D9::IDirect3DDevice9,
    ) -> Result<Self> {
        /* Implement ImGui for SDL2 GL */
        if !imgui_rs::ImGui_ImplSDL2_InitForD3D(window as _) {
            return Err(Error::ImplSDL2D3D);
        }
        /* Destructor to shutdown our impl SDL2 GL in case we fail */
        let our_impl_sdl2_lock_guard = scopeguard::guard((), |_| {
            imgui_rs::ImGui_ImplSDL2_Shutdown();
        });

        /* Implement ImGui for DX9 */
        if !imgui_rs::ImGui_ImplDX9_Init(device as _) {
            return Err(Error::ImplDX9);
        }
        /* Destructor to shutdown our impl DX9 in case we fail */
        let our_impl_dx9_lock_guard = scopeguard::guard((), |_| {
            imgui_rs::ImGui_ImplDX9_Shutdown();
        });

        std::mem::forget(our_impl_dx9_lock_guard);
        std::mem::forget(our_impl_sdl2_lock_guard);
        Ok(Self { window, device })
    }

    pub unsafe fn new_frame(&self) {
        imgui_rs::ImGui_ImplSDL2_NewFrame();
        imgui_rs::ImGui_ImplDX9_NewFrame();
    }

    pub unsafe fn end_frame(&self, draw_data: *mut imgui_rs::ImDrawData) {
        imgui_rs::ImGui_ImplDX9_RenderDrawData(draw_data);
    }

    pub unsafe fn process_event(&self, event: *mut sdl2_sys::SDL_Event) -> bool {
        imgui_rs::ImGui_ImplSDL2_ProcessEvent(event as _)
    }

    pub unsafe fn update_device_objects(&self, create: bool) {
        if create {
            imgui_rs::ImGui_ImplDX9_CreateDeviceObjects();
        } else {
            imgui_rs::ImGui_ImplDX9_InvalidateDeviceObjects();
        }
    }
}

impl Drop for DirectX9 {
    fn drop(&mut self) {
        unsafe {
            if let Some(io) = imgui_rs::ImGui::GetIO().as_ref() {
                if !io.BackendRendererUserData.is_null() {
                    imgui_rs::ImGui_ImplDX9_Shutdown();
                }
                if !io.BackendPlatformUserData.is_null() {
                    imgui_rs::ImGui_ImplSDL2_Shutdown();
                }
            }
        }
    }
}
