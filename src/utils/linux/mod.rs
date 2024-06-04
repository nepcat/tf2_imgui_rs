#![allow(clippy::missing_safety_doc)]

pub unsafe fn get_library_path(entry_point: *const libc::c_void) -> Option<*const libc::c_char> {
    let mut dl_info: libc::Dl_info = std::mem::zeroed();
    match libc::dladdr(entry_point, &mut dl_info) {
        0 => None,
        _ => match dl_info.dli_fname.is_null() {
            true => None,
            false => Some(dl_info.dli_fname),
        },
    }
}
