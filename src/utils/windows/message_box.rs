#![allow(clippy::missing_safety_doc)]

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Nul Error")]
    Nul(#[from] std::ffi::NulError),
}
pub type Result<T> = std::result::Result<T, Error>;

pub unsafe fn message_box<S1: AsRef<str>, S2: AsRef<str>>(
    hwnd: windows::Win32::Foundation::HWND,
    text: S1,
    title: S2,
    r#type: windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_STYLE,
) -> Result<windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_RESULT> {
    let c_str_text = std::ffi::CString::new(text.as_ref())?;
    let c_str_title = std::ffi::CString::new(title.as_ref())?;
    Ok(windows::Win32::UI::WindowsAndMessaging::MessageBoxA(
        hwnd,
        windows::core::PCSTR::from_raw(c_str_text.as_ptr() as _),
        windows::core::PCSTR::from_raw(c_str_title.as_ptr() as _),
        r#type,
    ))
}
