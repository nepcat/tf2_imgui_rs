#![allow(irrefutable_let_patterns)]

pub type Function = unsafe extern "system" fn(
    hwnd: windows::Win32::Foundation::HWND,
    msg: std::os::raw::c_uint,
    w_param: windows::Win32::Foundation::WPARAM,
    l_param: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to find original")]
    FindOriginal,

    #[error("Detour error")]
    Hooks(#[from] retour::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct WndProc {
    pub original: Function,
    hwnd: windows::Win32::Foundation::HWND,
}

impl WndProc {
    pub unsafe fn find_original(hwnd: windows::Win32::Foundation::HWND) -> Result<Self> {
        let original = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrA(
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::GWLP_WNDPROC,
        );
        match crate::utils::from_isize::FromIsize::new(original) {
            crate::utils::from_isize::FromIsize::Positive(position) => {
                let original = std::mem::transmute::<usize, Function>(position);

                Ok(Self { original, hwnd })
            }
            crate::utils::from_isize::FromIsize::Negative(_) => Err(Error::FindOriginal),
        }
    }

    pub unsafe fn hook_init(&self) -> Result<()> {
        let old_value = windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrA(
            self.hwnd,
            windows::Win32::UI::WindowsAndMessaging::GWLP_WNDPROC,
            our as _,
        );
        if old_value <= 0 {
            log::debug!("SetWindowLongPtrA() <= 0");
        } else if old_value as usize != self.original as usize {
            log::debug!(
                "SetWindowLongPtrA() old_value {} != self.original {}",
                old_value,
                self.original as usize
            );
        }
        /* TODO: Proper error checking */
        Ok(())
    }

    pub unsafe fn hook_restore(&self) -> Result<()> {
        let old_value = windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrA(
            self.hwnd,
            windows::Win32::UI::WindowsAndMessaging::GWLP_WNDPROC,
            self.original as _,
        );
        if old_value <= 0 {
            log::debug!("SetWindowLongPtrA() <= 0");
        } else if old_value as usize != our as usize {
            log::debug!(
                "SetWindowLongPtrA() old_value {} != our {}",
                old_value,
                our as usize
            );
        }
        /* TODO: Proper error checking */
        Ok(())
    }
}

unsafe extern "system" fn our(
    hwnd: windows::Win32::Foundation::HWND,
    msg: std::os::raw::c_uint,
    w_param: windows::Win32::Foundation::WPARAM,
    l_param: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    // log::debug!("Our hooked function called");
    match try_our(hwnd, msg, w_param, l_param) {
        Ok(result) => result,
        Err(error) => {
            log::error!("Failed to execute our hooked function, reason: {error:?}. Crashing!");
            std::process::exit(1);
        }
    }
}

unsafe fn try_our(
    hwnd: windows::Win32::Foundation::HWND,
    msg: std::os::raw::c_uint,
    w_param: windows::Win32::Foundation::WPARAM,
    l_param: windows::Win32::Foundation::LPARAM,
) -> anyhow::Result<windows::Win32::Foundation::LRESULT> {
    use anyhow::Context;
    let modules = crate::globals::MODULES
        .get()
        .context("Failed to get global modules")?;
    let original = &modules.tf2.hooks.wnd_proc.original;
    let mut run_original = true;

    let imgui = crate::globals::IMGUI.get();
    if let Some(imgui) = imgui
        && let crate::imgui::renderer::Renderer::Win32(
            crate::imgui::renderer::win32::Win32::DirectX9(directx9),
        ) = &imgui.renderer
    {
        directx9.process_event(msg, w_param, l_param);
        if imgui.menu.read().is_enabled {
            /* TODO: Filter events (msgs) */
            run_original = false;
        }
    }

    if run_original {
        Ok(windows::Win32::UI::WindowsAndMessaging::CallWindowProcA(
            Some(*original),
            hwnd,
            msg,
            w_param,
            l_param,
        ))
    } else {
        /* or return 0 */
        Ok(windows::Win32::Foundation::LRESULT(1))
    }
}
