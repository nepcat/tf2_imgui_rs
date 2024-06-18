#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid DX9 device")]
    InvalidDevice,

    #[error("Failed to impl for Win32")]
    ImplWin32,

    #[error("Failed to impl for DX9")]
    ImplDX9,

    #[error("Failed to get colors")]
    Colors(windows::core::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct DirectX9 {
    pub device: *mut windows::Win32::Graphics::Direct3D9::IDirect3DDevice9,

    window: windows::Win32::Foundation::HWND,
}

impl DirectX9 {
    pub unsafe fn init(
        window: windows::Win32::Foundation::HWND,
        device: *mut windows::Win32::Graphics::Direct3D9::IDirect3DDevice9,
    ) -> Result<Self> {
        /* Check if device is valid */
        if device.is_null() {
            return Err(Error::InvalidDevice);
        }

        /* Implement ImGui for Win32 */
        if !imgui_rs::ImGui_ImplWin32_Init(window.0 as _) {
            return Err(Error::ImplWin32);
        }
        /* Destructor to shutdown our impl Win32 in case we fail */
        let our_impl_win32_lock_guard = scopeguard::guard((), |_| {
            imgui_rs::ImGui_ImplWin32_Shutdown();
        });

        /* Implement ImGui for DX9 */
        if !imgui_rs::ImGui_ImplDX9_Init(device as _) {
            return Err(Error::ImplDX9);
        }
        /* Destructor to shutdown our impl DX9 in case we fail */
        let our_impl_dx9_lock_guard = scopeguard::guard((), |_| {
            imgui_rs::ImGui_ImplDX9_Shutdown();
        });

        scopeguard::ScopeGuard::into_inner(our_impl_dx9_lock_guard);
        scopeguard::ScopeGuard::into_inner(our_impl_win32_lock_guard);
        Ok(Self { window, device })
    }

    pub unsafe fn new_frame(&self) {
        imgui_rs::ImGui_ImplWin32_NewFrame();
        imgui_rs::ImGui_ImplDX9_NewFrame();
    }

    pub unsafe fn end_frame(&self, draw_data: *mut imgui_rs::ImDrawData) {
        imgui_rs::ImGui_ImplDX9_RenderDrawData(draw_data);
    }

    pub unsafe fn process_event(
        &self,
        msg: std::os::raw::c_uint,
        w_param: windows::Win32::Foundation::WPARAM,
        l_param: windows::Win32::Foundation::LPARAM,
    ) -> windows::Win32::Foundation::LRESULT {
        windows::Win32::Foundation::LRESULT(imgui_rs::ImGui_ImplWin32_WndProcHandler(
            self.window.0 as _,
            msg,
            w_param.0 as _,
            l_param.0 as _,
        ) as _)
    }

    pub unsafe fn update_device_objects(&self, create: bool) {
        if create {
            imgui_rs::ImGui_ImplDX9_CreateDeviceObjects();
        } else {
            imgui_rs::ImGui_ImplDX9_InvalidateDeviceObjects();
        }
    }

    /*pub unsafe fn replace_old_colors(&self) -> Result<Colors> {
        let device = self.device.as_mut().unwrap_unchecked();
        let mut colors = Colors::get(device)?;
        colors.replace();

        Ok(colors)
    }

    /* Not really necessary because they will be restored on drop() */
    pub unsafe fn restore_old_colors(&self, mut colors: Colors) {
        colors.restore();
    }*/
}

impl Drop for DirectX9 {
    fn drop(&mut self) {
        unsafe {
            if let Some(io) = imgui_rs::ImGui::GetIO().as_ref() {
                if !io.BackendRendererUserData.is_null() {
                    imgui_rs::ImGui_ImplDX9_Shutdown();
                }
                if !io.BackendPlatformUserData.is_null() {
                    imgui_rs::ImGui_ImplWin32_Shutdown();
                }
            }
        }
    }
}

/* I couldn't get it working while compiling cheat for windows-gnu target
 * and I don't want to waste time installing dependencies on a Windows machine with MSVC target
 * so I'll just comment out this part of the code for now. */
/*#[derive(Debug)]
pub struct Colors<'a> {
    pub color_write: u32,
    pub srgb_write: u32,

    /*pub vertex_declaration: windows::Win32::Graphics::Direct3D9::IDirect3DVertexDeclaration9,
    pub vertex_shader: windows::Win32::Graphics::Direct3D9::IDirect3DVertexShader9,*/

    /* Stored for drop */
    should_restore: bool,
    device: &'a mut windows::Win32::Graphics::Direct3D9::IDirect3DDevice9,
}

impl<'a> Colors<'a> {
    pub unsafe fn get(
        device: &'a mut windows::Win32::Graphics::Direct3D9::IDirect3DDevice9,
    ) -> Result<Self> {
        let mut color_write = 0;
        if let Err(error) = device.GetRenderState(
            windows::Win32::Graphics::Direct3D9::D3DRS_COLORWRITEENABLE,
            &mut color_write,
        ) {
            return Err(Error::Colors(error));
        }

        let mut srgb_write = 0;
        if let Err(error) = device.GetRenderState(
            windows::Win32::Graphics::Direct3D9::D3DRS_SRGBWRITEENABLE,
            &mut srgb_write,
        ) {
            return Err(Error::Colors(error));
        }

        /*let vertex_declaration = match device.GetVertexDeclaration() {
            Ok(vertex_declaration) => vertex_declaration,
            Err(error) => return Err(Error::Colors(error)),
        };

        let vertex_shader = match device.GetVertexShader() {
            Ok(vertex_shader) => vertex_shader,
            Err(error) => return Err(Error::Colors(error)),
        };*/

        Ok(Self {
            color_write,
            srgb_write,

            /*vertex_declaration,
            vertex_shader,*/
            should_restore: false,
            device,
        })
    }

    /* TODO: Error handling? */
    pub unsafe fn replace(&mut self) {
        /* color_write */
        let _ = self.device.SetRenderState(
            windows::Win32::Graphics::Direct3D9::D3DRS_COLORWRITEENABLE,
            0xffffffff,
        );
        /* srgb_write */
        let _ = self.device.SetRenderState(
            windows::Win32::Graphics::Direct3D9::D3DRS_SRGBWRITEENABLE,
            0x0,
        );

        self.should_restore = true;
    }

    /* TODO: Error handling? */
    pub unsafe fn restore(&mut self) {
        if self.should_restore {
            let _ = self.device.SetRenderState(
                windows::Win32::Graphics::Direct3D9::D3DRS_COLORWRITEENABLE,
                self.color_write,
            );
            let _ = self.device.SetRenderState(
                windows::Win32::Graphics::Direct3D9::D3DRS_SRGBWRITEENABLE,
                self.srgb_write,
            );

            /*let _ = self.device.SetVertexDeclaration(&self.vertex_declaration);
            let _ = self.device.SetVertexShader(&self.vertex_shader);*/

            self.should_restore = false;
        }
    }
}

impl<'a> Drop for Colors<'a> {
    fn drop(&mut self) {
        unsafe { self.restore() };
    }
}*/
