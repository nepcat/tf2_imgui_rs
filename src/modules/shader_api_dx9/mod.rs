pub mod variables;

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub static FILEPATH: &str = ".\\bin\\x64\\shaderapidx9.dll";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to get library")]
    Library,

    #[error("Failed to get maps")]
    Maps,

    #[error("Variables error")]
    Variables(#[from] variables::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

pub struct ShaderApiDX9 {
    library: crate::utils::module::Module,
    maps: Vec<crate::utils::module::maps::Map>,
    pub variables: variables::Variables,
}

unsafe impl Send for ShaderApiDX9 {}
unsafe impl Sync for ShaderApiDX9 {}

impl ShaderApiDX9 {
    pub unsafe fn new(
        library: crate::utils::module::Module,
        maps: Vec<crate::utils::module::maps::Map>,
    ) -> Result<Self> {
        let variables = variables::Variables::find(&maps)?;

        Ok(Self {
            library,
            maps,
            variables,
        })
    }
}
