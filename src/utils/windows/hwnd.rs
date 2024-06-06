#![allow(clippy::missing_safety_doc)]

/* TODO: Iterator maybe? */
pub unsafe fn get_all_windows_by_pid(pid: u32) -> Vec<windows::Win32::Foundation::HWND> {
    let mut result = Vec::new();

    let mut hwnd = windows::Win32::UI::WindowsAndMessaging::GetTopWindow(None);
    while hwnd != windows::Win32::Foundation::HWND::default() {
        let mut dw_pid = 0;
        windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(hwnd, Some(&mut dw_pid));

        if dw_pid == pid {
            result.push(hwnd);
        }

        hwnd = windows::Win32::UI::WindowsAndMessaging::GetWindow(
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::GW_HWNDNEXT,
        );
    }

    result
}

/* TODO: Iterator maybe? */
pub unsafe fn get_current_process_volvo_windows() -> Vec<windows::Win32::Foundation::HWND> {
    let windows = get_all_windows_by_pid(std::process::id());
    let mut result = Vec::new();

    for window in windows {
        let mut class_name = [0u8; 256];
        if windows::Win32::UI::WindowsAndMessaging::GetClassNameA(window, &mut class_name) == 0 {
            continue;
        }
        let class_name = std::ffi::CStr::from_bytes_with_nul_unchecked(&class_name);
        let Ok(utf8_class_name) = class_name.to_str() else {
            continue;
        };

        if utf8_class_name.starts_with("Valve001") {
            result.push(window);
        }
    }

    result
}
