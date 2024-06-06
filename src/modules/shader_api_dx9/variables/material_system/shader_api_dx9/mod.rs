pub mod shader_device_dx8;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("ShaderDeviceDX8 error")]
    ShaderDeviceDX8(#[from] shader_device_dx8::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct ShaderApiDX9 {
    pub shader_device_dx8: shader_device_dx8::ShaderDeviceDX8,
}

impl ShaderApiDX9 {
    pub unsafe fn find<V1: AsRef<[crate::utils::module::maps::Map]>>(maps: V1) -> Result<Self> {
        Ok(Self {
            shader_device_dx8: shader_device_dx8::ShaderDeviceDX8::find(maps)?,
        })
    }
}
