#![allow(clippy::missing_safety_doc)]

pub mod hooks;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub static FILEPATH: &str = "libSDL2-2.0.so";
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub static FILEPATH: &str = ".\\bin\\x64\\SDL2.dll";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to get library")]
    Library,

    #[error("Failed to get maps")]
    Maps,

    #[error("Hooks error")]
    Hooks(#[from] hooks::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

pub struct SDL2 {
    library: crate::utils::module::Module,
    maps: Vec<crate::utils::module::maps::Map>,

    pub hooks: hooks::Hooks,
}

impl SDL2 {
    pub unsafe fn new(
        library: crate::utils::module::Module,
        maps: Vec<crate::utils::module::maps::Map>,
    ) -> Result<Self> {
        let hooks = hooks::Hooks::find_original(&library)?;

        Ok(Self {
            library,
            maps,
            hooks,
        })
    }

    pub unsafe fn hooks_init(&self) {
        self.hooks.init();
    }

    pub unsafe fn hooks_restore(&self) {
        self.hooks.restore();
    }
}
