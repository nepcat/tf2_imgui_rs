pub mod gp_d3d_device;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("g_pD3DDevice error")]
    GPD3DDevice(#[from] gp_d3d_device::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct ShaderDeviceDX8 {
    pub gp_d3d_device: gp_d3d_device::Type,
}

impl ShaderDeviceDX8 {
    pub unsafe fn find<V1: AsRef<[crate::utils::module::maps::Map]>>(maps: V1) -> Result<Self> {
        Ok(Self {
            gp_d3d_device: gp_d3d_device::find(maps)?,
        })
    }
}
