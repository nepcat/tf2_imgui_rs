#[derive(Debug)]
pub struct Linux {
    pub library_path: Vec<std::os::raw::c_char>,
}

impl Linux {
    pub unsafe fn new(entry_point: usize) -> Option<Self> {
        let library_path =
            crate::utils::linux::get_library_path(entry_point as *const std::os::raw::c_void)?;
        let c_str = std::ffi::CStr::from_ptr(library_path);
        let vec = c_str
            .to_bytes_with_nul()
            .to_vec()
            .iter()
            .map(|byte| *byte as std::os::raw::c_char)
            .collect::<Vec<_>>();
        Some(Self { library_path: vec })
    }
}

impl Drop for Linux {
    fn drop(&mut self) {
        unsafe {
            log::debug!(
                "Unloading as library {:?}",
                std::ffi::CStr::from_ptr(self.library_path.as_ptr()).to_str()
            );
            let this = libc::dlopen(
                self.library_path.as_ptr(),
                libc::RTLD_LAZY | libc::RTLD_NOLOAD,
            );
            match this.is_null() {
                true => log::error!("dlopen returned null"),
                false => {
                    let mut i = 0;
                    while libc::dlclose(this) == 0 {
                        log::debug!("{i} dlclose()");
                        i += 1;
                    }
                }
            }
        }
    }
}
