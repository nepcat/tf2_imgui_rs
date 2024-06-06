#![allow(clippy::missing_safety_doc)]

#[cfg(target_os = "windows")]
pub mod d3d9;
#[cfg(target_os = "linux")]
pub mod sdl2;
#[cfg(target_os = "windows")]
pub mod shader_api_dx9;
#[cfg(target_os = "windows")]
pub mod tf2;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[cfg(target_os = "linux")]
    #[error("Failed to read proc maps")]
    ProcMaps,

    #[cfg(target_os = "linux")]
    #[error("SDL2 error")]
    SDL2(#[from] sdl2::Error),

    #[cfg(target_os = "windows")]
    #[error("D3D9 error")]
    D3D9(#[from] d3d9::Error),

    #[cfg(target_os = "windows")]
    #[error("ShaderApiDX9 error")]
    ShaderApiDX9(#[from] shader_api_dx9::Error),

    #[cfg(target_os = "windows")]
    #[error("TF2 error")]
    TF2(#[from] tf2::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

pub struct Modules {
    #[cfg(target_os = "linux")]
    pub sdl2: sdl2::SDL2,
    #[cfg(target_os = "windows")]
    pub d3d9: d3d9::D3D9,
    #[cfg(target_os = "windows")]
    pub shader_api_dx9: shader_api_dx9::ShaderApiDX9,
    #[cfg(target_os = "windows")]
    pub tf2: tf2::TF2,
}

impl Modules {
    pub unsafe fn get() -> Result<Self> {
        #[cfg(target_os = "linux")]
        let mut maps = {
            let fuckers = std::fs::read_to_string("/proc/self/maps");

            crate::utils::module::maps::parse_proc_maps(fuckers.map_err(|_| Error::ProcMaps)?)
        };

        #[cfg(target_os = "windows")]
        let tf2 = tf2::TF2::new()?;

        #[cfg(target_os = "linux")]
        let sdl2 = {
            #[cfg(target_os = "linux")]
            let (filepath, maps) = {
                crate::utils::module::maps::find_closest_map(&mut maps, sdl2::FILEPATH)
                    .ok_or(sdl2::Error::Maps)?
            };
            #[cfg(target_os = "windows")]
            let filepath = sdl2::FILEPATH;

            let library = crate::utils::module::Module::new(filepath)
                .map_err(|_| sdl2::Error::Library)?
                .ok_or(sdl2::Error::Library)?;

            #[cfg(target_os = "windows")]
            let maps = vec![crate::utils::module::maps::Map::get_from_module(&library)
                .map_err(|_| sdl2::Error::Maps)?];

            let sdl2 = sdl2::SDL2::new(library, maps)?;
            log::debug!("Got SDL2 library");
            sdl2
        };

        #[cfg(target_os = "windows")]
        let shader_api_dx9 = {
            #[cfg(target_os = "windows")]
            let filepath = shader_api_dx9::FILEPATH;

            let library = crate::utils::module::Module::new(filepath)
                .map_err(|_| shader_api_dx9::Error::Library)?
                .ok_or(shader_api_dx9::Error::Library)?;

            #[cfg(target_os = "windows")]
            let maps = vec![crate::utils::module::maps::Map::get_from_module(&library)
                .map_err(|_| shader_api_dx9::Error::Maps)?];

            let shader_api_dx9 = shader_api_dx9::ShaderApiDX9::new(library, maps)?;
            log::debug!("Got ShaderApiDX9 library");
            shader_api_dx9
        };

        #[cfg(target_os = "windows")]
        let d3d9 = {
            let d3d9 = d3d9::D3D9::new(&shader_api_dx9)?;
            log::debug!("Got D3D9 library");
            d3d9
        };

        Ok(Self {
            #[cfg(target_os = "linux")]
            sdl2,
            #[cfg(target_os = "windows")]
            d3d9,
            #[cfg(target_os = "windows")]
            shader_api_dx9,
            #[cfg(target_os = "windows")]
            tf2,
        })
    }

    pub unsafe fn hooks_init(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        self.sdl2.hooks_init();
        #[cfg(target_os = "windows")]
        {
            self.d3d9.hooks_init()?;
            self.tf2.hooks_init()?;
        }

        Ok(())
    }

    pub unsafe fn hooks_restore(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        self.sdl2.hooks_restore();
        #[cfg(target_os = "windows")]
        {
            self.d3d9.hooks_restore()?;
            self.tf2.hooks_restore()?;
        }

        Ok(())
    }
}
