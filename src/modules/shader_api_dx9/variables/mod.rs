pub mod material_system;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("MaterialSystem error")]
    MaterialSystem(#[from] material_system::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Variables {
    pub material_system: material_system::MaterialSystem,
}

impl Variables {
    pub unsafe fn find<V1: AsRef<[crate::utils::module::maps::Map]>>(maps: V1) -> Result<Self> {
        Ok(Self {
            material_system: material_system::MaterialSystem::find(maps)?,
        })
    }
}
