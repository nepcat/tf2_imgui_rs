#![allow(clippy::missing_safety_doc)]

pub mod maps;

#[cfg(target_os = "linux")]
type Handle = *mut libc::c_void;
#[cfg(target_os = "windows")]
type Handle = windows::Win32::Foundation::HMODULE;

#[derive(Debug, Clone)]
pub struct Module {
    handle: Handle,
}

unsafe impl Send for Module {}
unsafe impl Sync for Module {}

impl Module {
    pub unsafe fn new<S1: AsRef<str>>(name: S1) -> Result<Option<Self>, std::ffi::NulError> {
        let c_str = std::ffi::CString::new(name.as_ref())?;
        let name = c_str.as_ptr() as _;

        #[cfg(target_os = "linux")]
        let handle = {
            let handle = libc::dlopen(name, libc::RTLD_LAZY | libc::RTLD_NOLOAD);
            if handle.is_null() {
                return Ok(None);
            }
            handle
        };

        #[cfg(target_os = "windows")]
        let handle = {
            let result =
                windows::Win32::System::LibraryLoader::GetModuleHandleA(windows::core::PCSTR(name));
            match result {
                Ok(handle) => handle,
                Err(_) => return Ok(None),
            }
        };

        Ok(Some(Self { handle }))
    }

    pub fn from_handle(handle: Handle) -> Self {
        Self { handle }
    }

    pub fn get_handle(&self) -> &Handle {
        &self.handle
    }

    pub unsafe fn get_symbol<S1: AsRef<str>>(
        &self,
        symbol: S1,
    ) -> Result<Option<usize>, std::ffi::NulError> {
        let c_str = std::ffi::CString::new(symbol.as_ref())?;
        let name = c_str.as_ptr() as *const _;

        #[cfg(target_os = "linux")]
        return Ok({
            let result = libc::dlsym(self.handle, name);

            match result.is_null() {
                false => Some(result as usize),
                true => None,
            }
        });

        #[cfg(target_os = "windows")]
        return Ok({
            let result = windows::Win32::System::LibraryLoader::GetProcAddress(
                self.handle,
                windows::core::PCSTR(name),
            );
            result.map(|some| some as usize)
        });
    }
    // TODO: Function to get symbol using std::ffi::CStr
}

#[cfg(target_os = "linux")]
impl Drop for Module {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                libc::dlclose(self.handle);
            }
        }
    }
}
