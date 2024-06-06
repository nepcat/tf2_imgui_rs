pub mod shader_api_dx9;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("ShaderApiDX9 error")]
    ShaderApiDX9(#[from] shader_api_dx9::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct MaterialSystem {
    pub shader_api_dx9: shader_api_dx9::ShaderApiDX9,
}

impl MaterialSystem {
    pub unsafe fn find<V1: AsRef<[crate::utils::module::maps::Map]>>(maps: V1) -> Result<Self> {
        Ok(Self {
            shader_api_dx9: shader_api_dx9::ShaderApiDX9::find(maps)?,
        })
    }
}
