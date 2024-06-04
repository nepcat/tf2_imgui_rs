#![allow(clippy::missing_safety_doc)]

pub mod sdl2;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[cfg(target_os = "linux")]
    #[error("Failed to read proc maps")]
    ProcMaps,

    #[error("SDL2 error")]
    SDL2(#[from] sdl2::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

pub struct Modules {
    pub sdl2: sdl2::SDL2,
}

impl Modules {
    pub unsafe fn get() -> Result<Self> {
        #[cfg(target_os = "linux")]
        let mut maps = {
            let fuckers = std::fs::read_to_string("/proc/self/maps");

            crate::utils::module::maps::parse_proc_maps(fuckers.map_err(|_| Error::ProcMaps)?)
        };

        let sdl2 = {
            #[cfg(target_os = "linux")]
            let (filepath, _maps) = {
                crate::utils::module::maps::find_closest_map(&mut maps, sdl2::FILEPATH)
                    .ok_or(sdl2::Error::Maps)?
            };
            #[cfg(target_os = "windows")]
            let filepath = sdl2::FILEPATH;

            let library = crate::utils::module::Module::new(filepath)
                .map_err(|_| sdl2::Error::Library)?
                .ok_or(sdl2::Error::Library)?;

            #[cfg(target_os = "windows")]
            let _maps = vec![crate::utils::module::maps::Map::get_from_module(&library)
                .map_err(|_| sdl2::Error::Maps)?];

            sdl2::SDL2::new(library /*maps*/)?
        };
        log::debug!("Got SDL2 library");

        Ok(Self { sdl2 })
    }

    pub unsafe fn hooks_init(&self) -> Result<()> {
        self.sdl2.hooks_init();

        Ok(())
    }

    pub unsafe fn hooks_restore(&self) -> Result<()> {
        self.sdl2.hooks_restore();

        Ok(())
    }
}
